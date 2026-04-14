use std::collections::{HashMap, HashSet};

use crate::{
    ecs::resources::Position,
    game::battle::{
        core::{
            movement::{
                boundary_target_units, tile_center_units, ActionState, MovementSegmentEndKind,
                PlannedContinuation, BLOCKED_RETRY_DELAY_MS, YIELD_RETRY_DELAY_MS,
            },
            BattleCore,
        },
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{MovementStopReason, TimelineEvent},
    },
};

#[derive(Debug, Clone, Copy)]
struct SampledMotionSegment {
    pos_x_units: i64,
    pos_y_units: i64,
    vel_x_units_per_ms: i64,
    vel_y_units_per_ms: i64,
    moving_until_ms: Option<u64>,
}

enum ReadyMoveAttempt {
    Completed,
    Deferred,
    Moved(UnitInstanceId),
}

#[derive(Debug, Clone, Copy)]
struct DeferredContinuationReservation {
    unit_id: UnitInstanceId,
    next_step: Position,
    planned_continuation: PlannedContinuation,
}

enum NextStepReservationDecision {
    Reserved,
    DeferContinuationRetry,
    Yield,
    Blocked,
    WaitRepath,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MovementBlockerKind {
    DeferToHigherPriorityDueMover,
    YieldToDueMover,
    FriendlyStatic,
    HardFailure,
}

impl BattleCore {
    fn is_temporarily_blocked_by_friendly_unit(
        &self,
        unit_id: UnitInstanceId,
        blocker_id: UnitInstanceId,
    ) -> bool {
        let Some(unit) = self.units.get(&unit_id) else {
            return false;
        };
        let Some(blocker) = self.units.get(&blocker_id) else {
            return false;
        };
        !blocker.is_dead() && unit.owner == blocker.owner
    }

    fn classify_movement_blocker(
        &self,
        unit_id: UnitInstanceId,
        blocker_id: UnitInstanceId,
        priority: u8,
        due_set: &HashSet<UnitInstanceId>,
        priority_by_unit: &HashMap<UnitInstanceId, u8>,
    ) -> MovementBlockerKind {
        if due_set.contains(&blocker_id)
            && priority_by_unit
                .get(&blocker_id)
                .is_some_and(|blocker_priority| *blocker_priority > priority)
        {
            return MovementBlockerKind::DeferToHigherPriorityDueMover;
        }
        if due_set.contains(&blocker_id) {
            return MovementBlockerKind::YieldToDueMover;
        }
        if self.is_temporarily_blocked_by_friendly_unit(unit_id, blocker_id) {
            return MovementBlockerKind::FriendlyStatic;
        }
        MovementBlockerKind::HardFailure
    }

    pub(in crate::game::battle::core) fn interrupt_movement(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        reason: MovementStopReason,
        until_ms: Option<u64>,
        next_state: ActionState,
    ) -> bool {
        let movement_kind = match self.units.get(&unit_id).map(|unit| &unit.action_state) {
            Some(ActionState::Moving(_)) => 1,
            Some(ActionState::Holding { .. })
            | Some(ActionState::Yielding { .. })
            | Some(ActionState::Blocked { .. })
            | Some(ActionState::WaitRepath { .. }) => 2,
            _ => 0,
        };

        if movement_kind == 0 {
            return false;
        }

        if movement_kind == 1 {
            self.update_move_position_to(unit_id, now_ms);
        }

        self.battlefield.cancel_reservation(unit_id);
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = next_state;
        }

        self.record_movement_stopped(now_ms, unit_id, reason, until_ms);
        true
    }

    pub(in crate::game::battle::core) fn record_movement_stopped(
        &mut self,
        time_ms: u64,
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        until_ms: Option<u64>,
    ) {
        let Some(position) = self.battlefield.position_of(unit_instance_id) else {
            return;
        };
        let Some((pos_x_units, pos_y_units)) = self
            .units
            .get(&unit_instance_id)
            .map(|u| (u.pos_x_units, u.pos_y_units))
        else {
            return;
        };
        self.record_timeline(
            time_ms,
            TimelineEvent::MovementStopped {
                unit_instance_id,
                reason,
                position,
                pos_x_units,
                pos_y_units,
                until_ms,
            },
        );
    }

    pub(in crate::game::battle::core) fn update_move_position_to(
        &mut self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) {
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };
        let speed_units_per_ms = unit.stats.move_speed_units_per_ms.max(1);
        let ActionState::Moving(state) = &mut unit.action_state else {
            return;
        };

        let last = state.last_update_ms;
        if now_ms <= last {
            return;
        }

        let elapsed = now_ms - last;
        let speed = speed_units_per_ms as i64;
        let cap = speed.saturating_mul(elapsed as i64);

        let dx = state.target_x_units.saturating_sub(unit.pos_x_units);
        let dy = state.target_y_units.saturating_sub(unit.pos_y_units);

        let step_x = if dx >= 0 { dx.min(cap) } else { dx.max(-cap) };
        let step_y = if dy >= 0 { dy.min(cap) } else { dy.max(-cap) };

        unit.pos_x_units = unit.pos_x_units.saturating_add(step_x);
        unit.pos_y_units = unit.pos_y_units.saturating_add(step_y);
        state.last_update_ms = now_ms;
    }

    fn time_to_reach_target_ms(
        pos_x_units: i64,
        pos_y_units: i64,
        target_x_units: i64,
        target_y_units: i64,
        speed_units_per_ms: u32,
    ) -> u64 {
        let speed_units_per_ms = speed_units_per_ms.max(1) as u64;
        let dist_x = (target_x_units - pos_x_units).unsigned_abs();
        let dist_y = (target_y_units - pos_y_units).unsigned_abs();
        let dist_units = dist_x.max(dist_y);
        dist_units.div_ceil(speed_units_per_ms).max(1)
    }

    fn sample_motion_segment_at(
        &self,
        unit_instance_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<SampledMotionSegment> {
        let unit = self.units.get(&unit_instance_id)?;
        let mut pos_x_units = unit.pos_x_units;
        let mut pos_y_units = unit.pos_y_units;

        match &unit.action_state {
            ActionState::Moving(state) => {
                let speed_units_per_ms = unit.stats.move_speed_units_per_ms.max(1) as i64;
                let elapsed = now_ms.saturating_sub(state.last_update_ms);
                let cap = speed_units_per_ms.saturating_mul(elapsed as i64);

                let dx = state.target_x_units.saturating_sub(pos_x_units);
                let dy = state.target_y_units.saturating_sub(pos_y_units);
                let step_x = if dx >= 0 { dx.min(cap) } else { dx.max(-cap) };
                let step_y = if dy >= 0 { dy.min(cap) } else { dy.max(-cap) };

                pos_x_units = pos_x_units.saturating_add(step_x);
                pos_y_units = pos_y_units.saturating_add(step_y);

                let remaining_dx = state.target_x_units.saturating_sub(pos_x_units);
                let remaining_dy = state.target_y_units.saturating_sub(pos_y_units);
                let boundary_until_ms = now_ms.saturating_add(Self::time_to_reach_target_ms(
                    pos_x_units,
                    pos_y_units,
                    state.target_x_units,
                    state.target_y_units,
                    unit.stats.move_speed_units_per_ms.max(1),
                ));

                Some(SampledMotionSegment {
                    pos_x_units,
                    pos_y_units,
                    vel_x_units_per_ms: remaining_dx.signum().saturating_mul(speed_units_per_ms),
                    vel_y_units_per_ms: remaining_dy.signum().saturating_mul(speed_units_per_ms),
                    moving_until_ms: Some(
                        if state.step_end_kind == MovementSegmentEndKind::RangeEnter {
                            state.step_ends_at_ms.max(now_ms).min(boundary_until_ms)
                        } else {
                            boundary_until_ms
                        },
                    ),
                })
            }
            _ => Some(SampledMotionSegment {
                pos_x_units,
                pos_y_units,
                vel_x_units_per_ms: 0,
                vel_y_units_per_ms: 0,
                moving_until_ms: None,
            }),
        }
    }

    fn distance_minus_reach_sq_at(
        dx0: i64,
        dy0: i64,
        dvx: i64,
        dvy: i64,
        reach_units: i64,
        t_ms: u64,
    ) -> i128 {
        let t = i128::from(t_ms);
        let dx = i128::from(dx0) + i128::from(dvx) * t;
        let dy = i128::from(dy0) + i128::from(dvy) * t;
        let reach = i128::from(reach_units.unsigned_abs());
        dx * dx + dy * dy - reach * reach
    }

    fn div_floor_i128(lhs: i128, rhs: i128) -> i128 {
        let q = lhs / rhs;
        let r = lhs % rhs;
        if r != 0 && ((r > 0) != (rhs > 0)) {
            q - 1
        } else {
            q
        }
    }

    fn div_ceil_i128(lhs: i128, rhs: i128) -> i128 {
        let q = lhs / rhs;
        let r = lhs % rhs;
        if r != 0 && ((r > 0) == (rhs > 0)) {
            q + 1
        } else {
            q
        }
    }

    fn earliest_range_enter_dt_ms(
        attacker: SampledMotionSegment,
        target: SampledMotionSegment,
        reach_units: i64,
        max_dt_ms: u64,
    ) -> Option<u64> {
        if max_dt_ms == 0 {
            return None;
        }

        let dx0 = attacker.pos_x_units.saturating_sub(target.pos_x_units);
        let dy0 = attacker.pos_y_units.saturating_sub(target.pos_y_units);
        let dvx = attacker
            .vel_x_units_per_ms
            .saturating_sub(target.vel_x_units_per_ms);
        let dvy = attacker
            .vel_y_units_per_ms
            .saturating_sub(target.vel_y_units_per_ms);

        if Self::distance_minus_reach_sq_at(dx0, dy0, dvx, dvy, reach_units, 0) <= 0 {
            return Some(0);
        }

        let a = i128::from(dvx) * i128::from(dvx) + i128::from(dvy) * i128::from(dvy);
        if a == 0 {
            return None;
        }

        let b = 2 * (i128::from(dx0) * i128::from(dvx) + i128::from(dy0) * i128::from(dvy));

        let vertex_denom = 2 * a;
        let max_dt_i128 = i128::from(max_dt_ms);
        let mut candidates = [0_i128, max_dt_i128, 0_i128, 0_i128];
        candidates[2] = Self::div_floor_i128(-b, vertex_denom).clamp(0, max_dt_i128);
        candidates[3] = Self::div_ceil_i128(-b, vertex_denom).clamp(0, max_dt_i128);

        let mut min_t = 0_u64;
        let mut min_value = i128::MAX;
        for candidate in candidates {
            let t_ms = candidate as u64;
            let value = Self::distance_minus_reach_sq_at(dx0, dy0, dvx, dvy, reach_units, t_ms);
            if value < min_value || (value == min_value && t_ms < min_t) {
                min_value = value;
                min_t = t_ms;
            }
        }

        if min_value > 0 {
            return None;
        }

        let mut low = 1_u64;
        let mut high = min_t.max(1);
        while low < high {
            let mid = low + (high - low) / 2;
            if Self::distance_minus_reach_sq_at(dx0, dy0, dvx, dvy, reach_units, mid) <= 0 {
                high = mid;
            } else {
                low = mid + 1;
            }
        }

        Some(low)
    }

    fn earliest_instant_range_enter_dt_for_moving_unit(
        &self,
        unit_id: UnitInstanceId,
        now_ms: u64,
        boundary_dt_ms: u64,
        reach_units: i64,
    ) -> Option<u64> {
        let attacker = self.sample_motion_segment_at(unit_id, now_ms)?;
        let attacker_owner = self.units.get(&unit_id)?.owner;

        let mut earliest: Option<u64> = None;
        for target in self.units.values() {
            if target.is_dead() || target.owner == attacker_owner {
                continue;
            }

            let target_sample = self.sample_motion_segment_at(target.instance_id, now_ms)?;
            let valid_dt = target_sample
                .moving_until_ms
                .map(|until_ms| until_ms.saturating_sub(now_ms))
                .unwrap_or(boundary_dt_ms)
                .min(boundary_dt_ms);

            let Some(range_dt) =
                Self::earliest_range_enter_dt_ms(attacker, target_sample, reach_units, valid_dt)
            else {
                continue;
            };

            earliest = Some(match earliest {
                Some(best) => best.min(range_dt),
                None => range_dt,
            });
        }

        earliest
    }

    pub(in crate::game::battle::core) fn schedule_current_move_step(
        &mut self,
        unit_id: UnitInstanceId,
        now_ms: u64,
    ) -> Option<u64> {
        let sample = self.sample_motion_segment_at(unit_id, now_ms)?;
        let (base_uuid, target_x_units, target_y_units, speed_units_per_ms) = {
            let unit = self.units.get(&unit_id)?;
            let ActionState::Moving(state) = &unit.action_state else {
                return None;
            };
            (
                unit.base_uuid,
                state.target_x_units,
                state.target_y_units,
                unit.stats.move_speed_units_per_ms.max(1),
            )
        };

        let boundary_dt_ms = Self::time_to_reach_target_ms(
            sample.pos_x_units,
            sample.pos_y_units,
            target_x_units,
            target_y_units,
            speed_units_per_ms,
        );

        let policy = self.basic_attack_range_policy(base_uuid);
        if policy.use_continuous_range {
            if let Some(range_dt_ms) = self
                .earliest_instant_range_enter_dt_for_moving_unit(
                    unit_id,
                    now_ms,
                    boundary_dt_ms,
                    policy.instant_melee_reach_units,
                )
                .filter(|dt| *dt > 0 && *dt < boundary_dt_ms)
            {
                let end_ms = now_ms.saturating_add(range_dt_ms);
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    if let ActionState::Moving(state) = &mut unit.action_state {
                        state.step_started_at_ms = now_ms;
                        state.step_ends_at_ms = end_ms;
                        state.step_end_kind = MovementSegmentEndKind::RangeEnter;
                        return Some(end_ms);
                    }
                }
                return None;
            }
        }

        let end_ms = now_ms.saturating_add(boundary_dt_ms.max(1));
        if let Some(unit) = self.units.get_mut(&unit_id) {
            if let ActionState::Moving(state) = &mut unit.action_state {
                state.step_started_at_ms = now_ms;
                state.step_ends_at_ms = end_ms;
                state.step_end_kind = MovementSegmentEndKind::Boundary;
                return Some(end_ms);
            }
        }

        None
    }

    fn stop_moving_unit_on_target_in_range(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
    ) -> bool {
        let Some(unit) = self.units.get(&unit_id) else {
            return false;
        };
        if unit.is_dead() {
            return false;
        }
        if !matches!(unit.action_state, ActionState::Moving(_)) {
            return false;
        }

        let target_id = if let Some(target_id) =
            self.select_basic_attack_target(unit_id, unit.current_target, None)
        {
            target_id
        } else {
            if self.moving_unit_has_near_engage_lock(unit_id) {
                return false;
            }
            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.current_target = None;
            }
            return false;
        };

        self.interrupt_movement(
            now_ms,
            unit_id,
            MovementStopReason::TargetAcquired,
            None,
            ActionState::Idle,
        );
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.current_target = Some(target_id);
        }
        self.event_queue
            .push(BattleEvent::MovementIntent { time_ms: now_ms });

        true
    }

    fn moving_unit_has_near_engage_lock(&self, unit_id: UnitInstanceId) -> bool {
        let Some(unit) = self.units.get(&unit_id) else {
            return false;
        };
        let Some(_locked_target_id) =
            self.persisted_target_if_alive(unit.owner, unit.current_target)
        else {
            return false;
        };
        let ActionState::Moving(state) = &unit.action_state else {
            return false;
        };

        let next_index = state.path_cursor as usize + 1;
        state.path.len().saturating_sub(next_index) <= 1
    }

    pub(in crate::game::battle::core) fn post_move_retarget_at(&mut self, now_ms: u64) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for unit_id in unit_ids {
            let is_moving = self
                .units
                .get(&unit_id)
                .is_some_and(|u| !u.is_dead() && matches!(u.action_state, ActionState::Moving(_)));
            if !is_moving {
                continue;
            }

            // Keep continuous position in sync before recording MovementStopped.
            self.update_move_position_to(unit_id, now_ms);

            if self.battlefield.position_of(unit_id).is_none() {
                continue;
            }

            self.stop_moving_unit_on_target_in_range(now_ms, unit_id);
        }
    }

    pub fn handle_move_steps_at(&mut self, now_ms: u64, move_steps: Vec<(UnitInstanceId, u32)>) {
        let mut unique: Vec<(UnitInstanceId, u32)> = move_steps.into_iter().collect();
        unique.sort_by(|(a_id, a_e), (b_id, b_e)| {
            a_id.as_bytes()
                .cmp(b_id.as_bytes())
                .then_with(|| a_e.cmp(b_e))
        });
        unique.dedup();

        let abort_to_idle = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            core.interrupt_movement(
                now_ms,
                unit_id,
                MovementStopReason::InvalidState,
                None,
                ActionState::Idle,
            );
            core.event_queue
                .push(BattleEvent::MovementIntent { time_ms: now_ms });
        };

        let abort_to_wait_repath = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            let repath_counter = match core.units.get(&unit_id).map(|unit| &unit.action_state) {
                Some(ActionState::Moving(state)) => state.repath_counter.wrapping_add(1),
                Some(ActionState::Holding { repath_counter, .. })
                | Some(ActionState::Yielding { repath_counter, .. })
                | Some(ActionState::Blocked { repath_counter, .. })
                | Some(ActionState::WaitRepath { repath_counter, .. }) => {
                    repath_counter.wrapping_add(1)
                }
                _ => 1,
            };
            let until_ms = core.next_wait_repath_until_ms(now_ms, unit_id, repath_counter);

            core.interrupt_movement(
                now_ms,
                unit_id,
                MovementStopReason::WaitRepath,
                Some(until_ms),
                ActionState::WaitRepath {
                    until_ms,
                    repath_counter,
                },
            );
            core.event_queue
                .push(BattleEvent::MovementIntent { time_ms: until_ms });
        };

        let abort_to_yield = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            let repath_counter = match core.units.get(&unit_id).map(|unit| &unit.action_state) {
                Some(ActionState::Moving(state)) => state.repath_counter,
                Some(ActionState::Holding { repath_counter, .. })
                | Some(ActionState::Yielding { repath_counter, .. })
                | Some(ActionState::Blocked { repath_counter, .. })
                | Some(ActionState::WaitRepath { repath_counter, .. }) => *repath_counter,
                _ => 0,
            };
            core.enter_yield(now_ms, unit_id, YIELD_RETRY_DELAY_MS, repath_counter);
        };

        let abort_to_blocked = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            let repath_counter = match core.units.get(&unit_id).map(|unit| &unit.action_state) {
                Some(ActionState::Moving(state)) => state.repath_counter,
                Some(ActionState::Holding { repath_counter, .. })
                | Some(ActionState::Yielding { repath_counter, .. })
                | Some(ActionState::Blocked { repath_counter, .. })
                | Some(ActionState::WaitRepath { repath_counter, .. }) => *repath_counter,
                _ => 0,
            };
            core.enter_blocked(now_ms, unit_id, BLOCKED_RETRY_DELAY_MS, repath_counter);
        };

        let mut due: Vec<UnitInstanceId> = Vec::new();
        for (unit_id, expected_epoch) in &unique {
            let Some(unit) = self.units.get(unit_id) else {
                continue;
            };
            if unit.is_dead() {
                continue;
            }
            if unit.move_epoch != *expected_epoch {
                continue;
            }
            if !matches!(unit.action_state, ActionState::Moving(_)) {
                continue;
            }
            due.push(*unit_id);
        }

        for unit_id in &due {
            self.update_move_position_to(*unit_id, now_ms);
        }
        let due_set: HashSet<UnitInstanceId> = due.iter().copied().collect();

        let mut moving_unit: Vec<(UnitInstanceId, Position, Position, u8)> = Vec::new();
        for unit_id in &due {
            let Some(unit) = self.units.get(unit_id) else {
                continue;
            };
            if unit.is_dead() {
                continue;
            }

            let ActionState::Moving(state) = &unit.action_state else {
                continue;
            };

            if state.step_ends_at_ms != now_ms {
                continue;
            }

            let Some(from) = self.battlefield.position_of(*unit_id) else {
                continue;
            };

            // 이벤트 꼬인 경우 Idle 전환
            if from != state.step_from {
                abort_to_idle(self, *unit_id);
                continue;
            }

            let (pos_x, pos_y) = self
                .units
                .get(unit_id)
                .map(|u| (u.pos_x_units, u.pos_y_units))
                .unwrap_or_else(|| tile_center_units(from));
            let step_end_kind = self
                .units
                .get(unit_id)
                .and_then(|unit| match &unit.action_state {
                    ActionState::Moving(state) => Some(state.step_end_kind.clone()),
                    _ => None,
                })
                .unwrap_or(MovementSegmentEndKind::Boundary);

            if step_end_kind == MovementSegmentEndKind::RangeEnter {
                if self.stop_moving_unit_on_target_in_range(now_ms, *unit_id) {
                    continue;
                }
                if let Some(next_ms) = self.schedule_current_move_step(*unit_id, now_ms) {
                    self.schedule_move_step_at(*unit_id, next_ms);
                    continue;
                }
                abort_to_idle(self, *unit_id);
                continue;
            }

            let remaining_x = (state.target_x_units - pos_x).unsigned_abs();
            let remaining_y = (state.target_y_units - pos_y).unsigned_abs();
            if remaining_x.max(remaining_y) > 0 {
                if let Some(next_ms) = self.schedule_current_move_step(*unit_id, now_ms) {
                    self.schedule_move_step_at(*unit_id, next_ms);
                } else {
                    abort_to_idle(self, *unit_id);
                }
                continue;
            }

            let cursor = state.path_cursor as usize;
            if cursor >= state.path.len()
                || cursor + 1 >= state.path.len()
                || state.path[cursor] != from
            {
                abort_to_idle(self, *unit_id);
                continue;
            }
            let to = state.path[cursor + 1];
            if state.step_to != to {
                abort_to_idle(self, *unit_id);
                continue;
            }
            moving_unit.push((*unit_id, from, to, state.orchestrator_priority));
        }

        let priority_by_unit: HashMap<UnitInstanceId, u8> = moving_unit
            .iter()
            .map(|(unit_id, _from, _to, priority)| (*unit_id, *priority))
            .collect();

        let mut intents: Vec<(UnitInstanceId, Position, Position, u8)> = moving_unit;
        intents.sort_by(|a, b| {
            a.3.cmp(&b.3)
                .then_with(|| a.0.as_bytes().cmp(b.0.as_bytes()))
        });

        let mut claimed_to: HashSet<Position> = HashSet::new();
        let mut deferred_intents: Vec<(UnitInstanceId, Position, Position, u8)> = Vec::new();
        let mut deferred_continuation_reservations: Vec<DeferredContinuationReservation> =
            Vec::new();
        let mut committed_moves: Vec<UnitInstanceId> = Vec::new();

        for (unit_id, from, to, priority) in intents {
            match self.try_commit_ready_move_intent(
                now_ms,
                unit_id,
                from,
                to,
                priority,
                &due_set,
                &priority_by_unit,
                &mut claimed_to,
                &mut deferred_intents,
                abort_to_idle,
                abort_to_wait_repath,
                abort_to_blocked,
                abort_to_yield,
            ) {
                ReadyMoveAttempt::Moved(unit_id) => committed_moves.push(unit_id),
                ReadyMoveAttempt::Completed | ReadyMoveAttempt::Deferred => {}
            }
        }

        for (unit_id, from, to, priority) in deferred_intents {
            match self.try_commit_ready_move_intent(
                now_ms,
                unit_id,
                from,
                to,
                priority,
                &due_set,
                &priority_by_unit,
                &mut claimed_to,
                &mut Vec::new(),
                abort_to_idle,
                abort_to_wait_repath,
                abort_to_blocked,
                abort_to_yield,
            ) {
                ReadyMoveAttempt::Moved(unit_id) => committed_moves.push(unit_id),
                ReadyMoveAttempt::Completed | ReadyMoveAttempt::Deferred => {}
            }
        }

        for unit_id in committed_moves {
            self.finalize_committed_move_after_batch(
                now_ms,
                unit_id,
                &due_set,
                &priority_by_unit,
                &mut deferred_continuation_reservations,
                abort_to_idle,
                abort_to_wait_repath,
                abort_to_blocked,
                abort_to_yield,
            );
        }

        for deferred in deferred_continuation_reservations {
            self.try_reserve_deferred_continuation(
                now_ms,
                deferred,
                &due_set,
                &priority_by_unit,
                abort_to_wait_repath,
                abort_to_blocked,
                abort_to_yield,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn try_commit_ready_move_intent(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        from: Position,
        to: Position,
        priority: u8,
        due_set: &HashSet<UnitInstanceId>,
        priority_by_unit: &HashMap<UnitInstanceId, u8>,
        claimed_to: &mut HashSet<Position>,
        deferred_intents: &mut Vec<(UnitInstanceId, Position, Position, u8)>,
        abort_to_idle: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_wait_repath: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_blocked: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_yield: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
    ) -> ReadyMoveAttempt {
        if self.units.get(&unit_id).is_none_or(|u| u.is_dead()) {
            return ReadyMoveAttempt::Completed;
        }

        if claimed_to.contains(&to) {
            abort_to_yield(self, unit_id);
            return ReadyMoveAttempt::Completed;
        }

        if self
            .battlefield
            .position_of(unit_id)
            .is_some_and(|p| p != from)
        {
            abort_to_idle(self, unit_id);
            return ReadyMoveAttempt::Completed;
        }

        let mut cancel_soft_reservation_of: Option<UnitInstanceId> = None;
        if let Some(reservation) = self.battlefield.reservation_at(to) {
            if reservation.unit != unit_id {
                if self.battlefield.reservation_blocks_for(unit_id, to) {
                    match self.classify_movement_blocker(
                        unit_id,
                        reservation.unit,
                        priority,
                        due_set,
                        priority_by_unit,
                    ) {
                        MovementBlockerKind::DeferToHigherPriorityDueMover => {
                            deferred_intents.push((unit_id, from, to, priority));
                            return ReadyMoveAttempt::Deferred;
                        }
                        MovementBlockerKind::YieldToDueMover => {
                            abort_to_yield(self, unit_id);
                            return ReadyMoveAttempt::Completed;
                        }
                        MovementBlockerKind::FriendlyStatic => {
                            abort_to_blocked(self, unit_id);
                            return ReadyMoveAttempt::Completed;
                        }
                        MovementBlockerKind::HardFailure => {
                            abort_to_wait_repath(self, unit_id);
                            return ReadyMoveAttempt::Completed;
                        }
                    }
                }
                cancel_soft_reservation_of = Some(reservation.unit);
            }
        }

        if let Some(occupant) = self.battlefield.occupant(to).ok().flatten() {
            match self.classify_movement_blocker(
                unit_id,
                occupant,
                priority,
                due_set,
                priority_by_unit,
            ) {
                MovementBlockerKind::DeferToHigherPriorityDueMover => {
                    deferred_intents.push((unit_id, from, to, priority));
                    return ReadyMoveAttempt::Deferred;
                }
                MovementBlockerKind::YieldToDueMover => {
                    abort_to_yield(self, unit_id);
                    return ReadyMoveAttempt::Completed;
                }
                MovementBlockerKind::FriendlyStatic => {
                    abort_to_blocked(self, unit_id);
                    return ReadyMoveAttempt::Completed;
                }
                MovementBlockerKind::HardFailure => {
                    abort_to_wait_repath(self, unit_id);
                    return ReadyMoveAttempt::Completed;
                }
            }
        }

        if self.battlefield.move_unit(unit_id, to).is_err() {
            abort_to_wait_repath(self, unit_id);
            return ReadyMoveAttempt::Completed;
        }

        claimed_to.insert(to);

        if let Some(reserver) = cancel_soft_reservation_of {
            self.battlefield.cancel_reservation(reserver);
        }
        self.battlefield.cancel_reservation(unit_id);

        self.record_timeline(
            now_ms,
            TimelineEvent::UnitMoved {
                unit_instance_id: unit_id,
                from,
                to,
            },
        );

        ReadyMoveAttempt::Moved(unit_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn finalize_committed_move_after_batch(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        due_set: &HashSet<UnitInstanceId>,
        priority_by_unit: &HashMap<UnitInstanceId, u8>,
        deferred_continuation_reservations: &mut Vec<DeferredContinuationReservation>,
        abort_to_idle: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_wait_repath: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_blocked: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_yield: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
    ) {
        if !matches!(
            self.units.get(&unit_id).map(|u| &u.action_state),
            Some(ActionState::Moving(_))
        ) {
            return;
        }

        if self.stop_moving_unit_on_target_in_range(now_ms, unit_id) {
            return;
        }

        let Some(current_pos) = self.battlefield.position_of(unit_id) else {
            abort_to_idle(self, unit_id);
            return;
        };

        let mut next_step_ms: Option<u64> = None;
        let mut should_interrupt_as_arrived = false;
        let mut next_step_to_reserve: Option<Position> = None;
        let mut planned_continuation: Option<PlannedContinuation> = None;
        let mut priority: Option<u8> = None;

        if let Some(unit) = self.units.get_mut(&unit_id) {
            if let ActionState::Moving(state) = &mut unit.action_state {
                state.path_cursor = state.path_cursor.saturating_add(1);

                state.step_from = current_pos;
                state.step_started_at_ms = now_ms;
                priority = Some(state.orchestrator_priority);

                let cursor = state.path_cursor as usize;
                let arrived = state.reserved_destination == Some(current_pos)
                    || cursor + 1 >= state.path.len();
                if arrived {
                    should_interrupt_as_arrived = true;
                } else {
                    planned_continuation =
                        state.planned_continuation.clone().filter(|continuation| {
                            Some(continuation.step) == state.path.get(cursor + 1).copied()
                        });
                    next_step_to_reserve = state
                        .planned_continuation
                        .as_ref()
                        .map(|continuation| continuation.step)
                        .filter(|step| *step == state.path[cursor + 1])
                        .or_else(|| state.path.get(cursor + 1).copied());
                }
            }
        }

        let Some(priority) = priority else {
            abort_to_idle(self, unit_id);
            return;
        };

        if let Some(next_step) = next_step_to_reserve {
            match self.resolve_next_step_reservation_after_move(
                now_ms,
                unit_id,
                next_step,
                priority,
                planned_continuation.clone(),
                due_set,
                priority_by_unit,
            ) {
                NextStepReservationDecision::Reserved => {
                    next_step_ms = self.schedule_current_move_step(unit_id, now_ms);
                }
                NextStepReservationDecision::DeferContinuationRetry => {
                    let Some(mut planned_continuation) = planned_continuation else {
                        abort_to_yield(self, unit_id);
                        return;
                    };
                    if planned_continuation.retry_budget == 0 {
                        abort_to_yield(self, unit_id);
                        return;
                    }
                    planned_continuation.retry_budget =
                        planned_continuation.retry_budget.saturating_sub(1);
                    deferred_continuation_reservations.push(DeferredContinuationReservation {
                        unit_id,
                        next_step,
                        planned_continuation,
                    });
                    return;
                }
                NextStepReservationDecision::Yield => {
                    abort_to_yield(self, unit_id);
                    return;
                }
                NextStepReservationDecision::Blocked => {
                    abort_to_blocked(self, unit_id);
                    return;
                }
                NextStepReservationDecision::WaitRepath => {
                    abort_to_wait_repath(self, unit_id);
                    return;
                }
            }
        }

        if should_interrupt_as_arrived {
            self.interrupt_movement(
                now_ms,
                unit_id,
                MovementStopReason::Arrived,
                None,
                ActionState::Idle,
            );
            self.event_queue
                .push(BattleEvent::MovementIntent { time_ms: now_ms });
        }

        if let Some(ms) = next_step_ms {
            self.schedule_move_step_at(unit_id, ms);
        }
    }

    fn try_reserve_deferred_continuation(
        &mut self,
        now_ms: u64,
        deferred: DeferredContinuationReservation,
        due_set: &HashSet<UnitInstanceId>,
        priority_by_unit: &HashMap<UnitInstanceId, u8>,
        abort_to_wait_repath: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_blocked: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
        abort_to_yield: impl Fn(&mut BattleCore, UnitInstanceId) + Copy,
    ) {
        let DeferredContinuationReservation {
            unit_id,
            next_step,
            planned_continuation,
        } = deferred;
        if !matches!(
            self.units.get(&unit_id).map(|u| &u.action_state),
            Some(ActionState::Moving(_))
        ) {
            return;
        }

        match self.resolve_next_step_reservation_after_move(
            now_ms,
            unit_id,
            next_step,
            planned_continuation.owner_priority,
            Some(planned_continuation),
            due_set,
            priority_by_unit,
        ) {
            NextStepReservationDecision::Reserved => {}
            NextStepReservationDecision::DeferContinuationRetry
            | NextStepReservationDecision::Yield => {
                abort_to_yield(self, unit_id);
                return;
            }
            NextStepReservationDecision::Blocked => {
                abort_to_blocked(self, unit_id);
                return;
            }
            NextStepReservationDecision::WaitRepath => {
                abort_to_wait_repath(self, unit_id);
                return;
            }
        }

        if let Some(ms) = self.schedule_current_move_step(unit_id, now_ms) {
            self.schedule_move_step_at(unit_id, ms);
        }
    }

    fn resolve_next_step_reservation_after_move(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        next_step: Position,
        priority: u8,
        planned_continuation: Option<PlannedContinuation>,
        due_set: &HashSet<UnitInstanceId>,
        priority_by_unit: &HashMap<UnitInstanceId, u8>,
    ) -> NextStepReservationDecision {
        let planner_continuation = planned_continuation
            .as_ref()
            .filter(|continuation| continuation.step == next_step);
        let effective_priority = planner_continuation
            .map(|continuation| continuation.owner_priority)
            .unwrap_or(priority);
        let next_step_claim_priority =
            planner_continuation.and_then(|continuation| continuation.claim_priority);
        if self
            .battlefield
            .reserve(unit_id, next_step, now_ms)
            .is_err()
        {
            if let Some(reservation) = self.battlefield.reservation_at(next_step) {
                match self.classify_movement_blocker(
                    unit_id,
                    reservation.unit,
                    effective_priority,
                    due_set,
                    priority_by_unit,
                ) {
                    MovementBlockerKind::DeferToHigherPriorityDueMover => {
                        if priority_by_unit
                            .get(&reservation.unit)
                            .copied()
                            .zip(next_step_claim_priority)
                            .is_some_and(|(blocker_priority, claim_priority)| {
                                blocker_priority <= claim_priority
                            })
                        {
                            return NextStepReservationDecision::Yield;
                        }
                        if planner_continuation.is_none() {
                            return NextStepReservationDecision::Yield;
                        }
                        return NextStepReservationDecision::DeferContinuationRetry;
                    }
                    MovementBlockerKind::YieldToDueMover => {
                        return NextStepReservationDecision::Yield;
                    }
                    MovementBlockerKind::FriendlyStatic => {
                        return NextStepReservationDecision::Blocked;
                    }
                    MovementBlockerKind::HardFailure => {}
                }
            }
            if let Some(occupant) = self.battlefield.occupant(next_step).ok().flatten() {
                match self.classify_movement_blocker(
                    unit_id,
                    occupant,
                    effective_priority,
                    due_set,
                    priority_by_unit,
                ) {
                    MovementBlockerKind::DeferToHigherPriorityDueMover => {
                        if priority_by_unit
                            .get(&occupant)
                            .copied()
                            .zip(next_step_claim_priority)
                            .is_some_and(|(blocker_priority, claim_priority)| {
                                blocker_priority <= claim_priority
                            })
                        {
                            return NextStepReservationDecision::Yield;
                        }
                        if planner_continuation.is_none() {
                            return NextStepReservationDecision::Yield;
                        }
                        return NextStepReservationDecision::DeferContinuationRetry;
                    }
                    MovementBlockerKind::YieldToDueMover => {
                        return NextStepReservationDecision::Yield;
                    }
                    MovementBlockerKind::FriendlyStatic => {
                        return NextStepReservationDecision::Blocked;
                    }
                    MovementBlockerKind::HardFailure => {}
                }
            }
            return NextStepReservationDecision::WaitRepath;
        }

        self.apply_reserved_next_step(unit_id, next_step);
        NextStepReservationDecision::Reserved
    }

    fn apply_reserved_next_step(&mut self, unit_id: UnitInstanceId, next_step: Position) {
        if let Some(unit) = self.units.get_mut(&unit_id) {
            if let ActionState::Moving(state) = &mut unit.action_state {
                state.step_to = next_step;
                let (target_x, target_y) = boundary_target_units(state.step_from, state.step_to);
                state.target_x_units = target_x;
                state.target_y_units = target_y;
                let next_cursor = state.path_cursor as usize + 2;
                state.planned_continuation =
                    state
                        .path
                        .get(next_cursor)
                        .copied()
                        .map(|step| PlannedContinuation {
                            step,
                            owner_priority: state.orchestrator_priority,
                            claim_priority: None,
                            retry_budget: 1,
                        });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::core::movement::{
        MovementSegmentEndKind, MovementState, HALF_TILE_UNITS, TILE_UNITS_PER_TILE,
    };
    use crate::game::battle::core::types::RuntimeUnit;
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::{
        abnormality_data::AbnormalityDatabase, artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase, equipment_data::EquipmentDatabase, event_pools::EventPhasePool,
        event_pools::EventPoolConfig, pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase, shop_data::ShopDatabase, skill_data::SkillDatabase,
        GameDataBase,
    };
    use crate::game::enums::Side;
    use crate::game::stats::UnitStats;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
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

    fn place_unit(core: &mut BattleCore, unit_id: UnitInstanceId, pos: Position) {
        core.battlefield.place(unit_id, pos).unwrap();
        let unit = core.units.get_mut(&unit_id).unwrap();
        unit.pos_x_units = (pos.x as i64) * TILE_UNITS_PER_TILE as i64;
        unit.pos_y_units = (pos.y as i64) * TILE_UNITS_PER_TILE as i64;
    }

    #[test]
    fn time_to_reach_target_ms_uses_chebyshev_distance_and_ceil_division() {
        // dist=max(|dx|,|dy|)=11, speed=10 -> ceil(11/10)=2
        let dt = BattleCore::time_to_reach_target_ms(0, 0, 11, -1, 10);
        assert_eq!(dt, 2);

        // Even when already at target, it returns at least 1ms to avoid "zero time" loops.
        let dt = BattleCore::time_to_reach_target_ms(5, 5, 5, 5, 10);
        assert_eq!(dt, 1);
    }

    #[test]
    fn update_move_position_to_advances_position_with_speed_cap() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(1).into();
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10;

        let mut movement = MovementState::new_at(Position::new(0, 0), 100);
        movement.target_x_units = 100;
        movement.target_y_units = 0;
        movement.last_update_ms = 100;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
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
            },
        );

        // now_ms <= last_update_ms: no-op
        core.update_move_position_to(unit_id, 100);
        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.pos_x_units, 0);

        // elapsed=5ms, cap=10*5=50 units.
        core.update_move_position_to(unit_id, 105);
        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.pos_x_units, 50);

        // elapsed=5ms again, reaches target but should not overshoot.
        core.update_move_position_to(unit_id, 110);
        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.pos_x_units, 100);
    }

    #[test]
    fn interrupt_movement_updates_continuous_position_before_recording_stop() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(2).into();
        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10;

        let mut movement = MovementState::new_at(Position::new(0, 0), 100);
        movement.target_x_units = 100;
        movement.target_y_units = 0;
        movement.last_update_ms = 100;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
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
            },
        );
        core.battlefield
            .place(unit_id, Position::new(0, 0))
            .unwrap();

        assert!(core.interrupt_movement(
            105,
            unit_id,
            MovementStopReason::Died,
            None,
            ActionState::Dead,
        ));

        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.pos_x_units, 50);
        assert!(matches!(unit.action_state, ActionState::Dead));

        let stop_entry = core
            .timeline
            .entries
            .iter()
            .find(|entry| matches!(entry.event, TimelineEvent::MovementStopped { .. }))
            .expect("missing MovementStopped");

        match stop_entry.event {
            TimelineEvent::MovementStopped {
                reason,
                pos_x_units,
                pos_y_units,
                ..
            } => {
                assert_eq!(reason, MovementStopReason::Died);
                assert_eq!(pos_x_units, 50);
                assert_eq!(pos_y_units, 0);
            }
            _ => unreachable!("expected MovementStopped"),
        }
    }

    #[test]
    fn arrived_keeps_actual_continuous_stop_position_while_advancing_logical_tile() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(4).into();
        let from = Position::new(0, 0);
        let to = Position::new(1, 0);

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![from, to];
        movement.reserved_destination = Some(to);
        movement.step_from = from;
        movement.step_to = to;
        movement.target_x_units = HALF_TILE_UNITS as i64;
        movement.target_y_units = 0;
        movement.last_update_ms = 100;
        movement.step_started_at_ms = 100;
        movement.step_ends_at_ms = 150;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
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
            },
        );
        core.battlefield.place(unit_id, from).unwrap();

        core.handle_move_steps_at(150, vec![(unit_id, 0)]);

        let unit = core.units.get(&unit_id).unwrap();
        assert_eq!(unit.pos_x_units, HALF_TILE_UNITS as i64);
        assert!(matches!(unit.action_state, ActionState::Idle));
        assert_eq!(core.battlefield.position_of(unit_id), Some(to));

        let stop_entry = core
            .timeline
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    entry.event,
                    TimelineEvent::MovementStopped {
                        reason: MovementStopReason::Arrived,
                        ..
                    }
                )
            })
            .expect("missing Arrived MovementStopped");

        match stop_entry.event {
            TimelineEvent::MovementStopped {
                reason,
                position,
                pos_x_units,
                pos_y_units,
                ..
            } => {
                assert_eq!(reason, MovementStopReason::Arrived);
                assert_eq!(position, to);
                assert_eq!(pos_x_units, HALF_TILE_UNITS as i64);
                assert_eq!(pos_y_units, 0);
            }
            _ => unreachable!("expected Arrived MovementStopped"),
        }
    }

    #[test]
    fn moving_unit_retargets_to_nearer_enemy_when_locked_target_is_out_of_range() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(5).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(6).into();
        let nearer_enemy_id: UnitInstanceId = Uuid::from_u128(7).into();
        let tile = Position::new(1, 0);

        let mut movement = MovementState::new_at(Position::new(0, 0), 100);
        movement.path = vec![Position::new(0, 0), tile, Position::new(2, 0)];
        movement.step_from = Position::new(0, 0);
        movement.step_to = tile;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(locked_target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            locked_target_id,
            RuntimeUnit {
                instance_id: locked_target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );
        core.units.insert(
            nearer_enemy_id,
            RuntimeUnit {
                instance_id: nearer_enemy_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );

        place_unit(&mut core, unit_id, tile);
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, nearer_enemy_id, Position::new(2, 0));

        let stopped = core.stop_moving_unit_on_target_in_range(100, unit_id);

        assert!(stopped);
        assert_eq!(
            core.units.get(&unit_id).unwrap().current_target,
            Some(nearer_enemy_id)
        );
        assert!(matches!(
            core.units.get(&unit_id).unwrap().action_state,
            ActionState::Idle
        ));
    }

    #[test]
    fn moving_unit_engages_other_enemy_when_near_engage_locked_target_is_not_yet_in_range() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(15).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(16).into();
        let in_range_enemy_id: UnitInstanceId = Uuid::from_u128(17).into();
        let from = Position::new(1, 0);
        let toward = Position::new(2, 0);

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![Position::new(0, 0), from, toward];
        movement.path_cursor = 1;
        movement.step_from = from;
        movement.step_to = toward;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(locked_target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            locked_target_id,
            RuntimeUnit {
                instance_id: locked_target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );
        core.units.insert(
            in_range_enemy_id,
            RuntimeUnit {
                instance_id: in_range_enemy_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );

        place_unit(&mut core, unit_id, from);
        place_unit(&mut core, locked_target_id, Position::new(3, 0));
        place_unit(&mut core, in_range_enemy_id, Position::new(1, 1));

        let stopped = core.stop_moving_unit_on_target_in_range(100, unit_id);

        assert!(stopped);
        assert_eq!(
            core.units.get(&unit_id).unwrap().current_target,
            Some(in_range_enemy_id)
        );
        assert!(matches!(
            core.units.get(&unit_id).unwrap().action_state,
            ActionState::Idle
        ));
    }

    #[test]
    fn moving_unit_retargets_to_straight_enemy_over_equal_range_diagonal_enemy() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(18).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(19).into();
        let diagonal_enemy_id: UnitInstanceId = Uuid::from_u128(20).into();
        let straight_enemy_id: UnitInstanceId = Uuid::from_u128(21).into();
        let tile = Position::new(2, 2);

        let mut movement = MovementState::new_at(Position::new(2, 3), 100);
        movement.path = vec![Position::new(2, 3), tile, Position::new(2, 1)];
        movement.step_from = Position::new(2, 3);
        movement.step_to = tile;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(locked_target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        for enemy_id in [locked_target_id, diagonal_enemy_id, straight_enemy_id] {
            core.units.insert(
                enemy_id,
                RuntimeUnit {
                    instance_id: enemy_id,
                    owner: Side::Opponent,
                    base_uuid: Uuid::nil(),
                    stats: UnitStats::with_values(10, 10, 1, 0, 1),
                    pos_x_units: 0,
                    pos_y_units: 0,
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
                },
            );
        }

        place_unit(&mut core, unit_id, tile);
        place_unit(&mut core, locked_target_id, Position::new(2, 0));
        place_unit(&mut core, diagonal_enemy_id, Position::new(1, 1));
        place_unit(&mut core, straight_enemy_id, Position::new(2, 1));

        let stopped = core.stop_moving_unit_on_target_in_range(100, unit_id);

        assert!(stopped);
        assert_eq!(
            core.units.get(&unit_id).unwrap().current_target,
            Some(straight_enemy_id)
        );
        assert!(matches!(
            core.units.get(&unit_id).unwrap().action_state,
            ActionState::Idle
        ));
    }

    #[test]
    fn moving_unit_keeps_locked_target_when_almost_in_range_and_no_other_enemy_is_attackable() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(150).into();
        let locked_target_id: UnitInstanceId = Uuid::from_u128(151).into();
        let from = Position::new(1, 0);
        let toward = Position::new(2, 0);

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![Position::new(0, 0), from, toward];
        movement.path_cursor = 1;
        movement.step_from = from;
        movement.step_to = toward;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(locked_target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            locked_target_id,
            RuntimeUnit {
                instance_id: locked_target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );

        place_unit(&mut core, unit_id, from);
        place_unit(&mut core, locked_target_id, Position::new(3, 0));

        let stopped = core.stop_moving_unit_on_target_in_range(100, unit_id);

        assert!(!stopped);
        assert_eq!(
            core.units.get(&unit_id).unwrap().current_target,
            Some(locked_target_id)
        );
        assert!(matches!(
            core.units.get(&unit_id).unwrap().action_state,
            ActionState::Moving(_)
        ));
    }

    #[test]
    fn moving_unit_retargets_when_near_engage_locked_target_is_dead() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(18).into();
        let dead_target_id: UnitInstanceId = Uuid::from_u128(19).into();
        let in_range_enemy_id: UnitInstanceId = Uuid::from_u128(20).into();
        let from = Position::new(1, 0);
        let toward = Position::new(2, 0);

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![Position::new(0, 0), from, toward];
        movement.path_cursor = 1;
        movement.step_from = from;
        movement.step_to = toward;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(dead_target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            dead_target_id,
            RuntimeUnit {
                instance_id: dead_target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(0, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
                move_epoch: 0,
                action_state: ActionState::Dead,
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
            },
        );
        core.units.insert(
            in_range_enemy_id,
            RuntimeUnit {
                instance_id: in_range_enemy_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 0,
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
            },
        );

        place_unit(&mut core, unit_id, from);
        place_unit(&mut core, dead_target_id, Position::new(3, 0));
        place_unit(&mut core, in_range_enemy_id, Position::new(1, 1));

        let stopped = core.stop_moving_unit_on_target_in_range(100, unit_id);

        assert!(stopped);
        assert_eq!(
            core.units.get(&unit_id).unwrap().current_target,
            Some(in_range_enemy_id)
        );
        assert!(matches!(
            core.units.get(&unit_id).unwrap().action_state,
            ActionState::Idle
        ));
    }

    #[test]
    fn moving_unit_stops_immediately_when_step_ends_in_adjacent_melee_tile() {
        let mut core = new_core();

        let unit_id: UnitInstanceId = Uuid::from_u128(21).into();
        let target_id: UnitInstanceId = Uuid::from_u128(22).into();
        let from = Position::new(1, 1);
        let to = Position::new(1, 2);
        let lateral_follow_up = Position::new(0, 2);

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![from, to, lateral_follow_up];
        movement.reserved_destination = Some(lateral_follow_up);
        movement.step_from = from;
        movement.step_to = to;
        movement.target_x_units = 1_000_000;
        movement.target_y_units = 2_500_000;
        movement.last_update_ms = 100;
        movement.step_started_at_ms = 100;
        movement.step_ends_at_ms = 100;
        movement.step_end_kind = MovementSegmentEndKind::Boundary;

        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 1_000_000,
                pos_y_units: 2_500_000,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 1_000_000,
                pos_y_units: 3_000_000,
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
            },
        );

        place_unit(&mut core, unit_id, from);
        core.units.get_mut(&unit_id).unwrap().pos_y_units = 2_500_000;
        place_unit(&mut core, target_id, Position::new(1, 3));

        core.handle_move_steps_at(100, vec![(unit_id, 0)]);

        let unit = core.units.get(&unit_id).unwrap();
        assert!(
            matches!(unit.action_state, ActionState::Idle),
            "unexpected state after adjacent melee step: {:?}",
            unit.action_state
        );
        assert_eq!(unit.current_target, Some(target_id));
        assert_eq!(core.battlefield.position_of(unit_id), Some(to));
        assert_eq!(unit.pos_x_units, 1_000_000);
        assert_eq!(unit.pos_y_units, 2_500_000);

        let stop_entry = core
            .timeline
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    entry.event,
                    TimelineEvent::MovementStopped {
                        unit_instance_id,
                        reason: MovementStopReason::TargetAcquired,
                        ..
                    } if unit_instance_id == unit_id
                )
            })
            .expect("missing target-acquired stop");

        match stop_entry.event {
            TimelineEvent::MovementStopped {
                position,
                pos_x_units,
                pos_y_units,
                ..
            } => {
                assert_eq!(position, to);
                assert_eq!(pos_x_units, 1_000_000);
                assert_eq!(pos_y_units, 2_500_000);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn moving_unit_stops_on_range_enter_before_boundary() {
        let mut core = new_core();

        let attacker_id: UnitInstanceId = Uuid::from_u128(8).into();
        let target_id: UnitInstanceId = Uuid::from_u128(9).into();
        let from = Position::new(0, 3);
        let toward = Position::new(0, 2);

        let mut attacker_stats = UnitStats::with_values(10, 10, 1, 0, 1);
        attacker_stats.move_speed_units_per_ms = 10_000;

        let mut movement = MovementState::new_at(from, 100);
        movement.path = vec![from, toward];
        movement.step_from = from;
        movement.step_to = toward;
        movement.target_x_units = 0;
        movement.target_y_units = 2_500_000;
        movement.last_update_ms = 100;

        core.units.insert(
            attacker_id,
            RuntimeUnit {
                instance_id: attacker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: attacker_stats,
                pos_x_units: 0,
                pos_y_units: 3_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(movement),
                action_locks: Default::default(),
                current_target: Some(target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: UnitStats::with_values(10, 10, 1, 0, 1),
                pos_x_units: 0,
                pos_y_units: 1_600_000,
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
            },
        );

        place_unit(&mut core, attacker_id, from);
        place_unit(&mut core, target_id, Position::new(0, 2));
        core.units.get_mut(&target_id).unwrap().pos_y_units = 1_600_000;

        let scheduled_ms = core.schedule_current_move_step(attacker_id, 100).unwrap();
        assert_eq!(scheduled_ms, 140);
        let attacker = core.units.get(&attacker_id).unwrap();
        let ActionState::Moving(state) = &attacker.action_state else {
            panic!("attacker should still be moving");
        };
        assert_eq!(state.step_end_kind, MovementSegmentEndKind::RangeEnter);

        core.handle_move_steps_at(140, vec![(attacker_id, 0)]);

        let attacker = core.units.get(&attacker_id).unwrap();
        assert!(matches!(attacker.action_state, ActionState::Idle));
        assert_eq!(attacker.current_target, Some(target_id));
        assert_eq!(attacker.pos_y_units, 2_600_000);
        assert_eq!(core.battlefield.position_of(attacker_id), Some(from));

        let stop_entry = core
            .timeline
            .entries
            .iter()
            .find(|entry| {
                matches!(
                    entry.event,
                    TimelineEvent::MovementStopped {
                        unit_instance_id,
                        reason: MovementStopReason::TargetAcquired,
                        ..
                    } if unit_instance_id == attacker_id
                )
            })
            .expect("missing target-acquired stop");

        match stop_entry.event {
            TimelineEvent::MovementStopped {
                position,
                pos_y_units,
                ..
            } => {
                assert_eq!(position, from);
                assert_eq!(pos_y_units, 2_600_000);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn opposing_melee_units_stop_before_overlapping_on_mid_segment_range_enter() {
        let mut core = new_core();

        let attacker_id: UnitInstanceId = Uuid::from_u128(10).into();
        let target_id: UnitInstanceId = Uuid::from_u128(11).into();
        let shared_speed = 10_000;

        let mut attacker_stats = UnitStats::with_values(10, 10, 1, 0, 1);
        attacker_stats.move_speed_units_per_ms = shared_speed;
        let target_stats = attacker_stats;

        let mut attacker_move = MovementState::new_at(Position::new(0, 3), 100);
        attacker_move.path = vec![Position::new(0, 3), Position::new(0, 2)];
        attacker_move.step_from = Position::new(0, 3);
        attacker_move.step_to = Position::new(0, 2);
        attacker_move.target_x_units = 0;
        attacker_move.target_y_units = 2_500_000;
        attacker_move.last_update_ms = 100;

        let mut target_move = MovementState::new_at(Position::new(0, 2), 100);
        target_move.path = vec![Position::new(0, 2), Position::new(0, 3)];
        target_move.step_from = Position::new(0, 2);
        target_move.step_to = Position::new(0, 3);
        target_move.target_x_units = 0;
        target_move.target_y_units = 2_500_000;
        target_move.last_update_ms = 100;

        core.units.insert(
            attacker_id,
            RuntimeUnit {
                instance_id: attacker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats: attacker_stats,
                pos_x_units: 0,
                pos_y_units: 3_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(attacker_move),
                action_locks: Default::default(),
                current_target: Some(target_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );
        core.units.insert(
            target_id,
            RuntimeUnit {
                instance_id: target_id,
                owner: Side::Opponent,
                base_uuid: Uuid::nil(),
                stats: target_stats,
                pos_x_units: 0,
                pos_y_units: 1_800_000,
                move_epoch: 0,
                action_state: ActionState::Moving(target_move),
                action_locks: Default::default(),
                current_target: Some(attacker_id),
                next_basic_attack_ms: 0,
                pending_basic_attack: false,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                pending_skill_cast: None,
            },
        );

        place_unit(&mut core, attacker_id, Position::new(0, 3));
        place_unit(&mut core, target_id, Position::new(0, 2));
        core.units.get_mut(&target_id).unwrap().pos_y_units = 1_800_000;

        let attacker_ms = core.schedule_current_move_step(attacker_id, 100).unwrap();
        let target_ms = core.schedule_current_move_step(target_id, 100).unwrap();
        assert_eq!(attacker_ms, 110);
        assert_eq!(target_ms, 110);

        core.handle_move_steps_at(110, vec![(attacker_id, 0), (target_id, 0)]);

        let attacker = core.units.get(&attacker_id).unwrap();
        let target = core.units.get(&target_id).unwrap();
        assert!(matches!(attacker.action_state, ActionState::Idle));
        assert!(matches!(target.action_state, ActionState::Idle));
        assert_eq!(attacker.pos_y_units, 2_900_000);
        assert_eq!(target.pos_y_units, 1_900_000);
        assert_ne!(attacker.pos_y_units, target.pos_y_units);
    }

    #[test]
    fn same_tick_destination_conflict_yields_without_wait_repath() {
        let mut core = new_core();

        let first_id: UnitInstanceId = Uuid::from_u128(20).into();
        let second_id: UnitInstanceId = Uuid::from_u128(21).into();
        let shared_to = Position::new(1, 1);

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut first_move = MovementState::new_at(Position::new(0, 1), 100);
        first_move.path = vec![Position::new(0, 1), shared_to];
        first_move.step_from = Position::new(0, 1);
        first_move.step_to = shared_to;
        first_move.target_x_units = 500_000;
        first_move.target_y_units = 1_000_000;
        first_move.last_update_ms = 100;
        first_move.step_started_at_ms = 100;
        first_move.step_ends_at_ms = 150;

        let mut second_move = MovementState::new_at(Position::new(2, 1), 100);
        second_move.path = vec![Position::new(2, 1), shared_to];
        second_move.step_from = Position::new(2, 1);
        second_move.step_to = shared_to;
        second_move.target_x_units = 1_500_000;
        second_move.target_y_units = 1_000_000;
        second_move.last_update_ms = 100;
        second_move.step_started_at_ms = 100;
        second_move.step_ends_at_ms = 150;

        core.units.insert(
            first_id,
            RuntimeUnit {
                instance_id: first_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(first_move),
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
            },
        );
        core.units.insert(
            second_id,
            RuntimeUnit {
                instance_id: second_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(second_move),
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
            },
        );

        place_unit(&mut core, first_id, Position::new(0, 1));
        place_unit(&mut core, second_id, Position::new(2, 1));

        core.handle_move_steps_at(150, vec![(first_id, 0), (second_id, 0)]);

        assert_eq!(core.battlefield.position_of(first_id), Some(shared_to));
        assert_eq!(
            core.battlefield.position_of(second_id),
            Some(Position::new(2, 1))
        );
        assert!(matches!(
            core.units.get(&second_id).unwrap().action_state,
            ActionState::Yielding { until_ms: 160, .. }
        ));
        assert!(
            !core.timeline.entries.iter().any(|entry| matches!(
                entry.event,
                TimelineEvent::MovementStopped {
                    unit_instance_id,
                    reason: MovementStopReason::WaitRepath,
                    ..
                } if unit_instance_id == second_id
            )),
            "same-tick claim collision should yield instead of entering wait_repath"
        );
        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::MovementIntent { time_ms } if *time_ms == 160
        )));
    }

    #[test]
    fn same_tick_destination_conflict_uses_orchestrator_priority_before_uuid() {
        let mut core = new_core();

        let lower_uuid_id: UnitInstanceId = Uuid::from_u128(30).into();
        let higher_uuid_id: UnitInstanceId = Uuid::from_u128(31).into();
        let shared_to = Position::new(1, 1);

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut lower_priority_move = MovementState::new_at(Position::new(0, 1), 100);
        lower_priority_move.path = vec![Position::new(0, 1), shared_to];
        lower_priority_move.step_from = Position::new(0, 1);
        lower_priority_move.step_to = shared_to;
        lower_priority_move.target_x_units = 500_000;
        lower_priority_move.target_y_units = 1_000_000;
        lower_priority_move.last_update_ms = 100;
        lower_priority_move.step_started_at_ms = 100;
        lower_priority_move.step_ends_at_ms = 150;
        lower_priority_move.orchestrator_priority = 3;

        let mut higher_priority_move = MovementState::new_at(Position::new(2, 1), 100);
        higher_priority_move.path = vec![Position::new(2, 1), shared_to];
        higher_priority_move.step_from = Position::new(2, 1);
        higher_priority_move.step_to = shared_to;
        higher_priority_move.target_x_units = 1_500_000;
        higher_priority_move.target_y_units = 1_000_000;
        higher_priority_move.last_update_ms = 100;
        higher_priority_move.step_started_at_ms = 100;
        higher_priority_move.step_ends_at_ms = 150;
        higher_priority_move.orchestrator_priority = 0;

        core.units.insert(
            lower_uuid_id,
            RuntimeUnit {
                instance_id: lower_uuid_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(lower_priority_move),
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
            },
        );
        core.units.insert(
            higher_uuid_id,
            RuntimeUnit {
                instance_id: higher_uuid_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(higher_priority_move),
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
            },
        );

        place_unit(&mut core, lower_uuid_id, Position::new(0, 1));
        place_unit(&mut core, higher_uuid_id, Position::new(2, 1));

        core.handle_move_steps_at(150, vec![(lower_uuid_id, 0), (higher_uuid_id, 0)]);

        assert_eq!(
            core.battlefield.position_of(higher_uuid_id),
            Some(shared_to)
        );
        assert_eq!(
            core.battlefield.position_of(lower_uuid_id),
            Some(Position::new(0, 1))
        );
        assert!(matches!(
            core.units.get(&lower_uuid_id).unwrap().action_state,
            ActionState::Yielding { until_ms: 160, .. }
        ));
    }

    #[test]
    fn static_friendly_blocker_enters_blocked_instead_of_wait_repath() {
        let mut core = new_core();

        let mover_id: UnitInstanceId = Uuid::from_u128(35).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(36).into();
        let blocked_tile = Position::new(1, 1);

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut mover = MovementState::new_at(Position::new(0, 1), 100);
        mover.path = vec![Position::new(0, 1), blocked_tile];
        mover.step_from = Position::new(0, 1);
        mover.step_to = blocked_tile;
        mover.target_x_units = 500_000;
        mover.target_y_units = 1_000_000;
        mover.last_update_ms = 100;
        mover.step_started_at_ms = 100;
        mover.step_ends_at_ms = 150;

        core.units.insert(
            mover_id,
            RuntimeUnit {
                instance_id: mover_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(mover),
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
            },
        );
        core.units.insert(
            blocker_id,
            RuntimeUnit {
                instance_id: blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 1_000_000,
                pos_y_units: 1_000_000,
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
            },
        );

        place_unit(&mut core, mover_id, Position::new(0, 1));
        place_unit(&mut core, blocker_id, blocked_tile);

        core.handle_move_steps_at(150, vec![(mover_id, 0)]);

        assert_eq!(
            core.battlefield.position_of(mover_id),
            Some(Position::new(0, 1))
        );
        assert!(matches!(
            core.units.get(&mover_id).unwrap().action_state,
            ActionState::Blocked { until_ms: 180, .. }
        ));
        assert!(
            !core.timeline.entries.iter().any(|entry| matches!(
                entry.event,
                TimelineEvent::MovementStopped {
                    unit_instance_id,
                    reason: MovementStopReason::WaitRepath,
                    ..
                } if unit_instance_id == mover_id
            )),
            "static friendly blocker should be treated as blocked congestion, not wait_repath"
        );
        assert!(core.event_queue.iter().any(|event| matches!(
            event,
            BattleEvent::MovementIntent { time_ms } if *time_ms == 180
        )));
    }

    #[test]
    fn due_mover_that_will_vacate_tile_is_retried_in_same_tick_before_yielding() {
        let mut core = new_core();

        let higher_priority_id: UnitInstanceId = Uuid::from_u128(40).into();
        let lower_priority_blocker_id: UnitInstanceId = Uuid::from_u128(41).into();

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut higher_priority_move = MovementState::new_at(Position::new(1, 1), 100);
        higher_priority_move.path = vec![Position::new(1, 1), Position::new(2, 1)];
        higher_priority_move.step_from = Position::new(1, 1);
        higher_priority_move.step_to = Position::new(2, 1);
        higher_priority_move.target_x_units = 1_500_000;
        higher_priority_move.target_y_units = 1_000_000;
        higher_priority_move.last_update_ms = 100;
        higher_priority_move.step_started_at_ms = 100;
        higher_priority_move.step_ends_at_ms = 150;
        higher_priority_move.orchestrator_priority = 0;

        let mut lower_priority_move = MovementState::new_at(Position::new(2, 1), 100);
        lower_priority_move.path = vec![Position::new(2, 1), Position::new(3, 1)];
        lower_priority_move.step_from = Position::new(2, 1);
        lower_priority_move.step_to = Position::new(3, 1);
        lower_priority_move.target_x_units = 2_500_000;
        lower_priority_move.target_y_units = 1_000_000;
        lower_priority_move.last_update_ms = 100;
        lower_priority_move.step_started_at_ms = 100;
        lower_priority_move.step_ends_at_ms = 150;
        lower_priority_move.orchestrator_priority = 3;

        core.units.insert(
            higher_priority_id,
            RuntimeUnit {
                instance_id: higher_priority_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 1_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(higher_priority_move),
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
            },
        );
        core.units.insert(
            lower_priority_blocker_id,
            RuntimeUnit {
                instance_id: lower_priority_blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(lower_priority_move),
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
            },
        );

        place_unit(&mut core, higher_priority_id, Position::new(1, 1));
        place_unit(&mut core, lower_priority_blocker_id, Position::new(2, 1));

        core.handle_move_steps_at(
            150,
            vec![(higher_priority_id, 0), (lower_priority_blocker_id, 0)],
        );

        assert_eq!(
            core.battlefield.position_of(higher_priority_id),
            Some(Position::new(2, 1))
        );
        assert_eq!(
            core.battlefield.position_of(lower_priority_blocker_id),
            Some(Position::new(3, 1))
        );
        assert!(matches!(
            core.units.get(&higher_priority_id).unwrap().action_state,
            ActionState::Idle
        ));
        assert!(
            !core.event_queue.iter().any(|event| matches!(
                event,
                BattleEvent::MovementIntent { time_ms } if *time_ms == 160
            )),
            "same-tick vacate should complete in-place instead of scheduling yield retry"
        );
    }

    #[test]
    fn next_step_reservation_retries_in_same_tick_after_lower_priority_blocker_moves() {
        let mut core = new_core();

        let higher_priority_id: UnitInstanceId = Uuid::from_u128(50).into();
        let lower_priority_blocker_id: UnitInstanceId = Uuid::from_u128(51).into();

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut higher_priority_move = MovementState::new_at(Position::new(0, 1), 100);
        higher_priority_move.path = vec![
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        higher_priority_move.step_from = Position::new(0, 1);
        higher_priority_move.step_to = Position::new(1, 1);
        higher_priority_move.target_x_units = 500_000;
        higher_priority_move.target_y_units = 1_000_000;
        higher_priority_move.last_update_ms = 100;
        higher_priority_move.step_started_at_ms = 100;
        higher_priority_move.step_ends_at_ms = 150;
        higher_priority_move.orchestrator_priority = 0;
        higher_priority_move.planned_continuation = Some(PlannedContinuation {
            step: Position::new(2, 1),
            owner_priority: 0,
            claim_priority: None,
            retry_budget: 1,
        });

        let mut lower_priority_move = MovementState::new_at(Position::new(2, 1), 100);
        lower_priority_move.path = vec![Position::new(2, 1), Position::new(3, 1)];
        lower_priority_move.step_from = Position::new(2, 1);
        lower_priority_move.step_to = Position::new(3, 1);
        lower_priority_move.target_x_units = 2_500_000;
        lower_priority_move.target_y_units = 1_000_000;
        lower_priority_move.last_update_ms = 100;
        lower_priority_move.step_started_at_ms = 100;
        lower_priority_move.step_ends_at_ms = 150;
        lower_priority_move.orchestrator_priority = 3;

        core.units.insert(
            higher_priority_id,
            RuntimeUnit {
                instance_id: higher_priority_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(higher_priority_move),
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
            },
        );
        core.units.insert(
            lower_priority_blocker_id,
            RuntimeUnit {
                instance_id: lower_priority_blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(lower_priority_move),
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
            },
        );

        place_unit(&mut core, higher_priority_id, Position::new(0, 1));
        place_unit(&mut core, lower_priority_blocker_id, Position::new(2, 1));

        core.handle_move_steps_at(
            150,
            vec![(higher_priority_id, 0), (lower_priority_blocker_id, 0)],
        );

        let moving = core.units.get(&higher_priority_id).unwrap();
        let ActionState::Moving(state) = &moving.action_state else {
            panic!(
                "expected higher-priority unit to keep moving, got {:?}",
                moving.action_state
            );
        };
        assert_eq!(
            core.battlefield.position_of(higher_priority_id),
            Some(Position::new(1, 1))
        );
        assert_eq!(state.step_to, Position::new(2, 1));
        assert!(state.step_ends_at_ms > 150);
        assert!(
            !matches!(moving.action_state, ActionState::Yielding { .. }),
            "planner continuation should retry next-step reservation in same tick before yielding"
        );
    }

    #[test]
    fn generic_next_step_uses_settled_same_tick_board_before_yielding() {
        let mut core = new_core();

        let higher_priority_id: UnitInstanceId = Uuid::from_u128(60).into();
        let lower_priority_blocker_id: UnitInstanceId = Uuid::from_u128(61).into();

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut higher_priority_move = MovementState::new_at(Position::new(0, 1), 100);
        higher_priority_move.path = vec![
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        higher_priority_move.step_from = Position::new(0, 1);
        higher_priority_move.step_to = Position::new(1, 1);
        higher_priority_move.target_x_units = 500_000;
        higher_priority_move.target_y_units = 1_000_000;
        higher_priority_move.last_update_ms = 100;
        higher_priority_move.step_started_at_ms = 100;
        higher_priority_move.step_ends_at_ms = 150;
        higher_priority_move.orchestrator_priority = 0;

        let mut lower_priority_move = MovementState::new_at(Position::new(2, 1), 100);
        lower_priority_move.path = vec![Position::new(2, 1), Position::new(3, 1)];
        lower_priority_move.step_from = Position::new(2, 1);
        lower_priority_move.step_to = Position::new(3, 1);
        lower_priority_move.target_x_units = 2_500_000;
        lower_priority_move.target_y_units = 1_000_000;
        lower_priority_move.last_update_ms = 100;
        lower_priority_move.step_started_at_ms = 100;
        lower_priority_move.step_ends_at_ms = 150;
        lower_priority_move.orchestrator_priority = 3;

        core.units.insert(
            higher_priority_id,
            RuntimeUnit {
                instance_id: higher_priority_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(higher_priority_move),
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
            },
        );
        core.units.insert(
            lower_priority_blocker_id,
            RuntimeUnit {
                instance_id: lower_priority_blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(lower_priority_move),
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
            },
        );

        place_unit(&mut core, higher_priority_id, Position::new(0, 1));
        place_unit(&mut core, lower_priority_blocker_id, Position::new(2, 1));

        core.handle_move_steps_at(
            150,
            vec![(higher_priority_id, 0), (lower_priority_blocker_id, 0)],
        );

        assert_eq!(
            core.battlefield.position_of(higher_priority_id),
            Some(Position::new(1, 1))
        );
        let moving = core.units.get(&higher_priority_id).unwrap();
        let ActionState::Moving(state) = &moving.action_state else {
            panic!(
                "expected higher-priority unit to keep moving after settled same-tick board, got {:?}",
                moving.action_state
            );
        };
        assert_eq!(state.step_to, Position::new(2, 1));
        assert!(state.step_ends_at_ms > 150);
        assert!(
            !matches!(moving.action_state, ActionState::Yielding { .. }),
            "generic next-step should use settled same-tick board state before falling back to yield"
        );
    }

    #[test]
    fn exhausted_continuation_budget_does_not_block_settled_same_tick_progress() {
        let mut core = new_core();

        let higher_priority_id: UnitInstanceId = Uuid::from_u128(62).into();
        let lower_priority_blocker_id: UnitInstanceId = Uuid::from_u128(63).into();

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut higher_priority_move = MovementState::new_at(Position::new(0, 1), 100);
        higher_priority_move.path = vec![
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        higher_priority_move.step_from = Position::new(0, 1);
        higher_priority_move.step_to = Position::new(1, 1);
        higher_priority_move.target_x_units = 500_000;
        higher_priority_move.target_y_units = 1_000_000;
        higher_priority_move.last_update_ms = 100;
        higher_priority_move.step_started_at_ms = 100;
        higher_priority_move.step_ends_at_ms = 150;
        higher_priority_move.orchestrator_priority = 0;
        higher_priority_move.planned_continuation = Some(PlannedContinuation {
            step: Position::new(2, 1),
            owner_priority: 0,
            claim_priority: None,
            retry_budget: 0,
        });

        let mut lower_priority_move = MovementState::new_at(Position::new(2, 1), 100);
        lower_priority_move.path = vec![Position::new(2, 1), Position::new(3, 1)];
        lower_priority_move.step_from = Position::new(2, 1);
        lower_priority_move.step_to = Position::new(3, 1);
        lower_priority_move.target_x_units = 2_500_000;
        lower_priority_move.target_y_units = 1_000_000;
        lower_priority_move.last_update_ms = 100;
        lower_priority_move.step_started_at_ms = 100;
        lower_priority_move.step_ends_at_ms = 150;
        lower_priority_move.orchestrator_priority = 3;

        core.units.insert(
            higher_priority_id,
            RuntimeUnit {
                instance_id: higher_priority_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(higher_priority_move),
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
            },
        );
        core.units.insert(
            lower_priority_blocker_id,
            RuntimeUnit {
                instance_id: lower_priority_blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(lower_priority_move),
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
            },
        );

        place_unit(&mut core, higher_priority_id, Position::new(0, 1));
        place_unit(&mut core, lower_priority_blocker_id, Position::new(2, 1));

        core.handle_move_steps_at(
            150,
            vec![(higher_priority_id, 0), (lower_priority_blocker_id, 0)],
        );

        assert_eq!(
            core.battlefield.position_of(higher_priority_id),
            Some(Position::new(1, 1))
        );
        let moving = core.units.get(&higher_priority_id).unwrap();
        let ActionState::Moving(state) = &moving.action_state else {
            panic!(
                "expected higher-priority unit to keep moving after settled same-tick board despite exhausted budget, got {:?}",
                moving.action_state
            );
        };
        assert_eq!(state.step_to, Position::new(2, 1));
        assert!(state.step_ends_at_ms > 150);
        assert!(
            !matches!(moving.action_state, ActionState::Yielding { .. }),
            "exhausted retry budget should not matter when the settled same-tick board already opens the next step"
        );
    }

    #[test]
    fn next_step_friendly_blocker_enters_blocked_instead_of_wait_repath() {
        let mut core = new_core();

        let mover_id: UnitInstanceId = Uuid::from_u128(52).into();
        let blocker_id: UnitInstanceId = Uuid::from_u128(53).into();

        let mut stats = UnitStats::with_values(10, 10, 1, 0, 1);
        stats.move_speed_units_per_ms = 10_000;

        let mut mover = MovementState::new_at(Position::new(0, 1), 100);
        mover.path = vec![
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        mover.step_from = Position::new(0, 1);
        mover.step_to = Position::new(1, 1);
        mover.target_x_units = 500_000;
        mover.target_y_units = 1_000_000;
        mover.last_update_ms = 100;
        mover.step_started_at_ms = 100;
        mover.step_ends_at_ms = 150;

        core.units.insert(
            mover_id,
            RuntimeUnit {
                instance_id: mover_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 0,
                pos_y_units: 1_000_000,
                move_epoch: 0,
                action_state: ActionState::Moving(mover),
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
            },
        );
        core.units.insert(
            blocker_id,
            RuntimeUnit {
                instance_id: blocker_id,
                owner: Side::Player,
                base_uuid: Uuid::nil(),
                stats,
                pos_x_units: 2_000_000,
                pos_y_units: 1_000_000,
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
            },
        );

        place_unit(&mut core, mover_id, Position::new(0, 1));
        place_unit(&mut core, blocker_id, Position::new(2, 1));

        core.handle_move_steps_at(150, vec![(mover_id, 0)]);

        assert_eq!(
            core.battlefield.position_of(mover_id),
            Some(Position::new(1, 1))
        );
        assert!(matches!(
            core.units.get(&mover_id).unwrap().action_state,
            ActionState::Blocked { until_ms: 180, .. }
        ));
        assert!(
            !core.timeline.entries.iter().any(|entry| matches!(
                entry.event,
                TimelineEvent::MovementStopped {
                    unit_instance_id,
                    reason: MovementStopReason::WaitRepath,
                    ..
                } if unit_instance_id == mover_id
            )),
            "friendly congestion on next-step reservation should block, not wait_repath"
        );
    }
}
