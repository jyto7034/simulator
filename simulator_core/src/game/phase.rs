//! phase.rs
//!
//! 게임 시뮬레이터의 핵심 모듈
//! 이 모듈은 game와 관련된 기능을 제공합니다.

use std::collections::HashMap;

use crate::{
    card::types::PlayerKind,
    exception::{GameError, StateError},
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PlayerPhaseProgress {
    NotStarted,         // 아직 해당 페이즈 시작 안 함
    Entered,            // 페이즈에 막 진입했거나 기본적인 작업 대기 중
    ActionTaken,        // 페이즈 내 주요 액션 수행 (예: 드로우 완료, 공격 선언 완료)
    WaitingForOpponent, // 자신의 액션은 끝났고 상대방 대기 중
    Completed,          // 해당 페이즈 완전히 종료 (양쪽 동의)
}

#[derive(Clone)]
pub struct PhaseState {
    current_phase: Phase,
    // 플레이어별 진행 상태 저장
    player_progress: HashMap<PlayerKind, PlayerPhaseProgress>,
}

impl PhaseState {
    pub fn new(phase: Phase) -> Self {
        let mut progress = HashMap::new();
        // 초기 상태는 NotStarted 또는 Entered
        progress.insert(PlayerKind::Player1, PlayerPhaseProgress::NotStarted);
        progress.insert(PlayerKind::Player2, PlayerPhaseProgress::NotStarted);
        Self {
            current_phase: phase,
            player_progress: progress,
        }
    }

    pub fn get_phase(&self) -> Phase {
        self.current_phase
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.current_phase = phase;
        // 페이즈 전환 시 플레이어 상태 초기화
        self.reset_progress();
    }

    // 플레이어 진행 상태 가져오기
    pub fn get_player_progress(&self, player: PlayerKind) -> PlayerPhaseProgress {
        self.player_progress
            .get(&player)
            .cloned()
            .unwrap_or(PlayerPhaseProgress::NotStarted)
    }

    // 플레이어 진행 상태 업데이트
    pub fn update_player_progress(&mut self, player: PlayerKind, progress: PlayerPhaseProgress) {
        println!("PhaseState Update: Player {:?} -> {:?}", player, progress); // 로그 추가
        self.player_progress.insert(player, progress);
    }

    // 페이즈 전환 시 상태 초기화
    pub fn reset_progress(&mut self) {
        for progress in self.player_progress.values_mut() {
            *progress = PlayerPhaseProgress::Entered; // 새 페이즈는 Entered 상태로 시작
        }
        println!(
            "PhaseState Reset: All players progress set to Entered for phase {:?}",
            self.current_phase
        );
    }

    // 특정 상태인 플레이어가 있는지 확인 (예: 둘 다 Completed 인지)
    pub fn both_players_in_progress(&self, progress: PlayerPhaseProgress) -> bool {
        self.player_progress.len() == 2 && self.player_progress.values().all(|p| *p == progress)
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum PlayerActionStatus {
    NotYetActed,   // 아직 행동하지 않음
    ActedOrPassed, // 행동을 했거나 우선권을 넘김
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum MulliganStatus {
    NotStarted,                  // 시작 전
    DealingInitialHands,         // 초기 패 분배 중
    Player1Deciding(PlayerKind), // P1 멀리건 결정 대기 (현재 턴 플레이어 명시)
    Player2Deciding(PlayerKind), // P2 멀리건 결정 대기 (현재 턴 플레이어 명시)
    ApplyingMulligans,           // 멀리건 적용 중 (카드 교체 등)
    Completed,                   // 멀리건 완료
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum DrawPhaseStatus {
    TurnPlayerDraws, // 턴 플레이어가 드로우
    EffectsTrigger,  // 드로우 시 발동하는 효과 처리
    Completed,
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum StandbyPhaseStatus {
    EffectsTrigger, // 스탠바이 페이즈 시 발동하는 효과 처리
    Completed,
}

// 우선권 구현해야함.
// 우선권이란, 해당 턴에 어떤 플레이어가 먼저 행동할 수 있는지 권리를 나타내는 것.
// 턴 플레이어가 우선권을 가지며, 체인 발생 시 ( 효과 발동 ) / 우선권 포기 시 우선권이 이동함.
#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum MainPhaseStatus {
    OpenState, // 턴 플레이어가 자유롭게 행동 가능 (몬스터 소환, 마법/함정 발동/세트 등)
               // 체인 발생 시, 또는 우선권 이동 시 세부 상태가 더 필요할 수 있음
               // 예: WaitingForChainResponse, ResolvingChain
               // 유희왕은 메인 페이즈 1과 2가 동일한 행동을 할 수 있으므로,
               // MainPhaseStatus를 공유하고, Phase enum에서 MainPhase1, MainPhase2로 구분합니다.
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum BattlePhaseStep {
    StartStep(PlayerActionStatus), // 배틀 페이즈 개시 단계 (턴 플레이어 우선권)
    BattleStep(PlayerActionStatus), // 배틀 스텝 (몬스터 공격 선언 또는 종료)
    DamageStep(DamageStepSubPhase), // 데미지 스텝 (세부 단계로 진입)
    EndStep(PlayerActionStatus),   // 배틀 페이즈 종료 단계
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum DamageStepSubPhase {
    StartOfDamageStep,        // 데미지 스텝 개시시 (공격 대상이 앞면 표시가 됨 등)
    BeforeDamageCalculation,  // 데미지 계산 전 (공/수 증감 효과 발동)
    PerformDamageCalculation, // 데미지 계산 실행
    AfterDamageCalculation,   // 데미지 계산 후 (전투로 파괴된 몬스터 묘지로, 리버스 효과 발동 등)
    EndOfDamageStep,          // 데미지 스텝 종료시 (파괴 확정된 몬스터 묘지로 등)
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum EndPhaseStatus {
    EffectsTrigger, // 엔드 페이즈 시 발동하는 효과 처리
    TurnEnd,        // 실제 턴 종료
}

#[derive(Clone, PartialEq, Eq, Copy, Debug, PartialOrd, Ord)]
pub enum Phase {
    Mulligan(MulliganStatus),

    DrawPhase(DrawPhaseStatus),       // DP: 턴 플레이어가 1장 드로우
    StandbyPhase(StandbyPhaseStatus), // SP: 특정 효과 발동
    MainPhase1(MainPhaseStatus), // MP1: 몬스터 소환/반전소환/특수소환, 마법/함정 발동/세트, 효과 발동, 표시 형식 변경

    BattlePhase(BattlePhaseStep), // BP: 전투 수행
    // BattlePhaseStep enum이 세부 스텝(Start, Battle, Damage, End)을 관리
    MainPhase2(MainPhaseStatus), // MP2: MP1과 동일한 행동 가능 (공격한 몬스터는 표시 형식 변경 불가 등 일부 제약)
    EndPhase(EndPhaseStatus),    // EP: 특정 효과 발동, 턴 종료
}

impl Phase {
    /// 다음 페이즈로 진행합니다.
    /// 현재 턴 플레이어 정보가 필요할 수 있습니다.
    pub fn next(self, _current_turn_player: PlayerKind) -> Self {
        match self {
            Phase::Mulligan(status) => match status {
                MulliganStatus::Completed => Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws),
                _ => self, // 멀리건 내부 상태 진행은 별도 로직으로 처리
            },
            Phase::DrawPhase(status) => match status {
                DrawPhaseStatus::Completed => {
                    Phase::StandbyPhase(StandbyPhaseStatus::EffectsTrigger)
                }
                _ => self,
            },
            Phase::StandbyPhase(status) => match status {
                StandbyPhaseStatus::Completed => Phase::MainPhase1(MainPhaseStatus::OpenState),
                _ => self,
            },
            Phase::MainPhase1(_) => {
                Phase::BattlePhase(BattlePhaseStep::StartStep(PlayerActionStatus::NotYetActed))
            } // MP1에서 BP로 가는 것은 플레이어 선택
            Phase::BattlePhase(step) => match step {
                BattlePhaseStep::EndStep(PlayerActionStatus::ActedOrPassed) => {
                    Phase::MainPhase2(MainPhaseStatus::OpenState)
                }
                // 데미지 스텝의 각 하위 단계 진행은 BattlePhaseStep 내부 로직으로 처리
                _ => self,
            },
            Phase::MainPhase2(_) => Phase::EndPhase(EndPhaseStatus::EffectsTrigger), // MP2에서는 EP로 강제 진행
            Phase::EndPhase(status) => match status {
                EndPhaseStatus::TurnEnd => {
                    // 턴이 실제로 종료되면 다음 플레이어의 드로우 페이즈로 넘어감
                    // 이 때 current_turn_player가 변경되어야 함 (이 함수 외부에서 처리)
                    Phase::DrawPhase(DrawPhaseStatus::TurnPlayerDraws)
                }
                _ => self,
            },
        }
    }

    // 메인 페이즈에서 엔드 페이즈로 바로 넘어갈 수 있는 경우 (플레이어가 배틀 페이즈 스킵 선택)
    pub fn skip_to_end_phase(self) -> Result<Self, GameError> {
        match self {
            Phase::MainPhase1(_) | Phase::MainPhase2(_) => {
                Ok(Phase::EndPhase(EndPhaseStatus::EffectsTrigger))
            }
            _ => Err(GameError::State(StateError::InvalidPhaseTransition)), // 다른 페이즈에서는 스킵 불가
        }
    }
}

// Display implementations for all Status enums
impl std::fmt::Display for PlayerActionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerActionStatus::NotYetActed => write!(f, "NotYetActed"),
            PlayerActionStatus::ActedOrPassed => write!(f, "ActedOrPassed"),
        }
    }
}

impl std::fmt::Display for MulliganStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MulliganStatus::NotStarted => write!(f, "NotStarted"),
            MulliganStatus::DealingInitialHands => write!(f, "DealingInitialHands"),
            MulliganStatus::Player1Deciding(player) => write!(f, "Player1Deciding_{:?}", player),
            MulliganStatus::Player2Deciding(player) => write!(f, "Player2Deciding_{:?}", player),
            MulliganStatus::ApplyingMulligans => write!(f, "ApplyingMulligans"),
            MulliganStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl std::fmt::Display for DrawPhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawPhaseStatus::TurnPlayerDraws => write!(f, "TurnPlayerDraws"),
            DrawPhaseStatus::EffectsTrigger => write!(f, "EffectsTrigger"),
            DrawPhaseStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl std::fmt::Display for StandbyPhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandbyPhaseStatus::EffectsTrigger => write!(f, "EffectsTrigger"),
            StandbyPhaseStatus::Completed => write!(f, "Completed"),
        }
    }
}

impl std::fmt::Display for MainPhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainPhaseStatus::OpenState => write!(f, "OpenState"),
        }
    }
}

impl std::fmt::Display for DamageStepSubPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DamageStepSubPhase::StartOfDamageStep => write!(f, "StartOfDamageStep"),
            DamageStepSubPhase::BeforeDamageCalculation => write!(f, "BeforeDamageCalculation"),
            DamageStepSubPhase::PerformDamageCalculation => write!(f, "PerformDamageCalculation"),
            DamageStepSubPhase::AfterDamageCalculation => write!(f, "AfterDamageCalculation"),
            DamageStepSubPhase::EndOfDamageStep => write!(f, "EndOfDamageStep"),
        }
    }
}

impl std::fmt::Display for BattlePhaseStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattlePhaseStep::StartStep(status) => write!(f, "StartStep_{}", status),
            BattlePhaseStep::BattleStep(status) => write!(f, "BattleStep_{}", status),
            BattlePhaseStep::DamageStep(substep) => write!(f, "DamageStep_{}", substep),
            BattlePhaseStep::EndStep(status) => write!(f, "EndStep_{}", status),
        }
    }
}

impl std::fmt::Display for EndPhaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EndPhaseStatus::EffectsTrigger => write!(f, "EffectsTrigger"),
            EndPhaseStatus::TurnEnd => write!(f, "TurnEnd"),
        }
    }
}

// Main Phase Display implementation
impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::Mulligan(status) => write!(f, "Mulligan_{}", status),
            Phase::DrawPhase(status) => write!(f, "DrawPhase_{}", status),
            Phase::StandbyPhase(status) => write!(f, "StandbyPhase_{}", status),
            Phase::MainPhase1(status) => write!(f, "MainPhase1_{}", status),
            Phase::BattlePhase(step) => write!(f, "BattlePhase_{}", step),
            Phase::MainPhase2(status) => write!(f, "MainPhase2_{}", status),
            Phase::EndPhase(status) => write!(f, "EndPhase_{}", status),
        }
    }
}
