use serde::{Deserialize, Serialize};

pub type SkillId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillKind {
    Targeted,
    Untargeted,
}

impl Default for SkillKind {
    fn default() -> Self {
        Self::Targeted
    }
}

/// 집중(focus) 동안 허용되는 행동.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FocusPermissions {
    #[serde(default)]
    pub allows_move: bool,
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
    CurrentTarget,
    LowestHealthEnemy,
}

impl Default for UnitTargetRule {
    fn default() -> Self {
        Self::Nearest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    SelfUnit,
    EnemySingle {
        #[serde(default)]
        rule: UnitTargetRule,
    },
    Allies {
        area: SkillArea,
    },
    Enemies {
        area: SkillArea,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillArea {
    All,
    RadiusChebyshev { radius_tiles: u8 },
    Line { length_tiles: u8 },
}

impl Default for SkillArea {
    fn default() -> Self {
        Self::All
    }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillPresentationDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cast_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projectile_vfx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impact_vfx_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_anchor: Option<String>,
}

fn default_step_id() -> String {
    "step".to_string()
}

fn default_step_range_tiles() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStepDef {
    #[serde(default = "default_step_id")]
    pub id: String,
    #[serde(default)]
    pub delay_ms: u32,
    #[serde(default = "default_step_range_tiles")]
    pub range_tiles: u8,
    pub target: SkillTarget,
    #[serde(default)]
    pub delivery: DeliveryDef,
    #[serde(default)]
    pub effects: Vec<SkillEffectDef>,
    #[serde(default)]
    pub presentation: SkillPresentationDef,
}

fn default_skill_name() -> String {
    "Unnamed Skill".to_string()
}

/// 데이터 기반 스킬 정의 (RON 로드 대상)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: SkillId,
    #[serde(default = "default_skill_name")]
    pub name: String,
    #[serde(default)]
    pub kind: SkillKind,
    #[serde(default)]
    pub focus_time_ms: u32,
    #[serde(default)]
    pub focus_permissions: FocusPermissions,
    #[serde(default)]
    pub steps: Vec<SkillStepDef>,
}

/// 스킬 효과(초안): 구현 단계에서 커맨드/시스템으로 매핑될 수 있는 데이터 표현
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffectDef {
    Damage {
        amount: i32,
    },
    Heal {
        amount: i32,
    },
    ModifyResonance {
        amount: i32,
    },
    ModifyStats {
        modifier: crate::game::stats::StatModifier,
    },
    ApplyBuff {
        buff_id: String,
        duration_ms: u32,
    },
    ExtraAttack {
        count: u8,
    },
}

impl SkillDef {
    pub fn first_step(&self) -> Option<&SkillStepDef> {
        self.steps.first()
    }
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
    fn skill_def_ron_deserialization_reads_step_based_schema() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s1",
                name:"Test Skill",
                focus_time_ms:200,
                steps:[
                    (
                        id:"hit",
                        range_tiles:3,
                        target:EnemySingle(rule:Nearest),
                        delivery:Instant,
                        effects:[Damage(amount:10)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(def.id, "s1");
        assert_eq!(def.name, "Test Skill");
        assert_eq!(def.kind, SkillKind::Targeted);
        assert_eq!(def.focus_time_ms, 200);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].id, "hit");
        assert_eq!(def.steps[0].range_tiles, 3);
        assert!(matches!(
            def.steps[0].target,
            SkillTarget::EnemySingle { .. }
        ));
        assert!(matches!(def.steps[0].delivery, DeliveryDef::Instant));
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
