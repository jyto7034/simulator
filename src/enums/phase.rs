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
