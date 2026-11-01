use super::*;
use crate::game::battle::runtime_control as native;
use crate::runtime_observation::{self as observation, Operation, Outcome, Usage};
fn empty() -> CoreApplication {
    CoreApplication::new(crate::game::data::GameDataBuilder::empty().build_arc(), 41)
}
fn set_amount(
    app: &mut CoreApplication,
    amount: u32,
) -> Result<ApplicationResult, ApplicationError> {
    app.execute_admin(
        ApplicationAdminCommand::from_document(
            json!({"type":"admin_set_enkephalin","amount":amount}).into(),
        )
        .unwrap(),
    )
}
fn observe(
    app: &mut CoreApplication,
    id: u64,
) -> (observation::ObservationReceiver, observation::SessionReader) {
    let (sender, receiver) = observation::channel();
    let (observer, reader) = sender.session(id);
    app.set_runtime_observer(Some(observer));
    (receiver, reader)
}
#[test]
fn actual_commit_and_full_or_absent_collector_preserve_command_results_and_queries() {
    let mut measured = empty();
    let mut baseline = empty();
    let (receiver, reader) = observe(&mut measured, 7);
    for i in 0..observation::QUEUE_CAPACITY + 19 {
        assert_eq!(
            set_amount(&mut measured, i as u32).unwrap(),
            set_amount(&mut baseline, i as u32).unwrap()
        );
    }
    let total = (observation::QUEUE_CAPACITY + 19) as u64;
    assert_eq!(reader.snapshot(false).unwrap().commits, total);
    assert_eq!(receiver.counters().commits, total);
    assert_eq!(receiver.counters().dropped, 19);
    assert_eq!(
        receiver.counters().attempts[Operation::Admin.index() * 4 + Outcome::Accepted as usize],
        total
    );
    for kind in ApplicationSnapshotKind::ALL {
        assert_eq!(
            measured.snapshot(*kind).unwrap(),
            baseline.snapshot(*kind).unwrap()
        );
    }
    assert_eq!(
        receiver.counters().attempts.iter().sum::<u64>(),
        total,
        "read-only snapshots do not emit operations"
    );
    let records: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
    assert_eq!(records.len(), observation::QUEUE_CAPACITY);
    assert!(records
        .windows(2)
        .all(|p| p[0].sequence + 1 == p[1].sequence));
    drop(receiver);
    assert_eq!(
        set_amount(&mut measured, 31).unwrap(),
        set_amount(&mut baseline, 31).unwrap()
    );
    drop(measured);
    assert!(!reader.is_alive());
    assert!(reader.snapshot(false).is_none());
}
#[test]
fn discarded_preparation_detaches_tls_and_cannot_advance_commit_gauges() {
    let data = ValidatedGameData::load_live_embedded().unwrap();
    let mut app = CoreApplication::new(data, 43);
    let (receiver, reader) = observe(&mut app, 1);
    let mut other = empty();
    let (other_receiver, other_reader) = observe(&mut other, 2);
    let command = || {
        ApplicationCommand::from_document(json!({"type":"start_new_game","game_mode":crate::game::map::GameMode::Expedition,"replace_active_run":false}).into()).unwrap()
    };
    let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let prepared = app
        .prepare_non_battle_command(
            Uuid::from_u128(1),
            command(),
            ApplicationExecutionContext::default(),
        )
        .unwrap();
    assert_eq!(reader.snapshot(false).unwrap().commits, 0);
    assert!(receiver.try_recv().is_err());
    set_amount(&mut other, 8).unwrap();
    assert_eq!(other_reader.snapshot(false).unwrap().commits, 1);
    assert_eq!(other_receiver.try_recv().unwrap().session, 2);
    drop(prepared);
    assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
    let discarded = receiver.try_recv().unwrap();
    assert_eq!(discarded.outcome, Outcome::Discarded);
    assert!(discarded.committed.is_none());
    assert!(discarded.reason.is_none());
    assert_eq!(reader.snapshot(false).unwrap().commits, 0);
    let prepared = app
        .prepare_non_battle_command(
            Uuid::from_u128(1),
            command(),
            ApplicationExecutionContext::default(),
        )
        .unwrap();
    prepared.commit();
    let accepted = receiver.try_recv().unwrap();
    assert_eq!(accepted.outcome, Outcome::Accepted);
    assert_eq!(reader.snapshot(false).unwrap().commits, 1);
    assert_eq!(receiver.counters().commits, 1);
}
#[test]
fn nested_native_work_and_failed_attempt_peak_survive_rollback_exactly_once() {
    let mut app = empty();
    let (receiver, reader) = observe(&mut app, 3);
    let guard = app
        .observer
        .as_mut()
        .unwrap()
        .begin(Operation::HostTick, std::ptr::from_ref(&app.core) as usize)
        .unwrap();
    let limits = native::RuntimeFactLimits {
        max_work: 100,
        max_retained_bytes: 100,
        ..Default::default()
    };
    let result = native::run_deferred(|| {
        native::run(limits, || {
            native::seed_retained_extent(20)?;
            native::charge(2, "first")?;
            native::run(limits, || {
                native::admit_retained_growth(0, Some(40), "candidate")?;
                observation::peak(observation::PROJECTILES, 9);
                observation::peak(observation::PROJECTILES, 1);
                native::charge(94, "refused")
            })
        })
    });
    let error = ApplicationError::from_domain(result.unwrap_err());
    guard.finish_error(&error);
    let record = receiver.try_recv().unwrap();
    assert_eq!(record.outcome, Outcome::Rejected);
    assert_eq!(record.work, 7);
    assert_eq!(record.admitted_bytes, 60);
    assert_eq!(record.peak.0[observation::PROJECTILES], 9);
    assert!(record.committed.is_none());
    assert_eq!(
        observation::reason_labels()[record.reason.unwrap().0],
        "battle_runtime_work_limit"
    );
    assert_eq!(receiver.counters().reasons.iter().sum::<u64>(), 1);
    assert_eq!(
        reader.snapshot(true).unwrap().interval_peak.0[observation::PROJECTILES],
        9
    );
    assert_eq!(
        reader.snapshot(true).unwrap().interval_peak,
        Usage::default()
    );
    assert!(!native::has_operation_scope());
    set_amount(&mut app, 3).unwrap();
    assert_eq!(reader.snapshot(false).unwrap().commits, 1);
}
#[test]
fn a_nested_draft_commit_is_not_a_live_application_commit() {
    let mut app = empty();
    let (receiver, reader) = observe(&mut app, 4);
    let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
    let guard = app
        .observer
        .as_mut()
        .unwrap()
        .begin(Operation::Admin, std::ptr::from_ref(&app.core) as usize)
        .unwrap();
    let mut draft = app.core.clone();
    draft
        .execute_admin_command(
            ApplicationAdminCommand::from_document(
                json!({"type":"admin_set_enkephalin","amount":42}).into(),
            )
            .unwrap()
            .command,
        )
        .unwrap();
    let error = ApplicationError::from_domain(GameError::InvalidAction);
    guard.finish_error(&error);
    assert_eq!(reader.snapshot(false).unwrap().commits, 0);
    assert_eq!(receiver.counters().commits, 0);
    assert!(receiver.try_recv().unwrap().committed.is_none());
    assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
}
#[test]
fn finite_labels_have_unique_names_and_dense_operation_indices() {
    let labels = Operation::labels();
    assert_eq!(labels.len(), observation::OPERATION_COUNT);
    assert!(labels
        .iter()
        .enumerate()
        .all(|(index, (op, _))| op.index() == index));
    assert_eq!(
        labels
            .iter()
            .map(|(_, name)| name)
            .collect::<std::collections::HashSet<_>>()
            .len(),
        labels.len()
    );
    let reasons = observation::reason_labels();
    assert_eq!(reasons.len(), observation::REASON_COUNT);
    assert_eq!(
        reasons
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        reasons.len()
    );
}

fn cmrp_settings(cap: usize) -> crate::runtime_safety::RuntimeSafetySettings {
    let mut source = String::from_utf8(crate::runtime_safety::EMBEDDED_RUNTIME_SAFETY_RON.to_vec())
        .unwrap()
        .replacen(
            "max_retained_bytes: 16777216",
            "max_retained_bytes: 8388608",
            1,
        );
    let end = source.rfind(')').unwrap();
    source.insert_str(
        end,
        &format!("development_measurement: Some((revision:1, emergency_retained_bytes:{cap})),\n"),
    );
    crate::runtime_safety::RuntimeSafetySettings::from_ron(
        "cmrp observation fixture",
        source.as_bytes(),
    )
    .unwrap()
}
fn cmrp_native_record(
    operation: impl FnOnce() -> Result<(), GameError>,
) -> observation::OperationSummary {
    let mut app = empty();
    let (receiver, _) = observe(&mut app, 101);
    let guard = app
        .observer
        .as_mut()
        .unwrap()
        .begin(Operation::HostTick, std::ptr::from_ref(&app.core) as usize)
        .unwrap();
    let result = operation().map_err(ApplicationError::from_domain);
    guard.finish(&result);
    receiver.try_recv().unwrap()
}
#[test]
fn cmrp_accounting_actual_warning_and_emergency_exact_then_one_over() {
    for hard in [16 * 1024 * 1024, 32 * 1024 * 1024] {
        let settings = cmrp_settings(hard);
        let warning = settings.nominal_native_runtime().max_retained_bytes;
        let limits = settings.native_runtime();
        let record = cmrp_native_record(|| {
            native::run_with_profile(limits, &settings, || {
                native::seed_retained_extent(warning)?;
                native::admit_retained_growth(0, Some(0), "warning_exact")?;
                native::admit_retained_growth(0, Some(1), "warning_plus_one")?;
                native::run_with_profile(limits, &settings, || {
                    native::seed_retained_extent(0)?;
                    native::admit_retained_growth(0, Some(hard - warning - 1), "hard_exact")
                })?;
                native::admit_retained_growth(0, Some(1), "hard_plus_one")
            })
        });
        let usage = record.retained.unwrap();
        assert_eq!(usage.native_scopes, 1);
        assert_eq!(usage.profile_warning_bytes, Some(warning as u64));
        assert_eq!(usage.owner_hard_bytes, hard as u64);
        assert_eq!(usage.seed_attempted_bytes, Some(warning as u64));
        assert_eq!(usage.seed_accepted_bytes, Some(warning as u64));
        assert_eq!(usage.accepted_growth_bytes, (hard - warning) as u64);
        assert_eq!(usage.accepted_total_bytes, hard as u64);
        assert_eq!(usage.max_cumulative_requested_bytes, Some(hard as u64 + 1));
        assert_eq!(usage.first_soft_exceed.unwrap().site, "warning_plus_one");
        assert_eq!(
            usage.first_soft_exceed.unwrap().kind,
            observation::RetainedRequestKind::CumulativeGrowth
        );
        assert_eq!(usage.first_hard_exceed.unwrap().site, "hard_plus_one");
        assert_eq!(
            usage.first_hard_exceed.unwrap().requested_bytes,
            Some(hard as u64 + 1)
        );
        assert_eq!(usage.first_hard_exceed.unwrap().limit_bytes, hard as u64);
        assert_eq!(record.admitted_bytes, hard as u64);
        assert_eq!(record.outcome, Outcome::Rejected);
        assert!(record.committed.is_none());
        assert!(
            record.max_battle_owner_extent_bytes.is_none(),
            "helper requests are not whole-owner scans"
        );
    }
}
#[test]
fn cmrp_accounting_failed_seed_remains_attempted_and_small_owner_is_not_soft() {
    let settings = cmrp_settings(16 * 1024 * 1024);
    let limits = native::RuntimeFactLimits {
        max_retained_bytes: 255,
        ..settings.native_runtime()
    };
    let record = cmrp_native_record(|| {
        native::run_with_profile(limits, &settings, || native::seed_retained_extent(256))
    });
    let usage = record.retained.unwrap();
    assert_eq!(
        record.admitted_bytes, 256,
        "historical attempted seed is preserved"
    );
    assert_eq!(usage.seed_attempted_bytes, Some(256));
    assert_eq!(usage.seed_accepted_bytes, None);
    assert_eq!(usage.accepted_total_bytes, 0);
    assert_eq!(usage.profile_warning_bytes, Some(8 * 1024 * 1024));
    assert_eq!(usage.owner_hard_bytes, 255);
    assert!(usage.first_soft_exceed.is_none());
    assert_eq!(
        usage.first_hard_exceed.unwrap().site,
        "retained_operation_start"
    );
    assert_eq!(
        usage.first_hard_exceed.unwrap().kind,
        observation::RetainedRequestKind::Seed
    );
    let retry = cmrp_native_record(|| {
        native::run_with_profile(limits, &settings, || native::seed_retained_extent(255))
    });
    assert_eq!(retry.retained.unwrap().accepted_total_bytes, 255);
    assert!(retry.retained.unwrap().first_hard_exceed.is_none());
}
#[test]
fn cmrp_accounting_growth_refused_by_work_is_requested_but_not_accepted() {
    let settings = cmrp_settings(16 * 1024 * 1024);
    let limits = native::RuntimeFactLimits {
        max_work: 1,
        ..settings.native_runtime()
    };
    let record = cmrp_native_record(|| {
        native::run_with_profile(limits, &settings, || {
            native::seed_retained_extent(20)?;
            native::admit_retained_growth(0, Some(16), "growth_work_refused")
        })
    });
    let usage = record.retained.unwrap();
    assert_eq!(usage.max_owner_requested_bytes, Some(20));
    assert_eq!(usage.max_cumulative_requested_bytes, Some(36));
    assert_eq!(usage.accepted_total_bytes, 20);
    assert_eq!(usage.accepted_growth_bytes, 0);
    assert_eq!(record.admitted_bytes, 20);
    assert_eq!(
        observation::reason_labels()[record.reason.unwrap().0],
        "battle_runtime_work_limit"
    );
    assert!(usage.first_hard_exceed.is_none());
}
#[test]
fn cmrp_accounting_checked_overflow_keeps_raw_fault_and_first_site() {
    let settings = cmrp_settings(16 * 1024 * 1024);
    let record = cmrp_native_record(|| {
        native::run_with_profile(settings.native_runtime(), &settings, || {
            native::seed_retained_extent(20)?;
            let first =
                native::admit_retained_growth(0, None, "owner_extent_overflow").unwrap_err();
            let second = native::admit_retained_growth(0, Some(usize::MAX), "later_ignored_site")
                .unwrap_err();
            let GameError::BattleRuntimeFault(second_fault) = second else {
                panic!("typed sticky fault")
            };
            let GameError::BattleRuntimeFault(fault) = first else {
                panic!("typed native fault")
            };
            assert_eq!(
                fault, second_fault,
                "the original typed fault remains sticky"
            );
            assert_eq!(fault.code, native::RuntimeFaultCode::RetainedByteLimit);
            assert_eq!(fault.site, "owner_extent_overflow");
            assert_eq!(
                fault.requested,
                Some(u64::MAX),
                "old raw fault representation is unchanged"
            );
            Err(GameError::BattleRuntimeFault(fault))
        })
    });
    let usage = record.retained.unwrap();
    assert!(usage.owner_request_overflowed);
    let exceed = usage.first_hard_exceed.unwrap();
    assert_eq!(exceed.site, "owner_extent_overflow");
    assert_eq!(exceed.requested_bytes, None);
    assert!(exceed.overflowed);
    assert_eq!(usage.accepted_total_bytes, 20);
}
#[test]
fn cmrp_accounting_unscoped_capture_is_missing_and_candidate_extent_is_not_commit() {
    let missing =
        cmrp_native_record(|| native::admit_retained_growth(0, Some(16), "inactive_projection"));
    assert!(missing.retained.is_none());
    assert!(missing.max_battle_owner_extent_bytes.is_none());
    let record = cmrp_native_record(|| {
        native::run(Default::default(), || {
            native::seed_retained_extent(20)?;
            observation::extent(80);
            observation::extent(30);
            Err(GameError::InvalidAction)
        })
    });
    assert_eq!(record.max_battle_owner_extent_bytes, Some(80));
    assert_eq!(record.peak.0[observation::EXTENT], 80);
    assert!(record.committed.is_none());
    assert_eq!(
        record.retained.unwrap().profile_warning_bytes,
        None,
        "raw entry has no trusted profile metadata"
    );
    assert_eq!(record.retained.unwrap().accepted_total_bytes, 20);
}

#[test]
fn cmrp_identity_generation_discard_commit_private_fork_and_persistence_rollback_follow_owner() {
    let default_data = ValidatedGameData::load_live_embedded().unwrap();
    let development_data = ValidatedGameData::load_live_embedded_with_runtime_safety(Arc::new(
        cmrp_settings(16 * 1024 * 1024),
    ))
    .unwrap();
    let mut app = CoreApplication::new(default_data, 41);
    let before = app.measurement_identity();
    let expected = CoreApplication::new(development_data.clone(), 41).measurement_identity();
    assert_eq!(before.content_sha256, expected.content_sha256);
    assert_ne!(
        before.effective_profile_sha256,
        expected.effective_profile_sha256
    );
    assert_eq!(expected.profile_revision, Some(1));
    assert_eq!(expected.selected_profile_hard_bytes, 16 * 1024 * 1024);
    assert_eq!(expected.profile_warning_bytes, 8 * 1024 * 1024);
    assert_eq!(expected.run_seed, 41);
    let (receiver, reader) = observe(&mut app, 103);
    assert_eq!(
        reader.snapshot(false).unwrap().current.owner_extent,
        observation::NativeOwnerExtent::Unmeasured
    );
    set_amount(&mut app, 8).unwrap();
    let initial = receiver.try_recv().unwrap();
    assert_eq!(
        initial.committed.unwrap().owner_extent,
        observation::NativeOwnerExtent::NoNativeOwner
    );
    assert_eq!(initial.started_identity, Some(before));
    assert_eq!(initial.committed.unwrap().identity, Some(before));
    assert!(
        initial.retained.is_none(),
        "non-battle capture remains outside native scope"
    );
    let command = || {
        ApplicationCommand::from_document(json!({"type":"start_new_game","game_mode":crate::game::map::GameMode::Expedition,"replace_active_run":false}).into()).unwrap()
    };
    let prepared = app
        .prepare_command_with_generation(
            Uuid::from_u128(1),
            command(),
            ApplicationExecutionContext::default(),
            Some(development_data.clone()),
        )
        .unwrap();
    assert_eq!(
        reader.snapshot(false).unwrap().current.identity,
        Some(before)
    );
    drop(prepared);
    let discarded = receiver.try_recv().unwrap();
    assert_eq!(discarded.started_identity, Some(before));
    assert_eq!(discarded.prepared.unwrap().identity, Some(expected));
    assert!(discarded.committed.is_none());
    assert_eq!(discarded.outcome, Outcome::Discarded);
    assert_eq!(app.measurement_identity(), before);
    let rollback = app.capture_persistence_rollback().unwrap().unwrap();
    let prepared = app
        .prepare_command_with_generation(
            Uuid::from_u128(1),
            command(),
            ApplicationExecutionContext::default(),
            Some(development_data),
        )
        .unwrap();
    prepared.commit();
    let committed = receiver.try_recv().unwrap();
    assert_eq!(committed.started_identity, Some(before));
    assert_eq!(committed.prepared.unwrap().identity, Some(expected));
    assert_eq!(committed.committed.unwrap().identity, Some(expected));
    assert_eq!(
        reader.snapshot(false).unwrap().current.identity,
        Some(expected)
    );
    assert_eq!(app.measurement_identity(), expected);
    let cloned = app.fork_for_authoring().unwrap();
    assert_eq!(cloned.measurement_identity(), expected);
    assert!(cloned.observer.is_none());
    app.restore_persistence_rollback(rollback).unwrap();
    let restored = receiver.try_recv().unwrap();
    assert_eq!(restored.started_identity, Some(expected));
    assert_eq!(restored.prepared.unwrap().identity, Some(before));
    assert_eq!(restored.committed.unwrap().identity, Some(before));
    assert_eq!(app.measurement_identity(), before);
    assert_eq!(
        reader.snapshot(false).unwrap().current.identity,
        Some(before)
    );
    assert!(restored.retained.is_none());
    drop(app);
    assert!(
        !reader.is_alive(),
        "a live clone does not retain the observer lease"
    );
    assert_eq!(cloned.measurement_identity(), expected);
}
