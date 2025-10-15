use std::collections::HashMap;

use actix::{Actor, Addr};
use tracing::info;
use uuid::Uuid;

use crate::{
    behaviors::BehaviorType,
    observer_actor::{message::StartObservation, ObserverActor, Phase, PhaseCondition},
    player_actor::PlayerActor,
    schedules,
};

pub mod handler;
pub mod message;

#[derive(Debug, Clone)]
pub struct Scenario {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub normal_behavior: BehaviorType,
    pub abnormal_behavior: BehaviorType,
}

impl Scenario {
    /// 시나리오를 실행합니다 (ObserverActor + PlayerActor 2개 생성)
    pub fn run(self, completion_tx: Option<tokio::sync::oneshot::Sender<bool>>) -> Addr<ObserverActor> {
        info!("Starting scenario: {}", self.name);

        let test_session_id = Uuid::new_v4().to_string();
        info!(
            "Generated test_session_id for scenario {}: {}",
            self.name, test_session_id
        );

        let normal_player_id = Uuid::new_v4();
        let abnormal_player_id = Uuid::new_v4();

        let normal_schedule =
            schedules::get_schedule_for_normal(&self.normal_behavior);
        let abnormal_schedule = schedules::get_schedule_for_abnormal(&self.abnormal_behavior);
        let mut players_schedule: HashMap<Uuid, HashMap<Phase, PhaseCondition>> = HashMap::new();

        players_schedule.insert(normal_player_id, normal_schedule);
        players_schedule.insert(abnormal_player_id, abnormal_schedule);

        let mut observer = ObserverActor::new(
            "ws://127.0.0.1:8080".to_string(),
            self.name.clone(),
            test_session_id.clone(),
            players_schedule,
            HashMap::new(),
        );

        // completion_tx가 있으면 설정
        if let Some(tx) = completion_tx {
            observer = observer.with_completion_tx(tx);
        }

        let observer_addr = observer.start();

        let normal_behavior = Box::new(self.normal_behavior.clone());
        let abnormal_behavior = Box::new(self.abnormal_behavior.clone());

        let normal_actor = PlayerActor::new(
            observer_addr.clone(),
            normal_behavior,
            normal_player_id,
            test_session_id.clone(),
        );
        let abnormal_actor = PlayerActor::new(
            observer_addr.clone(),
            abnormal_behavior,
            abnormal_player_id,
            test_session_id.clone(),
        );

        normal_actor.start();
        abnormal_actor.start();

        observer_addr.do_send(StartObservation {
            player_ids: vec![normal_player_id, abnormal_player_id],
        });

        info!(
            "Created players for scenario {}: normal={}, abnormal={}, session={}",
            self.name, normal_player_id, abnormal_player_id, test_session_id
        );

        observer_addr
    }
}
