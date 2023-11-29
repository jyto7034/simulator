use std::{
    cell::RefCell,
    rc::Rc,
};

pub trait IResource {
    fn increase(&mut self) -> &mut Self;

    fn decrease(&mut self) -> &mut Self;

    fn set(&mut self, cost: usize) -> &mut Self;
}

#[derive(Clone, Debug)]
pub struct Count {
    cost: usize,
    limit: usize,
}

impl Count {
    pub fn new(cost: usize, limit: usize) -> Count {
        Count { cost, limit }
    }

    pub fn get(&self) -> usize {
        self.cost
    }

    pub fn is_empty(&self) -> bool {
        self.cost == 0
    }
}

impl IResource for Count {
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
}

use crate::{
    deck::{deck::Deck, Cards},
    enums::constant::*,
    exception::exception::Exception,
    task::procedure::Procedure,
    unit::{player::Player, Cost, Mana},
    utils::*,
};

use super::TimeManager;
pub struct GameConfig {
    pub player_1: Deck,
    pub player_2: Deck,
    pub attaker: usize,
    pub name: Vec<String>,
}

pub struct Game {
    pub player_1: Option<Rc<RefCell<Player>>>,
    pub player_2: Option<Rc<RefCell<Player>>>,

    pub procedure: Option<Rc<RefCell<Procedure>>>,
    pub time: TimeManager,
}

impl Game {
    pub fn new(procedure: Option<Rc<RefCell<Procedure>>>) -> Result<Game, Exception> {
        let game = Game {
            player_1: None,
            player_2: None,
            procedure,
            time: TimeManager::new(),
        };
        Ok(game)
    }

    // 구조적 문제 있긴함..
    pub fn to_cards(
        &self,
        deck_code1: DeckCode,
        deck_code2: DeckCode,
    ) -> Result<Vec<Cards>, Exception> {
        let v_cards = match utils::parse_json_to_deck_code() {
            Ok(json) => match utils::load_card_data(json) {
                Ok(data) => {
                    // println!("{:#?}", data);
                    data
                }
                Err(err) => {
                    panic!("{err}")
                }
            },
            Err(err) => {
                panic!("{err}")
            }
        };

        Ok(vec![v_cards[0].clone(), v_cards[1].clone()])
    }

    pub fn initialize(&mut self, config: GameConfig) -> Result<(), Exception> {
        // config 로부터 플레이어의 덱을 읽어와서 플레이어 데이터를 생성함.
        let v_cards =
            self.to_cards(config.player_1.raw_deck_code, config.player_2.raw_deck_code)?;
        let cards1 = v_cards[0].clone();
        let cards2 = v_cards[1].clone();

        const ATTACKER: usize = 0;
        const DEFENDER: usize = 1;

        // 2개의 Player 객체 생성
        self.player_1 = match self.player_1 {
            Some(_) => None,
            None => Some(Rc::new(RefCell::new(Player::new(
                None,
                PlayerType::Player1,
                HeroType::Name1,
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
                HeroType::Name1,
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
        if let Some(player_1) = &self.player_1 {
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

        if let Some(player_2) = &self.player_2 {
        } else {
            return Err(Exception::PlayerInitializeFailed);
        }

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

        // 리소스와 카드 초기화해주는 부분 람다함수로 리팩토링해야함.

        if let Some(player) = &self.player_1 {
            // 코스트와 마나를 설정해줍니다.
            player.as_ref().borrow_mut().set_mana(0);
            player.as_ref().borrow_mut().set_cost(0);
            
            // v_card 을 참조하여 Deck 에 카드를 push 합니다.
            // 아래 for 에서 임시 생성된 card 변수에 기록되어 있는 count 의 값만큼 해당 카드를 Deck 에 push 한다.
            let cards = player.as_ref().borrow_mut().get_cards().v_card.clone();
            for card in cards {
                // Deck 과 Hand 의 카드 갯수 관리 방법이 서로 상이해서, Hand 방법 즉, 카드 갯수로 관리 하는 방법으로 통일함.
                for _ in 0..card.get_count().get(){
                    player
                        .as_ref()
                        .borrow_mut()
                        .get_zone(ZoneType::DeckZone)
                        .get_cards()
                        .add_card(card.clone()).expect("add_card error");
                }
            }
        }
        
        if let Some(player) = &self.player_2 {
            // 코스트와 마나를 설정해줍니다.
            player.as_ref().borrow_mut().set_mana(0);
            player.as_ref().borrow_mut().set_cost(0);
            
            // v_card 을 참조하여 Deck 에 카드를 push 합니다.
            // 아래 for 에서 임시 생성된 card 변수에 기록되어 있는 count 의 값만큼 해당 카드를 Deck 에 push 한다.
            let cards = player.as_ref().borrow_mut().get_cards().v_card.clone();
            for card in cards {
                // Deck 과 Hand 의 카드 갯수 관리 방법이 서로 상이해서, Hand 방법 즉, 카드 갯수로 관리 하는 방법으로 통일함.
                for _ in 0..card.get_count().get(){
                    player
                        .as_ref()
                        .borrow_mut()
                        .get_zone(ZoneType::DeckZone)
                        .get_cards()
                        .add_card(card.clone()).expect("add_card error");
                }
            }
        }
        Ok(())
    }

    /// 멀리건 단계를 수행합니다.
    pub fn game_step_mulligun(&mut self) -> Result<(), Exception> {
        if let (Some(player1), Some(player2))= (&self.player_1, &self.player_2){
            let cards_1 = player1.as_ref().borrow_mut().choice_card(ChoiceType::Mulligun);
            let cards_2 = player2.as_ref().borrow_mut().choice_card(ChoiceType::Mulligun);

            for item in cards_1{
                player1.as_ref().borrow_mut().get_zone(ZoneType::HandZone).add_card(item.clone()).unwrap()
            }

            for item in cards_2{
                player1.as_ref().borrow_mut().get_zone(ZoneType::HandZone).add_card(item.clone()).unwrap()
            }
        }
        Ok(())
    }

    /// 라운드를 시작합니다.
    pub fn game_step_round_start(&mut self) -> Result<(), Exception>{
        // 먼저, 시간대를 낮에서 밤으로, 밤에서 낮으로 변경함.
        self.time.change_time();

        
        
        // 각 player 의 자원을 충전하고 각자의 덱에서 카드를 한 장 드로우 함.
        if let (Some(player1), Some(player2))= (&self.player_1, &self.player_2){
            player1.as_ref().borrow_mut().get_cost().set(5);
            player2.as_ref().borrow_mut().get_cost().set(5);

            player1.as_ref().borrow_mut().draw(ZoneType::DeckZone, CardDrawType::Top)?;
            player1.as_ref().borrow_mut().draw(ZoneType::DeckZone, CardDrawType::Top)?;
        }

        // 그런 뒤, 필드 카드의 효과를 발동함.

        // 필드 카드의 효과가 끝나면, 필드에 전개 되어 있는 카드의 효과를 발동함.
        todo!()
        
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
