mod common;

use std::collections::HashMap;
use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::{
    DeliveryDef, SkillDef, SkillEffectDef, SkillKind, SkillTarget, UnitTargetRule,
};
use game_core::game::battle::core::BattleCore;
use game_core::game::battle::replay::{types::TimelineReplayerConfig, TimelineReplayer};
use game_core::game::battle::timeline::{TimelineCause, TimelineEvent, TimelineRootCause};
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
    let mut battle = BattleCore::new(&player, &opponent, game_data.clone(), common::BOARD_SIZE, 12345);
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
        kind: SkillKind::Targeted,
        target: SkillTarget::EnemySingle {
            rule: UnitTargetRule::Nearest,
        },
        range_tiles: 1,
        cast_delay_ms: 0,
        focus_time_ms: 200,
        focus_permissions: Default::default(),
        delivery: DeliveryDef::Instant,
        effects: vec![SkillEffectDef::ApplyBuff {
            buff_id: "poison".to_string(),
            duration_ms: 5_000,
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
    let mut battle = BattleCore::new(&player, &opponent, game_data.clone(), common::BOARD_SIZE, 999);
    let mut world = World::new();
    let result = battle.run_battle(&mut world).unwrap();

    common::write_timeline_export(
        "battle_timeline_with_autocast_and_buff_tick",
        &result.timeline,
    );

    // Then: 최소 1회 오토캐스트/버프틱이 타임라인에 존재해야 한다.
    let has_autocast = result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::AutoCastStart { .. }));
    assert!(has_autocast, "오토캐스트가 최소 1회 발생해야 한다");

    // TODO: Add assertions for same-buff re-apply semantics (stack vs refresh, tick parent linkage).

    let has_buff_tick = result
        .timeline
        .entries
        .iter()
        .any(|e| matches!(e.event, TimelineEvent::BuffTick { .. }));
    assert!(has_buff_tick, "poison 버프틱이 최소 1회 발생해야 한다");

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

    let mut battle = BattleCore::new(&player, &opponent, game_data.clone(), common::BOARD_SIZE, 12345);
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
