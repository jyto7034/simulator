use uuid::Uuid;

use crate::{card::types::PlayerType, exception::GameError};

use super::Game;

impl Game {
    pub fn restore_then_reroll_mulligan_cards<T: Into<PlayerType>>(
        &mut self,
        player_type: T,
        exclude_cards: Vec<Uuid>,
    ) -> Result<Vec<Uuid>, GameError> {
        let player_type = player_type.into();
        self.restore_card(player_type, &exclude_cards)?;
        let new_cards = self.get_mulligan_cards(player_type, exclude_cards.len())?;
        Ok(new_cards)
    }
}
