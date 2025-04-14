use uuid::Uuid;

use crate::{
    card::{cards::Cards, insert::Insert, take::Take, Card},
    enums::DECK_ZONE_SIZE,
    exception::GameError,
};

use super::zone::Zone;

#[derive(Clone)]
pub struct Deck {
    zone_cards: Cards,
    zone_size: usize,
}

impl Deck {
    pub fn new() -> Deck {
        Deck {
            zone_cards: Cards::new(),
            zone_size: DECK_ZONE_SIZE,
        }
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    /// TODO: 무슨 방식으로(eg. 랜덤, 맨 위, 맨 아래) 넣을지 구현해야함.

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    pub fn remove_card(&mut self, _card: Card) -> Result<(), GameError> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl Zone for Deck {
    fn get_cards(&self) -> &Cards {
        &self.zone_cards
    }

    fn get_cards_mut(&mut self) -> &mut Cards {
        &mut self.zone_cards
    }

    fn remove_card(&mut self, uuid: Uuid) {
        todo!()
    }

    fn add_card(&mut self, cards: Vec<Card>, insert: Box<dyn Insert>) -> Result<(), GameError> {
        for card in cards {
            insert.insert(self, card)?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn take_card(&mut self, mut take_type: Box<dyn Take>) -> Result<Vec<Card>, GameError> {
        take_type.as_mut().take(self)
    }
}
