use crate::{enums::DeckCode, exception::exception::Exception, utils::utils};

use super::cards::Cards;

pub fn deckcode_to_cards(code1: DeckCode, code2: DeckCode) -> Result<Vec<Cards>, Exception> {
    let v_cards = match utils::load_card_data((code1, code2)) {
        Ok(data) => data,
        Err(err) => {
            panic!("{err}")
        }
    };
    Ok(vec![v_cards[0].clone(), v_cards[1].clone()])
}
