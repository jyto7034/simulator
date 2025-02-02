use crate::{
    card::{types::{CardType, OwnerType, PlayerType}, Card},
    enums::{CardLocation, ZoneType},
    exception::Exception,
    game::Game, zone::zone::Zone,
};

use super::{TargetCondition, TargetCount, TargetSelector};

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

impl TargetSelector for MulliganSelector {
    fn select_targets(&self, game: &Game, source: &Card) -> Result<Vec<Card>, Exception> {
        let valid_targets = self.get_valid_targets(game, source);

        if valid_targets.is_empty() {
            return Ok(vec![]); // 교체할 카드가 없어도 됨
        }

        // 실제 게임에서는 플레이어가 선택
        Ok(valid_targets)
    }

    fn has_valid_targets(&self, game: &Game, source: &Card) -> bool {
        !self.get_valid_targets(game, source).is_empty()
    }

    fn get_target_count(&self) -> TargetCount {
        self.count.clone()
    }

    fn clone_selector(&self) -> Box<dyn TargetSelector> {
        Box::new(Self {
            condition: self.condition.clone(),
            count: self.count.clone(),
        })
    }

    fn get_owner(&self) -> OwnerType {
        self.condition.owner
    }

    fn get_locations(&self) -> Vec<CardLocation> {
        self.condition.location.clone()
    }

    fn is_valid_target(&self, card: &Card, game: &Game, _source: &Card) -> bool {
        let location = self.condition.location.get(0).unwrap().0;
        assert_eq!(location, ZoneType::Deck);

        game.get_cards_by_player_and_zone_type(
            self.condition.owner.into(),
            location,
        )
        .contains(card)
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
        if PlayerType::Player1 == self.selector.condition.owner {
            game.get_player().get_mut().get_deck().take_card(uuid)
        }
        else if PlayerType::Player2 ==self.selector.condition.owner{

        }
    }

    pub fn select_cards(&mut self, _game: &Game, cards: Vec<Card>) -> Result<(), Exception> {
        if self.player_ready {
            return Err(Exception::InvalidOperation);
        }

        // 카드 선택 검증
        if let TargetCount::Exact(count) = self.selector.count{
            if cards.len() > count {
                // 예시: 최대 5장까지
                return Err(Exception::InvalidTargetCount);
            }
        }else{
            return Err(Exception::InvalidOperation);
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
