use crate::enums::constant;

/// 카드의 행동, 정보를 정의하는 구조체 입니다.
pub struct Card {
    card_type: constant::CardType,
}

impl Card {
    // pub fn dummy() -> Card{
    //     Card {  }
    // }

    pub fn is_dummy(&self) -> bool {
        true
    }
}
