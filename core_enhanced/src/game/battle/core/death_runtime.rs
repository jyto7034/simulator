use crate::game::{
    battle::{
        core::{
            commands::TriggerEffectContext, movement::ActionState, types::RuntimeUnitLifecycle,
            BattleCore,
        },
        damage::BattleCommand,
        event_log::{BattleLogEvent, BuffExpireReason, MovementStopReason},
        ids::UnitInstanceId,
    },
    enums::Side,
    stats::TriggerType,
};

impl BattleCore {
    fn death_trigger_commands(
        &self,
        killer_id: Option<UnitInstanceId>,
        dead_unit_id: UnitInstanceId,
        dead_owner: Side,
        time_ms: u64,
    ) -> Vec<BattleCommand> {
        let mut commands = Vec::new();
        let death_occurrence_id =
            self.proc_occurrence_id(TriggerType::OnDeath, dead_unit_id, killer_id, time_ms, 0);

        commands.extend(self.trigger_commands_from_effects(
            self.collect_all_triggers(dead_unit_id, TriggerType::OnDeath),
            TriggerEffectContext {
                trigger_unit_id: dead_unit_id,
                counterpart_unit_id: killer_id,
            },
            None,
            true,
        ));
        commands.extend(
            Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(dead_unit_id, TriggerType::OnDeath),
                dead_unit_id,
                crate::game::battle::core::commands::TriggerAbilityContext {
                    trigger_type: TriggerType::OnDeath,
                    trigger_unit_id: dead_unit_id,
                    counterpart_unit_id: killer_id,
                    target_id: None,
                    occurrence_id: death_occurrence_id,
                    occurrence_index: 0,
                },
            )
            .into_iter()
            .map(|command| match command {
                BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    binding_index,
                    proc_roll_identity,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    ..
                } => BattleCommand::TriggerAbility {
                    skill_id,
                    caster_id,
                    target_id,
                    activation_source,
                    binding_index,
                    proc_roll_identity,
                    proc_chance_percent,
                    internal_cooldown_ms,
                    max_triggers_per_battle,
                    allow_dead_caster: true,
                },
                other => other,
            }),
        );

        if let Some(killer_id) = killer_id {
            commands.extend(self.trigger_commands_from_effects(
                self.collect_all_triggers(killer_id, TriggerType::OnKill),
                TriggerEffectContext {
                    trigger_unit_id: killer_id,
                    counterpart_unit_id: Some(dead_unit_id),
                },
                Some(dead_unit_id),
                false,
            ));
            commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(killer_id, TriggerType::OnKill),
                killer_id,
                crate::game::battle::core::commands::TriggerAbilityContext {
                    trigger_type: TriggerType::OnKill,
                    trigger_unit_id: killer_id,
                    counterpart_unit_id: Some(dead_unit_id),
                    target_id: Some(dead_unit_id),
                    occurrence_id: death_occurrence_id,
                    occurrence_index: 0,
                },
            ));
        }

        let mut ally_ids: Vec<_> = self
            .units
            .values()
            .filter(|unit| {
                unit.instance_id != dead_unit_id && unit.owner == dead_owner && unit.is_active()
            })
            .map(|unit| unit.instance_id)
            .collect();
        ally_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for ally_id in ally_ids {
            commands.extend(self.trigger_commands_from_effects(
                self.collect_all_triggers(ally_id, TriggerType::OnAllyDeath),
                TriggerEffectContext {
                    trigger_unit_id: ally_id,
                    counterpart_unit_id: Some(dead_unit_id),
                },
                None,
                false,
            ));
            commands.extend(Self::activation_commands_from_bindings(
                self.collect_all_trigger_activations(ally_id, TriggerType::OnAllyDeath),
                ally_id,
                crate::game::battle::core::commands::TriggerAbilityContext {
                    trigger_type: TriggerType::OnAllyDeath,
                    trigger_unit_id: ally_id,
                    counterpart_unit_id: Some(dead_unit_id),
                    target_id: None,
                    occurrence_id: death_occurrence_id,
                    occurrence_index: 0,
                },
            ));
        }

        commands
    }

    pub(in crate::game::battle::core) fn finalize_unit_death(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        target_owner: Side,
        time_ms: u64,
    ) {
        let interrupted = self.interrupt_movement(
            time_ms,
            target_instance_id,
            MovementStopReason::Died,
            None,
            ActionState::Dead,
        );
        if !interrupted {
            if let Some(target) = self.units.get_mut(&target_instance_id) {
                target.lifecycle = RuntimeUnitLifecycle::Dead;
                target.stats.current_health = 0;
                target.move_epoch = target.move_epoch.wrapping_add(1);
                target.action_state = ActionState::Dead;
            }
            self.record_movement_stopped(
                time_ms,
                target_instance_id,
                MovementStopReason::Died,
                None,
            );
        }
        if interrupted {
            if let Some(target) = self.units.get_mut(&target_instance_id) {
                target.lifecycle = RuntimeUnitLifecycle::Dead;
                target.stats.current_health = 0;
                target.action_state = ActionState::Dead;
            }
        }

        self.clear_active_buffs_for_dead_unit(time_ms, target_instance_id);

        if let Some(target) = self.units.get(&target_instance_id) {
            self.graveyard.insert(
                target_instance_id,
                target.to_snapshot(target.body.projected_tile()),
            );
        }
        let target = self
            .units
            .get(&target_instance_id)
            .expect("finalize_unit_death requires an existing runtime unit");
        let world_position = target.body.position;
        let position = target.body.projected_tile();
        let death_seq = self.record_event_log(
            time_ms,
            BattleLogEvent::UnitDied {
                unit_instance_id: target_instance_id,
                owner: target_owner,
                killer_instance_id: source_instance_id,
                world_position: world_position.quantized_milli(),
                position,
            },
        );

        let death_commands = self.death_trigger_commands(
            source_instance_id,
            target_instance_id,
            target_owner,
            time_ms,
        );
        if !death_commands.is_empty() {
            self.with_recording_cause(death_seq, |core| {
                core.process_commands(death_commands, time_ms);
            });
        }

        // Death changes both occupancy and target validity globally.
        self.schedule_continuous_movement_tick(time_ms.saturating_add(1));
    }

    fn clear_active_buffs_for_dead_unit(&mut self, time_ms: u64, dead_unit_id: UnitInstanceId) {
        let mut expired = self
            .buffs
            .keys()
            .copied()
            .filter_map(|key| {
                if key.target_instance_id == dead_unit_id {
                    Some((key, BuffExpireReason::TargetDied))
                } else if key.caster_instance_id == dead_unit_id {
                    Some((key, BuffExpireReason::CasterDied))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        expired.sort_by(|(left_key, left_reason), (right_key, right_reason)| {
            left_key
                .target_instance_id
                .cmp(&right_key.target_instance_id)
                .then_with(|| {
                    left_key
                        .caster_instance_id
                        .cmp(&right_key.caster_instance_id)
                })
                .then_with(|| left_key.buff_id.as_u64().cmp(&right_key.buff_id.as_u64()))
                .then_with(|| (*left_reason as u8).cmp(&(*right_reason as u8)))
        });
        for (key, reason) in expired {
            if self.buffs.remove(&key).is_none() {
                continue;
            }
            self.record_event_log(
                time_ms,
                BattleLogEvent::BuffExpired {
                    caster_instance_id: key.caster_instance_id,
                    target_instance_id: key.target_instance_id,
                    buff_id: key.buff_id,
                    reason,
                },
            );
        }
    }
}
