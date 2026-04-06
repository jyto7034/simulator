use std::sync::Arc;

use crate::game::player_game_actor::messages::PlayerGameServerMessage;
use crate::game::player_game_actor::PlayerGameActor;
use crate::shared::protocol::ServerMessage;
use actix::{Addr, Message};
use game_core::game::data::GameDataBase;
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
#[rtype(result = "Option<Addr<PlayerGameActor>>")]
pub struct FindPlayer {
    pub player_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "Addr<PlayerGameActor>")]
pub struct GetOrCreatePlayerActor {
    pub player_id: Uuid,
    pub game_data: Arc<GameDataBase>,
    pub run_seed: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RouteToPlayer {
    pub player_id: Uuid,
    pub message: ServerMessage,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RouteToGamePlayer {
    pub player_id: Uuid,
    pub message: PlayerGameServerMessage,
}

#[derive(Message)]
#[rtype(result = "usize")]
pub struct GetPlayerCount;
