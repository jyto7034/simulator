use uuid::Uuid;

use crate::{
    card::{cards::Cards, take::Take, Card},
    enums::UNIT_ZONE_SIZE,
    exception::GameError,
};

use super::zone::Zone;

#[derive(Clone)]
pub struct Field {
    zone_cards: Cards,
    zone_size: usize,
}

impl Zone for Field {
    fn get_cards(&self) -> &Cards {
        &self.zone_cards
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

impl Field {
    pub fn new() -> Field {
        Field {
            zone_cards: Cards::new(),
            zone_size: UNIT_ZONE_SIZE,
        }
    }
}
