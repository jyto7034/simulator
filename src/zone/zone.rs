
use crate::{
    card::{cards::Cards, insert::Insert, take::Take, Card},
    enums::UUID,
    exception::GameError,
};

pub trait Zone {
    fn add_card(&mut self, card: Card, insert_type: Box<dyn Insert>) -> Result<(), GameError>;

    fn take_card(&mut self, take_type: Box<dyn Take>) -> Vec<Card>;

    fn remove_card(&mut self, uuid: UUID);

    fn get_cards(&self) -> &Cards;

    fn get_cards_mut(&mut self) -> &mut Cards;

    fn len(&self) -> usize;
}
