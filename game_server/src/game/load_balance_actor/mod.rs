use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, Context};
use tracing::info;
use uuid::Uuid;

use crate::game::player_game_actor::PlayerGameActor;
use crate::persistence::active_run_store::ActiveRunStore;
use crate::persistence::player_profile_store::PlayerProfilePersistence;

pub mod handlers;
pub mod messages;

pub struct LoadBalanceActor {
    #[cfg(feature = "runtime-observation")]
    observation: Option<crate::observability::Observability>,
    players: HashMap<Uuid, Addr<PlayerGameActor>>,
    active_run_store: Option<Arc<Mutex<ActiveRunStore>>>,
    player_profile_persistence: Option<Arc<Mutex<PlayerProfilePersistence>>>,
    approved_host: Option<Arc<game_core::mod_validation::ApprovedHost>>,
    approved_persistence: Option<Arc<crate::mod_admission::ApprovedPersistence>>,
}

impl LoadBalanceActor {
    #[cfg(feature = "runtime-observation")]
    pub fn with_observation(
        mut self,
        observation: Option<crate::observability::Observability>,
    ) -> Self {
        self.observation = observation;
        self
    }

    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            #[cfg(feature = "runtime-observation")]
            observation: None,
            active_run_store: None,
            player_profile_persistence: None,
            approved_host: None,
            approved_persistence: None,
        }
    }

    pub fn with_active_run_store(store: ActiveRunStore) -> Self {
        Self {
            players: HashMap::new(),
            #[cfg(feature = "runtime-observation")]
            observation: None,
            active_run_store: Some(Arc::new(Mutex::new(store))),
            player_profile_persistence: None,
            approved_host: None,
            approved_persistence: None,
        }
    }

    pub fn with_persistence(
        active_run_store: ActiveRunStore,
        mut player_profile_persistence: PlayerProfilePersistence,
    ) -> Self {
        player_profile_persistence
            .recover_pending_application(&active_run_store)
            .expect(
                "pending application transaction recovery must succeed before serving sessions",
            );
        Self {
            players: HashMap::new(),
            #[cfg(feature = "runtime-observation")]
            observation: None,
            active_run_store: Some(Arc::new(Mutex::new(active_run_store))),
            player_profile_persistence: Some(Arc::new(Mutex::new(player_profile_persistence))),
            approved_host: None,
            approved_persistence: None,
        }
    }
    pub fn with_approved_persistence(host: Arc<game_core::mod_validation::ApprovedHost>, persistence: crate::mod_admission::ApprovedPersistence) -> Self {
        let persistence = Arc::new(persistence);
        Self {
            players: HashMap::new(),
            #[cfg(feature = "runtime-observation")]
            observation: None,
            active_run_store: Some(Arc::clone(&persistence.active)),
            player_profile_persistence: Some(Arc::clone(&persistence.profile)),
            approved_host: Some(host), approved_persistence: Some(persistence),
        }
    }
}

impl Actor for LoadBalanceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LoadBalanceActor started");
    }
}
