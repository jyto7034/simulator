//! Outgoing damage from pure conditions over the admitted target strike.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::{
            conditional_damage::{
                damage_type_index, CompiledDamageCondition, ConditionalDamageScope,
            },
            flow::FlowComparison,
        },
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
            types::BattleUnitSourceIdentity,
        },
        behavior::GameError,
        enums::Side,
        value_expr::{
            self as expr, CompiledAmountPlan, EvalFrame, ExprMeter, HitPointField, TypedInteger,
            ValueRef, ValueSubject,
        },
    },
};
use std::{
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConditionalDamageContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub matched: bool,
    pub eligible: bool,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(ConditionalDamageContribution {
    equipment,
    matched,
    eligible,
    percent,
    definition_id,
    source_file,
    field_path
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConditionalDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub wearer: UnitInstanceId,
    pub target: UnitInstanceId,
    pub wearer_hp: u32,
    pub wearer_max_hp: u32,
    pub target_hp: u32,
    pub target_max_hp: u32,
    pub target_movement_impaired: Option<bool>,
    pub distance_squared: Option<f64>,
    pub percent: u32,
    /// All component types share the same condition/HP frame for this strike.
    pub physical_percent: u32,
    pub magic_percent: u32,
    pub true_percent: u32,
    pub contributors: Vec<ConditionalDamageContribution>,
}
crate::authoring::clone_shape::struct_shape!(ConditionalDamageWitness {
    generation,
    time_ms,
    actor,
    wearer,
    target,
    wearer_hp,
    wearer_max_hp,
    target_hp,
    target_max_hp,
    target_movement_impaired,
    distance_squared,
    percent,
    physical_percent,
    magic_percent,
    true_percent,
    contributors
});
impl ConditionalDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<ConditionalDamageContribution>())?,
        )?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
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
pub(super) struct Evaluation {
    owner: UnitInstanceId,
    pub(super) distance_squared: Option<f64>,
    pub(super) owner_hp: u32,
    pub(super) owner_max: u32,
    pub(super) target_hp: u32,
    pub(super) target_max: u32,
    target: UnitInstanceId,
    now: u64,
    pub(super) movement_impaired: Option<bool>,
}
impl Evaluation {
    pub(super) fn new(
        core: &BattleCore,
        owner: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<Self, GameError> {
        let wearer = core.units.get(&owner).ok_or(GameError::InvalidAction)?;
        let enemy = core.units.get(&target).ok_or(GameError::InvalidAction)?;
        Ok(Self {
            owner,
            target,
            now,
            owner_hp: wearer.stats.current_health,
            owner_max: wearer.stats.max_health,
            target_hp: enemy.stats.current_health,
            target_max: enemy.stats.max_health,
            movement_impaired: None,
            distance_squared: None,
        })
    }
    pub(super) fn amount(&self, plan: &CompiledAmountPlan) -> Result<i128, GameError> {
        let refs = &plan.requirements().references;
        runtime_control::admit_retained_growth(
            0,
            refs.len()
                .checked_mul(size_of::<(ValueRef, TypedInteger)>() + 128),
            "conditional_damage_frame",
        )?;
        let mut frame = EvalFrame::with_value_capacity(refs.len());
        for reference in refs {
            let ValueRef::HitPoints { subject, field } = reference else {
                return Err(runtime_control::invariant("conditional_damage_reference"));
            };
            let (hp, max) = match subject {
                ValueSubject::Source => (self.owner_hp, self.owner_max),
                ValueSubject::Target => (self.target_hp, self.target_max),
                _ => return Err(runtime_control::invariant("conditional_damage_subject")),
            };
            let value = match field {
                HitPointField::Current => hp,
                HitPointField::Max => max,
                HitPointField::Missing => max.saturating_sub(hp),
            };
            frame
                .insert(
                    reference.clone(),
                    TypedInteger::new(i128::from(value), reference.unit()),
                )
                .map_err(|_| runtime_control::invariant("conditional_damage_frame"))?;
        }
        let mut meter = ExprMeter::new(plan.evaluation_limits())
            .map_err(|_| runtime_control::invariant("conditional_damage_meter"))?
            .with_host_admission(super::value_expr_runtime::evaluation_host_admission);
        Ok(expr::evaluate(plan, &frame, &mut meter)
            .map_err(runtime_control::reject_expression)?
            .value
            .value)
    }
    pub(super) fn matches(
        &mut self,
        core: &BattleCore,
        condition: &CompiledDamageCondition,
    ) -> Result<bool, GameError> {
        control::charge(1, "conditional_damage_condition")?;
        Ok(match condition {
            CompiledDamageCondition::Distance { comparison, units } => {
                let distance = if let Some(value) = self.distance_squared {
                    value
                } else {
                    let a = core
                        .sample_unit_world_position_at(self.owner, self.now)
                        .ok_or_else(|| runtime_control::invariant("conditional_distance_owner"))?;
                    let b = core
                        .sample_unit_world_position_at(self.target, self.now)
                        .ok_or_else(|| runtime_control::invariant("conditional_distance_target"))?;
                    // Convert before subtraction/squaring so every finite f32 position remains finite.
                    let dx = f64::from(a.x()) - f64::from(b.x());
                    let dy = f64::from(a.y()) - f64::from(b.y());
                    let value = dx * dx + dy * dy;
                    self.distance_squared = Some(value);
                    value
                };
                let threshold =
                    f64::from(*units) / f64::from(super::movement::types::DATA_UNITS_PER_WORLD);
                let threshold = threshold * threshold;
                match comparison {
                    FlowComparison::Equal => distance == threshold,
                    FlowComparison::Greater => distance > threshold,
                    FlowComparison::AtLeast => distance >= threshold,
                    FlowComparison::Less => distance < threshold,
                }
            }
            CompiledDamageCondition::MovementImpaired => {
                if let Some(value) = self.movement_impaired {
                    value
                } else {
                    let value = core.target_movement_impaired(self.target, self.now)?;
                    self.movement_impaired = Some(value);
                    value
                }
            }
            CompiledDamageCondition::Compare {
                left,
                comparison,
                right,
            } => {
                let a = self.amount(left)?;
                let b = self.amount(right)?;
                match comparison {
                    FlowComparison::Equal => a == b,
                    FlowComparison::Greater => a > b,
                    FlowComparison::AtLeast => a >= b,
                    FlowComparison::Less => a < b,
                }
            }
            CompiledDamageCondition::Logic(logic) => {
                logic.evaluate(|child| self.matches(core, child))?
            }
        })
    }
}
impl BattleCore {
    /// One checked owner interpretation shared with nearby-ally damage.
    pub(super) fn owned_equipment_damage_source(
        &self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Option<UnitInstanceId> {
        if self.sustained.closed
            || (snapshot.source == DamageSource::Environment && !snapshot.fact_source.reaction)
            || snapshot.source_side != Side::Player
            || snapshot.source_id == target
            || !self.units.get(&target).is_some_and(|u| {
                u.is_active()
                    && u.is_combatant()
                    && u.stats.current_health > 0
                    && u.owner != snapshot.source_side
            })
        {
            return None;
        }
        let direct = self.units.get(&snapshot.source_id).is_some_and(|u| {
            matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
        });
        let owner = if direct {
            snapshot.source_id
        } else if let Some(origin) = snapshot.fact_source.independent_object() {
            if origin.object_id != snapshot.source_id {
                return None;
            }
            origin.creator
        } else {
            snapshot.source_id
        };
        self.units
            .get(&owner)
            .is_some_and(|u| {
                u.is_active()
                    && u.is_combatant()
                    && !u.corroded
                    && u.owner == Side::Player
                    && u.stats.current_health > 0
                    && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
            })
            .then_some(owner)
    }
    pub(super) fn conditional_damage_per_type(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<[i32; 3], GameError> {
        if !self
            .game_data
            .equipment_attack_catalog()
            .has_conditional_damage
        {
            return Ok([0; 3]);
        }
        let Some(owner) = self.owned_equipment_damage_source(snapshot, target) else {
            return Ok([0; 3]);
        };
        let providers =
            self.equipped_attack_providers(owner, |p| p.conditional_damage.is_some())?;
        if providers.is_empty() {
            return Ok([0; 3]);
        }
        let mut evaluation = Evaluation::new(self, owner, target, now)?;
        let mut totals = [0u32; 3];
        let type_index = damage_type_index(snapshot.damage_type);
        let mut contributors = Vec::new();
        for (equipment, program) in providers {
            let policy = program.conditional_damage.as_ref().unwrap();
            let source_eligible = policy.filter.scope == ConditionalDamageScope::Owned
                || (snapshot.source_id == owner
                    && !snapshot.fact_source.reaction
                    && snapshot.fact_source.guard_release().is_none()
                    && snapshot.fact_source.independent_object().is_none()
                    && !snapshot
                        .fact_source
                        .cast
                        .as_ref()
                        .is_some_and(|cast| cast.proc_origin.is_some())
                    && matches!(
                        snapshot.source,
                        DamageSource::BasicAttack | DamageSource::Ability | DamageSource::BuffTick
                    ));
            let matched = source_eligible && evaluation.matches(self, &policy.condition)?;
            let raw = if matched {
                evaluation.amount(&policy.percent)?
            } else {
                0
            };
            if !(0..=10_000).contains(&raw) {
                return Err(runtime_control::reject_expression(expr::ValueExprFault {
                    code: expr::ValueExprFaultCode::Overflow,
                    site: policy.percent.source().site.clone(),
                    node_index: 0,
                    expected_unit: Some(expr::ValueUnit::Scalar),
                    actual_unit: Some(expr::ValueUnit::Scalar),
                    phase: expr::ValueExprPhase::Narrow,
                    cause: "conditional damage increase must be from 0 to 10000 percent".into(),
                }));
            }
            let percent = raw as u32;
            for (index, total) in totals.iter_mut().enumerate() {
                if policy.filter.types[index] {
                    *total = total
                        .checked_add(percent)
                        .filter(|n| *n <= i32::MAX as u32)
                        .ok_or_else(|| runtime_control::invariant("conditional_damage_sum"))?;
                }
            }
            let eligible = source_eligible && policy.filter.types[type_index];
            if self.authoring_observation.is_some() {
                let origin = policy.percent.source();
                let bytes = size_of::<ConditionalDamageContribution>()
                    .checked_add(origin.site.action.definition_id.as_str().len())
                    .and_then(|n| n.checked_add(origin.source_file.len()))
                    .and_then(|n| n.checked_add(origin.field_path.len()));
                runtime_control::admit_retained_growth(
                    0,
                    bytes,
                    "conditional_damage_witness_copy",
                )?;
                control::charge(
                    bytes.ok_or(GameError::InvalidAction)? as u64,
                    "conditional_damage_witness_copy",
                )?;
                contributors.reserve_exact(1);
                contributors.push(ConditionalDamageContribution {
                    equipment,
                    matched,
                    eligible,
                    percent: if eligible { percent } else { 0 },
                    definition_id: origin.site.action.definition_id.to_string(),
                    source_file: origin.source_file.clone(),
                    field_path: origin.field_path.clone(),
                });
            }
        }
        if !contributors.is_empty() {
            let witness = ConditionalDamageWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor: snapshot.source_id,
                wearer: owner,
                target,
                wearer_hp: evaluation.owner_hp,
                wearer_max_hp: evaluation.owner_max,
                target_hp: evaluation.target_hp,
                target_max_hp: evaluation.target_max,
                target_movement_impaired: evaluation.movement_impaired,
                distance_squared: evaluation.distance_squared,
                percent: totals[type_index],
                physical_percent: totals[0],
                magic_percent: totals[1],
                true_percent: totals[2],
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "conditional_damage_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_conditional_damage = Some(witness);
        }
        control::poll("conditional_damage_evaluated")?;
        Ok(totals.map(|value| value as i32))
    }
}
