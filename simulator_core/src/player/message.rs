use crate::{
    card::{cards::CardVecExt, Card},
    enums::ZoneType,
    exception::GameError,
};
use actix::{Addr, Context, Handler, Message};
use tracing::info;
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

#[derive(Message)]
#[rtype(result = "()")] // 새로 뽑은 카드 목록 또는 에러 반환
pub struct SetOpponent {
    pub opponent: Addr<PlayerActor>,
}

impl Handler<SetOpponent> for PlayerActor {
    type Result = ();

    fn handle(&mut self, msg: SetOpponent, ctx: &mut Context<Self>) -> Self::Result {
        println!(
            "PLAYER ACTOR [{:?}]: Handling SetOpponent",
            self.player_type
        );

        // 상대방 플레이어를 설정
        self.opponent = Some(msg.opponent);
    }
}

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetCardsByUuid {
    pub uuid: Vec<Uuid>,
}

impl Handler<GetCardsByUuid> for PlayerActor {
    type Result = Vec<Card>; // 카드 목록 반환

    fn handle(&mut self, msg: GetCardsByUuid, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetCardsByUuid",
            self.player_type
        );

        // UUID에 해당하는 카드 목록을 반환
        let mut result = vec![];
        for uuid in msg.uuid {
            if let Some(card) = self.get_cards().find_by_uuid(uuid) {
                result.push(card.clone());
            } else {
                return vec![]; // 카드가 없으면 빈 벡터 반환
            }
        }
        result
    }
}
