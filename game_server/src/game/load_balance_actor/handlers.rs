use actix::Handler;
use tracing::{info, warn};

use super::messages::*;
use super::LoadBalanceActor;

impl Handler<Register> for LoadBalanceActor {
    type Result = ();

    fn handle(&mut self, msg: Register, _ctx: &mut Self::Context) -> Self::Result {
        info!("Registering player {}", msg.player_id);
        self.players.insert(msg.player_id, msg.addr);
        // 메트릭 업데이트 가능
    }
}

impl Handler<Deregister> for LoadBalanceActor {
    type Result = ();

    fn handle(&mut self, msg: Deregister, _ctx: &mut Self::Context) -> Self::Result {
        info!("Deregistering player {}", msg.player_id);
        self.players.remove(&msg.player_id);
    }
}

impl Handler<RouteToPlayer> for LoadBalanceActor {
    type Result = ();

    fn handle(&mut self, msg: RouteToPlayer, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(addr) = self.players.get(&msg.player_id) {
            addr.do_send(msg.message);
        } else {
            warn!("Player {} not found in LoadBalancer", msg.player_id);
        }
    }
}

impl Handler<GetPlayerCount> for LoadBalanceActor {
    type Result = usize;

    fn handle(&mut self, _msg: GetPlayerCount, _ctx: &mut Self::Context) -> Self::Result {
        self.players.len()
    }
}
