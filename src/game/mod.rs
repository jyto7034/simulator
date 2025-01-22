pub mod turn_manager;
pub mod game_step;

use turn_manager::TurnManager;

use crate::{card::Card, enums::{phase::Phase, DeckCode, PlayerType}, exception::Exception, unit::player::Player, utils::deckcode_to_cards, zone::field::Field, OptRcRef};

pub struct GameConfig {
    /// Player's Deckcode
    pub player_1_deckcode: DeckCode,
    pub player_2_deckcode: DeckCode,

    /// 1 : Player 1,
    /// 2 : Player 2
    pub attacker: usize,
}

/// 게임의 상태를 관리/저장 하는 구조체
/// Card 로 인한 모든 변경 사항은 Task 로써 저장되며,
/// 그것을 담은 Tasks 를 Procedure 에게 전달하여 게임 결과를 계산한다.
pub struct Game {
    pub player1: OptRcRef<Player>,
    pub player2: OptRcRef<Player>,
    pub current_phase: Phase,
    pub turn: TurnManager,
}

/// initialize 함수에 GameConfig 을 넣음으로써 두 플레이어의 Cards 을 설정한다.
impl Game {
    pub fn initialize(&mut self, _config: GameConfig) -> Result<(), Exception> {
        let cards = deckcode_to_cards(_config.player_1_deckcode, _config.player_2_deckcode)?;
        todo!()
    }
}

impl Game {
    pub fn get_player(&self, player_type: &PlayerType) -> &OptRcRef<Player> {
        match player_type {
            PlayerType::Player1 => &self.player1,
            PlayerType::Player2 => &self.player2,
            PlayerType::None => todo!(),
        }
    }

    pub fn draw_card(&self, player_type: &PlayerType) -> Result<(), Exception>{
        todo!()
    }
}

impl Game {    
    pub fn get_player_field_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_player_field_cards")
    }

    pub fn get_opponent_field_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_opponent_field_cards")
    }

    pub fn get_player_hand_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_player_hand_cards")
    }

    pub fn get_opponent_hand_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_opponent_hand_cards")
    }

    pub fn get_player_graveyard_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_player_graveyard_cards")
    }

    pub fn get_opponent_graveyard_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_opponent_graveyard_cards")
    }

    pub fn get_player_deck_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_player_deck_cards")
    }

    pub fn get_opponent_deck_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_opponent_deck_cards")
    }

    pub fn get_player_removed_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_player_removed_cards")
    }

    pub fn get_opponent_removed_cards(&self, player_type: &PlayerType) -> Vec<Card> {
        todo!("Implement get_opponent_removed_cards")
    }
}