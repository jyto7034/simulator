use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix::{Actor, Addr, Context};
use tracing::info;
use uuid::Uuid;

use crate::game::player_game_actor::PlayerGameActor;
use crate::persistence::active_run_store::ActiveRunStore;
use crate::persistence::encounter_compendium_store::EncounterCompendiumPersistence;

pub mod handlers;
pub mod messages;

pub struct LoadBalanceActor {
    players: HashMap<Uuid, Addr<PlayerGameActor>>,
    active_run_store: Option<Arc<Mutex<ActiveRunStore>>>,
    encounter_compendium_persistence: Option<Arc<Mutex<EncounterCompendiumPersistence>>>,
}

impl LoadBalanceActor {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            active_run_store: None,
            encounter_compendium_persistence: None,
        }
    }

    pub fn with_active_run_store(store: ActiveRunStore) -> Self {
        Self {
            players: HashMap::new(),
            active_run_store: Some(Arc::new(Mutex::new(store))),
            encounter_compendium_persistence: None,
        }
    }

    pub fn with_persistence(
        active_run_store: ActiveRunStore,
        encounter_compendium_persistence: EncounterCompendiumPersistence,
    ) -> Self {
        Self {
            players: HashMap::new(),
            active_run_store: Some(Arc::new(Mutex::new(active_run_store))),
            encounter_compendium_persistence: Some(Arc::new(Mutex::new(
                encounter_compendium_persistence,
            ))),
        }
    }
}

impl Actor for LoadBalanceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("LoadBalanceActor started");
    }
}
