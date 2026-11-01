use game_core::game::{
    behavior::{
        ActionCapabilityDto, ActionKind, ActionScope, DisplayConsumableItemSnapshotDto,
        DisplayItemSnapshotDto, InventorySnapshotDto, SelectedEventSnapshotDto,
        ShopOfferSnapshotDto, ShopPayloadEligibilityDto, ShopRefreshSnapshotDto,
        ShopSaleCandidateSnapshotDto,
    },
    data::{
        consumable_data::{
            ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata, ConsumableTargetPolicy,
            ConsumableTier,
        },
        shop_data::{ShopContext, ShopMerchandise},
    },
    enums::RiskLevel,
    resources::ConsumableItemDto,
};
use serde_json::json;
use uuid::Uuid;

fn main() {
    let tonic = ConsumableMetadata {
        id: "fixture_shop_tonic".to_string(),
        uuid: Uuid::from_u128(0xC500),
        name: "Fixture Shop Tonic".to_string(),
        description: "Rust-generated Shop offer".to_string(),
        tier: ConsumableTier::Common,
        rarity: RiskLevel::ZAYIN,
        price: 25,
        target_policy: ConsumableTargetPolicy::SingleEmployee,
        duration_policy: ConsumableDurationPolicy::NextCombatNode,
        effect: ConsumableEffect::TraumaMitigation { percent: 10 },
        live_pool: true,
    };
    let merchandise = ShopMerchandise::Consumable {
        consumable_id: tonic.id.clone(),
    };
    let item = DisplayItemSnapshotDto::Consumable(DisplayConsumableItemSnapshotDto {
        uuid: tonic.uuid,
        definition_id: tonic.id.clone(),
        name: tonic.name.clone(),
        description: tonic.description.clone(),
        tier: tonic.tier,
        rarity: tonic.rarity,
        price: tonic.price,
        target_policy: tonic.target_policy,
        duration_policy: tonic.duration_policy,
        effect: tonic.effect.clone(),
        kind: "consumable".to_string(),
        id: tonic.id.clone(),
        item_type: "consumable".to_string(),
    });
    let selected_event = SelectedEventSnapshotDto::Shop {
        id: "fixture_headquarters_shop".to_string(),
        name: "Fixture Headquarters Supply".to_string(),
        uuid: Uuid::from_u128(0x5500),
        context: ShopContext::HeadquartersSupport,
        offers: vec![ShopOfferSnapshotDto {
            offer_id: Uuid::from_u128(0x0F01),
            merchandise: merchandise.clone(),
            quantity: 3,
            total_price: 75,
            item,
            eligibility: ShopPayloadEligibilityDto::enabled(),
        }],
        refresh: ShopRefreshSnapshotDto {
            available: true,
            current_cost: Some(10),
            max_uses: 2,
            uses_consumed: 1,
            remaining_uses: 1,
            eligibility: ShopPayloadEligibilityDto::enabled(),
        },
        selling_enabled: true,
        sale_candidates: vec![ShopSaleCandidateSnapshotDto {
            item_uuid: tonic.uuid,
            merchandise,
            available_quantity: 7,
            unit_quote: 10,
            eligibility: ShopPayloadEligibilityDto::enabled(),
        }],
    };
    let inventory = InventorySnapshotDto {
        reserved_slot_count: 50,
        occupied_slot_count: 1,
        equipments: vec![],
        equipment_materials: vec![],
        consumables: vec![ConsumableItemDto::from_metadata(&tonic, 7)],
        skill_fragments: vec![],
        fragment_dust_usage: "reserved_legacy_asset_not_used_by_fragment_recipes".into(),
        fragment_dust: 0,
    };
    let output = json!({
        "game_state": "in_shop",
        "game_state_context": {
            "type": "in_shop",
            "shop_uuid": Uuid::from_u128(0x5500),
        },
        "action_capabilities": [
            ActionCapabilityDto::enabled(ActionKind::PurchaseItem, ActionScope::Gameplay),
            ActionCapabilityDto::enabled(ActionKind::SellItem, ActionScope::Gameplay),
            ActionCapabilityDto::enabled(ActionKind::RefreshShop, ActionScope::Gameplay),
            ActionCapabilityDto::enabled(ActionKind::ExitShop, ActionScope::Gameplay),
        ],
        "selected_event": selected_event,
        "inventory": inventory,
        "resources": { "enkephalin": 140 },
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("Shop fixture must serialize")
    );
}
