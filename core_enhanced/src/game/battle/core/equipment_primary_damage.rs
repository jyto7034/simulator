//! Live target-HP expressions modify only a normal attack's direct primary hit.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
        },
        behavior::GameError,
        value_expr::{self as expr, EvalFrame, ExprMeter, TypedInteger, ValueRef},
    },
};
use std::{
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrimaryDamageContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(PrimaryDamageContribution {
    equipment,
    percent,
    definition_id,
    source_file,
    field_path
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct PrimaryDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub action: u64,
    pub target_current_hp: u32,
    pub target_max_hp: u32,
    pub percent: u32,
    pub contributors: Vec<PrimaryDamageContribution>,
}
crate::authoring::clone_shape::struct_shape!(PrimaryDamageWitness {
    generation,
    time_ms,
    actor,
    target,
    action,
    target_current_hp,
    target_max_hp,
    percent,
    contributors
});
impl PrimaryDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<PrimaryDamageContribution>())?,
        )?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        for contribution in &self.contributors {
            bytes = bytes
                .checked_add(contribution.definition_id.capacity())?
                .checked_add(contribution.source_file.capacity())?
                .checked_add(contribution.field_path.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(super) fn equipment_primary_damage_percent(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        if !self.game_data.equipment_attack_catalog().has_primary_damage
            || snapshot.source != DamageSource::BasicAttack
            || snapshot.basic_primary != Some(target)
            || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
            || snapshot.fact_source.action.is_none()
            || snapshot.source_id == target
        {
            return Ok(0);
        }
        let Some(unit) = self.units.get(&target).filter(|unit| {
            unit.is_active() && unit.is_combatant() && unit.owner != snapshot.source_side
        }) else {
            return Ok(0);
        };
        let (current_hp, max_hp) = (unit.stats.current_health, unit.stats.max_health);
        let mut total = 0u32;
        let mut contributors = Vec::new();
        for (equipment, program) in self
            .equipped_attack_providers(snapshot.source_id, |p| p.primary_damage_percent.is_some())?
        {
            let plan = program.primary_damage_percent.as_ref().unwrap();
            let refs = &plan.requirements().references;
            runtime_control::admit_retained_growth(
                0,
                refs.len()
                    .checked_mul(size_of::<(ValueRef, TypedInteger)>() + 128),
                "primary_damage_frame",
            )?;
            let mut frame = EvalFrame::with_value_capacity(refs.len());
            for reference in refs {
                let ValueRef::HitPoints {
                    subject: expr::ValueSubject::Target,
                    field,
                } = reference
                else {
                    return Err(runtime_control::invariant("primary_damage_reference"));
                };
                let amount = match field {
                    expr::HitPointField::Current => current_hp,
                    expr::HitPointField::Max => max_hp,
                    expr::HitPointField::Missing => max_hp.saturating_sub(current_hp),
                };
                frame
                    .insert(
                        reference.clone(),
                        TypedInteger::new(i128::from(amount), reference.unit()),
                    )
                    .map_err(|_| runtime_control::invariant("primary_damage_frame"))?;
            }
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| runtime_control::invariant("primary_damage_meter"))?
                .with_host_admission(super::value_expr_runtime::evaluation_host_admission);
            let value = expr::evaluate(plan, &frame, &mut meter)
                .map_err(runtime_control::reject_expression)?
                .value;
            if !(0..=10_000).contains(&value.value) {
                return Err(runtime_control::reject_expression(expr::ValueExprFault {
                    code: expr::ValueExprFaultCode::Overflow,
                    site: plan.source().site.clone(),
                    node_index: 0,
                    expected_unit: Some(expr::ValueUnit::Scalar),
                    actual_unit: Some(value.unit),
                    phase: expr::ValueExprPhase::Narrow,
                    cause: "primary damage increase must be from 0 to 10000 percent".into(),
                }));
            }
            let percent = value.value as u32;
            total = total
                .checked_add(percent)
                .filter(|v| *v <= i32::MAX as u32)
                .ok_or_else(|| runtime_control::invariant("primary_damage_sum"))?;
            if self.authoring_observation.is_some() {
                let source = plan.source();
                let bytes = size_of::<PrimaryDamageContribution>()
                    .checked_add(source.site.action.definition_id.as_str().len())
                    .and_then(|v| v.checked_add(source.source_file.len()))
                    .and_then(|v| v.checked_add(source.field_path.len()));
                runtime_control::admit_retained_growth(0, bytes, "primary_damage_witness_copy")?;
                control::charge(
                    bytes.ok_or(GameError::InvalidAction)? as u64,
                    "primary_damage_witness_copy",
                )?;
                contributors.reserve_exact(1);
                contributors.push(PrimaryDamageContribution {
                    equipment,
                    percent,
                    definition_id: source.site.action.definition_id.to_string(),
                    source_file: source.source_file.clone(),
                    field_path: source.field_path.clone(),
                });
            }
        }
        if !contributors.is_empty() {
            let witness = PrimaryDamageWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor: snapshot.source_id,
                target,
                action: snapshot.fact_source.action.as_ref().unwrap().ordinal().0,
                target_current_hp: current_hp,
                target_max_hp: max_hp,
                percent: total,
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "primary_damage_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_primary_damage = Some(witness);
        }
        control::poll("primary_damage_evaluated")?;
        Ok(total as i32)
    }
}
