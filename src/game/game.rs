use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub trait IResource {
    fn increase(&mut self) -> &mut Self;

    fn decrease(&mut self) -> &mut Self;

    fn set(&mut self, cost: usize) -> &mut Self;
}

#[derive(Clone)]
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
    pub fn game_step_mulligun(&mut self) -> Result<(), Exception> {
        // player 을 언래핑 합니다.
        match (&self.player_1, &self.player_2) {
            (Some(player1), Some(player2)) => {
                // player1 의 deck 에서 랜덤한 카드 4장을 뽑습니다.
                let mullugun_cards_1 = player1
                    .as_ref()
                    .borrow_mut()
                    .draw(ZoneType::DeckZone, CardDrawType::Random, 4)
                    .ok();

                // player2 의 deck 에서 랜덤한 카드 4장을 뽑습니다.
                let mullugun_cards_2 = player2
                    .as_ref()
                    .borrow_mut()
                    .draw(ZoneType::DeckZone, CardDrawType::Random, 4)
                    .ok();

                // mullugun_cards 들을 언래핑합니다.
                match (mullugun_cards_1, mullugun_cards_2) {
                    (Some(cards_1), Some(cards_2)) => {
                        // mullugun_cards 들을 클라이언트들에게 보냅니다.

                        // 클라이언트들로부터 peak_card 정보를 받습니다.
                        // peak_card 는 멀리건에서 선택된 카드들의 집합입니다.
                        // 받은 정보를 토대로, 선택된 카드를 제외한 나머지는 다시 deck 에 넣습니다.
                        // 위 과정은 peak_card_put_back() 함수에서 처리합니다.
                        // 그리고 함수로부터 peak_card 를 반환받아, cards1, cards2 라는 변수들을 만들어 반환합니다.
                        let cards1 = player1
                            .as_ref()
                            .borrow_mut()
                            .peak_card_put_back(cards_1.clone())
                            .ok();
                        let cards2 = player2
                            .as_ref()
                            .borrow_mut()
                            .peak_card_put_back(cards_2.clone())
                            .ok();

                        // 선택된 카드들을 각 플레이어의 손패에 넣습니다.
                        match (cards1, cards2) {
                            (Some(cards1), Some(cards2)) => {
                                // cards1 를 순회하며 원본 카드를 가져와, clone 으로 손패에 넣습니다.
                                let action = |player: &Rc<RefCell<Player>>, cards: Vec<UUID>| {
                                    for card in cards {
                                        let card_origin = player1
                                            .as_ref()
                                            .borrow_mut()
                                            .get_cards()
                                            .search(FindType::FindByUUID(card), 1);
                                        player
                                            .as_ref()
                                            .borrow_mut()
                                            .get_zone(ZoneType::HandZone)
                                            .get_cards()
                                            .push(card_origin.get(0).unwrap().clone());
                                    }
                                };

                                action(player1, cards1);
                                action(player2, cards2);
                            }
                            _ => return Err(Exception::CardError),
                        } // end of (cards1, cards2)
                    }
                    _ => return Err(Exception::CardError),
                } // end of (mullugun_cards_1, mullugun_cards_2)
            }
            _ => return Err(Exception::PlayerDataNotIntegrity),
        }; // end of (&self.player_1, &self.player_2)

        Ok(())
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
