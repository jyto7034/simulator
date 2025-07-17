use actix::{Handler, MessageResult};

use crate::player_actor::{message::GetPlayerId, PlayerActor};

impl Handler<GetPlayerId> for PlayerActor {
    type Result = MessageResult<GetPlayerId>;

    fn handle(&mut self, _msg: GetPlayerId, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.player_id)
    }
}
