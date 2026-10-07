//! The first two probes deliberately activate a native allowance outside the
//! CoreApplication port, including its otherwise unmetered World capture. Their
//! typed outcome belongs to that injected outer scope, not the inner sentinel.
//! The third probe uses immutable host settings and the normal public boundary.
//! Setup reuses the inherited internal U08 fixture; public content loading and
//! numerical expectations remain independently covered by u08_relic_public.
use super::*;
use crate::application::{
    ApplicationCommand, ApplicationError, ApplicationErrorKind, ApplicationResult,
    ApplicationSnapshotKind, CoreApplication,
};
use crate::game::battle::runtime_control as native;

fn setup_core() -> (GameCore, Uuid, Uuid) {
    let (mut core, player, employees) = super::u08_relics::setup();
    let copy = super::u08_relics::grant(&mut core, 1)[0];
    core.execute(
        player,
        PlayerBehavior::EquipRelic {
            copy_uuid: copy,
            employee_uuid: employees[0],
            slot: 2,
        },
    )
    .unwrap();
    core.execute_admin_command(
        crate::game::world::AdminCommand::AdminGrantRelicCorruptionCatalyst { amount: 1 },
    )
    .unwrap();
    core.execute(player, PlayerBehavior::CorruptRelic { copy_uuid: copy })
        .unwrap();
    (core, player, employees[0])
}

fn setup() -> (CoreApplication, Uuid, Uuid) {
    let (core, player, employee) = setup_core();
    (CoreApplication::from_core(core), player, employee)
}

fn query(
    app: &mut CoreApplication,
    player: Uuid,
    employee: Uuid,
) -> Result<ApplicationResult, ApplicationError> {
    app.execute(
        player,
        ApplicationCommand::from_document(
            json!({
                "type":"request_relic_numeric_plan", "employee_uuid":employee,
            })
            .into(),
        )
        .unwrap(),
        Default::default(),
    )
}

fn roots(app: &CoreApplication) -> Value {
    let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
        .iter()
        .map(|kind| app.snapshot(*kind).unwrap().read_model().as_value().clone())
        .collect();
    let saved = app.export_active_run_checkpoint().unwrap().unwrap();
    json!({"snapshots":snapshots, "checkpoint_bytes":saved.core_payload_json(),
        "loads_used":saved.loads_used(), "certified":saved.run_peak_certified_power()})
}

fn fault(error: GameError, code: native::RuntimeFaultCode, site: &str, cap: usize) {
    let GameError::BattleRuntimeFault(error) = error else {
        panic!("expected native refusal: {error:?}")
    };
    assert_eq!(error.code, code);
    assert_eq!(error.site, site);
    assert_eq!(error.limit, Some(cap as u64));
    assert!(error.requested.unwrap() > cap as u64);
}

fn private_interruption(error: ApplicationError) {
    // ApplicationError::check_projection_control deliberately avoids allocating
    // a diagnostic under a sticky private fault. The outer scope discards this
    // envelope and returns its first typed interruption.
    assert_eq!(error.kind(), ApplicationErrorKind::ProjectionFailure);
    assert!(error.code().is_empty());
    assert_eq!(error.details().as_value(), &Value::Null);
}

#[test]
fn relic_query_outer_native_retained_admission_precedes_targets_penalties_and_generation() {
    let (mut probe, player, employee) = setup();
    let before = roots(&probe);
    let (result, admissions) = native::observe_retained_admissions(|| {
        native::run(native::RuntimeFactLimits::default(), || {
            Ok(query(&mut probe, player, employee))
        })
    });
    let result = result.unwrap().unwrap();
    let baseline_payload = result.payload().as_value().clone();
    assert_eq!(
        result.payload().as_value()["employee_uuid"],
        json!(employee)
    );
    assert_eq!(
        result.payload().as_value()["penalties"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(!result.payload().as_value()["targets"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(roots(&probe), before);
    assert!(admissions.windows(2).all(|rows| rows[0].2 <= rows[1].1));
    let final_admission = admissions.iter().rev().find(|row| row.2 > row.1).unwrap();
    let final_cap = final_admission.2;
    assert!(
        final_cap < 8 * 1024 * 1024,
        "this fixture must fit the existing engine retained ceiling"
    );
    println!("U08_QUERY_ADMISSIONS {admissions:?}");

    // Observed amounts locate the boundary. The required refusal sites, exact
    // admission prefix, outer fault category and unchanged roots are fixed
    // expectations independent of those amounts.
    for site in [
        "relic_plan_targets",
        "relic_plan_effect_rows",
        "relic_plan_penalty_rows",
        "relic_plan_generation",
    ] {
        let index = admissions
            .iter()
            .position(|row| row.0 == site && row.2 > row.1)
            .unwrap_or_else(|| panic!("missing positive {site} admission"));
        let cap = admissions[index].2 - 1;
        let (mut app, player, employee) = setup();
        let before = roots(&app);
        let mut private_error = None;
        let (result, accepted_prefix) = native::observe_retained_admissions(|| {
            native::run(
                native::RuntimeFactLimits {
                    max_retained_bytes: cap,
                    ..Default::default()
                },
                || {
                    let result = query(&mut app, player, employee);
                    private_error = result.as_ref().err().cloned();
                    Ok(result)
                },
            )
        });
        fault(
            result.unwrap_err(),
            native::RuntimeFaultCode::RetainedByteLimit,
            site,
            cap,
        );
        private_interruption(private_error.expect("the interrupted query must reject"));
        assert_eq!(
            accepted_prefix,
            admissions[..index],
            "failed admission must not record acceptance or permit a later query allocation"
        );
        assert_eq!(roots(&app), before);
        assert_eq!(
            query(&mut app, player, employee)
                .unwrap()
                .payload()
                .as_value(),
            &baseline_payload
        );
    }

    for (cap, succeeds) in [(final_cap, true), (final_cap - 1, false)] {
        let (mut app, player, employee) = setup();
        let before = roots(&app);
        let result = native::run(
            native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            },
            || Ok(query(&mut app, player, employee)),
        );
        if succeeds {
            result.unwrap().unwrap();
        } else {
            fault(
                result.unwrap_err(),
                native::RuntimeFaultCode::RetainedByteLimit,
                final_admission.0,
                cap,
            );
        }
        assert_eq!(roots(&app), before);
    }
}

#[test]
fn relic_query_outer_native_low_work_and_clone_limits_preserve_roots_and_recover_next_operation() {
    for (limits, code, site) in [
        (
            native::RuntimeFactLimits {
                max_work: 1,
                ..Default::default()
            },
            native::RuntimeFaultCode::WorkLimit,
            // The injected scope is already active at World entry. Its owner
            // extent scan visits one copy plus one slot map before clone scan.
            "relic_owner_extent",
        ),
        (
            native::RuntimeFactLimits {
                max_clone_slots: 1,
                ..Default::default()
            },
            native::RuntimeFaultCode::CloneLimit,
            "fact_clone_slots",
        ),
    ] {
        let (mut app, player, employee) = setup();
        let before = roots(&app);
        let mut private_error = None;
        let result = native::run(limits, || {
            let result = query(&mut app, player, employee);
            private_error = result.as_ref().err().cloned();
            Ok(result)
        });
        fault(result.unwrap_err(), code, site, 1);
        private_interruption(private_error.expect("native failure must reject the inner query"));
        assert_eq!(roots(&app), before);
        query(&mut app, player, employee).unwrap();
        assert_eq!(
            roots(&app),
            before,
            "a fresh operation must recover without changing UUID, save or loadout state"
        );
    }
}

fn setup_public_limits(
    limits: native::RuntimeFactLimits,
) -> (
    CoreApplication,
    Uuid,
    Uuid,
    crate::game::data::ValidatedGameData,
) {
    use crate::game::data::GameDataBuilder;
    use crate::runtime_safety::RuntimeSafetySettings;

    let defaults = native::RuntimeFactLimits::default();
    assert!(limits.max_retained_bytes <= defaults.max_retained_bytes);
    assert!(limits.max_retained_bytes <= 8 * 1024 * 1024);
    assert!(limits.max_work <= defaults.max_work);
    assert!(limits.max_clone_slots <= defaults.max_clone_slots);
    let mut raw = std::str::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON)
        .unwrap()
        .to_owned();
    for (field, before, after) in [
        ("max_work", defaults.max_work, limits.max_work),
        (
            "max_clone_slots",
            defaults.max_clone_slots as u64,
            limits.max_clone_slots as u64,
        ),
        (
            "max_retained_bytes",
            defaults.max_retained_bytes as u64,
            limits.max_retained_bytes as u64,
        ),
    ] {
        let original = format!("{field}: {before},");
        assert!(raw.contains(&original));
        // Native settings are the first block; authoring/expression settings
        // with the same field names remain byte-for-byte unchanged.
        raw = raw.replacen(&original, &format!("{field}: {after},"), 1);
    }
    let settings =
        Arc::new(RuntimeSafetySettings::from_ron("u08 query cap fixture", raw.as_bytes()).unwrap());
    assert_eq!(settings.native_runtime(), limits);
    assert!(settings.development_measurement().is_none());

    // Synthetic setup grants under default settings, then installs immutable
    // host settings before constructing the public application. This is not a
    // gameplay generation switch, a public limit setter, or a lowered setup cap.
    let (mut core, player, employee) = setup_core();
    let ordinary_generation = core.game_data.clone();
    let before = roots(&CoreApplication::from_core(core.clone()));
    let profiled: crate::game::data::ValidatedGameData = Arc::new(
        GameDataBuilder::from_validated(&ordinary_generation)
            .build_with_runtime_safety(settings)
            .unwrap(),
    )
    .into();
    assert_eq!(profiled.identity(), ordinary_generation.identity());
    core.game_data = profiled;
    let app = CoreApplication::from_core(core);
    assert_eq!(
        roots(&app),
        before,
        "host caps must not alter catalog identity, ledgers, UUID or save bytes"
    );
    (app, player, employee, ordinary_generation)
}

fn public_fault(error: &ApplicationError, code: &str, site: &str, cap: usize) {
    assert!(
        !native::has_operation_scope(),
        "public rejection must leave its native scope"
    );
    assert_eq!(error.kind(), ApplicationErrorKind::BattleRuntimeFault);
    assert_eq!(error.code(), code);
    let details = error.details().as_value();
    assert_eq!(
        details["code"],
        code.strip_prefix("battle_runtime_").unwrap()
    );
    assert_eq!(details["site"], site);
    assert_eq!(details["limit"], json!(cap));
    assert!(details["requested"].as_u64().unwrap() > cap as u64);
}

fn restore_public_query_after_refusal(
    app: &CoreApplication,
    generation: crate::game::data::ValidatedGameData,
    player: Uuid,
    employee: Uuid,
    baseline_payload: &Value,
) {
    let checkpoint = app.export_active_run_checkpoint().unwrap().unwrap();
    let original = checkpoint.core_payload_json().to_vec();
    let mut recovered = CoreApplication::new(generation, 0xa508cafe);
    recovered
        .continue_from_active_run_checkpoint(player, &checkpoint)
        .unwrap();
    assert_eq!(checkpoint.core_payload_json(), original);
    for kind in [
        ApplicationSnapshotKind::Inventory,
        ApplicationSnapshotKind::EmployeeRoster,
    ] {
        assert_eq!(
            recovered.snapshot(kind).unwrap(),
            app.snapshot(kind).unwrap()
        );
    }
    let before = roots(&recovered);
    assert_eq!(
        query(&mut recovered, player, employee)
            .unwrap()
            .payload()
            .as_value(),
        baseline_payload
    );
    assert_eq!(roots(&recovered), before);
}

#[test]
fn relic_public_query_host_settings_return_typed_faults_after_the_normal_scope_closes() {
    let (mut baseline, player, employee, _) = setup_public_limits(Default::default());
    let before = roots(&baseline);
    assert!(!native::has_operation_scope());
    // No injected scope: the production World entry activates the relic owner
    // after its rollback capture and closes that allowance before AppError.
    let (result, admissions) =
        native::observe_retained_admissions(|| query(&mut baseline, player, employee));
    let baseline_payload = result.unwrap().payload().as_value().clone();
    assert!(!native::has_operation_scope());
    assert_eq!(roots(&baseline), before);
    assert_eq!(baseline_payload["employee_uuid"], json!(employee));
    assert_eq!(baseline_payload["penalties"].as_array().unwrap().len(), 2);
    assert!(!baseline_payload["targets"].as_array().unwrap().is_empty());
    assert!(admissions.windows(2).all(|rows| rows[0].2 <= rows[1].1));
    println!("U08_QUERY_NORMAL_PUBLIC_ADMISSIONS {admissions:?}");

    for site in [
        "relic_plan_targets",
        "relic_plan_effect_rows",
        "relic_plan_penalty_rows",
        "relic_plan_generation",
    ] {
        let index = admissions
            .iter()
            .position(|row| row.0 == site && row.2 > row.1)
            .unwrap_or_else(|| panic!("normal entry missing positive {site} admission"));
        let cap = admissions[index].2 - 1;
        let (mut app, player, employee, generation) =
            setup_public_limits(native::RuntimeFactLimits {
                max_retained_bytes: cap,
                ..Default::default()
            });
        let before = roots(&app);
        let (result, prefix) =
            native::observe_retained_admissions(|| query(&mut app, player, employee));
        let error = result.unwrap_err();
        public_fault(&error, "battle_runtime_retained_byte_limit", site, cap);
        assert_eq!(
            error.details().as_value()["requested"],
            json!(admissions[index].2)
        );
        assert_eq!(
            prefix,
            admissions[..index],
            "no failed or later allocation is admitted"
        );
        assert_eq!(roots(&app), before);
        let retry = query(&mut app, player, employee).unwrap_err();
        assert_eq!(
            retry, error,
            "unchanged fixed host cap must reject deterministically"
        );
        assert_eq!(roots(&app), before);
        restore_public_query_after_refusal(&app, generation, player, employee, &baseline_payload);
    }

    let final_admission = admissions.iter().rev().find(|row| row.2 > row.1).unwrap();
    let final_cap = final_admission.2;
    for (cap, succeeds) in [(final_cap, true), (final_cap - 1, false)] {
        let (mut app, player, employee, _) = setup_public_limits(native::RuntimeFactLimits {
            max_retained_bytes: cap,
            ..Default::default()
        });
        let before = roots(&app);
        let result = query(&mut app, player, employee);
        if succeeds {
            assert_eq!(result.unwrap().payload().as_value(), &baseline_payload);
        } else {
            public_fault(
                &result.unwrap_err(),
                "battle_runtime_retained_byte_limit",
                final_admission.0,
                cap,
            );
        }
        assert_eq!(roots(&app), before);
    }

    for (limits, code, site) in [
        (
            native::RuntimeFactLimits {
                max_work: 1,
                ..Default::default()
            },
            "battle_runtime_work_limit",
            "relic_owner_extent",
        ),
        (
            native::RuntimeFactLimits {
                max_clone_slots: 1,
                ..Default::default()
            },
            "battle_runtime_clone_limit",
            "fact_clone_slots",
        ),
    ] {
        let (mut app, player, employee, generation) = setup_public_limits(limits);
        let before = roots(&app);
        let error = query(&mut app, player, employee).unwrap_err();
        public_fault(&error, code, site, 1);
        assert_eq!(roots(&app), before);
        assert_eq!(query(&mut app, player, employee).unwrap_err(), error);
        assert_eq!(roots(&app), before);
        restore_public_query_after_refusal(&app, generation, player, employee, &baseline_payload);
    }
}
