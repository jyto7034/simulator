mod common;

use std::sync::Arc;

use game_core::game::ability::{
    DeliveryDef, ProjectileHitPolicy, SkillCastTargetingDef, SkillDef, SkillEffectDef,
    SkillHitTargetFilter, SkillId, SkillKind, SkillPresentationDef, SkillProjectileCollisionDef,
    SkillStepCondition, SkillStepDef, SkillStepRepeat, SkillTarget, SkillUnitReference,
    StepTargetingMode, UnitTargetRule,
};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::enums::BattleEvent;
use game_core::game::battle::event_log::{
    AttackKind, BattleEventCause, BattleEventLog, BattleEventRootCause, BattleLogEvent,
    BattleProjectileGuidance, HpChangeReason,
};
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioEvent, ScenarioEventId,
    ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef, ScenarioUnitSpawn,
    WinCondition,
};
use game_core::game::battle::tile_range::{FacingDirection, TileRangePattern};
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::data::{
    abnormality_data::{AbnormalityMetadata, BasicAttackDef, MovementDef, ResonanceDef},
    skill_data::SkillDatabase,
    GameDataBase, GameDataBuilder,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use game_core::game::stats::{StatId, StatModifier, StatModifierKind};
use uuid::Uuid;

fn unit_draft(owned_uuid: Uuid, base_uuid: Uuid) -> BattleUnitDraft {
    BattleUnitDraft {
        owned_uuid,
        source: BattleUnitSource::Abnormality { base_uuid },
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        level: Tier::I,
        growth_stacks: GrowthStack::new(),
        equipped_items: vec![],
        equipped_item_enhancements: vec![],
    }
}

fn broad_defense_tile_range() -> TileRangePattern {
    TileRangePattern {
        include_anchor_tile: false,
        rows: vec![
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXX@XXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
            "XXXXXXXXX".to_string(),
        ],
    }
}

fn explicit_cast_targeting(target: SkillTarget) -> SkillCastTargetingDef {
    SkillCastTargetingDef::explicit(target, Default::default(), None, false)
}

fn explicit_nearest_enemy_cast_targeting() -> SkillCastTargetingDef {
    explicit_cast_targeting(SkillTarget::EnemySingle {
        rule: UnitTargetRule::Nearest,
    })
}

fn skill_with_broad_defense_tile_range(mut skill: SkillDef) -> SkillDef {
    let fallback_range = broad_defense_tile_range();
    let SkillCastTargetingDef::Explicit {
        defense_tile_range, ..
    } = &mut skill.cast_targeting;
    *defense_tile_range = Some(fallback_range.clone());

    for step in &mut skill.steps {
        step.defense_tile_range = Some(fallback_range.clone());
    }

    skill
}

fn spawn_group(
    id: &str,
    side: Side,
    required_for_victory: bool,
    units: Vec<(Uuid, Uuid, Position)>,
) -> ScenarioSpawnGroup {
    let group_id = ScenarioGroupId::new(id);
    let spawns = units
        .into_iter()
        .enumerate()
        .map(
            |(index, (owned_uuid, base_uuid, position))| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("{}_{}", group_id.0, index)),
                side,
                draft: unit_draft(owned_uuid, base_uuid),
                position,
                instance_salt: index as u32,
            },
        )
        .collect();

    ScenarioSpawnGroup {
        id: group_id,
        side,
        required_for_victory,
        enemy_movement_plan: None,
        spawns,
    }
}

fn battle_scenario(
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
) -> BattleScenario {
    let player_group_id = "player_initial";
    let enemy_group_id = "enemy_initial";
    BattleScenario {
        battlefield: BattleFieldSpec {
            width: common::BOARD_SIZE.0,
            height: common::BOARD_SIZE.1,
            valid_tiles: Vec::new(),
            obstacles: Vec::new(),
        },
        artifacts: Vec::new(),
        groups: vec![
            spawn_group(player_group_id, Side::Player, false, player_units),
            spawn_group(enemy_group_id, Side::Opponent, true, opponent_units),
        ],
        events: vec![
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_player_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(player_group_id),
                },
                once: true,
            },
            ScenarioEvent {
                id: ScenarioEventId::new("spawn_enemy_initial"),
                trigger: ScenarioTrigger::AtBattleStart,
                action: ScenarioAction::SpawnGroup {
                    group_id: ScenarioGroupId::new(enemy_group_id),
                },
                once: true,
            },
        ],
        win_condition: WinCondition::AllRequiredEnemyGroupsDefeated,
        tactical_plan: game_core::game::battle::scenario::TacticalPlan::default(),
    }
}

fn make_abnormality(
    id: &str,
    uuid: Uuid,
    skill_id: Option<&str>,
    attack: u32,
    max_health: u32,
    defense: i32,
    attack_interval_ms: u64,
    attack_range_units: u8,
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
        magic_resist: 0,
        threat_class: game_core::game::battle::types::BattleUnitThreatClass::Elite,
        movement: MovementDef {
            speed_units_per_ms: 3000,
            radius_units: 350_000,
        },
        basic_attack: BasicAttackDef {
            range_units: f32::from(attack_range_units),
            defense_tile_range: Some(broad_defense_tile_range()),
            interval_ms: attack_interval_ms,
            windup_ms: 0,
            delivery: attack_delivery,
            ..BasicAttackDef::default()
        },
        resonance: ResonanceDef {
            start: 0,
            max: resonance_max,
            gain_lock_ms: 0,
        },
        skill_id: skill_id.map(SkillId::from),
        mobility_kind: Default::default(),
        target_traits: Vec::new(),
    }
}

fn minimal_game_data(
    abnormalities: Vec<AbnormalityMetadata>,
    skills: Vec<SkillDef>,
) -> Arc<GameDataBase> {
    let skills = skills
        .into_iter()
        .map(skill_with_broad_defense_tile_range)
        .collect();
    GameDataBuilder::live_defaults()
        .with_abnormalities(abnormalities)
        .with_skills(SkillDatabase::new(skills))
        .build_arc()
}

fn minimal_game_data_without_skill_range_override(
    abnormalities: Vec<AbnormalityMetadata>,
    skills: Vec<SkillDef>,
) -> Arc<GameDataBase> {
    GameDataBuilder::live_defaults()
        .with_abnormalities(abnormalities)
        .with_skills(SkillDatabase::new(skills))
        .build_arc()
}

fn run_battle(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
) -> BattleEventLog {
    run_battle_with_setup(game_data, player_units, opponent_units, |_| {})
}

fn run_battle_with_setup<F>(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
    setup: F,
) -> BattleEventLog
where
    F: FnOnce(&mut BattleCore),
{
    let scenario = battle_scenario(player_units, opponent_units);
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 4242);
    battle
        .run_battle_with_post_spawn_setup(setup)
        .expect("battle runs")
        .event_log
}

fn run_battle_and_capture_core_with_setup<F>(
    game_data: Arc<GameDataBase>,
    player_units: Vec<(Uuid, Uuid, Position)>,
    opponent_units: Vec<(Uuid, Uuid, Position)>,
    setup: F,
) -> (BattleEventLog, BattleCore)
where
    F: FnOnce(&mut BattleCore),
{
    let scenario = battle_scenario(player_units, opponent_units);
    let mut battle = BattleCore::new_from_scenario(scenario, game_data, 4242);
    battle
        .run_battle_with_post_spawn_setup(setup)
        .expect("battle runs");
    (battle.event_log.clone(), battle)
}

fn parent_seq(cause: &BattleEventCause) -> Option<u64> {
    match cause {
        BattleEventCause::Parent { seq } => Some(*seq),
        BattleEventCause::Root { .. } => None,
    }
}

fn entry_caused_by_seq(
    event_log: &BattleEventLog,
    entry: &game_core::game::battle::event_log::BattleEventLogEntry,
    expected_seq: u64,
) -> bool {
    let Some(direct_parent_seq) = parent_seq(&entry.cause) else {
        return false;
    };
    if direct_parent_seq == expected_seq {
        return true;
    }

    event_log
        .entries
        .iter()
        .find(|entry| entry.seq == direct_parent_seq)
        .is_some_and(|parent| {
            matches!(parent.event, BattleLogEvent::SkillAreaDeclared { .. })
                && parent_seq(&parent.cause) == Some(expected_seq)
        })
}

fn spawned_unit_id(event_log: &BattleEventLog, base_uuid: Uuid, owner: Side) -> Uuid {
    event_log
        .entries
        .iter()
        .find_map(|entry| match entry.event {
            BattleLogEvent::UnitSpawned {
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
    event_log: &BattleEventLog,
    skill_id: &str,
    caster_instance_id: Uuid,
) -> (u64, u64) {
    let entry = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast {
                    skill_id: actual_skill_id,
                    caster_instance_id: actual_caster,
                    ..
                } if actual_skill_id == skill_id && *actual_caster == caster_instance_id.into()
            )
        })
        .unwrap_or_else(|| {
            let observed: Vec<_> = event_log
                .entries
                .iter()
                .filter_map(|entry| match &entry.event {
                    BattleLogEvent::AbilityCast {
                        skill_id,
                        caster_instance_id,
                        ..
                    } => Some((entry.time_ms, skill_id.clone(), *caster_instance_id)),
                    _ => None,
                })
                .collect();
            panic!(
                "missing AbilityCast skill_id={skill_id} caster_instance_id={caster_instance_id}; observed={observed:?}"
            )
        });

    (entry.seq, entry.time_ms)
}

fn step_entries_for_cast<'a>(
    event_log: &'a BattleEventLog,
    ability_seq: u64,
) -> Vec<&'a game_core::game::battle::event_log::BattleEventLogEntry> {
    event_log
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.event, BattleLogEvent::AbilityStepTriggered { .. })
                && parent_seq(&entry.cause) == Some(ability_seq)
        })
        .collect()
}

#[test]
fn mixed_target_skill_records_enemy_damage_then_self_buff() {
    let caster_base_uuid = Uuid::from_u128(0xAA11);
    let enemy_base_uuid = Uuid::from_u128(0xAA12);

    let skill = SkillDef {
        id: SkillId::from("enemy_then_self"),
        name: "enemy_then_self".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: explicit_nearest_enemy_cast_targeting(),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "enemy_burst".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage {
                    amount: 15,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "self_buff".to_string(),
                delay_ms: 50,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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

    let event_log = run_battle(
        game_data,
        vec![(Uuid::from_u128(1), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(2), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&event_log, "enemy_then_self", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "expected exactly 2 step entries for the first cast"
    );

    let first_step = steps[0];
    let second_step = steps[1];
    assert_eq!(first_step.time_ms, ability_time_ms);
    assert_eq!(second_step.time_ms, ability_time_ms + 50);

    assert!(event_log.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *target_instance_id == enemy_id.into()
                && *reason == HpChangeReason::Command
        ) && entry_caused_by_seq(&event_log, entry, first_step.seq)
    }));

    assert!(event_log.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            BattleLogEvent::StatChanged {
                target_instance_id,
                modifier,
                ..
            } if *target_instance_id == caster_id.into()
                && modifier.stat == StatId::Attack
                && modifier.kind == StatModifierKind::Flat
                && modifier.value == 5
        ) && entry_caused_by_seq(&event_log, entry, second_step.seq)
    }));
}

#[test]
fn self_then_retargeted_enemy_skill_resolves_second_step_at_execution_time() {
    let caster_base_uuid = Uuid::from_u128(0xBC11);
    let enemy_base_uuid = Uuid::from_u128(0xBC12);

    let skill = SkillDef {
        id: SkillId::from("self_then_retarget"),
        name: "self_then_retarget".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: explicit_cast_targeting(SkillTarget::SelfUnit),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "self_charge".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage {
                    amount: 25,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
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

    let event_log = run_battle(
        game_data,
        vec![(Uuid::from_u128(21), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(22), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&event_log, "self_then_retarget", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
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
        BattleLogEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == caster_id.into()
    ));

    assert!(matches!(
        &enemy_step.event,
        BattleLogEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy_id.into()
    ));

    assert!(event_log.entries.iter().any(|entry| {
        matches!(
            &entry.event,
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *target_instance_id == enemy_id.into()
                && *reason == HpChangeReason::Command
        ) && entry_caused_by_seq(&event_log, entry, enemy_step.seq)
    }));
}

#[test]
fn conditional_followup_waits_for_projectile_damage_resolution() {
    let caster_base_uuid = Uuid::from_u128(0xBC21);
    let enemy_base_uuid = Uuid::from_u128(0xBC22);

    let skill = SkillDef {
        id: SkillId::from("conditional_projectile_followup"),
        name: "conditional_projectile_followup".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: explicit_nearest_enemy_cast_targeting(),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "delayed_shot".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 250_000,
                    hit_policy: ProjectileHitPolicy::DirectionalCollision,
                    allow_targetless_cast: false,
                    max_range_tiles: Some(2),
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
                    collision: Default::default(),
                },
                effects: vec![SkillEffectDef::Damage {
                    amount: 20,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "heal_on_hit".to_string(),
                delay_ms: 1,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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

    let event_log = run_battle_with_setup(
        game_data,
        vec![(Uuid::from_u128(21), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(22), enemy_base_uuid, Position::new(0, 2))],
        |core| {
            let caster_id = core
                .units
                .values()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .map(|unit| unit.instance_id)
                .expect("missing caster runtime unit");

            core.enqueue_event(BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id: caster_id,
                cause: BattleEventCause::Root {
                    kind: BattleEventRootCause::System,
                },
            });
            core.units
                .get_mut(&caster_id)
                .expect("caster runtime unit should exist")
                .facing_direction = Some(FacingDirection::Down);
        },
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);
    let (ability_seq, ability_time_ms) =
        find_first_ability_cast_seq(&event_log, "conditional_projectile_followup", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
    assert_eq!(
        steps.len(),
        2,
        "follow-up step should be deferred, not dropped"
    );

    let projectile_step = steps[0];
    let followup_step = steps[1];

    let projectile_impact_entry = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && *reason == HpChangeReason::Command
            ) && entry_caused_by_seq(&event_log, entry, projectile_step.seq)
        })
        .expect("missing projectile hit");

    let followup_entry = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::StatChanged {
                    target_instance_id,
                    modifier,
                    ..
                } if *target_instance_id == caster_id.into()
                    && modifier.stat == StatId::Attack
                    && modifier.kind == StatModifierKind::Flat
                    && modifier.value == 5
            ) && entry_caused_by_seq(&event_log, entry, followup_step.seq)
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
        id: SkillId::from("self_charge_then_locked_shot"),
        name: "self_charge_then_locked_shot".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::Explicit {
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
        },
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "self_charge".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::CurrentTarget,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage {
                    amount: 25,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
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

    let event_log = run_battle(
        game_data,
        vec![(Uuid::from_u128(41), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(42), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);

    let autocast_start = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AutoCastStart {
                    caster_instance_id,
                    target: Some(game_core::game::battle::event_log::SkillCastTarget::Unit {
                        unit_instance_id
                    }),
                    ..
                } if *caster_instance_id == caster_id.into() && *unit_instance_id == enemy_id.into()
            )
        })
        .expect("missing AutoCastStart with explicit enemy cast target");

    let ability_cast = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityCast {
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
        find_first_ability_cast_seq(&event_log, "self_charge_then_locked_shot", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
    assert_eq!(steps.len(), 2, "expected exactly 2 executed steps");

    let self_step = steps[0];
    let enemy_step = steps[1];
    assert_eq!(self_step.time_ms, ability_time_ms);
    assert_eq!(enemy_step.time_ms, ability_time_ms + 10);

    assert!(matches!(
        &self_step.event,
        BattleLogEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == caster_id.into()
    ));
    assert!(matches!(
        &enemy_step.event,
        BattleLogEvent::AbilityStepTriggered {
            target_instance_id: Some(target_instance_id),
            ..
        } if *target_instance_id == enemy_id.into()
    ));
}

#[test]
fn ability_step_event_log_includes_presentation_metadata() {
    let caster_base_uuid = Uuid::from_u128(0xCA51);
    let enemy_base_uuid = Uuid::from_u128(0xCA52);

    let skill = SkillDef {
        id: SkillId::from("presentation_skill"),
        name: "presentation_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: explicit_nearest_enemy_cast_targeting(),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "judgement".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                hit_policy: ProjectileHitPolicy::TargetLocked,
                allow_targetless_cast: false,
                max_range_tiles: None,
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: Default::default(),
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 10,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
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

    let event_log = run_battle(
        game_data,
        vec![(Uuid::from_u128(31), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(32), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let step_entry = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::AbilityStepTriggered {
                    skill_id,
                    caster_instance_id,
                    presentation: Some(_),
                    ..
                } if skill_id == "presentation_skill" && *caster_instance_id == caster_id.into()
            )
        })
        .expect("missing AbilityStepTriggered with presentation metadata");

    match &step_entry.event {
        BattleLogEvent::AbilityStepTriggered {
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

    let launch_entry = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::SkillProjectileLaunched {
                    skill_id,
                    step_id,
                    caster_instance_id,
                    projectile_vfx_id: Some(projectile_vfx_id),
                    guidance: BattleProjectileGuidance::Homing,
                    ..
                } if skill_id == "presentation_skill"
                    && step_id == "judgement"
                    && *caster_instance_id == caster_id.into()
                    && projectile_vfx_id == "white_night_judgement"
            ) && entry_caused_by_seq(&event_log, entry, step_entry.seq)
        })
        .expect("missing SkillProjectileLaunched with projectile VFX metadata");

    let delivery_id = match launch_entry.event {
        BattleLogEvent::SkillProjectileLaunched {
            delivery_id,
            start,
            aim,
            ..
        } => {
            assert_ne!(start, aim);
            delivery_id
        }
        ref other => panic!("unexpected event: {other:?}"),
    };

    event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::SkillProjectileImpacted {
                    delivery_id: actual_delivery_id,
                    skill_id,
                    step_id,
                    caster_instance_id,
                    first_hit_unit_id: Some(_),
                    impact_vfx_id: Some(impact_vfx_id),
                    ..
                } if *actual_delivery_id == delivery_id
                    && skill_id == "presentation_skill"
                    && step_id == "judgement"
                    && *caster_instance_id == caster_id.into()
                    && impact_vfx_id == "white_night_judgement_hit"
            ) && entry_caused_by_seq(&event_log, entry, step_entry.seq)
        })
        .expect("missing SkillProjectileImpacted with impact VFX metadata");
}

#[test]
fn untargeted_projectile_miss_finalizes_step_and_cleans_up_damage_gated_followup() {
    let caster_base_uuid = Uuid::from_u128(0xCC21);
    let enemy_base_uuid = Uuid::from_u128(0xCC22);

    let skill = SkillDef {
        id: SkillId::from("untargeted_miss_then_check"),
        name: "untargeted_miss_then_check".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: explicit_nearest_enemy_cast_targeting(),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "missable_shot".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::ReuseCastTarget,
                when: Default::default(),
                repeat: Default::default(),
                delivery: DeliveryDef::Projectile {
                    speed_units_per_ms: 500_000,
                    hit_policy: ProjectileHitPolicy::DirectionalCollision,
                    allow_targetless_cast: false,
                    max_range_tiles: Some(2),
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
                    collision: SkillProjectileCollisionDef {
                        hit_targets: SkillHitTargetFilter::Allies,
                        ..Default::default()
                    },
                },
                effects: vec![SkillEffectDef::Damage {
                    amount: 20,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "followup_buff".to_string(),
                delay_ms: 1,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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

    let (event_log, battle) = run_battle_and_capture_core_with_setup(
        game_data,
        vec![(Uuid::from_u128(51), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(52), enemy_base_uuid, Position::new(0, 1))],
        |_| {},
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&event_log, "untargeted_miss_then_check", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
    assert_eq!(steps.len(), 1);

    let projectile_step = steps[0];

    assert!(
        !event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *reason == HpChangeReason::Command
            ) && entry_caused_by_seq(&event_log, entry, projectile_step.seq)
        }),
        "missed untargeted projectile should not apply command damage",
    );

    assert!(
        !event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::StatChanged {
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
        id: SkillId::from("untargeted_death_through_shot"),
        name: "untargeted_death_through_shot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: explicit_cast_targeting(SkillTarget::EnemySingle {
            rule: UnitTargetRule::LowestHealthEnemy,
        }),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "death_through_shot".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 5_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: false,
                max_range_tiles: Some(4),
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef {
                    radius_units: 1_000_000,
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 20,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
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
                    hit_policy: ProjectileHitPolicy::TargetLocked,
                    allow_targetless_cast: false,
                    max_range_tiles: None,
                    max_lifetime_ms: None,
                    max_kills: None,
                    max_pierces: None,
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

    let event_log = run_battle_with_setup(
        game_data,
        vec![
            (Uuid::from_u128(53), caster_base_uuid, Position::new(0, 0)),
            (Uuid::from_u128(54), finisher_base_uuid, Position::new(1, 0)),
        ],
        vec![
            (
                Uuid::from_u128(55),
                charge_target_base_uuid,
                Position::new(6, 0),
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
                .values_mut()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .expect("caster runtime unit should exist")
                .facing_direction = Some(FacingDirection::Down);
            core.units
                .get_mut(&finisher_id)
                .expect("finisher runtime unit should exist")
                .current_target = Some(doomed_target_id);
            core.units
                .get_mut(&finisher_id)
                .expect("finisher runtime unit should exist")
                .action_locks
                .lock_basic_attack_until(250);
            for unit in core.units.values_mut() {
                if unit.owner == Side::Opponent {
                    unit.action_locks.lock_basic_attack_until(60_000);
                }
            }
        },
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let finisher_id = spawned_unit_id(&event_log, finisher_base_uuid, Side::Player);
    let doomed_target_id = spawned_unit_id(&event_log, doomed_target_base_uuid, Side::Opponent);
    let later_unit_id = spawned_unit_id(&event_log, later_unit_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&event_log, "untargeted_death_through_shot", caster_id);
    let step = step_entries_for_cast(&event_log, ability_seq)
        .into_iter()
        .next()
        .expect("missing death_through_shot step");

    let doomed_death_time_ms = event_log
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            BattleLogEvent::HpChanged {
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

    let projectile_hits: Vec<(u64, Uuid)> = event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && entry_caused_by_seq(&event_log, entry, step.seq) =>
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
fn untargeted_piercing_projectile_respects_max_hits() {
    let caster_base_uuid = Uuid::from_u128(0xCC41);
    let first_base_uuid = Uuid::from_u128(0xCC42);
    let second_base_uuid = Uuid::from_u128(0xCC43);
    let third_base_uuid = Uuid::from_u128(0xCC44);

    let skill = SkillDef {
        id: SkillId::from("piercing_skillshot"),
        name: "piercing_skillshot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: explicit_cast_targeting(SkillTarget::EnemySingle {
            rule: UnitTargetRule::LowestHealthEnemy,
        }),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "piercing_shot".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::LowestHealthEnemy,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: false,
                max_range_tiles: Some(4),
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef {
                    piercing: true,
                    max_hits: Some(2),
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 15,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
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

    let event_log = run_battle_with_setup(
        game_data,
        vec![(Uuid::from_u128(71), caster_base_uuid, Position::new(0, 0))],
        vec![
            (Uuid::from_u128(72), first_base_uuid, Position::new(0, 1)),
            (Uuid::from_u128(73), second_base_uuid, Position::new(0, 2)),
            (Uuid::from_u128(74), third_base_uuid, Position::new(0, 3)),
        ],
        |core| {
            core.units
                .values_mut()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .expect("caster runtime unit should exist")
                .facing_direction = Some(FacingDirection::Down);
        },
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let first_id = spawned_unit_id(&event_log, first_base_uuid, Side::Opponent);
    let second_id = spawned_unit_id(&event_log, second_base_uuid, Side::Opponent);
    let third_id = spawned_unit_id(&event_log, third_base_uuid, Side::Opponent);
    let (ability_seq, _) = find_first_ability_cast_seq(&event_log, "piercing_skillshot", caster_id);
    let step = step_entries_for_cast(&event_log, ability_seq)
        .into_iter()
        .next()
        .expect("missing piercing step");

    let damage_targets: Vec<Uuid> = event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && entry_caused_by_seq(&event_log, entry, step.seq) =>
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
fn targetless_directional_collision_projectile_fires_along_caster_facing() {
    let caster_base_uuid = Uuid::from_u128(0xCC51);
    let enemy_base_uuid = Uuid::from_u128(0xCC52);

    let front_only_range = TileRangePattern {
        include_anchor_tile: false,
        rows: vec![".@.".to_string(), ".X.".to_string()],
    };
    let skill = SkillDef {
        id: SkillId::from("targetless_line_shot"),
        name: "targetless_line_shot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: SkillCastTargetingDef::explicit(
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            Default::default(),
            Some(front_only_range.clone()),
            false,
        ),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "line_shot".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: Some(front_only_range),
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: true,
                max_range_tiles: Some(4),
                max_lifetime_ms: None,
                max_kills: None,
                max_pierces: None,
                collision: SkillProjectileCollisionDef {
                    radius_units: 1_000_000,
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 15,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data_without_skill_range_override(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("targetless_line_shot"),
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
                100,
                0,
                1_000_000,
                8,
                DeliveryDef::Instant,
                100,
            ),
        ],
        vec![skill],
    );

    let event_log = run_battle_with_setup(
        game_data,
        vec![(Uuid::from_u128(75), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(76), enemy_base_uuid, Position::new(0, 3))],
        |core| {
            let caster_id = core
                .units
                .values_mut()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .expect("caster runtime unit should exist");
            caster_id.facing_direction = Some(FacingDirection::Down);
            let caster_instance_id = caster_id.instance_id;
            core.enqueue_event(BattleEvent::AutoCastStart {
                time_ms: 0,
                caster_instance_id,
                cause: BattleEventCause::Root {
                    kind: BattleEventRootCause::System,
                },
            });
        },
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&event_log, "targetless_line_shot", caster_id);
    let ability_entry = event_log
        .entries
        .iter()
        .find(|entry| entry.seq == ability_seq)
        .expect("missing ability entry");
    assert!(
        matches!(
            &ability_entry.event,
            BattleLogEvent::AbilityCast {
                target_instance_id: None,
                ..
            }
        ),
        "targetless directional projectile should not require a unit cast target"
    );
    let step = step_entries_for_cast(&event_log, ability_seq)
        .into_iter()
        .next()
        .expect("missing line shot step");

    let launch = event_log
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::SkillProjectileLaunched {
                    guidance: BattleProjectileGuidance::Fixed,
                    target: None,
                    ..
                }
            ) && entry_caused_by_seq(&event_log, entry, step.seq)
        })
        .expect("missing targetless fixed projectile launch");
    if let BattleLogEvent::SkillProjectileLaunched { start, aim, .. } = &launch.event {
        assert_eq!(start.x_milli, aim.x_milli);
        assert!(
            aim.y_milli > start.y_milli,
            "down-facing line projectile should aim toward increasing y"
        );
    }

    assert!(
        event_log.entries.iter().any(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason: HpChangeReason::Command,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && entry_caused_by_seq(&event_log, entry, step.seq)
            )
        }),
        "targetless directional projectile should damage the enemy along its facing line"
    );
}

#[test]
fn directional_collision_projectile_respects_max_kills_stop_policy() {
    let caster_base_uuid = Uuid::from_u128(0xCC61);
    let first_base_uuid = Uuid::from_u128(0xCC62);
    let second_base_uuid = Uuid::from_u128(0xCC63);

    let skill = SkillDef {
        id: SkillId::from("kill_limited_line_shot"),
        name: "kill_limited_line_shot".to_string(),
        kind: SkillKind::Untargeted,
        cast_targeting: explicit_nearest_enemy_cast_targeting(),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "kill_limited_line".to_string(),
            delay_ms: 0,
            range_policy: Default::default(),
            defense_tile_range: None,
            air_capable: false,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Projectile {
                speed_units_per_ms: 500_000,
                hit_policy: ProjectileHitPolicy::DirectionalCollision,
                allow_targetless_cast: false,
                max_range_tiles: Some(4),
                max_lifetime_ms: None,
                max_kills: Some(1),
                max_pierces: None,
                collision: SkillProjectileCollisionDef {
                    radius_units: 1_000_000,
                    piercing: true,
                    ..Default::default()
                },
            },
            effects: vec![SkillEffectDef::Damage {
                amount: 100,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
            presentation: SkillPresentationDef::default(),
        }],
    };

    let game_data = minimal_game_data(
        vec![
            make_abnormality(
                "caster",
                caster_base_uuid,
                Some("kill_limited_line_shot"),
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
                50,
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

    let event_log = run_battle_with_setup(
        game_data,
        vec![(Uuid::from_u128(77), caster_base_uuid, Position::new(0, 0))],
        vec![
            (Uuid::from_u128(78), first_base_uuid, Position::new(0, 1)),
            (Uuid::from_u128(79), second_base_uuid, Position::new(0, 2)),
        ],
        |core| {
            core.units
                .values_mut()
                .find(|unit| unit.owner == Side::Player && unit.base_uuid == caster_base_uuid)
                .expect("caster runtime unit should exist")
                .facing_direction = Some(FacingDirection::Down);
        },
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let first_id = spawned_unit_id(&event_log, first_base_uuid, Side::Opponent);
    let second_id = spawned_unit_id(&event_log, second_base_uuid, Side::Opponent);
    let (ability_seq, _) =
        find_first_ability_cast_seq(&event_log, "kill_limited_line_shot", caster_id);
    let step = step_entries_for_cast(&event_log, ability_seq)
        .into_iter()
        .next()
        .expect("missing kill-limited line step");

    let damage_targets: Vec<Uuid> = event_log
        .entries
        .iter()
        .filter_map(|entry| match &entry.event {
            BattleLogEvent::HpChanged {
                source_instance_id: Some(source_instance_id),
                target_instance_id,
                reason,
                ..
            } if *source_instance_id == caster_id.into()
                && *reason == HpChangeReason::Command
                && entry_caused_by_seq(&event_log, entry, step.seq) =>
            {
                Some((*target_instance_id).into())
            }
            _ => None,
        })
        .collect();

    assert_eq!(damage_targets, vec![first_id]);
    assert!(
        !damage_targets.contains(&second_id),
        "max_kills=1 should stop the piercing projectile after the first kill"
    );
}

#[test]
fn hit_gated_self_heal_and_buff_stack_repeat_attack_work_together() {
    let caster_base_uuid = Uuid::from_u128(0xBD11);
    let enemy_base_uuid = Uuid::from_u128(0xBD12);

    let skill = SkillDef {
        id: SkillId::from("predation_cycle"),
        name: "predation_cycle".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: explicit_cast_targeting(SkillTarget::SelfUnit),
        focus_time_ms: 100,
        focus_permissions: Default::default(),
        steps: vec![
            SkillStepDef {
                id: "prime_stacks".to_string(),
                delay_ms: 0,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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
                    SkillEffectDef::Damage {
                        amount: 5,
                        damage_type: game_core::game::battle::damage::DamageType::Magic,
                    },
                ],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "opening_strike".to_string(),
                delay_ms: 1,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::Nearest,
                },
                targeting: StepTargetingMode::RetargetOnStep,
                when: SkillStepCondition::Always,
                repeat: SkillStepRepeat::Once,
                delivery: DeliveryDef::Instant,
                effects: vec![SkillEffectDef::Damage {
                    amount: 12,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
                }],
                presentation: SkillPresentationDef::default(),
            },
            SkillStepDef {
                id: "heal_on_hit".to_string(),
                delay_ms: 2,
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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
                range_policy: Default::default(),
                defense_tile_range: None,
                air_capable: false,
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
                effects: vec![SkillEffectDef::Damage {
                    amount: 7,
                    damage_type: game_core::game::battle::damage::DamageType::Magic,
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

    let event_log = run_battle(
        game_data,
        vec![(Uuid::from_u128(31), caster_base_uuid, Position::new(0, 0))],
        vec![(Uuid::from_u128(32), enemy_base_uuid, Position::new(0, 1))],
    );

    let caster_id = spawned_unit_id(&event_log, caster_base_uuid, Side::Player);
    let enemy_id = spawned_unit_id(&event_log, enemy_base_uuid, Side::Opponent);
    let (ability_seq, _) = find_first_ability_cast_seq(&event_log, "predation_cycle", caster_id);
    let steps = step_entries_for_cast(&event_log, ability_seq);
    assert_eq!(steps.len(), 4, "expected all 4 steps to execute");

    let heal_step = steps[2];
    let barrage_step = steps[3];

    let heal_events = event_log
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::HpChanged {
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
            ) && entry_caused_by_seq(&event_log, entry, heal_step.seq)
        })
        .count();
    assert_eq!(
        heal_events, 1,
        "expected a single self-heal after the opening hit"
    );

    let barrage_hits = event_log
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                &entry.event,
                BattleLogEvent::HpChanged {
                    source_instance_id: Some(source_instance_id),
                    target_instance_id,
                    reason,
                    ..
                } if *source_instance_id == caster_id.into()
                    && *target_instance_id == enemy_id.into()
                    && *reason == HpChangeReason::Command
            ) && entry_caused_by_seq(&event_log, entry, barrage_step.seq)
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
    for abnormality in &mut abnormalities {
        abnormality.basic_attack.defense_tile_range = Some(broad_defense_tile_range());
    }
    abnormalities.push(make_abnormality(
        "skill_test_dummy",
        training_dummy_uuid,
        None,
        1,
        5_000,
        0,
        1_000_000,
        1,
        DeliveryDef::Instant,
        100,
    ));

    let game_data = GameDataBuilder::live_defaults()
        .with_abnormalities(abnormalities)
        .with_corroded_employee_data(Arc::clone(&base_game_data.corroded_employee_data))
        .with_corroded_wave_data(Arc::clone(&base_game_data.corroded_wave_data))
        .with_artifact_data(Arc::clone(&base_game_data.artifact_data))
        .with_equipment_data(Arc::clone(&base_game_data.equipment_data))
        .with_shop_data(Arc::clone(&base_game_data.shop_data))
        .with_reward_data(Arc::clone(&base_game_data.reward_data))
        .with_pve_data(Arc::clone(&base_game_data.pve_data))
        .with_skills(SkillDatabase::new(
            base_game_data
                .skill_data
                .skills
                .iter()
                .cloned()
                .map(skill_with_broad_defense_tile_range)
                .collect(),
        ))
        .with_skill_fragment_data(Arc::clone(&base_game_data.skill_fragment_data))
        .build_arc();

    let cases = [
        SkillCase {
            abnormality_id: "o-01-04_queen_of_hatred",
            skill_id: "queen_of_hatred_magical_beam",
            expected_steps: 2,
            expected_buff: None,
            expect_triggered_attacks: 0,
            expect_positive_command_hp_change: false,
            expect_negative_command_hp_change: false,
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

        let event_log = run_battle(
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

        let caster_id = spawned_unit_id(&event_log, tested_abnormality.uuid, Side::Player);
        let (ability_seq, _) = find_first_ability_cast_seq(&event_log, case.skill_id, caster_id);
        let steps = step_entries_for_cast(&event_log, ability_seq);
        assert_eq!(
            steps.len(),
            case.expected_steps,
            "unexpected step count for {}",
            case.abnormality_id
        );

        let step_seqs: Vec<u64> = steps.iter().map(|entry| entry.seq).collect();
        let caused_by_steps = |entry: &game_core::game::battle::event_log::BattleEventLogEntry| {
            step_seqs
                .iter()
                .any(|step_seq| entry_caused_by_seq(&event_log, entry, *step_seq))
        };

        if let Some(buff_name) = case.expected_buff {
            let expected_buff_id = BuffId::from_name(buff_name);
            assert!(
                event_log.entries.iter().any(|entry| {
                    matches!(&entry.event, BattleLogEvent::BuffApplied { caster_instance_id, buff_id, .. }
                        if *caster_instance_id == caster_id.into() && *buff_id == expected_buff_id)
                        && caused_by_steps(entry)
                }),
                "expected BuffApplied({buff_name}) for {}",
                case.abnormality_id
            );
        }

        if case.expect_triggered_attacks > 0 {
            let triggered_attacks = event_log
                .entries
                .iter()
                .filter(|entry| {
                    matches!(
                        &entry.event,
                        BattleLogEvent::AttackStart {
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
                event_log.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        BattleLogEvent::HpChanged {
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
                event_log.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        BattleLogEvent::HpChanged {
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
                event_log.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        BattleLogEvent::StatChanged {
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
                event_log.entries.iter().any(|entry| {
                    matches!(
                        &entry.event,
                        BattleLogEvent::ResonanceChanged { unit_instance_id, .. }
                        if *unit_instance_id == caster_id.into()
                    ) && caused_by_steps(entry)
                }),
                "expected ResonanceChanged for {}",
                case.abnormality_id
            );
        }
    }
}
