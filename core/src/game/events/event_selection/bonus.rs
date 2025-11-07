use serde::{Deserialize, Serialize};

use crate::game::events::{
    EventError, EventExecutor, EventGenerator, ExecutorContext, GeneratorContext,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BonusType {
    Gold,
    Experience,
    Item,
    Abnormality,
}

pub struct Bonus {}

pub struct BonusGenerator;

impl EventGenerator for BonusGenerator {
    type Output = Bonus;

    fn generate(&self, _ctx: &GeneratorContext) -> Self::Output {
        todo!()
    }
}

pub struct BonusExecutor;

impl EventExecutor for BonusExecutor {
    type Input = ();

    fn execute(&self, _ctx: &ExecutorContext, _input: Self::Input) -> Result<(), EventError> {
        todo!()
    }
}
