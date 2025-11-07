pub mod bonus;
pub mod random;
pub mod shop;

use serde::{Deserialize, Serialize};

use crate::game::{
    data::shop_data::ShopType,
    enums::OrdealType,
    events::{event_selection::bonus::BonusType, EventGenerator, GeneratorContext},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSelectionOptions {
    pub options: [EventOption; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventOption {
    Shop(ShopType),
    Bonus(BonusType),
    Random(String), // 랜덤 인카운터 ID
}

pub struct EventSelectionGenerator;

impl EventGenerator for EventSelectionGenerator {
    type Output = EventSelectionOptions;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        use crate::ecs::resources::GameProgression;
        use rand::SeedableRng;

        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        let current_ordeal = ctx
            .world
            .get_resource::<GameProgression>()
            .map(|p| p.current_ordeal)
            .unwrap_or(OrdealType::Dawn);

        let pool = ctx.game_data.event_pools.get_pool(current_ordeal);

        let options = pool.choose_one_from_each(&mut rng);

        EventSelectionOptions { options }
    }
}
