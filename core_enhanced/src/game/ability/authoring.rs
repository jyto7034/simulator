//! Typed authoring addresses for the skill executor. Tags describe an effect;
//! they never enable an operation or select a definition by identity.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{DeliveryDef, SkillEffectDef, SkillId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSite {
    pub definition_id: SkillId,
    pub action_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectSite {
    pub action: ActionSite,
    pub effect_id: String,
}

crate::authoring::clone_shape::struct_shape!(ActionSite {
    definition_id,
    action_id
});
crate::authoring::clone_shape::struct_shape!(EffectSite { action, effect_id });

/// Authored attribution, not a runtime grant or a persistent relic instance.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EffectOrigin {
    #[default]
    Original,
    Added {
        source_definition_id: SkillId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSource {
    pub source_file: String,
    pub site: EffectSite,
    pub field_path: String,
    pub origin_definition_id: SkillId,
    pub origin: EffectOrigin,
}

crate::authoring::clone_shape::enum_shape!(EffectOrigin {
    units: [Original], tuples: [], records: [Added { source_definition_id }]
});
crate::authoring::clone_shape::struct_shape!(EffectSource {
    source_file, site, field_path, origin_definition_id, origin
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSkillEffect {
    pub id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub origin: EffectOrigin,
    /// Bind a hostile attachment to an earlier damage in this action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<String>,
    pub op: SkillEffectDef,
    /// Reconstructed for each compilation. Never an authored or saved index.
    #[serde(skip)]
    pub(crate) source: Option<Arc<EffectSource>>,
}

impl AuthoredSkillEffect {
    pub fn new(id: impl Into<String>, op: SkillEffectDef) -> Self {
        Self {
            id: id.into(),
            tags: Vec::new(),
            origin: EffectOrigin::Original,
            op,
            attached_to: None,
            source: None,
        }
    }

    pub fn source(&self) -> Option<&Arc<EffectSource>> {
        self.source.as_ref()
    }

    pub fn literal_amount(&self) -> Option<i32> {
        match self.op {
            SkillEffectDef::Damage { amount, .. } | SkillEffectDef::Heal { amount } => Some(amount),
            _ => None,
        }
    }
}

impl SkillEffectDef {
    /// Explicit construction helper for typed callers. RON still requires id.
    pub fn authored(self, id: impl Into<String>) -> AuthoredSkillEffect {
        AuthoredSkillEffect::new(id, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralEffectOp {
    Damage,
    Heal,
}

impl LiteralEffectOp {
    pub fn of(op: &SkillEffectDef) -> Option<Self> {
        match op {
            SkillEffectDef::Damage { .. } => Some(Self::Damage),
            SkillEffectDef::Heal { .. } => Some(Self::Heal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectProperty {
    Amount,
    DamageType,
    // These belong to the action and are intentionally rejected as effect edits.
    Target,
    Delivery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PropertyType {
    NonnegativeI32,
    SignedI32,
    DamageType,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PropertyDescriptor {
    pub property: EffectProperty,
    pub value_type: PropertyType,
    pub unit: &'static str,
    pub editable: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct EffectSupport {
    pub op: LiteralEffectOp,
    pub consumer: &'static str,
    pub delivery: &'static str,
    pub properties: &'static [PropertyDescriptor],
}

const DAMAGE_PROPERTIES: &[PropertyDescriptor] = &[
    PropertyDescriptor {
        property: EffectProperty::Amount,
        value_type: PropertyType::NonnegativeI32,
        unit: "base damage",
        editable: true,
    },
    PropertyDescriptor {
        property: EffectProperty::DamageType,
        value_type: PropertyType::DamageType,
        unit: "damage type",
        editable: false,
    },
];
const HEAL_PROPERTIES: &[PropertyDescriptor] = &[PropertyDescriptor {
    property: EffectProperty::Amount,
    value_type: PropertyType::SignedI32,
    unit: "signed HP delta",
    editable: true,
}];

pub const EFFECT_SUPPORT: &[EffectSupport] = &[
    EffectSupport {
        op: LiteralEffectOp::Damage,
        consumer: "skill",
        delivery: "Instant",
        properties: DAMAGE_PROPERTIES,
    },
    EffectSupport {
        op: LiteralEffectOp::Heal,
        consumer: "skill",
        delivery: "Instant",
        properties: HEAL_PROPERTIES,
    },
];

pub fn effect_support(
    op: &SkillEffectDef,
    delivery: &DeliveryDef,
) -> Option<&'static EffectSupport> {
    if !matches!(delivery, DeliveryDef::Instant) {
        return None;
    }
    let op = LiteralEffectOp::of(op)?;
    EFFECT_SUPPORT.iter().find(|support| support.op == op)
}

#[derive(Debug, Clone, Default)]
pub struct EffectPredicate {
    pub all_tags: Vec<String>,
    pub op: Option<LiteralEffectOp>,
    pub property: Option<EffectProperty>,
}

impl EffectPredicate {
    /// An empty predicate matches each existing effect, never an absent effect.
    pub fn matches(&self, effect: &AuthoredSkillEffect) -> bool {
        self.all_tags.iter().all(|tag| effect.tags.contains(tag))
            && self
                .op
                .is_none_or(|op| LiteralEffectOp::of(&effect.op) == Some(op))
            && self.property.is_none_or(|property| match property {
                EffectProperty::Amount => effect.literal_amount().is_some(),
                EffectProperty::DamageType => matches!(effect.op, SkillEffectDef::Damage { .. }),
                EffectProperty::Target | EffectProperty::Delivery => false,
            })
    }
}

#[derive(Debug, Clone)]
pub enum EffectSelection {
    SameEffect(EffectPredicate),
    /// Each predicate needs a match. A match for one cannot satisfy an AND on
    /// another effect, and no definition identity is inferred from tags.
    EachExists(Vec<EffectPredicate>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthoringDiagnostic {
    pub source: Option<Arc<EffectSource>>,
    pub site: EffectSite,
    pub property: EffectProperty,
    pub affected_effects: Vec<EffectSite>,
    pub cause: String,
}

#[cfg(test)]
mod tests;
