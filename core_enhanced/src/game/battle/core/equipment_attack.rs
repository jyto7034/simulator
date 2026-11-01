use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{equipment_attack::*, state_program::StateKillEligibility},
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
        },
        behavior::GameError,
        value_expr::{
            self as expr, EvalFrame, ExprMeter, ResourceValueField, TypedInteger, ValueRef,
        },
    },
};
use std::{mem::size_of, sync::Arc};

#[derive(Debug, Clone, serde::Serialize)]
pub struct EquipmentSpeedContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub points: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(EquipmentSpeedContribution {
    equipment, points, definition_id, source_file, field_path
});
impl EquipmentSpeedContribution {
    fn dynamic_bytes(&self) -> Option<usize> {
        self.definition_id.capacity().checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())
    }
}

/// One successful regular attack's immutable timing inputs, not an active buff.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EquipmentSpeedWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub current_hp: u32,
    pub max_hp: u32,
    pub base_interval_ms: u64,
    pub final_interval_ms: u64,
    pub equipment_points: u32,
    pub contributors: Vec<EquipmentSpeedContribution>,
}
crate::authoring::clone_shape::struct_shape!(EquipmentSpeedWitness {
    generation, time_ms, actor, current_hp, max_hp, base_interval_ms,
    final_interval_ms, equipment_points, contributors
});
impl EquipmentSpeedWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.contributors.capacity().checked_mul(size_of::<EquipmentSpeedContribution>())?)?;
        if let Some(generation) = &self.generation {
            bytes = bytes.checked_add(std::mem::size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        for contribution in &self.contributors { bytes = bytes.checked_add(contribution.dynamic_bytes()?)?; }
        Some(bytes)
    }
}

pub(super) struct EquipmentSpeedEvaluation {
    pub points: u32,
    pub witness: Option<EquipmentSpeedWitness>,
}

/// Immutable admission snapshot, shared by every hit and released carrier of
/// one normal basic action. A primary victim can finalize death only once.
#[derive(Debug)]
pub(crate) struct EquipmentAttackCapture {
    pub primary: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub weapon_stage: crate::game::ability::weapon_marks::WeaponStage,
    pub program: Arc<CompiledEquipmentAttack>,
    pub bonus: u128,
    pub relic_copies: Vec<Option<uuid::Uuid>>,
}
impl PartialEq for EquipmentAttackCapture {
    fn eq(&self, other: &Self) -> bool {
        self.primary == other.primary
            && self.equipment == other.equipment
            && self.weapon_stage == other.weapon_stage
            && self.program.fingerprint == other.program.fingerprint
            && self.bonus == other.bonus
            && self.relic_copies == other.relic_copies
    }
}
impl Eq for EquipmentAttackCapture {}
impl EquipmentAttackCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        (size_of::<Self>() + 64)
            .checked_add(self.program.retained_bytes)?
            .checked_add(
                self.relic_copies
                    .capacity()
                    .checked_mul(size_of::<Option<uuid::Uuid>>())?,
            )
    }
}

/// All equipment hit-status contributors admitted with one normal attack.
/// Released carriers share this immutable list; weapon damage remains separate.
#[derive(Debug)]
pub(crate) struct EquipmentHitCapture {
    pub providers: Vec<EquipmentHitProvider>,
}
#[derive(Debug)]
pub(crate) struct EquipmentHitProvider {
    pub equipment: EquipmentSourceSnapshot,
    pub program: Arc<CompiledEquipmentAttack>,
}
impl PartialEq for EquipmentHitCapture {
    fn eq(&self, other: &Self) -> bool {
        self.providers.len() == other.providers.len()
            && self.providers.iter().zip(&other.providers).all(|(a, b)|
                a.equipment == b.equipment && a.program.fingerprint == b.program.fingerprint)
    }
}
impl Eq for EquipmentHitCapture {}
impl EquipmentHitCapture {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = (size_of::<Self>() + 64)
            .checked_add(self.providers.capacity().checked_mul(size_of::<EquipmentHitProvider>())?)?;
        for provider in &self.providers {
            bytes = bytes.checked_add(provider.program.retained_bytes)?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    /// A derived contribution, evaluated at admission rather than maintained as
    /// a second HP-dependent state. Equipment slots remain the authority.
    pub(super) fn equipment_basic_speed(&self, actor: UnitInstanceId, time_ms: u64, capture: bool)
        -> Result<EquipmentSpeedEvaluation, GameError> {
        let empty = || EquipmentSpeedEvaluation { points: 0, witness: None };
        let catalog = self.game_data.equipment_attack_catalog();
        if catalog.by_equipment.is_empty() { return Ok(empty()); }
        let Some(unit) = self.units.get(&actor).filter(|u| !u.corroded && u.is_active()) else {
            return Ok(empty());
        };
        let slots = unit.equipment_slots.as_ref();
        control::charge(if slots.is_some() { 3 } else { self.items.len() } as u64,
            "equipment_basic_speed_providers")?;
        let slot_items = slots.into_iter().flat_map(|s| s.slots.iter().flatten())
            .map(|s| self.items.get(&s.runtime_item_instance_id)
                .ok_or_else(|| runtime_control::invariant("equipment_basic_speed_slot")));
        let legacy_items = self.items.values().take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == actor).map(Ok);
        let mut total = 0u32;
        let mut contributors = Vec::new();
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != actor {
                return Err(runtime_control::invariant("equipment_basic_speed_owner"));
            }
            let Some(plan) = catalog.by_equipment.get(&item.base_uuid)
                .and_then(|p| p.basic_speed_points.as_ref()) else { continue };
            let refs = &plan.requirements().references;
            runtime_control::admit_retained_growth(0,
                refs.len().checked_mul(size_of::<(ValueRef, TypedInteger)>() + 128),
                "equipment_basic_speed_frame")?;
            let mut frame = EvalFrame::with_value_capacity(refs.len());
            for reference in refs {
                let ValueRef::HitPoints { subject: expr::ValueSubject::Source, field } = reference else {
                    return Err(runtime_control::invariant("equipment_basic_speed_reference"));
                };
                let value = match field {
                    expr::HitPointField::Current => unit.stats.current_health,
                    expr::HitPointField::Max => unit.stats.max_health,
                    expr::HitPointField::Missing => unit.stats.max_health.saturating_sub(unit.stats.current_health),
                };
                frame.insert(reference.clone(), TypedInteger::new(i128::from(value), reference.unit()))
                    .map_err(|_| runtime_control::invariant("equipment_basic_speed_frame"))?;
            }
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| runtime_control::invariant("equipment_basic_speed_meter"))?
                .with_host_admission(super::value_expr_runtime::evaluation_host_admission);
            let value = expr::evaluate(plan, &frame, &mut meter)
                .map_err(runtime_control::reject_expression)?.value;
            let fault = || runtime_control::reject_expression(expr::ValueExprFault {
                code: expr::ValueExprFaultCode::Overflow,
                site: plan.source().site.clone(), node_index: 0,
                expected_unit: Some(expr::ValueUnit::AttackSpeedPoints),
                actual_unit: Some(value.unit.clone()), phase: expr::ValueExprPhase::Narrow,
                cause: "equipment basic speed must be nonnegative and its sum fit u32".into(),
            });
            let points = u32::try_from(value.value).map_err(|_| fault())?;
            total = total.checked_add(points).ok_or_else(fault)?;
            if capture && self.authoring_observation.is_some() {
                let source = plan.source();
                let bytes = size_of::<EquipmentSpeedContribution>()
                    .checked_add(source.site.action.definition_id.as_str().len())
                    .and_then(|n| n.checked_add(source.source_file.len()))
                    .and_then(|n| n.checked_add(source.field_path.len()));
                runtime_control::admit_retained_growth(0, bytes, "equipment_speed_observation_copy")?;
                control::charge(bytes.unwrap_or(usize::MAX) as u64, "equipment_speed_observation_copy")?;
                contributors.reserve_exact(1);
                contributors.push(EquipmentSpeedContribution {
                    equipment: item.source_snapshot(), points,
                    definition_id: source.site.action.definition_id.to_string(),
                    source_file: source.source_file.clone(), field_path: source.field_path.clone(),
                });
            }
        }
        control::poll("equipment_basic_speed_evaluated")?;
        let witness = (!contributors.is_empty()).then(|| EquipmentSpeedWitness {
            generation: self.content_identity.clone(), time_ms, actor,
            current_hp: unit.stats.current_health, max_hp: unit.stats.max_health,
            base_interval_ms: unit.stats.attack_interval_ms.max(1), final_interval_ms: 0,
            equipment_points: total, contributors,
        });
        if let Some(witness) = &witness {
            runtime_control::admit_retained_growth(0, witness.retained_extent(), "equipment_speed_observation")?;
        }
        Ok(EquipmentSpeedEvaluation { points: total, witness })
    }

    pub(super) fn bind_equipment_attack(
        &mut self,
        snapshot: &mut DamageSourceSnapshot,
        primary: UnitInstanceId,
    ) -> Result<(), GameError> {
        let catalog = self.game_data.equipment_attack_catalog();
        if catalog.by_equipment.is_empty()
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || snapshot.fact_source.action.is_none()
        {
            return Ok(());
        }
        let Some(unit) = self.units.get(&snapshot.source_id).filter(|u| !u.corroded) else {
            return Ok(());
        };
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "equipment_attack_provider",
        )?;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or(GameError::InvalidAction)
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == snapshot.source_id)
            .map(Ok);
        let mut bound = None;
        let mut hits = EquipmentHitCapture { providers: Vec::new() };
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != snapshot.source_id {
                return Err(GameError::InvalidAction);
            }
            let Some(program) = catalog.by_equipment.get(&item.base_uuid) else {
                continue;
            };
            if program.on_hit_status.is_some() {
                runtime_control::admit_retained_growth(0,
                    hits.retained_extent().and_then(|n| n.checked_add(size_of::<EquipmentHitProvider>()))
                        .and_then(|n| n.checked_add(program.retained_bytes)),
                    "equipment_hit_capture")?;
                hits.providers.reserve_exact(1);
                hits.providers.push(EquipmentHitProvider {
                    equipment: item.source_snapshot(), program: Arc::clone(program),
                });
            }
            if program.bonus.is_none() && program.reactions.is_empty()
                && program.basic_lifesteal.is_none()
                && program.weapon_stages.is_none() { continue; }
            if bound.is_some() {
                return Err(GameError::InvalidAction);
            }
            bound = Some((item.source_snapshot(), item.weapon_stage, Arc::clone(program)));
        }
        if !hits.providers.is_empty() {
            control::charge((hits.providers.len() as u64).saturating_mul(u64::from(hits.providers.len().ilog2()) + 1), "equipment_hit_capture_sort")?;
            hits.providers.sort_by_key(|p| p.equipment.runtime_item_instance_id);
            snapshot.equipment_hits = Some(Arc::new(hits));
            control::poll("equipment_hits_bound")?;
        }
        let Some((equipment, weapon_stage, program)) = bound else {
            return Ok(());
        };
        runtime_control::admit_retained_growth(
            0,
            program
                .retained_bytes
                .checked_add(size_of::<EquipmentAttackCapture>() + 64)
                .and_then(|n| {
                    n.checked_add(program.reactions.len() * size_of::<Option<uuid::Uuid>>())
                }),
            "equipment_attack_capture",
        )?;
        self.prepare_runtime_resources()?;
        let mut source = snapshot.fact_source.clone();
        source.equipment = Some(equipment);
        let bonus = if let Some(plan) = &program.bonus {
            let refs = &plan.requirements().references;
            runtime_control::admit_retained_growth(
                0,
                refs.len()
                    .checked_mul(size_of::<(ValueRef, TypedInteger)>() + 128),
                "equipment_attack_frame",
            )?;
            let mut frame = EvalFrame::with_value_capacity(refs.len());
            for reference in refs {
                let ValueRef::Resource {
                    resource,
                    owner,
                    field,
                } = reference
                else {
                    return Err(GameError::InvalidAction);
                };
                let lease = self
                    .resolve_typed_resource_lease(resource, *owner, &source, None)
                    .map_err(|e| e.into_game_error())?;
                let value = self
                    .runtime_facts
                    .resources
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?
                    .store
                    .value(&lease)
                    .map_err(|e| e.into_game_error())?;
                let value = match field {
                    ResourceValueField::Balance => value.balance,
                    ResourceValueField::Reserved => value.reserved,
                    ResourceValueField::Available => value.available(),
                };
                frame
                    .insert(
                        reference.clone(),
                        TypedInteger::new(i128::from(value), reference.unit()),
                    )
                    .map_err(|_| GameError::InvalidAction)?;
            }
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| GameError::InvalidAction)?
                .with_host_admission(super::value_expr_runtime::evaluation_host_admission);
            let value = expr::evaluate(plan, &frame, &mut meter)
                .map_err(runtime_control::reject_expression)?
                .value;
            // Validate sign/type but retain the wide amount: clamping here
            // would underpay secondary damage for very large Run balances.
            expr::narrow_damage(
                &plan.source().site,
                &TypedInteger::new(value.value.min(i128::from(u32::MAX)), value.unit.clone()),
            )
            .map_err(runtime_control::reject_expression)?;
            value.value as u128
        } else {
            0
        };
        control::charge(
            (program.reactions.len() * 3) as u64,
            "equipment_attack_relic_capture",
        )?;
        let relic_copies = program
            .reactions
            .iter()
            .map(|r| {
                r.requires_relic
                    .as_ref()
                    .and_then(|id| self.equipped_normal_relic_copy(snapshot.source_id, id))
            })
            .collect();
        snapshot.equipment_attack = Some(Arc::new(EquipmentAttackCapture {
            primary,
            equipment,
            weapon_stage,
            program,
            bonus,
            relic_copies,
        }));
        control::poll("equipment_attack_bound")
    }

    /// The existing attack capture survives projectile release and carries the
    /// weapon policy once. Construct attribution only when settling real HP loss.
    pub(super) fn equipment_basic_lifesteal(
        &self, snapshot: &DamageSourceSnapshot,
    ) -> Result<Option<crate::game::battle::damage::BasicLifestealSnapshot>, GameError> {
        let Some(capture) = &snapshot.equipment_attack else { return Ok(None); };
        let Some(lifesteal) = &capture.program.basic_lifesteal else { return Ok(None); };
        if snapshot.fact_source.reaction { return Ok(None); }
        let mut origin = crate::authoring::clone_shape::controlled_clone(&snapshot.fact_source)?;
        origin.actor = Some(snapshot.source_id);
        origin.equipment = Some(capture.equipment);
        origin.authored = Some(Arc::clone(&lifesteal.source));
        runtime_control::admit_retained_growth(0,
            origin.retained_extent().and_then(|n| n.checked_add(size_of::<crate::game::battle::runtime_facts::RuntimeFactSource>() + 64)),
            "equipment_lifesteal_source")?;
        Ok(Some(crate::game::battle::damage::BasicLifestealSnapshot {
            percent: lifesteal.definition.percent,
            max_heal: lifesteal.definition.max_heal,
            equipment: Some(capture.equipment),
            fact_source: Some(Arc::new(origin.observation())),
        }))
    }

    pub(super) fn apply_equipment_attack_damage(
        &self,
        snapshot: &mut DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Result<Option<RuntimeFactSource>, GameError> {
        let Some(capture) = &snapshot.equipment_attack else {
            return Ok(None);
        };
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction {
            return Ok(None);
        }
        let percent = if target == capture.primary {
            100
        } else {
            capture.program.secondary_percent
        };
        // Quotient/remainder avoids overflow even at the expression's i128
        // maximum. The authored percent has already been checked as 0..100.
        let bonus = capture.bonus / 100 * u128::from(percent)
            + capture.bonus % 100 * u128::from(percent) / 100;
        let before = snapshot.base_damage;
        snapshot.base_damage =
            (u128::from(snapshot.base_damage) + bonus).min(u128::from(u32::MAX)) as u32;
        control::poll("equipment_attack_damage")?;
        if snapshot.base_damage == before {
            return Ok(None);
        }
        let plan = capture.program.bonus.as_ref()
            .ok_or_else(|| runtime_control::invariant("equipment_attack_bonus_source"))?;
        // The compiled program already accounts for its owned source payload.
        // Conservatively admit that extent before copying the smaller source;
        // the retained DamageFact accounts for the resulting contributor too.
        runtime_control::admit_retained_growth(
            0,
            capture.program.retained_bytes.checked_add(size_of::<RuntimeFactSource>() + 64),
            "equipment_attack_damage_source",
        )?;
        let source = RuntimeFactSource {
            actor: Some(snapshot.source_id),
            equipment: Some(capture.equipment),
            authored: Some(Arc::new(crate::authoring::clone_shape::controlled_clone(plan.source())?)),
            ..snapshot.fact_source.observation()
        };
        control::poll("equipment_attack_damage_source_ready")?;
        Ok(Some(source))
    }

    pub(super) fn equipment_attack_hit(
        &mut self, snapshot: &DamageSourceSnapshot, target: UnitInstanceId, now: u64,
        strike: crate::game::battle::runtime_facts::StrikeOrdinal,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.equipment_hits else { return Ok(()); };
        if snapshot.source != DamageSource::BasicAttack || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot.fact_source.cast.as_ref().is_some_and(|c| c.proc_origin.is_some())
            || snapshot.source_id == target
            || !self.units.get(&target).is_some_and(|u| u.is_active() && u.is_combatant() && u.owner != snapshot.source_side)
        { return Ok(()); }
        for provider in &capture.providers {
            let status = provider.program.on_hit_status.as_ref()
                .ok_or_else(|| runtime_control::invariant("equipment_hit_capture_status"))?;
            control::charge(1, "equipment_hit_chance")?;
            let seed = crate::game::determinism::seed_with_uuid(
                self.seed, 0x4551484954544152, target.as_uuid());
            let seed = crate::game::determinism::seed_with_uuid(
                seed, 0x4551484954434f50, provider.equipment.runtime_item_instance_id);
            let roll = crate::game::determinism::uuid_v4_from_seed(
                seed, 0x4551484954524f4c, strike.0).as_u128() % 100;
            if roll >= u128::from(status.definition.chance_percent) { continue; }
            let profile = crate::authoring::clone_shape::controlled_clone(&status.definition.profile)?;
            let mut source = crate::authoring::clone_shape::controlled_clone(&snapshot.fact_source)?;
            source.actor = Some(snapshot.source_id);
            source.equipment = Some(provider.equipment);
            source.authored = Some(Arc::clone(&status.source));
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let result = self.begin_committed_hit_status_with_refresh(snapshot.source_id, target, snapshot.source_side,
                status.definition.duration_ms, profile, now, status.definition.refresh_movement_slow);
            self.runtime_fact_source = prior;
            result?;
            control::poll("equipment_attack_hit_applied")?;
        }
        Ok(())
    }

    pub(super) fn equipment_attack_kill(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        victim: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        let Some(capture) = &snapshot.equipment_attack else {
            return Ok(());
        };
        if snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || victim != capture.primary
            || !self
                .units
                .get(&victim)
                .is_some_and(|u| u.lifecycle.is_dead() && u.owner != snapshot.source_side)
        {
            return Ok(());
        }
        let mut source = snapshot.fact_source.clone();
        source.equipment = Some(capture.equipment);
        source.reaction = true;
        for (i, reaction) in capture.program.reactions.iter().enumerate() {
            control::charge(1, "equipment_attack_reaction")?;
            if reaction.requires_relic.is_some() && capture.relic_copies[i].is_none() {
                continue;
            }
            let EquipmentAttackEvent::PrimaryBasicKill {
                eligibility,
                threat,
            } = reaction.event;
            if threat.is_some_and(|class| self.units[&victim].threat_class != class)
                || (eligibility == StateKillEligibility::EncounterCombatant
                    && !self.eligible_encounter_death(victim, snapshot.source_side)?)
            {
                continue;
            }
            self.run_equipment_reaction(&reaction.program, &source, snapshot.source_id, now)?;
            control::poll("equipment_attack_reacted")?;
        }
        Ok(())
    }
}
