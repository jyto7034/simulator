//! Immutable structural ownership at the same accepted-source boundary as the
//! original skill. A strong Arc keeps authored data alive; action/provider and
//! delivery leases remain the separate authority to execute it.
use super::BattleCore;
use crate::game::{
    ability::{authoring::EffectSite, SkillDef, SkillId},
    battle::{
        ids::UnitInstanceId, runtime_facts::RuntimeFactSource, types::BattleUnitSourceIdentity,
    },
    behavior::GameError,
    data::content_pack::ContentIdentity,
    relic::structural::{EffectiveEffect, EmployeeEffectivePlan, ProspectiveProvider},
    value_expr::{AmountPlanKind, CompiledAmountEntry},
};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

pub(crate) struct EffectiveBattleOwner {
    pub plans: BTreeMap<Uuid, Arc<EmployeeEffectivePlan>>,
    extent: usize,
}
impl std::fmt::Debug for EffectiveBattleOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectiveBattleOwner")
            .field("employees", &self.plans.len())
            .field("extent", &self.extent)
            .finish()
    }
}
impl EffectiveBattleOwner {
    pub(crate) fn new(
        plans: BTreeMap<Uuid, Arc<EmployeeEffectivePlan>>,
    ) -> Result<Option<Arc<Self>>, GameError> {
        if plans.is_empty() {
            return Ok(None);
        }
        let extent = plans
            .values()
            .try_fold(std::mem::size_of::<Self>(), |n, plan| {
                n.checked_add(plan.usage.reserved_bytes).and_then(|n| {
                    n.checked_add(std::mem::size_of::<(Uuid, Arc<EmployeeEffectivePlan>)>() + 64)
                })
            })
            .ok_or_else(|| {
                GameError::InvalidStaticData("effective owner extent overflow".into())
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(extent),
            "effective_battle_owner",
        )?;
        Ok(Some(Arc::new(Self { plans, extent })))
    }
    pub(crate) fn retained_extent(&self) -> usize {
        self.extent
    }
}
pub(crate) struct EffectiveSkillBinding {
    pub plan: Arc<EmployeeEffectivePlan>,
    pub provider: usize,
    pub actor: UnitInstanceId,
}
impl std::fmt::Debug for EffectiveSkillBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectiveSkillBinding")
            .field("fingerprint", &self.plan.fingerprint)
            .field("provider", &self.provider)
            .field("actor", &self.actor)
            .finish()
    }
}
impl PartialEq for EffectiveSkillBinding {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.plan, &other.plan)
            && self.provider == other.provider
            && self.actor == other.actor
    }
}
impl Eq for EffectiveSkillBinding {}
impl serde::Serialize for EffectiveSkillBinding {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut view = serializer.serialize_struct("EffectiveSkillObservation", 4)?;
        view.serialize_field("generation", self.plan.generation.identity())?;
        view.serialize_field("fingerprint", &self.plan.fingerprint)?;
        view.serialize_field("provider_index", &self.provider)?;
        view.serialize_field("employee_uuid", &self.plan.employee_uuid)?;
        view.end()
    }
}
impl EffectiveSkillBinding {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(64)?
            .checked_add(self.plan.usage.reserved_bytes)
    }
    pub(crate) fn execution(&self) -> &crate::game::relic::structural::EffectiveProviderPlan {
        &self.plan.execution[self.provider]
    }
    pub(crate) fn validate(
        &self,
        generation: &ContentIdentity,
        source: &RuntimeFactSource,
    ) -> Result<(), GameError> {
        if self.plan.generation.identity() != generation || source.actor != Some(self.actor) {
            return Err(GameError::InvalidStaticData(
                "effective generation/actor binding differs".into(),
            ));
        }
        match &self.plan.providers[self.provider] {
            ProspectiveProvider::Fragment(fragment)
                if source.equipment.is_none() && source.fragment.as_deref() == Some(fragment) => {}
            ProspectiveProvider::Equipment { copy_uuid, .. }
                if source.equipment.is_some_and(|equipment| {
                    equipment.persistent_copy_uuid == Some(*copy_uuid)
                }) => {}
            _ => {
                return Err(GameError::InvalidStaticData(
                    "effective provider copy binding differs".into(),
                ))
            }
        }
        if source.cast.as_ref().is_some_and(|cast| {
            cast.skill_id != self.execution().skill.id || cast.caster_instance_id != self.actor
        }) {
            return Err(GameError::InvalidStaticData(
                "effective cast binding differs".into(),
            ));
        }
        Ok(())
    }
    pub(crate) fn effect(&self, site: &EffectSite) -> Option<&EffectiveEffect> {
        self.execution()
            .effects
            .iter()
            .find(|(bound, _)| bound == site)
            .map(|(_, index)| &self.plan.effects[*index])
    }
    pub(crate) fn compose_amount(
        &self,
        site: &EffectSite,
        kind: AmountPlanKind,
        amount: super::value_expr_runtime::RuntimeAmount,
    ) -> Result<super::value_expr_runtime::RuntimeAmount, GameError> {
        use crate::game::relic::structural::{
            authoring::{RelicProperty, TypedRelicValue},
            capabilities,
        };
        let property = match kind {
            AmountPlanKind::Damage => RelicProperty::DamageAmount,
            AmountPlanKind::Shield => RelicProperty::PersonalShieldInitialAmount,
            AmountPlanKind::Heal => return Ok(amount),
        };
        crate::authoring::control::charge(
            self.execution().effects.len() as u64,
            "effective_amount_descriptor",
        )?;
        let effect = self.effect(site).ok_or_else(|| {
            GameError::InvalidStaticData("effective effect binding absent".into())
        })?;
        let Some(program) = effect.properties.iter().find(|p| p.property == property) else {
            return Ok(amount);
        };
        let base = match (kind, amount) {
            (AmountPlanKind::Damage, super::value_expr_runtime::RuntimeAmount::Damage(base))
            | (AmountPlanKind::Shield, super::value_expr_runtime::RuntimeAmount::Shield(base)) => {
                base
            }
            _ => {
                return Err(GameError::InvalidStaticData(
                    "effective amount has wrong runtime unit".into(),
                ))
            }
        };
        let descriptor = capabilities::descriptor(effect.operation, property).ok_or_else(|| {
            GameError::InvalidStaticData("effective property descriptor absent".into())
        })?;
        let value = program
            .program
            .apply(TypedRelicValue::Hp(i64::from(base)), descriptor)
            .map_err(|e| GameError::InvalidStaticData(e.to_string()))?;
        let final_value = u32::try_from(value.value())
            .map_err(|_| GameError::InvalidStaticData("effective amount overflow".into()))?;
        Ok(match kind {
            AmountPlanKind::Damage => super::value_expr_runtime::RuntimeAmount::Damage(final_value),
            AmountPlanKind::Shield => super::value_expr_runtime::RuntimeAmount::Shield(final_value),
            AmountPlanKind::Heal => unreachable!(),
        })
    }
}
impl BattleCore {
    pub(crate) fn set_effective_owner(
        &mut self,
        owner: Option<Arc<EffectiveBattleOwner>>,
    ) -> Result<(), GameError> {
        if let Some(owner) = &owner {
            let before = self.retained_runtime_fact_extent()?;
            crate::game::battle::runtime_control::admit_retained_growth(
                before,
                before.checked_add(owner.retained_extent()),
                "effective_battle_binding",
            )?;
            if self.content_identity.as_deref().is_none_or(|generation| {
                owner
                    .plans
                    .values()
                    .any(|plan| plan.generation.identity() != generation)
            }) {
                return Err(GameError::InvalidStaticData(
                    "effective owner generation differs from battle".into(),
                ));
            }
        }
        self.effective_owner = owner;
        Ok(())
    }
    pub(super) fn current_effective_skill<'a>(
        &'a self,
        id: &SkillId,
        actor: UnitInstanceId,
        source: &RuntimeFactSource,
    ) -> Option<&'a crate::game::relic::structural::EffectiveProviderPlan> {
        let owner = self.effective_owner.as_ref()?;
        let unit = self.units.get(&actor)?;
        let plan = owner.plans.get(&unit.source_owned_uuid)?;
        if crate::authoring::control::charge(
            plan.providers.len() as u64,
            "effective_readonly_provider",
        )
        .is_err()
        {
            return None;
        }
        let index = plan.providers.iter().position(|provider| match provider {
            ProspectiveProvider::Fragment(fragment) => {
                source.equipment.is_none()
                    && fragment.skill_id == *id
                    && source
                        .fragment
                        .as_deref()
                        .or(unit.fragment_source.as_deref())
                        == Some(fragment)
            }
            ProspectiveProvider::Equipment {
                copy_uuid,
                skill_id,
                ..
            } => {
                skill_id == id
                    && source.equipment.is_some_and(|equipment| {
                        equipment.persistent_copy_uuid == Some(*copy_uuid)
                            && self
                                .items
                                .get(&equipment.runtime_item_instance_id)
                                .is_some_and(|item| {
                                    item.owner_unit_instance == actor
                                        && item.source_snapshot() == equipment
                                })
                    })
            }
        })?;
        plan.execution.get(index)
    }
    pub(super) fn current_skill_view<'a>(
        &'a self,
        id: &SkillId,
        actor: UnitInstanceId,
        source: &RuntimeFactSource,
    ) -> Option<&'a SkillDef> {
        if let Some(plan) = self.current_effective_skill(id, actor, source) {
            return Some(plan.skill.as_ref());
        }
        self.game_data.skill_data.get_by_id(id)
    }
    pub(super) fn effective_effect_view<'a>(
        &'a self,
        site: &EffectSite,
        actor: UnitInstanceId,
        source: &'a RuntimeFactSource,
    ) -> Result<Option<&'a EffectiveEffect>, GameError> {
        if let Some(binding) = source.relic_effective() {
            crate::authoring::control::charge(
                binding.execution().effects.len() as u64,
                "effective_effect_view",
            )?;
            return Ok(binding.effect(site));
        }
        let Some(execution) =
            self.current_effective_skill(&site.action.definition_id, actor, source)
        else {
            crate::game::battle::runtime_control::poll()?;
            return Ok(None);
        };
        let unit = self.units.get(&actor).ok_or(GameError::UnitNotFound)?;
        let plan = &self
            .effective_owner
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .plans[&unit.source_owned_uuid];
        crate::authoring::control::charge(execution.effects.len() as u64, "effective_effect_view")?;
        Ok(execution
            .effects
            .iter()
            .find(|(bound, _)| bound == site)
            .map(|(_, index)| &plan.effects[*index]))
    }
    pub(super) fn cloned_current_skill(
        &self,
        id: &SkillId,
        actor: UnitInstanceId,
        source: &RuntimeFactSource,
    ) -> Result<Option<SkillDef>, GameError> {
        let skill = self.current_skill_view(id, actor, source);
        crate::game::battle::runtime_control::poll()?;
        let Some(skill) = skill else { return Ok(None) };
        crate::authoring::control::charge_serialized(skill, "effective_current_skill_copy")?;
        Ok(Some(skill.clone()))
    }
    pub(super) fn requires_explicit_structural_selection(
        &self,
        skill: &SkillDef,
        actor: UnitInstanceId,
        source: &RuntimeFactSource,
    ) -> Result<bool, GameError> {
        let selected = source
            .relic_effective()
            .map(|binding| binding.execution())
            .or_else(|| self.current_effective_skill(&skill.id, actor, source));
        crate::game::battle::runtime_control::poll()?;
        if selected.is_some_and(|plan| plan.selected_recipient_reach.is_some()) {
            return Ok(true);
        }
        // This declaration is new. Existing PlayerUnit skills retain their
        // historical selection path when they have no typed personal grant.
        if skill.cast_selection() != crate::game::ability::SkillCastSelection::PlayerUnit {
            return Ok(false);
        }
        for step in &skill.steps {
            crate::authoring::control::charge(1, "structural_explicit_selection_step")?;
            for effect in &step.effects {
                crate::authoring::control::charge(1, "structural_explicit_selection_effect")?;
                if matches!(&effect.op, crate::game::ability::SkillEffectDef::PersonalShieldGrant { definition }
                    if matches!(definition.recipients, crate::game::relic::structural::authoring::PersonalShieldRecipientsDef::SelectedPlayerEmployeeAtRelease { .. }))
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    pub(crate) fn current_skill_cast_selection(
        &self,
        id: &SkillId,
        actor: UnitInstanceId,
    ) -> Result<Option<crate::game::ability::SkillCastSelection>, GameError> {
        let selection = self
            .current_skill_view(id, actor, &RuntimeFactSource::default())
            .map(SkillDef::cast_selection);
        crate::game::battle::runtime_control::poll()?;
        Ok(selection)
    }
    pub(super) fn pending_skill_view<'a>(
        &'a self,
        pending: &super::types::PendingSkillCast,
        actor: UnitInstanceId,
    ) -> Option<&'a SkillDef> {
        if let Some(action) = &pending.action {
            let source = self.runtime_facts.accepted_source(action).ok()?;
            return self.skill_for_source(&pending.skill_id, source);
        }
        self.current_skill_view(&pending.skill_id, actor, &self.runtime_fact_source)
    }
    pub(super) fn selected_player_employee_in_reach(
        &self,
        caster: UnitInstanceId,
        recipient: UnitInstanceId,
        reach: u32,
    ) -> bool {
        use crate::game::enums::Side;
        let Some(source) = self.units.get(&caster) else {
            return false;
        };
        let Some(target) = self.units.get(&recipient) else {
            return false;
        };
        source.owner == Side::Player
            && target.owner == Side::Player
            && source.is_active()
            && source.is_combatant()
            && source.employee_trauma.is_some()
            && matches!(
                source.source_identity,
                BattleUnitSourceIdentity::Employee { .. }
            )
            && target.is_active()
            && target.is_combatant()
            && target.employee_trauma.is_some()
            && matches!(
                target.source_identity,
                BattleUnitSourceIdentity::Employee { .. }
            )
            && {
                let dx = f64::from(source.body.position.x - target.body.position.x);
                let dy = f64::from(source.body.position.y - target.body.position.y);
                let radius =
                    f64::from(reach) / f64::from(super::movement::types::DATA_UNITS_PER_WORLD);
                dx * dx + dy * dy <= radius * radius
            }
    }
    pub(super) fn bind_effective_skill_source(
        &self,
        id: &SkillId,
        actor: UnitInstanceId,
        source: &RuntimeFactSource,
    ) -> Result<RuntimeFactSource, GameError> {
        if let Some(bound) = source.relic_effective() {
            bound.validate(
                self.content_identity
                    .as_deref()
                    .ok_or(GameError::InvalidAction)?,
                source,
            )?;
            if bound.execution().skill.id != *id {
                return Err(GameError::InvalidStaticData(
                    "effective skill source differs".into(),
                ));
            }
            return Ok(source.clone());
        }
        let Some(owner) = &self.effective_owner else {
            return Ok(source.clone());
        };
        let unit = self.units.get(&actor).ok_or(GameError::UnitNotFound)?;
        if !matches!(
            &unit.source_identity,
            BattleUnitSourceIdentity::Employee { .. }
        ) {
            return Ok(source.clone());
        }
        let Some(plan) = owner.plans.get(&unit.source_owned_uuid) else {
            return Ok(source.clone());
        };
        crate::authoring::control::charge(
            plan.providers.len() as u64,
            "effective_provider_selection",
        )?;
        let provider = plan.providers.iter().position(|provider| match provider {
            ProspectiveProvider::Fragment(fragment) => {
                source.equipment.is_none()
                    && fragment.skill_id == *id
                    && source
                        .fragment
                        .as_deref()
                        .or(unit.fragment_source.as_deref())
                        == Some(fragment)
            }
            ProspectiveProvider::Equipment {
                definition_id,
                copy_uuid,
                skill_id,
            } => {
                skill_id == id
                    && source.equipment.is_some_and(|equipment| {
                        equipment.persistent_copy_uuid == Some(*copy_uuid)
                            && self
                                .items
                                .get(&equipment.runtime_item_instance_id)
                                .is_some_and(|item| {
                                    item.owner_unit_instance == actor
                                        && item.source_snapshot() == equipment
                                        && self
                                            .game_data
                                            .equipment_data
                                            .get_by_uuid(&item.base_uuid)
                                            .is_some_and(|d| &d.id == definition_id)
                                })
                    })
            }
        });
        let Some(provider) = provider else {
            return Ok(source.clone());
        };
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(std::mem::size_of::<EffectiveSkillBinding>() + 64),
            "effective_source_binding",
        )?;
        let mut source = source.clone();
        source.actor = Some(actor);
        if source.equipment.is_none() && source.fragment.is_none() {
            source.fragment = unit.fragment_source.clone();
        }
        source.with_relic_effective(Arc::new(EffectiveSkillBinding {
            plan: Arc::clone(plan),
            provider,
            actor,
        }))
    }
    pub(super) fn skill_for_source<'a>(
        &'a self,
        id: &SkillId,
        source: &'a RuntimeFactSource,
    ) -> Option<&'a SkillDef> {
        if let Some(binding) = source.relic_effective() {
            return (binding.execution().skill.id == *id)
                .then_some(binding.execution().skill.as_ref());
        }
        self.game_data.skill_data.get_by_id(id)
    }
    pub(super) fn cloned_skill_for_source(
        &self,
        id: &SkillId,
        source: &RuntimeFactSource,
    ) -> Result<Option<SkillDef>, GameError> {
        let Some(skill) = self.skill_for_source(id, source) else {
            return Ok(None);
        };
        crate::authoring::control::charge_serialized(skill, "effective_skill_definition_copy")?;
        Ok(Some(skill.clone()))
    }
    pub(super) fn amount_entry_for_source<'a>(
        &'a self,
        source: &'a RuntimeFactSource,
        site: &EffectSite,
    ) -> Option<&'a CompiledAmountEntry> {
        if let Some(binding) = source.relic_effective() {
            return binding.execution().amounts.get(site);
        }
        self.game_data.value_expr_catalog().get(site)
    }
}
