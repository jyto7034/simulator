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
        let actor = PlayerGameActor::new(
            msg.player_id,
            game_core::game::world::GameCore::new(msg.game_data, msg.run_seed),
            ctx.address(),
        )
        .start();
        self.players.insert(msg.player_id, actor.clone());
        actor
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
    use crate::shared::metrics::MetricsCtx;
    use game_core::game::data::{
        abnormality_data::AbnormalityDatabase,
        artifact_data::ArtifactDatabase,
        bonus_data::BonusDatabase,
        equipment_data::EquipmentDatabase,
        event_pools::{EventPhasePool, EventPoolConfig},
        pve_data::PveEncounterDatabase,
        random_event_data::RandomEventDatabase,
        shop_data::ShopDatabase,
        skill_data::SkillDatabase,
        GameDataBase, GameDataBaseParts,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn empty_game_data() -> Arc<GameDataBase> {
        let empty = EventPhasePool {
            shops: vec![],
            bonuses: vec![],
            random_events: vec![],
        };

        Arc::new(GameDataBase::new(GameDataBaseParts {
            abnormality_data: Arc::new(AbnormalityDatabase::new(vec![])),
            artifact_data: Arc::new(ArtifactDatabase::new(vec![])),
            equipment_data: Arc::new(EquipmentDatabase::new(vec![])),
            shop_data: Arc::new(ShopDatabase::new(vec![])),
            bonus_data: Arc::new(BonusDatabase::new(vec![])),
            random_event_data: Arc::new(RandomEventDatabase::new(vec![])),
            pve_data: Arc::new(PveEncounterDatabase::new(vec![])),
            skill_data: Arc::new(SkillDatabase::new(vec![])),
            event_pools: EventPoolConfig {
                dawn: empty.clone(),
                noon: empty.clone(),
                dusk: empty.clone(),
                midnight: empty.clone(),
                white: empty,
            },
        }))
    }

    #[actix_web::test]
    async fn get_or_create_player_actor_returns_same_actor_for_same_player() {
        let load_balance = LoadBalanceActor::new(Arc::new(MetricsCtx::new())).start();
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
}
