mod common;

use std::collections::HashMap;
use std::sync::Arc;

use game_core::game::ability::{
    AbilityActivationBinding, AbilityActivationDef, DeliveryDef, SkillCastTargetingDef, SkillDef,
    SkillEffectDef, SkillId, SkillKind, SkillPresentationDef, SkillStepDef, SkillTarget,
    StepTargetingMode, UnitTargetRule,
};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::replay::{
    types::{TimelineReplayViolationKind, TimelineReplayerConfig},
    TimelineReplayer,
};
use game_core::game::battle::scenario::{
    BattleFieldSpec, BattleScenario, ScenarioAction, ScenarioArtifact, ScenarioEvent,
    ScenarioEventId, ScenarioGroupId, ScenarioSpawnGroup, ScenarioTrigger, ScenarioUnitRef,
    ScenarioUnitSpawn, WinCondition,
};
use game_core::game::battle::timeline::{
    HpChangeReason, TimelineCause, TimelineEvent, TimelineRootCause,
};
use game_core::game::battle::types::{BattleUnitDraft, BattleUnitSource};
use game_core::game::battle::validation::{
    TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig, TimelineViolationKind,
};
use game_core::game::data::{
    abnormality_data::{AbnormalityMetadata, BasicAttackDef, ResonanceDef},
    artifact_data::ArtifactMetadata,
    equipment_data::{EquipmentMetadata, EquipmentType},
    skill_data::SkillDatabase,
    GameDataBase, GameDataBuilder,
};
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::growth::GrowthStack;
use game_core::game::resources::Position;
use game_core::game::stats::TriggerType;
use uuid::Uuid;

fn unit_draft(owned_uuid: Uuid, base_uuid: Uuid) -> BattleUnitDraft {
    BattleUnitDraft {
        owned_uuid,
        source: BattleUnitSource::Abnormality { base_uuid },
        level: Tier::I,
        growth_stacks: GrowthStack::new(),
        equipped_items: vec![],
        equipped_item_enhancements: vec![],
    }
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
    artifacts: Vec<ScenarioArtifact>,
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
        artifacts,
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

fn one_vs_one_scenario(
    player_owned_uuid: Uuid,
    player_base_uuid: Uuid,
    player_position: Position,
    opponent_owned_uuid: Uuid,
    opponent_base_uuid: Uuid,
    opponent_position: Position,
) -> BattleScenario {
    battle_scenario(
        vec![(player_owned_uuid, player_base_uuid, player_position)],
        vec![(opponent_owned_uuid, opponent_base_uuid, opponent_position)],
        Vec::new(),
    )
}

struct PoisonAutocastScenario {
    caster_attack_interval_ms: u64,
    caster_resonance_max: u32,
    caster_resonance_gain_lock_ms: u64,
    target_attack_interval_ms: u64,
    target_max_health: u32,
}

struct PoisonAutocastResult {
    game_data: Arc<GameDataBase>,
    expected_counts: TimelineExpectedCounts,
    timeline: game_core::game::battle::timeline::Timeline,
    caster_base_uuid: Uuid,
    target_base_uuid: Uuid,
    poison_id: BuffId,
}

fn run_poison_autocast_scenario(spec: PoisonAutocastScenario) -> PoisonAutocastResult {
    let caster_base_uuid = Uuid::from_u128(0xC0A5_7E01);
    let target_base_uuid = Uuid::from_u128(0xC0A5_7E02);

    let caster = AbnormalityMetadata {
        id: "caster".to_string(),
        uuid: caster_base_uuid,
        name: "Caster".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 9999,
        attack: 1,
        defense: 9999,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: BasicAttackDef {
            range_units: 1.0,
            interval_ms: spec.caster_attack_interval_ms,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
        resonance: ResonanceDef {
            start: 0,
            max: spec.caster_resonance_max,
            gain_lock_ms: spec.caster_resonance_gain_lock_ms,
        },
        skill_id: Some(SkillId::from("poison_skill")),
    };

    let target = AbnormalityMetadata {
        id: "target".to_string(),
        uuid: target_base_uuid,
        name: "Target".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: spec.target_max_health,
        attack: 1,
        defense: 9999,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: BasicAttackDef {
            range_units: 1.0,
            interval_ms: spec.target_attack_interval_ms,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
        resonance: ResonanceDef {
            start: 0,
            max: 100,
            gain_lock_ms: 0,
        },
        skill_id: None,
    };

    let skills = SkillDatabase::new(vec![SkillDef {
        id: SkillId::from("poison_skill"),
        name: "poison_skill".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::FirstStepTarget,
        focus_time_ms: 200,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
            range_units: 1.0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::ApplyBuff {
                buff_id: "poison".to_string(),
                duration_ms: 5_000,
            }],
            presentation: SkillPresentationDef::default(),
        }],
    }]);

    let game_data = GameDataBuilder::empty()
        .with_abnormalities(vec![caster, target])
        .with_skills(skills)
        .build_arc();

    let scenario = one_vs_one_scenario(
        Uuid::from_u128(0xDADA_0001),
        caster_base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(0xDADA_0002),
        target_base_uuid,
        Position::new(0, 1),
    );
    let expected_counts = TimelineExpectedCounts::from_scenario(&scenario);

    let mut battle = BattleCore::new_from_scenario(scenario, game_data.clone(), 999);
    let result = battle.run_battle().unwrap();

    PoisonAutocastResult {
        game_data,
        expected_counts,
        timeline: result.timeline,
        caster_base_uuid,
        target_base_uuid,
        poison_id: BuffId::from_name("poison"),
    }
}

fn find_spawned_unit_id(
    timeline: &game_core::game::battle::timeline::Timeline,
    base_uuid: Uuid,
    owner: Side,
) -> Uuid {
    timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                base_uuid: actual_base_uuid,
                owner: actual_owner,
                ..
            } if *actual_base_uuid == base_uuid && *actual_owner == owner => {
                Some(unit_instance_id.as_uuid())
            }
            _ => None,
        })
        .expect("missing spawned unit")
}

#[test]
fn on_battle_start_triggered_ability_is_replayable_and_parented_via_proc_event() {
    let caster_base_uuid = Uuid::from_u128(0x5100);
    let target_base_uuid = Uuid::from_u128(0x5200);
    let artifact_uuid = Uuid::from_u128(0x5300);

    let caster = AbnormalityMetadata {
        id: "caster".to_string(),
        uuid: caster_base_uuid,
        name: "Caster".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 100,
        attack: 10,
        defense: 0,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
    };
    let target = AbnormalityMetadata {
        id: "target".to_string(),
        uuid: target_base_uuid,
        name: "Target".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 100,
        attack: 1,
        defense: 0,
        magic_resist: 0,
        movement: Default::default(),
        basic_attack: BasicAttackDef::default(),
        resonance: ResonanceDef::default(),
        skill_id: None,
    };

    let artifact = ArtifactMetadata {
        id: "opening_artifact".to_string(),
        uuid: artifact_uuid,
        name: "Opening Artifact".to_string(),
        description: "proc".to_string(),
        rarity: RiskLevel::ZAYIN,
        price: 0,
        triggered_effects: HashMap::new(),
        ability_activations: vec![AbilityActivationBinding {
            ability_id: SkillId::from("opening_proc"),
            activation: AbilityActivationDef::TriggerProc {
                trigger: TriggerType::OnBattleStart,
                proc_chance_percent: 100,
                internal_cooldown_ms: 0,
                max_triggers_per_battle: Some(1),
            },
        }],
    };

    let skills = SkillDatabase::new(vec![SkillDef {
        id: SkillId::from("opening_proc"),
        name: "Opening Proc".to_string(),
        kind: SkillKind::Targeted,
        cast_targeting: SkillCastTargetingDef::Explicit {
            range_units: 3.0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
        },
        focus_time_ms: 0,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "hit".to_string(),
            delay_ms: 0,
            range_units: 3.0,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            targeting: StepTargetingMode::ReuseCastTarget,
            when: Default::default(),
            repeat: Default::default(),
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::Damage {
                amount: 7,
                damage_type: game_core::game::battle::damage::DamageType::Magic,
            }],
            presentation: SkillPresentationDef::default(),
        }],
    }]);

    let game_data = GameDataBuilder::empty()
        .with_abnormalities(vec![caster, target])
        .with_artifacts(vec![artifact])
        .with_equipment(vec![EquipmentMetadata {
            id: "noop".to_string(),
            uuid: Uuid::from_u128(0x5400),
            name: "noop".to_string(),
            equipment_type: EquipmentType::Weapon,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            triggered_effects: HashMap::new(),
            ability_activations: vec![],
        }])
        .with_skills(skills)
        .build_arc();

    let scenario = battle_scenario(
        vec![(
            Uuid::from_u128(0x5501),
            caster_base_uuid,
            Position::new(0, 0),
        )],
        vec![(
            Uuid::from_u128(0x5502),
            target_base_uuid,
            Position::new(0, 1),
        )],
        vec![ScenarioArtifact {
            side: Side::Player,
            base_uuid: artifact_uuid,
            instance_salt: 0,
        }],
    );

    let expected = TimelineExpectedCounts::from_scenario(&scenario);
    let mut battle = BattleCore::new_from_scenario(scenario, game_data.clone(), 7);
    let result = battle.run_battle().unwrap();

    let proc_entry = result
        .timeline
        .entries
        .iter()
        .find(|entry| matches!(entry.event, TimelineEvent::TriggeredAbilityProc { .. }))
        .expect("missing TriggeredAbilityProc");
    let ability_cast = result
        .timeline
        .entries
        .iter()
        .find(|entry| matches!(entry.event, TimelineEvent::AbilityCast { ref skill_id, .. } if skill_id == "opening_proc"))
        .expect("missing opening_proc AbilityCast");

    assert_eq!(
        ability_cast.cause,
        TimelineCause::Parent {
            seq: proc_entry.seq
        }
    );

    let replay = TimelineReplayer::new(game_data.clone(), TimelineReplayerConfig::default());
    replay
        .replay(&result.timeline)
        .expect("timeline should replay");

    let validator = TimelineValidator::new(TimelineValidatorConfig::default());
    assert!(validator
        .validate(&result.timeline, Some(expected), None)
        .is_ok());
}

#[test]
fn battle_timeline_replays_and_validates() {
    // Given: 테스트용 GameData와 1:1 전투 시나리오를 만든다.
    let game_data = common::create_test_game_data();

    let base_uuid = game_data.abnormality_data.items[0].uuid;
    let player_unit = Uuid::from_u128(0xAA01);
    let opponent_unit = Uuid::from_u128(0xBB01);

    // Given: 시작 위치를 멀리 두어 이동 이벤트도 생성되게 한다.
    let scenario = one_vs_one_scenario(
        player_unit,
        base_uuid,
        Position::new(0, 0),
        opponent_unit,
        base_uuid,
        Position::new(3, 3),
    );
    let expected = TimelineExpectedCounts::from_scenario(&scenario);

    // When: 전투를 실행해서 서버-권위 타임라인을 생성한다.
    let mut battle = BattleCore::new_from_scenario(scenario, game_data.clone(), 12345);
    let result = battle.run_battle().unwrap();

    common::write_timeline_export("battle_timeline_replays_and_validates", &result.timeline);

    // Then: Replay(원인/결과 관계)와 Validation(정합성) 모두 통과해야 한다.
    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&result.timeline, Some(expected), Some(game_data.as_ref()))
        .unwrap();
}

#[test]
fn battle_timeline_with_autocast_and_buff_tick_replays_and_validates() {
    let result = run_poison_autocast_scenario(PoisonAutocastScenario {
        caster_attack_interval_ms: 1_500,
        caster_resonance_max: 10,
        caster_resonance_gain_lock_ms: 5_000,
        target_attack_interval_ms: 5_000,
        target_max_health: 3,
    });

    common::write_timeline_export(
        "battle_timeline_with_autocast_and_buff_tick",
        &result.timeline,
    );

    assert!(result
        .timeline
        .entries
        .iter()
        .any(|entry| { matches!(entry.event, TimelineEvent::AutoCastStart { .. }) }));
    assert!(result
        .timeline
        .entries
        .iter()
        .any(|entry| { matches!(entry.event, TimelineEvent::BuffTick { .. }) }));

    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(result.game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(
            &result.timeline,
            Some(result.expected_counts),
            Some(result.game_data.as_ref()),
        )
        .unwrap();
}

#[test]
fn autocast_poison_skill_records_expected_parent_chain() {
    let result = run_poison_autocast_scenario(PoisonAutocastScenario {
        caster_attack_interval_ms: 300,
        caster_resonance_max: 20,
        caster_resonance_gain_lock_ms: 0,
        target_attack_interval_ms: 700,
        target_max_health: 20,
    });

    let caster_unit_id =
        find_spawned_unit_id(&result.timeline, result.caster_base_uuid, Side::Player);
    let target_unit_id =
        find_spawned_unit_id(&result.timeline, result.target_base_uuid, Side::Opponent);

    let autocast_start = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::AutoCastStart {
                    caster_instance_id,
                    ..
                } if caster_instance_id.as_uuid() == caster_unit_id
            )
        })
        .expect("오토캐스트 시작 이벤트가 있어야 한다");

    let ability_cast = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::AbilityCast {
                    ref skill_id,
                    caster_instance_id: actual_caster,
                    target_instance_id: Some(actual_target),
                } if skill_id == "poison_skill"
                    && actual_caster.as_uuid() == caster_unit_id
                    && actual_target.as_uuid() == target_unit_id
            )
        })
        .expect("AbilityCast(poison_skill)가 있어야 한다");

    assert_eq!(
        ability_cast.cause,
        TimelineCause::Parent {
            seq: autocast_start.seq
        }
    );

    let step_triggered = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                &entry.event,
                TimelineEvent::AbilityStepTriggered {
                    skill_id,
                    caster_instance_id,
                    target_instance_id,
                    ..
                } if skill_id == "poison_skill"
                    && caster_instance_id.as_uuid() == caster_unit_id
                    && *target_instance_id == Some(target_unit_id.into())
            )
        })
        .expect("poison AbilityStepTriggered가 있어야 한다");

    assert_eq!(
        step_triggered.cause,
        TimelineCause::Parent {
            seq: ability_cast.seq
        }
    );

    let buff_applied = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::BuffApplied {
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                    buff_id,
                    ..
                } if actual_caster.as_uuid() == caster_unit_id
                    && actual_target.as_uuid() == target_unit_id
                    && buff_id == result.poison_id
            )
        })
        .expect("poison BuffApplied가 있어야 한다");

    assert_eq!(
        buff_applied.cause,
        TimelineCause::Parent {
            seq: step_triggered.seq
        }
    );

    let autocast_end = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::AutoCastEnd {
                    caster_instance_id: actual_caster,
                } if actual_caster.as_uuid() == caster_unit_id
            )
        })
        .expect("AutoCastEnd가 있어야 한다");

    assert_eq!(
        autocast_end.cause,
        TimelineCause::Parent {
            seq: autocast_start.seq
        }
    );
}

#[test]
fn poison_buff_tick_and_fatal_death_record_expected_parent_chain() {
    let result = run_poison_autocast_scenario(PoisonAutocastScenario {
        caster_attack_interval_ms: 1_500,
        caster_resonance_max: 10,
        caster_resonance_gain_lock_ms: 5_000,
        target_attack_interval_ms: 5_000,
        target_max_health: 3,
    });

    let caster_unit_id =
        find_spawned_unit_id(&result.timeline, result.caster_base_uuid, Side::Player);
    let target_unit_id =
        find_spawned_unit_id(&result.timeline, result.target_base_uuid, Side::Opponent);

    let buff_applied = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::BuffApplied {
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                    buff_id,
                    ..
                } if actual_caster.as_uuid() == caster_unit_id
                    && actual_target.as_uuid() == target_unit_id
                    && buff_id == result.poison_id
            )
        })
        .expect("poison BuffApplied가 있어야 한다");

    let buff_ticks: Vec<_> = result
        .timeline
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.event,
                TimelineEvent::BuffTick {
                    caster_instance_id: actual_caster,
                    target_instance_id: actual_target,
                    buff_id,
                } if actual_caster.as_uuid() == caster_unit_id
                    && actual_target.as_uuid() == target_unit_id
                    && buff_id == result.poison_id
            )
        })
        .collect();
    assert_eq!(
        buff_ticks.len(),
        1,
        "poison buff tick은 1회만 발생해야 한다"
    );
    assert_eq!(
        buff_ticks[0].cause,
        TimelineCause::Parent {
            seq: buff_applied.seq
        }
    );

    let tick_damage = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::HpChanged {
                    source_instance_id: Some(actual_source),
                    target_instance_id: actual_target,
                    reason: HpChangeReason::Command,
                    ..
                } if actual_source.as_uuid() == caster_unit_id
                    && actual_target.as_uuid() == target_unit_id
            )
        })
        .expect("poison tick으로 인한 HpChanged가 있어야 한다");

    assert_eq!(
        tick_damage.cause,
        TimelineCause::Parent {
            seq: buff_ticks[0].seq
        }
    );

    let unit_died = result
        .timeline
        .entries
        .iter()
        .find(|entry| {
            matches!(
                entry.event,
                TimelineEvent::UnitDied {
                    unit_instance_id: actual_target,
                    killer_instance_id: Some(actual_killer),
                    ..
                } if actual_target.as_uuid() == target_unit_id
                    && actual_killer.as_uuid() == caster_unit_id
            )
        })
        .expect("poison tick으로 타겟이 사망해야 한다");

    assert_eq!(
        unit_died.cause,
        TimelineCause::Parent {
            seq: buff_ticks[0].seq
        }
    );
}

#[test]
fn tampered_timeline_missing_parent_on_hp_changed_is_rejected_by_validation() {
    // Given: 정상 전투 타임라인을 만든다.
    let game_data = common::create_test_game_data();

    let base_uuid = game_data.abnormality_data.items[0].uuid;
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(1),
        base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(2),
        base_uuid,
        Position::new(1, 0),
    );
    let expected = TimelineExpectedCounts::from_scenario(&scenario);

    let mut battle = BattleCore::new_from_scenario(scenario, game_data.clone(), 12345);
    let result = battle.run_battle().unwrap();

    common::write_timeline_export("tampered_timeline_original", &result.timeline);

    // Given: HpChanged 하나를 찾아 cause를 Root로 변조한다(원인 이벤트 끊기).
    let mut tampered = result.timeline.clone();
    let hp_index = tampered
        .entries
        .iter()
        .position(|e| matches!(e.event, TimelineEvent::HpChanged { .. }))
        .expect("battle timeline should contain HpChanged");
    tampered.entries[hp_index].cause = TimelineCause::Root {
        kind: TimelineRootCause::System,
    };

    common::write_timeline_export("tampered_timeline_missing_parent", &tampered);

    // When: Validation을 수행한다.
    let err = TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&tampered, Some(expected), Some(game_data.as_ref()))
        .unwrap_err();

    // Then: OutcomeMissingParent 위반이 반드시 포함되어야 한다.
    assert!(
        err.iter()
            .any(|v| v.kind == TimelineViolationKind::OutcomeMissingParent),
        "HpChanged는 기본적으로 Parent cause가 강제되어야 한다"
    );
}

#[test]
fn tampered_timeline_duplicate_seq_is_rejected_by_replay() {
    let game_data = common::create_test_game_data();

    let base_uuid = game_data.abnormality_data.items[0].uuid;
    let scenario = one_vs_one_scenario(
        Uuid::from_u128(11),
        base_uuid,
        Position::new(0, 0),
        Uuid::from_u128(12),
        base_uuid,
        Position::new(3, 3),
    );
    let mut battle = BattleCore::new_from_scenario(scenario, game_data.clone(), 12345);
    let result = battle.run_battle().unwrap();

    let mut tampered = result.timeline.clone();
    tampered.entries[1].seq = tampered.entries[0].seq;

    let err = TimelineReplayer::new(game_data, TimelineReplayerConfig::default())
        .replay(&tampered)
        .unwrap_err();

    assert!(
        err.iter()
            .any(|v| v.kind == TimelineReplayViolationKind::DuplicateSeq),
        "replay should reject duplicate seq before resolving parent/cause links"
    );
}
