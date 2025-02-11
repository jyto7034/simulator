use crate::{exception::GameError, game::Game, selector::TargetSelector, zone::zone::Zone};

use super::{types::StatType, Card};

pub trait Effect: Send + Sync {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), GameError>;
    fn can_activate(&self, game: &Game, source: &Card) -> bool;
    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError>;
}

pub struct DrawEffect {
    pub count: usize,
}

impl Effect for DrawEffect {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), GameError> {
        for _ in 0..self.count {
            game.draw_card(source.get_owner().into())?;
        }
        Ok(())
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        game.get_player_by_type(source.get_owner())
            .get()
            .get_deck()
            .len()
            >= self.count
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError> {
        todo!()
    }
}

pub struct ModifyStatEffect {
    pub stat_type: StatType,
    pub amount: i32,
    pub target_selector: Box<dyn TargetSelector>,
}

impl Effect for ModifyStatEffect {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), GameError> {
        let targets = self.target_selector.select_targets(game, source)?;
        for mut target in targets {
            target.modify_stat(self.stat_type, self.amount)?;
        }
        Ok(())
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        self.target_selector.has_valid_targets(game, source)
    }

    fn clone_effect(&self) -> Result<Box<dyn Effect>, GameError> {
        todo!()
    }
}
