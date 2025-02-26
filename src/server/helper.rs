use crate::{
    card::{insert::TopInsert, types::PlayerType},
    enums::UUID,
    exception::ServerError,
    game::Game,
    zone::zone::Zone,
};

pub fn process_mulligan_completion<T: Into<PlayerType> + Copy>(
    game: &mut Game,
    player_type: T,
) -> Result<Vec<UUID>, ServerError> {
    let selected_cards = game
        .get_player_by_type(player_type.into())
        .get()
        .get_mulligan_state_mut()
        .get_select_cards();
    let cards = game.get_cards_by_uuid(selected_cards.clone());
    game.get_player_by_type(player_type.into())
        .get()
        .get_hand_mut()
        .add_card(cards, Box::new(TopInsert))
        .map_err(|_| ServerError::InternalServerError)?;
    game.get_player_by_type(player_type.into())
        .get()
        .get_mulligan_state_mut()
        .confirm_selection();
    Ok(selected_cards)
}
