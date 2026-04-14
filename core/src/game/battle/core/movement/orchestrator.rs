use std::collections::{HashMap, HashSet};

use crate::{
    ecs::resources::Position,
    game::{
        battle::{core::BattleCore, ids::UnitInstanceId},
        enums::Side,
    },
};

use super::{
    boundary_target_units, ActionState, MovementState, PlannedContinuation, BLOCKED_RETRY_DELAY_MS,
    HOLD_RETRY_DELAY_MS, YIELD_RETRY_DELAY_MS,
};

#[derive(Debug, Clone, Copy)]
enum MovementIntentStateKind {
    Holding,
    Yielding,
    Blocked,
    Idle,
    WaitRepath,
}

impl MovementIntentStateKind {
    fn priority(self) -> u8 {
        match self {
            Self::Holding => 0,
            Self::Yielding => 1,
            Self::Blocked => 2,
            Self::Idle => 3,
            Self::WaitRepath => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MovementIntentContext {
    unit_id: UnitInstanceId,
    state_kind: MovementIntentStateKind,
    repath_counter: u32,
    start_pos: Position,
    owner: Side,
    base_uuid: uuid::Uuid,
    current_target: Option<UnitInstanceId>,
}

#[derive(Debug, Clone)]
struct MovementAdvanceCandidate {
    enemy_id: UnitInstanceId,
    enemy_pos: Position,
    destination: Position,
    path: Vec<Position>,
    first_step: Position,
    first_step_reserved_by_other: bool,
    second_step_blocked_by_friendly: bool,
}

#[derive(Debug, Clone)]
enum MovementDecision {
    Engage {
        unit_id: UnitInstanceId,
        target_id: UnitInstanceId,
    },
    Advance {
        unit_id: UnitInstanceId,
        enemy_id: UnitInstanceId,
        destination: Position,
        path: Vec<Position>,
        planned_continuation: Option<PlannedContinuation>,
        orchestrator_priority: u8,
        repath_counter: u32,
    },
    WaitRepath {
        unit_id: UnitInstanceId,
        until_ms: u64,
        repath_counter: u32,
    },
    Yield {
        unit_id: UnitInstanceId,
        retry_at_ms: u64,
        repath_counter: u32,
    },
    Blocked {
        unit_id: UnitInstanceId,
        retry_at_ms: u64,
        repath_counter: u32,
    },
    Hold {
        unit_id: UnitInstanceId,
        retry_at_ms: u64,
        repath_counter: u32,
    },
}

impl BattleCore {
    fn direct_continuous_melee_engage_target(
        &self,
        context: MovementIntentContext,
    ) -> Option<(UnitInstanceId, Position)> {
        let policy = self.basic_attack_range_policy(context.base_uuid);
        if !policy.use_continuous_range {
            return None;
        }

        let target_id = self
            .persisted_target_in_tile_range_for_continuous_melee(
                context.unit_id,
                context.current_target,
            )
            .or_else(|| {
                self.choose_attack_target_in_tile_range_for_continuous_melee(context.unit_id)
            })?;
        let target_pos = self.battlefield.position_of(target_id)?;

        if context.start_pos == target_pos
            || context.start_pos.chebyshev(&target_pos) > i32::from(policy.range_tiles)
        {
            return None;
        }

        Some((target_id, target_pos))
    }

    fn is_direct_continuous_melee_engage_approach(
        &self,
        unit_id: UnitInstanceId,
        enemy_id: UnitInstanceId,
        start_pos: Position,
        step: Position,
    ) -> bool {
        let Some(unit) = self.units.get(&unit_id) else {
            return false;
        };
        let policy = self.basic_attack_range_policy(unit.base_uuid);
        if !policy.use_continuous_range {
            return false;
        }

        step != start_pos
            && start_pos.chebyshev(&step) <= i32::from(policy.range_tiles)
            && self.battlefield.occupant(step).ok().flatten() == Some(enemy_id)
    }

    fn first_step_is_temporarily_blocked_by_friendly(
        &self,
        unit_id: UnitInstanceId,
        step: Position,
    ) -> bool {
        let Some(unit) = self.units.get(&unit_id) else {
            return false;
        };

        if self
            .battlefield
            .reservation_at(step)
            .is_some_and(|reservation| {
                self.units
                    .get(&reservation.unit)
                    .is_some_and(|blocker| !blocker.is_dead() && blocker.owner == unit.owner)
            })
        {
            return true;
        }

        self.battlefield
            .occupant(step)
            .ok()
            .flatten()
            .is_some_and(|occupant| {
                self.units
                    .get(&occupant)
                    .is_some_and(|blocker| !blocker.is_dead() && blocker.owner == unit.owner)
            })
    }

    pub fn compute_movement_intents(&mut self, now_ms: u64) {
        let contexts = self.collect_movement_intent_contexts(now_ms);
        let decisions = self.resolve_movement_intent_decisions(now_ms, &contexts);
        self.commit_movement_intent_decisions(now_ms, decisions);
    }

    fn collect_movement_intent_contexts(&mut self, now_ms: u64) -> Vec<MovementIntentContext> {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        let mut out = Vec::new();
        for unit_id in unit_ids {
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };

            let (state_kind, repath_counter) = match &unit.action_state {
                ActionState::Idle => (MovementIntentStateKind::Idle, 0),
                ActionState::Holding {
                    until_ms,
                    repath_counter,
                } => {
                    if *until_ms > now_ms {
                        continue;
                    }
                    (MovementIntentStateKind::Holding, *repath_counter)
                }
                ActionState::Yielding {
                    until_ms,
                    repath_counter,
                } => {
                    if *until_ms > now_ms {
                        continue;
                    }
                    (MovementIntentStateKind::Yielding, *repath_counter)
                }
                ActionState::Blocked {
                    until_ms,
                    repath_counter,
                } => {
                    if *until_ms > now_ms {
                        continue;
                    }
                    (MovementIntentStateKind::Blocked, *repath_counter)
                }
                ActionState::WaitRepath {
                    until_ms,
                    repath_counter,
                } => {
                    if *until_ms > now_ms {
                        continue;
                    }
                    (MovementIntentStateKind::WaitRepath, *repath_counter)
                }
                _ => continue,
            };

            if !unit.action_locks.can_move(now_ms) || unit.stats.move_speed_units_per_ms == 0 {
                continue;
            }

            let Some(start_pos) = self.battlefield.position_of(unit_id) else {
                continue;
            };

            out.push(MovementIntentContext {
                unit_id,
                state_kind,
                repath_counter,
                start_pos,
                owner: unit.owner,
                base_uuid: unit.base_uuid,
                current_target: unit.current_target,
            });
        }

        out.sort_by(|a, b| {
            a.state_kind
                .priority()
                .cmp(&b.state_kind.priority())
                .then_with(|| a.unit_id.as_bytes().cmp(b.unit_id.as_bytes()))
        });

        for context in &out {
            self.battlefield.cancel_reservation(context.unit_id);
        }

        out
    }

    fn resolve_movement_intent_decisions(
        &self,
        now_ms: u64,
        contexts: &[MovementIntentContext],
    ) -> Vec<MovementDecision> {
        let mut claimed_first_steps: HashSet<Position> = HashSet::new();
        let mut claimed_follow_up_steps: HashMap<(Side, Position), u8> = HashMap::new();
        let mut decisions = Vec::with_capacity(contexts.len());

        for context in contexts {
            if let Some(target_id) =
                self.persisted_target_in_range(context.unit_id, context.current_target)
            {
                decisions.push(MovementDecision::Engage {
                    unit_id: context.unit_id,
                    target_id,
                });
                continue;
            }

            if let Some(target_id) = self.choose_attack_target_in_range(context.unit_id) {
                decisions.push(MovementDecision::Engage {
                    unit_id: context.unit_id,
                    target_id,
                });
                continue;
            }

            if let Some((target_id, target_pos)) =
                self.direct_continuous_melee_engage_target(*context)
            {
                decisions.push(MovementDecision::Advance {
                    unit_id: context.unit_id,
                    enemy_id: target_id,
                    destination: target_pos,
                    path: vec![context.start_pos, target_pos],
                    planned_continuation: None,
                    orchestrator_priority: context.state_kind.priority(),
                    repath_counter: context.repath_counter,
                });
                continue;
            }

            let candidates = self.build_movement_advance_candidates(*context);
            let chosen = Self::choose_available_advance_candidate(
                context.owner,
                &candidates,
                &claimed_first_steps,
                &claimed_follow_up_steps,
            );

            if let Some(candidate) = chosen {
                if self.should_hold_position(*context, &candidates, &claimed_first_steps, candidate)
                {
                    decisions.push(MovementDecision::Hold {
                        unit_id: context.unit_id,
                        retry_at_ms: now_ms.saturating_add(HOLD_RETRY_DELAY_MS),
                        repath_counter: context.repath_counter,
                    });
                    continue;
                }
                claimed_first_steps.insert(candidate.first_step);
                let planned_continuation =
                    Self::follow_up_step(candidate).map(|follow_up_step| PlannedContinuation {
                        step: follow_up_step,
                        owner_priority: context.state_kind.priority(),
                        claim_priority: claimed_follow_up_steps
                            .get(&(context.owner, follow_up_step))
                            .copied(),
                        retry_budget: 1,
                    });
                if let Some(planned_continuation) = &planned_continuation {
                    claimed_follow_up_steps
                        .entry((context.owner, planned_continuation.step))
                        .or_insert(context.state_kind.priority());
                }
                decisions.push(MovementDecision::Advance {
                    unit_id: context.unit_id,
                    enemy_id: candidate.enemy_id,
                    destination: candidate.destination,
                    path: candidate.path.clone(),
                    planned_continuation,
                    orchestrator_priority: context.state_kind.priority(),
                    repath_counter: context.repath_counter,
                });
                continue;
            }

            if self.has_any_movement_candidate(*context) {
                decisions.push(MovementDecision::Yield {
                    unit_id: context.unit_id,
                    retry_at_ms: now_ms.saturating_add(YIELD_RETRY_DELAY_MS),
                    repath_counter: context.repath_counter,
                });
                continue;
            }

            if self.has_loose_attack_path(*context) {
                decisions.push(MovementDecision::Blocked {
                    unit_id: context.unit_id,
                    retry_at_ms: now_ms.saturating_add(BLOCKED_RETRY_DELAY_MS),
                    repath_counter: context.repath_counter,
                });
                continue;
            }

            decisions.push(MovementDecision::WaitRepath {
                unit_id: context.unit_id,
                until_ms: self.next_wait_repath_until_ms(
                    now_ms,
                    context.unit_id,
                    context.repath_counter.wrapping_add(1),
                ),
                repath_counter: context.repath_counter.wrapping_add(1),
            });
        }

        decisions
    }

    fn choose_available_advance_candidate<'a>(
        owner: Side,
        candidates: &'a [MovementAdvanceCandidate],
        claimed_first_steps: &HashSet<Position>,
        claimed_follow_up_steps: &HashMap<(Side, Position), u8>,
    ) -> Option<&'a MovementAdvanceCandidate> {
        let mut fallback: Option<&MovementAdvanceCandidate> = None;

        for candidate in candidates {
            if claimed_first_steps.contains(&candidate.first_step) {
                continue;
            }

            if fallback.is_none() {
                fallback = Some(candidate);
            }

            let follow_up_claimed = Self::follow_up_step(candidate).is_some_and(|follow_up_step| {
                claimed_follow_up_steps.contains_key(&(owner, follow_up_step))
            });

            if !follow_up_claimed {
                return Some(candidate);
            }
        }

        fallback
    }

    fn follow_up_step(candidate: &MovementAdvanceCandidate) -> Option<Position> {
        candidate.path.get(2).copied()
    }

    fn build_movement_advance_candidates(
        &self,
        context: MovementIntentContext,
    ) -> Vec<MovementAdvanceCandidate> {
        let battlefield = &self.battlefield;
        let bfs =
            match battlefield.bfs_map_8_for_side(context.start_pos, context.owner, |pos, tile| {
                if tile.occupant().is_some() {
                    return false;
                }
                if tile.reservation().is_some()
                    && battlefield.reservation_blocks_for(context.unit_id, pos)
                {
                    return false;
                }
                true
            }) {
                Ok(v) => v,
                Err(_) => return Vec::new(),
            };

        let mut enemies: Vec<(UnitInstanceId, Position)> = self
            .units
            .values()
            .filter_map(|u| {
                if u.is_dead() || u.owner == context.owner {
                    None
                } else {
                    let pos = self.battlefield.position_of(u.instance_id)?;
                    Some((u.instance_id, pos))
                }
            })
            .collect();
        enemies.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let plans = self.formulate_enemy_chase_plan(
            &bfs,
            context.owner,
            context.start_pos,
            enemies,
            self.basic_attack_range_tiles(context.base_uuid),
            |field, pos| {
                field
                    .idx(pos)
                    .ok()
                    .is_some_and(|idx| field.is_empty_tile(idx))
            },
        );

        let mut out = Vec::new();
        for plan in plans {
            for dest in plan.dest_candidates {
                let Some(path) = bfs.reconstruct_path_to(dest) else {
                    continue;
                };
                if path.len() < 2 {
                    continue;
                }
                out.push(MovementAdvanceCandidate {
                    enemy_id: plan.enemy_id,
                    enemy_pos: plan.enemy_pos,
                    destination: dest,
                    first_step: path[1],
                    first_step_reserved_by_other: self
                        .candidate_first_step_reserved_by_other(context.unit_id, &path),
                    second_step_blocked_by_friendly: self
                        .candidate_second_step_blocked_by_friendly(context.owner, &path),
                    path,
                });
            }
        }
        out.sort_by(|a, b| self.compare_advance_candidate(context, a, b));
        out
    }

    fn candidate_second_step_blocked_by_friendly(&self, owner: Side, path: &[Position]) -> bool {
        let Some(second_step) = path.get(2).copied() else {
            return false;
        };

        if self
            .battlefield
            .reservation_at(second_step)
            .is_some_and(|reservation| {
                self.units
                    .get(&reservation.unit)
                    .is_some_and(|unit| !unit.is_dead() && unit.owner == owner)
            })
        {
            return true;
        }

        self.battlefield
            .occupant(second_step)
            .ok()
            .flatten()
            .is_some_and(|occupant| {
                self.units
                    .get(&occupant)
                    .is_some_and(|unit| !unit.is_dead() && unit.owner == owner)
            })
    }

    fn candidate_first_step_reserved_by_other(
        &self,
        unit_id: UnitInstanceId,
        path: &[Position],
    ) -> bool {
        let Some(first_step) = path.get(1).copied() else {
            return false;
        };

        self.battlefield
            .reservation_at(first_step)
            .is_some_and(|reservation| reservation.unit != unit_id)
    }

    fn has_any_movement_candidate(&self, context: MovementIntentContext) -> bool {
        !self.build_movement_advance_candidates(context).is_empty()
    }

    fn should_hold_position(
        &self,
        context: MovementIntentContext,
        candidates: &[MovementAdvanceCandidate],
        claimed_first_steps: &HashSet<Position>,
        candidate: &MovementAdvanceCandidate,
    ) -> bool {
        let Some(locked_target) =
            self.persisted_target_if_alive(context.owner, context.current_target)
        else {
            return false;
        };
        if locked_target != candidate.enemy_id {
            return candidates.iter().any(|option| {
                option.enemy_id == locked_target
                    && Self::is_near_engage_candidate(option)
                    && !claimed_first_steps.contains(&option.first_step)
            });
        }

        false
    }

    fn has_loose_attack_path_to_enemy(
        &self,
        context: MovementIntentContext,
        enemy_id: UnitInstanceId,
    ) -> bool {
        let Some(enemy_pos) = self.battlefield.position_of(enemy_id) else {
            return false;
        };

        let loose_bfs = match self.battlefield.bfs_map_8_for_side(
            context.start_pos,
            context.owner,
            |_pos, _tile| true,
        ) {
            Ok(map) => map,
            Err(_) => return false,
        };

        let range_tiles = self.basic_attack_range_tiles(context.base_uuid) as i32;
        for y in (enemy_pos.y - range_tiles)..=(enemy_pos.y + range_tiles) {
            for x in (enemy_pos.x - range_tiles)..=(enemy_pos.x + range_tiles) {
                let pos = Position::new(x, y);
                if pos == enemy_pos
                    || !self.battlefield.in_bounds(pos)
                    || pos.chebyshev(&enemy_pos) > range_tiles
                {
                    continue;
                }
                if loose_bfs.distance_to(pos).is_some() {
                    return true;
                }
            }
        }

        false
    }

    fn has_loose_attack_path(&self, context: MovementIntentContext) -> bool {
        self.units.values().any(|unit| {
            if unit.is_dead() || unit.owner == context.owner {
                return false;
            }
            self.has_loose_attack_path_to_enemy(context, unit.instance_id)
        })
    }

    fn compare_advance_candidate(
        &self,
        context: MovementIntentContext,
        a: &MovementAdvanceCandidate,
        b: &MovementAdvanceCandidate,
    ) -> std::cmp::Ordering {
        let a_near_engage_locked =
            (context.current_target == Some(a.enemy_id) && Self::is_near_engage_candidate(a)) as u8;
        let b_near_engage_locked =
            (context.current_target == Some(b.enemy_id) && Self::is_near_engage_candidate(b)) as u8;
        let a_sticky_approach_locked = Self::preserves_locked_target_approach(context, a) as u8;
        let b_sticky_approach_locked = Self::preserves_locked_target_approach(context, b) as u8;
        let a_target_locked = (context.current_target == Some(a.enemy_id)) as u8;
        let b_target_locked = (context.current_target == Some(b.enemy_id)) as u8;

        b_near_engage_locked
            .cmp(&a_near_engage_locked)
            .then_with(|| b_sticky_approach_locked.cmp(&a_sticky_approach_locked))
            .then_with(|| b_target_locked.cmp(&a_target_locked))
            .then_with(|| {
                Self::enemy_distance_from_start(context.start_pos, a.enemy_pos).cmp(
                    &Self::enemy_distance_from_start(context.start_pos, b.enemy_pos),
                )
            })
            .then_with(|| self.compare_enemy_target_range_preference(a.enemy_id, b.enemy_id))
            .then_with(|| a.path.len().cmp(&b.path.len()))
            .then_with(|| {
                a.first_step_reserved_by_other
                    .cmp(&b.first_step_reserved_by_other)
            })
            .then_with(|| {
                a.second_step_blocked_by_friendly
                    .cmp(&b.second_step_blocked_by_friendly)
            })
            .then_with(|| {
                Self::distance_after_first_step(a.first_step, a.enemy_pos)
                    .cmp(&Self::distance_after_first_step(b.first_step, b.enemy_pos))
            })
            .then_with(|| {
                Self::lateral_shift(context.start_pos, a.first_step)
                    .cmp(&Self::lateral_shift(context.start_pos, b.first_step))
            })
            .then_with(|| {
                Self::forward_progress(context.owner, context.start_pos, b.first_step).cmp(
                    &Self::forward_progress(context.owner, context.start_pos, a.first_step),
                )
            })
            .then_with(|| {
                Self::enemy_axis_alignment(a.first_step, a.enemy_pos)
                    .cmp(&Self::enemy_axis_alignment(b.first_step, b.enemy_pos))
            })
            .then_with(|| {
                Self::compare_plan_preference(
                    context.owner,
                    context.start_pos,
                    a.enemy_pos,
                    b.enemy_pos,
                    a.destination,
                    b.destination,
                )
            })
            .then_with(|| a.enemy_id.as_bytes().cmp(b.enemy_id.as_bytes()))
            .then_with(|| a.destination.y.cmp(&b.destination.y))
            .then_with(|| a.destination.x.cmp(&b.destination.x))
    }

    fn forward_progress(owner: Side, start: Position, step: Position) -> i32 {
        match owner {
            Side::Player => start.y - step.y,
            Side::Opponent => step.y - start.y,
        }
    }

    fn enemy_distance_from_start(start: Position, enemy_pos: Position) -> i32 {
        start.chebyshev(&enemy_pos)
    }

    fn lateral_shift(start: Position, step: Position) -> i32 {
        (step.x - start.x).abs()
    }

    fn distance_after_first_step(step: Position, enemy_pos: Position) -> i32 {
        step.chebyshev(&enemy_pos)
    }

    fn enemy_axis_alignment(step: Position, enemy_pos: Position) -> i32 {
        (step.x - enemy_pos.x).abs()
    }

    fn is_near_engage_candidate(candidate: &MovementAdvanceCandidate) -> bool {
        candidate.path.len() <= 2
    }

    fn preserves_locked_target_approach(
        context: MovementIntentContext,
        candidate: &MovementAdvanceCandidate,
    ) -> bool {
        context.current_target == Some(candidate.enemy_id)
            && candidate.first_step.x == context.start_pos.x
            && candidate.destination.x == context.start_pos.x
    }

    fn commit_movement_intent_decisions(&mut self, now_ms: u64, decisions: Vec<MovementDecision>) {
        let mut started_moves = Vec::new();

        for decision in decisions {
            match decision {
                MovementDecision::Engage { unit_id, target_id } => {
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = Some(target_id);
                        unit.action_state = ActionState::Idle;
                    }
                }
                MovementDecision::Advance {
                    unit_id,
                    enemy_id,
                    destination,
                    path,
                    planned_continuation,
                    orchestrator_priority,
                    repath_counter,
                } => {
                    let Some(first_step) = path.get(1).copied() else {
                        let until_ms = self.next_wait_repath_until_ms(
                            now_ms,
                            unit_id,
                            repath_counter.wrapping_add(1),
                        );
                        self.enter_wait_repath(unit_id, until_ms, repath_counter.wrapping_add(1));
                        continue;
                    };

                    let Some(start_pos) = self.battlefield.position_of(unit_id) else {
                        self.battlefield.cancel_reservation(unit_id);
                        continue;
                    };

                    let direct_enemy_engage_approach = self
                        .is_direct_continuous_melee_engage_approach(
                            unit_id, enemy_id, start_pos, first_step,
                        );

                    if !direct_enemy_engage_approach
                        && self
                            .battlefield
                            .reserve(unit_id, first_step, now_ms)
                            .is_err()
                    {
                        if self.first_step_is_temporarily_blocked_by_friendly(unit_id, first_step) {
                            self.enter_blocked(
                                now_ms,
                                unit_id,
                                BLOCKED_RETRY_DELAY_MS,
                                repath_counter,
                            );
                            continue;
                        }
                        let until_ms = self.next_wait_repath_until_ms(
                            now_ms,
                            unit_id,
                            repath_counter.wrapping_add(1),
                        );
                        self.enter_wait_repath(unit_id, until_ms, repath_counter.wrapping_add(1));
                        continue;
                    }

                    let mut movement = MovementState::new_at(start_pos, now_ms);
                    movement.path = path;
                    movement.reserved_destination = Some(destination);
                    movement.planned_continuation = planned_continuation;
                    movement.repath_counter = repath_counter;
                    movement.orchestrator_priority = orchestrator_priority;
                    movement.step_from = start_pos;
                    movement.step_to = first_step;

                    let (target_x, target_y) =
                        boundary_target_units(movement.step_from, movement.step_to);
                    movement.target_x_units = target_x;
                    movement.target_y_units = target_y;
                    movement.step_started_at_ms = now_ms;

                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = Some(enemy_id);
                        unit.move_epoch = unit.move_epoch.wrapping_add(1);
                        unit.action_state = ActionState::Moving(movement);
                        started_moves.push(unit_id);
                    } else {
                        self.battlefield.cancel_reservation(unit_id);
                    }
                }
                MovementDecision::WaitRepath {
                    unit_id,
                    until_ms,
                    repath_counter,
                } => {
                    self.enter_wait_repath(unit_id, until_ms, repath_counter);
                }
                MovementDecision::Yield {
                    unit_id,
                    retry_at_ms,
                    repath_counter,
                } => {
                    self.enter_yield(
                        now_ms,
                        unit_id,
                        retry_at_ms.saturating_sub(now_ms),
                        repath_counter,
                    );
                }
                MovementDecision::Blocked {
                    unit_id,
                    retry_at_ms,
                    repath_counter,
                } => {
                    self.enter_blocked(
                        now_ms,
                        unit_id,
                        retry_at_ms.saturating_sub(now_ms),
                        repath_counter,
                    );
                }
                MovementDecision::Hold {
                    unit_id,
                    retry_at_ms,
                    repath_counter,
                } => {
                    self.enter_hold(
                        now_ms,
                        unit_id,
                        retry_at_ms.saturating_sub(now_ms),
                        repath_counter,
                    );
                }
            }
        }

        for unit_id in started_moves {
            if let Some(step_ends_at_ms) = self.schedule_current_move_step(unit_id, now_ms) {
                self.schedule_move_step_at(unit_id, step_ends_at_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase, event_pools::EventPhasePool,
        event_pools::EventPoolConfig, pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase, shop_data::ShopDatabase, skill_data::SkillDatabase,
        GameDataBase,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: std::collections::HashMap::new(),
        }
    }

    fn empty_game_data() -> Arc<GameDataBase> {
        let pool = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };
        let event_pools = EventPoolConfig {
            dawn: pool.clone(),
            noon: pool.clone(),
            dusk: pool.clone(),
            midnight: pool.clone(),
            white: pool,
        };

        Arc::new(GameDataBase::new(crate::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        }))
    }

    fn new_core() -> BattleCore {
        let deck = empty_deck();
        BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 123)
    }

    #[test]
    fn compare_advance_candidate_prefers_open_second_step_over_equal_detour() {
        let core = new_core();
        let enemy_id: UnitInstanceId = Uuid::from_u128(1).into();
        let context = MovementIntentContext {
            unit_id: Uuid::from_u128(10).into(),
            state_kind: MovementIntentStateKind::Idle,
            repath_counter: 0,
            start_pos: Position::new(2, 3),
            owner: Side::Player,
            base_uuid: Uuid::nil(),
            current_target: Some(enemy_id),
        };

        let blocked_left = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(1, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(1, 2),
                Position::new(1, 1),
            ],
            first_step: Position::new(1, 2),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: true,
        };
        let open_right = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(3, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(3, 2),
                Position::new(3, 1),
            ],
            first_step: Position::new(3, 2),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };

        assert_eq!(
            core.compare_advance_candidate(context, &blocked_left, &open_right),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            core.compare_advance_candidate(context, &open_right, &blocked_left),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn choose_available_advance_candidate_prefers_unclaimed_follow_up_step() {
        let enemy_id: UnitInstanceId = Uuid::from_u128(2).into();
        let blocked_follow_up = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(1, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(1, 2),
                Position::new(1, 1),
            ],
            first_step: Position::new(1, 2),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };
        let open_follow_up = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(3, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(3, 2),
                Position::new(3, 1),
            ],
            first_step: Position::new(3, 2),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };

        let claimed_first_steps = HashSet::new();
        let claimed_follow_up_steps = HashMap::from([((Side::Player, Position::new(1, 1)), 0_u8)]);

        let candidates = [blocked_follow_up.clone(), open_follow_up.clone()];
        let chosen = BattleCore::choose_available_advance_candidate(
            Side::Player,
            &candidates,
            &claimed_first_steps,
            &claimed_follow_up_steps,
        )
        .expect("expected an available candidate");

        assert_eq!(chosen.first_step, open_follow_up.first_step);

        let all_claimed_follow_ups = HashMap::from([
            ((Side::Player, Position::new(1, 1)), 0_u8),
            ((Side::Player, Position::new(3, 1)), 1_u8),
        ]);
        let fallback_candidates = [blocked_follow_up.clone(), open_follow_up];
        let fallback = BattleCore::choose_available_advance_candidate(
            Side::Player,
            &fallback_candidates,
            &claimed_first_steps,
            &all_claimed_follow_ups,
        )
        .expect("expected fallback candidate when all follow-up steps are claimed");

        assert_eq!(fallback.first_step, blocked_follow_up.first_step);
    }

    #[test]
    fn choose_available_advance_candidate_ignores_opposing_follow_up_claim() {
        let enemy_id: UnitInstanceId = Uuid::from_u128(20).into();
        let preferred = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(5, 1),
            destination: Position::new(5, 4),
            path: vec![
                Position::new(5, 6),
                Position::new(5, 5),
                Position::new(5, 4),
            ],
            first_step: Position::new(5, 5),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };
        let detour = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(5, 1),
            destination: Position::new(4, 4),
            path: vec![
                Position::new(5, 6),
                Position::new(5, 5),
                Position::new(4, 4),
            ],
            first_step: Position::new(5, 5),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };

        let claimed_first_steps = HashSet::new();
        let opposing_claims = HashMap::from([((Side::Opponent, Position::new(5, 4)), 0_u8)]);
        let candidates = [preferred.clone(), detour];

        let chosen = BattleCore::choose_available_advance_candidate(
            Side::Player,
            &candidates,
            &claimed_first_steps,
            &opposing_claims,
        )
        .expect("expected candidate despite opposing follow-up claim");

        assert_eq!(chosen.destination, preferred.destination);
    }

    #[test]
    fn compare_advance_candidate_prefers_unreserved_first_step_before_equal_detour() {
        let core = new_core();
        let enemy_id: UnitInstanceId = Uuid::from_u128(3).into();
        let context = MovementIntentContext {
            unit_id: Uuid::from_u128(11).into(),
            state_kind: MovementIntentStateKind::Idle,
            repath_counter: 0,
            start_pos: Position::new(2, 3),
            owner: Side::Player,
            base_uuid: Uuid::nil(),
            current_target: Some(enemy_id),
        };

        let reserved_first_step = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(1, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(1, 2),
                Position::new(1, 1),
            ],
            first_step: Position::new(1, 2),
            first_step_reserved_by_other: true,
            second_step_blocked_by_friendly: false,
        };
        let open_first_step = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(2, 0),
            destination: Position::new(3, 1),
            path: vec![
                Position::new(2, 3),
                Position::new(3, 2),
                Position::new(3, 1),
            ],
            first_step: Position::new(3, 2),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };

        assert_eq!(
            core.compare_advance_candidate(context, &reserved_first_step, &open_first_step),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            core.compare_advance_candidate(context, &open_first_step, &reserved_first_step),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn compare_advance_candidate_prefers_locked_sticky_approach_over_equal_lateral_finish() {
        let core = new_core();
        let enemy_id: UnitInstanceId = Uuid::from_u128(4).into();
        let context = MovementIntentContext {
            unit_id: Uuid::from_u128(12).into(),
            state_kind: MovementIntentStateKind::Idle,
            repath_counter: 0,
            start_pos: Position::new(1, 4),
            owner: Side::Player,
            base_uuid: Uuid::nil(),
            current_target: Some(enemy_id),
        };

        let straight_follow_up = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(0, 1),
            destination: Position::new(1, 2),
            path: vec![
                Position::new(1, 4),
                Position::new(1, 3),
                Position::new(1, 2),
            ],
            first_step: Position::new(1, 3),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };
        let lateral_finish = MovementAdvanceCandidate {
            enemy_id,
            enemy_pos: Position::new(0, 1),
            destination: Position::new(0, 2),
            path: vec![
                Position::new(1, 4),
                Position::new(1, 3),
                Position::new(0, 2),
            ],
            first_step: Position::new(1, 3),
            first_step_reserved_by_other: false,
            second_step_blocked_by_friendly: false,
        };

        assert_eq!(
            core.compare_advance_candidate(context, &straight_follow_up, &lateral_finish),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            core.compare_advance_candidate(context, &lateral_finish, &straight_follow_up),
            std::cmp::Ordering::Greater
        );
    }
}
