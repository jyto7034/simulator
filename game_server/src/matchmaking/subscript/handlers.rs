use actix::{ActorContext, Context, Handler};
use tracing::{debug, info, warn};

use crate::{
    matchmaking::subscript::{
        messages::{Deregister, ForwardServerMessage, Register},
        SubScriptionManager,
    },
    Stop,
};

impl Handler<ForwardServerMessage> for SubScriptionManager {
    type Result = ();
    fn handle(&mut self, msg: ForwardServerMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(session_addr) = self.sessions.get(&msg.player_id) {
            session_addr.do_send(msg.message);
        } else {
            // Session may have already cleaned up during graceful shutdown
            // This is a normal race condition, not an error
            debug!(
                "Session already cleaned up for player {} - message not forwarded",
                msg.player_id
            );
        }
    }
}

impl Handler<Register> for SubScriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Register, ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} registered", msg.player_id);
        if self.sessions.contains_key(&msg.player_id) {
            warn!(
                "Player {} is already registered. Reject request.",
                msg.player_id
            );
            ctx.stop();
        }
        self.sessions.insert(msg.player_id, msg.addr);
    }
}

impl Handler<Deregister> for SubScriptionManager {
    type Result = ();
    fn handle(&mut self, msg: Deregister, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Player {} deregistered.", msg.player_id);
        let _ = self.sessions.remove(&msg.player_id);
    }
}

impl Handler<Stop> for SubScriptionManager {
    type Result = ();

    fn handle(&mut self, msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        info!(
            "Stop message received in SubScriptionManager actor. Stopping actor. {:?}",
            msg.reason
        );
        self.sessions.clear();
        ctx.stop();
    }
}
