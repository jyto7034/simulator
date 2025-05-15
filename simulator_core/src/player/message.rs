use crate::{
    card::{
        cards::CardVecExt,
        insert::Insert,
        take::{RandomTake, Take},
        Card,
    },
    enums::{ZoneType, COUNT_OF_MULLIGAN_CARDS},
    exception::GameError,
    selector::TargetCount,
    zone::zone::Zone,
};
use actix::{Addr, Context, Handler, Message};
use tracing::info;
use uuid::Uuid;

use super::PlayerActor;

#[derive(Message)]
#[rtype(result = "Result<Vec<Uuid>, GameError>")]
pub struct RequestMulliganReroll {
    pub cards_to_restore: Vec<Uuid>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SetOpponent {
    pub opponent: Addr<PlayerActor>,
}

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetCardsByUuid {
    pub uuid: Vec<Uuid>,
}

#[derive(Message)]
#[rtype(result = "Result<(), GameError>")]
pub struct AddCardsToDeck {
    pub cards: Vec<Card>,
    pub insert: Box<dyn Insert>,
}

#[derive(Message)]
#[rtype(result = "Result<Vec<Card>, GameError>")]
pub struct GetCardFromDeck {
    pub take: Box<dyn Take>,
}

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetDeckCards;

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetHandCards;

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetFieldCards;

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetGraveyardCards;

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetMulliganDealCards;

impl Handler<GetMulliganDealCards> for PlayerActor {
    type Result = Vec<Card>;

    fn handle(&mut self, _: GetMulliganDealCards, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetMulliganDealCards",
            self.player_type
        );

        self.deck
            .take_card(Box::new(RandomTake(TargetCount::Exact(
                COUNT_OF_MULLIGAN_CARDS,
            ))))
            .unwrap()
            .into_iter()
            .map(|card| card.clone())
            .collect::<Vec<_>>()
    }
}

impl Handler<RequestMulliganReroll> for PlayerActor {
    type Result = Result<Vec<Uuid>, GameError>;

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

impl Handler<GetCardsByUuid> for PlayerActor {
    type Result = Vec<Card>;

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

impl Handler<AddCardsToDeck> for PlayerActor {
    type Result = Result<(), GameError>;

    fn handle(&mut self, msg: AddCardsToDeck, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling AddCardsToDeck",
            self.player_type
        );

        self.deck.add_card(msg.cards, msg.insert)
    }
}

impl Handler<GetCardFromDeck> for PlayerActor {
    type Result = Result<Vec<Card>, GameError>;

    fn handle(&mut self, msg: GetCardFromDeck, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetCardFromDeck",
            self.player_type
        );

        // 덱에서 카드를 가져옴
        let cards = self.deck.take_card(msg.take)?;
        Ok(cards)
    }
}

impl Handler<GetDeckCards> for PlayerActor {
    type Result = Vec<Card>;

    fn handle(&mut self, _: GetDeckCards, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetDeckCards",
            self.player_type
        );

        // 덱의 카드 목록을 반환
        self.deck.get_cards().clone()
    }
}

impl Handler<GetHandCards> for PlayerActor {
    type Result = Vec<Card>;

    fn handle(&mut self, _: GetHandCards, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetHandCards",
            self.player_type
        );

        // 덱의 카드 목록을 반환
        self.hand.get_cards().clone()
    }
}

impl Handler<GetFieldCards> for PlayerActor {
    type Result = Vec<Card>;

    fn handle(&mut self, _: GetFieldCards, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetFieldCards",
            self.player_type
        );

        // 덱의 카드 목록을 반환
        self.field.get_cards().clone()
    }
}

impl Handler<GetGraveyardCards> for PlayerActor {
    type Result = Vec<Card>;

    fn handle(&mut self, _: GetGraveyardCards, ctx: &mut Context<Self>) -> Self::Result {
        info!(
            "PLAYER ACTOR [{:?}]: Handling GetGraveyardCards",
            self.player_type
        );

        // 덱의 카드 목록을 반환
        self.graveyard.get_cards().clone()
    }
}
