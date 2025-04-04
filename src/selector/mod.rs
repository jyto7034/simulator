use std::sync::Arc;

pub mod automatic;
pub mod complex;
pub mod mulligan;
pub mod multi;
pub mod single;

use crate::{
    card::{
        types::{CardType, OwnerType},
        Card,
    },
    enums::ZoneType,
    exception::GameError,
    game::Game,
};

pub trait TargetSelector: Send + Sync {
    fn select_targets(&self, game: &Game, source: &Card) -> Result<Vec<Card>, GameError>;
    fn has_valid_targets(&self, game: &Game, source: &Card) -> bool;
    fn get_target_count(&self) -> TargetCount;
    fn clone_selector(&self) -> Box<dyn TargetSelector>;

    fn get_valid_targets(&self, game: &Game, source: &Card) -> Vec<Card> {
        let mut valid_targets = Vec::new();

        for location in self.get_locations() {
            let cards = game.get_cards_by_player_and_zone_type(self.get_owner().into(), location);

            for card in cards {
                if self.is_valid_target(&card, game, source) {
                    valid_targets.push(card);
                }
            }
        }

        valid_targets
    }

    fn get_owner(&self) -> OwnerType;

    fn get_locations(&self) -> Vec<ZoneType>;

    fn is_valid_target(&self, card: &Card, game: &Game, source: &Card) -> bool;
}

#[derive(Clone, Copy)]
pub enum TargetCount {
    Exact(usize),
    Range(usize, usize),
    Any,
    None,
}

/// 카드 선택 조건
/// - location: 카드의 위치
/// - owner: 카드의 소유자
/// - card_type: 카드의 타입
/// - custom_filter: 카드에 대한 사용자 정의 필터
#[derive(Clone)]
pub struct TargetCondition {
    location: Vec<ZoneType>,
    owner: OwnerType,
    card_type: Option<CardType>,
    custom_filter: Option<Arc<dyn Fn(&Card) -> bool + Send + Sync>>,
}

pub enum SelectorLogic {
    And,
    Or,
    Not(Box<dyn TargetSelector>),
}
