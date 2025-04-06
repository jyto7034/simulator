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
// -------------------------------------------------- FIELD
// [HM_001] Hieda no Akyuu - COST:?? [ATK:??/HP:?]
// - Set: Human, Rarity: C
// --------------------------------------------------------
// Text: 낮동안 인간 카드를 사용할 때 마다 서로 1장 드로우 한다.
// --------------------------------------------------------
// Behaviors:
// - ListenOtherEvent
// - DrawCardFromDeck
// --------------------------------------------------------
#[allow(non_snake_case)]
pub fn HM_001(card_json: &CardJson, count: i32) -> Card {
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
pub fn HM_002(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}

#[allow(non_snake_case)]
pub fn HM_003(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn HM_004(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn HM_005(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn HM_006(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn HM_007(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn HM_008(card_json: &CardJson, count: i32) -> Card {
    HM_001(card_json, count)
}
