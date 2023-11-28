use crate::deck::{Card, Cards};
use crate::enums::constant::CardType;
use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::Entity;
use crate::zone::Zone;

pub struct HandZone {
    pub zone_cards: Cards,
    zone_size: usize,
}
impl Zone for HandZone {
    /// 현재 Zone 에 존재하는 모든 카드를 반환합니다.
    fn get_cards(&mut self) -> &mut Cards {
        &mut self.zone_cards
    }

    /// 현재 Zone 에 카드를 추가 합니다.
    fn add_card(&mut self, card: Card) -> Result<(), Exception> {
        todo!()
    }

    /// 특정 카드를 현재 Zone 으로부터 삭제합니다.
    fn remove_card(&mut self, card: Card) -> Result<(), Exception> {
        todo!()
        // let prev_len = self.zone_cards.len();
        // self.zone_cards
        //     .v_card
        //     .retain(|item| item.get_uuid() == card.get_uuid());
        // if self.zone_cards.len() != prev_len {
        //     Ok(())
        // } else {
        //     Err(Exception::NothingToRemove)
        // }
    }
}

impl HandZone {
    pub fn new() -> HandZone {
        HandZone {
            zone_cards: Cards::new(&vec![]),
            zone_size: UNIT_ZONE_SIZE,
        }
    }

    pub fn test(&mut self) {
        self.zone_size = 1;
    }
}

impl Entity for HandZone {
    fn run(&self, game: &mut Game) -> Result<(), Exception> {
        todo!()
    }

    fn get_entity_type(&self) -> String {
        "Entity".to_string()
    }
}
