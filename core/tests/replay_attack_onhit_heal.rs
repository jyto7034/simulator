use std::{collections::HashMap, sync::Arc};

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::battle::{
    replay::{TimelineReplayer, TimelineReplayerConfig},
    validation::{TimelineExpectedCounts, TimelineValidator, TimelineValidatorConfig},
    BattleCore, GrowthStack, HpChangeReason, OwnedUnit, PlayerDeckInfo, TimelineEvent,
};
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Side, Tier};
use game_core::game::stats::{Effect, TriggerType, TriggeredEffects};
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

#[test]
fn attack_onhit_heal_is_predicted_in_strict_replay() {
    let player_base_uuid = Uuid::from_u128(0x1000);
    let opponent_base_uuid = Uuid::from_u128(0x2000);
    let onhit_heal_item_uuid = Uuid::from_u128(0x3000);

    let abnormalities = vec![
        AbnormalityMetadata {
            id: "player".to_string(),
            uuid: player_base_uuid,
            name: "Player".to_string(),
            risk_level: RiskLevel::TETH,
            price: 0,
            max_health: 200,
            attack: 25,
            defense: 0,
            attack_interval_ms: 500,
            resonance_start: 0,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![],
        },
        AbnormalityMetadata {
            id: "opponent".to_string(),
            uuid: opponent_base_uuid,
            name: "Opponent".to_string(),
            risk_level: RiskLevel::TETH,
            price: 0,
            max_health: 100,
            attack: 1,
            defense: 0,
            attack_interval_ms: 500,
            resonance_start: 0,
            resonance_max: 100,
            resonance_lock_ms: 1000,
            abilities: vec![],
        },
    ];

    let mut onhit_heal_effects = TriggeredEffects::new();
    onhit_heal_effects.insert(
        TriggerType::OnHit,
        vec![Effect::Heal {
            flat: 5,
            percent: 0,
        }],
    );

    let equipments = vec![EquipmentMetadata {
        id: "onhit_heal".to_string(),
        uuid: onhit_heal_item_uuid,
        name: "OnHit Heal".to_string(),
        equipment_type: EquipmentType::Accessory,
        rarity: RiskLevel::TETH,
        price: 0,
        allow_duplicate_equip: true,
        triggered_effects: onhit_heal_effects,
    }];

    let game_data = Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(abnormalities)),
        Arc::new(ArtifactDatabase::new(Vec::new())),
        Arc::new(EquipmentDatabase::new(equipments)),
        Arc::new(ShopDatabase::new(Vec::new())),
        Arc::new(BonusDatabase::new(Vec::new())),
        Arc::new(RandomEventDatabase::new(Vec::new())),
        Arc::new(PveEncounterDatabase::new(Vec::new())),
        empty_event_pools(),
    ));

    let mut player_positions = HashMap::new();
    player_positions.insert(player_base_uuid, Position::new(0, 0));
    let player_deck = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: player_base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![],
        }],
        artifacts: vec![],
        positions: player_positions,
    };

    let mut opponent_positions = HashMap::new();
    opponent_positions.insert(opponent_base_uuid, Position::new(1, 0));
    let opponent_deck = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: opponent_base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![onhit_heal_item_uuid],
        }],
        artifacts: vec![],
        positions: opponent_positions,
    };

    let mut world = World::new();
    let mut battle = BattleCore::new(&player_deck, &opponent_deck, game_data.clone(), (2, 1));
    let result = battle.run_battle(&mut world).expect("battle should run");

    let expected_counts = TimelineExpectedCounts::from_decks(&player_deck, &opponent_deck);
    let validator = TimelineValidator::new(TimelineValidatorConfig::default());
    validator
        .validate(&result.timeline, Some(expected_counts))
        .expect("timeline should validate");

    let replayer = TimelineReplayer::new(
        game_data,
        TimelineReplayerConfig {
            forbid_unexpected_outcomes_for_verified_causes: true,
            ..TimelineReplayerConfig::default()
        },
    );
    replayer
        .replay(&result.timeline)
        .expect("strict replay should succeed");

    let mut opponent_instance_id = None;
    for entry in &result.timeline.entries {
        if let TimelineEvent::UnitSpawned {
            unit_instance_id,
            owner: Side::Opponent,
            base_uuid,
            ..
        } = entry.event
        {
            if base_uuid == opponent_base_uuid {
                opponent_instance_id = Some(unit_instance_id);
            }
        }
    }
    let opponent_instance_id = opponent_instance_id.expect("opponent should spawn");

    let saw_onhit_heal = result
        .timeline
        .entries
        .iter()
        .any(|entry| match entry.event {
            TimelineEvent::HpChanged {
                source_instance_id: Some(source_id),
                target_instance_id,
                delta,
                reason: HpChangeReason::Command,
                ..
            } => {
                source_id == opponent_instance_id
                    && target_instance_id == opponent_instance_id
                    && delta > 0
            }
            _ => false,
        });
    assert!(saw_onhit_heal, "expected an on-hit heal HpChanged entry");
}
