use std::collections::HashMap;

use tracing::info;

use crate::{card::types::PlayerKind, game::phase::DrawPhaseStatus};

use super::phase::Phase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Aborted,
    AlreadyCancelled,
    UnexpectedGamePhase,
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
    pub player_states: HashMap<PlayerKind, PlayerState>,
}

impl GameStateManager {
    pub fn new() -> Self {
        Self {
            current_phase: GamePhase::Initial, // 초기 상태
            player_states: HashMap::new(),     // 빈 HashMap으로 시작
        }
    }

    pub fn current_phase(&self) -> GamePhase {
        self.current_phase.clone()
    }

    pub fn count_connected_players(&self) -> usize {
        // player_states에 있는 플레이어 수가 곧 연결된 플레이어 수
        self.player_states.len()
    }

    pub fn is_player_connected_by_kind(&self, player_kind: PlayerKind) -> Option<()> {
        // player_states에 해당 플레이어가 있으면 연결된 것
        self.player_states.get(&player_kind).map(|_| ())
    }

    // 플레이어 연결 시 player_states에 추가
    pub fn add_connected_player(&mut self, player: PlayerKind) {
        let mut player_state = PlayerState::new();
        player_state.connection_status = PlayerConnectionStatus::Connected;
        self.player_states.insert(player, player_state);
        info!("Player {:?} added to game state as Connected", player);
        self.check_and_transition_phase_after_connection();
    }

    // 플레이어 연결 해제 시 player_states에서 제거
    pub fn remove_connected_player(&mut self, player: PlayerKind) -> bool {
        if let Some(_) = self.player_states.remove(&player) {
            info!("Player {:?} removed from game state", player);
            true
        } else {
            info!(
                "Player {:?} was not in game state, nothing to remove",
                player
            );
            false
        }
    }

    // 기존 메서드 유지 (하위 호환성)
    pub fn update_player_connection_status(&mut self, player: PlayerKind, connected: bool) {
        if connected {
            self.add_connected_player(player);
        } else {
            self.remove_connected_player(player);
        }
    }

    fn check_and_transition_phase_after_connection(&mut self) {
        // 두 플레이어가 모두 연결되었는지 확인 (HashMap에 Player1과 Player2가 모두 있는지)
        if self.current_phase == GamePhase::Initial && self.count_connected_players() == 2 {
            self.transition_to_phase_internal(GamePhase::Mulligan);
        }
    }

    pub fn is_all_players_connected(&self) -> bool {
        // 실제로 연결된 플레이어가 2명인지 확인
        self.count_connected_players() == 2
    }

    fn are_all_players_connected_internal(&self) -> bool {
        self.count_connected_players() == 2
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
                        Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws),
                    ));
                }
            }
        }
    }

    fn all_players_mulligan_completed(&self) -> bool {
        // 연결된 모든 플레이어의 멀리건이 완료되었는지 확인
        self.player_states.len() == 2
            && self
                .player_states
                .values()
                .all(|s| s.mulligan_status == PlayerMulliganStatus::Completed)
    }
}
