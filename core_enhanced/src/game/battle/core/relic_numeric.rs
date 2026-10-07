//! Private immutable battle overlay. Published base catalog authority stays in
//! value_expr_runtime; release bindings carry no replacement compiled entry.
use super::{value_expr_runtime::RuntimeAmount, BattleCore};
use crate::game::{
    ability::{authoring::AuthoredSkillEffect, SkillEffectDef},
    battle::{
        damage::DamageType,
        ids::UnitInstanceId,
        runtime_facts::{ActionOrdinal, EquipmentSourceSnapshot, RuntimeFactSource},
        types::{BattleUnitDraft, BattleUnitSource, BattleUnitSourceIdentity},
    },
    behavior::GameError,
    data::{
        content_pack::ContentIdentity,
        relic_data::{RelicDamageTarget, RelicPenalty},
        GameDataBase,
    },
    relic::{
        numeric::{
            compose_damage, contribution_totals, RelicDamageContribution, RelicNumericFaultCode,
            RelicNumericTarget,
        },
        RelicInventory, RelicVariant,
    },
    value_expr::{
        AmountPlanKind, CompiledAmountPlan, ValueExprFault, ValueExprFaultCode, ValueExprPhase,
    },
};
use serde::Serialize;
use std::{mem::size_of, sync::Arc};
use uuid::Uuid;

/// Borrowed conditions shared by accepted runtime sources and prospective rows.
/// No action, actor, epoch or private binding can be created through this view.
pub(crate) struct RelicSelectorContext<'a> {
    pub(crate) target: RelicDamageTarget,
    pub(crate) tags: &'a [String],
    pub(crate) damage_type: DamageType,
    pub(crate) equipment: Option<Uuid>,
    pub(crate) fragment: Option<&'a crate::game::skill_fragment::FragmentSourceSnapshot>,
}

pub(crate) fn selector_state(
    data: &GameDataBase,
    selector: &crate::game::data::relic_data::RelicDamageSelector,
    context: &RelicSelectorContext<'_>,
) -> Result<crate::game::relic::plan::RelicNumericPlanEffectState, GameError> {
    use crate::game::relic::plan::{
        RelicNumericPlanDormantReason as Reason, RelicNumericPlanEffectState as State,
    };
    let dormant = |reason| Ok(State::Dormant { reason });
    if selector.target != RelicDamageTarget::AnyDamage && selector.target != context.target {
        return dormant(Reason::TargetKind);
    }
    if selector
        .damage_type
        .is_some_and(|wanted| wanted != context.damage_type)
    {
        return dormant(Reason::DamageType);
    }
    let mut tags_match = true;
    for wanted in &selector.all_tags {
        crate::authoring::control::charge(context.tags.len() as u64, "relic_numeric_tag")?;
        tags_match &= context.tags.iter().any(|actual| actual == wanted);
    }
    if !tags_match {
        return dormant(Reason::Tags);
    }
    if let Some(wanted) = &selector.equipment_id {
        crate::authoring::control::charge(
            data.equipment_data.items.len() as u64,
            "relic_numeric_equipment",
        )?;
        if !context
            .equipment
            .and_then(|id| data.equipment_data.get_by_uuid(&id))
            .is_some_and(|actual| &actual.id == wanted)
        {
            return dormant(Reason::Equipment);
        }
    }
    if let Some(wanted) = &selector.fragment_id {
        if !context
            .fragment
            .is_some_and(|actual| actual.definition_id.as_str() == wanted)
        {
            return dormant(Reason::Fragment);
        }
    }
    if let Some(wanted) = &selector.form_id {
        if !context
            .fragment
            .is_some_and(|actual| &actual.form_id == wanted)
        {
            return dormant(Reason::Form);
        }
    }
    Ok(State::Active)
}

pub(crate) fn penalize_employee_stats(
    inventory: &RelicInventory,
    data: &GameDataBase,
    employee: Uuid,
    mut stats: crate::game::stats::UnitStats,
) -> Result<crate::game::stats::UnitStats, GameError> {
    let mut defense = i128::from(stats.defense);
    let mut resistance = i128::from(stats.magic_resist);
    for copy in inventory.employee_slots(employee).into_iter().flatten() {
        crate::authoring::control::charge(1, "relic_penalty_slot")?;
        let copy = inventory
            .get(copy)
            .ok_or_else(|| invalid("relic penalty missing copy"))?;
        if copy.variant != RelicVariant::Corrupt {
            continue;
        }
        crate::authoring::control::charge(
            data.relic_data.relics.len() as u64,
            "relic_penalty_definition",
        )?;
        let variant = data
            .relic_data
            .get_by_id(&copy.definition_id)
            .and_then(|definition| definition.corruption.as_ref())
            .ok_or_else(|| invalid("relic penalty missing variant"))?;
        for penalty in &variant.penalties {
            crate::authoring::control::charge(1, "relic_penalty")?;
            match penalty {
                RelicPenalty::DefenseFlat { amount } => {
                    defense = defense
                        .checked_add(i128::from(*amount))
                        .ok_or_else(|| invalid("relic defense overflow"))?
                }
                RelicPenalty::MagicResistFlat { amount } => {
                    resistance = resistance
                        .checked_add(i128::from(*amount))
                        .ok_or_else(|| invalid("relic resistance overflow"))?
                }
            }
        }
    }
    stats.defense = i32::try_from(defense).map_err(|_| invalid("relic defense overflow"))?;
    stats.magic_resist =
        i32::try_from(resistance).map_err(|_| invalid("relic resistance overflow"))?;
    Ok(stats)
}

fn invalid(cause: &str) -> GameError {
    GameError::InvalidStaticData(cause.into())
}

pub(crate) struct RelicNumericOwner {
    inventory: RelicInventory,
    data: Arc<GameDataBase>,
    generation: Arc<ContentIdentity>,
    extent: usize,
}
impl std::fmt::Debug for RelicNumericOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelicNumericOwner")
            .field("inventory", &self.inventory)
            .field("generation", &self.generation)
            .field("extent", &self.extent)
            .finish_non_exhaustive()
    }
}
impl RelicNumericOwner {
    pub(crate) fn capture(
        inventory: &RelicInventory,
        data: Arc<GameDataBase>,
        generation: Arc<ContentIdentity>,
    ) -> Result<Option<Arc<Self>>, GameError> {
        if !inventory.has_equipped() {
            return Ok(None);
        }
        inventory.preflight_clone()?;
        let extent = inventory
            .retained_extent()?
            .checked_add(size_of::<Self>() + 64)
            .and_then(|n| n.checked_add(size_of::<ContentIdentity>() + 64))
            .and_then(|n| n.checked_add(generation.semantic_sha256.len()))
            .ok_or_else(|| invalid("relic numeric owner extent overflow"))?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(size_of::<Self>() + 64),
            "relic_numeric_owner",
        )?;
        Ok(Some(Arc::new(Self {
            inventory: crate::authoring::clone_shape::controlled_clone(inventory)?,
            data,
            generation,
            extent,
        })))
    }
    pub(crate) fn retained_extent(&self) -> usize {
        self.extent
    }
    fn selected(
        &self,
        employee: Uuid,
        target: &RelicNumericTarget,
        tags: &[String],
        damage_type: DamageType,
        source: &RuntimeFactSource,
    ) -> Result<Vec<RelicDamageContribution>, GameError> {
        let slots = self.inventory.employee_slots(employee);
        let mut bound = 0usize;
        let mut count = 0usize;
        // Admit all potentially selected rows before any ID cloning. Selection
        // and canonical sort each consume their actual authored input lengths.
        for copy in slots.into_iter().flatten() {
            crate::authoring::control::charge(1, "relic_numeric_slot")?;
            let copy = self
                .inventory
                .get(copy)
                .ok_or_else(|| invalid("relic numeric missing copy"))?;
            crate::authoring::control::charge(
                self.data.relic_data.relics.len() as u64,
                "relic_numeric_definition",
            )?;
            let definition = self
                .data
                .relic_data
                .get_by_id(&copy.definition_id)
                .ok_or_else(|| invalid("relic numeric missing definition"))?;
            let effects = match copy.variant {
                RelicVariant::Normal => &definition.effects,
                RelicVariant::Corrupt => {
                    &definition
                        .corruption
                        .as_ref()
                        .ok_or_else(|| invalid("relic numeric missing variant"))?
                        .effects
                }
            };
            for effect in effects {
                crate::authoring::control::charge(1, "relic_numeric_effect")?;
                count = count
                    .checked_add(1)
                    .ok_or_else(|| invalid("relic numeric count overflow"))?;
                bound = bound
                    .checked_add(size_of::<RelicDamageContribution>())
                    .and_then(|n| n.checked_add(copy.definition_id.len()))
                    .and_then(|n| n.checked_add(effect.id.len()))
                    .ok_or_else(|| invalid("relic numeric contribution extent overflow"))?;
            }
        }
        if count == 0 {
            return Ok(Vec::new());
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(bound),
            "relic_numeric_contributions",
        )?;
        let mut values = Vec::with_capacity(count);
        for copy in slots.into_iter().flatten() {
            let copy = self
                .inventory
                .get(copy)
                .ok_or_else(|| invalid("relic numeric missing copy"))?;
            crate::authoring::control::charge(
                self.data.relic_data.relics.len() as u64,
                "relic_numeric_definition",
            )?;
            let definition = self
                .data
                .relic_data
                .get_by_id(&copy.definition_id)
                .ok_or_else(|| invalid("relic numeric missing definition"))?;
            let effects = match copy.variant {
                RelicVariant::Normal => &definition.effects,
                RelicVariant::Corrupt => {
                    &definition
                        .corruption
                        .as_ref()
                        .ok_or_else(|| invalid("relic numeric missing variant"))?
                        .effects
                }
            };
            for effect in effects {
                crate::authoring::control::charge(1, "relic_numeric_select")?;
                let context = RelicSelectorContext {
                    target: match target {
                        RelicNumericTarget::SkillDamage { .. } => RelicDamageTarget::SkillDamage,
                        RelicNumericTarget::BasicAttackDamage { .. } => {
                            RelicDamageTarget::BasicAttackDamage
                        }
                    },
                    tags,
                    damage_type,
                    equipment: source.equipment.map(|provider| provider.definition_uuid),
                    fragment: source.fragment.as_deref(),
                };
                if selector_state(&self.data, &effect.selector, &context)?
                    != crate::game::relic::plan::RelicNumericPlanEffectState::Active
                {
                    continue;
                }
                if !crate::authoring::control::is_active() {
                    crate::game::resource::store::preflight_resource_clone(&copy.definition_id)
                        .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                    crate::game::resource::store::preflight_resource_clone(&effect.id)
                        .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                }
                values.push(RelicDamageContribution {
                    copy_uuid: copy.copy_uuid,
                    definition_id: crate::authoring::clone_shape::controlled_clone(
                        &copy.definition_id,
                    )?,
                    variant: copy.variant,
                    effect_id: crate::authoring::clone_shape::controlled_clone(&effect.id)?,
                    flat: effect.flat,
                    percent: effect.percent,
                });
            }
        }
        let sort_work = count
            .checked_mul((usize::BITS - count.leading_zeros()) as usize + 1)
            .ok_or_else(|| invalid("relic numeric sort overflow"))?;
        crate::authoring::control::charge(sort_work as u64, "relic_numeric_sort")?;
        values
            .sort_unstable_by(|a, b| (a.copy_uuid, &a.effect_id).cmp(&(b.copy_uuid, &b.effect_id)));
        Ok(values)
    }
    pub(crate) fn penalized_stats(
        &self,
        draft: &BattleUnitDraft,
        stats: crate::game::stats::UnitStats,
    ) -> Result<crate::game::stats::UnitStats, GameError> {
        if !matches!(draft.source, BattleUnitSource::Employee(_)) {
            return Ok(stats);
        }
        penalize_employee_stats(&self.inventory, &self.data, draft.owned_uuid, stats)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RelicNumericBinding {
    pub(crate) generation: Arc<ContentIdentity>,
    pub(crate) employee_uuid: Uuid,
    pub(crate) actor: UnitInstanceId,
    pub(crate) target: RelicNumericTarget,
    pub(crate) action_ordinal: Option<ActionOrdinal>,
    pub(crate) equipment: Option<EquipmentSourceSnapshot>,
    pub(crate) fragment: Option<Arc<crate::game::skill_fragment::FragmentSourceSnapshot>>,
    pub(crate) damage_type: DamageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) basic_profile_fingerprint: Option<[u8; 32]>,
    pub(crate) contributions: Vec<RelicDamageContribution>,
}
impl RelicNumericBinding {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(64)?
            .checked_add(size_of::<ContentIdentity>() + 64)?
            .checked_add(self.generation.semantic_sha256.len())?
            .checked_add(
                self.contributions
                    .capacity()
                    .checked_mul(size_of::<RelicDamageContribution>())?,
            )?;
        for row in &self.contributions {
            bytes = bytes
                .checked_add(row.definition_id.capacity())?
                .checked_add(row.effect_id.capacity())?;
        }
        if let RelicNumericTarget::SkillDamage { site } = &self.target {
            bytes = bytes
                .checked_add(site.action.definition_id.as_str().len())?
                .checked_add(site.action.action_id.capacity())?
                .checked_add(site.effect_id.capacity())?;
        }
        if let Some(fragment) = &self.fragment {
            bytes = bytes.checked_add(fragment.retained_extent()?.checked_add(64)?)?;
        }
        Some(bytes)
    }
    pub(crate) fn apply(&self, base: u32) -> Result<u32, RelicNumericFaultCode> {
        let (flat, percent) = contribution_totals(&self.contributions)?;
        compose_damage(base, flat, percent)
    }
    pub(crate) fn apply_amount(
        &self,
        plan: &CompiledAmountPlan,
        base: RuntimeAmount,
    ) -> Result<RuntimeAmount, ValueExprFault> {
        let RuntimeAmount::Damage(base) = base else {
            return Err(self.fault(
                plan,
                ValueExprFaultCode::Unavailable,
                "relic binding requires Damage capability",
            ));
        };
        self.apply(base).map(RuntimeAmount::Damage).map_err(|code| {
            self.fault(
                plan,
                match code {
                    RelicNumericFaultCode::Overflow => ValueExprFaultCode::Overflow,
                    RelicNumericFaultCode::InvalidCoefficient => ValueExprFaultCode::Unavailable,
                },
                "relic damage composition or final narrow failed",
            )
        })
    }
    fn fault(
        &self,
        plan: &CompiledAmountPlan,
        code: ValueExprFaultCode,
        cause: &str,
    ) -> ValueExprFault {
        ValueExprFault {
            code,
            site: plan.source().site.clone(),
            node_index: 0,
            expected_unit: None,
            actual_unit: None,
            phase: ValueExprPhase::Narrow,
            cause: cause.into(),
        }
    }
    pub(crate) fn validate_skill(
        &self,
        generation: &ContentIdentity,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
    ) -> Result<(), ValueExprFault> {
        if &*self.generation != generation
            || !matches!(&self.target, RelicNumericTarget::SkillDamage { site } if site == &plan.source().site)
            || source.actor != Some(self.actor)
            || self.action_ordinal != source.action_ordinal()
            || self.equipment != source.equipment
            || self.fragment != source.fragment
        {
            return Err(self.fault(plan, ValueExprFaultCode::Unavailable, "relic numeric binding differs from its immutable base generation/site/action/provider"));
        }
        Ok(())
    }
}

impl BattleCore {
    pub(crate) fn install_relic_numeric_owner(
        &mut self,
        owner: Option<Arc<RelicNumericOwner>>,
    ) -> Result<(), GameError> {
        if let Some(owner) = &owner {
            if !Arc::ptr_eq(&owner.data, &self.game_data)
                || self.content_identity.as_deref() != Some(owner.generation.as_ref())
            {
                return Err(invalid("relic numeric owner generation mismatch"));
            }
        }
        self.relic_numeric_owner = owner;
        Ok(())
    }
    pub(super) fn relic_penalized_stats(
        &self,
        draft: &BattleUnitDraft,
    ) -> Result<crate::game::stats::UnitStats, GameError> {
        let stats = draft.effective_stats(&self.game_data)?;
        match &self.relic_numeric_owner {
            Some(owner) => owner.penalized_stats(draft, stats),
            None => Ok(stats),
        }
    }
    pub(super) fn source_has_relics(&self, source: &RuntimeFactSource) -> bool {
        if source.relic_effective().is_some() {
            return true;
        }
        let Some(owner) = &self.relic_numeric_owner else {
            return false;
        };
        let Some(unit) = source.actor.and_then(|actor| self.units.get(&actor)) else {
            return false;
        };
        let BattleUnitSourceIdentity::Employee { employee_uuid, .. } = unit.source_identity else {
            return false;
        };
        owner
            .inventory
            .employee_slots(employee_uuid)
            .iter()
            .any(Option::is_some)
    }

    pub(super) fn bind_skill_relic_source(
        &self,
        kind: AmountPlanKind,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
    ) -> Result<RuntimeFactSource, GameError> {
        if kind != AmountPlanKind::Damage || self.relic_numeric_owner.is_none() {
            return Ok(source.clone());
        }
        let owner = self.relic_numeric_owner.as_ref().unwrap();
        let Some(actor) = source.actor else {
            return Ok(source.clone());
        };
        let Some(unit) = self.units.get(&actor) else {
            return Ok(source.clone());
        };
        let BattleUnitSourceIdentity::Employee { employee_uuid, .. } = unit.source_identity else {
            return Ok(source.clone());
        };
        // Resolve metadata from the exact already-validated original source.
        let skill = self
            .game_data
            .skill_data
            .get_by_id(plan.source().site.action.definition_id.as_str())
            .ok_or_else(|| invalid("relic numeric skill missing"))?;
        crate::authoring::control::charge(
            skill.steps.len() as u64,
            "relic_numeric_action_metadata",
        )?;
        let step = skill
            .steps
            .iter()
            .find(|step| step.id == plan.source().site.action.action_id)
            .ok_or_else(|| invalid("relic numeric action missing"))?;
        crate::authoring::control::charge(
            step.effects.len() as u64,
            "relic_numeric_effect_metadata",
        )?;
        let effect: &AuthoredSkillEffect = step
            .effects
            .iter()
            .find(|effect| {
                effect
                    .source()
                    .is_some_and(|authored| authored.as_ref() == plan.source())
            })
            .ok_or_else(|| invalid("relic numeric original effect missing"))?;
        let damage_type = match effect.op {
            SkillEffectDef::Damage { damage_type, .. }
            | SkillEffectDef::DamageExpr { damage_type, .. } => damage_type,
            _ => return Err(invalid("relic numeric non-damage metadata")),
        };
        let site_bytes = plan
            .source()
            .site
            .action
            .definition_id
            .as_str()
            .len()
            .checked_add(plan.source().site.action.action_id.len())
            .and_then(|n| n.checked_add(plan.source().site.effect_id.len()))
            .and_then(|n| n.checked_add(size_of::<RelicNumericBinding>() + 64))
            .ok_or_else(|| invalid("relic numeric binding extent overflow"))?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(site_bytes),
            "relic_numeric_binding",
        )?;
        let target = RelicNumericTarget::SkillDamage {
            site: plan.source().site.clone(),
        };
        let contributions =
            owner.selected(employee_uuid, &target, &effect.tags, damage_type, source)?;
        if contributions.is_empty() {
            return Ok(source.clone());
        }
        let binding = RelicNumericBinding {
            generation: owner.generation.clone(),
            employee_uuid,
            actor,
            target,
            action_ordinal: source.action_ordinal(),
            equipment: source.equipment,
            fragment: source.fragment.clone(),
            damage_type,
            basic_profile_fingerprint: None,
            contributions,
        };
        source.with_relic_numeric(Arc::new(binding))
    }
    pub(super) fn bind_basic_relic_source(
        &self,
        actor: UnitInstanceId,
        damage_type: DamageType,
        base: u32,
        source: &RuntimeFactSource,
    ) -> Result<(u32, RuntimeFactSource), GameError> {
        let Some(owner) = &self.relic_numeric_owner else {
            return Ok((base, source.clone()));
        };
        let Some(unit) = self.units.get(&actor) else {
            return Ok((base, source.clone()));
        };
        if !unit.basic_attack.action.is_damage() {
            return Ok((base, source.clone()));
        }
        let BattleUnitSourceIdentity::Employee { employee_uuid, .. } = unit.source_identity else {
            return Ok((base, source.clone()));
        };
        if owner
            .inventory
            .employee_slots(employee_uuid)
            .iter()
            .all(Option::is_none)
        {
            return Ok((base, source.clone()));
        }
        let mut equipment = None;
        if let Some(slots) = &unit.equipment_slots {
            for slot in slots.slots.iter().flatten() {
                crate::authoring::control::charge(
                    self.game_data.equipment_data.items.len() as u64,
                    "relic_basic_weapon",
                )?;
                if self
                    .game_data
                    .equipment_data
                    .get_by_uuid(&slot.binding.base_uuid)
                    .is_some_and(|item| {
                        item.equipment_type
                            == crate::game::data::equipment_data::EquipmentType::Weapon
                    })
                {
                    let item = self
                        .items
                        .get(&slot.runtime_item_instance_id)
                        .ok_or_else(|| invalid("relic basic weapon copy unavailable"))?;
                    if item.owner_unit_instance != actor
                        || item.base_uuid != slot.binding.base_uuid
                        || item.persistent_copy_uuid != slot.binding.persistent_copy_uuid
                    {
                        return Err(invalid("relic basic weapon binding mismatch"));
                    }
                    equipment = Some(item.source_snapshot());
                    break;
                }
            }
        }
        // The real weapon selects this producer. Original trigger/action
        // provenance remains in source; it is not overwritten by this provider.
        let provider_source = RuntimeFactSource {
            equipment,
            fragment: None,
            ..source.clone()
        };
        let target = RelicNumericTarget::BasicAttackDamage {
            actor,
            attack_epoch: unit.basic_attack_epoch,
        };
        let contributions =
            owner.selected(employee_uuid, &target, &[], damage_type, &provider_source)?;
        if contributions.is_empty() {
            return Ok((base, source.clone()));
        }
        crate::authoring::control::charge(contributions.len() as u64, "relic_basic_compose")?;
        // Hash the actual immutable profile without materializing a JSON value,
        // String or encoded buffer. The fingerprint is observation, not a grant.
        struct ProfileHash(sha2::Sha256);
        impl std::io::Write for ProfileHash {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                use sha2::Digest;
                crate::authoring::control::charge(bytes.len() as u64, "relic_basic_profile")
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;
                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        use sha2::Digest;
        let mut profile = ProfileHash(sha2::Sha256::new());
        let encoded = serde_json::to_writer(&mut profile, &unit.basic_attack);
        crate::authoring::control::poll("relic_basic_profile")?;
        encoded.map_err(|_| invalid("relic basic profile encoding failed"))?;
        let binding = RelicNumericBinding {
            generation: owner.generation.clone(),
            employee_uuid,
            actor,
            target,
            action_ordinal: source.action_ordinal(),
            equipment,
            fragment: None,
            damage_type,
            basic_profile_fingerprint: Some(profile.0.finalize().into()),
            contributions,
        };
        let effective = binding.apply(base).map_err(|_| {
            crate::game::battle::runtime_control::reject(
                crate::game::battle::runtime_control::BattleRuntimeFault {
                    code: crate::game::battle::runtime_control::RuntimeFaultCode::CounterOverflow,
                    site: "relic_basic_damage_narrow",
                    limit: Some(u64::from(u32::MAX)),
                    requested: None,
                },
            )
        })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(size_of::<RelicNumericBinding>() + 64),
            "relic_basic_binding",
        )?;
        Ok((effective, source.with_relic_numeric(Arc::new(binding))?))
    }
}
