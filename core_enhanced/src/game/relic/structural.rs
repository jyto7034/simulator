//! Personal immutable structural plans. Compilation/explanation is read-only;
//! these crate-private owners cannot be deserialized into execution authority.
pub mod authoring;
pub mod capabilities;
mod closure;
mod compiler;
mod execution;
mod publication;
pub(crate) use execution::EffectiveProviderPlan;
pub mod compose;
pub mod dto;
mod parse;
mod validation;

#[cfg(test)]
mod tests;

use super::{RelicInventory, RelicVariant};
use crate::game::{
    ability::{
        authoring::{ActionSite, EffectSite},
        DeliveryDef, SkillId,
    },
    behavior::GameError,
    data::ValidatedGameData,
    skill_fragment::FragmentSourceSnapshot,
    value_expr::AmountExpr,
};
use authoring::{EffectOperation, RelicProperty, StructuralRecipeDef, TypedRelicValue};
use compose::NumericProgram;
use serde::Serialize;
use std::{fmt, sync::Arc};
use uuid::Uuid;

pub(crate) use compiler::{
    compile_effective_plan, validate_effective_configuration, validate_publication,
};
pub(crate) use validation::shield as validate_personal_shield_definition;
pub(crate) use validation::validate_rules;

pub(crate) fn employee_has_structural_rules(
    inventory: &RelicInventory,
    employee: Uuid,
    data: &crate::game::data::GameDataBase,
) -> Result<bool, RelicPlanError> {
    crate::authoring::control::charge(
        (data.relic_data.relics.len() as u64)
            .checked_mul(3)
            .ok_or_else(|| {
                RelicPlanError::new(FaultCode::ArithmeticOverflow, "structural loadout scan")
            })?,
        "structural_loadout_presence",
    )
    .map_err(RelicPlanError::control)?;
    for id in inventory.employee_slots(employee).into_iter().flatten() {
        let copy = inventory.get(id).ok_or_else(|| {
            RelicPlanError::new(FaultCode::InvalidReference, "missing relic copy")
        })?;
        let definition = data
            .relic_data
            .get_by_id(&copy.definition_id)
            .ok_or_else(|| {
                RelicPlanError::new(FaultCode::InvalidReference, "missing relic definition")
            })?;
        if !validation::rules_for(definition, copy.variant).is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Read-only explanation of a prospective loadout. This is not publication,
/// an active World lookup, or a battle execution/admission capability.
/// No overlay is returned for a legacy-only loadout.
pub fn explain_effective_plan(
    generation: &ValidatedGameData,
    employee_uuid: Uuid,
    inventory: &RelicInventory,
    providers: &[ProspectiveProvider],
) -> Result<Option<dto::RelicEffectivePlanDto>, RelicPlanError> {
    compile_effective_plan(EffectivePlanInput {
        data: generation,
        employee_uuid,
        inventory,
        providers,
    })?
    .map(|plan| plan.explain())
    .transpose()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultCode {
    InvalidReference,
    UnsupportedOperation,
    UnitMismatch,
    MissingProperty,
    UnboundedGrantIdentity,
    NegativeDependencyCycle,
    UnprovenStructuralDependency,
    ReplaceConflict,
    StructuralWriteConflict,
    InvalidPreserveContract,
    ArithmeticOverflow,
    OutOfRange,
    ProviderUnavailable,
    StalePlanBinding,
    BudgetExceeded,
    Cancelled,
}
impl FaultCode {
    pub fn producer_code(self) -> &'static str {
        match self {
            Self::InvalidReference => "relic_structural_invalid_reference",
            Self::UnsupportedOperation => "relic_structural_unsupported_operation",
            Self::UnitMismatch => "relic_structural_unit_mismatch",
            Self::MissingProperty => "relic_structural_missing_property",
            Self::UnboundedGrantIdentity => "relic_structural_unbounded_identity",
            Self::NegativeDependencyCycle => "relic_structural_negative_cycle",
            Self::UnprovenStructuralDependency => "relic_structural_unproven_dependency",
            Self::ReplaceConflict => "relic_structural_replace_conflict",
            Self::StructuralWriteConflict => "relic_structural_write_conflict",
            Self::InvalidPreserveContract => "relic_structural_invalid_preserve_contract",
            Self::ArithmeticOverflow => "relic_structural_overflow",
            Self::OutOfRange => "relic_structural_out_of_range",
            Self::ProviderUnavailable => "relic_structural_provider_unavailable",
            Self::StalePlanBinding => "relic_structural_stale_binding",
            Self::BudgetExceeded => "relic_structural_budget_exceeded",
            Self::Cancelled => "relic_structural_cancelled",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct RelicPlanError {
    pub code: FaultCode,
    pub definition_id: Option<String>,
    pub rule_id: Option<String>,
    pub field_path: Option<String>,
    pub effect: Option<EffectiveEffectKey>,
    pub cause: String,
}
impl RelicPlanError {
    pub(crate) fn new(code: FaultCode, cause: impl Into<String>) -> Self {
        Self {
            code,
            definition_id: None,
            rule_id: None,
            field_path: None,
            effect: None,
            cause: cause.into(),
        }
    }
    pub(crate) fn at(mut self, definition: &str, rule: &str, field: impl Into<String>) -> Self {
        self.definition_id = Some(definition.to_owned());
        self.rule_id = Some(rule.to_owned());
        self.field_path = Some(field.into());
        self
    }
    pub(crate) fn control(error: GameError) -> Self {
        let cause = format!("{error:?}");
        Self::new(
            if cause.contains("cancelled") {
                FaultCode::Cancelled
            } else {
                FaultCode::BudgetExceeded
            },
            cause,
        )
    }
}
impl fmt::Display for RelicPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {:?}/{:?}/{:?}: {}",
            self.code.producer_code(),
            self.definition_id,
            self.rule_id,
            self.field_path,
            self.cause
        )
    }
}
impl std::error::Error for RelicPlanError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectiveEffectKey {
    Original {
        site: EffectSite,
    },
    Generated {
        relic_copy_uuid: Uuid,
        grant_rule_id: String,
        anchor: ActionSite,
        local_action_id: String,
        local_effect_id: String,
    },
}
/// Prospective copy identity; no runtime item, actor or action capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProspectiveProvider {
    Fragment(FragmentSourceSnapshot),
    Equipment {
        definition_id: String,
        copy_uuid: Uuid,
        skill_id: SkillId,
    },
}
impl ProspectiveProvider {
    pub(crate) fn skill_id(&self) -> &SkillId {
        match self {
            Self::Fragment(f) => &f.skill_id,
            Self::Equipment { skill_id, .. } => skill_id,
        }
    }
}
pub(crate) struct EffectivePlanInput<'a> {
    pub data: &'a ValidatedGameData,
    pub employee_uuid: Uuid,
    pub inventory: &'a RelicInventory,
    pub providers: &'a [ProspectiveProvider],
}
#[derive(Debug, Clone, Serialize)]
pub struct PlanContribution {
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub variant: RelicVariant,
    pub rule_id: String,
}
#[derive(Debug, Clone, Serialize)]
pub enum PropertyBase {
    Literal(TypedRelicValue),
    RuntimeExpression,
    Structural,
}
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveProperty {
    pub property: RelicProperty,
    pub base: PropertyBase,
    pub program: NumericProgram,
    pub final_value: Option<TypedRelicValue>,
    pub contributions: Vec<PlanContribution>,
}
#[derive(Debug, Clone)]
pub(crate) enum EffectBody {
    Original {
        skill_id: SkillId,
        step: usize,
        effect: usize,
    },
    Granted(Arc<authoring::GrantedEffectDef>),
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EffectiveEffect {
    pub key: EffectiveEffectKey,
    pub provider: usize,
    pub anchor: ActionSite,
    pub tags: Vec<String>,
    pub operation: EffectOperation,
    pub delivery: Arc<DeliveryDef>,
    pub properties: Vec<EffectiveProperty>,
    pub recipes: Vec<StructuralRecipeDef>,
    pub recipe_contributions: Vec<PlanContribution>,
    #[serde(skip)]
    pub body: EffectBody,
    #[serde(skip)]
    pub amount_capabilities: crate::game::value_expr::DeliveryCapabilities,
    pub amount: Option<Arc<AmountExpr>>,
    pub original: bool,
    pub present: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct CompileUsage {
    pub work: u64,
    pub reserved_bytes: usize,
    pub candidate_count: usize,
    pub binding_count: usize,
}
#[derive(Clone)]
pub(crate) struct EmployeeEffectivePlan {
    pub generation: ValidatedGameData,
    pub components: EffectivePlanComponents,
}
impl std::ops::Deref for EmployeeEffectivePlan {
    type Target = EffectivePlanComponents;
    fn deref(&self) -> &Self::Target {
        &self.components
    }
}
/// Pure producer output. It cannot construct a published generation or an
/// executable runtime binding; publication can inspect this before minting its
/// validated owner, and the ordinary compiler pins it only after validation.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EffectiveGrant {
    pub provider: usize,
    pub contribution: PlanContribution,
    pub anchor: ActionSite,
    pub body: Arc<authoring::GrantBodyDef>,
}
#[derive(Clone)]
pub(crate) struct EffectivePlanComponents {
    pub employee_uuid: Uuid,
    pub providers: Vec<ProspectiveProvider>,
    pub copies: Vec<super::OwnedRelic>,
    pub effects: Vec<EffectiveEffect>,
    pub grants: Vec<EffectiveGrant>,
    pub execution: Vec<execution::EffectiveProviderPlan>,
    pub rules: Vec<dto::RelicRuleStateDto>,
    pub fingerprint: String,
    pub usage: CompileUsage,
}
pub(crate) struct UnpublishedPlanInput<'a> {
    pub data: &'a crate::game::data::GameDataBase,
    pub generation: &'a crate::game::data::content_pack::ContentIdentity,
    pub employee_uuid: Uuid,
    pub inventory: &'a RelicInventory,
    pub providers: &'a [ProspectiveProvider],
}
impl EmployeeEffectivePlan {
    pub(crate) fn explain(&self) -> Result<dto::RelicEffectivePlanDto, RelicPlanError> {
        dto::explain(self)
    }
}
pub(crate) struct Work {
    pub work: u64,
    pub reserved: usize,
    limit: u64,
}
impl Work {
    pub(crate) fn new(limit: u64) -> Self {
        Self {
            work: 0,
            reserved: 0,
            limit,
        }
    }
    pub(crate) fn charge(&mut self, n: usize, site: &'static str) -> Result<(), RelicPlanError> {
        let n = u64::try_from(n)
            .map_err(|_| RelicPlanError::new(FaultCode::ArithmeticOverflow, "work conversion"))?;
        let next = self
            .work
            .checked_add(n)
            .ok_or_else(|| RelicPlanError::new(FaultCode::ArithmeticOverflow, "work sum"))?;
        if next > self.limit {
            return Err(RelicPlanError::new(
                FaultCode::BudgetExceeded,
                "structural compile work limit",
            ));
        }
        crate::authoring::control::charge(n, site).map_err(RelicPlanError::control)?;
        self.work = next;
        Ok(())
    }
    pub(crate) fn remaining(&self) -> u64 {
        self.limit - self.work
    }
    pub(crate) fn reserve(
        &mut self,
        bytes: Option<usize>,
        site: &'static str,
    ) -> Result<(), RelicPlanError> {
        let bytes = bytes
            .ok_or_else(|| RelicPlanError::new(FaultCode::ArithmeticOverflow, "storage bound"))?;
        let next = self
            .reserved
            .checked_add(bytes)
            .ok_or_else(|| RelicPlanError::new(FaultCode::ArithmeticOverflow, "storage sum"))?;
        crate::game::battle::runtime_control::admit_retained_growth(
            self.reserved,
            Some(next),
            site,
        )
        .map_err(RelicPlanError::control)?;
        self.reserved = next;
        Ok(())
    }
}
