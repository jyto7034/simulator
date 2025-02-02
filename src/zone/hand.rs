use crate::{
    card::{cards::Cards, take::Take, Card},
    enums::{UNIT_ZONE_SIZE, UUID},
    exception::Exception,
};

use super::zone::Zone;

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
    pub fn remove_card(&mut self, _card: Card) -> Result<(), Exception> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl Zone for Hand {
    fn get_cards(&self) -> &Cards {
        todo!()
    }

    fn get_cards_mut(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: UUID) {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn add_card(
        &mut self,
        card: Card,
        insert_type: Box<dyn crate::card::insert::Insert>,
    ) -> Result<(), Exception> {
        todo!()
    }
    
    fn take_card(&mut self, take_type: Box<dyn Take>) -> Card {
        todo!()
    }
}
