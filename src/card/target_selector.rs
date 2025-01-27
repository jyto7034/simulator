use crate::{enums::{CardLocation, ZoneType}, exception::Exception, game::Game};
use super::{types::{CardType, OwnerType, PlayerType}, Card};
use std::sync::Arc;

pub trait TargetSelector : Send + Sync{
    fn select_targets(&self, game: &Game, source: &Card) -> Result<Vec<Card>, Exception>;
    fn has_valid_targets(&self, game: &Game, source: &Card) -> bool;
    fn get_target_count(&self) -> TargetCount;
    fn clone_selector(&self) -> Box<dyn TargetSelector>;
}

pub enum TargetCount {
    Exact(usize),
    Range(usize, usize),  // min, max
    Any,
    None,
}

#[derive(Clone)]
pub struct TargetCondition {
    location: Vec<CardLocation>,
    owner: OwnerType,
    card_type: Option<CardType>,
    custom_filter: Option<Arc<dyn Fn(&Card) -> bool + Send + Sync>>,
}

pub struct SingleCardSelector {
    condition: TargetCondition,
}

impl SingleCardSelector {
    pub fn new(location: CardLocation, owner: OwnerType) -> Self {
        Self {
            condition: TargetCondition {
                location: vec![location],
                owner,
                card_type: None,
                custom_filter: None,
            }
        }
    }

    pub fn with_card_type(mut self, card_type: CardType) -> Self {
        self.condition.card_type = Some(card_type);
        self
    }

    pub fn with_filter<F>(mut self, filter: F) -> Self 
    where 
        F: Fn(&Card) -> bool + Send + Sync + 'static 
    {
        self.condition.custom_filter = Some(Arc::new(filter));
        self
    }

    fn get_valid_targets(&self, game: &Game, source: &Card) -> Vec<Card> {
        let mut valid_targets = Vec::new();

        
        // 위치별로 카드 수집
        for location in &self.condition.location {
            let cards = match location.0 {
                ZoneType::Field => self.get_field_cards(game, source),
                ZoneType::Hand => self.get_hand_cards(game, source),
                ZoneType::Graveyard => self.get_graveyard_cards(game, source),
                ZoneType::Deck => self.get_deck_cards(game, source),
                ZoneType::Effect => todo!(),
                ZoneType::None => todo!(),
            };

            // 조건에 맞는 카드 필터링
            for card in cards {
                if self.is_valid_target(&card, game, source) {
                    valid_targets.push(card);
                }
            }
        }

        valid_targets
    }

    fn is_valid_target(&self, card: &Card, game: &Game, source: &Card) -> bool {
        todo!()
    }

    // 각 위치별 카드 가져오기 함수들
    fn get_field_cards(&self, game: &Game, source: &Card) -> Vec<Card> {
        match self.condition.owner {
            OwnerType::Self_ => game.get_player_field_cards(source.get_owner().into()),
            OwnerType::Opponent => game.get_opponent_field_cards(source.get_owner().into()),
            OwnerType::Any => {
                let mut cards = game.get_player_field_cards(source.get_owner().into());
                cards.extend(game.get_opponent_field_cards(source.get_owner().into()));
                cards
            },
            OwnerType::None => Vec::new(),
        }
    }

    fn get_hand_cards(&self, game: &Game, source: &Card) -> Vec<Card> {
        match self.condition.owner {
            OwnerType::Self_ => game.get_player_hand_cards(source.get_owner().into()),
            OwnerType::Opponent => game.get_opponent_hand_cards(source.get_owner().into()),
            OwnerType::Any => {
                let mut cards = game.get_player_hand_cards(source.get_owner().into());
                cards.extend(game.get_opponent_hand_cards(source.get_owner().into()));
                cards
            },
            OwnerType::None => Vec::new(),
        }
    }

    fn get_graveyard_cards(&self, game: &Game, source: &Card) -> Vec<Card> {
        todo!()
    }
    
    fn get_deck_cards(&self, game: &Game, source: &Card) -> Vec<Card> {
        todo!()
    }
    
    fn get_removed_cards(&self, game: &Game, source: &Card) -> Vec<Card> {
        todo!()
    }
}

impl TargetSelector for SingleCardSelector {
    fn select_targets(&self, game: &Game, source: &Card) -> Result<Vec<Card>, Exception> {
        let valid_targets = self.get_valid_targets(game, source);
        
        if valid_targets.is_empty() {
            return Err(Exception::NoValidTargets);
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
}

// 다중 카드 선택기
pub struct MultiCardSelector {
    condition: TargetCondition,
    count: TargetCount,
}

// 자동 선택기 (가장 약한 카드, 가장 강한 카드 등)
pub struct AutomaticSelector {
    condition: TargetCondition,
    selection_type: AutoSelectType,
}

pub enum AutoSelectType {
    Weakest,
    Strongest,
    Random,
    All,
}

pub struct ComplexSelector {
    conditions: Vec<TargetCondition>,
    logic: SelectorLogic,
}

pub enum SelectorLogic {
    And,
    Or,
    Not(Box<dyn TargetSelector>),
}