use serde::{Deserialize, Serialize};

use crate::game::{
    enums::RiskLevel,
    events::{EventError, EventExecutor, EventGenerator, ExecutorContext, GeneratorContext},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomEventType {
    // 랜덤 몬스터 출현
    PvE,
    // 아티팩트/장비 업그레이드
    Upgrade,
    // 아티팩트/장비 특수 효과 인챈트
    Enchant,
    // 무료 아이템, 기물 등 제공
    Obtain,
}

// 구조가 조금 이상함.
// 나중에 마저 다듬기.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEvent {
    pub id: String,
    pub text: String,
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventData {
    pub event_type: RandomEventType,
    pub description: String,
    pub choices: Vec<RandomEvent>,
}

pub struct RandomEventGenerator;

impl EventGenerator for RandomEventGenerator {
    type Output = RandomEventData;

    fn generate(&self, _ctx: &GeneratorContext) -> Self::Output {
        todo!()
    }
}

pub struct RandomEventExecutor;

impl EventExecutor for RandomEventExecutor {
    type Input = String;

    fn execute(&self, _ctx: &ExecutorContext, _input: Self::Input) -> Result<(), EventError> {
        todo!()
    }
}
