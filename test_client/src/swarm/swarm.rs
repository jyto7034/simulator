use std::time::{Duration, Instant};

use actix::prelude::*;
use tracing::info;
use uuid::Uuid;

use crate::{
    behaviors::BehaviorType,
    observer_actor::{message::StartObservation, ObserverActor},
    player_actor::PlayerActor,
};

use super::{concrete::ConcreteConfig, schedule::spawn_schedule_constant, seed::uuid_for};

/// Swarm 런처: ConcreteConfig와 seed를 받아 플레이어들을 일정에 맞춰 생성/시작한다.
pub struct SwarmLauncher {
    pub match_server_url: String,
    pub test_name: String,
}

impl SwarmLauncher {
    pub fn new(match_server_url: String, test_name: String) -> Self {
        Self {
            match_server_url,
            test_name,
        }
    }

    pub fn run(&self, seed: u64, cfg: &ConcreteConfig) {
        let system = System::new();
        let match_server_url = self.match_server_url.clone();
        let test_name = self.test_name.clone();

        system.block_on(async move {
            // 1) Observer 준비 (단일 시나리오가 아닌, 전체 플레이어 이벤트 관찰용)
            let observer = ObserverActor::new(
                match_server_url,
                test_name,
                // dummy runner addr (unused in swarm mode)
                actix::Actor::create(|_ctx| {
                    crate::scenario_actor::ScenarioRunnerActor::new(vec![])
                }),
                std::collections::HashMap::new(),
                std::collections::HashMap::new(),
            );
            let observer_addr = observer.start();

            // 2) 스폰 스케줄 생성
            let schedule_ms = spawn_schedule_constant(seed, "swarm", cfg.player_count, cfg.cps, 0);

            // 3) Observer 이벤트 스트림 시작(플레이어 ID는 생성 후 추가적으로 보낼 수도 있음)
            observer_addr.do_send(StartObservation { player_ids: vec![] });

            // 4) 스케줄에 맞춰 플레이어 생성
            let t0 = Instant::now();
            for (i, ms) in schedule_ms.iter().enumerate() {
                let delay = *ms as u64;
                let observer_addr_cloned = observer_addr.clone();

                // 각 플레이어용 결정적 UUID
                let player_id: Uuid = uuid_for(seed, "player", i as u64);
                // behavior mix에서 index별로 결정적으로 할당
                let behavior: BehaviorType = crate::swarm::behavior_mix::behavior_for_index(
                    seed,
                    i as u64,
                    &cfg.behavior_mix,
                );

                actix::spawn(async move {
                    let now = Instant::now();
                    if now < t0 + Duration::from_millis(delay) {
                        tokio::time::sleep((t0 + Duration::from_millis(delay)) - now).await;
                    }

                    // 플레이어 생성 및 시작
                    let actor = PlayerActor::new(
                        observer_addr_cloned.clone(),
                        Box::new(behavior),
                        player_id,
                        true,
                    );
                    actor.start();
                });
            }

            info!(
                "Swarm launched: {} players over ~{:.2}s (cps={:.2})",
                cfg.player_count,
                (cfg.player_count as f64 / cfg.cps),
                cfg.cps
            );
        });

        system.run().unwrap();
    }
}
