mod common;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::{
    DeliveryDef, SkillAreaAnchorSource, SkillAreaDeliveryDef, SkillAreaShapeDef,
    SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillHitTargetFilter, SkillKind,
    SkillPresentationDef, SkillProjectileCollisionDef, SkillStepDef, SkillTarget,
    StepTargetingMode, UnitTargetRule,
};
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::damage::DamageType;
use game_core::game::battle::ids::UnitInstanceId;
use game_core::game::battle::placement::{PlacementBoard, PlacementSlotId};
use game_core::game::battle::timeline::{HpChangeReason, TimelineEvent};
use game_core::game::battle::types::{OwnedUnit, PlayerDeckInfo};
use game_core::game::data::{
    abnormality_data::{AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, MovementDef},
    artifact_data::ArtifactDatabase,
    bonus_data::BonusDatabase,
    equipment_data::EquipmentDatabase,
    pve_data::PveEncounterDatabase,
    random_event_data::RandomEventDatabase,
    shop_data::ShopDatabase,
    skill_data::SkillDatabase,
    GameDataBase,
};
use game_core::game::enums::Side;
use game_core::game::enums::{RiskLevel, Tier};
use game_core::game::growth::GrowthStack;
use uuid::Uuid;

const BASIC_ATTACK_PROJECTILE_SPEED_UNITS_PER_MS: u32 = 6_000;
const SHOWCASE_MELEE_MOVE_SPEED_UNITS_PER_MS: u32 = 1_700;
const SHOWCASE_RANGED_MOVE_SPEED_UNITS_PER_MS: u32 = 1_500;

fn unit(
    id: &str,
    uuid: Uuid,
    skill_id: Option<&str>,
    max_health: u32,
    attack: u32,
    range_units: f32,
    interval_ms: u64,
    delivery: DeliveryDef,
) -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: id.to_string(),
        uuid,
        name: id.to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health,
        attack,
        defense: 35,
        magic_resist: 35,
        movement: MovementDef {
            speed_units_per_ms: if range_units > 0.5 {
                SHOWCASE_RANGED_MOVE_SPEED_UNITS_PER_MS
            } else {
                SHOWCASE_MELEE_MOVE_SPEED_UNITS_PER_MS
            },
        },
        basic_attack: BasicAttackDef {
            range_units,
            interval_ms,
            windup_ms: 50,
            delivery,
        },
        resonance: game_core::game::data::abnormality_data::ResonanceDef {
            start: 0,
            max: 10,
            gain_lock_ms: 0,
        },
        skill_id: skill_id.map(str::to_string),
    }
}

fn deck(units: Vec<(Uuid, Uuid, Position)>) -> PlayerDeckInfo {
    let mut positions = HashMap::new();
    let mut owned_units = Vec::new();
    for (owned_uuid, base_uuid, position) in units {
        positions.insert(owned_uuid, position);
        owned_units.push(OwnedUnit {
            owned_uuid,
            base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        });
    }

    PlayerDeckInfo {
        units: owned_units,
        artifacts: vec![],
        positions,
    }
}

fn game_data(abnormalities: Vec<AbnormalityMetadata>, skills: Vec<SkillDef>) -> Arc<GameDataBase> {
    Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(abnormalities)),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(skills)),
            event_pools: common::empty_event_pools(),
        },
    ))
}

fn homing_projectile_skill() -> SkillDef {
    SkillDef {
        id: "unity_homing_bolt".to_string(),
        name: "unity_homing_bolt".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "homing_bolt".to_string(),
            delay_ms: 0,
            range_units: 6.0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 7_000,
                collision: SkillProjectileCollisionDef::default(),
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 18,
                damage_type: DamageType::Magic,
            }],
            presentation: SkillPresentationDef {
                cast_state: Some("Cast".to_string()),
                projectile_vfx_id: Some("unity_homing_bolt_projectile".to_string()),
                impact_vfx_id: Some("unity_homing_bolt_impact".to_string()),
                target_anchor: Some("Center".to_string()),
            },
        }],
    }
}

fn fixed_projectile_and_area_skill() -> SkillDef {
    SkillDef {
        id: "unity_line_and_cone".to_string(),
        name: "unity_line_and_cone".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "fixed_shot".to_string(),
                delay_ms: 0,
                range_units: 6.0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 6_000,
                    collision: SkillProjectileCollisionDef {
                        radius_units: 250_000,
                        hit_targets: SkillHitTargetFilter::Enemies,
                        piercing: true,
                        despawn_on_hit: Some(false),
                        max_hits: Some(2),
                    },
                },
                effects: vec![SkillEffectDef::Damage {
                    amount: 12,
                    damage_type: DamageType::Physical,
                }],
                presentation: SkillPresentationDef {
                    cast_state: Some("Shoot".to_string()),
                    projectile_vfx_id: Some("unity_fixed_shot_projectile".to_string()),
                    impact_vfx_id: Some("unity_fixed_shot_impact".to_string()),
                    target_anchor: None,
                },
            },
            SkillStepDef {
                id: "wedge_followup".to_string(),
                delay_ms: 50,
                range_units: 6.0,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Area {
                    area: SkillAreaDeliveryDef {
                        shape: SkillAreaShapeDef::Cone {
                            angle_degrees: 70,
                            length_units: 2_500_000,
                        },
                        anchor: SkillAreaAnchorSource::Caster,
                        hit_targets: SkillHitTargetFilter::Enemies,
                        include_caster: false,
                        tick_policy: game_core::game::ability::SkillAreaTickPolicy::EveryTick,
                        duration_ms: 0,
                        tick_interval_ms: None,
                    },
                },
                effects: vec![SkillEffectDef::Damage {
                    amount: 8,
                    damage_type: DamageType::Magic,
                }],
                presentation: SkillPresentationDef {
                    cast_state: Some("Cone".to_string()),
                    projectile_vfx_id: None,
                    impact_vfx_id: Some("unity_wedge_impact".to_string()),
                    target_anchor: None,
                },
            },
        ],
    }
}

fn assert_no_overlapping_movement_segments(timeline: &game_core::game::battle::timeline::Timeline) {
    let mut movement_segment_ends = HashMap::new();
    for entry in &timeline.entries {
        if let TimelineEvent::MovementSegmentStarted {
            unit_instance_id,
            started_at_ms,
            ends_at_ms,
            ..
        } = entry.event
        {
            if let Some(previous_end_ms) =
                movement_segment_ends.insert(unit_instance_id, ends_at_ms)
            {
                assert!(
                    started_at_ms >= previous_end_ms,
                    "movement segments for {unit_instance_id} must not overlap: previous_end_ms={previous_end_ms}, started_at_ms={started_at_ms}"
                );
            }
        }
    }
}

#[derive(Debug, Default)]
struct MovementQualityMetrics {
    movement_segment_count: usize,
    movement_stop_count: usize,
    max_segment_speed_units_per_sec: f32,
    very_short_segment_count: usize,
    idle_candidate_count: usize,
    spawned_unit_count: usize,
}

fn movement_quality_metrics(
    timeline: &game_core::game::battle::timeline::Timeline,
) -> MovementQualityMetrics {
    let mut metrics = MovementQualityMetrics::default();
    let mut spawned_units = HashSet::new();
    let mut participating_units = HashSet::new();

    for entry in &timeline.entries {
        match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id, ..
            } => {
                spawned_units.insert(*unit_instance_id);
            }
            TimelineEvent::MovementSegmentStarted {
                unit_instance_id,
                start,
                target,
                started_at_ms,
                ends_at_ms,
                ..
            } => {
                participating_units.insert(*unit_instance_id);
                metrics.movement_segment_count += 1;

                let duration_ms = ends_at_ms.saturating_sub(*started_at_ms);
                if duration_ms < 34 {
                    metrics.very_short_segment_count += 1;
                }
                if duration_ms > 0 {
                    let distance = start.to_world().distance(target.to_world());
                    let speed = distance / (duration_ms as f32 / 1_000.0);
                    metrics.max_segment_speed_units_per_sec =
                        metrics.max_segment_speed_units_per_sec.max(speed);
                }
            }
            TimelineEvent::MovementStopped {
                unit_instance_id, ..
            } => {
                participating_units.insert(*unit_instance_id);
                metrics.movement_stop_count += 1;
            }
            TimelineEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                ..
            } => {
                participating_units.insert(*attacker_instance_id);
                participating_units.insert(*target_instance_id);
            }
            TimelineEvent::BasicAttackProjectileLaunched {
                attacker_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::BasicAttackProjectileImpacted {
                attacker_instance_id,
                target_instance_id,
                ..
            } => {
                participating_units.insert(*attacker_instance_id);
                participating_units.insert(*target_instance_id);
            }
            TimelineEvent::SkillProjectileLaunched {
                caster_instance_id,
                target,
                ..
            } => {
                participating_units.insert(*caster_instance_id);
                insert_skill_target_units(target, &mut participating_units);
            }
            TimelineEvent::SkillProjectileImpacted {
                caster_instance_id,
                first_hit_unit_id,
                ..
            } => {
                participating_units.insert(*caster_instance_id);
                if let Some(unit_id) = first_hit_unit_id {
                    participating_units.insert(*unit_id);
                }
            }
            TimelineEvent::SkillAreaDeclared {
                caster_instance_id,
                target,
                ..
            } => {
                participating_units.insert(*caster_instance_id);
                insert_skill_target_units(target, &mut participating_units);
            }
            TimelineEvent::AbilityCast {
                caster_instance_id,
                target_instance_id,
                ..
            }
            | TimelineEvent::AbilityStepTriggered {
                caster_instance_id,
                target_instance_id,
                ..
            } => {
                participating_units.insert(*caster_instance_id);
                if let Some(unit_id) = target_instance_id {
                    participating_units.insert(*unit_id);
                }
            }
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                ..
            } => {
                if let Some(unit_id) = source_instance_id {
                    participating_units.insert(*unit_id);
                }
                participating_units.insert(*target_instance_id);
            }
            TimelineEvent::UnitDied {
                unit_instance_id, ..
            } => {
                participating_units.insert(*unit_instance_id);
            }
            _ => {}
        }
    }

    metrics.spawned_unit_count = spawned_units.len();
    metrics.idle_candidate_count = spawned_units
        .difference(&participating_units)
        .copied()
        .count();
    metrics
}

fn insert_skill_target_units(
    target: &Option<game_core::game::battle::timeline::SkillCastTarget>,
    participating_units: &mut HashSet<UnitInstanceId>,
) {
    match target {
        Some(game_core::game::battle::timeline::SkillCastTarget::Unit { unit_instance_id }) => {
            participating_units.insert(*unit_instance_id);
        }
        Some(game_core::game::battle::timeline::SkillCastTarget::Tile { .. }) | None => {}
    }
}

fn print_movement_quality(label: &str, metrics: &MovementQualityMetrics) {
    println!(
        "{label} movement quality: spawned={}, segments={}, stops={}, max_speed={:.3}, very_short_segments={}, idle_candidates={}",
        metrics.spawned_unit_count,
        metrics.movement_segment_count,
        metrics.movement_stop_count,
        metrics.max_segment_speed_units_per_sec,
        metrics.very_short_segment_count,
        metrics.idle_candidate_count
    );
}

#[test]
fn battle_start_maps_deck_position_to_tft_like_placement_slot_center() {
    let player_base = Uuid::from_u128(0xA500_0001);
    let opponent_base = Uuid::from_u128(0xB500_0001);
    let player_owned = Uuid::from_u128(0xC500_0001);
    let opponent_owned = Uuid::from_u128(0xD500_0001);
    let player_slot = Position::new(0, 1);

    let game_data = game_data(
        vec![
            unit(
                "placement_player",
                player_base,
                None,
                100,
                10,
                0.5,
                1_000,
                DeliveryDef::Instant,
            ),
            unit(
                "placement_opponent",
                opponent_base,
                None,
                100,
                10,
                0.5,
                1_000,
                DeliveryDef::Instant,
            ),
        ],
        vec![],
    );
    let player = deck(vec![(player_owned, player_base, player_slot)]);
    let opponent = deck(vec![(opponent_owned, opponent_base, Position::new(0, 6))]);
    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 5_500);
    let expected_spawn = PlacementBoard::new(common::BOARD_SIZE.0, common::BOARD_SIZE.1)
        .world_center(PlacementSlotId::from(player_slot));
    let mut world = World::new();

    battle
        .run_battle_with_setup(&mut world, |core| {
            let actual_spawn = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Player)
                .expect("player unit exists")
                .world_position();
            assert!(
                actual_spawn.distance(expected_spawn) <= 0.0001,
                "deck placement slot should spawn at TFT-like continuous center: actual={actual_spawn:?} expected={expected_spawn:?}"
            );
        })
        .expect("placement slot smoke battle runs");
}

#[test]
fn unity_contract_6v6_melee_rapier_showcase_exports_timeline() {
    let player_units = [
        Uuid::from_u128(0xA600_0001),
        Uuid::from_u128(0xA600_0002),
        Uuid::from_u128(0xA600_0003),
        Uuid::from_u128(0xA600_0004),
        Uuid::from_u128(0xA600_0005),
        Uuid::from_u128(0xA600_0006),
    ];
    let opponent_units = [
        Uuid::from_u128(0xB600_0001),
        Uuid::from_u128(0xB600_0002),
        Uuid::from_u128(0xB600_0003),
        Uuid::from_u128(0xB600_0004),
        Uuid::from_u128(0xB600_0005),
        Uuid::from_u128(0xB600_0006),
    ];

    let mut abnormalities = Vec::new();
    for (i, uuid) in player_units.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_6v6_player_melee_{i}"),
            *uuid,
            None,
            760 + (i as u32 * 20),
            50 + (i as u32 * 2),
            0.5,
            1750 + (i as u64 * 35),
            DeliveryDef::Instant,
        ));
    }
    for (i, uuid) in opponent_units.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_6v6_opponent_melee_{i}"),
            *uuid,
            None,
            740 + (i as u32 * 20),
            49 + (i as u32 * 2),
            0.5,
            1800 + (i as u64 * 35),
            DeliveryDef::Instant,
        ));
    }

    let player = deck(vec![
        (
            Uuid::from_u128(0xC600_0001),
            player_units[0],
            Position::new(0, 6),
        ),
        (
            Uuid::from_u128(0xC600_0002),
            player_units[1],
            Position::new(2, 6),
        ),
        (
            Uuid::from_u128(0xC600_0003),
            player_units[2],
            Position::new(4, 6),
        ),
        (
            Uuid::from_u128(0xC600_0004),
            player_units[3],
            Position::new(6, 6),
        ),
        (
            Uuid::from_u128(0xC600_0005),
            player_units[4],
            Position::new(1, 7),
        ),
        (
            Uuid::from_u128(0xC600_0006),
            player_units[5],
            Position::new(5, 7),
        ),
    ]);
    let opponent = deck(vec![
        (
            Uuid::from_u128(0xD600_0001),
            opponent_units[0],
            Position::new(0, 1),
        ),
        (
            Uuid::from_u128(0xD600_0002),
            opponent_units[1],
            Position::new(2, 1),
        ),
        (
            Uuid::from_u128(0xD600_0003),
            opponent_units[2],
            Position::new(4, 1),
        ),
        (
            Uuid::from_u128(0xD600_0004),
            opponent_units[3],
            Position::new(6, 1),
        ),
        (
            Uuid::from_u128(0xD600_0005),
            opponent_units[4],
            Position::new(1, 0),
        ),
        (
            Uuid::from_u128(0xD600_0006),
            opponent_units[5],
            Position::new(5, 0),
        ),
    ]);

    let game_data = game_data(abnormalities, vec![]);
    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 6_600);
    let mut world = World::new();
    let result = battle
        .run_battle_with_setup(&mut world, |core| {
            core.use_rapier_continuous_movement_backend();
        })
        .expect("6v6 melee rapier showcase battle runs");

    let timeline = &result.timeline;
    let path = common::write_timeline_export("unity_contract_6v6_melee_rapier_showcase", timeline);
    println!("wrote timeline: {}", path.display());

    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. })),
        "6v6 melee showcase must include continuous movement"
    );
    assert!(
        timeline.entries.iter().any(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::BasicAttack,
                    ..
                }
            )
        }),
        "6v6 melee showcase must include basic attack damage"
    );
    assert!(
        !timeline.entries.iter().any(|entry| matches!(
            entry.event,
            TimelineEvent::BasicAttackProjectileLaunched { .. }
        )),
        "6v6 melee showcase should use instant melee attacks only"
    );
    assert_no_overlapping_movement_segments(timeline);

    let movement_quality = movement_quality_metrics(timeline);
    print_movement_quality("6v6 melee", &movement_quality);
    assert_eq!(
        movement_quality.idle_candidate_count, 0,
        "6v6 melee showcase should not leave spawned units without movement/combat participation"
    );
    assert!(
        movement_quality.movement_segment_count >= 6,
        "6v6 melee showcase should produce enough movement segments to exercise Unity interpolation: {movement_quality:?}"
    );
    assert!(
        movement_quality.max_segment_speed_units_per_sec <= 2.0,
        "movement segment speed should stay near configured unit stats, got {movement_quality:?}"
    );
    assert!(
        movement_quality.very_short_segment_count * 10 <= movement_quality.movement_segment_count,
        "very short movement segments should not dominate Unity interpolation: {movement_quality:?}"
    );
}

#[test]
fn unity_contract_7v7_rapier_showcase_exports_timeline() {
    let p_homing = Uuid::from_u128(0xA700_0001);
    let p_fixed = Uuid::from_u128(0xA700_0002);
    let p_melee = [
        Uuid::from_u128(0xA700_0011),
        Uuid::from_u128(0xA700_0012),
        Uuid::from_u128(0xA700_0013),
    ];
    let p_ranged = [Uuid::from_u128(0xA700_0021), Uuid::from_u128(0xA700_0022)];
    let o_homing = Uuid::from_u128(0xB700_0001);
    let o_fixed = Uuid::from_u128(0xB700_0002);
    let o_melee = [
        Uuid::from_u128(0xB700_0011),
        Uuid::from_u128(0xB700_0012),
        Uuid::from_u128(0xB700_0013),
    ];
    let o_ranged = [Uuid::from_u128(0xB700_0021), Uuid::from_u128(0xB700_0022)];

    let mut abnormalities = vec![
        unit(
            "unity_player_homing",
            p_homing,
            Some("unity_homing_bolt"),
            760,
            52,
            7.0,
            1900,
            DeliveryDef::Projectile {
                speed_units_per_ms: BASIC_ATTACK_PROJECTILE_SPEED_UNITS_PER_MS,
                collision: Default::default(),
            },
        ),
        unit(
            "unity_player_fixed_area",
            p_fixed,
            Some("unity_line_and_cone"),
            850,
            60,
            0.5,
            1850,
            DeliveryDef::Instant,
        ),
        unit(
            "unity_opponent_homing",
            o_homing,
            Some("unity_homing_bolt"),
            740,
            50,
            7.0,
            1950,
            DeliveryDef::Projectile {
                speed_units_per_ms: BASIC_ATTACK_PROJECTILE_SPEED_UNITS_PER_MS,
                collision: Default::default(),
            },
        ),
        unit(
            "unity_opponent_fixed_area",
            o_fixed,
            Some("unity_line_and_cone"),
            840,
            58,
            0.5,
            1900,
            DeliveryDef::Instant,
        ),
    ];
    for (i, uuid) in p_melee.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_player_melee_{i}"),
            *uuid,
            None,
            820,
            55,
            0.5,
            1800,
            DeliveryDef::Instant,
        ));
    }
    for (i, uuid) in p_ranged.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_player_ranged_{i}"),
            *uuid,
            None,
            650,
            50,
            7.0,
            1900,
            DeliveryDef::Projectile {
                speed_units_per_ms: BASIC_ATTACK_PROJECTILE_SPEED_UNITS_PER_MS,
                collision: Default::default(),
            },
        ));
    }
    for (i, uuid) in o_melee.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_opponent_melee_{i}"),
            *uuid,
            None,
            800,
            54,
            0.5,
            1850,
            DeliveryDef::Instant,
        ));
    }
    for (i, uuid) in o_ranged.iter().enumerate() {
        abnormalities.push(unit(
            &format!("unity_opponent_ranged_{i}"),
            *uuid,
            None,
            640,
            49,
            7.0,
            1950,
            DeliveryDef::Projectile {
                speed_units_per_ms: BASIC_ATTACK_PROJECTILE_SPEED_UNITS_PER_MS,
                collision: Default::default(),
            },
        ));
    }

    let game_data = game_data(
        abnormalities,
        vec![homing_projectile_skill(), fixed_projectile_and_area_skill()],
    );
    let player = deck(vec![
        (Uuid::from_u128(0xC700_0001), p_homing, Position::new(1, 7)),
        (Uuid::from_u128(0xC700_0002), p_fixed, Position::new(3, 7)),
        (
            Uuid::from_u128(0xC700_0011),
            p_melee[0],
            Position::new(0, 6),
        ),
        (
            Uuid::from_u128(0xC700_0012),
            p_melee[1],
            Position::new(3, 6),
        ),
        (
            Uuid::from_u128(0xC700_0013),
            p_melee[2],
            Position::new(6, 6),
        ),
        (
            Uuid::from_u128(0xC700_0021),
            p_ranged[0],
            Position::new(2, 7),
        ),
        (
            Uuid::from_u128(0xC700_0022),
            p_ranged[1],
            Position::new(5, 7),
        ),
    ]);
    let opponent = deck(vec![
        (Uuid::from_u128(0xD700_0001), o_homing, Position::new(5, 0)),
        (Uuid::from_u128(0xD700_0002), o_fixed, Position::new(3, 0)),
        (
            Uuid::from_u128(0xD700_0011),
            o_melee[0],
            Position::new(0, 1),
        ),
        (
            Uuid::from_u128(0xD700_0012),
            o_melee[1],
            Position::new(3, 1),
        ),
        (
            Uuid::from_u128(0xD700_0013),
            o_melee[2],
            Position::new(6, 1),
        ),
        (
            Uuid::from_u128(0xD700_0021),
            o_ranged[0],
            Position::new(1, 0),
        ),
        (
            Uuid::from_u128(0xD700_0022),
            o_ranged[1],
            Position::new(4, 0),
        ),
    ]);

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 7_700);
    let mut world = World::new();
    let result = battle
        .run_battle_with_setup(&mut world, |core| {
            core.use_rapier_continuous_movement_backend();
        })
        .expect("7v7 unity contract showcase battle runs");

    let timeline = &result.timeline;
    let path = common::write_timeline_export("unity_contract_7v7_rapier_showcase", timeline);
    println!("wrote timeline: {}", path.display());

    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::MovementSegmentStarted { .. })),
        "showcase must include continuous movement"
    );
    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::MovementStopped { .. })),
        "showcase must include movement stop correction"
    );

    assert_no_overlapping_movement_segments(timeline);
    let movement_quality = movement_quality_metrics(timeline);
    print_movement_quality("7v7 mixed", &movement_quality);
    assert!(
        movement_quality.idle_candidate_count <= 2,
        "7v7 mixed showcase should not leave most spawned units without movement/combat participation: {movement_quality:?}"
    );
    assert!(
        movement_quality.movement_segment_count >= 6,
        "7v7 mixed showcase should produce enough movement for Unity interpolation: {movement_quality:?}"
    );
    assert!(
        movement_quality.max_segment_speed_units_per_sec <= 2.0,
        "movement segment speed should stay near configured unit stats, got {movement_quality:?}"
    );
    assert!(
        movement_quality.very_short_segment_count * 5 <= movement_quality.movement_segment_count,
        "very short movement segments should stay bounded in mixed showcase: {movement_quality:?}"
    );

    assert!(
        timeline.entries.iter().any(|entry| matches!(
            entry.event,
            TimelineEvent::BasicAttackProjectileLaunched { .. }
        )),
        "showcase must include basic attack projectile launch"
    );
    let shortest_basic_projectile_lifetime_ms = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::BasicAttackProjectileLaunched {
                fired_at_ms,
                expected_impact_time_ms,
                ..
            } => Some(expected_impact_time_ms.saturating_sub(*fired_at_ms)),
            _ => None,
        })
        .min()
        .expect("showcase must include basic attack projectile launch");
    assert!(
        shortest_basic_projectile_lifetime_ms >= 100,
        "basic attack projectile should stay visible across multiple render frames; shortest lifetime was {shortest_basic_projectile_lifetime_ms}ms"
    );
    assert!(
        timeline.entries.iter().any(|entry| matches!(
            entry.event,
            TimelineEvent::BasicAttackProjectileImpacted { .. }
        )),
        "showcase must include basic attack projectile impact"
    );
    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::SkillProjectileLaunched { .. })),
        "showcase must include skill projectile launch"
    );
    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::SkillProjectileImpacted { .. })),
        "showcase must include skill projectile impact"
    );
    assert!(
        timeline
            .entries
            .iter()
            .any(|entry| matches!(entry.event, TimelineEvent::SkillAreaDeclared { .. })),
        "showcase must include skill area declaration"
    );
    assert!(
        timeline.entries.iter().any(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::BasicAttack,
                    ..
                }
            )
        }),
        "showcase must include basic attack damage"
    );
}
