//! Closed queries over the last exact amount evaluation retained by a trial.

use serde::Serialize;

use super::{
    amount, control,
    query::{control_error, request_error},
    AuthoringError, BattleKey, ExecutionPhase,
};
use crate::{
    application::CoreApplication,
    game::{
        battle::{
            core::{
                amount_observation::RecordedAmountEvaluation,
                value_expr_runtime::{evaluate_compiled_amount, RuntimeAmount},
            },
            ids::UnitInstanceId,
            runtime_facts::{ActionOrdinal, EquipmentSourceSnapshot},
        },
        value_expr::{AmountPlanKind, ExprUsage},
        world::authoring_observation::StoredAmountWindow,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountBattle {
    Active,
    LastTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "selector", rename_all = "snake_case")]
pub enum AmountQuery {
    Latest {
        battle: AmountBattle,
    },
    Exact {
        battle_key: BattleKey,
        evaluation_ordinal: u64,
    },
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum UnavailableReason {
    ObservationDisabled,
    NoBattle,
    NotYetEvaluated,
    NotRetained,
}

enum Selection<'a> {
    Record(&'a RecordedAmountEvaluation),
    Unavailable(UnavailableReason),
}

#[derive(Serialize)]
struct UnavailableDocument {
    kind: &'static str,
    selector: AmountQuery,
    status: &'static str,
    reason: UnavailableReason,
}

#[derive(Serialize)]
struct AmountIdentity<'a> {
    content_generation: &'a crate::game::data::content_pack::ContentIdentity,
    battle_key: BattleKey,
    evaluation_ordinal: u64,
}

#[derive(Serialize)]
struct CastIdentity<'a> {
    skill_id: &'a str,
    caster_instance_id: UnitInstanceId,
    cast_seq: u64,
    ability_event_seq: u64,
}

#[derive(Serialize)]
struct SourceIdentity<'a> {
    reinput: Option<crate::game::battle::runtime_facts::CommittedReinputContext>,
    flow: Option<crate::game::battle::runtime_facts::FlowOccurrence>,
    actor: Option<UnitInstanceId>,
    equipment: Option<EquipmentSourceSnapshot>,
    fragment: Option<&'a crate::game::skill_fragment::FragmentSourceSnapshot>,
    action_ordinal: Option<ActionOrdinal>,
    delivery_ordinal: Option<u64>,
    cast: Option<CastIdentity<'a>>,
}

#[derive(Serialize)]
struct EvaluatedDocument<'a> {
    kind: &'static str,
    selector: AmountQuery,
    status: &'static str,
    basis: &'static str,
    identity: AmountIdentity<'a>,
    site: &'a crate::game::ability::authoring::EffectSite,
    semantic_fingerprint: &'a str,
    amount_kind: AmountPlanKind,
    provenance: crate::game::value_expr::AmountProvenance,
    moment: crate::game::value_expr::EvalMoment,
    source: SourceIdentity<'a>,
    release_target: Option<UnitInstanceId>,
    consumption_target: Option<UnitInstanceId>,
    release_time_ms: u64,
    consume_time_ms: u64,
    effect_index: usize,
    amount_semantics: &'static str,
    typed_base_amount: &'a crate::game::value_expr::TypedInteger,
    narrowed_base_amount: RuntimeAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    effective_relic_amount: Option<RuntimeAmount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relic_numeric: Option<&'a crate::game::battle::core::relic_numeric::RelicNumericBinding>,
    trace: &'a [crate::game::value_expr::TraceEntry],
    runtime_expr_usage: ExprUsage,
    query_expr_usage: ExprUsage,
    retained_extent_bytes: usize,
}

#[derive(Serialize)]
struct ExplainedDocument<'a> {
    #[serde(flatten)]
    evaluated: EvaluatedDocument<'a>,
    static_plan: amount::CompiledAmountExplanation<'a>,
}

pub(super) fn encode(
    application: &CoreApplication,
    request: AmountQuery,
    explanation: bool,
) -> Result<String, AuthoringError> {
    let kind = if explanation {
        "amount_explanation"
    } else {
        "amount_preview"
    };
    let selection = select(application, request)?;
    let Selection::Record(record) = selection else {
        let Selection::Unavailable(reason) = selection else {
            unreachable!()
        };
        return super::output::encode_result(&UnavailableDocument {
            kind,
            selector: request,
            status: "unavailable",
            reason,
        });
    };

    encode_record(application.content_identity(), record, request, explanation)
}

fn encode_record(
    content_identity: &crate::game::data::content_pack::ContentIdentity,
    record: &RecordedAmountEvaluation,
    request: AmountQuery,
    explanation: bool,
) -> Result<String, AuthoringError> {
    let kind = if explanation {
        "amount_explanation"
    } else {
        "amount_preview"
    };
    validate_record(content_identity, record)?;
    let mut replayed = evaluate_compiled_amount(record.plan.as_ref(), record.kind(), &record.frame)
        .map_err(|_| invariant("The retained amount frame could not be replayed."))?;
    let narrowed_base_amount = replayed.amount;
    if let Some(binding) = record.source.relic_numeric() {
        binding
            .validate_skill(content_identity, &record.plan, &record.source)
            .map_err(|_| {
                invariant("The retained relic binding differs from its original source.")
            })?;
        control::charge(binding.contributions.len() as u64, "recorded_relic_compose")
            .map_err(control_error)?;
        replayed.amount = binding
            .apply_amount(&record.plan, replayed.amount)
            .map_err(|_| invariant("The retained relic contribution could not be replayed."))?;
    }

    control::charge_serialized(&record.evaluated, "recorded_amount_compare")
        .and_then(|()| control::charge_serialized(&replayed.evaluated, "replayed_amount_compare"))
        .map_err(control_error)?;
    if replayed.amount != record.amount || replayed.evaluated != record.evaluated {
        return Err(invariant(
            "The retained result differs from evaluation of its exact immutable frame.",
        ));
    }
    control::charge_serialized(record.source.as_ref(), "recorded_amount_source")
        .map_err(control_error)?;
    control::admit(
        control::Resource::MetadataBytes,
        0,
        64,
        "recorded_amount_fingerprint",
    )
    .and_then(|()| control::charge(64, "recorded_amount_fingerprint"))
    .map_err(control_error)?;
    let fingerprint = record.plan.semantic_fingerprint_hex();
    let cast = record.source.cast.as_deref().map(|cast| CastIdentity {
        skill_id: cast.skill_id.as_str(),
        caster_instance_id: cast.caster_instance_id,
        cast_seq: cast.cast_seq,
        ability_event_seq: cast.ability_event_seq,
    });
    let evaluated = EvaluatedDocument {
        kind,
        selector: request,
        status: "evaluated",
        basis: "recorded_evaluation",
        identity: AmountIdentity {
            content_generation: &record.generation,
            battle_key: record.battle_key.into(),
            evaluation_ordinal: record.evaluation_ordinal,
        },
        site: record.site(),
        semantic_fingerprint: &fingerprint,
        amount_kind: record.kind(),
        provenance: record.provenance(),
        moment: record.moment(),
        source: SourceIdentity {
            reinput: record.source.reinput().map(AsRef::as_ref).copied(),
            flow: record.source.flow.as_deref().copied(),
            actor: record.source.actor,
            equipment: record.source.equipment,
            fragment: record.source.fragment.as_deref(),
            action_ordinal: record.source.action_ordinal(),
            delivery_ordinal: record
                .source
                .delivery
                .as_deref()
                .map(|delivery| delivery.ordinal.0),
            cast,
        },
        release_target: record.release_target,
        consumption_target: record.consumption_target,
        release_time_ms: record.release_time_ms,
        consume_time_ms: record.consume_time_ms,
        effect_index: record.effect_index,
        amount_semantics: "base_amount_before_mitigation",
        typed_base_amount: &replayed.evaluated.value,
        narrowed_base_amount,
        effective_relic_amount: record.source.relic_numeric().map(|_| record.amount),
        relic_numeric: record.source.relic_numeric().map(AsRef::as_ref),
        trace: &replayed.evaluated.trace,
        runtime_expr_usage: record.runtime_usage,
        query_expr_usage: replayed.usage,
        retained_extent_bytes: record.retained_extent(),
    };
    if explanation {
        let static_plan = amount::describe_plan(
            record.kind(),
            record.provenance() == crate::game::value_expr::AmountProvenance::AuthoredExpression,
            record.plan.as_ref(),
        )?;
        super::output::encode_result(&ExplainedDocument {
            evaluated,
            static_plan,
        })
    } else {
        super::output::encode_result(&evaluated)
    }
}

#[cfg(test)]
pub(crate) fn encode_record_for_test(
    record: &RecordedAmountEvaluation,
    request: AmountQuery,
    explanation: bool,
) -> Result<String, AuthoringError> {
    encode_record(record.generation.as_ref(), record, request, explanation)
}

fn select(
    application: &CoreApplication,
    request: AmountQuery,
) -> Result<Selection<'_>, AuthoringError> {
    let enabled = application
        .amount_observation_enabled()
        .map_err(application_error)?;
    if !enabled {
        return Ok(Selection::Unavailable(
            UnavailableReason::ObservationDisabled,
        ));
    }
    match request {
        AmountQuery::Latest { battle } => {
            let window = match battle {
                AmountBattle::Active => StoredAmountWindow::Active,
                AmountBattle::LastTerminal => StoredAmountWindow::LastTerminal,
            };
            match application
                .stored_amount(window)
                .map_err(application_error)?
            {
                None => Ok(Selection::Unavailable(UnavailableReason::NoBattle)),
                Some(owner) => match owner.record {
                    Some(record) => Ok(Selection::Record(record.as_ref())),
                    None => Ok(Selection::Unavailable(UnavailableReason::NotYetEvaluated)),
                },
            }
        }
        AmountQuery::Exact {
            battle_key,
            evaluation_ordinal,
        } => {
            if evaluation_ordinal == 0 {
                return Err(request_error(
                    "authoring_amount_query_invalid",
                    "evaluation_ordinal",
                    "An exact evaluation ordinal must be non-zero.",
                ));
            }
            let mut any_battle = false;
            for window in [StoredAmountWindow::Active, StoredAmountWindow::LastTerminal] {
                let Some(owner) = application
                    .stored_amount(window)
                    .map_err(application_error)?
                else {
                    continue;
                };
                any_battle = true;
                if BattleKey::from(owner.key) != battle_key {
                    continue;
                }
                return Ok(match owner.record {
                    Some(record) if record.evaluation_ordinal == evaluation_ordinal => {
                        Selection::Record(record)
                    }
                    Some(_) => Selection::Unavailable(UnavailableReason::NotRetained),
                    None => Selection::Unavailable(UnavailableReason::NotYetEvaluated),
                });
            }
            Ok(Selection::Unavailable(if any_battle {
                UnavailableReason::NotRetained
            } else {
                UnavailableReason::NoBattle
            }))
        }
    }
}

fn validate_record(
    content_identity: &crate::game::data::content_pack::ContentIdentity,
    record: &RecordedAmountEvaluation,
) -> Result<(), AuthoringError> {
    let plan = record.plan.as_ref();
    control::charge_serialized(record.source.as_ref(), "recorded_amount_identity")
        .map_err(control_error)?;
    let kind_matches = matches!(
        (record.kind(), record.amount),
        (AmountPlanKind::Damage, RuntimeAmount::Damage(_))
            | (AmountPlanKind::Heal, RuntimeAmount::Heal(_))
            | (AmountPlanKind::Shield, RuntimeAmount::Shield(_))
    );
    let source_matches = record.source.authored.as_deref() == Some(plan.source());
    let cast_matches = record.source.cast.as_ref().is_none_or(|cast| {
        record
            .source
            .actor
            .is_none_or(|actor| actor == cast.caster_instance_id)
    });
    if record.evaluation_ordinal == 0
        || record.generation.as_ref() != content_identity
        || record.site() != &plan.source().site
        || record.fingerprint() != plan.semantic_fingerprint()
        || record.provenance() != plan.provenance()
        || record.moment() != plan.moment()
        || record.release_time_ms > record.consume_time_ms
        || !kind_matches
        || !source_matches
        || !record.source_binding_matches()
        || !cast_matches
    {
        return Err(invariant(
            "The retained amount identity is inconsistent with its immutable plan and source.",
        ));
    }
    Ok(())
}

fn application_error(error: crate::application::ApplicationError) -> AuthoringError {
    AuthoringError::Application {
        phase: ExecutionPhase::Inspection,
        diagnostic: error,
    }
}

fn invariant(reason: &str) -> AuthoringError {
    request_error("authoring_amount_record_invalid", "record", reason)
}

#[cfg(test)]
pub(crate) fn validate_record_generation_for_test(
    identity: &crate::game::data::content_pack::ContentIdentity,
    record: &RecordedAmountEvaluation,
) -> Result<(), AuthoringError> {
    validate_record(identity, record)
}
