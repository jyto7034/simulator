use actix::{Actor, Context};
use tracing::info;

pub mod handler;
pub mod message;

/// match 가 결함 없이 잘 수행할 수 있도록 임의적으로 상황을 만들어고 테스트하여 더욱 견고하게 합니다.
/// 그것이 시나리오 입니다.
/// 한 시나리오에는 두 플레이어가 배정된 역할을 가지고 참여합니다.
/// 그런 시나리오들을 관리하는 Actor 입니다.
pub struct ScenarioActor {}

impl Actor for ScenarioActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ScenarioActor started");
    }
}
