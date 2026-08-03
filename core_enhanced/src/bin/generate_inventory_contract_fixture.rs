use std::collections::HashMap;

use game_core::game::{
    behavior::{InventoryEquipmentSnapshotDto, InventorySnapshotDto},
    data::{
        consumable_data::{
            ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata, ConsumableTargetPolicy,
            ConsumableTier,
        },
        equipment_data::{EquipmentMetadata, EquipmentType},
    },
    enums::RiskLevel,
    resources::{ConsumableItemDto, EquipmentItemDto},
};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let equipment = EquipmentMetadata {
        id: "fixture_blade".to_string(),
        uuid: Uuid::from_u128(0xE100),
        name: "Fixture Blade".to_string(),
        equipment_type: EquipmentType::Weapon,
        rarity: RiskLevel::TETH,
        price: 12,
        allow_duplicate_equip: true,
        bound: false,
        cannot_unequip_reason: "equipment_bound".to_string(),
        triggered_effects: HashMap::new(),
        ability_activations: vec![],
        weapon_profile: Some(Default::default()),
        event_check_modifiers: vec![],
        event_choice_keys: vec![],
    };
    let equipment_instance_uuid = Uuid::from_u128(0xE101);
    let consumable = ConsumableMetadata {
        id: "fixture_tonic".to_string(),
        uuid: Uuid::from_u128(0xC100),
        name: "Fixture Tonic".to_string(),
        description: "Rust-generated definition-backed stack".to_string(),
        tier: ConsumableTier::Common,
        rarity: RiskLevel::ZAYIN,
        price: 5,
        target_policy: ConsumableTargetPolicy::SingleEmployee,
        duration_policy: ConsumableDurationPolicy::NextCombatNode,
        effect: ConsumableEffect::TraumaMitigation { percent: 10 },
        live_pool: true,
    };
    let inventory = InventorySnapshotDto {
        reserved_slot_count: 60,
        occupied_slot_count: 2,
        equipments: vec![InventoryEquipmentSnapshotDto {
            instance_uuid: equipment_instance_uuid,
            item: EquipmentItemDto::from_owned_with_level(equipment_instance_uuid, &equipment, 0),
            equipped_to: None,
        }],
        equipment_materials: vec![],
        consumables: vec![ConsumableItemDto::from_metadata(&consumable, 7)],
        skill_fragments: vec![],
        skill_fragment_progress: vec![],
        pending_research_deliveries: vec![],
        fragment_dust: 0,
    };
    let output = json!({
        "game_state": "viewing_map",
        "game_state_context": { "type": "viewing_map" },
        "inventory": inventory,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("inventory fixture must serialize")
    );
}
