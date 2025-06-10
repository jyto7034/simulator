use actix::{Context, Handler, Message, ResponseFuture};

use crate::{
    card::{types::PlayerKind, Card},
    enums::ZoneType,
    game::GameActor,
    player::message::{GetDeckCards, GetFieldCards, GetGraveyardCards, GetHandCards},
};

#[derive(Message)]
#[rtype(result = "Vec<Card>")]
pub struct GetPlayerZoneCards {
    pub player_type: PlayerKind,
    pub zone: ZoneType,
}

impl Handler<GetPlayerZoneCards> for GameActor {
    type Result = ResponseFuture<Vec<Card>>;

    fn handle(&mut self, msg: GetPlayerZoneCards, _: &mut Context<Self>) -> Self::Result {
        let player_type = msg.player_type;
        let zone = msg.zone;

        let addr = self.get_player_addr_by_kind(player_type);
        Box::pin(async move {
            match zone {
                ZoneType::Deck => addr.send(GetDeckCards).await,
                ZoneType::Hand => addr.send(GetHandCards).await,
                ZoneType::Field => addr.send(GetFieldCards).await,
                ZoneType::Graveyard => addr.send(GetGraveyardCards).await,
                _ => panic!("Invalid zone type: {}", zone),
            }
            .unwrap()
        })
    }
}
