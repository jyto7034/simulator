use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::deck::{self, Card, Cards, Deck, card};
use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::Game;
use crate::unit::entity::Entity;
use crate::zone::{graveyard_zone, DeckZone, GraveyardZone, HandZone, Zone};

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
    name: String,
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
            hand_zone: HandZone::new(),
            deck_zone: DeckZone::new(),
            graveyard_zone: GraveyardZone::new(),
        }
    }

    fn _peak_card(&self, cards: Vec<UUID>) -> UUID {
        // 수정해야됨.
        cards.get(0).unwrap().clone()
    }

    // --------------------------------------------------------
    // 파라미터로 넘어온 Vec<Card> 에서 카드 하나를 선택 후 나머지를 다시 패에 넣습니다.
    // --------------------------------------------------------
    // Exceptions:
    // - 카드가 4장이 아닌, 3장 이하일 때, 혹은 아예 없을 때.
    // - 카드가 게임에서 삭제 당했을때?
    // --------------------------------------------------------
    pub fn peak_card_put_back(
        &mut self,
        mullugun_cards: Vec<String>,
    ) -> Result<UUID, Exception> {
        // 각 mullugun_cards 에서 카드 한 장을 뽑습니다.
        let peaked_card = self._peak_card(mullugun_cards.clone());

        // 나머지 카드를 추립니다.
        let remainder_cards: Vec<_> = mullugun_cards
            .iter()
            .filter(|card| card != &&peaked_card)
            .collect();

        // 나머지 카드들의 uuid 로 player 의 DeckZone 에서 원본 카드를 찾아내어, count 를 증가시킵니다.
        for item in remainder_cards{
            if let Some(card) = self.deck_zone.get_cards().v_card.iter_mut().find(|card| card.get_uuid() == item){
                let count = card.get_count();
                card.set_count(count + 1);
            }
        }
        
        Ok(peaked_card)
    }

    pub fn draw(
        &mut self,
        zone_type: ZoneType,
        draw_type: CardDrawType,
        count: usize,
    ) -> Result<Vec<UUID>, Exception> {
        // Zone 에 존재하는 카드의 uuid 를 count 만큼 꺼내옵니다.

        // zone_type 에 해당하는 Zone 의 카드를 가져옵니다
        let card_uuid: Vec<String> = self
            .get_zone(zone_type)
            .as_mut()
            .get_cards()
            .v_card
            .iter()
            .map(|card| card.get_uuid().clone())
            .collect();

        if card_uuid.len() == 0 {
            return Err(Exception::NoCardsLeft);
        }

        let mut ans = vec![];

        // 덱에 있는 모든 카드를 순회 합니다.
        for card in &mut self.cards.v_card {
            // hand 에서 draw 한 카드들의 uuid 를 가져옵니다.
            for hand_card_uuid in &card_uuid {
                // hand 에서 가져온 카드의 uuid 를 현재 순회중인 덱 카드와 동일한지 확인합니다.
                if hand_card_uuid == card.get_uuid() {
                    // 또한 해당 카드의 count 가 0 이 아닌지 확인합니다.
                    if card.get_count() != 0 {
                        // 기존의 count 를 저장하여 덱 카드의 count 를 수정합니다.
                        let count = card.get_count();
                        card.set_count(count - 1);

                        // 최종적으로 반환될 vec 에 카드를 넣습니다.
                        ans.push(card.get_uuid().clone());
                        break;
                    }
                }
            }
        }

        Ok(ans)
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

    pub fn get_zone(&mut self, zone_type: ZoneType) -> Box<&mut dyn Zone> {
        match zone_type {
            ZoneType::HandZone => Box::new(&mut self.hand_zone),
            ZoneType::DeckZone => todo!(),
            ZoneType::GraveyardZone => todo!(),
            ZoneType::FieldZone => todo!(),
            ZoneType::None => todo!(),
        }
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
