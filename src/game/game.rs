use std::{cell::RefCell, rc::Rc};

use crate::{
    card::deck::deckcode_to_cards,
    enums::{DeckCode, InsertType, PlayerType, ZoneType, PLAYER_1, PLAYER_2},
    exception::exception::Exception,
    unit::player::{Player, Resoruce},
};

pub struct GameConfig {
    /// Player's Deckcode
    pub player_1_deckcode: DeckCode,
    pub player_2_deckcode: DeckCode,

    /// 1 : Player 1,
    /// 2 : Player 2
    pub attaker: usize,

    //
    pub player_1_name: String,
    pub player_2_name: String,
}

/// 게임의 상태를 관리/저장 하는 구조체
/// Card 로 인한 모든 변경 사항은 Task 로써 저장되며,
/// 그것을 담은 Tasks 를 Procedure 에게 전달하여 게임 결과를 계산한다.
#[derive(Clone)]
pub struct Game {
    pub player1: Option<Rc<RefCell<Player>>>,
    pub player2: Option<Rc<RefCell<Player>>>,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), Exception> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;

        // Player 설정
        self.player1 = Some(Rc::new(RefCell::new(Player::new(
            None,
            PlayerType::Player1,
            cards[PLAYER_1].clone(),
            _config.player_1_name.clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ))));
        self.player1 = Some(Rc::new(RefCell::new(Player::new(
            None,
            PlayerType::Player2,
            cards[PLAYER_2].clone(),
            _config.player_2_name.clone(),
            Resoruce::new(0, 3),
            Resoruce::new(0, 3),
        ))));

        // 순환 참조이긴 한데, 딱히 문제 없음. 정리만 수동적으로 잘 정리해주면 됨
        if let Some(player) = &self.player1 {
            player.as_ref().borrow_mut().opponent =
                Some(Rc::clone(&self.player2.as_ref().unwrap()));
        }
        if let Some(player) = &self.player2 {
            player.as_ref().borrow_mut().opponent =
                Some(Rc::clone(&self.player1.as_ref().unwrap()));
        }

        // DeckZone 에 카드를 clone 으로 채워넣는다.
        if let Some(player) = &self.player1 {
            let cards = player.as_ref().borrow_mut().get_cards().clone();
            for card in cards.v_card {
                player
                    .as_ref()
                    .borrow_mut()
                    .add_card(ZoneType::DeckZone, card, InsertType::Top)?;
            }
        }
        if let Some(player) = &self.player2 {
            let cards = player.as_ref().borrow_mut().get_cards().clone();
            for card in cards.v_card {
                player
                    .as_ref()
                    .borrow_mut()
                    .add_card(ZoneType::DeckZone, card, InsertType::Top)?;
            }
        }

        // cost, mana 설정
        if let Some(player) = &self.player1 {
            player.as_ref().borrow_mut().set_cost(0);
            player.as_ref().borrow_mut().set_mana(0);
        }
        if let Some(player) = &self.player2 {
            player.as_ref().borrow_mut().set_cost(0);
            player.as_ref().borrow_mut().set_mana(0);
        }

        Ok(())
    }

    pub fn get_player(&self, player_type: PlayerType) -> Rc<RefCell<Player>> {
        match player_type {
            PlayerType::Player1 => Rc::clone(&self.player1.as_ref().unwrap()),
            PlayerType::Player2 => Rc::clone(&self.player2.as_ref().unwrap()),
            PlayerType::None => todo!(),
        }
    }
}
