use actix::Handler;

use crate::{game::player_game_actor::PlayerGameActor, shared::protocol::ServerMessage};

impl Handler<ServerMessage> for PlayerGameActor {
    type Result = ();

    fn handle(&mut self, _msg: ServerMessage, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
