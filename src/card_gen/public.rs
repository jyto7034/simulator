use crate::{card::Card, enums::{CardType, PlayerType}, procedure::behavior::Behavior, utils::{self, json::CardJson}};

#[allow(non_snake_case)]
pub fn PB_001(card_json: &CardJson, count: usize) -> Card {
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
pub fn PB_002(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_003(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_004(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_005(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_006(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_007(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}
#[allow(non_snake_case)]
pub fn PB_008(card_json: &CardJson, count: usize) -> Card {
    PB_001(card_json, count)
}