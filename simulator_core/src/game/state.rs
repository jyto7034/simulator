//! state.rs
//!
//! 게임 시뮬레이터의 핵심 모듈
//! 이 모듈은 game와 관련된 기능을 제공합니다.

use std::collections::HashMap;

use tracing::{error, info, warn};

use crate::{
    card::types::PlayerKind,
    game::phase::{DrawPhaseStatus, Phase},
};

/// 게임의 현재 단계를 나타내는 열거형입니다.
///
/// 게임의 진행 상황을 추적하고, 각 단계에 맞는 로직을 실행하는 데 사용됩니다.
///
/// # 예시
///
/// ```
/// use simulator_core::game::state::GamePhase;
///
/// let current_phase = GamePhase::Mulligan;
///
/// match current_phase {
///     GamePhase::Mulligan => {
///         // 멀리건 단계 처리
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    /// 게임이 중단된 상태입니다.
    Aborted,
    /// 게임이 종료 중 입니다.
    Stopping,
    /// 게임이 종료 되었습니다.
    Stopped,
    /// 게임이 이미 취소된 상태입니다.
    AlreadyCancelled,
    /// 예상치 못한 게임 단계입니다.
    UnexpectedGamePhase,
    /// 게임이 초기 상태입니다.
    Initial,
    /// 플레이어들을 기다리는 상태입니다.
    WaitingForPlayers,
    /// 멀리건 단계입니다.
    Mulligan,
    /// 플레이어의 턴입니다. PlayerKind는 플레이어의 종류(Player1, Player2), Phase는 턴의 세부 단계를 나타냅니다.
    PlayerTurn(PlayerKind, Phase),
}

/// 플레이어의 연결 상태를 나타내는 열거형입니다.
///
/// 플레이어가 게임에 연결되었는지 여부를 추적하는 데 사용됩니다.
///
/// # 예시
///
/// ```
/// use simulator_core::game::state::PlayerConnectionStatus;
///
/// let connection_status = PlayerConnectionStatus::Connected;
///
/// if connection_status == PlayerConnectionStatus::Connected {
///     // 플레이어가 연결됨
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerConnectionStatus {
    /// 플레이어가 연결 해제된 상태입니다.
    Disconnected,
    /// 플레이어가 연결된 상태입니다.
    Connected,
}

/// 플레이어의 멀리건 진행 상태를 나타내는 열거형입니다.
///
/// 플레이어가 멀리건을 얼마나 진행했는지 추적하는 데 사용됩니다.
///
/// # 예시
///
/// ```
/// use simulator_core::game::state::PlayerMulliganStatus;
///
/// let mulligan_status = PlayerMulliganStatus::CardsDealt;
///
/// match mulligan_status {
///     PlayerMulliganStatus::CardsDealt => {
///         // 카드 분배 처리
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerMulliganStatus {
    /// 멀리건이 시작되지 않은 상태입니다.
    NotStarted,
    /// 카드 분배가 완료된 상태입니다.
    CardsDealt,
    /// 플레이어의 결정을 기다리는 상태입니다.
    WaitingForDecision,
    /// 멀리건이 완료된 상태입니다.
    Completed,
}

// PlayerState 구조체는 변경 없음
/// 각 플레이어의 상태 정보를 저장하는 구조체입니다.
///
/// 연결 상태, 멀리건 상태 등을 포함합니다.
///
/// # 예시
///
/// ```
/// use simulator_core::game::state::{PlayerState, PlayerConnectionStatus, PlayerMulliganStatus};
///
/// let player_state = PlayerState {
///     connection_status: PlayerConnectionStatus::Connected,
///     mulligan_status: PlayerMulliganStatus::Completed,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PlayerState {
    /// 플레이어의 연결 상태입니다.
    pub connection_status: PlayerConnectionStatus,
    /// 플레이어의 멀리건 상태입니다.
    pub mulligan_status: PlayerMulliganStatus,
}

impl PlayerState {
    /// 새로운 PlayerState 인스턴스를 생성합니다.
    ///
    /// 기본적으로 연결 해제 상태 및 멀리건 미시작 상태로 초기화됩니다.
    ///
    /// # Returns
    ///
    /// 새로운 PlayerState 인스턴스를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::PlayerState;
    ///
    /// let player_state = PlayerState::new();
    /// assert_eq!(player_state.connection_status, simulator_core::game::state::PlayerConnectionStatus::Disconnected);
    /// assert_eq!(player_state.mulligan_status, simulator_core::game::state::PlayerMulliganStatus::NotStarted);
    /// ```
    fn new() -> Self {
        PlayerState {
            connection_status: PlayerConnectionStatus::Disconnected,
            mulligan_status: PlayerMulliganStatus::NotStarted,
        }
    }
}

/// 게임 상태를 관리하는 구조체입니다.
/// 플레이어 정보, 게임 설정 등을 저장합니다.
///
/// # 예시
///
/// ```
/// use std::collections::HashMap;
/// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerConnectionStatus, PlayerMulliganStatus};
///
/// let mut game_state_manager = GameStateManager::new();
/// game_state_manager.initialize_players();
///
/// assert_eq!(game_state_manager.player_states.len(), 2);
/// ```
// TODO: 상태 관리 방식 개선
// TODO: 불변성 강화
#[derive(Clone)]
pub struct GameStateManager {
    current_phase: GamePhase,
    /// 플레이어 상태를 저장하는 HashMap입니다. PlayerKind를 키로 사용합니다.
    pub player_states: HashMap<PlayerKind, PlayerState>,
}

impl GameStateManager {
    /// 새로운 GameStateManager 인스턴스를 생성합니다.
    ///
    /// 초기 게임 단계를 Initial로 설정하고, 플레이어 상태를 저장할 HashMap을 초기화합니다.
    ///
    /// # Returns
    ///
    /// 새로운 GameStateManager 인스턴스를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, GamePhase};
    ///
    /// let game_state_manager = GameStateManager::new();
    /// assert_eq!(game_state_manager.current_phase(), GamePhase::Initial);
    /// ```
    pub fn new() -> Self {
        Self {
            current_phase: GamePhase::Initial,
            player_states: HashMap::new(),
        }
    }

    /// 플레이어들을 초기화합니다.
    ///
    /// Player1과 Player2에 대한 PlayerState를 생성하고, HashMap에 저장합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind};
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.initialize_players();
    /// assert_eq!(game_state_manager.player_states.len(), 2);
    /// ```
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

    /// 현재 게임 단계를 반환합니다.
    ///
    /// # Returns
    ///
    /// 현재 게임 단계를 나타내는 GamePhase 열거형 값을 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, GamePhase};
    ///
    /// let game_state_manager = GameStateManager::new();
    /// assert_eq!(game_state_manager.current_phase(), GamePhase::Initial);
    /// ```
    pub fn current_phase(&self) -> GamePhase {
        self.current_phase.clone()
    }

    /// 연결된 플레이어의 수를 반환합니다.
    ///
    /// # Returns
    ///
    /// 연결된 플레이어의 수를 usize 타입으로 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerConnectionStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: PlayerConnectionStatus::Connected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    /// game_state_manager.player_states.insert(PlayerKind::Player2, PlayerState { connection_status: PlayerConnectionStatus::Disconnected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    ///
    /// assert_eq!(game_state_manager.count_connected_players(), 1);
    /// ```
    pub fn count_connected_players(&self) -> usize {
        self.player_states
            .values()
            .filter(|s| s.connection_status == PlayerConnectionStatus::Connected)
            .count()
    }

    /// 특정 플레이어가 연결되었는지 여부를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `player_kind` - 확인할 플레이어의 종류 (PlayerKind).
    ///
    /// # Returns
    ///
    /// 플레이어가 연결되어 있으면 true, 그렇지 않으면 false를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerConnectionStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: PlayerConnectionStatus::Connected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    ///
    /// assert_eq!(game_state_manager.is_player_connected(PlayerKind::Player1), true);
    /// assert_eq!(game_state_manager.is_player_connected(PlayerKind::Player2), false);
    /// ```
    pub fn is_player_connected(&self, player_kind: PlayerKind) -> bool {
        self.player_states.get(&player_kind).map_or(false, |s| {
            s.connection_status == PlayerConnectionStatus::Connected
        })
    }

    /// 게임이 아직 시작되지 않았는지(연결 대기 상태인지) 확인합니다.
    pub fn is_in_waiting_phase(&self) -> bool {
        matches!(
            self.current_phase,
            GamePhase::Initial | GamePhase::WaitingForPlayers
        )
    }

    /// 플레이어의 연결 상태를 업데이트합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 연결 상태를 업데이트할 플레이어의 종류 (PlayerKind).
    /// * `connected` - 연결 여부 (true: 연결됨, false: 연결 해제됨).
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerConnectionStatus};
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.initialize_players();
    /// game_state_manager.update_player_connection_status(PlayerKind::Player1, true);
    /// assert_eq!(game_state_manager.is_player_connected(PlayerKind::Player1), true);
    /// ```
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

    /// 모든 플레이어가 연결되었는지 확인합니다.
    ///
    /// # Returns
    ///
    /// 모든 플레이어가 연결되었으면 true, 그렇지 않으면 false를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerConnectionStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: PlayerConnectionStatus::Connected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    /// game_state_manager.player_states.insert(PlayerKind::Player2, PlayerState { connection_status: PlayerConnectionStatus::Connected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    ///
    /// assert_eq!(game_state_manager.is_all_players_connected(), true);
    /// ```
    pub fn is_all_players_connected(&self) -> bool {
        // player_states에 두 플레이어가 모두 있고, 둘 다 Connected 상태인지 확인
        self.player_states.len() == 2
            && self
                .player_states
                .values()
                .all(|s| s.connection_status == PlayerConnectionStatus::Connected)
    }

    /// 게임 단계를 변경합니다.
    ///
    /// # Arguments
    ///
    /// * `new_phase` - 변경할 새로운 게임 단계 (GamePhase).
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, GamePhase};
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.transition_to_phase(GamePhase::Mulligan);
    /// assert_eq!(game_state_manager.current_phase(), GamePhase::Mulligan);
    /// ```
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

    /// 플레이어의 멀리건 상태를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 멀리건 상태를 확인할 플레이어의 종류 (PlayerKind).
    ///
    /// # Returns
    ///
    /// 플레이어의 멀리건 상태를 나타내는 PlayerMulliganStatus 열거형 값의 Option을 반환합니다.
    /// 플레이어가 존재하지 않으면 None을 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerMulliganStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: simulator_core::game::state::PlayerConnectionStatus::Disconnected, mulligan_status: PlayerMulliganStatus::Completed });
    ///
    /// assert_eq!(game_state_manager.get_player_mulligan_status(PlayerKind::Player1), Some(PlayerMulliganStatus::Completed));
    /// assert_eq!(game_state_manager.get_player_mulligan_status(PlayerKind::Player2), None);
    /// ```
    pub fn get_player_mulligan_status(&self, player: PlayerKind) -> Option<PlayerMulliganStatus> {
        self.player_states
            .get(&player)
            .map(|s| s.mulligan_status.clone())
    }

    /// 플레이어의 연결 상태를 반환합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 연결 상태를 확인할 플레이어의 종류 (PlayerKind).
    ///
    /// # Returns
    ///
    /// 플레이어의 연결 상태를 나타내는 PlayerConnectionStatus 열거형 값의 Option을 반환합니다.
    /// 플레이어가 존재하지 않으면 None을 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerConnectionStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: PlayerConnectionStatus::Connected, mulligan_status: simulator_core::game::state::PlayerMulliganStatus::NotStarted });
    ///
    /// assert_eq!(game_state_manager.get_player_connection_status(PlayerKind::Player1), Some(PlayerConnectionStatus::Connected));
    /// assert_eq!(game_state_manager.get_player_connection_status(PlayerKind::Player2), None);
    /// ```
    pub fn get_player_connection_status(
        &self,
        player: PlayerKind,
    ) -> Option<PlayerConnectionStatus> {
        self.player_states
            .get(&player)
            .map(|s| s.connection_status.clone())
    }

    /// 플레이어의 멀리건 상태를 업데이트합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 멀리건 상태를 업데이트할 플레이어의 종류 (PlayerKind).
    /// * `new_status` - 업데이트할 새로운 멀리건 상태 (PlayerMulliganStatus).
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerMulliganStatus};
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.initialize_players();
    /// game_state_manager.update_player_mulligan_status(PlayerKind::Player1, PlayerMulliganStatus::Completed);
    /// assert_eq!(game_state_manager.get_player_mulligan_status(PlayerKind::Player1), Some(PlayerMulliganStatus::Completed));
    /// ```
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

    /// 모든 플레이어가 멀리건을 완료했는지 확인합니다.
    ///
    /// # Returns
    ///
    /// 모든 플레이어가 멀리건을 완료했으면 true, 그렇지 않으면 false를 반환합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::state::{GameStateManager, PlayerKind, PlayerState, PlayerMulliganStatus};
    /// use std::collections::HashMap;
    ///
    /// let mut game_state_manager = GameStateManager::new();
    /// game_state_manager.player_states.insert(PlayerKind::Player1, PlayerState { connection_status: simulator_core::game::state::PlayerConnectionStatus::Disconnected, mulligan_status: PlayerMulliganStatus::Completed });
    /// game_state_manager.player_states.insert(PlayerKind::Player2, PlayerState { connection_status: simulator_core::game::state::PlayerConnectionStatus::Disconnected, mulligan_status: PlayerMulliganStatus::Completed });
    ///
    /// assert_eq!(game_state_manager.all_players_mulligan_completed(), true);
    /// ```
    fn all_players_mulligan_completed(&self) -> bool {
        self.player_states.len() == 2
            && self
                .player_states
                .values()
                .all(|s| s.mulligan_status == PlayerMulliganStatus::Completed)
    }
}
