use crate::card::types::PlayerKind;

#[derive(Clone, PartialEq, Eq)]
pub struct Turn {
    current_turn: PlayerKind,
    turn_count: usize,
}

impl Turn {
    pub fn new() -> Self {
        Turn {
            current_turn: PlayerKind::Player1,
            turn_count: 0,
        }
    }

    pub fn get_turn_count(&self) -> usize {
        self.turn_count
    }

    pub fn increase_turn_count(&mut self) -> usize {
        self.turn_count += 1;
        self.turn_count
    }

    pub fn current_turn(&self) -> PlayerKind {
        self.current_turn
    }

    pub fn change_turn(&mut self) -> PlayerKind {
        self.current_turn = match self.current_turn {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        };
        self.current_turn
    }

    pub fn is_player_turn(&self) -> bool {
        self.current_turn == PlayerKind::Player1
    }

    pub fn is_opponent_turn(&self) -> bool {
        self.current_turn == PlayerKind::Player2
    }

    pub fn get_opponent_turn(&self) -> PlayerKind {
        match self.current_turn {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        }
    }

    // 특정 플레이어의 턴으로 강제 설정
    pub fn set_turn(&mut self, player: PlayerKind) {
        self.current_turn = player;
    }
}
