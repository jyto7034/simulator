use std::sync::Arc;

use crate::{
    card::{
        types::{CardType, OwnerType},
        Card,
    },
    enums::ZoneType,
    exception::GameError,
    game::Game,
};

use super::{TargetCondition, TargetCount, TargetSelector};

pub struct SingleCardSelector {
    condition: TargetCondition,
}

impl SingleCardSelector {
    pub fn new(location: ZoneType, owner: OwnerType) -> Self {
        Self {
            condition: TargetCondition {
                location: vec![location],
                owner,
                card_type: None,
                custom_filter: None,
            },
        }
    }

    pub fn with_card_type(mut self, card_type: CardType) -> Self {
        self.condition.card_type = Some(card_type);
        self
    }

    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&Card) -> bool + Send + Sync + 'static,
    {
        self.condition.custom_filter = Some(Arc::new(filter));
        self
    }
}

impl TargetSelector for SingleCardSelector {
    fn select_targets(&self, game: &Game, source: &Card) -> Result<Vec<Card>, GameError> {
        let valid_targets = self.get_valid_targets(game, source);

        if valid_targets.is_empty() {
            return Err(GameError::NoValidTargets);
        }

        // 실제 게임에서는 플레이어가 선택
        Ok(vec![valid_targets[0].clone()])
    }

    fn has_valid_targets(&self, game: &Game, source: &Card) -> bool {
        !self.get_valid_targets(game, source).is_empty()
    }

    fn get_target_count(&self) -> TargetCount {
        TargetCount::Exact(1)
    }

    fn clone_selector(&self) -> Box<dyn TargetSelector> {
        Box::new(Self {
            condition: self.condition.clone(),
        })
    }

    fn get_owner(&self) -> OwnerType {
        todo!()
    }

    fn get_locations(&self) -> Vec<ZoneType> {
        todo!()
    }

    fn is_valid_target(&self, card: &Card, game: &Game, source: &Card) -> bool {
        todo!()
    }
}
