//! A mitigated hit owns a finite repayment schedule, independent of later hits.
use super::*;
use crate::game::battle::{
    damage::{
        calculate_damage, DamageContext, DamageModifiers, DamageSource, DamageSourceSnapshot,
        DamageType,
    },
    enums::BattleEvent,
    event_log::HpChangeReason,
    runtime_facts::ProtectionReceipt,
};

#[derive(Debug, Clone)]
pub(in crate::game::battle::core) struct Debt {
    pub(in crate::game::battle::core) owner: UnitInstanceId,
    equipment: EquipmentSourceSnapshot,
    pub(in crate::game::battle::core) remaining: u32,
    ticks_left: u32,
    tick_ms: u32,
    next_at: u64,
    incoming: Arc<RuntimeFactSource>,
    snapshot: DamageSourceSnapshot,
}
crate::authoring::clone_shape::struct_shape!(Debt {
    owner,
    equipment,
    remaining,
    ticks_left,
    tick_ms,
    next_at,
    incoming,
    snapshot
});
impl Debt {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        size_of::<Self>()
            .checked_add(self.incoming.retained_extent()?)?
            .checked_add(self.snapshot.fact_retained_extent()?)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum DeferredOperation {
    Store,
    Repay,
    ClearOnKill,
}
crate::authoring::clone_shape::copy_shape!(DeferredOperation);

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeferredDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub operation: DeferredOperation,
    pub amount: u64,
    pub actual_hp_loss: u32,
    pub remaining: u64,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(DeferredDamageWitness {
    generation,
    target,
    equipment,
    time_ms,
    operation,
    amount,
    actual_hp_loss,
    remaining,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl DeferredDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}

impl BattleCore {
    pub(in crate::game::battle::core) fn reconcile_deferred_damage(
        &mut self,
    ) -> Result<(), GameError> {
        control::charge(
            self.equipment_defense.deferred.len() as u64,
            "deferred_reconcile",
        )?;
        let units = &self.units;
        let items = &self.items;
        let catalog = self.game_data.equipment_defense_catalog();
        self.equipment_defense.deferred.retain(|_, debt| {
            debt.remaining > 0
                && units.get(&debt.owner).is_some_and(|u| {
                    u.is_active()
                        && u.is_combatant()
                        && u.equipment_slots.as_ref().is_none_or(|s| {
                            s.slots.iter().flatten().any(|s| {
                                s.runtime_item_instance_id
                                    == debt.equipment.runtime_item_instance_id
                            })
                        })
                })
                && items
                    .get(&debt.equipment.runtime_item_instance_id)
                    .is_some_and(|item| {
                        item.owner_unit_instance == debt.owner
                            && item.base_uuid == debt.equipment.definition_uuid
                            && catalog.by_equipment.get(&item.base_uuid).is_some_and(|p| {
                                matches!(p.program, EquipmentDefenseProgram::DeferredDamage { .. })
                            })
                    })
        });
        Ok(())
    }

    fn next_deferred_provider(
        &self,
        owner: UnitInstanceId,
        cursor: Option<uuid::Uuid>,
    ) -> Result<Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentDefense>)>, GameError> {
        let mut selected = None;
        self.visit_equipment_defenses(owner, |equipment, plan| {
            let copy = equipment.runtime_item_instance_id;
            if cursor.is_some_and(|c| copy <= c)
                || !matches!(plan.program, EquipmentDefenseProgram::DeferredDamage { .. })
            {
                return;
            }
            if selected
                .as_ref()
                .is_none_or(|(old, _): &(EquipmentSourceSnapshot, _)| {
                    copy < old.runtime_item_instance_id
                })
            {
                selected = Some((equipment, Arc::clone(plan)));
            }
        })?;
        Ok(selected)
    }
    fn deferred_eligible(&self, target: UnitInstanceId, snapshot: &DamageSourceSnapshot) -> bool {
        self.game_data.equipment_defense_catalog().has_deferred
            && snapshot.source_id != target
            && self.units.get(&target).is_some_and(|u| {
                u.is_active() && u.is_combatant() && !u.corroded && u.owner != snapshot.source_side
            })
    }
    pub(in crate::game::battle::core) fn immediate_primary_after_deferral(
        &self,
        target: UnitInstanceId,
        mut amount: u32,
        snapshot: &DamageSourceSnapshot,
    ) -> Result<u32, GameError> {
        if !self.deferred_eligible(target, snapshot) {
            return Ok(amount);
        }
        let mut cursor = None;
        while let Some((equipment, plan)) = self.next_deferred_provider(target, cursor)? {
            cursor = Some(equipment.runtime_item_instance_id);
            let EquipmentDefenseProgram::DeferredDamage { defer_percent, .. } = plan.program else {
                unreachable!()
            };
            amount -= (u64::from(amount) * u64::from(defer_percent) / 100) as u32;
        }
        Ok(amount)
    }

    pub(in crate::game::battle::core) fn defer_equipment_damage(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        amount: u32,
        now: u64,
    ) -> Result<u32, GameError> {
        if amount == 0 || !self.deferred_eligible(target, incoming) {
            return Ok(0);
        }
        self.reconcile_deferred_damage()?;
        let mut pending = amount;
        let mut cursor = None;
        while let Some((equipment, plan)) = self.next_deferred_provider(target, cursor)? {
            cursor = Some(equipment.runtime_item_instance_id);
            let EquipmentDefenseProgram::DeferredDamage {
                defer_percent,
                duration_ms,
                tick_ms,
                ..
            } = plan.program
            else {
                unreachable!()
            };
            let deferred = (u64::from(pending) * u64::from(defer_percent) / 100) as u32;
            if deferred == 0 {
                continue;
            }
            let next_at = now
                .checked_add(u64::from(tick_ms))
                .ok_or_else(|| runtime_control::invariant("deferred_schedule_overflow"))?;
            now.checked_add(u64::from(duration_ms))
                .ok_or_else(|| runtime_control::invariant("deferred_end_overflow"))?;
            incoming.fact_source.admit_owner("deferred_incoming")?;
            let source = RuntimeFactSource {
                actor: Some(incoming.source_id),
                equipment: Some(equipment),
                authored: plan.deferred_source.clone(),
                reaction: true,
                use_control: false,
                ..Default::default()
            };
            source.admit_owner("deferred_repayment_source")?;
            let snapshot = DamageSourceSnapshot {
                fact_source: source,
                basic_primary: None,
                reactive_capture: None,
                mobile_capture: None,
                weapon_mark_capture: None,
                presence_capture: None,
                weapon_growth: None,
                equipment_attack: None,
            equipment_hits: None,
            release_gap: None,
            magazine: None,
                prepared_basic: None,
                state_basic_capture: None,
                skill_cast_source: None,
                value_expr_capture: None,
                source_id: incoming.source_id,
                source_side: incoming.source_side,
                source_attack: 0,
                source: DamageSource::Environment,
                damage_type: DamageType::True,
                base_damage: deferred,
                modifiers: DamageModifiers::default(),
                crit_roll_percent: None,
                crit_roll_identity: incoming.crit_roll_identity,
                minimum_damage: 0,
                committed_at_ms: now,
                basic_lifesteal: Vec::new(),
                on_attack_modifiers: DamageModifiers::default(),
                on_attack_bonus_damage: Vec::new(),
                on_attack_modifier_sources: Vec::new(),
            };
            let debt = Debt {
                owner: target,
                equipment,
                remaining: deferred,
                ticks_left: duration_ms / tick_ms,
                tick_ms,
                next_at,
                incoming: Arc::new(incoming.fact_source.observation()),
                snapshot,
            };
            let retained = self.equipment_defense.retained_extent();
            runtime_control::admit_retained_growth(
                retained.unwrap_or(usize::MAX),
                retained.and_then(|n| debt.retained_extent().and_then(|d| n.checked_add(d + 104))),
                "deferred_record_storage",
            )?;
            self.admit_authoring_runtime_growth(1, "deferred_record_entity")?;
            let id = self
                .equipment_defense
                .next_debt
                .checked_add(1)
                .ok_or_else(|| runtime_control::invariant("deferred_id_overflow"))?;
            self.equipment_defense.next_debt = id;
            self.equipment_defense.deferred.insert(id, debt);
            self.event_queue.push(BattleEvent::DeferredDamageTick {
                time_ms: next_at,
                debt: id,
            });
            pending -= deferred;
            self.observe_deferred(
                target,
                equipment,
                now,
                DeferredOperation::Store,
                u64::from(deferred),
                0,
                &incoming.fact_source,
            )?;
            control::poll("deferred_stored")?;
        }
        Ok(amount - pending)
    }

    pub(in crate::game::battle::core) fn deferred_damage_tick(
        &mut self,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(now)?;
        let Some(debt) = self.equipment_defense.deferred.get(&id) else {
            return Ok(());
        };
        if debt.next_at != now {
            return Ok(());
        }
        runtime_control::admit_retained_growth(0, debt.retained_extent(), "deferred_tick_copy")?;
        let mut debt = debt.clone();
        let amount = debt.remaining / debt.ticks_left;
        debt.remaining -= amount;
        debt.ticks_left -= 1;
        if debt.remaining == 0 || debt.ticks_left == 0 {
            self.equipment_defense.deferred.remove(&id);
        } else {
            debt.next_at = now
                .checked_add(u64::from(debt.tick_ms))
                .ok_or_else(|| runtime_control::invariant("deferred_tick_overflow"))?;
            self.event_queue.push(BattleEvent::DeferredDamageTick {
                time_ms: debt.next_at,
                debt: id,
            });
            self.equipment_defense.deferred.insert(id, debt.clone());
        }
        let old = std::mem::replace(
            &mut self.runtime_fact_source,
            debt.snapshot.fact_source.clone(),
        );
        let prior_scope = self.begin_strike_scope();
        let applied = self.apply_deferred_repayment(&mut debt.snapshot, debt.owner, amount, now);
        self.end_strike_scope(prior_scope);
        self.runtime_fact_source = old;
        let lost = applied?;
        self.observe_deferred(
            debt.owner,
            debt.equipment,
            now,
            DeferredOperation::Repay,
            u64::from(amount),
            lost,
            &debt.incoming,
        )?;
        control::poll("deferred_repaid")?;
        Ok(())
    }

    fn apply_deferred_repayment(
        &mut self,
        snapshot: &mut DamageSourceSnapshot,
        target: UnitInstanceId,
        amount: u32,
        now: u64,
    ) -> Result<u32, GameError> {
        if amount == 0 {
            return Ok(0);
        }
        let Some(unit) = self.units.get(&target).filter(|u| u.is_active()) else {
            return Ok(0);
        };
        let hp = unit.stats.current_health;
        snapshot.base_damage = amount;
        snapshot.committed_at_ms = now;
        let ctx = DamageContext {
            attacker_side: snapshot.source_side,
            target_side: unit.owner,
            attacker_attack: 0,
            target_armor: 0,
            target_magic_resist: 0,
            target_incoming_modifiers: DamageModifiers::default(),
            target_current_hp: hp,
            target_max_hp: unit.stats.max_health,
            on_attack_effects: &[],
            on_hit_effects: &[],
        };
        let mut result = calculate_damage(&snapshot.request(target), &ctx);
        let mut protection = ProtectionReceipt {
            mitigated_damage: amount,
            ..Default::default()
        };
        if self.sustained_invulnerable(target, now)? {
            protection.prevented = amount;
        } else {
            self.prevent_sustained_death(target, amount, now, &mut protection)?;
        }
        result.final_damage = amount - protection.prevented;
        result.target_remaining_hp = hp.saturating_sub(result.final_damage);
        result.target_killed = hp > 0 && result.target_remaining_hp == 0;
        let lost = hp - result.target_remaining_hp;
        let strike = self.admit_damage_strike(target)?;
        let fact = self.prepare_damage_fact(
            &result,
            protection,
            strike,
            snapshot,
            None,
            &[],
            DamageModifiers::default(),
            hp,
            now,
        )?;
        self.apply_damage_result_and_record(
            Some(snapshot.source_id),
            &result,
            fact,
            now,
            HpChangeReason::Command,
        );
        control::poll("deferred_hp_applied")?;
        if lost > 0 && !result.target_killed {
            self.release_overheal_reserve(target, &snapshot.fact_source, now)?;
        }
        self.relation_shields_after_damage(target, now)?;
        Ok(lost)
    }

    pub(in crate::game::battle::core) fn clear_deferred_on_kill(
        &mut self,
        killer: Option<UnitInstanceId>,
        dead: UnitInstanceId,
        dead_side: Side,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_deferred {
            return Ok(());
        }
        let Some(killer) = killer.filter(|id| {
            *id != dead
                && self.units.get(id).is_some_and(|u| {
                    u.is_active() && u.is_combatant() && !u.corroded && u.owner != dead_side
                })
        }) else {
            return Ok(());
        };
        self.reconcile_deferred_damage()?;
        let mut cursor = None;
        while let Some((equipment, plan)) = self.next_deferred_provider(killer, cursor)? {
            cursor = Some(equipment.runtime_item_instance_id);
            let EquipmentDefenseProgram::DeferredDamage {
                kill_clear_percent, ..
            } = plan.program
            else {
                unreachable!()
            };
            let total = self.deferred_balance(killer, equipment.runtime_item_instance_id)?;
            let clear = total / 100 * u64::from(kill_clear_percent)
                + total % 100 * u64::from(kill_clear_percent) / 100;
            if clear == 0 {
                continue;
            }
            let mut left = clear;
            control::charge(
                self.equipment_defense.deferred.len() as u64,
                "deferred_clear_scan",
            )?;
            for debt in self.equipment_defense.deferred.values_mut().filter(|r| {
                r.owner == killer
                    && r.equipment.runtime_item_instance_id == equipment.runtime_item_instance_id
            }) {
                let remove = left.min(u64::from(debt.remaining)) as u32;
                debt.remaining -= remove;
                left -= u64::from(remove);
                if left == 0 {
                    break;
                }
            }
            self.reconcile_deferred_damage()?;
            let incoming = self.runtime_fact_source.observation();
            self.observe_deferred(
                killer,
                equipment,
                now,
                DeferredOperation::ClearOnKill,
                clear,
                0,
                &incoming,
            )?;
            control::poll("deferred_cleared_on_kill")?;
        }
        Ok(())
    }
    fn deferred_balance(&self, owner: UnitInstanceId, copy: uuid::Uuid) -> Result<u64, GameError> {
        control::charge(
            self.equipment_defense.deferred.len() as u64,
            "deferred_balance",
        )?;
        self.equipment_defense
            .deferred
            .values()
            .filter(|r| r.owner == owner && r.equipment.runtime_item_instance_id == copy)
            .try_fold(0u64, |n, r| {
                n.checked_add(u64::from(r.remaining))
                    .ok_or_else(|| runtime_control::invariant("deferred_balance_overflow"))
            })
    }
    #[allow(clippy::too_many_arguments)]
    fn observe_deferred(
        &mut self,
        target: UnitInstanceId,
        equipment: EquipmentSourceSnapshot,
        now: u64,
        operation: DeferredOperation,
        amount: u64,
        actual_hp_loss: u32,
        incoming: &RuntimeFactSource,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let remaining = self.deferred_balance(target, equipment.runtime_item_instance_id)?;
        let plan =
            &self.game_data.equipment_defense_catalog().by_equipment[&equipment.definition_uuid];
        incoming.admit_owner("deferred_witness_source")?;
        runtime_control::admit_retained_growth(
            0,
            incoming.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<DeferredDamageWitness>()
                        + plan.definition_id.len()
                        + plan.source_file.len()
                        + plan.field_path.len(),
                )
            }),
            "deferred_witness_storage",
        )?;
        let witness = DeferredDamageWitness {
            generation: self.content_identity.clone(),
            target,
            equipment,
            time_ms: now,
            operation,
            amount,
            actual_hp_loss,
            remaining,
            definition_id: plan.definition_id.clone(),
            source_file: plan.source_file.clone(),
            field_path: plan.field_path.clone(),
            incoming: incoming.observation(),
        };
        runtime_control::admit_retained_growth(
            0,
            witness.retained_extent(),
            "deferred_observation",
        )?;
        if let Some(o) = &mut self.authoring_observation {
            o.latest_deferred_damage = Some(witness);
        }
        Ok(())
    }
}
