use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        battle::{
            battlefield::bfs::BfsMap,
            core::{
                movement::{boundary_target_units, tile_center_units},
                BattleCore,
            },
            ids::UnitInstanceId,
        },
        enums::Side,
    },
};

use super::{ActionState, EnemyChasePlan, MovementState};

impl BattleCore {
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

            let Some(start_pos) = self.battlefield.position_of(unit_id) else {
                continue;
            };

            let (owner, base_uuid) = (unit.owner, unit.base_uuid);

            self.battlefield.cancel_reservation(unit_id);

            let range_tiles = self.basic_attack_range_tiles(base_uuid);

            if let Some(target_id) =
                self.choose_attack_target_in_range(owner, start_pos, range_tiles)
            {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = Some(target_id);
                    unit.action_state = ActionState::Idle;
                }
                continue;
            }

            let battlefield = &self.battlefield;
            let bfs = match battlefield.bfs_map_8(start_pos, |pos, tile| {
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
                        let Some(pos) = self.battlefield.position_of(u.instance_id) else {
                            return None;
                        };
                        Some((u.instance_id, pos))
                    }
                })
                .collect();
            enemies.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let plans =
                self.formulate_enemy_chase_plan(&bfs, enemies, range_tiles, |field, pos| {
                    field
                        .idx(pos)
                        .ok()
                        .is_some_and(|idx| field.is_empty_tile(idx))
                });

            let mut reserved: Option<(UnitInstanceId, Position)> = None;
            for plan in plans {
                for dest in plan.dest_candidates {
                    if self.battlefield.reserve(unit_id, dest, now_ms).is_ok() {
                        reserved = Some((plan.enemy_id, dest));
                        break;
                    }
                }
                if reserved.is_some() {
                    break;
                }
            }

            let Some((enemy_id, dest)) = reserved else {
                continue;
            };

            let Some(path) = bfs.reconstruct_path_to(dest) else {
                continue;
            };
            if path.len() < 2 {
                continue;
            }

            let speed_units_per_ms = self
                .units
                .get(&unit_id)
                .map(|u| u.stats.move_speed_units_per_ms.max(1))
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
            let dt_ms = ((dist_units + denom - 1) / denom).max(1);
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
            da.cmp(db)
                .then_with(|| pa.y.cmp(&pb.y))
                .then_with(|| pa.x.cmp(&pb.x))
        });
        out
    }

    pub fn formulate_enemy_chase_plan(
        &self,
        bfs_map: &BfsMap,
        enemies: Vec<(UnitInstanceId, Position)>,
        range_tiles: u8,
        mut is_empty_tile: impl FnMut(&crate::game::battle::battlefield::Battlefield, Position) -> bool,
    ) -> Vec<EnemyChasePlan> {
        let mut out: Vec<EnemyChasePlan> = Vec::new();

        for (enemy_id, enemy_pos) in enemies {
            let candidates = self.destination_candidates_in_order(
                bfs_map,
                enemy_pos,
                range_tiles,
                &mut is_empty_tile,
            );

            let Some((chase_dist, _)) = candidates.first().copied() else {
                continue;
            };
            out.push(EnemyChasePlan {
                enemy_id,
                chase_dist,
                dest_candidates: candidates.into_iter().map(|(_, p)| p).collect(),
            });
        }

        out.sort_by(|a, b| {
            a.chase_dist
                .cmp(&b.chase_dist)
                .then_with(|| a.enemy_id.as_bytes().cmp(b.enemy_id.as_bytes()))
        });
        out
    }
}
