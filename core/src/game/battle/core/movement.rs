use std::collections::{BTreeSet, HashMap, HashSet};

use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        battle::{
            core::RuntimeUnit,
            enums::BattleEvent,
            runtime_field::{BfsMap, RuntimeField},
            timeline::TimelineEvent,
        },
        determinism,
        enums::Side,
    },
};

use super::{BattleCore, MoveState};

const TILE_UNITS_PER_TILE: u64 = 1_000_000;
const SOFT_OCCUPIED_MS: u64 = 1000;

fn ceil_div_u64(a: u64, b: u64) -> u64 {
    if a == 0 {
        return 0;
    }
    (a + b.saturating_sub(1)) / b.max(1)
}

fn battlefield_forward_vector(owner: Side) -> (i32, i32) {
    // TFT-like orientation (for now): opponent is "top", player is "bottom".
    // If the game later switches to a left/right layout, centralize the change here.
    match owner {
        Side::Player => (0, 1),
        Side::Opponent => (0, -1),
    }
}

#[derive(Debug, Clone)]
struct EnemyChasePlan {
    enemy_id: Uuid,
    chase_dist: u32,
    dest_candidates: Vec<Position>,
}

#[derive(Debug, Clone)]
struct OccupancyState {
    width: u8,
    height: u8,
    by_idx: Vec<BTreeSet<Uuid>>,
}

impl OccupancyState {
    fn new(width: u8, height: u8) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            by_idx: vec![BTreeSet::new(); len],
        }
    }

    fn idx(&self, pos: Position) -> Option<usize> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i32 || pos.y >= self.height as i32 {
            return None;
        }
        Some((pos.y as usize) * (self.width as usize) + (pos.x as usize))
    }

    fn insert(&mut self, unit: Uuid, pos: Position) {
        let Some(idx) = self.idx(pos) else {
            return;
        };
        self.by_idx[idx].insert(unit);
    }

    fn remove(&mut self, unit: Uuid, pos: Position) {
        let Some(idx) = self.idx(pos) else {
            return;
        };
        self.by_idx[idx].remove(&unit);
    }

    fn occupants(&self, pos: Position) -> Option<&BTreeSet<Uuid>> {
        let idx = self.idx(pos)?;
        Some(&self.by_idx[idx])
    }

    fn is_empty(&self, pos: Position) -> bool {
        self.occupants(pos).map(|s| s.is_empty()).unwrap_or(true)
    }

    fn len(&self, pos: Position) -> usize {
        self.occupants(pos).map(|s| s.len()).unwrap_or(0)
    }

    fn primary(&self, pos: Position) -> Option<Uuid> {
        self.occupants(pos).and_then(|s| s.iter().next().copied())
    }

    fn contains(&self, unit: Uuid, pos: Position) -> bool {
        self.occupants(pos)
            .map(|s| s.contains(&unit))
            .unwrap_or(false)
    }

    fn from_units(field: &RuntimeField, units: &HashMap<Uuid, RuntimeUnit>) -> Self {
        let mut occ = Self::new(field.width, field.height);
        for unit in units.values() {
            if unit.stats.current_health == 0 {
                continue;
            }
            // Ghost is intentionally non-blocking.
            if matches!(unit.move_state, MoveState::Ghost) {
                continue;
            }
            occ.insert(unit.instance_id, unit.position);
        }
        occ
    }
}

fn is_unit_moving_for_bfs(units: &HashMap<Uuid, RuntimeUnit>, id: Uuid) -> bool {
    matches!(
        units.get(&id).map(|u| u.move_state),
        Some(MoveState::Moving)
    )
}

fn destination_candidates_in_order(
    field: &RuntimeField,
    bfs: &BfsMap,
    enemy_pos: Position,
    range_tiles: u8,
    mut is_empty_tile: impl FnMut(Position) -> bool,
) -> Vec<(u32, Position)> {
    let mut out: Vec<(u32, Position)> = Vec::new();

    for y in 0..(field.height as i32) {
        for x in 0..(field.width as i32) {
            let pos = Position::new(x, y);
            // pos 를 기준으로 enemy pos 의 거리가 유닛 사거리보다 길다면 무시
            if pos.chebyshev(&enemy_pos) > range_tiles as i32 {
                continue;
            }
            if !is_empty_tile(pos) {
                continue;
            }

            let Some(dist) = bfs.distance_to(pos) else {
                continue;
            };

            out.push((dist, pos));
        }
    }

    out.sort_by(|(da, pa), (db, pb)| {
        da.cmp(db)
            .then_with(|| pa.y.cmp(&pb.y))
            .then_with(|| pa.x.cmp(&pb.x))
    });
    out
}

fn enemy_plans_in_order(
    field: &RuntimeField,
    bfs: &BfsMap,
    enemies: &[(Uuid, Position)],
    range_tiles: u8,
    mut is_empty_tile: impl FnMut(Position) -> bool,
) -> Vec<EnemyChasePlan> {
    let mut out: Vec<EnemyChasePlan> = Vec::new();

    for (enemy_id, enemy_pos) in enemies {
        let candidates = destination_candidates_in_order(
            field,
            bfs,
            *enemy_pos,
            range_tiles,
            &mut is_empty_tile,
        );
        let Some((chase_dist, _)) = candidates.first().copied() else {
            continue;
        };
        out.push(EnemyChasePlan {
            enemy_id: *enemy_id,
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

impl BattleCore {
    fn basic_attack_range_tiles(&self, unit_base_uuid: Uuid) -> u8 {
        self.game_data
            .abnormality_data
            .get_by_uuid(&unit_base_uuid)
            .map(|m| m.basic_attack.range_tiles)
            .unwrap_or(1)
            .max(1)
    }

    fn movement_jitter_ms(&self, unit_id: Uuid, repath_counter: u32) -> u64 {
        // Spec: jitter_ms = hash(run_seed, unit_uuid, repath_counter) % 17
        // We implement this deterministically using uuid_v4_from_seed.
        const MOVEMENT_JITTER_NS: u64 = 0x4D_4F_56_45_4A_49_54u64; // "MOVEJIT"

        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&unit_id.as_bytes()[..8]);
        let namespace = u64::from_be_bytes(bytes) ^ MOVEMENT_JITTER_NS;

        let u =
            determinism::uuid_v4_from_seed(self.movement_seed, namespace, repath_counter as u64);
        (u.as_bytes()[0] as u64) % 17
    }

    fn choose_attack_target_in_range(
        &self,
        attacker_owner: Side,
        attacker_pos: Position,
        range_tiles: u8,
    ) -> Option<Uuid> {
        let mut best: Option<(i32, Uuid)> = None;
        for unit in self.units.values() {
            if unit.stats.current_health == 0 {
                continue;
            }
            if unit.owner == attacker_owner {
                continue;
            }
            let d = attacker_pos.chebyshev(&unit.position);
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

    pub(super) fn schedule_movement_intent(&mut self, time_ms: u64) {
        self.event_queue
            .push(BattleEvent::MovementIntent { time_ms });
    }

    pub(super) fn expire_soft_occupancy(&mut self, now_ms: u64) {
        let occ = OccupancyState::from_units(&self.movement_field, &self.units);

        for unit in self.units.values_mut() {
            if unit.stats.current_health == 0 {
                unit.soft_until_ms = None;
                continue;
            }

            // Soft occupancy is tied to the current tile while stationary.
            if matches!(unit.move_state, MoveState::Moving | MoveState::Ghost) {
                unit.soft_until_ms = None;
                continue;
            }

            let Some(until_ms) = unit.soft_until_ms else {
                continue;
            };

            if now_ms < until_ms {
                continue;
            }

            // Only keep the expired timestamp for the primary occupant if there is still overlap;
            // this allows deterministic eviction of non-owners.
            if occ.len(unit.position) <= 1 {
                unit.soft_until_ms = None;
                continue;
            }

            if occ.primary(unit.position) != Some(unit.instance_id) {
                unit.soft_until_ms = None;
            }
        }
    }

    fn schedule_move_step_for_unit(&mut self, unit_instance_id: Uuid, now_ms: u64) {
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };
        if unit.stats.current_health == 0 {
            return;
        }
        if !matches!(unit.move_state, MoveState::Moving | MoveState::Ghost) {
            unit.move_next_step_ms = None;
            return;
        }

        let speed = unit.move_speed_units_per_ms.max(1) as u64;
        let progress = unit.move_progress_units.min(TILE_UNITS_PER_TILE);
        let remaining = TILE_UNITS_PER_TILE.saturating_sub(progress);
        let dt = ceil_div_u64(remaining, speed).max(1);
        let next_ms = now_ms.saturating_add(dt);

        unit.move_next_step_ms = Some(next_ms);
        self.event_queue
            .push(crate::game::battle::enums::BattleEvent::MoveStep {
                time_ms: next_ms,
                unit_instance_id,
            });
    }

    fn update_move_progress_to(&mut self, unit_instance_id: Uuid, now_ms: u64) {
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };
        if unit.stats.current_health == 0 {
            return;
        }
        if !matches!(unit.move_state, MoveState::Moving | MoveState::Ghost) {
            return;
        }

        let last = unit.move_last_update_ms;
        if now_ms <= last {
            return;
        }

        let elapsed = now_ms - last;
        let speed = unit.move_speed_units_per_ms as u64;
        let added = elapsed.saturating_mul(speed);
        unit.move_progress_units = unit.move_progress_units.saturating_add(added);
        unit.move_last_update_ms = now_ms;
    }

    fn stop_movement_and_soft_occupy(&mut self, unit_instance_id: Uuid, now_ms: u64) {
        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };

        self.movement_field.cancel_reservation(unit_instance_id);
        unit.reserved_destination = None;
        unit.move_path.clear();
        unit.move_progress_units = 0;
        unit.move_next_step_ms = None;
        unit.move_last_update_ms = now_ms;
        unit.move_state = MoveState::Acquire;

        // Mark the unit as a temporary "soft occupant" so allies can pass/overlap its tile.
        unit.soft_until_ms = Some(now_ms.saturating_add(SOFT_OCCUPIED_MS));

        // If an autocast was pending while moving, it can now start.
        self.schedule_pending_autocast_for(unit_instance_id, now_ms);
    }

    pub(super) fn apply_hard_cc(&mut self, unit_instance_id: Uuid, until_ms: u64, now_ms: u64) {
        self.movement_field.cancel_reservation(unit_instance_id);

        let Some(unit) = self.units.get_mut(&unit_instance_id) else {
            return;
        };
        if unit.stats.current_health == 0 {
            return;
        }

        unit.move_state = MoveState::CCLocked { until_ms };
        unit.reserved_destination = None;
        unit.move_path.clear();
        unit.move_progress_units = 0;
        unit.move_next_step_ms = None;
        unit.move_last_update_ms = now_ms;

        // Block actions until CC ends.
        // NOTE: BuffExpire is processed after Attack within the same `time_ms` bucket, so we
        // lock actions until `until_ms + 1` to avoid an action firing before the lock is lifted.
        unit.next_action_time = unit.next_action_time.max(until_ms.saturating_add(1));
        // Keep pending casts intact; they may trigger after CC ends.

        // Ensure we re-evaluate intents when the lock ends.
        self.schedule_movement_intent(until_ms);
    }

    pub(super) fn release_hard_cc(&mut self, unit_instance_id: Uuid, now_ms: u64) {
        let (owner, pos, overlapped_by_other) = {
            let Some(unit) = self.units.get(&unit_instance_id) else {
                return;
            };
            if unit.stats.current_health == 0 {
                return;
            }
            if !matches!(unit.move_state, MoveState::CCLocked { .. }) {
                return;
            }

            let occ = OccupancyState::from_units(&self.movement_field, &self.units);
            let overlapped =
                occ.len(unit.position) > 1 && occ.primary(unit.position) != Some(unit_instance_id);

            (unit.owner, unit.position, overlapped)
        };

        // Ghost trigger rule (agreed): all adjacent tiles are blocked for normal movement,
        // and the current tile is occupied/soft-occupied by another unit.
        let occ = OccupancyState::from_units(&self.movement_field, &self.units);
        let mut has_valid_neighbor = false;
        for n in RuntimeField::bfs_neighbors(pos) {
            if !self.movement_field.in_bounds(n) {
                continue;
            }
            let reserved_by = self.movement_field.reserved_unit_at(n);
            if reserved_by.is_some() && reserved_by != Some(unit_instance_id) {
                continue;
            }

            let can_step = if occ.is_empty(n) {
                true
            } else {
                match occ.primary(n).and_then(|id| self.units.get(&id)) {
                    None => false,
                    Some(occupant) if occupant.owner != owner => false,
                    Some(occupant) => occupant
                        .soft_until_ms
                        .is_some_and(|until_ms| now_ms < until_ms),
                }
            };
            if can_step {
                has_valid_neighbor = true;
                break;
            }
        }

        if overlapped_by_other && !has_valid_neighbor {
            self.start_ghost_escape(unit_instance_id, now_ms);
            return;
        }

        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_state = MoveState::Acquire;
            unit.move_next_step_ms = None;
            unit.move_progress_units = 0;
            unit.move_last_update_ms = now_ms;
            unit.next_action_time = 0;
        }
        self.schedule_movement_intent(now_ms);
        // Allow casts on the next tick boundary after CC ends.
        self.schedule_pending_autocast_for(unit_instance_id, now_ms.saturating_add(1));
    }

    fn start_ghost_escape(&mut self, unit_instance_id: Uuid, now_ms: u64) {
        let (owner, start_pos) = match self.units.get(&unit_instance_id) {
            Some(unit) if unit.stats.current_health > 0 => (unit.owner, unit.position),
            _ => return,
        };

        self.movement_field.cancel_reservation(unit_instance_id);
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.reserved_destination = None;
            unit.soft_until_ms = None;
        }

        // Mark as Ghost up-front so it becomes non-blocking for occupancy checks.
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_state = MoveState::Ghost;
            unit.move_next_step_ms = None;
            unit.move_progress_units = 0;
            unit.move_last_update_ms = now_ms;
            unit.move_path.clear();
        }

        let occ = OccupancyState::from_units(&self.movement_field, &self.units);

        // If already on an empty tile (material occupancy), just exit Ghost immediately.
        if occ.is_empty(start_pos) && self.movement_field.reserved_unit_at(start_pos).is_none() {
            if let Some(unit) = self.units.get_mut(&unit_instance_id) {
                unit.move_state = MoveState::Acquire;
                unit.move_path.clear();
                unit.move_next_step_ms = None;
                unit.move_progress_units = 0;
                unit.move_last_update_ms = now_ms;
            }
            self.schedule_movement_intent(now_ms);
            self.schedule_pending_autocast_for(unit_instance_id, now_ms);
            return;
        }

        let bfs = match self.movement_field.bfs_map(start_pos, |_| true) {
            Ok(v) => v,
            Err(_) => return,
        };

        let (fx, fy) = battlefield_forward_vector(owner);

        let mut best: Option<(u32, u8, Position)> = None;
        for y in 0..(self.movement_field.height as i32) {
            for x in 0..(self.movement_field.width as i32) {
                let pos = Position::new(x, y);
                if !occ.is_empty(pos) {
                    continue;
                }
                if self.movement_field.reserved_unit_at(pos).is_some() {
                    continue;
                }
                let Some(dist) = bfs.distance_to(pos) else {
                    continue;
                };
                if dist == 0 {
                    continue;
                }

                let dx = pos.x - start_pos.x;
                let dy = pos.y - start_pos.y;
                let dot = dx * fx + dy * fy;
                let dir_prio: u8 = if dot > 0 {
                    0 // friendly direction
                } else if dot < 0 {
                    1 // enemy direction
                } else {
                    2 // neutral
                };

                match best {
                    None => best = Some((dist, dir_prio, pos)),
                    Some((bd, bp, bpos)) => {
                        if (dist, dir_prio, pos.y, pos.x) < (bd, bp, bpos.y, bpos.x) {
                            best = Some((dist, dir_prio, pos));
                        }
                    }
                }
            }
        }

        let Some((_, _, dest)) = best else {
            // Assumption: field always has at least one empty tile.
            return;
        };
        let path = bfs.reconstruct_path_to(dest).unwrap_or_default();

        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_state = MoveState::Ghost;
            unit.reserved_destination = None;
            unit.move_path = path;
            unit.move_progress_units = 0;
            unit.move_last_update_ms = now_ms;
            unit.move_next_step_ms = None;
        }

        self.schedule_move_step_for_unit(unit_instance_id, now_ms);
    }

    pub(super) fn compute_movement_intents(&mut self, now_ms: u64) {
        let mut unit_ids: Vec<Uuid> = self.units.keys().copied().collect();
        unit_ids.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

        // Resolve any same-side overlaps that survived past the soft window by pushing all
        // non-primary occupants into Ghost.
        {
            let occ = OccupancyState::from_units(&self.movement_field, &self.units);
            let mut evict: Vec<Uuid> = Vec::new();
            for y in 0..(self.movement_field.height as i32) {
                for x in 0..(self.movement_field.width as i32) {
                    let pos = Position::new(x, y);
                    if occ.len(pos) <= 1 {
                        continue;
                    }
                    let Some(primary) = occ.primary(pos) else {
                        continue;
                    };
                    let until_ms = self.units.get(&primary).and_then(|u| u.soft_until_ms);
                    let expired = until_ms.is_none() || now_ms >= until_ms.unwrap();
                    if !expired {
                        continue;
                    }
                    let Some(occupants) = occ.occupants(pos) else {
                        continue;
                    };
                    for u in occupants.iter() {
                        if *u != primary {
                            evict.push(*u);
                        }
                    }
                }
            }
            evict.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            for unit_id in evict {
                self.start_ghost_escape(unit_id, now_ms);
            }
        }

        let occ = OccupancyState::from_units(&self.movement_field, &self.units);

        for unit_id in unit_ids {
            let (owner, base_uuid, start_pos, repath_counter) = {
                let Some(unit) = self.units.get(&unit_id) else {
                    continue;
                };
                if unit.stats.current_health == 0 {
                    continue;
                }

                let eligible = match unit.move_state {
                    MoveState::Acquire => true,
                    MoveState::WaitRepath { until_ms } => now_ms >= until_ms,
                    _ => false,
                };
                if !eligible {
                    continue;
                }

                (
                    unit.owner,
                    unit.base_uuid,
                    unit.position,
                    unit.repath_counter,
                )
            };

            self.movement_field.cancel_reservation(unit_id);

            let range_tiles = self.basic_attack_range_tiles(base_uuid);

            if let Some(target_id) =
                self.choose_attack_target_in_range(owner, start_pos, range_tiles)
            {
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.current_target = Some(target_id);
                    unit.move_state = MoveState::Acquire;
                    unit.reserved_destination = None;
                    unit.move_path.clear();
                    unit.move_next_step_ms = None;
                }
                continue;
            }

            let bfs = match self.movement_field.bfs_map(start_pos, |pos| {
                // Reservations always block pathfinding.
                if self.movement_field.reserved_unit_at(pos).is_some() {
                    return false;
                }

                if occ.is_empty(pos) {
                    return true;
                }

                let Some(primary) = occ.primary(pos) else {
                    return true;
                };
                let Some(occupant) = self.units.get(&primary) else {
                    return true;
                };
                if occupant.owner != owner {
                    return false;
                }

                if is_unit_moving_for_bfs(&self.units, primary) {
                    return true;
                }

                occupant
                    .soft_until_ms
                    .is_some_and(|until_ms| now_ms < until_ms)
            }) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut enemies: Vec<(Uuid, Position)> = self
                .units
                .values()
                .filter_map(|u| {
                    if u.stats.current_health == 0 || u.owner == owner {
                        None
                    } else {
                        Some((u.instance_id, u.position))
                    }
                })
                .collect();
            enemies.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let plans =
                enemy_plans_in_order(&self.movement_field, &bfs, &enemies, range_tiles, |p| {
                    occ.is_empty(p) && self.movement_field.reserved_unit_at(p).is_none()
                });

            let mut reserved: Option<(Uuid, Position)> = None;
            for plan in plans {
                for dest in plan.dest_candidates {
                    if occ.is_empty(dest)
                        && self.movement_field.reserved_unit_at(dest).is_none()
                        && self.movement_field.reserve(unit_id, dest).is_ok()
                    {
                        reserved = Some((plan.enemy_id, dest));
                        break;
                    }
                }
                if reserved.is_some() {
                    break;
                }
            }

            match reserved {
                Some((enemy_id, dest)) => {
                    let path = bfs.reconstruct_path_to(dest).unwrap_or_default();
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = Some(enemy_id);
                        unit.move_state = MoveState::Moving;
                        unit.reserved_destination = Some(dest);
                        unit.move_path = path;
                        unit.move_progress_units = 0;
                        unit.move_last_update_ms = now_ms;
                        unit.soft_until_ms = None;
                    }
                    self.schedule_move_step_for_unit(unit_id, now_ms);
                }
                None => {
                    let jitter = self.movement_jitter_ms(unit_id, repath_counter);
                    let until_ms = now_ms.saturating_add(100).saturating_add(jitter);
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.current_target = None;
                        unit.move_state = MoveState::WaitRepath { until_ms };
                        unit.reserved_destination = None;
                        unit.move_path.clear();
                        unit.repath_counter = unit.repath_counter.wrapping_add(1);
                        unit.move_next_step_ms = None;
                    }
                    self.schedule_movement_intent(until_ms);
                }
            }
        }
    }

    pub(super) fn handle_unit_died_for_movement(&mut self, unit_instance_id: Uuid, time_ms: u64) {
        if let Some(unit) = self.units.get_mut(&unit_instance_id) {
            unit.move_state = MoveState::Dead;
            unit.move_next_step_ms = None;
        }
        self.movement_field.remove_unit(unit_instance_id);
        self.schedule_movement_intent(time_ms);
    }

    pub(super) fn handle_move_steps_at(&mut self, now_ms: u64, unit_ids: Vec<Uuid>) {
        // Dedupe + stable order.
        let mut unique: Vec<Uuid> = unit_ids.into_iter().collect();
        unique.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        unique.dedup();

        // Update progress up-front so swap intent uses consistent state.
        for unit_id in &unique {
            self.update_move_progress_to(*unit_id, now_ms);
        }

        // Prepare step intents (from -> to) for units that are actually due this time.
        let mut step_from_to: HashMap<Uuid, (Position, Position)> = HashMap::new();

        for unit_id in &unique {
            let Some(unit) = self.units.get(unit_id) else {
                continue;
            };
            if unit.stats.current_health == 0 {
                continue;
            }
            if !matches!(unit.move_state, MoveState::Moving | MoveState::Ghost) {
                continue;
            }
            if unit.move_next_step_ms != Some(now_ms) {
                continue;
            }

            if matches!(unit.move_state, MoveState::Moving) {
                // If an enemy becomes attackable while moving, stop immediately.
                let range_tiles = self.basic_attack_range_tiles(unit.base_uuid);
                if let Some(target_id) =
                    self.choose_attack_target_in_range(unit.owner, unit.position, range_tiles)
                {
                    if let Some(unit) = self.units.get_mut(unit_id) {
                        unit.current_target = Some(target_id);
                    }
                    self.stop_movement_and_soft_occupy(*unit_id, now_ms);
                    continue;
                }
            }

            // Need at least 1 tile worth of progress to attempt a step.
            if unit.move_progress_units < TILE_UNITS_PER_TILE {
                self.schedule_move_step_for_unit(*unit_id, now_ms);
                continue;
            }

            let from = unit.position;
            let Some((_, to)) = next_tile_in_path(unit, from) else {
                // Arrived (or path got invalid). If we're already on the reserved destination,
                // finish movement cleanly; otherwise repath.
                if matches!(unit.move_state, MoveState::Ghost) {
                    // Ghost path ended/invalid; recompute from current position.
                    self.start_ghost_escape(*unit_id, now_ms);
                } else {
                    let arrived_here = unit
                        .reserved_destination
                        .map(|d| d == from)
                        .unwrap_or(false)
                        || unit.move_path.len() <= 1;
                    if arrived_here {
                        self.movement_field.cancel_reservation(*unit_id);
                        if let Some(unit) = self.units.get_mut(unit_id) {
                            unit.move_state = MoveState::Acquire;
                            unit.reserved_destination = None;
                            unit.move_path.clear();
                            unit.move_progress_units = 0;
                            unit.move_next_step_ms = None;
                            unit.move_last_update_ms = now_ms;
                        }
                        self.schedule_movement_intent(now_ms);
                        self.schedule_pending_autocast_for(*unit_id, now_ms);
                    } else {
                        let repath_counter = unit.repath_counter;
                        let jitter = self.movement_jitter_ms(*unit_id, repath_counter);
                        let until_ms = now_ms.saturating_add(100).saturating_add(jitter);
                        self.movement_field.cancel_reservation(*unit_id);
                        if let Some(unit) = self.units.get_mut(unit_id) {
                            unit.move_state = MoveState::WaitRepath { until_ms };
                            unit.reserved_destination = None;
                            unit.move_path.clear();
                            unit.move_progress_units = 0;
                            unit.move_next_step_ms = None;
                            unit.move_last_update_ms = now_ms;
                            unit.repath_counter = unit.repath_counter.wrapping_add(1);
                        }
                        self.schedule_movement_intent(until_ms);
                    }
                }
                continue;
            };
            step_from_to.insert(*unit_id, (from, to));
        }

        let mut occ = OccupancyState::from_units(&self.movement_field, &self.units);
        let mut any_moved = false;

        // Defensive repair: reservations must never point at a materially occupied tile.
        // (Ghost does not count as occupied for reservations.)
        for unit_id in self.units.keys().copied().collect::<Vec<_>>() {
            let Some(reserved) = self.movement_field.reserved_destination_of(unit_id) else {
                continue;
            };
            if occ.is_empty(reserved) {
                continue;
            }
            self.movement_field.cancel_reservation(unit_id);
            if let Some(unit) = self.units.get_mut(&unit_id) {
                if unit.reserved_destination == Some(reserved) {
                    unit.reserved_destination = None;
                }
                if matches!(unit.move_state, MoveState::Moving) {
                    unit.move_state = MoveState::Acquire;
                    unit.move_path.clear();
                    unit.move_next_step_ms = None;
                    unit.move_progress_units = 0;
                    unit.move_last_update_ms = now_ms;
                }
            }
            self.schedule_movement_intent(now_ms);
        }

        // Detect swap pairs (A<->B) that are allowed: same-side, both Moving, same tick due.
        let mut swap_pairs: Vec<(Uuid, Uuid)> = Vec::new();
        {
            let mut seen: HashSet<Uuid> = HashSet::new();
            for (&a, &(a_from, a_to)) in &step_from_to {
                if seen.contains(&a) {
                    continue;
                }
                let Some(a_unit) = self.units.get(&a) else {
                    continue;
                };
                if !matches!(a_unit.move_state, MoveState::Moving) {
                    continue;
                }

                let Some(b) = step_from_to
                    .iter()
                    .find(|(&b, &(b_from, b_to))| b != a && b_from == a_to && b_to == a_from)
                    .map(|(&b, _)| b)
                else {
                    continue;
                };
                let Some(b_unit) = self.units.get(&b) else {
                    continue;
                };
                if !matches!(b_unit.move_state, MoveState::Moving) {
                    continue;
                }
                if a_unit.owner != b_unit.owner {
                    continue;
                }

                let (lo, hi) = if a.as_bytes() < b.as_bytes() {
                    (a, b)
                } else {
                    (b, a)
                };
                swap_pairs.push((lo, hi));
                seen.insert(lo);
                seen.insert(hi);
            }
        }
        swap_pairs.sort_by(|(a1, b1), (a2, b2)| {
            a1.as_bytes()
                .cmp(a2.as_bytes())
                .then_with(|| b1.as_bytes().cmp(b2.as_bytes()))
        });
        swap_pairs.dedup();

        let mut in_swap: HashSet<Uuid> = HashSet::new();

        // Apply swaps first (atomically).
        for (a, b) in swap_pairs {
            let Some((a_from, a_to)) = step_from_to.get(&a).copied() else {
                continue;
            };
            let Some((b_from, b_to)) = step_from_to.get(&b).copied() else {
                continue;
            };
            if !(a_from == b_to && a_to == b_from) {
                continue;
            }
            let (owner_a, repath_a) = match self.units.get(&a) {
                Some(u)
                    if u.stats.current_health > 0 && matches!(u.move_state, MoveState::Moving) =>
                {
                    (u.owner, u.repath_counter)
                }
                _ => continue,
            };
            let (owner_b, repath_b) = match self.units.get(&b) {
                Some(u)
                    if u.stats.current_health > 0 && matches!(u.move_state, MoveState::Moving) =>
                {
                    (u.owner, u.repath_counter)
                }
                _ => continue,
            };
            if owner_a != owner_b {
                continue;
            }

            // Reservations must not block the swap destinations (defensive; normally impossible).
            for (to, allow1, allow2) in [(a_to, a, b), (b_to, a, b)] {
                let reserved_by = self.movement_field.reserved_unit_at(to);
                if reserved_by.is_some()
                    && reserved_by != Some(allow1)
                    && reserved_by != Some(allow2)
                {
                    // If the destination is reserved by someone else, abort swap and repath later.
                    for (unit_id, repath_counter) in [(a, repath_a), (b, repath_b)] {
                        let jitter = self.movement_jitter_ms(unit_id, repath_counter);
                        let until_ms = now_ms.saturating_add(100).saturating_add(jitter);
                        self.movement_field.cancel_reservation(unit_id);
                        if let Some(unit) = self.units.get_mut(&unit_id) {
                            unit.move_state = MoveState::WaitRepath { until_ms };
                            unit.reserved_destination = None;
                            unit.move_path.clear();
                            unit.move_next_step_ms = None;
                            unit.move_progress_units = 0;
                            unit.move_last_update_ms = now_ms;
                            unit.repath_counter = unit.repath_counter.wrapping_add(1);
                        }
                        self.schedule_movement_intent(until_ms);
                    }
                    continue;
                }
            }

            // Apply the swap atomically in occupancy.
            occ.remove(a, a_from);
            occ.remove(b, b_from);
            occ.insert(a, a_to);
            occ.insert(b, b_to);

            // Update units (order-independent).
            for (unit_id, to) in [(a, a_to), (b, b_to)] {
                let Some(unit) = self.units.get_mut(&unit_id) else {
                    continue;
                };
                unit.move_progress_units =
                    unit.move_progress_units.saturating_sub(TILE_UNITS_PER_TILE);
                unit.position = to;
                unit.move_last_update_ms = now_ms;
                advance_path(unit);
            }

            // Record timeline in uuid order (a<b by construction).
            self.record_timeline(
                now_ms,
                TimelineEvent::UnitMoved {
                    unit_instance_id: a,
                    from: a_from,
                    to: a_to,
                },
            );
            self.record_timeline(
                now_ms,
                TimelineEvent::UnitMoved {
                    unit_instance_id: b,
                    from: b_from,
                    to: b_to,
                },
            );

            // Finalize arrival/continuation per unit.
            for unit_id in [a, b] {
                let arrived = self
                    .units
                    .get(&unit_id)
                    .and_then(|u| {
                        Some(
                            u.reserved_destination
                                .map(|d| d == u.position)
                                .unwrap_or(false)
                                || u.move_path.len() <= 1,
                        )
                    })
                    .unwrap_or(false);

                if arrived {
                    self.movement_field.cancel_reservation(unit_id);
                    if let Some(unit) = self.units.get_mut(&unit_id) {
                        unit.reserved_destination = None;
                        unit.move_path.clear();
                        unit.move_state = MoveState::Acquire;
                        unit.move_next_step_ms = None;
                        unit.move_last_update_ms = now_ms;
                        unit.move_progress_units = 0;
                    }
                    self.schedule_movement_intent(now_ms);
                    self.schedule_pending_autocast_for(unit_id, now_ms);
                } else {
                    self.schedule_move_step_for_unit(unit_id, now_ms);
                }
            }

            in_swap.insert(a);
            in_swap.insert(b);
            any_moved = true;
        }

        // Apply remaining moves in uuid order.
        for unit_id in unique {
            if in_swap.contains(&unit_id) {
                continue;
            }
            let Some((from, to)) = step_from_to.get(&unit_id).copied() else {
                continue;
            };

            let (owner, repath_counter, state) = match self.units.get(&unit_id) {
                Some(unit)
                    if unit.stats.current_health > 0
                        && matches!(unit.move_state, MoveState::Moving | MoveState::Ghost) =>
                {
                    (unit.owner, unit.repath_counter, unit.move_state)
                }
                _ => continue,
            };

            if matches!(state, MoveState::Ghost) {
                // Ghost ignores swap blocking and tile occupancy while moving.
                let last = self
                    .units
                    .get(&unit_id)
                    .and_then(|u| u.move_path.last().copied());

                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.move_progress_units =
                        unit.move_progress_units.saturating_sub(TILE_UNITS_PER_TILE);
                    unit.position = to;
                    unit.move_last_update_ms = now_ms;
                    advance_path(unit);
                }

                // If we reached the planned destination, try to claim it and exit Ghost.
                if Some(to) == last {
                    let is_empty =
                        occ.is_empty(to) && self.movement_field.reserved_unit_at(to).is_none();
                    if is_empty {
                        if let Some(unit) = self.units.get_mut(&unit_id) {
                            unit.move_state = MoveState::Acquire;
                            unit.move_path.clear();
                            unit.move_next_step_ms = None;
                            unit.move_progress_units = 0;
                            unit.soft_until_ms = None;
                        }
                        // Ghost becomes material again; reflect in occupancy for subsequent moves.
                        occ.insert(unit_id, to);
                        self.schedule_movement_intent(now_ms);
                        self.schedule_pending_autocast_for(unit_id, now_ms);
                    } else {
                        self.start_ghost_escape(unit_id, now_ms);
                    }
                } else {
                    self.schedule_move_step_for_unit(unit_id, now_ms);
                }

                any_moved = true;
                self.record_timeline(
                    now_ms,
                    TimelineEvent::UnitMoved {
                        unit_instance_id: unit_id,
                        from,
                        to,
                    },
                );
                continue;
            }

            // Validate destination at the moment of application using unit positions as source of truth.
            if !self.movement_field.in_bounds(to) {
                self.stop_movement_and_soft_occupy(unit_id, now_ms);
                self.schedule_movement_intent(now_ms);
                continue;
            }

            let reserved_by = self.movement_field.reserved_unit_at(to);
            let reserved_blocks = reserved_by.is_some() && reserved_by != Some(unit_id);

            let can_enter = if reserved_blocks {
                false
            } else if occ.is_empty(to) {
                true
            } else {
                match occ.primary(to).and_then(|id| self.units.get(&id)) {
                    None => true,
                    Some(occupant_unit) if occupant_unit.owner != owner => false,
                    Some(occupant_unit) => occupant_unit
                        .soft_until_ms
                        .is_some_and(|until_ms| now_ms < until_ms),
                }
            };

            if !can_enter {
                // Path became invalid; fall back to repath.
                let jitter = self.movement_jitter_ms(unit_id, repath_counter);
                let until_ms = now_ms.saturating_add(100).saturating_add(jitter);
                self.movement_field.cancel_reservation(unit_id);
                if let Some(unit) = self.units.get_mut(&unit_id) {
                    unit.move_state = MoveState::WaitRepath { until_ms };
                    unit.reserved_destination = None;
                    unit.move_path.clear();
                    unit.move_next_step_ms = None;
                    unit.move_progress_units = 0;
                    unit.move_last_update_ms = now_ms;
                    unit.repath_counter = unit.repath_counter.wrapping_add(1);
                }
                self.schedule_movement_intent(until_ms);
                continue;
            }

            // Leave current tile (frees immediately), then enter destination.
            occ.remove(unit_id, from);
            // Enter empty or same-side soft tile.
            occ.insert(unit_id, to);

            let arrived;
            let mut should_continue = false;
            {
                let Some(unit) = self.units.get_mut(&unit_id) else {
                    continue;
                };

                // Consume one tile worth of progress for exactly one step.
                unit.move_progress_units =
                    unit.move_progress_units.saturating_sub(TILE_UNITS_PER_TILE);

                unit.position = to;

                // Advance path (keep it aligned with current position).
                advance_path(unit);

                arrived = unit.reserved_destination.map(|d| d == to).unwrap_or(false)
                    || unit.move_path.len() <= 1;

                if arrived {
                    self.movement_field.cancel_reservation(unit_id);
                    unit.reserved_destination = None;
                    unit.move_path.clear();
                    unit.move_state = MoveState::Acquire;
                    unit.move_next_step_ms = None;
                    unit.move_last_update_ms = now_ms;
                    unit.move_progress_units = 0;
                } else {
                    unit.move_last_update_ms = now_ms;
                    should_continue = true;
                }
            }

            any_moved = true;
            self.record_timeline(
                now_ms,
                TimelineEvent::UnitMoved {
                    unit_instance_id: unit_id,
                    from,
                    to,
                },
            );

            if arrived {
                self.schedule_movement_intent(now_ms);
                self.schedule_pending_autocast_for(unit_id, now_ms);
            } else if should_continue {
                self.schedule_move_step_for_unit(unit_id, now_ms);
            }
        }

        if any_moved {
            self.schedule_movement_intent(now_ms);
        }
    }
}

fn next_tile_in_path(unit: &RuntimeUnit, from: Position) -> Option<(usize, Position)> {
    // Path is expected to be [current, ..., destination].
    if unit.move_path.is_empty() {
        return None;
    }
    // Fast path: current is at index 0.
    if unit.move_path.first().copied() == Some(from) {
        return unit.move_path.get(1).copied().map(|to| (0, to));
    }
    // Fallback: find current.
    let idx = unit.move_path.iter().position(|p| *p == from)?;
    unit.move_path.get(idx + 1).copied().map(|to| (idx, to))
}

fn advance_path(unit: &mut RuntimeUnit) {
    if unit.move_path.is_empty() {
        return;
    }
    // Keep path[0] == current position when possible.
    if unit.move_path.first().copied() == Some(unit.position) {
        return;
    }
    if let Some(idx) = unit.move_path.iter().position(|p| *p == unit.position) {
        unit.move_path.drain(0..idx);
    } else {
        unit.move_path.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use crate::game::battle::buffs::BuffId;
    use crate::game::battle::timeline::Timeline;
    use crate::game::battle::timeline::TimelineCause;
    use crate::game::battle::types::PlayerDeckInfo;
    use crate::game::data::abnormality_data::AbnormalityDatabase;
    use crate::game::data::artifact_data::ArtifactDatabase;
    use crate::game::data::bonus_data::BonusDatabase;
    use crate::game::data::equipment_data::EquipmentDatabase;
    use crate::game::data::event_pools::{EventPhasePool, EventPoolConfig};
    use crate::game::data::pve_data::PveEncounterDatabase;
    use crate::game::data::random_event_data::RandomEventDatabase;
    use crate::game::data::shop_data::ShopDatabase;
    use crate::game::data::skill_data::SkillDatabase;
    use crate::game::data::GameDataBase;
    use crate::game::stats::UnitStats;

    #[test]
    fn chase_target_uses_attack_position_distance_and_skips_unreachable_enemy() {
        // 4x4 field, mover at (1,1), range=1.
        // Enemy A at (1,0) is "close" but fully surrounded by blockers so there is no Empty tile
        // within range to stand on. Enemy B at (3,3) has an adjacent empty tile reachable.
        let field = RuntimeField::new(4, 4);

        let mover_pos = Position::new(1, 1);

        let enemy_a = Uuid::from_u128(2);
        let enemy_a_pos = Position::new(1, 0);

        // Block all 8 neighbors of enemy_a that are in bounds and within cheb<=1.
        let mut occupied: HashSet<Position> = HashSet::new();
        occupied.insert(mover_pos);
        occupied.insert(enemy_a_pos);
        for y in 0..=1 {
            for x in 0..=2 {
                let pos = Position::new(x, y);
                if pos == enemy_a_pos {
                    continue;
                }
                // Don't block mover's start.
                if pos == mover_pos {
                    continue;
                }
                occupied.insert(pos);
            }
        }

        let enemy_b = Uuid::from_u128(3);
        let enemy_b_pos = Position::new(3, 3);
        occupied.insert(enemy_b_pos);

        let bfs = field
            .bfs_map(mover_pos, |pos| !occupied.contains(&pos))
            .unwrap();

        let enemies = vec![(enemy_a, enemy_a_pos), (enemy_b, enemy_b_pos)];
        let plans = enemy_plans_in_order(&field, &bfs, &enemies, 1, |pos| {
            !occupied.contains(&pos) && field.reserved_unit_at(pos).is_none()
        });

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].enemy_id, enemy_b);
    }

    #[test]
    fn destination_candidates_are_sorted_by_bfs_then_tile_order() {
        let field = RuntimeField::new(4, 4);
        let mover_pos = Position::new(1, 1);
        let enemy_pos = Position::new(2, 2);

        let mut blocked: HashSet<Position> = HashSet::new();
        blocked.insert(enemy_pos);

        let bfs = field
            .bfs_map(mover_pos, |pos| !blocked.contains(&pos))
            .unwrap();

        let candidates = destination_candidates_in_order(&field, &bfs, enemy_pos, 1, |pos| {
            !blocked.contains(&pos) && field.reserved_unit_at(pos).is_none()
        });
        assert!(!candidates.is_empty());

        // Same BFS distance candidates should be ordered by (y,x).
        let mut last: Option<(u32, Position)> = None;
        for item in candidates {
            if let Some((d, p)) = last {
                assert!(
                    (item.0 > d) || (item.0 == d && (item.1.y, item.1.x) >= (p.y, p.x)),
                    "candidates not sorted"
                );
            }
            last = Some(item);
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

    fn empty_deck() -> PlayerDeckInfo {
        PlayerDeckInfo {
            units: vec![],
            artifacts: vec![],
            positions: HashMap::new(),
        }
    }

    fn new_test_core(field_size: (u8, u8)) -> BattleCore {
        let game_data = empty_game_data();
        let deck = empty_deck();
        let mut core = BattleCore::new(&deck, &deck, game_data, field_size, 123);
        core.timeline = Timeline::new();
        core
    }

    fn insert_moving_unit(
        core: &mut BattleCore,
        unit_id: Uuid,
        owner: Side,
        pos: Position,
        path: Vec<Position>,
        now_ms: u64,
    ) {
        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner,
                base_uuid: Uuid::from_u128(0xDEAD),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: pos,
                current_target: None,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 1000,
                resonance_gain_locked_until_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                move_state: MoveState::Moving,
                soft_until_ms: None,
                move_progress_units: TILE_UNITS_PER_TILE,
                move_last_update_ms: now_ms,
                move_speed_units_per_ms: 3000,
                move_next_step_ms: Some(now_ms),
                reserved_destination: None,
                move_path: path,
                repath_counter: 0,
            },
        );
    }

    fn insert_acquire_unit(core: &mut BattleCore, unit_id: Uuid, owner: Side, pos: Position) {
        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner,
                base_uuid: Uuid::from_u128(0xDEAD),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: pos,
                current_target: None,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 1000,
                resonance_gain_locked_until_ms: 0,
                next_action_time: 0,
                pending_cast: false,
                pending_cast_cause: None,
                move_state: MoveState::Acquire,
                soft_until_ms: None,
                move_progress_units: 0,
                move_last_update_ms: 0,
                move_speed_units_per_ms: 3000,
                move_next_step_ms: None,
                reserved_destination: None,
                move_path: Vec::new(),
                repath_counter: 0,
            },
        );
    }

    #[test]
    fn expired_soft_overlap_evicts_all_non_owners_to_ghost() {
        let now_ms = 100;
        let mut core = new_test_core((4, 4));

        let pos = Position::new(1, 1);
        let owner_id = Uuid::from_u128(1);
        let u2 = Uuid::from_u128(2);
        let u3 = Uuid::from_u128(3);

        insert_acquire_unit(&mut core, owner_id, Side::Player, pos);
        insert_acquire_unit(&mut core, u2, Side::Player, pos);
        insert_acquire_unit(&mut core, u3, Side::Player, pos);
        if let Some(owner) = core.units.get_mut(&owner_id) {
            owner.soft_until_ms = Some(50);
        }

        // Mirror the runtime loop: expire soft occupancy at the start of the time slice,
        // then compute intents (which performs eviction when overlap survives past the window).
        core.expire_soft_occupancy(now_ms);

        core.compute_movement_intents(now_ms);

        assert!(matches!(
            core.units.get(&owner_id).unwrap().move_state,
            MoveState::Acquire | MoveState::WaitRepath { .. }
        ));
        assert!(matches!(
            core.units.get(&u2).unwrap().move_state,
            MoveState::Ghost
        ));
        assert!(matches!(
            core.units.get(&u3).unwrap().move_state,
            MoveState::Ghost
        ));

        // After a ghost step, the leavers should no longer be on the overlapped tile.
        for unit_id in [u2, u3] {
            let Some(unit) = core.units.get_mut(&unit_id) else {
                continue;
            };
            unit.move_progress_units = TILE_UNITS_PER_TILE;
            unit.move_next_step_ms = Some(now_ms);
            unit.move_last_update_ms = now_ms;
        }

        core.handle_move_steps_at(now_ms, vec![u2, u3]);

        assert_ne!(core.units.get(&u2).unwrap().position, pos);
        assert_ne!(core.units.get(&u3).unwrap().position, pos);

        // With overlap resolved, the owner's expired soft marker should clear.
        core.expire_soft_occupancy(999);
        assert_eq!(core.units.get(&owner_id).unwrap().soft_until_ms, None);
    }

    #[test]
    fn hard_cc_locks_unit_and_cancels_reservation() {
        let now_ms = 100;
        let mut core = new_test_core((4, 4));

        let unit_id = Uuid::from_u128(1);
        let caster_id = Uuid::from_u128(999);

        insert_moving_unit(
            &mut core,
            unit_id,
            Side::Player,
            Position::new(1, 1),
            vec![Position::new(1, 1), Position::new(1, 2)],
            now_ms,
        );

        let dest = Position::new(3, 3);
        core.movement_field.reserve(unit_id, dest).unwrap();
        if let Some(unit) = core.units.get_mut(&unit_id) {
            unit.reserved_destination = Some(dest);
        }

        let duration_ms = 50;
        core.process_event(
            BattleEvent::ApplyBuff {
                time_ms: now_ms,
                caster_instance_id: caster_id,
                target_instance_id: unit_id,
                buff_id: BuffId::from_name("stun"),
                duration_ms,
                cause: TimelineCause::default(),
            },
            now_ms,
        )
        .unwrap();

        assert_eq!(core.movement_field.reserved_destination_of(unit_id), None);
        let unit = core.units.get(&unit_id).unwrap();
        assert!(matches!(
            unit.move_state,
            MoveState::CCLocked {
                until_ms
            } if until_ms == now_ms + duration_ms
        ));
        assert_eq!(unit.move_next_step_ms, None);
        assert!(unit.next_action_time >= now_ms + duration_ms);
    }

    #[test]
    fn hard_cc_release_triggers_ghost_escape_when_surrounded() {
        let now_ms = 100;
        let mut core = new_test_core((4, 4));

        let occupier_id = Uuid::from_u128(1);
        let unit_id = Uuid::from_u128(2);
        let start = Position::new(1, 1);

        insert_acquire_unit(&mut core, occupier_id, Side::Player, start);
        core.units.insert(
            unit_id,
            RuntimeUnit {
                instance_id: unit_id,
                owner: Side::Player,
                base_uuid: Uuid::from_u128(0xDEAD),
                stats: UnitStats::with_values(10, 10, 1, 0, 1000),
                position: start,
                current_target: None,
                resonance_current: 0,
                resonance_max: 100,
                resonance_lock_ms: 1000,
                resonance_gain_locked_until_ms: 0,
                next_action_time: now_ms,
                pending_cast: false,
                pending_cast_cause: None,
                move_state: MoveState::CCLocked { until_ms: now_ms },
                soft_until_ms: None,
                move_progress_units: 0,
                move_last_update_ms: now_ms,
                move_speed_units_per_ms: 3000,
                move_next_step_ms: None,
                reserved_destination: None,
                move_path: Vec::new(),
                repath_counter: 0,
            },
        );

        // Surround the unit with stationary occupants.
        let mut blocker_id = 10u128;
        for n in RuntimeField::bfs_neighbors(start) {
            if !core.movement_field.in_bounds(n) {
                continue;
            }
            blocker_id += 1;
            insert_acquire_unit(&mut core, Uuid::from_u128(blocker_id), Side::Opponent, n);
        }

        core.release_hard_cc(unit_id, now_ms);

        let unit = core.units.get(&unit_id).unwrap();
        assert!(matches!(unit.move_state, MoveState::Ghost));
        let dest = *unit.move_path.last().expect("ghost path");
        // Player friendly direction is "down" (+y): pick a destination with larger y
        // when distance ties exist.
        assert!(dest.y > start.y);
    }

    #[test]
    fn autocast_start_while_moving_is_not_rescheduled_same_tick() {
        let now_ms = 100;
        let mut core = new_test_core((4, 4));

        let unit_id = Uuid::from_u128(1);
        insert_moving_unit(
            &mut core,
            unit_id,
            Side::Player,
            Position::new(1, 1),
            vec![Position::new(1, 1), Position::new(1, 2)],
            now_ms,
        );

        core.event_queue.clear();
        core.process_event(
            BattleEvent::AutoCastStart {
                time_ms: now_ms,
                caster_instance_id: unit_id,
                cause: TimelineCause::default(),
            },
            now_ms,
        )
        .unwrap();

        assert!(core.units.get(&unit_id).unwrap().pending_cast);
        assert!(core.event_queue.is_empty());
    }

    #[test]
    fn pending_autocast_is_scheduled_when_unit_becomes_actionable() {
        let now_ms = 100;
        let mut core = new_test_core((4, 4));

        let unit_id = Uuid::from_u128(1);
        insert_moving_unit(
            &mut core,
            unit_id,
            Side::Player,
            Position::new(1, 1),
            vec![Position::new(1, 1), Position::new(1, 2)],
            now_ms,
        );

        if let Some(unit) = core.units.get_mut(&unit_id) {
            unit.pending_cast = true;
            unit.pending_cast_cause = None;
        }

        core.event_queue.clear();
        core.schedule_pending_autocasts(now_ms);
        assert!(core.units.get(&unit_id).unwrap().pending_cast);
        assert!(core.event_queue.is_empty());

        if let Some(unit) = core.units.get_mut(&unit_id) {
            unit.move_state = MoveState::Acquire;
            unit.move_next_step_ms = None;
        }
        core.schedule_pending_autocast_for(unit_id, now_ms);

        let event = core.event_queue.pop().expect("autocast event scheduled");
        assert!(matches!(
            event,
            BattleEvent::AutoCastStart {
                time_ms,
                caster_instance_id,
                ..
            } if time_ms == now_ms && caster_instance_id == unit_id
        ));
        assert!(!core.units.get(&unit_id).unwrap().pending_cast);
    }

    #[test]
    fn move_frees_previous_tile_and_allows_following_into_it_same_tick() {
        let now_ms = 100;
        let mut core = new_test_core((3, 3));

        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);

        insert_moving_unit(
            &mut core,
            a,
            Side::Player,
            Position::new(0, 0),
            vec![Position::new(0, 0), Position::new(0, 1)],
            now_ms,
        );
        insert_moving_unit(
            &mut core,
            b,
            Side::Player,
            Position::new(1, 0),
            vec![Position::new(1, 0), Position::new(0, 0)],
            now_ms,
        );

        core.handle_move_steps_at(now_ms, vec![a, b]);

        assert_eq!(core.units.get(&a).unwrap().position, Position::new(0, 1));
        assert_eq!(core.units.get(&b).unwrap().position, Position::new(0, 0));
    }

    #[test]
    fn swap_is_allowed_for_allies_when_both_due() {
        let now_ms = 100;
        let mut core = new_test_core((3, 3));

        let lower = Uuid::from_u128(1);
        let higher = Uuid::from_u128(2);

        insert_moving_unit(
            &mut core,
            lower,
            Side::Player,
            Position::new(0, 0),
            vec![Position::new(0, 0), Position::new(1, 0)],
            now_ms,
        );
        insert_moving_unit(
            &mut core,
            higher,
            Side::Player,
            Position::new(1, 0),
            vec![Position::new(1, 0), Position::new(0, 0)],
            now_ms,
        );

        core.handle_move_steps_at(now_ms, vec![higher, lower]);

        // Swap is allowed for allies when both are moving and due in the same tick.
        assert_eq!(
            core.units.get(&lower).unwrap().position,
            Position::new(1, 0)
        );
        assert_eq!(
            core.units.get(&higher).unwrap().position,
            Position::new(0, 0)
        );
    }
}
