//! One immutable, replayable amount evaluation per observed battle.
//!
//! The retained plan, frame and runtime source stay crate-private. Public
//! authoring queries receive projections only and cannot recover capabilities.

use std::{mem::size_of, sync::Arc};

use crate::{
    authoring::control,
    game::{
        ability::authoring::EffectSite,
        battle::{ids::UnitInstanceId, runtime_facts::RuntimeFactSource},
        behavior::GameError,
        data::content_pack::ContentIdentity,
        value_expr::{
            typed_integer_dynamic_bytes, AmountPlanKind, AmountProvenance, CompiledAmountPlan,
            EvalFrame, EvalMoment, EvaluatedAmount, ExprUsage,
        },
    },
};

use super::{
    authoring_observation::ObservationKey,
    value_expr_runtime::{RuntimeAmount, RuntimeAmountEvaluation, RuntimeAmountObservation},
    BattleCore,
};

#[derive(Debug)]
pub(crate) struct PendingAmountEvaluation {
    pub(crate) generation: Arc<ContentIdentity>,
    pub(crate) fingerprint: [u8; 32],
    pub(crate) kind: AmountPlanKind,
    pub(crate) provenance: AmountProvenance,
    pub(crate) moment: EvalMoment,
    pub(crate) source: RuntimeFactSource,
    pub(crate) release_target: Option<UnitInstanceId>,
    pub(crate) consumption_target: Option<UnitInstanceId>,
    pub(crate) release_time_ms: u64,
    pub(crate) consume_time_ms: u64,
    pub(crate) effect_index: usize,
    pub(crate) plan: Arc<CompiledAmountPlan>,
    pub(crate) frame: Arc<EvalFrame>,
    pub(crate) evaluated: EvaluatedAmount,
    pub(crate) amount: RuntimeAmount,
    pub(crate) runtime_usage: ExprUsage,
    evaluation_advance: u64,
    retained_extent: usize,
}

impl PendingAmountEvaluation {
    pub(crate) fn from_runtime(
        evaluation: RuntimeAmountEvaluation,
    ) -> Result<Option<Self>, GameError> {
        let RuntimeAmountEvaluation {
            amount,
            evaluated,
            usage,
            observation,
        } = evaluation;
        let Some(RuntimeAmountObservation {
            generation,
            source,
            release_target,
            consumption_target,
            release_time_ms,
            consume_time_ms,
            effect_index,
            plan,
            frame,
        }) = observation
        else {
            return Ok(None);
        };
        let retained_extent = retained_extent(&generation, &source, &plan, &frame, &evaluated)
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "recorded amount retained extent overflowed".to_string(),
                )
            })?;
        let kind = match amount {
            RuntimeAmount::Damage(_) => AmountPlanKind::Damage,
            RuntimeAmount::Heal(_) => AmountPlanKind::Heal,
            RuntimeAmount::Shield(_) => AmountPlanKind::Shield,
        };
        Ok(Some(Self {
            generation,
            fingerprint: plan.semantic_fingerprint(),
            kind,
            provenance: plan.provenance(),
            moment: plan.moment(),
            source,
            release_target,
            consumption_target,
            release_time_ms,
            consume_time_ms,
            effect_index,
            plan,
            frame,
            evaluated,
            amount,
            runtime_usage: usage,
            evaluation_advance: 1,
            retained_extent,
        }))
    }

    pub(crate) fn set_evaluation_advance(&mut self, count: usize) -> Result<(), GameError> {
        self.evaluation_advance = u64::try_from(count).map_err(|_| {
            GameError::InvalidStaticData("recorded amount evaluation count overflowed".into())
        })?;
        if self.evaluation_advance == 0 {
            return Err(GameError::InvalidStaticData(
                "recorded amount evaluation count is zero".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn retained_extent(&self) -> usize {
        self.retained_extent
    }
}

#[derive(Debug)]
pub(crate) struct RecordedAmountEvaluation {
    pub(crate) battle_key: ObservationKey,
    pub(crate) evaluation_ordinal: u64,
    pub(crate) generation: Arc<ContentIdentity>,
    pub(crate) site: EffectSite,
    pub(crate) fingerprint: [u8; 32],
    pub(crate) kind: AmountPlanKind,
    pub(crate) provenance: AmountProvenance,
    pub(crate) moment: EvalMoment,
    pub(crate) source: Arc<RuntimeFactSource>,
    bound_source: Arc<RuntimeFactSource>,
    pub(crate) release_target: Option<UnitInstanceId>,
    pub(crate) consumption_target: Option<UnitInstanceId>,
    pub(crate) release_time_ms: u64,
    pub(crate) consume_time_ms: u64,
    pub(crate) effect_index: usize,
    pub(crate) plan: Arc<CompiledAmountPlan>,
    pub(crate) frame: Arc<EvalFrame>,
    pub(crate) evaluated: EvaluatedAmount,
    pub(crate) amount: RuntimeAmount,
    pub(crate) runtime_usage: ExprUsage,
    retained_extent: usize,
}

impl RecordedAmountEvaluation {
    pub(crate) fn source_binding_matches(&self) -> bool {
        self.source == self.bound_source
            && self
                .source
                .delivery
                .as_ref()
                .is_none_or(|delivery| self.source.action.as_ref() == Some(&delivery.action))
    }
    pub(crate) fn site(&self) -> &EffectSite {
        &self.site
    }
    pub(crate) fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub(crate) fn kind(&self) -> AmountPlanKind {
        self.kind
    }
    pub(crate) fn provenance(&self) -> AmountProvenance {
        self.provenance
    }
    pub(crate) fn moment(&self) -> EvalMoment {
        self.moment
    }
    pub(crate) fn retained_extent(&self) -> usize {
        self.retained_extent
    }
}

impl BattleCore {
    pub(crate) fn amount_observation_enabled(&self) -> bool {
        self.authoring_observation
            .as_ref()
            .is_some_and(|owner| owner.amount_observation_enabled)
    }

    pub(super) fn commit_amount_evaluation(
        &mut self,
        pending: Option<PendingAmountEvaluation>,
    ) -> Result<(), GameError> {
        let Some(pending) = pending else {
            return Ok(());
        };
        let state = self.authoring_observation.as_mut().ok_or_else(|| {
            GameError::InvalidStaticData("recorded amount observation owner is absent".into())
        })?;
        if !state.amount_observation_enabled || state.failed() {
            return Err(GameError::InvalidStaticData(
                "recorded amount observation owner is invalid".into(),
            ));
        }
        let battle_key = state.key.ok_or_else(|| {
            GameError::InvalidStaticData("recorded amount battle key is absent".into())
        })?;
        let evaluation_ordinal = state
            .amount_evaluations
            .checked_add(pending.evaluation_advance)
            .ok_or_else(|| {
                GameError::InvalidStaticData("recorded amount ordinal overflowed".into())
            })?;
        let old_extent = state
            .latest_amount
            .as_ref()
            .map_or(0, |record| record.retained_extent());
        let basis = self
            .retained_runtime_fact_extent()?
            .checked_sub(old_extent)
            .ok_or_else(|| {
                GameError::InvalidStaticData("recorded amount basis underflowed".into())
            })?;
        let peak = basis
            .checked_add(old_extent)
            .and_then(|value| value.checked_add(pending.retained_extent()))
            .ok_or_else(|| {
                GameError::InvalidStaticData("recorded amount peak overflowed".into())
            })?;
        crate::game::battle::runtime_control::admit_retained_growth(
            basis,
            Some(peak),
            "recorded_amount_replacement_peak",
        )?;
        control::charge(1, "recorded_amount_commit")?;
        // The candidate only borrows/share-clones existing allocation owners.
        // Allocate the record's own site and source owners after peak admission.
        let site = pending.plan.source().site.clone();
        let source = Arc::new(pending.source);
        let record = Arc::new(RecordedAmountEvaluation {
            battle_key,
            evaluation_ordinal,
            generation: pending.generation,
            site,
            fingerprint: pending.fingerprint,
            kind: pending.kind,
            provenance: pending.provenance,
            moment: pending.moment,
            bound_source: Arc::clone(&source),
            source,
            release_target: pending.release_target,
            consumption_target: pending.consumption_target,
            release_time_ms: pending.release_time_ms,
            consume_time_ms: pending.consume_time_ms,
            effect_index: pending.effect_index,
            plan: pending.plan,
            frame: pending.frame,
            evaluated: pending.evaluated,
            amount: pending.amount,
            runtime_usage: pending.runtime_usage,
            retained_extent: pending.retained_extent,
        });
        let state = self
            .authoring_observation
            .as_mut()
            .expect("validated amount observation owner");
        state.amount_evaluations = evaluation_ordinal;
        state.latest_amount = Some(record);
        Ok(())
    }
}

fn retained_extent(
    generation: &ContentIdentity,
    source: &RuntimeFactSource,
    plan: &CompiledAmountPlan,
    frame: &EvalFrame,
    evaluated: &EvaluatedAmount,
) -> Option<usize> {
    let mut bytes = size_of::<RecordedAmountEvaluation>()
        .checked_add(64 * 6)?
        .checked_add(generation.semantic_sha256.len())?
        .checked_add(source.retained_extent()?)?
        .checked_add(size_of::<EffectSite>())?
        .checked_add(plan.source().site.action.definition_id.len())?
        .checked_add(plan.source().site.action.action_id.len())?
        .checked_add(plan.source().site.effect_id.len())?
        .checked_add(plan.cost().plan_retained_bytes)?
        .checked_add(frame.retained_extent()?)?
        .checked_add(
            evaluated
                .trace
                .capacity()
                .checked_mul(size_of::<crate::game::value_expr::TraceEntry>())?,
        )?
        .checked_add(typed_integer_dynamic_bytes(&evaluated.value))?;
    for entry in &evaluated.trace {
        bytes = bytes
            .checked_add(entry.member_identity.as_ref().map_or(0, String::len))?
            .checked_add(typed_integer_dynamic_bytes(&entry.value))?;
    }
    Some(bytes)
}
