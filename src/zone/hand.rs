use uuid::Uuid;

use crate::{
    card::{
        cards::{CardVecExt, Cards},
        insert::Insert,
        take::Take,
        Card,
    },
    enums::UNIT_ZONE_SIZE,
    exception::GameError,
};

use super::zone::Zone;

#[derive(Clone)]
pub struct Hand {
    zone_cards: Cards,
    zone_size: usize,
}

impl Hand {
    pub fn new() -> Hand {
        Hand {
            zone_cards: Cards::new(),
            zone_size: UNIT_ZONE_SIZE,
        }
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    pub fn remove_card(&mut self, card: Card) -> Result<(), GameError> {
        self.zone_cards
            .remove_by_uuid(card.get_uuid())
            .map(|_| ())
            .ok_or(GameError::CardNotFound)
    }
}

impl Zone for Hand {
    fn get_cards(&self) -> &Cards {
        &self.zone_cards
    }

    fn get_cards_mut(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: Uuid) {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn add_card(&mut self, cards: Vec<Card>, insert: Box<dyn Insert>) -> Result<(), GameError> {
        for card in cards {
            insert.insert(&mut self.zone_cards, card)?;
        }
        Ok(())
    }

    fn take_card(&mut self, take_type: Box<dyn Take>) -> Result<Vec<Card>, GameError> {
        todo!()
    }
}
