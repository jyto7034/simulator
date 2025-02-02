use crate::{
    card::{cards::Cards, take::Take, Card},
    enums::{DECK_ZONE_SIZE, UUID},
    exception::Exception,
};

use super::zone::Zone;

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
    pub fn remove_card(&mut self, _card: Card) -> Result<(), Exception> {
        // 카드 관리 방법 변경에 따라, 재작성해야함.
        todo!();
    }
}

impl Zone for Deck {
    fn get_cards(&self) -> &Cards {
        todo!()
    }

    fn get_cards_mut(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: UUID) {
        todo!()
    }

    fn add_card(
        &mut self,
        card: Card,
        insert_type: Box<dyn crate::card::insert::Insert>,
    ) -> Result<(), Exception> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn take_card(&mut self, take_type: Box<dyn Take>) -> Card {
        todo!()
    }
}
