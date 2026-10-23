use super::*;
use crate::game::{
    ability::sustained::SustainedEffectDef,
    battle::{
        damage::{DamageModifiers, DamageSource, DamageType},
        event_log::HpChangeReason,
        runtime_facts::{ActionCancelReason, ActionTerminalKind},
        types::BattleUnitSourceIdentity,
    },
};
impl BattleCore {
    pub(in crate::game::battle::core) fn release_counter_channel(
        &mut self,
        caster: UnitInstanceId,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let key = self.counter_key(caster).ok_or(GameError::InvalidAction)?;
        let o = self
            .sustained
            .counters
            .owners
            .get_mut(&key)
            .ok_or(GameError::InvalidAction)?;
        let admitted = o.pending.take().ok_or(GameError::InvalidAction)?;
        if self.runtime_fact_source.action.as_ref() != Some(&admitted.action) || o.active.is_some()
        {
            return Err(GameError::InvalidAction);
        }
        let entry = &o.program.entries[admitted.entry];
        let phase_end = now
            .checked_add(u64::from(entry.phases[0].duration_ms))
            .ok_or(GameError::InvalidAction)?;
        let pause = entry.pause_basic;
        // The authored effect site stays attached to the private delivery ticket
        // so common resonance ownership ends with the complete channel.
        let mut snapshot = self
            .damage_source_snapshot_template_for_unit(
                caster,
                DamageSource::Ability,
                DamageType::Magic,
                1,
                DamageModifiers::default(),
                0,
                now,
                false,
            )
            .ok_or(GameError::InvalidAction)?;
        let ticket = self
            .runtime_facts
            .delivery(&admitted.action)?
            .ok_or(GameError::InvalidAction)?;
        snapshot.fact_source.resource_capabilities = RuntimeFactSource::resource_capabilities(
            snapshot.fact_source.resource_context().cloned(),
            ticket.resource.clone(),
        )?;
        snapshot.fact_source.delivery = Some(ticket);
        let id = self.next_counter_id()?;
        let fact_source = snapshot.fact_source.clone();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            snapshot
                .fact_retained_extent()
                .and_then(|n| n.checked_add(std::mem::size_of::<Active>())),
            "counter_channel_activation",
        )?;
        self.sustained.counters.owners.get_mut(&key).unwrap().active = Some(Active {
            id,
            admitted,
            snapshot,
            phase: 0,
            phase_end,
            next_at: now,
            tick: 0,
            damaged: false,
            healed: false,
        });
        if pause {
            self.pause_channel_basic_attack(caster)?;
        }
        self.event_queue.push_with_context(
            BattleEvent::CounterChannelTick {
                time_ms: now,
                source: key,
                generation: id,
            },
            None,
            fact_source,
        );
        Ok(super::super::types::SkillStepResult {
            applied_effect_count: 1,
            ..Default::default()
        })
    }
    fn counter_activation_exists(&self, key: Uuid, id: u64) -> bool {
        self.sustained
            .counters
            .owners
            .get(&key)
            .and_then(|o| o.active.as_ref())
            .is_some_and(|a| a.id == id)
    }
    pub(in crate::game::battle::core) fn counter_channel_tick(
        &mut self,
        key: Uuid,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(o) = self.sustained.counters.owners.get(&key) else {
            return Ok(());
        };
        let Some(active) = o.active.as_ref().filter(|a| a.id == id && a.next_at == now) else {
            return Ok(());
        };
        let mut a = controlled_clone(active)?;
        let p = o.program.clone();
        let caster = o.caster;
        let entry = &p.entries[a.admitted.entry];
        if now == a.phase_end {
            a.phase += 1;
            if a.phase == entry.phases.len() {
                return self.finish_counter_channel(key, now, true, false);
            }
            a.phase_end = now
                .checked_add(u64::from(entry.phases[a.phase].duration_ms))
                .ok_or(GameError::InvalidAction)?;
        }
        let phase = &entry.phases[a.phase];
        let range = match &phase.range {
            PhaseRange::Fixed(r) => r,
            PhaseRange::Consumed(v) => {
                &v.iter()
                    .find(|(count, _)| *count == a.admitted.consumed)
                    .ok_or(GameError::InvalidAction)?
                    .1
            }
        };
        let (origin, facing) = match phase.origin {
            PhaseOrigin::SelectedTile => (a.admitted.tile, FacingDirection::Up),
            PhaseOrigin::SelectedDirection => (a.admitted.origin, a.admitted.facing),
        };
        let tiles = range
            .pattern
            .affected_tiles(origin, facing)
            .map_err(GameError::InvalidStaticData)?;
        let prior = std::mem::replace(
            &mut self.runtime_fact_source,
            a.snapshot.fact_source.clone(),
        );
        let result = (|| -> Result<(), GameError> {
            for query in &phase.queries {
                if !self.counter_activation_exists(key, id) {
                    return Ok(());
                }
                let targets = self.collect_tile_area_targets_at(
                    now,
                    a.snapshot.source_side,
                    caster,
                    &tiles,
                    query.targets,
                    query.include_caster,
                );
                control::poll("counter_phase_query")?;
                for target in targets {
                    if !self.counter_activation_exists(key, id) {
                        return Ok(());
                    }
                    if query.corrupted_only
                        && !self.units.get(&target).is_some_and(|u| {
                            matches!(
                                u.source_identity,
                                BattleUnitSourceIdentity::CorrodedEmployee { .. }
                            )
                        })
                    {
                        continue;
                    }
                    match &query.effect {
                        PhaseEffect::Pull { distance_units } => self.pull_enemy_toward_tile(
                            caster,
                            target,
                            a.admitted.tile,
                            *distance_units,
                            now,
                        )?,
                        PhaseEffect::Damage {
                            amount,
                            damage_type,
                        } => {
                            let mut snapshot = controlled_clone(&a.snapshot)?;
                            snapshot.base_damage =
                                program::amount(&p, amount, a.admitted.consumed, false)?;
                            snapshot.damage_type = *damage_type;
                            snapshot.committed_at_ms = now;
                            let snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                                snapshot, target, a.tick,
                            );
                            if let Some(summary) = self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                                target_id: target, source_snapshot: snapshot, committed_at_ms: now,
                                policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                            }) { a.damaged |= summary.actual_hp_loss > 0 && self.units.get(&target).is_some_and(|u| u.owner != a.snapshot.source_side); }
                            control::poll("counter_phase_damage")?;
                        }
                        PhaseEffect::Heal { amount } => {
                            if self.stress_protected_target(target) {
                                continue;
                            }
                            let before = self
                                .units
                                .get(&target)
                                .map_or(0, |u| u.stats.current_health);
                            let healing = program::amount(&p, amount, a.admitted.consumed, true)?;
                            self.apply_hp_delta_and_record(
                                Some(caster),
                                target,
                                healing as i32,
                                now,
                                HpChangeReason::Command,
                            );
                            control::poll("counter_phase_heal")?;
                            if self
                                .units
                                .get(&target)
                                .is_some_and(|u| u.stats.current_health > before)
                            {
                                a.healed = true;
                                for modifier in &p.modifiers {
                                    if let CounterChange::HealBuff {
                                        definition,
                                        duration_ms,
                                        profile,
                                    } = &modifier.change
                                    {
                                        if definition == &amount.definition
                                            && self.counter_activation_exists(key, id)
                                        {
                                            let source =
                                                self.runtime_fact_source.independent_origin()?;
                                            let prior_buff = std::mem::replace(
                                                &mut self.runtime_fact_source,
                                                source,
                                            );
                                            let buff = self.begin_sustained(
                                                caster,
                                                target,
                                                SustainedEffectDef::TargetState {
                                                    duration_ms: *duration_ms,
                                                    profile: controlled_clone(profile)?,
                                                },
                                                now,
                                            );
                                            self.runtime_fact_source = prior_buff;
                                            buff?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        self.runtime_fact_source = prior;
        result?;
        // A lethal self hit or a nested interruption may have removed the
        // owner. Never reinsert the local copy after such cleanup.
        if self.counter_activation_exists(key, id) {
            a.tick = a.tick.checked_add(1).ok_or(GameError::InvalidAction)?;
            a.next_at = now
                .checked_add(u64::from(phase.interval_ms))
                .ok_or(GameError::InvalidAction)?
                .min(a.phase_end);
            self.event_queue.push_with_context(
                BattleEvent::CounterChannelTick {
                    time_ms: a.next_at,
                    source: key,
                    generation: id,
                },
                None,
                a.snapshot.fact_source.clone(),
            );
            self.sustained.counters.owners.get_mut(&key).unwrap().active = Some(a);
        }
        Ok(())
    }
    pub(super) fn finish_counter_channel(
        &mut self,
        key: Uuid,
        now: u64,
        natural: bool,
        terminal: bool,
    ) -> Result<(), GameError> {
        let Some(o) = self.sustained.counters.owners.get_mut(&key) else {
            return Ok(());
        };
        let Some(a) = o.active.take() else {
            return Ok(());
        };
        let entry = &o.program.entries[a.admitted.entry];
        let reset = entry.spend == CounterSpend::EndAll;
        let pause = entry.pause_basic;
        let caster = o.caster;
        let award = if natural {
            entry
                .completion_award
                .as_ref()
                .map(|award| {
                    u64::from(award.base)
                        + if a.damaged && a.healed {
                            u64::from(award.damage_and_healing_bonus)
                        } else {
                            0
                        }
                })
                .unwrap_or(0)
        } else {
            0
        };
        let award = program::gain(&o.program, CounterGainTarget::Completion, award)?;
        if reset {
            o.count = 0;
        }
        o.count = o.count.saturating_add(award).min(o.program.capacity);
        let cap = o.count >= o.program.capacity;
        if !natural {
            self.runtime_facts.terminal(
                &a.admitted.action,
                ActionTerminalKind::Cancelled(ActionCancelReason::Interrupted),
                now,
            )?;
        }
        self.event_queue
            .discard_counter_events(Some(key), cap, true)?;
        if let Some(ticket) = &a.snapshot.fact_source.delivery {
            self.runtime_facts.close_delivery(ticket, now)?;
        }
        if cap {
            self.sustained
                .counters
                .owners
                .get_mut(&key)
                .unwrap()
                .next_gain = None;
        }
        if reset && !terminal {
            self.schedule_counter_gain(key, now)?;
        }
        self.reconcile_resonance_use(caster);
        if pause && !terminal && self.units.get(&caster).is_some_and(|u| u.is_active()) {
            self.event_queue.push(BattleEvent::AttackStart {
                time_ms: now,
                attacker_instance_id: caster,
                target_instance_id: None,
                schedule_next: true,
                cause: Default::default(),
            });
        }
        control::poll("counter_channel_finished")
    }
}
