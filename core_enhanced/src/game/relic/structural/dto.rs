//! Owned explanation only. No source/action/resource token or executable owner.
use super::{
    capabilities::{structural_capabilities, RelicCapabilityDescriptor},
    CompileUsage, EffectiveEffectKey, EffectiveProperty, EmployeeEffectivePlan, PlanContribution,
    ProspectiveProvider, RelicPlanError, StructuralRecipeDef,
};
use crate::game::data::content_pack::ContentIdentity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub enum RelicRuleState {
    Active,
    Dormant { reason: RelicDormantReason },
}
#[derive(Debug, Clone, Serialize)]
pub enum RelicDormantReason {
    ConditionUnmet,
    NoMatchingEffect,
    ProviderUnavailable,
}
#[derive(Debug, Clone, Serialize)]
pub struct RelicRuleStateDto {
    pub contribution: PlanContribution,
    pub state: RelicRuleState,
    pub targets: Vec<EffectiveEffectAddressDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveEffectAddressDto {
    pub provider_index: usize,
    pub effect: EffectiveEffectKey,
}
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveEffectDto {
    pub key: EffectiveEffectKey,
    pub provider_index: usize,
    pub operation: super::authoring::EffectOperation,
    pub tags: Vec<String>,
    pub properties: Vec<EffectiveProperty>,
    pub recipes: Vec<StructuralRecipeDef>,
    pub recipe_contributions: Vec<PlanContribution>,
    /// Owned explanation of the source amount; no compiled execution token.
    pub amount: Option<std::sync::Arc<crate::game::value_expr::AmountExpr>>,
    pub source_creation: Option<super::authoring::ShieldCreationModifierDef>,
}
#[derive(Debug, Clone, Serialize)]
pub struct RelicEffectivePlanDto {
    pub schema_version: u32,
    pub scope: &'static str,
    /// A compiled overlay is not yet a battle execution acceptance witness.
    pub execution_status: &'static str,
    pub generation: ContentIdentity,
    pub employee_uuid: Uuid,
    pub fingerprint: String,
    pub providers: Vec<ProspectiveProvider>,
    pub rules: Vec<RelicRuleStateDto>,
    pub effects: Vec<EffectiveEffectDto>,
    pub capabilities: Vec<RelicCapabilityDescriptor>,
    pub compile_usage: CompileUsage,
}
pub(crate) fn explain(
    plan: &EmployeeEffectivePlan,
) -> Result<RelicEffectivePlanDto, RelicPlanError> {
    let mut work = super::Work::new(
        plan.generation
            .runtime_safety()
            .expression()
            .max_compile_work,
    );
    work.charge(
        plan.effects
            .len()
            .checked_add(plan.rules.len())
            .ok_or_else(|| {
                RelicPlanError::new(super::FaultCode::ArithmeticOverflow, "DTO count")
            })?,
        "relic_structural_explain",
    )?;
    super::compiler::admit_serialized_copy(plan.generation.identity(), &mut work)?;
    super::compiler::admit_serialized_copy(&plan.fingerprint, &mut work)?;
    super::compiler::admit_serialized_copy(&plan.providers, &mut work)?;
    super::compiler::admit_serialized_copy(&plan.rules, &mut work)?;
    let mut dto = RelicEffectivePlanDto {
        schema_version: 1,
        scope: "prospective_loadout",
        execution_status: "runtime_supported_prospective_plan",
        generation: plan.generation.identity().clone(),
        employee_uuid: plan.employee_uuid,
        fingerprint: plan.fingerprint.clone(),
        providers: plan.providers.clone(),
        rules: plan.rules.clone(),
        effects: Vec::new(),
        capabilities: structural_capabilities().to_vec(),
        compile_usage: plan.usage.clone(),
    };
    work.reserve(
        plan.effects
            .len()
            .checked_mul(std::mem::size_of::<EffectiveEffectDto>()),
        "relic_structural_explain_storage",
    )?;
    for e in plan.effects.iter().filter(|e| e.present) {
        super::compiler::admit_serialized_copy(e, &mut work)?;
        let amount = e
            .amount
            .as_deref()
            .or_else(|| super::compiler::original_expression(e, plan.generation.database()));
        let creation = super::compiler::personal_creation(e, plan.generation.database());
        super::compiler::admit_serialized_copy(&amount, &mut work)?;
        super::compiler::admit_serialized_copy(&creation, &mut work)?;
        dto.effects.push(EffectiveEffectDto {
            key: e.key.clone(),
            provider_index: e.provider,
            operation: e.operation,
            tags: e.tags.clone(),
            properties: e.properties.clone(),
            recipes: e.recipes.clone(),
            recipe_contributions: e.recipe_contributions.clone(),
            amount: amount.map(|amount| std::sync::Arc::new(amount.clone())),
            source_creation: creation.cloned(),
        });
    }
    let bound = dto
        .effects
        .len()
        .checked_mul(dto.effects.len())
        .ok_or_else(|| {
            RelicPlanError::new(super::FaultCode::ArithmeticOverflow, "DTO sort bound")
        })?;
    work.charge(bound, "relic_structural_explain_order")?;
    dto.effects
        .sort_by(|a, b| (a.provider_index, &a.key).cmp(&(b.provider_index, &b.key)));
    Ok(dto)
}
