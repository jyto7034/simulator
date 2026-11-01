use super::{
    authoring::*, capabilities, compose::NumericProgram, dto::*, validation, CompileUsage,
    EffectBody, EffectiveEffect, EffectiveEffectKey, EffectivePlanComponents, EffectivePlanInput,
    EffectiveProperty, EmployeeEffectivePlan, FaultCode, PlanContribution, PropertyBase,
    ProspectiveProvider, RelicPlanError, UnpublishedPlanInput, Work,
};
use crate::game::{
    ability::{
        authoring::ActionSite,
        sustained::{SupportCoefficientScope, SustainedEffectDef},
        DeliveryDef, SkillCastSelection, SkillEffectDef, SkillTarget,
    },
    data::{
        relic_data::{RelicDamageTarget, RelicDefinition},
        GameDataBase,
    },
    value_expr::{AmountExpr, DeliveryCapabilities},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{io::Write, mem::size_of, sync::Arc};

pub(crate) struct Binding<'a> {
    pub rule: &'a RelicRuleDef,
    pub contribution: PlanContribution,
    pub provider: usize,
    pub target: Option<usize>,
    pub anchor: ActionSite,
    pub granted: Vec<usize>,
    pub active: bool,
}
fn overflow() -> RelicPlanError {
    RelicPlanError::new(FaultCode::ArithmeticOverflow, "structural bound overflow")
}
fn unsupported(cause: &str) -> RelicPlanError {
    RelicPlanError::new(FaultCode::UnsupportedOperation, cause)
}

pub(crate) fn serialized_size(
    value: &impl Serialize,
    work: &mut Work,
) -> Result<usize, RelicPlanError> {
    struct Count<'a> {
        bytes: usize,
        work: &'a mut Work,
        failure: Option<RelicPlanError>,
    }
    impl Write for Count<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if let Err(e) = self.work.charge(bytes.len(), "relic_structural_owned_text") {
                self.failure = Some(e);
                return Err(std::io::Error::other("structural work refused"));
            }
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut out = Count {
        bytes: 0,
        work,
        failure: None,
    };
    let result = serde_json::to_writer(&mut out, value);
    if let Some(e) = out.failure {
        return Err(e);
    }
    result.map_err(|e| RelicPlanError::new(FaultCode::InvalidReference, e.to_string()))?;
    Ok(out.bytes)
}
pub(super) use super::storage::admit_owned_copy;
// Mutation detaches only the selected candidate. Nested immutable Arc payloads
// are still shared; completed-plan ownership measures every owning edge.
pub(super) fn edit_effect<'a>(
    effect: &'a mut super::SharedEffect,
    work: &mut Work,
) -> Result<&'a mut EffectiveEffect, RelicPlanError> {
    if Arc::strong_count(effect) != 1 || Arc::weak_count(effect) != 0 {
        super::storage::admit_shared_clone(effect.as_ref(), work)?;
    }
    Ok(Arc::make_mut(effect))
}

fn literal(property: RelicProperty, value: TypedRelicValue) -> EffectiveProperty {
    EffectiveProperty {
        property,
        base: PropertyBase::Literal(value),
        program: NumericProgram::default(),
        final_value: None,
        contributions: Vec::new(),
    }
}
fn expression(property: RelicProperty) -> EffectiveProperty {
    EffectiveProperty {
        property,
        base: PropertyBase::RuntimeExpression,
        program: NumericProgram::default(),
        final_value: None,
        contributions: Vec::new(),
    }
}
fn properties(
    op: &SkillEffectDef,
    delivery: &DeliveryDef,
) -> (EffectOperation, Vec<EffectiveProperty>) {
    let (operation, mut values) = match op {
        SkillEffectDef::Damage { amount, .. } => (
            EffectOperation::Damage,
            vec![literal(
                RelicProperty::DamageAmount,
                TypedRelicValue::Hp(i64::from(amount.unsigned_abs())),
            )],
        ),
        SkillEffectDef::DamageExpr { .. } => (
            EffectOperation::Damage,
            vec![expression(RelicProperty::DamageAmount)],
        ),
        SkillEffectDef::Heal { .. } | SkillEffectDef::HealExpr { .. } => {
            (EffectOperation::Heal, Vec::new())
        }
        SkillEffectDef::HealthCost { .. } => (EffectOperation::HealthCost, Vec::new()),
        SkillEffectDef::PersonalShieldGrant { definition } => {
            let mut p = vec![expression(RelicProperty::PersonalShieldInitialAmount)];
            if let PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { radius_units } =
                definition.recipients
            {
                if definition.support.as_ref().and_then(|s| s.range.as_ref()).is_none() { p.push(literal(
                    RelicProperty::SupportAreaRadius,
                    TypedRelicValue::DataUnits(i64::from(radius_units)),
                )); }
            }
            (EffectOperation::PersonalShieldGrant, p)
        }
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::ChannelAura { aura, .. },
        } => {
            let mut p = if aura.support.as_ref().and_then(|s| s.range.as_ref()).is_none() { vec![literal(
                RelicProperty::SupportAreaRadius,
                TypedRelicValue::DataUnits(i64::from(aura.radius_units)),
            )] } else { Vec::new() };
            if aura.basic_scope == SupportCoefficientScope::RegularOnly {
                p.push(literal(
                    RelicProperty::RegularBasicDamagePercent,
                    TypedRelicValue::PercentPoints(i64::from(aura.basic_damage_percent)),
                ));
                p.push(literal(
                    RelicProperty::RegularBasicSpeedPoints,
                    TypedRelicValue::SpeedPoints(i64::from(aura.basic_speed_points)),
                ));
            }
            (EffectOperation::ChannelAura, p)
        }
        _ => (EffectOperation::Other, Vec::new()),
    };
    if operation == EffectOperation::Damage && matches!(delivery, DeliveryDef::TileArea { .. }) {
        values.push(EffectiveProperty {
            property: RelicProperty::DamageFootprint,
            base: PropertyBase::Structural,
            program: NumericProgram::default(),
            final_value: None,
            contributions: Vec::new(),
        });
    }
    (operation, values)
}

fn publication_effect_growth(
    effects: &mut Vec<super::SharedEffect>,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    if effects.len() == effects.capacity() {
        let capacity = effects
            .capacity()
            .checked_mul(2)
            .ok_or_else(overflow)?
            .max(4);
        // Previous reservations still include the old buffer during growth.
        work.reserve(
            capacity
                .checked_mul(size_of::<super::SharedEffect>())
                .and_then(|n| n.checked_add(64)),
            "relic_publication_effect_capacity",
        )?;
        effects.reserve_exact(capacity - effects.len());
    }
    Ok(())
}

fn admit_publication_original_effect(
    effects: &mut Vec<super::SharedEffect>,
    skill: &crate::game::ability::SkillDef,
    step: &crate::game::ability::SkillStepDef,
    effect: &crate::game::ability::authoring::AuthoredSkillEffect,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    work.charge(effect.tags.len(), "relic_publication_original_tags")?;
    let site_bytes = skill
        .id
        .as_str()
        .len()
        .checked_add(step.id.len())
        .and_then(|n| n.checked_add(effect.id.len()))
        .ok_or_else(overflow)?;
    // One fresh site, its key clone, and the body's separate skill ID. The
    // original site's action moves into anchor; its effect ID is temporary.
    let mut text = site_bytes
        .checked_mul(2)
        .and_then(|n| n.checked_add(skill.id.as_str().len()))
        .ok_or_else(overflow)?;
    let mut strings = effect.tags.len().checked_add(7).ok_or_else(overflow)?;
    for tag in &effect.tags {
        text = text.checked_add(tag.len()).ok_or_else(overflow)?;
    }
    if let Some(expression) = source_expr(&effect.op) {
        // The amount proof really constructs an owned compiled graph. Keep its
        // existing closed-expression upper bound; do not price the unrelated
        // whole authored effect as if it were cloned into EffectBody::Original.
        admit_owned_copy(expression, work)?;
        text = text
            .checked_add(site_bytes)
            .and_then(|n| n.checked_add(skill.id.as_str().len()))
            .and_then(|n| n.checked_add("typed:effective_plan".len() + "amount".len()))
            .ok_or_else(overflow)?;
        strings = strings.checked_add(6).ok_or_else(overflow)?;
    }
    // properties() starts with one inline value. A second/third push can
    // replace that allocation with capacity four; include both at that instant.
    // Default NumericProgram/contribution Vecs own no buffers initially.
    let property_slots = match &effect.op {
        SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. } => {
            if matches!(step.delivery, DeliveryDef::TileArea { .. }) {
                5
            } else {
                1
            }
        }
        SkillEffectDef::PersonalShieldGrant { definition } => {
            if matches!(
                definition.recipients,
                PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { .. }
            ) {
                5
            } else {
                1
            }
        }
        SkillEffectDef::Sustained {
            definition: SustainedEffectDef::ChannelAura { aura, .. },
        } => {
            if aura.basic_scope == SupportCoefficientScope::RegularOnly {
                5
            } else {
                1
            }
        }
        _ => 0usize,
    };
    let allocations = strings
        .checked_add(usize::from(!effect.tags.is_empty()))
        .and_then(|n| {
            n.checked_add(match property_slots {
                0 => 0,
                1 => 1,
                _ => 2,
            })
        })
        .and_then(|n| n.checked_mul(64))
        .ok_or_else(overflow)?;
    work.reserve(
        effect
            .tags
            .len()
            .checked_mul(size_of::<String>())
            .and_then(|n| n.checked_add(text))
            .and_then(|n| {
                property_slots
                    .checked_mul(size_of::<EffectiveProperty>())
                    .and_then(|p| n.checked_add(p))
            })
            .and_then(|n| n.checked_add(allocations)),
        "relic_publication_original_payload",
    )?;
    let clone_slots = text
        .checked_add(strings)
        .and_then(|n| n.checked_add(1))
        .ok_or_else(overflow)?;
    crate::authoring::control::admit(
        crate::authoring::control::Resource::CloneSlots,
        0,
        clone_slots,
        "relic_publication_original_copy",
    )
    .map_err(RelicPlanError::control)?;
    work.charge(
        clone_slots
            .checked_add(property_slots)
            .ok_or_else(overflow)?,
        "relic_publication_original_copy",
    )?;
    crate::game::battle::runtime_control::charge_clone(
        clone_slots,
        "relic_publication_original_copy",
    )
    .map_err(RelicPlanError::control)?;
    // Every record owns its inline fields, but its step delivery is an Arc
    // clone of the one separately admitted delivery, not another deep copy.
    publication_effect_growth(effects, work)
}
pub(super) fn primitive(g: &GrantPrimitiveDef) -> SkillEffectDef {
    match g {
        GrantPrimitiveDef::Damage {
            amount,
            damage_type,
        } => SkillEffectDef::Damage {
            amount: *amount,
            damage_type: *damage_type,
        },
        GrantPrimitiveDef::DamageExpr {
            amount,
            damage_type,
        } => SkillEffectDef::DamageExpr {
            amount: (**amount).clone(),
            damage_type: *damage_type,
        },
        GrantPrimitiveDef::Heal { amount } => SkillEffectDef::Heal { amount: *amount },
        GrantPrimitiveDef::HealExpr { amount } => SkillEffectDef::HealExpr {
            amount: (**amount).clone(),
        },
        GrantPrimitiveDef::HealthCost {
            amount,
            allow_lethal,
        } => SkillEffectDef::HealthCost {
            amount: *amount,
            allow_lethal: *allow_lethal,
        },
        GrantPrimitiveDef::PersonalShieldGrant { definition } => {
            SkillEffectDef::PersonalShieldGrant {
                definition: definition.clone(),
            }
        }
    }
}
pub(super) fn amount_caps(
    data: &GameDataBase,
    skill: &crate::game::ability::SkillDef,
    target: &SkillTarget,
    delivery: &DeliveryDef,
    work: &mut Work,
) -> Result<DeliveryCapabilities, RelicPlanError> {
    let mut caps =
        crate::game::value_expr::delivery_capabilities(skill.cast_selection(), target, delivery);
    // Preserve deadline capabilities from declared reinput child references.
    for s in &data.skill_data.skills {
        for step in &s.steps {
            work.charge(step.effects.len().checked_add(1).ok_or_else(overflow)?,
                "relic_structural_deadline_owner")?;
            for effect in &step.effects {
                let mut owns_deadline = false;
                if let SkillEffectDef::Sustained {
                    definition: SustainedEffectDef::BasicAttackOverlay { goodbye_skill: Some(id), .. }
                } = &effect.op {
                    work.charge(skill.id.as_str().len().min(id.as_str().len()) + 1,
                        "relic_structural_deadline_reference")?;
                    owns_deadline = id == &skill.id;
                }
                if let SkillEffectDef::Sustained { definition: SustainedEffectDef::State { program, .. } } = &effect.op {
                    // Published programs have a strict 256-operation bound. Charge
                    // that bound before traversing nested conditional child refs.
                    work.charge(crate::game::ability::state_program::MAX_STATE_OPERATIONS, "state_child_deadline_scan")?;
                    let mut comparison = Ok(());
                    program.visit_children(|id| {
                        if comparison.is_ok() {
                            comparison = work.charge(skill.id.as_str().len().min(id.as_str().len()) + 1,
                                "relic_structural_deadline_reference");
                            if comparison.is_ok() { owns_deadline |= id == &skill.id; }
                        }
                    });
                    comparison?;
                }
                if owns_deadline {
                    caps.deadline = true;
                    return Ok(caps);
                }
            }
        }
    }
    Ok(caps)
}
fn resolve_static_amount(
    op: &SkillEffectDef,
    values: &mut [EffectiveProperty],
    value: Option<i128>,
) -> Result<(), RelicPlanError> {
    let Some(value) = value else {
        return Ok(());
    };
    let in_range = match op {
        SkillEffectDef::HealExpr { .. } => i32::try_from(value).is_ok(),
        _ => u32::try_from(value).is_ok(),
    };
    if !in_range {
        return Err(RelicPlanError::new(
            FaultCode::OutOfRange,
            "static amount outside its existing narrowing range",
        ));
    }
    if let Some(p) = values.iter_mut().find(|p| {
        matches!(
            p.property,
            RelicProperty::DamageAmount | RelicProperty::PersonalShieldInitialAmount
        )
    }) {
        p.base = PropertyBase::Literal(TypedRelicValue::Hp(
            i64::try_from(value).map_err(|_| overflow())?,
        ));
    }
    Ok(())
}
fn validate_provider(
    p: &ProspectiveProvider,
    data: &GameDataBase,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    admit_owned_copy(p, work)?;
    let valid = match p {
        ProspectiveProvider::Fragment(f) => {
            work.charge(
                data.skill_fragment_data.fragments.len(),
                "relic_structural_fragment_lookup",
            )?;
            !f.copy_uuid.is_nil() && data.skill_fragment_data.get_by_id(&f.definition_id)
                .is_some_and(|d| matches!(&d.effect,
                    crate::game::data::skill_fragment_data::SkillFragmentEffectDef::ActiveSkill { forms, .. }
                    if forms.get(&f.form_id).is_some_and(|form| form.skill_id == f.skill_id)))
        }
        ProspectiveProvider::Equipment {
            definition_id,
            copy_uuid,
            skill_id,
        } => {
            work.charge(
                data.equipment_data.items.len(),
                "relic_structural_equipment_lookup",
            )?;
            !copy_uuid.is_nil()
                && data
                    .equipment_data
                    .get_by_id(definition_id)
                    .is_some_and(|d| {
                        d.ability_activations
                            .iter()
                            .any(|a| a.ability_id == *skill_id)
                    })
        }
    };
    work.charge(
        data.skill_data.skills.len(),
        "relic_structural_skill_lookup",
    )?;
    if !valid || data.skill_data.get_by_id(p.skill_id()).is_none() {
        return Err(RelicPlanError::new(
            FaultCode::ProviderUnavailable,
            "provider/copy/form/skill mismatch",
        ));
    }
    Ok(())
}
fn source_expr(op: &SkillEffectDef) -> Option<&AmountExpr> {
    match op {
        SkillEffectDef::DamageExpr { amount, .. }
        | SkillEffectDef::HealExpr { amount, .. }
        | SkillEffectDef::ShieldExpr { amount, .. } => Some(amount),
        SkillEffectDef::PersonalShieldGrant { definition } => Some(&definition.amount),
        _ => None,
    }
}
pub(crate) fn original_expression<'a>(
    e: &'a EffectiveEffect,
    data: &'a GameDataBase,
) -> Option<&'a AmountExpr> {
    match &e.body {
        EffectBody::Original {
            skill_id,
            step,
            effect,
        } => source_expr(&data.skill_data.get_by_id(skill_id)?.steps[*step].effects[*effect].op),
        EffectBody::Granted(g) => match &g.op {
            GrantPrimitiveDef::DamageExpr { amount, .. }
            | GrantPrimitiveDef::HealExpr { amount } => Some(amount),
            GrantPrimitiveDef::PersonalShieldGrant { definition } => Some(&definition.amount),
            _ => None,
        },
    }
}
pub(crate) fn personal_creation<'a>(
    e: &'a EffectiveEffect,
    data: &'a GameDataBase,
) -> Option<&'a ShieldCreationModifierDef> {
    match &e.body {
        EffectBody::Original {
            skill_id,
            step,
            effect,
        } => match &data.skill_data.get_by_id(skill_id)?.steps[*step].effects[*effect].op {
            SkillEffectDef::PersonalShieldGrant { definition } => {
                definition.source_creation.as_ref()
            }
            _ => None,
        },
        EffectBody::Granted(g) => match &g.op {
            GrantPrimitiveDef::PersonalShieldGrant { definition } => {
                definition.source_creation.as_ref()
            }
            _ => None,
        },
    }
}
fn validate_grant_carrier(
    op: &SkillEffectDef,
    target: &SkillTarget,
    selection: SkillCastSelection,
    delivery: &DeliveryDef,
) -> Result<(), RelicPlanError> {
    match op {
        SkillEffectDef::HealthCost { .. }
            if !matches!(target, SkillTarget::SelfUnit)
                || !matches!(delivery, DeliveryDef::Instant) =>
        {
            Err(RelicPlanError::new(
                FaultCode::InvalidPreserveContract,
                "granted HP cost requires its existing Instant SelfUnit carrier",
            ))
        }
        SkillEffectDef::PersonalShieldGrant { definition } => {
            validation::shield(definition)?;
            let valid = match definition.recipients {
                PersonalShieldRecipientsDef::PlayerEmployeesAtRelease { .. } => {
                    matches!(target, SkillTarget::SelfUnit)
                }
                PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease { .. } => {
                    matches!(target, SkillTarget::AllySingle)
                        && selection == SkillCastSelection::PlayerUnit
                }
            };
            if valid && matches!(delivery, DeliveryDef::Instant) {
                Ok(())
            } else {
                Err(RelicPlanError::new(
                    FaultCode::InvalidPreserveContract,
                    "personal grant requires its explicit Instant player carrier",
                ))
            }
        }
        _ => Ok(()),
    }
}
pub(crate) fn compile_effective_plan(
    input: EffectivePlanInput<'_>,
) -> Result<Option<Arc<EmployeeEffectivePlan>>, RelicPlanError> {
    compile_components(UnpublishedPlanInput {
        data: input.data.database(),
        generation: input.data.identity(),
        employee_uuid: input.employee_uuid,
        inventory: input.inventory,
        providers: input.providers,
    })
    .map(|components| {
        components.map(|components| {
            Arc::new(EmployeeEffectivePlan {
                generation: input.data.clone(),
                components,
            })
        })
    })
}
pub(crate) fn compile_components(
    input: UnpublishedPlanInput<'_>,
) -> Result<Option<EffectivePlanComponents>, RelicPlanError> {
    compile_components_with_basis(input, 0, false, None, None)
}

// This immutable base is independent of the relic loadout. It contains every
// provider in the exact full context, with its original copy/form identities.
// Only publication owns these caches; runtime compilers retain their contract.
pub(super) struct PreparedProviders<'a> {
    data: &'a GameDataBase,
    providers: Arc<Vec<ProspectiveProvider>>,
    effects: Vec<super::SharedEffect>,
    execution: Vec<super::execution::EffectiveProviderPlan>,
    storage: super::storage::PreparedStorage,
}

#[cfg(test)]
mod prepared_provider_tests {
    use super::*;

    #[test]
    fn publication_prepared_loadout_rejects_other_inventory_employee_and_generation() {
        let (data, old) = crate::game::battle::core::relic_structural_tests::prepared_plan();
        let database = data.database();
        let definition = database.relic_data.get_by_id("tb_native_relic").unwrap();
        let inventory = crate::game::relic::RelicInventory::publication_input(old.employee_uuid,&[(definition,crate::game::relic::RelicVariant::Normal)],&database.relic_data).unwrap().unwrap();
        let other_inventory = crate::game::relic::RelicInventory::publication_input(old.employee_uuid,&[(definition,crate::game::relic::RelicVariant::Normal)],&database.relic_data).unwrap().unwrap();
        let other_database = crate::game::data::GameDataBuilder::from_validated(&data).build();
        let (providers,basis)=PreparedProviders::new(database,&old.providers,0).unwrap();
        let (loadout,reserved)=PreparedLoadout::new(database,&inventory,old.employee_uuid,basis).unwrap();
        for mode in 0..3 {
            let result=compile_components_with_basis(UnpublishedPlanInput{
                data:if mode==0 {&other_database}else{database},
                generation:data.identity(),
                employee_uuid:if mode==1 {uuid::Uuid::from_u128(0xd279)}else{old.employee_uuid},
                inventory:if mode==2 {&other_inventory}else{&inventory},
                providers:&old.providers,
            },basis+reserved,true,Some(&providers),Some(&loadout));
            assert!(result.err().unwrap().to_string().contains("loadout input mismatch"));
        }
        compile_components_with_basis(UnpublishedPlanInput{data:database,generation:data.identity(),employee_uuid:old.employee_uuid,inventory:&inventory,providers:&old.providers},basis+reserved,true,Some(&providers),Some(&loadout)).unwrap().unwrap();
    }

    #[test]
    fn publication_prepared_full_context_matches_fresh_and_keeps_cache_immutable() {
        let (data, old) = crate::game::battle::core::relic_structural_tests::prepared_plan();
        let database = data.database();
        let definition = database.relic_data.get_by_id("tb_native_relic").unwrap();
        let inventory = crate::game::relic::RelicInventory::publication_input(
            old.employee_uuid,
            &[(definition, crate::game::relic::RelicVariant::Normal)],
            &database.relic_data,
        )
        .unwrap()
        .unwrap();
        let empty = crate::game::relic::RelicInventory::default();
        let generation = data.identity();
        let (prepared, basis) = PreparedProviders::new(database, &old.providers, 0).unwrap();
        let original = serde_json::to_value((&prepared.providers, &prepared.effects)).unwrap();
        for inventory in [&inventory, &empty, &inventory] {
            let input = || UnpublishedPlanInput {
                data: database,
                generation,
                employee_uuid: old.employee_uuid,
                inventory,
                providers: &old.providers,
            };
            let fresh = compile_components_with_basis(input(), basis, true, None, None).unwrap();
            let (loadout, reserved) = PreparedLoadout::new(database, inventory, old.employee_uuid, basis).unwrap();
            let reused = compile_components_with_basis(input(), basis + reserved, true, Some(&prepared), Some(&loadout)).unwrap();
            assert_eq!(fresh.is_some(), reused.is_some());
            if let (Some(a), Some(b)) = (fresh, reused) {
                assert_eq!(
                    serde_json::to_value((
                        &a.providers,
                        &a.effects,
                        &a.grants,
                        &a.rules,
                        &a.action_modifiers
                    ))
                    .unwrap(),
                    serde_json::to_value((
                        &b.providers,
                        &b.effects,
                        &b.grants,
                        &b.rules,
                        &b.action_modifiers
                    ))
                    .unwrap()
                );
                // Cloning the prepared Vec removes spare capacity; semantic
                // equivalence does not require identical allocation history.
                assert!(b.retained_bytes <= a.retained_bytes);
                for plan in [&a, &b] {
                    let mut work = Work::new(u64::MAX);
                    assert_eq!(
                        plan.retained_bytes,
                        super::super::storage::measure(plan, generation, &mut work).unwrap()
                    );
                }
                assert_eq!(a.execution.len(), b.execution.len());
                for (a, b) in a.execution.iter().zip(&b.execution) {
                    assert_eq!(
                        serde_json::to_value((&a.skill, &a.flow, &a.effects)).unwrap(),
                        serde_json::to_value((&b.skill, &b.flow, &b.effects)).unwrap()
                    );
                    assert_eq!(a.amounts.len(), b.amounts.len());
                    assert_eq!(
                        format!("{:?}", a.amounts.states()),
                        format!("{:?}", b.amounts.states())
                    );
                    for ((site_a, entry_a), (site_b, entry_b)) in
                        a.amounts.entries().zip(b.amounts.entries())
                    {
                        assert_eq!(site_a, site_b);
                        assert_eq!(entry_a.kind(), entry_b.kind());
                        let mut left = entry_a.plan().as_ref().clone();
                        let mut right = entry_b.plan().as_ref().clone();
                        // Each compiler gives the amount compiler its actual
                        // remaining work. That compile-only cap may differ.
                        for plan in [&mut left, &mut right] {
                            assert!(plan.cost().compile_work <= plan.limits.max_compile_work);
                            assert!(
                                plan.limits.max_compile_work
                                    <= database.runtime_safety().expression().max_compile_work
                            );
                            plan.limits.max_compile_work = 0;
                        }
                        assert_eq!(format!("{left:?}"), format!("{right:?}"));
                    }
                    assert!(Arc::ptr_eq(
                        &b.flow,
                        &database
                            .skill_data
                            .compiled_flow(b.skill.id.as_str())
                            .unwrap()
                    ));
                }
            }
            assert_eq!(
                serde_json::to_value((&prepared.providers, &prepared.effects)).unwrap(),
                original
            );
        }
    }
}
impl<'a> PreparedProviders<'a> {
    pub(super) fn providers(&self) -> &[ProspectiveProvider] {
        &self.providers
    }
    pub(super) fn new(
        data: &'a GameDataBase,
        providers: &[ProspectiveProvider],
        retained_basis: usize,
    ) -> Result<(Self, usize), RelicPlanError> {
        let mut work = Work::for_publication(
            data.runtime_safety().expression().max_compile_work,
            retained_basis,
        );
        let (providers, mut effects) = prepare_providers(data, providers, &mut work, true)?;
        for shared in &mut effects {
            let effect = edit_effect(shared, &mut work)?;
            for property in &mut effect.properties {
                if let PropertyBase::Literal(value) = property.base {
                    property.final_value = Some(value);
                }
            }
        }
        // This is a private original-input lowering, not a relic proof or a
        // published runtime plan. Whole-context rules still run for each set.
        let baseline = EffectivePlanComponents {
            employee_uuid: uuid::Uuid::nil(),
            providers,
            copies: Vec::new(),
            effects,
            grants: Vec::new(),
            execution: Vec::new(),
            causal_rules: Vec::new(),
            wedge_rules: Vec::new(),
            action_modifiers: Vec::new(),
            rules: Vec::new(),
            fingerprint: String::new(),
            usage: CompileUsage {
                work: 0,
                reserved_bytes: 0,
                candidate_count: 0,
                binding_count: 0,
            },
            retained_bytes: 0,
        };
        let execution = super::execution::lower(&baseline, data, &mut work, None)?;
        let storage = super::storage::PreparedStorage::new(&baseline.providers, &baseline.effects, &execution, &mut work)?;
        Ok((
            Self {
                data,
                providers: baseline.providers,
                effects: baseline.effects,
                execution,
                storage,
            },
            work.reserved,
        ))
    }
}
fn prepare_providers(
    data: &GameDataBase,
    input_providers: &[ProspectiveProvider],
    work: &mut Work,
    publication: bool,
) -> Result<(Arc<Vec<ProspectiveProvider>>, Vec<super::SharedEffect>), RelicPlanError> {
    let mut providers = Vec::new();
    work.reserve(
        input_providers
            .len()
            .checked_mul(size_of::<ProspectiveProvider>()),
        "relic_structural_providers",
    )?;
    for p in input_providers {
        validate_provider(p, data, work)?;
        providers.push(p.clone());
    }
    // A zero/one-provider vector is already canonical and cannot duplicate a
    // provider. Avoid synthesizing JSON sort keys when there is no comparison.
    let providers: Vec<_> = if providers.len() < 2 {
        providers
    } else {
        work.reserve(
            providers
                .len()
                .checked_mul(size_of::<String>() + size_of::<(ProspectiveProvider, String)>()),
            "relic_structural_provider_order_storage",
        )?;
        let mut provider_order = Vec::with_capacity(providers.len());
        for p in &providers {
            let bytes = serialized_size(p, work)?;
            work.reserve(Some(bytes), "relic_structural_provider_order")?;
            provider_order
                .push(serde_json::to_string(p).map_err(|e| {
                    RelicPlanError::new(FaultCode::InvalidReference, e.to_string())
                })?);
        }
        let mut ordered: Vec<_> = providers.into_iter().zip(provider_order).collect();
        // Keep canonical JSON order AND order-independent usage in the DTO.
        // This fixed insertion network performs the same finite comparisons
        // for every permutation. Each comparison admits the longest key plus
        // a possible swap before either operation; cancellation returns at once.
        let comparison_work = ordered
            .iter()
            .map(|(_, key)| key.len())
            .max()
            .unwrap_or(0)
            .checked_add(2)
            .ok_or_else(overflow)?;
        for index in 1..ordered.len() {
            for at in (1..=index).rev() {
                work.charge(comparison_work, "relic_structural_provider_order")?;
                if ordered[at - 1].1 > ordered[at].1 {
                    ordered.swap(at - 1, at);
                }
            }
        }
        for pair in ordered.windows(2) {
            work.charge(
                pair[0].1.len().min(pair[1].1.len()) + 1,
                "relic_structural_provider_duplicate",
            )?;
            if pair[0].1 == pair[1].1 {
                return Err(RelicPlanError::new(
                    FaultCode::InvalidReference,
                    "duplicate provider",
                ));
            }
        }
        ordered.into_iter().map(|(p, _)| p).collect()
    };
    let mut effects = Vec::new();
    for (pi, p) in providers.iter().enumerate() {
        let skill = data.skill_data.get_by_id(p.skill_id()).unwrap();
        for (si, step) in skill.steps.iter().enumerate() {
            admit_owned_copy(&step.delivery, work)?;
            let delivery = Arc::new(step.delivery.clone());
            let amount_capabilities = amount_caps(data, skill, &step.target, &step.delivery, work)?;
            for (ei, e) in step.effects.iter().enumerate() {
                if publication {
                    admit_publication_original_effect(&mut effects, skill, step, e, work)?;
                } else {
                    admit_owned_copy(e, work)?;
                }
                let site = validation::effect_site(&skill.id, &step.id, &e.id);
                let (operation, mut properties) = properties(&e.op, &step.delivery);
                if let Some(expr) = source_expr(&e.op) {
                    let value = validation::amount(
                        expr,
                        site.clone(),
                        operation == EffectOperation::PersonalShieldGrant,
                        data,
                        amount_capabilities,
                        work,
                    )?;
                    resolve_static_amount(&e.op, &mut properties, value)?;
                }
                work.reserve(Some(size_of::<EffectiveEffect>() + 64), "effective_effect_owner")?;
                effects.push(Arc::new(EffectiveEffect {
                    key: EffectiveEffectKey::Original { site: site.clone() },
                    provider: pi,
                    anchor: site.action,
                    tags: e.tags.clone(),
                    operation,
                    delivery: Arc::clone(&delivery),
                    properties,
                    recipes: Vec::new(),
                    recipe_contributions: Vec::new(),
                    body: EffectBody::Original {
                        skill_id: skill.id.clone(),
                        step: si,
                        effect: ei,
                    },
                    amount_capabilities,
                    amount: None,
                    original: true,
                    present: true,
                }));
            }
        }
    }
    work.reserve(Some(size_of::<Vec<ProspectiveProvider>>() + 64), "effective_provider_identity_owner")?;
    Ok((Arc::new(providers), effects))
}

// Only catalog publication shares this borrowed proof. The entire immutable
// inventory, employee and generation remain its identity; each provider context
// still resolves all conditions, grants, numeric edits and action changes.
pub(super) struct PreparedLoadout<'a> {
    data: &'a GameDataBase,
    inventory: &'a crate::game::relic::RelicInventory,
    employee: uuid::Uuid,
    copies: Vec<(&'a crate::game::relic::OwnedRelic, &'a RelicDefinition)>,
    structural: bool,
}
impl<'a> PreparedLoadout<'a> {
    pub(super) fn new(
        data: &'a GameDataBase,
        inventory: &'a crate::game::relic::RelicInventory,
        employee: uuid::Uuid,
        retained_basis: usize,
    ) -> Result<(Self, usize), RelicPlanError> {
        let mut work = Work::for_publication(
            data.runtime_safety().expression().max_compile_work,
            retained_basis,
        );
        let result = prepare_loadout(data, inventory, employee, &mut work, true)?;
        Ok((result, work.reserved))
    }
}
fn prepare_loadout<'a>(
    data: &'a GameDataBase,
    inventory: &'a crate::game::relic::RelicInventory,
    employee: uuid::Uuid,
    work: &mut Work,
    publication: bool,
) -> Result<PreparedLoadout<'a>, RelicPlanError> {
    let slots = inventory.employee_slots(employee);
    inventory
        .validate_employee_slots(&slots, &data.relic_data)
        .map_err(RelicPlanError::control)?;
    if publication {
        work.reserve(
            Some(4 * size_of::<(&super::super::OwnedRelic, &RelicDefinition)>() + 64),
            "relic_publication_borrowed_copies",
        )?;
    }
    let mut borrowed = Vec::new();
    for id in slots.into_iter().flatten() {
        let copy = inventory.get(id).ok_or_else(|| {
            RelicPlanError::new(FaultCode::InvalidReference, "missing relic copy")
        })?;
        let definition = data
            .relic_data
            .get_by_id(&copy.definition_id)
            .ok_or_else(|| {
                RelicPlanError::new(FaultCode::InvalidReference, "missing relic definition")
            })?;
        borrowed.push((copy, definition));
    }
    // U08-only loadouts keep the existing numeric/original consumer path.
    let mut structural = false;
    for (copy, definition) in &borrowed {
        structural |= !validation::rules_for(definition, copy.variant).is_empty()
            || super::support::declares_conversion_for(data, &copy.definition_id, copy.variant)?
            || super::action::declares(data, &copy.definition_id, copy.variant)?;
    }
    if structural {
        work.charge(9, "relic_structural_copy_order")?;
        borrowed.sort_by_key(|(c, _)| c.copy_uuid);
        for (c, d) in &borrowed {
            validation::validate_rules(&d.id, validation::rules_for(d, c.variant))?;
        }
    }
    Ok(PreparedLoadout {
        data,
        inventory,
        employee,
        copies: borrowed,
        structural,
    })
}

pub(super) fn compile_publication_components(
    input: UnpublishedPlanInput<'_>,
    retained_basis: usize,
    prepared: &PreparedProviders<'_>,
    loadout: &PreparedLoadout<'_>,
) -> Result<(), RelicPlanError> {
    // Publication may use a nonsemantic fingerprint placeholder. Its proof
    // graph is discarded here and cannot enter a runtime plan or public DTO.
    compile_components_with_basis(input, retained_basis, true, Some(prepared), Some(loadout)).map(|_| ())
}

fn compile_components_with_basis(
    input: UnpublishedPlanInput<'_>,
    retained_basis: usize,
    publication: bool,
    prepared: Option<&PreparedProviders<'_>>,
    prepared_loadout: Option<&PreparedLoadout<'_>>,
) -> Result<Option<EffectivePlanComponents>, RelicPlanError> {
    if input.employee_uuid.is_nil() {
        return Err(RelicPlanError::new(
            FaultCode::InvalidReference,
            "nil employee",
        ));
    }
    let data = input.data;
    let limit = data.runtime_safety().expression().max_compile_work;
    let mut work = if publication {
        Work::for_publication(limit, retained_basis)
    } else {
        Work::new(limit)
    };
    let local_loadout;
    let loadout = if let Some(prepared) = prepared_loadout {
        work.charge(1, "publication_prepared_loadout_check")?;
        if !std::ptr::eq(prepared.data, data)
            || !std::ptr::eq(prepared.inventory, input.inventory)
            || prepared.employee != input.employee_uuid {
            return Err(RelicPlanError::new(FaultCode::InvalidReference, "publication loadout input mismatch"));
        }
        prepared
    } else {
        local_loadout = prepare_loadout(data, input.inventory, input.employee_uuid, &mut work, publication)?;
        &local_loadout
    };
    if !loadout.structural { return Ok(None); }
    let borrowed = &loadout.copies;
    let (providers, mut effects) = if let Some(prepared) = prepared {
        work.charge(1, "publication_prepared_context_check")?;
        if !std::ptr::eq(prepared.data, data) {
            return Err(RelicPlanError::new(
                FaultCode::InvalidReference,
                "publication provider generation mismatch",
            ));
        }
        if !std::ptr::eq(input.providers, prepared.providers.as_slice()) {
            let bytes = serialized_size(&input.providers, &mut work)?;
            work.charge(bytes, "publication_prepared_context_check")?;
            if input.providers != prepared.providers.as_slice() {
                return Err(RelicPlanError::new(
                    FaultCode::InvalidReference,
                    "publication provider context mismatch",
                ));
            }
        }
        super::storage::admit_shared_clone(&prepared.providers, &mut work)?;
        super::storage::admit_shared_clone(&prepared.effects, &mut work)?;
        (prepared.providers.clone(), prepared.effects.clone())
    } else {
        prepare_providers(data, input.providers, &mut work, publication)?
    };
    let conversion_rules = super::support::apply_conversions(data, &borrowed, &mut effects, &mut work)?;
    let mut bindings = Vec::new();
    for (copy, d) in borrowed {
        for rule in validation::rules_for(d, copy.variant) {
            let contribution = PlanContribution {
                copy_uuid: copy.copy_uuid,
                definition_id: d.id.clone(),
                variant: copy.variant,
                rule_id: rule.id.clone(),
            };
            if let RelicRuleActionDef::Grant(body) = &rule.apply {
                for (pi, p) in providers.iter().enumerate() {
                    if !super::closure::provider_matches(&rule.selector.provider, p) {
                        continue;
                    }
                    let skill = data.skill_data.get_by_id(p.skill_id()).unwrap();
                    let action_id = match body {
                        GrantBodyDef::InlineEffects { action_id, .. } => action_id,
                        GrantBodyDef::FiniteFlow {
                            anchor_action_id, ..
                        } => anchor_action_id,
                    };
                    let Some(step) = skill.steps.iter().find(|s| &s.id == action_id) else {
                        continue;
                    };
                    let anchor = ActionSite {
                        definition_id: skill.id.clone(),
                        action_id: action_id.clone(),
                    };
                    let mut granted = Vec::new();
                    match body {
                        GrantBodyDef::InlineEffects {
                            effects: declared, ..
                        } => {
                            for e in declared {
                                add_granted(
                                    &mut effects,
                                    &mut granted,
                                    e,
                                    pi,
                                    &anchor,
                                    &step.id,
                                    &step.delivery,
                                    &step.target,
                                    skill.cast_selection(),
                                    amount_caps(
                                        data,
                                        skill,
                                        &step.target,
                                        &step.delivery,
                                        &mut work,
                                    )?,
                                    &contribution,
                                    data,
                                    &mut work,
                                    publication,
                                )?;
                            }
                        }
                        GrantBodyDef::FiniteFlow { steps, flow, .. } => {
                            super::execution::validate_grant_flow(
                                skill, &step.id, steps, flow, data, &mut work,
                            )?;
                            for s in steps {
                                for e in &s.effects {
                                    add_granted(
                                        &mut effects,
                                        &mut granted,
                                        e,
                                        pi,
                                        &anchor,
                                        &s.id,
                                        &s.delivery,
                                        &s.target,
                                        skill.cast_selection(),
                                        amount_caps(
                                            data,
                                            skill,
                                            &s.target,
                                            &s.delivery,
                                            &mut work,
                                        )?,
                                        &contribution,
                                        data,
                                        &mut work,
                                        publication,
                                    )?;
                                }
                            }
                        }
                    }
                    work.reserve(
                        Some(size_of::<Binding<'_>>()),
                        "relic_structural_grant_binding",
                    )?;
                    bindings.push(Binding {
                        rule,
                        contribution: contribution.clone(),
                        provider: pi,
                        target: None,
                        anchor,
                        granted,
                        active: false,
                    });
                }
            }
        }
    }
    let candidate_count = effects.len();
    // Non-grant bindings use the fixed universe, including dormant generated sites.
    for (copy, d) in borrowed {
        for rule in validation::rules_for(d, copy.variant) {
            if matches!(
                rule.apply,
                RelicRuleActionDef::Grant(_)
                    | RelicRuleActionDef::Subscribe(_)
                    | RelicRuleActionDef::Periodic { .. }
            ) {
                continue;
            }
            for (i, e) in effects.iter().enumerate() {
                if !super::closure::matches(&rule.selector, e, &providers, false, &mut work)? {
                    continue;
                }
                work.reserve(
                    Some(size_of::<Binding<'_>>()),
                    "relic_structural_effect_binding",
                )?;
                bindings.push(Binding {
                    rule,
                    contribution: PlanContribution {
                        copy_uuid: copy.copy_uuid,
                        definition_id: d.id.clone(),
                        variant: copy.variant,
                        rule_id: rule.id.clone(),
                    },
                    provider: e.provider,
                    target: Some(i),
                    anchor: e.anchor.clone(),
                    granted: Vec::new(),
                    active: false,
                });
            }
        }
    }
    super::closure::solve(&mut bindings, &mut effects, &providers, &mut work)?;
    for b in bindings.iter().filter(|b| b.active) {
        if let (
            Some(target),
            RelicRuleActionDef::Edit {
                property,
                operation,
            },
        ) = (b.target, &b.rule.apply)
        {
            let e = edit_effect(&mut effects[target], &mut work)?;
            let descriptor = capabilities::descriptor(e.operation, *property)
                .ok_or_else(|| unsupported("operation/property unsupported"))?;
            let p = e
                .properties
                .iter_mut()
                .find(|p| p.property == *property)
                .ok_or_else(|| {
                    RelicPlanError::new(
                        FaultCode::MissingProperty,
                        "edit property absent from effective structure",
                    )
                })?;
            p.program
                .include(*operation, descriptor)
                .map_err(|err| err.at(&b.contribution.definition_id, &b.rule.id, "apply.Edit"))?;
            admit_owned_copy(&b.contribution, &mut work)?;
            p.contributions.push(b.contribution.clone());
        }
    }
    normalize_legacy(&borrowed, &providers, &mut effects, data, &mut work)?;
    for e in effects.iter_mut().filter(|e| e.present) {
        if let Some(override_amount) = &e.amount {
            let original = original_expression(e, data)
                .ok_or_else(|| unsupported("amount recipe requires original amount"))?;
            if override_amount.moment != original.moment {
                return Err(RelicPlanError::new(
                    FaultCode::InvalidPreserveContract,
                    "single recipe preserves original evaluation moment",
                ));
            }
            let site = match &e.key {
                EffectiveEffectKey::Original { site } => site.clone(),
                EffectiveEffectKey::Generated {
                    anchor,
                    local_action_id,
                    local_effect_id,
                    ..
                } => {
                    validation::effect_site(&anchor.definition_id, local_action_id, local_effect_id)
                }
            };
            let value = validation::amount(
                override_amount,
                site,
                true,
                data,
                e.amount_capabilities,
                &mut work,
            )?;
            let p = edit_effect(e, &mut work)?
                .properties
                .iter_mut()
                .find(|p| p.property == RelicProperty::PersonalShieldInitialAmount)
                .unwrap();
            p.base = match value {
                Some(v) => PropertyBase::Literal(TypedRelicValue::Hp(
                    i64::try_from(v).map_err(|_| overflow())?,
                )),
                None => PropertyBase::RuntimeExpression,
            };
        }
        for index in 0..e.properties.len() {
            let p = &e.properties[index];
            let d = capabilities::descriptor(e.operation, p.property)
                .ok_or_else(|| unsupported("property descriptor missing"))?;
            match p.base {
                PropertyBase::Literal(v) => {
                    let value = Some(p.program.apply(v, d)?);
                    if p.final_value != value {
                        edit_effect(e, &mut work)?.properties[index].final_value = value;
                    }
                }
                PropertyBase::RuntimeExpression => p.program.validate_dynamic(d)?,
                PropertyBase::Structural => {}
            }
        }
        if let Some(program) = personal_creation(e, data) {
            if let Some(value) = e
                .properties
                .iter()
                .find(|p| p.property == RelicProperty::PersonalShieldInitialAmount)
                .and_then(|p| p.final_value)
            {
                let mut creation = NumericProgram::default();
                let descriptor = capabilities::descriptor(
                    e.operation,
                    RelicProperty::PersonalShieldInitialAmount,
                )
                .unwrap();
                for operation in &program.operations {
                    creation.include(*operation, descriptor)?;
                }
                creation.apply(value, descriptor)?;
            }
        }
    }
    let mut rules = conversion_rules;
    for (copy, d) in borrowed {
        for rule in validation::rules_for(d, copy.variant) {
            let mut targets = Vec::new();
            let mut matched = false;
            for b in bindings
                .iter()
                .filter(|b| b.contribution.copy_uuid == copy.copy_uuid && b.rule.id == rule.id)
            {
                matched = true;
                if b.active {
                    if let Some(n) = b.target {
                        targets.push(EffectiveEffectAddressDto {
                            provider_index: effects[n].provider,
                            effect: effects[n].key.clone(),
                        });
                    }
                    for &n in &b.granted {
                        targets.push(EffectiveEffectAddressDto {
                            provider_index: effects[n].provider,
                            effect: effects[n].key.clone(),
                        });
                    }
                }
            }
            work.charge(
                targets
                    .len()
                    .checked_mul(targets.len())
                    .ok_or_else(overflow)?,
                "relic_structural_target_order",
            )?;
            targets
                .sort_by(|a, b| (a.provider_index, &a.effect).cmp(&(b.provider_index, &b.effect)));
            targets.dedup();
            rules.push(RelicRuleStateDto {
                contribution: PlanContribution {
                    copy_uuid: copy.copy_uuid,
                    definition_id: d.id.clone(),
                    variant: copy.variant,
                    rule_id: rule.id.clone(),
                },
                state: if matches!(
                    rule.apply,
                    RelicRuleActionDef::Subscribe(_) | RelicRuleActionDef::Periodic { .. }
                ) {
                    super::causal::rule_state(rule, &effects, &providers, &mut work)?
                } else if targets.is_empty() {
                    RelicRuleState::Dormant {
                        reason: if matched {
                            RelicDormantReason::ConditionUnmet
                        } else if providers.is_empty() {
                            RelicDormantReason::ProviderUnavailable
                        } else {
                            RelicDormantReason::NoMatchingEffect
                        },
                    }
                } else {
                    RelicRuleState::Active
                },
                targets,
            });
        }
    }
    let mut grants = Vec::new();
    for binding in bindings.iter().filter(|binding| binding.active) {
        if let RelicRuleActionDef::Grant(body) = &binding.rule.apply {
            admit_owned_copy(body, &mut work)?;
            admit_owned_copy(&binding.contribution, &mut work)?;
            admit_owned_copy(&binding.anchor, &mut work)?;
            work.reserve(
                Some(size_of::<super::EffectiveGrant>()),
                "relic_structural_grant_owner",
            )?;
            grants.push(super::EffectiveGrant {
                provider: binding.provider,
                contribution: binding.contribution.clone(),
                anchor: binding.anchor.clone(),
                body: Arc::new(body.clone()),
            });
        }
    }
    let binding_count = bindings.len();
    let mut causal_rules = Vec::new();
    let mut wedge_rules = Vec::new();
    for (copy, definition) in borrowed {
        for rule in validation::rules_for(definition, copy.variant) {
            let contribution = PlanContribution {
                copy_uuid: copy.copy_uuid,
                definition_id: definition.id.clone(),
                variant: copy.variant,
                rule_id: rule.id.clone(),
            };
            match &rule.apply {
                RelicRuleActionDef::Subscribe(_) | RelicRuleActionDef::Periodic { .. } => {
                    causal_rules.push(
                        super::causal::compile_rule(rule, contribution, data, &mut work)
                            .map_err(|e| e.at(&definition.id, &rule.id, "causal"))?,
                    );
                }
                RelicRuleActionDef::Wedge { increment_percent } => {
                    let sites = bindings
                        .iter()
                        .filter(|b| {
                            b.active
                                && b.contribution.copy_uuid == copy.copy_uuid
                                && b.rule.id == rule.id
                        })
                        .filter_map(|b| b.target)
                        .filter_map(|index| match &effects[index].key {
                            EffectiveEffectKey::Original { site }
                                if effects[index].operation == EffectOperation::Damage =>
                            {
                                Some(site.clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    admit_owned_copy(&sites, &mut work)?;
                    work.reserve(
                        Some(size_of::<super::causal::EffectiveWedgeRule>()),
                        "wedge_rule_owner",
                    )?;
                    wedge_rules.push(super::causal::EffectiveWedgeRule {
                        contribution,
                        selector: rule.selector.clone(),
                        condition: rule.when.clone(),
                        increment_percent: *increment_percent,
                        cap_percent: None,
                        effect_sites: sites,
                    });
                }
                _ => {}
            }
        }
    }
    let mut plan = EffectivePlanComponents {
        employee_uuid: input.employee_uuid,
        providers,
        copies: borrowed.iter().map(|(c, _)| (*c).clone()).collect(),
        effects,
        grants,
        execution: Vec::new(),
        causal_rules,
        wedge_rules,
        action_modifiers: Vec::new(),
        rules,
        fingerprint: String::new(),
        usage: CompileUsage {
            work: work.work,
            reserved_bytes: work.reserved,
            candidate_count,
            binding_count,
        },
        retained_bytes: 0,
    };
    super::action::resolve(&mut plan,data,&mut work)?;
    if publication {
        // The proof never exposes a fingerprint. Reserve the runtime digest's
        // full owned capacity without filling a discarded placeholder string.
        // Completed storage measures capacity, including its allocation owner.
        work.reserve(Some(64 + 64), "relic_publication_fingerprint")?;
        plan.fingerprint = String::with_capacity(64);
    } else {
        #[derive(Serialize)]
        struct Fingerprint<'a> {
            generation: &'a crate::game::data::content_pack::ContentIdentity,
            employee_uuid: uuid::Uuid,
            providers: &'a [ProspectiveProvider],
            copies: &'a [super::super::OwnedRelic],
            effects: Vec<&'a EffectiveEffect>,
            rules: &'a [RelicRuleStateDto],
            grants: &'a [super::EffectiveGrant],
            causal_rules: &'a [super::causal::EffectiveCausalRule],
            wedge_rules: &'a [super::causal::EffectiveWedgeRule],
            action_modifiers: &'a [super::action::EffectiveActionModifier],
        }
        work.reserve(
            plan.effects
                .len()
                .checked_mul(size_of::<&EffectiveEffect>()),
            "relic_structural_fingerprint_order",
        )?;
        work.charge(
            plan.effects
                .len()
                .checked_mul(plan.effects.len())
                .ok_or_else(overflow)?,
            "relic_structural_fingerprint_order",
        )?;
        let mut selected: Vec<_> = plan.effects.iter().filter(|e| e.present).map(AsRef::as_ref).collect();
        selected.sort_by(|a, b| (a.provider, &a.key).cmp(&(b.provider, &b.key)));
        let fingerprint = Fingerprint {
            generation: input.generation,
            employee_uuid: plan.employee_uuid,
            providers: &plan.providers,
            copies: &plan.copies,
            effects: selected,
            rules: &plan.rules,
            grants: &plan.grants,
            causal_rules: &plan.causal_rules,
            wedge_rules: &plan.wedge_rules,
            action_modifiers: &plan.action_modifiers,
        };
        let bytes = serialized_size(&fingerprint, &mut work)?;
        work.reserve(Some(bytes), "relic_structural_fingerprint")?;
        work.charge(bytes, "relic_structural_fingerprint_copy")?;
        let encoded = serde_json::to_vec(&fingerprint)
            .map_err(|e| RelicPlanError::new(FaultCode::InvalidReference, e.to_string()))?;
        work.charge(bytes.div_ceil(32), "relic_structural_fingerprint_hash")?;
        plan.fingerprint = format!("{:x}", Sha256::digest(&encoded));
    }
    plan.execution = super::execution::lower(
        &plan,
        data,
        &mut work,
        prepared.map(|p| (p.effects.as_slice(), p.execution.as_slice())),
    )?;
    plan.retained_bytes = if let Some(prepared) = prepared {
        prepared.storage.measure(&plan, input.generation, &mut work, &prepared.providers, &prepared.effects, &prepared.execution)?
    } else {
        super::storage::measure(&plan, input.generation, &mut work)?
    };
    // Allocation admission counts Arc cloning as a handle operation. Validate
    // the complete conservative owner extent as well, without inventing a
    // second allocation; actual runtime ownership is admitted at publication.
    let completed_extent = retained_basis
        .checked_add(plan.retained_bytes)
        .ok_or_else(overflow)?;
    crate::game::battle::runtime_control::admit_retained_growth(
        completed_extent,
        Some(completed_extent),
        "relic_structural_completed_extent",
    )
    .map_err(RelicPlanError::control)?;
    plan.usage.work = work.work;
    plan.usage.reserved_bytes = work.reserved;
    crate::authoring::control::poll("relic_structural_plan_complete")
        .map_err(RelicPlanError::control)?;
    Ok(Some(plan))
}
fn add_granted(
    nodes: &mut Vec<super::SharedEffect>,
    indices: &mut Vec<usize>,
    e: &GrantedEffectDef,
    provider: usize,
    anchor: &ActionSite,
    local_action: &str,
    delivery: &DeliveryDef,
    target: &SkillTarget,
    cast_selection: SkillCastSelection,
    amount_capabilities: DeliveryCapabilities,
    c: &PlanContribution,
    data: &GameDataBase,
    work: &mut Work,
    publication: bool,
) -> Result<(), RelicPlanError> {
    admit_owned_copy(e, work)?;
    admit_owned_copy(delivery, work)?;
    let op = primitive(&e.op);
    validate_grant_carrier(&op, target, cast_selection, delivery)?;
    let (operation, mut properties) = properties(&op, delivery);
    let site = validation::effect_site(&anchor.definition_id, local_action, &e.id);
    if let Some(expr) = source_expr(&op) {
        let value = validation::amount(
            expr,
            site,
            operation == EffectOperation::PersonalShieldGrant,
            data,
            amount_capabilities,
            work,
        )?;
        resolve_static_amount(&op, &mut properties, value)?;
    }
    let key = EffectiveEffectKey::Generated {
        relic_copy_uuid: c.copy_uuid,
        grant_rule_id: c.rule_id.clone(),
        anchor: anchor.clone(),
        local_action_id: local_action.into(),
        local_effect_id: e.id.clone(),
    };
    work.charge(nodes.len(), "relic_structural_identity_check")?;
    if nodes.iter().any(|n| n.provider == provider && n.key == key) {
        return Err(RelicPlanError::new(
            FaultCode::UnboundedGrantIdentity,
            "duplicate finite grant site",
        ));
    }
    work.reserve(
        Some(size_of::<EffectiveEffect>() + 64),
        "relic_structural_candidate",
    )?;
    if publication {
        publication_effect_growth(nodes, work)?;
    }
    indices.push(nodes.len());
    nodes.push(Arc::new(EffectiveEffect {
        key,
        provider,
        anchor: anchor.clone(),
        tags: e.tags.clone(),
        operation,
        delivery: Arc::new(delivery.clone()),
        properties,
        recipes: Vec::new(),
        recipe_contributions: Vec::new(),
        body: EffectBody::Granted(Arc::new(e.clone())),
        amount_capabilities,
        amount: None,
        original: false,
        present: false,
    }));
    Ok(())
}
impl EffectiveEffect {
    pub(crate) fn has_property(&self, p: RelicProperty) -> bool {
        self.properties.iter().any(|v| v.property == p)
    }
    pub(crate) fn include_recipe(
        &mut self,
        r: &StructuralRecipeDef,
        c: &PlanContribution,
        work: &mut Work,
    ) -> Result<(), RelicPlanError> {
        admit_owned_copy(r, work)?;
        let kind = |r: &StructuralRecipeDef| match r {
            StructuralRecipeDef::ChannelHealthUpkeep { .. } => 5,
            StructuralRecipeDef::SupportSelectionPattern { .. } => 4,
            StructuralRecipeDef::DamageFootprint { .. } => 0,
            StructuralRecipeDef::FixedRecipientSupport => 1,
            StructuralRecipeDef::RegularBasicSupport { .. } => 2,
            StructuralRecipeDef::PersonalShieldSingle { .. } => 3,
        };
        if let Some(old) = self.recipes.iter().find(|old| kind(old) == kind(r)) {
            if old != r {
                return Err(RelicPlanError::new(
                    FaultCode::StructuralWriteConflict,
                    "different writes by same recipe family",
                ));
            }
        } else {
            match r {
                StructuralRecipeDef::ChannelHealthUpkeep { policy } => {
                    if self.operation != EffectOperation::ChannelAura {
                        return Err(unsupported("HP upkeep requires ChannelAura"));
                    }
                    policy.validate().map_err(|e| unsupported(&e))?;
                }
                StructuralRecipeDef::SupportSelectionPattern { .. } => {
                    if !matches!(self.operation, EffectOperation::PersonalShieldGrant | EffectOperation::ChannelAura) {
                        return Err(unsupported("support selection pattern requires a support effect"));
                    }
                }
                StructuralRecipeDef::DamageFootprint { .. } => {
                    if self.operation != EffectOperation::Damage
                        || matches!(&*self.delivery, DeliveryDef::Projectile { .. })
                    {
                        return Err(unsupported(
                            "effect-local footprint requires Instant/TileArea damage",
                        ));
                    }
                    if !self.has_property(RelicProperty::DamageFootprint) {
                        self.properties.push(EffectiveProperty {
                            property: RelicProperty::DamageFootprint,
                            base: PropertyBase::Structural,
                            program: NumericProgram::default(),
                            final_value: None,
                            contributions: Vec::new(),
                        });
                    }
                }
                StructuralRecipeDef::FixedRecipientSupport => {
                    if self.operation != EffectOperation::ChannelAura {
                        return Err(unsupported("fixed channel recipe requires ChannelAura"));
                    }
                    self.properties
                        .retain(|p| p.property != RelicProperty::SupportAreaRadius);
                }
                StructuralRecipeDef::PersonalShieldSingle { initial_amount } => {
                    if self.operation != EffectOperation::PersonalShieldGrant {
                        return Err(unsupported(
                            "personal-single recipe requires PersonalShieldGrant",
                        ));
                    }
                    self.properties
                        .retain(|p| p.property != RelicProperty::SupportAreaRadius);
                    self.amount = Some(Arc::clone(initial_amount));
                }
                StructuralRecipeDef::RegularBasicSupport {
                    damage_percent,
                    speed_points,
                } => {
                    if self.operation != EffectOperation::ChannelAura {
                        return Err(unsupported("regular coefficients require ChannelAura"));
                    }
                    self.properties.retain(|p| {
                        !matches!(
                            p.property,
                            RelicProperty::RegularBasicDamagePercent
                                | RelicProperty::RegularBasicSpeedPoints
                        )
                    });
                    self.properties.push(literal(
                        RelicProperty::RegularBasicDamagePercent,
                        TypedRelicValue::PercentPoints(i64::from(*damage_percent)),
                    ));
                    self.properties.push(literal(
                        RelicProperty::RegularBasicSpeedPoints,
                        TypedRelicValue::SpeedPoints(i64::from(*speed_points)),
                    ));
                }
            }
            self.recipes.push(r.clone());
        }
        admit_owned_copy(c, work)?;
        self.recipe_contributions.push(c.clone());
        Ok(())
    }
}
fn normalize_legacy(
    borrowed: &[(&super::super::OwnedRelic, &RelicDefinition)],
    providers: &[ProspectiveProvider],
    nodes: &mut [super::SharedEffect],
    data: &GameDataBase,
    work: &mut Work,
) -> Result<(), RelicPlanError> {
    for (copy, d) in borrowed {
        let effects = match copy.variant {
            super::super::RelicVariant::Normal => &d.effects,
            super::super::RelicVariant::Corrupt => {
                d.corruption.as_ref().map_or(&d.effects, |v| &v.effects)
            }
        };
        for legacy in effects {
            if legacy.selector.target == RelicDamageTarget::BasicAttackDamage {
                continue;
            }
            for n in nodes
                .iter_mut()
                .filter(|n| n.present && n.operation == EffectOperation::Damage)
            {
                if !super::closure::legacy_matches(&legacy.selector, n, providers, work)? {
                    continue;
                }
                let damage_type = match &n.body {
                    EffectBody::Original {
                        skill_id,
                        step,
                        effect,
                    } => match &data.skill_data.get_by_id(skill_id).unwrap().steps[*step].effects
                        [*effect]
                        .op
                    {
                        SkillEffectDef::Damage { damage_type, .. }
                        | SkillEffectDef::DamageExpr { damage_type, .. } => Some(*damage_type),
                        _ => None,
                    },
                    EffectBody::Granted(e) => match &e.op {
                        GrantPrimitiveDef::Damage { damage_type, .. }
                        | GrantPrimitiveDef::DamageExpr { damage_type, .. } => Some(*damage_type),
                        _ => None,
                    },
                };
                if legacy
                    .selector
                    .damage_type
                    .is_some_and(|d| damage_type != Some(d))
                {
                    continue;
                }
                let p = edit_effect(n, work)?
                    .properties
                    .iter_mut()
                    .find(|p| p.property == RelicProperty::DamageAmount)
                    .unwrap();
                let desc =
                    capabilities::descriptor(EffectOperation::Damage, RelicProperty::DamageAmount)
                        .unwrap();
                p.program
                    .include(NumericEditDef::Add(TypedRelicValue::Hp(legacy.flat)), desc)?;
                p.program.include(
                    NumericEditDef::Scale {
                        increment_percent: legacy.percent,
                    },
                    desc,
                )?;
                p.contributions.push(PlanContribution {
                    copy_uuid: copy.copy_uuid,
                    definition_id: d.id.clone(),
                    variant: copy.variant,
                    rule_id: legacy.id.clone(),
                });
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_effective_configuration(
    input: EffectivePlanInput<'_>,
) -> Result<(), RelicPlanError> {
    compile_effective_plan(input).map(|_| ())
}
/// Publish only after typed references and every legal bounded co-equip plan
/// have been validated against the real immutable lowering/consumer catalog.
pub(crate) fn validate_publication(
    data: &GameDataBase,
) -> Result<(), crate::game::data::live_ron::LiveRonError> {
    use crate::game::data::live_ron::{LiveRonError, LiveRonPhase};
    super::action::validate_references(data).map_err(|error| LiveRonError::validation("data/skills/base.ron", "skills", None::<String>, Some("action_policy"), error.to_string()))?;
    super::support::validate_references(data).map_err(|error| LiveRonError::validation(
        "data/skills/base.ron", "skills", None::<String>, Some("support.conversion".to_string()), error.to_string()
    ).with_phase(LiveRonPhase::Compile).with_code(error.code.producer_code()))?;
    for (i, definition) in data.relic_data.relics.iter().enumerate() {
        for (corrupt, rules) in [
            (false, &definition.rules),
            (
                true,
                definition
                    .corruption
                    .as_ref()
                    .map_or(&definition.rules, |c| &c.rules),
            ),
        ] {
            if rules.is_empty() {
                continue;
            }
            if let Err(error) = validation::validate_rules(&definition.id, rules)
                .and_then(|()| validation::provider_references(rules, data))
            {
                return Err(LiveRonError::validation(
                    "data/relics/base.ron",
                    "relics",
                    Some(definition.id.clone()),
                    Some(format!(
                        "relics[{i}].{}rules",
                        if corrupt { "corruption." } else { "" }
                    )),
                    error.to_string(),
                )
                .with_phase(LiveRonPhase::Compile)
                .with_code(error.code.producer_code()));
            }
        }
    }
    super::publication::validate(data).map_err(|error| {
        LiveRonError::validation(
            "data/relics/base.ron",
            "relics",
            error.definition_id.clone(),
            error
                .field_path
                .clone()
                .or_else(|| Some("legal_coequip_plans".into())),
            error.to_string(),
        )
        .with_phase(LiveRonPhase::Compile)
        .with_code(error.code.producer_code())
    })
}
