#![cfg(feature = "timeline_checks")]

use std::{collections::HashMap, sync::Arc};

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::ability::AbilityId;
use game_core::game::battle::{
    replay::{TimelineReplayer, TimelineReplayerConfig},
    validation::{TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig},
    BattleCore, BattleWinner, GrowthStack, HpChangeReason, OwnedArtifact, OwnedUnit,
    PlayerDeckInfo, Timeline, TimelineEvent,
};
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::{ArtifactDatabase, ArtifactMetadata};
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::stats::{
    Effect, StatId, StatModifier, StatModifierKind, TriggerType, TriggeredEffects,
};
use uuid::Uuid;

#[test]
fn realistic_battle_resonance_autocast_and_poison_tick() {
    let caster_base_uuid = Uuid::from_u128(0x1000);
    let ally_base_uuid = Uuid::from_u128(0x1001);
    let enemy_tank_base_uuid = Uuid::from_u128(0x2000);
    let enemy_dps_base_uuid = Uuid::from_u128(0x2001);

    let artifact_uuid = Uuid::from_u128(0x3000);
    let venom_blade_uuid = Uuid::from_u128(0x4000);

    let mut artifact_effects = TriggeredEffects::new();
    artifact_effects.insert(
        TriggerType::Permanent,
        vec![Effect::Modifier(StatModifier {
            stat: StatId::MaxHealth,
            kind: StatModifierKind::Flat,
            value: 20,
        })],
    );

    let mut venom_blade_effects = TriggeredEffects::new();
    venom_blade_effects.insert(
        TriggerType::Permanent,
        vec![Effect::Modifier(StatModifier {
            stat: StatId::Attack,
            kind: StatModifierKind::Flat,
            value: 5,
        })],
    );
    venom_blade_effects.insert(
        TriggerType::OnAttack,
        vec![Effect::ApplyBuff {
            buff_id: "poison".to_string(),
            duration_ms: 2500,
        }],
    );

    let abnormalities = vec![
        AbnormalityMetadata {
            id: "player_caster".to_string(),
            uuid: caster_base_uuid,
            name: "Player Caster".to_string(),
            risk_level: RiskLevel::HE,
            price: 0,
            max_health: 150,
            attack: 20,
            defense: 2,
            attack_interval_ms: 500,
            resonance_start: 90,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![AbilityId::UnknownDistortionStrike],
        },
        AbnormalityMetadata {
            id: "player_ally".to_string(),
            uuid: ally_base_uuid,
            name: "Player Ally".to_string(),
            risk_level: RiskLevel::TETH,
            price: 0,
            max_health: 120,
            attack: 12,
            defense: 1,
            attack_interval_ms: 800,
            resonance_start: 0,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![],
        },
        AbnormalityMetadata {
            id: "enemy_tank".to_string(),
            uuid: enemy_tank_base_uuid,
            name: "Enemy Tank".to_string(),
            risk_level: RiskLevel::HE,
            price: 0,
            max_health: 250,
            attack: 15,
            defense: 2,
            attack_interval_ms: 700,
            resonance_start: 0,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![],
        },
        AbnormalityMetadata {
            id: "enemy_dps".to_string(),
            uuid: enemy_dps_base_uuid,
            name: "Enemy DPS".to_string(),
            risk_level: RiskLevel::HE,
            price: 0,
            max_health: 160,
            attack: 18,
            defense: 1,
            attack_interval_ms: 600,
            resonance_start: 0,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![],
        },
    ];

    let artifacts = vec![ArtifactMetadata {
        id: "artifact_banner".to_string(),
        uuid: artifact_uuid,
        name: "Battle Banner".to_string(),
        description: "Permanent HP boost".to_string(),
        rarity: RiskLevel::TETH,
        price: 0,
        triggered_effects: artifact_effects,
    }];

    let equipments = vec![EquipmentMetadata {
        id: "venom_blade".to_string(),
        uuid: venom_blade_uuid,
        name: "Venom Blade".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::TETH,
        price: 0,
        allow_duplicate_equip: true,
        triggered_effects: venom_blade_effects,
    }];

    let empty_pool = EventPhasePool {
        shops: vec![],
        bonuses: vec![],
        random_events: vec![],
    };
    let event_pools = EventPoolConfig {
        dawn: empty_pool.clone(),
        noon: empty_pool.clone(),
        dusk: empty_pool.clone(),
        midnight: empty_pool.clone(),
        white: empty_pool,
    };

    let game_data = Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(abnormalities)),
        Arc::new(ArtifactDatabase::new(artifacts)),
        Arc::new(EquipmentDatabase::new(equipments)),
        Arc::new(ShopDatabase::new(vec![])),
        Arc::new(BonusDatabase::new(vec![])),
        Arc::new(RandomEventDatabase::new(vec![])),
        Arc::new(PveEncounterDatabase::new(vec![])),
        event_pools,
    ));

    let mut player_positions = HashMap::new();
    player_positions.insert(caster_base_uuid, Position::new(0, 0));
    player_positions.insert(ally_base_uuid, Position::new(0, 1));

    let mut opponent_positions = HashMap::new();
    opponent_positions.insert(enemy_tank_base_uuid, Position::new(2, 0));
    opponent_positions.insert(enemy_dps_base_uuid, Position::new(2, 1));

    let player_deck = PlayerDeckInfo {
        units: vec![
            OwnedUnit {
                base_uuid: caster_base_uuid,
                level: Tier::I,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![venom_blade_uuid],
            },
            OwnedUnit {
                base_uuid: ally_base_uuid,
                level: Tier::I,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
            },
        ],
        artifacts: vec![OwnedArtifact {
            base_uuid: artifact_uuid,
        }],
        positions: player_positions,
    };

    let opponent_deck = PlayerDeckInfo {
        units: vec![
            OwnedUnit {
                base_uuid: enemy_tank_base_uuid,
                level: Tier::I,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
            },
            OwnedUnit {
                base_uuid: enemy_dps_base_uuid,
                level: Tier::I,
                growth_stacks: GrowthStack::new(),
                equipped_items: vec![],
            },
        ],
        artifacts: vec![],
        positions: opponent_positions,
    };

    let mut world = World::new();
    let mut battle = BattleCore::new(&player_deck, &opponent_deck, game_data.clone(), (3, 2));
    let result = battle.run_battle(&mut world).expect("battle should run");
    assert_eq!(result.winner, BattleWinner::Player);

    // Timeline invariants are validated by the shared validation layer.
    let expected_counts = TimelineExpectedCounts::from_decks(&player_deck, &opponent_deck);
    let validator = TimelineValidator::new(TimelineValidatorConfig::default());
    validator
        .validate(&result.timeline, Some(expected_counts))
        .unwrap_or_else(|violations| {
            let summary = violations
                .into_iter()
                .map(|v| {
                    let index = v
                        .entry_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    format!("[{}] {:?}: {}", index, v.kind, v.message)
                })
                .collect::<Vec<_>>()
                .join("\n");
            panic!("timeline validation failed:\n{summary}");
        });

    let replayer = TimelineReplayer::new(
        game_data.clone(),
        TimelineReplayerConfig {
            forbid_unexpected_outcomes_for_verified_causes: true,
            ..TimelineReplayerConfig::default()
        },
    );
    replayer
        .replay(&result.timeline)
        .unwrap_or_else(|violations| {
            let summary = violations
                .into_iter()
                .map(|v| {
                    let index = v
                        .entry_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    format!("[{}] {:?}: {}", index, v.kind, v.message)
                })
                .collect::<Vec<_>>()
                .join("\n");
            panic!("timeline predictive replay failed:\n{summary}");
        });

    let mut caster_instance_id = None;
    let mut enemy_tank_instance_id = None;
    let mut caster_spawn_stats = None;

    for entry in &result.timeline.entries {
        if let TimelineEvent::UnitSpawned {
            unit_instance_id,
            owner,
            base_uuid,
            stats,
            ..
        } = entry.event
        {
            if owner == Side::Player && base_uuid == caster_base_uuid {
                caster_instance_id = Some(unit_instance_id);
                caster_spawn_stats = Some(stats);
            }
            if owner == Side::Opponent && base_uuid == enemy_tank_base_uuid {
                enemy_tank_instance_id = Some(unit_instance_id);
            }
        }
    }

    let caster_instance_id = caster_instance_id.expect("caster should spawn");
    let enemy_tank_instance_id = enemy_tank_instance_id.expect("enemy tank should spawn");
    let caster_spawn_stats = caster_spawn_stats.expect("caster spawn stats should be recorded");

    // Permanent effects from artifact (+20 max HP) and equipment (+5 attack) should apply.
    assert_eq!(caster_spawn_stats.max_health, 170);
    assert_eq!(caster_spawn_stats.current_health, 170);
    assert_eq!(caster_spawn_stats.attack, 25);

    // Resonance reaches full on the first attack and triggers exactly one autocast.
    let resonance_casts = result
        .timeline
        .entries
        .iter()
        .filter(|entry| match entry.event {
            TimelineEvent::AbilityCast {
                ability_id,
                caster_instance_id: caster_id,
                ..
            } => {
                ability_id == AbilityId::UnknownDistortionStrike && caster_id == caster_instance_id
            }
            _ => false,
        })
        .count();
    assert_eq!(resonance_casts, 1);

    // Poison tick should resolve before the caster's 1500ms attack (BuffTick priority > Attack).
    let poison_tick_index = result
        .timeline
        .entries
        .iter()
        .position(|entry| match entry.event {
            TimelineEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                reason: HpChangeReason::Command,
                ..
            } => {
                entry.time_ms == 1500
                    && source_instance_id == Some(caster_instance_id)
                    && target_instance_id == enemy_tank_instance_id
                    && delta < 0
            }
            _ => false,
        });

    let caster_attack_index = result
        .timeline
        .entries
        .iter()
        .position(|entry| match entry.event {
            TimelineEvent::Attack {
                attacker_instance_id,
                ..
            } => entry.time_ms == 1500 && attacker_instance_id == caster_instance_id,
            _ => false,
        });

    let poison_tick_index = poison_tick_index.expect("expected poison tick HP change at 1500ms");
    let caster_attack_index =
        caster_attack_index.expect("expected caster attack at 1500ms to exist");
    assert!(poison_tick_index < caster_attack_index);

    // JSON roundtrip should be stable.
    let json = result.timeline.to_json_string().unwrap();
    let loaded: Timeline = serde_json::from_str(&json).unwrap();
    let json_roundtrip = loaded.to_json_string().unwrap();
    assert_eq!(json, json_roundtrip);
}
