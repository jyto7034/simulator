use crate::game::{
    ability::{
        authoring::AuthoredSkillEffect, flow::*, resonance::ResonanceAdmission, DeliveryDef,
        FocusPermissions, SkillCastTargetingDef, SkillId, SkillKind, SkillTarget,
    },
    battle::damage::DamageType,
    value_expr::AmountExpr,
};
use serde::{Deserialize, Serialize};

/// A format-neutral meaning model. JSON is the initial headless input adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub version: u32,
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub predicates: Vec<super::predicates::PredicateDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<crate::game::resource::ResourceDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_attacks: Vec<EquipmentAttack>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_defenses: Vec<EquipmentDefense>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_supports: Vec<EquipmentSupport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub equipment_states: Vec<EquipmentState>,
    pub values: Vec<NamedValue>,
    pub definitions: Vec<Definition>,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentAttack {
    pub equipment: String,
    pub program: Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDefense {
    pub equipment: String,
    pub program: Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentSupport {
    pub equipment: String,
    pub program: Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentState {
    pub equipment: String,
    pub program: Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedValue {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Amount,
    Milliseconds,
    Count,
    Target,
    Delivery,
    TilePattern,
    Effects,
    AttackProgram,
    DefenseProgram,
    SupportProgram,
    StateProgram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Value {
    Amount(AmountExpr),
    Milliseconds(u64),
    Count(u32),
    Target(SkillTarget),
    Delivery(DeliveryDef),
    TilePattern(crate::game::battle::tile_range::TileRangePattern),
    Effects(Vec<AuthoredSkillEffect>),
    AttackProgram(crate::game::ability::equipment_attack::EquipmentAttackProgram),
    DefenseProgram(crate::game::ability::equipment_defense::EquipmentDefenseProgram),
    SupportProgram(crate::game::ability::equipment_support::EquipmentSupportProgram),
    StateProgram(crate::game::ability::state_program::StateProgram),
}
impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Amount(_) => ValueType::Amount,
            Self::Milliseconds(_) => ValueType::Milliseconds,
            Self::Count(_) => ValueType::Count,
            Self::Target(_) => ValueType::Target,
            Self::Delivery(_) => ValueType::Delivery,
            Self::TilePattern(_) => ValueType::TilePattern,
            Self::Effects(_) => ValueType::Effects,
            Self::AttackProgram(_) => ValueType::AttackProgram,
            Self::DefenseProgram(_) => ValueType::DefenseProgram,
            Self::SupportProgram(_) => ValueType::SupportProgram,
            Self::StateProgram(_) => ValueType::StateProgram,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Argument {
    Literal(Value),
    Shared(String),
    Parameter(String),
    /// Preserved by ordinary serialization; never replaced with a default.
    Unresolved(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamedArgument {
    pub name: String,
    pub value: Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Flow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub id: SkillId,
    pub name: String,
    pub kind: SkillKind,
    pub cast_targeting: SkillCastTargetingDef,
    pub prepare_ms: u32,
    pub focus_permissions: FocusPermissions,
    #[serde(deserialize_with = "crate::game::data::deserialize_required_option")]
    pub admission: Option<ResonanceAdmission>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<SkillModifier>,
    pub body: Flow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Effect {
    Damage {
        id: String,
        tags: Vec<String>,
        amount: Argument,
        damage_type: DamageType,
    },
    Heal {
        id: String,
        tags: Vec<String>,
        amount: Argument,
    },
    /// Independent recipient pools owned by the existing shield runtime.
    PersonalShieldGrant {
        id: String,
        tags: Vec<String>,
        amount: Argument,
        duration: Argument,
        priority: i32,
        recipients: crate::game::relic::structural::authoring::PersonalShieldRecipientsDef,
        support: Option<crate::game::relic::structural::support::SupportPolicyDef>,
        replace_only_recipients: bool,
        source_creation:
            Option<crate::game::relic::structural::authoring::ShieldCreationModifierDef>,
    },
    /// Explicit native primitive attachments; no skill-family dispatch.
    Append { effects: Argument },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Flow {
    Action {
        id: String,
        target: Argument,
        delivery: Argument,
        effects: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        modifiers: Vec<ActionModifier>,
        wait: FlowMilestone,
    },
    Wait {
        id: String,
        duration: Argument,
    },
    Sequence {
        id: String,
        children: Vec<Flow>,
    },
    Parallel {
        id: String,
        branches: Vec<Flow>,
    },
    Branch {
        id: String,
        condition: FlowCondition,
        then_node: Box<Flow>,
        else_node: Box<Flow>,
    },
    Repeat {
        id: String,
        count: Argument,
        interval: Argument,
        body: Box<Flow>,
    },
    Call {
        id: String,
        definition: String,
        arguments: Vec<NamedArgument>,
    },
    Unsupported {
        id: String,
        operation: String,
    },
}

/// These conditions change the whole skill, independent of local flow branches.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillModifier {
    pub id: String,
    pub relic_definition: String,
    pub variant: crate::game::relic::RelicVariant,
    pub change: SkillChange,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SkillChange {
    ProtectInterruption,
    /// One cast/target counter shared by all direct damage actions in this
    /// skill, including expanded calls and repeat iterations. Released child
    /// effects retain the existing runtime ownership; proc damage is excluded.
    Multihit {
        increment_percent: Argument,
        cap_percent: Argument,
    },
}

/// Conditional changes belong to this Action's effects, including appended
/// inputs. The compiler supplies the expanded step identity at each call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionModifier {
    pub id: String,
    pub relic_definition: String,
    pub variant: crate::game::relic::RelicVariant,
    pub change: ActionChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActionChange {
    Range {
        pattern: Argument,
    },
    AreaDuration {
        percent: Argument,
    },
    AreaRampRate {
        status: usize,
        reached_rules: Vec<usize>,
        numerator: Argument,
        denominator: Argument,
    },
    AreaControlVulnerability {
        status: usize,
        percent: Argument,
        duration: Argument,
    },
    ControlVulnerability {
        effect: String,
        percent: Argument,
        duration: Argument,
    },
    Duration {
        effect: String,
        percent: Argument,
    },
    SupportRadius {
        effect: String,
        percent: Argument,
    },
    TauntDamage {
        effect: String,
        taken_percent: Argument,
        basic_percent: Argument,
    },
}
impl ActionChange {
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Range { .. } => "Range",
            Self::AreaDuration { .. } => "AreaDuration",
            Self::AreaRampRate { .. } => "AreaRampRate",
            Self::AreaControlVulnerability { .. } => "AreaControlVulnerability",
            Self::ControlVulnerability { .. } => "ControlVulnerability",
            Self::Duration { .. } => "Duration",
            Self::SupportRadius { .. } => "SupportRadius",
            Self::TauntDamage { .. } => "TauntDamage",
        }
    }
}
impl Flow {
    pub fn id(&self) -> &str {
        match self {
            Self::Action { id, .. }
            | Self::Wait { id, .. }
            | Self::Sequence { id, .. }
            | Self::Parallel { id, .. }
            | Self::Branch { id, .. }
            | Self::Repeat { id, .. }
            | Self::Call { id, .. }
            | Self::Unsupported { id, .. } => id,
        }
    }
}
