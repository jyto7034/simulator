use crate::enums::PlayerType;

#[derive(Clone)]
pub struct TurnManager{
    current_turn: PlayerType,
    turn_count: usize,
}

impl TurnManager {
    pub fn new() -> Self {
        TurnManager {
            current_turn: PlayerType::Player1,
            turn_count: 0
        }
    }

    pub fn get_turn_count(&self) -> usize{
        self.turn_count
    }

    pub fn increase_turn_count(&mut self) -> usize{
        self.turn_count += 1;
        self.turn_count
    }

    pub fn current_turn(&self) -> &PlayerType {
        &self.current_turn
    }

    pub fn change_turn(&mut self) -> &PlayerType {
        self.current_turn = match self.current_turn {
            PlayerType::Player1 => PlayerType::Player2,
            PlayerType::Player2 => PlayerType::Player1,
            PlayerType::None => panic!("PlayerType is None"),
        };
        &self.current_turn
    }

    pub fn is_player1_turn(&self) -> bool {
        self.current_turn == PlayerType::Player1
    }

    pub fn is_player2_turn(&self) -> bool {
        self.current_turn == PlayerType::Player2
    }

    pub fn get_opponent_turn(&self) -> PlayerType {
        match self.current_turn {
            PlayerType::Player1 => PlayerType::Player2,
            PlayerType::Player2 => PlayerType::Player1,
            PlayerType::None => panic!("PlayerType is None"),
        }
    }

    // 특정 플레이어의 턴으로 강제 설정
    pub fn set_turn(&mut self, player: PlayerType) {
        self.current_turn = player;
    }
}