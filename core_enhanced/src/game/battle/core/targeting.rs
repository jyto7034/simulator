use std::cmp::Ordering;

use uuid::Uuid;

use crate::game::{
    ability::DeliveryDef,
    battle::{core::BattleCore, ids::UnitInstanceId, timeline::AttackDelivery},
    enums::Side,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::game::battle::core) struct BasicAttackRangePolicy {
    pub delivery: AttackDelivery,
    pub range_units: f32,
}

impl BattleCore {
    pub(in crate::game::battle::core) fn enemy_target_range_units(
        &self,
        unit_id: UnitInstanceId,
    ) -> f32 {
        self.units
            .get(&unit_id)
            .map(|unit| self.basic_attack_range_units(unit.base_uuid))
            .unwrap_or(f32::MAX)
    }

    pub(in crate::game::battle::core) fn compare_enemy_target_range_preference(
        &self,
        a: UnitInstanceId,
        b: UnitInstanceId,
    ) -> Ordering {
        self.enemy_target_range_units(a)
            .total_cmp(&self.enemy_target_range_units(b))
    }

    pub(in crate::game::battle::core) fn compare_enemy_target_preference(
        &self,
        a: UnitInstanceId,
        b: UnitInstanceId,
    ) -> Ordering {
        self.compare_enemy_target_range_preference(a, b)
            .then_with(|| a.as_bytes().cmp(b.as_bytes()))
    }

    pub(in crate::game::battle::core) fn basic_attack_range_policy(
        &self,
        unit_base_uuid: Uuid,
    ) -> BasicAttackRangePolicy {
        BasicAttackRangePolicy {
            delivery: self.basic_attack_delivery(unit_base_uuid),
            range_units: self.basic_attack_range_units(unit_base_uuid),
        }
    }

    fn basic_attack_target_distance_sq_world(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
    ) -> Option<f32> {
        let attacker = self.unit_body_view(attacker_instance_id)?;
        let target = self.unit_body_view(target_id)?;
        Some(attacker.position.distance_squared(target.position))
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

    fn is_basic_attack_target_in_range_with_policy(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        policy: BasicAttackRangePolicy,
    ) -> bool {
        let Some(attacker) = self.unit_body_view(attacker_instance_id) else {
            return false;
        };
        let Some(target) = self.unit_body_view(target_id) else {
            return false;
        };

        attacker.can_reach(&target, policy.range_units)
    }

    pub fn choose_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return None;
        };
        let attacker_owner = attacker.owner;
        let policy = self.basic_attack_range_policy(attacker.base_uuid);

        let mut best: Option<(f32, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() || unit.owner == attacker_owner {
                continue;
            }

            if !self.is_basic_attack_target_in_range_with_policy(
                attacker_instance_id,
                unit.instance_id,
                policy,
            ) {
                continue;
            }

            let Some(distance_key) =
                self.basic_attack_target_distance_sq_world(attacker_instance_id, unit.instance_id)
            else {
                continue;
            };

            match best {
                None => best = Some((distance_key, unit.instance_id)),
                Some((best_d, _)) if distance_key < best_d => {
                    best = Some((distance_key, unit.instance_id))
                }
                Some((best_d, best_id))
                    if distance_key.total_cmp(&best_d).is_eq()
                        && self
                            .compare_enemy_target_preference(unit.instance_id, best_id)
                            .is_lt() =>
                {
                    best = Some((distance_key, unit.instance_id))
                }
                _ => {}
            }
        }

        best.map(|(_, id)| id)
    }

    pub(in crate::game::battle::core) fn choose_enemy_target_in_range_units(
        &self,
        caster_instance_id: UnitInstanceId,
        attacker_owner: Side,
        range_units: f32,
    ) -> Option<UnitInstanceId> {
        let caster_body = self.unit_body_view(caster_instance_id)?;
        let mut best: Option<(f32, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() || unit.owner == attacker_owner {
                continue;
            }

            let Some(target_body) = self.unit_body_view(unit.instance_id) else {
                continue;
            };
            if !caster_body.can_reach(&target_body, range_units) {
                continue;
            }
            let distance_sq = caster_body.position.distance_squared(target_body.position);

            match best {
                None => best = Some((distance_sq, unit.instance_id)),
                Some((best_distance_sq, _)) if distance_sq < best_distance_sq => {
                    best = Some((distance_sq, unit.instance_id))
                }
                Some((best_distance_sq, best_id))
                    if distance_sq.total_cmp(&best_distance_sq).is_eq()
                        && self
                            .compare_enemy_target_preference(unit.instance_id, best_id)
                            .is_lt() =>
                {
                    best = Some((distance_sq, unit.instance_id))
                }
                _ => {}
            }
        }

        best.map(|(_, id)| id)
    }

    pub fn basic_attack_range_units(&self, unit_base_uuid: Uuid) -> f32 {
        self.game_data
            .abnormality_data
            .get_by_uuid(&unit_base_uuid)
            .map(|m| m.basic_attack.range_units)
            .unwrap_or(1.0)
            .max(0.0)
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
            DeliveryDef::Area { .. } => AttackDelivery::Instant,
        }
    }
}
