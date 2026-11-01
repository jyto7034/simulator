//! Read-only loadout projection. These addresses explain possible damage; they
//! do not contain an actor, action ordinal, epoch or execution capability.
use serde::Serialize;
use uuid::Uuid;

use super::{numeric::RelicNumericFaultCode, RelicVariant};
use crate::game::{
    ability::authoring::EffectSite,
    data::{content_pack::ContentIdentity, relic_data::RelicPenalty},
    skill_fragment::FragmentSourceSnapshot,
    stats::UnitStats,
    value_expr::ValueExprFault,
};

#[derive(Debug, Clone, Serialize)]
pub struct RelicNumericPlanDto {
    pub employee_uuid: Uuid,
    pub generation: ContentIdentity,
    pub scope: RelicNumericPlanScope,
    /// Final battle projection before relic penalties, not certified power.
    pub base_battle_stats: Option<UnitStats>,
    pub effective_battle_stats: Option<UnitStats>,
    pub stats_unavailable_reason: Option<RelicNumericPlanUnresolvedReason>,
    pub penalties: Vec<RelicNumericPlanPenaltyDto>,
    pub targets: Vec<RelicNumericPlanTargetDto>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicNumericPlanScope {
    ProspectiveLoadout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RelicProspectiveEquipmentDto {
    pub definition_uuid: Uuid,
    pub persistent_copy_uuid: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelicProspectiveProviderDto {
    pub equipment: Option<RelicProspectiveEquipmentDto>,
    pub fragment: Option<FragmentSourceSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelicNumericPlanTargetDto {
    SkillDamage {
        site: EffectSite,
        provider: RelicProspectiveProviderDto,
        evaluation: RelicNumericPlanEvaluationDto,
        effects: Vec<RelicNumericPlanEffectDto>,
    },
    BasicAttackDamage {
        weapon: Option<RelicProspectiveEquipmentDto>,
        evaluation: RelicNumericPlanEvaluationDto,
        effects: Vec<RelicNumericPlanEffectDto>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelicNumericPlanEvaluationDto {
    Available {
        base_amount: u32,
        effective_amount: u32,
    },
    Unresolved {
        reason: RelicNumericPlanUnresolvedReason,
    },
    Rejected {
        fault: RelicNumericPlanFaultDto,
    },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicNumericPlanUnresolvedReason {
    RuntimeContextRequired,
    ProviderUnavailable,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum RelicNumericPlanFaultDto {
    Base { fault: ValueExprFault },
    Final { code: RelicNumericFaultCode },
}

#[derive(Debug, Clone, Serialize)]
pub struct RelicNumericPlanEffectDto {
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub variant: RelicVariant,
    pub effect_id: String,
    pub flat: i64,
    pub percent: i64,
    pub state: RelicNumericPlanEffectState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RelicNumericPlanEffectState {
    Active,
    Dormant {
        reason: RelicNumericPlanDormantReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelicNumericPlanDormantReason {
    TargetKind,
    DamageType,
    Tags,
    Equipment,
    Fragment,
    Form,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelicNumericPlanPenaltyDto {
    pub copy_uuid: Uuid,
    pub definition_id: String,
    pub penalty: RelicPenalty,
}

use crate::game::{
    ability::{authoring::ActionSite, SkillEffectDef, SkillId},
    battle::{
        core::{
            relic_numeric::{selector_state, RelicSelectorContext},
            value_expr_runtime::{evaluate_compiled_amount, RuntimeAmount},
        },
        runtime_control as native,
        types::UnitCombatProfile,
    },
    behavior::GameError,
    data::{equipment_data::EquipmentType, relic_data::RelicDamageTarget, GameDataBase},
    resources::item_slot::ItemSlot,
    value_expr::{can_evaluate_without_external_context, AmountPlanKind, EvalFrame},
};
use std::mem::size_of;

fn invalid(cause: &str) -> GameError {
    GameError::InvalidStaticData(cause.into())
}
fn checked(bytes: Option<usize>) -> Result<usize, GameError> {
    bytes.ok_or_else(|| {
        native::reject(native::BattleRuntimeFault {
            code: native::RuntimeFaultCode::CounterOverflow,
            site: "relic_numeric_plan",
            limit: None,
            requested: None,
        })
    })
}
fn clone_value<T: Clone + crate::authoring::clone_shape::CloneShape>(
    value: &T,
) -> Result<T, GameError> {
    if !crate::authoring::control::is_active() {
        crate::game::resource::store::preflight_resource_clone(value)
            .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
    }
    crate::authoring::clone_shape::controlled_clone(value)
}

/// The authored loadout is already validated by the battle draft projection.
/// Providers below mirror manual Unit sources and item ability activations;
/// equipping a weapon never assigns it to the fragment's manual skill source.
pub(crate) fn build_plan(
    employee: Uuid,
    inventory: &super::RelicInventory,
    item_slot: &ItemSlot,
    data: &GameDataBase,
    profile: &UnitCombatProfile,
    base_stats: UnitStats,
    generation: &ContentIdentity,
) -> Result<RelicNumericPlanDto, GameError> {
    let mut rows = usize::from(profile.basic_attack.action.is_damage());
    if let Some(skill) = &profile.skill_id {
        rows = checked(rows.checked_add(damage_rows(data, skill)?))?;
    }
    for item in item_slot.iter() {
        let definition = equipment(data, item.base_uuid)?;
        crate::authoring::control::charge(
            definition.ability_activations.len() as u64,
            "relic_plan_providers",
        )?;
        for activation in &definition.ability_activations {
            rows = checked(rows.checked_add(damage_rows(data, &activation.ability_id)?))?;
        }
    }
    native::admit_retained_growth(
        0,
        Some(checked(
            rows.checked_mul(size_of::<RelicNumericPlanTargetDto>()),
        )?),
        "relic_plan_targets",
    )?;
    let mut targets = Vec::with_capacity(rows);
    if let Some(skill) = &profile.skill_id {
        add_skill_targets(
            &mut targets,
            inventory,
            data,
            employee,
            skill,
            None,
            profile
                .fragment_source
                .as_ref()
                .filter(|source| &source.skill_id == skill),
        )?;
    }
    let mut weapon = None;
    for item in item_slot.iter() {
        let definition = equipment(data, item.base_uuid)?;
        let provider = RelicProspectiveEquipmentDto {
            definition_uuid: item.base_uuid,
            persistent_copy_uuid: item.instance_uuid,
        };
        if definition.equipment_type == EquipmentType::Weapon {
            weapon = Some(provider);
        }
        for activation in &definition.ability_activations {
            crate::authoring::control::charge(1, "relic_plan_provider")?;
            add_skill_targets(
                &mut targets,
                inventory,
                data,
                employee,
                &activation.ability_id,
                Some(provider),
                None,
            )?;
        }
    }
    if profile.basic_attack.action.is_damage() {
        let effects = effect_rows(
            inventory,
            data,
            employee,
            &RelicSelectorContext {
                target: RelicDamageTarget::BasicAttackDamage,
                tags: &[],
                damage_type: profile.basic_attack.damage_type,
                equipment: weapon.map(|item| item.definition_uuid),
                fragment: None,
            },
        )?;
        // This is the base loadout amount, before runtime aura/target/critical
        // state. The prospective scope never claims a live target result.
        let evaluation = compose(base_stats.attack, &effects)?;
        targets.push(RelicNumericPlanTargetDto::BasicAttackDamage {
            weapon,
            evaluation,
            effects,
        });
    }
    let penalties = penalty_rows(inventory, data, employee)?;
    let effective_stats = crate::game::battle::core::relic_numeric::penalize_employee_stats(
        inventory, data, employee, base_stats,
    )?;
    native::admit_retained_growth(
        0,
        Some(generation.semantic_sha256.len()),
        "relic_plan_generation",
    )?;
    let generation = clone_value(generation)?;
    Ok(RelicNumericPlanDto {
        employee_uuid: employee,
        generation,
        scope: RelicNumericPlanScope::ProspectiveLoadout,
        base_battle_stats: Some(base_stats),
        effective_battle_stats: Some(effective_stats),
        stats_unavailable_reason: None,
        penalties,
        targets,
    })
}

fn equipment(
    data: &GameDataBase,
    id: Uuid,
) -> Result<&crate::game::data::equipment_data::EquipmentMetadata, GameError> {
    crate::authoring::control::charge(
        data.equipment_data.items.len() as u64,
        "relic_plan_equipment",
    )?;
    data.equipment_data
        .get_by_uuid(&id)
        .ok_or(GameError::InventoryItemNotFound)
}
fn skill<'a>(
    data: &'a GameDataBase,
    id: &SkillId,
) -> Result<&'a crate::game::ability::SkillDef, GameError> {
    crate::authoring::control::charge(data.skill_data.skills.len() as u64, "relic_plan_skill")?;
    data.skill_data
        .get_by_id(id)
        .ok_or_else(|| invalid("relic plan skill unavailable"))
}
fn damage_rows(data: &GameDataBase, id: &SkillId) -> Result<usize, GameError> {
    let mut count = 0usize;
    for step in &skill(data, id)?.steps {
        crate::authoring::control::charge(step.effects.len() as u64, "relic_plan_effect_scan")?;
        for effect in &step.effects {
            if matches!(
                effect.op,
                SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. }
            ) {
                count = checked(count.checked_add(1))?;
            }
        }
    }
    Ok(count)
}
fn add_skill_targets(
    rows: &mut Vec<RelicNumericPlanTargetDto>,
    inventory: &super::RelicInventory,
    data: &GameDataBase,
    employee: Uuid,
    id: &SkillId,
    equipment: Option<RelicProspectiveEquipmentDto>,
    fragment: Option<&FragmentSourceSnapshot>,
) -> Result<(), GameError> {
    for step in &skill(data, id)?.steps {
        crate::authoring::control::charge(step.effects.len() as u64, "relic_plan_effect_scan")?;
        for effect in &step.effects {
            let damage_type = match effect.op {
                SkillEffectDef::Damage { damage_type, .. }
                | SkillEffectDef::DamageExpr { damage_type, .. } => damage_type,
                _ => continue,
            };
            native::admit_retained_growth(
                0,
                Some(checked(
                    id.as_str()
                        .len()
                        .checked_add(step.id.len())
                        .and_then(|n| n.checked_add(effect.id.len())),
                )?),
                "relic_plan_site",
            )?;
            let site = EffectSite {
                action: ActionSite {
                    definition_id: clone_value(id)?,
                    action_id: clone_value(&step.id)?,
                },
                effect_id: clone_value(&effect.id)?,
            };
            let effects = effect_rows(
                inventory,
                data,
                employee,
                &RelicSelectorContext {
                    target: RelicDamageTarget::SkillDamage,
                    tags: &effect.tags,
                    damage_type,
                    equipment: equipment.map(|item| item.definition_uuid),
                    fragment,
                },
            )?;
            let entry = data
                .value_expr_catalog()
                .get(&site)
                .ok_or_else(|| invalid("relic plan base catalog address unavailable"))?;
            if entry.kind() != AmountPlanKind::Damage {
                return Err(invalid("relic plan base catalog kind mismatch"));
            }
            let evaluation = if !can_evaluate_without_external_context(entry.plan()) {
                RelicNumericPlanEvaluationDto::Unresolved {
                    reason: RelicNumericPlanUnresolvedReason::RuntimeContextRequired,
                }
            } else {
                match evaluate_compiled_amount(entry.plan(), entry.kind(), &EvalFrame::default()) {
                    Ok(value) => match value.amount {
                        RuntimeAmount::Damage(base) => compose(base, &effects)?,
                        _ => return Err(invalid("relic plan base amount kind mismatch")),
                    },
                    Err(fault) => RelicNumericPlanEvaluationDto::Rejected {
                        fault: RelicNumericPlanFaultDto::Base { fault },
                    },
                }
            };
            let fragment = if let Some(fragment) = fragment {
                let bytes = checked(
                    fragment
                        .definition_id
                        .as_str()
                        .len()
                        .checked_add(fragment.form_id.len())
                        .and_then(|n| n.checked_add(fragment.skill_id.as_str().len())),
                )?;
                native::admit_retained_growth(0, Some(bytes), "relic_plan_fragment")?;
                Some(clone_value(fragment)?)
            } else {
                None
            };
            rows.push(RelicNumericPlanTargetDto::SkillDamage {
                site,
                provider: RelicProspectiveProviderDto {
                    equipment,
                    fragment,
                },
                evaluation,
                effects,
            });
        }
    }
    Ok(())
}

fn compose(
    base: u32,
    effects: &[RelicNumericPlanEffectDto],
) -> Result<RelicNumericPlanEvaluationDto, GameError> {
    crate::authoring::control::charge(effects.len() as u64, "relic_plan_compose")?;
    let totals = effects
        .iter()
        .filter(|row| row.state == RelicNumericPlanEffectState::Active)
        .try_fold((0i128, 0i128), |(flat, percent), row| {
            if row.flat < 0 || row.percent < 0 {
                return Err(RelicNumericFaultCode::InvalidCoefficient);
            }
            Ok((
                flat.checked_add(i128::from(row.flat))
                    .ok_or(RelicNumericFaultCode::Overflow)?,
                percent
                    .checked_add(i128::from(row.percent))
                    .ok_or(RelicNumericFaultCode::Overflow)?,
            ))
        });
    Ok(
        match totals.and_then(|(flat, percent)| super::numeric::compose_damage(base, flat, percent))
        {
            Ok(effective) => RelicNumericPlanEvaluationDto::Available {
                base_amount: base,
                effective_amount: effective,
            },
            Err(code) => RelicNumericPlanEvaluationDto::Rejected {
                fault: RelicNumericPlanFaultDto::Final { code },
            },
        },
    )
}

fn effect_rows(
    inventory: &super::RelicInventory,
    data: &GameDataBase,
    employee: Uuid,
    context: &RelicSelectorContext<'_>,
) -> Result<Vec<RelicNumericPlanEffectDto>, GameError> {
    let mut count = 0usize;
    let mut bytes = 0usize;
    for id in inventory.employee_slots(employee).into_iter().flatten() {
        let copy = inventory
            .get(id)
            .ok_or_else(|| invalid("relic plan missing copy"))?;
        for effect in copy_effects(data, copy)? {
            crate::authoring::control::charge(1, "relic_plan_row_bound")?;
            count = checked(count.checked_add(1))?;
            bytes = checked(
                bytes
                    .checked_add(size_of::<RelicNumericPlanEffectDto>())
                    .and_then(|n| n.checked_add(copy.definition_id.len()))
                    .and_then(|n| n.checked_add(effect.id.len())),
            )?;
        }
    }
    native::admit_retained_growth(0, Some(bytes), "relic_plan_effect_rows")?;
    let mut rows = Vec::with_capacity(count);
    for id in inventory.employee_slots(employee).into_iter().flatten() {
        let copy = inventory
            .get(id)
            .ok_or_else(|| invalid("relic plan missing copy"))?;
        for effect in copy_effects(data, copy)? {
            crate::authoring::control::charge(1, "relic_plan_row")?;
            rows.push(RelicNumericPlanEffectDto {
                copy_uuid: id,
                definition_id: clone_value(&copy.definition_id)?,
                variant: copy.variant,
                effect_id: clone_value(&effect.id)?,
                flat: effect.flat,
                percent: effect.percent,
                state: selector_state(data, &effect.selector, context)?,
            });
        }
    }
    let work = checked(count.checked_mul((usize::BITS - count.leading_zeros()) as usize + 1))?;
    crate::authoring::control::charge(work as u64, "relic_plan_row_sort")?;
    rows.sort_unstable_by(|a, b| (a.copy_uuid, &a.effect_id).cmp(&(b.copy_uuid, &b.effect_id)));
    Ok(rows)
}
fn copy_effects<'a>(
    data: &'a GameDataBase,
    copy: &super::OwnedRelic,
) -> Result<&'a [crate::game::data::relic_data::RelicDamageEffect], GameError> {
    crate::authoring::control::charge(
        data.relic_data.relics.len() as u64,
        "relic_plan_definition",
    )?;
    let definition = data
        .relic_data
        .get_by_id(&copy.definition_id)
        .ok_or_else(|| invalid("relic plan definition unavailable"))?;
    Ok(match copy.variant {
        RelicVariant::Normal => &definition.effects,
        RelicVariant::Corrupt => {
            &definition
                .corruption
                .as_ref()
                .ok_or_else(|| invalid("relic plan variant unavailable"))?
                .effects
        }
    })
}
fn penalty_rows(
    inventory: &super::RelicInventory,
    data: &GameDataBase,
    employee: Uuid,
) -> Result<Vec<RelicNumericPlanPenaltyDto>, GameError> {
    let mut count = 0usize;
    let mut bytes = 0usize;
    for id in inventory.employee_slots(employee).into_iter().flatten() {
        let copy = inventory
            .get(id)
            .ok_or_else(|| invalid("relic plan penalty copy unavailable"))?;
        if copy.variant != RelicVariant::Corrupt {
            continue;
        }
        let definition = data
            .relic_data
            .get_by_id(&copy.definition_id)
            .ok_or_else(|| invalid("relic plan penalty definition unavailable"))?;
        let variant = definition
            .corruption
            .as_ref()
            .ok_or_else(|| invalid("relic plan penalty variant unavailable"))?;
        crate::authoring::control::charge(
            data.relic_data.relics.len() as u64 + variant.penalties.len() as u64,
            "relic_plan_penalty_bound",
        )?;
        count = checked(count.checked_add(variant.penalties.len()))?;
        bytes =
            checked(
                bytes.checked_add(checked(variant.penalties.len().checked_mul(
                    size_of::<RelicNumericPlanPenaltyDto>() + copy.definition_id.len(),
                ))?),
            )?;
    }
    native::admit_retained_growth(0, Some(bytes), "relic_plan_penalty_rows")?;
    let mut rows = Vec::with_capacity(count);
    for id in inventory.employee_slots(employee).into_iter().flatten() {
        let copy = inventory
            .get(id)
            .ok_or_else(|| invalid("relic plan penalty copy unavailable"))?;
        if copy.variant != RelicVariant::Corrupt {
            continue;
        }
        crate::authoring::control::charge(
            data.relic_data.relics.len() as u64,
            "relic_plan_penalty_definition",
        )?;
        let variant = data
            .relic_data
            .get_by_id(&copy.definition_id)
            .and_then(|definition| definition.corruption.as_ref())
            .ok_or_else(|| invalid("relic plan penalty variant unavailable"))?;
        for penalty in &variant.penalties {
            crate::authoring::control::charge(1, "relic_plan_penalty_row")?;
            rows.push(RelicNumericPlanPenaltyDto {
                copy_uuid: id,
                definition_id: clone_value(&copy.definition_id)?,
                penalty: *penalty,
            });
        }
    }
    crate::authoring::control::charge(
        checked(count.checked_mul((usize::BITS - count.leading_zeros()) as usize + 1))? as u64,
        "relic_plan_penalty_sort",
    )?;
    rows.sort_unstable_by_key(|row| row.copy_uuid);
    Ok(rows)
}

/// A missing required weapon is a prospective provider gap, not a battle
/// success or permission to suppress an unrelated validation/budget failure.
pub(crate) fn unavailable_plan(
    employee: Uuid,
    inventory: &super::RelicInventory,
    data: &GameDataBase,
    generation: &ContentIdentity,
) -> Result<RelicNumericPlanDto, GameError> {
    let penalties = penalty_rows(inventory, data, employee)?;
    native::admit_retained_growth(
        0,
        Some(generation.semantic_sha256.len()),
        "relic_plan_generation",
    )?;
    Ok(RelicNumericPlanDto {
        employee_uuid: employee,
        generation: clone_value(generation)?,
        scope: RelicNumericPlanScope::ProspectiveLoadout,
        base_battle_stats: None,
        effective_battle_stats: None,
        stats_unavailable_reason: Some(RelicNumericPlanUnresolvedReason::ProviderUnavailable),
        penalties,
        targets: Vec::new(),
    })
}
