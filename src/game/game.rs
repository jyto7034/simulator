use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    deck::{deck::Deck, Card, Cards},
    enums::constant::{self, CardDrawType, PlayerType},
    exception::exception::Exception,
    task::procedure::Procedure,
    unit::{player::Player, Cost, Mana},
};

use super::TimeManager;
pub struct GameConfig {
    pub player_1: Deck,
    pub player_2: Deck,
    pub attaker: u32,
    pub name: Vec<String>,
}

pub struct Game {
    pub player_1: Option<Rc<RefCell<Player>>>,
    pub player_2: Option<Rc<RefCell<Player>>>,

    pub procedure: Option<Option<Weak<RefCell<Procedure>>>>,
    pub time: TimeManager,
}

impl Game {
    pub fn new(procedure: Option<Weak<RefCell<Procedure>>>) -> Result<Game, Exception> {
        let game = Game {
            player_1: None,
            player_2: None,
            procedure: Some(procedure),
            time: TimeManager::new(),
        };
        Ok(game)
    }

    pub fn initialize(&mut self, config: GameConfig) -> Result<(), Exception> {
        // config 로부터 플레이어의 덱을 읽어와서 플레이어 데이터를 생성함.
        let cards1 = match config.player_1.to_cards() {
            Ok(data) => data,
            Err(_) => Cards::dummy(),
        };

        if cards1.empty() {
            return Err(Exception::DeckParseError);
        }

        let cards2 = match config.player_2.to_cards() {
            Ok(data) => data,
            Err(_) => Cards::dummy(),
        };

        if cards2.empty() {
            return Err(Exception::DeckParseError);
        }

        // cards1, 2 를 game 의 cards 에 넣어야함.

        const ATTACKER: usize = 0;
        const DEFENDER: usize = 1;

        // 2개의 Player 객체 생성
        self.player_1 = match self.player_1 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player::new(
                None,
                PlayerType::Player1,
                constant::HeroType::Name1,
                cards1,
                String::clone(&config.name[ATTACKER]),
                Cost::new(0, 0),
                Mana::new(0, 0),
            )))),
        };

        self.player_2 = match self.player_2 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player::new(
                None,
                PlayerType::Player2,
                constant::HeroType::Name1,
                cards2,
                String::clone(&config.name[DEFENDER]),
                Cost::new(0, 0),
                Mana::new(0, 0),
            )))),
        };

        // opponent 설정
        if let Some(player_1) = &self.player_1 {
            player_1
                .as_ref()
                .borrow_mut()
                .set_opponent(&Some(Rc::downgrade(self.player_2.as_ref().unwrap())));
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

        if let Some(player_2) = &self.player_2 {
            player_2
                .as_ref()
                .borrow_mut()
                .set_opponent(&Some(Rc::downgrade(self.player_1.as_ref().unwrap())));
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

        Ok(())
    }

    fn check_player_data_integrity(&self) -> Result<(), Exception> {
        if let Some(player1) = self.player_1.as_ref() {
            if let Some(_) = player1.borrow().get_opponent().as_ref() {
                Ok(())
            } else {
                Err(Exception::PlayerDataNotIntegrity)
            }
        } else {
            Err(Exception::PlayerDataNotIntegrity)
        }
    }

    /// 게임의 초입 부분입니다.
    pub fn game_step_initialize(&mut self) -> Result<(), Exception> {
        // 데이터 무결성을 확인합니다.
        self.check_player_data_integrity()?;

        // 코스트와 마나를 설정해줍니다.
        if let Some(player) = &self.player_1 {
            player.as_ref().borrow_mut().set_mana(0);
            player.as_ref().borrow_mut().set_cost(0);
        }

        if let Some(player) = &self.player_1 {
            player.as_ref().borrow_mut().set_mana(0);
            player.as_ref().borrow_mut().set_cost(0);
        }
        Ok(())
    }

    /// 멀리건 단계를 수행합니다.
    pub fn game_step_mulligun(&mut self) {
        if let Some(player) = &self.player_1 {
            let card = player
                .as_ref()
                .borrow_mut()
                .get_hand_zone()
                .draw(CardDrawType::Top, Some(1 as usize));
        }
        // 각 player 의 덱에서 카드 4장을 뽑음.

        // player 가 선택한 카드를 핸드에 넣고, 나머지는 덱에 무작위로 넣음.
    }

    /// 라운드를 시작합니다.
    pub fn game_step_round_start(&mut self) {
        // 먼저, 시간대를 낮에서 밤으로, 밤에서 낮으로 변경함.

        // 각 player 의 자원을 충전하고 각자의 덱에서 카드를 한 장 드로우 함.

        // 그런 뒤, 필드 카드의 효과를 발동함.

        // 필드 카드의 효과가 끝나면, 필드에 전개 되어 있는 카드의 효과를 발동함.
    }

    /// 공격 턴을 수행합니다.
    pub fn game_step_round_attack_turn(&mut self) {
        loop {
            // 카드 전개

            // 공격 버튼
        }
    }

    /// 방어 턴을 수행합니다.
    pub fn game_step_round_defense_turn(&mut self) {}

    /// 모든 턴을 끝내고, 모든 카드를 수행하고 라운드를 종료합니다.
    pub fn game_step_round_end(&mut self) {}

    pub fn next_step() {}
}
