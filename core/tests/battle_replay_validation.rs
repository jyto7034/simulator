mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::{
    DeliveryDef, SkillDef, SkillEffectDef, SkillKind, SkillPresentationDef, SkillStepDef,
    SkillTarget, UnitTargetRule,
};
use game_core::game::battle::buffs::BuffId;
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::replay::{types::TimelineReplayerConfig, TimelineReplayer};
use game_core::game::battle::timeline::{
    HpChangeReason, TimelineCause, TimelineEvent, TimelineRootCause,
};
use game_core::game::battle::types::{OwnedUnit, PlayerDeckInfo};
use game_core::game::battle::validation::{
    TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig, TimelineViolationKind,
};
use game_core::game::data::{
    abnormality_data::{AbnormalityDatabase, AbnormalityMetadata, BasicAttackDef, ResonanceDef},
    artifact_data::ArtifactDatabase,
    bonus_data::BonusDatabase,
    equipment_data::EquipmentDatabase,
    event_pools::{EventPhasePool, EventPoolConfig},
    pve_data::PveEncounterDatabase,
    random_event_data::RandomEventDatabase,
    shop_data::ShopDatabase,
    skill_data::SkillDatabase,
    GameDataBase,
};
use game_core::game::enums::{RiskLevel, Tier};
use game_core::game::growth::GrowthStack;
use uuid::Uuid;

fn empty_event_pools() -> EventPoolConfig {
    let pool = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };
    EventPoolConfig {
        dawn: pool.clone(),
        noon: pool.clone(),
        dusk: pool.clone(),
        midnight: pool.clone(),
        white: pool,
    }
}

fn deck_single_unit(owned_uuid: Uuid, base_uuid: Uuid, pos: Position) -> PlayerDeckInfo {
    let mut positions = HashMap::new();
    positions.insert(owned_uuid, pos);
    PlayerDeckInfo {
        units: vec![OwnedUnit {
            owned_uuid,
            base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions,
    }
}

#[test]
fn battle_timeline_replays_and_validates() {
    // Given: 테스트용 GameData와 1:1 전투 덱을 만든다.
    let game_data = common::create_test_game_data();

    let base_uuid = game_data.abnormality_data.items[0].uuid;
    let player_unit = Uuid::from_u128(0xAA01);
    let opponent_unit = Uuid::from_u128(0xBB01);

    // Given: 시작 위치를 멀리 두어 이동 이벤트도 생성되게 한다.
    let player = deck_single_unit(player_unit, base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(opponent_unit, base_uuid, Position::new(3, 3));

    // When: 전투를 실행해서 서버-권위 타임라인을 생성한다.
    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        12345,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export("battle_timeline_replays_and_validates", &result.timeline);

    // Then: Replay(원인/결과 관계)와 Validation(정합성) 모두 통과해야 한다.
    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    let expected = TimelineExpectedCounts::from_decks(&player, &opponent);
    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&result.timeline, Some(expected), Some(game_data.as_ref()))
        .unwrap();
}

#[test]
fn battle_timeline_with_autocast_and_buff_tick_replays_and_validates() {
    // Given: 오토캐스트 + 버프틱이 반드시 발생하는 결정적 스펙을 구성한다.
    // - 기본 공격은 데미지를 거의 주지 않도록(방어력 매우 큼)
    // - 공명 max=20(기본 공격 2번이면 가득 참) → 오토캐스트 트리거
    // - 스킬은 적 1명에게 poison 버프(주기 피해 2) 부여
    // - 타겟 HP=2 → 첫 poison tick에서 사망(전투가 짧게 끝남)
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
        movement: Default::default(),
        basic_attack: BasicAttackDef {
            range_tiles: 1,
            interval_ms: 300,
            windup_ms: 0,
            delivery: DeliveryDef::Instant,
        },
        resonance: ResonanceDef {
            start: 0,
            max: 20,
            gain_lock_ms: 0,
        },
        skill_id: Some("poison_skill".to_string()),
    };

    let target = AbnormalityMetadata {
        id: "target".to_string(),
        uuid: target_base_uuid,
        name: "Target".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        // NOTE: 기본 공격(최소 1 데미지) + poison tick(2 데미지)이 발생해도
        // tick 시점까지 살아있도록 테스트 전용 HP를 충분히 준다.
        max_health: 7,
        attack: 1,
        defense: 9999,
        movement: Default::default(),
        basic_attack: BasicAttackDef {
            range_tiles: 1,
            interval_ms: 300,
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
        id: "poison_skill".to_string(),
        name: "poison_skill".to_string(),
        kind: SkillKind::Targeted,
        focus_time_ms: 200,
        focus_permissions: Default::default(),
        steps: vec![SkillStepDef {
            id: "step_01".to_string(),
            delay_ms: 0,
            range_tiles: 1,
            target: SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest,
            },
            delivery: DeliveryDef::Instant,
            effects: vec![SkillEffectDef::ApplyBuff {
                buff_id: "poison".to_string(),
                duration_ms: 5_000,
            }],
            presentation: SkillPresentationDef::default(),
        }],
    }]);

    let game_data = Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(vec![caster, target])),
        Arc::new(ArtifactDatabase::new(vec![])),
        Arc::new(EquipmentDatabase::new(vec![])),
        Arc::new(ShopDatabase::new(vec![])),
        Arc::new(BonusDatabase::new(vec![])),
        Arc::new(RandomEventDatabase::new(vec![])),
        Arc::new(PveEncounterDatabase::new(vec![])),
        Arc::new(skills),
        empty_event_pools(),
    ));

    let player_owned = Uuid::from_u128(0xDADA_0001);
    let opponent_owned = Uuid::from_u128(0xDADA_0002);
    let player = deck_single_unit(player_owned, caster_base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(opponent_owned, target_base_uuid, Position::new(0, 1));

    // When: 전투를 실행한다.
    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        999,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "battle_timeline_with_autocast_and_buff_tick",
        &result.timeline,
    );

    let poison_id = BuffId::from_name("poison");

    let caster_unit_id = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                base_uuid,
                ..
            } if *base_uuid == caster_base_uuid => Some(*unit_instance_id),
            _ => None,
        })
        .expect("missing caster spawn");

    let target_unit_id = result
        .timeline
        .entries
        .iter()
        .find_map(|entry| match &entry.event {
            TimelineEvent::UnitSpawned {
                unit_instance_id,
                base_uuid,
                ..
            } if *base_uuid == target_base_uuid => Some(*unit_instance_id),
            _ => None,
        })
        .expect("missing target spawn");

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
                } if caster_instance_id == caster_unit_id
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
                    && actual_caster == caster_unit_id
                    && actual_target == target_unit_id
            )
        })
        .expect("AbilityCast(poison_skill)가 있어야 한다");

    assert_eq!(
        ability_cast.cause,
        TimelineCause::Parent {
            seq: autocast_start.seq
        },
        "AbilityCast는 AutoCastStart의 자식이어야 한다"
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
                } if actual_caster == caster_unit_id
                    && actual_target == target_unit_id
                    && buff_id == poison_id
            )
        })
        .expect("poison BuffApplied가 있어야 한다");

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
                    && *caster_instance_id == caster_unit_id
                    && *target_instance_id == Some(target_unit_id)
            )
        })
        .expect("poison AbilityStepTriggered가 있어야 한다");

    assert_eq!(
        buff_applied.cause,
        TimelineCause::Parent {
            seq: step_triggered.seq
        },
        "BuffApplied는 AbilityStepTriggered의 자식이어야 한다"
    );

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
                } if actual_caster == caster_unit_id
                    && actual_target == target_unit_id
                    && buff_id == poison_id
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
        },
        "첫 BuffTick은 BuffApplied의 자식이어야 한다"
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
                } if actual_source == caster_unit_id && actual_target == target_unit_id
            )
        })
        .expect("poison tick으로 인한 HpChanged가 있어야 한다");

    assert_eq!(
        tick_damage.cause,
        TimelineCause::Parent {
            seq: buff_ticks[0].seq
        },
        "버프 틱 데미지는 BuffTick의 자식이어야 한다"
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
                } if actual_target == target_unit_id && actual_killer == caster_unit_id
            )
        })
        .expect("poison tick으로 타겟이 사망해야 한다");

    assert_eq!(
        unit_died.cause,
        TimelineCause::Parent {
            seq: buff_ticks[0].seq
        },
        "UnitDied는 치명적인 BuffTick의 자식이어야 한다"
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
                } if actual_caster == caster_unit_id
            )
        })
        .expect("AutoCastEnd가 있어야 한다");

    assert_eq!(
        autocast_end.cause,
        TimelineCause::Parent {
            seq: autocast_start.seq
        },
        "AutoCastEnd는 같은 AutoCastStart의 자식이어야 한다"
    );

    // Then: Replay/Validation이 모두 통과해야 한다.
    let mut replay_config = TimelineReplayerConfig::default();
    replay_config.validate_unit_base_uuid = true;
    TimelineReplayer::new(game_data.clone(), replay_config)
        .replay(&result.timeline)
        .unwrap();

    let expected = TimelineExpectedCounts::from_decks(&player, &opponent);
    TimelineValidator::new(TimelineValidatorConfig::default())
        .validate(&result.timeline, Some(expected), Some(game_data.as_ref()))
        .unwrap();
}

#[test]
fn tampered_timeline_missing_parent_on_hp_changed_is_rejected_by_validation() {
    // Given: 정상 전투 타임라인을 만든다.
    let game_data = common::create_test_game_data();

    let base_uuid = game_data.abnormality_data.items[0].uuid;
    let player = deck_single_unit(Uuid::from_u128(1), base_uuid, Position::new(0, 0));
    let opponent = deck_single_unit(Uuid::from_u128(2), base_uuid, Position::new(3, 3));

    let mut battle = BattleCore::new(
        &player,
        &opponent,
        game_data.clone(),
        common::BOARD_SIZE,
        12345,
    );
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

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
    let expected = TimelineExpectedCounts::from_decks(&player, &opponent);
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
