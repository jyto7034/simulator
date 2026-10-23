use super::*;
use crate::game::{
    behavior::{GameStateContextDto, RunSnapshotDto},
    data::ValidatedGameData,
    map::{MapNodeCategory, MapNodeState},
};

fn command(value: Value) -> ApplicationCommand {
    ApplicationCommand::from_document(value.into()).unwrap()
}
fn execute(app: &mut CoreApplication, value: Value) -> Result<ApplicationResult, ApplicationError> {
    app.execute(Uuid::from_u128(0x5203), command(value), Default::default())
}
fn waiting(max_work: u64) -> CoreApplication {
    let text = std::str::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
        .unwrap()
        .replacen("max_work: 10000000", &format!("max_work: {max_work}"), 1);
    let settings = Arc::new(
        crate::runtime_safety::RuntimeSafetySettings::from_ron("fault-test", text.as_bytes())
            .unwrap(),
    );
    let data = ValidatedGameData::load_live_embedded_with_runtime_safety(settings).unwrap();
    waiting_with_data(data)
}
fn waiting_with_data(data: ValidatedGameData) -> CoreApplication {
    let mut app = CoreApplication::new(data, 23);
    execute(
        &mut app,
        json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":false}),
    )
    .unwrap();
    for _ in 0..128 {
        let view: RunSnapshotDto = serde_json::from_value(
            app.snapshot(ApplicationSnapshotKind::Run)
                .unwrap()
                .read_model()
                .as_value()
                .clone(),
        )
        .unwrap();
        if let Some(vn) = view.visual_novel {
            let value = if let Some(choice) = vn.choices.iter().find(|choice| choice.selectable) {
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})
            };
            execute(&mut app, value).unwrap();
            continue;
        }
        if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = view.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|row| row.employee_definition_id)
                .collect();
            execute(
                &mut app,
                json!({"type":"select_starter_employees","employee_definition_ids":ids,"loan_copy_ids":[]}),
            )
            .unwrap();
            continue;
        }
        let map = view.map.unwrap();
        if let Some(node) = map.nodes.iter().find(|node| {
            node.state == MapNodeState::Available && node.category == MapNodeCategory::Combat
        }) {
            execute(
                &mut app,
                json!({"type":"select_map_node","node_id":node.id}),
            )
            .unwrap();
            return app;
        }
        let node = map
            .nodes
            .iter()
            .find(|node| {
                node.state == MapNodeState::Available && node.category == MapNodeCategory::Start
            })
            .unwrap();
        execute(
            &mut app,
            json!({"type":"select_map_node","node_id":node.id}),
        )
        .unwrap();
        execute(&mut app, json!({"type":"confirm_enter_node"})).unwrap();
        execute(&mut app, json!({"type":"complete_node"})).unwrap();
    }
    panic!("frontmatter bound exceeded")
}
fn enter(app: &mut CoreApplication) {
    execute(app, json!({"type":"confirm_enter_node"})).unwrap();
}
fn rejected<T>(result: Result<T, ApplicationError>) {
    assert_eq!(
        result.err().expect("fault gate must reject").code(),
        "battle_faulted"
    );
}
fn committed_battle(app: &CoreApplication) -> Value {
    ApplicationResult::from_behavior(&app.core.active_battle_reconnect_projection(0).unwrap())
        .unwrap()
        .payload()
        .as_value()
        .clone()
}

#[test]
fn native_fault_preserves_last_root_first_cause_and_every_public_mutation_gate() {
    let mut app = waiting(1000);
    let token = app.capture_persistence_rollback().unwrap().unwrap();
    let checkpoint = ApplicationCheckpoint::new(Vec::new(), 0, 0);
    let generation = ValidatedGameData::load_live_embedded().unwrap();
    enter(&mut app);
    let before = app.core.get_run_snapshot_json().unwrap();
    let battle = committed_battle(&app);
    let resources = serde_json::to_value(app.run_resources()).unwrap();
    let error = app.advance_active_battle_by(60_000, 0).unwrap_err();
    assert_eq!(error.code(), "battle_runtime_work_limit");
    let first = *app.battle_fault().unwrap();
    assert_eq!(first.operation, BattleFaultOperation::Elapsed);
    assert_eq!(before, app.core.get_run_snapshot_json().unwrap());
    assert_eq!(battle, committed_battle(&app));
    assert_eq!(
        resources,
        serde_json::to_value(app.run_resources()).unwrap()
    );
    let revision = app.transaction_revision;
    for _ in 0..64 {
        rejected(app.advance_active_battle_by(1, 0));
        rejected(app.advance_active_battle_for_host_tick(1, 0));
        rejected(app.prepare_active_battle_for_host_tick(1, 0));
        rejected(
            app.prepare_admin_command(
                ApplicationAdminCommand::from_document(
                    json!({"type":"admin_set_enkephalin","amount":1}).into(),
                )
                .unwrap(),
            ),
        );
        rejected(app.prepare_command(
            Uuid::nil(),
            command(json!({"type":"pause_battle"})),
            Default::default(),
        ));
        rejected(app.prepare_command_with_generation(
            Uuid::nil(),
            command(
                json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":true}),
            ),
            Default::default(),
            Some(generation.clone()),
        ));
        rejected(app.advance_active_battle_to_next_event_bucket());
        rejected(app.pause_active_battle_for_disconnect());
        rejected(execute(&mut app, json!({"type":"pause_battle"})));
        rejected(execute(&mut app, json!({"type":"request_battle_state"})));
        rejected(app.reset_player_profile());
        rejected(app.merge_player_profile(json!({}).into()));
        rejected(app.merge_encounter_compendium_profile(json!({}).into()));
        rejected(app.export_active_run_checkpoint());
        rejected(app.capture_persistence_rollback());
        rejected(app.active_battle_reconnect(0));
        rejected(app.prepare_non_battle_command(
            Uuid::nil(),
            command(
                json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":true}),
            ),
            Default::default(),
        ));
        rejected(app.continue_from_active_run_checkpoint(Uuid::nil(), &checkpoint));
        rejected(
            app.execute_admin(
                ApplicationAdminCommand::from_document(
                    json!({"type":"admin_set_enkephalin","amount":1}).into(),
                )
                .unwrap(),
            ),
        );
        rejected(app.execute_with_generation(
            Uuid::nil(),
            command(
                json!({"type":"start_new_game","game_mode":"Tutorial","replace_active_run":true}),
            ),
            Default::default(),
            generation.clone(),
        ));
        rejected(app.convert_active_run_v7(&[], app.content_identity()));
        rejected(app.convert_active_run_v8(&[], app.content_identity()));
        assert_eq!(app.battle_fault(), Some(&first));
        assert_eq!(app.transaction_revision, revision);
        let run = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
        assert_eq!(run.read_model().as_value()["battle_fault"], json!(first));
        assert_eq!(
            run.read_model().as_value()["action_capabilities"],
            json!([])
        );
        assert_eq!(
            run.read_model().as_value()["game_state_context"]["can_retreat"],
            false
        );
        assert!(run.read_model().as_value()["game_state_context"]["deployment"].is_null());
        assert_eq!(
            app.snapshot(ApplicationSnapshotKind::ActionCapabilities)
                .unwrap()
                .read_model()
                .as_value(),
            &json!([])
        );
        assert!(app.snapshot(ApplicationSnapshotKind::PlayerProfile).is_ok());
    }
    rejected(app.restore_persistence_rollback(token));
    #[cfg(feature = "test-fixtures")]
    rejected(app.replace_active_battle_event_log_fixture(1));
    assert_eq!(
        app.fork_for_authoring().unwrap().battle_fault(),
        Some(&first)
    );
    assert_eq!(before, app.core.get_run_snapshot_json().unwrap());
    assert_eq!(battle, committed_battle(&app));
    assert!(std::mem::size_of::<ApplicationBattleFault>() <= 128);
}

#[test]
fn normal_refusals_queries_and_profile_errors_do_not_fault() {
    let mut app = waiting(10_000_000);
    enter(&mut app);
    assert_eq!(
        execute(&mut app, json!({"type":"pause_battle"}))
            .unwrap_err()
            .kind(),
        ApplicationErrorKind::MissingBattleStateAuthority
    );
    assert!(app.active_battle_reconnect(u64::MAX).is_err());
    assert!(app
        .merge_player_profile(json!({"highest_floor_reached":"invalid"}).into())
        .is_err());
    let bad = ApplicationAdminCommand::from_document(
        json!({"type":"admin_enter_event","event_id":"absent"}).into(),
    )
    .unwrap();
    assert!(app.execute_admin(bad).is_err());
    assert!(app.battle_fault().is_none());
    app.advance_active_battle_by(1, 0).unwrap();
}

#[cfg(feature = "runtime-observation")]
#[test]
fn terminal_observation_keeps_failure_work_and_lease_without_a_fake_commit_or_repeat() {
    use crate::runtime_observation::{self as obs, Outcome};
    let mut app = waiting(1000);
    let (sender, receiver) = obs::channel();
    let (observer, reader) = sender.session(301);
    app.set_runtime_observer(Some(observer));
    enter(&mut app);
    assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Accepted);
    let live = reader.snapshot(false).unwrap();
    assert!(live.current.battle.is_some());
    assert_eq!(
        app.advance_active_battle_by(60_000, 0).unwrap_err().code(),
        "battle_runtime_work_limit"
    );
    let failure = receiver.try_recv().unwrap();
    assert_eq!(failure.outcome, Outcome::Rejected);
    assert!(failure.work > 0);
    let ended = reader.snapshot(false).unwrap();
    assert_eq!(ended.commits, live.commits);
    assert!(ended.current.battle.is_none());
    assert!(ended.current.usage.0.iter().all(|value| *value == 0));
    assert!(reader.is_alive());
    let counters = receiver.counters();
    for _ in 0..32 {
        rejected(app.advance_active_battle_for_host_tick(100, 0));
    }
    assert!(receiver.try_recv().is_err());
    let after = receiver.counters();
    assert_eq!(after.attempts, counters.attempts);
    assert_eq!(after.reasons, counters.reasons);
    assert_eq!(after.commits, counters.commits);
    assert_eq!(after.dropped, counters.dropped);
}

#[test]
fn interrupted_diagnostic_keeps_fixed_first_fault_and_origin_excludes_read_only_errors() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut app = waiting(10_000_000);
    enter(&mut app);
    let original = committed_battle(&app);
    // A read-only producer's identical error variant must not end the battle.
    let error = app.runtime_error(
        GameError::InvalidStaticData("query".into()),
        BattleFaultOperation::Command(ApplicationCommandKind::RequestBattleState),
        false,
    );
    assert_eq!(error.kind(), ApplicationErrorKind::InvalidStaticData);
    assert!(app.battle_fault().is_none());
    let limits = ExecutionLimits {
        max_result_bytes: 1,
        ..Default::default()
    };
    let token = CancellationToken::default();
    let result = control::run(&limits, &token, ExecutionPhase::TrialExecution, || {
        app.runtime_error(
            GameError::InvalidStaticData("x".repeat(1_000_000)),
            BattleFaultOperation::NextEvent,
            true,
        )
    });
    assert!(result.is_err());
    let first = *app.battle_fault().unwrap();
    assert_eq!(
        first.cause,
        BattleFaultCause::Internal {
            code: "invalid_static_data"
        }
    );
    assert!(serde_json::to_vec(&first).unwrap().len() < 512);
    assert_eq!(committed_battle(&app), original);
    for _ in 0..32 {
        app.runtime_error(
            GameError::BattleStateRevisionOverflow,
            BattleFaultOperation::Elapsed,
            true,
        );
        assert_eq!(app.battle_fault(), Some(&first));
    }
    rejected(app.advance_active_battle_to_next_event_bucket());
}

fn current_context(app: &CoreApplication) -> ApplicationExecutionContext {
    let authority = app.core.active_battle_authority().unwrap();
    ApplicationExecutionContext {
        expected_battle_authority: Some(ApplicationBattleAuthority {
            battle_uuid: authority.battle_uuid,
            battle_state_revision: authority.battle_state_revision,
        }),
        ..Default::default()
    }
}

#[test]
fn actual_dynamic_expression_failure_preserves_complete_root_and_latches_at_manual_command() {
    use crate::game::value_expr::{ValueExprFaultCode, ValueExprPhase};
    let (core, player, employee, activation) = GameCore::u03_expression_fault_fixture();
    let mut app = CoreApplication::from_core(core);
    let invalid = command(
        json!({"type":"activate_skill","employee_uuid":employee,"skill_id":"missing_skill","target":null}),
    );
    let refusal = app
        .execute(player, invalid, current_context(&app))
        .unwrap_err();
    assert_eq!(refusal.kind(), ApplicationErrorKind::InvalidStaticData);
    assert_eq!(refusal.code(), "invalid_static_data");
    assert!(
        app.battle_fault().is_none(),
        "invalid client skill must not be terminal"
    );
    let before = app.core.u03_complete_root_fingerprint();
    let input = command(
        json!({"type":"reinput_sustained_skill","employee_uuid":employee,"skill_id":"r08_strike","activation":activation,"target":null}),
    );
    let error = app
        .execute(player, input, current_context(&app))
        .unwrap_err();
    assert_eq!(error.code(), "value_expression_fault");
    let fault = *app.battle_fault().unwrap();
    assert_eq!(
        fault.operation,
        BattleFaultOperation::Command(ApplicationCommandKind::ActivateSkill)
    );
    assert!(matches!(
        fault.cause,
        BattleFaultCause::Expression {
            code: ValueExprFaultCode::DivideByZero,
            phase: ValueExprPhase::Evaluate,
            ..
        }
    ));
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    rejected(app.advance_active_battle_by(1, 0));
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
}

#[test]
fn actual_revision_overflow_is_terminal_but_postcommit_projection_cancellation_is_not() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let data = ValidatedGameData::load_live_embedded().unwrap();
    let limits = ExecutionLimits {
        max_clone_slots: 1 << 20,
        max_metadata_bytes: 64 * 1024 * 1024,
        max_work: 100_000_000,
        ..Default::default()
    };
    let (mut app, _, provenance) = control::run_with_provenance(
        &limits,
        &CancellationToken::default(),
        ExecutionPhase::TrialExecution,
        control::NativeStorageProvenance::fresh(),
        || {
            let mut app = waiting_with_data(data);
            enter(&mut app);
            app
        },
    )
    .unwrap();
    let before = committed_battle(&app);
    let cancellation = CancellationToken::default();
    let output = control::with_test_cancellation("application_diagnostic_entry", 1, || {
        control::run_with_provenance(
            &limits,
            &cancellation,
            ExecutionPhase::TrialExecution,
            provenance,
            || app.advance_active_battle_by(1, 0),
        )
    });
    assert_eq!(
        output
            .err()
            .expect("projection cancellation must abort the authoring scope")
            .code,
        "authoring_cancelled"
    );
    assert!(cancellation.is_cancelled());
    assert!(app.battle_fault().is_none());
    assert_ne!(
        committed_battle(&app),
        before,
        "domain commit precedes the interrupted public projection"
    );
    app.core.u03_exhaust_battle_revision_fixture();
    let before = app.core.u03_complete_root_fingerprint();
    let error = app
        .execute(
            Uuid::from_u128(0x5203),
            command(json!({"type":"pause_battle"})),
            current_context(&app),
        )
        .unwrap_err();
    assert_eq!(error.code(), "battle_state_revision_overflow");
    assert_eq!(
        app.battle_fault().unwrap().cause,
        BattleFaultCause::Internal {
            code: "battle_state_revision_overflow"
        }
    );
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
}

#[test]
fn trusted_reconciliation_origins_fault_once_and_preserve_complete_root() {
    use crate::game::battle::runtime_control::RuntimeFaultCode;
    for (kind, site) in [
        ("incapacitated", "reconcile_incapacitated_roster"),
        ("corroded_roster", "reconcile_corroded_roster"),
        ("corroded_source", "reconcile_corroded_source"),
        ("context", "application_context"),
    ] {
        let (mut core, _, _, _) = GameCore::u03_reconcile_origin_fixture();
        core.u03_corrupt_reconcile_fixture(kind);
        let mut app = CoreApplication::from_core(core);
        let authority = current_context(&app).expected_battle_authority.unwrap();
        let before = app.core.u03_complete_root_fingerprint();
        let first_error = app.advance_active_battle_by(50, 0).unwrap_err();
        assert_eq!(first_error.code(), "battle_runtime_invariant", "{kind}");
        let first = *app.battle_fault().unwrap();
        assert_eq!(first.authority, authority);
        assert_eq!(first.operation, BattleFaultOperation::Elapsed);
        assert!(
            matches!(first.cause, BattleFaultCause::Native { fault }
            if fault.code == RuntimeFaultCode::Invariant && fault.site == site
                && fault.limit.is_none() && fault.requested.is_none()),
            "{first:?}"
        );
        assert_eq!(app.core.u03_complete_root_fingerprint(), before);
        for _ in 0..64 {
            rejected(app.advance_active_battle_by(50, 0));
            rejected(app.advance_active_battle_for_host_tick(100, 0));
            rejected(app.advance_active_battle_to_next_event_bucket());
            assert_eq!(app.battle_fault(), Some(&first));
        }
        assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    }
}

#[test]
fn ordinary_missing_employee_is_recoverable_at_the_real_public_producer() {
    let (core, player, _, position) = GameCore::u03_reconcile_origin_fixture();
    let mut app = CoreApplication::from_core(core);
    let before = app.core.u03_complete_root_fingerprint();
    let missing = command(json!({"type":"deploy_unit", "employee_uuid":Uuid::nil(),
        "position":position, "facing":"right"}));
    let error = app
        .execute(player, missing, current_context(&app))
        .unwrap_err();
    assert_eq!(error.code(), "unit_not_found");
    assert!(app.battle_fault().is_none());
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    app.advance_active_battle_by(50, 0).unwrap();
    assert!(app.battle_fault().is_none());
}

#[test]
fn trusted_reconciliation_fault_at_manual_deployment_retains_original_authority() {
    let (core, player, employee, position) = GameCore::u03_manual_reconcile_fixture();
    let mut app = CoreApplication::from_core(core);
    let before = app.core.u03_complete_root_fingerprint();
    let authority = current_context(&app).expected_battle_authority.unwrap();
    let deploy = command(json!({"type":"deploy_unit", "employee_uuid":employee,
        "position":position, "facing":"right"}));
    let error = app
        .execute(player, deploy, current_context(&app))
        .unwrap_err();
    assert_eq!(error.code(), "battle_runtime_invariant");
    let first = *app.battle_fault().unwrap();
    assert_eq!(first.authority, authority);
    assert_eq!(
        first.operation,
        BattleFaultOperation::Command(ApplicationCommandKind::DeployUnit)
    );
    assert!(matches!(first.cause, BattleFaultCause::Native { fault }
        if fault.site == "reconcile_corroded_roster"));
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    rejected(app.advance_active_battle_by(50, 0));
    assert_eq!(app.battle_fault(), Some(&first));
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
}

#[test]
fn ordinary_invalid_targeting_action_is_recoverable_at_the_real_public_producer() {
    let (core, player, employee, activation) = GameCore::u03_expression_fault_fixture();
    let mut app = CoreApplication::from_core(core);
    let before = app.core.u03_complete_root_fingerprint();
    let input = command(
        json!({"type":"reinput_sustained_skill", "employee_uuid":employee,
        "skill_id":"r08_strike", "activation":activation + 1, "target":null}),
    );
    let error = app
        .execute(player, input, current_context(&app))
        .unwrap_err();
    assert_eq!(error.code(), "invalid_action");
    assert!(app.battle_fault().is_none());
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    app.advance_active_battle_by(1, 0).unwrap();
    assert!(app.battle_fault().is_none());
}

#[test]
fn trusted_boss_activation_failure_preserves_first_cause_at_public_advance() {
    let mut app = CoreApplication::from_core(GameCore::u03_boss_activation_fault_fixture());
    let before = app.core.u03_complete_root_fingerprint();
    let error = app.advance_active_battle_by(60_000, 0).unwrap_err();
    assert_eq!(error.code(), "battle_runtime_invariant");
    let first = *app.battle_fault().unwrap();
    assert!(matches!(first.cause, BattleFaultCause::Native { fault }
        if fault.site == "boss_runtime_boundary"));
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    rejected(app.advance_active_battle_by(1, 0));
    assert_eq!(app.battle_fault(), Some(&first));
}

#[cfg(feature = "runtime-observation")]
#[test]
fn trusted_origin_fault_ends_observation_once_without_committing_candidate() {
    use crate::runtime_observation::{self as obs, Outcome};
    for kind in [
        "incapacitated",
        "corroded_roster",
        "corroded_source",
        "context",
    ] {
        let (core, _, _, _) = GameCore::u03_reconcile_origin_fixture();
        let mut app = CoreApplication::from_core(core);
        let (sender, receiver) = obs::channel();
        let (observer, reader) = sender.session(302);
        app.set_runtime_observer(Some(observer));
        app.advance_active_battle_by(1, 0).unwrap();
        assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Accepted);
        let before = reader.snapshot(false).unwrap();
        assert!(before.current.battle.is_some());
        app.core.u03_corrupt_reconcile_fixture(kind);
        let root = app.core.u03_complete_root_fingerprint();
        assert_eq!(
            app.advance_active_battle_by(50, 0).unwrap_err().code(),
            "battle_runtime_invariant"
        );
        assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Rejected);
        let ended = reader.snapshot(false).unwrap();
        assert_eq!(ended.commits, before.commits);
        assert!(ended.current.battle.is_none());
        assert!(ended.current.usage.0.iter().all(|value| *value == 0));
        assert!(reader.is_alive());
        let counters = receiver.counters();
        for _ in 0..32 {
            rejected(app.advance_active_battle_for_host_tick(100, 0));
        }
        assert!(receiver.try_recv().is_err());
        let after = receiver.counters();
        assert_eq!(after.attempts, counters.attempts);
        assert_eq!(after.reasons, counters.reasons);
        assert_eq!(after.commits, counters.commits);
        assert_eq!(after.dropped, counters.dropped);
        assert_eq!(app.core.u03_complete_root_fingerprint(), root);
    }
}

fn assert_actual_manual_origin_fault(
    core: GameCore,
    player: Uuid,
    input: serde_json::Value,
    code: &str,
    site: &str,
) {
    let mut app = CoreApplication::from_core(core);
    let before = app.core.u03_complete_root_fingerprint();
    let authority = current_context(&app).expected_battle_authority.unwrap();
    let error = app
        .execute(player, command(input.clone()), current_context(&app))
        .unwrap_err();
    assert_eq!(error.code(), code);
    let first = *app.battle_fault().unwrap();
    assert_eq!(first.authority, authority);
    assert!(
        matches!(first.cause, BattleFaultCause::Native { fault }
        if fault.site == site && fault.limit.is_none() && fault.requested.is_none()),
        "{first:?}"
    );
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
    for _ in 0..64 {
        rejected(app.execute(player, command(input.clone()), current_context(&app)));
        rejected(app.advance_active_battle_by(1, 0));
        rejected(app.advance_active_battle_for_host_tick(100, 0));
        rejected(app.advance_active_battle_to_next_event_bucket());
        rejected(app.ensure_battle_not_faulted());
        assert_eq!(app.battle_fault(), Some(&first));
    }
    assert_eq!(app.core.u03_complete_root_fingerprint(), before);
}

#[test]
fn trusted_goodbye_definition_failure_faults_at_public_reinput() {
    let (mut core, player, employee, activation) = GameCore::u03_sustained_origin_fixture(false);
    core.u03_corrupt_goodbye_fixture();
    assert_actual_manual_origin_fault(
        core,
        player,
        json!({"type":"reinput_sustained_skill",
        "employee_uuid":employee, "skill_id":"r08_strike", "activation":activation, "target":null}),
        "battle_runtime_invariant",
        "reinput_goodbye_definition",
    );
}

#[test]
fn admitted_cast_deadline_overflow_faults_for_zero_and_positive_focus() {
    for focus in [0, 100] {
        let (core, player, employee) = GameCore::u03_admitted_deadline_fixture(focus);
        assert_actual_manual_origin_fault(
            core,
            player,
            json!({"type":"activate_skill",
            "employee_uuid":employee, "skill_id":"r08_strike", "target":null}),
            "battle_runtime_counter_overflow",
            "admitted_cast_deadline",
        );
    }
}

#[test]
fn trusted_swap_deployment_references_fault_for_either_missing_native_unit() {
    for first in [true, false] {
        let (core, player, first_employee, second_employee) =
            GameCore::u03_swap_origin_fixture(first);
        assert_actual_manual_origin_fault(
            core,
            player,
            json!({"type":"tactical_swap_units",
            "first_employee_uuid":first_employee, "second_employee_uuid":second_employee,
            "first_facing":"right", "second_facing":"right"}),
            "battle_runtime_invariant",
            "application_deployed_unit",
        );
    }
}

#[test]
fn ordinary_reinput_and_duplicate_hello_refusals_remain_recoverable() {
    for kind in [
        "activation",
        "target",
        "hello",
        "unsupported",
        "client_skill",
    ] {
        let (core, player, employee, activation) =
            GameCore::u03_sustained_origin_fixture(kind == "unsupported");
        let mut app = CoreApplication::from_core(core);
        let before = app.core.u03_complete_root_fingerprint();
        let input = if kind == "hello" || kind == "client_skill" {
            json!({"type":"activate_skill", "employee_uuid":employee,
                "skill_id":if kind == "client_skill" {"u03_missing_client_skill"} else {"r08_strike"}, "target":null})
        } else {
            json!({"type":"reinput_sustained_skill", "employee_uuid":employee, "skill_id":"r08_strike",
                "activation":if kind == "activation" {activation+1} else {activation},
                "target":if kind == "target" {json!({"type":"Unit", "unit_instance_id":Uuid::nil()})} else {serde_json::Value::Null}})
        };
        let error = app
            .execute(player, command(input), current_context(&app))
            .unwrap_err();
        assert_eq!(
            error.code(),
            if kind == "client_skill" {
                "invalid_static_data"
            } else if kind == "hello" {
                "action_locked"
            } else {
                "invalid_action"
            },
            "{kind}"
        );
        assert!(app.battle_fault().is_none(), "{kind}");
        assert_eq!(app.core.u03_complete_root_fingerprint(), before, "{kind}");
        app.advance_active_battle_by(1, 0).unwrap();
        assert!(app.battle_fault().is_none());
    }
}

#[cfg(feature = "runtime-observation")]
#[test]
fn trusted_goodbye_failure_ends_observation_without_publishing_candidate() {
    use crate::runtime_observation::{self as obs, Outcome};
    let (core, player, employee, activation) = GameCore::u03_sustained_origin_fixture(false);
    let mut app = CoreApplication::from_core(core);
    let (sender, receiver) = obs::channel();
    let (observer, reader) = sender.session(303);
    app.set_runtime_observer(Some(observer));
    app.advance_active_battle_by(1, 0).unwrap();
    assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Accepted);
    let before = reader.snapshot(false).unwrap();
    assert!(before.current.battle.is_some());
    app.core.u03_corrupt_goodbye_fixture();
    let root = app.core.u03_complete_root_fingerprint();
    let input = json!({"type":"reinput_sustained_skill", "employee_uuid":employee,
        "skill_id":"r08_strike", "activation":activation, "target":null});
    assert_eq!(
        app.execute(player, command(input), current_context(&app))
            .unwrap_err()
            .code(),
        "battle_runtime_invariant"
    );
    assert_eq!(receiver.try_recv().unwrap().outcome, Outcome::Rejected);
    let ended = reader.snapshot(false).unwrap();
    assert_eq!(ended.commits, before.commits);
    assert!(ended.current.battle.is_none());
    assert!(ended.current.usage.0.iter().all(|value| *value == 0));
    let first = *app.battle_fault().unwrap();
    let counters = receiver.counters();
    for _ in 0..32 {
        rejected(app.advance_active_battle_for_host_tick(100, 0));
    }
    assert!(receiver.try_recv().is_err());
    let after = receiver.counters();
    assert_eq!(after.attempts, counters.attempts);
    assert_eq!(after.reasons, counters.reasons);
    assert_eq!(after.commits, counters.commits);
    assert_eq!(app.battle_fault(), Some(&first));
    assert_eq!(app.core.u03_complete_root_fingerprint(), root);
}

#[test]
fn u04_prepared_command_and_host_tick_drop_preserve_roots_and_commit_matches_immediate_ports() {
    let player = Uuid::from_u128(0x5203);
    let mut app = waiting(10_000_000);
    let before = app.core.get_run_snapshot_json().unwrap();
    let checkpoint = app.export_active_run_checkpoint().unwrap();
    let enter_command = || command(json!({"type":"confirm_enter_node"}));
    {
        let prepared = app
            .prepare_command(player, enter_command(), Default::default())
            .unwrap();
        assert!(prepared.enters_battle());
        assert!(!prepared.terminal());
        assert!(prepared.checkpoint().is_some());
    }
    assert_eq!(app.core.get_run_snapshot_json().unwrap(), before);
    assert_eq!(app.export_active_run_checkpoint().unwrap(), checkpoint);
    assert!(app.battle_fault().is_none());
    app.prepare_command(player, enter_command(), Default::default())
        .unwrap()
        .commit();
    let live = committed_battle(&app);
    let checkpoint = app.export_active_run_checkpoint().unwrap();
    {
        let context = current_context(&app);
        let prepared = app
            .prepare_command(player, command(json!({"type":"pause_battle"})), context)
            .unwrap();
        assert_eq!(
            prepared.result().kind(),
            ApplicationResultKind::BattlePlaybackChanged
        );
    }
    assert_eq!(committed_battle(&app), live);
    assert_eq!(app.export_active_run_checkpoint().unwrap(), checkpoint);
    let mut direct = CoreApplication::from_core(app.core.clone());
    let expected = direct.advance_active_battle_for_host_tick(50, 0).unwrap();
    {
        let prepared = app.prepare_active_battle_for_host_tick(50, 0).unwrap();
        assert_eq!(
            prepared
                .result()
                .as_ref()
                .map(|result| result.payload().as_value()),
            expected.as_ref().map(|result| result.payload().as_value())
        );
    }
    assert_eq!(committed_battle(&app), live);
    assert_eq!(app.export_active_run_checkpoint().unwrap(), checkpoint);
    let actual = app
        .prepare_active_battle_for_host_tick(50, 0)
        .unwrap()
        .commit();
    assert_eq!(
        actual.as_ref().map(|result| result.payload().as_value()),
        expected.as_ref().map(|result| result.payload().as_value())
    );
    assert_eq!(committed_battle(&app), committed_battle(&direct));
    assert!(app.battle_fault().is_none());
}

#[test]
fn u04_prepared_host_tick_fault_retains_first_origin_and_original_authority() {
    let mut app = waiting(2000);
    enter(&mut app);
    let before = app.core.get_run_snapshot_json().unwrap();
    let battle = committed_battle(&app);
    let error = app
        .prepare_active_battle_for_host_tick(60_000, 0)
        .err()
        .expect("actual work refusal");
    assert_eq!(error.code(), "battle_runtime_work_limit");
    let first = *app.battle_fault().unwrap();
    assert_eq!(first.operation, BattleFaultOperation::HostTick);
    assert_eq!(app.core.get_run_snapshot_json().unwrap(), before);
    assert_eq!(committed_battle(&app), battle);
    rejected(app.prepare_active_battle_for_host_tick(1, 0));
    assert_eq!(app.battle_fault(), Some(&first));
}
