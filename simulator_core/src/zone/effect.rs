use uuid::Uuid;

use crate::{
    card::{cards::Cards, take::Take, Card},
    enums::UNIT_ZONE_SIZE,
    exception::GameError,
};

use super::zone::Zone;

#[derive(Clone)]
pub struct Effect {
    zone_cards: Cards,
    zone_size: usize,
}

impl Effect {
    pub fn new() -> Effect {
        Effect {
            zone_cards: Cards::new(),
            zone_size: UNIT_ZONE_SIZE,
        }
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    pub fn remove_card(&mut self, _card: Card) -> Result<(), GameError> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl Zone for Effect {
    fn get_cards(&self) -> &Cards {
        todo!()
    }

    fn get_cards_mut(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: Uuid) {
        todo!()
    }

    fn add_card(
        &mut self,
        cards: Vec<Card>,
        insert: Box<dyn crate::card::insert::Insert>,
    ) -> Result<(), GameError> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn take_card(&mut self, take_type: Box<dyn Take>) -> Result<Vec<Card>, GameError> {
        todo!()
    }
}
