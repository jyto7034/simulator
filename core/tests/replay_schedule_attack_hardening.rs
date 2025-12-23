use std::{collections::HashMap, sync::Arc};

use game_core::ecs::resources::Position;
use game_core::game::battle::{
    replay::{TimelineReplayViolationKind, TimelineReplayer, TimelineReplayerConfig},
    timeline::AttackKind,
    BattleWinner, Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION,
};
use game_core::game::data::abnormality_data::AbnormalityDatabase;
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Side};
use game_core::game::stats::{Effect, TriggerType, TriggeredEffects, UnitStats};
use uuid::Uuid;

fn empty_event_pools() -> EventPoolConfig {
    let empty = EventPhasePool {
        shops: Vec::new(),
        bonuses: Vec::new(),
        random_events: Vec::new(),
    };
    EventPoolConfig {
        dawn: empty.clone(),
        noon: empty.clone(),
        dusk: empty.clone(),
        midnight: empty.clone(),
        white: empty,
    }
}

fn game_data_with_item(uuid: Uuid, triggered_effects: TriggeredEffects) -> Arc<GameDataBase> {
    let item = EquipmentMetadata {
        id: "test_item".to_string(),
        uuid,
        name: "test_item".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: true,
        triggered_effects,
    };

    Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(Vec::new())),
        Arc::new(ArtifactDatabase::new(Vec::new())),
        Arc::new(EquipmentDatabase::new(vec![item])),
        Arc::new(ShopDatabase::new(Vec::new())),
        Arc::new(BonusDatabase::new(Vec::new())),
        Arc::new(RandomEventDatabase::new(Vec::new())),
        Arc::new(PveEncounterDatabase::new(Vec::new())),
        empty_event_pools(),
    ))
}

fn spawn_stats() -> UnitStats {
    UnitStats::with_values(100, 100, 10, 0, 1000)
}

#[test]
fn replayer_requires_expected_triggered_attacks_from_extra_attack() {
    let attacker_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);
    let item_uuid = Uuid::from_u128(0x2000);

    let mut triggered_effects: TriggeredEffects = HashMap::new();
    triggered_effects.insert(
        TriggerType::OnAttack,
        vec![Effect::Ability(
            game_core::game::ability::AbilityId::RedShoesBerserk,
        )],
    );

    let game_data = game_data_with_item(item_uuid, triggered_effects);
    let replayer = TimelineReplayer::new(game_data, TimelineReplayerConfig::default());

    let timeline = Timeline {
        version: TIMELINE_VERSION,
        entries: vec![
            TimelineEntry {
                time_ms: 0,
                seq: 0,
                cause_seq: None,
                event: TimelineEvent::BattleStart {
                    width: 1,
                    height: 1,
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 1,
                cause_seq: None,
                event: TimelineEvent::UnitSpawned {
                    unit_instance_id: attacker_id,
                    owner: Side::Player,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 2,
                cause_seq: None,
                event: TimelineEvent::UnitSpawned {
                    unit_instance_id: target_id,
                    owner: Side::Opponent,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::ItemSpawned {
                    item_instance_id: Uuid::from_u128(0xC),
                    owner: Side::Player,
                    owner_unit_instance_id: attacker_id,
                    base_uuid: item_uuid,
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 4,
                cause_seq: None,
                event: TimelineEvent::Attack {
                    attacker_instance_id: attacker_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 5,
                cause_seq: Some(4),
                event: TimelineEvent::HpChanged {
                    source_instance_id: Some(attacker_id),
                    target_instance_id: target_id,
                    delta: -10,
                    hp_before: 100,
                    hp_after: 90,
                    reason: game_core::game::battle::HpChangeReason::BasicAttack,
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 6,
                cause_seq: Some(4),
                event: TimelineEvent::AbilityCast {
                    ability_id: game_core::game::ability::AbilityId::RedShoesBerserk,
                    caster_instance_id: attacker_id,
                    target_instance_id: Some(target_id),
                },
            },
            // ExtraAttack should schedule Triggered Attack decisions (but they are missing here).
            TimelineEntry {
                time_ms: 200,
                seq: 7,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let violations = replayer.replay(&timeline).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineReplayViolationKind::ExpectedDecisionMissing));
}

#[test]
fn replayer_rejects_unexpected_triggered_attack_decision_in_strict_mode() {
    let caster_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);

    let game_data = Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(Vec::new())),
        Arc::new(ArtifactDatabase::new(Vec::new())),
        Arc::new(EquipmentDatabase::new(Vec::new())),
        Arc::new(ShopDatabase::new(Vec::new())),
        Arc::new(BonusDatabase::new(Vec::new())),
        Arc::new(RandomEventDatabase::new(Vec::new())),
        Arc::new(PveEncounterDatabase::new(Vec::new())),
        empty_event_pools(),
    ));

    let replayer = TimelineReplayer::new(
        game_data,
        TimelineReplayerConfig {
            validate_basic_attack_outcomes: false,
            forbid_unexpected_outcomes_for_verified_causes: true,
            ..TimelineReplayerConfig::default()
        },
    );

    let timeline = Timeline {
        version: TIMELINE_VERSION,
        entries: vec![
            TimelineEntry {
                time_ms: 0,
                seq: 0,
                cause_seq: None,
                event: TimelineEvent::BattleStart {
                    width: 1,
                    height: 1,
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 1,
                cause_seq: None,
                event: TimelineEvent::UnitSpawned {
                    unit_instance_id: caster_id,
                    owner: Side::Player,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 2,
                cause_seq: None,
                event: TimelineEvent::UnitSpawned {
                    unit_instance_id: target_id,
                    owner: Side::Opponent,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::AbilityCast {
                    ability_id: game_core::game::ability::AbilityId::RedShoesBerserk,
                    caster_instance_id: caster_id,
                    target_instance_id: Some(target_id),
                },
            },
            // RedShoesBerserk is defined as ExtraAttack(count=2). The 3rd one should be rejected.
            TimelineEntry {
                time_ms: 100,
                seq: 4,
                cause_seq: Some(3),
                event: TimelineEvent::Attack {
                    attacker_instance_id: caster_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 5,
                cause_seq: Some(3),
                event: TimelineEvent::Attack {
                    attacker_instance_id: caster_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 100,
                seq: 6,
                cause_seq: Some(3),
                event: TimelineEvent::Attack {
                    attacker_instance_id: caster_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 200,
                seq: 7,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let violations = replayer.replay(&timeline).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineReplayViolationKind::UnexpectedDecision));
}
