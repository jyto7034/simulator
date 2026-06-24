use std::collections::{HashMap, HashSet};

use crate::game::{
    battle::{
        core::{BattleCore, BlockRuntimeState},
        ids::UnitInstanceId,
        scenario::{EnemyMovementPlan, PlayerMovementPlan, WinCondition},
        types::BattleUnitRole,
    },
    enums::Side,
};

use super::path::{route_progress_along_cells, route_total_length};

impl BattleCore {
    pub(in crate::game::battle::core) fn refresh_block_state(&mut self) {
        if !matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) {
            self.replace_block_state(BlockRuntimeState::default());
            return;
        }

        let mut blocker_ids = self
            .units
            .values()
            .filter(|unit| self.is_valid_blocker(unit.instance_id))
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        blocker_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut enemy_ids = self
            .units
            .values()
            .filter(|unit| self.is_valid_blockable_enemy(unit.instance_id))
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        enemy_ids.sort_by(|a, b| self.compare_enemy_block_priority(*a, *b));

        let mut remaining_capacity = blocker_ids
            .iter()
            .filter_map(|blocker_id| {
                self.units
                    .get(blocker_id)
                    .map(|unit| (*blocker_id, unit.block_capacity))
            })
            .collect::<HashMap<_, _>>();
        let mut block_state = BlockRuntimeState::default();

        let mut existing_engagements = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<Vec<_>>();
        existing_engagements.sort_by(|(left_enemy, left_blocker), (right_enemy, right_blocker)| {
            left_blocker
                .as_bytes()
                .cmp(right_blocker.as_bytes())
                .then_with(|| left_enemy.as_bytes().cmp(right_enemy.as_bytes()))
        });

        for (enemy_id, blocker_id) in existing_engagements {
            if !self.is_valid_blocker(blocker_id) || !self.is_valid_blockable_enemy(enemy_id) {
                continue;
            }
            let Some(capacity) = remaining_capacity.get_mut(&blocker_id) else {
                continue;
            };
            if *capacity == 0 {
                continue;
            }
            *capacity = capacity.saturating_sub(1);
            block_state
                .blocker_to_enemies
                .entry(blocker_id)
                .or_default()
                .push(enemy_id);
            block_state.enemy_to_blocker.insert(enemy_id, blocker_id);
        }

        for enemy_id in enemy_ids {
            if block_state.enemy_to_blocker.contains_key(&enemy_id) {
                continue;
            }
            let Some(blocker_id) =
                self.closest_available_blocker(enemy_id, &blocker_ids, &remaining_capacity)
            else {
                continue;
            };
            if let Some(capacity) = remaining_capacity.get_mut(&blocker_id) {
                *capacity = capacity.saturating_sub(1);
            }
            block_state
                .blocker_to_enemies
                .entry(blocker_id)
                .or_default()
                .push(enemy_id);
            block_state.enemy_to_blocker.insert(enemy_id, blocker_id);
        }

        for enemy_ids in block_state.blocker_to_enemies.values_mut() {
            enemy_ids.sort_by(|a, b| self.compare_enemy_block_priority(*a, *b));
        }

        self.replace_block_state(block_state);
    }

    fn replace_block_state(&mut self, next_state: BlockRuntimeState) {
        let previous_state = std::mem::replace(&mut self.block_state, next_state);
        self.clear_released_block_target_preferences(&previous_state);
        self.apply_block_target_preferences();
    }

    pub(in crate::game::battle::core) fn blocked_by(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        self.block_state.enemy_to_blocker.get(&enemy_id).copied()
    }

    pub(in crate::game::battle::core) fn first_blocked_enemy(
        &self,
        blocker_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        self.block_state
            .blocker_to_enemies
            .get(&blocker_id)
            .and_then(|enemy_ids| enemy_ids.first().copied())
    }

    pub(in crate::game::battle::core) fn fixed_defense_route_end_target_for_enemy(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        if !self.is_fixed_defense_route_enemy(enemy_id) || self.blocked_by(enemy_id).is_some() {
            return None;
        }
        let endpoint = self.fixed_defense_route_end_point_for_enemy(enemy_id)?;
        let enemy = self.units.get(&enemy_id)?;
        if enemy.body.position.distance(endpoint) > 0.25 {
            return None;
        }
        self.protected_unit_id()
    }

    pub(in crate::game::battle::core) fn protected_unit_id(&self) -> Option<UnitInstanceId> {
        let unit_ref = match &self.scenario.win_condition {
            WinCondition::ProtectUnit { unit_ref }
            | WinCondition::ProtectUnitUntil { unit_ref, .. } => Some(unit_ref),
            _ => None,
        };

        unit_ref
            .and_then(|unit_ref| self.scenario_runtime.unit_refs.get(unit_ref).copied())
            .or_else(|| {
                self.units
                    .values()
                    .filter(|unit| unit.role == BattleUnitRole::DefenseObject && unit.is_active())
                    .map(|unit| unit.instance_id)
                    .min_by(|a, b| a.as_bytes().cmp(b.as_bytes()))
            })
    }

    pub(in crate::game::battle::core) fn is_fixed_defense_route_enemy(
        &self,
        unit_id: UnitInstanceId,
    ) -> bool {
        matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) && self
            .units
            .get(&unit_id)
            .is_some_and(|unit| unit.owner == Side::Opponent)
    }

    fn fixed_defense_route_end_point_for_enemy(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<super::types::WorldVec2> {
        let plan = self
            .units
            .get(&enemy_id)
            .and_then(|unit| unit.enemy_movement_plan.as_ref())
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => cells
                .last()
                .copied()
                .map(super::types::WorldVec2::from_tile_center),
        }
    }

    fn closest_available_blocker(
        &self,
        enemy_id: UnitInstanceId,
        blocker_ids: &[UnitInstanceId],
        remaining_capacity: &HashMap<UnitInstanceId, u32>,
    ) -> Option<UnitInstanceId> {
        blocker_ids
            .iter()
            .copied()
            .filter(|blocker_id| remaining_capacity.get(blocker_id).copied().unwrap_or(0) > 0)
            .filter(|blocker_id| self.is_enemy_inside_block_radius(enemy_id, *blocker_id))
            .min_by(|a, b| {
                let distance_order = self
                    .block_distance_sq(*a, enemy_id)
                    .unwrap_or(f32::MAX)
                    .total_cmp(&self.block_distance_sq(*b, enemy_id).unwrap_or(f32::MAX));
                distance_order.then_with(|| a.as_bytes().cmp(b.as_bytes()))
            })
    }

    fn is_valid_blocker(&self, blocker_id: UnitInstanceId) -> bool {
        self.units.get(&blocker_id).is_some_and(|unit| {
            unit.owner == Side::Player
                && unit.is_combatant()
                && unit.is_active()
                && unit.block_capacity > 0
                && unit.block_radius_units > 0.0
        })
    }

    fn is_valid_blockable_enemy(&self, enemy_id: UnitInstanceId) -> bool {
        self.units.get(&enemy_id).is_some_and(|unit| {
            unit.owner == Side::Opponent
                && unit.is_combatant()
                && unit.is_active()
                && unit.blockable
                && !unit.is_airborne()
        })
    }

    fn compare_enemy_block_priority(
        &self,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> std::cmp::Ordering {
        let progress_order = self
            .enemy_route_progress(right)
            .total_cmp(&self.enemy_route_progress(left));
        progress_order
            .then_with(|| {
                let left_spawn_order = self
                    .units
                    .get(&left)
                    .map(|unit| unit.spawn_order)
                    .unwrap_or(u64::MAX);
                let right_spawn_order = self
                    .units
                    .get(&right)
                    .map(|unit| unit.spawn_order)
                    .unwrap_or(u64::MAX);
                left_spawn_order.cmp(&right_spawn_order)
            })
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    }

    pub(in crate::game::battle::core) fn enemy_route_progress(
        &self,
        enemy_id: UnitInstanceId,
    ) -> f32 {
        let Some(enemy) = self.units.get(&enemy_id) else {
            return 0.0;
        };
        let plan = enemy
            .enemy_movement_plan
            .as_ref()
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => {
                route_progress_along_cells(cells, enemy.body.position).unwrap_or(0.0)
            }
        }
    }

    pub(in crate::game::battle::core) fn enemy_route_remaining(
        &self,
        enemy_id: UnitInstanceId,
    ) -> Option<f32> {
        let enemy = self.units.get(&enemy_id)?;
        let plan = enemy
            .enemy_movement_plan
            .as_ref()
            .unwrap_or(&self.scenario.tactical_plan.enemy_plan);
        match plan {
            EnemyMovementPlan::PathAlongCells { cells } => {
                let progress = route_progress_along_cells(cells, enemy.body.position)?;
                let total = route_total_length(cells)?;
                Some((total - progress).max(0.0))
            }
        }
    }

    fn is_enemy_inside_block_radius(
        &self,
        enemy_id: UnitInstanceId,
        blocker_id: UnitInstanceId,
    ) -> bool {
        let Some(distance_sq) = self.block_distance_sq(blocker_id, enemy_id) else {
            return false;
        };
        let Some(blocker) = self.units.get(&blocker_id) else {
            return false;
        };
        let radius = blocker.block_radius_units.max(0.0);
        distance_sq <= radius * radius
    }

    fn block_distance_sq(
        &self,
        blocker_id: UnitInstanceId,
        enemy_id: UnitInstanceId,
    ) -> Option<f32> {
        let blocker = self.units.get(&blocker_id)?;
        let enemy = self.units.get(&enemy_id)?;
        Some(blocker.body.position.distance_squared(enemy.body.position))
    }

    fn clear_released_block_target_preferences(&mut self, previous_state: &BlockRuntimeState) {
        let active_pairs = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<HashSet<_>>();

        for (enemy_id, blocker_id) in previous_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
        {
            if active_pairs.contains(&(enemy_id, blocker_id)) {
                continue;
            }
            if let Some(enemy) = self.units.get_mut(&enemy_id) {
                if enemy.current_target == Some(blocker_id) {
                    enemy.current_target = None;
                }
            }
            if let Some(blocker) = self.units.get_mut(&blocker_id) {
                if blocker.current_target == Some(enemy_id) {
                    blocker.current_target = None;
                }
            }
        }
    }

    fn apply_block_target_preferences(&mut self) {
        let blocker_targets = self
            .block_state
            .blocker_to_enemies
            .iter()
            .filter_map(|(blocker_id, enemy_ids)| {
                enemy_ids.first().map(|enemy_id| (*blocker_id, *enemy_id))
            })
            .collect::<Vec<_>>();
        let enemy_targets = self
            .block_state
            .enemy_to_blocker
            .iter()
            .map(|(enemy_id, blocker_id)| (*enemy_id, *blocker_id))
            .collect::<Vec<_>>();

        for (unit_id, target_id) in blocker_targets.into_iter().chain(enemy_targets) {
            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.current_target = Some(target_id);
            }
        }
    }
}
