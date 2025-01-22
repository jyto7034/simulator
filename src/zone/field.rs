use crate::{
    card::{cards::Cards, Card}, enums::{UNIT_ZONE_SIZE, UUID}, exception::Exception
};

use super::zone::Zone;

pub struct Field {
    zone_cards: Cards,
    zone_size: usize,
}

impl Zone for Field {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_cards(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: UUID) {
        todo!()
    }
    
    fn add_card(&mut self, card: Card, insert_type: Box<dyn crate::card::insert::Insert>) -> Result<(), Exception> {
        todo!()
    }
    
    fn len(&self) -> usize {
        todo!()
    }
}

impl Field{
    pub fn new() -> Field {
        Field {
            zone_cards: Cards::new(),
            zone_size: UNIT_ZONE_SIZE,
        }
    }
}