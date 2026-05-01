mod common;

use std::sync::Arc;

use bevy_ecs::world::World;
use game_core::ecs::resources::Position;
use game_core::game::battle::{BattleCore, GrowthStack, OwnedArtifact, OwnedUnit, PlayerDeckInfo};
use game_core::game::behavior::GameError;
use game_core::game::data::abnormality_data::{AbnormalityDatabase, AbnormalityMetadata};
use game_core::game::data::artifact_data::ArtifactDatabase;
use game_core::game::data::bonus_data::BonusDatabase;
use game_core::game::data::equipment_data::{EquipmentDatabase, EquipmentMetadata, EquipmentType};
use game_core::game::data::event_pools::{EventPhasePool, EventPoolConfig};
use game_core::game::data::pve_data::PveEncounterDatabase;
use game_core::game::data::random_event_data::RandomEventDatabase;
use game_core::game::data::shop_data::ShopDatabase;
use game_core::game::data::GameDataBase;
use game_core::game::enums::{RiskLevel, Tier};
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
fn rejects_duplicate_artifacts_in_same_deck() {
    let game_data = common::create_test_game_data();
    let duplicate_artifact_uuid = Uuid::parse_str("a0000001-0000-0000-0000-000000000001").unwrap();

    let player = PlayerDeckInfo {
        units: Vec::new(),
        artifacts: vec![
            OwnedArtifact {
                base_uuid: duplicate_artifact_uuid,
            },
            OwnedArtifact {
                base_uuid: duplicate_artifact_uuid,
            },
        ],
        positions: Default::default(),
    };
    let opponent = PlayerDeckInfo {
        units: Vec::new(),
        artifacts: Vec::new(),
        positions: Default::default(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (1, 1));
    let mut world = World::new();
    assert!(matches!(
        battle.run_battle(&mut world),
        Err(GameError::InvalidAction)
    ));
}

#[test]
fn rejects_duplicate_item_equips_when_disallowed() {
    let unit_base_uuid = Uuid::from_u128(0x1000);
    let item_uuid = Uuid::from_u128(0x2000);

    let abnormality = AbnormalityMetadata {
        id: "unit".to_string(),
        uuid: unit_base_uuid,
        name: "unit".to_string(),
        risk_level: RiskLevel::ZAYIN,
        price: 0,
        max_health: 10,
        attack: 1,
        defense: 0,
        attack_interval_ms: 1000,
        resonance_start: 0,
        resonance_max: 100,
        resonance_lock_ms: 1000,
        abilities: Vec::new(),
    };

    let item = EquipmentMetadata {
        id: "item".to_string(),
        uuid: item_uuid,
        name: "item".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::ZAYIN,
        price: 0,
        allow_duplicate_equip: false,
        triggered_effects: Default::default(),
    };

    let game_data = Arc::new(GameDataBase::new(
        Arc::new(AbnormalityDatabase::new(vec![abnormality])),
        Arc::new(ArtifactDatabase::new(Vec::new())),
        Arc::new(EquipmentDatabase::new(vec![item])),
        Arc::new(ShopDatabase::new(Vec::new())),
        Arc::new(BonusDatabase::new(Vec::new())),
        Arc::new(RandomEventDatabase::new(Vec::new())),
        Arc::new(PveEncounterDatabase::new(Vec::new())),
        empty_event_pools(),
    ));

    let player = PlayerDeckInfo {
        units: vec![OwnedUnit {
            base_uuid: unit_base_uuid,
            level: Tier::I,
            growth_stacks: GrowthStack::new(),
            equipped_items: vec![item_uuid, item_uuid],
        }],
        artifacts: Vec::new(),
        positions: [(unit_base_uuid, Position::new(0, 0))]
            .into_iter()
            .collect(),
    };
    let opponent = PlayerDeckInfo {
        units: Vec::new(),
        artifacts: Vec::new(),
        positions: Default::default(),
    };

    let mut battle = BattleCore::new(&player, &opponent, game_data, (1, 1));
    let mut world = World::new();
    assert!(matches!(
        battle.run_battle(&mut world),
        Err(GameError::InvalidAction)
    ));
}
