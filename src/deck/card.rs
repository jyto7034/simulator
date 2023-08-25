use crate::enums::constant::{self, CardType};

/// 카드의 행동, 정보를 정의하는 구조체 입니다.
pub struct Card {
    pub card_type: constant::CardType,
    pub uuid: String,
    pub name: String,
    pub count: usize,
}

impl Card {
    pub fn dummy() -> Card {
        Card {
            card_type: CardType::Dummy,
            uuid: "".to_string(),
            name: "".to_string(),
            count: 0,
        }
    }

    pub fn is_dummy(&self) -> bool {
        true
    }
}
