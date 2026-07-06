use std::cmp::Ordering;

use uuid::Uuid;

use super::RuntimeUnit;
use crate::game::{
    ability::DeliveryDef,
    battle::types::effective_basic_attack_tile_range,
    battle::{
        core::BattleCore,
        event_log::AttackDelivery,
        ids::UnitInstanceId,
        tile_range::{TileRangePattern, TileRangePolicy},
    },
    data::equipment_data::TargetingProfile,
    enums::Side,
};

#[derive(Debug, Clone, PartialEq)]
pub(in crate::game::battle::core) struct BasicAttackRangePolicy {
    pub delivery: AttackDelivery,
    pub range_policy: TileRangePolicy,
    pub tile_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

impl BattleCore {
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

        self.is_target_in_tile_range_policy(
            attacker_instance_id,
            target_id,
            policy.range_policy,
            policy.tile_range.as_ref(),
        )
    }

    pub(in crate::game::battle::core) fn single_target_can_target_unit(
        &self,
        target_id: UnitInstanceId,
        air_capable: bool,
    ) -> bool {
        self.units
            .get(&target_id)
            .is_some_and(|target| target.is_active() && (!target.is_airborne() || air_capable))
    }

    pub(in crate::game::battle::core) fn is_target_in_tile_range_policy(
        &self,
        attacker_instance_id: UnitInstanceId,
        target_id: UnitInstanceId,
        range_policy: TileRangePolicy,
        tile_range: Option<&TileRangePattern>,
    ) -> bool {
        if matches!(range_policy, TileRangePolicy::WholeFieldValidTiles) {
            let Some(target_body) = self.unit_body_view(target_id) else {
                return false;
            };
            return self
                .battlefield
                .is_valid_tile(target_body.position.project_to_tile());
        }

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
            if !unit.is_active() || unit.owner == attacker_owner {
                continue;
            }

            if !self.is_basic_attack_target_in_range_with_policy(
                attacker_instance_id,
                unit.instance_id,
                &policy,
            ) {
                continue;
            }
            if !self.is_basic_attack_useful_target(attacker_instance_id, unit.instance_id) {
                continue;
            }

            candidates.push(unit.instance_id);
        }

        candidates
            .into_iter()
            .min_by(|a, b| self.compare_targeting_profile_candidate(targeting_profile, *a, *b))
    }

    fn compare_targeting_profile_candidate(
        &self,
        profile: TargetingProfile,
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
                .is_some_and(|unit| unit.is_active())
                && self.is_basic_attack_target_in_range_with_policy(
                    attacker_instance_id,
                    protected_id,
                    &policy,
                )
                && self.is_basic_attack_useful_target(attacker_instance_id, protected_id)
            {
                return Some(protected_id);
            }
        }

        self.units
            .values()
            .filter(|unit| {
                unit.owner == Side::Player
                    && unit.is_combatant()
                    && unit.is_active()
                    && self.is_basic_attack_target_in_range_with_policy(
                        attacker_instance_id,
                        unit.instance_id,
                        &policy,
                    )
                    && self.is_basic_attack_useful_target(attacker_instance_id, unit.instance_id)
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
            range_policy: unit.basic_attack.range_policy,
            tile_range: match unit.basic_attack.range_policy {
                TileRangePolicy::Pattern => {
                    Some(effective_basic_attack_tile_range(&unit.basic_attack))
                }
                TileRangePolicy::WholeFieldValidTiles => None,
            },
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
            scenario::{BattleScenario, EnemyMovementPlan, PlayerMovementPlan},
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
            lifecycle: crate::game::battle::core::types::RuntimeUnitLifecycle::Active,
            spawn_order: id as u64,
            source_owned_uuid: unit_id.as_uuid(),
            owner,
            role: BattleUnitRole::Combatant,
            threat_class: crate::game::battle::types::BattleUnitThreatClass::Normal,
            base_uuid: Uuid::nil(),
            source_identity: crate::game::battle::types::BattleUnitSourceIdentity::TestFixture {
                base_uuid: Uuid::nil(),
            },
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
            ranged_reposition_until_ms: 0,
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

    fn add_enemy_attacker(core: &mut BattleCore, profile: TargetingProfile) -> UnitInstanceId {
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(1));
        let mut attacker = unit(1, Side::Opponent, WorldVec2::new(0.0, 1.0));
        attacker.basic_attack.targeting_profile = profile;
        core.units.insert(attacker_id, attacker);
        attacker_id
    }

    fn add_player_target(core: &mut BattleCore, id: u128, position: WorldVec2) -> UnitInstanceId {
        let target_id = UnitInstanceId::from(Uuid::from_u128(id));
        core.units
            .insert(target_id, unit(id, Side::Player, position));
        target_id
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
    fn whole_field_basic_attack_uses_valid_tiles_without_facing() {
        let mut core = test_core();
        let attacker_id = add_attacker(&mut core, TargetingProfile::DefaultForward);
        let target_id = add_enemy(&mut core, 10, WorldVec2::new(7.0, 2.0), 5.0);
        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.facing_direction = None;
        attacker.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        attacker.basic_attack.defense_tile_range = None;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(target_id)
        );
        assert!(core.resolve_basic_attack(attacker_id, target_id, 0));
    }

    #[test]
    fn whole_field_basic_attack_rejects_void_tiles() {
        let mut core = test_core();
        core.battlefield =
            crate::game::battle::battlefield::BattlefieldLayout::new_with_valid_tiles(
                3,
                3,
                vec![Position::new(0, 1), Position::new(1, 1)],
            );
        let attacker_id = add_attacker(&mut core, TargetingProfile::DefaultForward);
        let void_target_id = add_enemy(&mut core, 10, WorldVec2::new(2.0, 1.0), 5.0);
        let valid_target_id = add_enemy(&mut core, 11, WorldVec2::new(1.0, 1.0), 5.0);
        let attacker = core.units.get_mut(&attacker_id).unwrap();
        attacker.facing_direction = None;
        attacker.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
        attacker.basic_attack.defense_tile_range = None;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(valid_target_id)
        );
        assert!(!core.resolve_basic_attack(attacker_id, void_target_id, 0));
        assert!(core.resolve_basic_attack(attacker_id, valid_target_id, 0));
    }

    #[test]
    fn basic_attack_targeting_excludes_zero_damage_without_hostile_effect() {
        let mut core = test_core();
        let attacker_id = add_attacker(&mut core, TargetingProfile::DefaultForward);
        core.units.get_mut(&attacker_id).unwrap().stats.attack = 0;
        add_enemy(&mut core, 10, WorldVec2::new(1.0, 1.0), 5.0);

        assert_eq!(core.choose_attack_target_in_range(attacker_id), None);
        assert_eq!(
            core.select_basic_attack_target(attacker_id, None, None),
            None
        );
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
    fn enemy_ranged_low_defense_profile_beats_nearest_target() {
        let mut core = test_core();
        let attacker_id = add_enemy_attacker(&mut core, TargetingProfile::LowDefenseFirst);
        let near_sturdy = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
        let far_fragile = add_player_target(&mut core, 11, WorldVec2::new(4.0, 1.0));
        core.units.get_mut(&near_sturdy).unwrap().stats.defense = 50;
        core.units.get_mut(&far_fragile).unwrap().stats.defense = -5;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(far_fragile)
        );
    }

    #[test]
    fn enemy_ranged_low_magic_resist_profile_beats_nearest_target() {
        let mut core = test_core();
        let attacker_id = add_enemy_attacker(&mut core, TargetingProfile::LowMagicResistFirst);
        let near_resistant = add_player_target(&mut core, 20, WorldVec2::new(1.0, 1.0));
        let far_vulnerable = add_player_target(&mut core, 21, WorldVec2::new(4.0, 1.0));
        core.units
            .get_mut(&near_resistant)
            .unwrap()
            .stats
            .magic_resist = 70;
        core.units
            .get_mut(&far_vulnerable)
            .unwrap()
            .stats
            .magic_resist = -10;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(far_vulnerable)
        );
    }

    #[test]
    fn enemy_ranged_blocker_overrides_targeting_profile() {
        let mut core = test_core();
        core.scenario.tactical_plan.player_plan = PlayerMovementPlan::FixedDefense;
        let attacker_id = add_enemy_attacker(&mut core, TargetingProfile::LowDefenseFirst);
        let blocker_id = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
        let fragile_id = add_player_target(&mut core, 11, WorldVec2::new(4.0, 1.0));
        core.units.get_mut(&blocker_id).unwrap().stats.defense = 50;
        core.units.get_mut(&fragile_id).unwrap().stats.defense = -5;
        let blocker = core.units.get_mut(&blocker_id).unwrap();
        blocker.block_capacity = 1;
        blocker.block_radius_units = 2.0;
        core.refresh_block_state();

        assert_eq!(core.blocked_by(attacker_id), Some(blocker_id));
        assert_eq!(
            core.select_basic_attack_target(attacker_id, None, None),
            Some(blocker_id)
        );
    }

    #[test]
    fn enemy_ranged_persisted_target_overrides_new_profile_selection() {
        let mut core = test_core();
        core.scenario.tactical_plan.player_plan = PlayerMovementPlan::FixedDefense;
        let attacker_id = add_enemy_attacker(&mut core, TargetingProfile::LowDefenseFirst);
        let persisted_id = add_player_target(&mut core, 20, WorldVec2::new(1.0, 1.0));
        let fragile_id = add_player_target(&mut core, 21, WorldVec2::new(4.0, 1.0));
        core.units.get_mut(&persisted_id).unwrap().stats.defense = 50;
        core.units.get_mut(&fragile_id).unwrap().stats.defense = -5;

        assert_eq!(
            core.select_basic_attack_target(attacker_id, Some(persisted_id), None),
            Some(persisted_id)
        );
    }

    #[test]
    fn enemy_ranged_profile_selection_excludes_out_of_range_targets() {
        let mut core = test_core();
        let attacker_id = add_enemy_attacker(&mut core, TargetingProfile::LowDefenseFirst);
        let in_range_id = add_player_target(&mut core, 30, WorldVec2::new(1.0, 1.0));
        let out_of_range_id = add_player_target(&mut core, 31, WorldVec2::new(7.0, 1.0));
        core.units.get_mut(&in_range_id).unwrap().stats.defense = 50;
        core.units.get_mut(&out_of_range_id).unwrap().stats.defense = -5;

        assert_eq!(
            core.choose_attack_target_in_range(attacker_id),
            Some(in_range_id)
        );
    }
}
