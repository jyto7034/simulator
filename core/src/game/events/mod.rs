use crate::GeneratorContext;

pub mod event_selection;
pub mod ordeal_battle;
pub mod suppression;

pub trait EventGenerator {
    type Output;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output;
}

pub trait EventExecutor {
    type Input;

    fn execute(&self, ctx: &GeneratorContext, input: Self::Input) -> Result<(), EventError>;
}

#[derive(Debug)]
pub enum EventError {
    InvalidSelection,
    InsufficientResources,
}
