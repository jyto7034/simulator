use std::collections::HashMap;

use tracing::info;

use crate::card::types::PlayerKind;

use super::phase::Phase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Initial, // 액터 생성 직후
    WaitingForPlayers,
    Mulligan,
    PlayerTurn(PlayerKind, Phase), // 현재 턴인 플레이어와 해당 턴의 세부 페이즈
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerConnectionStatus {
    Disconnected,
    Connected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerMulliganStatus {
    NotStarted,
    CardsDealt,
    WaitingForDecision,
    Completed,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    connection_status: PlayerConnectionStatus,
    mulligan_status: PlayerMulliganStatus,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            connection_status: PlayerConnectionStatus::Disconnected,
            mulligan_status: PlayerMulliganStatus::NotStarted,
        }
    }
}

#[derive(Clone)] // 디버깅 등을 위해 필요할 수 있음
pub struct GameStateManager {
    current_phase: GamePhase,
    player_states: HashMap<PlayerKind, PlayerState>,
}

impl GameStateManager {
    pub fn new() -> Self {
        let mut player_states = HashMap::new();
        player_states.insert(PlayerKind::Player1, PlayerState::new());
        player_states.insert(PlayerKind::Player2, PlayerState::new());
        Self {
            current_phase: GamePhase::Initial, // 초기 상태
            player_states,
        }
    }

    pub fn current_phase(&self) -> GamePhase {
        self.current_phase.clone()
    }

    pub fn update_player_connection_status(&mut self, player: PlayerKind, connected: bool) {
        if let Some(state) = self.player_states.get_mut(&player) {
            state.connection_status = if connected {
                PlayerConnectionStatus::Connected
            } else {
                PlayerConnectionStatus::Disconnected
            };
            info!(
                "Player {:?} connection status updated to: {:?}",
                player, state.connection_status
            );
            self.check_and_transition_phase_after_connection();
        }
    }

    fn check_and_transition_phase_after_connection(&mut self) {
        if self.current_phase == GamePhase::Initial && self.are_all_players_connected_internal() {
            self.transition_to_phase_internal(GamePhase::Mulligan);
        }
    }

    pub fn are_all_players_connected(&self) -> bool {
        self.player_states
            .values()
            .all(|s| s.connection_status == PlayerConnectionStatus::Connected)
    }

    fn are_all_players_connected_internal(&self) -> bool {
        self.player_states
            .values()
            .all(|s| s.connection_status == PlayerConnectionStatus::Connected)
    }

    // 상태 전이 로직
    pub fn transition_to_phase(&mut self, new_phase: GamePhase) {
        info!(
            "Game phase transitioning from {:?} to {:?}",
            self.current_phase, new_phase
        );
        self.current_phase = new_phase;
        // 새 페이즈에 따른 플레이어 상태 초기화 등 추가 로직
        match self.current_phase {
            GamePhase::Mulligan => {
                for state in self.player_states.values_mut() {
                    state.mulligan_status = PlayerMulliganStatus::NotStarted;
                }
            }
            // 다른 페이즈 전환 시 초기화 로직
            _ => {}
        }
    }

    // 내부용 (self를 &mut로 받지 않음)
    fn transition_to_phase_internal(&mut self, new_phase: GamePhase) {
        info!(
            "Game phase transitioning from {:?} to {:?}",
            self.current_phase, new_phase
        );
        self.current_phase = new_phase;
        match self.current_phase {
            GamePhase::Mulligan => {
                for state in self.player_states.values_mut() {
                    state.mulligan_status = PlayerMulliganStatus::NotStarted;
                }
            }
            _ => {}
        }
    }

    pub fn get_player_mulligan_status(&self, player: PlayerKind) -> Option<PlayerMulliganStatus> {
        self.player_states
            .get(&player)
            .map(|s| s.mulligan_status.clone())
    }

    pub fn update_player_mulligan_status(
        &mut self,
        player: PlayerKind,
        new_status: PlayerMulliganStatus,
    ) {
        if let Some(state) = self.player_states.get_mut(&player) {
            info!(
                "Player {:?} mulligan status updated from {:?} to {:?}",
                player, state.mulligan_status, new_status
            );
            state.mulligan_status = new_status.clone();

            // 두 플레이어 모두 멀리건 완료 시 다음 단계로 전환하는 로직 예시
            if new_status == PlayerMulliganStatus::Completed {
                if self.all_players_mulligan_completed() {
                    // 예시: Player1의 턴, DrawPhase로 전환
                    self.transition_to_phase(GamePhase::PlayerTurn(
                        PlayerKind::Player1,
                        Phase::DrawPhase,
                    ));
                }
            }
        }
    }

    fn all_players_mulligan_completed(&self) -> bool {
        self.player_states
            .values()
            .all(|s| s.mulligan_status == PlayerMulliganStatus::Completed)
    }
}
