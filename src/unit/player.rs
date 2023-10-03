use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::deck::Cards;
use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::unit::entity::Entity;
use crate::zone::{DeckZone, GraveyardZone, HandZone};

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

    hand_zone: HandZone,
    deck_zone: DeckZone,
    graveyard_zone: GraveyardZone,
}

impl Entity for Player {
    fn get_entity_type(&self) -> String {
        "Player".to_string()
    }
    fn run(&self) -> Result<(), Exception> {
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
            hand_zone: HandZone::new(),
            deck_zone: DeckZone::new(),
            graveyard_zone: GraveyardZone::new(),
        }
    }

    pub fn draw(
        &mut self,
        zone_type: ZoneType,
        draw_type: CardDrawType,
        count_of_card: usize,
    ) -> Result<Vec<&UUID>, Exception> {
        match zone_type {
            ZoneType::HandZone => {
                let cards = self.hand_zone.zone_cards.draw(draw_type, count_of_card);
                if !cards.is_empty() {
                    let ans: Vec<&UUID> = cards.iter().map(|card| card.get_uuid()).collect();
                    for card_uuid in ans {
                        let card = self
                            .cards
                            .search(FindType::FindByUUID(card_uuid.clone()), 1)[0];
                        let count = card.get_count();
                        if count > 0 {
                            card.set_count(count - 1);
                        }
                    }
                    Ok(ans)
                } else {
                    Err(Exception::FailedToDrawCard)
                }
            }
            ZoneType::DeckZone => todo!(),
            ZoneType::GraveyardZone => todo!(),
            ZoneType::FieldZone => todo!(),
            ZoneType::None => todo!(),
        }
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
