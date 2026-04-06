mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::{
    DeliveryDef, SkillCastTargetingDef, SkillDef, SkillEffectDef, SkillKind, SkillPresentationDef,
    SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget, SkillUnitReference,
    StepTargetingMode, UnitTargetRule,
};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::timeline::{
    AttackKind, HpChangeReason, Timeline, TimelineCause, TimelineEvent,
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
    let player = deck(player_units);
    let opponent = deck(opponent_units);

    let mut battle = BattleCore::new(&player, &opponent, game_data, common::BOARD_SIZE, 4242);
    let mut world = World::new();
    battle.run_battle(&mut world).expect("battle runs").timeline
}

fn parent_seq(cause: &TimelineCause) -> Option<u64> {
    match cause {
        TimelineCause::Parent { seq } => Some(*seq),
        TimelineCause::Root { .. } => None,
    }
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
fn ron_added_abnormalities_trigger_expected_skill_effects_in_battle() {
    struct SkillCase {
        abnormality_id: &'static str,
        skill_id: &'static str,
        expected_steps: usize,
        expected_buff: Option<&'static str>,
        expect_triggered_attacks: usize,
        expect_command_hp_change: bool,
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
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-56_punishing_bird",
            skill_id: "punishing_bird_rapid_peck",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 3,
            expect_command_hp_change: false,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-40_big_bird",
            skill_id: "big_bird_dark_lamp",
            expected_steps: 2,
            expected_buff: Some("silence"),
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-02-62_judgement_bird",
            skill_id: "judgement_bird_scales",
            expected_steps: 2,
            expected_buff: Some("stun"),
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-01-04_queen_of_hatred",
            skill_id: "queen_of_hatred_magical_beam",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "f-01-57_little_red",
            skill_id: "little_red_hunt_the_prey",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 2,
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "t-01-75_mountain",
            skill_id: "mountain_mass_consumption",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: true,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "d-03-109_melting_love",
            skill_id: "melting_love_slime_infection",
            expected_steps: 2,
            expected_buff: Some("poison"),
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: false,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-06-20_nothing_there",
            skill_id: "nothing_there_goodbye",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
            expect_stat_change: true,
            expect_resonance_change: false,
        },
        SkillCase {
            abnormality_id: "o-01-45_white_night",
            skill_id: "white_night_pale_benediction",
            expected_steps: 3,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_command_hp_change: true,
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

        if case.expect_command_hp_change {
            assert!(
                timeline.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        TimelineEvent::HpChanged {
                            source_instance_id: Some(source_instance_id),
                            reason: HpChangeReason::Command,
                            ..
                        } if *source_instance_id == caster_id.into()
                    ) && caused_by_steps(entry)
                }),
                "expected command HpChanged for {}",
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
