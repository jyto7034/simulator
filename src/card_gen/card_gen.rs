use crate::deck::Card;

pub struct CardGenertor {}

impl CardGenertor {
    pub fn gen_card_by_id(id: String) -> Card {
        Card::dummy()
    }
}
