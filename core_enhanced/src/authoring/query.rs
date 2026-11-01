use super::{
    amount, control, definition, output, AuthoringContent, AuthoringError, CancellationToken,
    ExecutionLimits, ExecutionPhase, ExecutionUsage, RequestFailure,
};
use crate::game::{
    ability::{
        authoring::{self, AuthoredSkillEffect, EffectSite, EffectSource, EffectSupport},
        DeliveryDef, SkillTarget, StepTargetingMode,
    },
    data::content_pack::{
        CatalogKind, ContentContract, ContentIdentity, DefinitionKey, DefinitionKind,
        DefinitionSource, PackManifest, SourceFileReceipt, SourceReceipt,
    },
};
use serde::{Deserialize, Serialize};

/// A completed, size-checked JSON document. It owns no generation or mutable
/// gameplay authority. Usage measures preparation of this document.
#[derive(Debug)]
pub struct AuthoringQueryResult {
    document: String,
    usage: ExecutionUsage,
}
impl AuthoringQueryResult {
    pub(super) fn prepared(document: String, usage: ExecutionUsage) -> Self {
        Self { document, usage }
    }
    pub fn document(&self) -> &str {
        &self.document
    }
    pub fn usage(&self) -> &ExecutionUsage {
        &self.usage
    }
    pub fn into_document(self) -> String {
        self.document
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExplainQuery {
    Definition { key: DefinitionKey },
    CompiledGraph { key: DefinitionKey },
    Effect { site: EffectSite },
}

/// A bounded receipt for the already validated, unchanged publication.
pub fn validation_receipt(
    content: &AuthoringContent,
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
) -> Result<AuthoringQueryResult, AuthoringError> {
    #[derive(Serialize)]
    struct Receipt<'a> {
        status: &'static str,
        identity: &'a ContentIdentity,
        runtime_safety_sha256: &'a str,
        source_receipt: &'a SourceReceipt,
        validation_usage: &'a ExecutionUsage,
    }
    run_query(limits, cancellation, || {
        // Admit the closed receipt's source strings before JSON escape scanning.
        // Version fields are fixed Copy values; exhaustive patterns guard growth.
        let ContentIdentity {
            contract,
            semantic_sha256,
        } = content.token.identity();
        let ContentContract {
            pack_format,
            catalog_schema,
            authoring,
            execution,
            fingerprint,
        } = contract;
        fn copy_leaf<T: Copy>(_: &T) {}
        copy_leaf(pack_format);
        copy_leaf(catalog_schema);
        copy_leaf(authoring);
        copy_leaf(execution);
        copy_leaf(fingerprint);
        let SourceReceipt { files } = content.token.source_receipt();
        control::admit(
            control::Resource::Definitions,
            0,
            files.len(),
            "validation_receipt_files",
        )
        .map_err(control_error)?;
        control::charge(
            (semantic_sha256.len() as u64).saturating_add(256),
            "validation_receipt_source_scan",
        )
        .map_err(control_error)?;
        for SourceFileReceipt {
            pack_id,
            path,
            byte_count,
            sha256,
        } in files
        {
            copy_leaf(byte_count);
            let bytes = pack_id
                .len()
                .checked_add(path.len())
                .and_then(|n| n.checked_add(sha256.len()))
                .and_then(|n| n.checked_add(128))
                .unwrap_or(usize::MAX);
            control::charge(bytes as u64, "validation_receipt_source_scan")
                .map_err(control_error)?;
        }
        output::encode_result(&Receipt {
            status: "validated",
            identity: content.token.identity(),
            runtime_safety_sha256: content.token.runtime_safety().effective_sha256(),
            source_receipt: content.token.source_receipt(),
            validation_usage: content.validation_usage(),
        })
    })
}

#[derive(Serialize)]
struct RuntimeObservationSupport {
    authority: &'static str,
    fact_kinds: &'static [&'static str],
    source_slots: &'static [&'static str],
    action_milestones: &'static [&'static str],
    retention: &'static str,
    availability: &'static str,
}

const RUNTIME_OBSERVATION: RuntimeObservationSupport = RuntimeObservationSupport {
    authority: "actual battle mutations, independent of optional event logs and trace",
    fact_kinds: &["damage", "heal", "cost", "resource_change", "death"],
    source_slots: &["actor", "equipment", "cast", "authored", "action_ordinal", "flow", "reinput"],
    action_milestones: &["accepted", "released", "control_terminal", "effects_settled"],
    retention: "bounded current actions, aggregates and latest witnesses; no full event history",
    availability: "active trial runtime_facts and immutable terminal runtime_facts; unknown source slots are null",
};

#[derive(Serialize)]
struct Description<'a> {
    typed_resources: TypedResourceSupport,
    identity: &'a ContentIdentity,
    runtime_safety_sha256: &'a str,
    source_receipt: &'a SourceReceipt,
    catalog_kinds: &'static [CatalogKind],
    definition_kinds: &'static [DefinitionKind],
    definitions: &'a [(DefinitionKey, DefinitionSource)],
    editable_effect_support: &'static [EffectSupport],
    supported_capabilities: &'static [&'static str],
    explain_queries: &'static [&'static str],
    runtime_observation: &'static RuntimeObservationSupport,
    value_expression_explanation: &'static ValueExpressionExplanationSupport,
}

#[derive(Serialize)]
struct ValueExpressionExplanationSupport {
    static_effect_explain: &'static str,
    numeric_preview: &'static str,
    authored_expression_publication: &'static str,
}

const VALUE_EXPRESSION_EXPLANATION: ValueExpressionExplanationSupport =
    ValueExpressionExplanationSupport {
        static_effect_explain:
            "compiled amount structure, units, moment, requirements, source, fingerprint and cost",
        numeric_preview:
            "not supported by static explain; runtime context is never guessed or evaluated",
        authored_expression_publication:
            "validated DamageExpr/HealExpr Snapshot/Live subset is supported",
    };

#[derive(Serialize)]
struct TypedResourceSupport {
    effect: &'static str,
    operations: &'static [&'static str],
    owner_selectors: &'static [&'static str],
    lifetimes: &'static [&'static str],
    observations: &'static str,
    atomicity: &'static str,
}

/// Describe the actual published inventory and current effect-edit support.
pub fn describe(
    content: &AuthoringContent,
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
) -> Result<AuthoringQueryResult, AuthoringError> {
    run_query(limits, cancellation, || {
        let metadata = content.token.source_metadata().ok_or_else(|| {
            request_error(
                "authoring_source_unavailable",
                "content",
                "This publication has no external resolution metadata.",
            )
        })?;
        control::admit(
            control::Resource::Definitions,
            0,
            metadata.definitions().len(),
            "describe_definitions",
        )
        .map_err(control_error)?;
        output::encode_result(&Description {
            typed_resources: TypedResourceSupport {
                effect: "Resources { operations: [...] }",
                operations: &["Grant(u32)", "Decay(u32)", "Consume(u32)"],
                owner_selectors: &["SourceEmployee", "SourceEquipment", "SourceFragment", "TargetEmployee"],
                lifetimes: &["Run", "Attempt", "Deployment", "Activation", "Delivery", "Chain"],
                observations: "borrowed run_resources and active_typed_resources; sparse entries use explicit catalog initial; scope ordinals are diagnostic only; terminal costs and changes remain in runtime_facts",
                atomicity: "authored-order batch within the enclosing application/timestamp transaction; Consume reserves and commits synchronously; shortage rejects the whole transaction",
            },
            identity: content.token.identity(),
            runtime_safety_sha256: content.token.runtime_safety().effective_sha256(),
            source_receipt: content.token.source_receipt(),
            catalog_kinds: CatalogKind::ALL,
            definition_kinds: DefinitionKind::ALL,
            definitions: metadata.definitions(),
            editable_effect_support: authoring::EFFECT_SUPPORT,
            supported_capabilities: PackManifest::SUPPORTED_CAPABILITIES,
            explain_queries: &["definition", "effect", "compiled_graph"],
            runtime_observation: &RUNTIME_OBSERVATION,
            value_expression_explanation: &VALUE_EXPRESSION_EXPLANATION,
        })
    })
}

#[derive(Serialize)]
struct EffectExplanation<'a> {
    flow: std::sync::Arc<crate::game::ability::flow::CompiledSkillFlow>,
    identity: &'a ContentIdentity,
    runtime_safety_sha256: &'a str,
    definition_source: Option<&'a DefinitionSource>,
    site: &'a EffectSite,
    target: &'a SkillTarget,
    targeting: &'a StepTargetingMode,
    delivery: &'a DeliveryDef,
    effect: &'a AuthoredSkillEffect,
    source: Option<&'a EffectSource>,
    origin_definition_source: Option<&'a DefinitionSource>,
    editable_support: Option<&'static EffectSupport>,
    runtime_observation: &'static RuntimeObservationSupport,
    value_expression: Option<amount::CompiledAmountExplanation<'a>>,
}

/// Explain exact definition resolution or a compiled skill effect's authored
/// provenance. Queries never read source files again or execute gameplay.
pub fn explain(
    content: &AuthoringContent,
    query: &ExplainQuery,
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
) -> Result<AuthoringQueryResult, AuthoringError> {
    run_query(limits, cancellation, || {
        control::charge_serialized(query, "explain_request").map_err(control_error)?;
        let metadata = content.token.source_metadata().ok_or_else(|| {
            request_error(
                "authoring_source_unavailable",
                "content",
                "This publication has no external resolution metadata.",
            )
        })?;
        control::charge(
            metadata.definitions().len() as u64 + 1,
            "explain_definition_source",
        )
        .map_err(control_error)?;
        match query {
            ExplainQuery::Definition { key } | ExplainQuery::CompiledGraph { key } => {
                if matches!(query, ExplainQuery::CompiledGraph { .. })
                    && !matches!(
                        key.kind,
                        DefinitionKind::Skill
                            | DefinitionKind::BossPattern
                            | DefinitionKind::BossActorBehavior
                            | DefinitionKind::BossEncounter
                    )
                {
                    return Err(request_error(
                        "authoring_query_unsupported",
                        "key.kind",
                        "This kind has no compiled boss graph.",
                    ));
                }
                control::charge(
                    (metadata.writers().len() as u64 + metadata.definitions().len() as u64 + 1)
                        .saturating_mul(key.id.len() as u64 + 1),
                    "explain_writers_and_key",
                )
                .map_err(control_error)?;
                let source = metadata.source_for(key);
                if matches!(query, ExplainQuery::CompiledGraph { .. }) && source.is_none() {
                    return Err(request_error(
                        "authoring_definition_not_found",
                        "key",
                        "No compiled graph with this exact key is present in this publication.",
                    ));
                }
                let writers: Vec<_> = metadata
                    .writers()
                    .iter()
                    .filter(|writer| &writer.target == key)
                    .collect();
                if source.is_none() && writers.is_empty() {
                    return Err(request_error(
                        "authoring_definition_not_found",
                        "key",
                        "No definition or writer has this exact key.",
                    ));
                }
                if matches!(query, ExplainQuery::CompiledGraph { .. })
                    && key.kind == DefinitionKind::Skill
                {
                    definition::skill_flow_document(content, key, source, &writers)
                } else {
                    definition::document(content, key, source, &writers)
                }
            }
            ExplainQuery::Effect { site } => {
                control::charge(
                    (metadata.definitions().len() as u64 + 1)
                        .saturating_mul(site.action.definition_id.as_str().len() as u64 + 1),
                    "explain_effect_owner_source",
                )
                .map_err(control_error)?;
                let skill = content
                    .token
                    .skill_data
                    .get_by_id(site.action.definition_id.as_str())
                    .ok_or_else(|| {
                        request_error(
                            "authoring_definition_not_found",
                            "site.action.definition_id",
                            "No skill has this exact ID.",
                        )
                    })?;
                control::charge(
                    (skill.steps.len() as u64 + 1)
                        .saturating_mul(site.action.action_id.len() as u64 + 1),
                    "explain_actions",
                )
                .map_err(control_error)?;
                let step = skill
                    .steps
                    .iter()
                    .find(|step| step.id == site.action.action_id)
                    .ok_or_else(|| {
                        request_error(
                            "authoring_action_not_found",
                            "site.action.action_id",
                            "No action in this skill has this exact ID.",
                        )
                    })?;
                control::charge(
                    (step.effects.len() as u64 + 1).saturating_mul(site.effect_id.len() as u64 + 1),
                    "explain_effects",
                )
                .map_err(control_error)?;
                let effect = step
                    .effects
                    .iter()
                    .find(|effect| effect.id == site.effect_id)
                    .ok_or_else(|| {
                        request_error(
                            "authoring_effect_not_found",
                            "site.effect_id",
                            "No effect in this action has this exact ID.",
                        )
                    })?;
                let key = DefinitionKey {
                    kind: DefinitionKind::Skill,
                    id: skill.id.as_str().to_owned(),
                };
                let origin_id = match &effect.origin {
                    authoring::EffectOrigin::Original => &skill.id,
                    authoring::EffectOrigin::Added {
                        source_definition_id,
                    } => source_definition_id,
                };
                control::charge(
                    (metadata.definitions().len() as u64 + 1)
                        .saturating_mul(origin_id.as_str().len() as u64 + 1),
                    "explain_origin_source",
                )
                .map_err(control_error)?;
                let origin_key = DefinitionKey {
                    kind: DefinitionKind::Skill,
                    id: origin_id.as_str().to_owned(),
                };
                let catalog = content.token.value_expr_catalog();
                amount::charge_catalog_lookup(catalog.len(), site)?;
                let value_expression = amount::explain_effect(site, effect, catalog.get(site))?;
                let flow = content
                    .token
                    .skill_data
                    .compiled_flow(skill.id.as_str())
                    .map_err(|cause| {
                        request_error("authoring_definition_projection_missing", "flow", &cause)
                    })?;
                output::encode_result(&EffectExplanation {
                    flow,
                    identity: content.token.identity(),
                    runtime_safety_sha256: content.token.runtime_safety().effective_sha256(),
                    definition_source: metadata.source_for(&key),
                    site,
                    target: &step.target,
                    targeting: &step.targeting,
                    delivery: &step.delivery,
                    effect,
                    source: effect.source().map(|source| source.as_ref()),
                    origin_definition_source: metadata.source_for(&origin_key),
                    editable_support: authoring::effect_support(&effect.op, &step.delivery),
                    runtime_observation: &RUNTIME_OBSERVATION,
                    value_expression,
                })
            }
        }
    })
}

fn run_query(
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
    body: impl FnOnce() -> Result<String, AuthoringError>,
) -> Result<AuthoringQueryResult, AuthoringError> {
    let (document, usage) = control::run(limits, cancellation, ExecutionPhase::Inspection, body)
        .map_err(|diagnostic| AuthoringError::Execution { diagnostic })?;
    Ok(AuthoringQueryResult {
        document: document?,
        usage,
    })
}

pub(super) fn request_error(
    code: &'static str,
    field_path: &'static str,
    reason: &str,
) -> AuthoringError {
    AuthoringError::Request {
        diagnostic: RequestFailure {
            code,
            phase: ExecutionPhase::Inspection,
            field_path,
            reason: reason.to_owned(),
        },
    }
}

pub(super) fn control_error(error: crate::game::behavior::GameError) -> AuthoringError {
    // The enclosing run_query returns the first structured execution failure.
    request_error(
        "authoring_query_interrupted",
        "query",
        &format!("{error:?}"),
    )
}
