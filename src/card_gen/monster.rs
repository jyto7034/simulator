use crate::{
    card::{
        types::{OwnerType, StatType},
        Card,
    },
    effect::{DrawEffect, ModifyStatEffect},
    enums::ZoneType,
    selector::single::SingleCardSelector,
    utils::json::CardJson,
};

use super::builder::CardBuilder;

#[allow(non_snake_case)]
pub fn MT_001(card_json: &CardJson, count: i32) -> Card {
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
pub fn MT_002(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}

#[allow(non_snake_case)]
pub fn MT_003(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_004(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_005(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_006(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_007(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_008(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_009(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_010(card_json: &CardJson, count: i32) -> Card {
    MT_001(card_json, count)
}
