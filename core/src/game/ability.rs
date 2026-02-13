use serde::{Deserialize, Serialize};

pub type SkillId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillKind {
    Targeted,
    Untargeted,
}

/// 집중(focus) 동안 허용되는 행동.
///
/// - 기본값은 "아무것도 허용하지 않음"(= 하드 락)으로 둔다.
/// - 런타임에서 실제로 "언제까지 락인지"는 `ActionLocks`가 관리한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusPermissions {
    /// 집중 동안 이동 허용
    #[serde(default)]
    pub allows_move: bool,
    /// 집중 동안 기본 공격 "시작" 허용
    #[serde(default)]
    pub allows_basic_attack: bool,
}

impl Default for FocusPermissions {
    fn default() -> Self {
        Self {
            allows_move: false,
            allows_basic_attack: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitTargetRule {
    Nearest,
}

impl Default for UnitTargetRule {
    fn default() -> Self {
        Self::Nearest
    }
}

/// - 이 값은 "이번 캐스트에서 선택된 타겟"이 아니라, 스킬의 타겟팅/적용 규칙(메타데이터)이다.
/// - 실제 선택 결과(예: 특정 유닛/타일)는 타임라인/캐스트 컨텍스트가 가진다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    /// 자신에게만 적용
    SelfUnit,
    /// 단일 적에게 적용 (타겟 선택 규칙 포함)
    EnemySingle {
        #[serde(default)]
        rule: UnitTargetRule,
    },
    /// 아군 전체에게 적용(범위는 스킬 정의가 결정)
    Allies { area: SkillArea },
    /// 적군 전체에게 적용(범위는 스킬 정의가 결정)
    Enemies { area: SkillArea },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillArea {
    /// 전장 전체
    All,
    /// caster 타일 기준 chebyshev 반경
    RadiusChebyshev { radius_tiles: u8 },
}

impl Default for SkillArea {
    fn default() -> Self {
        Self::All
    }
}

fn default_skill_target() -> SkillTarget {
    SkillTarget::SelfUnit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryDef {
    Instant,
    Projectile { speed_units_per_ms: u32 },
}

impl Default for DeliveryDef {
    fn default() -> Self {
        Self::Instant
    }
}

fn default_cast_delay_ms() -> u32 {
    10
}

/// 데이터 기반 스킬 정의 (RON 로드 대상)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: SkillId,
    pub kind: SkillKind,
    #[serde(default = "default_skill_target")]
    pub target: SkillTarget,
    pub range_tiles: u8,
    #[serde(default = "default_cast_delay_ms")]
    pub cast_delay_ms: u32,
    #[serde(default)]
    pub focus_time_ms: u32,
    #[serde(default)]
    pub focus_permissions: FocusPermissions,
    pub delivery: DeliveryDef,
    #[serde(default)]
    pub effects: Vec<SkillEffectDef>,
}

/// 스킬 효과(초안): 구현 단계에서 커맨드/시스템으로 매핑될 수 있는 데이터 표현
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffectDef {
    Damage { amount: i32 },
    Heal { amount: i32 },
    ApplyBuff { buff_id: String, duration_ms: u32 },
    ExtraAttack { count: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_permissions_default_is_hard_lock() {
        let perms = FocusPermissions::default();
        assert!(!perms.allows_move);
        assert!(!perms.allows_basic_attack);
    }

    #[test]
    fn skill_def_ron_deserialization_applies_defaults() {
        let def: SkillDef =
            ron::de::from_str(r#"(id:"s1", kind:Targeted, range_tiles:1, delivery:Instant)"#)
                .unwrap();

        assert_eq!(def.id, "s1");
        assert_eq!(def.kind, SkillKind::Targeted);
        assert_eq!(def.target, SkillTarget::SelfUnit);
        assert_eq!(def.range_tiles, 1);
        assert_eq!(def.cast_delay_ms, 10);
        assert_eq!(def.focus_time_ms, 0);
        assert_eq!(def.focus_permissions, FocusPermissions::default());
        assert!(matches!(def.delivery, DeliveryDef::Instant));
        assert!(def.effects.is_empty());
    }

    #[test]
    fn enemy_single_target_defaults_to_nearest_rule_when_field_missing() {
        let target: SkillTarget = serde_json::from_str(r#"{"EnemySingle":{}}"#).unwrap();
        assert_eq!(
            target,
            SkillTarget::EnemySingle {
                rule: UnitTargetRule::Nearest
            }
        );
    }
}
