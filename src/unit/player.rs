use std::cell::RefCell;
use std::rc::Rc;

use crate::deck::Cards;
use crate::enums::constant;
use crate::unit::entity::Entity;
use crate::unit::Hero;

/// 플레이어를 행동, 상태 등을 다루는 구조체 입니다.
pub struct Player {
    pub opponent: Option<Rc<RefCell<Player>>>,
    pub hero: constant::HeroType,
    pub cards: Cards,
}

impl Entity for Player {
    fn get_entity_type(&self) -> String {
        "Player".to_string()
    }
}

impl Player {}
