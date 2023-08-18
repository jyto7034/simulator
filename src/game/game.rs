use std::{cell::RefCell, rc::Rc, borrow::BorrowMut};

use crate::{
    deck::{deck::Deck, Card, Cards},
    enums::constant,
    exception::exception::Exception,
    unit::player::{self, Player},
};
pub struct GameConfig {
    pub player_1: Deck,
    pub player_2: Deck,
    pub attaker: u32,
}

pub struct Game {
    pub player_1: Option<Rc<RefCell<Player>>>,
    pub player_2: Option<Rc<RefCell<Player>>>,
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

        self.player_1 = match self.player_1 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player {
                opponent: None,
                hero: constant::HeroType::Name1,
                cards: cards1,
            }))),
        };


        self.player_2 = match self.player_2 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player {
                opponent: None,
                hero: constant::HeroType::Name1,
                cards: cards2,
            }))),
        };

        *self.player_1.unwrap().borrow_mut() = Rc::clone(self.player_2.unwrap().as_ref());
        
        Err(Exception::DeckParseError)
    }

    pub fn next_step() {}
}
