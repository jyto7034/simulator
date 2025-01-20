use crate::{card::Card, enums::{CardType, PlayerType}, procedure::behavior::Behavior, utils::{self, json::CardJson}};

#[allow(non_snake_case)]
pub fn MT_001(card_json: &CardJson, count: usize) -> Card {
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