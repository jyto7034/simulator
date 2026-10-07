use actix::{Actor, Addr, Context, Recipient, SpawnHandle};
use game_core::application::{ApplicationSnapshotKind, CoreApplication};
use serde::de::DeserializeOwned;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tracing::info;
use uuid::Uuid;

use crate::game::load_balance_actor::{messages::Deregister, LoadBalanceActor};
use crate::persistence::active_run_store::ActiveRunStore;
use crate::persistence::player_profile_store::PlayerProfilePersistence;

pub mod handlers;
pub mod messages;
pub mod session;
pub mod state;

use messages::ForceDisconnect;
use messages::PlayerGameServerMessage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BattleEventDeliveryCursor {
    pub(crate) battle_uuid: Uuid,
    pub(crate) last_sent_seq: u64,
}

pub struct PlayerGameActor {
    pub(crate) player_id: Uuid,
    pub(crate) game_core: CoreApplication,
    pub(crate) load_balance_addr: Addr<LoadBalanceActor>,
    pub(crate) active_session_id: Option<Uuid>,
    pub(crate) socket: Option<Recipient<PlayerGameServerMessage>>,
    pub(crate) session_control: Option<Recipient<ForceDisconnect>>,
    pub(crate) disconnect_timer: Option<SpawnHandle>,
    pub(crate) live_battle_tick: Option<SpawnHandle>,
    pub(crate) battle_event_cursor: Option<BattleEventDeliveryCursor>,
    pub(crate) disconnect_ttl: Duration,
    pub(crate) live_battle_tick_interval: Duration,
    pub(crate) active_run_store: Option<Arc<Mutex<ActiveRunStore>>>,
    pub(crate) active_run_revision: Option<crate::persistence::active_run_store::ActiveRunRevision>,
    pub(crate) player_profile_persistence: Option<Arc<Mutex<PlayerProfilePersistence>>>,
}

impl PlayerGameActor {
    pub fn new(
        player_id: Uuid,
        game_core: CoreApplication,
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
            battle_event_cursor: None,
            disconnect_ttl: Duration::from_secs(300),
            live_battle_tick_interval: Duration::from_millis(100),
            active_run_store: None,
            active_run_revision: None,
            player_profile_persistence: None,
        }
    }

    pub fn with_active_run_store(mut self, store: Arc<Mutex<ActiveRunStore>>) -> Self {
        self.active_run_store = Some(store);
        self
    }

    pub fn with_player_profile_persistence(
        mut self,
        persistence: Arc<Mutex<PlayerProfilePersistence>>,
    ) -> Self {
        self.player_profile_persistence = Some(persistence);
        self
    }

    pub fn with_disconnect_ttl(mut self, disconnect_ttl: Duration) -> Self {
        self.disconnect_ttl = disconnect_ttl;
        self
    }

    pub fn with_live_battle_tick_interval(mut self, interval: Duration) -> Self {
        self.live_battle_tick_interval = interval;
        self
    }

    pub(crate) fn read_snapshot<T: DeserializeOwned>(
        &self,
        kind: ApplicationSnapshotKind,
    ) -> Result<T, state::PlayerGameActorError> {
        let snapshot = self.game_core.snapshot(kind)?;
        serde_json::from_value(snapshot.read_model().as_value().clone()).map_err(|error| {
            state::PlayerGameActorError::new("serialization_failed", error.to_string())
        })
    }

    pub(crate) fn read_player_profile(
        &self,
    ) -> Result<
        game_core::game::encounter_compendium::PlayerProfileState,
        state::PlayerGameActorError,
    > {
        self.read_snapshot(ApplicationSnapshotKind::PlayerProfile)
    }

    pub(crate) fn has_active_live_battle(&self) -> bool {
        self.game_core.has_active_battle()
    }
}

impl Actor for PlayerGameActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("PlayerGameActor started for player {}", self.player_id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(persistence) = self
            .player_profile_persistence
            .as_ref()
            .filter(|_| self.game_core.battle_fault().is_none())
        {
            if let Ok(mut persistence) = persistence.lock() {
                match self.read_player_profile() {
                    Ok(profile) => {
                        persistence.merge_and_persist(profile);
                    }
                    Err(error) => {
                        tracing::warn!(code = %error.code, "profile snapshot failed at actor stop")
                    }
                }
            }
        }
        self.load_balance_addr.do_send(Deregister {
            player_id: self.player_id,
        });
    }
}
