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

#[macro_export]
macro_rules! downcast_effect {
    ($effect:expr, $target_type:ty) => {
        if $effect.get_effect_type() == <$target_type>::static_effect_type() {
            if let Some(specific) = $effect.as_any().downcast_ref::<$target_type>() {
                Some(specific)
            } else {
                None
            }
        } else {
            None
        }
    };
}

pub async fn wait_for_input() -> Result<(), GameError> {
    todo!()
}
