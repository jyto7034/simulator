use std::collections::HashMap;

use crate::game::{
    battle::{
        core::BattleCore,
        ids::UnitInstanceId,
        scenario::{EnemyMovementPlan, PlayerMovementPlan},
    },
    enums::Side,
    resources::Position,
};

use super::types::{MovementGoal, WorldVec2};

impl BattleCore {
    pub fn build_continuous_attack_goals(&mut self) -> HashMap<UnitInstanceId, MovementGoal> {
        self.refresh_block_state();
        self.build_continuous_attack_goals_from_current_block_state()
    }

    pub(in crate::game::battle::core) fn build_continuous_attack_goals_from_current_block_state(
        &mut self,
    ) -> HashMap<UnitInstanceId, MovementGoal> {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut goals = HashMap::new();
        for unit_id in unit_ids {
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };
            if unit.is_dead() || !unit.can_move() {
                continue;
            }
            if self.blocked_by(unit_id).is_some() {
                continue;
            }

            let goal = match unit.owner {
                Side::Player => match self.scenario.tactical_plan.player_plan {
                    PlayerMovementPlan::FixedDefense => self.fixed_defense_goal(unit_id),
                },
                Side::Opponent => match self.enemy_movement_plan_for_unit(unit_id) {
                    EnemyMovementPlan::PathAlongCells { cells } => {
                        self.path_along_cells_goal(unit_id, &cells)
                    }
                },
            };

            if let Some(goal) = goal {
                goals.insert(unit_id, goal);
            }
        }

        goals
    }

    fn enemy_movement_plan_for_unit(&self, unit_id: UnitInstanceId) -> EnemyMovementPlan {
        self.units
            .get(&unit_id)
            .and_then(|unit| unit.enemy_movement_plan.clone())
            .unwrap_or_else(|| self.scenario.tactical_plan.enemy_plan.clone())
    }

    fn fixed_defense_goal(&self, unit_id: UnitInstanceId) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        let target_id = self
            .first_blocked_enemy(unit_id)
            .or_else(|| self.closest_enemy_in_attack_range(unit_id))?;
        Some(MovementGoal::AttackUnit {
            target_id,
            desired_range: unit.basic_attack.range_units.max(0.0),
            approach_point: None,
        })
    }

    fn path_along_cells_goal(
        &self,
        unit_id: UnitInstanceId,
        cells: &[Position],
    ) -> Option<MovementGoal> {
        let unit = self.units.get(&unit_id)?;
        if unit.is_airborne() {
            if let Some(target_id) = self.airborne_enemy_basic_attack_target_in_range(unit_id) {
                return Some(MovementGoal::AttackUnit {
                    target_id,
                    desired_range: unit.basic_attack.range_units.max(0.0),
                    approach_point: None,
                });
            }
        }

        if let Some(target_id) = self.fixed_defense_route_end_target_for_enemy(unit_id) {
            return Some(MovementGoal::AttackUnit {
                target_id,
                desired_range: unit.basic_attack.range_units.max(0.0),
                approach_point: None,
            });
        }

        if !self.is_fixed_defense_route_enemy(unit_id) {
            if let Some(target_id) = self.closest_enemy_in_attack_range(unit_id) {
                return Some(MovementGoal::AttackUnit {
                    target_id,
                    desired_range: unit.basic_attack.range_units.max(0.0),
                    approach_point: None,
                });
            }
        }

        let target = self.active_cell_path_target(unit.body.position, cells, 0.25)?;
        Some(MovementGoal::MoveToPoint {
            point: target,
            stop_radius: 0.1,
        })
    }

    fn active_cell_path_target(
        &self,
        current: WorldVec2,
        cells: &[Position],
        reach_radius: f32,
    ) -> Option<WorldVec2> {
        let mut last_valid = None;

        for cell in cells {
            let target = WorldVec2::from_tile_center(*cell);
            last_valid = Some(target);
            if current.distance(target) > reach_radius {
                return Some(target);
            }
        }

        last_valid
    }

    fn closest_enemy_in_attack_range(&self, unit_id: UnitInstanceId) -> Option<UnitInstanceId> {
        let unit = self.units.get(&unit_id)?;
        let desired_range = unit.basic_attack.range_units.max(0.0);
        self.units
            .values()
            .filter(|candidate| {
                !candidate.is_dead()
                    && candidate.owner != unit.owner
                    && unit.body.can_reach(&candidate.body, desired_range)
            })
            .min_by(|a, b| {
                unit.body
                    .position
                    .distance_squared(a.body.position)
                    .total_cmp(&unit.body.position.distance_squared(b.body.position))
                    .then_with(|| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()))
            })
            .map(|target| target.instance_id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use super::*;
    use crate::game::{
        battle::{
            core::{
                movement::{types::UnitBody, ActionState},
                RuntimeUnit,
            },
            ids::UnitInstanceId,
            scenario::{BattleScenario, EnemyMovementPlan, PlayerMovementPlan, TacticalPlan},
            types::MobilityKind,
        },
        data::{abnormality_data::BasicAttackDef, GameDataBase, GameDataBuilder},
        resources::Position,
        stats::UnitStats,
    };

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn core_with_player_plan(player_plan: PlayerMovementPlan) -> BattleCore {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            player_plan,
            ..TacticalPlan::default()
        };
        BattleCore::new_from_scenario(scenario, empty_game_data(), 123)
    }

    fn fixed_defense_core_with_enemy_exit() -> BattleCore {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            player_plan: PlayerMovementPlan::FixedDefense,
            enemy_plan: EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(7, 1)],
            },
            ..TacticalPlan::default()
        };
        BattleCore::new_from_scenario(scenario, empty_game_data(), 123)
    }

    fn runtime_unit(
        id: u128,
        owner: Side,
        position: WorldVec2,
        anchor: Option<WorldVec2>,
    ) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 1_000_000;
        let mut basic_attack = BasicAttackDef::default();
        basic_attack.range_units = 0.5;

        RuntimeUnit {
            instance_id: UnitInstanceId::from(Uuid::from_u128(id)),
            spawn_order: id as u64,
            source_owned_uuid: Uuid::from_u128(id),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            base_uuid: Uuid::nil(),
            stats,
            incoming_damage_modifiers: Default::default(),
            basic_attack,
            skill_id: None,
            skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
            body: UnitBody::new_at(position, 0.35, 1.0),
            tactical_anchor: anchor,
            enemy_movement_plan: None,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: Default::default(),
            target_traits: Vec::new(),
            facing_direction: None,
            move_epoch: 0,
            action_state: ActionState::Idle,
            action_locks: Default::default(),
            current_target: None,
            next_basic_attack_ms: 0,
            pending_basic_attack: false,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    fn blocking_player(id: u128, position: WorldVec2, capacity: u32) -> RuntimeUnit {
        let mut unit = runtime_unit(id, Side::Player, position, Some(position));
        unit.block_capacity = capacity;
        unit.block_radius_units = 1.0;
        unit.blockable = false;
        unit.basic_attack.range_units = 1.0;
        unit
    }

    fn blockable_enemy(id: u128, position: WorldVec2) -> RuntimeUnit {
        let mut unit = runtime_unit(id, Side::Opponent, position, None);
        unit.blockable = true;
        unit.basic_attack.range_units = 1.0;
        unit
    }

    fn defense_object(id: u128, position: WorldVec2) -> RuntimeUnit {
        let mut unit = runtime_unit(id, Side::Player, position, Some(position));
        unit.role = crate::game::battle::types::BattleUnitRole::DefenseObject;
        unit.stats.move_speed_units_per_ms = 0;
        unit.basic_attack.range_units = 0.0;
        unit.blockable = false;
        unit
    }

    fn airborne_enemy(id: u128, position: WorldVec2) -> RuntimeUnit {
        let mut unit = runtime_unit(id, Side::Opponent, position, None);
        unit.mobility_kind = MobilityKind::Airborne;
        unit.blockable = false;
        unit.basic_attack.range_units = 1.0;
        unit
    }

    #[test]
    fn fixed_defense_attacks_enemy_in_range_without_approach() {
        let mut core = core_with_player_plan(PlayerMovementPlan::FixedDefense);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(1.4, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&player_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: None,
                ..
            }) if *target_id == enemy_id
        ));
    }

    #[test]
    fn fixed_defense_does_not_move_toward_enemy_outside_range() {
        let mut core = core_with_player_plan(PlayerMovementPlan::FixedDefense);
        let player_id = UnitInstanceId::from(Uuid::from_u128(1));
        core.units.insert(
            player_id,
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Opponent, WorldVec2::new(6.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&player_id),
            "fixed defense must preserve deployment instead of chasing"
        );
    }

    #[test]
    fn airborne_enemy_is_not_blocked_even_inside_block_radius() {
        let mut core = core_with_player_plan(PlayerMovementPlan::FixedDefense);
        let blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units
            .insert(blocker_id, blocking_player(1, WorldVec2::new(1.0, 1.0), 2));
        let mut enemy = airborne_enemy(2, WorldVec2::new(1.2, 1.0));
        enemy.blockable = true;
        core.units.insert(enemy_id, enemy);

        core.refresh_block_state();

        assert_eq!(core.blocked_by(enemy_id), None);
        assert_eq!(core.first_blocked_enemy(blocker_id), None);
    }

    #[test]
    fn single_target_basic_attack_requires_air_capable_for_airborne_target() {
        let mut core = core_with_player_plan(PlayerMovementPlan::FixedDefense);
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let target_id = UnitInstanceId::from(Uuid::from_u128(2));
        let mut attacker = runtime_unit(1, Side::Opponent, WorldVec2::new(1.0, 1.0), None);
        attacker.basic_attack.range_units = 2.0;
        let mut target = runtime_unit(2, Side::Player, WorldVec2::new(1.5, 1.0), None);
        target.mobility_kind = MobilityKind::Airborne;
        core.units.insert(attacker_id, attacker);
        core.units.insert(target_id, target);

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);

        core.units
            .get_mut(&attacker_id)
            .unwrap()
            .basic_attack
            .air_capable = true;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
    }

    #[test]
    fn airborne_enemy_prioritizes_protected_target_in_attack_range() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(1));
        let protected_id = UnitInstanceId::from(Uuid::from_u128(2));
        let closer_combatant_id = UnitInstanceId::from(Uuid::from_u128(3));
        let mut enemy = airborne_enemy(1, WorldVec2::new(1.0, 1.0));
        enemy.basic_attack.range_units = 3.0;
        core.units.insert(enemy_id, enemy);
        core.units
            .insert(protected_id, defense_object(2, WorldVec2::new(2.5, 1.0)));
        core.units.insert(
            closer_combatant_id,
            runtime_unit(3, Side::Player, WorldVec2::new(1.4, 1.0), None),
        );

        assert_eq!(
            core.choose_attack_target_in_range(enemy_id),
            Some(protected_id)
        );
    }

    #[test]
    fn airborne_enemy_targets_nearest_player_combatant_when_protected_target_out_of_range() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(1));
        let protected_id = UnitInstanceId::from(Uuid::from_u128(2));
        let near_id = UnitInstanceId::from(Uuid::from_u128(3));
        let far_id = UnitInstanceId::from(Uuid::from_u128(4));
        let mut enemy = airborne_enemy(1, WorldVec2::new(1.0, 1.0));
        enemy.basic_attack.range_units = 1.0;
        core.units.insert(enemy_id, enemy);
        core.units
            .insert(protected_id, defense_object(2, WorldVec2::new(5.0, 1.0)));
        core.units.insert(
            near_id,
            runtime_unit(3, Side::Player, WorldVec2::new(1.4, 1.0), None),
        );
        core.units.insert(
            far_id,
            runtime_unit(4, Side::Player, WorldVec2::new(1.8, 1.0), None),
        );

        assert_eq!(core.choose_attack_target_in_range(enemy_id), Some(near_id));
    }

    #[test]
    fn airborne_route_enemy_attacks_in_range_before_continuing_route() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(1));
        let target_id = UnitInstanceId::from(Uuid::from_u128(2));
        let mut enemy = airborne_enemy(1, WorldVec2::new(1.0, 1.0));
        enemy.basic_attack.range_units = 1.0;
        core.units.insert(enemy_id, enemy);
        core.units.insert(
            target_id,
            runtime_unit(2, Side::Player, WorldVec2::new(1.4, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::AttackUnit {
                target_id: actual_target,
                approach_point: None,
                ..
            }) if *actual_target == target_id
        ));
    }

    #[test]
    fn airborne_route_enemy_continues_route_when_no_target_is_in_range() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut enemy = airborne_enemy(1, WorldVec2::new(1.0, 1.0));
        enemy.basic_attack.range_units = 0.5;
        core.units.insert(enemy_id, enemy);
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(2)),
            runtime_unit(2, Side::Player, WorldVec2::new(3.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { .. })
        ));
    }

    #[test]
    fn enemy_path_along_cells_skips_reached_waypoints_and_moves_to_next() {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(2, 1), Position::new(6, 1)],
            },
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(1)),
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 3.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(2.5, 1.5), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(6, 1))
        ));
    }

    #[test]
    fn enemy_empty_path_does_not_fall_back_to_chasing_far_player() {
        let mut scenario = BattleScenario::empty((8, 4));
        scenario.tactical_plan = TacticalPlan {
            enemy_plan: EnemyMovementPlan::PathAlongCells { cells: Vec::new() },
            ..TacticalPlan::default()
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            UnitInstanceId::from(Uuid::from_u128(1)),
            runtime_unit(1, Side::Player, WorldVec2::new(1.0, 1.0), None),
        );
        core.units.insert(
            enemy_id,
            runtime_unit(2, Side::Opponent, WorldVec2::new(7.0, 1.0), None),
        );

        let goals = core.build_continuous_attack_goals();

        assert!(
            !goals.contains_key(&enemy_id),
            "empty enemy path should not become unlimited free engage"
        );
    }

    #[test]
    fn fixed_defense_blocks_near_blockable_enemy_and_prioritizes_attack() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units
            .insert(blocker_id, blocking_player(1, WorldVec2::new(1.0, 1.0), 1));
        core.units
            .insert(enemy_id, blockable_enemy(2, WorldVec2::new(1.5, 1.0)));

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&blocker_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: None,
                ..
            }) if *target_id == enemy_id
        ));
        assert!(
            !goals.contains_key(&enemy_id),
            "blocked enemy should not receive a movement goal"
        );
        let movement_input = core.build_continuous_movement_input_with_goals(0, 50, &goals);
        let blocked_enemy_input = movement_input
            .units
            .iter()
            .find(|unit| unit.unit_id == enemy_id)
            .expect("blocked enemy should be part of movement input");
        assert!(
            !blocked_enemy_input.can_move,
            "blocked enemy should be movement-locked for the tick"
        );
        assert_eq!(
            core.units.get(&blocker_id).unwrap().current_target,
            Some(enemy_id)
        );
        assert_eq!(
            core.units.get(&enemy_id).unwrap().current_target,
            Some(blocker_id)
        );
    }

    #[test]
    fn fixed_defense_block_capacity_prioritizes_route_progress_and_allows_excess_to_pass() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let first_enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let second_enemy_id = UnitInstanceId::from(Uuid::from_u128(3));
        let third_enemy_id = UnitInstanceId::from(Uuid::from_u128(4));
        let fourth_enemy_id = UnitInstanceId::from(Uuid::from_u128(5));
        core.units
            .insert(blocker_id, blocking_player(1, WorldVec2::new(1.5, 1.5), 2));
        for (id, unit_id, x) in [
            (2, first_enemy_id, 1.55),
            (3, second_enemy_id, 1.65),
            (4, third_enemy_id, 1.75),
            (5, fourth_enemy_id, 1.85),
        ] {
            let mut enemy = blockable_enemy(id, WorldVec2::new(x, 1.5));
            enemy.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
                cells: vec![Position::new(1, 1), Position::new(7, 1)],
            });
            core.units.insert(unit_id, enemy);
        }

        let goals = core.build_continuous_attack_goals();

        assert!(!goals.contains_key(&fourth_enemy_id));
        assert!(!goals.contains_key(&third_enemy_id));
        assert!(matches!(
            goals.get(&second_enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
        assert!(matches!(
            goals.get(&first_enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
        assert_eq!(
            core.units.get(&blocker_id).unwrap().current_target,
            Some(fourth_enemy_id)
        );
    }

    #[test]
    fn fixed_defense_block_tie_uses_lower_blocker_id() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let lower_blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let higher_blocker_id = UnitInstanceId::from(Uuid::from_u128(3));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units.insert(
            lower_blocker_id,
            blocking_player(1, WorldVec2::new(1.0, 1.0), 1),
        );
        core.units.insert(
            higher_blocker_id,
            blocking_player(3, WorldVec2::new(1.0, 2.0), 1),
        );
        core.units
            .insert(enemy_id, blockable_enemy(2, WorldVec2::new(1.0, 1.5)));

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&lower_blocker_id),
            Some(MovementGoal::AttackUnit { target_id, .. }) if *target_id == enemy_id
        ));
        assert_eq!(
            core.units.get(&enemy_id).unwrap().current_target,
            Some(lower_blocker_id)
        );
    }

    #[test]
    fn fixed_defense_block_tie_uses_enemy_spawn_order_before_unit_id() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let early_enemy_id = UnitInstanceId::from(Uuid::from_u128(9));
        let late_enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units
            .insert(blocker_id, blocking_player(1, WorldVec2::new(1.5, 1.5), 1));

        let mut early_enemy = blockable_enemy(9, WorldVec2::new(1.6, 1.5));
        early_enemy.spawn_order = 10;
        early_enemy.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
            cells: vec![Position::new(1, 1), Position::new(7, 1)],
        });
        let mut late_enemy = blockable_enemy(2, WorldVec2::new(1.6, 1.5));
        late_enemy.spawn_order = 20;
        late_enemy.enemy_movement_plan = early_enemy.enemy_movement_plan.clone();
        core.units.insert(early_enemy_id, early_enemy);
        core.units.insert(late_enemy_id, late_enemy);

        let goals = core.build_continuous_attack_goals();

        assert!(!goals.contains_key(&early_enemy_id));
        assert!(matches!(
            goals.get(&late_enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
        assert_eq!(
            core.units.get(&blocker_id).unwrap().current_target,
            Some(early_enemy_id)
        );
    }

    #[test]
    fn fixed_defense_releases_block_when_enemy_leaves_radius() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let blocker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        core.units
            .insert(blocker_id, blocking_player(1, WorldVec2::new(1.0, 1.0), 1));
        core.units
            .insert(enemy_id, blockable_enemy(2, WorldVec2::new(1.5, 1.0)));
        let first_goals = core.build_continuous_attack_goals();
        assert!(!first_goals.contains_key(&enemy_id));

        core.units
            .get_mut(&enemy_id)
            .unwrap()
            .set_world_position(WorldVec2::new(4.0, 1.0));
        let second_goals = core.build_continuous_attack_goals();

        assert!(matches!(
            second_goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
    }

    #[test]
    fn fixed_defense_route_enemy_targets_defense_object_at_route_end() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let object_id = UnitInstanceId::from(Uuid::from_u128(9));
        let endpoint = WorldVec2::from_tile_center(Position::new(7, 1));
        core.units.insert(enemy_id, blockable_enemy(2, endpoint));
        core.units.insert(object_id, defense_object(9, endpoint));

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: None,
                ..
            }) if *target_id == object_id
        ));
        assert_eq!(
            core.select_basic_attack_target(enemy_id, None, None),
            Some(object_id)
        );
    }

    #[test]
    fn fixed_defense_route_enemy_uses_unit_route_override_for_endpoint_targeting() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let object_id = UnitInstanceId::from(Uuid::from_u128(9));
        let unit_endpoint = WorldVec2::from_tile_center(Position::new(2, 2));

        let mut enemy = blockable_enemy(2, unit_endpoint);
        enemy.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
            cells: vec![Position::new(2, 2)],
        });
        core.units.insert(enemy_id, enemy);
        core.units
            .insert(object_id, defense_object(9, unit_endpoint));

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::AttackUnit {
                target_id,
                approach_point: None,
                ..
            }) if *target_id == object_id
        ));
        assert_eq!(
            core.select_basic_attack_target(enemy_id, None, None),
            Some(object_id)
        );
    }

    #[test]
    fn fixed_defense_route_enemy_before_endpoint_ignores_incidental_targets() {
        let mut core = fixed_defense_core_with_enemy_exit();
        let bystander_id = UnitInstanceId::from(Uuid::from_u128(1));
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(2));
        let mut bystander = runtime_unit(1, Side::Player, WorldVec2::new(1.4, 1.0), None);
        bystander.block_capacity = 0;
        bystander.block_radius_units = 0.0;
        let enemy = blockable_enemy(2, WorldVec2::new(1.0, 1.0));
        core.units.insert(bystander_id, bystander);
        core.units.insert(enemy_id, enemy);

        let goals = core.build_continuous_attack_goals();

        assert!(matches!(
            goals.get(&enemy_id),
            Some(MovementGoal::MoveToPoint { point, .. })
                if *point == WorldVec2::from_tile_center(Position::new(7, 1))
        ));
        assert_eq!(core.select_basic_attack_target(enemy_id, None, None), None);
    }
}
