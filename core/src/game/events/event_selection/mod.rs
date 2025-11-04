pub mod bonus;
pub mod random;
pub mod shop;

use rand::{seq::SliceRandom, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::{
    game::events::{
        event_selection::{bonus::BonusType, random::RandomEventType},
        EventGenerator,
    },
    GeneratorContext,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSelectionOptions {
    pub options: [EventOption; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventOption {
    Shop,
    Bonus(BonusType),
    Random(RandomEventType),
}

pub struct EventSelectionGenerator;

impl EventGenerator for EventSelectionGenerator {
    type Output = EventSelectionOptions;

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        let mut rng = rand::rngs::StdRng::seed_from_u64(ctx.random_seed);

        // Random 이벤트의 경우 가짓수가 수 십개가 될 수 있음.
        // 때문에 대책을 세워야함.
        let pool = vec![
            EventOption::Shop,
            EventOption::Bonus(BonusType::Gold),
            EventOption::Bonus(BonusType::Experience),
            EventOption::Bonus(BonusType::Item),
        ];

        let mut selected = pool
            .choose_multiple(&mut rng, 3)
            .cloned()
            .collect::<Vec<_>>();

        EventSelectionOptions {
            options: [
                selected.pop().unwrap(),
                selected.pop().unwrap(),
                selected.pop().unwrap(),
            ],
        }
    }
}
