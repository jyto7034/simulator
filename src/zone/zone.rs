use std::any::Any;

use crate::{
    card::{cards::Cards, insert::Insert, Card}, enums::UUID, exception::Exception
};

pub trait Zone: Any {
    fn as_any(&mut self) -> &mut dyn Any;

    fn add_card(&mut self, card: Card, insert_type: Box<dyn Insert>) -> Result<(), Exception>;

    fn get_cards(&mut self) -> &mut Cards;

    fn remove_card(&mut self, uuid: UUID);

    fn len(&self) -> usize;
}
