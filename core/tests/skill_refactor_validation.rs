mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::{
    DeliveryDef, SkillAreaAnchorSource, SkillAreaDeliveryDef, SkillAreaShapeDef,
    SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillHitTargetFilter, SkillKind,
    SkillPresentationDef, SkillProjectileCollisionDef, SkillStepCondition, SkillStepDef,
    SkillStepRepeat, SkillTarget, SkillUnitReference, StepTargetingMode, UnitTargetRule,
};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::core::{
    movement::{ActionState, MovementState},
    BattleCore,
};
use game_core::game::battle::enums::BattleEvent;
use game_core::game::battle::timeline::{
    AttackKind, HpChangeReason, Timeline, TimelineCause, TimelineEvent, TimelineRootCause,
};
use game_core::game::battle::types::{OwnedUnit, PlayerDeckInfo};
use game_core::game::data::{
    abnormality_data::{
        AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef,
    },
    artifact_data::ArtifactDatabase,
    bonus_data::BonusDatabase,
    equipment_data::EquipmentDatabase,
    pve_data::PveEncounterDatabase,
    random_event_data::RandomEventDatabase,
    shop_data::ShopDatabase,
    skill_data::SkillDatabase,
    GameDataBase,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::stats::{StatId, StatModifier, StatModifierKind};
use uuid::Uuid;

fn deck(units: Vec<(Uuid, Uuid, Position)>) -> PlayerDeckInfo {
    let mut positions = HashMap::new();
    let mut owned_units = Vec::new();

    for (owned_uuid, base_uuid, pos) in units {
        positions.insert(owned_uuid, pos);
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

fn make_abnormality(
    id: &str,
    uuid: Uuid,
    skill_id: Option<&str>,
    attack: u32,
    max_health: u32,
    defense: u32,
    attack_interval_ms: u64,
    attack_range_tiles: u8,
    attack_delivery: DeliveryDef,
    resonance_max: u32,
) -> AbnormalityMetadata {
    AbnormalityMetadata {
        id: id.to_string(),
        uuid,
        name: id.to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health,
        attack,
        defense,
        movement: MovementDef {
            speed_units_per_ms: 3000,
        },
        basic_attack: BasicAttackDef {
            range_tiles: attack_range_tiles,
            interval_ms: attack_interval_ms,
            windup_ms: 0,
            delivery: attack_delivery,
        },
        resonance: ResonanceDef {
            start: 0,
            max: resonance_max,
            gain_lock_ms: 0,
        },
        skill_id: skill_id.map(str::to_string),
    }
}

fn minimal_game_data(
    abnormalities: Vec<AbnormalityMetadata>,
    skills: Vec<SkillDef>,
) -> Arc<GameDataBase> {
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

fn run_battle(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
) -> Timeline {
    run_battle_with_setup(game_data, player_units, opponent_units, |_| {})
}

fn run_battle_with_setup<F>(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
    setup: F,
) -> Timeline
where
    F: FnOnce(&mut BattleCore),
{
    let player = deck(player_units);
    let opponent = deck(opponent_units);

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 4242);
    let mut world = World::new();
    battle
        .run_battle_with_setup(&mut world, setup)
        .expect("battle runs")
        .timeline
}

fn run_battle_and_capture_core_with_setup<F>(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
    setup: F,
) -> (Timeline, BattleCore)
where
    F: FnOnce(&mut BattleCore),
{
    let player = deck(player_units);
    let opponent = deck(opponent_units);

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 4242);
    let mut world = World::new();
    battle
        .run_battle_with_setup(&mut world, setup)
        .expect("battle runs");
    (battle.timeline.clone(), battle)
}

fn parent_seq(cause: &TimelineCause) -> Option<u64> {
    match cause {
        TimelineCause::Parent { seq } => Some(*seq),
        TimelineCause::Root { .. } => None,
    }
}

fn set_linear_motion(
    core: &mut BattleCore,
    unit_id: Uuid,
    from: Position,
    from_x_units: i64,
    from_y_units: i64,
    target_x_units: i64,
    target_y_units: i64,
    started_at_ms: u64,
) {
    let unit_id = unit_id.into();
    let mut movement = MovementState::new_at(from, started_at_ms);
    movement.last_update_ms = started_at_ms;
    movement.step_start_x_units = from_x_units;
    movement.step_start_y_units = from_y_units;
    movement.target_x_units = target_x_units;
    movement.target_y_units = target_y_units;
    movement.step_started_at_ms = started_at_ms;
    movement.step_ends_at_ms = started_at_ms;

    let unit = core
        .units
        .get_mut(&unit_id)
        .expect("runtime unit should exist for motion patch");
    unit.pos_x_units = from_x_units;
    unit.pos_y_units = from_y_units;
    unit.action_state = ActionState::Moving(movement);
}

fn spawned_unit_id(timeline: &Timeline, base_uuid: Uuid, owner: Side) -> Uuid {
    timeline
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                base_uuid: actual_base_uuid,
                owner: actual_owner,
                ..
            } if actual_base_uuid == base_uuid && actual_owner == owner => {
                Some(unit_instance_id.into())
            }
            _ => None,
        })
        .expect("missing UnitSpawned")
}

fn find_first_ability_cast_seq(
    timeline: &Timeline,
    skill_id: &str,
    caster_instance_id: Uuid,
) -> (u64, u64) {
    let entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityCast {
                    skill_id: actual_skill_id,
                    caster_instance_id: actual_caster,
                    ..
                } if actual_skill_id == skill_id && *actual_caster == caster_instance_id.into()
            )
        })
        .expect("missing AbilityCast");

    (entry.seq, entry.time_ms)
}

fn step_entries_for_cast<'a>(
    timeline: &'a Timeline,
    ability_seq: u64,
) -> Vec<&'a game_core::game::battle::timeline::TimelineEntry> {
    timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.event, TimelineEvent::AbilityStepTriggered { .. })
                && parent_seq(&entry.cause) == Some(ability_seq)
        })
        .collect()
}

#[test]
fn enemy_radius_area_anchors_on_the_nearest_enemy_instead_of_the_caster_tile() {
    let caster_base_uuid = Uuid::from_u128(0xAA01);
    let enemy_a_base_uuid = Uuid::from_u128(0xAA02);
    let enemy_b_base_uuid = Uuid::from_u128(0xAA03);

    let skill = SkillDef {
        id: "remote_nova".to_string(),
        name: "remote_nova".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "remote_blast".to_string(),
            delay_ms: 0,
            range_tiles: 4,
            target: SkillTarget::Enemies {
                area: game_core::game::ability::SkillArea::RadiusChebyshev { radius_tiles: 1 },
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::Damage { amount: 30 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("remote_nova"),
                1,
                200,
                0,
                300,
                4,
                DeliveryDef::Instant,
                1,
            ),
            make_abnormality(
                "enemy_a",
                enemy_a_base_uuid,
                None,
                1,
                150,
                0,
                5_000,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy_b",
                enemy_b_base_uuid,
                None,
                1,
                150,
                0,
                5_000,
                1,
                DeliveryDef::Instant,
                10,
            ),
        ],
        vec![skill],
    );

    let caster_owned_id = Uuid::from_u128(0xAA11);
    let enemy_a_owned_id = Uuid::from_u128(0xAA12);
    let enemy_b_owned_id = Uuid::from_u128(0xAA13);
    let timeline = run_battle_with_setup(
        game_data,
        vec![(caster_owned_id, caster_base_uuid, Position::new(1, 1))],
        vec![
            (enemy_a_owned_id, enemy_a_base_uuid, Position::new(3, 1)),
            (enemy_b_owned_id, enemy_b_base_uuid, Position::new(4, 1)),
        ],
        |core| {
            let caster_instance_id = core
                .battlefield
                .occupant(Position::new(1, 1))
                .expect("inspect caster tile")
                .expect("caster should exist");
            core.enqueue_event(BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id,
                cause: TimelineCause::default(),
            });
        },
    );
    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_a_id = spawned_unit_id(&timeline, enemy_a_base_uuid, Side::Opponent);
    let enemy_b_id = spawned_unit_id(&timeline, enemy_b_base_uuid, Side::Opponent);

    let (ability_seq, _) = find_first_ability_cast_seq(&timeline, "remote_nova", caster_id);
    let step_seq = step_entries_for_cast(&timeline, ability_seq)
        .into_iter()
        .find_map(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityStepTriggered { step_id, .. } if step_id == "remote_blast"
            )
            .then_some(entry.seq)
        })
        .expect("missing remote_blast step");

    let damaged_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                target_instance_id,
                delta,
                ..
            } if parent_seq(&entry.cause) == Some(step_seq) && *delta == -30 => {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        damaged_targets,
        vec![enemy_a_id, enemy_b_id],
        "enemy-centered radius AoE should anchor on the nearest enemy cluster, not the caster tile"
    );
}

#[test]
fn delayed_area_reuses_the_original_cast_target_snapshot_after_that_target_dies() {
    let caster_base_uuid = Uuid::from_u128(0xAA21);
    let primary_base_uuid = Uuid::from_u128(0xAA22);
    let nearby_base_uuid = Uuid::from_u128(0xAA23);

    let skill = SkillDef {
        id: "delayed_corpse_burst".to_string(),
        name: "delayed_corpse_burst".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "kill_primary".to_string(),
                delay_ms: 0,
                range_tiles: 4,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 200 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "corpse_burst".to_string(),
                delay_ms: 20,
                range_tiles: 4,
                target: SkillTarget::Enemies {
                    area: game_core::game::ability::SkillArea::RadiusChebyshev { radius_tiles: 1 },
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::IfPreviousStepDealtDamage,
                repeat: Default::default(),
                delivery: DeliveryDef::Area {
                    area: SkillAreaDeliveryDef {
                        shape: SkillAreaShapeDef::Circle {
                            radius_units: 1_100_000,
                        },
                        anchor: SkillAreaAnchorSource::CastTarget,
                        hit_targets: SkillHitTargetFilter::Enemies,
                        include_caster: false,
                        tick_policy: game_core::game::ability::SkillAreaTickPolicy::EveryTick,
                        duration_ms: 0,
                        tick_interval_ms: None,
                    },
                },
                effects: vec![SkillEffectDef::Damage { amount: 25 }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("delayed_corpse_burst"),
                1,
                200,
                0,
                300,
                4,
                DeliveryDef::Instant,
                1,
            ),
            make_abnormality(
                "primary",
                primary_base_uuid,
                None,
                1,
                100,
                0,
                5_000,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "nearby",
                nearby_base_uuid,
                None,
                1,
                150,
                0,
                5_000,
                1,
                DeliveryDef::Instant,
                10,
            ),
        ],
        vec![skill],
    );

    let caster_owned_id = Uuid::from_u128(0xAA31);
    let primary_owned_id = Uuid::from_u128(0xAA32);
    let nearby_owned_id = Uuid::from_u128(0xAA33);
    let timeline = run_battle_with_setup(
        game_data,
        vec![(caster_owned_id, caster_base_uuid, Position::new(1, 1))],
        vec![
            (primary_owned_id, primary_base_uuid, Position::new(3, 1)),
            (nearby_owned_id, nearby_base_uuid, Position::new(4, 1)),
        ],
        |core| {
            let caster_instance_id = core
                .battlefield
                .occupant(Position::new(1, 1))
                .expect("inspect caster tile")
                .expect("caster should exist");
            core.enqueue_event(BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id,
                cause: TimelineCause::default(),
            });
        },
    );
    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let nearby_id = spawned_unit_id(&timeline, nearby_base_uuid, Side::Opponent);

    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "delayed_corpse_burst", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    let burst_step_seq = steps
        .into_iter()
        .find_map(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityStepTriggered { step_id, .. } if step_id == "corpse_burst"
            )
            .then_some(entry.seq)
        })
        .expect("missing corpse_burst step");

    let burst_hits: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                target_instance_id,
                delta,
                ..
            } if parent_seq(&entry.cause) == Some(burst_step_seq) && *delta == -25 => {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        burst_hits,
        vec![nearby_id],
        "delayed AoE should explode at the original cast target snapshot after the primary target dies"
    );
}

#[test]
fn mixed_target_skill_records_enemy_damage_then_self_buff() {
    let caster_base_uuid = Uuid::from_u128(0xAA11);
    let enemy_base_uuid = Uuid::from_u128(0xAA12);

    let skill = SkillDef {
        id: "enemy_then_self".to_string(),
        name: "enemy_then_self".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "enemy_burst".to_string(),
                delay_ms: 0,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 15 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "self_buff".to_string(),
                delay_ms: 50,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 5,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("enemy_then_self"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(1), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(2), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&timeline, "enemy_then_self", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "expected exactly 2 step entries for the first cast"
    );

    let first_step = steps[0];
    let second_step = steps[1];
    assert_eq!(first_step.time_ms, ability_time_ms);
    assert_eq!(second_step.time_ms, ability_time_ms + 50);

    assert!(timeline.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *target_instance_id == enemy_id.into()
                && *reason == HpChangeReason::Command
        ) && parent_seq(&entry.cause) == Some(first_step.seq)
    }));

    assert!(timeline.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::StatChanged {
                target_instance_id,
                modifier,
                ..
            } if *target_instance_id == caster_id.into()
                && modifier.stat == StatId::Attack
                && modifier.kind == StatModifierKind::Flat
                && modifier.value == 5
        ) && parent_seq(&entry.cause) == Some(second_step.seq)
    }));
}

#[test]
fn mixed_delivery_skill_delays_projectile_impact_beyond_followup_step() {
    let caster_base_uuid = Uuid::from_u128(0xBB11);
    let enemy_base_uuid = Uuid::from_u128(0xBB12);

    let skill = SkillDef {
        id: "projectile_then_self_buff".to_string(),
        name: "projectile_then_self_buff".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "projectile_hit".to_string(),
                delay_ms: 0,
                range_tiles: 2,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 500_000,
                    collision: Default::default(),
                },
                effects: vec![SkillEffectDef::Damage { amount: 20 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "self_buff".to_string(),
                delay_ms: 1,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 3,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("projectile_then_self_buff"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(11), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(12), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&timeline, "projectile_then_self_buff", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "expected exactly 2 step entries for the first cast"
    );

    let projectile_step = steps[0];
    let self_step = steps[1];
    assert_eq!(projectile_step.time_ms, ability_time_ms);
    assert_eq!(self_step.time_ms, ability_time_ms + 1);

    let self_buff_entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::StatChanged {
                    target_instance_id,
                    modifier,
                    ..
                } if *target_instance_id == caster_id.into()
                    && modifier.stat == StatId::Attack
                    && modifier.value == 3
            ) && parent_seq(&entry.cause) == Some(self_step.seq)
        })
        .expect("missing self buff StatChanged");

    let projectile_impact_entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && *reason == HpChangeReason::Command
            ) && parent_seq(&entry.cause) == Some(projectile_step.seq)
        })
        .expect("missing projectile-delivered HpChanged");

    assert!(
        projectile_impact_entry.time_ms > self_buff_entry.time_ms,
        "projectile impact should happen after the follow-up instant self step"
    );
}

#[test]
fn self_then_retargeted_enemy_skill_resolves_second_step_at_execution_time() {
    let caster_base_uuid = Uuid::from_u128(0xBC11);
    let enemy_base_uuid = Uuid::from_u128(0xBC12);

    let skill = SkillDef {
        id: "self_then_retarget".to_string(),
        name: "self_then_retarget".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "self_charge".to_string(),
                delay_ms: 0,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 4,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "retargeted_strike".to_string(),
                delay_ms: 10,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 25 }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("self_then_retarget"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(21), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(22), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&timeline, "self_then_retarget", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "expected exactly 2 step entries for the first cast"
    );

    let self_step = steps[0];
    let enemy_step = steps[1];
    assert_eq!(self_step.time_ms, ability_time_ms);
    assert_eq!(enemy_step.time_ms, ability_time_ms + 10);

    assert!(matches!(
        &self_step.event,
        TimelineEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == caster_id.into()
    ));

    assert!(matches!(
        &enemy_step.event,
        TimelineEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy_id.into()
    ));

    assert!(timeline.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *target_instance_id == enemy_id.into()
                && *reason == HpChangeReason::Command
        ) && parent_seq(&entry.cause) == Some(enemy_step.seq)
    }));
}

#[test]
fn conditional_followup_waits_for_projectile_damage_resolution() {
    let caster_base_uuid = Uuid::from_u128(0xBC21);
    let enemy_base_uuid = Uuid::from_u128(0xBC22);

    let skill = SkillDef {
        id: "conditional_projectile_followup".to_string(),
        name: "conditional_projectile_followup".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "delayed_shot".to_string(),
                delay_ms: 0,
                range_tiles: 3,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 250_000,
                    collision: Default::default(),
                },
                effects: vec![SkillEffectDef::Damage { amount: 20 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "heal_on_hit".to_string(),
                delay_ms: 1,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::IfPreviousStepDealtDamage,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 5,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("conditional_projectile_followup"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                0,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(21), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(22), enemy_base_uuid, Position::new(0, 2))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&timeline, "conditional_projectile_followup", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "follow-up step should be deferred, not dropped"
    );

    let projectile_step = steps[0];
    let followup_step = steps[1];

    let projectile_impact_entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && *reason == HpChangeReason::Command
            ) && parent_seq(&entry.cause) == Some(projectile_step.seq)
        })
        .expect("missing projectile hit");

    let followup_entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::StatChanged {
                    target_instance_id,
                    modifier,
                    ..
                } if *target_instance_id == caster_id.into()
                    && modifier.stat == StatId::Attack
                    && modifier.kind == StatModifierKind::Flat
                    && modifier.value == 5
            ) && parent_seq(&entry.cause) == Some(followup_step.seq)
        })
        .expect("missing deferred follow-up buff");

    assert!(
        followup_step.time_ms > ability_time_ms + 1,
        "follow-up should not execute at its original delay while projectile result is pending"
    );
    assert!(
        followup_step.time_ms >= projectile_impact_entry.time_ms,
        "follow-up must wait until projectile damage has actually resolved"
    );
    assert!(
        followup_entry.time_ms >= projectile_impact_entry.time_ms,
        "follow-up effect should happen after projectile damage resolution"
    );
}

#[test]
fn explicit_cast_targeting_separates_cast_context_from_step_execution_targets() {
    let caster_base_uuid = Uuid::from_u128(0xBC21);
    let enemy_base_uuid = Uuid::from_u128(0xBC22);

    let skill = SkillDef {
        id: "self_charge_then_locked_shot".to_string(),
        name: "self_charge_then_locked_shot".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::Explicit {
            range_tiles: 1,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
        },
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "self_charge".to_string(),
                delay_ms: 0,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 6,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "locked_shot".to_string(),
                delay_ms: 10,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 25 }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("self_charge_then_locked_shot"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(41), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(42), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);

    let autocast_start = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AutoCastStart {
                    caster_instance_id,
                    target: Some(game_core::game::battle::timeline::SkillCastTarget::Unit {
                        unit_instance_id
                    }),
                    ..
                } if *caster_instance_id == caster_id.into() && *unit_instance_id == enemy_id.into()
            )
        })
        .expect("missing AutoCastStart with explicit enemy cast target");

    let ability_cast = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityCast {
                    skill_id,
                    caster_instance_id,
                    target_instance_id: Some(target_instance_id),
                } if skill_id == "self_charge_then_locked_shot"
                    && *caster_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
            )
        })
        .expect("missing AbilityCast with explicit enemy cast target");

    assert_eq!(
        parent_seq(&ability_cast.cause),
        Some(autocast_start.seq),
        "AbilityCast should be parented to the explicit AutoCastStart"
    );

    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&timeline, "self_charge_then_locked_shot", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(steps.len(), 2, "expected exactly 2 executed steps");

    let self_step = steps[0];
    let enemy_step = steps[1];
    assert_eq!(self_step.time_ms, ability_time_ms);
    assert_eq!(enemy_step.time_ms, ability_time_ms + 10);

    assert!(matches!(
        &self_step.event,
        TimelineEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == caster_id.into()
    ));
    assert!(matches!(
        &enemy_step.event,
        TimelineEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy_id.into()
    ));
}

#[test]
fn ability_step_timeline_includes_presentation_metadata() {
    let caster_base_uuid = Uuid::from_u128(0xCA51);
    let enemy_base_uuid = Uuid::from_u128(0xCA52);

    let skill = SkillDef {
        id: "presentation_skill".to_string(),
        name: "presentation_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "judgement".to_string(),
            delay_ms: 0,
            range_tiles: 1,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                collision: Default::default(),
            },
            effects: vec![SkillEffectDef::Damage { amount: 10 }],
            presentation: SkillPresentationDef {
                cast_state: Some("Cast".to_string()),
                projectile_vfx_id: Some("white_night_judgement".to_string()),
                impact_vfx_id: Some("white_night_judgement_hit".to_string()),
                target_anchor: Some("Head".to_string()),
            },
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("presentation_skill"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(31), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(32), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let step_entry = timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityStepTriggered {
                    skill_id,
                    caster_instance_id,
                    presentation: Some(_),
                    ..
                } if skill_id == "presentation_skill" && *caster_instance_id == caster_id.into()
            )
        })
        .expect("missing AbilityStepTriggered with presentation metadata");

    match &step_entry.event {
        TimelineEvent::AbilityStepTriggered {
            presentation: Some(presentation),
            ..
        } => {
            assert_eq!(presentation.cast_state.as_deref(), Some("Cast"));
            assert_eq!(
                presentation.projectile_vfx_id.as_deref(),
                Some("white_night_judgement")
            );
            assert_eq!(
                presentation.impact_vfx_id.as_deref(),
                Some("white_night_judgement_hit")
            );
            assert_eq!(presentation.target_anchor.as_deref(), Some("Head"));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn untargeted_projectile_hits_first_blocker_before_cast_target() {
    let caster_base_uuid = Uuid::from_u128(0xCC11);
    let blocker_base_uuid = Uuid::from_u128(0xCC12);
    let backline_base_uuid = Uuid::from_u128(0xCC13);

    let skill = SkillDef {
        id: "untargeted_piercing_test".to_string(),
        name: "untargeted_piercing_test".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "skillshot".to_string(),
            delay_ms: 0,
            range_tiles: 3,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                collision: Default::default(),
            },
            effects: vec![SkillEffectDef::Damage { amount: 40 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("untargeted_piercing_test"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "blocker",
                blocker_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "backline",
                backline_base_uuid,
                None,
                1,
                50,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(41), caster_base_uuid, Position::new(0, 0))],
        vec![
            (Uuid::from_u128(42), blocker_base_uuid, Position::new(0, 1)),
            (Uuid::from_u128(43), backline_base_uuid, Position::new(0, 2)),
        ],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let blocker_id = spawned_unit_id(&timeline, blocker_base_uuid, Side::Opponent);
    let backline_id = spawned_unit_id(&timeline, backline_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "untargeted_piercing_test", caster_id);
    let step = step_entries_for_cast(&timeline, ability_seq)
        .into_iter()
        .next()
        .expect("missing skillshot step");

    let damage_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && parent_seq(&entry.cause) == Some(step.seq) =>
            {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert_eq!(damage_targets, vec![blocker_id]);
    assert!(
        !damage_targets.contains(&backline_id),
        "backline cast target should not be hit before the frontline blocker"
    );
}

#[test]
fn untargeted_projectile_miss_finalizes_step_and_cleans_up_damage_gated_followup() {
    let caster_base_uuid = Uuid::from_u128(0xCC21);
    let enemy_base_uuid = Uuid::from_u128(0xCC22);

    let skill = SkillDef {
        id: "untargeted_miss_then_check".to_string(),
        name: "untargeted_miss_then_check".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "missable_shot".to_string(),
                delay_ms: 0,
                range_tiles: 2,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 500_000,
                    collision: SkillProjectileCollisionDef {
                        hit_targets: SkillHitTargetFilter::Allies,
                        ..Default::default()
                    },
                },
                effects: vec![SkillEffectDef::Damage { amount: 20 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "followup_buff".to_string(),
                delay_ms: 1,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::IfPreviousStepDealtDamage,
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::ModifyStats {
                    modifier: StatModifier {
                        stat: StatId::Attack,
                        kind: StatModifierKind::Flat,
                        value: 7,
                    },
                }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("untargeted_miss_then_check"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let (timeline, battle) = run_battle_and_capture_core_with_setup(
        game_data,
        vec![(Uuid::from_u128(51), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(52), enemy_base_uuid, Position::new(0, 1))],
        |_| {},
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "untargeted_miss_then_check", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(steps.len(), 1);

    let projectile_step = steps[0];

    assert!(
        !timeline.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *reason == HpChangeReason::Command
            ) && parent_seq(&entry.cause) == Some(projectile_step.seq)
        }),
        "missed untargeted projectile should not apply command damage",
    );

    assert!(
        !timeline.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::StatChanged {
                    target_instance_id,
                    modifier,
                    ..
                } if *target_instance_id == caster_id.into()
                    && modifier.stat == StatId::Attack
                    && modifier.kind == StatModifierKind::Flat
                    && modifier.value == 7
            )
        }),
        "follow-up step gated on previous damage must stay disabled after projectile miss"
    );

    assert!(
        battle.debug_skill_cast_state(ability_seq).is_none(),
        "terminal no-hit projectile resolution should clean up the cast instead of leaving deferred or pending step state behind"
    );
}

#[test]
fn untargeted_projectile_still_hits_later_unit_after_cast_target_dies() {
    let caster_base_uuid = Uuid::from_u128(0xCC23);
    let finisher_base_uuid = Uuid::from_u128(0xCC24);
    let doomed_target_base_uuid = Uuid::from_u128(0xCC25);
    let later_unit_base_uuid = Uuid::from_u128(0xCC26);
    let charge_target_base_uuid = Uuid::from_u128(0xCC27);

    let skill = SkillDef {
        id: "untargeted_death_through_shot".to_string(),
        name: "untargeted_death_through_shot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "death_through_shot".to_string(),
            delay_ms: 0,
            range_tiles: 4,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 5_000,
                collision: SkillProjectileCollisionDef {
                    radius_units: 1_000_000,
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage { amount: 20 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("untargeted_death_through_shot"),
                1,
                120,
                0,
                300,
                2,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "finisher",
                finisher_base_uuid,
                None,
                200,
                120,
                0,
                500,
                3,
                DeliveryDef::Projectile {
                    speed_units_per_ms: 10_000,
                    collision: Default::default(),
                },
                100,
            ),
            make_abnormality(
                "charge_target",
                charge_target_base_uuid,
                None,
                1,
                400,
                0,
                1_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "doomed_target",
                doomed_target_base_uuid,
                None,
                1,
                100,
                0,
                1_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "later_unit",
                later_unit_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle_with_setup(
        game_data,
        vec![
            (Uuid::from_u128(53), caster_base_uuid, Position::new(0, 0)),
            (Uuid::from_u128(54), finisher_base_uuid, Position::new(1, 0)),
        ],
        vec![
            (
                Uuid::from_u128(55),
                charge_target_base_uuid,
                Position::new(2, 0),
            ),
            (
                Uuid::from_u128(56),
                doomed_target_base_uuid,
                Position::new(0, 3),
            ),
            (
                Uuid::from_u128(57),
                later_unit_base_uuid,
                Position::new(0, 4),
            ),
        ],
        |core| {
            let finisher_id = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == finisher_base_uuid)
                .map(|unit| unit.instance_id)
                .expect("missing finisher runtime unit");
            let doomed_target_id = core
                .units
                .values()
                .find(|unit| {
                    unit.owner == Side::Opponent && unit.base_uuid == doomed_target_base_uuid
                })
                .map(|unit| unit.instance_id)
                .expect("missing doomed target runtime unit");
            core.units
                .get_mut(&finisher_id)
                .expect("finisher runtime unit should exist")
                .current_target = Some(doomed_target_id);
        },
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let finisher_id = spawned_unit_id(&timeline, finisher_base_uuid, Side::Player);
    let doomed_target_id = spawned_unit_id(&timeline, doomed_target_base_uuid, Side::Opponent);
    let later_unit_id = spawned_unit_id(&timeline, later_unit_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "untargeted_death_through_shot", caster_id);
    let step = step_entries_for_cast(&timeline, ability_seq)
        .into_iter()
        .next()
        .expect("missing death_through_shot step");

    let doomed_death_time_ms = timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                hp_after,
                reason,
                ..
            } if *source_instance_id == finisher_id.into()
                && *target_instance_id == doomed_target_id.into()
                && *reason == HpChangeReason::BasicAttack
                && *hp_after == 0 =>
            {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("finisher should kill the original cast target after launch");

    let projectile_hits: Vec<(u64, Uuid)> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && parent_seq(&entry.cause) == Some(step.seq) =>
            {
                Some((entry.time_ms, (*target_instance_id).into()))
            }
            _ => None,
        })
        .collect();

    assert_eq!(
        projectile_hits.len(),
        1,
        "expected exactly one projectile-delivered hit"
    );
    assert_eq!(
        projectile_hits[0].1,
        later_unit_id,
        "fixed projectile should continue to a later valid unit after the original cast target dies"
    );
    assert!(
        projectile_hits[0].0 > doomed_death_time_ms,
        "projectile hit should happen after the original cast target dies"
    );
}

#[test]
fn untargeted_projectile_hits_moving_target_after_post_launch_stun_that_would_otherwise_miss() {
    let caster_base_uuid = Uuid::from_u128(0xCC28);
    let target_base_uuid = Uuid::from_u128(0xCC29);

    let skill = SkillDef {
        id: "untargeted_stun_window_shot".to_string(),
        name: "untargeted_stun_window_shot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "stun_window_shot".to_string(),
            delay_ms: 0,
            range_tiles: 4,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 1_000_000,
                collision: SkillProjectileCollisionDef {
                    radius_units: 150_000,
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage { amount: 20 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("untargeted_stun_window_shot"),
                1,
                120,
                0,
                1_000_000,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "moving_target",
                target_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let player_units = vec![(Uuid::from_u128(58), caster_base_uuid, Position::new(0, 0))];
    let opponent_units = vec![(Uuid::from_u128(59), target_base_uuid, Position::new(0, 2))];

    let build_timeline = |apply_stun: bool| {
        run_battle_with_setup(
            Arc::clone(&game_data),
            player_units.clone(),
            opponent_units.clone(),
            |core| {
                let caster_id = core
                    .units
                    .values()
                    .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                    .map(|unit| Uuid::from(unit.instance_id))
                    .expect("missing caster runtime unit");
                let target_id = core
                    .units
                    .values()
                    .find(|unit| unit.owner == Side::Opponent && unit.base_uuid == target_base_uuid)
                    .map(|unit| Uuid::from(unit.instance_id))
                    .expect("missing moving target runtime unit");
                core.units
                    .get_mut(&target_id.into())
                    .expect("moving target runtime unit should exist")
                    .stats
                    .move_speed_units_per_ms = 250_000;

                set_linear_motion(
                    core,
                    target_id,
                    Position::new(0, 2),
                    0,
                    2_000_000,
                    300_000,
                    4_000_000,
                    0,
                );

                core.enqueue_event(BattleEvent::AutoCastStart {
                    time_ms: 0,
                    caster_instance_id: caster_id.into(),
                    cause: TimelineCause::Root {
                        kind: TimelineRootCause::System,
                    },
                });

                if apply_stun {
                    core.enqueue_event(BattleEvent::ApplyBuff {
                        time_ms: 1,
                        caster_instance_id: target_id.into(),
                        target_instance_id: target_id.into(),
                        buff_id: BuffId::from_name("stun"),
                        duration_ms: 10,
                        cause: TimelineCause::Root {
                            kind: TimelineRootCause::System,
                        },
                    });
                }
            },
        )
    };

    let baseline_timeline = build_timeline(false);
    let stunned_timeline = build_timeline(true);

    let stunned_caster_id = spawned_unit_id(&stunned_timeline, caster_base_uuid, Side::Player);
    let stunned_target_id = spawned_unit_id(&stunned_timeline, target_base_uuid, Side::Opponent);
    let stunned_hit_time_ms = stunned_timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                reason,
                ..
            } if *source_instance_id == stunned_caster_id.into()
                && *reason == HpChangeReason::Command =>
            {
                Some(entry.time_ms)
            }
            _ => None,
        })
        .expect("stunned projectile scenario should hit the moving target");

    assert!(
        stunned_timeline.entries.iter().any(|entry| {
            entry.time_ms == 1
                && matches!(
                    &entry.event,
                    TimelineEvent::BuffApplied {
                        target_instance_id,
                        buff_id,
                        ..
                    } if *target_instance_id == stunned_target_id.into()
                        && *buff_id == BuffId::from_name("stun")
                )
        }),
        "post-launch stun should be applied before the projectile hit"
    );
    assert!(
        !baseline_timeline.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    reason: HpChangeReason::Command,
                    ..
                }
            )
        }),
        "without post-launch hard CC, the fixed projectile should miss the moving target in this setup"
    );
    assert!(
        stunned_hit_time_ms >= 1,
        "post-launch hard CC should convert the same moving-target scenario into a real hit"
    );
}

#[test]
fn targeted_homing_followup_area_uses_impact_context_instead_of_live_target_position() {
    let caster_base_uuid = Uuid::from_u128(0xCC31);
    let primary_base_uuid = Uuid::from_u128(0xCC32);
    let splash_base_uuid = Uuid::from_u128(0xCC33);

    let skill = SkillDef {
        id: "targeted_homing_burst".to_string(),
        name: "targeted_homing_burst".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "homing_shot".to_string(),
                delay_ms: 0,
                range_tiles: 3,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 500_000,
                    collision: Default::default(),
                },
                effects: vec![SkillEffectDef::Damage { amount: 20 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "impact_burst".to_string(),
                delay_ms: 10,
                range_tiles: 3,
                target: SkillTarget::Enemies {
                    area: game_core::game::ability::SkillArea::RadiusChebyshev { radius_tiles: 1 },
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::IfPreviousStepDealtDamage,
                repeat: Default::default(),
                delivery: DeliveryDef::Area {
                    area: SkillAreaDeliveryDef {
                        shape: SkillAreaShapeDef::Circle {
                            radius_units: 200_000,
                        },
                        anchor: SkillAreaAnchorSource::ImpactContext,
                        hit_targets: SkillHitTargetFilter::Enemies,
                        include_caster: false,
                        tick_policy: game_core::game::ability::SkillAreaTickPolicy::EveryTick,
                        duration_ms: 0,
                        tick_interval_ms: None,
                    },
                },
                effects: vec![SkillEffectDef::Damage { amount: 7 }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("targeted_homing_burst"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "primary",
                primary_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "splash",
                splash_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle_with_setup(
        game_data,
        vec![(Uuid::from_u128(61), caster_base_uuid, Position::new(0, 0))],
        vec![
            (Uuid::from_u128(62), primary_base_uuid, Position::new(0, 2)),
            (Uuid::from_u128(63), splash_base_uuid, Position::new(1, 2)),
        ],
        |core| {
            let caster_id = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .map(|unit| unit.instance_id)
                .expect("missing caster runtime unit");
            let primary_id = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Opponent && unit.base_uuid == primary_base_uuid)
                .map(|unit| unit.instance_id)
                .expect("missing primary runtime unit");
            let splash_id = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Opponent && unit.base_uuid == splash_base_uuid)
                .map(|unit| unit.instance_id)
                .expect("missing splash runtime unit");

            let primary = core
                .units
                .get_mut(&primary_id)
                .expect("primary runtime unit should exist");
            primary.stats.move_speed_units_per_ms = 100_000;
            primary.action_locks.lock_movement_until(20);

            set_linear_motion(
                core,
                Uuid::from(primary_id),
                Position::new(0, 2),
                0,
                2_000_000,
                1_500_000,
                2_000_000,
                0,
            );

            let splash = core
                .units
                .get_mut(&splash_id)
                .expect("splash runtime unit should exist");
            splash.action_locks.lock_movement_until(20);
            splash.pos_x_units = 450_000;
            splash.pos_y_units = 2_000_000;
            splash.action_state = ActionState::Idle;

            core.enqueue_event(BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: TimelineCause::Root {
                    kind: TimelineRootCause::System,
                },
            });
        },
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let primary_id = spawned_unit_id(&timeline, primary_base_uuid, Side::Opponent);
    let splash_id = spawned_unit_id(&timeline, splash_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "targeted_homing_burst", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "expected projectile hit and follow-up area step"
    );

    let burst_step = steps[1];
    let homing_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && parent_seq(&entry.cause) == Some(steps[0].seq) =>
            {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        homing_targets,
        vec![primary_id],
        "the homing projectile should still land on the designated primary target before the follow-up area"
    );

    let burst_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && parent_seq(&entry.cause) == Some(burst_step.seq) =>
            {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert!(
        burst_targets.contains(&splash_id),
        "the follow-up area should still include the enemy that stayed near the projectile impact point: burst_targets={burst_targets:?}, primary_id={primary_id:?}, splash_id={splash_id:?}"
    );
    assert!(
        !burst_targets.contains(&primary_id),
        "the follow-up area should not snap to the primary target's later moved position: burst_targets={burst_targets:?}, primary_id={primary_id:?}, splash_id={splash_id:?}"
    );
}

#[test]
fn untargeted_piercing_projectile_respects_max_hits() {
    let caster_base_uuid = Uuid::from_u128(0xCC41);
    let first_base_uuid = Uuid::from_u128(0xCC42);
    let second_base_uuid = Uuid::from_u128(0xCC43);
    let third_base_uuid = Uuid::from_u128(0xCC44);

    let skill = SkillDef {
        id: "piercing_skillshot".to_string(),
        name: "piercing_skillshot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "piercing_shot".to_string(),
            delay_ms: 0,
            range_tiles: 4,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                collision: SkillProjectileCollisionDef {
                    piercing: true,
                    max_hits: Some(2),
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage { amount: 15 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("piercing_skillshot"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "first",
                first_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "second",
                second_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "third",
                third_base_uuid,
                None,
                1,
                50,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(71), caster_base_uuid, Position::new(0, 0))],
        vec![
            (Uuid::from_u128(72), first_base_uuid, Position::new(0, 1)),
            (Uuid::from_u128(73), second_base_uuid, Position::new(0, 2)),
            (Uuid::from_u128(74), third_base_uuid, Position::new(0, 3)),
        ],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let first_id = spawned_unit_id(&timeline, first_base_uuid, Side::Opponent);
    let second_id = spawned_unit_id(&timeline, second_base_uuid, Side::Opponent);
    let third_id = spawned_unit_id(&timeline, third_base_uuid, Side::Opponent);
    let (ability_seq, _) = find_first_ability_cast_seq(&timeline, "piercing_skillshot", caster_id);
    let step = step_entries_for_cast(&timeline, ability_seq)
        .into_iter()
        .next()
        .expect("missing piercing step");

    let damage_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && parent_seq(&entry.cause) == Some(step.seq) =>
            {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert_eq!(damage_targets, vec![first_id, second_id]);
    assert!(
        !damage_targets.contains(&third_id),
        "piercing projectile with max_hits=2 should stop before the third unit"
    );
}

#[test]
fn untargeted_projectile_legacy_despawn_false_still_pierces_for_compat() {
    let caster_base_uuid = Uuid::from_u128(0xCC51);
    let first_base_uuid = Uuid::from_u128(0xCC52);
    let second_base_uuid = Uuid::from_u128(0xCC53);
    let third_base_uuid = Uuid::from_u128(0xCC54);

    let skill = SkillDef {
        id: "legacy_piercing_skillshot".to_string(),
        name: "legacy_piercing_skillshot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "legacy_piercing_shot".to_string(),
            delay_ms: 0,
            range_tiles: 4,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                collision: SkillProjectileCollisionDef {
                    despawn_on_hit: Some(false),
                    max_hits: Some(2),
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage { amount: 15 }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("legacy_piercing_skillshot"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "first",
                first_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "second",
                second_base_uuid,
                None,
                1,
                200,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
            make_abnormality(
                "third_target",
                third_base_uuid,
                None,
                1,
                50,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let caster_owned_uuid = Uuid::from_u128(0xDD51);
    let first_owned_uuid = Uuid::from_u128(0xDD52);
    let second_owned_uuid = Uuid::from_u128(0xDD53);
    let third_owned_uuid = Uuid::from_u128(0xDD54);

    let timeline = run_battle(
        game_data,
        vec![(caster_owned_uuid, caster_base_uuid, Position::new(0, 0))],
        vec![
            (first_owned_uuid, first_base_uuid, Position::new(0, 1)),
            (second_owned_uuid, second_base_uuid, Position::new(0, 2)),
            (third_owned_uuid, third_base_uuid, Position::new(0, 3)),
        ],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let first_id = spawned_unit_id(&timeline, first_base_uuid, Side::Opponent);
    let second_id = spawned_unit_id(&timeline, second_base_uuid, Side::Opponent);
    let third_id = spawned_unit_id(&timeline, third_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&timeline, "legacy_piercing_skillshot", caster_id);
    let step_entry = step_entries_for_cast(&timeline, ability_seq)
        .into_iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityStepTriggered { step_id, .. } if step_id == "legacy_piercing_shot"
            )
        })
        .expect("missing legacy piercing step");
    let step_seq = step_entry.seq;

    let damaged_targets: Vec<Uuid> = timeline
        .entries
        .iter()
        .filter(|entry| parent_seq(&entry.cause) == Some(step_seq))
        .filter_map(|entry| match entry.event {
            TimelineEvent::HpChanged {
                target_instance_id,
                delta,
                reason,
                ..
            } if delta < 0 && reason == HpChangeReason::Command => {
                Some::<Uuid>(target_instance_id.into())
            }
            _ => None,
        })
        .collect();

    assert!(
        damaged_targets.contains(&first_id),
        "legacy despawn_on_hit=false should still pierce into the first blocker"
    );
    assert!(
        damaged_targets.contains(&second_id),
        "legacy despawn_on_hit=false should still pierce into the second blocker"
    );
    assert!(
        !damaged_targets.contains(&third_id),
        "legacy compatibility should still respect max_hits=2 and stop before the third unit"
    );
}

#[test]
fn hit_gated_self_heal_and_buff_stack_repeat_attack_work_together() {
    let caster_base_uuid = Uuid::from_u128(0xBD11);
    let enemy_base_uuid = Uuid::from_u128(0xBD12);

    let skill = SkillDef {
        id: "predation_cycle".to_string(),
        name: "predation_cycle".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "prime_stacks".to_string(),
                delay_ms: 0,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::Times { count: 3 },
                delivery: DeliveryDef::Instant,
                effects: vec![
                    SkillEffectDef::ApplyBuff {
                        buff_id: "poison".to_string(),
                        duration_ms: 5_000,
                    },
                    SkillEffectDef::Damage { amount: 5 },
                ],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "opening_strike".to_string(),
                delay_ms: 1,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 12 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "heal_on_hit".to_string(),
                delay_ms: 2,
                range_tiles: 1,
                target: SkillTarget::SelfUnit,
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::IfPreviousStepDealtDamage,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Heal { amount: 8 }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "stacked_barrage".to_string(),
                delay_ms: 3,
                range_tiles: 1,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::ByBuffStacks {
                    unit: SkillUnitReference::SelfUnit,
                    buff_id: "poison".to_string(),
                    max: Some(5),
                },
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage { amount: 7 }],
                presentation: SkillPresentationDef::default(),
            },
        ],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("predation_cycle"),
                1,
                120,
                0,
                300,
                1,
                DeliveryDef::Instant,
                10,
            ),
            make_abnormality(
                "enemy",
                enemy_base_uuid,
                None,
                1,
                500,
                9999,
                1_000_000,
                1,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let timeline = run_battle(
        game_data,
        vec![(Uuid::from_u128(31), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(32), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&timeline, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&timeline, enemy_base_uuid, Side::Opponent);
    let (ability_seq, _) = find_first_ability_cast_seq(&timeline, "predation_cycle", caster_id);
    let steps = step_entries_for_cast(&timeline, ability_seq);
    assert_eq!(steps.len(), 4, "expected all 4 steps to execute");

    let heal_step = steps[2];
    let barrage_step = steps[3];

    let heal_events = timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    hp_after,
                    hp_before,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == caster_id.into()
                    && *reason == HpChangeReason::Command
                    && hp_after > hp_before
            ) && parent_seq(&entry.cause) == Some(heal_step.seq)
        })
        .count();
    assert_eq!(
        heal_events, 1,
        "expected a single self-heal after the opening hit"
    );

    let barrage_hits = timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && *reason == HpChangeReason::Command
            ) && parent_seq(&entry.cause) == Some(barrage_step.seq)
        })
        .count();
    assert_eq!(
        barrage_hits, 3,
        "expected repeated barrage hits to match the caster's 3 poison stacks"
    );
}

#[test]
fn ron_added_abnormalities_emit_expected_skill_event_categories_in_battle_smoke() {
    struct SkillCase {
        abnormality_id: &'static str,
        skill_id: &'static str,
        expected_steps: usize,
        expected_buff: Option<&'static str>,
        expect_triggered_attacks: usize,
        expect_positive_command_hp_change: bool,
        expect_negative_command_hp_change: bool,
        expect_stat_change: bool,
        expect_resonance_change: bool,
    }

    let base_game_data = common::load_game_data_from_ron();
    let training_dummy_uuid = Uuid::from_u128(0xDEAD_BEEF);
    let mut abnormalities = base_game_data.abnormality_data.items.clone();
    abnormalities.push(make_abnormality(
        "skill_test_dummy",
        training_dummy_uuid,
        None,
        1,
        5_000,
        9_999,
        1_000_000,
        1,
        DeliveryDef::Instant,
        100,
    ));

    let game_data = Arc::new(GameDataBase::new(
        game_core::game::data::GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(abnormalities)),
            artifact_data: Arc::clone(&base_game_data.artifact_data),
            equipment_data: Arc::clone(&base_game_data.equipment_data),
            shop_data: Arc::clone(&base_game_data.shop_data),
            bonus_data: Arc::clone(&base_game_data.bonus_data),
            random_event_data: Arc::clone(&base_game_data.random_event_data),
            pve_data: Arc::clone(&base_game_data.pve_data),
            skill_data: Arc::clone(&base_game_data.skill_data),
            event_pools: base_game_data.event_pools.clone(),
        },
    ));

    let cases = [
        SkillCase {
            abnormality_id: "o-03-03_one_sin",
            skill_id: "one_sin_penitence",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-56_punishing_bird",
            skill_id: "punishing_bird_rapid_peck",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 3,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: false,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-40_big_bird",
            skill_id: "big_bird_dark_lamp",
            expected_steps: 2,
            expected_buff: Some("silence"),
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-62_judgement_bird",
            skill_id: "judgement_bird_scales",
            expected_steps: 2,
            expected_buff: Some("stun"),
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-01-04_queen_of_hatred",
            skill_id: "queen_of_hatred_magical_beam",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "f-01-57_little_red",
            skill_id: "little_red_hunt_the_prey",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 2,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "t-01-75_mountain",
            skill_id: "mountain_mass_consumption",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: true,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "d-03-109_melting_love",
            skill_id: "melting_love_slime_infection",
            expected_steps: 2,
            expected_buff: Some("poison"),
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-06-20_nothing_there",
            skill_id: "nothing_there_goodbye",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: true,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-01-45_white_night",
            skill_id: "white_night_pale_benediction",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: true,
            expect_stat_change: true,
            expect_resonance_change: false,
        },
    ];

    for (index, case) in cases.iter().enumerate() {
        let tested_abnormality = game_data
            .abnormality_data
            .get_by_id(case.abnormality_id)
            .unwrap_or_else(|| panic!("missing abnormality {}", case.abnormality_id));

        assert_eq!(
            tested_abnormality.skill_id.as_deref(),
            Some(case.skill_id),
            "unexpected skill_id for {}",
            case.abnormality_id
        );

        let timeline = run_battle(
            Arc::clone(&game_data),
            vec![
                (
                    Uuid::from_u128(0x1000 + index as u128),
                    tested_abnormality.uuid,
                    Position::new(0, 0),
                ),
                (
                    Uuid::from_u128(0x2000 + index as u128),
                    training_dummy_uuid,
                    Position::new(0, 2),
                ),
            ],
            vec![(
                Uuid::from_u128(0x3000 + index as u128),
                training_dummy_uuid,
                Position::new(0, 1),
            )],
        );

        let caster_id = spawned_unit_id(&timeline, tested_abnormality.uuid, Side::Player);
        let (ability_seq, _) = find_first_ability_cast_seq(&timeline, case.skill_id, caster_id);
        let steps = step_entries_for_cast(&timeline, ability_seq);
        assert_eq!(
            steps.len(),
            case.expected_steps,
            "unexpected step count for {}",
            case.abnormality_id
        );

        let step_seqs: Vec<u64> = steps.iter().map(|entry| entry.seq).collect();
        let caused_by_steps = |entry: &game_core::game::battle::timeline::TimelineEntry| {
            parent_seq(&entry.cause).is_some_and(|seq| step_seqs.contains(&seq))
        };

        if let Some(buff_name) = case.expected_buff {
            let expected_buff_id = BuffId::from_name(buff_name);
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(&entry.event, TimelineEvent::BuffApplied { caster_instance_id, buff_id, .. }
                        if *caster_instance_id == caster_id.into() && *buff_id == expected_buff_id)
                        && caused_by_steps(entry)
                }),
                "expected BuffApplied({buff_name}) for {}",
                case.abnormality_id
            );
        }

        if case.expect_triggered_attacks > 0 {
            let triggered_attacks = timeline
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::AttackStart {
                            attacker_instance_id,
                            kind: Some(AttackKind::Triggered),
                            ..
                        } if *attacker_instance_id == caster_id.into()
                    ) && caused_by_steps(entry)
                })
                .count();
            assert!(
                triggered_attacks >= case.expect_triggered_attacks,
                "expected at least {} triggered attacks for {} but got {}",
                case.expect_triggered_attacks,
                case.abnormality_id,
                triggered_attacks
            );
        }

        if case.expect_positive_command_hp_change {
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::HpChanged {
                            source_instance_id: Some(source_instance_id),
                            delta,
                            reason: HpChangeReason::Command,
                            ..
                        } if *source_instance_id == caster_id.into()
                            && *delta > 0
                    ) && caused_by_steps(entry)
                }),
                "expected positive command HpChanged for {}",
                case.abnormality_id
            );
        }

        if case.expect_negative_command_hp_change {
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::HpChanged {
                            source_instance_id: Some(source_instance_id),
                            delta,
                            reason: HpChangeReason::Command,
                            ..
                        } if *source_instance_id == caster_id.into()
                            && *delta < 0
                    ) && caused_by_steps(entry)
                }),
                "expected negative command HpChanged for {}",
                case.abnormality_id
            );
        }

        if case.expect_stat_change {
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::StatChanged {
                            source_instance_id: _,
                            ..
                        }
                    ) && caused_by_steps(entry)
                }),
                "expected StatChanged for {}",
                case.abnormality_id
            );
        }

        if case.expect_resonance_change {
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::ResonanceChanged { unit_instance_id, .. }
                        if *unit_instance_id == caster_id.into()
                    ) && caused_by_steps(entry)
                }),
                "expected ResonanceChanged for {}",
                case.abnormality_id
            );
        }
    }
}
