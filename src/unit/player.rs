use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::deck::{cards, Card, Cards};
use crate::enums::constant::*;
use crate::exception::exception::Exception;
use crate::game::{Game, IResource};
use crate::unit::entity::Entity;
use crate::zone::{DeckZone, GraveyardZone, HandZone, Zone};

pub struct Cost {
    cost: usize,
    limit: usize,
}

impl Cost {
    pub fn new(cost: usize, limit: usize) -> Cost {
        Cost { cost, limit }
    }
}

pub struct Mana {
    cost: usize,
    limit: usize,
}

impl Mana {
    pub fn new(cost: usize, limit: usize) -> Mana {
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

    fn set(&mut self, cost: usize) -> &mut Self {
        self.cost = cost;
        self
    }

    fn add(&mut self, cost: usize) {
        self.cost += cost;
    }

    fn get(&self) -> usize {
        self.cost
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

    fn set(&mut self, cost: usize) -> &mut Self {
        self.cost = cost;
        self
    }

    fn add(&mut self, cost: usize) {
        self.cost += cost;
    }

    fn get(&self) -> usize {
        self.cost
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
    // 주어진 파라미터에 따라 draw 합니다.
    // 만약 count 가 해당 Zone 이 갖고 있는 카드의 갯수를 초과한다면
    // Zone 이 갖고 있는 만큼만 return 합니다.
    // --------------------------------------------------------
    // TODO:
    //  - [?] 객체 단위 관리로 변경해야함.
    //  - draw 의 Result 제대로 처리해야함.
    // --------------------------------------------------------
    pub fn draw(
        &mut self,
        zone_type: ZoneType,
        draw_type: CardDrawType,
    ) -> Result<Vec<Card>, Exception> {
        // 전처리 해야됨. 아마도

        self.get_zone(zone_type).get_cards().draw(draw_type)
    }

    // --------------------------------------------------------
    // ChoiceType 에 따라 처리합니다.
    // --------------------------------------------------------
    // Parameters:
    // --------------------------------------------------------
    // Exceptions:
    // --------------------------------------------------------
    pub fn choice_card(&mut self, choice_type: ChoiceType) -> Vec<Card> {
        match choice_type {
            ChoiceType::Mulligun => {
                match self.draw(ZoneType::DeckZone, CardDrawType::Random(4)) {
                    Ok(mut mulligun_cards) => {
                        // TODO !!
                        // 먼저 뽑혀진 카드를 클라이언트에게 전송합니다.

                        // TODO !!
                        // 클라이언트로부터 선택된 카드들의 uuid 정보를 받습니다. 임의로 0 설정.
                        let selected_cards = vec![mulligun_cards.get(0).unwrap().clone()];

                        // mulligun_cards.retain(|item| selected_cards.contains(&item));
                        //
                        for selected_card in &selected_cards {
                            for (idx, item) in mulligun_cards.clone().iter().enumerate() {
                                if item.cmp(CardParam::Card(selected_card.clone())) {
                                    mulligun_cards.remove(idx);
                                    break;
                                }
                            }
                        }
                        // 선택된 카드들을 다시 랜덤으로 넣습니다.
                        for card_to_put in selected_cards.iter() {
                            self.get_zone(ZoneType::DeckZone)
                                .add_card(card_to_put.clone())
                                .expect("add_card error");
                        }

                        match self.draw(
                            ZoneType::DeckZone,
                            CardDrawType::Random(selected_cards.len()),
                        ) {
                            Ok(mut new_mulligun_cards) => {
                                new_mulligun_cards.append(&mut mulligun_cards);
                                return new_mulligun_cards;
                            }
                            Err(_) => panic!("choice_card draw error"),
                        }
                    }
                    Err(_) => todo!(),
                }
            }
            ChoiceType::Target => todo!(),
        }
    }

    pub fn add_card(&mut self, zone_type: ZoneType, count: Option<i32>, card: Card) {
        self.get_zone(zone_type)
            .as_mut()
            .get_cards()
            .add_card(card)
            .expect("add_card error");
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

    pub fn get_cost(&mut self) -> &mut Cost {
        &mut self.cost
    }

    pub fn get_mana(&mut self) -> &mut Mana {
        &mut self.mana
    }

    pub fn get_zone(&mut self, zone_type: ZoneType) -> Box<&mut dyn Zone> {
        match zone_type {
            ZoneType::HandZone => Box::new(&mut self.hand_zone),
            ZoneType::DeckZone => Box::new(&mut self.deck_zone),
            ZoneType::GraveyardZone => Box::new(&mut self.graveyard_zone),
            _ => panic!("Unknown Zone"),
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

    pub fn set_cost(&mut self, cost: usize) {
        self.cost.set(cost);
    }

    pub fn set_mana(&mut self, cost: usize) {
        self.mana.set(cost);
    }
}
