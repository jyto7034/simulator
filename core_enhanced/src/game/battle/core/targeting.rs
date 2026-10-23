use crate::authoring::control;
use std::{borrow::Cow, cmp::Ordering};

use super::RuntimeUnit;
use crate::game::{
    ability::DeliveryDef,
    battle::types::default_basic_attack_tile_range_for_role,
    battle::{
        core::BattleCore,
        event_log::AttackDelivery,
        ids::UnitInstanceId,
        tile_range::{TileRangePattern, TileRangePolicy, TileReachQuery},
    },
    data::equipment_data::TargetingProfile,
    enums::Side,
};

#[derive(Debug, Clone, PartialEq)]
pub(in crate::game::battle::core) struct BasicAttackRangePolicy<'a> {
    pub delivery: AttackDelivery,
    pub range_policy: TileRangePolicy,
    pub tile_range: Option<Cow<'a, TileRangePattern>>,
    pub air_capable: bool,
}

impl BattleCore {
    pub(crate) fn basic_action_snapshot(
        &self,
        actor: UnitInstanceId,
        include_candidates: bool,
    ) -> Result<crate::game::behavior::LiveBattleBasicActionDto, crate::game::behavior::GameError>
    {
        use crate::game::behavior::{LiveBattleBasicActionDto, LiveBattleBasicActionKind};
        let unit = self
            .units
            .get(&actor)
            .ok_or(crate::game::behavior::GameError::UnitNotFound)?;
        let blocking = self
            .block_state
            .blocker_to_enemies
            .get(&actor)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let candidate_capacity = if include_candidates {
            self.units.len()
        } else {
            0
        };
        let slots = candidate_capacity
            .checked_mul(2)
            .and_then(|n| n.checked_add(blocking.len()));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            slots.and_then(|n| n.checked_mul(std::mem::size_of::<UnitInstanceId>())),
            "basic_action_query_storage",
        )?;
        control::charge(
            (candidate_capacity + blocking.len() + 1) as u64,
            "basic_action_query",
        )?;
        let mut result = LiveBattleBasicActionDto {
            kind: if unit.basic_attack.action.is_heal() {
                LiveBattleBasicActionKind::Heal
            } else {
                LiveBattleBasicActionKind::Damage
            },
            blocked_by: self.blocked_by(actor),
            blocking: blocking.to_vec(),
            preview_available: include_candidates,
            selected_target_id: None,
            eligible_target_ids: Vec::with_capacity(candidate_capacity),
            block_exception_target_ids: Vec::with_capacity(candidate_capacity),
        };
        if include_candidates {
            for target in self.authoring_target_candidates()? {
                let eligible = if unit.basic_attack.action.is_heal() {
                    self.is_basic_heal_target(actor, target.instance_id)
                } else {
                    self.is_alive_enemy(target.instance_id, unit.owner)
                        && self.is_basic_attack_target_in_range(actor, target.instance_id)
                        && self.is_basic_attack_useful_target(actor, target.instance_id)
                        && self
                            .taunt_source(actor)
                            .is_none_or(|source| source == target.instance_id)
                };
                if eligible {
                    result.eligible_target_ids.push(target.instance_id);
                    if !unit.basic_attack.action.is_heal()
                        && (self.blocked_by(actor) == Some(target.instance_id)
                            || self.blocked_by(target.instance_id) == Some(actor))
                    {
                        result.block_exception_target_ids.push(target.instance_id);
                    }
                }
            }
            result.eligible_target_ids.sort_unstable();
            result.block_exception_target_ids.sort_unstable();
            result.selected_target_id =
                self.select_basic_attack_target(actor, unit.current_target, None);
        }
        Ok(result)
    }

    pub(crate) fn runtime_range_previews(
        &self,
        unit_id: UnitInstanceId,
        position: crate::game::resources::Position,
        facing: Option<crate::game::battle::tile_range::FacingDirection>,
    ) -> Result<crate::game::behavior::LiveBattleRangePreviewsDto, crate::game::behavior::GameError>
    {
        let unit = self
            .units
            .get(&unit_id)
            .ok_or(crate::game::behavior::GameError::UnitNotFound)?;
        Ok(
            crate::game::range_preview::range_previews_for_runtime_unit_at(
                unit,
                self.game_data.as_ref(),
                &self.battlefield,
                position,
                facing,
            ),
        )
    }

    /// Keep accounting order stable without changing ordinary target scans or
    /// the existing gameplay comparators and current-target preference.
    pub(in crate::game::battle::core) fn authoring_target_candidates(
        &self,
    ) -> Result<impl Iterator<Item = &RuntimeUnit>, crate::game::behavior::GameError> {
        let mut ordinary = self.units.values();
        let mut ordered = if control::is_active() {
            self.admit_authoring_runtime_growth(0, "target_candidate_storage")?;
            let count = self.units.len() as u64;
            control::charge(
                count
                    .saturating_mul(u64::from(usize::BITS - self.units.len().leading_zeros()) + 2)
                    .saturating_add(1),
                "target_candidate_order",
            )?;
            let mut units: Vec<_> = self.units.values().collect();
            units.sort_unstable_by_key(|unit| unit.instance_id);
            Some(units.into_iter())
        } else {
            None
        };
        Ok(std::iter::from_fn(move || match &mut ordered {
            Some(units) => units.next(),
            None => ordinary.next(),
        }))
    }

    pub(super) fn basic_attack_target_distance_sq_world(
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
        policy: &BasicAttackRangePolicy<'_>,
    ) -> bool {
        if !self.approach_taunt_attack_allowed(attacker_instance_id, target_id) { return false; }
        if self.units.get(&target_id).is_some_and(|target| {
            target.role == crate::game::battle::types::BattleUnitRole::IndependentWall
        }) && self.blocked_by(attacker_instance_id) != Some(target_id)
        {
            return false;
        }
        if !self.single_target_can_target_unit(target_id, policy.air_capable) {
            return false;
        }

        if self
            .units
            .get(&attacker_instance_id)
            .is_some_and(|attacker| {
                !attacker.basic_attack.action.is_heal()
                    && self
                        .units
                        .get(&target_id)
                        .is_some_and(|target| target.owner != attacker.owner)
            })
            && self.approach_taunt(attacker_instance_id).is_none()
            && (self.blocked_by(attacker_instance_id) == Some(target_id)
                || self.blocked_by(target_id) == Some(attacker_instance_id))
        {
            return true;
        }

        if let Some(line) = self.basic_line_profile(attacker_instance_id) {
            return self.line_target_in_range(attacker_instance_id,target_id,line);
        }
        self.is_target_in_tile_range_policy(
            attacker_instance_id,
            target_id,
            policy.range_policy,
            policy.tile_range.as_deref(),
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
        let Some(target_body) = self.unit_body_view(target_id) else {
            return false;
        };
        self.tile_reach_query_for_unit(attacker_instance_id, range_policy, tile_range)
            .is_some_and(|query| query.contains(target_body.position.project_to_tile()))
    }

    pub(in crate::game::battle::core) fn tile_reach_query_for_unit<'a>(
        &'a self,
        source_instance_id: UnitInstanceId,
        range_policy: TileRangePolicy,
        tile_range: Option<&'a TileRangePattern>,
    ) -> Option<TileReachQuery<'a>> {
        control::poll("target_tile_query").ok()?;
        if matches!(range_policy, TileRangePolicy::Pattern) {
            if let Some(pattern) = tile_range {
                // Constructor validation and the caller's contains/materialize
                // both project the pattern. Fail before the pure query's expects.
                Self::charge_authoring_tile_pattern(pattern, 2).ok()?;
            }
        }
        let (anchor, facing) = match range_policy {
            TileRangePolicy::Pattern => {
                let source = self.units.get(&source_instance_id)?;
                let source_body = self.unit_body_view(source_instance_id)?;
                (
                    Some(source_body.position.project_to_tile()),
                    source.facing_direction,
                )
            }
            TileRangePolicy::WholeFieldValidTiles => (None, None),
        };
        TileReachQuery::try_new(&self.battlefield, range_policy, tile_range, anchor, facing).ok()
    }

    pub fn choose_attack_target_in_range(
        &self,
        attacker_instance_id: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        let Some(attacker) = self.units.get(&attacker_instance_id) else {
            return None;
        };
        if attacker.basic_attack.action.is_heal() {
            return self.choose_basic_heal_target(attacker_instance_id);
        }
        if let Some(forced) = self.taunt_source(attacker_instance_id) {
            return (self.is_alive_enemy(forced, attacker.owner)
                && self.is_basic_attack_target_in_range(attacker_instance_id, forced)
                && self.is_basic_attack_useful_target(attacker_instance_id, forced))
            .then_some(forced);
        }
        if attacker.is_airborne() && attacker.owner == Side::Opponent {
            return self.airborne_enemy_basic_attack_target_in_range(attacker_instance_id);
        }
        let attacker_owner = attacker.owner;
        let policy = self.basic_attack_range_policy_for_unit(attacker);
        let targeting_profile = attacker.basic_attack.targeting_profile;

        self.authoring_target_candidates()
            .ok()?
            .filter(|unit| unit.is_active() && unit.owner != attacker_owner)
            .filter(|unit| {
                self.is_basic_attack_target_in_range_with_policy(
                    attacker_instance_id,
                    unit.instance_id,
                    &policy,
                )
            })
            .filter(|unit| {
                self.is_basic_attack_useful_target(attacker_instance_id, unit.instance_id)
            })
            .map(|unit| unit.instance_id)
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

    pub(super) fn compare_default_forward_target(
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

    pub(super) fn compare_enemy_spawn_order(&self, left: UnitInstanceId, right: UnitInstanceId) -> Ordering {
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

        self.authoring_target_candidates()
            .ok()?
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

    pub(in crate::game::battle::core) fn basic_attack_range_policy_for_unit<'a>(
        &self,
        unit: &'a RuntimeUnit,
    ) -> BasicAttackRangePolicy<'a> {
        BasicAttackRangePolicy {
            delivery: Self::basic_attack_delivery(&unit.basic_attack.delivery),
            range_policy: unit.basic_attack.range_policy,
            tile_range: match unit.basic_attack.range_policy {
                TileRangePolicy::Pattern => Some(
                    unit.basic_attack
                        .defense_tile_range
                        .as_ref()
                        .map(Cow::Borrowed)
                        .unwrap_or_else(|| {
                            Cow::Owned(default_basic_attack_tile_range_for_role(
                                unit.basic_attack.range_role,
                            ))
                        }),
                ),
                TileRangePolicy::WholeFieldValidTiles => None,
            },
            air_capable: unit.basic_attack.air_capable,
        }
    }

    fn basic_attack_delivery(delivery: &DeliveryDef) -> AttackDelivery {
        match delivery {
            DeliveryDef::Instant => AttackDelivery::Instant,
            DeliveryDef::Projectile { .. } => AttackDelivery::Projectile,
            DeliveryDef::TileArea { .. } => AttackDelivery::Instant,
        }
    }

    pub(in crate::game::battle::core) fn basic_attack_delivery_for_unit(
        &self,
        unit_id: UnitInstanceId,
    ) -> AttackDelivery {
        self.units
            .get(&unit_id)
            .map(|unit| Self::basic_attack_delivery(&unit.basic_attack.delivery))
            .unwrap_or(AttackDelivery::Instant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        ability::{DeliveryDef, SkillActivationMode},
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
    use uuid::Uuid;

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
            employee_trauma: None,
            corroded: false,
            fragment_source: None,
            basic_attack_speed_percent: 100,
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
            equipment_slots: None,
            stats: UnitStats::with_values(100, 100, 10, 0, 1000),
            incoming_damage_modifiers: DamageModifiers::default(),
            basic_attack: BasicAttackDef {
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
            basic_attack_epoch: 0,
            basic_attack_windup_until_ms: None,
            ranged_reposition_until_ms: 0,
            resonance_current: 0,
            resonance_max: 100,
            resonance_lock_ms: 0,
            resonance_use: None,
            next_resonance_pulse_ms: None,
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

    #[test]
    fn basic_attack_release_delivery_reads_runtime_unit_profile() {
        let mut core = test_core();
        let attacker_id = UnitInstanceId::from(Uuid::from_u128(0xD311));
        let mut attacker = unit(0xD311, Side::Player, WorldVec2::new(0.0, 1.0));
        attacker.basic_attack.delivery = DeliveryDef::Projectile {
            speed_units_per_ms: 2_500,
            hit_policy: Default::default(),
            allow_targetless_cast: false,
            max_range_tiles: None,
            max_lifetime_ms: None,
            max_kills: None,
            max_pierces: None,
            collision: Default::default(),
        };
        core.units.insert(attacker_id, attacker);

        assert_eq!(
            core.basic_attack_delivery_for_unit(attacker_id),
            AttackDelivery::Projectile
        );
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
        std::sync::Arc::make_mut(&mut core.scenario)
            .tactical_plan
            .player_plan = PlayerMovementPlan::FixedDefense;
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
        std::sync::Arc::make_mut(&mut core.scenario)
            .tactical_plan
            .player_plan = PlayerMovementPlan::FixedDefense;
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
    mod basic_heal_tests {
        use super::*;
        use crate::game::{
            battle::{
                enums::BattleEvent,
                event_log::{AttackKind, BattleEventCause, BattleLogEvent, HpChangeReason},
            },
            data::abnormality_data::BasicAttackAction,
        };

        fn healer(core: &mut BattleCore) -> UnitInstanceId {
            let id = add_attacker(core, TargetingProfile::DefaultForward);
            let unit = core.units.get_mut(&id).unwrap();
            unit.basic_attack.action = BasicAttackAction::Heal {
                flat: 10,
                spell_power_percent: 50,
                footprint: None,
            };
            unit.basic_attack.range_policy = TileRangePolicy::WholeFieldValidTiles;
            unit.basic_attack.defense_tile_range = None;
            unit.basic_attack.windup_ms = 50;
            unit.stats.spell_power = 20;
            id
        }

        fn start(core: &mut BattleCore, actor: UnitInstanceId) {
            core.process_event(
                BattleEvent::AttackStart {
                    time_ms: 0,
                    attacker_instance_id: actor,
                    target_instance_id: None,
                    schedule_next: false,
                    cause: BattleEventCause::default(),
                },
                0,
            )
            .unwrap();
        }

        #[test]
        fn basic_heal_selection_uses_ratio_distance_id_and_allows_self() {
            let mut core = test_core();
            let actor = healer(&mut core);
            let near = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            let far = add_player_target(&mut core, 11, WorldVec2::new(2.0, 1.0));
            let enemy = add_enemy(&mut core, 12, WorldVec2::new(1.0, 1.0), 1.0);
            core.units.get_mut(&enemy).unwrap().stats.current_health = 1;
            core.units.get_mut(&near).unwrap().stats.current_health = 50;
            core.units.get_mut(&far).unwrap().stats.current_health = 40;
            assert_eq!(
                core.select_basic_attack_target(actor, Some(near), Some(enemy)),
                Some(far)
            );
            core.units.get_mut(&near).unwrap().stats.current_health = 40;
            assert_eq!(core.choose_attack_target_in_range(actor), Some(near));
            core.units.get_mut(&far).unwrap().body.position = WorldVec2::new(1.0, 1.0);
            assert_eq!(core.choose_attack_target_in_range(actor), Some(near));
            core.units.get_mut(&actor).unwrap().stats.current_health = 10;
            assert_eq!(core.choose_attack_target_in_range(actor), Some(actor));
        }

        #[test]
        fn basic_heal_zero_effective_amount_is_idle_and_absent_from_query() {
            for footprint in [
                None,
                Some(TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["X@X".into()],
                }),
            ] {
                for (flat, spell_power_percent, spell_power) in [(0, 0, 0), (0, 100, 0), (0, 1, 99)]
                {
                    let mut core = test_core();
                    let actor = healer(&mut core);
                    let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
                    core.units.get_mut(&target).unwrap().stats.current_health = 50;
                    let unit = core.units.get_mut(&actor).unwrap();
                    unit.basic_attack.action = BasicAttackAction::Heal {
                        flat,
                        spell_power_percent,
                        footprint: footprint.clone(),
                    };
                    unit.stats.spell_power = spell_power;
                    assert!(unit.basic_attack.try_validate_runtime_contract().is_ok());
                    assert_eq!(core.choose_attack_target_in_range(actor), None);
                    let preview = core.basic_action_snapshot(actor, true).unwrap();
                    assert!(preview.eligible_target_ids.is_empty());
                    assert_eq!(preview.selected_target_id, None);
                    start(&mut core, actor);
                    assert!(core.event_queue.is_empty());
                    assert_eq!(core.units[&target].stats.current_health, 50);
                    assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
                    assert!(!core.event_log.entries_after_seq(0).iter().any(|entry| {
                        matches!(entry.event, BattleLogEvent::AttackStart { .. })
                    }));

                    if spell_power_percent > 0 {
                        // A current stat increase makes the same authored definition useful.
                        core.units.get_mut(&actor).unwrap().stats.spell_power = 100;
                        let preview = core.basic_action_snapshot(actor, true).unwrap();
                        assert_eq!(preview.eligible_target_ids, vec![target]);
                        assert_eq!(preview.selected_target_id, Some(target));
                        start(&mut core, actor);
                        let event = core.event_queue.pop().unwrap();
                        core.process_event(event, 50).unwrap();
                        assert!(core.units[&target].stats.current_health > 50);
                        assert_eq!(core.runtime_fact_summary().damage_hp_lost, 0);
                    }
                }
            }
        }

        #[test]
        fn basic_heal_release_rechecks_current_amount_and_closes_failed_action() {
            for footprint in [
                None,
                Some(TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["X@X".into()],
                }),
            ] {
                let mut core = test_core();
                let actor = healer(&mut core);
                let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
                let other = add_player_target(&mut core, 11, WorldVec2::new(5.0, 1.0));
                core.units.get_mut(&target).unwrap().stats.current_health = 50;
                core.units.get_mut(&other).unwrap().stats.current_health = 60;
                core.units.get_mut(&actor).unwrap().basic_attack.action = BasicAttackAction::Heal {
                    flat: 0,
                    spell_power_percent: 50,
                    footprint,
                };
                start(&mut core, actor);
                let event = core.event_queue.pop().unwrap();
                let mut positive = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
                // Both copies retain the fixed target even when another ally becomes needier.
                for candidate in [&mut core, &mut positive] {
                    candidate
                        .units
                        .get_mut(&other)
                        .unwrap()
                        .stats
                        .current_health = 1;
                }
                core.units.get_mut(&actor).unwrap().stats.spell_power = 0;
                positive.units.get_mut(&actor).unwrap().stats.spell_power = 40;
                let preview = core.basic_action_snapshot(actor, true).unwrap();
                assert!(preview.eligible_target_ids.is_empty());
                core.process_event(event.clone(), 50).unwrap();
                positive.process_event(event, 50).unwrap();
                assert_eq!(core.units[&target].stats.current_health, 50);
                assert_eq!(positive.units[&target].stats.current_health, 70);
                for candidate in [&core, &positive] {
                    assert_eq!(candidate.units[&other].stats.current_health, 1);
                    assert_eq!(candidate.runtime_fact_view().active_actions().count(), 0);
                    assert_eq!(candidate.units[&actor].basic_attack_windup_until_ms, None);
                    assert_eq!(candidate.runtime_fact_summary().damage_hp_lost, 0);
                }
                assert!(core
                    .event_log
                    .entries_after_seq(0)
                    .iter()
                    .any(|entry| { matches!(entry.event, BattleLogEvent::AttackMiss { .. }) }));
                assert!(!core.event_log.entries_after_seq(0).iter().any(|entry| {
                    matches!(entry.event, BattleLogEvent::HpChanged { delta, .. } if delta > 0)
                }));
            }
        }

        #[test]
        fn basic_heal_queue_clone_releases_pure_actual_hp_and_closes_source() {
            let mut core = test_core();
            let actor = healer(&mut core);
            let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            core.units.get_mut(&target).unwrap().stats.current_health = 95;
            start(&mut core, actor);
            let event = core.event_queue.pop().unwrap();
            assert!(matches!(
                event.event.view().as_ref(),
                BattleEvent::BasicHealResolve { .. }
            ));
            let mut clone = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
            clone.process_event(event.clone(), 50).unwrap();
            core.process_event(event, 50).unwrap();
            assert_eq!(core.units[&target].stats.current_health, 100);
            assert_eq!(clone.units[&target].stats.current_health, 100);
            let entries = core.event_log.entries_after_seq(0);
            let changes: Vec<_> = entries
                .iter()
                .filter_map(|entry| match &entry.event {
                    BattleLogEvent::HpChanged {
                        delta,
                        damage_source,
                        critical,
                        ..
                    } => Some((*delta, damage_source, critical)),
                    _ => None,
                })
                .collect();
            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].0, 5);
            assert!(changes[0].1.is_none() && changes[0].2.is_none());
            assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
        }

        #[test]
        fn basic_heal_dead_fixed_target_is_not_revived_or_retargeted() {
            let mut core = test_core();
            let actor = healer(&mut core);
            let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            let other = add_player_target(&mut core, 11, WorldVec2::new(2.0, 1.0));
            core.units.get_mut(&target).unwrap().stats.current_health = 10;
            core.units.get_mut(&other).unwrap().stats.current_health = 20;
            start(&mut core, actor);
            core.apply_hp_delta_and_record(None, target, -100, 50, HpChangeReason::Command);
            let event = core.event_queue.pop().unwrap();
            core.process_event(event, 50).unwrap();
            assert_eq!(core.units[&target].stats.current_health, 0);
            assert_eq!(core.units[&other].stats.current_health, 20);
            assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
        }

        #[test]
        fn basic_heal_area_uses_release_target_tile_and_excludes_enemies() {
            let mut core = test_core();
            let actor = healer(&mut core);
            let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            let ally = add_player_target(&mut core, 11, WorldVec2::new(5.0, 1.0));
            let old = add_player_target(&mut core, 12, WorldVec2::new(0.0, 1.0));
            let enemy = add_enemy(&mut core, 13, WorldVec2::new(4.0, 1.0), 1.0);
            for id in [target, ally, old, enemy] {
                core.units.get_mut(&id).unwrap().stats.current_health = 50;
            }
            core.units.get_mut(&target).unwrap().stats.current_health = 10;
            core.units.get_mut(&actor).unwrap().basic_attack.action = BasicAttackAction::Heal {
                flat: 7,
                spell_power_percent: 0,
                footprint: Some(TileRangePattern {
                    include_anchor_tile: true,
                    rows: vec!["X@X".into()],
                }),
            };
            start(&mut core, actor);
            core.units.get_mut(&target).unwrap().body.position = WorldVec2::new(4.0, 1.0);
            let event = core.event_queue.pop().unwrap();
            core.process_event(event, 50).unwrap();
            assert_eq!(core.units[&target].stats.current_health, 17);
            assert_eq!(core.units[&ally].stats.current_health, 57);
            assert_eq!(core.units[&old].stats.current_health, 50);
            assert_eq!(core.units[&enemy].stats.current_health, 50);
        }

        #[test]
        fn basic_heal_epoch_cancel_and_transaction_failure_preserve_hp() {
            let mut core = test_core();
            let actor = healer(&mut core);
            let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            core.units.get_mut(&target).unwrap().stats.current_health = 50;
            let before = core.event_log.entries_after_seq(0);
            let result: Result<(), crate::game::behavior::GameError> =
                core.runtime_transaction(|candidate| {
                    assert!(candidate.resolve_basic_heal(actor, target, 0));
                    Err(crate::game::behavior::GameError::InvalidStaticData(
                        "injected after heal".into(),
                    ))
                });
            assert!(result.is_err());
            assert_eq!(core.units[&target].stats.current_health, 50);
            assert_eq!(
                serde_json::to_value(core.event_log.entries_after_seq(0)).unwrap(),
                serde_json::to_value(before).unwrap()
            );
            start(&mut core, actor);
            core.units.get_mut(&actor).unwrap().basic_attack_epoch += 1;
            let event = core.event_queue.pop().unwrap();
            core.process_event(event, 50).unwrap();
            assert_eq!(core.units[&target].stats.current_health, 50);
            assert_eq!(core.runtime_fact_view().active_actions().count(), 0);
        }

        #[test]
        fn basic_heal_overlay_rejects_damage_delivery_and_consumes_instant_once() {
            use crate::game::ability::sustained::SustainedEffectDef;
            let mut core = test_core();
            let actor = healer(&mut core);
            let target = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            core.units.get_mut(&target).unwrap().stats.current_health = 50;
            let projectile: SustainedEffectDef = ron::de::from_str(
                "BasicAttackOverlay(duration_ms: 1000, next_attack_only: true, profile: (range_policy: whole_field_valid_tiles, defense_tile_range: None, delivery: Projectile(speed_units_per_ms: 1000)))",
            ).unwrap();
            assert!(projectile.validate().is_ok());
            assert!(core.begin_sustained(actor, actor, projectile, 0).is_err());
            assert!(core.sustained_effect_snapshot().unwrap().active.is_empty());
            let original = serde_json::to_value(&core.units[&actor].basic_attack).unwrap();
            let instant: SustainedEffectDef = ron::de::from_str(
                "BasicAttackOverlay(duration_ms: 1000, next_attack_only: true, profile: (range_policy: pattern, defense_tile_range: Some((include_anchor_tile: true, rows: [\"X@X\"])), delivery: Instant))",
            ).unwrap();
            core.begin_sustained(actor, actor, instant, 0).unwrap();
            let mut clone = crate::authoring::clone_shape::controlled_clone(&core).unwrap();
            assert!(clone.resolve_basic_heal(actor, target, 50));
            assert_eq!(clone.units[&target].stats.current_health, 70);
            assert!(clone.sustained_effect_snapshot().unwrap().active.is_empty());
            assert_eq!(
                serde_json::to_value(&clone.units[&actor].basic_attack).unwrap(),
                original
            );
            assert_eq!(core.sustained_effect_snapshot().unwrap().active.len(), 1);
            assert_eq!(core.units[&target].stats.current_health, 50);
            assert_eq!(clone.runtime_fact_summary().damage_hp_lost, 0);
        }

        #[test]
        fn basic_heal_definition_rejects_damage_delivery_and_unknown_fields() {
            let mut profile = BasicAttackDef::default();
            profile.action = BasicAttackAction::Heal {
                flat: 10,
                spell_power_percent: 50,
                footprint: None,
            };
            assert!(profile.try_validate_runtime_contract().is_ok());
            profile.delivery = DeliveryDef::Projectile {
                speed_units_per_ms: 1000,
                hit_policy: Default::default(),
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            };
            assert_eq!(
                profile
                    .try_validate_runtime_contract()
                    .unwrap_err()
                    .field_suffix,
                "action"
            );
            assert!(ron::de::from_str::<BasicAttackAction>(
                "Heal(flat: 1, spell_power_percent: 0, footprint: None, damage: 3)"
            )
            .is_err());
        }

        #[test]
        fn basic_heal_resolve_precedes_rest_but_follows_damage() {
            let heal = BattleEvent::BasicHealResolve {
                time_ms: 50,
                attacker_instance_id: Uuid::from_u128(1).into(),
                target_instance_id: Uuid::from_u128(2).into(),
                attack_epoch: 0,
                kind: AttackKind::Auto,
                cause: BattleEventCause::default(),
            };
            let rest = BattleEvent::RestTick {
                time_ms: 50,
                unit_instance_id: Uuid::from_u128(2).into(),
                rest_epoch: 0,
            };
            assert!(heal > rest);
            // Skill damage release precedes basic recovery in the timestamp heap.
            let damage = BattleEvent::BuffTick {
                time_ms: 50,
                caster_instance_id: Uuid::from_u128(1).into(),
                target_instance_id: Uuid::from_u128(2).into(),
                buff_id: crate::game::battle::buffs::BuffId::from_name("test"),
                cause: BattleEventCause::default(),
            };
            assert!(damage > heal);
        }
        #[test]
        fn u06_ally_skill_selector_uses_allies_and_latest_heal_usefulness() {
            use crate::game::ability::{SkillDef, SkillTargetReach};
            use crate::game::battle::{
                core::skill_runtime::cast::SkillTargetUsefulness, event_log::SkillCastTarget,
            };
            let skill: SkillDef = ron::de::from_str(r#"(
                id:"u06_ally_heal", name:"Heal", use_lifetime:Action,
                cast_targeting:Explicit(selection:automatic_unit, target:AllySingle, reach:global, air_capable:false),
                steps:[(id:"heal",target:AllySingle,delivery:Instant,effects:[(id:"hp",op:Heal(amount:30))])],
            )"#).unwrap();
            let mut core = test_core();
            let actor = healer(&mut core);
            let ally = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            let enemy = add_enemy(&mut core, 11, WorldVec2::new(1.0, 1.0), 1.0);
            core.units.get_mut(&enemy).unwrap().stats.current_health = 1;
            assert_eq!(
                core.choose_ally_skill_target(
                    actor,
                    SkillTargetReach::Global,
                    None,
                    false,
                    SkillTargetUsefulness::Cast {
                        time_ms: 0,
                        skill: &skill
                    }
                ),
                None
            );
            core.units.get_mut(&ally).unwrap().stats.current_health = 40;
            assert_eq!(
                core.choose_ally_skill_target(
                    actor,
                    SkillTargetReach::Global,
                    None,
                    false,
                    SkillTargetUsefulness::Cast {
                        time_ms: 0,
                        skill: &skill
                    }
                ),
                Some(ally)
            );
            assert!(core.admitted_skill_target_is_valid(
                actor,
                &skill,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: ally
                })
            ));
            assert!(!core.admitted_skill_target_is_valid(
                actor,
                &skill,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: enemy
                })
            ));
            core.apply_hp_delta_and_record(None, ally, -100, 10, HpChangeReason::Command);
            assert!(!core.admitted_skill_target_is_valid(
                actor,
                &skill,
                Some(SkillCastTarget::Unit {
                    unit_instance_id: ally
                })
            ));
            assert_eq!(
                core.resolve_skill_step_context(
                    10,
                    1,
                    0,
                    actor,
                    &skill,
                    &skill.steps[0],
                    Some(SkillCastTarget::Unit {
                        unit_instance_id: ally
                    })
                ),
                None
            );
        }

        #[test]
        fn u06_ally_skill_selector_respects_authored_range_and_air_capability() {
            use crate::game::ability::{SkillDef, SkillTargetReach};
            use crate::game::battle::core::skill_runtime::cast::SkillTargetUsefulness;
            let skill: SkillDef = ron::de::from_str(r#"(
                id:"u06_ally_heal",use_lifetime:Action,
                cast_targeting:Explicit(selection:player_unit,target:AllySingle,reach:pattern,
                    target_range:Some((include_anchor_tile:true,rows:["@X"])),air_capable:false),
                steps:[(id:"heal",target:AllySingle,delivery:Instant,effects:[(id:"hp",op:Heal(amount:30))])],
            )"#).unwrap();
            let mut core = test_core();
            let actor = healer(&mut core);
            let near = add_player_target(&mut core, 10, WorldVec2::new(1.0, 1.0));
            let far = add_player_target(&mut core, 11, WorldVec2::new(3.0, 1.0));
            core.units.get_mut(&near).unwrap().stats.current_health = 50;
            core.units.get_mut(&far).unwrap().stats.current_health = 1;
            let (_, _, range, _) = skill.cast_target_definition().unwrap();
            assert_eq!(
                core.choose_ally_skill_target(
                    actor,
                    SkillTargetReach::Pattern,
                    range,
                    false,
                    SkillTargetUsefulness::Cast {
                        time_ms: 0,
                        skill: &skill
                    }
                ),
                Some(near)
            );
            core.units.get_mut(&near).unwrap().mobility_kind = MobilityKind::Airborne;
            assert_eq!(
                core.choose_ally_skill_target(
                    actor,
                    SkillTargetReach::Pattern,
                    range,
                    false,
                    SkillTargetUsefulness::Cast {
                        time_ms: 0,
                        skill: &skill
                    }
                ),
                None
            );
            assert_eq!(
                core.choose_ally_skill_target(
                    actor,
                    SkillTargetReach::Pattern,
                    range,
                    true,
                    SkillTargetUsefulness::Cast {
                        time_ms: 0,
                        skill: &skill
                    }
                ),
                Some(near)
            );
        }
    }
}
