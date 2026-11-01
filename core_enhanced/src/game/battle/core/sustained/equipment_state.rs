use super::*;
use crate::game::{
    ability::equipment_state::EquipmentStateView,
    battle::{
        damage::DamageSourceSnapshot, runtime_control, runtime_facts::EquipmentSourceSnapshot,
    },
};

#[derive(Debug, Clone)]
pub(super) struct EquipmentRecord {
    pub(super) owner: UnitInstanceId,
    pub(super) equipment: EquipmentSourceSnapshot,
    pub(super) source: RuntimeFactSource,
    pub(super) state: state_program::StateRecord,
    pub(super) next_tick: Option<u64>,
    pub(super) pending_resonance: Vec<EquipmentResonanceReward>,
}
#[derive(Debug, Clone)]
pub(super) struct EquipmentResonanceReward {
    amount: i32,
    source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(EquipmentResonanceReward { amount, source });

pub(in crate::game::battle::core) struct EquipmentKillCapture {
    activations: Vec<u64>,
    victim: UnitInstanceId,
    had_owner_dot: bool,
    direct: Option<crate::game::battle::damage::DamageSource>,
}
crate::authoring::clone_shape::struct_shape!(EquipmentRecord {
    owner,
    equipment,
    source,
    state,
    next_tick,
    pending_resonance
});
impl EquipmentRecord {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut rewards = self
            .pending_resonance
            .capacity()
            .checked_mul(std::mem::size_of::<EquipmentResonanceReward>())?;
        for reward in &self.pending_resonance {
            rewards = rewards.checked_add(reward.source.retained_extent()?)?;
        }
        std::mem::size_of::<Self>()
            .checked_add(96)?
            .checked_add(self.source.retained_extent()?)?
            .checked_add(rewards)?
            .checked_add(self.state.retained_extent()?)
    }
}
impl BattleCore {
    pub(super) fn equipped_state_owner_valid(&self, record: &EquipmentRecord) -> bool {
        self.units.get(&record.owner).is_some_and(|unit| {
            unit.is_active()
                && unit.is_combatant()
                && unit.stats.current_health > 0
                && !unit.corroded
                && unit.equipment_slots.as_ref().is_none_or(|slots| {
                    slots.slots.iter().flatten().any(|slot| {
                        slot.runtime_item_instance_id == record.equipment.runtime_item_instance_id
                    })
                })
        }) && self
            .items
            .get(&record.equipment.runtime_item_instance_id)
            .is_some_and(|item| {
                item.owner_unit_instance == record.owner
                    && item.source_snapshot() == record.equipment
                    && self
                        .game_data
                        .equipment_state_catalog()
                        .by_equipment
                        .contains_key(&item.base_uuid)
            })
    }
    pub(super) fn reconcile_equipment_states(&mut self, now: u64) -> Result<(), GameError> {
        loop {
            control::charge(
                self.sustained.equipped.len() as u64,
                "equipment_state_reconcile",
            )?;
            let Some(id) = self
                .sustained
                .equipped
                .iter()
                .find(|(_, r)| !self.equipped_state_owner_valid(r))
                .map(|(id, _)| *id)
            else {
                break;
            };
            self.end_equipment_state(id, now, SustainedEndReason::SourceUnavailable)?;
        }
        if self.sustained.closed || !self.game_data.equipment_state_catalog().has_periodic {
            return Ok(());
        }
        let mut cursor = None;
        loop {
            control::charge(self.items.len() as u64, "equipment_periodic_providers")?;
            let candidate = self
                .items
                .iter()
                .filter(|(id, _)| cursor.is_none_or(|c| **id > c))
                .filter(|(id, item)| {
                    self.units.get(&item.owner_unit_instance).is_some_and(|u| {
                        u.is_active()
                            && u.is_combatant()
                            && u.stats.current_health > 0
                            && !u.corroded
                            && u.equipment_slots.as_ref().is_none_or(|slots| {
                                slots
                                    .slots
                                    .iter()
                                    .flatten()
                                    .any(|s| s.runtime_item_instance_id == **id)
                            })
                    }) && self
                        .game_data
                        .equipment_state_catalog()
                        .by_equipment
                        .get(&item.base_uuid)
                        .is_some_and(|p| p.program.periodic_interval_ms.is_some())
                })
                .min_by_key(|(id, _)| **id)
                .map(|(id, item)| (*id, item.owner_unit_instance, item.source_snapshot()));
            let Some((id, owner, equipment)) = candidate else {
                break;
            };
            cursor = Some(id);
            self.ensure_equipment_state(owner, equipment, now)?;
        }
        Ok(())
    }
    pub(super) fn ensure_equipment_state(
        &mut self,
        owner: UnitInstanceId,
        equipment: EquipmentSourceSnapshot,
        now: u64,
    ) -> Result<u64, GameError> {
        control::charge(
            self.sustained.equipped.len() as u64,
            "equipment_state_lookup",
        )?;
        if let Some((id, _)) = self
            .sustained
            .equipped
            .iter()
            .find(|(_, r)| r.owner == owner && r.equipment == equipment)
        {
            return Ok(*id);
        }
        let compiled =
            &self.game_data.equipment_state_catalog().by_equipment[&equipment.definition_uuid];
        let interval = compiled.program.periodic_interval_ms;
        let next_tick = interval
            .map(|interval| {
                now.checked_add(u64::from(interval))
                    .ok_or_else(|| overflow("equipment_periodic_deadline"))
            })
            .transpose()?;
        let record = EquipmentRecord {
            owner,
            equipment,
            next_tick,
            pending_resonance: Vec::new(),
            source: RuntimeFactSource {
                actor: Some(owner),
                equipment: Some(equipment),
                authored: Some(Arc::clone(&compiled.source)),
                ..Default::default()
            },
            state: state_program::StateRecord::new(Arc::clone(&compiled.program))?,
        };
        runtime_control::admit_retained_growth(
            0,
            record.retained_extent(),
            "equipment_state_record",
        )?;
        self.admit_authoring_runtime_growth(1, "equipment_state_entity")?;
        let id = self
            .sustained
            .next
            .checked_add(1)
            .ok_or_else(|| overflow("equipment_state_activation"))?;
        self.sustained.next = id;
        self.sustained.equipped.insert(id, record);
        if let Some(time_ms) = next_tick {
            self.event_queue.push(BattleEvent::EquipmentStateTick {
                time_ms,
                activation: id,
            });
        }
        Ok(id)
    }
    pub(in crate::game::battle::core) fn equipment_state_tick(
        &mut self,
        activation: u64,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_equipment_states(now)?;
        let Some(record) = self
            .sustained
            .equipped
            .get(&activation)
            .filter(|r| r.next_tick == Some(now))
        else {
            return Ok(());
        };
        let interval = record
            .state
            .program
            .periodic_interval_ms
            .expect("periodic record owns its interval");
        let next = now
            .checked_add(u64::from(interval))
            .ok_or_else(|| overflow("equipment_periodic_deadline"))?;
        // Advance before the reaction: nested reconciliation cannot replay this tick.
        self.sustained
            .equipped
            .get_mut(&activation)
            .unwrap()
            .next_tick = Some(next);
        self.run_state_event(activation, state_program::StateEventInput::Periodic, now)?;
        self.reconcile_equipment_states(now)?;
        if self
            .sustained
            .equipped
            .get(&activation)
            .is_some_and(|r| r.next_tick == Some(next))
        {
            self.event_queue.push(BattleEvent::EquipmentStateTick {
                time_ms: next,
                activation,
            });
        }
        Ok(())
    }
    pub(super) fn end_equipment_state(
        &mut self,
        id: u64,
        now: u64,
        reason: SustainedEndReason,
    ) -> Result<(), GameError> {
        let Some(record) = self.sustained.equipped.get(&id) else {
            return Ok(());
        };
        let equipment = record.equipment;
        let owner = record.owner;
        let authored = Arc::clone(
            record
                .source
                .authored
                .as_ref()
                .expect("equipment state authored source"),
        );
        // Independent finite rewards keep this equipment attribution. End them
        // as well when the wearing relationship ceases to exist.
        loop {
            control::charge(
                self.sustained.records.len() as u64,
                "equipment_state_reward_cleanup",
            )?;
            let Some(child) = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| {
                    r.caster == owner
                        && r.target == owner
                        && r.target_state.is_some()
                        && r.source.equipment == Some(equipment)
                        && r.source.authored.as_ref().is_some_and(|a| {
                            a.source_file == authored.source_file
                                && a.field_path
                                    .strip_prefix(&authored.field_path)
                                    .is_some_and(|tail| tail.starts_with('.'))
                        })
                })
                .map(|(id, _)| *id)
            else {
                break;
            };
            self.end_sustained(child, now, reason)?;
        }
        self.sustained.ended = self
            .sustained
            .ended
            .checked_add(1)
            .ok_or_else(|| overflow("equipment_state_end"))?;
        self.event_queue.discard_sustained_expiry(id)?;
        self.sustained.equipped.remove(&id);
        self.sustained.latest_end = Some(SustainedEndReceipt {
            activation: id,
            caster: owner,
            target: owner,
            time_ms: now,
            reason,
        });
        Ok(())
    }
    pub(super) fn equipment_states_after_damage(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        hp_lost: u32,
        direct: bool,
        now: u64,
    ) -> Result<(), GameError> {
        if self
            .game_data
            .equipment_state_catalog()
            .by_equipment
            .is_empty()
        {
            return Ok(());
        }
        self.reconcile_equipment_states(now)?;
        if self.sustained.closed
            || !self
                .units
                .get(&target)
                .is_some_and(|u| u.is_active() && !u.corroded && u.is_combatant())
        {
            return Ok(());
        }
        let action = incoming.fact_source.action.as_ref().map(|a| a.ordinal().0);
        let hostile = self.units[&target].owner != incoming.source_side;
        // Provider order is deterministic and no handler can create equipment.
        // Use a cursor rather than retaining an unbounded collection of copies.
        let mut cursor = None;
        loop {
            control::charge(self.items.len() as u64, "equipment_state_providers")?;
            let candidate = self
                .items
                .iter()
                .filter(|(id, _)| cursor.is_none_or(|c| **id > c))
                .filter(|(id, item)| {
                    item.owner_unit_instance == target
                        && self.units[&target]
                            .equipment_slots
                            .as_ref()
                            .is_none_or(|slots| {
                                slots
                                    .slots
                                    .iter()
                                    .flatten()
                                    .any(|s| s.runtime_item_instance_id == **id)
                            })
                        && self
                            .game_data
                            .equipment_state_catalog()
                            .by_equipment
                            .contains_key(&item.base_uuid)
                })
                .min_by_key(|(id, _)| **id);
            let Some((item_id, item)) = candidate else {
                break;
            };
            cursor = Some(*item_id);
            let equipment = item.source_snapshot();
            let compiled = &self.game_data.equipment_state_catalog().by_equipment[&item.base_uuid];
            control::charge(
                compiled.program.handlers.len() as u64,
                "equipment_state_eligibility",
            )?;
            if !compiled.program.handlers.iter().any(|h| {
                use crate::game::ability::state_program::{StateEvent, StateRelation};
                let (relation, once_per_action) = match h.event {
                    StateEvent::HpLost { relation, once_per_action } => (relation, once_per_action),
                    StateEvent::DirectHpLost { relation, once_per_action } if direct => (relation, once_per_action),
                    _ => return false,
                };
                (!once_per_action || action.is_some()) &&
                    (relation == StateRelation::Any || (relation == StateRelation::Hostile) == hostile)
            }) { continue; }

            let activation = self.ensure_equipment_state(target, equipment, now)?;
            self.run_state_event(
                activation,
                state_program::StateEventInput::HpLost(hostile, action, hp_lost, direct),
                now,
            )?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn capture_equipment_kill(
        &mut self,
        source: &DamageSourceSnapshot,
        victim: UnitInstanceId,
        consumed_owner_dot: bool,
        direct_base_lethal: bool,
        now: u64,
    ) -> Result<Option<EquipmentKillCapture>, GameError> {
        use crate::game::{ability::state_program::StateEvent, battle::damage::DamageSource};
        if !self.game_data.equipment_state_catalog().has_owner_kill
            || self.sustained.closed
            // Passive reflection uses Environment to suppress ordinary hit procs.
            // Its engine-owned reaction provenance still credits the living owner.
            || (source.source == DamageSource::Environment && !source.fact_source.reaction)
            || !self.units.get(&source.source_id).is_some_and(|u| {
                u.is_active() && u.is_combatant() && u.stats.current_health > 0 && !u.corroded
            })
            || !self.units.get(&victim).is_some_and(|u| {
                u.is_active()
                    && u.is_combatant()
                    && u.stats.current_health > 0
                    && u.owner != source.source_side
            })
        {
            return Ok(None);
        }
        let direct = if direct_base_lethal { self.equipment_direct_source(source) } else { None };
        let mut activations = Vec::new();
        let mut cursor = None;
        loop {
            control::charge(self.items.len() as u64, "equipment_kill_providers")?;
            let candidate = self
                .items
                .iter()
                .filter(|(id, item)| {
                    cursor.is_none_or(|c| **id > c)
                        && item.owner_unit_instance == source.source_id
                        && self.units[&source.source_id]
                            .equipment_slots
                            .as_ref()
                            .is_none_or(|slots| {
                                slots
                                    .slots
                                    .iter()
                                    .flatten()
                                    .any(|slot| slot.runtime_item_instance_id == **id)
                            })
                        && self
                            .game_data
                            .equipment_state_catalog()
                            .by_equipment
                            .get(&item.base_uuid)
                            .is_some_and(|c| {
                                c.program
                                    .handlers
                                    .iter()
                                    .any(|h| h.event == StateEvent::OwnerKill || (direct.is_some() && matches!(h.event, StateEvent::OwnerDirectKill { .. })))
                            })
                })
                .min_by_key(|(id, _)| **id)
                .map(|(id, item)| (*id, item.source_snapshot()));
            let Some((id, equipment)) = candidate else {
                break;
            };
            cursor = Some(id);
            runtime_control::admit_retained_growth(
                activations.len() * std::mem::size_of::<u64>(),
                activations
                    .len()
                    .checked_add(1)
                    .and_then(|n| n.checked_mul(std::mem::size_of::<u64>())),
                "equipment_kill_capture",
            )?;
            let activation = self.ensure_equipment_state(source.source_id, equipment, now)?;
            activations.reserve_exact(1);
            activations.push(activation);
        }
        if activations.is_empty() {
            return Ok(None);
        }
        let had_owner_dot =
            consumed_owner_dot || self.target_has_owned_dot(source.source_id, victim, now)?;
        Ok(Some(EquipmentKillCapture {
            activations,
            victim,
            had_owner_dot,
            direct,
        }))
    }

    pub(in crate::game::battle::core) fn dispatch_equipment_kill(
        &mut self,
        capture: EquipmentKillCapture,
        now: u64,
    ) -> Result<(), GameError> {
        if !self
            .units
            .get(&capture.victim)
            .is_some_and(|u| u.lifecycle.is_dead())
        {
            return Ok(());
        }
        for activation in capture.activations {
            control::charge(1, "equipment_kill_dispatch")?;
            if !self
                .sustained
                .equipped
                .get(&activation)
                .is_some_and(|r| self.equipped_state_owner_valid(r))
            {
                continue;
            }
            self.run_state_event(
                activation,
                state_program::StateEventInput::OwnerKill {
                    victim: capture.victim,
                    had_owner_dot: capture.had_owner_dot,
                    direct: capture.direct,
                },
                now,
            )?;
        }
        Ok(())
    }

    pub(super) fn queue_equipment_resonance_reward(
        &mut self,
        owner: UnitInstanceId,
        source: &RuntimeFactSource,
        amount: i32,
        authored: Arc<crate::game::ability::authoring::EffectSource>,
    ) -> Result<(), GameError> {
        if amount == 0 {
            return Ok(());
        }
        let equipment = source.equipment.ok_or(GameError::InvalidAction)?;
        control::charge(
            self.sustained.equipped.len() as u64,
            "equipment_reward_owner",
        )?;
        let activation = self
            .sustained
            .equipped
            .iter()
            .find(|(_, r)| {
                r.owner == owner && r.equipment == equipment && self.equipped_state_owner_valid(r)
            })
            .map(|(id, _)| *id)
            .ok_or(GameError::InvalidAction)?;
        source.admit_owner("equipment_reward_source")?;
        let mut source = source.clone();
        source.authored = Some(authored);
        runtime_control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<EquipmentResonanceReward>())),
            "equipment_reward_storage",
        )?;
        self.admit_authoring_runtime_growth(1, "equipment_reward_entity")?;
        let rewards = &mut self
            .sustained
            .equipped
            .get_mut(&activation)
            .unwrap()
            .pending_resonance;
        rewards.reserve_exact(1);
        rewards.push(EquipmentResonanceReward { amount, source });
        control::poll("equipment_reward_queued")
    }

    /// Called after event deliveries and causal work settle. Never bypass the
    /// global gain lock or schedule another cast inside the lethal damage stack.
    pub(in crate::game::battle::core) fn settle_equipment_resonance_rewards(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_state_catalog().has_owner_kill {
            return Ok(());
        }
        loop {
            control::charge(
                self.sustained.equipped.len() as u64,
                "equipment_reward_settlement",
            )?;
            let candidate = self
                .sustained
                .equipped
                .iter()
                .find(|(_, r)| {
                    !r.pending_resonance.is_empty()
                        && self.equipped_state_owner_valid(r)
                        && !self.resonance_use_has_obligations(r.owner)
                })
                .map(|(id, r)| (*id, r.owner));
            let Some((activation, owner)) = candidate else {
                break;
            };
            let rewards = &mut self
                .sustained
                .equipped
                .get_mut(&activation)
                .unwrap()
                .pending_resonance;
            control::charge(rewards.len() as u64, "equipment_reward_shift")?;
            let reward = rewards.remove(0);
            let prior = std::mem::replace(&mut self.runtime_fact_source, reward.source);
            let authored = self.runtime_fact_source.authored.clone();
            self.process_skill_commands(
                vec![(
                    crate::game::battle::damage::BattleCommand::ModifyResonance {
                        target_id: owner,
                        amount: reward.amount,
                        allow_autocast_when_full: true,
                    },
                    authored,
                )],
                now,
                owner,
                None,
            );
            self.runtime_fact_source = prior;
            runtime_control::poll()?;
            control::poll("equipment_reward_settled")?;
        }
        self.reconcile_ready_wait(now)
    }

    pub(super) fn equipment_state_views(&self) -> Result<Vec<EquipmentStateView>, GameError> {
        control::charge(
            self.sustained.equipped.len() as u64,
            "equipment_state_query",
        )?;
        runtime_control::admit_retained_growth(
            0,
            self.sustained
                .equipped
                .len()
                .checked_mul(std::mem::size_of::<EquipmentStateView>()),
            "equipment_state_query_storage",
        )?;
        self.sustained
            .equipped
            .iter()
            .map(|(id, r)| {
                runtime_control::admit_retained_growth(
                    0,
                    r.retained_extent(),
                    "equipment_state_query_record",
                )?;
                control::charge(
                    r.state.program.fields.len() as u64,
                    "equipment_state_query_fields",
                )?;
                let authored = r
                    .source
                    .authored
                    .as_deref()
                    .expect("equipment state authored source");
                control::charge_serialized(authored, "equipment_state_query_source")?;
                Ok(EquipmentStateView {
                    activation: *id,
                    owner: r.owner,
                    equipment_definition: r.equipment.definition_uuid,
                    runtime_item_instance_id: r.equipment.runtime_item_instance_id,
                    persistent_copy_uuid: r.equipment.persistent_copy_uuid,
                    source: authored.clone(),
                    state: r.state.view(),
                })
            })
            .collect()
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn equipment_direct_source(&self, source: &DamageSourceSnapshot) -> Option<crate::game::battle::damage::DamageSource> {
        use crate::game::battle::damage::DamageSource;
        if source.fact_source.reaction || source.fact_source.guard_release().is_some()
            || source.fact_source.mobile_effect().is_some()
            || source.fact_source.cast.as_ref().or(source.skill_cast_source.as_ref()).is_some_and(|c| c.proc_origin.is_some() || c.caster_instance_id != source.source_id)
        { return None; }
        match source.source {
            DamageSource::BasicAttack if source.fact_source.action.is_some() => Some(DamageSource::BasicAttack),
            DamageSource::Ability if source.fact_source.cast.as_ref().or(source.skill_cast_source.as_ref()).is_some()
                || source.fact_source.action.as_ref().is_some_and(|a| self.runtime_facts.query(a).is_ok_and(|a| a.actor == source.source_id && a.skill_id.is_some())) => Some(DamageSource::Ability),
            _ => None,
        }
    }
    pub(in crate::game::battle::core) fn consume_equipment_cast_damage(&mut self, owner: UnitInstanceId, now: u64)
        -> Result<Option<Arc<crate::game::battle::skill_source::CastDamageSnapshot>>, GameError> {
        use crate::game::{ability::state_program::StateEvent, battle::skill_source::{CastDamageSnapshot, CastDamageContribution}};
        if !self.game_data.equipment_state_catalog().has_owner_cast || self.sustained.closed
            || !self.units.get(&owner).is_some_and(|u| u.is_active() && u.is_combatant() && u.stats.current_health > 0 && !u.corroded)
        { return Ok(None); }
        self.reconcile_equipment_states(now)?;
        let mut contributors = Vec::new(); let mut total=0u32; let mut cursor=None;
        loop {
            control::charge(self.items.len() as u64, "equipment_cast_providers")?;
            let candidate=self.items.iter().filter(|(id,item)| cursor.is_none_or(|c| **id>c) && item.owner_unit_instance==owner
                && self.units[&owner].equipment_slots.as_ref().is_none_or(|slots| slots.slots.iter().flatten().any(|s| s.runtime_item_instance_id==**id))
                && self.game_data.equipment_state_catalog().by_equipment.get(&item.base_uuid).is_some_and(|c| c.program.handlers.iter().any(|h| h.event==StateEvent::OwnerSkillCast)))
                .min_by_key(|(id,_)| **id).map(|(id,item)| (*id,item.source_snapshot()));
            let Some((id,equipment))=candidate else { break; }; cursor=Some(id);
            let activation=self.ensure_equipment_state(owner,equipment,now)?;
            for (percent, authored) in self.run_state_event_output(activation,state_program::StateEventInput::OwnerSkillCast,now)? {
                total=total.checked_add(percent).filter(|n| *n<=i32::MAX as u32).ok_or_else(|| overflow("equipment_cast_damage_sum"))?;
                runtime_control::admit_retained_growth(0,Some(std::mem::size_of::<CastDamageContribution>()),"equipment_cast_contribution")?;
                contributors.reserve_exact(1); contributors.push(CastDamageContribution{equipment,percent,authored});
            }
        }
        if contributors.is_empty() { return Ok(None); }
        let snapshot=CastDamageSnapshot{actor:owner,percent:total,contributors};
        runtime_control::admit_retained_growth(0,snapshot.retained_extent(),"equipment_cast_capture")?;
        Ok(Some(Arc::new(snapshot)))
    }
    pub(in crate::game::battle::core) fn captured_cast_damage_percent(&self, source: &DamageSourceSnapshot) -> i32 {
        use crate::game::battle::damage::DamageSource;
        let Some(capture) = source.fact_source.cast_damage().filter(|capture| capture.actor==source.source_id) else { return 0; };
        if self.equipment_direct_source(source)!=Some(DamageSource::Ability) { return 0; }
        capture.percent as i32
    }
}
