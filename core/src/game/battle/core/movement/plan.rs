use std::cmp::Ordering;
use uuid::Uuid;

use crate::{
    ecs::resources::Position,
    game::{
        ability::DeliveryDef,
        battle::{
            battlefield::bfs::BfsMap, core::BattleCore, ids::UnitInstanceId,
            timeline::AttackDelivery,
        },
        enums::Side,
    },
};

use super::{EnemyChasePlan, TILE_UNITS_PER_TILE};

pub(in crate::game::battle::core) const INSTANT_BASIC_ATTACK_MELEE_REACH_UNITS: i64 =
    TILE_UNITS_PER_TILE as i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::game::battle::core) struct BasicAttackRangePolicy {
    pub delivery: AttackDelivery,
    pub range_tiles: u8,
    pub instant_melee_reach_units: i64,
    pub use_continuous_range: bool,
}

impl BattleCore {
    pub(in crate::game::battle::core) fn enemy_target_range_tiles(
        &self,
        unit_id: UnitInstanceId,
    ) -> u8 {
        self.units
            .get(&unit_id)
            .map(|unit| self.basic_attack_range_tiles(unit.base_uuid))
            .unwrap_or(u8::MAX)
    }

    pub(in crate::game::battle::core) fn compare_enemy_target_range_preference(
        &self,
        a: UnitInstanceId,
        b: UnitInstanceId,
    ) -> Ordering {
        self.enemy_target_range_tiles(a)
            .cmp(&self.enemy_target_range_tiles(b))
    }

    pub(in crate::game::battle::core) fn compare_enemy_target_preference(
        &self,
        a: UnitInstanceId,
        b: UnitInstanceId,
    ) -> Ordering {
        self.compare_enemy_target_range_preference(a, b)
            .then_with(|| a.as_bytes().cmp(b.as_bytes()))
    }

    pub(in crate::game::battle::core) fn compare_in_range_target_preference(
        &self,
        owner: Side,
        attacker_pos: Position,
        a_id: UnitInstanceId,
        a_pos: Position,
        b_id: UnitInstanceId,
        b_pos: Position,
    ) -> Ordering {
        self.compare_enemy_target_range_preference(a_id, b_id)
            .then_with(|| {
                Self::compare_plan_preference(owner, attacker_pos, a_pos, b_pos, a_pos, b_pos)
            })
            .then_with(|| a_id.as_bytes().cmp(b_id.as_bytes()))
    }

    pub(in crate::game::battle::core) fn basic_attack_range_policy(
        &self,
        unit_base_uuid: Uuid,
    ) -> BasicAttackRangePolicy {
        BasicAttackRangePolicy {
            delivery: self.basic_attack_delivery(unit_base_uuid),
            range_tiles: self.basic_attack_range_tiles(unit_base_uuid),
            instant_melee_reach_units: INSTANT_BASIC_ATTACK_MELEE_REACH_UNITS,
            use_continuous_range: self.basic_attack_delivery(unit_base_uuid)
                == AttackDelivery::Instant
                && self.basic_attack_range_tiles(unit_base_uuid) <= 1,
        }
    }

    fn basic_attack_target_distance_key(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        policy: BasicAttackRangePolicy,
    ) -> Option<u128> {
        if policy.use_continuous_range {
            let attacker = self.units.get(&attacker_instance_id)?;
            let target = self.units.get(&target_id)?;
            let dx = i128::from(attacker.pos_x_units) - i128::from(target.pos_x_units);
            let dy = i128::from(attacker.pos_y_units) - i128::from(target.pos_y_units);
            Some((dx * dx + dy * dy) as u128)
        } else {
            let attacker_pos = self.battlefield.position_of(attacker_instance_id)?;
            let target_pos = self.battlefield.position_of(target_id)?;
            Some(attacker_pos.chebyshev(&target_pos).max(0) as u128)
        }
    }

    pub(in crate::game::battle::core) fn is_basic_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> bool {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return false;
        };
        let policy = self.basic_attack_range_policy(attacker.base_uuid);
        self.is_basic_attack_target_in_range_with_policy(attacker_instance_id, target_id, policy)
    }

    pub(in crate::game::battle::core) fn persisted_target_in_tile_range_for_continuous_melee(
        &self,
        attacker_instance_id: UnitInstanceId,
        current_target: Option<UnitInstanceId>,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        let policy = self.basic_attack_range_policy(attacker.base_uuid);
        if !policy.use_continuous_range {
            return None;
        }

        let attacker_pos = self.battlefield.position_of(attacker_instance_id)?;
        self.persisted_target_if_alive(attacker.owner, current_target)
            .filter(|id| {
                self.battlefield.position_of(*id).is_some_and(|target_pos| {
                    attacker_pos.chebyshev(&target_pos) <= i32::from(policy.range_tiles)
                })
            })
    }

    pub(in crate::game::battle::core) fn choose_attack_target_in_tile_range_for_continuous_melee(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        let policy = self.basic_attack_range_policy(attacker.base_uuid);
        if !policy.use_continuous_range {
            return None;
        }

        let attacker_pos = self.battlefield.position_of(attacker_instance_id)?;
        self.choose_enemy_target_in_tile_range(attacker.owner, attacker_pos, policy.range_tiles)
    }

    fn is_basic_attack_target_in_range_with_policy(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        policy: BasicAttackRangePolicy,
    ) -> bool {
        if policy.use_continuous_range {
            self.basic_attack_target_distance_key(attacker_instance_id, target_id, policy)
                .is_some_and(|distance_sq| {
                    let reach_sq =
                        u128::from(policy.instant_melee_reach_units.unsigned_abs()).pow(2);
                    distance_sq <= reach_sq
                })
        } else {
            self.basic_attack_target_distance_key(attacker_instance_id, target_id, policy)
                .is_some_and(|distance_tiles| distance_tiles <= u128::from(policy.range_tiles))
        }
    }

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
            .then_with(|| {
                (a.x - mover_start.x)
                    .abs()
                    .cmp(&(b.x - mover_start.x).abs())
            })
            .then_with(|| (a.x - enemy_pos.x).abs().cmp(&(b.x - enemy_pos.x).abs()))
            .then_with(|| a.x.cmp(&b.x))
            .then_with(|| a.y.cmp(&b.y))
    }

    pub(in crate::game::battle::core) fn compare_plan_preference(
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
                (best_dest_a.x - mover_start.x)
                    .abs()
                    .cmp(&(best_dest_b.x - mover_start.x).abs())
            })
            .then_with(|| {
                (best_dest_a.x - enemy_a.x)
                    .abs()
                    .cmp(&(best_dest_b.x - enemy_b.x).abs())
            })
            .then_with(|| best_dest_a.x.cmp(&best_dest_b.x))
            .then_with(|| best_dest_a.y.cmp(&best_dest_b.y))
    }

    pub fn choose_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return None;
        };
        let attacker_owner = attacker.owner;
        let attacker_pos = self.battlefield.position_of(attacker_instance_id);
        let policy = self.basic_attack_range_policy(attacker.base_uuid);

        let mut best: Option<(u128, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() {
                continue;
            }

            if unit.owner == attacker_owner {
                continue;
            }

            if !self.is_basic_attack_target_in_range_with_policy(
                attacker_instance_id,
                unit.instance_id,
                policy,
            ) {
                continue;
            }

            let Some(distance_key) = self.basic_attack_target_distance_key(
                attacker_instance_id,
                unit.instance_id,
                policy,
            ) else {
                continue;
            };

            match best {
                None => best = Some((distance_key, unit.instance_id)),
                Some((best_d, _)) if distance_key < best_d => {
                    best = Some((distance_key, unit.instance_id))
                }
                Some((best_d, best_id))
                    if distance_key == best_d
                        && match (
                            attacker_pos,
                            self.battlefield.position_of(unit.instance_id),
                            self.battlefield.position_of(best_id),
                        ) {
                            (Some(attacker_pos), Some(unit_pos), Some(best_pos)) => self
                                .compare_in_range_target_preference(
                                    attacker_owner,
                                    attacker_pos,
                                    unit.instance_id,
                                    unit_pos,
                                    best_id,
                                    best_pos,
                                )
                                .is_lt(),
                            _ => self
                                .compare_enemy_target_preference(unit.instance_id, best_id)
                                .is_lt(),
                        } =>
                {
                    best = Some((distance_key, unit.instance_id))
                }
                _ => {}
            }
        }

        best.map(|(_, id)| id)
    }

    pub(in crate::game::battle::core) fn choose_enemy_target_in_tile_range(
        &self,
        attacker_owner: Side,
        attacker_pos: Position,
        range_tiles: u8,
    ) -> Option<UnitInstanceId> {
        let mut best: Option<(i32, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() || unit.owner == attacker_owner {
                continue;
            }

            let Some(unit_pos) = self.battlefield.position_of(unit.instance_id) else {
                continue;
            };
            let distance = attacker_pos.chebyshev(&unit_pos);
            if distance > range_tiles as i32 {
                continue;
            }

            match best {
                None => best = Some((distance, unit.instance_id)),
                Some((best_distance, _)) if distance < best_distance => {
                    best = Some((distance, unit.instance_id))
                }
                Some((best_distance, best_id))
                    if distance == best_distance
                        && self
                            .battlefield
                            .position_of(best_id)
                            .is_some_and(|best_pos| {
                                self.compare_in_range_target_preference(
                                    attacker_owner,
                                    attacker_pos,
                                    unit.instance_id,
                                    unit_pos,
                                    best_id,
                                    best_pos,
                                )
                                .is_lt()
                            }) =>
                {
                    best = Some((distance, unit.instance_id))
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
                .then_with(|| self.compare_enemy_target_range_preference(a.enemy_id, b.enemy_id))
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
