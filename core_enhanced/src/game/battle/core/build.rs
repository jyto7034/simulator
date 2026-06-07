use std::collections::HashSet;

use uuid::Uuid;

use crate::game::{
    battle::{
        ids::UnitInstanceId,
        scenario::{ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef, ScenarioUnitSpawn},
        timeline::{TimelineCause, TimelineRootCause},
        types::BattleUnitDraft,
    },
    behavior::GameError,
    enums::Side,
    resources::Position,
};

use super::{
    movement::{types::UnitBody, ActionState},
    BattleCore, RuntimeArtifact, RuntimeItem, RuntimeUnit,
};

impl BattleCore {
    pub(super) fn build_runtime_artifacts_from_scenario(&mut self) -> Result<(), GameError> {
        let mut seen_artifacts: HashSet<(crate::game::enums::Side, Uuid)> = HashSet::new();
        for artifact in &self.scenario.artifacts {
            if !seen_artifacts.insert((artifact.side, artifact.base_uuid)) {
                return Err(GameError::InvalidAction);
            }
            let instance_id = Self::make_artifact_instance_id(
                artifact.base_uuid,
                artifact.side,
                artifact.instance_salt,
            );
            if self
                .artifacts
                .insert(
                    instance_id,
                    RuntimeArtifact {
                        instance_id,
                        owner: artifact.side,
                        base_uuid: artifact.base_uuid,
                    },
                )
                .is_some()
            {
                return Err(GameError::InvalidAction);
            }
        }

        Ok(())
    }

    pub(super) fn spawn_scenario_group(
        &mut self,
        group_id: &crate::game::battle::scenario::ScenarioGroupId,
    ) -> Result<Vec<UnitInstanceId>, GameError> {
        if self.scenario_runtime.spawned_groups.contains(group_id) {
            return Ok(Vec::new());
        }
        let group = self
            .scenario
            .group(group_id)
            .cloned()
            .ok_or(GameError::InvalidAction)?;

        let artifact_base_uuids: Vec<Uuid> = self
            .scenario
            .artifacts
            .iter()
            .filter(|artifact| artifact.side == group.side)
            .map(|artifact| artifact.base_uuid)
            .collect();

        let mut spawned_unit_ids = Vec::new();
        for spawn in &group.spawns {
            let unit = &spawn.draft;
            let combat_profile = unit.combat_profile(&self.game_data)?;
            let mut stats = unit.effective_stats(&self.game_data, &artifact_base_uuids)?;

            let base_uuid = unit.base_uuid();
            let instance_id = Self::make_instance_id(base_uuid, group.side, spawn.instance_salt);
            let spawn_order = self.scenario_runtime.next_spawn_order;
            self.scenario_runtime.next_spawn_order =
                self.scenario_runtime.next_spawn_order.saturating_add(1);
            let resonance_start = combat_profile.resonance.start;
            let resonance_max = combat_profile.resonance.max.max(1);
            let resonance_lock_ms = combat_profile.resonance.gain_lock_ms;
            let resonance_current = resonance_start.min(resonance_max);
            let move_speed_units_per_ms = combat_profile.movement.speed_units_per_ms;
            stats.move_speed_units_per_ms = move_speed_units_per_ms;
            let placed_position =
                self.place_scenario_spawn(instance_id, spawn.position, group.side)?;
            let spawn_position =
                super::movement::types::WorldVec2::from_tile_center(placed_position);
            let radius = combat_profile.movement.radius_units as f32
                / super::movement::types::DATA_UNITS_PER_WORLD;
            let body = UnitBody::new_at(
                spawn_position,
                radius,
                move_speed_units_per_ms as f32 * 1_000.0
                    / super::movement::types::DATA_UNITS_PER_WORLD,
            );
            let tactical_anchor = Some(spawn_position);
            let (block_capacity, block_radius_units) = runtime_blocking_stats(&combat_profile);
            if self
                .units
                .insert(
                    instance_id,
                    RuntimeUnit {
                        instance_id,
                        spawn_order,
                        source_owned_uuid: unit.owned_uuid,
                        owner: group.side,
                        role: unit.source.role(),
                        base_uuid,
                        stats,
                        incoming_damage_modifiers: combat_profile.incoming_damage_modifiers,
                        basic_attack: combat_profile.basic_attack,
                        skill_id: combat_profile.skill_id,
                        skill_activation_mode: combat_profile.skill_activation_mode,
                        body,
                        tactical_anchor,
                        enemy_movement_plan: group.enemy_movement_plan.clone(),
                        block_capacity,
                        block_radius_units,
                        blockable: combat_profile.blockable,
                        mobility_kind: combat_profile.mobility_kind,
                        target_traits: combat_profile.target_traits.clone(),
                        facing_direction: (group.side == Side::Player)
                            .then_some(crate::game::battle::tile_range::FacingDirection::Right),
                        move_epoch: 0,
                        action_state: ActionState::Idle,
                        action_locks: Default::default(),
                        current_target: None,
                        next_basic_attack_ms: 0,
                        pending_basic_attack: false,
                        resonance_current,
                        resonance_max,
                        resonance_lock_ms,
                        next_action_time: 0,
                        pending_cast: false,
                        pending_cast_cause: None,
                        pending_skill_cast: None,
                    },
                )
                .is_some()
            {
                self.battlefield.remove(instance_id);
                return Err(GameError::InvalidAction);
            }
            self.scenario_runtime
                .unit_refs
                .insert(spawn.unit_ref.clone(), instance_id);
            spawned_unit_ids.push(instance_id);

            for (idx, equipment_uuid) in unit.equipped_items.iter().enumerate() {
                let item_instance_id = Self::make_item_instance_id(
                    *equipment_uuid,
                    group.side,
                    instance_id,
                    idx as u32,
                );
                self.items.insert(
                    item_instance_id,
                    RuntimeItem {
                        instance_id: item_instance_id,
                        owner: group.side,
                        owner_unit_instance: instance_id,
                        base_uuid: *equipment_uuid,
                    },
                );
            }
        }

        self.scenario_runtime
            .spawned_groups
            .insert(group_id.clone());
        Ok(spawned_unit_ids)
    }

    fn place_scenario_spawn(
        &mut self,
        instance_id: UnitInstanceId,
        preferred_position: Position,
        side: Side,
    ) -> Result<Position, GameError> {
        match self.battlefield.place(instance_id, preferred_position) {
            Ok(()) => Ok(preferred_position),
            Err(GameError::PositionOccupied) if side == Side::Opponent => {
                let fallback = self
                    .nearest_open_spawn_position(preferred_position)
                    .ok_or(GameError::PositionOccupied)?;
                self.battlefield.place(instance_id, fallback)?;
                Ok(fallback)
            }
            Err(error) => Err(error),
        }
    }

    fn nearest_open_spawn_position(&self, preferred_position: Position) -> Option<Position> {
        let mut candidates = Vec::new();
        for y in 0..self.battlefield.height() as i32 {
            for x in 0..self.battlefield.width() as i32 {
                let position = Position::new(x, y);
                if !self.battlefield.is_walkable_tile(position) {
                    continue;
                }
                if !matches!(self.battlefield.occupant(position), Ok(None)) {
                    continue;
                }
                candidates.push(position);
            }
        }

        candidates.sort_by_key(|position| {
            (
                position.chebyshev(&preferred_position),
                position.manhattan(&preferred_position),
                position.y,
                position.x,
            )
        });
        candidates.into_iter().next()
    }

    pub(in crate::game::battle::core) fn deploy_player_unit(
        &mut self,
        draft: BattleUnitDraft,
        position: Position,
        facing: crate::game::battle::tile_range::FacingDirection,
        instance_salt: u32,
        time_ms: u64,
    ) -> Result<UnitInstanceId, GameError> {
        let group_id = ScenarioGroupId::new(format!("live_player_deploy_{instance_salt}"));
        let unit_ref = ScenarioUnitRef::new(format!("{}_0", group_id.0));
        self.scenario.groups.push(ScenarioSpawnGroup {
            id: group_id.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref,
                side: Side::Player,
                draft,
                position,
                instance_salt,
            }],
        });

        let spawned_unit_ids = self.spawn_scenario_group(&group_id)?;
        let unit_id = spawned_unit_ids
            .first()
            .copied()
            .ok_or(GameError::InvalidAction)?;
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.facing_direction = Some(facing);
        }
        self.record_spawned_units(
            time_ms,
            TimelineCause::Root {
                kind: TimelineRootCause::System,
            },
            &spawned_unit_ids,
        );
        self.schedule_initial_attack_for_unit(unit_id, time_ms);
        Ok(unit_id)
    }

    pub(in crate::game::battle::core) fn withdraw_unit(
        &mut self,
        unit_id: UnitInstanceId,
    ) -> Result<(), GameError> {
        if self.units.remove(&unit_id).is_none() {
            return Err(GameError::UnitNotFound);
        }
        self.battlefield.remove(unit_id);
        self.items
            .retain(|_, item| item.owner_unit_instance != unit_id);
        self.active_movement_segments.remove(&unit_id);
        for unit in self.units.values_mut() {
            if unit.current_target == Some(unit_id) {
                unit.current_target = None;
            }
        }
        self.block_state = Default::default();
        Ok(())
    }

    pub(super) fn build_runtime_field(&mut self) -> Result<(), GameError> {
        for unit_id in self.units.keys().copied().collect::<Vec<_>>() {
            if self.battlefield.position_of(unit_id).is_none() {
                return Err(GameError::UnitNotFound);
            }
        }
        Ok(())
    }
}

fn runtime_blocking_stats(
    combat_profile: &crate::game::battle::types::UnitCombatProfile,
) -> (u32, f32) {
    if matches!(
        combat_profile.deployment_affinity,
        crate::game::battle::types::DeploymentAffinity::PlatformOnly
    ) {
        (0, 0.0)
    } else {
        (
            combat_profile.block_capacity,
            combat_profile.block_radius_units,
        )
    }
}
