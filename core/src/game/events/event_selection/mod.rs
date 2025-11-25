pub mod bonus;
pub mod random;
pub mod shop;

use crate::game::{
    enums::GameOption,
    events::{EventGenerator, GeneratorContext},
};

use self::{bonus::BonusGenerator, random::RandomEventGenerator, shop::ShopGenerator};

pub struct EventSelectionGenerator;

impl EventGenerator for EventSelectionGenerator {
    type Output = [GameOption; 3];

    fn generate(&self, ctx: &GeneratorContext) -> Self::Output {
        // 각 Generator에게 위임하여 3개의 GameOption 생성
        let shop = ShopGenerator.generate(ctx);
        let bonus = BonusGenerator.generate(ctx);
        let random = RandomEventGenerator.generate(ctx);

        [shop, bonus, random]
    }
}
