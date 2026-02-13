use std::collections::HashSet;

use crate::{
    ecs::resources::Position,
    game::battle::{
        core::{
            movement::{boundary_target_units, tile_center_units, ActionState},
            BattleCore,
        },
        enums::BattleEvent,
        ids::UnitInstanceId,
        timeline::{MovementStopReason, TimelineEvent},
    },
    game::determinism,
};

impl BattleCore {
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
        if unit.is_dead() {
            return;
        }
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
        ((dist_units + speed_units_per_ms - 1) / speed_units_per_ms).max(1)
    }

    fn stop_moving_unit_on_target_in_range_at_tile(
        &mut self,
        now_ms: u64,
        unit_id: UnitInstanceId,
        tile: Position,
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

        let range_tiles = self.basic_attack_range_tiles(unit.base_uuid);
        let Some(target_id) = self.choose_attack_target_in_range(unit.owner, tile, range_tiles)
        else {
            return false;
        };

        self.record_movement_stopped(now_ms, unit_id, MovementStopReason::TargetAcquired, None);
        self.battlefield.cancel_reservation(unit_id);
        if let Some(unit) = self.units.get_mut(&unit_id) {
            unit.current_target = Some(target_id);
            unit.move_epoch = unit.move_epoch.wrapping_add(1);
            unit.action_state = ActionState::Idle;
        }
        self.event_queue
            .push(BattleEvent::MovementIntent { time_ms: now_ms });

        true
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

            let Some(tile) = self.battlefield.position_of(unit_id) else {
                continue;
            };

            self.stop_moving_unit_on_target_in_range_at_tile(now_ms, unit_id, tile);
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

        const REPATH_BASE_DELAY_MS: u64 = 100;

        let abort_to_idle = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            core.battlefield.cancel_reservation(unit_id);
            if let Some(unit) = core.units.get_mut(&unit_id) {
                unit.move_epoch = unit.move_epoch.wrapping_add(1);
                unit.action_state = ActionState::Idle;
            }
            core.record_movement_stopped(now_ms, unit_id, MovementStopReason::InvalidState, None);
            core.event_queue
                .push(BattleEvent::MovementIntent { time_ms: now_ms });
        };

        let abort_to_wait_repath = |core: &mut BattleCore, unit_id: UnitInstanceId| {
            core.battlefield.cancel_reservation(unit_id);

            let Some(unit) = core.units.get_mut(&unit_id) else {
                return;
            };

            let repath_counter = match &unit.action_state {
                ActionState::Moving(state) => state.repath_counter.wrapping_add(1),
                ActionState::WaitRepath { repath_counter, .. } => repath_counter.wrapping_add(1),
                _ => 1,
            };

            let jitter_ms = determinism::repath_jitter_ms(core.seed, unit_id, repath_counter);
            let until_ms = now_ms
                .saturating_add(REPATH_BASE_DELAY_MS)
                .saturating_add(jitter_ms);
            unit.action_state = ActionState::WaitRepath {
                until_ms,
                repath_counter,
            };
            unit.move_epoch = unit.move_epoch.wrapping_add(1);

            core.record_movement_stopped(
                now_ms,
                unit_id,
                MovementStopReason::WaitRepath,
                Some(until_ms),
            );
            core.event_queue
                .push(BattleEvent::MovementIntent { time_ms: until_ms });
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

        let mut moving_unit: Vec<(UnitInstanceId, Position, Position)> = Vec::new();
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
            let remaining_x = (state.target_x_units - pos_x).unsigned_abs();
            let remaining_y = (state.target_y_units - pos_y).unsigned_abs();
            if remaining_x.max(remaining_y) > 0 {
                let speed = unit.stats.move_speed_units_per_ms.max(1);
                let next_ms = now_ms.saturating_add(Self::time_to_reach_target_ms(
                    pos_x,
                    pos_y,
                    state.target_x_units,
                    state.target_y_units,
                    speed,
                ));
                if let Some(unit) = self.units.get_mut(unit_id) {
                    if let ActionState::Moving(state) = &mut unit.action_state {
                        state.step_started_at_ms = now_ms;
                        state.step_ends_at_ms = next_ms;
                    }
                }
                self.schedule_move_step_at(*unit_id, next_ms);
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
            moving_unit.push((*unit_id, from, to));
        }

        let mut intents: Vec<(UnitInstanceId, Position, Position)> = moving_unit;
        intents.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let mut claimed_to: HashSet<Position> = HashSet::new();

        for (unit_id, from, to) in intents {
            if self.units.get(&unit_id).is_none_or(|u| u.is_dead()) {
                continue;
            }

            if claimed_to.contains(&to) {
                abort_to_wait_repath(self, unit_id);
                continue;
            }

            if self
                .battlefield
                .position_of(unit_id)
                .is_some_and(|p| p != from)
            {
                abort_to_idle(self, unit_id);
                continue;
            }

            let mut cancel_soft_reservation_of: Option<UnitInstanceId> = None;
            if let Some(reservation) = self.battlefield.reservation_at(to) {
                if reservation.unit != unit_id {
                    if self.battlefield.reservation_blocks_for(unit_id, to) {
                        abort_to_wait_repath(self, unit_id);
                        continue;
                    }
                    cancel_soft_reservation_of = Some(reservation.unit);
                }
            }

            if self.battlefield.occupant(to).ok().flatten().is_some() {
                abort_to_wait_repath(self, unit_id);
                continue;
            }

            if self.battlefield.move_unit(unit_id, to).is_err() {
                abort_to_wait_repath(self, unit_id);
                continue;
            }

            claimed_to.insert(to);

            if let Some(reserver) = cancel_soft_reservation_of {
                self.battlefield.cancel_reservation(reserver);
            }

            self.record_timeline(
                now_ms,
                TimelineEvent::UnitMoved {
                    unit_instance_id: unit_id,
                    from,
                    to,
                },
            );

            if self.stop_moving_unit_on_target_in_range_at_tile(now_ms, unit_id, to) {
                continue;
            }

            let mut next_step_ms: Option<u64> = None;

            if let Some(unit) = self.units.get_mut(&unit_id) {
                if let ActionState::Moving(state) = &mut unit.action_state {
                    state.path_cursor = state.path_cursor.saturating_add(1);

                    state.step_from = to;
                    state.step_started_at_ms = now_ms;

                    let cursor = state.path_cursor as usize;
                    let arrived =
                        state.reserved_destination == Some(to) || cursor + 1 >= state.path.len();
                    if arrived {
                        self.battlefield.cancel_reservation(unit_id);
                        unit.move_epoch = unit.move_epoch.wrapping_add(1);
                        unit.action_state = ActionState::Idle;
                        self.record_movement_stopped(
                            now_ms,
                            unit_id,
                            MovementStopReason::Arrived,
                            None,
                        );
                        self.event_queue
                            .push(BattleEvent::MovementIntent { time_ms: now_ms });
                    } else {
                        state.step_to = state.path[cursor + 1];
                        let (target_x, target_y) =
                            boundary_target_units(state.step_from, state.step_to);
                        state.target_x_units = target_x;
                        state.target_y_units = target_y;
                        let speed = unit.stats.move_speed_units_per_ms.max(1);
                        state.step_ends_at_ms =
                            now_ms.saturating_add(Self::time_to_reach_target_ms(
                                unit.pos_x_units,
                                unit.pos_y_units,
                                target_x,
                                target_y,
                                speed,
                            ));
                        next_step_ms = Some(state.step_ends_at_ms);
                    }
                }
            }

            if let Some(ms) = next_step_ms {
                self.schedule_move_step_at(unit_id, ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::core::movement::MovementState;
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

        Arc::new(GameDataBase::new(
            Arc::new(AbnormalityDatabase::new(vec![])),
            Arc::new(ArtifactDatabase::new(vec![])),
            Arc::new(EquipmentDatabase::new(vec![])),
            Arc::new(ShopDatabase::new(vec![])),
            Arc::new(BonusDatabase::new(vec![])),
            Arc::new(RandomEventDatabase::new(vec![])),
            Arc::new(PveEncounterDatabase::new(vec![])),
            Arc::new(SkillDatabase::new(vec![])),
            event_pools,
        ))
    }

    fn new_core() -> BattleCore {
        let deck = empty_deck();
        BattleCore::new(&deck, &deck, empty_game_data(), (4, 4), 123)
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
                pending_autocast: None,
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
}
