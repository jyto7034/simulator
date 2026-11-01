//! One immutable percentage evaluated over a finalized spatial damage batch.
//! No clock window, cross-hit accumulator, extra damage, or defensive preview.
pub use super::equipment_primary_damage::PrimaryDamageContribution as AreaDamageContribution;
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        battle::{ids::UnitInstanceId, runtime_control, runtime_facts::RuntimeFactSource},
        behavior::GameError,
        enums::Side,
        value_expr::{self as expr, EvalFrame, EvalMember, ExprMeter},
    },
};
use std::{
    collections::BTreeMap,
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AreaDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub action: Option<u64>,
    pub target_count: u32,
    pub percent: u32,
    pub contributors: Vec<AreaDamageContribution>,
}
crate::authoring::clone_shape::struct_shape!(AreaDamageWitness {
    generation,
    time_ms,
    actor,
    action,
    target_count,
    percent,
    contributors
});
impl AreaDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<AreaDamageContribution>())?,
        )?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        for c in &self.contributors {
            bytes = bytes
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    /// Call only after this single area's geometry, mobility, and tick policy
    /// have selected its targets, and before its first damage application.
    /// Shields, immunity, evasion and damage reductions are downstream defenses;
    /// evaluating this count neither previews nor consumes their proc state.
    pub(super) fn area_damage_percent(
        &mut self,
        actor: UnitInstanceId,
        side: Side,
        source: &RuntimeFactSource,
        targets: &[UnitInstanceId],
        now: u64,
    ) -> Result<i32, GameError> {
        if !self.game_data.equipment_attack_catalog().has_area_damage {
            return Ok(0);
        }
        let providers =
            self.equipped_attack_providers(actor, |p| p.area_damage_percent.is_some())?;
        if providers.is_empty() {
            return Ok(0);
        }
        control::charge(
            (targets.len() as u64)
                .saturating_mul(2 + u64::from(usize::BITS - targets.len().leading_zeros())),
            "area_damage_target_count",
        )?;
        runtime_control::admit_retained_growth(
            0,
            targets
                .len()
                .checked_mul(size_of::<UnitInstanceId>() + size_of::<EvalMember>() + 128),
            "area_damage_frame",
        )?;
        let mut unique: Vec<_> = targets
            .iter()
            .copied()
            .filter(|target| {
                *target != actor
                    && self
                        .units
                        .get(target)
                        .is_some_and(|u| u.is_active() && u.is_combatant() && u.owner != side)
            })
            .collect();
        unique.sort_unstable();
        unique.dedup();
        let target_count = u32::try_from(unique.len())
            .map_err(|_| runtime_control::invariant("area_damage_count"))?;
        let mut frame = EvalFrame::default();
        frame.set_resolved_targets(
            unique
                .into_iter()
                .map(|id| EvalMember {
                    identity: id.to_string(),
                    value: None,
                    owner: None,
                    projected_tile: None,
                    values: BTreeMap::new(),
                })
                .collect(),
        );
        let mut total = 0u32;
        let mut contributors = Vec::new();
        for (equipment, program) in providers {
            let plan = program.area_damage_percent.as_ref().unwrap();
            let mut meter = ExprMeter::new(plan.evaluation_limits())
                .map_err(|_| runtime_control::invariant("area_damage_meter"))?
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
                    cause: "area damage increase must be from 0 to 10000 percent".into(),
                }));
            }
            let percent = value.value as u32;
            total = total
                .checked_add(percent)
                .filter(|n| *n <= i32::MAX as u32)
                .ok_or_else(|| runtime_control::invariant("area_damage_sum"))?;
            if self.authoring_observation.is_some() {
                let origin = plan.source();
                let bytes = size_of::<AreaDamageContribution>()
                    .checked_add(origin.site.action.definition_id.as_str().len())
                    .and_then(|n| n.checked_add(origin.source_file.len()))
                    .and_then(|n| n.checked_add(origin.field_path.len()));
                runtime_control::admit_retained_growth(0, bytes, "area_damage_witness_copy")?;
                control::charge(
                    bytes.ok_or(GameError::InvalidAction)? as u64,
                    "area_damage_witness_copy",
                )?;
                contributors.reserve_exact(1);
                contributors.push(AreaDamageContribution {
                    equipment,
                    percent,
                    definition_id: origin.site.action.definition_id.to_string(),
                    source_file: origin.source_file.clone(),
                    field_path: origin.field_path.clone(),
                });
            }
        }
        if !contributors.is_empty() {
            let witness = AreaDamageWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor,
                action: source.action.as_ref().map(|a| a.ordinal().0),
                target_count,
                percent: total,
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "area_damage_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_area_damage = Some(witness);
        }
        control::poll("area_damage_evaluated")?;
        Ok(total as i32)
    }
}
