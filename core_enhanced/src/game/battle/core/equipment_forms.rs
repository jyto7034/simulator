use super::BattleCore;
use crate::{authoring::control, game::{
    ability::equipment_forms::EquipmentFormEffect,
    battle::{damage::{DamageSource, DamageSourceSnapshot}, ids::UnitInstanceId,
        runtime_control, runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource}, types::BattleVerdict},
    behavior::GameError,
    resource::{ResourceMutation, ResourceOperation, ResourceOwner, ResourceOwnerSelector},
    stats::{StatId, StatModifier, StatModifierKind},
}};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum FormOperation { Equipped, Earned, Confirmed, Discarded }
crate::authoring::clone_shape::copy_shape!(FormOperation);
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub equipment_copy: uuid::Uuid,
    pub operation: FormOperation,
    pub pending: u32,
    pub confirmed: u32,
    pub form: crate::game::ability::equipment_forms::EquipmentFormView,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(FormWitness {
    generation, time_ms, actor, equipment_copy, operation, pending, confirmed,
    form, definition_id, source_file, field_path
});
impl FormWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = std::mem::size_of::<Self>().checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?.checked_add(self.field_path.capacity())?
            .checked_add(self.form.name.capacity())?.checked_add(self.form.description.capacity())?.checked_add(self.form.appearance.capacity())?;
        if let Some(generation) = &self.generation {
            bytes = bytes.checked_add(std::mem::size_of_val(generation.as_ref()))?.checked_add(generation.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}

/// Registration survives deployment loss; balances live only in ResourceStore.
#[derive(Debug, Clone, Default)]
pub(super) struct FormRegistrations {
    copies: BTreeMap<uuid::Uuid, (UnitInstanceId, EquipmentSourceSnapshot)>,
}
crate::authoring::clone_shape::struct_shape!(FormRegistrations { copies });
impl FormRegistrations {
    pub(super) fn len(&self) -> usize { self.copies.len() }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.len().checked_mul(128 + std::mem::size_of::<(uuid::Uuid, UnitInstanceId, EquipmentSourceSnapshot)>())
    }
}

impl BattleCore {
    pub(super) fn equipment_form_kill(&mut self, source: &DamageSourceSnapshot, victim: UnitInstanceId, now: u64) -> Result<(), GameError> {
        if source.fact_source.reaction || source.fact_source.guard_release().is_some()
            || source.fact_source.cast.as_ref().is_some_and(|c| c.proc_origin.is_some())
            || !matches!(source.source, DamageSource::BasicAttack | DamageSource::Ability)
            || !self.eligible_encounter_death(victim, source.source_side)? { return Ok(()); }
        for (equipment, program) in self.equipped_attack_providers(source.source_id, |p| p.victory_forms.is_some())? {
            let forms = program.victory_forms.as_ref().unwrap();
            let copy = equipment.persistent_copy_uuid.unwrap();
            if !self.sustained.forms.copies.contains_key(&copy) {
                self.admit_authoring_runtime_growth(1, "form_registration")?;
                runtime_control::admit_retained_growth(0, Some(128 + std::mem::size_of::<(uuid::Uuid, UnitInstanceId, EquipmentSourceSnapshot)>()), "form_registration_storage")?;
                self.sustained.forms.copies.insert(copy, (source.source_id, equipment));
            }
            let mut fact = source.fact_source.clone();
            fact.actor = Some(source.source_id);
            fact.equipment = Some(equipment);
            fact.authored = Some(Arc::clone(&program.forms_source));
            let prior = std::mem::replace(&mut self.runtime_fact_source, fact);
            let result = self.apply_typed_resource_operations(&[ResourceOperation {
                resource: forms.pending.clone(), owner: ResourceOwnerSelector::SourceEquipment,
                operation: ResourceMutation::Grant(1),
            }], &[], now);
            self.runtime_fact_source = prior;
            result?;
            self.observe_equipment_form(source.source_id, copy, &program, now, FormOperation::Earned)?;
        }
        control::poll("equipment_form_earned")
    }

    pub(super) fn settle_equipment_forms(&mut self, verdict: BattleVerdict, now: u64) -> Result<(), GameError> {
        let count = self.sustained.forms.len();
        control::charge(count as u64, "form_settlement")?;
        runtime_control::admit_retained_growth(0, count.checked_mul(std::mem::size_of::<(uuid::Uuid, UnitInstanceId, EquipmentSourceSnapshot)>()), "form_settlement_capture")?;
        let copies: Vec<_> = self.sustained.forms.copies.iter().map(|(copy, (actor, equipment))| (*copy, *actor, *equipment)).collect();
        for (copy, actor, equipment) in copies {
            let program = Arc::clone(self.game_data.equipment_attack_catalog().by_equipment.get(&equipment.definition_uuid).ok_or(GameError::InvalidAction)?);
            let forms = program.victory_forms.as_ref().ok_or(GameError::InvalidAction)?;
            let runtime = self.runtime_facts.resources.as_ref().ok_or(GameError::InvalidAction)?;
            let pending = runtime.store.bind(&forms.pending, ResourceOwner::EquipmentCopy(copy), &runtime.attempt).map_err(|e| e.into_game_error())?;
            let amount = runtime.store.value(&pending).map_err(|e| e.into_game_error())?.balance;
            if amount == 0 { continue; }
            let mut operations = vec![(pending, ResourceMutation::Consume(amount))];
            if verdict == BattleVerdict::Victory {
                let confirmed = runtime.store.bind(&forms.confirmed, ResourceOwner::EquipmentCopy(copy), &runtime.run.as_ref().ok_or(GameError::InvalidAction)?.scope).map_err(|e| e.into_game_error())?;
                operations.push((confirmed, ResourceMutation::Grant(1)));
            }
            self.apply_typed_resource_batch(&operations, &RuntimeFactSource {
                actor: Some(actor), equipment: Some(equipment), authored: Some(Arc::clone(&program.forms_source)), ..Default::default()
            }, now)?;
            self.observe_equipment_form(actor, copy, &program, now, if verdict == BattleVerdict::Victory { FormOperation::Confirmed } else { FormOperation::Discarded })?;
        }
        control::poll("equipment_forms_settled")
    }

    /// The current form is derived from the mounted Run, never cached per unit.
    fn equipment_form_totals(&self, actor: UnitInstanceId) -> Result<[i32; 3], GameError> {
        let mut totals = [0i32; 3];
        if self.sustained.closed { return Ok(totals); }
        let Some(runtime) = self.runtime_facts.resources.as_ref().filter(|r| r.run.is_some()) else { return Ok(totals); };
        for (equipment, program) in self.equipped_attack_providers(actor, |p| p.victory_forms.is_some())? {
            let forms = program.victory_forms.as_ref().unwrap();
            let lease = runtime.store.bind(&forms.confirmed, ResourceOwner::EquipmentCopy(equipment.persistent_copy_uuid.unwrap()), &runtime.run.as_ref().unwrap().scope).map_err(|e| e.into_game_error())?;
            let balance = runtime.store.value(&lease).map_err(|e| e.into_game_error())?.balance;
            let (index, value) = match forms.selected(balance).2.effect {
                EquipmentFormEffect::MaxHealth { amount } => (0, amount),
                EquipmentFormEffect::Attack { amount } => (1, amount),
                EquipmentFormEffect::IncomingDamage { percent } => (2, percent),
            };
            totals[index] = totals[index].checked_add(value as i32).ok_or_else(|| runtime_control::invariant("equipment_form_effect_overflow"))?;
        }
        Ok(totals)
    }

    pub(super) fn equipment_form_modifiers(&self, actor: UnitInstanceId) -> Result<[StatModifier; 2], GameError> {
        let totals = self.equipment_form_totals(actor)?;
        Ok([
            StatModifier { stat: StatId::MaxHealth, kind: StatModifierKind::Flat, value: totals[0] },
            StatModifier { stat: StatId::Attack, kind: StatModifierKind::Flat, value: totals[1] },
        ])
    }
    pub(super) fn equipment_form_damage_percent(&self, actor: UnitInstanceId) -> Result<i32, GameError> {
        Ok(self.equipment_form_totals(actor)?[2])
    }
    pub(super) fn reconcile_equipment_forms(&mut self) -> Result<(), GameError> {
        let catalog = self.game_data.equipment_attack_catalog();
        control::charge(catalog.by_equipment.len() as u64, "form_stat_catalog")?;
        if !catalog.by_equipment.values().any(|p| p.victory_forms.is_some()) { return Ok(()); }
        control::charge(self.units.len() as u64, "form_stat_units")?;
        runtime_control::admit_retained_growth(0, self.units.len().checked_mul(std::mem::size_of::<UnitInstanceId>()), "form_stat_targets")?;
        let targets: Vec<_> = self.units.keys().copied().collect();
        for target in targets {
            let before = (self.units[&target].stats.max_health, self.units[&target].stats.attack);
            let active = self.equipment_form_modifiers(target)?.iter().any(|m| m.value != 0);
            if active { self.ensure_passive_stat_base(target)?; }
            if active || self.has_sustained_stat_base(target) { self.refresh_sustained_target(target)?; }
            // Incoming-damage forms do not alter UnitStats. Seed the latest
            // observation once even when there is no HP/Attack delta, then
            // preserve actual earned/settled observations across idle refreshes.
            let needs_initial_observation = self.runtime_facts.resources.as_ref()
                .is_some_and(|runtime| runtime.run.is_some())
                && self.authoring_observation.as_ref()
                    .is_some_and(|observation| observation.latest_equipment_form.is_none());
            if !self.sustained.closed && (needs_initial_observation
                || before != (self.units[&target].stats.max_health, self.units[&target].stats.attack)) {
                for (equipment, program) in self.equipped_attack_providers(target, |p| p.victory_forms.is_some())? {
                    self.observe_equipment_form(target, equipment.persistent_copy_uuid.unwrap(), &program, self.sustained_observed_time(), FormOperation::Equipped)?;
                }
            }
        }
        Ok(())
    }
    fn observe_equipment_form(&mut self, actor: UnitInstanceId, copy: uuid::Uuid,
        program: &crate::game::ability::equipment_attack::CompiledEquipmentAttack,
        time_ms: u64, operation: FormOperation) -> Result<(), GameError> {
        if self.authoring_observation.is_none() { return Ok(()); }
        let forms = program.victory_forms.as_ref().unwrap();
        let runtime = self.runtime_facts.resources.as_ref().ok_or(GameError::InvalidAction)?;
        let read = |id, scope| {
            let lease = runtime.store.bind(id, ResourceOwner::EquipmentCopy(copy), scope).map_err(|e| e.into_game_error())?;
            runtime.store.value(&lease).map(|v| v.balance).map_err(|e| e.into_game_error())
        };
        let pending = read(&forms.pending, &runtime.attempt)?;
        let confirmed = read(&forms.confirmed, &runtime.run.as_ref().ok_or(GameError::InvalidAction)?.scope)?;
        let origin = &program.forms_source;
        let selected = forms.selected(confirmed).2;
        let bytes = std::mem::size_of::<FormWitness>() + selected.name.len() + selected.description.len() + selected.appearance.len()
            + origin.site.action.definition_id.as_str().len() + origin.source_file.len() + origin.field_path.len();
        runtime_control::admit_retained_growth(0, Some(bytes), "form_witness_copy")?;
        control::charge(bytes as u64, "form_witness_copy")?;
        let witness = FormWitness {
            generation: self.content_identity.clone(), time_ms, actor, equipment_copy: copy, operation,
            pending, confirmed, form: forms.view(confirmed),
            definition_id: origin.site.action.definition_id.to_string(), source_file: origin.source_file.clone(), field_path: origin.field_path.clone(),
        };
        runtime_control::admit_retained_growth(0, witness.retained_extent(), "form_observation")?;
        self.authoring_observation.as_mut().unwrap().latest_equipment_form = Some(witness);
        Ok(())
    }

}
