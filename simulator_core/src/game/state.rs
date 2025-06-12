use std::collections::HashMap;

use tracing::{error, info, warn};

use crate::{
    card::types::PlayerKind,
    game::phase::{DrawPhaseStatus, Phase},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Aborted,
    AlreadyCancelled,
    UnexpectedGamePhase,
    Initial,
    WaitingForPlayers,
    Mulligan,
    PlayerTurn(PlayerKind, Phase),
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

// PlayerState 구조체는 변경 없음
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub connection_status: PlayerConnectionStatus,
    pub mulligan_status: PlayerMulliganStatus,
}

impl PlayerState {
    fn new() -> Self {
        PlayerState {
            connection_status: PlayerConnectionStatus::Disconnected,
            mulligan_status: PlayerMulliganStatus::NotStarted,
        }
    }
}

#[derive(Clone)]
pub struct GameStateManager {
    current_phase: GamePhase,
    pub player_states: HashMap<PlayerKind, PlayerState>,
}

impl GameStateManager {
    pub fn new() -> Self {
        Self {
            current_phase: GamePhase::Initial,
            player_states: HashMap::new(),
        }
    }

    pub fn initialize_players(&mut self) {
        self.player_states
            .insert(PlayerKind::Player1, PlayerState::new());
        self.player_states
            .insert(PlayerKind::Player2, PlayerState::new());
        info!(
            "GameStateManager initialized for players {:?} and {:?}",
            PlayerKind::Player1,
            PlayerKind::Player2
        );
    }

    pub fn current_phase(&self) -> GamePhase {
        self.current_phase.clone()
    }

    pub fn count_connected_players(&self) -> usize {
        self.player_states
            .values()
            .filter(|s| s.connection_status == PlayerConnectionStatus::Connected)
            .count()
    }

    pub fn is_player_connected(&self, player_kind: PlayerKind) -> bool {
        self.player_states.get(&player_kind).map_or(false, |s| {
            s.connection_status == PlayerConnectionStatus::Connected
        })
    }

    pub fn update_player_connection_status(&mut self, player: PlayerKind, connected: bool) {
        let new_status = if connected {
            PlayerConnectionStatus::Connected
        } else {
            PlayerConnectionStatus::Disconnected
        };

        if let Some(player_state) = self.player_states.get_mut(&player) {
            if player_state.connection_status != new_status {
                info!(
                    "Player {:?} connection status changed from {:?} to {:?}",
                    player, player_state.connection_status, new_status
                );
                player_state.connection_status = new_status;
            } else {
                warn!(
                    "Player {:?} connection status already {:?}. No change.",
                    player, new_status
                );
            }
        } else {
            error!(
                "Attempted to update connection status for an uninitialized player: {:?}",
                player
            );
            return;
        }

        if connected && self.current_phase == GamePhase::Initial && self.is_all_players_connected()
        {
            self.transition_to_phase(GamePhase::Mulligan);
        }
    }

    pub fn is_all_players_connected(&self) -> bool {
        // player_states에 두 플레이어가 모두 있고, 둘 다 Connected 상태인지 확인
        self.player_states.len() == 2
            && self
                .player_states
                .values()
                .all(|s| s.connection_status == PlayerConnectionStatus::Connected)
    }

    pub fn transition_to_phase(&mut self, new_phase: GamePhase) {
        info!(
            "Game phase transitioning from {:?} to {:?}",
            self.current_phase, new_phase
        );
        self.current_phase = new_phase.clone();

        // 새 페이즈에 따른 플레이어 상태 초기화 등 추가 로직
        match new_phase {
            GamePhase::Mulligan => {
                for state in self.player_states.values_mut() {
                    // 이미 멀리건을 완료한 플레이어의 상태는 초기화하지 않도록 방어 코드 추가 가능
                    if state.mulligan_status != PlayerMulliganStatus::Completed {
                        state.mulligan_status = PlayerMulliganStatus::NotStarted;
                    }
                }
            }
            // 다른 페이즈 전환 시 필요한 초기화 로직 추가
            _ => {}
        }
    }

    pub fn get_player_mulligan_status(&self, player: PlayerKind) -> Option<PlayerMulliganStatus> {
        self.player_states
            .get(&player)
            .map(|s| s.mulligan_status.clone())
    }

    pub fn get_player_connection_status(
        &self,
        player: PlayerKind,
    ) -> Option<PlayerConnectionStatus> {
        self.player_states
            .get(&player)
            .map(|s| s.connection_status.clone())
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
            state.mulligan_status = new_status;

            // 두 플레이어 모두 멀리건 완료 시 다음 단계로 전환
            if state.mulligan_status == PlayerMulliganStatus::Completed
                && self.all_players_mulligan_completed()
            {
                self.transition_to_phase(GamePhase::PlayerTurn(
                    PlayerKind::Player1, // 선공 플레이어 정보 필요
                    Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws),
                ));
            }
        }
    }

    fn all_players_mulligan_completed(&self) -> bool {
        self.player_states.len() == 2
            && self
                .player_states
                .values()
                .all(|s| s.mulligan_status == PlayerMulliganStatus::Completed)
    }
}
