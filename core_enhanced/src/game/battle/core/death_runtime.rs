use crate::game::{
    battle::{
        core::{
            commands::RuntimeCommandBatch, movement::ActionState,
            triggers::TriggerOccurrenceContext, types::RuntimeUnitLifecycle, BattleCore, UnitBuffRemovalCause,
        },
        event_log::{BattleLogEvent, MovementStopReason},
        ids::UnitInstanceId,
    },
    enums::Side,
    stats::TriggerType,
};

impl BattleCore {
    fn death_trigger_commands(
        &mut self,
        killer_id: Option<UnitInstanceId>,
        dead_unit_id: UnitInstanceId,
        dead_owner: Side,
        time_ms: u64,
    ) -> Vec<RuntimeCommandBatch> {
        let mut commands = Vec::new();
        let death_occurrence_id =
            self.proc_occurrence_id(TriggerType::OnDeath, dead_unit_id, killer_id, time_ms, 0);

        commands.extend(
            self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                trigger_type: TriggerType::OnDeath,
                trigger_unit_id: dead_unit_id,
                counterpart_unit_id: killer_id,
                target_id: None,
                occurrence_id: death_occurrence_id,
                occurrence_index: 0,
                allow_dead_caster: true,
                committed_hit: None,
                time_ms,
            })
            .into_command_batches(),
        );

        if let Some(killer_id) = killer_id {
            commands.extend(
                self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                    trigger_type: TriggerType::OnKill,
                    trigger_unit_id: killer_id,
                    counterpart_unit_id: Some(dead_unit_id),
                    target_id: Some(dead_unit_id),
                    occurrence_id: death_occurrence_id,
                    occurrence_index: 0,
                    allow_dead_caster: false,
                    committed_hit: None,
                    time_ms,
                })
                .into_command_batches(),
            );
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
            commands.extend(
                self.dispatch_item_trigger_occurrence(TriggerOccurrenceContext {
                    trigger_type: TriggerType::OnAllyDeath,
                    trigger_unit_id: ally_id,
                    counterpart_unit_id: Some(dead_unit_id),
                    target_id: None,
                    occurrence_id: death_occurrence_id,
                    occurrence_index: 0,
                    allow_dead_caster: false,
                    committed_hit: None,
                    time_ms,
                })
                .into_command_batches(),
            );
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
        if let Some(object) = self.independent_objects.objects.get(&target_instance_id) {
            let epoch = object.epoch;
            let _ = self.retire_independent_object(
                target_instance_id,
                epoch,
                time_ms,
                super::independent_objects::ObjectRemovalReason::Destroyed,
            );
            return;
        }
        let prepared =
            match self.prepare_death_fact(source_instance_id, target_instance_id, time_ms) {
                Ok(Some(value)) => value,
                _ => return,
            };
        self.finalize_prepared_death(
            source_instance_id,
            target_instance_id,
            target_owner,
            time_ms,
            prepared,
            None,
        );
    }

    pub(super) fn finalize_prepared_death(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        target_owner: Side,
        time_ms: u64,
        prepared: super::runtime_facts::PreparedDeath,
        lethal_damage: Option<std::sync::Arc<crate::game::battle::runtime_facts::DamageFact>>,
    ) {
        self.finalize_prepared_death_with_dispatch(
            source_instance_id,
            target_instance_id,
            target_owner,
            time_ms,
            prepared,
            lethal_damage,
            true,
        );
    }

    pub(super) fn finalize_prepared_death_with_dispatch(
        &mut self,
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        target_owner: Side,
        time_ms: u64,
        prepared: super::runtime_facts::PreparedDeath,
        lethal_damage: Option<std::sync::Arc<crate::game::battle::runtime_facts::DamageFact>>,
        dispatch_optional: bool,
    ) {
        self.cancel_structured_flows(
            target_instance_id,
            time_ms,
            crate::game::battle::runtime_facts::ActionCancelReason::CasterDied,
        );
        self.settle_field_work_progress(time_ms);
        if let Some(resources) = self.runtime_facts.resources.as_mut() {
            if resources.end_deployment(target_instance_id).is_err() {
                return;
            }
        }
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

        if self
            .cancel_causal_periodics_for(target_instance_id)
            .is_err()
        {
            return;
        }

        if self.status_target_died(target_instance_id,lethal_damage.as_deref(),prepared.cause==crate::game::battle::runtime_facts::DeathCause::Ordinary,time_ms).is_err() {return;}
        if self.mobile_target_died(target_instance_id,time_ms).is_err() {return;}
        if self.volley_target_died(target_instance_id,time_ms).is_err() {return;}
        if self.relation_target_died(target_instance_id,time_ms).is_err() { return; }
        if self.weapon_mark_death(target_instance_id, time_ms).is_err() { return; }
        if self.presence_death(target_instance_id, time_ms).is_err() { return; }

        // Wave ActorDefeated is an edge observed after actor binding. Keeping
        // this epoch in active-battle state prevents absence/despawn from being
        // mistaken for death and makes snapshot restoration rollback-safe.
        if self.reconcile_sustained(time_ms).is_err() {
            return;
        }
        self.cancel_resonance_use_control(target_instance_id);
        let employee_pending = self
            .units
            .get_mut(&target_instance_id)
            .filter(|unit| Self::is_employee_resonance_unit(unit))
            .and_then(|unit| {
                unit.resonance_use = None;
                unit.next_resonance_pulse_ms = None;
                unit.pending_cast = false;
                unit.pending_cast_cause = None;
                unit.pending_skill_cast.take()
            });
        if let Some(pending) = employee_pending {
            self.cancel_pending_action(
                pending.action.as_ref(),
                time_ms,
                crate::game::battle::runtime_facts::ActionCancelReason::CasterDied,
            );
        }
        if self.cancel_uses_with_unavailable_targets(time_ms).is_err() {
            return;
        }
        self.note_wave_actor_death(target_instance_id);
        if self
            .record_death_fact(prepared, lethal_damage, time_ms)
            .is_err()
        {
            return;
        }

        self.clear_active_buffs_for_removed_unit(
            time_ms,
            target_instance_id,
            UnitBuffRemovalCause::Died,
        );

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

        if self
            .observe_causal_death(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
                dispatch_optional,
            )
            .is_err()
        {
            return;
        }
        let death_command_batches = if dispatch_optional {
            self.death_trigger_commands(
                source_instance_id,
                target_instance_id,
                target_owner,
                time_ms,
            )
        } else {
            Vec::new()
        };
        if !death_command_batches.is_empty() {
            self.with_recording_cause(death_seq, |core| {
                core.process_command_batches(death_command_batches, time_ms);
            });
        }

        self.reconcile_field_work(time_ms);

        // Death changes both occupancy and target validity globally.
        self.schedule_continuous_movement_tick(time_ms.saturating_add(1));
    }
}
