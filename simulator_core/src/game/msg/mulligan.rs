use actix::{Context, Handler, Message, ResponseFuture};
use uuid::Uuid;

use crate::{
    card::{cards::CardVecExt, insert::BottomInsert, take::RandomTake, types::PlayerKind, Card},
    exception::GameError,
    game::GameActor,
    player::message::{AddCardsToDeck, GetCardFromDeck},
    selector::TargetCount,
};

#[derive(Message)]
#[rtype(result = "Result<Vec<Card>, GameError>")]
pub struct RerollRequestMulliganCard {
    pub player_type: PlayerKind,
    pub cards: Vec<Uuid>,
}

impl Handler<RerollRequestMulliganCard> for GameActor {
    type Result = ResponseFuture<Result<Vec<Card>, GameError>>;

    fn handle(&mut self, msg: RerollRequestMulliganCard, _: &mut Context<Self>) -> Self::Result {
        let player_type = msg.player_type;

        let mut cards = vec![];
        let player_cards = self
            .all_cards
            .get(&player_type)
            .unwrap_or_else(|| panic!("Player cards not found for player type: {:?}", player_type));
        for uuid in msg.cards {
            if let Some(card) = player_cards.find_by_uuid(uuid.clone()) {
                cards.push(card.clone());
            } else {
                todo!()
            }
        }
        let addr = self.get_player_addr_by_kind(player_type);
        Box::pin(async move {
            addr.do_send(AddCardsToDeck {
                cards,
                insert: Box::new(BottomInsert),
            });

            addr.send(GetCardFromDeck {
                take: Box::new(RandomTake(TargetCount::Exact(5))),
            })
            .await?
        })
    }
}
