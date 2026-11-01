use crate::game::{
    battle::{
        core::movement::{types::WorldVec2, ActionState},
        event_log::BattleLogEvent,
        ids::UnitInstanceId,
        tile_range::FacingDirection,
        types::{BattleUnitRole, TacticalTileKind},
    },
    behavior::{ActionDisabledReasonCode, ActionKind, GameError},
    enums::Side,
    resources::Position,
};

use super::BattleCore;

impl BattleCore {
    /// Returns the active allied logical occupant of a player tactical slot.
    /// Enemy overlap is deliberately ignored by the deployment/relocation contract.
    #[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
    pub fn player_tactical_slot_occupant(
        &self,
        position: Position,
        excluding: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let excluded = excluding.into_iter().collect::<Vec<_>>();
        self.player_tactical_slot_occupant_excluding(position, &excluded)
    }

    pub(super) fn player_tactical_slot_occupant_excluding(
        &self,
        position: Position,
        excluding: &[UnitInstanceId],
    ) -> Option<UnitInstanceId> {
        self.units
            .values()
            .filter(|unit| {
                unit.is_active()
                    && (unit.owner == Side::Player
                        || self
                            .independent_objects
                            .objects
                            .contains_key(&unit.instance_id))
                    && matches!(
                        unit.role,
                        BattleUnitRole::Combatant
                            | BattleUnitRole::DefenseObject
                            | BattleUnitRole::IndependentWall
                    )
                    && !excluding.contains(&unit.instance_id)
                    && unit.body.projected_tile() == position
            })
            .map(|unit| unit.instance_id)
            .chain(
                self.independent_objects
                    .objects
                    .values()
                    .filter(|object| {
                        !self.units.contains_key(&object.id)
                            && object.position == position
                            && !excluding.contains(&object.id)
                    })
                    .map(|object| object.id),
            )
            .min()
    }

    pub(super) fn validate_player_tactical_destination_excluding(
        &self,
        position: Position,
        excluding: &[UnitInstanceId],
    ) -> Result<(), GameError> {
        self.battlefield.ensure_walkable_tile(position)?;
        if self
            .player_tactical_slot_occupant_excluding(position, excluding)
            .is_some()
        {
            return Err(GameError::PositionOccupied);
        }
        Ok(())
    }

    pub fn validate_player_tactical_destination(
        &self,
        position: Position,
        excluding: Option<UnitInstanceId>,
    ) -> Result<(), GameError> {
        let excluded = excluding.into_iter().collect::<Vec<_>>();
        self.validate_player_tactical_destination_excluding(position, &excluded)
    }

    pub fn tactical_relocation_disabled_reason(
        &self,
        unit_instance_id: UnitInstanceId,
        time_ms: u64,
    ) -> Option<ActionDisabledReasonCode> {
        if self.channel_active(unit_instance_id) || self.rooted(unit_instance_id) {
            return Some(ActionDisabledReasonCode::ActionLocked);
        }
        let Some(unit) = self.units.get(&unit_instance_id) else {
            return Some(ActionDisabledReasonCode::UnitUnavailable);
        };
        let Some(state) = self.tactical_states.get(&unit_instance_id) else {
            return Some(ActionDisabledReasonCode::WeaponRequired);
        };
        if unit.owner != Side::Player || !unit.is_combatant() || !unit.is_active() {
            return Some(ActionDisabledReasonCode::UnitUnavailable);
        }
        if self
            .unit_control_state(unit_instance_id, time_ms)
            .is_some_and(|control| control.hard_cc_active())
        {
            return Some(ActionDisabledReasonCode::HardCrowdControl);
        }
        if unit.basic_attack_windup_until_ms.is_some() || unit.pending_skill_cast.is_some() {
            return Some(ActionDisabledReasonCode::ActionInProgress);
        }
        if time_ms < state.relocation_cooldown_until_ms {
            return Some(ActionDisabledReasonCode::RelocationCooldown);
        }
        None
    }

    pub fn validate_tactical_relocation(
        &self,
        unit_instance_id: UnitInstanceId,
        destination: Position,
        tile_kind: TacticalTileKind,
        facing: FacingDirection,
        time_ms: u64,
    ) -> Result<(), GameError> {
        if let Some(reason) = self.tactical_relocation_disabled_reason(unit_instance_id, time_ms) {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalRelocateUnit,
                reason,
            });
        }
        self.validate_player_tactical_destination(destination, Some(unit_instance_id))?;
        self.validate_field_work_cell_destination(destination, tile_kind)?;

        let unit = self
            .units
            .get(&unit_instance_id)
            .ok_or(GameError::UnitNotFound)?;
        if unit.body.projected_tile() == destination && unit.facing_direction == Some(facing) {
            return Err(GameError::ActionUnavailable {
                action: ActionKind::TacticalRelocateUnit,
                reason: ActionDisabledReasonCode::AlreadyActive,
            });
        }

        self.validate_tactical_destination_profile(unit_instance_id, tile_kind)
    }

    pub(super) fn validate_tactical_destination_profile(
        &self,
        unit_instance_id: UnitInstanceId,
        tile_kind: TacticalTileKind,
    ) -> Result<(), GameError> {
        let state = self
            .tactical_states
            .get(&unit_instance_id)
            .ok_or(GameError::InvalidAction)?;
        let affinity_allowed = match tile_kind {
            TacticalTileKind::Ground => state.weapon_profile.deployment_affinity.allows_ground(),
            TacticalTileKind::Platform => {
                state.weapon_profile.deployment_affinity.allows_platform()
            }
        };
        if !affinity_allowed {
            return Err(GameError::InvalidAction);
        }
        match tile_kind {
            TacticalTileKind::Ground if state.weapon_profile.ground_defense_focus.is_none() => {
                return Err(GameError::InvalidStaticData(
                    "ground relocation weapon profile is missing ground_defense_focus".to_string(),
                ));
            }
            TacticalTileKind::Platform if state.weapon_profile.platform_rest.is_none() => {
                return Err(GameError::InvalidStaticData(
                    "platform relocation weapon profile is missing platform_rest".to_string(),
                ));
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn release_tactical_engagements(&mut self, unit_instance_ids: &[UnitInstanceId]) {
        for unit_instance_id in unit_instance_ids {
            let released_enemy_ids = self
                .block_state
                .blocker_to_enemies
                .remove(unit_instance_id)
                .unwrap_or_default();
            for enemy_id in released_enemy_ids {
                self.block_state.enemy_to_blocker.remove(&enemy_id);
            }
        }
        for unit in self.units.values_mut() {
            if unit_instance_ids.contains(&unit.instance_id)
                || unit
                    .current_target
                    .is_some_and(|target| unit_instance_ids.contains(&target))
            {
                unit.current_target = None;
            }
        }
    }

    pub(super) fn apply_tactical_position_update(
        &mut self,
        unit_instance_id: UnitInstanceId,
        destination: Position,
        tile_kind: TacticalTileKind,
        facing: FacingDirection,
        cooldown_until_ms: u64,
    ) {
        let old_tile_kind = self
            .tactical_states
            .get(&unit_instance_id)
            .expect("validated tactical unit must retain tactical state")
            .tile_kind;
        let tile_kind_changed = old_tile_kind != tile_kind;
        let destination_world = WorldVec2::from_tile_center(destination);

        {
            let unit = self
                .units
                .get_mut(&unit_instance_id)
                .expect("validated tactical unit must remain live");
            unit.body.position = destination_world;
            unit.tactical_anchor = Some(destination_world);
            unit.facing_direction = Some(facing);
            unit.current_target = None;
            unit.action_state = ActionState::Idle;
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
        }
        self.active_movement_segments.remove(&unit_instance_id);

        {
            let state = self
                .tactical_states
                .get_mut(&unit_instance_id)
                .expect("validated tactical unit must retain tactical state");
            state.relocation_cooldown_until_ms = cooldown_until_ms;
            if tile_kind_changed {
                state.tile_kind = tile_kind;
            }
        }
    }

    pub fn tactically_relocate_unit(
        &mut self,
        unit_instance_id: UnitInstanceId,
        destination: Position,
        tile_kind: TacticalTileKind,
        facing: FacingDirection,
        time_ms: u64,
    ) -> Result<(), GameError> {
        self.runtime_transaction(|core| {
            core.tactically_relocate_unit_in_candidate(
                unit_instance_id,
                destination,
                tile_kind,
                facing,
                time_ms,
            )
        })
    }

    pub(in crate::game::battle::core) fn tactically_relocate_unit_in_candidate(
        &mut self,
        unit_instance_id: UnitInstanceId,
        destination: Position,
        tile_kind: TacticalTileKind,
        facing: FacingDirection,
        time_ms: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(time_ms)?;
        self.validate_tactical_relocation(
            unit_instance_id,
            destination,
            tile_kind,
            facing,
            time_ms,
        )?;
        self.settle_field_work_progress(time_ms);

        let (from_position, from_world_position, before_facing) = {
            let unit = self
                .units
                .get(&unit_instance_id)
                .ok_or(GameError::UnitNotFound)?;
            (
                unit.body.projected_tile(),
                unit.body.position,
                unit.facing_direction.unwrap_or(FacingDirection::Right),
            )
        };
        let cooldown_until_ms =
            time_ms.saturating_add(self.game_data.run_policy.tactical_relocation.cooldown_ms);
        let destination_world = WorldVec2::from_tile_center(destination);
        let before_effects = self
            .derived_tactical_effects(unit_instance_id)
            .expect("validated tactical unit must retain tactical effects");

        // Release old engagements before moving. Targets are recalculated after the
        // atomic body/tile-effect update below.
        self.release_tactical_engagements(&[unit_instance_id]);
        self.apply_tactical_position_update(
            unit_instance_id,
            destination,
            tile_kind,
            facing,
            cooldown_until_ms,
        );

        self.reconcile_field_work_with_deferred_tactical_effects(time_ms, &[unit_instance_id]);
        self.begin_relocation_defense(unit_instance_id, time_ms, from_position != destination)?;
        self.reconcile_tactical_effects(unit_instance_id, before_effects, time_ms);

        self.reconcile_sustained(time_ms)?;
        self.refresh_block_state();
        self.record_event_log(
            time_ms,
            BattleLogEvent::UnitTacticallyRelocated {
                unit_instance_id,
                from_position,
                to_position: destination,
                from_world_position: from_world_position.quantized_milli(),
                to_world_position: destination_world.quantized_milli(),
                before_facing,
                after_facing: facing,
                cooldown_until_ms,
            },
        );
        self.try_start_pending_basic_attacks(time_ms);
        self.schedule_pending_autocasts(time_ms);
        Ok(())
    }
}
