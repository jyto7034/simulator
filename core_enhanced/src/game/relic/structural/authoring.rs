//! Strict, finite authoring. These definitions do not confer execution authority.
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::game::{
    ability::{flow::SkillFlowDef, DeliveryDef, SkillTarget},
    battle::{damage::DamageType, tile_range::TileRangePattern},
    value_expr::AmountExpr,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicRuleDef {
    pub id: String,
    #[serde(default)]
    pub selector: StructuralSelectorDef,
    #[serde(default)]
    pub when: RelicConditionDef,
    pub apply: RelicRuleActionDef,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralSelectorDef {
    #[serde(default)]
    pub provider: ProviderPredicateDef,
    #[serde(default)]
    pub effect: EffectPredicateDef,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderPredicateDef {
    #[serde(default)]
    pub equipment_id: Option<String>,
    #[serde(default)]
    pub fragment_id: Option<String>,
    #[serde(default)]
    pub form_id: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectPredicateDef {
    #[serde(default)]
    pub all_tags: Vec<String>,
    #[serde(default)]
    pub op: Option<EffectOperation>,
    #[serde(default)]
    pub has_property: Option<RelicProperty>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub enum RelicConditionDef {
    #[default]
    Always,
    SameEffect(EffectPredicateDef),
    Exists(StructuralSelectorDef),
    All(Vec<RelicConditionDef>),
    Any(Vec<RelicConditionDef>),
    Not(Box<RelicConditionDef>),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RelicRuleActionDef {
    Grant(GrantBodyDef),
    Edit {
        property: RelicProperty,
        operation: NumericEditDef,
    },
    Recipe(StructuralRecipeDef),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrantBodyDef {
    InlineEffects {
        action_id: String,
        effects: Vec<GrantedEffectDef>,
    },
    FiniteFlow {
        anchor_action_id: String,
        steps: Vec<GrantedFlowStepDef>,
        flow: Arc<SkillFlowDef>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantedEffectDef {
    pub id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub op: GrantPrimitiveDef,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantedFlowStepDef {
    pub id: String,
    #[serde(default)]
    pub delay_ms: u32,
    pub target: SkillTarget,
    #[serde(default)]
    pub delivery: Arc<DeliveryDef>,
    pub effects: Vec<GrantedEffectDef>,
}
/// The closed primitive union deliberately contains no rule/subscriber/template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GrantPrimitiveDef {
    Damage {
        amount: i32,
        damage_type: DamageType,
    },
    DamageExpr {
        amount: Arc<AmountExpr>,
        damage_type: DamageType,
    },
    Heal {
        amount: i32,
    },
    HealExpr {
        amount: Arc<AmountExpr>,
    },
    HealthCost {
        amount: u32,
        allow_lethal: bool,
    },
    PersonalShieldGrant {
        definition: PersonalShieldGrantDef,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EffectOperation {
    Damage,
    Heal,
    HealthCost,
    PersonalShieldGrant,
    ChannelAura,
    Other,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RelicProperty {
    DamageAmount,
    DamageFootprint,
    SupportAreaRadius,
    PersonalShieldInitialAmount,
    RegularBasicDamagePercent,
    RegularBasicSpeedPoints,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TypedRelicValue {
    Hp(i64),
    PercentPoints(i64),
    SpeedPoints(i64),
    DataUnits(i64),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NumericEditDef {
    Add(TypedRelicValue),
    Scale { increment_percent: i64 },
    Replace(TypedRelicValue),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum StructuralRecipeDef {
    DamageFootprint {
        footprint: Arc<TileRangePattern>,
    },
    FixedRecipientSupport,
    RegularBasicSupport {
        damage_percent: u32,
        speed_points: u32,
    },
    PersonalShieldSingle {
        initial_amount: Arc<AmountExpr>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonalShieldGrantDef {
    pub amount: Arc<AmountExpr>,
    pub duration_ms: u32,
    pub priority: i32,
    pub recipients: PersonalShieldRecipientsDef,
    /// New authored typed program, not an existing hidden equipment bonus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_creation: Option<ShieldCreationModifierDef>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PersonalShieldRecipientsDef {
    PlayerEmployeesAtRelease { radius_units: u32 },
    SelectedPlayerEmployeeAtRelease { authored_reach_units: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShieldCreationModifierDef {
    pub operations: Vec<NumericEditDef>,
}

crate::authoring::clone_shape::copy_shape!(
    EffectOperation,
    RelicProperty,
    TypedRelicValue,
    NumericEditDef,
    PersonalShieldRecipientsDef
);
crate::authoring::clone_shape::struct_shape!(RelicRuleDef {
    id,
    selector,
    when,
    apply
});
crate::authoring::clone_shape::struct_shape!(StructuralSelectorDef { provider, effect });
crate::authoring::clone_shape::struct_shape!(ProviderPredicateDef {
    equipment_id,
    fragment_id,
    form_id
});
crate::authoring::clone_shape::struct_shape!(EffectPredicateDef {
    all_tags,
    op,
    has_property
});
crate::authoring::clone_shape::enum_shape!(RelicConditionDef {
    units: [Always],
    tuples: [SameEffect(p), Exists(s), All(v), Any(v), Not(v)],
    records: []
});
crate::authoring::clone_shape::enum_shape!(RelicRuleActionDef {
    units: [],
    tuples: [Grant(g), Recipe(r)],
    records: [Edit {
        property,
        operation
    }]
});
crate::authoring::clone_shape::enum_shape!(GrantBodyDef {
    units: [],
    tuples: [],
    records: [
        InlineEffects { action_id, effects },
        FiniteFlow {
            anchor_action_id,
            steps,
            flow
        }
    ]
});
crate::authoring::clone_shape::struct_shape!(GrantedEffectDef { id, tags, op });
crate::authoring::clone_shape::struct_shape!(GrantedFlowStepDef {
    id,
    delay_ms,
    target,
    delivery,
    effects
});
crate::authoring::clone_shape::enum_shape!(GrantPrimitiveDef {
    units: [],
    tuples: [],
    records: [
        Damage {
            amount,
            damage_type
        },
        DamageExpr {
            amount,
            damage_type
        },
        Heal { amount },
        HealExpr { amount },
        HealthCost {
            amount,
            allow_lethal
        },
        PersonalShieldGrant { definition }
    ]
});
crate::authoring::clone_shape::enum_shape!(StructuralRecipeDef {
    units: [FixedRecipientSupport],
    tuples: [],
    records: [
        DamageFootprint { footprint },
        RegularBasicSupport {
            damage_percent,
            speed_points
        },
        PersonalShieldSingle { initial_amount }
    ]
});
crate::authoring::clone_shape::struct_shape!(PersonalShieldGrantDef {
    amount,
    duration_ms,
    priority,
    recipients,
    source_creation
});
crate::authoring::clone_shape::struct_shape!(ShieldCreationModifierDef { operations });
