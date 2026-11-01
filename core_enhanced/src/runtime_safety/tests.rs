use super::*;

fn text() -> String {
    String::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON.to_vec()).unwrap()
}
fn parse(s: &str) -> Result<RuntimeSafetySettings, RuntimeSafetyError> {
    RuntimeSafetySettings::from_ron("/host/settings.ron", s.as_bytes())
}

fn legacy_eight_mib_text() -> String {
    let source = text();
    assert!(source.contains("max_retained_bytes: 16777216"));
    source.replacen(
        "max_retained_bytes: 16777216",
        "max_retained_bytes: 8388608",
        1,
    )
}

pub(crate) fn cmrp_profile_text(emergency: usize) -> String {
    let mut source = legacy_eight_mib_text();
    let end = source.rfind(')').unwrap();
    source.insert_str(end, &format!("development_measurement: Some((revision: 1, emergency_retained_bytes: {emergency})),\n"));
    source
}

#[test]
fn cmrp_default_and_legacy_eight_mib_canonical_identities_are_distinct() {
    let settings = parse(&text()).unwrap();
    let legacy = parse(&legacy_eight_mib_text()).unwrap();
    assert_eq!(legacy.native_runtime().max_retained_bytes, 8 * 1024 * 1024);
    assert_eq!(
        legacy.effective_sha256(),
        "a7b932aa614c11c8adf2b0e43c35c89f841056d877b78800e935d4d0c61f4a49"
    );
    assert_eq!(
        settings.native_runtime().max_retained_bytes,
        16 * 1024 * 1024
    );
    assert_ne!(settings.effective_sha256(), legacy.effective_sha256());
    assert!(settings.development_measurement().is_none());
    assert_eq!(settings.native_runtime(), RuntimeFactLimits::ENGINE_MAX);
    let mut explicit_none = text();
    let end = explicit_none.rfind(')').unwrap();
    explicit_none.insert_str(end, "development_measurement: None,\n");
    let selected = parse(&explicit_none).unwrap();
    assert_eq!(selected.effective_sha256(), settings.effective_sha256());
    assert_ne!(selected.raw_sha256(), settings.raw_sha256());
}

#[test]
fn cmrp_trusted_profile_selects_supported_measurement_caps() {
    for cap in [16 * 1024 * 1024, 32 * 1024 * 1024, 64 * 1024 * 1024] {
        let settings = parse(&cmrp_profile_text(cap)).unwrap();
        assert_eq!(
            settings.nominal_native_runtime(),
            RuntimeFactLimits {
                max_retained_bytes: 8 * 1024 * 1024,
                ..RuntimeFactLimits::ENGINE_MAX
            }
        );
        let mut effective = settings.native_runtime();
        assert_eq!(effective.max_retained_bytes, cap);
        effective.max_retained_bytes = RuntimeFactLimits::ENGINE_MAX.max_retained_bytes;
        assert_eq!(effective, RuntimeFactLimits::ENGINE_MAX);
        let profile = settings.development_measurement().unwrap();
        assert_eq!(profile.revision(), 1);
        assert_eq!(profile.emergency_retained_bytes(), cap);
        assert_ne!(
            settings.effective_sha256(),
            RuntimeSafetySettings::embedded().effective_sha256()
        );
        assert_eq!(
            settings.native_runtime().validate().is_ok(),
            cap == 16 * 1024 * 1024,
            "raw public limits stop at the new sixteen MiB ceiling"
        );
    }
    let raw_over = RuntimeFactLimits {
        max_retained_bytes: 16 * 1024 * 1024 + 1,
        ..RuntimeFactLimits::ENGINE_MAX
    };
    assert!(raw_over.validate().is_err());
}

#[test]
fn cmrp_profile_rejects_revision_unknown_duplicate_negative_overflow_and_cap() {
    let valid = cmrp_profile_text(16 * 1024 * 1024);
    let invalid = [
        valid.replace("revision: 1, emergency", "revision: 2, emergency"),
        valid.replace("revision: 1, emergency", "revision: -1, emergency"),
        valid.replace(
            "revision: 1, emergency",
            "revision: 1, surprise: 1, emergency",
        ),
        valid.replace(
            "revision: 1, emergency",
            "revision: 1, revision: 1, emergency",
        ),
        valid.replace(
            "emergency_retained_bytes: 16777216",
            "emergency_retained_bytes: -1",
        ),
        valid.replace(
            "emergency_retained_bytes: 16777216",
            "emergency_retained_bytes: 18446744073709551616",
        ),
        valid.replace(
            "emergency_retained_bytes: 16777216",
            "emergency_retained_bytes: 0",
        ),
        valid.replace(
            "emergency_retained_bytes: 16777216",
            "emergency_retained_bytes: 8388608",
        ),
        valid.replace(
            "emergency_retained_bytes: 16777216",
            "emergency_retained_bytes: 67108865",
        ),
        valid.replace(
            "max_retained_bytes: 8388608",
            "max_retained_bytes: 16777217",
        ),
    ];
    for source in invalid {
        assert!(parse(&source).is_err(), "{source}");
    }
    let equal_to_default = cmrp_profile_text(16 * 1024 * 1024).replacen(
        "max_retained_bytes: 8388608",
        "max_retained_bytes: 16777216",
        1,
    );
    assert!(
        parse(&equal_to_default).is_err(),
        "development cap must exceed its nominal profile"
    );
}

#[test]
fn original_defaults_and_no_file_profile_have_one_effective_identity() {
    let settings = parse(&text()).unwrap();
    let shared = RuntimeSafetySettings::embedded_shared();
    assert_eq!(settings.effective_sha256(), shared.effective_sha256());
    assert_eq!(RuntimeFactLimits::default(), RuntimeFactLimits::ENGINE_MAX);
    assert_eq!(ExprParseLimits::default(), ExprParseLimits::HARD_MAX);
    assert_eq!(settings.native_runtime().max_work, 10_000_000);
    assert_eq!(settings.authoring_execution().max_clone_slots, 65_536);
    assert_eq!(settings.expression().max_compile_work, 65_536);
    let trial = crate::authoring::TrialLimits::default();
    assert_eq!(trial.max_total_work, 1 << 30);
    assert_eq!(trial.max_receipt_bytes, 64 * 1024 * 1024);
    assert_eq!(settings.battle_history().max_events, 8192);
    assert_eq!(
        settings.battle_history().max_encoded_entry_array_bytes,
        4 * 1024 * 1024
    );
    assert_eq!(settings.battle_history().max_span_ms, 180_000);
}

#[test]
fn raw_and_effective_identity_are_distinct_and_profiles_are_immutable() {
    let original = parse(&text()).unwrap();
    let comments = parse(&format!("/* comment */\n{}\n// whitespace\n", text())).unwrap();
    assert_ne!(original.raw_sha256(), comments.raw_sha256());
    assert_eq!(original.effective_sha256(), comments.effective_sha256());
    let smaller = parse(&text().replacen("max_work: 10000000", "max_work: 99", 1)).unwrap();
    assert_eq!(smaller.native_runtime().max_work, 99);
    assert_ne!(original.effective_sha256(), smaller.effective_sha256());
    assert_eq!(original.native_runtime().max_work, 10_000_000);
    assert_eq!(RuntimeFactLimits::default().max_work, 10_000_000);
}

#[test]
fn settings_reject_schema_unknown_duplicate_negative_overflow_zero_and_ceilings() {
    for (from, to, field) in [
        ("schema_version: 1", "schema_version: 2", "schema_version"),
        ("schema_version: 1", "schema_version: -1", "schema_version"),
        (
            "max_retained_bytes: 16777216",
            "max_retained_bytes: 16777217",
            "native_runtime.max_retained_bytes",
        ),
        (
            "max_work: 10000000",
            "max_work: 0",
            "native_runtime.max_work",
        ),
        (
            "max_work: 10000000",
            "max_work: 10000001",
            "native_runtime.max_work",
        ),
        (
            "max_work: 10000000",
            "max_work: 18446744073709551616",
            "native_runtime.max_work",
        ),
        (
            "max_depth: 64",
            "max_depth: 65",
            "expression.parse.max_depth",
        ),
        (
            "max_total_work: 1073741824",
            "max_total_work: 274877906945",
            "authoring_trial.max_total_work",
        ),
        (
            "max_events: 8192",
            "max_events: 8193",
            "battle_history.max_events",
        ),
    ] {
        let error = parse(&text().replacen(from, to, 1)).unwrap_err();
        assert_eq!(error.source, "/host/settings.ron");
        assert!(error.field.contains(field), "{from} → {to}: {error:?}");
    }
    for extra in ["extra: 1,", "schema_version: 1,"] {
        let input = text().replacen(
            "schema_version: 1,",
            &format!("schema_version: 1, {extra}"),
            1,
        );
        let error = parse(&input).unwrap_err();
        assert!(!error.message.is_empty());
    }
    assert!(parse(&format!("{} ()", text())).is_err());
}

#[test]
fn parser_is_bounded_before_deserialization_including_nested_comments() {
    let mut at_limit = text();
    at_limit.push_str(&" ".repeat(MAX_SETTINGS_BYTES - at_limit.len()));
    parse(&at_limit).unwrap();
    at_limit.push(' ');
    assert!(parse(&at_limit).unwrap_err().message.contains("64KiB"));
    for input in ["(".repeat(1000), "/*".repeat(1000)] {
        assert!(parse(&input).unwrap_err().message.contains("nesting"));
    }
    // Brackets in comments are not configuration nesting.
    parse(&format!("// {}\n{}", "(".repeat(100), text())).unwrap();
}

#[test]
fn structural_errors_identify_the_offending_field_at_each_level() {
    for (from, to, expected) in [
        (
            "schema_version: 1,",
            "extra: 1, schema_version: 1,",
            "extra",
        ),
        (
            "schema_version: 1,",
            "schema_version: 1, schema_version: 1,",
            "schema_version",
        ),
        (
            "max_active_actions: 4096,",
            "",
            "native_runtime.max_active_actions",
        ),
        (
            "max_active_actions: 4096,",
            "surprise: 4,",
            "native_runtime.surprise",
        ),
        (
            "max_active_actions: 4096,",
            "max_active_actions: 4096, max_active_actions: 4,",
            "native_runtime.max_active_actions",
        ),
        (
            "max_input_bytes: 65536,",
            "",
            "expression.parse.max_input_bytes",
        ),
        (
            "max_total_work: 1073741824",
            "unknown: 1",
            "authoring_trial.unknown",
        ),
    ] {
        assert_eq!(
            parse(&text().replacen(from, to, 1)).unwrap_err().field,
            expected
        );
    }
}

#[cfg(unix)]
#[test]
fn explicit_authoring_admission_and_trial_limits_consume_the_profile() {
    use crate::{
        authoring::{self, AuthoringError, CancellationToken, TrialLimits},
        game::data::content_pack::{tests::ExternalFixture, InputLimits},
    };
    let fixture = ExternalFixture::official();
    let settings = Arc::new(
        parse(
            &text()
                .replace(
                    "authoring_execution: (\n        max_work: 10000000",
                    "authoring_execution: (\n        max_work: 1",
                )
                .replace("max_total_work: 1073741824", "max_total_work: 123")
                .replace("max_receipt_bytes: 67108864", "max_receipt_bytes: 456"),
        )
        .unwrap(),
    );
    let error = authoring::validate_with_runtime_safety(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
        &CancellationToken::default(),
        settings.clone(),
    )
    .err()
    .unwrap();
    assert!(
        matches!(error, AuthoringError::Execution { diagnostic } if diagnostic.limit == Some(1)),
        "the explicit profile must govern admission work"
    );
    let trial = TrialLimits::with_runtime_safety(&settings);
    assert_eq!(trial.execution.max_work, 1);
    assert_eq!(trial.max_total_work, 123);
    assert_eq!(trial.max_receipt_bytes, 456);
    assert_eq!(TrialLimits::default().execution.max_work, 10_000_000);
}

#[test]
fn compiler_limits_preserve_existing_typed_range_and_parser_ceiling() {
    let input = text().replacen(
        "compile_and_evaluate: (\n            max_depth: 64",
        "compile_and_evaluate: (\n            max_depth: 4096",
        1,
    );
    let settings = parse(&input).unwrap();
    assert_eq!(settings.expression().max_depth, 4096);
    assert_eq!(settings.expression_parse().max_depth, 64);
    settings.expression().validate().unwrap();
}

#[test]
fn history_import_and_deserialize_apply_the_host_window_and_keep_cursor_gaps() {
    use crate::game::battle::{
        event_log::{BattleEventCause, BattleEventLog, BattleEventLogEntry, BattleLogEvent},
        types::BattleWinner,
    };
    let settings = parse(&text().replace("max_events: 8192", "max_events: 2")).unwrap();
    let entries = (1..=5)
        .map(|seq| BattleEventLogEntry {
            time_ms: seq * 10,
            seq,
            cause: BattleEventCause::default(),
            source_command_id: None,
            skill_effect_source: None,
            event: BattleLogEvent::BattleEnd {
                winner: BattleWinner::Draw,
            },
        })
        .collect::<Vec<_>>();
    let imported = BattleEventLog::from_entries_with_runtime_safety(entries.clone(), &settings);
    assert_eq!(imported.entries.len(), 2);
    assert_eq!(imported.floor_seq(), 3);
    assert!(imported.validate_cursor(0).is_err());
    assert!(imported.validate_cursor(3).is_ok());
    let raw = serde_json::json!({"version": 32, "entries": entries}).to_string();
    let mut decoder = serde_json::Deserializer::from_str(&raw);
    let loaded = BattleEventLog::deserialize_with_runtime_safety(&mut decoder, &settings).unwrap();
    assert_eq!(loaded.entries.len(), 2);
    assert_eq!(loaded.floor_seq(), 3);
    assert_eq!(loaded.latest_seq(), 5);
    assert_eq!(
        loaded.encoded_entry_array_bytes(),
        imported.encoded_entry_array_bytes()
    );
    let forged = raw.replacen("{", "{\"limits\":{\"max_events\":999999},", 1);
    assert!(serde_json::from_str::<BattleEventLog>(&forged).is_err());
    let tiny = text().replace(
        "max_encoded_entry_array_bytes: 4194304",
        "max_encoded_entry_array_bytes: 1",
    );
    assert!(parse(&tiny).is_err());
}

#[cfg(unix)]
#[test]
fn external_generation_keeps_semantics_and_separates_profile_admission() {
    use crate::game::data::content_pack::{self, tests::ExternalFixture, InputLimits};
    let fixture = ExternalFixture::official();
    let original = fixture.load();
    let settings = Arc::new(parse(&text().replace("max_events: 8192", "max_events: 2")).unwrap());
    let selected = content_pack::load_with_runtime_safety(
        fixture.root(),
        &fixture.selection(),
        &InputLimits::default(),
        settings.clone(),
    )
    .unwrap();
    assert_eq!(original.identity(), selected.identity());
    assert_eq!(original.source_receipt(), selected.source_receipt());
    assert_ne!(
        original.admission_identity().runtime_safety_sha256,
        selected.admission_identity().runtime_safety_sha256
    );
    assert_eq!(selected.runtime_safety().battle_history().max_events, 2);
    selected
        .runtime_safety()
        .verify_effective_sha256(settings.effective_sha256())
        .unwrap();
    assert!(selected
        .runtime_safety()
        .verify_effective_sha256(original.runtime_safety().effective_sha256())
        .is_err());
    let app = crate::application::CoreApplication::new(selected, 17);
    assert_eq!(
        app.runtime_safety().effective_sha256(),
        settings.effective_sha256()
    );
    assert_eq!(original.runtime_safety().battle_history().max_events, 8192);
}

#[cfg(unix)]
#[test]
fn public_content_parser_compiler_and_immutable_plan_consume_selected_profile() {
    use crate::game::{
        data::content_pack::{self, tests::ExternalFixture, InputLimits},
        value_expr::{evaluate, EvalFrame, ExprMeter},
    };
    let fixture = ExternalFixture::official();
    let skill_file = "official/data/skills/base.ron";
    let skills = fixture.read(skill_file);
    let extra = include_str!("../../tests/fixtures/r08/skills.ron").replace("r08_strike", "profile_expression")
        .replace("Damage(amount: 100, damage_type: True)", "DamageExpr(amount: (moment: Snapshot, root: Add([Const((value: 2, unit: Magnitude)), Const((value: 3, unit: Magnitude))])), damage_type: True)");
    let extra = extra
        .trim()
        .strip_prefix("SkillDatabase(skills: [")
        .unwrap()
        .strip_suffix("])")
        .unwrap();
    fixture.write(
        skill_file,
        skills.replacen("skills: [", &format!("skills: [{extra},"), 1),
    );
    let load = |settings: Arc<RuntimeSafetySettings>| {
        content_pack::load_with_runtime_safety(
            fixture.root(),
            &fixture.selection(),
            &InputLimits::default(),
            settings,
        )
    };
    let selected = Arc::new(parse(&text().replace("max_work: 65536", "max_work: 1024")).unwrap());
    let (original, generation) = std::thread::scope(|scope| {
        let default = scope.spawn(|| load(RuntimeSafetySettings::embedded_shared()).unwrap());
        let custom = scope.spawn(|| load(selected.clone()).unwrap());
        (default.join().unwrap(), custom.join().unwrap())
    });
    let small_parse =
        Arc::new(parse(&text().replacen("max_nodes: 1024", "max_nodes: 1", 1)).unwrap());
    assert!(
        load(small_parse).is_err(),
        "parse admission must consume the profile before publication"
    );
    let small_compile = Arc::new(
        parse(&text().replace(
            "compile_and_evaluate: (\n            max_depth: 64,\n            max_nodes: 1024",
            "compile_and_evaluate: (\n            max_depth: 64,\n            max_nodes: 1",
        ))
        .unwrap(),
    );
    assert!(
        load(small_compile).is_err(),
        "catalog compilation must consume the profile"
    );
    assert_eq!(generation.identity(), original.identity());
    let find = |data: &crate::game::data::ValidatedGameData| {
        data.value_expr_catalog()
            .entries()
            .find(|(site, _)| site.action.definition_id.as_str() == "profile_expression")
            .unwrap()
            .1
            .plan()
            .clone()
    };
    let plan = find(&generation);
    assert_eq!(plan.evaluation_limits(), selected.expression());
    assert_eq!(
        plan.semantic_fingerprint(),
        find(&original).semantic_fingerprint()
    );
    let result = evaluate(
        &plan,
        &EvalFrame::default(),
        &mut ExprMeter::new(selected.expression()).unwrap(),
    )
    .unwrap();
    assert_eq!(result.value.value, 5);
    assert!(
        evaluate(
            &plan,
            &EvalFrame::default(),
            &mut ExprMeter::new(ExprLimits::default()).unwrap()
        )
        .is_err(),
        "larger meter cannot silently reuse smaller-profile plan"
    );
}

#[test]
fn selected_native_limits_are_installed_before_the_first_battle_operation() {
    use crate::game::{
        battle::{
            core::BattleCore,
            runtime_control::RuntimeFaultCode,
            scenario::*,
            types::{BattleUnitDraft, BattleUnitSource, BattleUnitThreatClass, UnitCombatProfile},
        },
        behavior::GameError,
        data::GameDataBuilder,
        enums::{Side, Tier},
        growth::GrowthStack,
        resources::Position,
    };
    let settings =
        Arc::new(parse(&text().replacen("max_work: 10000000", "max_work: 1", 1)).unwrap());
    let database = Arc::new(
        GameDataBuilder::empty()
            .build_with_runtime_safety(settings)
            .unwrap(),
    );
    let mut scenario = BattleScenario::empty((4, 4));
    let id = ScenarioGroupId::new("two_actors");
    scenario.groups.push(ScenarioSpawnGroup {
        id: id.clone(),
        side: Side::Opponent,
        required_for_victory: true,
        enemy_movement_plan: None,
        spawns: (1..=2)
            .map(|n| ScenarioUnitSpawn {
                unit_ref: ScenarioUnitRef::new(format!("actor{n}")),
                side: Side::Opponent,
                draft: BattleUnitDraft {
                    owned_uuid: uuid::Uuid::from_u128(n),
                    source: BattleUnitSource::TestFixture {
                        base_uuid: uuid::Uuid::from_u128(n + 10),
                        profile: UnitCombatProfile::employee_default(),
                    },
                    threat_class: BattleUnitThreatClass::Normal,
                    level: Tier::I,
                    stat_scale: Default::default(),
                    growth_stacks: GrowthStack::new(),
                    equipment_layout: None,
                    equipped_items: vec![],
                    equipped_item_enhancements: vec![],
                    employee_start_condition: None,
                },
                position: Position::new(n as i32, 1),
                instance_salt: 0,
            })
            .collect(),
    });
    scenario.events.push(ScenarioEvent {
        id: ScenarioEventId::new("start"),
        trigger: ScenarioTrigger::AtTimeMs(0),
        action: ScenarioAction::SpawnGroup { group_id: id },
    });
    let mut core = BattleCore::new_from_scenario(scenario.clone(), database, 17);
    assert_eq!(core.runtime_fact_limits().max_work, 1);
    let failure = core.start_battle_execution().unwrap_err();
    assert!(
        matches!(failure, GameError::BattleRuntimeFault(f) if f.code == RuntimeFaultCode::WorkLimit),
        "{failure:?}"
    );
    let mut normal =
        BattleCore::new_from_scenario(scenario.clone(), GameDataBuilder::empty().build_arc(), 17);
    let mut execution = normal.start_battle_execution().unwrap();
    normal.step_battle_execution(&mut execution).unwrap();
    assert!(normal.event_log.entries.len() > 1);
    let profile = Arc::new(parse(&text().replace("max_events: 8192", "max_events: 1")).unwrap());
    let data = Arc::new(
        GameDataBuilder::empty()
            .build_with_runtime_safety(profile)
            .unwrap(),
    );
    let mut bounded = BattleCore::new_from_scenario(scenario, data, 17);
    let mut execution = bounded.start_battle_execution().unwrap();
    bounded.step_battle_execution(&mut execution).unwrap();
    assert_eq!(bounded.event_log.entries.len(), 1);
    assert!(bounded.event_log.floor_seq() > 0);
    assert_eq!(
        bounded.event_log.latest_seq(),
        normal.event_log.latest_seq()
    );
    assert_eq!(
        serde_json::to_value(bounded.runtime_fact_snapshot().unwrap()).unwrap(),
        serde_json::to_value(normal.runtime_fact_snapshot().unwrap()).unwrap()
    );
}

#[test]
fn a_short_history_span_does_not_end_battle_or_change_final_statistics() {
    use crate::game::{
        battle::{core::BattleCore, scenario::BattleScenario},
        data::GameDataBuilder,
    };
    let small = Arc::new(
        parse(
            &text()
                .replace("max_events: 8192", "max_events: 1")
                .replace("max_span_ms: 180000", "max_span_ms: 1"),
        )
        .unwrap(),
    );
    let run = |settings: Arc<RuntimeSafetySettings>| {
        let data = Arc::new(
            GameDataBuilder::empty()
                .build_with_runtime_safety(settings)
                .unwrap(),
        );
        let mut scenario = BattleScenario::empty((4, 4));
        scenario.terminal_outcomes = BattleScenario::fixture_time_victory_outcomes(1000);
        let mut battle = BattleCore::new_from_scenario(scenario, data, 17);
        let mut execution = battle.start_battle_execution().unwrap();
        battle
            .step_battle_execution_until(&mut execution, 500)
            .unwrap();
        assert!(!execution.is_finished());
        battle
            .step_battle_execution_until(&mut execution, 1000)
            .unwrap();
        assert!(execution.is_finished());
        let result = battle.finalize_battle_execution(&mut execution).unwrap();
        (battle.event_log, result)
    };
    let (full_log, full) = run(RuntimeSafetySettings::embedded_shared());
    let (small_log, bounded) = run(small);
    assert_eq!(full.result_stats, bounded.result_stats);
    assert_eq!(bounded.result_stats.battle.duration_ms, 1000);
    assert_eq!(full.winner, bounded.winner);
    assert_eq!(full_log.latest_seq(), small_log.latest_seq());
    assert_eq!(small_log.entries.len(), 1);
    assert!(small_log.floor_seq() > 0);
    assert!(small_log.validate_cursor(0).is_err());
    assert!(small_log.retained_time_span_ms() <= 1);
}
