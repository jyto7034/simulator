use actix_ws::Session;

use crate::{
    card::{insert::TopInsert, types::PlayerType},
    enums::UUID,
    exception::{MulliganError, ServerError},
    game::Game,
    serialize_error,
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

/// 에러 메시지를 전송하고, 전송 성공 여부를 반환합니다.
/// # return
/// - 성공 시 Some(())
/// - 실패 시 None
pub async fn send_error_and_check(session: &mut Session, error_msg: MulliganError) -> Option<()> {
    // 에러 메시지 직렬화
    let Ok(error_json) = serialize_error!(error_msg) else {
        return None; // 직렬화 실패
    };

    // 메시지 전송
    match session.text(error_json).await {
        Ok(_) => Some(()),
        Err(_) => None,
    }
}
