use crate::{card::Card, enums::{CardType, PlayerType}, procedure::behavior::Behavior, utils::{self, json::CardJson}};


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
        let uuid = match utils::generate_uuid() {
            Ok(data) => data,
            Err(err) => {
                panic!("test func failed {err}");
            }
        };
        let mut bvs = vec![];
        bvs.push(Behavior::DrawCardFromDeck);
        let name = if let Some(name) = &card_json.name {
            name
        } else {
            panic!("Card creating error");
        };
        Card::new(
            CardType::Unit,
            uuid,
            name.clone(),
            bvs,
            card_json.clone(),
            PlayerType::None,
        )
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