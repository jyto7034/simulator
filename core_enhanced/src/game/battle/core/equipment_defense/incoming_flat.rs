//! A target strike owns one finite fixed allowance. Observation is evidence only.
use super::*;
use crate::game::{
    battle::damage::{DamageSource, DamageSourceSnapshot},
    value_expr::{
        self as expr, CompiledAmountPlan, EvalFrame, ExprMeter, StatSelector, TypedInteger,
        ValueRef, ValueSubject,
    },
};

#[derive(Debug, Clone, Copy, Default)]
pub(in crate::game::battle::core) struct FlatAllowance {
    pub equipment: Option<EquipmentSourceSnapshot>,
    pub defense: i32,
    pub total: u32,
    pub remaining: u32,
}
crate::authoring::clone_shape::copy_shape!(FlatAllowance);

#[derive(Debug, Clone, serde::Serialize)]
pub struct IncomingFlatWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub strike: u64,
    pub defense: i32,
    pub allowance: u32,
    pub before: u32,
    pub used: u32,
    pub remaining: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(IncomingFlatWitness {
    generation,
    target,
    equipment,
    time_ms,
    strike,
    defense,
    allowance,
    before,
    used,
    remaining,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl IncomingFlatWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}

fn evaluate(plan: &CompiledAmountPlan, defense: i32) -> Result<u32, GameError> {
    let refs = &plan.requirements().references;
    runtime_control::admit_retained_growth(
        0,
        refs.len()
            .checked_mul(size_of::<(ValueRef, TypedInteger)>() + 128),
        "incoming_flat_frame",
    )?;
    let mut frame = EvalFrame::with_value_capacity(refs.len());
    for reference in refs {
        control::charge(1, "incoming_flat_frame_reference")?;
        if !matches!(
            reference,
            ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Defense
            }
        ) {
            return Err(runtime_control::invariant("incoming_flat_reference"));
        }
        frame
            .insert(
                reference.clone(),
                TypedInteger::new(i128::from(defense), reference.unit()),
            )
            .map_err(|_| runtime_control::invariant("incoming_flat_frame"))?;
    }
    let mut meter = ExprMeter::new(plan.evaluation_limits())
        .map_err(|_| runtime_control::invariant("incoming_flat_meter"))?
        .with_host_admission(super::super::value_expr_runtime::evaluation_host_admission);
    let value = expr::evaluate(plan, &frame, &mut meter)
        .map_err(runtime_control::reject_expression)?
        .value;
    if !(0..=i128::from(i32::MAX)).contains(&value.value) {
        return Err(runtime_control::reject_expression(expr::ValueExprFault {
            code: expr::ValueExprFaultCode::Overflow,
            site: plan.source().site.clone(),
            node_index: 0,
            expected_unit: Some(expr::ValueUnit::Magnitude),
            actual_unit: Some(value.unit),
            phase: expr::ValueExprPhase::Narrow,
            cause: "fixed reduction must be from 0 to i32::MAX".into(),
        }));
    }
    Ok(value.value as u32)
}

impl BattleCore {
    pub(in crate::game::battle::core) fn incoming_flat_eligible(
        &self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
    ) -> bool {
        self.game_data.equipment_defense_catalog().has_incoming_flat
            && incoming.source == DamageSource::BasicAttack
            && !incoming.fact_source.reaction
            && incoming.fact_source.guard_release().is_none()
            && !incoming
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
            && incoming.source_id != target
            && self.units.get(&target).is_some_and(|u| {
                u.is_active() && u.is_combatant() && !u.corroded && u.owner != incoming.source_side
            })
    }

    pub(in crate::game::battle::core) fn evaluate_incoming_flat(
        &self,
        target: UnitInstanceId,
    ) -> Result<FlatAllowance, GameError> {
        let defense = self
            .units
            .get(&target)
            .ok_or(GameError::InvalidAction)?
            .stats
            .defense;
        let mut selected: Result<FlatAllowance, GameError> = Ok(FlatAllowance {
            defense,
            ..Default::default()
        });
        self.visit_equipment_defenses(target, |equipment, program| {
            let Some(plan) = &program.incoming_flat else {
                return;
            };
            if selected.is_err() {
                return;
            }
            match evaluate(plan, defense) {
                Err(error) => selected = Err(error),
                Ok(amount) => {
                    let choice = selected.as_mut().unwrap();
                    if choice.equipment.is_none()
                        || amount > choice.total
                        || (amount == choice.total
                            && equipment.runtime_item_instance_id
                                < choice.equipment.unwrap().runtime_item_instance_id)
                    {
                        *choice = FlatAllowance {
                            equipment: Some(equipment),
                            defense,
                            total: amount,
                            remaining: amount,
                        };
                    }
                }
            }
        })?;
        control::poll("incoming_flat_evaluated")?;
        selected
    }

    pub(in crate::game::battle::core) fn observe_incoming_flat(
        &mut self,
        target: UnitInstanceId,
        incoming: &DamageSourceSnapshot,
        now: u64,
        strike: u64,
        allowance: FlatAllowance,
        before: u32,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let Some(equipment) = allowance.equipment else {
            return Ok(());
        };
        let plan = self
            .game_data
            .equipment_defense_catalog()
            .by_equipment
            .get(&equipment.definition_uuid)
            .and_then(|p| p.incoming_flat.as_ref())
            .ok_or_else(|| runtime_control::invariant("incoming_flat_observation_origin"))?;
        let source = plan.source();
        let extent = incoming.fact_source.retained_extent().and_then(|n| {
            n.checked_add(
                size_of::<IncomingFlatWitness>()
                    + source.site.action.definition_id.as_str().len()
                    + source.source_file.len()
                    + source.field_path.len(),
            )
        });
        runtime_control::admit_retained_growth(0, extent, "incoming_flat_witness_copy")?;
        control::charge(
            extent.ok_or(GameError::InvalidAction)? as u64,
            "incoming_flat_witness_copy",
        )?;
        let witness = IncomingFlatWitness {
            generation: self.content_identity.clone(),
            target,
            equipment,
            time_ms: now,
            strike,
            defense: allowance.defense,
            allowance: allowance.total,
            before,
            used: before
                .checked_sub(allowance.remaining)
                .ok_or_else(|| runtime_control::invariant("incoming_flat_allowance"))?,
            remaining: allowance.remaining,
            definition_id: source.site.action.definition_id.to_string(),
            source_file: source.source_file.clone(),
            field_path: source.field_path.clone(),
            incoming: incoming.fact_source.observation(),
        };
        runtime_control::admit_retained_growth(
            0,
            witness.retained_extent(),
            "incoming_flat_observation",
        )?;
        self.authoring_observation
            .as_mut()
            .unwrap()
            .latest_incoming_flat = Some(witness);
        Ok(())
    }
}
