use crate::{exception::Exception, game::Game, zone::zone::Zone};

use super::{target_selector::TargetSelector, types::StatType, Card};

pub trait Effect: Send + Sync  {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), Exception>;
    fn can_activate(&self, game: &Game, source: &Card) -> bool;
    fn clone_effect(&self) -> Result<Box<dyn Effect>, Exception>;
}

pub struct DrawEffect {
    pub count: usize,
}

impl Effect for DrawEffect {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), Exception> {
        for _ in 0..self.count {
            game.draw_card(source.get_owner())?;
        }
        Ok(())
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        game.get_player(source.get_owner()).get().get_deck_zone().len() >= self.count
    }
    
    fn clone_effect(&self) -> Result<Box<dyn Effect>, Exception> {
        todo!()
    }
}

pub struct ModifyStatEffect {
    pub stat_type: StatType,
    pub amount: i32,
    pub target_selector: Box<dyn TargetSelector>,
}

impl Effect for ModifyStatEffect {
    fn apply(&self, game: &mut Game, source: &Card) -> Result<(), Exception> {
        let targets = self.target_selector.select_targets(game, source)?;
        for mut target in targets {
            target.modify_stat(self.stat_type, self.amount)?;
        }
        Ok(())
    }

    fn can_activate(&self, game: &Game, source: &Card) -> bool {
        self.target_selector.has_valid_targets(game, source)
    }
    
    fn clone_effect(&self) -> Result<Box<dyn Effect>, Exception> {
        todo!()
    }
}