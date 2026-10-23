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
    Pending { reason: RelicPendingReason },
    Dormant { reason: RelicDormantReason },
}
#[derive(Debug, Clone, Serialize)]
pub enum RelicPendingReason {
    ActualEventPredicate,
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
    /// Action/delivery changes include valid sites with no per-recipient effects.
    pub action_modifiers: Vec<ActionModifierDto>,
    pub causal_rules: Vec<CausalRuleDto>,
    pub wedge_rules: Vec<WedgeRuleDto>,
    pub capabilities: Vec<RelicCapabilityDescriptor>,
    pub causal_capabilities: Vec<super::capabilities::RelicCausalCapabilityDescriptor>,
    pub compile_usage: CompileUsage,
}
#[derive(Debug, Clone, Serialize)]
pub struct ActionModifierDto {
    pub provider_index: usize,
    pub contribution: PlanContribution,
    pub change: crate::game::ability::action::ActionChange,
}
#[derive(Debug, Clone, Serialize)]
pub struct CausalRuleDto {
    pub contribution: PlanContribution,
    pub selector: super::authoring::StructuralSelectorDef,
    pub condition: super::authoring::RelicConditionDef,
    pub event: Option<super::authoring::CausalEventDef>,
    pub source: crate::game::data::relic_data::RelicDamageSelector,
    pub interval_ms: Option<u64>,
    pub payload: super::authoring::CausalPayloadDef,
}
#[derive(Debug, Clone, Serialize)]
pub struct WedgeRuleDto {
    pub contribution: PlanContribution,
    pub selector: super::authoring::StructuralSelectorDef,
    pub condition: super::authoring::RelicConditionDef,
    pub increment_percent: u32,
    pub effect_sites: Vec<crate::game::ability::authoring::EffectSite>,
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
        action_modifiers: Vec::new(),
        causal_rules: Vec::new(),
        wedge_rules: Vec::new(),
        capabilities: structural_capabilities().to_vec(),
        causal_capabilities: super::capabilities::causal_capabilities().to_vec(),
        compile_usage: plan.usage.clone(),
    };
    for modifier in &plan.action_modifiers {
        super::compiler::admit_serialized_copy(modifier, &mut work)?;
        work.reserve(Some(std::mem::size_of::<ActionModifierDto>()), "action_modifier_dto")?;
        dto.action_modifiers.push(ActionModifierDto {provider_index:modifier.provider, contribution:modifier.contribution.clone(), change:modifier.change.clone()});
    }
    for rule in &plan.causal_rules {
        super::compiler::admit_serialized_copy(rule, &mut work)?;
        work.reserve(
            Some(std::mem::size_of::<CausalRuleDto>()),
            "causal_dto_entry",
        )?;
        let entry = CausalRuleDto {
            contribution: rule.contribution.clone(),
            selector: rule.selector.clone(),
            condition: rule.condition.clone(),
            event: rule.event,
            source: rule.source.clone(),
            interval_ms: rule.interval_ms,
            payload: rule.payload.clone(),
        };
        dto.causal_rules.push(entry);
    }
    for rule in &plan.wedge_rules {
        super::compiler::admit_serialized_copy(rule, &mut work)?;
        work.reserve(Some(std::mem::size_of::<WedgeRuleDto>()), "wedge_dto_entry")?;
        dto.wedge_rules.push(WedgeRuleDto {
            contribution: rule.contribution.clone(),
            selector: rule.selector.clone(),
            condition: rule.condition.clone(),
            increment_percent: rule.increment_percent,
            effect_sites: rule.effect_sites.clone(),
        });
    }
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
