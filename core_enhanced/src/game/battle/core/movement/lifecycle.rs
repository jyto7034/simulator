use crate::game::battle::{
    core::{movement::ActionState, BattleCore},
    ids::UnitInstanceId,
    timeline::{MovementStopReason, TimelineEvent},
};

impl BattleCore {
    pub(in crate::game::battle::core) fn interrupt_movement(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        reason: MovementStopReason,
        until_ms: Option<u64>,
        next_state: ActionState,
    ) -> bool {
        let was_moving = self.units.get(&unit_id).is_some_and(|unit| {
            !matches!(unit.action_state, ActionState::Idle | ActionState::Dead)
        });

        if !was_moving {
            return false;
        }

        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.body.velocity = Default::default();
            unit.action_state = next_state;
        }

        self.record_movement_stopped(now_ms, unit_id, reason, until_ms);
        true
    }

    pub(in crate::game::battle::core) fn record_movement_stopped(
        &mut self,
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        until_ms: Option<u64>,
    ) {
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return;
        };
        self.record_timeline(
            time_ms,
            TimelineEvent::MovementStopped {
                unit_instance_id,
                reason,
                world_position: unit.body.position.quantized_milli(),
                until_ms,
            },
        );
    }
}
