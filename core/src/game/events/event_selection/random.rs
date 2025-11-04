use serde::{Deserialize, Serialize};

use crate::game::events::{EventExecutor, EventGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomEventType {
    LowRisk,
    MediumRisk,
    HighRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventData {
    pub event_type: RandomEventType,
    pub description: String,
    pub choices: Vec<RandomEventChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomEventChoice {
    pub id: String,
    pub text: String,
    pub risk: f32, // 0.0 ~ 1.0
}

pub struct RandomEventGenerator;

impl EventGenerator for RandomEventGenerator {
    type Output = RandomEventData;

    fn generate(&self, ctx: &crate::GeneratorContext) -> Self::Output {
        todo!()
    }
}

impl EventExecutor for RandomEventGenerator {
    type Input = String;

    fn execute(
        &self,
        ctx: &crate::GeneratorContext,
        input: Self::Input,
    ) -> Result<(), crate::game::events::EventError> {
        todo!()
    }
}
