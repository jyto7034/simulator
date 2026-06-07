use std::cmp::Ordering;

use uuid::Uuid;

use super::RuntimeUnit;
use crate::game::{
    ability::DeliveryDef,
    battle::{
        core::BattleCore, ids::UnitInstanceId, scenario::PlayerMovementPlan,
        tile_range::TileRangePattern, timeline::AttackDelivery,
    },
    data::equipment_data::TargetingProfile,
    enums::Side,
};

const SPLASH_CLUSTER_RADIUS_UNITS: f32 = 1.5;

#[derive(Debug, Clone, PartialEq)]
pub(in crate::game::battle::core) struct BasicAttackRangePolicy {
    pub delivery: AttackDelivery,
    pub range_units: f32,
    pub defense_tile_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

impl BattleCore {
    pub(in crate::game::battle::core) fn enemy_target_range_units(
        &self,
        unit_id: UnitInstanceId,
    ) -> f32 {
        self.units
            .get(&unit_id)
            .map(|unit| unit.basic_attack.range_units.max(0.0))
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
        let policy = self.basic_attack_range_policy_for_unit(attacker);
        self.is_basic_attack_target_in_range_with_policy(attacker_instance_id, target_id, &policy)
    }

    fn is_basic_attack_target_in_range_with_policy(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        policy: &BasicAttackRangePolicy,
    ) -> bool {
        if !self.single_target_can_target_unit(target_id, policy.air_capable) {
            return false;
        }

        if self.is_defense_route_player_unit(attacker_instance_id) {
            return self.is_target_in_defense_tile_range(
                attacker_instance_id,
                target_id,
                policy.defense_tile_range.as_ref(),
            );
        }

        let Some(attacker) = self.unit_body_view(attacker_instance_id) else {
            return false;
        };
        let Some(target) = self.unit_body_view(target_id) else {
            return false;
        };

        attacker.can_reach(&target, policy.range_units)
    }

    pub(in crate::game::battle::core) fn single_target_can_target_unit(
        &self,
        target_id: UnitInstanceId,
        air_capable: bool,
    ) -> bool {
        self.units
            .get(&target_id)
            .is_some_and(|target| !target.is_airborne() || air_capable)
    }

    pub(in crate::game::battle::core) fn is_defense_route_player_unit(
        &self,
        unit_id: UnitInstanceId,
    ) -> bool {
        matches!(
            self.scenario.tactical_plan.player_plan,
            PlayerMovementPlan::FixedDefense
        ) && self
            .units
            .get(&unit_id)
            .is_some_and(|unit| unit.owner == Side::Player)
    }

    pub(in crate::game::battle::core) fn is_target_in_defense_tile_range(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        tile_range: Option<&TileRangePattern>,
    ) -> bool {
        let Some(tile_range) = tile_range else {
            return false;
        };
        if tile_range.validate().is_err() {
            return false;
        }
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return false;
        };
        let Some(facing) = attacker.facing_direction else {
            return false;
        };
        let Some(attacker_body) = self.unit_body_view(attacker_instance_id) else {
            return false;
        };
        let Some(target_body) = self.unit_body_view(target_id) else {
            return false;
        };

        tile_range
            .contains_target_tile(
                attacker_body.position.project_to_tile(),
                facing,
                target_body.position.project_to_tile(),
            )
            .unwrap_or(false)
    }

    pub fn choose_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return None;
        };
        if attacker.is_airborne() && attacker.owner == Side::Opponent {
            return self.airborne_enemy_basic_attack_target_in_range(attacker_instance_id);
        }
        let attacker_owner = attacker.owner;
        let policy = self.basic_attack_range_policy_for_unit(attacker);
        let targeting_profile = attacker.basic_attack.targeting_profile;

        let mut candidates = Vec::new();
        for unit in self.units.values() {
            if unit.is_dead() || unit.owner == attacker_owner {
                continue;
            }

            if !self.is_basic_attack_target_in_range_with_policy(
                attacker_instance_id,
                unit.instance_id,
                &policy,
            ) {
                continue;
            }

            candidates.push(unit.instance_id);
        }

        if attacker_owner != Side::Player {
            return candidates.into_iter().min_by(|a, b| {
                self.compare_nearest_basic_attack_target(attacker_instance_id, *a, *b)
            });
        }

        candidates.into_iter().min_by(|a, b| {
            self.compare_targeting_profile_candidate(
                attacker_instance_id,
                targeting_profile,
                policy.air_capable,
                *a,
                *b,
            )
        })
    }

    fn compare_nearest_basic_attack_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> Ordering {
        let left_distance = self
            .basic_attack_target_distance_sq_world(attacker_instance_id, left)
            .unwrap_or(f32::MAX);
        let right_distance = self
            .basic_attack_target_distance_sq_world(attacker_instance_id, right)
            .unwrap_or(f32::MAX);
        left_distance
            .total_cmp(&right_distance)
            .then_with(|| self.compare_enemy_target_preference(left, right))
    }

    fn compare_targeting_profile_candidate(
        &self,
        attacker_instance_id: UnitInstanceId,
        profile: TargetingProfile,
        air_capable: bool,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> Ordering {
        match profile {
            TargetingProfile::DefaultForward => self.compare_default_forward_target(left, right),
            TargetingProfile::AirFirst => self
                .compare_air_first_target(left, right)
                .then_with(|| self.compare_default_forward_target(left, right)),
            TargetingProfile::LowDefenseFirst => self
                .compare_low_defense_target(left, right)
                .then_with(|| self.compare_default_forward_target(left, right)),
            TargetingProfile::LowMagicResistFirst => self
                .compare_low_magic_resist_target(left, right)
                .then_with(|| self.compare_default_forward_target(left, right)),
            TargetingProfile::SplashClusterFirst => self
                .compare_splash_cluster_target(attacker_instance_id, air_capable, left, right)
                .then_with(|| self.compare_default_forward_target(left, right)),
        }
    }

    fn compare_air_first_target(&self, left: UnitInstanceId, right: UnitInstanceId) -> Ordering {
        let left_airborne = self.units.get(&left).is_some_and(RuntimeUnit::is_airborne);
        let right_airborne = self.units.get(&right).is_some_and(RuntimeUnit::is_airborne);
        right_airborne.cmp(&left_airborne)
    }

    fn compare_low_defense_target(&self, left: UnitInstanceId, right: UnitInstanceId) -> Ordering {
        let left_defense = self
            .units
            .get(&left)
            .map(|unit| unit.stats.defense)
            .unwrap_or(i32::MAX);
        let right_defense = self
            .units
            .get(&right)
            .map(|unit| unit.stats.defense)
            .unwrap_or(i32::MAX);
        left_defense.cmp(&right_defense)
    }

    fn compare_low_magic_resist_target(
        &self,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> Ordering {
        let left_resist = self
            .units
            .get(&left)
            .map(|unit| unit.stats.magic_resist)
            .unwrap_or(i32::MAX);
        let right_resist = self
            .units
            .get(&right)
            .map(|unit| unit.stats.magic_resist)
            .unwrap_or(i32::MAX);
        left_resist.cmp(&right_resist)
    }

    fn compare_splash_cluster_target(
        &self,
        attacker_instance_id: UnitInstanceId,
        air_capable: bool,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> Ordering {
        let left_score = self.splash_cluster_score(attacker_instance_id, left, air_capable);
        let right_score = self.splash_cluster_score(attacker_instance_id, right, air_capable);
        right_score.cmp(&left_score)
    }

    fn splash_cluster_score(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        air_capable: bool,
    ) -> usize {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return 0;
        };
        let Some(target_body) = self.unit_body_view(target_id) else {
            return 0;
        };
        let radius_sq = SPLASH_CLUSTER_RADIUS_UNITS * SPLASH_CLUSTER_RADIUS_UNITS;
        self.units
            .values()
            .filter(|unit| {
                unit.owner != attacker.owner
                    && !unit.is_dead()
                    && self.single_target_can_target_unit(unit.instance_id, air_capable)
                    && self.unit_body_view(unit.instance_id).is_some_and(|body| {
                        target_body.position.distance_squared(body.position) <= radius_sq
                    })
            })
            .count()
    }

    fn compare_default_forward_target(
        &self,
        left: UnitInstanceId,
        right: UnitInstanceId,
    ) -> Ordering {
        match (
            self.enemy_route_remaining(left),
            self.enemy_route_remaining(right),
        ) {
            (Some(left_remaining), Some(right_remaining)) => left_remaining
                .total_cmp(&right_remaining)
                .then_with(|| self.compare_enemy_spawn_order(left, right)),
            _ => self
                .enemy_route_progress(right)
                .total_cmp(&self.enemy_route_progress(left))
                .then_with(|| self.compare_enemy_spawn_order(left, right)),
        }
    }

    fn compare_enemy_spawn_order(&self, left: UnitInstanceId, right: UnitInstanceId) -> Ordering {
        let left_spawn_order = self
            .units
            .get(&left)
            .map(|unit| unit.spawn_order)
            .unwrap_or(u64::MAX);
        let right_spawn_order = self
            .units
            .get(&right)
            .map(|unit| unit.spawn_order)
            .unwrap_or(u64::MAX);
        left_spawn_order
            .cmp(&right_spawn_order)
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    }

    pub(in crate::game::battle::core) fn airborne_enemy_basic_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let attacker = self.units.get(&attacker_instance_id)?;
        if !attacker.is_airborne() || attacker.owner != Side::Opponent {
            return None;
        }
        let policy = self.basic_attack_range_policy_for_unit(attacker);

        if let Some(protected_id) = self.protected_unit_id() {
            if self
                .units
                .get(&protected_id)
                .is_some_and(|unit| !unit.is_dead())
                && self.is_basic_attack_target_in_range_with_policy(
                    attacker_instance_id,
                    protected_id,
                    &policy,
                )
            {
                return Some(protected_id);
            }
        }

        self.units
            .values()
            .filter(|unit| {
                unit.owner == Side::Player
                    && unit.is_combatant()
                    && !unit.is_dead()
                    && self.is_basic_attack_target_in_range_with_policy(
                        attacker_instance_id,
                        unit.instance_id,
                        &policy,
                    )
            })
            .min_by(|a, b| {
                self.basic_attack_target_distance_sq_world(attacker_instance_id, a.instance_id)
                    .unwrap_or(f32::MAX)
                    .total_cmp(
                        &self
                            .basic_attack_target_distance_sq_world(
                                attacker_instance_id,
                                b.instance_id,
                            )
                            .unwrap_or(f32::MAX),
                    )
                    .then_with(|| a.instance_id.as_bytes().cmp(b.instance_id.as_bytes()))
            })
            .map(|unit| unit.instance_id)
    }

    pub(in crate::game::battle::core) fn choose_enemy_target_in_range_units(
        &self,
        caster_instance_id: UnitInstanceId,
        attacker_owner: Side,
        range_units: f32,
        air_capable: bool,
    ) -> Option<UnitInstanceId> {
        let caster_body = self.unit_body_view(caster_instance_id)?;
        let mut best: Option<(f32, UnitInstanceId)> = None;
        for unit in self.units.values() {
            if unit.is_dead() || unit.owner == attacker_owner {
                continue;
            }
            if unit.is_airborne() && !air_capable {
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
            DeliveryDef::TileArea { .. } => {
                panic!("basic_attack delivery TileArea is invalid; validate BasicAttackDef data")
            }
        }
    }

    pub(in crate::game::battle::core) fn basic_attack_range_policy_for_unit(
        &self,
        unit: &RuntimeUnit,
    ) -> BasicAttackRangePolicy {
        BasicAttackRangePolicy {
            delivery: match unit.basic_attack.delivery {
                DeliveryDef::Instant => AttackDelivery::Instant,
                DeliveryDef::Projectile { .. } => AttackDelivery::Projectile,
                DeliveryDef::TileArea { .. } => {
                    panic!(
                        "basic_attack delivery TileArea is invalid; validate BasicAttackDef data"
                    )
                }
            },
            range_units: unit.basic_attack.range_units.max(0.0),
            defense_tile_range: unit.basic_attack.defense_tile_range.clone(),
            air_capable: unit.basic_attack.air_capable,
        }
    }

    pub(in crate::game::battle::core) fn basic_attack_delivery_for_unit(
        &self,
        unit_id: UnitInstanceId,
    ) -> AttackDelivery {
        self.units
            .get(&unit_id)
            .map(|unit| self.basic_attack_range_policy_for_unit(unit).delivery)
            .unwrap_or(AttackDelivery::Instant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        ability::SkillActivationMode,
        battle::{
            core::{
                movement::{types::WorldVec2, ActionState},
                types::RuntimeUnit,
            },
            damage::DamageModifiers,
            scenario::{BattleScenario, EnemyMovementPlan},
            tile_range::FacingDirection,
            types::{BattleUnitRole, MobilityKind},
        },
        data::{
            abnormality_data::BasicAttackDef, equipment_data::WeaponRangeRole, GameDataBuilder,
        },
        resources::Position,
        stats::UnitStats,
    };

    fn test_core() -> BattleCore {
        BattleCore::new_from_scenario(
            BattleScenario::empty((8, 3)),
            GameDataBuilder::empty().build_arc(),
            0,
        )
    }

    fn unit(id: u128, owner: Side, position: WorldVec2) -> RuntimeUnit {
        let unit_id = UnitInstanceId::from(Uuid::from_u128(id));
        let mut body = crate::game::battle::core::movement::types::UnitBody::default();
        body.position = position;
        RuntimeUnit {
            instance_id: unit_id,
            spawn_order: id as u64,
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: BattleUnitRole::Combatant,
            base_uuid: Uuid::nil(),
            stats: UnitStats::with_values(100, 100, 10, 0, 1000),
            incoming_damage_modifiers: DamageModifiers::default(),
            basic_attack: BasicAttackDef {
                range_units: 10.0,
                defense_tile_range: Some(TileRangePattern {
                    include_anchor_tile: false,
                    rows: vec![
                        "XXXXXXXXXXX".to_string(),
                        "XXXXXXXXXXX".to_string(),
                        "XXXXX@XXXXX".to_string(),
                        "XXXXXXXXXXX".to_string(),
                        "XXXXXXXXXXX".to_string(),
                    ],
                }),
                air_capable: true,
                range_role: WeaponRangeRole::Ranged,
                ..Default::default()
            },
            skill_id: None,
            skill_activation_mode: SkillActivationMode::Auto,
            body,
            tactical_anchor: None,
            enemy_movement_plan: None,
            block_capacity: 0,
            block_radius_units: 0.0,
            blockable: true,
            mobility_kind: MobilityKind::Ground,
            target_traits: Vec::new(),
            facing_direction: Some(FacingDirection::Up),
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

    fn add_attacker(core: &mut BattleCore, profile: TargetingProfile) -> UnitInstanceId {
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut attacker = unit(1, Side::Player, WorldVec2::new(0.0, 1.0));
        attacker.basic_attack.targeting_profile = profile;
        core.units.insert(attacker_id, attacker);
        attacker_id
    }

    fn add_enemy(
        core: &mut BattleCore,
        id: u128,
        position: WorldVec2,
        route_progress_x: f32,
    ) -> UnitInstanceId {
        let enemy_id = UnitInstanceId::from(Uuid::from_u128(id));
        let mut enemy = unit(id, Side::Opponent, position);
        enemy.enemy_movement_plan = Some(EnemyMovementPlan::PathAlongCells {
            cells: vec![
                Position { x: 0, y: 1 },
                Position {
                    x: route_progress_x as i32,
                    y: 1,
                },
            ],
        });
        core.units.insert(enemy_id, enemy);
        enemy_id
    }

    #[test]
    fn default_forward_prefers_enemy_closest_to_route_end() {
        let mut core = test_core();
        let attacker_id = add_attacker(&mut core, TargetingProfile::DefaultForward);
        let early = add_enemy(&mut core, 10, WorldVec2::new(1.0, 1.0), 5.0);
        let late = add_enemy(&mut core, 11, WorldVec2::new(4.0, 1.0), 5.0);
        let early_remaining = core.enemy_route_remaining(early).unwrap();
        let late_remaining = core.enemy_route_remaining(late).unwrap();
        assert!(
            late_remaining < early_remaining,
            "late={late_remaining}, early={early_remaining}"
        );

        assert_eq!(core.choose_attack_target_in_range(attacker_id), Some(late));
        assert_ne!(core.choose_attack_target_in_range(attacker_id), Some(early));
    }

    #[test]
    fn air_first_prefers_airborne_target_before_forward_fallback() {
        let mut core = test_core();
        let attacker_id = add_attacker(&mut core, TargetingProfile::AirFirst);
        let ground = add_enemy(&mut core, 10, WorldVec2::new(4.0, 1.0), 5.0);
        let airborne = add_enemy(&mut core, 11, WorldVec2::new(1.0, 1.0), 5.0);
        core.units.get_mut(&airborne).unwrap().mobility_kind = MobilityKind::Airborne;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(airborne)
        );
        assert_ne!(
            core.choose_attack_target_in_range(attacker_id),
            Some(ground)
        );
    }

    #[test]
    fn low_defense_and_low_magic_resist_profiles_prefer_weaker_stat() {
        let mut defense_core = test_core();
        let defense_attacker = add_attacker(&mut defense_core, TargetingProfile::LowDefenseFirst);
        let sturdy = add_enemy(&mut defense_core, 10, WorldVec2::new(4.0, 1.0), 5.0);
        let fragile = add_enemy(&mut defense_core, 11, WorldVec2::new(1.0, 1.0), 5.0);
        defense_core.units.get_mut(&sturdy).unwrap().stats.defense = 50;
        defense_core.units.get_mut(&fragile).unwrap().stats.defense = -5;
        assert_eq!(
            defense_core.choose_attack_target_in_range(defense_attacker),
            Some(fragile)
        );

        let mut resist_core = test_core();
        let resist_attacker = add_attacker(&mut resist_core, TargetingProfile::LowMagicResistFirst);
        let high_resist = add_enemy(&mut resist_core, 20, WorldVec2::new(4.0, 1.0), 5.0);
        let low_resist = add_enemy(&mut resist_core, 21, WorldVec2::new(1.0, 1.0), 5.0);
        resist_core
            .units
            .get_mut(&high_resist)
            .unwrap()
            .stats
            .magic_resist = 70;
        resist_core
            .units
            .get_mut(&low_resist)
            .unwrap()
            .stats
            .magic_resist = -10;
        assert_eq!(
            resist_core.choose_attack_target_in_range(resist_attacker),
            Some(low_resist)
        );
    }

    #[test]
    fn splash_cluster_prefers_target_with_more_nearby_enemies() {
        let mut core = test_core();
        let attacker_id = add_attacker(&mut core, TargetingProfile::SplashClusterFirst);
        let isolated = add_enemy(&mut core, 10, WorldVec2::new(4.5, 1.0), 5.0);
        let clustered = add_enemy(&mut core, 11, WorldVec2::new(1.0, 1.0), 5.0);
        add_enemy(&mut core, 12, WorldVec2::new(0.6, 1.0), 5.0);
        add_enemy(&mut core, 13, WorldVec2::new(1.0, 0.6), 5.0);

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(clustered)
        );
        assert_ne!(
            core.choose_attack_target_in_range(attacker_id),
            Some(isolated)
        );
    }
}
