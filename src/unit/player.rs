use std::cell::RefCell;
use std::rc::Rc;

use crate::deck::Cards;
use crate::enums::constant;
use crate::unit::entity::Entity;
use crate::unit::Hero;

pub trait IResource {
    fn increase(&mut self) -> &mut Self;

    fn decrease(&mut self) -> &mut Self;

    fn set(&mut self, cost: u32) -> &mut Self;
}

pub struct Cost {
    cost: u32,
    limit: u32,
}

impl Cost {
    pub fn new(cost: u32, limit: u32) -> Cost {
        Cost { cost, limit }
    }
}

pub struct Mana {
    cost: u32,
    limit: u32,
}

impl Mana {
    pub fn new(cost: u32, limit: u32) -> Mana {
        Mana { cost, limit }
    }
}

impl IResource for Mana {
    fn increase(&mut self) -> &mut Self {
        self.cost += 1;
        self
    }

    fn decrease(&mut self) -> &mut Self {
        self.cost -= 1;
        self
    }

    fn set(&mut self, cost: u32) -> &mut Self {
        self.cost = cost;
        self
    }
}

impl IResource for Cost {
    fn increase(&mut self) -> &mut Self {
        self.cost += 1;
        self
    }

    fn decrease(&mut self) -> &mut Self {
        self.cost -= 1;
        self
    }

    fn set(&mut self, cost: u32) -> &mut Self {
        self.cost = cost;
        self
    }
}

/// 플레이어를 행동, 상태 등을 다루는 구조체 입니다.
pub struct Player {
    pub opponent: Option<Rc<RefCell<Player>>>,
    pub hero: constant::HeroType,
    pub cards: Cards,
    pub name: String,
    pub cost: Cost,
    pub mana: Mana,
}

impl Entity for Player {
    fn get_entity_type(&self) -> String {
        "Player".to_string()
    }
}

impl Player {}
