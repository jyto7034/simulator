use serde::{Deserialize, Serialize};

use crate::game::{
    enums::{OrdealType, PhaseType},
    events::{EventError, EventExecutor, EventGenerator},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressSelectionOptions {
    pub options: [SuppressOption; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressOption {}

pub struct SuppressionGenerator;

impl EventGenerator for SuppressionGenerator {
    type Output = SuppressSelectionOptions;

    fn generate(&self, ctx: &super::GeneratorContext) -> Self::Output {
        use crate::ecs::resources::GameProgression;
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // TODO: map_or 변경
        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        let current_phase = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_phase)
            .unwrap_or(PhaseType::I);

        let options = [SuppressOption {}, SuppressOption {}, SuppressOption {}];

        SuppressSelectionOptions { options }
    }
}

pub struct SuppressionExecutor;

impl EventExecutor for SuppressionExecutor {
    type Input = SuppressOption;

    fn execute(&self, ctx: &super::ExecutorContext, input: Self::Input) -> Result<(), EventError> {
        todo!()
    }
}
