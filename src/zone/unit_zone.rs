use crate::{
    card::{Card, cards::Cards},
    enums::{CardParam, InsertType, UUID},
    exception::Exception,
};

use super::zone::Zone;

pub struct UnitZone {
    zone_cards: Cards,
    zone_size: usize,
}

impl Zone for UnitZone {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn add_card(&mut self, card: Card, insert_type: InsertType) -> Result<(), Exception> {
        todo!()
    }

    fn get_cards(&mut self) -> &mut Cards {
        todo!()
    }

    fn remove_card(&mut self, uuid: UUID) {
        self.zone_cards.remove(CardParam::Uuid(uuid));
    }
}
