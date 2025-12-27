#![cfg(feature = "timeline_checks")]

use std::sync::Arc;

use game_core::ecs::resources::Position;
use game_core::game::battle::{
    replay::{TimelineReplayViolationKind, TimelineReplayer, TimelineReplayerConfig},
    validation::{TimelineValidator, TimelineValidatorConfig, TimelineViolationKind},
    BattleWinner, Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION,
};
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::EquipmentDatabase;
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Side};
use game_core::game::stats::UnitStats;
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

fn game_data_with_resonance_unit(base_uuid: Uuid) -> Arc<GameDataBase> {
    let abnormality = AbnormalityMetadata {
        id: "test_unit".to_string(),
        uuid: base_uuid,
        name: "test_unit".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 100,
        attack: 1,
        defense: 0,
        attack_interval_ms: 1000,
        resonance_start: 100,
        resonance_max: 100,
        resonance_lock_ms: 1000,
        abilities: Vec::new(),
    };

    Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(vec![abnormality])),
        Arc::new(ArtifactDatabase::new(Vec::new())),
        Arc::new(EquipmentDatabase::new(Vec::new())),
        Arc::new(ShopDatabase::new(Vec::new())),
        Arc::new(BonusDatabase::new(Vec::new())),
        Arc::new(RandomEventDatabase::new(Vec::new())),
        Arc::new(PveEncounterDatabase::new(Vec::new())),
        empty_event_pools(),
    ))
}

fn spawn_stats() -> UnitStats {
    UnitStats::with_values(100, 100, 1, 0, 1000)
}

#[test]
fn validator_rejects_missing_autocast_end() {
    let caster_id = Uuid::from_u128(0xA);
    let base_uuid = Uuid::from_u128(0x1000);

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
                time_ms: 10,
                seq: 2,
                cause_seq: None,
                event: TimelineEvent::AutoCastStart {
                    caster_instance_id: caster_id,
                    ability_id: None,
                    target_instance_id: None,
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig::default());
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::AutoCastPairInvalid));
}

#[test]
fn replayer_rejects_missing_autocast_end() {
    let caster_id = Uuid::from_u128(0xA);
    let base_uuid = Uuid::from_u128(0x1000);

    let game_data = game_data_with_resonance_unit(base_uuid);
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
                    unit_instance_id: caster_id,
                    owner: Side::Player,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 2,
                cause_seq: None,
                event: TimelineEvent::AutoCastStart {
                    caster_instance_id: caster_id,
                    ability_id: None,
                    target_instance_id: None,
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 3,
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
        .any(|v| v.kind == TimelineReplayViolationKind::InvalidAutoCastEvent));
}
