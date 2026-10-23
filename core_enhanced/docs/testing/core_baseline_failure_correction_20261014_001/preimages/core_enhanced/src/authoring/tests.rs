use super::*;
use crate::game::data::content_pack::tests::ExternalFixture;

fn attempt(
    fixture: &ExternalFixture,
    limits: &AuthoringLimits,
) -> Result<AuthoringContent, AuthoringError> {
    validate(
        fixture.root(),
        &fixture.selection(),
        limits,
        &CancellationToken::default(),
    )
}
fn failure(result: Result<AuthoringContent, AuthoringError>) -> ExecutionFailure {
    match result {
        Err(AuthoringError::Execution { diagnostic }) => diagnostic,
        Err(other) => panic!("wrong rejection: {other}"),
        Ok(_) => panic!("candidate published"),
    }
}

#[test]
fn controlled_external_load_uses_the_same_token_and_exact_generated_work_boundary() {
    let fixture = ExternalFixture::official();
    let plain = fixture.load();
    let mut limits = AuthoringLimits::default();
    let controlled = attempt(&fixture, &limits).unwrap();
    assert_eq!(controlled.token().identity(), plain.identity());
    assert_eq!(controlled.token().source_receipt(), plain.source_receipt());
    let usage = controlled.validation_usage();
    println!(
        "actual_authoring_validation_usage={}",
        serde_json::to_string(usage).unwrap()
    );
    assert!(usage.work > 0);
    assert!(usage.pending_events_peak > 0);
    assert!(usage.retained_entities_peak > 0);
    assert!(usage.metadata_bytes_peak > 0);
    assert_eq!(
        usage.definitions_peak,
        plain.source_metadata().unwrap().definitions().len()
    );
    limits.execution.max_work = usage.work;
    assert_eq!(
        attempt(&fixture, &limits).unwrap().validation_usage(),
        usage
    );
    limits.execution.max_work -= 1;
    let rejected = failure(attempt(&fixture, &limits));
    assert_eq!(rejected.code, "authoring_work_limit");
    assert_eq!(rejected.phase, ExecutionPhase::GeneratedSetup);
    assert_eq!(controlled.token().identity(), plain.identity());
    // Failed control must not poison the ordinary loader or the next attempt.
    assert_eq!(fixture.load().identity(), plain.identity());
    assert_eq!(
        attempt(&fixture, &AuthoringLimits::default())
            .unwrap()
            .token()
            .identity(),
        plain.identity()
    );
}

#[test]
fn controlled_external_resource_publication_work_boundary_returns_error_without_unwind() {
    for nonempty in [false, true] {
        let fixture = ExternalFixture::official();
        let path = "official/data/resources/base.ron";
        if nonempty {
            fixture.write(
                path,
                r#"(resources: [
                (id: "publication_a", owner: Employee, lifetime: Run, initial: 1, cap: 2),
                (id: "publication_b", owner: EquipmentCopy, lifetime: Attempt, initial: 1, cap: 2),
            ])"#,
            );
        }
        let original_source = fixture.read(path);
        let held = attempt(&fixture, &AuthoringLimits::default()).unwrap();
        let identity = held.token().identity().clone();
        let receipt = held.token().source_receipt().clone();
        let mut limits = AuthoringLimits::default();
        // Find the public FinalValidation -> GeneratedSetup boundary from actual
        // errors. No constructor charge count or implementation hook is used.
        let mut low = 1;
        let mut high = held.validation_usage().work;
        while low < high {
            let middle = low + (high - low) / 2;
            limits.execution.max_work = middle;
            match attempt(&fixture, &limits) {
                Ok(_) => high = middle,
                Err(AuthoringError::Execution { diagnostic }) => {
                    assert_eq!(diagnostic.code, "authoring_work_limit");
                    if diagnostic.phase == ExecutionPhase::GeneratedSetup {
                        high = middle;
                    } else {
                        low = middle + 1;
                    }
                }
                Err(other) => panic!("unexpected content rejection: {other}"),
            }
        }
        assert!(low > 1);
        for _ in 0..2 {
            limits.execution.max_work = low - 1;
            let rejected = failure(attempt(&fixture, &limits));
            assert_eq!(rejected.code, "authoring_work_limit");
            assert_eq!(rejected.phase, ExecutionPhase::FinalValidation);
            assert!(
                rejected.site.starts_with("resource_reference_"),
                "{rejected:?}"
            );
            println!(
                "publication_resource_boundary nonempty={nonempty} budget={}: {rejected:?}",
                low - 1
            );
            assert_eq!(held.token().identity(), &identity);
            assert_eq!(held.token().source_receipt(), &receipt);
            assert_eq!(fixture.read(path), original_source);
        }
        limits.execution.max_work = low;
        let accepted_prefix = failure(attempt(&fixture, &limits));
        assert_eq!(accepted_prefix.phase, ExecutionPhase::GeneratedSetup);
        assert_eq!(accepted_prefix.code, "authoring_work_limit");
        let retry = attempt(&fixture, &AuthoringLimits::default()).unwrap();
        assert_eq!(retry.token().identity(), &identity);
        assert_eq!(retry.token().source_receipt(), &receipt);
        assert_eq!(retry.validation_usage(), held.validation_usage());
        assert_eq!(fixture.load().identity(), &identity);
    }
}

#[test]
fn controlled_metadata_caps_are_exact_and_rejection_keeps_existing_content() {
    let fixture = ExternalFixture::official();
    let a = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let mut limits = AuthoringLimits::default();
    limits.execution.max_metadata_bytes = a.validation_usage().metadata_bytes_peak;
    assert_eq!(
        attempt(&fixture, &limits).unwrap().token().identity(),
        a.token().identity()
    );
    limits.execution.max_metadata_bytes -= 1;
    let rejected = failure(attempt(&fixture, &limits));
    assert_eq!(rejected.code, "authoring_metadata_limit");
    assert_eq!(rejected.phase, ExecutionPhase::Resolution);
    limits = AuthoringLimits::default();
    limits.execution.max_definitions = a.validation_usage().definitions_peak - 1;
    assert_eq!(
        failure(attempt(&fixture, &limits)).code,
        "authoring_definition_limit"
    );
    assert_eq!(
        a.token().identity(),
        attempt(&fixture, &AuthoringLimits::default())
            .unwrap()
            .token()
            .identity()
    );
}

#[test]
fn cancelled_and_invalid_limits_fail_without_installing_a_scope_or_publishing() {
    let fixture = ExternalFixture::official();
    let token = CancellationToken::default();
    token.cancel();
    let rejected = failure(validate(
        fixture.root(),
        &fixture.selection(),
        &AuthoringLimits::default(),
        &token,
    ));
    assert_eq!(rejected.code, "authoring_cancelled");
    assert_eq!(rejected.phase, ExecutionPhase::Acquisition);
    let mut limits = AuthoringLimits::default();
    limits.input.max_file_bytes += 1;
    assert_eq!(
        failure(attempt(&fixture, &limits)).code,
        "authoring_limits_invalid"
    );
    limits = AuthoringLimits::default();
    limits.execution.max_work = 0;
    assert_eq!(
        failure(attempt(&fixture, &limits)).code,
        "authoring_limits_invalid"
    );
    assert!(control::poll("ordinary_after_failed_entry").is_ok());
    assert!(attempt(&fixture, &AuthoringLimits::default()).is_ok());
}

#[test]
fn common_validation_preserves_the_production_diagnostic_without_guessing_a_pack() {
    let fixture = ExternalFixture::official();
    fixture.write("official/pack.ron", "()");
    let plain = content_pack::load(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
    )
    .err()
    .unwrap();
    let Err(AuthoringError::Content { diagnostic }) =
        attempt(&fixture, &AuthoringLimits::default())
    else {
        panic!("content diagnostic expected");
    };
    assert_eq!(diagnostic, plain);
    assert_eq!(diagnostic.pack_id, None);
}

#[test]
fn excessive_authored_spawn_count_is_rejected_before_appearance_seed_expansion() {
    use crate::game::data::pve_data::{PveEncounterDatabase, PveWaveEnemyData, PveWaveSource};
    let fixture = ExternalFixture::official();
    let path = "official/data/pve/encounters.ron";
    let mut pve: PveEncounterDatabase = ron::de::from_str(&fixture.read(path)).unwrap();
    pve.encounters[0].waves[0].source =
        PveWaveSource::Manual(vec![PveWaveEnemyData::CorrodedEmployee {
            profile_id: "corroded_guard".into(),
            tier: crate::game::enums::Tier::I,
            count: 1_000_000,
        }]);
    fixture.write(path, ron::ser::to_string(&pve).unwrap());
    let rejected = failure(attempt(&fixture, &AuthoringLimits::default()));
    assert_eq!(rejected.code, "authoring_entity_limit");
    assert_eq!(rejected.site, "preview_wave_entities");
    assert_eq!(rejected.phase, ExecutionPhase::GeneratedSetup);
    assert_eq!(rejected.requested, Some(1_000_000));
}

#[test]
fn describe_uses_current_support_and_immutable_source_with_exact_output_limit() {
    let fixture = ExternalFixture::official();
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let cancellation = CancellationToken::default();
    let invalid = ExecutionLimits {
        max_result_bytes: 0,
        ..Default::default()
    };
    assert!(
        matches!(describe(&content, &invalid, &cancellation), Err(AuthoringError::Execution { diagnostic }) if diagnostic.code == "authoring_limits_invalid" && diagnostic.phase == ExecutionPhase::Inspection)
    );
    let first = describe(&content, &ExecutionLimits::default(), &cancellation).unwrap();
    let value: serde_json::Value = serde_json::from_str(first.document()).unwrap();
    assert_eq!(value["catalog_kinds"].as_array().unwrap().len(), 26);
    assert_eq!(value["definition_kinds"].as_array().unwrap().len(), 45);
    assert!(value["catalog_kinds"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("Relics")));
    assert!(value["definition_kinds"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("Relic")));
    assert!(value["definition_kinds"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("IndependentObject")));
    assert_eq!(
        value["definitions"].as_array().unwrap().len(),
        content.validation_usage().definitions_peak
    );
    assert_eq!(
        value["editable_effect_support"],
        serde_json::to_value(crate::game::ability::authoring::EFFECT_SUPPORT).unwrap()
    );
    assert_eq!(
        value["identity"],
        serde_json::to_value(content.token().identity()).unwrap()
    );
    assert_eq!(first.usage().result_bytes_peak, first.document().len());
    assert_eq!(
        value["value_expression_explanation"]["static_effect_explain"],
        "compiled amount structure, units, moment, requirements, source, fingerprint and cost"
    );
    assert_eq!(
        value["value_expression_explanation"]["numeric_preview"],
        "not supported by static explain; runtime context is never guessed or evaluated"
    );
    assert_eq!(
        value["value_expression_explanation"]["authored_expression_publication"],
        "validated DamageExpr/HealExpr Snapshot/Live subset is supported"
    );
    let mut limits = ExecutionLimits {
        max_result_bytes: first.document().len(),
        ..Default::default()
    };
    assert_eq!(
        describe(&content, &limits, &cancellation)
            .unwrap()
            .document(),
        first.document()
    );
    limits.max_result_bytes -= 1;
    let Err(AuthoringError::Execution { diagnostic }) = describe(&content, &limits, &cancellation)
    else {
        panic!("oversized query returned success");
    };
    assert_eq!(diagnostic.code, "authoring_result_limit");
    assert_eq!(diagnostic.phase, ExecutionPhase::Inspection);
    fixture.write("official/pack.ron", "not a manifest");
    let repeated = describe(&content, &ExecutionLimits::default(), &cancellation).unwrap();
    assert_eq!(repeated.document(), first.document());
    assert_eq!(repeated.usage(), first.usage());
    cancellation.cancel();
    assert!(
        matches!(describe(&content, &ExecutionLimits::default(), &cancellation), Err(AuthoringError::Execution { diagnostic }) if diagnostic.code == "authoring_cancelled")
    );
}

#[test]
fn public_effect_explain_describes_actual_legacy_damage_heal_and_non_amount_effects() {
    use crate::game::{ability::SkillEffectDef, value_expr::AmountPlanKind};

    let fixture = ExternalFixture::official();
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let mut amount_sites = Vec::new();
    for kind in [AmountPlanKind::Damage, AmountPlanKind::Heal] {
        let (site, entry) = content
            .token()
            .value_expr_catalog()
            .entries()
            .find(|(_, entry)| entry.kind() == kind)
            .unwrap_or_else(|| panic!("official catalog has no {kind:?} plan"));
        amount_sites.push((site.clone(), entry.kind()));
    }

    for (site, expected_kind) in amount_sites {
        let entry = content.token().value_expr_catalog().get(&site).unwrap();
        let skill = content
            .token()
            .skill_data
            .get_by_id(site.action.definition_id.as_str())
            .unwrap();
        let step = skill
            .steps
            .iter()
            .find(|step| step.id == site.action.action_id)
            .unwrap();
        let effect = step
            .effects
            .iter()
            .find(|effect| effect.id == site.effect_id)
            .unwrap();
        let result = explain(
            &content,
            &ExplainQuery::Effect { site: site.clone() },
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(result.document()).unwrap();
        let described = &value["value_expression"];
        assert_eq!(described["representation"], "compiled_amount_plan");
        assert_eq!(
            described["kind"],
            serde_json::to_value(expected_kind).unwrap()
        );
        assert_eq!(
            described["compiler_version"],
            crate::game::value_expr::VALUE_EXPR_COMPILER_VERSION
        );
        assert_eq!(
            described["provenance"],
            serde_json::to_value(entry.plan().provenance()).unwrap()
        );
        assert_eq!(
            described["moment"],
            serde_json::to_value(entry.plan().moment()).unwrap()
        );
        assert_eq!(
            described["result_unit"],
            serde_json::to_value(entry.plan().result_unit()).unwrap()
        );
        assert_eq!(
            described["source"],
            serde_json::to_value(entry.plan().source()).unwrap()
        );
        assert_eq!(
            described["semantic_fingerprint"],
            entry.plan().semantic_fingerprint_hex()
        );
        assert_eq!(
            described["requirements"],
            serde_json::to_value(entry.plan().requirements()).unwrap()
        );
        assert_eq!(
            described["cost"],
            serde_json::to_value(entry.plan().cost()).unwrap()
        );
        assert_eq!(described["publication"], "legacy_supported");
        assert_eq!(described["evaluation"]["context"], "context_free");
        assert_eq!(described["evaluation"]["performed"], false);
        assert!(described.get("amount").is_none());
        assert!(described.get("trace").is_none());
        assert!(described.get("final_damage").is_none());
        let segments = described["segments"].as_array().unwrap();
        assert_eq!(segments.len(), entry.plan().segment_count());
        for (index, segment) in segments.iter().enumerate() {
            assert_eq!(segment["index"], index);
            assert_eq!(
                segment["ops"],
                serde_json::to_value(entry.plan().segment(index).unwrap()).unwrap()
            );
        }
        assert_eq!(value["effect"], serde_json::to_value(effect).unwrap());
        assert!(matches!(
            effect.op,
            SkillEffectDef::Damage { .. } | SkillEffectDef::Heal { .. }
        ));
        assert!(effect.literal_amount().is_some());
    }

    let (site, effect) = content
        .token()
        .skill_data
        .skills
        .iter()
        .flat_map(|skill| {
            skill.steps.iter().flat_map(move |step| {
                step.effects.iter().map(move |effect| {
                    (
                        crate::game::ability::authoring::EffectSite {
                            action: crate::game::ability::authoring::ActionSite {
                                definition_id: skill.id.clone(),
                                action_id: step.id.clone(),
                            },
                            effect_id: effect.id.clone(),
                        },
                        effect,
                    )
                })
            })
        })
        .find(|(_, effect)| {
            !matches!(
                effect.op,
                SkillEffectDef::Damage { .. }
                    | SkillEffectDef::DamageExpr { .. }
                    | SkillEffectDef::Heal { .. }
                    | SkillEffectDef::HealExpr { .. }
            )
        })
        .expect("official catalog has a non-amount effect");
    assert!(content.token().value_expr_catalog().get(&site).is_none());
    let result = explain(
        &content,
        &ExplainQuery::Effect { site: site.clone() },
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(result.document()).unwrap();
    assert!(value["value_expression"].is_null());

    let amount_site = content
        .token()
        .value_expr_catalog()
        .entries()
        .next()
        .unwrap()
        .0
        .clone();
    let amount_skill = content
        .token()
        .skill_data
        .get_by_id(amount_site.action.definition_id.as_str())
        .unwrap();
    let amount_effect = amount_skill
        .steps
        .iter()
        .find(|step| step.id == amount_site.action.action_id)
        .unwrap()
        .effects
        .iter()
        .find(|effect| effect.id == amount_site.effect_id)
        .unwrap();
    let (missing, _) = control::run(
        &ExecutionLimits::default(),
        &CancellationToken::default(),
        ExecutionPhase::Inspection,
        || super::amount::explain_effect(&amount_site, amount_effect, None),
    )
    .unwrap();
    assert!(
        matches!(missing, Err(AuthoringError::Request { diagnostic }) if diagnostic.code == "authoring_amount_plan_unavailable")
    );
    assert_eq!(value["effect"], serde_json::to_value(effect).unwrap());
}

#[test]
fn effect_explain_follows_same_id_winners_and_keeps_old_generation_immutable() {
    use crate::game::data::{content_pack::*, skill_data::RawSkillDatabase};

    fn install(
        fixture: &ExternalFixture,
        pack_id: &str,
        amount: i32,
    ) -> (DefinitionKey, crate::game::ability::authoring::EffectSite) {
        let raw =
            RawSkillDatabase::from_ron("fixture", &fixture.read("official/data/skills/base.ron"))
                .unwrap();
        let mut document = serde_json::to_value(&raw).unwrap();
        let skill_id = document["skills"][0]["id"].as_str().unwrap().to_owned();
        let action_id = document["skills"][0]["steps"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let effect_id = document["skills"][0]["steps"][0]["effects"][0]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            document["skills"][0]["steps"][0]["effects"][0]["op"]["Damage"]["amount"],
            60
        );
        document["skills"][0]["steps"][0]["effects"][0]["op"]["Damage"]["amount"] = amount.into();
        document["skills"].as_array_mut().unwrap().truncate(1);
        document["range_presets"] = serde_json::json!([]);
        let edited: RawSkillDatabase = serde_json::from_value(document).unwrap();
        let target = DefinitionKey {
            kind: DefinitionKind::Skill,
            id: skill_id.clone(),
        };
        let site = crate::game::ability::authoring::EffectSite {
            action: crate::game::ability::authoring::ActionSite {
                definition_id: skill_id.into(),
                action_id,
            },
            effect_id,
        };
        fixture.write(
            &format!("{pack_id}/skills.ron"),
            ron::ser::to_string(&edited).unwrap(),
        );
        fixture.manifest(
            pack_id,
            PackManifest {
                pack_id: pack_id.into(),
                pack_version: "1".into(),
                namespace: PackNamespace::Global,
                contract: ContentContract::current(),
                required_capabilities: vec![],
                dependencies: vec!["official".into()],
                files: vec![CatalogFile {
                    kind: CatalogKind::Skills,
                    path: "skills.ron".into(),
                }],
                operations: vec![DefinitionOperation {
                    target: target.clone(),
                    operation: OperationKind::Replace,
                    after: vec!["official".into()],
                }],
            },
        );
        (target, site)
    }

    let fixture = ExternalFixture::official();
    let (target, site) = install(&fixture, "explain_override_a", 61);
    install(&fixture, "explain_override_b", 62);
    let selection = |winner: &str| crate::game::data::content_pack::ContentSelection {
        packs: vec![
            "official".into(),
            "explain_override_a".into(),
            "explain_override_b".into(),
        ],
        winners: vec![WriterSelection {
            target: target.clone(),
            pack_id: winner.into(),
        }],
    };
    let load = |winner: &str| {
        validate(
            fixture.root(),
            &selection(winner),
            &AuthoringLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap()
    };
    let old = load("explain_override_a");
    let query = ExplainQuery::Effect { site: site.clone() };
    let old_document = explain(
        &old,
        &query,
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap()
    .into_document();
    let new = load("explain_override_b");
    let new_document = explain(
        &new,
        &query,
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap()
    .into_document();
    let old_value: serde_json::Value = serde_json::from_str(&old_document).unwrap();
    let new_value: serde_json::Value = serde_json::from_str(&new_document).unwrap();
    assert_eq!(
        old_value["definition_source"]["pack_id"],
        "explain_override_a"
    );
    assert_eq!(
        new_value["definition_source"]["pack_id"],
        "explain_override_b"
    );
    assert_eq!(
        old_value["value_expression"]["source"]["source_file"],
        "explain_override_a/skills.ron"
    );
    assert_eq!(
        new_value["value_expression"]["source"]["source_file"],
        "explain_override_b/skills.ron"
    );
    assert_ne!(
        old_value["value_expression"]["semantic_fingerprint"],
        new_value["value_expression"]["semantic_fingerprint"]
    );
    assert_ne!(old.token().identity(), new.token().identity());
    assert_eq!(
        explain(
            &old,
            &query,
            &ExecutionLimits::default(),
            &CancellationToken::default(),
        )
        .unwrap()
        .document(),
        old_document
    );
}

#[test]
fn effect_explain_limits_cancellation_and_determinism_cover_the_compiled_descriptor() {
    let fixture = ExternalFixture::official();
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let (site, entry) = content
        .token()
        .value_expr_catalog()
        .entries()
        .next()
        .unwrap();
    let query = ExplainQuery::Effect { site: site.clone() };
    let fingerprint = entry.plan().semantic_fingerprint();
    let identity = content.token().identity().clone();
    let first = explain(
        &content,
        &query,
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let repeated = explain(
        &content,
        &query,
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    assert_eq!(first.document(), repeated.document());
    assert_eq!(first.usage(), repeated.usage());

    for (limits, code) in [
        (
            ExecutionLimits {
                max_work: first.usage().work - 1,
                ..Default::default()
            },
            "authoring_work_limit",
        ),
        (
            ExecutionLimits {
                max_metadata_bytes: first.usage().metadata_bytes_peak - 1,
                ..Default::default()
            },
            "authoring_metadata_limit",
        ),
        (
            ExecutionLimits {
                max_result_bytes: first.document().len() - 1,
                ..Default::default()
            },
            "authoring_result_limit",
        ),
    ] {
        let Err(AuthoringError::Execution { diagnostic }) =
            explain(&content, &query, &limits, &CancellationToken::default())
        else {
            panic!("low {code} limit returned success");
        };
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.phase, ExecutionPhase::Inspection);
    }
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(
        matches!(explain(&content, &query, &ExecutionLimits::default(), &cancelled), Err(AuthoringError::Execution { diagnostic }) if diagnostic.code == "authoring_cancelled")
    );
    assert_eq!(content.token().identity(), &identity);
    assert_eq!(
        content
            .token()
            .value_expr_catalog()
            .get(site)
            .unwrap()
            .plan()
            .semantic_fingerprint(),
        fingerprint
    );
}

#[test]
fn authored_amount_publication_accepts_supported_and_rejects_unsupported_members() {
    let fixture = ExternalFixture::official();
    let path = "official/data/skills/base.ron";
    let source = fixture.read(path);
    let changed = source.replacen(
        "Damage(amount: 60, damage_type: Magic)",
        "DamageExpr(amount: (moment: Snapshot, root: Const((value: 60, unit: Magnitude))), damage_type: Magic)",
        1,
    );
    assert_ne!(source, changed);
    fixture.write(path, changed);
    attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let changed = fixture.read(path).replacen(
        "root: Const((value: 60, unit: Magnitude))",
        "root: Aggregate(source: ResolvedTargets, grouping: ByOwner, reduction: Sum, outer_reduction: Some(Sum), member: Read(Stat(subject: AggregateMember, stat: Attack)))", 1);
    fixture.write(path, changed);
    let Err(AuthoringError::Content { diagnostic }) =
        attempt(&fixture, &AuthoringLimits::default())
    else {
        panic!("unsupported implicit owner was published");
    };
    assert_eq!(diagnostic.code, "content_capability_unsupported");
}

#[test]
fn explain_preserves_actual_definition_effect_origin_and_rejects_missing_sites() {
    use crate::game::{
        ability::authoring::{ActionSite, EffectSite},
        data::content_pack::{DefinitionKey, DefinitionKind},
    };
    let fixture = ExternalFixture::official();
    use crate::game::data::skill_data::RawSkillDatabase;
    let raw = RawSkillDatabase::from_ron("fixture", &fixture.read("official/data/skills/base.ron"))
        .unwrap();
    let skill_id = raw.skills[0].id.clone();
    let origin_id = raw.skills[1].id.clone();
    let mut authored = serde_json::to_value(&raw).unwrap();
    authored["skills"][0]["steps"][0]["effects"][0]["origin"] =
        serde_json::json!({"Added": {"source_definition_id": origin_id}});
    let edited: RawSkillDatabase = serde_json::from_value(authored).unwrap();
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&edited).unwrap(),
    );
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let cancellation = CancellationToken::default();
    let limits = ExecutionLimits::default();
    let skill = content
        .token()
        .skill_data
        .get_by_id(skill_id.as_str())
        .unwrap();
    let step = skill
        .steps
        .iter()
        .find(|step| !step.effects.is_empty())
        .unwrap();
    let effect = &step.effects[0];
    let site = EffectSite {
        action: ActionSite {
            definition_id: skill.id.clone(),
            action_id: step.id.clone(),
        },
        effect_id: effect.id.clone(),
    };
    let key = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: skill.id.as_str().to_owned(),
    };
    let definition = explain(
        &content,
        &ExplainQuery::Definition { key: key.clone() },
        &limits,
        &cancellation,
    )
    .unwrap();
    let defined: serde_json::Value = serde_json::from_str(definition.document()).unwrap();
    assert_eq!(defined["present"], true);
    assert_eq!(
        defined["source"],
        serde_json::to_value(content.token().source_metadata().unwrap().source_for(&key)).unwrap()
    );
    assert_eq!(defined["writers"].as_array().unwrap().len(), 1);
    let query = ExplainQuery::Effect { site: site.clone() };
    let explained = explain(&content, &query, &limits, &cancellation).unwrap();
    let value: serde_json::Value = serde_json::from_str(explained.document()).unwrap();
    assert_eq!(value["site"], serde_json::to_value(&site).unwrap());
    assert_eq!(value["effect"], serde_json::to_value(effect).unwrap());
    assert_eq!(
        value["source"],
        serde_json::to_value(effect.source()).unwrap()
    );
    let origin_key = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: origin_id.as_str().to_owned(),
    };
    assert_eq!(
        value["origin_definition_source"],
        serde_json::to_value(
            content
                .token()
                .source_metadata()
                .unwrap()
                .source_for(&origin_key)
        )
        .unwrap()
    );
    assert!(
        matches!(&effect.origin, crate::game::ability::authoring::EffectOrigin::Added { source_definition_id } if source_definition_id == &origin_id)
    );
    assert_eq!(
        value["editable_support"],
        serde_json::to_value(crate::game::ability::authoring::effect_support(
            &effect.op,
            &step.delivery
        ))
        .unwrap()
    );
    fixture.write(
        "official/data/skills/base.ron",
        "invalid source after publication",
    );
    assert_eq!(
        explain(&content, &query, &limits, &cancellation)
            .unwrap()
            .document(),
        explained.document()
    );
    let mut missing = site;
    missing.effect_id = "missing_effect".to_owned();
    assert!(
        matches!(explain(&content, &ExplainQuery::Effect { site: missing }, &limits, &cancellation), Err(AuthoringError::Request { diagnostic }) if diagnostic.code == "authoring_effect_not_found")
    );
    assert!(serde_json::from_str::<ExplainQuery>(r#"{"query":"future_forecast"}"#).is_err());
    assert!(serde_json::from_str::<ExplainQuery>(
        r#"{"query":"definition","key":{"kind":"Skill","id":"x"},"extra":true}"#
    )
    .is_err());
}

#[test]
fn explain_reads_every_published_definition_and_keeps_compiled_graph_edges_as_ids() {
    use crate::game::data::content_pack::DefinitionKind;
    let fixture = ExternalFixture::official();
    // The official resource catalog is intentionally empty. Publish an explicit
    // external fixture definition so this test still queries every supported kind.
    fixture.write("official/data/resources/base.ron", "(resources: [(id: \"query_resource\", owner: Employee, lifetime: Run, initial: 2, cap: 3)])");
    // Relics also have no invented official balance; publish an external definition.
    let relic_input = r#"(relics: [(id: "query_relic",
        uuid: "a508e200-0000-0000-0000-000000000001",
        name: "Query relic", description: "External inspection fixture",
        rarity: "probe", allow_duplicates: false,
        effects: [(id: "normal_damage", selector: (), flat: 3, percent: 4)],
        corruption: Some((effects: [(id: "corrupt_damage", selector: (), flat: 5, percent: 6)],
            penalties: [DefenseFlat(amount: -2)])),
        dismantle_equipment_dust: 7)])"#;
    fixture.write("official/data/relics/base.ron", relic_input);
    let expected_relic: crate::game::data::relic_data::RelicDatabase =
        ron::de::from_str(relic_input).unwrap();
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    let mut kinds = std::collections::BTreeSet::new();
    let limits = ExecutionLimits::default();
    for (key, source) in content.token().source_metadata().unwrap().definitions() {
        let result = explain(
            &content,
            &ExplainQuery::Definition { key: key.clone() },
            &limits,
            &CancellationToken::default(),
        )
        .unwrap_or_else(|error| panic!("{key:?}: {error}"));
        let value: serde_json::Value = serde_json::from_str(result.document()).unwrap();
        assert_eq!(value["key"], serde_json::to_value(key).unwrap());
        assert_eq!(value["source"], serde_json::to_value(source).unwrap());
        assert_eq!(value["present"], true);
        assert!(!value["value"].is_null(), "{key:?}");
        assert_eq!(result.usage().result_bytes_peak, result.document().len());
        kinds.insert(key.kind);
        if key.kind == DefinitionKind::Relic {
            assert_eq!(key.id, "query_relic");
            assert_eq!(source.pack_id, "official");
            assert_eq!(source.source_file, "official/data/relics/base.ron");
            assert_eq!(
                value["value"],
                serde_json::to_value(&expected_relic.relics[0]).unwrap()
            );
        }
        if matches!(
            key.kind,
            DefinitionKind::BossPattern
                | DefinitionKind::BossActorBehavior
                | DefinitionKind::BossEncounter
        ) {
            assert_eq!(value["representation"], "compiled_graph");
            assert_eq!(value["index_space"], "compiled");
            let graph = explain(
                &content,
                &ExplainQuery::CompiledGraph { key: key.clone() },
                &limits,
                &CancellationToken::default(),
            )
            .unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(graph.document()).unwrap(),
                value
            );
        }
        if key.kind == DefinitionKind::BossActorBehavior {
            for node in value["value"]["nodes"].as_array().unwrap() {
                if let Some(call) = node.get("PatternCall") {
                    for candidate in call["candidates"].as_array().unwrap() {
                        assert!(candidate.get("pattern").is_none());
                        let id = crate::game::data::boss_runtime_data::BossPatternId::new(
                            candidate["pattern_id"].as_str().unwrap(),
                        );
                        assert!(content.token().boss_runtime_data.get_pattern(&id).is_some());
                    }
                }
            }
        }
        if key.kind == DefinitionKind::BossEncounter {
            for phase in value["value"]["phases"].as_array().unwrap() {
                for binding in phase["actor_behaviors"].as_array().unwrap() {
                    assert!(binding.get("behavior").is_none());
                    let id = crate::game::data::boss_runtime_data::BossActorBehaviorId::new(
                        binding["behavior_id"].as_str().unwrap(),
                    );
                    assert!(content
                        .token()
                        .boss_runtime_data
                        .get_actor_behavior(&id)
                        .is_some());
                }
            }
        }
    }
    println!("actual_definition_query_kinds={kinds:?}");
    assert_eq!(kinds, DefinitionKind::ALL.iter().copied().collect());
    println!(
        "actual_definition_query_count={}",
        content
            .token()
            .source_metadata()
            .unwrap()
            .definitions()
            .len()
    );
    assert!(kinds.contains(&DefinitionKind::Skill));
    assert!(kinds.contains(&DefinitionKind::BossActorBehavior));
    assert!(kinds.contains(&DefinitionKind::BossEncounter));
    assert!(kinds.contains(&DefinitionKind::SummonedUnit));
    let key = content
        .token()
        .source_metadata()
        .unwrap()
        .definitions()
        .iter()
        .find(|(key, _)| key.kind == DefinitionKind::Skill)
        .unwrap()
        .0
        .clone();
    let graph = explain(
        &content,
        &ExplainQuery::CompiledGraph { key: key.clone() },
        &limits,
        &CancellationToken::default(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(graph.document()).unwrap();
    assert_eq!(value["representation"], "compiled_graph");
    assert_eq!(value["index_space"], "compiled");
    assert_eq!(
        value["value"],
        serde_json::to_value(content.token().skill_data.compiled_flow(&key.id).unwrap()).unwrap()
    );
}

#[test]
fn explain_selected_remove_has_no_resolved_body_and_unknown_is_an_error() {
    use crate::game::data::{content_pack::*, skill_data::RawSkillDatabase};
    let fixture = ExternalFixture::official();
    let raw = RawSkillDatabase::from_ron("fixture", &fixture.read("official/data/skills/base.ron"))
        .unwrap();
    let mut extra = raw.skills[0].clone();
    extra.id = "explain_removed_skill".into();
    let key = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: extra.id.as_str().to_owned(),
    };
    use crate::game::data::boss_runtime_data::BossRuntimeCatalogData;
    let boss: BossRuntimeCatalogData =
        ron::de::from_str(&fixture.read("official/data/boss_runtime/base.ron")).unwrap();
    let mut pattern = boss.patterns[0].clone();
    pattern.id =
        crate::game::data::boss_runtime_data::BossPatternId::new("explain_removed_pattern");
    let pattern_key = DefinitionKey {
        kind: DefinitionKind::BossPattern,
        id: pattern.id.as_str().to_owned(),
    };
    fixture.write(
        "addition/boss.ron",
        ron::ser::to_string(&BossRuntimeCatalogData {
            patterns: vec![pattern],
            ..Default::default()
        })
        .unwrap(),
    );
    fixture.write(
        "addition/skills.ron",
        ron::ser::to_string(&RawSkillDatabase {
            range_presets: vec![],
            skills: vec![extra],
        })
        .unwrap(),
    );
    fixture.manifest(
        "addition",
        PackManifest {
            pack_id: "addition".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["official".into()],
            files: vec![
                CatalogFile {
                    kind: CatalogKind::Skills,
                    path: "skills.ron".into(),
                },
                CatalogFile {
                    kind: CatalogKind::BossRuntime,
                    path: "boss.ron".into(),
                },
            ],
            operations: vec![],
        },
    );
    fixture.manifest(
        "removal",
        PackManifest {
            pack_id: "removal".into(),
            pack_version: "1".into(),
            namespace: PackNamespace::Global,
            contract: ContentContract::current(),
            required_capabilities: vec![],
            dependencies: vec!["addition".into()],
            files: vec![],
            operations: vec![
                DefinitionOperation {
                    target: key.clone(),
                    operation: OperationKind::Remove,
                    after: vec!["addition".into()],
                },
                DefinitionOperation {
                    target: pattern_key.clone(),
                    operation: OperationKind::Remove,
                    after: vec!["addition".into()],
                },
            ],
        },
    );
    let content = validate(
        fixture.root(),
        &ContentSelection {
            packs: vec!["official".into(), "addition".into(), "removal".into()],
            winners: vec![],
        },
        &AuthoringLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let result = explain(
        &content,
        &ExplainQuery::Definition { key: key.clone() },
        &ExecutionLimits::default(),
        &CancellationToken::default(),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_str(result.document()).unwrap();
    assert_eq!(value["present"], false);
    assert_eq!(value["representation"], "removed");
    assert!(value["source"].is_null());
    assert!(value["value"].is_null());
    assert_eq!(value["writers"].as_array().unwrap().len(), 2);
    assert!(value["writers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v["pack_id"] == "removal" && v["selected"] == true));
    assert!(
        matches!(explain(&content, &ExplainQuery::CompiledGraph { key: pattern_key }, &ExecutionLimits::default(), &CancellationToken::default()), Err(AuthoringError::Request { diagnostic }) if diagnostic.code == "authoring_definition_not_found")
    );
    let missing = DefinitionKey {
        kind: DefinitionKind::Skill,
        id: "unknown_definition".into(),
    };
    assert!(
        matches!(explain(&content, &ExplainQuery::Definition { key: missing }, &ExecutionLimits::default(), &CancellationToken::default()), Err(AuthoringError::Request { diagnostic }) if diagnostic.code == "authoring_definition_not_found")
    );
}

#[test]
fn explain_prices_long_action_and_effect_comparisons_before_searching() {
    use crate::game::{
        ability::authoring::{ActionSite, EffectSite},
        data::skill_data::RawSkillDatabase,
    };
    let fixture = ExternalFixture::official();
    let raw = RawSkillDatabase::from_ron("fixture", &fixture.read("official/data/skills/base.ron"))
        .unwrap();
    let mut document = serde_json::to_value(&raw).unwrap();
    let mut skill = document["skills"][0].clone();
    skill["id"] = "query_long_ids".into();
    let action_prefix = "a".repeat(510);
    let effect_prefix = "e".repeat(510);
    let mut step = skill["steps"][0].clone();
    let effect = step["effects"][0].clone();
    step["effects"] = (0..32)
        .map(|index| {
            let mut value = effect.clone();
            value["id"] = format!("{effect_prefix}{index:02}").into();
            value
        })
        .collect::<Vec<_>>()
        .into();
    skill["steps"] = (0..32)
        .map(|index| {
            let mut value = step.clone();
            value["id"] = format!("{action_prefix}{index:02}").into();
            value
        })
        .collect::<Vec<_>>()
        .into();
    document["skills"].as_array_mut().unwrap().push(skill);
    let edited: RawSkillDatabase = serde_json::from_value(document).unwrap();
    fixture.write(
        "official/data/skills/base.ron",
        ron::ser::to_string(&edited).unwrap(),
    );
    let content = attempt(&fixture, &AuthoringLimits::default()).unwrap();
    for (action, max_work, site) in [
        (format!("{action_prefix}zz"), 15_000, "explain_actions"),
        (format!("{action_prefix}00"), 30_000, "explain_effects"),
    ] {
        let query = ExplainQuery::Effect {
            site: EffectSite {
                action: ActionSite {
                    definition_id: "query_long_ids".into(),
                    action_id: action,
                },
                effect_id: format!("{effect_prefix}zz"),
            },
        };
        let Err(AuthoringError::Execution { diagnostic }) = explain(
            &content,
            &query,
            &ExecutionLimits {
                max_work,
                ..Default::default()
            },
            &CancellationToken::default(),
        ) else {
            panic!("long comparison search was not bounded");
        };
        assert_eq!(diagnostic.code, "authoring_work_limit");
        assert_eq!(diagnostic.site, site);
        assert_eq!(diagnostic.phase, ExecutionPhase::Inspection);
    }
}
