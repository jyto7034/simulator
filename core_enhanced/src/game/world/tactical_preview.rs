use super::GameCore;
use crate::game::{
    battle::{ids::UnitInstanceId, types::TacticalTileKind},
    behavior::{
        ActionDisabledReasonCode, ActionKind, BattlePayloadEligibilityDto, BehaviorResult,
        GameError, TacticalCommandPreviewInput, TacticalCommandPreviewResultDto,
        TacticalCommandPreviewUnitDto,
    },
    combat_preview::DeploymentZoneKind,
};
use uuid::Uuid;

impl GameCore {
    pub(super) fn handle_preview_tactical_command(
        &self,
        command: TacticalCommandPreviewInput,
    ) -> Result<BehaviorResult, GameError> {
        let active = self
            .state
            .active_battle
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let actor = |employee_uuid: Uuid| -> Result<UnitInstanceId, GameError> {
            let employee = self
                .roster()?
                .get(&employee_uuid)
                .ok_or(GameError::UnitNotFound)?;
            if let Some(reason) = employee.battle_deployment_disabled_reason() {
                return Err(GameError::ActionUnavailable {
                    action: ActionKind::PreviewTacticalCommand,
                    reason,
                });
            }
            active
                .live_deployment
                .as_ref()
                .and_then(|deployment| deployment.deployed_units.get(&employee_uuid))
                .map(|unit| unit.unit_instance_id)
                .ok_or(GameError::ActionUnavailable {
                    action: ActionKind::PreviewTacticalCommand,
                    reason: ActionDisabledReasonCode::UnitUnavailable,
                })
        };
        let tile_kind = |position| -> Result<TacticalTileKind, GameError> {
            match Self::deployment_zone_kind_for_cell(&active.combat_preview, position)
                .ok_or(GameError::OutOfBounds)?
            {
                DeploymentZoneKind::Ground => Ok(TacticalTileKind::Ground),
                DeploymentZoneKind::Platform => Ok(TacticalTileKind::Platform),
            }
        };
        let cost = self.run_policy().tactical_relocation.cost;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            std::mem::size_of::<TacticalCommandPreviewUnitDto>().checked_mul(2),
            "tactical_preview_storage",
        )?;
        let mut units = Vec::with_capacity(2);
        let time_ms = active.execution.last_event_time_ms();
        let validation = match &command {
            TacticalCommandPreviewInput::Relocate {
                employee_uuid,
                destination,
                facing,
            } => {
                let id = actor(*employee_uuid)?;
                let kind = tile_kind(*destination)?;
                let validation = active.battle.validate_tactical_relocation(
                    id,
                    *destination,
                    kind,
                    *facing,
                    time_ms,
                );
                units.push(TacticalCommandPreviewUnitDto {
                    employee_uuid: *employee_uuid,
                    unit_instance_id: id,
                    destination: *destination,
                    facing: *facing,
                    forced_defense_ms: if active.battle.destination_starts_field_work(*destination)
                    {
                        0
                    } else {
                        1500
                    },
                    range_previews: active.battle.runtime_range_previews(
                        id,
                        *destination,
                        Some(*facing),
                    )?,
                });
                validation
            }
            TacticalCommandPreviewInput::Swap {
                first_employee_uuid,
                second_employee_uuid,
                first_facing,
                second_facing,
            } => {
                let first = actor(*first_employee_uuid)?;
                let second = actor(*second_employee_uuid)?;
                let first_destination = active
                    .battle
                    .unit_projected_tile(second)
                    .ok_or(GameError::UnitNotFound)?;
                let second_destination = active
                    .battle
                    .unit_projected_tile(first)
                    .ok_or(GameError::UnitNotFound)?;
                let validation = active.battle.validate_tactical_swap(
                    first,
                    second,
                    tile_kind(first_destination)?,
                    tile_kind(second_destination)?,
                    time_ms,
                );
                for (employee_uuid, id, destination, facing) in [
                    (
                        *first_employee_uuid,
                        first,
                        first_destination,
                        *first_facing,
                    ),
                    (
                        *second_employee_uuid,
                        second,
                        second_destination,
                        *second_facing,
                    ),
                ] {
                    units.push(TacticalCommandPreviewUnitDto {
                        employee_uuid,
                        unit_instance_id: id,
                        destination,
                        facing,
                        forced_defense_ms: if active
                            .battle
                            .destination_starts_field_work(destination)
                        {
                            0
                        } else {
                            1500
                        },
                        range_previews: active.battle.runtime_range_previews(
                            id,
                            destination,
                            Some(facing),
                        )?,
                    });
                }
                validation
            }
        };
        let eligibility = match validation {
            Err(GameError::ActionUnavailable { reason, .. }) => {
                BattlePayloadEligibilityDto::disabled(reason)
            }
            Err(other) => return Err(other),
            Ok(()) if active.settings_open() => {
                BattlePayloadEligibilityDto::disabled(ActionDisabledReasonCode::SettingsOpen)
            }
            Ok(()) if !active.tactical_resource.can_pay(cost) => {
                BattlePayloadEligibilityDto::disabled(
                    ActionDisabledReasonCode::InsufficientResources,
                )
            }
            Ok(()) => BattlePayloadEligibilityDto::enabled(),
        };
        Ok(BehaviorResult::TacticalCommandPreview {
            result: TacticalCommandPreviewResultDto {
                battle_uuid: active.battle_uuid,
                battle_state_revision: active.battle_state_revision,
                command,
                cost,
                relocation_cooldown_ms: self.run_policy().tactical_relocation.cooldown_ms,
                forced_defense_ms: 1500,
                eligibility,
                units,
            },
        })
    }
}
