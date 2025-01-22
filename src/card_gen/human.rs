use crate::{card::{effect::{DrawEffect, ModifyStatEffect}, target_selector::SingleCardSelector, types::{OwnerType, StatType}, Card}, enums::{CardLocation, ZoneType}, utils::json::CardJson};

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
    pub fn HM_001(card_json: &CardJson, count: usize) -> Card {
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
    pub fn HM_002(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }

    #[allow(non_snake_case)]
    pub fn HM_003(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }
    #[allow(non_snake_case)]
    pub fn HM_004(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }
    #[allow(non_snake_case)]
    pub fn HM_005(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }
    #[allow(non_snake_case)]
    pub fn HM_006(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }
    #[allow(non_snake_case)]
    pub fn HM_007(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }
    #[allow(non_snake_case)]
    pub fn HM_008(card_json: &CardJson, count: usize) -> Card {
        HM_001(card_json, count)
    }