use std::collections::HashMap;

use crate::card::types::PlayerKind;

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
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Phase {
    Mulligan,

    // 가장 먼저 시작되는 드로우 페이즈 ( 기타 자원 등 증가함. )
    DrawPhase,

    // 메인 페이즈 진입 전 시작되는 페이즈
    StandbyPhase,

    // 메인 페이즈 개시시
    MainPhaseStart,
    // 메인 페이즈 개시중
    MainPhase1,

    // 배틀 페이즈 진입
    BattlePhaseStart,
    // 배틀 페이즈 중
    BattleStep,
    // 데미지 스텝 개시시
    BattleDamageStepStart,
    // 데미지 계산 전
    BattleDamageStepCalculationBefore,
    // 데미지 계산 중
    BattleDamageStepCalculationStart,
    // 데미지 계산 후
    BattleDamageStepCalculationEnd,
    // 데미지 스텝 종료시
    BattleDamageStepEnd,
    // 데미지 페이즈 종료
    BattlePhaseEnd,

    // 메인 페이즈2 시작
    MainPhase2,

    // 턴 종료
    EndPhase,
}

impl From<String> for Phase {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "mulligan" => Phase::Mulligan,
            "drawphase" => Phase::DrawPhase,
            "standbyphase" => Phase::StandbyPhase,
            "mainphasestart" => Phase::MainPhaseStart,
            "mainphase1" => Phase::MainPhase1,
            "battlephasestart" => Phase::BattlePhaseStart,
            "battlestep" => Phase::BattleStep,
            "battledamagestepstart" => Phase::BattleDamageStepStart,
            "battledamagestepcalculationbefore" => Phase::BattleDamageStepCalculationBefore,
            "battledamagestepcalculationstart" => Phase::BattleDamageStepCalculationStart,
            "battledamagestepcalculationend" => Phase::BattleDamageStepCalculationEnd,
            "battledamagestepend" => Phase::BattleDamageStepEnd,
            "battlephaseend" => Phase::BattlePhaseEnd,
            "mainphase2" => Phase::MainPhase2,
            "endphase" => Phase::EndPhase,
            _ => panic!("Invalid Phase string: {}", value),
        }
    }
}

impl PartialOrd for Phase {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Phase {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order().cmp(&other.order())
    }
}

impl Phase {
    fn order(&self) -> u8 {
        match self {
            Phase::Mulligan => 0,
            Phase::DrawPhase => 1,
            Phase::StandbyPhase => 2,
            Phase::MainPhaseStart => 3,
            Phase::MainPhase1 => 4,
            Phase::BattlePhaseStart => 5,
            Phase::BattleStep => 6,
            Phase::BattleDamageStepStart => 7,
            Phase::BattleDamageStepCalculationBefore => 8,
            Phase::BattleDamageStepCalculationStart => 9,
            Phase::BattleDamageStepCalculationEnd => 10,
            Phase::BattleDamageStepEnd => 11,
            Phase::BattlePhaseEnd => 12,
            Phase::MainPhase2 => 13,
            Phase::EndPhase => 14,
        }
    }

    /// 현재 페이즈가 드로우 페이즈인지 확인
    pub fn is_draw_phase(&self) -> bool {
        matches!(self, Phase::DrawPhase)
    }

    /// 현재 페이즈가 스탠바이 페이즈인지 확인
    pub fn is_standby_phase(&self) -> bool {
        matches!(self, Phase::StandbyPhase)
    }

    /// 메인 페이즈 1 관련 체크
    pub fn is_main_phase_1(&self) -> bool {
        matches!(self, Phase::MainPhase1)
    }

    pub fn is_main_phase_1_start(&self) -> bool {
        matches!(self, Phase::MainPhaseStart)
    }

    /// 배틀 페이즈 관련 체크
    pub fn is_battle_phase(&self) -> bool {
        matches!(
            self,
            Phase::BattlePhaseStart
                | Phase::BattleStep
                | Phase::BattleDamageStepStart
                | Phase::BattleDamageStepCalculationBefore
                | Phase::BattleDamageStepCalculationStart
                | Phase::BattleDamageStepCalculationEnd
                | Phase::BattleDamageStepEnd
                | Phase::BattlePhaseEnd
        )
    }

    pub fn is_battle_step(&self) -> bool {
        matches!(self, Phase::BattleStep)
    }

    pub fn is_damage_step(&self) -> bool {
        matches!(
            self,
            Phase::BattleDamageStepStart
                | Phase::BattleDamageStepCalculationBefore
                | Phase::BattleDamageStepCalculationStart
                | Phase::BattleDamageStepCalculationEnd
                | Phase::BattleDamageStepEnd
        )
    }

    pub fn is_damage_calculation(&self) -> bool {
        matches!(self, Phase::BattleDamageStepCalculationStart)
    }

    pub fn is_before_damage_calculation(&self) -> bool {
        matches!(self, Phase::BattleDamageStepCalculationBefore)
    }

    pub fn is_after_damage_calculation(&self) -> bool {
        matches!(self, Phase::BattleDamageStepCalculationEnd)
    }

    /// 메인 페이즈 2 체크
    pub fn is_main_phase_2(&self) -> bool {
        matches!(self, Phase::MainPhase2)
    }

    /// 엔드 페이즈 체크
    pub fn is_end_phase(&self) -> bool {
        matches!(self, Phase::EndPhase)
    }

    /// 메인 페이즈 체크 (1과 2 모두)
    pub fn is_main_phase(&self) -> bool {
        matches!(
            self,
            Phase::MainPhaseStart | Phase::MainPhase1 | Phase::MainPhase2
        )
    }

    /// 일반 소환이 가능한 페이즈인지 체크
    pub fn can_normal_summon(&self) -> bool {
        matches!(self, Phase::MainPhase1 | Phase::MainPhase2)
    }

    /// 공격이 가능한 페이즈인지 체크
    pub fn can_attack(&self) -> bool {
        matches!(self, Phase::BattleStep)
    }

    /// 현재 페이즈가 개시시인지 체크
    pub fn is_phase_start(&self) -> bool {
        matches!(
            self,
            Phase::MainPhaseStart | Phase::BattlePhaseStart | Phase::BattleDamageStepStart
        )
    }

    /// 다음 페이즈 반환
    pub fn next_phase(&self) -> Phase {
        match self {
            Phase::Mulligan => Phase::DrawPhase,
            Phase::DrawPhase => Phase::StandbyPhase,
            Phase::StandbyPhase => Phase::MainPhaseStart,
            Phase::MainPhaseStart => Phase::MainPhase1,
            Phase::MainPhase1 => Phase::BattlePhaseStart,
            Phase::BattlePhaseStart => Phase::BattleStep,
            Phase::BattleStep => Phase::BattleDamageStepStart,
            Phase::BattleDamageStepStart => Phase::BattleDamageStepCalculationBefore,
            Phase::BattleDamageStepCalculationBefore => Phase::BattleDamageStepCalculationStart,
            Phase::BattleDamageStepCalculationStart => Phase::BattleDamageStepCalculationEnd,
            Phase::BattleDamageStepCalculationEnd => Phase::BattleDamageStepEnd,
            Phase::BattleDamageStepEnd => Phase::BattlePhaseEnd,
            Phase::BattlePhaseEnd => Phase::MainPhase2,
            Phase::MainPhase2 => Phase::EndPhase,
            Phase::EndPhase => Phase::DrawPhase,
        }
    }

    pub fn move_to_next_phase(&mut self) {
        *self = self.next_phase();
    }

    pub fn set_phase(&mut self, phase: Phase) {
        *self = phase;
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Mulligan => "Mulligan",
            Phase::DrawPhase => "DrawPhase",
            Phase::StandbyPhase => "StandbyPhase",
            Phase::MainPhaseStart => "MainPhaseStart",
            Phase::MainPhase1 => "MainPhase1",
            Phase::BattlePhaseStart => "BattlePhaseStart",
            Phase::BattleStep => "BattleStep",
            Phase::BattleDamageStepStart => "BattleDamageStepStart",
            Phase::BattleDamageStepCalculationBefore => "BattleDamageStepCalculationBefore",
            Phase::BattleDamageStepCalculationStart => "BattleDamageStepCalculationStart",
            Phase::BattleDamageStepCalculationEnd => "BattleDamageStepCalculationEnd",
            Phase::BattleDamageStepEnd => "BattleDamageStepEnd",
            Phase::BattlePhaseEnd => "BattlePhaseEnd",
            Phase::MainPhase2 => "MainPhase2",
            Phase::EndPhase => "EndPhase",
        }
    }
}
