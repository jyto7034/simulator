//! Exhaustive bounded publication over legal three-copy relic multisets and
//! legal equipment sets, with one compatible active fragment form at a time.
//! Symbolic copy IDs and pure profiles are proof inputs, never execution leases.
use super::ProspectiveProvider;
use super::{compiler, validation, FaultCode, RelicPlanError, UnpublishedPlanInput};
use crate::game::{
    battle::types::UnitCombatProfile,
    data::{
        content_pack::{ContentContract, ContentIdentity},
        equipment_data::{EquipmentMetadata, EquipmentType},
        relic_data::RelicDefinition,
        skill_fragment_data::SkillFragmentEffectDef,
        GameDataBase,
    },
    relic::{RelicInventory, RelicVariant},
    resources::item_slot::{EquippedRef, ItemSlot, SlotLayoutKind},
    skill_fragment::FragmentSourceSnapshot,
};
use std::mem::size_of;
use uuid::Uuid;

fn overflow() -> RelicPlanError {
    RelicPlanError::new(
        FaultCode::ArithmeticOverflow,
        "publication set bound overflow",
    )
}
fn charge(n: usize, site: &'static str) -> Result<(), RelicPlanError> {
    crate::authoring::control::charge(u64::try_from(n).map_err(|_| overflow())?, site)
        .map_err(RelicPlanError::control)
}
fn admit(bytes: Option<usize>, site: &'static str) -> Result<(), RelicPlanError> {
    crate::game::battle::runtime_control::admit_retained_growth(0, bytes, site)
        .map_err(RelicPlanError::control)
}
fn cube(n: usize) -> Result<usize, RelicPlanError> {
    n.checked_mul(n)
        .and_then(|v| v.checked_mul(n))
        .ok_or_else(overflow)
}

pub(super) fn validate(data: &GameDataBase) -> Result<(), RelicPlanError> {
    charge(
        data.relic_data.relics.len(),
        "publication_structural_presence",
    )?;
    let has_structural = data.relic_data.relics.iter().any(|definition| {
        !definition.rules.is_empty()
            || definition
                .corruption
                .as_ref()
                .is_some_and(|c| !c.rules.is_empty())
    });
    if !has_structural {
        return Ok(());
    }
    let variant_capacity = data
        .relic_data
        .relics
        .len()
        .checked_mul(2)
        .ok_or_else(overflow)?;
    admit(
        variant_capacity.checked_mul(size_of::<(&RelicDefinition, RelicVariant)>()),
        "publication_variants",
    )?;
    let mut variants = Vec::with_capacity(variant_capacity);
    charge(data.relic_data.relics.len(), "publication_variant_scan")?;
    for definition in &data.relic_data.relics {
        variants.push((definition, RelicVariant::Normal));
        if definition.corruption.is_some() {
            variants.push((definition, RelicVariant::Corrupt));
        }
    }
    // Empty choice plus all definitions/variants. Multisets are sufficient:
    // copy-local closure and all property composition are order-independent.
    // Every repeated definition still has distinct symbolic copy identities.
    let relic_bound = cube(variants.len().checked_add(1).ok_or_else(overflow)?)?;
    let equipment_bound = cube(
        data.equipment_data
            .items
            .len()
            .checked_add(1)
            .ok_or_else(overflow)?,
    )?;
    charge(
        data.skill_fragment_data.fragments.len(),
        "publication_form_bound",
    )?;
    let form_bound = data
        .skill_fragment_data
        .fragments
        .iter()
        .try_fold(1usize, |n, fragment| {
            n.checked_add(match &fragment.effect {
                SkillFragmentEffectDef::ActiveSkill { forms, .. } => forms.len(),
                _ => 0,
            })
        })
        .ok_or_else(overflow)?;
    charge(
        relic_bound
            .checked_mul(equipment_bound)
            .and_then(|n| n.checked_mul(form_bound))
            .ok_or_else(overflow)?,
        "publication_combination_bound",
    )?;
    // This identity is private, discarded with each proof plan and cannot be
    // published. Actual runtime compilation pins the real selected generation.
    admit(
        Some(size_of::<ContentIdentity>() + 64),
        "publication_identity",
    )?;
    let identity = ContentIdentity {
        contract: ContentContract::current(),
        semantic_sha256: "0".repeat(64),
    };
    let employee = Uuid::from_u128(0x435253415f50524f4f46);
    for a in 0..=variants.len() {
        for b in a..=variants.len() {
            for c in b..=variants.len() {
                let indices = [a, b, c];
                let mut selected: [Option<(&RelicDefinition, RelicVariant)>; 3] = [None; 3];
                let mut count = 0;
                for index in indices {
                    if index > 0 {
                        selected[count] = Some(variants[index - 1]);
                        count += 1;
                    }
                }
                if !selected
                    .iter()
                    .flatten()
                    .any(|(d, v)| !validation::rules_for(d, *v).is_empty())
                {
                    continue;
                }
                admit(
                    count.checked_mul(size_of::<(&RelicDefinition, RelicVariant)>()),
                    "publication_relic_selection",
                )?;
                let mut definitions = Vec::with_capacity(count);
                definitions.extend(selected.into_iter().flatten());
                let Some(inventory) =
                    RelicInventory::publication_input(employee, &definitions, &data.relic_data)
                        .map_err(RelicPlanError::control)?
                else {
                    continue;
                };
                equipment_sets(data, &identity, employee, &inventory)?;
            }
        }
    }
    Ok(())
}

fn equipment_sets(
    data: &GameDataBase,
    identity: &ContentIdentity,
    employee: Uuid,
    inventory: &RelicInventory,
) -> Result<(), RelicPlanError> {
    let items = &data.equipment_data.items;
    for a in 0..=items.len() {
        for b in a..=items.len() {
            for c in b..=items.len() {
                let indices = [a, b, c];
                let mut equipment = [None; 3];
                let mut count = 0;
                for index in indices {
                    if index > 0 {
                        equipment[count] = Some(&items[index - 1]);
                        count += 1;
                    }
                }
                // Any3 contains every set also representable in ByType. Use the actual
                // slot admission for duplicates/capacity; no definition union shortcut.
                admit(
                    Some(size_of::<ItemSlot>() + 4 * size_of::<EquippedRef>()),
                    "publication_equipment_slots",
                )?;
                let mut slots = ItemSlot::new(SlotLayoutKind::Any3);
                let mut valid = true;
                for (index, definition) in equipment.iter().flatten().enumerate() {
                    charge(3, "publication_equipment_admission")?;
                    if slots
                        .equip(
                            EquippedRef {
                                instance_uuid: Uuid::from_u128(index as u128 + 0x10),
                                base_uuid: definition.uuid,
                                equipment_type: definition.equipment_type,
                            },
                            definition.allow_duplicate_equip,
                        )
                        .is_err()
                    {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    continue;
                }
                let provider_count = equipment
                    .iter()
                    .flatten()
                    .try_fold(0usize, |n, d| n.checked_add(d.ability_activations.len()))
                    .ok_or_else(overflow)?;
                admit(
                    provider_count
                        .checked_add(1)
                        .and_then(|n| n.checked_mul(size_of::<ProspectiveProvider>())),
                    "publication_provider_table",
                )?;
                let mut providers = Vec::with_capacity(provider_count + 1);
                for (index, definition) in equipment.iter().flatten().enumerate() {
                    for activation in &definition.ability_activations {
                        let copy_uuid = Uuid::from_u128(index as u128 + 0x10);
                        charge(
                            providers
                                .len()
                                .checked_mul(
                                    definition
                                        .id
                                        .len()
                                        .checked_add(activation.ability_id.as_str().len())
                                        .and_then(|n| n.checked_add(1))
                                        .ok_or_else(overflow)?,
                                )
                                .ok_or_else(overflow)?,
                            "publication_provider_dedup",
                        )?;
                        if providers.iter().any(|provider| matches!(provider, ProspectiveProvider::Equipment { definition_id, copy_uuid: old_copy, skill_id }
                    if definition_id == &definition.id && *old_copy == copy_uuid && skill_id == &activation.ability_id)) { continue; }
                        charge(
                            definition
                                .id
                                .len()
                                .checked_add(activation.ability_id.as_str().len())
                                .ok_or_else(overflow)?,
                            "publication_equipment_provider",
                        )?;
                        admit(
                            definition
                                .id
                                .len()
                                .checked_add(activation.ability_id.as_str().len()),
                            "publication_equipment_provider_text",
                        )?;
                        providers.push(ProspectiveProvider::Equipment {
                            definition_id: definition.id.clone(),
                            copy_uuid: Uuid::from_u128(index as u128 + 0x10),
                            skill_id: activation.ability_id.clone(),
                        });
                    }
                }
                compile(data, identity, employee, inventory, &providers)?;
                let mut weapon_definition: Option<&EquipmentMetadata> = None;
                let mut weapon_count = 0usize;
                for definition in equipment.iter().flatten() {
                    if definition.equipment_type == EquipmentType::Weapon {
                        weapon_count += 1;
                        weapon_definition = Some(definition);
                    }
                }
                // Actual safe-zone/fragment and battle consumers reject multiple
                // weapons. Equipment-only sets were still checked independently above.
                if weapon_count != 1 {
                    continue;
                }
                let Some(weapon) =
                    weapon_definition.and_then(|definition| definition.weapon_profile.as_ref())
                else {
                    continue;
                };
                crate::game::resource::store::preflight_resource_clone(weapon)
                    .map_err(|e| RelicPlanError::control(e.into_game_error()))?;
                crate::authoring::control::charge_serialized(weapon, "publication_weapon_profile")
                    .map_err(RelicPlanError::control)?;
                admit(
                    Some(size_of::<UnitCombatProfile>() + 256),
                    "publication_compatibility_profile",
                )?;
                let mut profile = UnitCombatProfile::employee_default();
                profile
                    .apply_weapon_profile(weapon)
                    .map_err(RelicPlanError::control)?;
                charge(
                    data.skill_fragment_data.fragments.len(),
                    "publication_fragment_scan",
                )?;
                for fragment in &data.skill_fragment_data.fragments {
                    let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &fragment.effect else {
                        continue;
                    };
                    crate::authoring::control::charge_serialized(
                        &fragment.compatibility,
                        "publication_fragment_compatibility",
                    )
                    .map_err(RelicPlanError::control)?;
                    for (form_id, form) in forms {
                        crate::authoring::control::charge_serialized(
                            &form.compatibility,
                            "publication_form_compatibility",
                        )
                        .map_err(RelicPlanError::control)?;
                        admit(Some(32 * size_of::<crate::game::data::skill_fragment_data::SkillFragmentCompatibilityFailureCode>()), "publication_compatibility_report")?;
                        if !fragment
                            .form_compatibility_report(form_id, &profile)
                            .is_some_and(|report| report.is_compatible)
                        {
                            continue;
                        }
                        let bytes = fragment
                            .id
                            .as_str()
                            .len()
                            .checked_add(form_id.len())
                            .and_then(|n| n.checked_add(form.skill_id.as_str().len()))
                            .ok_or_else(overflow)?;
                        charge(bytes, "publication_fragment_provider")?;
                        admit(Some(bytes), "publication_fragment_provider_text")?;
                        providers.push(ProspectiveProvider::Fragment(FragmentSourceSnapshot {
                            copy_uuid: Uuid::from_u128(0x20),
                            definition_id: fragment.id.clone(),
                            form_id: form_id.clone(),
                            skill_id: form.skill_id.clone(),
                        }));
                        compile(data, identity, employee, inventory, &providers)?;
                        providers.pop();
                    }
                }
            }
        }
    }
    Ok(())
}
fn compile(
    data: &GameDataBase,
    identity: &ContentIdentity,
    employee: Uuid,
    inventory: &RelicInventory,
    providers: &[ProspectiveProvider],
) -> Result<(), RelicPlanError> {
    charge(1, "publication_plan")?;
    compiler::compile_components(UnpublishedPlanInput {
        data,
        generation: identity,
        employee_uuid: employee,
        inventory,
        providers,
    })
    .map(|_| ())
}
