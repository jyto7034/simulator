use std::collections::HashMap;

use crate::{
    game::resources::Position,
    game::{
        battle::{
            core::{movement::types::ActiveMovementSegment, BattleCore},
            event_log::BattleLogEvent,
            ids::UnitInstanceId,
        },
        enums::Side,
    },
};

use super::{
    steering,
    types::{MovementGoal, UnitBody, WorldVec2},
    MovementSegmentEndKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum MovementTerrainPolicy {
    #[default]
    Ground,
    Airborne,
}

impl MovementTerrainPolicy {
    pub(super) fn applies_static_obstacles(self) -> bool {
        matches!(self, Self::Ground)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct MovementUnitInput {
    pub(super) unit_id: UnitInstanceId,
    pub(super) body: UnitBody,
    pub(super) goal: Option<MovementGoal>,
    pub(super) terrain_policy: MovementTerrainPolicy,
    pub(super) can_move: bool,
    pub(super) is_dead: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MovementStaticObstacle {
    pub(super) obstacle_id: u64,
    pub(super) center: WorldVec2,
    pub(super) half_extents: WorldVec2,
}

impl MovementStaticObstacle {
    fn tile(tile: Position) -> Self {
        let center = WorldVec2::from_tile_center(tile);
        Self {
            obstacle_id: ((tile.x as u64) << 32) ^ (tile.y as u32 as u64),
            center,
            half_extents: WorldVec2::new(0.5, 0.5),
        }
    }

    fn void_tile(tile: Position) -> Self {
        let mut obstacle = Self::tile(tile);
        obstacle.obstacle_id ^= 0x8000_0000_0000_0000;
        obstacle
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct MovementTickInput {
    pub(super) dt_ms: u64,
    pub(super) board_width_units: f32,
    pub(super) board_height_units: f32,
    pub(super) units: Vec<MovementUnitInput>,
    pub(super) static_obstacles: Vec<MovementStaticObstacle>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum MovementOutput {
    BodyMoved {
        unit_id: UnitInstanceId,
        from: WorldVec2,
        to: WorldVec2,
    },
    MovementStopped {
        unit_id: UnitInstanceId,
    },
    TargetReached {
        unit_id: UnitInstanceId,
        target_id: UnitInstanceId,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct MovementTickResult {
    pub(super) outputs: Vec<MovementOutput>,
}

pub(super) trait MovementEngine {
    fn tick(&mut self, input: MovementTickInput) -> MovementTickResult;
}

fn canonicalize_movement_units(units: &mut [MovementUnitInput]) {
    units.sort_by(|a, b| a.unit_id.as_bytes().cmp(b.unit_id.as_bytes()));
}

pub(super) fn clamp_to_board(
    position: WorldVec2,
    radius: f32,
    board_width: f32,
    board_height: f32,
) -> WorldVec2 {
    let min_x = radius.max(0.0);
    let min_y = radius.max(0.0);
    let max_x = (board_width - radius).max(min_x);
    let max_y = (board_height - radius).max(min_y);

    WorldVec2::new(
        position.x.clamp(min_x, max_x),
        position.y.clamp(min_y, max_y),
    )
}

pub(super) trait ContinuousMovementResolver {
    fn prepare_tick(
        &mut self,
        _units: &[MovementUnitInput],
        _static_obstacles: &[MovementStaticObstacle],
    ) {
    }

    fn corrected_position(
        &mut self,
        unit: &MovementUnitInput,
        desired_to: WorldVec2,
        board_width_units: f32,
        board_height_units: f32,
        static_obstacles: &[MovementStaticObstacle],
        dt_seconds: f32,
    ) -> WorldVec2;

    fn apply_position(&mut self, _unit_id: UnitInstanceId, _position: WorldVec2) {}
}

pub(super) fn run_continuous_movement_tick(
    resolver: &mut impl ContinuousMovementResolver,
    input: MovementTickInput,
) -> MovementTickResult {
    let MovementTickInput {
        dt_ms,
        board_width_units,
        board_height_units,
        mut units,
        static_obstacles,
    } = input;

    canonicalize_movement_units(&mut units);
    resolver.prepare_tick(&units, &static_obstacles);

    let mut outputs = Vec::with_capacity(units.len());
    let dt_seconds = super::types::FiniteBattleSeconds::from_millis(dt_ms).get();

    for unit in &units {
        if unit.is_dead {
            outputs.push(MovementOutput::MovementStopped {
                unit_id: unit.unit_id,
            });
            continue;
        }

        if !unit.can_move {
            resolver.apply_position(unit.unit_id, unit.body.position);
            push_body_moved_if_changed(
                &mut outputs,
                unit.unit_id,
                unit.body.position,
                unit.body.position,
            );
            outputs.push(MovementOutput::MovementStopped {
                unit_id: unit.unit_id,
            });
            continue;
        }

        if let Some(reached) = steering::reached_goal(unit, &units) {
            resolver.apply_position(unit.unit_id, unit.body.position);
            push_body_moved_if_changed(
                &mut outputs,
                unit.unit_id,
                unit.body.position,
                unit.body.position,
            );
            outputs.push(reached);
            continue;
        }

        let Some(target) = steering::goal_target_position(unit, &units) else {
            resolver.apply_position(unit.unit_id, unit.body.position);
            push_body_moved_if_changed(
                &mut outputs,
                unit.unit_id,
                unit.body.position,
                unit.body.position,
            );
            outputs.push(MovementOutput::MovementStopped {
                unit_id: unit.unit_id,
            });
            continue;
        };

        let displacement = steering::steered_displacement(unit, target, dt_seconds);
        let desired_to = clamp_to_board(
            unit.body.position + displacement,
            unit.body.radius,
            board_width_units,
            board_height_units,
        );
        let to = resolver.corrected_position(
            unit,
            desired_to,
            board_width_units,
            board_height_units,
            &static_obstacles,
            dt_seconds,
        );
        resolver.apply_position(unit.unit_id, to);

        if (to - unit.body.position).length_squared() <= f32::EPSILON {
            push_body_moved_if_changed(&mut outputs, unit.unit_id, unit.body.position, to);
            if unit.terrain_policy.applies_static_obstacles()
                && !static_obstacles.is_empty()
                && (desired_to - unit.body.position).length_squared() > f32::EPSILON
            {
                outputs.push(MovementOutput::MovementStopped {
                    unit_id: unit.unit_id,
                });
            }
            continue;
        }

        push_body_moved_if_changed(&mut outputs, unit.unit_id, unit.body.position, to);
    }

    MovementTickResult { outputs }
}

fn push_body_moved_if_changed(
    outputs: &mut Vec<MovementOutput>,
    unit_id: UnitInstanceId,
    from: WorldVec2,
    to: WorldVec2,
) {
    if (to - from).length_squared() <= f32::EPSILON {
        return;
    }

    outputs.push(MovementOutput::BodyMoved { unit_id, from, to });
}

impl BattleCore {
    fn record_continuous_movement_segment(
        &mut self,
        time_ms: u64,
        dt_ms: u64,
        unit_id: UnitInstanceId,
        from: WorldVec2,
        to: WorldVec2,
    ) {
        let ends_at_ms = time_ms.saturating_add(dt_ms);
        let target = to.quantized_milli();

        self.record_event_log(
            time_ms,
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id: unit_id,
                start: from.quantized_milli(),
                target,
                started_at_ms: time_ms,
                ends_at_ms,
                end_kind: MovementSegmentEndKind::Boundary,
            },
        );
        self.active_movement_segments.insert(
            unit_id,
            ActiveMovementSegment {
                start: from,
                target: to,
                started_at_ms: time_ms,
                ends_at_ms,
            },
        );
    }

    fn stop_continuous_movement_segment(&mut self, unit_id: UnitInstanceId) {
        self.active_movement_segments.remove(&unit_id);
    }

    pub(in crate::game::battle::core) fn run_continuous_attack_movement_tick(
        &mut self,
        now_ms: u64,
        dt_ms: u64,
    ) {
        self.refresh_block_state();
        let goals = self.build_continuous_attack_goals_from_current_block_state(now_ms);
        self.run_continuous_movement_tick(now_ms, dt_ms, &goals);
    }

    pub(super) fn build_continuous_movement_input_with_goals(
        &self,
        now_ms: u64,
        dt_ms: u64,
        goals: &HashMap<UnitInstanceId, MovementGoal>,
    ) -> MovementTickInput {
        let units = self
            .units
            .values()
            .map(|unit| {
                let unit_id = unit.instance_id;
                let is_blocked = self.blocked_by(unit_id).is_some();
                let hard_cc_blocked = self
                    .active_hard_cc_release_time_ms(unit_id, now_ms)
                    .is_some();
                MovementUnitInput {
                    unit_id,
                    body: unit.movement_body_view(),
                    goal: goals.get(&unit_id).copied(),
                    terrain_policy: if unit.is_airborne() {
                        MovementTerrainPolicy::Airborne
                    } else {
                        MovementTerrainPolicy::Ground
                    },
                    can_move: !self.channel_active(unit_id)
                        && !self.rooted(unit_id)
                        && unit.action_locks.can_move(now_ms)
                        && unit.can_move()
                        && !is_blocked
                        && !hard_cc_blocked,
                    is_dead: unit.is_dead(),
                }
            })
            .collect();

        MovementTickInput {
            dt_ms,
            board_width_units: f32::from(self.battlefield.width()),
            board_height_units: f32::from(self.battlefield.height()),
            units,
            static_obstacles: self.continuous_static_obstacles(),
        }
    }

    fn continuous_static_obstacles(&self) -> Vec<MovementStaticObstacle> {
        let mut obstacles = self
            .battlefield
            .static_obstacles()
            .into_iter()
            .map(MovementStaticObstacle::tile)
            .collect::<Vec<_>>();
        obstacles.extend(
            self.battlefield
                .void_tiles()
                .into_iter()
                .map(MovementStaticObstacle::void_tile),
        );
        obstacles
    }

    fn apply_continuous_movement_outputs(
        &mut self,
        time_ms: u64,
        dt_ms: u64,
        outputs: &[MovementOutput],
    ) {
        for output in outputs {
            match *output {
                MovementOutput::BodyMoved { unit_id, from, to } => {
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        let owner = unit.owner;
                        unit.body.position = to;
                        if owner == Side::Opponent {
                            if let Some(facing) =
                                crate::game::battle::tile_range::FacingDirection::from_delta(
                                    to.x - from.x,
                                    to.y - from.y,
                                )
                            {
                                unit.facing_direction = Some(facing);
                            }
                        }
                        self.record_continuous_movement_segment(time_ms, dt_ms, unit_id, from, to);
                    }
                }
                MovementOutput::TargetReached { unit_id, target_id } => {
                    self.stop_continuous_movement_segment(unit_id);
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = Some(target_id);
                    }
                }
                MovementOutput::MovementStopped { unit_id } => {
                    self.stop_continuous_movement_segment(unit_id);
                }
            }
        }
    }

    #[cfg(test)]
    pub(in crate::game::battle::core) fn run_test_movement_tick(
        &mut self,
        now_ms: u64,
        dt_ms: u64,
        goals: &HashMap<UnitInstanceId, MovementGoal>,
    ) {
        self.run_continuous_movement_tick(now_ms, dt_ms, goals);
    }

    pub(super) fn run_continuous_movement_tick(
        &mut self,
        now_ms: u64,
        dt_ms: u64,
        goals: &HashMap<UnitInstanceId, MovementGoal>,
    ) -> MovementTickResult {
        if std::sync::Arc::get_mut(&mut self.movement_backend).is_none() {
            let Ok(owned) =
                crate::authoring::clone_shape::controlled_clone(self.movement_backend.as_ref())
            else {
                return MovementTickResult::default();
            };
            self.movement_backend = std::sync::Arc::new(owned);
        }
        self.refresh_block_state();
        let input = self.build_continuous_movement_input_with_goals(now_ms, dt_ms, goals);
        let mut result = std::sync::Arc::get_mut(&mut self.movement_backend)
            .expect("movement backend has a unique writer")
            .tick(input);
        self.constrain_block_contacts(&mut result.outputs);
        self.apply_continuous_movement_outputs(now_ms, dt_ms, &result.outputs);
        self.refresh_block_state();
        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use uuid::Uuid;

    use crate::game::{
        battle::{
            core::{movement::ActionState, types::RuntimeUnit, BattleCore},
            scenario::BattleScenario,
        },
        data::{GameDataBase, GameDataBuilder},
        enums::Side,
        stats::UnitStats,
    };

    use super::*;
    use crate::game::battle::core::movement::types::{UnitBody, DEFAULT_UNIT_RADIUS};

    fn empty_game_data() -> Arc<GameDataBase> {
        GameDataBuilder::empty().build_arc()
    }

    fn new_core() -> BattleCore {
        BattleCore::new_from_scenario(BattleScenario::empty((4, 4)), empty_game_data(), 123)
    }

    fn runtime_unit(unit_id: UnitInstanceId, owner: Side, position: WorldVec2) -> RuntimeUnit {
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 1_000_000;

        RuntimeUnit {
            employee_trauma: None,
            corroded: false,
            fragment_source: None,
            basic_attack_speed_percent: 100,
            instance_id: unit_id,
            lifecycle: crate::game::battle::core::types::RuntimeUnitLifecycle::Active,
            spawn_order: u64::from(unit_id.as_bytes()[15]),
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: crate::game::battle::types::BattleUnitRole::Combatant,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            base_uuid: Uuid::nil(),
            source_identity: crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                base_uuid: Uuid::nil(),
            },
            equipment_slots: None,
            stats,
            incoming_damage_modifiers: Default::default(),
            basic_attack: Default::default(),
            skill_id: None,
            skill_activation_mode: crate::game::ability::SkillActivationMode::Auto,
            body: UnitBody::new_at(position, DEFAULT_UNIT_RADIUS, 1.0),
            tactical_anchor: Some(position),
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
            basic_attack_epoch: 0,
            basic_attack_windup_until_ms: None,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            resonance_use: None,
            next_resonance_pulse_ms: None,
            next_action_time: 0,
            pending_cast: false,
            pending_cast_cause: None,
            pending_skill_cast: None,
        }
    }

    #[test]
    fn battle_core_records_immutable_movement_segments_per_tick() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(12).into();
        core.units.insert(
            unit_id,
            runtime_unit(unit_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let mut goals = HashMap::new();
        goals.insert(
            unit_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(10.0, 1.0),
                stop_radius: 0.1,
            },
        );

        core.run_continuous_movement_tick(0, 50, &goals);
        let first_event = core
            .event_log
            .entries
            .iter()
            .find_map(|entry| match &entry.event {
                BattleLogEvent::MovementSegmentStarted {
                    unit_instance_id,
                    start,
                    target,
                    started_at_ms,
                    ends_at_ms,
                    ..
                } if *unit_instance_id == unit_id => {
                    Some((entry.seq, *start, *target, *started_at_ms, *ends_at_ms))
                }
                _ => None,
            })
            .expect("first movement segment should be recorded");

        core.run_continuous_movement_tick(50, 50, &goals);
        let segments = core
            .event_log
            .entries
            .iter()
            .filter_map(|entry| match &entry.event {
                BattleLogEvent::MovementSegmentStarted {
                    unit_instance_id,
                    start,
                    target,
                    started_at_ms,
                    ends_at_ms,
                    ..
                } if *unit_instance_id == unit_id => {
                    Some((entry.seq, *start, *target, *started_at_ms, *ends_at_ms))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0], first_event,
            "first movement event must remain immutable after later ticks"
        );
        assert_eq!(segments[0].3, 0);
        assert_eq!(segments[0].4, 50);
        assert_eq!(segments[1].3, 50);
        assert_eq!(segments[1].4, 100);
        assert_eq!(
            segments[0].2, segments[1].1,
            "adjacent live movement segments should connect target to next start"
        );
    }

    #[test]
    fn battle_core_does_not_record_movement_segment_without_body_motion() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(13).into();
        core.units.insert(
            unit_id,
            runtime_unit(unit_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let goals = HashMap::new();

        core.run_continuous_movement_tick(0, 50, &goals);

        assert!(!core.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::MovementSegmentStarted { unit_instance_id, .. }
                if unit_instance_id == unit_id
        )));
    }

    #[test]
    fn battle_core_default_continuous_tick_moves_with_explicit_goal() {
        let mut core = new_core();
        let unit_id: UnitInstanceId = Uuid::from_u128(11).into();
        core.units.insert(
            unit_id,
            runtime_unit(unit_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let mut goals = HashMap::new();
        goals.insert(
            unit_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(2.0, 1.0),
                stop_radius: 0.1,
            },
        );

        let result = core.run_continuous_movement_tick(0, 500, &goals);

        assert!(matches!(
            result.outputs.as_slice(),
            [MovementOutput::BodyMoved { unit_id: moved, .. }] if *moved == unit_id
        ));
        assert_eq!(
            core.unit_world_position(unit_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
    }

    #[test]
    fn battle_core_default_moving_locked_and_no_goal_observables_are_stable() {
        let moving_id: UnitInstanceId = Uuid::from_u128(30).into();
        let locked_id: UnitInstanceId = Uuid::from_u128(31).into();
        let idle_id: UnitInstanceId = Uuid::from_u128(32).into();
        let start = WorldVec2::new(1.0, 1.0);
        let mut goals = HashMap::new();
        goals.insert(
            moving_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(3.0, 1.0),
                stop_radius: 0.1,
            },
        );

        let mut moving = new_core();
        moving
            .units
            .insert(moving_id, runtime_unit(moving_id, Side::Player, start));
        let moved = moving.run_continuous_movement_tick(0, 500, &goals);

        assert!(matches!(
            moved.outputs.as_slice(),
            [MovementOutput::BodyMoved {
                unit_id,
                from,
                to,
                ..
            }] if *unit_id == moving_id
                && *from == start
                && *to == WorldVec2::new(1.5, 1.0)
        ));
        assert_eq!(
            moving.unit_world_position(moving_id),
            Some(WorldVec2::new(1.5, 1.0))
        );
        assert!(moving.event_log.entries.iter().any(|entry| matches!(
            entry.event,
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id,
                start: event_start,
                target,
                started_at_ms: 0,
                ends_at_ms: 500,
                ..
            } if unit_instance_id == moving_id
                && event_start == start.quantized_milli()
                && target == WorldVec2::new(1.5, 1.0).quantized_milli()
        )));

        let mut locked = new_core();
        let mut locked_unit = runtime_unit(locked_id, Side::Player, start);
        locked_unit.action_locks.movement_until_ms = 1_000;
        locked.units.insert(locked_id, locked_unit);
        goals.clear();
        goals.insert(
            locked_id,
            MovementGoal::MoveToPoint {
                point: WorldVec2::new(3.0, 1.0),
                stop_radius: 0.1,
            },
        );
        let stopped = locked.run_continuous_movement_tick(0, 500, &goals);

        assert!(matches!(
            stopped.outputs.as_slice(),
            [MovementOutput::MovementStopped { unit_id, .. }] if *unit_id == locked_id
        ));
        assert_eq!(locked.unit_world_position(locked_id), Some(start));
        assert!(locked.event_log.entries.is_empty());

        let mut idle = new_core();
        idle.units
            .insert(idle_id, runtime_unit(idle_id, Side::Player, start));
        let no_goal = idle.run_continuous_movement_tick(0, 500, &HashMap::new());

        assert!(matches!(
            no_goal.outputs.as_slice(),
            [MovementOutput::MovementStopped { unit_id, .. }] if *unit_id == idle_id
        ));
        assert_eq!(idle.unit_world_position(idle_id), Some(start));
        assert!(idle.event_log.entries.is_empty());
    }

    #[test]
    fn battle_core_default_movement_is_repeatable_across_unit_insertion_order() {
        let first_id: UnitInstanceId = Uuid::from_u128(20).into();
        let second_id: UnitInstanceId = Uuid::from_u128(21).into();
        let mut canonical = new_core();
        let mut reversed = new_core();

        canonical.units.insert(
            first_id,
            runtime_unit(first_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );
        canonical.units.insert(
            second_id,
            runtime_unit(second_id, Side::Player, WorldVec2::new(1.0, 2.0)),
        );
        reversed.units.insert(
            second_id,
            runtime_unit(second_id, Side::Player, WorldVec2::new(1.0, 2.0)),
        );
        reversed.units.insert(
            first_id,
            runtime_unit(first_id, Side::Player, WorldVec2::new(1.0, 1.0)),
        );

        let goals = HashMap::from([
            (
                first_id,
                MovementGoal::MoveToPoint {
                    point: WorldVec2::new(3.0, 1.0),
                    stop_radius: 0.1,
                },
            ),
            (
                second_id,
                MovementGoal::MoveToPoint {
                    point: WorldVec2::new(3.0, 2.0),
                    stop_radius: 0.1,
                },
            ),
        ]);

        let canonical_results = [
            canonical.run_continuous_movement_tick(0, 500, &goals),
            canonical.run_continuous_movement_tick(500, 500, &goals),
        ];
        let reversed_results = [
            reversed.run_continuous_movement_tick(0, 500, &goals),
            reversed.run_continuous_movement_tick(500, 500, &goals),
        ];

        assert_eq!(canonical_results, reversed_results);
        for unit_id in [first_id, second_id] {
            assert_eq!(
                canonical.unit_world_position(unit_id),
                reversed.unit_world_position(unit_id)
            );
        }
        assert_eq!(
            canonical.event_log.to_json_string().unwrap(),
            reversed.event_log.to_json_string().unwrap()
        );
    }

    #[test]
    fn battle_core_movement_input_projects_battlefield_static_obstacles() {
        let mut core = new_core();
        let obstacle = Position::new(2, 2);
        core.battlefield.add_static_obstacle(obstacle).unwrap();

        let input = core.build_continuous_movement_input_with_goals(0, 50, &HashMap::new());

        assert_eq!(
            input.static_obstacles,
            vec![MovementStaticObstacle::tile(obstacle)]
        );
    }

    #[test]
    fn battle_core_movement_input_projects_non_rectangular_void_tiles_as_static_obstacles() {
        let scenario = BattleScenario {
            battlefield: crate::game::battle::scenario::BattleFieldSpec {
            walkable_paths: Vec::new(),
                object_placement: None,
                width: 3,
                height: 3,
                valid_tiles: vec![
                    Position::new(1, 0),
                    Position::new(1, 1),
                    Position::new(1, 2),
                ],
                obstacles: vec![Position::new(1, 1)],
            },
            groups: Vec::new(),
            events: Vec::new(),
            terminal_outcomes: BattleScenario::fixture_required_group_outcomes(),
            boss_runtime: None,
            tactical_plan: crate::game::battle::scenario::TacticalPlan::default(),
        };
        let mut core = BattleCore::new_from_scenario(scenario, empty_game_data(), 123);
        core.battlefield
            .add_static_obstacle(Position::new(1, 1))
            .unwrap();

        let input = core.build_continuous_movement_input_with_goals(0, 50, &HashMap::new());

        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::tile(Position::new(1, 1))));
        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(0, 0))));
        assert!(input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(2, 2))));
        assert!(!input
            .static_obstacles
            .contains(&MovementStaticObstacle::void_tile(Position::new(1, 0))));
    }
}
