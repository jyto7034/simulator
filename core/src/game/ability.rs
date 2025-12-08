use serde::{Deserialize, Serialize};

use crate::game::stats::StatModifier;

/// 스킬/어빌리티 식별자
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AbilityId {
    /// Scorched Girl 전용 폭발 스킬
    ScorchedExplosion,
    /// Plague Doctor – 광역 힐
    PlagueMassHeal,
    /// Red Shoes – 광폭화 연속 공격
    RedShoesBerserk,
    /// Fragment of the Universe – 우주적 폭발
    FragmentOfUniverseNova,
    /// Spider Bud – 중첩형 출혈/독
    SpiderBudPoisonStack,
    /// Fairy Festival – 아군 전체 버프
    FairyFestivalBlessing,
    /// 랜덤 이벤트용 Unknown Distortion 스킬
    UnknownDistortionStrike,
}

/// 스킬 발동 트리거
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AbilityTrigger {
    /// 기본 공격 시
    OnAttack,
    /// 피격 시
    OnHit,
    /// 전투 시작 시 1회
    OnBattleStart,
    /// 주기적으로 발동 (쿨다운과 별개)
    Periodic { interval_ms: u64 },
}

/// 효과 대상 범위
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TargetScope {
    SelfUnit,
    AllySingle,
    AllyAll,
    EnemySingle,
    EnemyAll,
}

/// 스킬이 실제로 만들어내는 효과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityEffect {
    /// 직접 데미지를 가함 (StatModifier와 별개, 고정 값)
    DirectDamage { amount: i32, target: TargetScope },

    /// 힐 효과
    Heal { amount: i32, target: TargetScope },

    /// 추가 기본 공격 1회 (또는 n회)
    ExtraAttack { count: u8, target: TargetScope },

    /// 스탯 변경(버프/디버프) – StatModifier 재사용
    StatModifiers {
        modifiers: Vec<StatModifier>,
        target: TargetScope,
    },
}

/// 하나의 스킬 정의 (정적 데이터)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: AbilityId,
    pub trigger: AbilityTrigger,
    pub cooldown_ms: u64,
    pub effects: Vec<AbilityEffect>,
}
