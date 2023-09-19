use crate::deck::{Card, Cards};
use crate::exception::exception::Exception;

/// 플레이어의 덱 정보를 다루는 구조체입니다.
pub struct Deck {
    pub raw_deck_code: String,
}

impl Deck {
    /// Deck 의 멤버 변수. raw_deck_code 를 해석하여 card 객체의 집합인 cards 객체를 반환하는 함수입니다.
    pub fn to_cards(&self) -> Result<Cards, Exception> {
        let cards: Vec<Card> = vec![];

        Ok(Cards { v_card: cards })
    }

    pub fn get_hero() {}
}

pub struct PlayerDeck {}
