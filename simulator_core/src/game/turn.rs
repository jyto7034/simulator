use crate::card::types::PlayerKind;

use super::phase::{DrawPhaseStatus, MulliganStatus, Phase, PlayerActionStatus};

pub struct TurnState {
    pub current_turn_plyaer: PlayerKind,
    pub priority_holder: PlayerKind,
    pub turn_count: usize,
    pub current_phase: Phase,
    pub turn_player_action_status: PlayerActionStatus, // 턴 플레이어의 행동 상태
    pub non_turn_player_action_status: PlayerActionStatus,
}

impl TurnState {
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

    // 우선권 이전 로직
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

    // 플레이어가 행동을 했을 때 호출 (예: 카드 발동)
    pub fn player_acted(&mut self, acting_player: PlayerKind) {
        todo!()
        // self.pass_priority();
    }

    // 페이즈/스텝 자동 진행 시도 (양쪽 플레이어가 모두 행동을 마쳤을 때)
    fn attempt_to_advance_phase_or_step(&mut self) {
        todo!()
    }

    // 다음 턴으로 넘어갈 때 호출
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
    pub fn get_turn_count(&self) -> usize {
        self.turn_count
    }

    pub fn increase_turn_count(&mut self) -> usize {
        self.turn_count += 1;
        self.turn_count
    }

    pub fn current_turn(&self) -> PlayerKind {
        self.current_turn_plyaer
    }

    pub fn change_turn(&mut self) -> PlayerKind {
        self.current_turn_plyaer = match self.current_turn_plyaer {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        };
        self.current_turn_plyaer
    }

    pub fn is_player_turn(&self) -> bool {
        self.current_turn_plyaer == PlayerKind::Player1
    }

    pub fn is_opponent_turn(&self) -> bool {
        self.current_turn_plyaer == PlayerKind::Player2
    }

    pub fn get_opponent_turn(&self) -> PlayerKind {
        match self.current_turn_plyaer {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        }
    }

    // 특정 플레이어의 턴으로 강제 설정
    pub fn set_turn(&mut self, player: PlayerKind) {
        self.current_turn_plyaer = player;
    }
}
