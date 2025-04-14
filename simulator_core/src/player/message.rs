use crate::{enums::ZoneType, exception::GameError};
use actix::{Context, Handler, Message};
use uuid::Uuid;

use super::PlayerActor;

#[derive(Message)]
#[rtype(result = "Result<Vec<Uuid>, GameError>")] // 새로 뽑은 카드 목록 또는 에러 반환
pub struct RequestMulliganReroll {
    pub cards_to_restore: Vec<Uuid>, // 덱으로 되돌릴 카드 UUID 목록
}

impl Handler<RequestMulliganReroll> for PlayerActor {
    type Result = Result<Vec<Uuid>, GameError>; // 새로 뽑은 카드 반환

    fn handle(&mut self, msg: RequestMulliganReroll, ctx: &mut Context<Self>) -> Self::Result {
        println!(
            "PLAYER ACTOR [{:?}]: Handling RequestMulliganReroll",
            self.player_type
        );

        // 카드를 복원
        let cards_to_restore = self.get_cards_by_uuids(&msg.cards_to_restore)?;
        self.restore_cards(&cards_to_restore, ZoneType::Deck)?;

        // 복원 시킨 카드 갯수 만큼 카드를 뽑음.
        let new_cards = self.get_new_mulligan_cards(
            self.player_type,
            cards_to_restore.len(), // 뽑을 카드 갯수
        )?;

        // 뽑은 카드의 갯수를 확인
        if new_cards.len() != cards_to_restore.len() {
            return Err(GameError::InternalServerError);
        }

        // 뽑은 카드를 mulligan state 에 넣음.
        self.mulligan_state.add_select_cards(new_cards.clone());

        // 뽑은 카드 목록을 반환
        Ok(new_cards)
    }
}
