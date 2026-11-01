use crate::game::battle::{
    core::BattleCore,
    event_log::{BattleLogEvent, MovementStopReason},
    ids::UnitInstanceId,
};

impl BattleCore {
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
        self.record_event_log(
            time_ms,
            BattleLogEvent::MovementStopped {
                unit_instance_id,
                reason,
                world_position: unit.body.position.quantized_milli(),
                stopped_at_ms: time_ms,
                until_ms,
            },
        );
    }
}
