//! turn.rs
//! 
//! 게임 시뮬레이터의 핵심 모듈
//! 이 모듈은 game와 관련된 기능을 제공합니다.

use crate::card::types::PlayerKind;

use super::phase::{DrawPhaseStatus, MulliganStatus, Phase, PlayerActionStatus};

/// `TurnState` 구조체는 게임 턴의 상태를 나타냅니다.
///
/// 이 구조체는 현재 턴 플레이어, 우선권 소유자, 턴 카운트, 현재 페이즈,
/// 각 플레이어의 행동 상태를 포함합니다. 게임 턴의 진행과 관련된 모든 정보를 관리합니다.
///
/// # Examples
///
/// ```
/// use simulator_core::game::turn::TurnState;
/// use simulator_core::card::types::PlayerKind;
/// use simulator_core::game::phase::{Phase, MulliganStatus, PlayerActionStatus};
///
/// let mut turn_state = TurnState::new(PlayerKind::Player1);
/// assert_eq!(turn_state.current_turn_plyaer, PlayerKind::Player1);
/// assert_eq!(turn_state.turn_count, 1);
/// assert_eq!(turn_state.current_phase, Phase::Mulligan(MulliganStatus::NotStarted));
/// ```
#[derive(Debug, PartialEq)]
pub struct TurnState {
    /// 현재 턴 플레이어
    pub current_turn_plyaer: PlayerKind,
    /// 현재 우선권 소유자
    pub priority_holder: PlayerKind,
    /// 턴 카운트
    pub turn_count: usize,
    /// 현재 페이즈
    pub current_phase: Phase,
    /// 턴 플레이어의 행동 상태
    pub turn_player_action_status: PlayerActionStatus,
    /// 턴이 아닌 플레이어의 행동 상태
    pub non_turn_player_action_status: PlayerActionStatus,
}

impl TurnState {
    /// 새로운 `TurnState` 인스턴스를 생성합니다.
    ///
    /// 초기 턴 플레이어, 턴 카운트, 초기 페이즈, 우선권 소유자를 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `starting_player` - 게임을 시작하는 플레이어 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    ///
    /// # Returns
    ///
    /// * `TurnState` - 새로운 `TurnState` 인스턴스
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.current_turn_plyaer, PlayerKind::Player1);
    /// assert_eq!(turn_state.turn_count, 1);
    /// ```
    pub fn new(starting_player: PlayerKind) -> Self {
        // 초기 페이즈 (예: 멀리건 또는 드로우 페이즈) 설정
        let initial_phase = Phase::Mulligan(MulliganStatus::NotStarted); // 혹은 DrawPhase
        Self {
            current_turn_plyaer: starting_player,
            turn_count: 1, // 또는 0에서 시작
            current_phase: initial_phase,
            priority_holder: starting_player, // 턴 시작 시 턴 플레이어가 우선권을 가짐
            turn_player_action_status: PlayerActionStatus::NotYetActed,
            non_turn_player_action_status: PlayerActionStatus::NotYetActed,
        }
    }

    /// 우선권을 상대방에게 넘깁니다.
    ///
    /// 현재 우선권을 가진 플레이어의 상태를 `ActedOrPassed`로 변경하고,
    /// 우선권을 상대방에게 넘깁니다. 상대방의 행동 상태를 `NotYetActed`로 초기화합니다.
    /// 양쪽 모두 `ActedOrPassed` 상태면 `attempt_to_advance_phase_or_step` 함수를 호출합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    /// use simulator_core::game::phase::{Phase, MulliganStatus, PlayerActionStatus};
    ///
    /// let mut turn_state = TurnState::new(PlayerKind::Player1);
    /// turn_state.pass_priority();
    /// assert_eq!(turn_state.priority_holder, PlayerKind::Player2);
    /// ```
    pub fn pass_priority(&mut self) {
        // 현재 우선권 가진 플레이어의 상태를 ActedOrPassed로 변경
        if self.priority_holder == self.current_turn_plyaer {
            self.turn_player_action_status = PlayerActionStatus::ActedOrPassed;
        } else {
            self.non_turn_player_action_status = PlayerActionStatus::ActedOrPassed;
        }

        // 우선권을 상대방에게 넘김
        self.priority_holder = self.priority_holder.reverse();

        // 상대방의 행동 상태를 NotYetActed로 초기화 (새로 우선권을 받았으므로)
        if self.priority_holder == self.current_turn_plyaer {
            self.turn_player_action_status = PlayerActionStatus::NotYetActed;
        } else {
            self.non_turn_player_action_status = PlayerActionStatus::NotYetActed;
        }

        // 양쪽 모두 ActedOrPassed 상태면 페이즈/스텝 진행 로직 호출 가능
        if self.turn_player_action_status == PlayerActionStatus::ActedOrPassed
            && self.non_turn_player_action_status == PlayerActionStatus::ActedOrPassed
        {
            // 여기서 다음 페이즈/스텝으로 진행하거나,
            // 또는 이 함수를 호출한 곳에서 이 상태를 확인하고 진행 로직을 실행
            self.attempt_to_advance_phase_or_step();
        }
    }

    /// 플레이어가 행동을 했을 때 호출됩니다 (예: 카드 발동).
    ///
    /// TODO: 플레이어가 행동을 했을 때 필요한 로직을 구현해야 합니다.
    /// 현재는 `todo!()` 매크로로 표시되어 있습니다.
    ///
    /// # Arguments
    ///
    /// * `acting_player` - 행동을 한 플레이어 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    pub fn player_acted(&mut self, acting_player: PlayerKind) {
        todo!()
        // self.pass_priority();
    }

    /// 페이즈/스텝 자동 진행을 시도합니다 (양쪽 플레이어가 모두 행동을 마쳤을 때).
    ///
    /// TODO: 페이즈/스텝 자동 진행에 필요한 로직을 구현해야 합니다.
    /// 현재는 `todo!()` 매크로로 표시되어 있습니다.
    fn attempt_to_advance_phase_or_step(&mut self) {
        todo!()
    }

    /// 다음 턴으로 넘어갑니다.
    ///
    /// 현재 턴 플레이어를 변경하고, 턴 카운트를 증가시키고, 현재 페이즈를 드로우 페이즈로 변경하고,
    /// 우선권 소유자를 현재 턴 플레이어로 설정하고, 각 플레이어의 행동 상태를 초기화합니다.
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    /// use simulator_core::game::phase::{Phase, DrawPhaseStatus, MulliganStatus, PlayerActionStatus};
    ///
    /// let mut turn_state = TurnState::new(PlayerKind::Player1);
    /// turn_state.advance_turn();
    /// assert_eq!(turn_state.current_turn_plyaer, PlayerKind::Player2);
    /// assert_eq!(turn_state.turn_count, 2);
    /// assert_eq!(turn_state.current_phase, Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws));
    /// ```
    pub fn advance_turn(&mut self) {
        self.current_turn_plyaer = self.current_turn_plyaer.reverse();
        self.turn_count += 1;
        self.current_phase = Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws);
        self.priority_holder = self.current_turn_plyaer;
        self.turn_player_action_status = PlayerActionStatus::NotYetActed;
        self.non_turn_player_action_status = PlayerActionStatus::NotYetActed;
    }
}

impl TurnState {
    /// 현재 턴 카운트를 반환합니다.
    ///
    /// # Returns
    ///
    /// * `usize` - 현재 턴 카운트
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.get_turn_count(), 1);
    /// ```
    pub fn get_turn_count(&self) -> usize {
        self.turn_count
    }

    /// 턴 카운트를 1 증가시키고, 증가된 턴 카운트를 반환합니다.
    ///
    /// # Returns
    ///
    /// * `usize` - 증가된 턴 카운트
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let mut turn_state = TurnState::new(PlayerKind::Player1);
    /// let new_turn_count = turn_state.increase_turn_count();
    /// assert_eq!(new_turn_count, 2);
    /// assert_eq!(turn_state.get_turn_count(), 2);
    /// ```
    pub fn increase_turn_count(&mut self) -> usize {
        self.turn_count += 1;
        self.turn_count
    }

    /// 현재 턴 플레이어를 반환합니다.
    ///
    /// # Returns
    ///
    /// * `PlayerKind` - 현재 턴 플레이어 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.current_turn(), PlayerKind::Player1);
    /// ```
    pub fn current_turn(&self) -> PlayerKind {
        self.current_turn_plyaer
    }

    /// 현재 턴 플레이어를 변경하고, 변경된 턴 플레이어를 반환합니다.
    ///
    /// 현재 턴 플레이어가 `PlayerKind::Player1`이면 `PlayerKind::Player2`로,
    /// `PlayerKind::Player2`이면 `PlayerKind::Player1`로 변경합니다.
    ///
    /// # Returns
    ///
    /// * `PlayerKind` - 변경된 턴 플레이어 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let mut turn_state = TurnState::new(PlayerKind::Player1);
    /// let new_turn_player = turn_state.change_turn();
    /// assert_eq!(new_turn_player, PlayerKind::Player2);
    /// assert_eq!(turn_state.current_turn(), PlayerKind::Player2);
    /// ```
    pub fn change_turn(&mut self) -> PlayerKind {
        self.current_turn_plyaer = match self.current_turn_plyaer {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        };
        self.current_turn_plyaer
    }

    /// 현재 턴이 플레이어 1의 턴인지 확인합니다.
    ///
    /// # Returns
    ///
    /// * `bool` - 현재 턴이 플레이어 1의 턴이면 `true`, 그렇지 않으면 `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.is_player_turn(), true);
    /// ```
    pub fn is_player_turn(&self) -> bool {
        self.current_turn_plyaer == PlayerKind::Player1
    }

    /// 현재 턴이 상대방의 턴인지 확인합니다.
    ///
    /// # Returns
    ///
    /// * `bool` - 현재 턴이 상대방의 턴이면 `true`, 그렇지 않으면 `false`
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.is_opponent_turn(), false);
    /// ```
    pub fn is_opponent_turn(&self) -> bool {
        self.current_turn_plyaer == PlayerKind::Player2
    }

    /// 현재 턴 플레이어의 상대방을 반환합니다.
    ///
    /// # Returns
    ///
    /// * `PlayerKind` - 현재 턴 플레이어의 상대방 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let turn_state = TurnState::new(PlayerKind::Player1);
    /// assert_eq!(turn_state.get_opponent_turn(), PlayerKind::Player2);
    /// ```
    pub fn get_opponent_turn(&self) -> PlayerKind {
        match self.current_turn_plyaer {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        }
    }

    /// 특정 플레이어의 턴으로 강제 설정합니다.
    ///
    /// # Arguments
    ///
    /// * `player` - 설정할 턴 플레이어 (`PlayerKind::Player1` 또는 `PlayerKind::Player2`)
    ///
    /// # Examples
    ///
    /// ```
    /// use simulator_core::game::turn::TurnState;
    /// use simulator_core::card::types::PlayerKind;
    ///
    /// let mut turn_state = TurnState::new(PlayerKind::Player1);
    /// turn_state.set_turn(PlayerKind::Player2);
    /// assert_eq!(turn_state.current_turn(), PlayerKind::Player2);
    /// ```
    pub fn set_turn(&mut self, player: PlayerKind) {
        self.current_turn_plyaer = player;
    }
}