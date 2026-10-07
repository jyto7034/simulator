//! Generation- and occurrence-bound adapter from battle state to the shared
//! value-expression evaluator. Sequential effect ownership consumes this seam;
//! it does not duplicate expression arithmetic or publish authored content.

use std::{collections::BTreeMap, mem::size_of, sync::Arc};

use crate::game::{
    ability::{
        authoring::{ActionSite, AuthoredSkillEffect},
        SkillEffectDef, SkillStepDef,
    },
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{ActionHandle, ActionQueryError, DeliveryTicket, RuntimeFactSource},
    },
    behavior::GameError,
    data::content_pack::ContentIdentity,
    resource::{
        store::{ResourceError, ResourceFaultCode},
        ResourceOwnerSelector,
    },
    value_expr::{
        can_evaluate_without_external_context, evaluate, narrow_damage, narrow_heal,
        value_ref_dynamic_bytes, AmountPlanKind, CompiledAmountEntry, CompiledAmountPlan,
        CompiledOp, EvalFrame, EvalMember, EvalMoment, EvaluatedAmount, ExprMeter, ExprUsage,
        FiniteSource, GroupOwner, GroupOwnerDomain, Grouping, HitPointField,
        PreparedResolvedTargetView, ProjectedTile, ResolvedOwnerRelation, ResolvedTargetPredicate,
        ResourceValueField, ResultField, ResultSelector, StatSelector, TimeValue, TypedInteger,
        ValueExprFault, ValueExprFaultCode, ValueExprPhase, ValueRef, ValueSubject, ValueUnit,
    },
};

use super::BattleCore;

#[cfg(test)]
thread_local! {
    static RESOLVED_SLOT_ITEM_LOOKUPS: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

#[cfg(test)]
fn record_resolved_slot_item_lookup() {
    RESOLVED_SLOT_ITEM_LOOKUPS.with(|lookups| lookups.set(lookups.get() + 1));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub(crate) enum RuntimeAmount {
    Damage(u32),
    Heal(i32),
    Shield(u32),
}

#[derive(Debug)]
pub(crate) struct RuntimeAmountEvaluation {
    pub(crate) amount: RuntimeAmount,
    pub(crate) evaluated: EvaluatedAmount,
    pub(crate) usage: ExprUsage,
    pub(crate) observation: Option<RuntimeAmountObservation>,
}

#[derive(Debug)]
pub(crate) struct RuntimeAmountObservation {
    pub(crate) generation: Arc<ContentIdentity>,
    pub(crate) source: RuntimeFactSource,
    pub(crate) release_target: Option<UnitInstanceId>,
    pub(crate) consumption_target: Option<UnitInstanceId>,
    pub(crate) release_time_ms: u64,
    pub(crate) consume_time_ms: u64,
    pub(crate) effect_index: usize,
    pub(crate) plan: Arc<CompiledAmountPlan>,
    pub(crate) frame: Arc<EvalFrame>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EffectResultProjection {
    damage_hp_lost: i128,
    healing_delta: i128,
    resource_deltas: Vec<i128>,
    damage_strikes: i128,
    damaging_strikes: i128,
    count: i128,
}
crate::authoring::clone_shape::struct_shape!(EffectResultProjection {
    damage_hp_lost,
    healing_delta,
    resource_deltas,
    damage_strikes,
    damaging_strikes,
    count
});

impl EffectResultProjection {
    fn zero(resource_count: usize) -> Self {
        Self {
            resource_deltas: vec![0; resource_count],
            ..Self::default()
        }
    }

    fn checked_merge(&mut self, value: &Self) -> Option<()> {
        if self.resource_deltas.len() != value.resource_deltas.len() {
            return None;
        }
        let damage_hp_lost = self.damage_hp_lost.checked_add(value.damage_hp_lost)?;
        let healing_delta = self.healing_delta.checked_add(value.healing_delta)?;
        let damage_strikes = self.damage_strikes.checked_add(value.damage_strikes)?;
        let damaging_strikes = self.damaging_strikes.checked_add(value.damaging_strikes)?;
        let count = self.count.checked_add(value.count)?;
        for (slot, delta) in self.resource_deltas.iter().zip(&value.resource_deltas) {
            slot.checked_add(*delta)?;
        }
        self.damage_hp_lost = damage_hp_lost;
        self.healing_delta = healing_delta;
        self.damage_strikes = damage_strikes;
        self.damaging_strikes = damaging_strikes;
        self.count = count;
        for (slot, delta) in self.resource_deltas.iter_mut().zip(&value.resource_deltas) {
            *slot = slot.checked_add(*delta)?;
        }
        Some(())
    }
}

/// Compact completed frontier owned by one active action. Every successful
/// effect occurrence contributes once, including effects in other steps and
/// deliveries. The active action drops this private witness at settlement.
#[derive(Debug, Clone)]
pub(crate) struct ActionResultWitness {
    definition_id: crate::game::ability::SkillId,
    resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    cumulative: EffectResultProjection,
    completed_effects: u64,
}
crate::authoring::clone_shape::struct_shape!(ActionResultWitness {
    definition_id,
    resource_ids,
    cumulative,
    completed_effects
});

impl ActionResultWitness {
    pub(crate) fn new(
        definition_id: crate::game::ability::SkillId,
        resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    ) -> Self {
        Self {
            definition_id,
            cumulative: EffectResultProjection::zero(resource_ids.len()),
            resource_ids,
            completed_effects: 0,
        }
    }

    pub(crate) fn resource_ids(&self) -> Arc<Vec<crate::game::resource::ResourceId>> {
        Arc::clone(&self.resource_ids)
    }

    pub(crate) fn definition_id(&self) -> &crate::game::ability::SkillId {
        &self.definition_id
    }

    pub(crate) fn retained_extent(&self) -> Option<usize> {
        size_of::<Self>()
            .checked_add(self.definition_id.as_str().len())?
            .checked_add(
                self.resource_ids
                    .capacity()
                    .checked_mul(size_of::<crate::game::resource::ResourceId>())?,
            )?
            .checked_add(self.resource_ids.len().checked_mul(size_of::<i128>())?)?
            .checked_add(
                self.resource_ids
                    .iter()
                    .try_fold(0usize, |bytes, id| bytes.checked_add(id.as_str().len()))?,
            )?
            .checked_add(128)
    }

    pub(crate) fn settle(
        &mut self,
        site: &ActionSite,
        projection: &EffectResultProjection,
    ) -> Result<(), GameError> {
        if site.definition_id != self.definition_id {
            return Err(GameError::InvalidStaticData(
                "completed action result belongs to another authored definition".to_string(),
            ));
        }
        let work = u64::try_from(self.resource_ids.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| {
                GameError::InvalidStaticData("action result settle work overflowed".to_string())
            })?;
        crate::authoring::control::charge(work, "value_expr_action_result_settle")?;
        let completed = self.completed_effects.checked_add(1).ok_or_else(|| {
            GameError::InvalidStaticData("completed action effect count overflowed".to_string())
        })?;
        self.cumulative.checked_merge(projection).ok_or_else(|| {
            GameError::InvalidStaticData("completed action result overflowed".to_string())
        })?;
        self.completed_effects = completed;
        Ok(())
    }

    pub(in crate::game::battle::core) fn merge_completed(
        &mut self,
        prior: &Self,
    ) -> Result<(), GameError> {
        if self.definition_id != prior.definition_id || self.resource_ids != prior.resource_ids {
            return Err(GameError::InvalidStaticData(
                "scoped result basis differs".into(),
            ));
        }
        crate::authoring::control::charge(
            self.resource_ids.len() as u64 + 1,
            "flow_result_scope_merge",
        )?;
        self.cumulative
            .checked_merge(&prior.cumulative)
            .ok_or(GameError::InvalidAction)?;
        self.completed_effects = self
            .completed_effects
            .checked_add(prior.completed_effects)
            .ok_or(GameError::InvalidAction)?;
        Ok(())
    }
    fn value(
        &self,
        plan: &CompiledAmountPlan,
        field: &ResultField,
        node_index: usize,
    ) -> Result<TypedInteger, ValueExprFault> {
        if self.definition_id != plan.source().site.action.definition_id {
            return Err(fault(
                plan,
                ValueExprFaultCode::DifferentAttempt,
                node_index,
                "completed action result belongs to another authored definition",
            ));
        }
        if self.completed_effects == 0 {
            return Err(fault(
                plan,
                ValueExprFaultCode::NotSettled,
                node_index,
                "this action has no completed effect before the current frontier",
            ));
        }
        projected_result_value(
            &self.cumulative,
            &self.resource_ids,
            plan,
            &ResultSelector::CurrentActionPrefix,
            field,
            node_index,
        )
    }
}

fn projected_result_value(
    projection: &EffectResultProjection,
    resource_ids: &[crate::game::resource::ResourceId],
    plan: &CompiledAmountPlan,
    selector: &ResultSelector,
    field: &ResultField,
    node_index: usize,
) -> Result<TypedInteger, ValueExprFault> {
    let value = match field {
        ResultField::DamageHpLost => projection.damage_hp_lost,
        ResultField::HealingDelta => projection.healing_delta,
        ResultField::ResourceChangeDelta(resource) => {
            let index = resource_ids
                .iter()
                .position(|candidate| candidate == resource)
                .ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        node_index,
                        format!(
                            "resource result projection '{}' was not admitted",
                            resource.as_str()
                        ),
                    )
                })?;
            projection.resource_deltas[index]
        }
        ResultField::DamageStrikes => projection.damage_strikes,
        ResultField::DamagingStrikes => projection.damaging_strikes,
        ResultField::Count => projection.count,
    };
    Ok(TypedInteger::new(
        value,
        ValueRef::Result {
            selector: selector.clone(),
            field: field.clone(),
        }
        .unit(),
    ))
}

/// Bounded, direct-fact accumulator for one executing authored effect. It is
/// installed only while that effect's commands run and is restored around
/// nested execution.
#[derive(Debug, Clone)]
pub(crate) struct EffectResultAccumulator {
    source: RuntimeFactSource,
    resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    projection: EffectResultProjection,
    overflowed: bool,
}
crate::authoring::clone_shape::struct_shape!(EffectResultAccumulator {
    source,
    resource_ids,
    projection,
    overflowed
});

impl EffectResultAccumulator {
    fn accepts(&self, executing: &RuntimeFactSource, fact: &RuntimeFactSource) -> bool {
        // Persisted facts intentionally omit the private delivery ticket. The
        // command's still-live source supplies that occurrence at observation.
        self.source.action == executing.action
            && self.source.delivery == executing.delivery
            && self.source.action == fact.action
            && self.source.authored.as_ref().map(|value| &value.site)
                == executing.authored.as_ref().map(|value| &value.site)
            && self.source.authored.as_ref().map(|value| &value.site)
                == fact.authored.as_ref().map(|value| &value.site)
    }

    fn add(slot: &mut i128, delta: i128, overflowed: &mut bool) {
        match slot.checked_add(delta) {
            Some(value) => *slot = value,
            None => *overflowed = true,
        }
    }

    fn observe_damage(
        &mut self,
        executing: &RuntimeFactSource,
        source: &RuntimeFactSource,
        hp_lost: u32,
        first: bool,
        first_positive: bool,
    ) {
        if !self.accepts(executing, source) {
            return;
        }
        Self::add(
            &mut self.projection.damage_hp_lost,
            i128::from(hp_lost),
            &mut self.overflowed,
        );
        Self::add(
            &mut self.projection.damage_strikes,
            i128::from(u8::from(first)),
            &mut self.overflowed,
        );
        Self::add(
            &mut self.projection.damaging_strikes,
            i128::from(u8::from(first_positive)),
            &mut self.overflowed,
        );
        Self::add(&mut self.projection.count, 1, &mut self.overflowed);
    }

    fn observe_heal(
        &mut self,
        executing: &RuntimeFactSource,
        source: &RuntimeFactSource,
        delta: i64,
    ) {
        if !self.accepts(executing, source) {
            return;
        }
        Self::add(
            &mut self.projection.healing_delta,
            i128::from(delta),
            &mut self.overflowed,
        );
        Self::add(&mut self.projection.count, 1, &mut self.overflowed);
    }

    fn observe_resource(
        &mut self,
        executing: &RuntimeFactSource,
        source: &RuntimeFactSource,
        resource: &crate::game::battle::runtime_facts::FactResource,
        delta: i64,
    ) {
        if !self.accepts(executing, source) {
            return;
        }
        if let crate::game::battle::runtime_facts::FactResource::Typed { id, .. } = resource {
            if let Some(index) = self
                .resource_ids
                .iter()
                .position(|candidate| candidate == id.as_ref())
            {
                Self::add(
                    &mut self.projection.resource_deltas[index],
                    i128::from(delta),
                    &mut self.overflowed,
                );
            }
        }
        Self::add(&mut self.projection.count, 1, &mut self.overflowed);
    }
}

#[derive(Debug)]
struct EffectCaptureRow {
    captures: Vec<Arc<EvalCapture>>,
}

/// Release-time carrier shared by delayed delivery snapshots. Rows are aligned
/// with authored effect order; each row is either absent or has one capture per
/// fixed release target (or one unbound Live capture).
#[derive(Debug)]
pub(crate) struct ValueExprDeliveryCapture {
    action: ActionHandle,
    delivery: Option<Arc<DeliveryTicket>>,
    action_site: ActionSite,
    effect_ids: Arc<Vec<String>>,
    resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    rows: Vec<Option<EffectCaptureRow>>,
    retained_extent: usize,
}

impl ValueExprDeliveryCapture {
    pub(crate) fn retained_extent(&self) -> usize {
        self.retained_extent
    }

    pub(crate) fn resource_ids(&self) -> Arc<Vec<crate::game::resource::ResourceId>> {
        Arc::clone(&self.resource_ids)
    }

    pub(crate) fn capture_for(
        &self,
        effect_index: usize,
        target: Option<UnitInstanceId>,
    ) -> Option<&EvalCapture> {
        let row = self.rows.get(effect_index)?.as_ref()?;
        row.captures
            .iter()
            .find(|capture| capture.target_at_release == target)
            .or_else(|| {
                row.captures
                    .iter()
                    .find(|capture| capture.target_at_release.is_none())
            })
            .map(Arc::as_ref)
    }

    pub(crate) fn has_effect_capture(&self, effect_index: usize) -> bool {
        self.rows
            .get(effect_index)
            .and_then(Option::as_ref)
            .is_some_and(|row| !row.captures.is_empty())
    }

    pub(crate) fn rebind_consumption_source(
        &self,
        source: &RuntimeFactSource,
    ) -> Result<Arc<Self>, GameError> {
        if source.action.as_ref() != Some(&self.action) || source.delivery.is_none() {
            return Err(GameError::InvalidStaticData(
                "value-expression carrier cannot bind a different action or absent delivery"
                    .to_string(),
            ));
        }
        let capture_count = self.rows.iter().try_fold(0usize, |count, row| {
            count.checked_add(row.as_ref().map_or(0, |row| row.captures.len()))
        });
        // Each merged source retains only owners from the incoming source and
        // its original capture. The original carrier plus incoming extents
        // therefore bounds every rebound capture before collection allocation.
        let admission_extent = capture_count
            .and_then(|count| source.retained_extent()?.checked_mul(count))
            .and_then(|source_bytes| self.retained_extent.checked_add(source_bytes))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression carrier rebind size overflowed".to_string(),
                )
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(admission_extent),
            "value_expr_carrier_rebind",
        )?;
        let mut retained_extent = self.retained_extent;
        let mut rows = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let Some(row) = row else {
                rows.push(None);
                continue;
            };
            let mut captures = Vec::with_capacity(row.captures.len());
            for capture in &row.captures {
                let mut rebound_source =
                    source.with_exact_relic_numeric(capture.source.relic_numeric())?;
                rebound_source.authored = capture.source.authored.clone();
                let extent = capture_extent(
                    &capture.plan,
                    &rebound_source,
                    capture.plan.moment() == EvalMoment::Snapshot,
                )
                .ok_or_else(|| {
                    GameError::InvalidStaticData(
                        "value-expression rebound capture extent overflowed".to_string(),
                    )
                })?;
                retained_extent = retained_extent
                    .checked_sub(capture.retained_extent)
                    .and_then(|value| value.checked_add(extent))
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "value-expression carrier rebind extent overflowed".to_string(),
                        )
                    })?;
                captures.push(Arc::new(EvalCapture {
                    generation: Arc::clone(&capture.generation),
                    site: capture.site.clone(),
                    fingerprint: capture.fingerprint,
                    kind: capture.kind,
                    plan: Arc::clone(&capture.plan),
                    source: Arc::new(rebound_source),
                    target_at_release: capture.target_at_release,
                    release_time_ms: capture.release_time_ms,
                    deadline_ms: capture.deadline_ms,
                    snapshot_frame: capture.snapshot_frame.clone(),
                    retained_extent: extent,
                }));
            }
            rows.push(Some(EffectCaptureRow { captures }));
        }
        Ok(Arc::new(Self {
            action: self.action.clone(),
            delivery: source.delivery.clone(),
            action_site: self.action_site.clone(),
            effect_ids: Arc::clone(&self.effect_ids),
            resource_ids: Arc::clone(&self.resource_ids),
            rows,
            retained_extent,
        }))
    }
}

#[derive(Debug)]
pub(crate) struct EffectResultPrefix {
    action: ActionHandle,
    delivery: Option<Arc<DeliveryTicket>>,
    action_site: ActionSite,
    effect_ids: Arc<Vec<String>>,
    resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    settled: Vec<Option<EffectResultProjection>>,
    cumulative: EffectResultProjection,
}

impl EffectResultPrefix {
    pub(crate) fn from_capture(
        capture: &ValueExprDeliveryCapture,
        source: &RuntimeFactSource,
        resource_ids: Arc<Vec<crate::game::resource::ResourceId>>,
    ) -> Result<Self, GameError> {
        if source.action.as_ref() != Some(&capture.action) || source.delivery != capture.delivery {
            return Err(GameError::InvalidStaticData(
                "value-expression prefix carrier belongs to a different action/delivery occurrence"
                    .to_string(),
            ));
        }
        let resource_count = resource_ids.len();
        let settled_bytes = capture
            .effect_ids
            .len()
            .checked_mul(size_of::<Option<EffectResultProjection>>())
            .and_then(|value| value.checked_add(resource_count.checked_mul(size_of::<i128>())?))
            .and_then(|value| value.checked_add(capture.action_site.definition_id.as_str().len()))
            .and_then(|value| value.checked_add(capture.action_site.action_id.len()))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression prefix allocation accounting overflowed".to_string(),
                )
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(settled_bytes),
            "value_expr_prefix_witness",
        )?;
        let mut settled = Vec::with_capacity(capture.effect_ids.len());
        settled.resize_with(capture.effect_ids.len(), || None);
        Ok(Self {
            action: capture.action.clone(),
            delivery: capture.delivery.clone(),
            action_site: capture.action_site.clone(),
            effect_ids: Arc::clone(&capture.effect_ids),
            resource_ids,
            settled,
            cumulative: EffectResultProjection::zero(resource_count),
        })
    }

    pub(crate) fn resource_ids(&self) -> Arc<Vec<crate::game::resource::ResourceId>> {
        Arc::clone(&self.resource_ids)
    }

    pub(crate) fn settle(
        &mut self,
        effect_index: usize,
        projection: EffectResultProjection,
    ) -> Result<(), GameError> {
        crate::authoring::control::charge(
            self.settled.len() as u64 + self.resource_ids.len() as u64,
            "value_expr_prefix_settle",
        )?;
        if effect_index >= self.settled.len()
            || self.settled[..effect_index].iter().any(Option::is_none)
            || self.settled[effect_index].is_some()
        {
            return Err(GameError::InvalidStaticData(
                "value-expression effect frontier was settled out of order".to_string(),
            ));
        }
        self.cumulative.checked_merge(&projection).ok_or_else(|| {
            GameError::InvalidStaticData(
                "value-expression completed-prefix projection overflowed".to_string(),
            )
        })?;
        self.settled[effect_index] = Some(projection);
        Ok(())
    }

    fn value(
        &self,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
        current_effect_index: usize,
        selector: &ResultSelector,
        field: &ResultField,
        node_index: usize,
    ) -> Result<TypedInteger, ValueExprFault> {
        if source.action.as_ref() != Some(&self.action)
            || source.delivery != self.delivery
            || plan.source().site.action != self.action_site
            || self
                .effect_ids
                .get(current_effect_index)
                .map(String::as_str)
                != Some(plan.source().site.effect_id.as_str())
        {
            return Err(fault(
                plan,
                ValueExprFaultCode::DifferentAttempt,
                node_index,
                "completed-prefix witness does not match this action, delivery, or effect occurrence",
            ));
        }
        let projection = match selector {
            ResultSelector::EarlierEffect(effect_id) => {
                let Some(index) = self
                    .effect_ids
                    .iter()
                    .position(|candidate| candidate == effect_id)
                else {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        node_index,
                        format!("EarlierEffect('{effect_id}') is absent from this action"),
                    ));
                };
                if index >= current_effect_index {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::FutureResult,
                        node_index,
                        format!("effect {index} is not before effect {current_effect_index}"),
                    ));
                }
                crate::authoring::control::charge(
                    self.settled.len() as u64,
                    "value_expr_prefix_frontier",
                )
                .map_err(|error| control_fault(plan, error, "prefix frontier"))?;
                if self.settled[..current_effect_index]
                    .iter()
                    .any(Option::is_none)
                    || self.settled[current_effect_index..]
                        .iter()
                        .any(Option::is_some)
                {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::NotSettled,
                        node_index,
                        "witness is not at the exact preceding effect frontier",
                    ));
                }
                self.settled[index].as_ref().ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::NotSettled,
                        node_index,
                        format!("EarlierEffect('{effect_id}') has no completed witness"),
                    )
                })?
            }
            ResultSelector::CurrentActionPrefix => {
                crate::authoring::control::charge(
                    current_effect_index as u64,
                    "value_expr_prefix_frontier",
                )
                .map_err(|error| control_fault(plan, error, "prefix frontier"))?;
                if self.settled[..current_effect_index]
                    .iter()
                    .any(Option::is_none)
                {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::NotSettled,
                        node_index,
                        "current effect has no complete preceding frontier",
                    ));
                }
                return Err(fault(
                    plan,
                    ValueExprFaultCode::Unavailable,
                    node_index,
                    "CurrentActionPrefix requires an action-wide witness; a delivery-local prefix is insufficient",
                ));
            }
        };
        projected_result_value(
            projection,
            &self.resource_ids,
            plan,
            selector,
            field,
            node_index,
        )
    }
}

/// Immutable release record. The plan and frame are shared owners; cloning a
/// carrier never walks either graph.
#[derive(Debug)]
pub(crate) struct EvalCapture {
    generation: Arc<ContentIdentity>,
    site: crate::game::ability::authoring::EffectSite,
    fingerprint: [u8; 32],
    kind: AmountPlanKind,
    plan: Arc<CompiledAmountPlan>,
    source: Arc<RuntimeFactSource>,
    target_at_release: Option<UnitInstanceId>,
    release_time_ms: u64,
    deadline_ms: Option<u64>,
    snapshot_frame: Option<Arc<EvalFrame>>,
    retained_extent: usize,
}

impl EvalCapture {
    pub(crate) fn retained_extent(&self) -> usize {
        self.retained_extent
    }

    pub(super) fn source(&self) -> &RuntimeFactSource {
        &self.source
    }

    pub(crate) fn plan(&self) -> &CompiledAmountPlan {
        &self.plan
    }
}

fn fault(
    plan: &CompiledAmountPlan,
    code: ValueExprFaultCode,
    node_index: usize,
    cause: impl Into<String>,
) -> ValueExprFault {
    ValueExprFault {
        code,
        site: plan.source().site.clone(),
        node_index,
        expected_unit: None,
        actual_unit: None,
        phase: ValueExprPhase::Evaluate,
        cause: cause.into(),
    }
}

fn reference_node(plan: &CompiledAmountPlan, reference: &ValueRef) -> usize {
    (0..plan.segment_count())
        .filter_map(|segment| plan.segment(segment))
        .flat_map(|segment| segment.iter())
        .find_map(|operation| match operation {
            crate::game::value_expr::CompiledOp::Read {
                reference: candidate,
                node_index,
            } if candidate == reference => Some(*node_index),
            _ => None,
        })
        .unwrap_or(0)
}

fn frame_reference_node(
    plan: &CompiledAmountPlan,
    reference: &ValueRef,
) -> Result<usize, ValueExprFault> {
    for segment_index in 0..plan.segment_count() {
        if let Some(segment) = plan.segment(segment_index) {
            for operation in segment {
                crate::authoring::control::charge(1, "value_expr_frame_plan_lookup")
                    .map_err(|error| control_fault(plan, error, "frame plan lookup"))?;
                if let crate::game::value_expr::CompiledOp::Read {
                    reference: candidate,
                    node_index,
                } = operation
                {
                    if candidate == reference {
                        return Ok(*node_index);
                    }
                }
            }
        }
    }
    Ok(0)
}

fn evaluation_host_admission(
    plan: &CompiledAmountPlan,
    node_index: usize,
    work: u64,
    retained: usize,
) -> Result<(), ValueExprFault> {
    let map_error = |error, site| {
        let mut diagnostic = control_fault(plan, error, site);
        diagnostic.node_index = node_index;
        diagnostic
    };
    crate::authoring::control::poll("value_expr_evaluation_poll")
        .map_err(|error| map_error(error, "evaluation poll"))?;
    if work != 0 {
        crate::authoring::control::charge(work, "value_expr_evaluation")
            .map_err(|error| map_error(error, "evaluation work"))?;
    }
    if retained != 0 {
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(retained),
            "value_expr_evaluation_retained",
        )
        .map_err(|error| map_error(error, "evaluation retained"))?;
    }
    Ok(())
}

/// Shared numerical kernel used after a runtime has already selected and
/// validated the exact plan and frame. Recorded queries deliberately enter
/// here instead of recreating runtime identities or capabilities.
pub(crate) fn evaluate_compiled_amount(
    plan: &CompiledAmountPlan,
    kind: AmountPlanKind,
    frame: &EvalFrame,
) -> Result<RuntimeAmountEvaluation, ValueExprFault> {
    let mut meter = ExprMeter::new(plan.evaluation_limits())
        .map_err(|cause| {
            fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                format!("compiled evaluation limits are invalid: {cause}"),
            )
        })?
        .with_host_admission(evaluation_host_admission);
    let evaluated = evaluate(plan, frame, &mut meter);
    crate::authoring::control::poll("value_expr_runtime_complete")
        .map_err(|error| control_fault(plan, error, "evaluation completion"))?;
    let evaluated = evaluated?;
    let amount = match kind {
        AmountPlanKind::Damage => {
            RuntimeAmount::Damage(narrow_damage(&plan.source().site, &evaluated.value)?)
        }
        AmountPlanKind::Heal => {
            RuntimeAmount::Heal(narrow_heal(&plan.source().site, &evaluated.value)?)
        }
        AmountPlanKind::Shield => {
            RuntimeAmount::Shield(narrow_damage(&plan.source().site, &evaluated.value)?)
        }
    };
    Ok(RuntimeAmountEvaluation {
        amount,
        evaluated,
        usage: meter.usage(),
        observation: None,
    })
}

fn control_fault(
    plan: &CompiledAmountPlan,
    error: GameError,
    site: &'static str,
) -> ValueExprFault {
    let code = match &error {
        GameError::InvalidStaticData(cause) if cause.starts_with("authoring_cancelled") => {
            ValueExprFaultCode::Cancelled
        }
        GameError::InvalidStaticData(cause)
            if cause.starts_with("authoring_work_limit")
                || cause.starts_with("authoring_retained") =>
        {
            ValueExprFaultCode::BudgetExceeded
        }
        GameError::BattleRuntimeFault(runtime)
            if matches!(
                runtime.code,
                crate::game::battle::runtime_control::RuntimeFaultCode::WorkLimit
                    | crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit
                    | crate::game::battle::runtime_control::RuntimeFaultCode::CloneLimit
            ) =>
        {
            ValueExprFaultCode::BudgetExceeded
        }
        _ => ValueExprFaultCode::Unavailable,
    };
    fault(
        plan,
        code,
        0,
        format!("{site} rejected while preparing value-expression context: {error:?}"),
    )
}

fn resource_fault(
    plan: &CompiledAmountPlan,
    reference: &ValueRef,
    error: ResourceError,
) -> ValueExprFault {
    let code = match &error {
        ResourceError::Fault(
            ResourceFaultCode::InvalidOwner | ResourceFaultCode::OwnerKindMismatch,
        ) => ValueExprFaultCode::InvalidOwner,
        ResourceError::Fault(
            ResourceFaultCode::StaleScope
            | ResourceFaultCode::StaleLease
            | ResourceFaultCode::StaleRevision,
        ) => ValueExprFaultCode::StaleLease,
        ResourceError::Fault(ResourceFaultCode::Limit | ResourceFaultCode::CounterOverflow) => {
            ValueExprFaultCode::BudgetExceeded
        }
        ResourceError::Control(error) => {
            return control_fault(plan, error.clone(), "resource value read")
        }
        ResourceError::Definition(_)
        | ResourceError::Fault(
            ResourceFaultCode::UnknownDefinition
            | ResourceFaultCode::WrongLifetime
            | ResourceFaultCode::StaleReservation
            | ResourceFaultCode::Insufficient
            | ResourceFaultCode::ReservationConflict
            | ResourceFaultCode::Invariant,
        ) => ValueExprFaultCode::MissingContext,
    };
    fault(
        plan,
        code,
        reference_node(plan, reference),
        format!("resource context could not be read: {error:?}"),
    )
}

fn action_fault(
    plan: &CompiledAmountPlan,
    error: ActionQueryError,
    identity: &'static str,
) -> ValueExprFault {
    let code = match error {
        ActionQueryError::DifferentAttempt => ValueExprFaultCode::DifferentAttempt,
        ActionQueryError::NotSettled => ValueExprFaultCode::NotSettled,
        ActionQueryError::ForcedPause
        | ActionQueryError::Unavailable
        | ActionQueryError::StaleOccurrence => ValueExprFaultCode::Unavailable,
    };
    fault(
        plan,
        code,
        0,
        format!("{identity} occurrence is not valid for this battle candidate: {error:?}"),
    )
}

fn capture_extent(
    plan: &CompiledAmountPlan,
    source: &RuntimeFactSource,
    snapshot: bool,
) -> Option<usize> {
    let (_, _, frame_dynamic) = frame_storage_bound(plan)?;
    let reference_slots = plan
        .requirements()
        .references
        .len()
        .checked_mul(size_of::<(ValueRef, TypedInteger)>().checked_add(256)?)?;
    let reference_text = frame_dynamic.checked_mul(2)?;
    size_of::<EvalCapture>()
        .checked_add(64 * 5)?
        .checked_add(plan.cost().plan_retained_bytes)?
        .checked_add(source.retained_extent()?)?
        .checked_add(reference_text)?
        .checked_add(if snapshot {
            size_of::<EvalFrame>()
                .checked_add(reference_slots)?
                // The frame's owned string payload is distinct from the
                // captured plan/source payload above. Its bounded Vec storage
                // fits in reference_slots; this completes the fourfold string
                // bound used for live frame construction.
                .checked_add(frame_dynamic.checked_mul(2)?)?
                .checked_add(128)?
        } else {
            0
        })
}

// Vec::with_capacity(count) requests exactly count * size_of::<T>() bytes.
// The per-reference reserve also covers transient key/unit/resource clones
// made while the frame is populated. No map-node or growth estimate is used.
fn frame_storage_bound(plan: &CompiledAmountPlan) -> Option<(usize, usize, usize)> {
    let mut count = 0usize;
    let mut dynamic = 0usize;
    for reference in &plan.requirements().references {
        if reference.is_member_scoped() {
            continue;
        }
        count = count.checked_add(1)?;
        dynamic = dynamic.checked_add(value_ref_dynamic_bytes(reference)?)?;
    }
    let slots = count.checked_mul(size_of::<(ValueRef, TypedInteger)>().checked_add(256)?)?;
    let bytes = slots.checked_add(dynamic.checked_mul(4)?)?;
    Some((count, bytes, dynamic))
}

#[derive(Clone, Copy)]
struct ResolvedAggregateSpec<'a> {
    source: &'a FiniteSource,
    grouping: Grouping,
    node_index: usize,
    member_segment: usize,
    member_reference_extent: usize,
}

/// Canonical read-only member views for one effect and one evaluation moment.
/// Snapshot frames and per-target frames share these allocations.
#[derive(Debug)]
pub(crate) struct ResolvedMemberFrame {
    views: Arc<Vec<PreparedResolvedTargetView>>,
    retained_extent: usize,
}

impl BattleCore {
    pub(crate) fn prepare_resolved_target_members(
        &self,
        plan: &CompiledAmountPlan,
        targets: Option<&[UnitInstanceId]>,
    ) -> Result<Option<ResolvedMemberFrame>, ValueExprFault> {
        self.prepare_resolved_target_members_with_base(plan, targets, 0)
    }

    fn prepare_resolved_target_members_with_base(
        &self,
        plan: &CompiledAmountPlan,
        targets: Option<&[UnitInstanceId]>,
        retained_base: usize,
    ) -> Result<Option<ResolvedMemberFrame>, ValueExprFault> {
        if !plan.requirements().resolved_targets {
            return Ok(None);
        }
        let targets = targets.ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::UnsupportedMoment,
                0,
                "this evaluation moment has no finalized resolved target set",
            )
        })?;
        let limits = plan.evaluation_limits();
        let mut aggregate_count = 0usize;
        let mut total_member_ref_count = 0usize;
        let mut total_predicate_count = 0usize;
        let mut relation_view_count = 0usize;
        let mut plan_scan_work = 0u64;
        for segment_index in 0..plan.segment_count() {
            for op in plan.segment(segment_index).unwrap_or(&[]) {
                plan_scan_work = plan_scan_work.checked_add(1).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        op.node_index(),
                        "resolved aggregate plan scan work overflowed",
                    )
                })?;
                let CompiledOp::Aggregate {
                    source,
                    grouping,
                    node_index,
                    member_segment,
                    ..
                } = op
                else {
                    continue;
                };
                if matches!(
                    source,
                    FiniteSource::ResolvedTargets | FiniteSource::ResolvedTargetsWhere(_)
                ) {
                    match grouping {
                        Grouping::ByOwner => {
                            return Err(fault(
                                plan,
                                ValueExprFaultCode::InvalidOwner,
                                *node_index,
                                "resolved ByOwner requires a declared owner domain",
                            ))
                        }
                        Grouping::ByResolvedOwner(_) => {
                            relation_view_count =
                                relation_view_count.checked_add(1).ok_or_else(|| {
                                    fault(
                                        plan,
                                        ValueExprFaultCode::Overflow,
                                        *node_index,
                                        "resolved relation view count overflowed",
                                    )
                                })?;
                        }
                        Grouping::ByProjectedTile | Grouping::WholeSet => {}
                    }
                    aggregate_count = aggregate_count.checked_add(1).ok_or_else(|| {
                        fault(
                            plan,
                            ValueExprFaultCode::Overflow,
                            *node_index,
                            "resolved aggregate count overflowed",
                        )
                    })?;
                    total_predicate_count = total_predicate_count
                        .checked_add(match source {
                            FiniteSource::ResolvedTargetsWhere(predicates) => predicates.len(),
                            FiniteSource::AuthoredMembers(_) | FiniteSource::ResolvedTargets => 0,
                        })
                        .ok_or_else(|| {
                            fault(
                                plan,
                                ValueExprFaultCode::Overflow,
                                *node_index,
                                "resolved predicate count overflowed",
                            )
                        })?;
                    for member_op in plan.segment(*member_segment).unwrap_or(&[]) {
                        plan_scan_work = plan_scan_work.checked_add(1).ok_or_else(|| {
                            fault(
                                plan,
                                ValueExprFaultCode::Overflow,
                                *node_index,
                                "resolved member plan scan work overflowed",
                            )
                        })?;
                        if let CompiledOp::Read {
                            reference,
                            node_index: read_node_index,
                        } = member_op
                        {
                            if !reference.is_member_scoped() {
                                continue;
                            }
                            if matches!(reference, ValueRef::AggregateMember { .. }) {
                                return Err(fault(
                                    plan,
                                    ValueExprFaultCode::MissingContext,
                                    *read_node_index,
                                    "resolved units have no implicit authored member scalar",
                                ));
                            }
                            total_member_ref_count =
                                total_member_ref_count.checked_add(1).ok_or_else(|| {
                                    fault(
                                        plan,
                                        ValueExprFaultCode::Overflow,
                                        *read_node_index,
                                        "resolved member reference count overflowed",
                                    )
                                })?;
                        }
                    }
                }
            }
        }
        if aggregate_count == 0 {
            return Err(fault(
                plan,
                ValueExprFaultCode::MissingContext,
                0,
                "compiled requirements declare resolved targets without a resolved aggregate",
            ));
        }
        let raw = targets.len();
        let raw_u64 = u64::try_from(raw).map_err(|_| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved target count overflowed",
            )
        })?;
        let levels = u64::from(usize::BITS - raw.leading_zeros()) + 1;
        let refs_u64 = u64::try_from(total_member_ref_count).map_err(|_| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "member reference count overflowed",
            )
        })?;
        let predicates_u64 = u64::try_from(total_predicate_count).map_err(|_| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved predicate count overflowed the work counter",
            )
        })?;
        let relations_u64 = u64::try_from(relation_view_count).map_err(|_| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved relation count overflowed the work counter",
            )
        })?;
        let aggregates_u64 = u64::try_from(aggregate_count).map_err(|_| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved aggregate count overflowed the work counter",
            )
        })?;
        // Predicate and relation terms are doubled because the count pass and
        // the view construction pass each invoke them at most once per member.
        // Slot invocations perform one exact RuntimeItem map lookup, so these
        // existing logical work terms also bound every equipment lookup.
        let per_raw = levels
            .checked_add(aggregates_u64.checked_mul(3).unwrap_or(u64::MAX))
            .and_then(|value| value.checked_add(predicates_u64.checked_mul(2)?))
            .and_then(|value| value.checked_add(relations_u64.checked_mul(2)?))
            .and_then(|value| value.checked_add(refs_u64))
            .unwrap_or(u64::MAX);
        let work = raw_u64
            .checked_mul(per_raw)
            .and_then(|value| value.checked_add(raw_u64))
            .and_then(|value| value.checked_add(plan_scan_work))
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Overflow,
                    0,
                    "resolved target preparation work overflowed",
                )
            })?;
        if work > limits.max_work {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                "resolved target preparation exceeds the work limit",
            ));
        }
        let scratch_extent = aggregate_count
            .checked_mul(size_of::<ResolvedAggregateSpec<'_>>() + size_of::<usize>())
            .and_then(|value| value.checked_add(raw.checked_mul(size_of::<UnitInstanceId>())?))
            .and_then(|value| value.checked_add(128))
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Overflow,
                    0,
                    "resolved preparation scratch storage overflowed",
                )
            })?;
        if scratch_extent > limits.max_retained_bytes {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                "resolved preparation scratch exceeds the retained limit",
            ));
        }
        let scratch_basis = retained_base.checked_add(scratch_extent).ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved scratch admission basis overflowed",
            )
        })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            retained_base,
            Some(scratch_basis),
            "value_expr_resolved_member_scratch",
        )
        .map_err(|error| control_fault(plan, error, "resolved member scratch admission"))?;
        crate::authoring::control::charge(work, "value_expr_resolved_member_prepare")
            .map_err(|error| control_fault(plan, error, "resolved member work"))?;

        let mut specs = Vec::with_capacity(aggregate_count);
        for segment_index in 0..plan.segment_count() {
            for op in plan.segment(segment_index).unwrap_or(&[]) {
                let CompiledOp::Aggregate {
                    source,
                    grouping,
                    node_index,
                    member_segment,
                    ..
                } = op
                else {
                    continue;
                };
                if !matches!(
                    source,
                    FiniteSource::ResolvedTargets | FiniteSource::ResolvedTargetsWhere(_)
                ) {
                    continue;
                }
                let mut member_reference_extent = 0usize;
                for member_op in plan.segment(*member_segment).unwrap_or(&[]) {
                    let CompiledOp::Read { reference, .. } = member_op else {
                        continue;
                    };
                    if !reference.is_member_scoped() {
                        continue;
                    }
                    member_reference_extent = member_reference_extent
                        .checked_add(size_of::<(ValueRef, TypedInteger)>() + 128)
                        .and_then(|value| value.checked_add(value_ref_dynamic_bytes(reference)?))
                        .ok_or_else(|| {
                            fault(
                                plan,
                                ValueExprFaultCode::Overflow,
                                *node_index,
                                "resolved member reference storage overflowed",
                            )
                        })?;
                }
                specs.push(ResolvedAggregateSpec {
                    source,
                    grouping: *grouping,
                    node_index: *node_index,
                    member_segment: *member_segment,
                    member_reference_extent,
                });
            }
        }
        specs.sort_by_key(|spec| spec.node_index);

        let mut canonical = Vec::with_capacity(raw);
        canonical.extend_from_slice(targets);
        canonical.sort_unstable();
        canonical.dedup();
        if canonical.len() > limits.max_dynamic_members {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                "canonical resolved members exceed the member limit",
            ));
        }
        for identity in &canonical {
            self.units
                .get(identity)
                .filter(|unit| unit.lifecycle.is_active())
                .ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::MissingContext,
                        0,
                        "resolved target is missing or inactive at this moment",
                    )
                })?;
        }

        let mut view_member_counts = Vec::with_capacity(specs.len());
        let mut total_view_members = 0usize;
        for spec in &specs {
            let mut count = 0usize;
            for identity in &canonical {
                let unit = &self.units[identity];
                if !self.resolved_member_passes(plan, spec, unit)? {
                    continue;
                }
                if let Grouping::ByResolvedOwner(relation) = spec.grouping {
                    self.resolved_relation_owner(plan, spec.node_index, unit, relation)?;
                }
                count = count.checked_add(1).ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Overflow,
                        spec.node_index,
                        "resolved view member count overflowed",
                    )
                })?;
            }
            total_view_members = total_view_members.checked_add(count).ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Overflow,
                    spec.node_index,
                    "total resolved view member count overflowed",
                )
            })?;
            view_member_counts.push(count);
        }
        if total_view_members > limits.max_dynamic_members {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                "all resolved aggregate views together exceed the member limit",
            ));
        }

        let retained_extent = specs
            .iter()
            .zip(&view_member_counts)
            .try_fold(
                size_of::<Arc<Vec<PreparedResolvedTargetView>>>()
                    + specs.len() * size_of::<PreparedResolvedTargetView>()
                    + 128,
                |bytes, (spec, count)| {
                    let per_member = size_of::<EvalMember>()
                        .checked_add(36 + 128)?
                        .checked_add(if matches!(spec.grouping, Grouping::ByResolvedOwner(_)) {
                            size_of::<GroupOwner>() + 36
                        } else {
                            0
                        })?
                        .checked_add(spec.member_reference_extent)?;
                    bytes
                        .checked_add(size_of::<Arc<Vec<EvalMember>>>() + 128)?
                        .checked_add(count.checked_mul(per_member)?)
                },
            )
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Overflow,
                    0,
                    "resolved member total storage overflowed",
                )
            })?;
        let peak_extent = scratch_extent.checked_add(retained_extent).ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "resolved member peak storage overflowed",
            )
        })?;
        if peak_extent > limits.max_retained_bytes {
            return Err(fault(
                plan,
                ValueExprFaultCode::BudgetExceeded,
                0,
                "resolved member storage exceeds the retained limit",
            ));
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            scratch_basis,
            retained_base.checked_add(peak_extent),
            "value_expr_resolved_members",
        )
        .map_err(|error| control_fault(plan, error, "resolved member admission"))?;

        let mut views = Vec::with_capacity(specs.len());
        for (spec, member_count) in specs.iter().zip(view_member_counts) {
            let mut members = Vec::with_capacity(member_count);
            for identity in &canonical {
                crate::authoring::control::poll("value_expr_resolved_member_read")
                    .map_err(|error| control_fault(plan, error, "resolved member read"))?;
                let unit = &self.units[identity];
                if !self.resolved_member_passes(plan, spec, unit)? {
                    continue;
                }
                let owner = match spec.grouping {
                    Grouping::ByResolvedOwner(relation) => {
                        Some(self.resolved_relation_owner(plan, spec.node_index, unit, relation)?)
                    }
                    Grouping::WholeSet | Grouping::ByOwner | Grouping::ByProjectedTile => None,
                };
                let mut values = BTreeMap::new();
                for member_op in plan.segment(spec.member_segment).unwrap_or(&[]) {
                    let CompiledOp::Read {
                        reference,
                        node_index,
                    } = member_op
                    else {
                        continue;
                    };
                    if !reference.is_member_scoped() {
                        continue;
                    }
                    let value = self.resolved_member_value(plan, *node_index, unit, reference)?;
                    values.insert(reference.clone(), value);
                }
                let projected_tile = if spec.grouping == Grouping::ByProjectedTile {
                    let position = unit.body.projected_tile();
                    Some(ProjectedTile {
                        x: position.x,
                        y: position.y,
                    })
                } else {
                    None
                };
                members.push(EvalMember {
                    identity: identity.as_uuid().hyphenated().to_string(),
                    value: None,
                    owner,
                    projected_tile,
                    values,
                });
            }
            views.push(PreparedResolvedTargetView {
                node_index: spec.node_index,
                members: Arc::new(members),
            });
        }
        Ok(Some(ResolvedMemberFrame {
            views: Arc::new(views),
            retained_extent: peak_extent,
        }))
    }

    fn resolved_member_passes(
        &self,
        plan: &CompiledAmountPlan,
        spec: &ResolvedAggregateSpec<'_>,
        unit: &super::RuntimeUnit,
    ) -> Result<bool, ValueExprFault> {
        let predicates = match spec.source {
            FiniteSource::ResolvedTargetsWhere(predicates) => predicates.as_slice(),
            FiniteSource::AuthoredMembers(_) | FiniteSource::ResolvedTargets => &[],
        };
        for predicate in predicates {
            let passes = match predicate {
                ResolvedTargetPredicate::IsActualEmployee => self
                    .actual_employee_uuid(plan, spec.node_index, unit)?
                    .is_some(),
                ResolvedTargetPredicate::HasEquippedSlot(address) => self
                    .resolved_slot_binding(plan, spec.node_index, unit, *address)?
                    .is_some(),
            };
            if !passes {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn actual_employee_uuid(
        &self,
        plan: &CompiledAmountPlan,
        node_index: usize,
        unit: &super::RuntimeUnit,
    ) -> Result<Option<uuid::Uuid>, ValueExprFault> {
        match &unit.source_identity {
            crate::game::battle::types::BattleUnitSourceIdentity::Employee {
                employee_uuid,
                ..
            } => {
                if employee_uuid.is_nil() {
                    Err(fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        "resolved Employee source has a nil employee UUID",
                    ))
                } else {
                    Ok(Some(*employee_uuid))
                }
            }
            _ => Ok(None),
        }
    }

    fn resolved_slot_binding<'a>(
        &'a self,
        plan: &CompiledAmountPlan,
        node_index: usize,
        unit: &'a super::RuntimeUnit,
        address: crate::game::resources::item_slot::EquipmentSlotAddress,
    ) -> Result<Option<&'a crate::game::battle::types::BattleEquipmentBinding>, ValueExprFault>
    {
        let slots = unit.equipment_slots.as_deref().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                "resolved unit has no equipment-layout projection",
            )
        })?;
        if !address.is_valid() || !address.belongs_to(slots.layout) {
            return Err(fault(
                plan,
                ValueExprFaultCode::InvalidOwner,
                node_index,
                format!(
                    "equipment slot {address:?} does not belong to layout {:?}",
                    slots.layout
                ),
            ));
        }
        let index = match address {
            crate::game::resources::item_slot::EquipmentSlotAddress::Weapon
            | crate::game::resources::item_slot::EquipmentSlotAddress::Any3(0) => 0,
            crate::game::resources::item_slot::EquipmentSlotAddress::Armor
            | crate::game::resources::item_slot::EquipmentSlotAddress::Any3(1) => 1,
            crate::game::resources::item_slot::EquipmentSlotAddress::Accessory
            | crate::game::resources::item_slot::EquipmentSlotAddress::Any3(2) => 2,
            crate::game::resources::item_slot::EquipmentSlotAddress::Any3(_) => unreachable!(),
        };
        let Some(entry) = slots.slots[index].as_ref() else {
            return Ok(None);
        };
        let binding = &entry.binding;
        if binding.slot != Some(address) {
            return Err(fault(
                plan,
                ValueExprFaultCode::InvalidOwner,
                node_index,
                "equipment slot projection contains a mismatched binding address",
            ));
        }
        if let Some(copy_uuid) = binding.persistent_copy_uuid {
            if copy_uuid.is_nil() {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    "equipment slot projection contains a nil actual-copy UUID",
                ));
            }
            if entry.runtime_item_instance_id.is_nil() {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    "equipment slot projection contains a nil runtime item ID",
                ));
            }
            #[cfg(test)]
            record_resolved_slot_item_lookup();
            let runtime_item = self
                .items
                .get(&entry.runtime_item_instance_id)
                .filter(|item| {
                    item.instance_id == entry.runtime_item_instance_id
                        && item.owner_unit_instance == unit.instance_id
                        && item.base_uuid == binding.base_uuid
                        && item.persistent_copy_uuid == Some(copy_uuid)
                });
            if runtime_item.is_none() {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::InvalidOwner,
                    node_index,
                    "equipment slot actual-copy identity is stale or inconsistent",
                ));
            }
        }
        Ok(Some(binding))
    }

    fn resolved_relation_owner(
        &self,
        plan: &CompiledAmountPlan,
        node_index: usize,
        unit: &super::RuntimeUnit,
        relation: ResolvedOwnerRelation,
    ) -> Result<GroupOwner, ValueExprFault> {
        let (domain, identity) = match relation {
            ResolvedOwnerRelation::Side => (
                GroupOwnerDomain::Side,
                match unit.owner {
                    crate::game::enums::Side::Player => "player".to_string(),
                    crate::game::enums::Side::Opponent => "opponent".to_string(),
                },
            ),
            ResolvedOwnerRelation::ActualEmployee => {
                let employee = self
                    .actual_employee_uuid(plan, node_index, unit)?
                    .ok_or_else(|| {
                        fault(
                            plan,
                            ValueExprFaultCode::InvalidOwner,
                            node_index,
                            "resolved unit is not an actual Employee source",
                        )
                    })?;
                (
                    GroupOwnerDomain::Employee,
                    employee.hyphenated().to_string(),
                )
            }
            ResolvedOwnerRelation::EquippedSlot(address) => {
                let binding = self
                    .resolved_slot_binding(plan, node_index, unit, address)?
                    .ok_or_else(|| {
                        fault(
                            plan,
                            ValueExprFaultCode::InvalidOwner,
                            node_index,
                            "resolved equipment relation points to a known empty slot",
                        )
                    })?;
                let copy_uuid = binding.persistent_copy_uuid.ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::InvalidOwner,
                        node_index,
                        "resolved equipment relation has no actual-copy UUID",
                    )
                })?;
                (
                    GroupOwnerDomain::EquipmentCopy,
                    copy_uuid.hyphenated().to_string(),
                )
            }
        };
        Ok(GroupOwner { domain, identity })
    }

    fn resolved_member_value(
        &self,
        plan: &CompiledAmountPlan,
        node_index: usize,
        unit: &super::RuntimeUnit,
        reference: &ValueRef,
    ) -> Result<TypedInteger, ValueExprFault> {
        match reference {
            ValueRef::Stat {
                subject: ValueSubject::AggregateMember,
                stat: StatSelector::Attack,
            } => Ok(TypedInteger::new(
                i128::from(unit.stats.attack),
                ValueUnit::Magnitude,
            )),
            ValueRef::HitPoints {
                subject: ValueSubject::AggregateMember,
                field,
            } => {
                if unit.stats.current_health > unit.stats.max_health {
                    return Err(fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        node_index,
                        "resolved member HP invariant is false",
                    ));
                }
                let value = match field {
                    HitPointField::Current => unit.stats.current_health,
                    HitPointField::Max => unit.stats.max_health,
                    HitPointField::Missing => unit.stats.max_health - unit.stats.current_health,
                };
                Ok(TypedInteger::new(i128::from(value), ValueUnit::Magnitude))
            }
            ValueRef::Stat {
                subject: ValueSubject::AggregateMember,
                stat,
            } => Err(fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                format!("resolved member {stat:?} awaits the WP03 producer"),
            )),
            ValueRef::AggregateMember { .. } => Err(fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                "resolved units have no implicit authored member scalar",
            )),
            _ => Err(fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                "resolved member reference has no declared producer",
            )),
        }
    }
}

impl BattleCore {
    /// Install the one compact action frontier before its first effect. Scan
    /// the entire compiled skill so an earlier literal step can project a
    /// resource requested by a later expression step.
    pub(crate) fn ensure_value_expr_action_witness(
        &mut self,
        skill_id: &crate::game::ability::SkillId,
        source: &RuntimeFactSource,
    ) -> Result<(), GameError> {
        let Some(skill) = self.skill_for_source(skill_id, source) else {
            // Programmatic literal effects can execute outside the authored
            // catalog. They cannot request an action result projection.
            return Ok(());
        };
        if !skill.steps.iter().any(|step| {
            step.effects.iter().any(|effect| {
                matches!(
                    effect.op,
                    SkillEffectDef::DamageExpr { .. }
                        | SkillEffectDef::HealExpr { .. }
                        | SkillEffectDef::ShieldExpr { .. }
                        | SkillEffectDef::PersonalShieldGrant { .. }
                )
            })
        }) {
            return Ok(());
        }
        let mut requested = false;
        let mut resource_count = 0usize;
        let mut resource_bytes = 0usize;
        for step in &skill.steps {
            for effect in &step.effects {
                crate::authoring::control::charge(1, "action_result_skill_scan")?;
                let Some(entry) = effect
                    .source()
                    .and_then(|authored| self.amount_entry_for_source(source, &authored.site))
                else {
                    continue;
                };
                for reference in &entry.plan().requirements().references {
                    crate::authoring::control::charge(1, "action_result_reference_scan")?;
                    if let ValueRef::Result { selector, field } = reference {
                        requested |= matches!(selector, ResultSelector::CurrentActionPrefix);
                        if let ResultField::ResourceChangeDelta(id) = field {
                            resource_count = resource_count.checked_add(1).ok_or_else(|| {
                                GameError::InvalidStaticData(
                                    "action result resource count overflowed".to_string(),
                                )
                            })?;
                            resource_bytes = resource_bytes
                                .checked_add(id.as_str().len())
                                .ok_or_else(|| {
                                    GameError::InvalidStaticData(
                                        "action result resource extent overflowed".to_string(),
                                    )
                                })?;
                        }
                    }
                }
            }
        }
        if !requested {
            return Ok(());
        }
        let action = source.action.as_ref().ok_or_else(|| {
            GameError::InvalidStaticData("action result has no action occurrence".to_string())
        })?;
        self.runtime_facts
            .validate_action_source(action, source)
            .map_err(|error| {
                GameError::InvalidStaticData(format!("action result source is invalid: {error:?}"))
            })?;
        if let Some(witness) =
            self.runtime_facts
                .action_result_witness(action)
                .map_err(|error| {
                    GameError::InvalidStaticData(format!(
                        "action result occurrence is invalid: {error:?}"
                    ))
                })?
        {
            if witness.definition_id() != skill_id {
                return Err(GameError::InvalidStaticData(
                    "action result belongs to another skill definition".to_string(),
                ));
            }
            return Ok(());
        }
        let extent = resource_count
            .checked_mul(size_of::<crate::game::resource::ResourceId>() + size_of::<i128>())
            .and_then(|bytes| bytes.checked_add(resource_bytes))
            .and_then(|bytes| bytes.checked_add(skill_id.as_str().len()))
            .and_then(|bytes| bytes.checked_add(size_of::<ActionResultWitness>() + 128))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "action result allocation accounting overflowed".to_string(),
                )
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(extent),
            "action_result_install",
        )?;
        let sortable_bytes = resource_count
            .checked_mul(size_of::<crate::game::resource::ResourceId>())
            .and_then(|bytes| bytes.checked_add(resource_bytes))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "action result resource sort size overflowed".to_string(),
                )
            })?;
        let sort_levels = u64::from(usize::BITS - resource_count.leading_zeros()) + 1;
        let sort_work = u64::try_from(sortable_bytes)
            .ok()
            .and_then(|bytes| bytes.checked_mul(sort_levels))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "action result resource sort work overflowed".to_string(),
                )
            })?;
        crate::authoring::control::charge(sort_work, "action_result_resource_sort")?;
        let skill_id_work = u64::try_from(skill_id.as_str().len()).map_err(|_| {
            GameError::InvalidStaticData("action result skill ID work overflowed".to_string())
        })?;
        crate::authoring::control::charge(skill_id_work, "action_result_skill_id_clone")?;
        let mut resource_ids = Vec::with_capacity(resource_count);
        for step in &skill.steps {
            for effect in &step.effects {
                crate::authoring::control::charge(1, "action_result_resource_collect")?;
                let Some(entry) = effect
                    .source()
                    .and_then(|authored| self.amount_entry_for_source(source, &authored.site))
                else {
                    continue;
                };
                for reference in &entry.plan().requirements().references {
                    crate::authoring::control::charge(1, "action_result_resource_clone")?;
                    if let ValueRef::Result {
                        field: ResultField::ResourceChangeDelta(id),
                        ..
                    } = reference
                    {
                        resource_ids.push(id.clone());
                    }
                }
            }
        }
        resource_ids.sort_unstable();
        resource_ids.dedup();
        let witness = ActionResultWitness::new(skill_id.clone(), Arc::new(resource_ids));
        self.runtime_facts
            .install_action_result_witness(action, skill_id, witness)
    }

    fn read_value_expr_action_result(
        &self,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
        field: &ResultField,
        node_index: usize,
    ) -> Result<TypedInteger, ValueExprFault> {
        if source.authored.as_deref() != Some(plan.source()) {
            return Err(fault(
                plan,
                ValueExprFaultCode::DifferentAttempt,
                node_index,
                "result source does not match the compiled effect site",
            ));
        }
        let action = source.action.as_ref().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                "result read has no action occurrence",
            )
        })?;
        self.runtime_facts
            .validate_action_source(action, source)
            .map_err(|error| action_fault(plan, error, "action result source"))?;
        if let Some(delivery) = source.delivery.as_deref() {
            self.runtime_facts
                .validate_live_delivery(delivery)
                .map_err(|error| action_fault(plan, error, "action result delivery"))?;
        }
        let witness = self
            .runtime_facts
            .action_result_witness(action)
            .map_err(|error| action_fault(plan, error, "action result"))?
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::NotSettled,
                    node_index,
                    "action result has no completed frontier",
                )
            })?;
        let structural_child = source
            .relic_effective()
            .zip(source.flow.as_ref())
            .is_some_and(|(binding, occurrence)| {
                binding
                    .execution()
                    .flow
                    .side_branches
                    .iter()
                    .map(|branch| branch.root)
                    .min()
                    .is_some_and(|first| occurrence.node_index >= first)
            });
        let selected = if structural_child {
            self.structural_result_witness(source).ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::NotSettled,
                    node_index,
                    "structural occurrence has no proved result frontier",
                )
            })?
        } else {
            witness
        };
        selected.value(plan, field, node_index)
    }

    pub(crate) fn build_value_expr_delivery_capture(
        &self,
        step: &SkillStepDef,
        source: &RuntimeFactSource,
        release_targets: Option<&[UnitInstanceId]>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
    ) -> Result<Option<Arc<ValueExprDeliveryCapture>>, GameError> {
        crate::authoring::control::charge(
            step.effects.len() as u64,
            "value_expr_carrier_effect_scan",
        )?;
        let expression_count = step
            .effects
            .iter()
            .filter(|effect| {
                (self.source_has_relics(source)
                    && matches!(effect.op, SkillEffectDef::Damage { .. }))
                    || matches!(
                        effect.op,
                        SkillEffectDef::DamageExpr { .. }
                            | SkillEffectDef::HealExpr { .. }
                            | SkillEffectDef::ShieldExpr { .. }
                            | SkillEffectDef::PersonalShieldGrant { .. }
                    )
            })
            .count();
        if expression_count == 0 {
            return Ok(None);
        }
        crate::authoring::control::charge(
            step.effects.len() as u64,
            "value_expr_carrier_metadata_scan",
        )?;
        let action = source.action.clone().ok_or_else(|| {
            GameError::InvalidStaticData(
                "value-expression delivery has no action occurrence".to_string(),
            )
        })?;
        let first_source = step
            .effects
            .iter()
            .find_map(AuthoredSkillEffect::source)
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "compiled value-expression step has no authored source metadata".to_string(),
                )
            })?;
        let action_site = &first_source.site.action;
        let effect_ids_bytes = step.effects.iter().try_fold(0usize, |total, effect| {
            total.checked_add(effect.id.len()).ok_or(())
        });
        let captures_per_row = release_targets.map_or(1, |targets| targets.len().max(1));
        let has_footprint = if let Some(binding) = source.relic_effective() {
            crate::authoring::control::charge(
                step.effects.len() as u64,
                "effect_footprint_capture_shape",
            )?;
            step.effects.iter().filter_map(AuthoredSkillEffect::source).filter_map(|effect| binding.effect(&effect.site))
                .any(|effect| effect.recipes.iter().any(|recipe| matches!(recipe,
                    crate::game::relic::structural::authoring::StructuralRecipeDef::DamageFootprint { .. })))
        } else {
            false
        };
        let captures_per_row = if has_footprint && release_targets.is_some() {
            captures_per_row.max(self.units.len().max(1))
        } else {
            captures_per_row
        };
        let row_slots = expression_count
            .checked_mul(captures_per_row)
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression carrier cardinality overflowed".to_string(),
                )
            })?;
        let structural_extent = size_of::<ValueExprDeliveryCapture>()
            .checked_add(
                step.effects
                    .len()
                    .checked_mul(size_of::<Option<EffectCaptureRow>>())
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "value-expression carrier size overflowed".to_string(),
                        )
                    })?,
            )
            .and_then(|value| {
                value.checked_add(row_slots.checked_mul(size_of::<Arc<EvalCapture>>())?)
            })
            .and_then(|value| value.checked_add(effect_ids_bytes.ok()?))
            .and_then(|value| {
                value.checked_add(step.effects.len().checked_mul(size_of::<String>())?)
            })
            .and_then(|value| value.checked_add(action_site.definition_id.as_str().len()))
            .and_then(|value| value.checked_add(action_site.action_id.len()))
            .and_then(|value| value.checked_add(256))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression carrier retained extent overflowed".to_string(),
                )
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(structural_extent),
            "value_expr_delivery_carrier",
        )?;

        // Admit an upper bound before cloning any ID. Sorting is in-place;
        // duplicate removal does not shrink the admitted backing allocation.
        let mut resource_count = 0usize;
        let mut resource_extent = 0usize;
        for effect in &step.effects {
            crate::authoring::control::charge(1, "value_expr_resource_effect_scan")?;
            let Some(authored) = effect.source() else {
                continue;
            };
            let Some(entry) = self.amount_entry_for_source(source, &authored.site) else {
                continue;
            };
            for reference in &entry.plan().requirements().references {
                crate::authoring::control::charge(1, "value_expr_resource_reference_scan")?;
                if let ValueRef::Result {
                    field: ResultField::ResourceChangeDelta(resource),
                    ..
                } = reference
                {
                    resource_count = resource_count.checked_add(1).ok_or_else(|| {
                        GameError::InvalidStaticData("result resource count overflowed".to_string())
                    })?;
                    resource_extent = resource_extent
                        .checked_add(size_of::<crate::game::resource::ResourceId>())
                        .and_then(|value| value.checked_add(resource.as_str().len()))
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(
                                "result resource extent overflowed".to_string(),
                            )
                        })?;
                }
            }
        }
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(resource_extent),
            "value_expr_result_projection",
        )?;
        let sort_work = (resource_extent as u64)
            .checked_mul(u64::from(usize::BITS - resource_count.leading_zeros()) + 1)
            .ok_or_else(|| {
                GameError::InvalidStaticData("result resource sort work overflowed".to_string())
            })?;
        crate::authoring::control::charge(sort_work, "value_expr_resource_sort")?;
        let mut resource_ids = Vec::with_capacity(resource_count);
        for effect in &step.effects {
            crate::authoring::control::charge(1, "value_expr_resource_collect")?;
            let Some(entry) = effect
                .source()
                .and_then(|authored| self.amount_entry_for_source(source, &authored.site))
            else {
                continue;
            };
            for reference in &entry.plan().requirements().references {
                crate::authoring::control::charge(1, "value_expr_resource_clone")?;
                if let ValueRef::Result {
                    field: ResultField::ResourceChangeDelta(resource),
                    ..
                } = reference
                {
                    resource_ids.push(resource.clone());
                }
            }
        }
        resource_ids.sort_unstable();
        resource_ids.dedup();

        let effect_ids = Arc::new(
            step.effects
                .iter()
                .map(|effect| effect.id.clone())
                .collect::<Vec<_>>(),
        );
        let resource_ids = Arc::new(resource_ids);
        let mut retained_extent =
            structural_extent
                .checked_add(resource_extent)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(
                        "value-expression carrier retained extent overflowed".to_string(),
                    )
                })?;
        let mut rows = Vec::with_capacity(step.effects.len());
        for (effect_index, effect) in step.effects.iter().enumerate() {
            let selected_targets = if release_targets.is_some() && has_footprint {
                self.effective_effect_targets(effect_index, effect, step, source, release_time_ms)?
            } else {
                None
            };
            let release_targets = if matches!(effect.op, SkillEffectDef::PersonalShieldGrant { .. })
            {
                None
            } else {
                selected_targets
                    .as_ref()
                    .map(|selection| selection.targets.as_slice())
                    .or(release_targets)
            };
            let captures_per_row = release_targets.map_or(1, |targets| targets.len().max(1));
            let is_expression = matches!(
                effect.op,
                SkillEffectDef::DamageExpr { .. }
                    | SkillEffectDef::HealExpr { .. }
                    | SkillEffectDef::ShieldExpr { .. }
                    | SkillEffectDef::PersonalShieldGrant { .. }
            );
            if !is_expression
                && !(self.source_has_relics(source)
                    && matches!(effect.op, SkillEffectDef::Damage { .. }))
            {
                rows.push(None);
                continue;
            }
            let authored = effect.source().ok_or_else(|| {
                GameError::InvalidStaticData(
                    "compiled value-expression effect has no source metadata".to_string(),
                )
            })?;
            if &authored.site.action != action_site {
                return Err(GameError::InvalidStaticData(
                    "value-expression carrier contains effects from different actions".to_string(),
                ));
            }
            let entry = self
                .amount_entry_for_source(source, &authored.site)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(
                        "compiled value-expression entry is absent from the content generation"
                            .to_string(),
                    )
                })?;
            let prepared = if entry.plan().moment() == EvalMoment::Snapshot
                && entry.plan().requirements().resolved_targets
            {
                let retained_base = self
                    .retained_runtime_fact_extent()?
                    .checked_add(retained_extent)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "resolved snapshot retained basis overflowed".to_string(),
                        )
                    })?;
                self.prepare_resolved_target_members_with_base(
                    entry.plan(),
                    release_targets,
                    retained_base,
                )
                .map_err(GameError::ValueExpressionFault)?
            } else {
                None
            };
            if let Some(members) = &prepared {
                retained_extent = retained_extent
                    .checked_add(members.retained_extent)
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "resolved snapshot member extent overflowed".to_string(),
                        )
                    })?;
            }
            let mut effect_source = source.clone();
            effect_source.authored = Some(Arc::clone(authored));
            let mut captures = Vec::with_capacity(captures_per_row);
            for target_index in 0..captures_per_row {
                let target = release_targets.and_then(|targets| targets.get(target_index).copied());
                let capture = self
                    .capture_value_expr_entry_with_context(
                        entry,
                        &effect_source,
                        target,
                        release_time_ms,
                        deadline_ms,
                        prepared.as_ref(),
                    )
                    .map_err(GameError::ValueExpressionFault)?;
                retained_extent = retained_extent
                    .checked_add(capture.retained_extent())
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(
                            "value-expression carrier retained extent overflowed".to_string(),
                        )
                    })?;
                captures.push(capture);
            }
            rows.push(Some(EffectCaptureRow { captures }));
        }
        Ok(Some(Arc::new(ValueExprDeliveryCapture {
            action,
            delivery: source.delivery.clone(),
            action_site: action_site.clone(),
            effect_ids,
            resource_ids,
            rows,
            retained_extent,
        })))
    }

    pub(crate) fn begin_effect_result_projection(
        &mut self,
        source: RuntimeFactSource,
        resource_ids: Option<Arc<Vec<crate::game::resource::ResourceId>>>,
    ) -> Result<Option<EffectResultAccumulator>, GameError> {
        let bytes = resource_ids
            .as_ref()
            .map_or(0, |ids| ids.len())
            .checked_mul(size_of::<i128>())
            .and_then(|value| value.checked_add(size_of::<EffectResultAccumulator>() + 128))
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression effect projection size overflowed".to_string(),
                )
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            Some(bytes),
            "value_expr_effect_projection",
        )?;
        let resource_ids = resource_ids.unwrap_or_else(|| Arc::new(Vec::new()));
        let projection = EffectResultAccumulator {
            source,
            projection: EffectResultProjection::zero(resource_ids.len()),
            resource_ids,
            overflowed: false,
        };
        Ok(self.runtime_effect_projection.replace(projection))
    }

    pub(crate) fn finish_effect_result_projection(
        &mut self,
        prior: Option<EffectResultAccumulator>,
    ) -> Result<EffectResultProjection, GameError> {
        let current =
            std::mem::replace(&mut self.runtime_effect_projection, prior).ok_or_else(|| {
                GameError::InvalidStaticData(
                    "value-expression effect projection was not active".to_string(),
                )
            })?;
        if current.overflowed {
            return Err(GameError::InvalidStaticData(
                "value-expression effect result projection overflowed".to_string(),
            ));
        }
        Ok(current.projection)
    }

    pub(crate) fn observe_effect_damage_result(
        &mut self,
        source: &RuntimeFactSource,
        hp_lost: u32,
        first: bool,
        first_positive: bool,
    ) {
        if let Some(projection) = &mut self.runtime_effect_projection {
            projection.observe_damage(
                &self.runtime_fact_source,
                source,
                hp_lost,
                first,
                first_positive,
            );
        }
    }

    pub(crate) fn observe_effect_heal_result(&mut self, source: &RuntimeFactSource, delta: i64) {
        if let Some(projection) = &mut self.runtime_effect_projection {
            projection.observe_heal(&self.runtime_fact_source, source, delta);
        }
    }

    pub(crate) fn observe_effect_resource_result(
        &mut self,
        source: &RuntimeFactSource,
        resource: &crate::game::battle::runtime_facts::FactResource,
        delta: i64,
    ) {
        if let Some(projection) = &mut self.runtime_effect_projection {
            projection.observe_resource(&self.runtime_fact_source, source, resource, delta);
        }
    }

    pub(crate) fn capture_value_expr_entry(
        &self,
        entry: &CompiledAmountEntry,
        source: &RuntimeFactSource,
        target_at_release: Option<UnitInstanceId>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
    ) -> Result<Arc<EvalCapture>, ValueExprFault> {
        self.capture_value_expr_entry_with_context(
            entry,
            source,
            target_at_release,
            release_time_ms,
            deadline_ms,
            None,
        )
    }

    fn capture_value_expr_entry_with_context(
        &self,
        entry: &CompiledAmountEntry,
        source: &RuntimeFactSource,
        target_at_release: Option<UnitInstanceId>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
        members: Option<&ResolvedMemberFrame>,
    ) -> Result<Arc<EvalCapture>, ValueExprFault> {
        self.capture_value_expr_entry_with_prefix(
            entry,
            source,
            target_at_release,
            release_time_ms,
            deadline_ms,
            None,
            usize::MAX,
            members,
        )
    }

    pub(crate) fn capture_value_expr_entry_with_prefix(
        &self,
        entry: &CompiledAmountEntry,
        source: &RuntimeFactSource,
        target_at_release: Option<UnitInstanceId>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
        members: Option<&ResolvedMemberFrame>,
    ) -> Result<Arc<EvalCapture>, ValueExprFault> {
        let plan = entry.plan();
        let current = self
            .amount_entry_for_source(source, &plan.source().site)
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::Unavailable,
                    0,
                    "compiled entry site is absent from this battle content generation",
                )
            })?;
        if current.kind() != entry.kind() || !Arc::ptr_eq(current.plan(), entry.plan()) {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "compiled entry is not the exact plan owned by this battle content generation",
            ));
        }
        self.capture_value_expr_plan_with_prefix(
            entry.kind(),
            Arc::clone(entry.plan()),
            source,
            target_at_release,
            release_time_ms,
            deadline_ms,
            prefix,
            effect_index,
            members,
        )
    }

    #[cfg(test)]
    fn capture_value_expr_plan(
        &self,
        kind: AmountPlanKind,
        plan: Arc<CompiledAmountPlan>,
        source: &RuntimeFactSource,
        target_at_release: Option<UnitInstanceId>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
    ) -> Result<Arc<EvalCapture>, ValueExprFault> {
        self.capture_value_expr_plan_with_prefix(
            kind,
            plan,
            source,
            target_at_release,
            release_time_ms,
            deadline_ms,
            None,
            usize::MAX,
            None,
        )
    }

    fn capture_value_expr_plan_with_prefix(
        &self,
        kind: AmountPlanKind,
        plan: Arc<CompiledAmountPlan>,
        source: &RuntimeFactSource,
        target_at_release: Option<UnitInstanceId>,
        release_time_ms: u64,
        deadline_ms: Option<u64>,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
        members: Option<&ResolvedMemberFrame>,
    ) -> Result<Arc<EvalCapture>, ValueExprFault> {
        crate::authoring::control::poll("value_expr_capture_entry")
            .map_err(|error| control_fault(&plan, error, "capture entry"))?;
        let generation = self.content_identity.as_ref().cloned().ok_or_else(|| {
            fault(
                &plan,
                ValueExprFaultCode::MissingContext,
                0,
                "battle candidate has no validated content-generation identity",
            )
        })?;
        let authored = source.authored.as_deref().ok_or_else(|| {
            fault(
                &plan,
                ValueExprFaultCode::MissingContext,
                0,
                "runtime effect source has no authored source identity",
            )
        })?;
        if let Some(binding) = source.relic_effective() {
            binding
                .validate(&generation, source)
                .map_err(|error| control_fault(&plan, error, "effective source binding"))?;
            let entry = binding
                .execution()
                .amounts
                .get(&plan.source().site)
                .ok_or_else(|| {
                    fault(
                        &plan,
                        ValueExprFaultCode::Unavailable,
                        0,
                        "effective plan-local entry absent",
                    )
                })?;
            if entry.kind() != kind || !Arc::ptr_eq(entry.plan(), &plan) {
                return Err(fault(
                    &plan,
                    ValueExprFaultCode::Unavailable,
                    0,
                    "effective compiled entry is not the exact pinned plan",
                ));
            }
        }
        if authored != plan.source() {
            return Err(fault(
                &plan,
                ValueExprFaultCode::Unavailable,
                0,
                "runtime effect source does not match the compiled plan site/source",
            ));
        }
        let action = source.action.as_ref().ok_or_else(|| {
            fault(
                &plan,
                ValueExprFaultCode::MissingContext,
                0,
                "runtime effect source has no action occurrence",
            )
        })?;
        let accepted_provider = self
            .runtime_facts
            .validate_action_source(action, source)
            .map_err(|error| action_fault(&plan, error, "action source"))?;
        // Some child/proc effects acquire a provider after action acceptance.
        // In that case the accepted action has no provider snapshot; bind the
        // claimed copy to the actor's actual runtime item instead.
        if accepted_provider.is_none() {
            if let Some(provider) = source.equipment {
                let valid = self
                    .items
                    .get(&provider.runtime_item_instance_id)
                    .is_some_and(|item| {
                        Some(item.owner_unit_instance) == source.actor
                            && item.source_snapshot() == provider
                    });
                if !valid {
                    return Err(fault(
                        &plan,
                        ValueExprFaultCode::Unavailable,
                        0,
                        "effect provider is not an item copy owned by the action actor",
                    ));
                }
            }
        }
        if let Some(delivery) = source.delivery.as_deref() {
            if delivery.action != *action {
                return Err(fault(
                    &plan,
                    ValueExprFaultCode::Unavailable,
                    0,
                    "delivery occurrence belongs to a different action",
                ));
            }
            self.runtime_facts
                .validate_live_delivery(delivery)
                .map_err(|error| action_fault(&plan, error, "delivery"))?;
        }
        if source.actor.is_none() {
            return Err(fault(
                &plan,
                ValueExprFaultCode::MissingContext,
                0,
                "runtime effect source has no source actor identity",
            ));
        }

        // Original catalog/source/action/provider gates run first.
        let relic_source;
        let source = if kind == AmountPlanKind::Damage
            && source.relic_effective().is_none()
            && self.source_has_relics(source)
        {
            relic_source = self
                .bind_skill_relic_source(kind, &plan, source)
                .map_err(|error| control_fault(&plan, error, "relic numeric binding"))?;
            &relic_source
        } else {
            source
        };

        let snapshot = plan.moment() == EvalMoment::Snapshot;
        let retained_extent = capture_extent(&plan, source, snapshot).ok_or_else(|| {
            fault(
                &plan,
                ValueExprFaultCode::Overflow,
                0,
                "capture retained-extent accounting overflowed",
            )
        })?;
        let current_extent = self
            .retained_runtime_fact_extent()
            .map_err(|error| control_fault(&plan, error, "capture basis"))?;
        crate::game::battle::runtime_control::admit_retained_growth(
            current_extent,
            current_extent.checked_add(retained_extent),
            "value_expr_capture",
        )
        .map_err(|error| control_fault(&plan, error, "capture admission"))?;

        let snapshot_frame = if snapshot {
            Some(Arc::new(self.build_value_expr_frame_with_members(
                &plan,
                source,
                target_at_release,
                release_time_ms,
                deadline_ms,
                prefix,
                effect_index,
                members,
            )?))
        } else {
            None
        };
        let capture = EvalCapture {
            generation,
            site: plan.source().site.clone(),
            fingerprint: plan.semantic_fingerprint(),
            kind,
            plan,
            source: Arc::new(source.clone()),
            target_at_release,
            release_time_ms,
            deadline_ms,
            snapshot_frame,
            retained_extent,
        };
        let capture = Arc::new(capture);
        crate::authoring::control::poll("value_expr_capture_complete")
            .map_err(|error| control_fault(&capture.plan, error, "capture completion"))?;
        Ok(capture)
    }

    #[cfg(test)]
    pub(crate) fn evaluate_value_expr_capture(
        &self,
        capture: &EvalCapture,
        target_at_consume: Option<UnitInstanceId>,
        consume_time_ms: u64,
    ) -> Result<RuntimeAmountEvaluation, ValueExprFault> {
        self.evaluate_value_expr_capture_with_prefix(
            capture,
            target_at_consume,
            consume_time_ms,
            None,
            usize::MAX,
        )
    }

    pub(crate) fn evaluate_value_expr_capture_with_prefix(
        &self,
        capture: &EvalCapture,
        target_at_consume: Option<UnitInstanceId>,
        consume_time_ms: u64,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
    ) -> Result<RuntimeAmountEvaluation, ValueExprFault> {
        self.evaluate_value_expr_capture_with_resolved(
            capture,
            target_at_consume,
            consume_time_ms,
            prefix,
            effect_index,
            None,
        )
    }

    pub(crate) fn evaluate_value_expr_capture_with_resolved(
        &self,
        capture: &EvalCapture,
        target_at_consume: Option<UnitInstanceId>,
        consume_time_ms: u64,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
        members: Option<&ResolvedMemberFrame>,
    ) -> Result<RuntimeAmountEvaluation, ValueExprFault> {
        let plan = capture.plan.as_ref();
        crate::authoring::control::poll("value_expr_runtime_entry")
            .map_err(|error| control_fault(plan, error, "evaluation entry"))?;
        let generation = self.content_identity.as_deref().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                0,
                "battle candidate has no validated content-generation identity",
            )
        })?;
        if consume_time_ms < capture.release_time_ms {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "consume time precedes the capture release time",
            ));
        }
        if generation != capture.generation.as_ref() {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "capture belongs to a different content generation",
            ));
        }
        if capture.site != plan.source().site || capture.fingerprint != plan.semantic_fingerprint()
        {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "capture site/fingerprint does not match its compiled plan",
            ));
        }
        let action = capture.source.action.as_ref().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                0,
                "capture has no action occurrence",
            )
        })?;
        self.runtime_facts
            .validate_live_action(action)
            .map_err(|error| action_fault(plan, error, "action"))?;
        if let Some(delivery) = capture.source.delivery.as_deref() {
            self.runtime_facts
                .validate_live_delivery(delivery)
                .map_err(|error| action_fault(plan, error, "delivery"))?;
        }

        let live_target = match (capture.target_at_release, target_at_consume) {
            (Some(released), Some(consumed)) if released != consumed => {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::Unavailable,
                    0,
                    "consume target differs from the target fixed at release",
                ));
            }
            (Some(released), _) => Some(released),
            (None, consumed) => consumed,
        };
        let frame = match plan.moment() {
            EvalMoment::Snapshot => {
                Arc::clone(capture.snapshot_frame.as_ref().ok_or_else(|| {
                    fault(
                        plan,
                        ValueExprFaultCode::Unavailable,
                        0,
                        "Snapshot capture has no release frame",
                    )
                })?)
            }
            EvalMoment::Live => Arc::new(self.build_value_expr_frame_with_members(
                plan,
                &capture.source,
                live_target,
                consume_time_ms,
                capture.deadline_ms,
                prefix,
                effect_index,
                members,
            )?),
        };
        let mut result = evaluate_compiled_amount(plan, capture.kind, frame.as_ref())?;
        if let Some(binding) = capture.source.relic_effective() {
            binding
                .validate(generation, &capture.source)
                .map_err(|error| control_fault(plan, error, "effective amount binding"))?;
            result.amount = binding
                .compose_amount(&plan.source().site, capture.kind, result.amount)
                .map_err(|error| control_fault(plan, error, "effective amount composition"))?;
        }
        if let Some(binding) = capture.source.relic_numeric() {
            binding.validate_skill(generation, plan, &capture.source)?;
            crate::authoring::control::charge(
                binding.contributions.len() as u64,
                "relic_numeric_compose",
            )
            .map_err(|error| control_fault(plan, error, "relic damage composition"))?;
            result.amount = binding.apply_amount(plan, result.amount)?;
        }

        if self.amount_observation_enabled() {
            result.observation = Some(RuntimeAmountObservation {
                generation: Arc::clone(&capture.generation),
                source: capture.source.as_ref().clone(),
                release_target: capture.target_at_release,
                consumption_target: target_at_consume,
                release_time_ms: capture.release_time_ms,
                consume_time_ms,
                effect_index,
                plan: Arc::clone(&capture.plan),
                frame,
            });
        }
        Ok(result)
    }

    pub(crate) fn evaluate_legacy_amount_for_observation(
        &self,
        entry: &CompiledAmountEntry,
        source: &RuntimeFactSource,
        target: UnitInstanceId,
        release_time_ms: u64,
        consume_time_ms: u64,
        effect_index: usize,
    ) -> Result<RuntimeAmountEvaluation, ValueExprFault> {
        let plan = entry.plan();
        if entry.is_authored_expression() || !can_evaluate_without_external_context(plan) {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "legacy observation requires its published context-free literal plan",
            ));
        }
        let authored = source.authored.as_deref().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                0,
                "legacy observed effect has no authored source identity",
            )
        })?;
        if authored != plan.source() {
            return Err(fault(
                plan,
                ValueExprFaultCode::Unavailable,
                0,
                "legacy observed source does not match its compiled plan",
            ));
        }
        let generation = self.content_identity.as_ref().cloned().ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                0,
                "observed battle has no content-generation identity",
            )
        })?;
        let frame = Arc::new(EvalFrame::default());
        let mut result = evaluate_compiled_amount(plan, entry.kind(), frame.as_ref())?;
        result.observation = Some(RuntimeAmountObservation {
            generation,
            source: source.clone(),
            release_target: Some(target),
            consumption_target: Some(target),
            release_time_ms,
            consume_time_ms,
            effect_index,
            plan: Arc::clone(entry.plan()),
            frame,
        });
        Ok(result)
    }

    #[cfg(test)]
    fn build_value_expr_frame(
        &self,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
        target: Option<UnitInstanceId>,
        now_ms: u64,
        deadline_ms: Option<u64>,
    ) -> Result<EvalFrame, ValueExprFault> {
        self.build_value_expr_frame_with_prefix(
            plan,
            source,
            target,
            now_ms,
            deadline_ms,
            None,
            usize::MAX,
        )
    }

    fn build_value_expr_frame_with_prefix(
        &self,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
        target: Option<UnitInstanceId>,
        now_ms: u64,
        deadline_ms: Option<u64>,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
    ) -> Result<EvalFrame, ValueExprFault> {
        self.build_value_expr_frame_with_members(
            plan,
            source,
            target,
            now_ms,
            deadline_ms,
            prefix,
            effect_index,
            None,
        )
    }

    fn build_value_expr_frame_with_members(
        &self,
        plan: &CompiledAmountPlan,
        source: &RuntimeFactSource,
        target: Option<UnitInstanceId>,
        now_ms: u64,
        deadline_ms: Option<u64>,
        prefix: Option<&EffectResultPrefix>,
        effect_index: usize,
        members: Option<&ResolvedMemberFrame>,
    ) -> Result<EvalFrame, ValueExprFault> {
        if plan.requirements().resolved_targets && members.is_none() {
            return Err(fault(
                plan,
                ValueExprFaultCode::UnsupportedMoment,
                0,
                "this evaluation moment has no finalized resolved target set",
            ));
        }
        let (frame_capacity, frame_bytes, _) = frame_storage_bound(plan).ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::Overflow,
                0,
                "frame allocation size overflowed",
            )
        })?;
        if plan.moment() == EvalMoment::Live {
            crate::game::battle::runtime_control::admit_retained_growth(
                0,
                Some(frame_bytes),
                "value_expr_live_frame",
            )
            .map_err(|error| control_fault(plan, error, "live frame admission"))?;
        }
        let mut frame = EvalFrame::with_value_capacity(frame_capacity);
        if let Some(members) = members {
            frame.set_resolved_target_views_shared(Arc::clone(&members.views));
        }
        for reference in &plan.requirements().references {
            crate::authoring::control::charge(1, "value_expr_frame_reference")
                .map_err(|error| control_fault(plan, error, "frame reference"))?;
            if reference.is_member_scoped() {
                continue;
            }
            let node_index = frame_reference_node(plan, reference)?;
            crate::authoring::control::charge(1, "value_expr_frame_read")
                .map_err(|error| control_fault(plan, error, "frame read"))?;
            let value = match reference {
                ValueRef::Stat { subject, stat } => {
                    let unit = self.value_expr_unit(plan, *subject, source, target, node_index)?;
                    match stat {
                        StatSelector::Attack => {
                            TypedInteger::new(i128::from(unit.stats.attack), ValueUnit::Magnitude)
                        }
                        StatSelector::SpellPower => TypedInteger::new(
                            i128::from(unit.stats.spell_power),
                            ValueUnit::Magnitude,
                        ),
                        StatSelector::AttackSpeedPoints => TypedInteger::new(
                            i128::from(unit.basic_attack_speed_percent),
                            ValueUnit::AttackSpeedPoints,
                        ),
                    }
                }
                ValueRef::HitPoints { subject, field } => {
                    let unit = self.value_expr_unit(plan, *subject, source, target, node_index)?;
                    if unit.stats.current_health > unit.stats.max_health {
                        return Err(fault(
                            plan,
                            ValueExprFaultCode::Unavailable,
                            node_index,
                            "runtime HP invariant current <= max is false",
                        ));
                    }
                    let value = match field {
                        HitPointField::Current => unit.stats.current_health,
                        HitPointField::Max => unit.stats.max_health,
                        HitPointField::Missing => unit.stats.max_health - unit.stats.current_health,
                    };
                    TypedInteger::new(i128::from(value), ValueUnit::Magnitude)
                }
                ValueRef::Resource {
                    resource,
                    owner,
                    field,
                } => {
                    let subject = match owner {
                        ResourceOwnerSelector::TargetEmployee => ValueSubject::Target,
                        ResourceOwnerSelector::SourceEmployee
                        | ResourceOwnerSelector::SourceEquipment
                        | ResourceOwnerSelector::SourceFragment => ValueSubject::Source,
                    };
                    self.value_expr_unit(plan, subject, source, target, node_index)?;
                    let lease = self
                        .resolve_typed_resource_lease(resource, *owner, source, target)
                        .map_err(|error| resource_fault(plan, reference, error))?;
                    let runtime = self.runtime_facts.resources.as_ref().ok_or_else(|| {
                        fault(
                            plan,
                            ValueExprFaultCode::MissingContext,
                            node_index,
                            "runtime resources were not prepared by the owning entry",
                        )
                    })?;
                    let current = runtime
                        .store
                        .value(&lease)
                        .map_err(|error| resource_fault(plan, reference, error))?;
                    let value = match field {
                        ResourceValueField::Balance => current.balance,
                        ResourceValueField::Reserved => current.reserved,
                        ResourceValueField::Available => current.available(),
                    };
                    TypedInteger::new(i128::from(value), ValueUnit::Resource(resource.clone()))
                }
                ValueRef::Time(TimeValue::BattleElapsed) => {
                    TypedInteger::new(i128::from(now_ms), ValueUnit::Milliseconds)
                }
                ValueRef::Time(TimeValue::RemainingTime) => {
                    if let Some(context) = source.reinput() {
                        TypedInteger::new(i128::from(context.remaining_ms), ValueUnit::Milliseconds)
                    } else {
                        let deadline = deadline_ms.ok_or_else(|| {
                            fault(
                                plan,
                                ValueExprFaultCode::MissingContext,
                                node_index,
                                "RemainingTime requires an explicit action/delivery deadline",
                            )
                        })?;
                        TypedInteger::new(
                            i128::from(deadline.saturating_sub(now_ms)),
                            ValueUnit::Milliseconds,
                        )
                    }
                }
                ValueRef::Result { selector, field } => match selector {
                    ResultSelector::CurrentActionPrefix => {
                        self.read_value_expr_action_result(plan, source, field, node_index)?
                    }
                    ResultSelector::EarlierEffect(_) => prefix
                        .ok_or_else(|| {
                            fault(
                                plan,
                                ValueExprFaultCode::NotSettled,
                                node_index,
                                format!(
                                    "result selector {selector:?} has no completed-prefix witness"
                                ),
                            )
                        })?
                        .value(plan, source, effect_index, selector, field, node_index)?,
                },
                ValueRef::AggregateMember { .. } => continue,
            };
            frame
                .insert_bounded(reference.clone(), value, frame_capacity)
                .map_err(|cause| {
                    fault(plan, ValueExprFaultCode::UnitMismatch, node_index, cause)
                })?;
        }
        Ok(frame)
    }

    fn value_expr_unit<'a>(
        &'a self,
        plan: &CompiledAmountPlan,
        subject: ValueSubject,
        source: &RuntimeFactSource,
        target: Option<UnitInstanceId>,
        node_index: usize,
    ) -> Result<&'a super::types::RuntimeUnit, ValueExprFault> {
        let identity = match subject {
            ValueSubject::Source => source.actor,
            ValueSubject::Target => target,
            ValueSubject::AggregateMember => {
                return Err(fault(
                    plan,
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    "aggregate-member value requires a resolved member frame",
                ));
            }
        }
        .ok_or_else(|| {
            fault(
                plan,
                ValueExprFaultCode::MissingContext,
                node_index,
                format!("{subject:?} unit identity is not fixed for this evaluation"),
            )
        })?;
        self.units
            .get(&identity)
            .filter(|unit| unit.lifecycle.is_active())
            .ok_or_else(|| {
                fault(
                    plan,
                    ValueExprFaultCode::MissingContext,
                    node_index,
                    format!("{subject:?} unit is missing or inactive at the evaluation moment"),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    mod aggregate_replay;
    mod amount_replay;
    mod completion;
    mod legacy_amount;
    mod relic_amount;

    use std::collections::BTreeSet;

    use uuid::Uuid;

    use super::*;
    use crate::authoring::control::{self, CancellationToken, ExecutionLimits, ExecutionPhase};
    use crate::game::resources::item_slot::{EquipmentSlotAddress, SlotLayoutKind};
    use crate::game::{
        ability::{
            authoring::{ActionSite, EffectOrigin, EffectSite, EffectSource},
            SkillId,
        },
        battle::{
            scenario::{
                BattleScenario, ScenarioGroupId, ScenarioSpawnGroup, ScenarioUnitRef,
                ScenarioUnitSpawn,
            },
            types::{
                BattleEquipmentBinding, BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass,
                UnitCombatProfile,
            },
        },
        data::{
            equipment_data::{EquipmentMetadata, EquipmentType},
            skill_data::SkillDatabase,
            GameDataBuilder, ValidatedGameData,
        },
        enums::{RiskLevel, Side, Tier},
        growth::GrowthStack,
        resource::{
            ResourceCatalog, ResourceDefinition, ResourceId, ResourceLifetime, ResourceMutation,
            ResourceOperation, ResourceOwner, ResourceOwnerKind, ResourceOwnerSelector,
            RunResourceState,
        },
        resources::Position,
        stats::UnitStats,
        value_expr::{
            compile, AmountExpr, AmountProvenance, DeliveryCapabilities, ExprLimits, FiniteSource,
            Grouping, HitPointField, Reduction, ResolvedOwnerRelation, ResolvedTargetPredicate,
            ResultField, StatSelector, ValueExpr,
        },
    };

    fn source(effect_id: &str) -> EffectSource {
        EffectSource {
            source_file: "fixture/wp02_runtime_frame.ron".to_string(),
            site: EffectSite {
                action: ActionSite {
                    definition_id: SkillId::new("wp02_runtime_frame"),
                    action_id: "step_0".to_string(),
                },
                effect_id: effect_id.to_string(),
            },
            field_path: format!("steps[0].effects[{effect_id}].op"),
            origin_definition_id: SkillId::new("wp02_runtime_frame"),
            origin: EffectOrigin::Original,
        }
    }

    fn plan(moment: EvalMoment, effect_id: &str, root: ValueExpr) -> Arc<CompiledAmountPlan> {
        Arc::new(
            compile(
                &AmountExpr::authored(moment, root),
                &source(effect_id),
                DeliveryCapabilities::all(),
                &ExprLimits::default(),
            )
            .unwrap(),
        )
    }

    fn draft(owned: u128, base: u128, hp: u32, attack: u32) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(hp, hp, attack, 0, 100);
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned),
            source: BattleUnitSource::TestFixture {
                base_uuid: Uuid::from_u128(base),
                profile,
            },
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: None,
            equipped_items: Vec::new(),
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn employee_draft(
        owned: u128,
        hp: u32,
        attack: u32,
        bindings: Vec<BattleEquipmentBinding>,
    ) -> BattleUnitDraft {
        let mut profile = UnitCombatProfile::employee_default();
        profile.weapon_profile = Some(Default::default());
        profile.stats = UnitStats::with_values(hp, hp, attack, 0, 100);
        BattleUnitDraft {
            owned_uuid: Uuid::from_u128(owned),
            source: BattleUnitSource::Employee(profile),
            threat_class: BattleUnitThreatClass::Normal,
            level: Tier::I,
            stat_scale: Default::default(),
            growth_stacks: GrowthStack::new(),
            equipment_layout: Some(SlotLayoutKind::ByType),
            equipped_items: bindings,
            equipped_item_enhancements: Vec::new(),
            employee_start_condition: None,
        }
    }

    fn equipment_metadata(
        id: &str,
        base_uuid: Uuid,
        equipment_type: EquipmentType,
    ) -> EquipmentMetadata {
        EquipmentMetadata {
            id: id.to_string(),
            uuid: base_uuid,
            name: id.to_string(),
            equipment_type,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "fixture".to_string(),
            triggered_effects: Default::default(),
            ability_activations: Vec::new(),
            weapon_profile: (equipment_type == EquipmentType::Weapon).then(Default::default),
            event_check_modifiers: Vec::new(),
            event_choice_keys: Vec::new(),
        }
    }

    struct OwnerFixture {
        core: BattleCore,
        employee_a: UnitInstanceId,
        employee_b: UnitInstanceId,
        summon: UnitInstanceId,
        enemy: UnitInstanceId,
        weapon_x: Uuid,
        armor_y: Uuid,
        weapon_z: Uuid,
    }

    fn owner_fixture(skills: Option<SkillDatabase>) -> OwnerFixture {
        owner_fixture_with_unrelated(skills, 0)
    }

    fn owner_fixture_with_unrelated(
        skills: Option<SkillDatabase>,
        unrelated: usize,
    ) -> OwnerFixture {
        let weapon_base = Uuid::from_u128(0x5101);
        let armor_base = Uuid::from_u128(0x5102);
        let weapon_x = Uuid::from_u128(0x5201);
        let armor_y = Uuid::from_u128(0x5202);
        let weapon_z = Uuid::from_u128(0x5203);
        let mut builder = GameDataBuilder::empty().with_equipment(vec![
            equipment_metadata("owner_weapon", weapon_base, EquipmentType::Weapon),
            equipment_metadata("owner_armor", armor_base, EquipmentType::Armor),
        ]);
        if let Some(skills) = skills {
            builder = builder.with_skills(skills);
        }
        let database = builder.build();
        let mut scenario = BattleScenario::empty((128, 128));
        let player_group = ScenarioGroupId::new("owner_players");
        scenario.groups.push(ScenarioSpawnGroup {
            id: player_group.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee_a"),
                    side: Side::Player,
                    draft: employee_draft(
                        0x5001,
                        100,
                        3,
                        vec![
                            BattleEquipmentBinding {
                                base_uuid: weapon_base,
                                persistent_copy_uuid: Some(weapon_x),
                                slot: Some(EquipmentSlotAddress::Weapon),
                            },
                            BattleEquipmentBinding {
                                base_uuid: armor_base,
                                persistent_copy_uuid: Some(armor_y),
                                slot: Some(EquipmentSlotAddress::Armor),
                            },
                        ],
                    ),
                    position: Position::new(1, 1),
                    instance_salt: 0,
                },
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee_b"),
                    side: Side::Player,
                    draft: employee_draft(
                        0x5002,
                        100,
                        5,
                        vec![BattleEquipmentBinding {
                            base_uuid: weapon_base,
                            persistent_copy_uuid: Some(weapon_z),
                            slot: Some(EquipmentSlotAddress::Weapon),
                        }],
                    ),
                    position: Position::new(2, 1),
                    instance_salt: 1,
                },
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("summon"),
                    side: Side::Player,
                    draft: draft(0x5003, 0x5004, 100, 11),
                    position: Position::new(3, 1),
                    instance_salt: 2,
                },
            ],
        });
        for index in 0..unrelated {
            scenario.groups[0].spawns.push(ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("owner_unrelated_{index}")),
                side: Side::Player,
                draft: employee_draft(
                    0x100000 + index as u128,
                    100,
                    3,
                    vec![BattleEquipmentBinding {
                        base_uuid: weapon_base,
                        persistent_copy_uuid: Some(Uuid::from_u128(0x200000 + index as u128)),
                        slot: Some(EquipmentSlotAddress::Weapon),
                    }],
                ),
                position: Position::new((index % 100) as i32, (10 + index / 100) as i32),
                instance_salt: u32::try_from(index + 10).unwrap(),
            });
        }
        let enemy_group = ScenarioGroupId::new("owner_enemies");
        scenario.groups.push(ScenarioSpawnGroup {
            id: enemy_group.clone(),
            side: Side::Opponent,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new("enemy"),
                side: Side::Opponent,
                draft: draft(0x5005, 0x5006, 100, 7),
                position: Position::new(4, 1),
                instance_salt: 0,
            }],
        });
        let mut core = BattleCore::new_from_scenario(
            scenario,
            ValidatedGameData::from(Arc::new(database)),
            19,
        );
        let players = core.spawn_scenario_group(&player_group).unwrap();
        let enemies = core.spawn_scenario_group(&enemy_group).unwrap();
        OwnerFixture {
            core,
            employee_a: players[0],
            employee_b: players[1],
            summon: players[2],
            enemy: enemies[0],
            weapon_x,
            armor_y,
            weapon_z,
        }
    }

    #[test]
    fn battle_spawn_rejects_invalid_slot_and_actual_copy_projections_before_insertion() {
        let weapon_base = Uuid::from_u128(0x5301);
        let armor_base = Uuid::from_u128(0x5302);
        let database = Arc::new(
            GameDataBuilder::empty()
                .with_equipment(vec![
                    equipment_metadata("projection_weapon", weapon_base, EquipmentType::Weapon),
                    equipment_metadata("projection_armor", armor_base, EquipmentType::Armor),
                ])
                .build(),
        );
        let weapon = |copy, slot| BattleEquipmentBinding {
            base_uuid: weapon_base,
            persistent_copy_uuid: copy,
            slot,
        };
        let armor = |copy, slot| BattleEquipmentBinding {
            base_uuid: armor_base,
            persistent_copy_uuid: copy,
            slot,
        };
        let duplicate_copy = Uuid::from_u128(0x5401);
        let mut missing_layout = employee_draft(
            0x5501,
            100,
            3,
            vec![weapon(
                Some(Uuid::from_u128(0x5402)),
                Some(EquipmentSlotAddress::Weapon),
            )],
        );
        missing_layout.equipment_layout = None;
        let cases = vec![
            missing_layout,
            employee_draft(
                0x5502,
                100,
                3,
                vec![weapon(Some(Uuid::from_u128(0x5403)), None)],
            ),
            employee_draft(
                0x5503,
                100,
                3,
                vec![
                    weapon(
                        Some(Uuid::from_u128(0x5404)),
                        Some(EquipmentSlotAddress::Weapon),
                    ),
                    armor(
                        Some(Uuid::from_u128(0x5405)),
                        Some(EquipmentSlotAddress::Weapon),
                    ),
                ],
            ),
            employee_draft(
                0x5504,
                100,
                3,
                vec![
                    weapon(Some(duplicate_copy), Some(EquipmentSlotAddress::Weapon)),
                    armor(Some(duplicate_copy), Some(EquipmentSlotAddress::Armor)),
                ],
            ),
            employee_draft(
                0x5505,
                100,
                3,
                vec![weapon(
                    Some(Uuid::nil()),
                    Some(EquipmentSlotAddress::Weapon),
                )],
            ),
        ];

        for (index, draft) in cases.into_iter().enumerate() {
            let group_id = ScenarioGroupId::new(format!("invalid_projection_{index}"));
            let mut scenario = BattleScenario::empty((3, 3));
            scenario.groups.push(ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns: vec![ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee"),
                    side: Side::Player,
                    draft,
                    position: Position::new(1, 1),
                    instance_salt: 0,
                }],
            });
            let mut core = BattleCore::new_from_scenario(
                scenario,
                ValidatedGameData::from(Arc::clone(&database)),
                23,
            );
            assert!(matches!(
                core.spawn_scenario_group(&group_id),
                Err(GameError::InvalidStaticData(_))
            ));
            assert!(core.units.is_empty());
            assert!(core.items.is_empty());
        }
    }

    #[test]
    fn equipment_projection_keeps_source_order_item_ids_for_by_type_and_any3() {
        let weapon_base = Uuid::from_u128(0x5601);
        let armor_base = Uuid::from_u128(0x5602);
        let database = Arc::new(
            GameDataBuilder::empty()
                .with_equipment(vec![
                    equipment_metadata("id_weapon", weapon_base, EquipmentType::Weapon),
                    equipment_metadata("id_armor", armor_base, EquipmentType::Armor),
                ])
                .build(),
        );
        let spawn = |draft: BattleUnitDraft, case: &str| {
            let group_id = ScenarioGroupId::new(case);
            let mut scenario = BattleScenario::empty((4, 4));
            scenario.groups.push(ScenarioSpawnGroup {
                id: group_id.clone(),
                side: Side::Player,
                required_for_victory: false,
                enemy_movement_plan: None,
                spawns: vec![ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("employee"),
                    side: Side::Player,
                    draft,
                    position: Position::new(1, 1),
                    instance_salt: 7,
                }],
            });
            let mut core = BattleCore::new_from_scenario(
                scenario,
                ValidatedGameData::from(Arc::clone(&database)),
                31,
            );
            let unit = core.spawn_scenario_group(&group_id).unwrap()[0];
            (core, unit)
        };

        let armor_copy = Uuid::from_u128(0x5701);
        let mut armor_draft = draft(0x5801, 0x5800, 100, 3);
        armor_draft.equipment_layout = Some(SlotLayoutKind::ByType);
        armor_draft.equipped_items = vec![BattleEquipmentBinding {
            base_uuid: armor_base,
            persistent_copy_uuid: Some(armor_copy),
            slot: Some(EquipmentSlotAddress::Armor),
        }];
        let (by_type, unit) = spawn(armor_draft, "by_type_source_order");
        let armor_entry = by_type.units[&unit].equipment_slots.as_ref().unwrap().slots[1].unwrap();
        assert_eq!(
            armor_entry.runtime_item_instance_id,
            BattleCore::make_item_instance_id(armor_base, Side::Player, unit, 0)
        );
        assert_eq!(
            by_type.items[&armor_entry.runtime_item_instance_id].persistent_copy_uuid,
            Some(armor_copy)
        );

        let first_copy = Uuid::from_u128(0x5702);
        let second_copy = Uuid::from_u128(0x5703);
        let mut any3_draft = draft(0x5802, 0x5803, 100, 3);
        any3_draft.equipped_items = vec![
            BattleEquipmentBinding {
                base_uuid: weapon_base,
                persistent_copy_uuid: Some(first_copy),
                slot: Some(EquipmentSlotAddress::Any3(1)),
            },
            BattleEquipmentBinding {
                base_uuid: weapon_base,
                persistent_copy_uuid: Some(second_copy),
                slot: Some(EquipmentSlotAddress::Any3(0)),
            },
        ];
        any3_draft.equipment_layout = Some(SlotLayoutKind::Any3);
        let (any3, unit) = spawn(any3_draft, "any3_source_order");
        let slots = any3.units[&unit].equipment_slots.as_ref().unwrap();
        let slot_zero = slots.slots[0].unwrap();
        let slot_one = slots.slots[1].unwrap();
        assert_eq!(slot_zero.binding.persistent_copy_uuid, Some(second_copy));
        assert_eq!(slot_one.binding.persistent_copy_uuid, Some(first_copy));
        assert_eq!(
            slot_one.runtime_item_instance_id,
            BattleCore::make_item_instance_id(weapon_base, Side::Player, unit, 0)
        );
        assert_eq!(
            slot_zero.runtime_item_instance_id,
            BattleCore::make_item_instance_id(weapon_base, Side::Player, unit, 1)
        );
        let provider_ids = any3
            .runtime_fact_view()
            .providers()
            .map(|(provider, _)| {
                (
                    provider.persistent_copy_uuid.unwrap(),
                    provider.runtime_item_instance_id,
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(provider_ids[&first_copy], slot_one.runtime_item_instance_id);
        assert_eq!(
            provider_ids[&second_copy],
            slot_zero.runtime_item_instance_id
        );
    }

    #[test]
    fn equipment_projection_retained_extent_and_arc_reuse_include_runtime_ids() {
        use crate::game::battle::runtime_control as native;

        let fixture = owner_fixture(None);
        let source = fixture.core.units[&fixture.employee_a].clone();
        let draft = fixture.core.scenario.groups[0].spawns[0].draft.clone();
        let basis = fixture.core.retained_runtime_fact_extent().unwrap();
        let projection_extent = super::super::types::BattleEquipmentSlots::retained_extent();
        assert!(
            std::mem::size_of::<super::super::types::RuntimeEquipmentSlot>()
                >= std::mem::size_of::<BattleEquipmentBinding>() + std::mem::size_of::<Uuid>()
        );

        let rejected = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: basis + projection_extent - 1,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(basis)?;
                fixture
                    .core
                    .prepare_battle_equipment_slots(&draft, Side::Player, fixture.employee_a)
                    .map(|_| ())
            },
        );
        assert!(matches!(
            rejected,
            Err(GameError::BattleRuntimeFault(
                crate::game::battle::runtime_control::BattleRuntimeFault {
                    code: crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,
                    ..
                }
            ))
        ));
        let projected = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: basis + projection_extent,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(basis)?;
                fixture.core.prepare_battle_equipment_slots(
                    &draft,
                    Side::Player,
                    fixture.employee_a,
                )
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            projected.slots[0].unwrap().runtime_item_instance_id,
            source.equipment_slots.as_ref().unwrap().slots[0]
                .unwrap()
                .runtime_item_instance_id
        );

        let cloned = source.clone();
        assert!(Arc::ptr_eq(
            source.equipment_slots.as_ref().unwrap(),
            cloned.equipment_slots.as_ref().unwrap()
        ));
        let mut scratch = fixture.core.units[&fixture.employee_b].clone();
        scratch.equipment_slots = None;
        scratch.reuse_timestamp_value(&source);
        assert!(Arc::ptr_eq(
            source.equipment_slots.as_ref().unwrap(),
            scratch.equipment_slots.as_ref().unwrap()
        ));
    }

    fn core_with_units(
        resources: Vec<ResourceDefinition>,
    ) -> (BattleCore, UnitInstanceId, UnitInstanceId) {
        core_with_units_and_skills(resources, None)
    }

    fn core_with_units_and_skills(
        resources: Vec<ResourceDefinition>,
        skills: Option<SkillDatabase>,
    ) -> (BattleCore, UnitInstanceId, UnitInstanceId) {
        core_with_units_and_equipment(resources, skills, None)
    }

    fn core_with_units_and_equipment(
        resources: Vec<ResourceDefinition>,
        skills: Option<SkillDatabase>,
        source_equipment: Option<BattleEquipmentBinding>,
    ) -> (BattleCore, UnitInstanceId, UnitInstanceId) {
        let mut builder = GameDataBuilder::empty();
        if let Some(skills) = skills {
            builder = builder.with_skills(skills);
        }
        if let Some(binding) = source_equipment {
            builder = builder.with_equipment(vec![EquipmentMetadata {
                id: "frame_fixture_item".to_string(),
                uuid: binding.base_uuid,
                name: "Frame fixture item".to_string(),
                equipment_type: EquipmentType::Weapon,
                rarity: RiskLevel::ZAYIN,
                price: 0,
                allow_duplicate_equip: true,
                bound: false,
                cannot_unequip_reason: "fixture".to_string(),
                triggered_effects: Default::default(),
                ability_activations: Vec::new(),
                weapon_profile: Some(Default::default()),
                event_check_modifiers: Vec::new(),
                event_choice_keys: Vec::new(),
            }]);
        }
        let database = builder
            .with_resources(ResourceCatalog { resources })
            .build();
        let database = Arc::new(database);
        let group_id = ScenarioGroupId::new("initial");
        let mut scenario = BattleScenario::empty((4, 4));
        let mut source_draft = draft(0x2201, 0x2202, 100, 10);
        source_draft.equipped_items.extend(source_equipment);
        scenario.groups.push(ScenarioSpawnGroup {
            id: group_id.clone(),
            side: Side::Player,
            required_for_victory: false,
            enemy_movement_plan: None,
            spawns: vec![
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("source"),
                    side: Side::Player,
                    draft: source_draft,
                    position: Position::new(1, 1),
                    instance_salt: 0,
                },
                ScenarioUnitSpawn {
                    unit_ref: ScenarioUnitRef::new("target"),
                    side: Side::Player,
                    draft: draft(0x2203, 0x2204, 80, 3),
                    position: Position::new(2, 1),
                    instance_salt: 1,
                },
            ],
        });
        let validated = ValidatedGameData::from(database);
        let mut core = BattleCore::new_from_scenario(scenario, validated, 17);
        let units = core.spawn_scenario_group(&group_id).unwrap();
        assert_eq!(units.len(), 2);
        if let Some(runtime) = core.runtime_facts.resources.as_mut() {
            let mut owners = BTreeSet::from([
                ResourceOwner::Employee(Uuid::from_u128(0x2201)),
                ResourceOwner::Employee(Uuid::from_u128(0x2203)),
            ]);
            if let Some(copy) =
                source_equipment.and_then(|equipment| equipment.persistent_copy_uuid)
            {
                owners.insert(ResourceOwner::EquipmentCopy(copy));
            }
            runtime.store.reconcile_owners(&owners).unwrap();
        }
        (core, units[0], units[1])
    }

    fn occurrence(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        plan: &CompiledAmountPlan,
    ) -> RuntimeFactSource {
        let authored = Arc::new(plan.source().clone());
        let basis = RuntimeFactSource {
            actor: Some(actor),
            authored: Some(authored.clone()),
            ..Default::default()
        };
        let action = core
            .runtime_facts
            .accept(actor, None, basis, 1, false)
            .unwrap();
        RuntimeFactSource {
            actor: Some(actor),
            authored: Some(authored),
            action: Some(action),
            ..Default::default()
        }
    }

    fn raw(plan: &CompiledAmountPlan, frame: &EvalFrame) -> TypedInteger {
        let mut meter = ExprMeter::new(plan.evaluation_limits()).unwrap();
        evaluate(plan, frame, &mut meter).unwrap().value
    }

    #[test]
    fn evaluation_respects_remaining_native_work_and_recovers_next_operation() {
        use crate::game::battle::runtime_control as native;

        let expression = plan(
            EvalMoment::Live,
            "host_work",
            ValueExpr::Add(
                (0..128)
                    .map(|_| {
                        ValueExpr::Read(ValueRef::Stat {
                            subject: ValueSubject::Source,
                            stat: StatSelector::Attack,
                        })
                    })
                    .collect(),
            ),
        );
        let (mut core, actor, _) = core_with_units(Vec::new());
        let source = occurrence(&mut core, actor, &expression);
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &source, None, 10, None)
            .unwrap();
        let tight = native::run(
            native::RuntimeFactLimits {
                max_work: 1,
                ..Default::default()
            },
            || {
                let evaluated = core.evaluate_value_expr_capture(&capture, None, 11);
                assert!(evaluated.is_err());
                Ok(())
            },
        );
        assert!(matches!(tight, Err(GameError::BattleRuntimeFault(_))));
        let result = native::run(native::RuntimeFactLimits::default(), || {
            Ok(core.evaluate_value_expr_capture(&capture, None, 11))
        })
        .unwrap()
        .unwrap();
        assert_eq!(result.amount, RuntimeAmount::Damage(1280));
    }

    #[test]
    fn evaluation_retained_allocation_is_admitted_before_use() {
        use crate::game::battle::runtime_control as native;

        let expression = plan(
            EvalMoment::Live,
            "host_retained",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, actor, _) = core_with_units(Vec::new());
        let source = occurrence(&mut core, actor, &expression);
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &source, None, 10, None)
            .unwrap();
        let retained = core.retained_runtime_fact_extent().unwrap() + capture.retained_extent();
        let tight = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: retained + 1,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(retained)?;
                let evaluated = core.evaluate_value_expr_capture(&capture, None, 11);
                assert!(evaluated.is_err());
                Ok(())
            },
        );
        assert!(matches!(tight, Err(GameError::BattleRuntimeFault(_))));
    }

    #[test]
    fn evaluation_charges_authoring_host_and_observes_cancellation() {
        let expression = plan(
            EvalMoment::Live,
            "authoring_work",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, actor, _) = core_with_units(Vec::new());
        let source = occurrence(&mut core, actor, &expression);
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &source, None, 10, None)
            .unwrap();
        let limits = ExecutionLimits {
            max_work: 1,
            ..Default::default()
        };
        let token = CancellationToken::default();
        let tight = control::run(&limits, &token, ExecutionPhase::TrialExecution, || {
            core.evaluate_value_expr_capture(&capture, None, 11)
        });
        assert!(matches!(tight, Err(error) if error.code == "authoring_work_limit"));
        token.cancel();
        let cancelled = control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || core.evaluate_value_expr_capture(&capture, None, 11),
        );
        assert!(matches!(cancelled, Err(error) if error.code == "authoring_cancelled"));
    }

    #[test]
    fn generation_catalog_entry_drives_capture_and_supported_publication() {
        let skills = SkillDatabase::from_ron(
            "fixture/wp02_catalog.ron",
            r#"SkillDatabase(
                skills: [SkillDef(
                    id: "wp02_runtime_frame", name: "WP02", kind: Targeted,
                    focus_time_ms: 0,
                    use_lifetime: Action,
                    cast_targeting: Explicit(
                        selection: automatic_unit,
                        target: EnemySingle(rule: Nearest),
                        reach: global,
                        air_capable: false,
                    ),
                    steps: [SkillStepDef(
                        id: "step_0", delay_ms: 0,
                        target: EnemySingle(rule: Nearest), delivery: Instant,
                        effects: [(id: "catalog", op: DamageExpr(
                            amount: (moment: Snapshot, root: Add([
                                Const((value: 2, unit: Magnitude)),
                                Const((value: 3, unit: Magnitude)),
                            ])),
                            damage_type: True,
                        ))],
                    )],
                )],
            )"#,
        )
        .unwrap();
        let (mut core, source_id, _) = core_with_units_and_skills(Vec::new(), Some(skills.clone()));
        let (site, amount_plan) = {
            let (site, entry) = core
                .game_data
                .value_expr_catalog()
                .entries()
                .next()
                .unwrap();
            (site.clone(), Arc::clone(entry.plan()))
        };
        let source = occurrence(&mut core, source_id, &amount_plan);
        let entry = core.game_data.value_expr_catalog().get(&site).unwrap();

        let (alien_core, _, _) = core_with_units_and_skills(Vec::new(), Some(skills));
        let alien_entry = alien_core
            .game_data
            .value_expr_catalog()
            .get(&site)
            .unwrap();
        let error = core
            .capture_value_expr_entry(alien_entry, &source, None, 10, None)
            .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::Unavailable);
        assert!(error.cause.contains("exact plan"));

        let capture = core
            .capture_value_expr_entry(entry, &source, None, 10, None)
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, None, 11)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(5)
        );
        core.game_data
            .value_expr_catalog()
            .ensure_runtime_publishable()
            .unwrap();
    }

    #[test]
    fn snapshot_preserves_release_values_while_live_reads_effect_start_state() {
        let root = ValueExpr::Add(vec![
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
            ValueExpr::Read(ValueRef::HitPoints {
                subject: ValueSubject::Target,
                field: HitPointField::Current,
            }),
        ]);
        let snapshot_plan = plan(EvalMoment::Snapshot, "snapshot", root.clone());
        let live_plan = plan(EvalMoment::Live, "live", root);
        let (mut core, source_id, target_id) = core_with_units(Vec::new());
        let snapshot_source = occurrence(&mut core, source_id, &snapshot_plan);
        let live_source = RuntimeFactSource {
            authored: Some(Arc::new(live_plan.source().clone())),
            ..snapshot_source.clone()
        };

        let snapshot = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                snapshot_plan,
                &snapshot_source,
                Some(target_id),
                10,
                None,
            )
            .unwrap();
        let live = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                live_plan,
                &live_source,
                Some(target_id),
                10,
                None,
            )
            .unwrap();

        core.units.get_mut(&source_id).unwrap().stats.attack = 20;
        core.units.get_mut(&target_id).unwrap().stats.current_health = 30;

        let snapshot_value = core
            .evaluate_value_expr_capture(&snapshot, Some(target_id), 20)
            .unwrap();
        let live_value = core
            .evaluate_value_expr_capture(&live, Some(target_id), 20)
            .unwrap();
        assert_eq!(snapshot_value.amount, RuntimeAmount::Damage(90));
        assert_eq!(snapshot_value.evaluated.value.value, 90);
        assert!(snapshot_value.usage.work > 0);
        assert_eq!(live_value.amount, RuntimeAmount::Damage(50));
        assert!(snapshot.retained_extent() >= size_of::<EvalCapture>());

        core.units.remove(&source_id);
        core.units.remove(&target_id);
        assert_eq!(
            core.evaluate_value_expr_capture(&snapshot, Some(target_id), 21)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(90)
        );
        let error = core
            .evaluate_value_expr_capture(&live, Some(target_id), 21)
            .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::MissingContext);
        assert_eq!(error.site, live.plan.source().site);
    }

    #[test]
    fn target_and_time_context_never_fall_back_to_zero_or_release_time() {
        let target_plan = plan(
            EvalMoment::Snapshot,
            "target",
            ValueExpr::Read(ValueRef::HitPoints {
                subject: ValueSubject::Target,
                field: HitPointField::Current,
            }),
        );
        let live_target_plan = plan(
            EvalMoment::Live,
            "live_target",
            ValueExpr::Read(ValueRef::HitPoints {
                subject: ValueSubject::Target,
                field: HitPointField::Current,
            }),
        );
        let (mut core, source_id, target_id) = core_with_units(Vec::new());
        let target_source = occurrence(&mut core, source_id, &target_plan);
        let error = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                target_plan,
                &target_source,
                None,
                10,
                None,
            )
            .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::MissingContext);

        let live_source = RuntimeFactSource {
            authored: Some(Arc::new(live_target_plan.source().clone())),
            ..target_source.clone()
        };
        let live = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                live_target_plan,
                &live_source,
                None,
                10,
                None,
            )
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&live, None, 20)
                .unwrap_err()
                .code,
            ValueExprFaultCode::MissingContext
        );
        assert_eq!(
            core.evaluate_value_expr_capture(&live, Some(target_id), 9)
                .unwrap_err()
                .code,
            ValueExprFaultCode::Unavailable
        );

        let elapsed_snapshot = plan(
            EvalMoment::Snapshot,
            "elapsed_snapshot",
            ValueExpr::Read(ValueRef::Time(TimeValue::BattleElapsed)),
        );
        let elapsed_live = plan(
            EvalMoment::Live,
            "elapsed_live",
            ValueExpr::Read(ValueRef::Time(TimeValue::BattleElapsed)),
        );
        let snapshot_source = RuntimeFactSource {
            authored: Some(Arc::new(elapsed_snapshot.source().clone())),
            ..target_source.clone()
        };
        let live_source = RuntimeFactSource {
            authored: Some(Arc::new(elapsed_live.source().clone())),
            ..target_source
        };
        let snapshot = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                elapsed_snapshot,
                &snapshot_source,
                None,
                10,
                None,
            )
            .unwrap();
        assert_eq!(
            raw(&snapshot.plan, snapshot.snapshot_frame.as_deref().unwrap()).value,
            10
        );
        let live_frame = core
            .build_value_expr_frame(&elapsed_live, &live_source, None, 99, None)
            .unwrap();
        assert_eq!(raw(&elapsed_live, &live_frame).value, 99);

        let remaining = plan(
            EvalMoment::Live,
            "remaining",
            ValueExpr::Read(ValueRef::Time(TimeValue::RemainingTime)),
        );
        let remaining_source = RuntimeFactSource {
            authored: Some(Arc::new(remaining.source().clone())),
            ..live_source
        };
        assert_eq!(
            core.build_value_expr_frame(&remaining, &remaining_source, None, 99, None)
                .unwrap_err()
                .code,
            ValueExprFaultCode::MissingContext
        );
        let expired = core
            .build_value_expr_frame(&remaining, &remaining_source, None, 99, Some(80))
            .unwrap();
        assert_eq!(raw(&remaining, &expired).value, 0);
    }

    #[test]
    fn exact_generation_action_and_delivery_identities_are_required() {
        let amount_plan = plan(
            EvalMoment::Snapshot,
            "identity",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, source_id, _) = core_with_units(Vec::new());
        let mut source = occurrence(&mut core, source_id, &amount_plan);
        let action = source.action.clone().unwrap();
        let delivery = core.runtime_facts.delivery(&action).unwrap().unwrap();
        source.delivery = Some(delivery.clone());
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, amount_plan, &source, None, 10, None)
            .unwrap();

        let (other, _, _) = core_with_units(Vec::new());
        let error = other
            .evaluate_value_expr_capture(&capture, None, 11)
            .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::DifferentAttempt);
        assert_eq!(error.site.effect_id, "identity");

        core.runtime_facts.close_delivery(&delivery, 11).unwrap();
        let error = core
            .evaluate_value_expr_capture(&capture, None, 12)
            .unwrap_err();
        assert_eq!(error.code, ValueExprFaultCode::Unavailable);
        assert!(error.cause.contains("delivery occurrence"));

        let (mut generation_mismatch, _, _) = core_with_units(Vec::new());
        let current = generation_mismatch.content_identity.as_ref().unwrap();
        generation_mismatch.content_identity = Some(Arc::new(ContentIdentity {
            contract: current.contract.clone(),
            semantic_sha256: "different-generation".to_string(),
        }));
        assert_eq!(
            generation_mismatch
                .evaluate_value_expr_capture(&capture, None, 12)
                .unwrap_err()
                .code,
            ValueExprFaultCode::Unavailable
        );
    }

    #[test]
    fn action_source_actor_and_exact_provider_copy_are_bound_at_capture() {
        use crate::game::battle::runtime_facts::EquipmentSourceSnapshot;

        let expression = plan(
            EvalMoment::Snapshot,
            "bound_source",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, actor, other) = core_with_units(Vec::new());
        let source = occurrence(&mut core, actor, &expression);
        let before = core.transaction_debug_snapshot();
        let mut substituted = source.clone();
        substituted.actor = Some(other);
        assert_eq!(
            core.capture_value_expr_plan(
                AmountPlanKind::Damage,
                expression.clone(),
                &substituted,
                None,
                10,
                None,
            )
            .unwrap_err()
            .code,
            ValueExprFaultCode::Unavailable
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        let capture = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                expression.clone(),
                &source,
                None,
                10,
                None,
            )
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, None, 11)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(10)
        );

        let provider = EquipmentSourceSnapshot {
            definition_uuid: Uuid::from_u128(91),
            runtime_item_instance_id: Uuid::from_u128(92),
            persistent_copy_uuid: Some(Uuid::from_u128(93)),
        };
        let mut bound = RuntimeFactSource {
            actor: Some(actor),
            equipment: Some(provider),
            authored: Some(Arc::new(expression.source().clone())),
            ..Default::default()
        };
        bound.action = Some(
            core.runtime_facts
                .accept(actor, None, bound.clone(), 2, false)
                .unwrap(),
        );
        for changed in [
            EquipmentSourceSnapshot {
                persistent_copy_uuid: Some(Uuid::from_u128(94)),
                ..provider
            },
            EquipmentSourceSnapshot {
                runtime_item_instance_id: Uuid::from_u128(95),
                ..provider
            },
            EquipmentSourceSnapshot {
                definition_uuid: Uuid::from_u128(96),
                ..provider
            },
        ] {
            let mut substituted = bound.clone();
            substituted.equipment = Some(changed);
            assert_eq!(
                core.capture_value_expr_plan(
                    AmountPlanKind::Damage,
                    expression.clone(),
                    &substituted,
                    None,
                    10,
                    None,
                )
                .unwrap_err()
                .code,
                ValueExprFaultCode::Unavailable
            );
        }
        assert!(core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &bound, None, 10, None)
            .is_ok());
    }

    #[test]
    fn scenario_equipment_without_copy_and_valid_child_delivery_keep_exact_action_binding() {
        let expression = plan(
            EvalMoment::Snapshot,
            "scenario_provider",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, actor, _) = core_with_units_and_equipment(
            Vec::new(),
            None,
            Some(BattleEquipmentBinding {
                base_uuid: Uuid::from_u128(91),
                persistent_copy_uuid: None,
                slot: None,
            }),
        );
        let provider = core
            .items
            .values()
            .find(|item| item.owner_unit_instance == actor)
            .unwrap()
            .source_snapshot();
        assert_eq!(provider.persistent_copy_uuid, None);
        let mut source = occurrence(&mut core, actor, &expression);
        source.equipment = Some(provider);
        let action = source.action.clone().unwrap();
        source.delivery = core.runtime_facts.delivery(&action).unwrap();
        let capture = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                expression.clone(),
                &source,
                None,
                10,
                None,
            )
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, None, 11)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(10)
        );

        let before = core.transaction_debug_snapshot();
        let mut switched = source.clone();
        switched
            .equipment
            .as_mut()
            .unwrap()
            .runtime_item_instance_id = Uuid::from_u128(999);
        assert_eq!(
            core.capture_value_expr_plan(
                AmountPlanKind::Damage,
                expression.clone(),
                &switched,
                None,
                10,
                None,
            )
            .unwrap_err()
            .code,
            ValueExprFaultCode::Unavailable
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        let other_action = core
            .runtime_facts
            .accept(actor, None, source.clone(), 2, false)
            .unwrap();
        switched = source.clone();
        switched.delivery = core.runtime_facts.delivery(&other_action).unwrap();
        let before_delivery_rejection = core.transaction_debug_snapshot();
        assert_eq!(
            core.capture_value_expr_plan(
                AmountPlanKind::Damage,
                expression.clone(),
                &switched,
                None,
                10,
                None,
            )
            .unwrap_err()
            .code,
            ValueExprFaultCode::Unavailable
        );
        assert_eq!(core.transaction_debug_snapshot(), before_delivery_rejection);
        // The rejection did not consume the original action or delivery.
        assert!(core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &source, None, 10, None)
            .is_ok());
    }

    #[test]
    fn live_resource_rejects_dead_target_and_keeps_snapshot_value_and_store_unchanged() {
        let resource = ResourceId::new("meter".to_string()).unwrap();
        let (mut core, actor, target) = core_with_units(vec![ResourceDefinition {
            id: resource.clone(),
            owner: ResourceOwnerKind::Employee,
            lifetime: ResourceLifetime::Attempt,
            initial: 7,
            cap: 30,
        }]);
        let root = || ValueExpr::Mul {
            value: Box::new(ValueExpr::Const(TypedInteger::new(1, ValueUnit::Magnitude))),
            scalar: Box::new(ValueExpr::Ratio {
                numerator: Box::new(ValueExpr::Read(ValueRef::Resource {
                    resource: resource.clone(),
                    owner: ResourceOwnerSelector::TargetEmployee,
                    field: ResourceValueField::Balance,
                })),
                denominator: Box::new(ValueExpr::Const(TypedInteger::new(
                    1,
                    ValueUnit::Resource(resource.clone()),
                ))),
                rounding: crate::game::value_expr::Rounding::TowardZero,
            }),
        };
        let live = plan(EvalMoment::Live, "dead_target_live", root());
        let snapshot = plan(EvalMoment::Snapshot, "dead_target_snapshot", root());
        let mut source = occurrence(&mut core, actor, &live);
        let live_capture = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                live,
                &source,
                Some(target),
                10,
                None,
            )
            .unwrap();
        source.authored = Some(Arc::new(snapshot.source().clone()));
        let snapshot_capture = core
            .capture_value_expr_plan(
                AmountPlanKind::Damage,
                snapshot,
                &source,
                Some(target),
                10,
                None,
            )
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&live_capture, Some(target), 10)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(7)
        );
        core.finalize_unit_death(Some(actor), target, Side::Player, 11);
        assert!(core.units.get(&target).unwrap().lifecycle.is_dead());
        let revision = core.typed_resources().unwrap().revision();
        let reservations = core.typed_resources().unwrap().reservation_count();
        let before = core.transaction_debug_snapshot();
        assert_eq!(
            core.evaluate_value_expr_capture(&live_capture, Some(target), 11)
                .unwrap_err()
                .code,
            ValueExprFaultCode::MissingContext
        );
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(
            core.evaluate_value_expr_capture(&snapshot_capture, Some(target), 11)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(7)
        );
        assert_eq!(core.typed_resources().unwrap().revision(), revision);
        assert_eq!(
            core.typed_resources().unwrap().reservation_count(),
            reservations
        );
    }

    #[test]
    fn live_resource_unit_availability_is_separate_from_owner_lease_lifetime() {
        let employee = ResourceId::new("employee_meter".to_string()).unwrap();
        let equipment = ResourceId::new("equipment_meter".to_string()).unwrap();
        let copy = Uuid::from_u128(0x4101);
        let make_core = || {
            core_with_units_and_equipment(
                vec![
                    ResourceDefinition {
                        id: employee.clone(),
                        owner: ResourceOwnerKind::Employee,
                        lifetime: ResourceLifetime::Attempt,
                        initial: 7,
                        cap: 30,
                    },
                    ResourceDefinition {
                        id: equipment.clone(),
                        owner: ResourceOwnerKind::EquipmentCopy,
                        lifetime: ResourceLifetime::Attempt,
                        initial: 9,
                        cap: 30,
                    },
                ],
                None,
                Some(BattleEquipmentBinding {
                    base_uuid: Uuid::from_u128(0x4001),
                    persistent_copy_uuid: Some(copy),
                    slot: None,
                }),
            )
        };
        let plans = || {
            [
                (
                    ResourceOwnerSelector::SourceEmployee,
                    plan(
                        EvalMoment::Live,
                        "source_employee",
                        ValueExpr::Read(ValueRef::Resource {
                            resource: employee.clone(),
                            owner: ResourceOwnerSelector::SourceEmployee,
                            field: ResourceValueField::Balance,
                        }),
                    ),
                    7,
                ),
                (
                    ResourceOwnerSelector::TargetEmployee,
                    plan(
                        EvalMoment::Live,
                        "target_employee",
                        ValueExpr::Read(ValueRef::Resource {
                            resource: employee.clone(),
                            owner: ResourceOwnerSelector::TargetEmployee,
                            field: ResourceValueField::Balance,
                        }),
                    ),
                    7,
                ),
                (
                    ResourceOwnerSelector::SourceEquipment,
                    plan(
                        EvalMoment::Live,
                        "source_equipment",
                        ValueExpr::Read(ValueRef::Resource {
                            resource: equipment.clone(),
                            owner: ResourceOwnerSelector::SourceEquipment,
                            field: ResourceValueField::Balance,
                        }),
                    ),
                    9,
                ),
            ]
        };
        for lifecycle in ["dead", "withdrawn", "missing"] {
            let (mut core, actor, target) = make_core();
            let provider = core
                .items
                .values()
                .find(|item| item.owner_unit_instance == actor)
                .unwrap()
                .source_snapshot();
            let checks = plans();
            let mut sources = checks
                .iter()
                .map(|(_, plan, _)| {
                    let mut source = occurrence(&mut core, actor, plan);
                    source.equipment = Some(provider);
                    source
                })
                .collect::<Vec<_>>();
            let snapshots = [
                (ResourceOwnerSelector::SourceEmployee, employee.clone(), 7),
                (ResourceOwnerSelector::TargetEmployee, employee.clone(), 7),
                (ResourceOwnerSelector::SourceEquipment, equipment.clone(), 9),
            ]
            .map(|(owner, resource, expected)| {
                let snapshot = plan(
                    EvalMoment::Snapshot,
                    &format!("snapshot_{owner:?}"),
                    ValueExpr::Read(ValueRef::Resource {
                        resource,
                        owner,
                        field: ResourceValueField::Balance,
                    }),
                );
                let mut source = occurrence(&mut core, actor, &snapshot);
                source.equipment = Some(provider);
                let frame = core
                    .build_value_expr_frame(&snapshot, &source, Some(target), 10, None)
                    .unwrap();
                (snapshot, frame, expected)
            });
            for ((_, plan, expected), source) in checks.iter().zip(&sources) {
                let frame = core
                    .build_value_expr_frame(plan, source, Some(target), 10, None)
                    .unwrap();
                assert_eq!(raw(plan, &frame).value, *expected);
            }
            let before = core.typed_resources().unwrap().revision();
            match lifecycle {
                "dead" => core.finalize_unit_death(Some(actor), target, Side::Player, 11),
                "withdrawn" => core.withdraw_unit(target, 11).unwrap(),
                "missing" => {
                    core.units.remove(&target);
                }
                _ => unreachable!(),
            }
            let after_lifecycle = core.typed_resources().unwrap().revision();
            let reservations = core.typed_resources().unwrap().reservation_count();
            let before_failure = core.transaction_debug_snapshot();
            assert_eq!(
                core.build_value_expr_frame(&checks[1].1, &sources[1], Some(target), 11, None)
                    .unwrap_err()
                    .code,
                ValueExprFaultCode::MissingContext,
                "target {lifecycle}"
            );
            assert_eq!(core.transaction_debug_snapshot(), before_failure);
            assert_eq!(core.typed_resources().unwrap().revision(), after_lifecycle);
            assert_eq!(
                core.typed_resources().unwrap().reservation_count(),
                reservations
            );
            assert!(after_lifecycle >= before);

            // For a source that has left, both employee and equipment reads
            // fail at the unit boundary before any lease or balance access.
            match lifecycle {
                "dead" => core.finalize_unit_death(None, actor, Side::Player, 12),
                "withdrawn" => core.withdraw_unit(actor, 12).unwrap(),
                "missing" => {
                    core.units.remove(&actor);
                }
                _ => unreachable!(),
            }
            for index in [0, 2] {
                assert_eq!(
                    core.build_value_expr_frame(
                        &checks[index].1,
                        &sources[index],
                        Some(target),
                        12,
                        None
                    )
                    .unwrap_err()
                    .code,
                    ValueExprFaultCode::MissingContext,
                    "source {lifecycle} selector {:?}",
                    checks[index].0
                );
            }
            for (snapshot, frame, expected) in &snapshots {
                assert_eq!(raw(snapshot, frame).value, *expected);
            }
            sources.clear();
        }
    }

    #[test]
    fn live_target_resource_obeys_availability_across_run_attempt_and_deployment() {
        for lifetime in [
            ResourceLifetime::Run,
            ResourceLifetime::Attempt,
            ResourceLifetime::Deployment,
        ] {
            let id = ResourceId::new(format!("target_{lifetime:?}")).unwrap();
            let (mut core, actor, target) = core_with_units(vec![ResourceDefinition {
                id: id.clone(),
                owner: ResourceOwnerKind::Employee,
                lifetime,
                initial: 7,
                cap: 30,
            }]);
            if lifetime == ResourceLifetime::Run {
                core.runtime_facts
                    .resources
                    .as_mut()
                    .unwrap()
                    .mount_run(
                        &RunResourceState::default(),
                        &BTreeSet::from([
                            ResourceOwner::Employee(Uuid::from_u128(0x2201)),
                            ResourceOwner::Employee(Uuid::from_u128(0x2203)),
                        ]),
                    )
                    .unwrap();
            }
            let root = || {
                ValueExpr::Read(ValueRef::Resource {
                    resource: id.clone(),
                    owner: ResourceOwnerSelector::TargetEmployee,
                    field: ResourceValueField::Balance,
                })
            };
            let live = plan(EvalMoment::Live, "lifetime_live", root());
            let snapshot = plan(EvalMoment::Snapshot, "lifetime_snapshot", root());
            let source = occurrence(&mut core, actor, &live);
            let live_frame = core
                .build_value_expr_frame(&live, &source, Some(target), 10, None)
                .unwrap();
            let snapshot_frame = core
                .build_value_expr_frame(&snapshot, &source, Some(target), 10, None)
                .unwrap();
            assert_eq!(raw(&live, &live_frame).value, 7);
            core.finalize_unit_death(Some(actor), target, Side::Player, 11);
            assert!(core.units.get(&target).unwrap().lifecycle.is_dead());
            assert_eq!(raw(&snapshot, &snapshot_frame).value, 7);
            assert_eq!(
                core.build_value_expr_frame(&live, &source, Some(target), 11, None)
                    .unwrap_err()
                    .code,
                ValueExprFaultCode::MissingContext,
                "{lifetime:?}"
            );
            let lease = core.resolve_typed_resource_lease(
                &id,
                ResourceOwnerSelector::TargetEmployee,
                &source,
                Some(target),
            );
            assert_eq!(lease.is_ok(), lifetime != ResourceLifetime::Deployment);
        }
    }

    #[test]
    fn released_snapshot_delivery_survives_source_withdrawal() {
        let expression = plan(
            EvalMoment::Snapshot,
            "departed_source_snapshot",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        let (mut core, actor, _) = core_with_units(Vec::new());
        let mut source = occurrence(&mut core, actor, &expression);
        let action = source.action.clone().unwrap();
        source.delivery = core.runtime_facts.delivery(&action).unwrap();
        core.runtime_facts.release(&action, 0, 9).unwrap();
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, expression, &source, None, 10, None)
            .unwrap();
        core.withdraw_unit(actor, 11).unwrap();
        assert!(!core.units.get(&actor).unwrap().lifecycle.is_active());
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, None, 12)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(10)
        );
    }

    #[test]
    fn runtime_amounts_use_shared_damage_and_heal_narrowing_with_actual_site() {
        let target_hp = ValueExpr::Read(ValueRef::HitPoints {
            subject: ValueSubject::Target,
            field: HitPointField::Current,
        });
        let heal_plan = plan(EvalMoment::Live, "heal", target_hp.clone());
        let damage_underflow = plan(
            EvalMoment::Live,
            "damage_underflow",
            ValueExpr::Add(vec![
                target_hp.clone(),
                ValueExpr::Const(TypedInteger::new(-100, ValueUnit::Magnitude)),
            ]),
        );
        let heal_overflow = plan(
            EvalMoment::Live,
            "heal_overflow",
            ValueExpr::Add(vec![
                target_hp,
                ValueExpr::Const(TypedInteger::new(
                    i128::from(i32::MAX),
                    ValueUnit::Magnitude,
                )),
            ]),
        );
        let (mut core, source_id, target_id) = core_with_units(Vec::new());
        let base = occurrence(&mut core, source_id, &heal_plan);

        let evaluate = |core: &BattleCore,
                        current: Arc<CompiledAmountPlan>,
                        kind: AmountPlanKind|
         -> Result<RuntimeAmountEvaluation, ValueExprFault> {
            let source = RuntimeFactSource {
                authored: Some(Arc::new(current.source().clone())),
                ..base.clone()
            };
            let capture =
                core.capture_value_expr_plan(kind, current, &source, Some(target_id), 10, None)?;
            core.evaluate_value_expr_capture(&capture, Some(target_id), 11)
        };

        assert_eq!(
            evaluate(&core, heal_plan, AmountPlanKind::Heal)
                .unwrap()
                .amount,
            RuntimeAmount::Heal(80)
        );
        for (current, kind, effect_id) in [
            (damage_underflow, AmountPlanKind::Damage, "damage_underflow"),
            (heal_overflow, AmountPlanKind::Heal, "heal_overflow"),
        ] {
            let error = evaluate(&core, current, kind).unwrap_err();
            assert_eq!(error.code, ValueExprFaultCode::Overflow);
            assert_eq!(error.phase, ValueExprPhase::Narrow);
            assert_eq!(error.site.effect_id, effect_id);
        }
    }

    #[test]
    fn resource_frame_reuses_exact_owner_scope_and_read_only_store_values() {
        let resource = ResourceId::new("meter".to_string()).unwrap();
        let definition = ResourceDefinition {
            id: resource.clone(),
            owner: ResourceOwnerKind::Employee,
            lifetime: ResourceLifetime::Attempt,
            initial: 7,
            cap: 30,
        };
        let (mut core, source_id, target_id) = core_with_units(vec![definition]);
        let resource_plan = |effect_id: &str, owner, field| {
            plan(
                EvalMoment::Live,
                effect_id,
                ValueExpr::Read(ValueRef::Resource {
                    resource: resource.clone(),
                    owner,
                    field,
                }),
            )
        };
        let source_balance = resource_plan(
            "source_balance",
            ResourceOwnerSelector::SourceEmployee,
            ResourceValueField::Balance,
        );
        let mut source = occurrence(&mut core, source_id, &source_balance);
        let before_revision = core.typed_resources().unwrap().revision();
        let frame = core
            .build_value_expr_frame(&source_balance, &source, Some(target_id), 10, None)
            .unwrap();
        assert_eq!(raw(&source_balance, &frame).value, 7);
        assert_eq!(core.typed_resources().unwrap().revision(), before_revision);

        let lease = core
            .resolve_typed_resource_lease(
                &resource,
                ResourceOwnerSelector::SourceEmployee,
                &source,
                Some(target_id),
            )
            .unwrap();
        let current = core
            .runtime_facts
            .resources
            .as_ref()
            .unwrap()
            .store
            .value(&lease)
            .unwrap();
        let _reservation = core
            .runtime_facts
            .resources
            .as_mut()
            .unwrap()
            .store
            .reserve(&[(lease, current.revision, 3)])
            .unwrap();
        let reserved_revision = core.typed_resources().unwrap().revision();
        let reserved_count = core.typed_resources().unwrap().reservation_count();

        for (effect_id, field, expected) in [
            ("balance", ResourceValueField::Balance, 7),
            ("reserved", ResourceValueField::Reserved, 3),
            ("available", ResourceValueField::Available, 4),
        ] {
            let current_plan =
                resource_plan(effect_id, ResourceOwnerSelector::SourceEmployee, field);
            source.authored = Some(Arc::new(current_plan.source().clone()));
            let frame = core
                .build_value_expr_frame(&current_plan, &source, Some(target_id), 10, None)
                .unwrap();
            assert_eq!(raw(&current_plan, &frame).value, expected);
        }
        assert_eq!(
            core.typed_resources().unwrap().revision(),
            reserved_revision
        );
        assert_eq!(
            core.typed_resources().unwrap().reservation_count(),
            reserved_count
        );

        core.runtime_fact_source = source.clone();
        core.apply_typed_resource_operations(
            &[ResourceOperation {
                resource: resource.clone(),
                owner: ResourceOwnerSelector::TargetEmployee,
                operation: ResourceMutation::Grant(4),
            }],
            &[target_id],
            11,
        )
        .unwrap();
        let target_balance = resource_plan(
            "target_balance",
            ResourceOwnerSelector::TargetEmployee,
            ResourceValueField::Balance,
        );
        source.authored = Some(Arc::new(target_balance.source().clone()));
        let target_frame = core
            .build_value_expr_frame(&target_balance, &source, Some(target_id), 12, None)
            .unwrap();
        assert_eq!(raw(&target_balance, &target_frame).value, 11);
        let source_frame = core
            .build_value_expr_frame(
                &source_balance,
                &RuntimeFactSource {
                    authored: Some(Arc::new(source_balance.source().clone())),
                    ..source
                },
                Some(target_id),
                12,
                None,
            )
            .unwrap();
        assert_eq!(raw(&source_balance, &source_frame).value, 7);
    }

    #[test]
    fn disconnected_dynamic_contexts_and_cancellation_are_typed_without_mutation() {
        let spell = plan(
            EvalMoment::Live,
            "spell",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Target,
                stat: StatSelector::SpellPower,
            }),
        );
        let result = plan(
            EvalMoment::Live,
            "result",
            ValueExpr::Read(ValueRef::Result {
                selector: ResultSelector::CurrentActionPrefix,
                field: ResultField::Count,
            }),
        );
        let aggregate = plan(
            EvalMoment::Live,
            "aggregate",
            ValueExpr::Aggregate {
                source: FiniteSource::ResolvedTargets,
                grouping: Grouping::WholeSet,
                reduction: Reduction::Count,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                    unit: ValueUnit::Magnitude,
                })),
            },
        );
        let (mut core, source_id, _) = core_with_units(Vec::new());
        let base = occurrence(&mut core, source_id, &spell);
        for (current, expected) in [
            (&spell, ValueExprFaultCode::MissingContext),
            (&result, ValueExprFaultCode::NotSettled),
            (&aggregate, ValueExprFaultCode::UnsupportedMoment),
        ] {
            let source = RuntimeFactSource {
                authored: Some(Arc::new(current.source().clone())),
                ..base.clone()
            };
            let capture = core
                .capture_value_expr_plan(
                    AmountPlanKind::Damage,
                    Arc::clone(current),
                    &source,
                    None,
                    10,
                    None,
                )
                .unwrap();
            let error = core
                .evaluate_value_expr_capture(&capture, None, 11)
                .unwrap_err();
            assert_eq!(error.code, expected);
            assert_eq!(error.site, current.source().site);
        }

        let tight_limits = ExecutionLimits {
            max_work: 1,
            ..Default::default()
        };
        let before = core.transaction_debug_snapshot();
        let mut observed = None;
        let result = control::run(
            &tight_limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                observed = Some(
                    core.capture_value_expr_plan(
                        AmountPlanKind::Damage,
                        Arc::clone(&spell),
                        &RuntimeFactSource {
                            authored: Some(Arc::new(spell.source().clone())),
                            ..base.clone()
                        },
                        None,
                        10,
                        None,
                    )
                    .unwrap_err()
                    .code,
                );
            },
        );
        assert!(result.is_err());
        assert_eq!(observed, Some(ValueExprFaultCode::BudgetExceeded));
        assert_eq!(core.transaction_debug_snapshot(), before);

        let cancellation = CancellationToken::default();
        let before = core.transaction_debug_snapshot();
        let mut observed = None;
        let result = control::run(
            &ExecutionLimits::default(),
            &cancellation,
            ExecutionPhase::TrialExecution,
            || {
                cancellation.cancel();
                observed = Some(
                    core.capture_value_expr_plan(
                        AmountPlanKind::Damage,
                        Arc::clone(&spell),
                        &RuntimeFactSource {
                            authored: Some(Arc::new(spell.source().clone())),
                            ..base
                        },
                        None,
                        10,
                        None,
                    )
                    .unwrap_err()
                    .code,
                );
            },
        );
        assert!(result.is_err());
        assert_eq!(observed, Some(ValueExprFaultCode::Cancelled));
        assert_eq!(core.transaction_debug_snapshot(), before);
    }

    fn actual_cast_binding_sources(
        same_actor: bool,
    ) -> (
        BattleCore,
        CompiledAmountEntry,
        RuntimeFactSource,
        RuntimeFactSource,
    ) {
        let skills = SkillDatabase::from_ron(
            "fixture/cast_binding.ron",
            r#"SkillDatabase(skills: [SkillDef(
                id: "wp02_runtime_frame", name: "binding", kind: Targeted, focus_time_ms: 0,
                use_lifetime: Action,
                cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
                steps: [SkillStepDef(id: "step_0", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "binding", op: DamageExpr(amount: (moment: Live, root: Read(Stat(subject: Source, stat: Attack))), damage_type: True))])]
            )])"#,
        )
        .unwrap();
        let (mut core, actor, other) = core_with_units_and_skills(Vec::new(), Some(skills));
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .next()
            .unwrap()
            .1
            .clone();
        core.invoke_ability(
            10,
            actor,
            "wp02_runtime_frame",
            None,
            Default::default(),
            false,
        );
        core.invoke_ability(
            11,
            if same_actor { actor } else { other },
            "wp02_runtime_frame",
            None,
            Default::default(),
            false,
        );
        assert_eq!(core.active_skill_casts.len(), 2);
        let mut sources: Vec<_> = core
            .active_skill_casts
            .values()
            .map(|cast| cast.fact_source.clone())
            .collect();
        sources.sort_by_key(|source| source.cast.as_ref().unwrap().cast_seq);
        let mut second = sources.pop().unwrap();
        let mut first = sources.pop().unwrap();
        first.authored = Some(Arc::new(entry.plan().source().clone()));
        second.authored = first.authored.clone();
        (core, entry, first, second)
    }

    #[test]
    fn actual_cast_action_binding_rejects_other_actor_and_same_actor_occurrence() {
        for same_actor in [false, true] {
            let (mut core, entry, first, second) = actual_cast_binding_sources(same_actor);
            let expected = if same_actor { 10 } else { 3 };
            for (source, damage) in [(&first, 10), (&second, expected)] {
                let capture = core
                    .capture_value_expr_entry(&entry, source, None, 12, None)
                    .unwrap();
                assert_eq!(
                    core.evaluate_value_expr_capture(&capture, None, 13)
                        .unwrap()
                        .amount,
                    RuntimeAmount::Damage(damage)
                );
            }
            let before = core.transaction_debug_snapshot();
            let mut wrong = first.clone();
            wrong.cast = second.cast.clone();
            assert_eq!(
                core.capture_value_expr_entry(&entry, &wrong, None, 12, None)
                    .unwrap_err()
                    .code,
                ValueExprFaultCode::Unavailable
            );
            assert_eq!(core.transaction_debug_snapshot(), before);
            assert!(core
                .capture_value_expr_entry(&entry, &first, None, 12, None)
                .is_ok());

            let basic = occurrence(&mut core, first.actor.unwrap(), entry.plan());
            assert!(basic.cast.is_none());
            assert!(core
                .capture_value_expr_entry(&entry, &basic, None, 12, None)
                .is_ok());
        }
    }

    #[test]
    fn bound_cast_delivery_survives_control_cleanup_and_departed_source_snapshot() {
        let (mut core, entry, mut source, _) = actual_cast_binding_sources(true);
        let action = source.action.clone().unwrap();
        let cast_seq = source.cast.as_ref().unwrap().cast_seq;
        source.delivery = core.runtime_facts.delivery(&action).unwrap();
        core.runtime_facts.dispatched(&action, 0, 12).unwrap();
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .outstanding_deliveries,
            1
        );
        core.active_skill_casts.remove(&cast_seq);
        let live = core
            .capture_value_expr_entry(&entry, &source, None, 13, None)
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&live, None, 14)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(10)
        );

        let snapshot = plan(
            EvalMoment::Snapshot,
            "departed_cast_source",
            ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::Source,
                stat: StatSelector::Attack,
            }),
        );
        source.authored = Some(Arc::new(snapshot.source().clone()));
        let capture = core
            .capture_value_expr_plan(AmountPlanKind::Damage, snapshot, &source, None, 15, None)
            .unwrap();
        core.withdraw_unit(source.actor.unwrap(), 16).unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture(&capture, None, 17)
                .unwrap()
                .amount,
            RuntimeAmount::Damage(10)
        );
    }

    fn ordered_skill(effects: &str) -> SkillDatabase {
        SkillDatabase::from_ron(
            "fixture/wp02_ordered_runtime.ron",
            &format!(
                r#"SkillDatabase(skills: [SkillDef(
                    id: "wp02_runtime_frame", name: "ordered", kind: Targeted,
                    focus_time_ms: 0,
                    use_lifetime: Action,
                    cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
                    steps: [SkillStepDef(
                        id: "step_0", delay_ms: 0, target: EnemySingle(rule: Nearest),
                        delivery: Instant, effects: [{effects}]
                    )]
                )])"#
            ),
        )
        .unwrap()
    }

    fn ordered_fixture(
        effects: &str,
    ) -> (
        BattleCore,
        UnitInstanceId,
        UnitInstanceId,
        crate::game::ability::SkillDef,
        RuntimeFactSource,
    ) {
        let skills = ordered_skill(effects);
        let (mut core, source_id, target_id) = core_with_units_and_skills(Vec::new(), Some(skills));
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let source = occurrence(&mut core, source_id, entry.plan());
        core.runtime_fact_source = source.clone();
        (core, source_id, target_id, skill, source)
    }

    fn resolved_attack(
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
    ) -> ValueExpr {
        ValueExpr::Aggregate {
            source: FiniteSource::ResolvedTargets,
            grouping,
            reduction,
            outer_reduction,
            member: Box::new(ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::AggregateMember,
                stat: StatSelector::Attack,
            })),
        }
    }

    fn resolved_attack_from(
        source: FiniteSource,
        grouping: Grouping,
        reduction: Reduction,
        outer_reduction: Option<Reduction>,
    ) -> ValueExpr {
        ValueExpr::Aggregate {
            source,
            grouping,
            reduction,
            outer_reduction,
            member: Box::new(ValueExpr::Read(ValueRef::Stat {
                subject: ValueSubject::AggregateMember,
                stat: StatSelector::Attack,
            })),
        }
    }

    fn evaluate_resolved_plan(
        core: &mut BattleCore,
        actor: UnitInstanceId,
        plan: &Arc<CompiledAmountPlan>,
        targets: &[UnitInstanceId],
    ) -> (TypedInteger, ResolvedMemberFrame) {
        let source = occurrence(core, actor, plan);
        let members = core
            .prepare_resolved_target_members(plan, Some(targets))
            .unwrap()
            .unwrap();
        let frame = core
            .build_value_expr_frame_with_members(
                plan,
                &source,
                targets.first().copied(),
                10,
                None,
                None,
                0,
                Some(&members),
            )
            .unwrap();
        (raw(plan, &frame), members)
    }

    fn reset_resolved_slot_item_lookups() {
        RESOLVED_SLOT_ITEM_LOOKUPS.with(|lookups| lookups.set(0));
    }

    fn resolved_slot_item_lookups() -> usize {
        RESOLVED_SLOT_ITEM_LOOKUPS.with(std::cell::Cell::get)
    }

    #[test]
    fn equipment_lookup_count_is_constant_when_unrelated_battle_items_grow() {
        let mut observed = None;
        for unrelated in [0, 100, 1_000] {
            let fixture = owner_fixture_with_unrelated(None, unrelated);
            let current = plan(
                EvalMoment::Live,
                "direct_equipment_lookup",
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
                    ]),
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Weapon,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
            );
            reset_resolved_slot_item_lookups();
            let prepared = fixture
                .core
                .prepare_resolved_target_members(&current, Some(&[fixture.employee_a]))
                .unwrap()
                .unwrap();
            assert_eq!(prepared.views[0].members.len(), 1);
            let lookups = resolved_slot_item_lookups();
            assert_eq!(lookups, 4);
            assert_eq!(observed.get_or_insert(lookups), &lookups);
            assert_eq!(fixture.core.items.len(), unrelated + 3);
        }
    }

    #[test]
    fn equipment_lookup_count_tracks_predicates_relations_members_and_nodes() {
        let fixture = owner_fixture(None);
        let targets = [fixture.employee_a, fixture.employee_b];
        let cases = [
            (
                "predicate_only",
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
                    ]),
                    Grouping::WholeSet,
                    Reduction::Sum,
                    None,
                ),
                4,
            ),
            (
                "relation_only",
                resolved_attack_from(
                    FiniteSource::ResolvedTargets,
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Weapon,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                4,
            ),
            (
                "predicate_and_relation",
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
                    ]),
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Weapon,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                8,
            ),
        ];
        for (name, expression, expected) in cases {
            let current = plan(EvalMoment::Live, name, expression);
            reset_resolved_slot_item_lookups();
            fixture
                .core
                .prepare_resolved_target_members(&current, Some(&targets))
                .unwrap()
                .unwrap();
            assert_eq!(resolved_slot_item_lookups(), expected, "{name}");
        }

        let multiple = plan(
            EvalMoment::Live,
            "multiple_equipment_nodes",
            ValueExpr::Add(vec![
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
                    ]),
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Weapon,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Armor),
                    ]),
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Armor,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
            ]),
        );
        reset_resolved_slot_item_lookups();
        let prepared = fixture
            .core
            .prepare_resolved_target_members(&multiple, Some(&targets))
            .unwrap()
            .unwrap();
        assert_eq!(prepared.views.len(), 2);
        assert_eq!(resolved_slot_item_lookups(), 12);
        assert!(resolved_slot_item_lookups() <= 2 * targets.len() * (2 + 2));
    }

    #[test]
    fn actual_units_group_by_side_employee_and_equipment_copy_without_view_contamination() {
        let mut fixture = owner_fixture(None);
        let targets = [
            fixture.employee_a,
            fixture.employee_b,
            fixture.summon,
            fixture.enemy,
            fixture.employee_b,
        ];

        let side = plan(
            EvalMoment::Live,
            "side",
            resolved_attack_from(
                FiniteSource::ResolvedTargets,
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::Side),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        );
        let (side_value, side_members) = evaluate_resolved_plan(
            &mut fixture.core,
            fixture.employee_a,
            &side,
            &[
                fixture.employee_a,
                fixture.employee_b,
                fixture.enemy,
                fixture.employee_b,
            ],
        );
        assert_eq!(side_value.value, 12);
        assert_eq!(side_members.views[0].members.len(), 3);
        assert!(side_members.views[0].members.iter().all(|member| member
            .owner
            .as_ref()
            .unwrap()
            .domain
            == GroupOwnerDomain::Side));

        let employee_unfiltered = plan(
            EvalMoment::Live,
            "employee_unfiltered",
            resolved_attack_from(
                FiniteSource::ResolvedTargets,
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::ActualEmployee),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        );
        assert_eq!(
            fixture
                .core
                .prepare_resolved_target_members(&employee_unfiltered, Some(&targets))
                .unwrap_err()
                .code,
            ValueExprFaultCode::InvalidOwner
        );

        let employee_source =
            FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::IsActualEmployee]);
        let employee = plan(
            EvalMoment::Live,
            "employee",
            resolved_attack_from(
                employee_source.clone(),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::ActualEmployee),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        );
        let (employee_value, employee_members) =
            evaluate_resolved_plan(&mut fixture.core, fixture.employee_a, &employee, &targets);
        assert_eq!(employee_value.value, 8);
        assert_eq!(employee_members.views[0].members.len(), 2);
        assert!(employee_members.views[0]
            .members
            .iter()
            .all(|member| { member.owner.as_ref().unwrap().domain == GroupOwnerDomain::Employee }));

        let weapon_source = FiniteSource::ResolvedTargetsWhere(vec![
            ResolvedTargetPredicate::IsActualEmployee,
            ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
        ]);
        let weapon = plan(
            EvalMoment::Live,
            "weapon",
            resolved_attack_from(
                weapon_source.clone(),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                    EquipmentSlotAddress::Weapon,
                )),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        );
        let (weapon_value, weapon_members) =
            evaluate_resolved_plan(&mut fixture.core, fixture.employee_a, &weapon, &targets);
        assert_eq!(weapon_value.value, 8);
        let weapon_keys = weapon_members.views[0]
            .members
            .iter()
            .map(|member| member.owner.as_ref().unwrap().identity.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            weapon_keys,
            BTreeSet::from([
                fixture.weapon_x.hyphenated().to_string(),
                fixture.weapon_z.hyphenated().to_string(),
            ])
        );

        let armor = plan(
            EvalMoment::Live,
            "armor",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![
                    ResolvedTargetPredicate::IsActualEmployee,
                    ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Armor),
                ]),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                    EquipmentSlotAddress::Armor,
                )),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        );
        let (armor_value, armor_members) =
            evaluate_resolved_plan(&mut fixture.core, fixture.employee_a, &armor, &targets);
        assert_eq!(armor_value.value, 3);
        assert_eq!(armor_members.views[0].members.len(), 1);
        assert_eq!(
            armor_members.views[0].members[0]
                .owner
                .as_ref()
                .unwrap()
                .identity,
            fixture.armor_y.hyphenated().to_string()
        );

        let combined = plan(
            EvalMoment::Live,
            "combined",
            ValueExpr::Add(vec![
                resolved_attack_from(
                    FiniteSource::ResolvedTargets,
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::Side),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                resolved_attack_from(
                    employee_source,
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::ActualEmployee),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                resolved_attack_from(
                    weapon_source,
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Weapon,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
                resolved_attack_from(
                    FiniteSource::ResolvedTargetsWhere(vec![
                        ResolvedTargetPredicate::IsActualEmployee,
                        ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Armor),
                    ]),
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                        EquipmentSlotAddress::Armor,
                    )),
                    Reduction::Max,
                    Some(Reduction::Sum),
                ),
            ]),
        );
        let (combined_value, combined_members) =
            evaluate_resolved_plan(&mut fixture.core, fixture.employee_a, &combined, &targets);
        assert_eq!(combined_value.value, 37);
        assert_eq!(combined_members.views.len(), 4);
        assert_eq!(
            combined_members
                .views
                .iter()
                .map(|view| view.members.len())
                .collect::<Vec<_>>(),
            vec![4, 2, 2, 1]
        );
    }

    #[test]
    fn equipment_relation_distinguishes_empty_unknown_missing_copy_and_stale_identity() {
        let fixture = owner_fixture(None);
        let relation = |address| {
            plan(
                EvalMoment::Live,
                "slot_fault",
                resolved_attack_from(
                    FiniteSource::ResolvedTargets,
                    Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(address)),
                    Reduction::Sum,
                    Some(Reduction::Sum),
                ),
            )
        };

        let armor = relation(EquipmentSlotAddress::Armor);
        assert_eq!(
            fixture
                .core
                .prepare_resolved_target_members(&armor, Some(&[fixture.employee_b]))
                .unwrap_err()
                .code,
            ValueExprFaultCode::InvalidOwner
        );
        let armor_filtered = plan(
            EvalMoment::Live,
            "empty_slot_filtered",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::HasEquippedSlot(
                    EquipmentSlotAddress::Armor,
                )]),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                    EquipmentSlotAddress::Armor,
                )),
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        );
        let filtered = fixture
            .core
            .prepare_resolved_target_members(&armor_filtered, Some(&[fixture.employee_b]))
            .unwrap()
            .unwrap();
        assert!(filtered.views[0].members.is_empty());

        let weapon = relation(EquipmentSlotAddress::Weapon);
        assert_eq!(
            fixture
                .core
                .prepare_resolved_target_members(&weapon, Some(&[fixture.summon]))
                .unwrap_err()
                .code,
            ValueExprFaultCode::MissingContext
        );
        let wrong_layout = relation(EquipmentSlotAddress::Any3(0));
        assert_eq!(
            fixture
                .core
                .prepare_resolved_target_members(&wrong_layout, Some(&[fixture.employee_a]))
                .unwrap_err()
                .code,
            ValueExprFaultCode::InvalidOwner
        );

        let mut any3 = owner_fixture(None);
        let slots = Arc::make_mut(
            any3.core
                .units
                .get_mut(&any3.employee_a)
                .unwrap()
                .equipment_slots
                .as_mut()
                .unwrap(),
        );
        slots.layout = SlotLayoutKind::Any3;
        slots.slots[0].as_mut().unwrap().binding.slot = Some(EquipmentSlotAddress::Any3(0));
        slots.slots[1] = None;
        let any3_plan = plan(
            EvalMoment::Live,
            "any3_slot",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::HasEquippedSlot(
                    EquipmentSlotAddress::Any3(0),
                )]),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                    EquipmentSlotAddress::Any3(0),
                )),
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        );
        let (any3_value, any3_members) = evaluate_resolved_plan(
            &mut any3.core,
            any3.employee_a,
            &any3_plan,
            &[any3.employee_a],
        );
        assert_eq!(any3_value.value, 3);
        assert_eq!(
            any3_members.views[0].members[0]
                .owner
                .as_ref()
                .unwrap()
                .identity,
            any3.weapon_x.hyphenated().to_string()
        );

        let short_circuit = plan(
            EvalMoment::Live,
            "short_circuit",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![
                    ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Weapon),
                    ResolvedTargetPredicate::IsActualEmployee,
                ]),
                Grouping::WholeSet,
                Reduction::Sum,
                None,
            ),
        );
        let prepared = fixture
            .core
            .prepare_resolved_target_members(&short_circuit, Some(&[fixture.summon]))
            .unwrap()
            .unwrap();
        assert!(prepared.views[0].members.is_empty());

        let mut no_copy = owner_fixture(None);
        let slots = Arc::make_mut(
            no_copy
                .core
                .units
                .get_mut(&no_copy.employee_a)
                .unwrap()
                .equipment_slots
                .as_mut()
                .unwrap(),
        );
        slots.slots[0]
            .as_mut()
            .unwrap()
            .binding
            .persistent_copy_uuid = None;
        assert_eq!(
            no_copy
                .core
                .prepare_resolved_target_members(&weapon, Some(&[no_copy.employee_a]))
                .unwrap_err()
                .code,
            ValueExprFaultCode::InvalidOwner
        );

        let mut stale = owner_fixture(None);
        stale
            .core
            .items
            .retain(|_, item| item.owner_unit_instance != stale.employee_a);
        assert_eq!(
            stale
                .core
                .prepare_resolved_target_members(&weapon, Some(&[stale.employee_a]))
                .unwrap_err()
                .code,
            ValueExprFaultCode::InvalidOwner
        );
    }

    #[test]
    fn equipment_relation_rejects_every_direct_runtime_item_identity_mismatch() {
        let current = plan(
            EvalMoment::Live,
            "direct_identity_fault",
            resolved_attack_from(
                FiniteSource::ResolvedTargets,
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::EquippedSlot(
                    EquipmentSlotAddress::Weapon,
                )),
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        );
        let reject = |fixture: &OwnerFixture, expected_lookups| {
            reset_resolved_slot_item_lookups();
            assert_eq!(
                fixture
                    .core
                    .prepare_resolved_target_members(&current, Some(&[fixture.employee_a]),)
                    .unwrap_err()
                    .code,
                ValueExprFaultCode::InvalidOwner
            );
            assert_eq!(resolved_slot_item_lookups(), expected_lookups);
        };

        let mut wrong_slot_id = owner_fixture(None);
        let other_id = wrong_slot_id.core.units[&wrong_slot_id.employee_b]
            .equipment_slots
            .as_ref()
            .unwrap()
            .slots[0]
            .unwrap()
            .runtime_item_instance_id;
        Arc::make_mut(
            wrong_slot_id
                .core
                .units
                .get_mut(&wrong_slot_id.employee_a)
                .unwrap()
                .equipment_slots
                .as_mut()
                .unwrap(),
        )
        .slots[0]
            .as_mut()
            .unwrap()
            .runtime_item_instance_id = other_id;
        reject(&wrong_slot_id, 1);

        let mut nil_slot_id = owner_fixture(None);
        Arc::make_mut(
            nil_slot_id
                .core
                .units
                .get_mut(&nil_slot_id.employee_a)
                .unwrap()
                .equipment_slots
                .as_mut()
                .unwrap(),
        )
        .slots[0]
            .as_mut()
            .unwrap()
            .runtime_item_instance_id = Uuid::nil();
        reject(&nil_slot_id, 0);

        let mut missing = owner_fixture(None);
        let item_id = missing.core.units[&missing.employee_a]
            .equipment_slots
            .as_ref()
            .unwrap()
            .slots[0]
            .unwrap()
            .runtime_item_instance_id;
        missing.core.items.remove(&item_id);
        reject(&missing, 1);

        for corrupt in ["instance", "owner", "base", "copy"] {
            let mut fixture = owner_fixture(None);
            let item_id = fixture.core.units[&fixture.employee_a]
                .equipment_slots
                .as_ref()
                .unwrap()
                .slots[0]
                .unwrap()
                .runtime_item_instance_id;
            let item = fixture.core.items.get_mut(&item_id).unwrap();
            match corrupt {
                "instance" => item.instance_id = Uuid::from_u128(0xDEAD),
                "owner" => item.owner_unit_instance = fixture.employee_b,
                "base" => item.base_uuid = Uuid::from_u128(0xBADD),
                "copy" => item.persistent_copy_uuid = Some(Uuid::from_u128(0xBAD_C0FE)),
                _ => unreachable!(),
            }
            reject(&fixture, 1);
        }
    }

    #[test]
    fn filtered_empty_reductions_and_snapshot_views_keep_their_exact_observation() {
        let mut fixture = owner_fixture(None);
        let empty_source = FiniteSource::ResolvedTargetsWhere(vec![
            ResolvedTargetPredicate::IsActualEmployee,
            ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Accessory),
        ]);
        let targets = [fixture.employee_a, fixture.employee_b, fixture.summon];
        for (reduction, expected) in [(Reduction::Count, 0), (Reduction::Sum, 0)] {
            let member = if reduction == Reduction::Count {
                ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::AggregateMember,
                    stat: StatSelector::Attack,
                })
            } else {
                ValueExpr::Read(ValueRef::Stat {
                    subject: ValueSubject::AggregateMember,
                    stat: StatSelector::Attack,
                })
            };
            let current = plan(
                EvalMoment::Live,
                "empty_filtered",
                ValueExpr::Aggregate {
                    source: empty_source.clone(),
                    grouping: Grouping::WholeSet,
                    reduction,
                    outer_reduction: None,
                    member: Box::new(member),
                },
            );
            let (value, members) =
                evaluate_resolved_plan(&mut fixture.core, fixture.employee_a, &current, &targets);
            assert_eq!(value.value, expected);
            assert!(members.views[0].members.is_empty());
        }
        let maximum = plan(
            EvalMoment::Live,
            "empty_filtered_max",
            resolved_attack_from(empty_source, Grouping::WholeSet, Reduction::Max, None),
        );
        let source = occurrence(&mut fixture.core, fixture.employee_a, &maximum);
        let members = fixture
            .core
            .prepare_resolved_target_members(&maximum, Some(&targets))
            .unwrap()
            .unwrap();
        let frame = fixture
            .core
            .build_value_expr_frame_with_members(
                &maximum,
                &source,
                Some(fixture.employee_a),
                10,
                None,
                None,
                0,
                Some(&members),
            )
            .unwrap();
        let mut meter = ExprMeter::new(maximum.evaluation_limits()).unwrap();
        assert_eq!(
            evaluate(&maximum, &frame, &mut meter).unwrap_err().code,
            ValueExprFaultCode::Unavailable
        );

        let snapshot_plan = plan(
            EvalMoment::Snapshot,
            "snapshot_owner",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![ResolvedTargetPredicate::IsActualEmployee]),
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::ActualEmployee),
                Reduction::Sum,
                Some(Reduction::Sum),
            ),
        );
        let snapshot = fixture
            .core
            .prepare_resolved_target_members(&snapshot_plan, Some(&targets))
            .unwrap()
            .unwrap();
        let frozen = Arc::clone(&snapshot.views);
        fixture
            .core
            .units
            .get_mut(&fixture.employee_a)
            .unwrap()
            .stats
            .attack = 9;
        let live = fixture
            .core
            .prepare_resolved_target_members(&snapshot_plan, Some(&targets))
            .unwrap()
            .unwrap();
        let attack_ref = ValueRef::Stat {
            subject: ValueSubject::AggregateMember,
            stat: StatSelector::Attack,
        };
        assert_eq!(frozen[0].members.len(), 2);
        assert_eq!(frozen[0].members[0].values[&attack_ref].value, 3);
        assert_eq!(live.views[0].members[0].values[&attack_ref].value, 9);
        assert_eq!(frozen[0].members[0].owner, live.views[0].members[0].owner);
    }

    #[test]
    fn resolved_view_total_limit_and_filtered_scan_work_are_enforced() {
        let fixture = owner_fixture(None);
        let root = ValueExpr::Add(vec![
            resolved_attack_from(
                FiniteSource::ResolvedTargets,
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::Side),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
            resolved_attack_from(
                FiniteSource::ResolvedTargets,
                Grouping::ByResolvedOwner(ResolvedOwnerRelation::Side),
                Reduction::Max,
                Some(Reduction::Sum),
            ),
        ]);
        let targets = [fixture.employee_a, fixture.employee_b, fixture.enemy];
        for (limit, accepted) in [(6, true), (5, false)] {
            let mut limits = ExprLimits::default();
            limits.max_dynamic_members = limit;
            let current = Arc::new(
                compile(
                    &AmountExpr::authored(EvalMoment::Live, root.clone()),
                    &source("view_total_limit"),
                    DeliveryCapabilities::all(),
                    &limits,
                )
                .unwrap(),
            );
            let result = fixture
                .core
                .prepare_resolved_target_members(&current, Some(&targets));
            if accepted {
                let prepared = result.unwrap().unwrap();
                assert_eq!(
                    prepared
                        .views
                        .iter()
                        .map(|view| view.members.len())
                        .sum::<usize>(),
                    limit
                );
            } else {
                assert_eq!(result.unwrap_err().code, ValueExprFaultCode::BudgetExceeded);
            }
        }

        let filtered_empty = plan(
            EvalMoment::Live,
            "filtered_scan_work",
            resolved_attack_from(
                FiniteSource::ResolvedTargetsWhere(vec![
                    ResolvedTargetPredicate::IsActualEmployee,
                    ResolvedTargetPredicate::HasEquippedSlot(EquipmentSlotAddress::Accessory),
                ]),
                Grouping::WholeSet,
                Reduction::Sum,
                None,
            ),
        );
        let token = CancellationToken::default();
        let result = control::run(
            &ExecutionLimits {
                max_work: 1,
                ..Default::default()
            },
            &token,
            ExecutionPhase::TrialExecution,
            || {
                fixture
                    .core
                    .prepare_resolved_target_members(&filtered_empty, Some(&targets))
            },
        );
        assert!(matches!(result, Err(error) if error.code == "authoring_work_limit"));
    }

    #[test]
    fn employee_filter_changes_only_aggregate_input_and_relation_fault_rolls_back() {
        let employee_amount = r#"Aggregate(
            source: ResolvedTargetsWhere([IsActualEmployee]),
            grouping: ByResolvedOwner(ActualEmployee),
            reduction: Max, outer_reduction: Some(Sum),
            member: Read(Stat(subject: AggregateMember, stat: Attack)))"#;
        let effects = format!(
            r#"(id: "employee_damage", op: DamageExpr(
                amount: (moment: Live, root: {employee_amount}), damage_type: True))"#,
        );
        let mut fixture = owner_fixture(Some(ordered_skill(&effects)));
        let skill = fixture
            .core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = fixture
            .core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let runtime_source = occurrence(&mut fixture.core, fixture.employee_a, entry.plan());
        fixture.core.runtime_fact_source = runtime_source;
        let targets = [fixture.employee_a, fixture.employee_b, fixture.summon];
        let result = fixture
            .core
            .execute_ordered_skill_effects(
                fixture.employee_a,
                &skill.id,
                &skill.steps[0],
                &targets,
                10,
                10,
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.resolved_target_count, targets.len());
        for target in targets {
            assert_eq!(fixture.core.units[&target].stats.current_health, 92);
        }

        let faulting_effects = format!(
            r#"(id: "first", op: Damage(amount: 7, damage_type: True)),
                (id: "bad_relation", op: HealExpr(amount: (moment: Live,
                    root: Aggregate(source: ResolvedTargets,
                        grouping: ByResolvedOwner(ActualEmployee),
                        reduction: Sum, outer_reduction: Some(Sum),
                        member: Read(Stat(subject: AggregateMember, stat: Attack))))))"#,
        );
        let mut faulting = owner_fixture(Some(ordered_skill(&faulting_effects)));
        let skill = faulting
            .core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = faulting
            .core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let runtime_source = occurrence(&mut faulting.core, faulting.employee_a, entry.plan());
        faulting.core.runtime_fact_source = runtime_source;
        let before = faulting.core.transaction_debug_snapshot();
        let caster = faulting.employee_a;
        let targets = [faulting.employee_b, faulting.summon];
        let error = faulting
            .core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    caster,
                    &skill.id,
                    &skill.steps[0],
                    &targets,
                    10,
                    10,
                    None,
                    None,
                )
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::InvalidOwner,
                ..
            })
        ));
        assert_eq!(faulting.core.transaction_debug_snapshot(), before);
    }

    #[test]
    fn actual_resolved_members_canonicalize_identity_and_read_projected_tiles() {
        let (mut core, source_id, target_id) = core_with_units(Vec::new());
        core.units.get_mut(&source_id).unwrap().stats.attack = 2;
        core.units.get_mut(&target_id).unwrap().stats.attack = 5;
        let third_id = UnitInstanceId(Uuid::from_u128(0x44_1234));
        let mut third = core.units[&target_id].clone();
        third.instance_id = third_id;
        third.stats.attack = 3;
        core.units.insert(third_id, third);
        for (grouping, inner, outer, expected) in [
            (Grouping::WholeSet, Reduction::Sum, None, 10),
            (Grouping::WholeSet, Reduction::Max, None, 5),
            (Grouping::WholeSet, Reduction::Count, None, 3),
            (
                Grouping::ByProjectedTile,
                Reduction::Max,
                Some(Reduction::Sum),
                7,
            ),
            (
                Grouping::ByProjectedTile,
                Reduction::Count,
                Some(Reduction::Max),
                2,
            ),
        ] {
            let plan = plan(
                EvalMoment::Live,
                "aggregate",
                resolved_attack(grouping, inner, outer),
            );
            let source = occurrence(&mut core, source_id, &plan);
            let mut outcomes = Vec::new();
            for targets in [
                vec![source_id, target_id, third_id, target_id],
                vec![third_id, source_id, target_id],
            ] {
                let members = core
                    .prepare_resolved_target_members(&plan, Some(&targets))
                    .unwrap()
                    .unwrap();
                let frame = core
                    .build_value_expr_frame_with_members(
                        &plan,
                        &source,
                        Some(target_id),
                        10,
                        None,
                        None,
                        0,
                        Some(&members),
                    )
                    .unwrap();
                let mut meter = ExprMeter::new(plan.evaluation_limits()).unwrap();
                let outcome = evaluate(&plan, &frame, &mut meter).unwrap();
                assert_eq!(outcome.value.value, expected);
                let usage = meter.usage();
                assert_eq!(usage.raw_members, 3);
                assert_eq!(usage.dynamic_members, 3);
                assert_eq!(
                    usage.groups,
                    if grouping == Grouping::WholeSet { 1 } else { 2 }
                );
                println!(
                    "D04_WORKLOAD grouping={grouping:?} reduction={inner:?} raw_input={} canonical={} retained={} usage={usage:?} amount={expected}",
                    targets.len(),
                    members.views[0].members.len(),
                    members.retained_extent,
                );
                outcomes.push(outcome);
            }
            assert_eq!(outcomes[0].trace, outcomes[1].trace);
            assert_eq!(outcomes[0].value, outcomes[1].value);
        }
    }

    #[test]
    fn empty_resolved_set_is_distinct_from_missing_and_unsupported_context() {
        let (mut core, source_id, _) = core_with_units(Vec::new());
        for (reduction, expected) in [(Reduction::Count, 0), (Reduction::Sum, 0)] {
            let plan = plan(
                EvalMoment::Live,
                "aggregate",
                resolved_attack(Grouping::WholeSet, reduction, None),
            );
            let source = occurrence(&mut core, source_id, &plan);
            let empty = core
                .prepare_resolved_target_members(&plan, Some(&[]))
                .unwrap()
                .unwrap();
            let frame = core
                .build_value_expr_frame_with_members(
                    &plan,
                    &source,
                    None,
                    10,
                    None,
                    None,
                    0,
                    Some(&empty),
                )
                .unwrap();
            assert_eq!(raw(&plan, &frame).value, expected);
            assert_eq!(
                core.prepare_resolved_target_members(&plan, None)
                    .err()
                    .unwrap()
                    .code,
                ValueExprFaultCode::UnsupportedMoment
            );
            let mut meter = ExprMeter::new(plan.evaluation_limits()).unwrap();
            assert_eq!(
                evaluate(&plan, &EvalFrame::default(), &mut meter)
                    .unwrap_err()
                    .code,
                ValueExprFaultCode::MissingContext
            );
        }
        let max = plan(
            EvalMoment::Live,
            "aggregate",
            resolved_attack(Grouping::WholeSet, Reduction::Max, None),
        );
        let source = occurrence(&mut core, source_id, &max);
        let empty = core
            .prepare_resolved_target_members(&max, Some(&[]))
            .unwrap()
            .unwrap();
        let frame = core
            .build_value_expr_frame_with_members(
                &max,
                &source,
                None,
                10,
                None,
                None,
                0,
                Some(&empty),
            )
            .unwrap();
        let mut meter = ExprMeter::new(max.evaluation_limits()).unwrap();
        assert_eq!(
            evaluate(&max, &frame, &mut meter).unwrap_err().code,
            ValueExprFaultCode::Unavailable
        );
        let owner = plan(
            EvalMoment::Live,
            "aggregate",
            resolved_attack(Grouping::ByOwner, Reduction::Sum, Some(Reduction::Sum)),
        );
        assert_eq!(
            core.prepare_resolved_target_members(&owner, Some(&[source_id]))
                .err()
                .unwrap()
                .code,
            ValueExprFaultCode::InvalidOwner
        );
        let scalar = plan(
            EvalMoment::Live,
            "aggregate",
            ValueExpr::Aggregate {
                source: FiniteSource::ResolvedTargets,
                grouping: Grouping::WholeSet,
                reduction: Reduction::Sum,
                outer_reduction: None,
                member: Box::new(ValueExpr::Read(ValueRef::AggregateMember {
                    unit: ValueUnit::Magnitude,
                })),
            },
        );
        assert_eq!(
            core.prepare_resolved_target_members(&scalar, Some(&[source_id]))
                .err()
                .unwrap()
                .code,
            ValueExprFaultCode::MissingContext
        );
    }

    #[test]
    fn ordered_effect_uses_one_live_member_set_before_mutation_then_rebuilds_it() {
        let amount = r#"Ratio(
            numerator: Aggregate(source: ResolvedTargets, grouping: WholeSet,
                reduction: Sum, outer_reduction: None,
                member: Read(HitPoints(subject: AggregateMember, field: Current))),
            denominator: Const((value: 4, unit: Scalar)), rounding: TowardZero)"#;
        let effects = format!(
            r#"(id: "first", op: DamageExpr(amount: (moment: Live, root: {amount}), damage_type: True)),
               (id: "second", op: DamageExpr(amount: (moment: Live, root: {amount}), damage_type: True))"#,
        );
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(&effects);
        for identity in [source_id, target_id] {
            let unit = core.units.get_mut(&identity).unwrap();
            unit.stats.max_health = 10;
            unit.stats.current_health = 10;
        }
        core.execute_ordered_skill_effects(
            source_id,
            &skill.id,
            &skill.steps[0],
            &[source_id, target_id],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        assert_eq!(core.units[&source_id].stats.current_health, 3);
        assert_eq!(core.units[&target_id].stats.current_health, 3);
    }

    #[test]
    fn released_member_values_and_tiles_remain_snapshot_while_live_advances() {
        let amount = r#"Aggregate(source: ResolvedTargets,
            grouping: ByProjectedTile, reduction: Max, outer_reduction: Some(Sum),
            member: Read(Stat(subject: AggregateMember, stat: Attack)))"#;
        let effects = format!(
            r#"(id: "snapshot", op: DamageExpr(amount: (moment: Snapshot, root: {amount}), damage_type: True)),
               (id: "live", op: DamageExpr(amount: (moment: Live, root: {amount}), damage_type: True))"#,
        );
        let (mut core, source_id, target_id, skill, source) = ordered_fixture(&effects);
        core.units.get_mut(&source_id).unwrap().stats.attack = 2;
        core.units.get_mut(&target_id).unwrap().stats.attack = 5;
        let targets = [source_id, target_id];
        let carrier = core
            .build_value_expr_delivery_capture(&skill.steps[0], &source, Some(&targets), 10, None)
            .unwrap()
            .unwrap();
        let snapshot = carrier.capture_for(0, Some(target_id)).unwrap();
        let live = carrier.capture_for(1, Some(target_id)).unwrap();
        let new_position = core.units[&source_id].body.position;
        let target = core.units.get_mut(&target_id).unwrap();
        target.stats.attack = 9;
        target.body.position = new_position;
        let live_members = core
            .prepare_resolved_target_members(live.plan(), Some(&targets))
            .unwrap()
            .unwrap();
        let snapshot_amount = core
            .evaluate_value_expr_capture_with_resolved(snapshot, Some(target_id), 11, None, 0, None)
            .unwrap();
        let live_amount = core
            .evaluate_value_expr_capture_with_resolved(
                live,
                Some(target_id),
                11,
                None,
                1,
                Some(&live_members),
            )
            .unwrap();
        assert_eq!(snapshot_amount.amount, RuntimeAmount::Damage(7));
        assert_eq!(live_amount.amount, RuntimeAmount::Damage(9));
    }

    #[test]
    fn resolved_member_preparation_admits_exact_native_extent_before_copy() {
        use crate::game::battle::runtime_control as native;
        let (core, source_id, target_id) = core_with_units(Vec::new());
        let plan = plan(
            EvalMoment::Live,
            "aggregate",
            resolved_attack(Grouping::WholeSet, Reduction::Sum, None),
        );
        let targets = [source_id, target_id, target_id];
        let required = core
            .prepare_resolved_target_members(&plan, Some(&targets))
            .unwrap()
            .unwrap()
            .retained_extent;
        let rejected = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: required - 1,
                ..Default::default()
            },
            || {
                let error = core
                    .prepare_resolved_target_members(&plan, Some(&targets))
                    .err()
                    .unwrap();
                assert_eq!(error.code, ValueExprFaultCode::BudgetExceeded);
                Ok(())
            },
        );
        assert!(matches!(rejected, Err(GameError::BattleRuntimeFault(_))));
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: required,
                ..Default::default()
            },
            || {
                let accepted = core
                    .prepare_resolved_target_members(&plan, Some(&targets))
                    .unwrap()
                    .unwrap();
                assert_eq!(accepted.views.len(), 1);
                assert_eq!(accepted.views[0].members.len(), 2);
                Ok(())
            },
        )
        .unwrap();
        let token = CancellationToken::default();
        let low_work = control::run(
            &ExecutionLimits {
                max_work: 1,
                ..Default::default()
            },
            &token,
            ExecutionPhase::TrialExecution,
            || core.prepare_resolved_target_members(&plan, Some(&targets)),
        );
        assert!(matches!(low_work, Err(error) if error.code == "authoring_work_limit"));
        token.cancel();
        let cancelled = control::run(
            &ExecutionLimits::default(),
            &token,
            ExecutionPhase::TrialExecution,
            || core.prepare_resolved_target_members(&plan, Some(&targets)),
        );
        assert!(matches!(cancelled, Err(error) if error.code == "authoring_cancelled"));
    }

    #[test]
    fn aggregate_fault_restores_the_current_candidate_and_allows_clean_retry() {
        let effects = r#"
            (id: "damage", op: Damage(amount: 7, damage_type: True)),
            (id: "bad_owner", op: HealExpr(amount: (moment: Live,
                root: Aggregate(source: ResolvedTargets, grouping: ByOwner,
                    reduction: Sum, outer_reduction: Some(Sum),
                    member: Read(Stat(subject: AggregateMember, stat: Attack)))))),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        let before = core.transaction_debug_snapshot();
        let error = core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    source_id,
                    &skill.id,
                    &skill.steps[0],
                    &[target_id],
                    10,
                    10,
                    None,
                    None,
                )
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::InvalidOwner,
                ..
            })
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        let mut valid_step = skill.steps[0].clone();
        valid_step.effects.truncate(1);
        core.runtime_transaction(|candidate| {
            candidate.execute_ordered_skill_effects(
                source_id,
                &skill.id,
                &valid_step,
                &[target_id],
                10,
                10,
                None,
                None,
            )
        })
        .unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, 73);
    }

    #[test]
    fn ordered_live_result_uses_actual_damage_after_mitigation() {
        let effects = r#"
            (id: "damage", op: Damage(amount: 7, damage_type: Physical)),
            (id: "heal", op: HealExpr(amount: (
                moment: Live,
                root: Read(Result(selector: EarlierEffect("damage"), field: DamageHpLost)),
            ))),
        "#;
        for (defense, expected) in [(0, 7_i64), (75, 4_i64)] {
            let (mut core, source_id, target_id, skill, source) = ordered_fixture(effects);
            core.units.get_mut(&target_id).unwrap().stats.defense = defense;
            let action = source.action.clone().unwrap();
            let result = core
                .execute_ordered_skill_effects(
                    source_id,
                    &skill.id,
                    &skill.steps[0],
                    &[target_id],
                    10,
                    10,
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(result.actual_damage_target_count, 1);
            assert_eq!(core.units[&target_id].stats.current_health, 80);
            let facts = &core.runtime_facts.query(&action).unwrap().facts;
            assert_eq!(facts.damage_hp_lost, expected as u64);
            assert_eq!(facts.healing_delta, expected);
        }
    }

    #[test]
    fn one_effect_evaluates_every_target_before_any_target_mutates() {
        let effects = r#"
            (id: "heal", op: HealExpr(amount: (
                moment: Live,
                root: Read(HitPoints(subject: Source, field: Missing)),
            ))),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        core.units.get_mut(&source_id).unwrap().stats.current_health = 90;
        core.units.get_mut(&target_id).unwrap().stats.current_health = 70;
        core.execute_ordered_skill_effects(
            source_id,
            &skill.id,
            &skill.steps[0],
            &[source_id, target_id],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        assert_eq!(core.units[&source_id].stats.current_health, 100);
        assert_eq!(core.units[&target_id].stats.current_health, 80);
    }

    #[test]
    fn snapshot_result_requires_a_witness_completed_before_release() {
        let effects = r#"
            (id: "damage", op: Damage(amount: 7, damage_type: True)),
            (id: "heal", op: HealExpr(amount: (
                moment: Snapshot,
                root: Read(Result(selector: EarlierEffect("damage"), field: DamageHpLost)),
            ))),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        let before = core.transaction_debug_snapshot();
        let error = core
            .execute_ordered_skill_effects(
                source_id,
                &skill.id,
                &skill.steps[0],
                &[target_id],
                10,
                10,
                None,
                None,
            )
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::NotSettled,
                ..
            })
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.units[&target_id].stats.current_health, 80);
    }

    #[test]
    fn later_effect_failure_rolls_back_the_whole_runtime_candidate() {
        let effects = r#"
            (id: "damage", op: Damage(amount: 200, damage_type: True)),
            (id: "heal", op: HealExpr(amount: (
                moment: Live,
                root: Read(Stat(subject: Target, stat: Attack)),
            ))),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        let before = core.transaction_debug_snapshot();
        let error = core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    source_id,
                    &skill.id,
                    &skill.steps[0],
                    &[target_id],
                    10,
                    10,
                    None,
                    None,
                )
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::MissingContext,
                ..
            })
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.units[&target_id].stats.current_health, 80);
        assert!(core.units[&target_id].is_active());
    }

    #[test]
    fn ordered_literal_counts_targets_once_per_delivery_and_merges_repeats() {
        let skills = ordered_skill(
            r#"(id: "d", op: Damage(amount: 1, damage_type: True)),
            (id: "h", op: Heal(amount: 1))"#,
        );
        let (mut core, actor, target) = core_with_units_and_skills(vec![], Some(skills));
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let mut total = crate::game::battle::core::types::SkillStepResult::default();
        for _ in 0..2 {
            let result = core
                .execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &skill.steps[0],
                    &[actor, target],
                    10,
                    10,
                    None,
                    None,
                )
                .unwrap();
            assert_eq!(result.resolved_target_count, 2);
            assert_eq!(result.actual_damage_target_count, 2);
            total.merge(&result);
        }
        assert_eq!(total.resolved_target_count, 4);
        let result = core
            .execute_ordered_skill_effects(
                actor,
                &skill.id,
                &skill.steps[0],
                &[],
                10,
                10,
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.resolved_target_count, 0);
    }

    #[test]
    fn ordered_mixed_counts_targets_once() {
        let (mut core, actor, target, skill, _) = ordered_fixture(
            r#"
            (id: "d", op: Damage(amount: 1, damage_type: True)),
            (id: "h", op: HealExpr(amount: (moment: Live,
                root: Read(Result(selector: EarlierEffect("d"), field: DamageHpLost))))),
        "#,
        );
        let result = core
            .execute_ordered_skill_effects(
                actor,
                &skill.id,
                &skill.steps[0],
                &[target],
                10,
                10,
                None,
                None,
            )
            .unwrap();
        assert_eq!(result.resolved_target_count, 1);
        assert_eq!(core.units[&target].stats.current_health, 80);
    }

    #[test]
    fn action_prefix_sums_completed_effects_across_steps_and_deliveries() {
        let skills = SkillDatabase::from_ron("fixture/action_prefix.ron", r#"SkillDatabase(skills: [SkillDef(
            id: "wp02_runtime_frame", name: "cross", kind: Targeted, focus_time_ms: 0,
            use_lifetime: Action,
            cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
            steps: [
                SkillStepDef(id: "first", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 7, damage_type: True))]),
                SkillStepDef(id: "second", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 3, damage_type: True)),
                        (id: "heal", op: HealExpr(amount: (moment: Live,
                            root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))])
            ])])"#).unwrap();
        let (mut core, actor, target) = core_with_units_and_skills(vec![], Some(skills));
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let source = occurrence(&mut core, actor, entry.plan());
        let action = source.action.clone().unwrap();
        let mut finite_parent = source;
        finite_parent.allows_finite_children = true;
        let first_delivery = core.reserve_finite_source(finite_parent.clone()).unwrap();
        let second_delivery = core.reserve_finite_source(finite_parent).unwrap();
        assert_ne!(first_delivery.delivery, second_delivery.delivery);
        core.runtime_fact_source = first_delivery.clone();
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            &[target],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        let unrelated = occurrence(&mut core, actor, entry.plan());
        let unrelated_capture = core
            .capture_value_expr_entry(&entry, &unrelated, None, 10, None)
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture_with_prefix(&unrelated_capture, None, 11, None, 1)
                .unwrap_err()
                .code,
            ValueExprFaultCode::NotSettled
        );
        core.runtime_fact_source = second_delivery.clone();
        let result = core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &skill.steps[1],
                    &[target],
                    11,
                    11,
                    None,
                    None,
                )
            })
            .unwrap();
        assert_eq!(result.actual_damage_target_count, 1);
        assert_eq!(core.units[&target].stats.current_health, 80);
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .damage_hp_lost,
            10
        );
        core.runtime_facts
            .close_delivery(first_delivery.delivery.as_deref().unwrap(), 12)
            .unwrap();
        core.runtime_facts
            .close_delivery(second_delivery.delivery.as_deref().unwrap(), 12)
            .unwrap();
        assert_eq!(
            core.runtime_facts
                .query(&action)
                .unwrap()
                .facts
                .healing_delta,
            10
        );
    }

    #[test]
    fn action_prefix_later_fault_rolls_back_current_candidate_only() {
        let skills = SkillDatabase::from_ron("fixture/action_prefix_rollback.ron", r#"SkillDatabase(skills: [SkillDef(
            id: "wp02_runtime_frame", name: "cross", kind: Targeted, focus_time_ms: 0,
            use_lifetime: Action,
            cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
            steps: [
                SkillStepDef(id: "first", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 7, damage_type: True))]),
                SkillStepDef(id: "second", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 3, damage_type: True)),
                        (id: "heal", op: HealExpr(amount: (moment: Live,
                            root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost))))),
                        (id: "missing_stat", op: DamageExpr(amount: (moment: Live,
                            root: Mul(value: Const((value: 1, unit: Magnitude)), scalar: Ratio(numerator: Const((value: 1, unit: Magnitude)), denominator: Read(Stat(subject: Source, stat: SpellPower)), rounding: TowardZero))), damage_type: True))])
            ])])"#).unwrap();
        let (mut core, actor, target) = core_with_units_and_skills(vec![], Some(skills));
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let source = occurrence(&mut core, actor, entry.plan());
        let action = source.action.clone().unwrap();
        core.runtime_fact_source = source;
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            &[target],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        let before = core.transaction_debug_snapshot();
        let error = core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &skill.steps[1],
                    &[target],
                    11,
                    11,
                    None,
                    None,
                )
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::DivideByZero,
                ..
            })
        ));
        assert_eq!(core.transaction_debug_snapshot(), before);
        assert_eq!(core.units[&target].stats.current_health, 73);
        let facts = &core.runtime_facts.query(&action).unwrap().facts;
        assert_eq!(facts.damage_hp_lost, 7);
        assert_eq!(facts.healing_delta, 0);
        let release = core
            .build_value_expr_delivery_capture(
                &skill.steps[1],
                &core.runtime_fact_source,
                Some(&[]),
                12,
                None,
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture_with_prefix(
                release.capture_for(1, None).unwrap(),
                None,
                12,
                None,
                1
            )
            .unwrap()
            .amount,
            RuntimeAmount::Heal(7)
        );
    }

    #[test]
    fn release_result_snapshot_keeps_earlier_frontier_while_live_advances() {
        let skills = SkillDatabase::from_ron("fixture/release_result.ron", r#"SkillDatabase(skills: [SkillDef(
            id: "wp02_runtime_frame", name: "release", kind: Targeted, focus_time_ms: 0,
            use_lifetime: Action,
            cast_targeting: Explicit(selection: automatic_unit, target: EnemySingle(rule: Nearest), reach: global, air_capable: false),
            steps: [
                SkillStepDef(id: "first", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 7, damage_type: True))]),
                SkillStepDef(id: "deferred", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "snapshot", op: HealExpr(amount: (moment: Snapshot,
                        root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost))))),
                    (id: "live", op: HealExpr(amount: (moment: Live,
                        root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost))))) ]),
                SkillStepDef(id: "third", delay_ms: 0, target: EnemySingle(rule: Nearest), delivery: Instant,
                    effects: [(id: "damage", op: Damage(amount: 3, damage_type: True))])
            ])])"#).unwrap();
        let (mut core, actor, target) = core_with_units_and_skills(vec![], Some(skills));
        core.units.get_mut(&target).unwrap().stats.max_health = 100;
        let skill = core
            .game_data
            .skill_data
            .get_by_id("wp02_runtime_frame")
            .unwrap()
            .clone();
        let entry = core
            .game_data
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.is_authored_expression())
            .unwrap()
            .1
            .clone();
        let source = occurrence(&mut core, actor, entry.plan());
        core.runtime_fact_source = source.clone();
        let missing = core
            .build_value_expr_delivery_capture(&skill.steps[1], &source, Some(&[]), 9, None)
            .unwrap_err();
        assert!(matches!(
            missing,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::NotSettled,
                ..
            })
        ));
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[0],
            &[target],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        let mut finite_source = source.clone();
        finite_source.allows_finite_children = true;
        let finite_source = core.reserve_finite_source(finite_source).unwrap();
        assert!(finite_source.delivery.is_some());
        let release = core
            .build_value_expr_delivery_capture(&skill.steps[1], &finite_source, Some(&[]), 11, None)
            .unwrap()
            .unwrap();
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[2],
            &[target],
            12,
            12,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            core.evaluate_value_expr_capture_with_prefix(
                release.capture_for(0, None).unwrap(),
                None,
                13,
                None,
                0
            )
            .unwrap()
            .amount,
            RuntimeAmount::Heal(7)
        );
        assert_eq!(
            core.evaluate_value_expr_capture_with_prefix(
                release.capture_for(1, None).unwrap(),
                None,
                13,
                None,
                1
            )
            .unwrap()
            .amount,
            RuntimeAmount::Heal(10)
        );
        let mut template = core
            .damage_source_snapshot_for_unit(
                actor,
                target,
                crate::game::battle::damage::DamageSource::Ability,
                crate::game::battle::damage::DamageType::True,
                0,
                Default::default(),
                0,
                11,
                false,
            )
            .unwrap();
        template.fact_source = finite_source.clone();
        template.value_expr_capture = Some(Arc::clone(&release));
        core.runtime_fact_source = finite_source.clone();
        core.execute_ordered_skill_effects(
            actor,
            &skill.id,
            &skill.steps[1],
            &[target],
            13,
            11,
            Some(&template),
            None,
        )
        .unwrap();
        assert_eq!(core.units[&target].stats.current_health, 87);
        core.runtime_facts
            .close_delivery(finite_source.delivery.as_deref().unwrap(), 13)
            .unwrap();
    }

    #[test]
    fn action_result_resource_projection_preserves_typed_zero_and_sum() {
        let id = ResourceId::new("meter".to_string()).unwrap();
        let plan = plan(
            EvalMoment::Live,
            "resource_delta",
            ValueExpr::Read(ValueRef::Result {
                selector: ResultSelector::CurrentActionPrefix,
                field: ResultField::ResourceChangeDelta(id.clone()),
            }),
        );
        let mut witness = ActionResultWitness::new(
            plan.source().site.action.definition_id.clone(),
            Arc::new(vec![id.clone()]),
        );
        let zero = EffectResultProjection::zero(1);
        witness.settle(&plan.source().site.action, &zero).unwrap();
        assert_eq!(
            witness
                .value(&plan, &ResultField::ResourceChangeDelta(id.clone()), 0)
                .unwrap(),
            TypedInteger::new(0, ValueUnit::Resource(id.clone()))
        );
        let mut next = EffectResultProjection::zero(1);
        next.resource_deltas[0] = 5;
        witness.settle(&plan.source().site.action, &next).unwrap();
        assert_eq!(
            witness
                .value(&plan, &ResultField::ResourceChangeDelta(id.clone()), 0)
                .unwrap(),
            TypedInteger::new(5, ValueUnit::Resource(id))
        );
    }

    #[test]
    fn action_result_witness_requires_retained_admission_before_install() {
        use crate::game::battle::runtime_control as native;
        let (mut core, _, _, skill, source) = ordered_fixture(
            r#"(id: "d", op: Damage(amount: 1, damage_type: True)),
                (id: "h", op: HealExpr(amount: (moment: Live,
                    root: Read(Result(selector: CurrentActionPrefix, field: DamageHpLost)))))"#,
        );
        let action = source.action.as_ref().unwrap().clone();
        let basis = core.retained_runtime_fact_extent().unwrap();
        let required = ActionResultWitness::new(skill.id.clone(), Arc::new(Vec::new()))
            .retained_extent()
            .unwrap();
        let tight = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: basis + required - 1,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(basis)?;
                core.ensure_value_expr_action_witness(&skill.id, &source)
            },
        );
        assert!(matches!(tight, Err(GameError::BattleRuntimeFault(_))));
        assert!(core
            .runtime_facts
            .action_result_witness(&action)
            .unwrap()
            .is_none());
        native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: basis + required,
                ..Default::default()
            },
            || {
                native::seed_retained_extent(basis)?;
                core.ensure_value_expr_action_witness(&skill.id, &source)
            },
        )
        .unwrap();
        assert!(core
            .runtime_facts
            .action_result_witness(&action)
            .unwrap()
            .is_some());
    }

    #[test]
    fn prefix_rejects_missing_unsettled_future_self_and_other_occurrence() {
        let result_plan = plan(
            EvalMoment::Live,
            "current",
            ValueExpr::Read(ValueRef::Result {
                selector: ResultSelector::EarlierEffect("first".to_string()),
                field: ResultField::DamageHpLost,
            }),
        );
        let (mut core, source_id, _) = core_with_units(Vec::new());
        let source = occurrence(&mut core, source_id, &result_plan);
        let action = source.action.clone().unwrap();
        let zero = EffectResultProjection::zero(0);
        let mut prefix = EffectResultPrefix {
            action,
            delivery: None,
            action_site: result_plan.source().site.action.clone(),
            effect_ids: Arc::new(vec![
                "first".to_string(),
                "current".to_string(),
                "future".to_string(),
            ]),
            resource_ids: Arc::new(Vec::new()),
            settled: vec![Some(zero.clone()), None, None],
            cumulative: zero,
        };
        let valid = core
            .build_value_expr_frame_with_prefix(
                &result_plan,
                &source,
                None,
                10,
                None,
                Some(&prefix),
                1,
            )
            .unwrap();
        assert_eq!(raw(&result_plan, &valid).value, 0);

        prefix.settled[0] = None;
        assert_eq!(
            core.build_value_expr_frame_with_prefix(
                &result_plan,
                &source,
                None,
                10,
                None,
                Some(&prefix),
                1,
            )
            .unwrap_err()
            .code,
            ValueExprFaultCode::NotSettled
        );
        prefix.settled[0] = Some(EffectResultProjection::zero(0));

        for (selector, expected) in [
            (
                ResultSelector::EarlierEffect("missing".to_string()),
                ValueExprFaultCode::Unavailable,
            ),
            (
                ResultSelector::EarlierEffect("current".to_string()),
                ValueExprFaultCode::FutureResult,
            ),
            (
                ResultSelector::EarlierEffect("future".to_string()),
                ValueExprFaultCode::FutureResult,
            ),
        ] {
            let candidate = plan(
                EvalMoment::Live,
                "current",
                ValueExpr::Read(ValueRef::Result {
                    selector,
                    field: ResultField::DamageHpLost,
                }),
            );
            let candidate_source = RuntimeFactSource {
                authored: Some(Arc::new(candidate.source().clone())),
                ..source.clone()
            };
            assert_eq!(
                core.build_value_expr_frame_with_prefix(
                    &candidate,
                    &candidate_source,
                    None,
                    10,
                    None,
                    Some(&prefix),
                    1,
                )
                .unwrap_err()
                .code,
                expected
            );
        }

        let other = occurrence(&mut core, source_id, &result_plan);
        assert_eq!(
            core.build_value_expr_frame_with_prefix(
                &result_plan,
                &other,
                None,
                10,
                None,
                Some(&prefix),
                1,
            )
            .unwrap_err()
            .code,
            ValueExprFaultCode::DifferentAttempt
        );
    }

    #[test]
    fn d07_b01_b04_legacy_consumer_and_recorded_replay_share_one_numerical_kernel() {
        let effects = r#"
            (id: "legacy", op: Damage(amount: 7, damage_type: True)),
            (id: "dynamic", op: DamageExpr(amount: (moment: Live,
                root: Const((value: 11, unit: Magnitude))), damage_type: True)),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(super::super::authoring_observation::ObservationKey {
            entry_ordinal: 1,
            battle_uuid: Uuid::from_u128(0xd07b01),
        })
        .unwrap();

        let mut legacy = skill.steps[0].clone();
        legacy.effects.truncate(1);
        let SkillEffectDef::Damage { amount, .. } = &mut legacy.effects[0].op else {
            unreachable!()
        };
        *amount = -7;
        let before_hp = core.units[&target_id].stats.current_health;
        core.execute_ordered_skill_effects(
            source_id,
            &skill.id,
            &legacy,
            &[target_id],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, before_hp - 7);
        let first = Arc::clone(
            core.authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap(),
        );
        assert_eq!(first.evaluation_ordinal, 1);
        assert_eq!(first.amount, RuntimeAmount::Damage(7));
        assert_eq!(first.evaluated.value.value, 7);
        assert_eq!(
            first.plan.provenance(),
            AmountProvenance::LegacyDamageLiteral
        );
        let replay = evaluate_compiled_amount(&first.plan, first.kind(), &first.frame).unwrap();
        assert_eq!(replay.amount, first.amount);
        assert_eq!(replay.evaluated, first.evaluated);

        core.execute_ordered_skill_effects(
            source_id,
            &skill.id,
            &skill.steps[0],
            &[target_id],
            11,
            11,
            None,
            None,
        )
        .unwrap();
        assert_eq!(core.units[&target_id].stats.current_health, before_hp - 25);
        let latest = core
            .authoring_observation
            .as_ref()
            .unwrap()
            .latest_amount
            .as_ref()
            .unwrap();
        assert_eq!(latest.evaluation_ordinal, 3);
        assert_eq!(latest.amount, RuntimeAmount::Damage(11));
        assert_eq!(
            latest.plan.provenance(),
            AmountProvenance::AuthoredExpression
        );
        assert_eq!(latest.release_target, Some(target_id));
        assert_eq!(latest.consumption_target, Some(target_id));
        assert_eq!(first.evaluation_ordinal, 1);

        let query = crate::authoring::AmountQuery::Exact {
            battle_key: crate::authoring::BattleKey::from(latest.battle_key),
            evaluation_ordinal: latest.evaluation_ordinal,
        };
        let (documents, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::Inspection,
            || {
                Ok::<_, crate::authoring::AuthoringError>((
                    crate::authoring::encode_record_for_test(latest, query, false)?,
                    crate::authoring::encode_record_for_test(latest, query, true)?,
                ))
            },
        )
        .unwrap();
        let (preview, explanation) = documents.unwrap();
        let preview_json: serde_json::Value = serde_json::from_str(&preview).unwrap();
        let explanation_json: serde_json::Value = serde_json::from_str(&explanation).unwrap();
        assert_eq!(preview_json["moment"], "Live");
        assert_eq!(preview_json["basis"], "recorded_evaluation");
        assert_eq!(
            preview_json["typed_base_amount"],
            serde_json::to_value(&latest.evaluated.value).unwrap()
        );
        assert_eq!(
            preview_json["trace"],
            serde_json::to_value(&latest.evaluated.trace).unwrap()
        );
        assert_eq!(
            preview_json["typed_base_amount"],
            explanation_json["typed_base_amount"]
        );
        assert_eq!(preview_json["trace"], explanation_json["trace"]);
        assert_eq!(
            explanation_json["static_plan"]["evaluation"]["performed"],
            false
        );
        eprintln!("D07_B_LIVE_PREVIEW={preview}");
        eprintln!("D07_B_LIVE_EXPLANATION={explanation}");
    }

    #[test]
    fn d07_b06_b09_multi_target_order_and_zero_evaluation_replace_one_record() {
        let effects = r#"
            (id: "zero", op: Damage(amount: 0, damage_type: True)),
            (id: "dynamic", op: DamageExpr(amount: (moment: Live,
                root: Const((value: 1, unit: Magnitude))), damage_type: True)),
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(super::super::authoring_observation::ObservationKey {
            entry_ordinal: 4,
            battle_uuid: Uuid::from_u128(0xd07b06),
        })
        .unwrap();
        let mut zero = skill.steps[0].clone();
        zero.effects.truncate(1);
        let before = [
            core.units[&source_id].stats.current_health,
            core.units[&target_id].stats.current_health,
        ];
        core.execute_ordered_skill_effects(
            source_id,
            &skill.id,
            &zero,
            &[source_id, target_id],
            10,
            10,
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            before,
            [
                core.units[&source_id].stats.current_health,
                core.units[&target_id].stats.current_health,
            ]
        );
        let observer = core.authoring_observation.as_ref().unwrap();
        assert_eq!(observer.amount_evaluations, 2);
        let latest = observer.latest_amount.as_ref().unwrap();
        assert_eq!(latest.evaluation_ordinal, 2);
        assert_eq!(latest.consumption_target, Some(target_id));
        assert_eq!(latest.amount, RuntimeAmount::Damage(0));
    }

    #[test]
    fn d07_b10_failed_later_effect_rolls_back_record_and_ordinal_with_gameplay() {
        let effects = r#"
            (id: "first", op: Damage(amount: 7, damage_type: True)),
            (id: "bad", op: HealExpr(amount: (moment: Live,
                root: Aggregate(source: ResolvedTargets,
                    grouping: ByOwner,
                    reduction: Sum, outer_reduction: Some(Sum),
                    member: Read(Stat(subject: AggregateMember, stat: Attack))))))
        "#;
        let (mut core, source_id, target_id, skill, _) = ordered_fixture(effects);
        core.enable_authoring_observation_with_options(Default::default(), true);
        core.bind_authoring_observation(super::super::authoring_observation::ObservationKey {
            entry_ordinal: 8,
            battle_uuid: Uuid::from_u128(0xd07b10),
        })
        .unwrap();
        let before_hp = core.units[&target_id].stats.current_health;
        let error = core
            .runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    source_id,
                    &skill.id,
                    &skill.steps[0],
                    &[target_id],
                    10,
                    10,
                    None,
                    None,
                )
            })
            .unwrap_err();
        assert!(matches!(
            error,
            GameError::ValueExpressionFault(ValueExprFault {
                code: ValueExprFaultCode::InvalidOwner,
                ..
            })
        ));
        assert_eq!(core.units[&target_id].stats.current_health, before_hp);
        let observer = core.authoring_observation.as_ref().unwrap();
        assert_eq!(observer.amount_evaluations, 0);
        assert!(observer.latest_amount.is_none());
    }

    #[test]
    fn d07_f02_c02_amount_buffer_admits_exact_bytes_and_rejects_short_or_overflow() {
        use crate::game::battle::runtime_control::{
            self as native, RuntimeFactLimits, RuntimeFaultCode,
        };
        let count = 3;
        let bytes = count * size_of::<RuntimeAmount>();
        let (result, admissions) = native::observe_retained_admissions(|| {
            native::run(
                RuntimeFactLimits {
                    max_retained_bytes: bytes,
                    ..Default::default()
                },
                || super::super::sim::allocate_evaluated_amounts(count),
            )
        });
        let values = result.unwrap();
        assert!(values.is_empty());
        assert_eq!(values.capacity(), count);
        assert_eq!(admissions, [("value_expr_evaluated_amounts", 0, bytes)]);
        for (count, cap, requested) in [
            (count, bytes - 1, bytes as u64),
            (usize::MAX / size_of::<RuntimeAmount>() + 1, bytes, u64::MAX),
        ] {
            let error = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || super::super::sim::allocate_evaluated_amounts(count),
            )
            .unwrap_err();
            let GameError::BattleRuntimeFault(fault) = error else {
                panic!("{error:?}")
            };
            assert_eq!(fault.code, RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.site, "value_expr_evaluated_amounts");
            assert_eq!(fault.limit, Some(cap as u64));
            assert_eq!(fault.requested, Some(requested));
        }
    }

    #[test]
    fn d07_f02_c02_legacy_multitarget_admission_preserves_accepted_record_on_refusal() {
        use crate::game::battle::runtime_control::{
            self as native, RuntimeFactLimits, RuntimeFaultCode,
        };
        for legacy in ["Damage(amount: 7, damage_type: True)", "Heal(amount: 3)"] {
            let effects = format!(
                r#"
                (id: "legacy", op: {legacy}),
                (id: "dynamic", op: DamageExpr(amount: (moment: Live,
                    root: Const((value: 1, unit: Magnitude))), damage_type: True)),
            "#
            );
            let (mut core, actor, target, skill, _) = ordered_fixture(&effects);
            core.enable_authoring_observation_with_options(Default::default(), true);
            core.bind_authoring_observation(super::super::authoring_observation::ObservationKey {
                entry_ordinal: 12,
                battle_uuid: Uuid::from_u128(0xf02),
            })
            .unwrap();
            core.units.get_mut(&actor).unwrap().stats.current_health = 50;
            core.units.get_mut(&target).unwrap().stats.current_health = 50;
            let mut step = skill.steps[0].clone();
            step.effects.truncate(1);
            core.runtime_transaction(|candidate| {
                candidate.execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &step,
                    &[target],
                    10,
                    10,
                    None,
                    None,
                )
            })
            .unwrap();
            let old_record = Arc::clone(
                core.authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_amount
                    .as_ref()
                    .unwrap(),
            );
            let old_count = core
                .authoring_observation
                .as_ref()
                .unwrap()
                .amount_evaluations;
            assert_eq!(old_count, 1);
            let before = core.transaction_debug_snapshot();
            let queue_before = core.event_queue_debug_snapshot();
            let facts_before = format!("{:?}", core.runtime_facts);
            let execute = |candidate: &mut BattleCore| {
                candidate.execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &step,
                    &[actor, target],
                    11,
                    11,
                    None,
                    None,
                )
            };
            // Measure a complete successful operation, including transaction setup.
            let mut measured = core.clone();
            let (result, admissions) =
                native::observe_retained_admissions(|| measured.runtime_transaction(execute));
            result.unwrap();
            let position = admissions
                .iter()
                .position(|(site, _, _)| *site == "value_expr_evaluated_amounts")
                .expect("actual legacy executor must use the admitted amount buffer");
            let array_required = admissions[position].2;
            let previous_required = admissions[position].1;
            assert_eq!(
                array_required - previous_required,
                2 * size_of::<RuntimeAmount>()
            );
            let full_required = admissions.last().unwrap().2;
            assert!(full_required > array_required);

            let error = native::run(
                RuntimeFactLimits {
                    max_retained_bytes: array_required - 1,
                    ..Default::default()
                },
                || core.runtime_transaction(execute),
            )
            .unwrap_err();
            let GameError::BattleRuntimeFault(fault) = error else {
                panic!("{error:?}")
            };
            assert_eq!(fault.code, RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.site, "value_expr_evaluated_amounts");
            assert_eq!(fault.requested, Some(array_required as u64));
            assert_eq!(fault.limit, Some((array_required - 1) as u64));
            assert_eq!(core.transaction_debug_snapshot(), before);
            assert_eq!(core.event_queue_debug_snapshot(), queue_before);
            assert_eq!(format!("{:?}", core.runtime_facts), facts_before);
            let observer = core.authoring_observation.as_ref().unwrap();
            assert_eq!(observer.amount_evaluations, old_count);
            assert!(Arc::ptr_eq(
                observer.latest_amount.as_ref().unwrap(),
                &old_record
            ));
            let replay =
                evaluate_compiled_amount(&old_record.plan, old_record.kind(), &old_record.frame)
                    .unwrap();
            assert_eq!(replay.evaluated, old_record.evaluated);
            assert_eq!(replay.amount, old_record.amount);

            // Exactly enough for this buffer admits it; later allocations still need fuel.
            let (result, exact_trace) = native::observe_retained_admissions(|| {
                native::run(
                    RuntimeFactLimits {
                        max_retained_bytes: array_required,
                        ..Default::default()
                    },
                    || core.runtime_transaction(execute),
                )
            });
            assert!(result.is_err());
            assert!(exact_trace.contains(&(
                "value_expr_evaluated_amounts",
                previous_required,
                array_required
            )));
            assert_eq!(core.transaction_debug_snapshot(), before);
            assert!(Arc::ptr_eq(
                core.authoring_observation
                    .as_ref()
                    .unwrap()
                    .latest_amount
                    .as_ref()
                    .unwrap(),
                &old_record
            ));
            assert_eq!(
                core.authoring_observation
                    .as_ref()
                    .unwrap()
                    .amount_evaluations,
                old_count
            );

            // A fresh operation with the measured full inclusive allowance succeeds.
            native::run(
                RuntimeFactLimits {
                    max_retained_bytes: full_required,
                    ..Default::default()
                },
                || core.runtime_transaction(execute),
            )
            .unwrap();
            assert_eq!(
                core.transaction_debug_snapshot(),
                measured.transaction_debug_snapshot()
            );
            assert_eq!(
                core.authoring_observation
                    .as_ref()
                    .unwrap()
                    .amount_evaluations,
                old_count + 2
            );
            let latest = core
                .authoring_observation
                .as_ref()
                .unwrap()
                .latest_amount
                .as_ref()
                .unwrap();
            assert_eq!(latest.evaluation_ordinal, old_count + 2);
            assert_eq!(latest.consumption_target, Some(target));
            assert!(!Arc::ptr_eq(latest, &old_record));
            eprintln!("F02_C02 legacy={legacy} targets=2 array_bytes={} array_cumulative={array_required} full_cumulative={full_required} refusal_site={}", 2 * size_of::<RuntimeAmount>(), fault.site);
        }
    }

    #[test]
    fn d07_f02_c02_buffer_shared_with_expression_and_skipped_for_unobserved_or_uncatalogued_legacy()
    {
        use crate::game::battle::runtime_control as native;
        let effects = r#"
            (id: "legacy", op: Damage(amount: 7, damage_type: True)),
            (id: "dynamic", op: DamageExpr(amount: (moment: Live,
                root: Const((value: 1, unit: Magnitude))), damage_type: True)),
        "#;
        let (core, actor, target, skill, _) = ordered_fixture(effects);
        let mut step = skill.steps[0].clone();
        step.effects.truncate(1);
        let mut expected = None;
        let mut observed_gameplay = None;
        for (observe, catalogued) in [(true, true), (false, true), (true, false)] {
            let mut candidate = core.clone();
            candidate.enable_authoring_observation_with_options(Default::default(), observe);
            candidate
                .bind_authoring_observation(super::super::authoring_observation::ObservationKey {
                    entry_ordinal: 13,
                    battle_uuid: Uuid::from_u128(0xf0203),
                })
                .unwrap();
            let mut current = step.clone();
            if !catalogued {
                current.effects[0].source = None;
            }
            let hp_before = [
                candidate.units[&actor].stats.current_health,
                candidate.units[&target].stats.current_health,
            ];
            let (result, admissions) = native::observe_retained_admissions(|| {
                candidate.runtime_transaction(|inner| {
                    inner.execute_ordered_skill_effects(
                        actor,
                        &skill.id,
                        &current,
                        &[actor, target],
                        10,
                        10,
                        None,
                        None,
                    )
                })
            });
            result.unwrap();
            let hp_after = [
                candidate.units[&actor].stats.current_health,
                candidate.units[&target].stats.current_health,
            ];
            assert_eq!(hp_after, [hp_before[0] - 7, hp_before[1] - 7]);
            assert_eq!(expected.get_or_insert(hp_after), &hp_after);
            if catalogued {
                let gameplay = (
                    candidate.transaction_debug_snapshot(),
                    candidate.event_queue_debug_snapshot(),
                    format!("{:?}", candidate.runtime_facts),
                );
                assert_eq!(observed_gameplay.get_or_insert(gameplay.clone()), &gameplay);
            }
            assert_eq!(
                admissions
                    .iter()
                    .filter(|(site, _, _)| *site == "value_expr_evaluated_amounts")
                    .count(),
                usize::from(observe && catalogued)
            );
            assert_eq!(
                candidate
                    .authoring_observation
                    .as_ref()
                    .unwrap()
                    .amount_evaluations,
                if observe && catalogued { 2 } else { 0 }
            );
        }
        let mut candidate = core;
        let mut expression = skill.steps[0].clone();
        expression.effects.remove(0);
        let (result, admissions) = native::observe_retained_admissions(|| {
            candidate.runtime_transaction(|inner| {
                inner.execute_ordered_skill_effects(
                    actor,
                    &skill.id,
                    &expression,
                    &[actor, target],
                    10,
                    10,
                    None,
                    None,
                )
            })
        });
        result.unwrap();
        assert_eq!(
            admissions
                .iter()
                .filter(|(site, _, _)| *site == "value_expr_evaluated_amounts")
                .count(),
            1
        );
    }
    #[test]
    fn r07_f08_fragment_lease_rejects_forgery_and_old_snapshot_survives_withdrawal() {
        for lifetime in [ResourceLifetime::Attempt, ResourceLifetime::Deployment] {
            let id = ResourceId::new("copy_meter".into()).unwrap();
            let (mut core, actor, _) = core_with_units(vec![ResourceDefinition {
                id: id.clone(),
                owner: ResourceOwnerKind::FragmentCopy,
                lifetime,
                initial: 7,
                cap: 30,
            }]);
            let copy = Uuid::from_u128(0x707);
            let fragment = Arc::new(crate::game::skill_fragment::FragmentSourceSnapshot {
                copy_uuid: copy,
                definition_id: crate::game::data::skill_fragment_data::SkillFragmentId::from(
                    "fragment",
                ),
                form_id: "base".into(),
                skill_id: SkillId::from("fragment_skill"),
            });
            core.units.get_mut(&actor).unwrap().fragment_source = Some(fragment.clone());
            core.runtime_facts
                .resources
                .as_mut()
                .unwrap()
                .store
                .reconcile_owners(&BTreeSet::from([
                    ResourceOwner::Employee(Uuid::from_u128(0x2201)),
                    ResourceOwner::Employee(Uuid::from_u128(0x2203)),
                    ResourceOwner::FragmentCopy(copy),
                ]))
                .unwrap();
            let expr = plan(
                EvalMoment::Snapshot,
                "fragment_read",
                ValueExpr::Read(ValueRef::Resource {
                    resource: id.clone(),
                    owner: ResourceOwnerSelector::SourceFragment,
                    field: ResourceValueField::Balance,
                }),
            );
            let basis = RuntimeFactSource {
                actor: Some(actor),
                fragment: Some(fragment.clone()),
                authored: Some(Arc::new(expr.source().clone())),
                ..Default::default()
            };
            let action = core
                .runtime_facts
                .accept(actor, None, basis.clone(), 1, false)
                .unwrap();
            let mut source = RuntimeFactSource {
                action: Some(action.clone()),
                ..basis
            };
            source.delivery = core.runtime_facts.delivery(&action).unwrap();
            core.runtime_facts.release(&action, 0, 9).unwrap();
            let frame = core
                .build_value_expr_frame(&expr, &source, None, 10, None)
                .unwrap();
            let before = core.transaction_debug_snapshot();
            let mut forged = source.clone();
            forged.fragment = Some(Arc::new((*fragment).clone()));
            assert!(core
                .resolve_typed_resource_lease(
                    &id,
                    ResourceOwnerSelector::SourceFragment,
                    &forged,
                    None
                )
                .is_err());
            assert!(core
                .build_value_expr_frame(&expr, &forged, None, 10, None)
                .is_err());
            assert_eq!(core.transaction_debug_snapshot(), before);
            core.withdraw_unit(actor, 11).unwrap();
            assert_eq!(
                core.resolve_typed_resource_lease(
                    &id,
                    ResourceOwnerSelector::SourceFragment,
                    &source,
                    None
                )
                .is_ok(),
                lifetime == ResourceLifetime::Attempt
            );
            assert_eq!(raw(&expr, &frame).value, 7);
            assert_eq!(source.fragment.as_ref().unwrap().copy_uuid, copy);
            assert_eq!(source.fragment.as_ref().unwrap().form_id, "base");
            let live = plan(
                EvalMoment::Live,
                "fragment_live",
                ValueExpr::Read(ValueRef::Resource {
                    resource: id,
                    owner: ResourceOwnerSelector::SourceFragment,
                    field: ResourceValueField::Balance,
                }),
            );
            source.authored = Some(Arc::new(live.source().clone()));
            assert!(core
                .build_value_expr_frame(&live, &source, None, 12, None)
                .is_err());
        }
    }
}
