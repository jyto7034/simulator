use crate::{
    card::{take::TopTake, types::{CardType, PlayerType}, Card},
    enums::{CardLocation, ZoneType},
    exception::GameError,
    game::Game, zone::zone::Zone,
};

use super::{TargetCondition, TargetCount};

pub struct MulliganSelector {
    condition: TargetCondition,
    count: TargetCount,
}

impl MulliganSelector {
    pub fn new(condition: TargetCondition, count: TargetCount) -> Self {
        Self {
            condition,
            count,
        }
    }
}

pub struct MulliganState {
    selector: MulliganSelector,
    player_ready: bool,
    selected_cards: Vec<Card>,
}

impl MulliganState {
    pub fn new(player_type: PlayerType, count: usize) -> Self {
        let condition = TargetCondition{
            location: vec![CardLocation(ZoneType::Deck)],
            owner: player_type.into(),
            card_type: Some(CardType::Any),
            custom_filter: None,
        };

        Self {
            selector: MulliganSelector::new(condition, TargetCount::Exact(count)),
            player_ready: false,
            selected_cards: Vec::new(),
        }
    }

    pub fn draw_cards(&self, game: &mut Game) -> Vec<Card>{
        if PlayerType::Player1 == self.selector.condition.owner.into() {
            game.get_player().get_mut().get_deck_mut().take_card(Box::new(TopTake(self.selector.count)))
        }
        else {
            game.get_opponent().get_mut().get_deck_mut().take_card(Box::new(TopTake(self.selector.count)))
        }
    }

    pub fn select_cards(&mut self, _game: &Game, cards: Vec<Card>) -> Result<(), GameError> {
        if self.player_ready {
            return Err(GameError::InvalidOperation);
        }

        // 카드 선택 검증
        if let TargetCount::Exact(count) = self.selector.count{
            if cards.len() > count {
                // 예시: 최대 5장까지
                return Err(GameError::InvalidTargetCount);
            }
        }else{
            return Err(GameError::InvalidOperation);
        }

        self.selected_cards = cards;
        Ok(())
    }

    pub fn confirm_selection(&mut self) {
        self.player_ready = true;
    }

    pub fn is_ready(&self) -> bool {
        self.player_ready
    }

    pub fn get_selected_cards(&self) -> &Vec<Card> {
        &self.selected_cards
    }
}
