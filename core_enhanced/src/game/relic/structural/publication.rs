//! Exhaustive publication over actual legal three-slot witnesses. Only exact
//! whole provider contexts share a proof, within this invocation and relic set.
//! Symbolic copy IDs and pure profiles never become execution authority.
use super::{
    compiler, validation, FaultCode, ProspectiveProvider, RelicPlanError, UnpublishedPlanInput,
};
use crate::authoring::clone_shape::CloneShape;
use crate::game::{
    ability::{DeliveryDef, SkillId},
    battle::{runtime_control, tile_range::TileRangePattern, types::UnitCombatProfile},
    data::{
        abnormality_data::BasicAttackAction,
        content_pack::{ContentContract, ContentIdentity},
        equipment_data::{EquipmentMetadata, EquipmentType, WeaponCombatProfile},
        relic_data::RelicDefinition,
        skill_fragment_data::{SkillFragmentEffectDef, SkillFragmentForm, SkillFragmentMetadata},
        GameDataBase,
    },
    relic::{OwnedRelic, RelicInventory, RelicVariant},
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
fn clone_charge(slots: usize, site: &'static str) -> Result<(), RelicPlanError> {
    crate::authoring::control::admit(
        crate::authoring::control::Resource::CloneSlots,
        0,
        slots,
        site,
    )
    .map_err(RelicPlanError::control)?;
    charge(slots, site)?;
    runtime_control::charge_clone(slots, site).map_err(RelicPlanError::control)
}
// Three choices with replacement, including the empty choice. Divide before
// multiplying so an intermediate cube cannot overflow a representable bound.
fn multisets(choices: usize) -> Result<usize, RelicPlanError> {
    let mut factors = [
        choices,
        choices.checked_add(1).ok_or_else(overflow)?,
        choices.checked_add(2).ok_or_else(overflow)?,
    ];
    for divisor in [2, 3] {
        let factor = factors
            .iter_mut()
            .find(|n| **n % divisor == 0)
            .ok_or_else(overflow)?;
        *factor /= divisor;
    }
    factors
        .into_iter()
        .try_fold(1usize, |n, f| n.checked_mul(f))
        .ok_or_else(overflow)
}

#[derive(Default)]
struct PublicationWork {
    // Conservative invocation-local allocation basis, including replaced cache
    // buffers and small preparation temporaries. No dropped allocation refunds
    // an active attempt; every compiler adds its own reservations to this basis.
    held: usize,
}
impl PublicationWork {
    fn reserve(&mut self, bytes: Option<usize>, site: &'static str) -> Result<(), RelicPlanError> {
        let next = bytes
            // Match completed-plan storage's allocation bookkeeping allowance.
            // Batched owners add their additional allocations at the call site.
            .and_then(|n| if n == 0 { Some(0) } else { n.checked_add(64) })
            .and_then(|n| self.held.checked_add(n))
            .ok_or_else(overflow)?;
        runtime_control::admit_retained_growth(self.held, Some(next), site)
            .map_err(RelicPlanError::control)?;
        self.held = next;
        Ok(())
    }
    fn grow<T>(&mut self, values: &mut Vec<T>, site: &'static str) -> Result<(), RelicPlanError> {
        if values.len() == values.capacity() {
            let capacity = values
                .capacity()
                .checked_mul(2)
                .ok_or_else(overflow)?
                .max(4);
            // Include both old and replacement buffers during reallocation.
            self.reserve(capacity.checked_mul(size_of::<T>()), site)?;
            values.reserve_exact(capacity - values.len());
        }
        Ok(())
    }
    fn temporary(&self, bytes: usize, site: &'static str) -> Result<(), RelicPlanError> {
        let bytes = if bytes == 0 {
            Some(0)
        } else {
            bytes.checked_add(64)
        };
        runtime_control::admit_retained_growth(
            self.held,
            bytes.and_then(|n| self.held.checked_add(n)),
            site,
        )
        .map_err(RelicPlanError::control)
    }
}

// A token refers to this immutable, invocation-local FULL binding. Distinct
// definitions/forms remain distinct tokens even if their skill matches. Tokens
// are an injective representation, never selector-based provider omission or
// copy-identity normalization. The vector also retains every original UUID.
#[derive(Clone, Copy)]
enum Binding<'a> {
    Equipment {
        definition: &'a EquipmentMetadata,
        skill: &'a SkillId,
    },
    Fragment {
        definition: &'a SkillFragmentMetadata,
        form_id: &'a String,
        form: &'a SkillFragmentForm,
    },
}
type BindingCopy = (usize, Uuid);
struct Catalog<'a> {
    bindings: Vec<Binding<'a>>,
    equipment: Vec<Vec<usize>>,
    forms: Vec<usize>,
    compatible: Vec<Option<Vec<usize>>>,
    max_equipment_bindings: usize,
}
impl<'a> Catalog<'a> {
    fn new(data: &'a GameDataBase, work: &mut PublicationWork) -> Result<Self, RelicPlanError> {
        charge(data.equipment_data.items.len(), "publication_binding_scan")?;
        charge(
            data.skill_fragment_data.fragments.len(),
            "publication_form_scan",
        )?;
        let equipment_count = data
            .equipment_data
            .items
            .iter()
            .try_fold(0usize, |n, d| n.checked_add(d.ability_activations.len()))
            .ok_or_else(overflow)?;
        let form_count = data
            .skill_fragment_data
            .fragments
            .iter()
            .try_fold(0usize, |n, f| {
                n.checked_add(match &f.effect {
                    SkillFragmentEffectDef::ActiveSkill { forms, .. } => forms.len(),
                    _ => 0,
                })
            })
            .ok_or_else(overflow)?;
        let binding_count = equipment_count
            .checked_add(form_count)
            .ok_or_else(overflow)?;
        charge(binding_count, "publication_binding_bound")?;
        work.reserve(
            binding_count.checked_mul(size_of::<Binding<'a>>()),
            "publication_binding_table",
        )?;
        work.reserve(
            data.equipment_data
                .items
                .len()
                .checked_mul(size_of::<Vec<usize>>()),
            "publication_equipment_bindings",
        )?;
        work.reserve(
            form_count.checked_mul(size_of::<usize>()),
            "publication_form_bindings",
        )?;
        work.reserve(
            data.equipment_data
                .items
                .len()
                .checked_mul(size_of::<Option<Vec<usize>>>()),
            "publication_compatibility_cache",
        )?;
        let mut result = Self {
            bindings: Vec::with_capacity(binding_count),
            equipment: Vec::with_capacity(data.equipment_data.items.len()),
            forms: Vec::with_capacity(form_count),
            compatible: Vec::with_capacity(data.equipment_data.items.len()),
            max_equipment_bindings: 0,
        };
        for definition in &data.equipment_data.items {
            work.reserve(
                definition
                    .ability_activations
                    .len()
                    .checked_mul(size_of::<usize>()),
                "publication_equipment_binding_ids",
            )?;
            let mut tokens = Vec::with_capacity(definition.ability_activations.len());
            for activation in &definition.ability_activations {
                charge(
                    tokens
                        .len()
                        .checked_mul(
                            activation
                                .ability_id
                                .as_str()
                                .len()
                                .checked_add(1)
                                .ok_or_else(overflow)?,
                        )
                        .ok_or_else(overflow)?,
                    "publication_provider_dedup",
                )?;
                if tokens.iter().any(|token| matches!(result.bindings[*token], Binding::Equipment { skill, .. } if skill == &activation.ability_id)) {
                    continue;
                }
                charge(
                    definition
                        .id
                        .len()
                        .checked_add(activation.ability_id.as_str().len())
                        .ok_or_else(overflow)?,
                    "publication_equipment_binding",
                )?;
                tokens.push(result.bindings.len());
                result.bindings.push(Binding::Equipment {
                    definition,
                    skill: &activation.ability_id,
                });
            }
            result.max_equipment_bindings = result.max_equipment_bindings.max(tokens.len());
            result.equipment.push(tokens);
            result.compatible.push(None);
        }
        for definition in &data.skill_fragment_data.fragments {
            let SkillFragmentEffectDef::ActiveSkill { forms, .. } = &definition.effect else {
                continue;
            };
            for (form_id, form) in forms {
                charge(
                    definition
                        .id
                        .as_str()
                        .len()
                        .checked_add(form_id.len())
                        .and_then(|n| n.checked_add(form.skill_id.as_str().len()))
                        .ok_or_else(overflow)?,
                    "publication_fragment_binding",
                )?;
                result.forms.push(result.bindings.len());
                result.bindings.push(Binding::Fragment {
                    definition,
                    form_id,
                    form,
                });
            }
        }
        Ok(result)
    }
    fn compatibility(
        &mut self,
        weapon_index: usize,
        data: &GameDataBase,
        work: &mut PublicationWork,
    ) -> Result<(), RelicPlanError> {
        charge(1, "publication_compatibility_lookup")?;
        if self.compatible[weapon_index].is_some() {
            return Ok(());
        }
        let Some(weapon) = data.equipment_data.items[weapon_index]
            .weapon_profile
            .as_ref()
        else {
            return Ok(());
        };
        // The native profile and native form predicate are unchanged. Their
        // inputs depend on this exact weapon definition, not unrelated gear.
        work.reserve(profile_bytes(weapon)?, "publication_compatibility_profile")?;
        crate::game::resource::store::preflight_resource_clone(weapon)
            .map_err(|e| RelicPlanError::control(e.into_game_error()))?;
        crate::game::resource::store::preflight_resource_clone(weapon)
            .map_err(|e| RelicPlanError::control(e.into_game_error()))?;
        let mut clone_meter = crate::authoring::control::CloneMeter::default();
        weapon
            .visit_clone_shape(&mut clone_meter)
            .map_err(RelicPlanError::control)?;
        weapon
            .visit_clone_shape(&mut clone_meter)
            .map_err(RelicPlanError::control)?;
        clone_meter
            .finish(
                "publication_profile_clone_storage",
                "publication_profile_clone",
            )
            .map_err(RelicPlanError::control)?;
        crate::authoring::control::charge_serialized(weapon, "publication_weapon_profile")
            .map_err(RelicPlanError::control)?;
        let mut profile = UnitCombatProfile::employee_default();
        profile
            .apply_weapon_profile(weapon)
            .map_err(RelicPlanError::control)?;
        charge(self.forms.len(), "publication_compatibility_bound")?;
        work.reserve(
            self.forms.len().checked_mul(size_of::<usize>()),
            "publication_compatible_forms",
        )?;
        let mut compatible = Vec::with_capacity(self.forms.len());
        for &token in &self.forms {
            let Binding::Fragment {
                definition,
                form_id,
                form,
            } = self.bindings[token]
            else {
                unreachable!();
            };
            crate::authoring::control::charge_serialized(
                &definition.compatibility,
                "publication_fragment_compatibility",
            )
            .map_err(RelicPlanError::control)?;
            crate::authoring::control::charge_serialized(
                &form.compatibility,
                "publication_form_compatibility",
            )
            .map_err(RelicPlanError::control)?;
            work.temporary(
                32 * size_of::<
                    crate::game::data::skill_fragment_data::SkillFragmentCompatibilityFailureCode,
                >(),
                "publication_compatibility_report",
            )?;
            if definition
                .form_compatibility_report(form_id, &profile)
                .is_some_and(|report| report.is_compatible)
            {
                compatible.push(token);
            }
        }
        self.compatible[weapon_index] = Some(compatible);
        Ok(())
    }
    fn provider(
        &self,
        key: BindingCopy,
        work: &mut PublicationWork,
    ) -> Result<ProspectiveProvider, RelicPlanError> {
        let (token, copy_uuid) = key;
        let (bytes, strings) = match self.bindings[token] {
            Binding::Equipment { definition, skill } => (
                definition
                    .id
                    .len()
                    .checked_add(skill.as_str().len())
                    .ok_or_else(overflow)?,
                2usize,
            ),
            Binding::Fragment {
                definition,
                form_id,
                form,
            } => (
                definition
                    .id
                    .as_str()
                    .len()
                    .checked_add(form_id.len())
                    .and_then(|n| n.checked_add(form.skill_id.as_str().len()))
                    .ok_or_else(overflow)?,
                3,
            ),
        };
        work.reserve(
            strings.checked_mul(64).and_then(|n| n.checked_add(bytes)),
            "publication_provider_text",
        )?;
        let clone_slots = bytes.checked_add(strings).ok_or_else(overflow)?;
        clone_charge(clone_slots, "publication_provider_clone")?;
        Ok(match self.bindings[token] {
            Binding::Equipment { definition, skill } => ProspectiveProvider::Equipment {
                definition_id: definition.id.clone(),
                copy_uuid,
                skill_id: skill.clone(),
            },
            Binding::Fragment {
                definition,
                form_id,
                form,
            } => ProspectiveProvider::Fragment(FragmentSourceSnapshot {
                definition_id: definition.id.clone(),
                copy_uuid,
                form_id: form_id.clone(),
                skill_id: form.skill_id.clone(),
            }),
        })
    }
}

fn pattern_bytes(pattern: &TileRangePattern) -> Result<usize, RelicPlanError> {
    charge(pattern.rows.len(), "publication_profile_storage_scan")?;
    let allocations = pattern
        .rows
        .len()
        .checked_add(usize::from(!pattern.rows.is_empty()))
        .and_then(|n| n.checked_mul(64))
        .ok_or_else(overflow)?;
    pattern.rows.iter().try_fold(
        pattern
            .rows
            .len()
            .checked_mul(size_of::<String>())
            .and_then(|n| n.checked_add(allocations))
            .ok_or_else(overflow)?,
        |n, row| n.checked_add(row.len()).ok_or_else(overflow),
    )
}
fn profile_bytes(weapon: &WeaponCombatProfile) -> Result<Option<usize>, RelicPlanError> {
    let mut owned = pattern_bytes(&weapon.defense_tile_range)?;
    if let Some(marks) = &weapon.mark_weapon { owned=owned.checked_add(marks.retained_dynamic_extent().ok_or_else(overflow)?).ok_or_else(overflow)?; }
    if let BasicAttackAction::Heal {
        footprint: Some(pattern),
        ..
    } = &weapon.action
    {
        owned = owned
            .checked_add(pattern_bytes(pattern)?)
            .ok_or_else(overflow)?;
    }
    if let DeliveryDef::TileArea { footprint, .. } = &weapon.delivery {
        owned = owned
            .checked_add(pattern_bytes(footprint)?)
            .ok_or_else(overflow)?;
    }
    // apply_weapon_profile clones each owned leaf into both the basic attack and
    // full weapon snapshot. The employee default also allocates three rows.
    Ok(owned.checked_mul(2).and_then(|n| {
        n.checked_add(size_of::<UnitCombatProfile>() + 3 * (size_of::<String>() + 3) + 4 * 64)
    }))
}

struct Context {
    equipment: Vec<BindingCopy>,
    weapons: Vec<usize>,
    // Alternatives, NEVER a union: each token becomes one separate full vector.
    fragments: Vec<usize>,
}
fn search_work(length: usize, key_length: usize) -> Result<usize, RelicPlanError> {
    let comparisons = usize::BITS as usize - length.leading_zeros() as usize + 1;
    comparisons
        .checked_mul(key_length.checked_add(1).ok_or_else(overflow)?)
        .ok_or_else(overflow)
}
fn insert_id(
    values: &mut Vec<usize>,
    value: usize,
    work: &mut PublicationWork,
    site: &'static str,
) -> Result<bool, RelicPlanError> {
    charge(search_work(values.len(), 1)?, site)?;
    match values.binary_search(&value) {
        Ok(_) => Ok(false),
        Err(index) => {
            charge(values.len(), site)?;
            work.grow(values, site)?;
            values.insert(index, value);
            Ok(true)
        }
    }
}
fn discover(
    data: &GameDataBase,
    catalog: &mut Catalog<'_>,
    work: &mut PublicationWork,
) -> Result<Vec<Context>, RelicPlanError> {
    let items = &data.equipment_data.items;
    charge(
        multisets(items.len().checked_add(1).ok_or_else(overflow)?)?,
        "publication_equipment_discovery_bound",
    )?;
    work.reserve(
        Some(size_of::<ItemSlot>() + 4 * size_of::<EquippedRef>()),
        "publication_equipment_slots",
    )?;
    let mut slots = ItemSlot::new(SlotLayoutKind::Any3);
    let key_capacity = catalog
        .max_equipment_bindings
        .checked_mul(3)
        .ok_or_else(overflow)?;
    work.reserve(
        key_capacity.checked_mul(size_of::<BindingCopy>()),
        "publication_equipment_key",
    )?;
    let mut key = Vec::with_capacity(key_capacity);
    let mut contexts: Vec<Context> = Vec::new();
    for a in 0..=items.len() {
        for b in a..=items.len() {
            for c in b..=items.len() {
                charge(1, "publication_equipment_witness")?;
                while !slots.is_empty() {
                    charge(slots.len(), "publication_equipment_reset")?;
                    let copy = Uuid::from_u128(slots.len() as u128 - 1 + 0x10);
                    slots.remove_by_instance(copy).ok_or_else(|| {
                        RelicPlanError::new(FaultCode::InvalidReference, "publication slot reset")
                    })?;
                }
                key.clear();
                let mut count = 0usize;
                let mut weapon_count = 0usize;
                let mut weapon_index = 0usize;
                let mut valid = true;
                for choice in [a, b, c] {
                    if choice == 0 {
                        continue;
                    }
                    let index = choice - 1;
                    let definition = &items[index];
                    let copy_uuid = Uuid::from_u128(count as u128 + 0x10);
                    charge(3, "publication_equipment_admission")?;
                    if !definition.allow_duplicate_equip {
                        work.temporary(
                            size_of::<std::slice::Iter<'_, EquippedRef>>(),
                            "publication_equipment_duplicate_scan",
                        )?;
                    }
                    if slots
                        .equip(
                            EquippedRef {
                                instance_uuid: copy_uuid,
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
                    charge(
                        catalog.equipment[index].len(),
                        "publication_equipment_key_fill",
                    )?;
                    key.extend(
                        catalog.equipment[index]
                            .iter()
                            .map(|&token| (token, copy_uuid)),
                    );
                    count += 1;
                    if definition.equipment_type == EquipmentType::Weapon {
                        weapon_count += 1;
                        weapon_index = index;
                    }
                }
                if !valid {
                    continue;
                }
                // Numeric comparisons still compare every full binding and UUID
                // through the immutable exact token table. No source text clones.
                charge(
                    search_work(contexts.len(), key.len())?,
                    "publication_context_lookup",
                )?;
                let context_index =
                    match contexts.binary_search_by(|context| context.equipment.cmp(&key)) {
                        Ok(index) => index,
                        Err(index) => {
                            charge(
                                contexts.len().checked_add(key.len()).ok_or_else(overflow)?,
                                "publication_context_insert",
                            )?;
                            work.grow(&mut contexts, "publication_context_table")?;
                            work.reserve(
                                key.len().checked_mul(size_of::<BindingCopy>()),
                                "publication_context_key",
                            )?;
                            clone_charge(
                                key.len()
                                    .checked_mul(3)
                                    .and_then(|n| n.checked_add(1))
                                    .ok_or_else(overflow)?,
                                "publication_context_key_copy",
                            )?;
                            contexts.insert(
                                index,
                                Context {
                                    equipment: key.clone(),
                                    weapons: Vec::new(),
                                    fragments: Vec::new(),
                                },
                            );
                            index
                        }
                    };
                if weapon_count != 1 || items[weapon_index].weapon_profile.is_none() {
                    continue;
                }
                // Visit each legal weapon witness; reuse only an identical static
                // profile/form predicate, and exact equipment vector expansion.
                let context = &mut contexts[context_index];
                if !insert_id(
                    &mut context.weapons,
                    weapon_index,
                    work,
                    "publication_weapon_witness_lookup",
                )? {
                    continue;
                }
                catalog.compatibility(weapon_index, data, work)?;
                for &token in catalog.compatible[weapon_index]
                    .as_ref()
                    .expect("profile checked")
                {
                    insert_id(
                        &mut context.fragments,
                        token,
                        work,
                        "publication_fragment_context_lookup",
                    )?;
                }
            }
        }
    }
    Ok(contexts)
}

pub(super) fn validate(data: &GameDataBase) -> Result<(), RelicPlanError> {
    charge(
        data.relic_data.relics.len(),
        "publication_structural_presence",
    )?;
    let mut has_structural = false;
    for definition in &data.relic_data.relics {
        has_structural |= !definition.rules.is_empty() || definition.corruption.as_ref().is_some_and(|c| !c.rules.is_empty())
            || super::support::declares_conversion_for(data, &definition.id, RelicVariant::Normal)?
            || super::support::declares_conversion_for(data, &definition.id, RelicVariant::Corrupt)?
            || super::action::declares(data, &definition.id, RelicVariant::Normal)?
            || super::action::declares(data, &definition.id, RelicVariant::Corrupt)?;
    }
    if !has_structural {
        return Ok(());
    }
    let mut work = PublicationWork::default();
    let capacity = data
        .relic_data
        .relics
        .len()
        .checked_mul(2)
        .ok_or_else(overflow)?;
    work.reserve(
        capacity.checked_mul(size_of::<(&RelicDefinition, RelicVariant)>()),
        "publication_variants",
    )?;
    let mut variants = Vec::with_capacity(capacity);
    charge(data.relic_data.relics.len(), "publication_variant_scan")?;
    for definition in &data.relic_data.relics {
        variants.push((definition, RelicVariant::Normal));
        if definition.corruption.is_some() {
            variants.push((definition, RelicVariant::Corrupt));
        }
    }
    let relic_bound = multisets(variants.len().checked_add(1).ok_or_else(overflow)?)?;
    charge(relic_bound, "publication_relic_discovery_bound")?;
    let mut catalog = Catalog::new(data, &mut work)?;
    let contexts = discover(data, &mut catalog, &mut work)?;
    charge(contexts.len(), "publication_context_count")?;
    let context_count = contexts
        .iter()
        .try_fold(0usize, |n, c| {
            n.checked_add(1)
                .and_then(|n| n.checked_add(c.fragments.len()))
        })
        .ok_or_else(overflow)?;
    // Only proof-equivalent full inputs share this smaller bound. Every actual
    // equipment witness has already passed real admission and compatibility.
    charge(
        relic_bound
            .checked_mul(context_count)
            .ok_or_else(overflow)?,
        "publication_context_relic_bound",
    )?;
    work.reserve(
        Some(size_of::<ContentIdentity>() + 64),
        "publication_identity",
    )?;
    charge(64, "publication_identity_text")?;
    let identity = ContentIdentity {
        contract: ContentContract::current(),
        semantic_sha256: "0".repeat(64),
    };
    let employee = Uuid::from_u128(0x435253415f50524f4f46);
    for a in 0..=variants.len() {
        for b in a..=variants.len() {
            for c in b..=variants.len() {
                charge(3, "publication_relic_witness")?;
                let mut selected = [None; 3];
                let mut count = 0usize;
                for index in [a, b, c] {
                    if index > 0 {
                        selected[count] = Some(variants[index - 1]);
                        count += 1;
                    }
                }
                let mut has_structural = false;
                for (definition, variant) in selected.iter().flatten() {
                    has_structural |= !validation::rules_for(definition, *variant).is_empty()
                        || super::support::declares_conversion_for(data, &definition.id, *variant)?
                        || super::action::declares(data, &definition.id, *variant)?;
                }
                if !has_structural {
                    continue;
                }
                work.reserve(
                    count.checked_mul(size_of::<(&RelicDefinition, RelicVariant)>()),
                    "publication_relic_selection",
                )?;
                let mut definitions = Vec::with_capacity(count);
                definitions.extend(selected.into_iter().flatten());
                // publication_input keeps its own original admission. This extra
                // basis guarantees its allocation also fits beside the caches.
                // At most three copies and one employee occupy one BTree node
                // apiece. Match the native relic_copy_grant 11-entry/224-byte
                // node bound, including spare entries and both tree headers.
                let inventory_bytes = definitions.iter().try_fold(
                    size_of::<(Uuid, OwnedRelic)>()
                        .checked_add(size_of::<(Uuid, [Option<Uuid>; 3])>())
                        .and_then(|n| n.checked_mul(11))
                        .and_then(|n| n.checked_add(size_of::<RelicInventory>() + 448))
                        .ok_or_else(overflow)?,
                    |n, (d, _)| n.checked_add(d.id.len()).and_then(|n| n.checked_add(64)),
                );
                work.reserve(inventory_bytes, "publication_inventory_with_cache")?;
                let clone_slots = definitions
                    .iter()
                    .try_fold(0usize, |n, (d, _)| {
                        d.id.len()
                            .checked_add(1)
                            .and_then(|bytes| n.checked_add(bytes))
                    })
                    .ok_or_else(overflow)?;
                clone_charge(clone_slots, "publication_inventory_copy")?;
                let Some(inventory) =
                    RelicInventory::publication_input(employee, &definitions, &data.relic_data)
                        .map_err(RelicPlanError::control)?
                else {
                    continue;
                };
                for context in &contexts {
                    charge(
                        context
                            .equipment
                            .len()
                            .checked_add(context.fragments.len())
                            .ok_or_else(overflow)?,
                        "publication_context_expand",
                    )?;
                    work.reserve(
                        context
                            .equipment
                            .len()
                            .checked_add(1)
                            .and_then(|n| n.checked_mul(size_of::<ProspectiveProvider>())),
                        "publication_provider_table",
                    )?;
                    let mut providers = Vec::with_capacity(context.equipment.len() + 1);
                    for &key in &context.equipment {
                        providers.push(catalog.provider(key, &mut work)?);
                    }
                    compile(data, &identity, employee, &inventory, &providers, work.held)?;
                    for &token in &context.fragments {
                        providers
                            .push(catalog.provider((token, Uuid::from_u128(0x20)), &mut work)?);
                        compile(data, &identity, employee, &inventory, &providers, work.held)?;
                        providers.pop();
                    }
                }
            }
        }
    }
    crate::authoring::control::poll("publication_complete").map_err(RelicPlanError::control)
}
fn compile(
    data: &GameDataBase,
    identity: &ContentIdentity,
    employee: Uuid,
    inventory: &RelicInventory,
    providers: &[ProspectiveProvider],
    retained_basis: usize,
) -> Result<(), RelicPlanError> {
    charge(1, "publication_plan")?;
    compiler::compile_publication_components(
        UnpublishedPlanInput {
            data,
            generation: identity,
            employee_uuid: employee,
            inventory,
            providers,
        },
        retained_basis,
    )
}
