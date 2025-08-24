use actix::{Context, Handler};
use tracing::info;

use crate::subscript::{messages::GracefulShutdown, SubScriptionManager};

impl Handler<GracefulShutdown> for SubScriptionManager {
    type Result = ();

    fn handle(&mut self, _msg: GracefulShutdown, _ctx: &mut Context<Self>) -> Self::Result {
        info!("Reconnect message received. Attempt: {}. Waiting for a delay of {:?} before next attempt.", 1, 2);
    }
}
