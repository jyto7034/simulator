use actix::{Actor, Addr, Context, Recipient, SpawnHandle};
use game_core::game::{resources::GameState, world::GameCore};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

use crate::game::load_balance_actor::{messages::Deregister, LoadBalanceActor};

pub mod handlers;
pub mod messages;
pub mod session;
pub mod state;

use messages::ForceDisconnect;
use messages::PlayerGameServerMessage;

pub struct PlayerGameActor {
    pub(crate) player_id: Uuid,
    pub(crate) game_core: GameCore,
    pub(crate) load_balance_addr: Addr<LoadBalanceActor>,
    pub(crate) active_session_id: Option<Uuid>,
    pub(crate) socket: Option<Recipient<PlayerGameServerMessage>>,
    pub(crate) session_control: Option<Recipient<ForceDisconnect>>,
    pub(crate) disconnect_timer: Option<SpawnHandle>,
    pub(crate) live_battle_tick: Option<SpawnHandle>,
    pub(crate) disconnect_ttl: Duration,
    pub(crate) live_battle_tick_interval: Duration,
}

impl PlayerGameActor {
    pub fn new(
        player_id: Uuid,
        game_core: GameCore,
        load_balance_addr: Addr<LoadBalanceActor>,
    ) -> Self {
        Self {
            player_id,
            game_core,
            load_balance_addr,
            active_session_id: None,
            socket: None,
            session_control: None,
            disconnect_timer: None,
            live_battle_tick: None,
            disconnect_ttl: Duration::from_secs(300),
            live_battle_tick_interval: Duration::from_millis(100),
        }
    }

    pub fn with_disconnect_ttl(mut self, disconnect_ttl: Duration) -> Self {
        self.disconnect_ttl = disconnect_ttl;
        self
    }

    pub fn with_live_battle_tick_interval(mut self, interval: Duration) -> Self {
        self.live_battle_tick_interval = interval;
        self
    }

    pub(crate) fn has_active_live_battle(&self) -> bool {
        matches!(self.game_core.get_state(), GameState::InBattle { .. })
    }
}

impl Actor for PlayerGameActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("PlayerGameActor started for player {}", self.player_id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.load_balance_addr.do_send(Deregister {
            player_id: self.player_id,
        });
    }
}
