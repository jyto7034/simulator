
use crate::{
    card::{cards::Cards, insert::Insert, take::Take, Card},
    enums::UUID,
    exception::Exception,
};

pub trait Zone {
    fn add_card(&mut self, card: Card, insert_type: Box<dyn Insert>) -> Result<(), Exception>;

    fn take_card(&mut self, take_type: Box<dyn Take>) -> Card;

    fn remove_card(&mut self, uuid: UUID);

    fn get_cards(&self) -> &Cards;

    fn get_cards_mut(&mut self) -> &mut Cards;

    fn len(&self) -> usize;
}
