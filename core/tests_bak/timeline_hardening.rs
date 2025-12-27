#![cfg(feature = "timeline_checks")]

use std::sync::Arc;

use game_core::ecs::resources::Position;
use game_core::game::battle::{
    buffs::BuffId,
    replay::{TimelineReplayViolationKind, TimelineReplayer, TimelineReplayerConfig},
    timeline::AttackKind,
    validation::{TimelineValidator, TimelineValidatorConfig, TimelineViolationKind},
    BattleWinner, Timeline, TimelineEntry, TimelineEvent, TIMELINE_VERSION,
};
use game_core::game::data::abnormality_data::AbnormalityDatabase;
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::EquipmentDatabase;
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::Side;
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

fn empty_game_data() -> Arc<GameDataBase> {
    Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(Vec::new())),
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
    UnitStats::with_values(100, 100, 10, 0, 1000)
}

#[test]
fn validator_rejects_unit_reference_before_spawn() {
    let attacker_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
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
                    unit_instance_id: attacker_id,
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
                event: TimelineEvent::Attack {
                    attacker_instance_id: attacker_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 3,
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
                time_ms: 30,
                seq: 4,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Player,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig {
        require_attack_has_basic_hp_change: false,
        ..TimelineValidatorConfig::default()
    });
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::UnitReferencedBeforeSpawn));
}

#[test]
fn validator_rejects_attack_missing_kind() {
    let attacker_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
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
                time_ms: 10,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::Attack {
                    attacker_instance_id: attacker_id,
                    target_instance_id: target_id,
                    kind: None,
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 4,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig {
        require_attack_has_basic_hp_change: false,
        ..TimelineValidatorConfig::default()
    });
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::AttackKindMissing));
}

#[test]
fn validator_rejects_unit_died_with_positive_hp() {
    let unit_id = Uuid::from_u128(0xA);
    let enemy_id = Uuid::from_u128(0xB);
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
                    unit_instance_id: unit_id,
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
                    unit_instance_id: enemy_id,
                    owner: Side::Opponent,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::Attack {
                    attacker_instance_id: enemy_id,
                    target_instance_id: unit_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 4,
                cause_seq: Some(3),
                event: TimelineEvent::UnitDied {
                    unit_instance_id: unit_id,
                    owner: Side::Player,
                    killer_instance_id: Some(enemy_id),
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 5,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig {
        require_attack_has_basic_hp_change: false,
        ..TimelineValidatorConfig::default()
    });
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::UnitDiedWhileAlive));
}

#[test]
fn validator_rejects_buff_applied_by_dead_caster() {
    let caster_id = Uuid::from_u128(0xA);
    let attacker_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);
    let poison = BuffId::from_name("poison");

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
                    unit_instance_id: attacker_id,
                    owner: Side::Opponent,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: spawn_stats(),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::Attack {
                    attacker_instance_id: attacker_id,
                    target_instance_id: caster_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 4,
                cause_seq: Some(3),
                event: TimelineEvent::HpChanged {
                    source_instance_id: Some(attacker_id),
                    target_instance_id: caster_id,
                    delta: -100,
                    hp_before: 100,
                    hp_after: 0,
                    reason: game_core::game::battle::timeline::HpChangeReason::BasicAttack,
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 5,
                cause_seq: Some(3),
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster_id,
                    target_instance_id: attacker_id,
                    buff_id: poison,
                    duration_ms: 1000,
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 6,
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
        .any(|v| v.kind == TimelineViolationKind::BuffAppliedByDeadCaster));
}

#[test]
fn validator_checks_outcome_causes_without_contiguous_seq() {
    let attacker_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);

    let timeline = Timeline {
        version: TIMELINE_VERSION,
        entries: vec![
            TimelineEntry {
                time_ms: 0,
                seq: 10,
                cause_seq: None,
                event: TimelineEvent::BattleStart {
                    width: 1,
                    height: 1,
                },
            },
            TimelineEntry {
                time_ms: 0,
                seq: 20,
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
                seq: 30,
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
                time_ms: 10,
                seq: 40,
                cause_seq: None,
                event: TimelineEvent::Attack {
                    attacker_instance_id: attacker_id,
                    target_instance_id: target_id,
                    kind: Some(AttackKind::Triggered),
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 50,
                cause_seq: None,
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
                time_ms: 20,
                seq: 60,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig {
        require_contiguous_seq: false,
        require_attack_has_basic_hp_change: false,
        ..TimelineValidatorConfig::default()
    });
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::OutcomeMissingCauseSeq));
}

#[test]
fn validator_rejects_timeline_version_mismatch() {
    let unit_instance_id = Uuid::from_u128(0xA);
    let base_uuid = Uuid::from_u128(0x1000);

    let timeline = Timeline {
        version: TIMELINE_VERSION + 1,
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
                    unit_instance_id,
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
        .any(|v| v.kind == TimelineViolationKind::TimelineVersionMismatch));
}

#[test]
fn validator_rejects_stat_changed_with_invalid_stats_after() {
    let unit_instance_id = Uuid::from_u128(0xA);
    let base_uuid = Uuid::from_u128(0x1000);

    let before = spawn_stats();
    let mut after = before;
    after.attack_interval_ms = 0;

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
                    unit_instance_id,
                    owner: Side::Player,
                    base_uuid,
                    position: Position::new(0, 0),
                    stats: before,
                },
            },
            TimelineEntry {
                time_ms: 1,
                seq: 2,
                cause_seq: None,
                event: TimelineEvent::StatChanged {
                    source_instance_id: None,
                    target_instance_id: unit_instance_id,
                    modifier: game_core::game::stats::StatModifier {
                        stat: game_core::game::stats::StatId::AttackIntervalMs,
                        kind: game_core::game::stats::StatModifierKind::Flat,
                        value: -9999,
                    },
                    stats_before: before,
                    stats_after: after,
                },
            },
            TimelineEntry {
                time_ms: 2,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::BattleEnd {
                    winner: BattleWinner::Draw,
                },
            },
        ],
    };

    let validator = TimelineValidator::new(TimelineValidatorConfig {
        validate_outcome_cause_seq: false,
        ..TimelineValidatorConfig::default()
    });
    let violations = validator.validate(&timeline, None).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineViolationKind::StatsAfterInvalid));
}

#[test]
fn validator_rejects_buff_tick_without_applied_buff() {
    let caster_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);
    let poison = BuffId::from_name("poison");

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
                time_ms: 1000,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::BuffTick {
                    caster_instance_id: caster_id,
                    target_instance_id: target_id,
                    buff_id: poison,
                },
            },
            TimelineEntry {
                time_ms: 2000,
                seq: 4,
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
        .any(|v| v.kind == TimelineViolationKind::BuffTickInvalid));
}

#[test]
fn replayer_rejects_timeline_version_mismatch() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(game_data, TimelineReplayerConfig::default());

    let timeline = Timeline {
        version: TIMELINE_VERSION + 1,
        entries: vec![TimelineEntry {
            time_ms: 0,
            seq: 0,
            cause_seq: None,
            event: TimelineEvent::BattleStart {
                width: 1,
                height: 1,
            },
        }],
    };

    let violations = replayer.replay(&timeline).unwrap_err();
    assert!(violations
        .iter()
        .any(|v| v.kind == TimelineReplayViolationKind::TimelineVersionMismatch));
}

#[test]
fn replayer_rejects_buff_tick_without_applied_buff() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(
        game_data,
        TimelineReplayerConfig {
            validate_buff_tick_outcomes: false,
            ..TimelineReplayerConfig::default()
        },
    );

    let caster_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);
    let poison = BuffId::from_name("poison");

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
                time_ms: 1000,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::BuffTick {
                    caster_instance_id: caster_id,
                    target_instance_id: target_id,
                    buff_id: poison,
                },
            },
            TimelineEntry {
                time_ms: 2000,
                seq: 4,
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
        .any(|v| v.kind == TimelineReplayViolationKind::InvalidBuffEvent));
}

#[test]
fn replayer_rejects_unit_died_with_positive_hp() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(
        game_data,
        TimelineReplayerConfig {
            validate_basic_attack_outcomes: false,
            validate_ability_outcomes: false,
            validate_buff_tick_outcomes: false,
            validate_expected_decisions: false,
            ..TimelineReplayerConfig::default()
        },
    );

    let unit_id = Uuid::from_u128(0xA);
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
                    unit_instance_id: unit_id,
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
                event: TimelineEvent::UnitDied {
                    unit_instance_id: unit_id,
                    owner: Side::Player,
                    killer_instance_id: None,
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
        .any(|v| v.kind == TimelineReplayViolationKind::OutcomeMismatch));
}

#[test]
fn replayer_rejects_buff_applied_by_dead_caster() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(
        game_data,
        TimelineReplayerConfig {
            validate_basic_attack_outcomes: false,
            validate_ability_outcomes: false,
            validate_buff_tick_outcomes: false,
            validate_expected_decisions: false,
            ..TimelineReplayerConfig::default()
        },
    );

    let caster_id = Uuid::from_u128(0xA);
    let target_id = Uuid::from_u128(0xB);
    let base_uuid = Uuid::from_u128(0x1000);
    let poison = BuffId::from_name("poison");

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
                time_ms: 10,
                seq: 3,
                cause_seq: None,
                event: TimelineEvent::HpChanged {
                    source_instance_id: None,
                    target_instance_id: caster_id,
                    delta: -100,
                    hp_before: 100,
                    hp_after: 0,
                    reason: game_core::game::battle::timeline::HpChangeReason::Command,
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 4,
                cause_seq: None,
                event: TimelineEvent::UnitDied {
                    unit_instance_id: caster_id,
                    owner: Side::Player,
                    killer_instance_id: None,
                },
            },
            TimelineEntry {
                time_ms: 10,
                seq: 5,
                cause_seq: None,
                event: TimelineEvent::BuffApplied {
                    caster_instance_id: caster_id,
                    target_instance_id: target_id,
                    buff_id: poison,
                    duration_ms: 1000,
                },
            },
            TimelineEntry {
                time_ms: 20,
                seq: 6,
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
        .any(|v| v.kind == TimelineReplayViolationKind::InvalidBuffEvent));
}

#[test]
fn replayer_rejects_unknown_item_base_uuid() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(game_data, TimelineReplayerConfig::default());

    let unit_id = Uuid::from_u128(0xA);
    let base_uuid = Uuid::from_u128(0x1000);
    let item_instance_id = Uuid::from_u128(0xB);
    let unknown_item_uuid = Uuid::from_u128(0xDEAD_BEEF);

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
                    unit_instance_id: unit_id,
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
                event: TimelineEvent::ItemSpawned {
                    item_instance_id,
                    owner: Side::Player,
                    owner_unit_instance_id: unit_id,
                    base_uuid: unknown_item_uuid,
                },
            },
            TimelineEntry {
                time_ms: 1,
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
        .any(|v| v.kind == TimelineReplayViolationKind::UnknownItemReference));
}

#[test]
fn replayer_rejects_unknown_artifact_base_uuid() {
    let game_data = empty_game_data();
    let replayer = TimelineReplayer::new(game_data, TimelineReplayerConfig::default());

    let artifact_instance_id = Uuid::from_u128(0xA);
    let unknown_artifact_uuid = Uuid::from_u128(0xCAFE_BABE);

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
                event: TimelineEvent::ArtifactSpawned {
                    artifact_instance_id,
                    owner: Side::Player,
                    base_uuid: unknown_artifact_uuid,
                },
            },
            TimelineEntry {
                time_ms: 1,
                seq: 2,
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
        .any(|v| v.kind == TimelineReplayViolationKind::UnknownArtifactReference));
}
