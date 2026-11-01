use crate::game::{
    battle::{
        core::movement::types::WorldVec2, event_log::BattleLogEvent, ids::UnitInstanceId,
        tile_range::FacingDirection, types::TacticalTileKind,
    },
    behavior::{ActionDisabledReasonCode, ActionKind, GameError},
};

use super::BattleCore;

impl BattleCore {
    #[allow(clippy::too_many_arguments)]
    pub fn validate_tactical_swap(
        &self,
        first_unit_id: UnitInstanceId,
        second_unit_id: UnitInstanceId,
        first_destination_tile_kind: TacticalTileKind,
        second_destination_tile_kind: TacticalTileKind,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if first_unit_id == second_unit_id {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalSwapUnits,
                reason: ActionDisabledReasonCode::AlreadyActive,
            });
        }

        for unit_id in [first_unit_id, second_unit_id] {
            if let Some(reason) = self.tactical_relocation_disabled_reason(unit_id, time_ms) {
                return Err(GameError::ActionUnavailable {
                    action: ActionKind::TacticalSwapUnits,
                    reason,
                });
            }
            if self.active_movement_segments.contains_key(&unit_id) {
                return Err(GameError::ActionUnavailable {
                    action: ActionKind::TacticalSwapUnits,
                    reason: ActionDisabledReasonCode::ActionInProgress,
                });
            }
        }

        let first_position = self
            .unit_projected_tile(first_unit_id)
            .ok_or(GameError::UnitNotFound)?;
        let second_position = self
            .unit_projected_tile(second_unit_id)
            .ok_or(GameError::UnitNotFound)?;
        if first_position == second_position {
            return Err(GameError::InvalidAction);
        }

        let pair = [first_unit_id, second_unit_id];
        self.validate_player_tactical_destination_excluding(second_position, &pair)?;
        self.validate_player_tactical_destination_excluding(first_position, &pair)?;
        self.validate_tactical_destination_profile(first_unit_id, first_destination_tile_kind)?;
        self.validate_tactical_destination_profile(second_unit_id, second_destination_tile_kind)?;
        self.validate_field_work_cell_destination(second_position, first_destination_tile_kind)?;
        self.validate_field_work_cell_destination(first_position, second_destination_tile_kind)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tactically_swap_units(
        &mut self,
        first_unit_id: UnitInstanceId,
        second_unit_id: UnitInstanceId,
        first_destination_tile_kind: TacticalTileKind,
        second_destination_tile_kind: TacticalTileKind,
        first_facing: FacingDirection,
        second_facing: FacingDirection,
        time_ms: u64,
    ) -> Result<(), GameError> {
        self.runtime_transaction(|core| {
            core.tactically_swap_units_in_candidate(
                first_unit_id,
                second_unit_id,
                first_destination_tile_kind,
                second_destination_tile_kind,
                first_facing,
                second_facing,
                time_ms,
            )
        })
    }

    pub(in crate::game::battle::core) fn tactically_swap_units_in_candidate(
        &mut self,
        first_unit_id: UnitInstanceId,
        second_unit_id: UnitInstanceId,
        first_destination_tile_kind: TacticalTileKind,
        second_destination_tile_kind: TacticalTileKind,
        first_facing: FacingDirection,
        second_facing: FacingDirection,
        time_ms: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(time_ms)?;
        self.validate_tactical_swap(
            first_unit_id,
            second_unit_id,
            first_destination_tile_kind,
            second_destination_tile_kind,
            time_ms,
        )?;
        self.settle_field_work_progress(time_ms);

        let (first_from_position, first_from_world_position, first_before_facing) = {
            let unit = self
                .units
                .get(&first_unit_id)
                .ok_or(GameError::UnitNotFound)?;
            (
                unit.body.projected_tile(),
                unit.body.position,
                unit.facing_direction.unwrap_or(FacingDirection::Right),
            )
        };
        let (second_from_position, second_from_world_position, second_before_facing) = {
            let unit = self
                .units
                .get(&second_unit_id)
                .ok_or(GameError::UnitNotFound)?;
            (
                unit.body.projected_tile(),
                unit.body.position,
                unit.facing_direction.unwrap_or(FacingDirection::Right),
            )
        };
        let cooldown_until_ms =
            time_ms.saturating_add(self.game_data.run_policy.tactical_relocation.cooldown_ms);
        let first_before_effects = self
            .derived_tactical_effects(first_unit_id)
            .expect("validated first tactical unit must retain tactical effects");
        let second_before_effects = self
            .derived_tactical_effects(second_unit_id)
            .expect("validated second tactical unit must retain tactical effects");

        // Both units are preflighted before either body is changed. Engagements are
        // released as one set, both bodies/effects are committed, and only then is
        // derived targeting rebuilt.
        self.release_tactical_engagements(&[first_unit_id, second_unit_id]);
        self.apply_tactical_position_update(
            first_unit_id,
            second_from_position,
            first_destination_tile_kind,
            first_facing,
            cooldown_until_ms,
        );
        self.apply_tactical_position_update(
            second_unit_id,
            first_from_position,
            second_destination_tile_kind,
            second_facing,
            cooldown_until_ms,
        );

        self.reconcile_field_work_with_deferred_tactical_effects(
            time_ms,
            &[first_unit_id, second_unit_id],
        );
        self.begin_relocation_defense(
            first_unit_id,
            time_ms,
            first_from_position != second_from_position,
        )?;
        self.begin_relocation_defense(
            second_unit_id,
            time_ms,
            first_from_position != second_from_position,
        )?;
        self.reconcile_tactical_effects(first_unit_id, first_before_effects, time_ms);
        self.reconcile_tactical_effects(second_unit_id, second_before_effects, time_ms);

        self.reconcile_sustained(time_ms)?;
        self.refresh_block_state();
        self.record_event_log(
            time_ms,
            BattleLogEvent::UnitsTacticallySwapped {
                first_unit_instance_id: first_unit_id,
                second_unit_instance_id: second_unit_id,
                first_from_position,
                first_to_position: second_from_position,
                second_from_position,
                second_to_position: first_from_position,
                first_from_world_position: first_from_world_position.quantized_milli(),
                first_to_world_position: WorldVec2::from_tile_center(second_from_position)
                    .quantized_milli(),
                second_from_world_position: second_from_world_position.quantized_milli(),
                second_to_world_position: WorldVec2::from_tile_center(first_from_position)
                    .quantized_milli(),
                first_before_facing,
                first_after_facing: first_facing,
                second_before_facing,
                second_after_facing: second_facing,
                cooldown_until_ms,
            },
        );
        self.try_start_pending_basic_attacks(time_ms);
        self.schedule_pending_autocasts(time_ms);
        Ok(())
    }
}
