use crate::game::player_game_actor::PlayerGameActor;
use crate::shared::protocol::ServerMessage;
use actix::{Addr, Message};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Register {
    pub player_id: Uuid,
    pub addr: Addr<PlayerGameActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Deregister {
    pub player_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RouteToPlayer {
    pub player_id: Uuid,
    pub message: ServerMessage,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetPlayerCount;
