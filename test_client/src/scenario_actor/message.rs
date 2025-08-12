use crate::scenario_actor::ScenarioResult;
use actix::Message;

/// SingleScenarioActor ScenarioRunnerActor
#[derive(Message)]
#[rtype(result = "()")]
pub struct ScenarioCompleted {
    pub scenario_id: uuid::Uuid,
    pub result: ScenarioResult,
}

/// PlayerActor SingleScenarioActor
#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerCompleted {
    pub player_id: uuid::Uuid,
    pub result: crate::BehaviorResult,
}
