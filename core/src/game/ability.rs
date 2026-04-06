use serde::{Deserialize, Serialize};

use crate::game::stats::TriggerType;

pub type SkillId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillKind {
    #[default]
    Targeted,
    Untargeted,
}

/// 집중(focus) 동안 허용되는 행동.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FocusPermissions {
    #[serde(default)]
    pub allows_move: bool,
    #[serde(default)]
    pub allows_basic_attack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnitTargetRule {
    #[default]
    Nearest,
    CurrentTarget,
    LowestHealthEnemy,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillArea {
    #[default]
    All,
    RadiusChebyshev {
        radius_tiles: u8,
    },
    Line {
        length_tiles: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DeliveryDef {
    #[default]
    Instant,
    Projectile {
        speed_units_per_ms: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StepTargetingMode {
    #[default]
    ReuseCastTarget,
    RetargetOnStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillUnitReference {
    SelfUnit,
    StepTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillStepCondition {
    #[default]
    Always,
    IfPreviousStepDealtDamage,
    IfCasterHasBuff {
        buff_id: String,
        #[serde(default = "default_min_stacks")]
        min_stacks: u8,
    },
}

fn default_min_stacks() -> u8 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillStepRepeat {
    #[default]
    Once,
    Times {
        count: u8,
    },
    ByBuffStacks {
        unit: SkillUnitReference,
        buff_id: String,
        #[serde(default)]
        max: Option<u8>,
    },
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

impl SkillPresentationDef {
    pub fn is_empty(&self) -> bool {
        self.cast_state
            .as_deref()
            .map_or(true, |value| value.trim().is_empty())
            && self
                .projectile_vfx_id
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
            && self
                .impact_vfx_id
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
            && self
                .target_anchor
                .as_deref()
                .map_or(true, |value| value.trim().is_empty())
    }
}

fn default_proc_chance_percent() -> u8 {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityActivationDef {
    TriggerProc {
        trigger: TriggerType,
        #[serde(default = "default_proc_chance_percent")]
        proc_chance_percent: u8,
        #[serde(default)]
        internal_cooldown_ms: u64,
        #[serde(default)]
        max_triggers_per_battle: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityActivationBinding {
    pub ability_id: SkillId,
    pub activation: AbilityActivationDef,
}

fn default_step_id() -> String {
    "step".to_string()
}

fn default_step_range_tiles() -> u8 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillCastTargetingDef {
    /// Backward-compatible default: infer the cast-level target from the first step.
    #[default]
    FirstStepTarget,
    /// Explicit cast-level targeting independent from individual step execution targets.
    Explicit {
        #[serde(default = "default_step_range_tiles")]
        range_tiles: u8,
        target: SkillTarget,
    },
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
    pub targeting: StepTargetingMode,
    #[serde(default)]
    pub when: SkillStepCondition,
    #[serde(default)]
    pub repeat: SkillStepRepeat,
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
    pub cast_targeting: SkillCastTargetingDef,
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

    pub fn cast_target_definition(&self) -> Option<(u8, &SkillTarget)> {
        match &self.cast_targeting {
            SkillCastTargetingDef::FirstStepTarget => self
                .first_step()
                .map(|step| (step.range_tiles, &step.target)),
            SkillCastTargetingDef::Explicit {
                range_tiles,
                target,
            } => Some((*range_tiles, target)),
        }
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
        assert_eq!(def.cast_targeting, SkillCastTargetingDef::FirstStepTarget);
        assert_eq!(def.focus_time_ms, 200);
        assert_eq!(def.steps.len(), 1);
        assert_eq!(def.steps[0].id, "hit");
        assert_eq!(def.steps[0].range_tiles, 3);
        assert_eq!(def.steps[0].targeting, StepTargetingMode::ReuseCastTarget);
        assert_eq!(def.steps[0].when, SkillStepCondition::Always);
        assert_eq!(def.steps[0].repeat, SkillStepRepeat::Once);
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

    #[test]
    fn skill_step_ron_deserialization_reads_explicit_targeting_mode() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s2",
                steps:[
                    (
                        id:"retarget",
                        range_tiles:2,
                        target:EnemySingle(rule:Nearest),
                        targeting:RetargetOnStep,
                        effects:[Damage(amount:10)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(def.steps[0].targeting, StepTargetingMode::RetargetOnStep);
    }

    #[test]
    fn skill_step_ron_deserialization_reads_condition_and_repeat() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s3",
                steps:[
                    (
                        id:"followup",
                        range_tiles:1,
                        target:SelfUnit,
                        when:IfPreviousStepDealtDamage,
                        repeat:ByBuffStacks(
                            unit:SelfUnit,
                            buff_id:"poison",
                            max:Some(3),
                        ),
                        effects:[Heal(amount:10)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            def.steps[0].when,
            SkillStepCondition::IfPreviousStepDealtDamage
        );
        assert_eq!(
            def.steps[0].repeat,
            SkillStepRepeat::ByBuffStacks {
                unit: SkillUnitReference::SelfUnit,
                buff_id: "poison".to_string(),
                max: Some(3),
            }
        );
    }

    #[test]
    fn skill_def_ron_deserialization_reads_explicit_cast_targeting() {
        let def: SkillDef = ron::de::from_str(
            r#"
            (
                id:"s4",
                cast_targeting:Explicit(
                    range_tiles:4,
                    target:EnemySingle(rule:LowestHealthEnemy),
                ),
                steps:[
                    (
                        id:"charge",
                        target:SelfUnit,
                        effects:[Heal(amount:10)],
                    ),
                ],
            )
            "#,
        )
        .unwrap();

        assert_eq!(
            def.cast_targeting,
            SkillCastTargetingDef::Explicit {
                range_tiles: 4,
                target: SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
            }
        );
        assert_eq!(
            def.cast_target_definition(),
            Some((
                4,
                &SkillTarget::EnemySingle {
                    rule: UnitTargetRule::LowestHealthEnemy,
                },
            ))
        );
    }
}
