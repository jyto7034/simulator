use actix::{Actor, Addr, AsyncContext, Handler};
use tracing::{info, warn};

use super::messages::*;
use super::LoadBalanceActor;
use crate::game::player_game_actor::PlayerGameActor;

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

impl Handler<FindPlayer> for LoadBalanceActor {
    type Result = Option<Addr<PlayerGameActor>>;

    fn handle(&mut self, msg: FindPlayer, _ctx: &mut Self::Context) -> Self::Result {
        self.players.get(&msg.player_id).cloned()
    }
}

impl Handler<GetOrCreatePlayerActor> for LoadBalanceActor {
    type Result = Addr<PlayerGameActor>;

    fn handle(&mut self, msg: GetOrCreatePlayerActor, ctx: &mut Self::Context) -> Self::Result {
        if let Some(existing) = self.players.get(&msg.player_id) {
            return existing.clone();
        }

        info!("Creating PlayerGameActor for player {}", msg.player_id);
        let player_profile = self
            .player_profile_persistence
            .as_ref()
            .and_then(|persistence| persistence.lock().ok()?.profile().ok())
            .unwrap_or_default();
        let mut game_core =
            game_core::application::CoreApplication::new(msg.game_data, msg.run_seed);
        #[cfg(feature = "runtime-observation")]
        if let Some(observation) = &self.observation {
            observation.attach(&mut game_core);
        }
        game_core
            .merge_player_profile(game_core::application::ApplicationDocument::new(
                serde_json::to_value(player_profile).expect("validated profile serialization"),
            ))
            .expect("stored player profile was validated before server publication");
        let mut actor = PlayerGameActor::new(msg.player_id, game_core, ctx.address());
        if let Some(store) = self.active_run_store.clone() {
            actor = actor.with_active_run_store(store);
        }
        if let Some(persistence) = self.player_profile_persistence.clone() {
            actor = actor.with_player_profile_persistence(persistence);
        }
        let actor = actor.start();
        self.players.insert(msg.player_id, actor.clone());
        actor
    }
}

impl Handler<RouteToGamePlayer> for LoadBalanceActor {
    type Result = ();

    fn handle(&mut self, msg: RouteToGamePlayer, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(addr) = self.players.get(&msg.player_id) {
            addr.do_send(
                crate::game::player_game_actor::messages::PushServerMessage {
                    message: msg.message,
                },
            );
        } else {
            warn!("Unity player {} not found in LoadBalancer", msg.player_id);
        }
    }
}

impl Handler<GetPlayerCount> for LoadBalanceActor {
    type Result = usize;

    fn handle(&mut self, _msg: GetPlayerCount, _ctx: &mut Self::Context) -> Self::Result {
        self.players.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::player_game_actor::messages::GetEncounterCompendiumProfile;
    use game_core::game::data::GameDataBuilder;
    use game_core::game::encounter_compendium::EncounterCompendiumProfileState;
    use uuid::Uuid;

    fn empty_game_data() -> game_core::game::data::ValidatedGameData {
        GameDataBuilder::empty().build_arc().into()
    }

    #[actix_web::test]
    async fn get_or_create_player_actor_returns_same_actor_for_same_player() {
        let load_balance = LoadBalanceActor::new().start();
        let player_id = Uuid::new_v4();
        let game_data = empty_game_data();

        let first = load_balance
            .send(GetOrCreatePlayerActor {
                player_id,
                game_data: game_data.clone(),
                run_seed: 11,
            })
            .await
            .expect("first get_or_create should succeed");

        let second = load_balance
            .send(GetOrCreatePlayerActor {
                player_id,
                game_data,
                run_seed: 22,
            })
            .await
            .expect("second get_or_create should succeed");

        assert_eq!(first, second);
        assert_eq!(
            load_balance
                .send(GetPlayerCount)
                .await
                .expect("player count query should succeed"),
            1
        );
    }

    #[actix_web::test]
    async fn different_transport_players_share_local_install_player_profile() {
        let directory = tempfile::tempdir().unwrap();
        let active_store = crate::persistence::active_run_store::ActiveRunStore::open(
            directory.path().join("active"),
        )
        .unwrap();
        let profile_store = crate::persistence::player_profile_store::PlayerProfileStore::open(
            directory.path().join("profile"),
        )
        .unwrap();
        let mut persistence =
            crate::persistence::player_profile_store::PlayerProfilePersistence::bootstrap(
                profile_store,
            );
        let second_player_id = Uuid::new_v4();
        let game_data = game_core::game::data::ValidatedGameData::load_live_embedded()
            .expect("embedded live gameplay data should load");
        let abnormality_id = game_data
            .abnormality_data
            .items
            .first()
            .expect("live abnormality catalog")
            .id
            .clone();
        let profile = EncounterCompendiumProfileState {
            registered_ids: std::collections::BTreeSet::from([abnormality_id]),
        };
        assert!(matches!(
            persistence.merge_and_persist(profile.clone()),
            crate::persistence::player_profile_store::PlayerProfilePersistOutcome::Saved
        ));
        let load_balance = LoadBalanceActor::with_persistence(active_store, persistence).start();

        let actor = load_balance
            .send(GetOrCreatePlayerActor {
                player_id: second_player_id,
                game_data,
                run_seed: 99,
            })
            .await
            .expect("actor recreation should succeed");
        let restored = actor
            .send(GetEncounterCompendiumProfile)
            .await
            .expect("profile query should succeed");

        assert_eq!(restored.unwrap(), profile);
    }
}
