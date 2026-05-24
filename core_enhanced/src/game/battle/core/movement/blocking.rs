use std::collections::HashMap;

use crate::game::{
    battle::{
        core::{BattleCore, BlockRuntimeState},
        ids::UnitInstanceId,
        scenario::{EnemyMovementPlan, PlayerMovementPlan, WinCondition},
        types::BattleUnitRole,
    },
    enums::Side,
};

impl BattleCore {
    pub(in crate::game::battle::core) fn refresh_block_state(&mut self) {
        if !matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) {
            self.block_state = BlockRuntimeState::default();
            return;
        }

        let mut blocker_ids = self
            .units
            .values()
            .filter(|unit| {
                unit.owner == Side::Player
                    && unit.is_combatant()
                    && !unit.is_dead()
                    && unit.block_capacity > 0
                    && unit.block_radius_units > 0.0
            })
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        blocker_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut enemy_ids = self
            .units
            .values()
            .filter(|unit| {
                unit.owner == Side::Opponent
                    && unit.is_combatant()
                    && !unit.is_dead()
                    && unit.blockable
            })
            .map(|unit| unit.instance_id)
            .collect::<Vec<_>>();
        enemy_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut remaining_capacity = blocker_ids
            .iter()
            .filter_map(|blocker_id| {
                self.units
                    .get(blocker_id)
                    .map(|unit| (*blocker_id, unit.block_capacity))
            })
            .collect::<HashMap<_, _>>();
        let mut block_state = BlockRuntimeState::default();

        for enemy_id in enemy_ids {
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

        for (blocker_id, enemy_ids) in &mut block_state.blocker_to_enemies {
            enemy_ids.sort_by(|a, b| {
                let distance_order = self
                    .block_distance_sq(*blocker_id, *a)
                    .unwrap_or(f32::MAX)
                    .total_cmp(&self.block_distance_sq(*blocker_id, *b).unwrap_or(f32::MAX));
                distance_order.then_with(|| a.as_bytes().cmp(b.as_bytes()))
            });
        }

        self.block_state = block_state;
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
            WinCondition::ProtectUnit { unit_ref } => Some(unit_ref),
            _ => None,
        };

        unit_ref
            .and_then(|unit_ref| self.scenario_runtime.unit_refs.get(unit_ref).copied())
            .or_else(|| {
                self.units
                    .values()
                    .filter(|unit| unit.role == BattleUnitRole::DefenseObject && !unit.is_dead())
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
        let point_id = match plan {
            EnemyMovementPlan::PathToPoint { point_id } => Some(point_id),
            EnemyMovementPlan::PathAlongPath { point_ids } => point_ids.last(),
            EnemyMovementPlan::AssaultPlayer => None,
        }?;
        self.scenario
            .tactical_plan
            .point(point_id)
            .map(|point| super::types::WorldVec2::from_tile_center(point.position))
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
