use crate::{card::{effect::{DrawEffect, ModifyStatEffect}, target_selector::SingleCardSelector, types::{OwnerType, StatType}, Card}, enums::{CardLocation, ZoneType}, utils::json::CardJson};

use super::builder::CardBuilder;

#[allow(non_snake_case)]
pub fn MT_001(card_json: &CardJson, count: usize) -> Card {
    CardBuilder::new(card_json)
        .unwrap()
        .add_effect(DrawEffect { count: 2 })
        .add_effect(ModifyStatEffect {
            stat_type: StatType::Attack,
            amount: 2,
            target_selector: Box::new(SingleCardSelector::new(
                CardLocation(ZoneType::None),
                OwnerType::Any,
            )),
        })
        .build()
}
#[allow(non_snake_case)]
pub fn MT_002(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_003(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_004(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_005(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_006(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_007(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_008(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_009(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn MT_010(card_json: &CardJson, count: usize) -> Card {
    MT_001(card_json, count)
}