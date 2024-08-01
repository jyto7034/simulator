use std::any::Any;

use crate::{
    card::{card::Card, cards::Cards},
    enums::{InsertType, UUID},
    exception::exception::Exception,
};

pub trait Zone: Any {
    fn as_any(&mut self) -> &mut dyn Any;

    fn add_card(&mut self, card: Card, insert_type: InsertType) -> Result<(), Exception>;

    fn get_cards(&mut self) -> &mut Cards;

    fn remove_card(&mut self, uuid: UUID);
}
