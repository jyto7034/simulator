use std::{cell::RefCell, rc::Rc};

use crate::{
    deck::{deck::Deck, Cards},
    enums::constant,
    exception::exception::Exception,
    task::procedure::Procedure,
    unit::{player::Player, Cost, IResource, Mana},
};
pub struct GameConfig {
    pub player_1: Deck,
    pub player_2: Deck,
    pub attaker: u32,
    pub name: Vec<String>,
}

pub struct Game {
    pub player_1: Option<Rc<RefCell<Player>>>,
    pub player_2: Option<Rc<RefCell<Player>>>,

    pub task: Procedure,
}

impl Game {
    pub fn initialize(&mut self, config: GameConfig) -> Result<u32, Exception> {
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

        const ATTACKER: usize = 0;
        const DEFENDER: usize = 1;

        // 2개의 Player 객체 생성
        self.player_1 = match self.player_1 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player {
                opponent: None,
                hero: constant::HeroType::Name1,
                cards: cards1,
                name: String::clone(&config.name[ATTACKER]),
                cost: Cost::new(0, 0),
                mana: Mana::new(0, 0),
            }))),
        };

        self.player_2 = match self.player_2 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player {
                opponent: None,
                hero: constant::HeroType::Name2,
                cards: cards2,
                name: String::clone(&config.name[DEFENDER]),
                cost: Cost::new(0, 0),
                mana: Mana::new(0, 0),
            }))),
        };

        // opponent 설정
        if let Some(player_1) = &self.player_1 {
            player_1.as_ref().borrow_mut().opponent =
                Some(Rc::clone(self.player_2.as_ref().unwrap()));
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

        if let Some(player_2) = &self.player_2 {
            player_2.as_ref().borrow_mut().opponent =
                Some(Rc::clone(self.player_1.as_ref().unwrap()));
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

        Ok(1)
    }

    fn check_player_data_integrity(&self) -> Result<(), Exception> {
        if let Some(player1) = self.player_1.as_ref() {
            if let Some(_) = player1.borrow().opponent.as_ref() {
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
        self.player_1
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .cost
            .set(1);
        self.player_1
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .mana
            .set(0);

        self.player_2
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .cost
            .set(1);
        self.player_2
            .as_ref()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .mana
            .set(0);
        Ok(())
    }

    /// 멀리건 단계를 수행합니다.
    pub fn game_step_mulligun(&mut self) {}

    /// 라운드를 시작합니다.
    pub fn game_step_round_start(&mut self) {}

    /// 공격 턴을 수행합니다.
    pub fn game_step_round_attack_turn(&mut self) {}

    /// 방어 턴을 수행합니다.
    pub fn game_step_round_defense_turn(&mut self) {}

    /// 모든 턴을 끝내고, 모든 카드를 수행하고 라운드를 종료합니다.
    pub fn game_step_round_end(&mut self) {}

    pub fn next_step() {}
}
