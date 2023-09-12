use crate::enums::constant::{self, CardType};
use crate::game::Behavior;
use crate::utils::json::CardJson;

/// 카드의 행동, 정보를 정의하는 구조체 입니다.

#[derive(Clone, Debug)]
pub struct Card {
    pub card_type: constant::CardType,
    pub uuid: String,
    pub name: String,
    pub count: usize,
    behavior_table: Vec<Behavior>,
    card_json: CardJson,
}

impl Card {
    pub fn dummy() -> Card {
        Card {
            card_type: CardType::Dummy,
            uuid: "".to_string(),
            name: "dummy".to_string(),
            count: 0,
            behavior_table: vec![],
            card_json: CardJson::new(),
        }
    }

    pub fn is_dummy(&self) -> bool {
        true
    }

    pub fn execution() {}
}
