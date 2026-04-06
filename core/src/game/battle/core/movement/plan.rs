use std::cmp::Ordering;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::DeliveryDef,
        battle::{
            battlefield::bfs::BfsMap,
            core::{
                movement::{boundary_target_units, tile_center_units},
                BattleCore,
            },
            ids::UnitInstanceId,
            timeline::AttackDelivery,
        },
        determinism,
        enums::Side,
    },
};

use super::{ActionState, EnemyChasePlan, MovementState};

impl BattleCore {
    fn compare_destination_preference(
        owner: Side,
        mover_start: Position,
        enemy_pos: Position,
        a: Position,
        b: Position,
    ) -> Ordering {
        let forward_cmp = match owner {
            Side::Player => a.y.cmp(&b.y),
            Side::Opponent => b.y.cmp(&a.y),
        };

        forward_cmp
            .then_with(|| a.chebyshev(&enemy_pos).cmp(&b.chebyshev(&enemy_pos)))
            .then_with(|| (a.x - enemy_pos.x).abs().cmp(&(b.x - enemy_pos.x).abs()))
            .then_with(|| {
                (a.x - mover_start.x)
                    .abs()
                    .cmp(&(b.x - mover_start.x).abs())
            })
            .then_with(|| a.x.cmp(&b.x))
            .then_with(|| a.y.cmp(&b.y))
    }

    fn compare_plan_preference(
        owner: Side,
        mover_start: Position,
        enemy_a: Position,
        enemy_b: Position,
        best_dest_a: Position,
        best_dest_b: Position,
    ) -> Ordering {
        let enemy_forward_cmp = match owner {
            Side::Player => enemy_a.y.cmp(&enemy_b.y),
            Side::Opponent => enemy_b.y.cmp(&enemy_a.y),
        };

        enemy_forward_cmp
            .then_with(|| {
                (enemy_a.x - mover_start.x)
                    .abs()
                    .cmp(&(enemy_b.x - mover_start.x).abs())
            })
            .then_with(|| match owner {
                Side::Player => best_dest_a.y.cmp(&best_dest_b.y),
                Side::Opponent => best_dest_b.y.cmp(&best_dest_a.y),
            })
            .then_with(|| {
                best_dest_a
                    .chebyshev(&enemy_a)
                    .cmp(&best_dest_b.chebyshev(&enemy_b))
            })
            .then_with(|| {
                (best_dest_a.x - enemy_a.x)
                    .abs()
                    .cmp(&(best_dest_b.x - enemy_b.x).abs())
            })
            .then_with(|| {
                (best_dest_a.x - mover_start.x)
                    .abs()
                    .cmp(&(best_dest_b.x - mover_start.x).abs())
            })
            .then_with(|| best_dest_a.x.cmp(&best_dest_b.x))
            .then_with(|| best_dest_a.y.cmp(&best_dest_b.y))
    }

    pub fn choose_attack_target_in_range(
        &self,
        attacker_owner: Side,
        attacker_pos: Position,
        range_tiles: u8,
    ) -> Option<UnitInstanceId> {
        let mut best: Option<(i32, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() {
                continue;
            }

            if unit.owner == attacker_owner {
                continue;
            }

            let Some(unit_pos) = self.battlefield.position_of(unit.instance_id) else {
                continue;
            };
            let d = attacker_pos.chebyshev(&unit_pos);
            if d > range_tiles as i32 {
                continue;
            }

            match best {
                None => best = Some((d, unit.instance_id)),
                Some((best_d, _)) if d < best_d => best = Some((d, unit.instance_id)),
                Some((best_d, best_id))
                    if d == best_d && unit.instance_id.as_bytes() < best_id.as_bytes() =>
                {
                    best = Some((d, unit.instance_id))
                }
                _ => {}
            }
        }

        best.map(|(_, id)| id)
    }

    pub fn basic_attack_range_tiles(&self, unit_base_uuid: Uuid) -> u8 {
        self.game_data
            .abnormality_data
            .get_by_uuid(&unit_base_uuid)
            .map(|m| m.basic_attack.range_tiles)
            .unwrap_or(1)
            .max(1)
    }

    pub fn basic_attack_delivery(&self, unit_base_uuid: Uuid) -> AttackDelivery {
        match self
            .game_data
            .abnormality_data
            .get_by_uuid(&unit_base_uuid)
            .map(|m| m.basic_attack.delivery.clone())
            .unwrap_or(DeliveryDef::Instant)
        {
            DeliveryDef::Instant => AttackDelivery::Instant,
            DeliveryDef::Projectile { .. } => AttackDelivery::Projectile,
        }
    }

    pub fn compute_movement_intents(&mut self, now_ms: u64) {
        let mut unit_ids: Vec<UnitInstanceId> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        for unit_id in unit_ids {
            let Some(unit) = self.units.get(&unit_id) else {
                continue;
            };

            let repath_counter = match &unit.action_state {
                ActionState::Idle => 0,
                ActionState::WaitRepath {
                    until_ms,
                    repath_counter,
                } => {
                    if *until_ms > now_ms {
                        continue;
                    }
                    *repath_counter
                }
                _ => continue,
            };

            // Hard CC / action locks: do not plan movement while locked.
            if !unit.action_locks.can_move(now_ms) {
                continue;
            }
            if unit.stats.move_speed_units_per_ms == 0 {
                continue;
            }

            let Some(start_pos) = self.battlefield.position_of(unit_id) else {
                continue;
            };

            let (owner, base_uuid, current_target) =
                (unit.owner, unit.base_uuid, unit.current_target);

            self.battlefield.cancel_reservation(unit_id);

            let range_tiles = self.basic_attack_range_tiles(base_uuid);

            if let Some(target_id) =
                self.persisted_target_in_range(owner, current_target, start_pos, range_tiles)
            {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = Some(target_id);
                    unit.action_state = ActionState::Idle;
                }
                continue;
            }

            if let Some(target_id) =
                self.choose_attack_target_in_range(owner, start_pos, range_tiles)
            {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = Some(target_id);
                    unit.action_state = ActionState::Idle;
                }
                continue;
            }

            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.current_target = None;
            }

            let battlefield = &self.battlefield;
            let bfs = match battlefield.bfs_map_8_for_side(start_pos, owner, |pos, tile| {
                if tile.occupant().is_some() {
                    return false;
                }
                if tile.reservation().is_some() && battlefield.reservation_blocks_for(unit_id, pos)
                {
                    return false;
                }
                true
            }) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut enemies: Vec<(UnitInstanceId, Position)> = self
                .units
                .values()
                .filter_map(|u| {
                    if u.is_dead() || u.owner == owner {
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
                owner,
                start_pos,
                enemies,
                range_tiles,
                |field, pos| {
                    field
                        .idx(pos)
                        .ok()
                        .is_some_and(|idx| field.is_empty_tile(idx))
                },
            );

            let mut reserved: Option<(UnitInstanceId, Position, Vec<Position>)> = None;
            for plan in plans {
                for dest in plan.dest_candidates {
                    let Some(path) = bfs.reconstruct_path_to(dest) else {
                        continue;
                    };
                    if path.len() < 2 {
                        continue;
                    }

                    let first_step = path[1];
                    if self
                        .battlefield
                        .reserve(unit_id, first_step, now_ms)
                        .is_ok()
                    {
                        reserved = Some((plan.enemy_id, dest, path));
                        break;
                    }
                }
                if reserved.is_some() {
                    break;
                }
            }

            let Some((enemy_id, dest, path)) = reserved else {
                const REPATH_BASE_DELAY_MS: u64 = 100;

                let until_ms = now_ms.saturating_add(REPATH_BASE_DELAY_MS).saturating_add(
                    determinism::repath_jitter_ms(
                        self.seed,
                        unit_id,
                        repath_counter.wrapping_add(1),
                    ),
                );
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.action_state = ActionState::WaitRepath {
                        until_ms,
                        repath_counter: repath_counter.wrapping_add(1),
                    };
                }
                self.schedule_movement_intent(until_ms);
                continue;
            };

            let speed_units_per_ms = self
                .units
                .get(&unit_id)
                .map(|u| u.stats.move_speed_units_per_ms)
                .unwrap_or(1)
                .max(1);

            let mut movement = MovementState::new_at(start_pos, now_ms);
            movement.path = path;
            movement.reserved_destination = Some(dest);
            movement.repath_counter = repath_counter;
            movement.step_from = start_pos;
            movement.step_to = movement.path[1];

            // Move towards the boundary between `step_from` and `step_to` (occupancy switches at that boundary).
            let (target_x, target_y) = boundary_target_units(movement.step_from, movement.step_to);
            movement.target_x_units = target_x;
            movement.target_y_units = target_y;

            let (pos_x, pos_y) = self
                .units
                .get(&unit_id)
                .map(|u| (u.pos_x_units, u.pos_y_units))
                .unwrap_or_else(|| tile_center_units(start_pos));

            let dist_x = (target_x - pos_x).unsigned_abs();
            let dist_y = (target_y - pos_y).unsigned_abs();
            let dist_units = dist_x.max(dist_y);
            let denom = speed_units_per_ms as u64;
            let dt_ms = dist_units.div_ceil(denom).max(1);
            let step_ends_at_ms = now_ms.saturating_add(dt_ms);
            movement.step_started_at_ms = now_ms;
            movement.step_ends_at_ms = step_ends_at_ms;

            if let Some(unit) = self.units.get_mut(&unit_id) {
                unit.current_target = Some(enemy_id);
                unit.move_epoch = unit.move_epoch.wrapping_add(1);
                unit.action_state = ActionState::Moving(movement);
            }
            self.schedule_move_step_at(unit_id, step_ends_at_ms);
        }
    }

    pub fn destination_candidates_in_order(
        &self,
        bfs: &BfsMap,
        owner: Side,
        mover_start: Position,
        enemy_pos: Position,
        range_tiles: u8,
        mut is_empty_tile: impl FnMut(&crate::game::battle::battlefield::Battlefield, Position) -> bool,
    ) -> Vec<(u32, Position)> {
        let mut out: Vec<(u32, Position)> = Vec::new();
        let r = range_tiles as i32;
        for y in (enemy_pos.y - r)..=(enemy_pos.y + r) {
            for x in (enemy_pos.x - r)..=(enemy_pos.x + r) {
                let pos = Position::new(x, y);
                if !self.battlefield.in_bounds(pos)
                    || pos.chebyshev(&enemy_pos) > r
                    || !is_empty_tile(&self.battlefield, pos)
                {
                    continue;
                }
                if let Some(d) = bfs.distance_to(pos) {
                    out.push((d, pos));
                }
            }
        }
        out.sort_by(|(da, pa), (db, pb)| {
            da.cmp(db).then_with(|| {
                Self::compare_destination_preference(owner, mover_start, enemy_pos, *pa, *pb)
            })
        });
        out
    }

    pub fn formulate_enemy_chase_plan(
        &self,
        bfs_map: &BfsMap,
        owner: Side,
        mover_start: Position,
        enemies: Vec<(UnitInstanceId, Position)>,
        range_tiles: u8,
        mut is_empty_tile: impl FnMut(&crate::game::battle::battlefield::Battlefield, Position) -> bool,
    ) -> Vec<EnemyChasePlan> {
        let mut out: Vec<EnemyChasePlan> = Vec::new();

        for (enemy_id, enemy_pos) in enemies {
            let candidates = self.destination_candidates_in_order(
                bfs_map,
                owner,
                mover_start,
                enemy_pos,
                range_tiles,
                &mut is_empty_tile,
            );

            let Some((chase_dist, best_dest)) = candidates.first().copied() else {
                continue;
            };
            out.push(EnemyChasePlan {
                enemy_id,
                enemy_pos,
                chase_dist,
                best_dest,
                dest_candidates: candidates.into_iter().map(|(_, p)| p).collect(),
            });
        }

        out.sort_by(|a, b| {
            a.chase_dist
                .cmp(&b.chase_dist)
                .then_with(|| {
                    Self::compare_plan_preference(
                        owner,
                        mover_start,
                        a.enemy_pos,
                        b.enemy_pos,
                        a.best_dest,
                        b.best_dest,
                    )
                })
                .then_with(|| a.enemy_id.as_bytes().cmp(b.enemy_id.as_bytes()))
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_plan_preference_is_antisymmetric_for_tied_inputs() {
        let mover_start = Position::new(1, 0);
        let enemy_a = Position::new(0, 0);
        let enemy_b = Position::new(2, 0);
        let best_dest_a = Position::new(0, 0);
        let best_dest_b = Position::new(1, 0);

        let ab = BattleCore::compare_plan_preference(
            Side::Player,
            mover_start,
            enemy_a,
            enemy_b,
            best_dest_a,
            best_dest_b,
        );
        let ba = BattleCore::compare_plan_preference(
            Side::Player,
            mover_start,
            enemy_b,
            enemy_a,
            best_dest_b,
            best_dest_a,
        );

        assert_eq!(ab, ba.reverse());
    }
}
