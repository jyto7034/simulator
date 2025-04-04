use crate::{
    card::{
        effect::{DrawEffect, ModifyStatEffect},
        types::{OwnerType, StatType},
        Card,
    },
    enums::ZoneType,
    selector::single::SingleCardSelector,
    utils::json::CardJson,
};

use super::builder::CardBuilder;

#[allow(non_snake_case)]
pub fn PB_001(card_json: &CardJson, count: i32) -> Card {
    CardBuilder::new(card_json)
        .unwrap()
        .add_effect(DrawEffect { count: 2 })
        .add_effect(ModifyStatEffect {
            stat_type: StatType::Attack,
            amount: 2,
            target_selector: Box::new(SingleCardSelector::new(ZoneType::None, OwnerType::Any)),
        })
        .build()
}
#[allow(non_snake_case)]
pub fn PB_002(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_003(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_004(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_005(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_006(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_007(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_008(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_009(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_010(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_011(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_012(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_013(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_014(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_015(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_016(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_017(card_json: &CardJson, count: i32) -> Card {
    PB_001(card_json, count)
}
