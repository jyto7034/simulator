use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::deck::{Cards, Deck, self};
use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::entity::Entity;
use crate::zone::{DeckZone, GraveyardZone, HandZone, Zone, graveyard_zone};

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
    opponent: Option<Rc<RefCell<Player>>>,
    player_type: PlayerType,
    hero: HeroType,
    cards: Cards,
    pub name: String,
    cost: Cost,
    mana: Mana,

    hand_zone: Option<Rc<RefCell<HandZone>>>,
    deck_zone: Option<Rc<RefCell<DeckZone>>>,
    graveyard_zone: Option<Rc<RefCell<GraveyardZone>>>,
}

impl Entity for Player {
    fn get_entity_type(&self) -> String {
        "Player".to_string()
    }
    fn run(&self, game: &mut Game) -> Result<(), Exception> {
        Ok(())
    }
}

impl Player {
    pub fn new(
        opponent: Option<Rc<RefCell<Player>>>,
        player_type: PlayerType,
        hero: HeroType,
        cards: Cards,
        name: String,
        cost: Cost,
        mana: Mana,
    ) -> Player {
        Player {
            opponent,
            player_type,
            hero,
            cards,
            name,
            cost,
            mana,
            hand_zone: Some(Rc::new(RefCell::new(HandZone::new()))),
            deck_zone: Some(Rc::new(RefCell::new(DeckZone::new()))),
            graveyard_zone: Some(Rc::new(RefCell::new(GraveyardZone::new()))),
        }
    }

    pub fn draw(
        &mut self,
        zone_type: ZoneType,
        draw_type: CardDrawType,
        count: usize,
    ) -> Result<Vec<UUID>, Exception> {
        // Zone 에 존재하는 카드의 uuid 를 count 만큼 꺼내옵니다.

        // zone_type 에 해당하는 Zone 의 카드를 가져옵니다
        let d: Vec<UUID> = self.get_zone(zone_type).as_ref().unwrap().borrow_mut().get_cards().v_card.iter().map(|card| card.get_uuid().clone()).collect();
        
        Ok(d.clone())
    }

    pub fn get_opponent(&self) -> &Option<Rc<RefCell<Player>>> {
        &self.opponent
    }

    pub fn get_hero(&self) -> &HeroType {
        &self.hero
    }

    pub fn get_cards(&self) -> &Cards {
        &self.cards
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_cost(&self) -> &Cost {
        &self.cost
    }

    pub fn get_mana(&self) -> &Mana {
        &self.mana
    }

    pub fn get_zone(&mut self, zone_type: ZoneType) -> Option<Rc<RefCell<dyn Zone>>>{
        match zone_type{
            ZoneType::HandZone => Some(Rc::clone(&self.hand_zone.unwrap())),
            ZoneType::DeckZone => Some(Rc::new(RefCell::new(&self.deck_zone))),
            ZoneType::GraveyardZone => Some(Rc::new(RefCell::new(self.graveyard_zone))),
            _ => {
                None
            }
        }
    }

    pub fn get_hand_zone(&mut self) -> &mut HandZone {
        &mut self.hand_zone
    }

    pub fn get_deck_zone(&mut self) -> &mut DeckZone {
        &mut self.deck_zone
    }

    pub fn get_graveyard_zone(&mut self) -> &mut GraveyardZone {
        &mut self.graveyard_zone
    }

    // Setter 함수들
    pub fn set_opponent(&mut self, new_opponent: &Option<Weak<RefCell<Player>>>) {
        if let Some(data) = new_opponent.as_ref().unwrap().upgrade() {
            self.opponent = Some(Rc::clone(&data));
        }
    }

    pub fn set_hero(&mut self, new_hero: HeroType) {
        self.hero = new_hero;
    }

    pub fn set_cards(&mut self, new_cards: Cards) {
        self.cards = new_cards;
    }

    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn set_cost(&mut self, cost: u32) {
        self.cost.set(cost);
    }

    pub fn set_mana(&mut self, cost: u32) {
        self.mana.set(cost);
    }
}
