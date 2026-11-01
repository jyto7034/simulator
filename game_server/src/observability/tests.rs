use super::*;
use game_core::{
    application::{ApplicationAdminCommand, ApplicationSnapshotKind},
    game::data::GameDataBuilder,
};
fn setup() -> (Observability, ObservationReceiver, Worker, PrometheusHandle) {
    let (sender, receiver) = core::channel();
    let control = Arc::new(Control {
        sender,
        sessions: Mutex::new(Vec::new()),
        next_session: AtomicU64::new(1),
        registration_drops: AtomicU64::new(0),
        detail: Mutex::new(Detail::default()),
        stop: AtomicBool::new(false),
        stopped: Notify::new(),
        stalled: AtomicBool::new(false),
    });
    let recorder = PrometheusBuilder::new()
        .set_buckets(&[1., 10., 100., 1000.])
        .unwrap()
        .build_recorder();
    let handle = recorder.handle();
    let worker = Worker {
        handles: Handles::new(&recorder),
        owners: HashMap::new(),
        gate: RetirementGate::default(),
    };
    (Observability { control }, receiver, worker, handle)
}
fn app() -> CoreApplication {
    static DATA: std::sync::OnceLock<game_core::game::data::ValidatedGameData> =
        std::sync::OnceLock::new();
    let data = DATA.get_or_init(|| GameDataBuilder::empty().build_arc().into());
    CoreApplication::new(data.clone(), 7)
}
fn command(app: &mut CoreApplication, n: u32) {
    app.execute_admin(
        ApplicationAdminCommand::from_document(
            serde_json::json!({"type":"admin_set_enkephalin","amount":n}).into(),
        )
        .unwrap(),
    )
    .unwrap();
}
fn sample(session: u64, battle: u128, value: u64) -> SessionSnapshot {
    SessionSnapshot {
        session,
        commits: 1,
        current: core::CommittedSnapshot {
            battle: Some(Uuid::from_u128(battle)),
            usage: Usage([value; USAGE_FIELDS]),
            owner_extent: core::NativeOwnerExtent::Battle { bytes: value },
            identity: None,
        },
        interval_peak: Usage([value * 2; USAGE_FIELDS]),
    }
}
#[test]
fn actual_queue_batch_dedup_and_dead_lease_cleanup_do_not_depend_on_end_messages() {
    let (obs, receiver, mut worker, handle) = setup();
    let mut app = app();
    assert!(obs.attach(&mut app));
    for n in 0..core::QUEUE_CAPACITY + 37 {
        command(&mut app, n as u32);
    }
    worker.sessions(&obs.control, false);
    let first = receiver.try_recv().unwrap();
    assert!(worker.consume(first));
    assert!(!worker.consume(first));
    handle.run_upkeep();
    assert_eq!(worker.batch(&receiver, &handle, &obs.control), WORKER_BATCH);
    worker.handles.counters(&receiver, 0);
    let scrape = handle.render();
    assert!(scrape.contains("core_runtime_commits_total 1061"));
    assert!(scrape.contains("core_runtime_observation_dropped_total 37"));
    assert!(
        scrape.contains("core_runtime_work_count{operation=\"admin\"} 129"),
        "{scrape}"
    );
    drop(app);
    assert!(worker.sessions(&obs.control, true).is_empty());
    assert!(worker.owners.is_empty());
    for _ in 0..20 {
        worker.batch(&receiver, &handle, &obs.control);
    }
    assert!(obs.control.sessions.lock().unwrap().is_empty());
    assert!(handle
        .render()
        .contains("core_runtime_work_count{operation=\"admin\"} 129"));
}
#[test]
fn selection_switch_ring_rollover_end_and_aggregate_subtraction_are_exact() {
    let (obs, _, worker, handle) = setup();
    worker.snapshot(&obs.control, vec![sample(1, 101, 3), sample(2, 202, 5)]);
    obs.select(Some(1)).unwrap();
    worker.snapshot(&obs.control, vec![sample(1, 101, 4), sample(2, 202, 6)]);
    let scrape = handle.render();
    assert!(scrape.contains("core_runtime_active_battles 2"));
    assert!(scrape.contains("core_runtime_usage{quantity=\"units\",statistic=\"sum\"} 10"));
    assert!(scrape.contains(
        "core_runtime_selected_usage{slot=\"0\",quantity=\"units\",statistic=\"current\"} 4"
    ));
    obs.select(Some(2)).unwrap();
    for _ in 0..RING_CAPACITY + 11 {
        worker.snapshot(&obs.control, vec![sample(1, 101, 4), sample(2, 202, 7)]);
    }
    {
        let detail = obs.control.detail.lock().unwrap();
        assert_eq!(detail.ring.len(), RING_CAPACITY);
        assert_eq!(detail.generation, 2);
        assert!(detail.ring.iter().all(|row| row.session == Some(2)));
    }
    worker.snapshot(&obs.control, vec![sample(1, 101, 4)]);
    {
        let detail = obs.control.detail.lock().unwrap();
        assert!(detail.selected_session.is_none());
        assert_eq!(detail.ring.back().unwrap().boundary, Some("ended"));
    }
    let scrape = handle.render();
    assert!(scrape.contains("core_runtime_active_battles 1"));
    assert!(scrape.contains("core_runtime_selected_active{slot=\"0\"} 0"));
    assert!(
        !scrape.contains("00000000-0000"),
        "identities belong only in detail metadata"
    );
    worker.snapshot(&obs.control, vec![]);
    assert!(handle.render().contains("core_runtime_active_battles 0"));
    assert!(obs.select(Some(987)).is_err());
}
#[test]
fn stopped_scrapes_and_pinned_retirement_cannot_accumulate_histogram_batches() {
    let (obs, receiver, mut worker, handle) = setup();
    let mut app = app();
    obs.attach(&mut app);
    worker.sessions(&obs.control, false);
    for n in 0..WORKER_BATCH {
        command(&mut app, n as u32);
    }
    let blocker = crossbeam_epoch::pin();
    assert_eq!(worker.batch(&receiver, &handle, &obs.control), WORKER_BATCH);
    for n in 0..core::QUEUE_CAPACITY + 3 {
        command(&mut app, n as u32);
        assert_eq!(worker.batch(&receiver, &handle, &obs.control), 0);
    }
    assert_eq!(receiver.counters().dropped, 3);
    let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
    drop(blocker);
    let mut resumed = 0;
    for _ in 0..100 {
        resumed += worker.batch(&receiver, &handle, &obs.control);
    }
    assert!(resumed > 0);
    assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
    assert!(handle
        .render()
        .contains("core_runtime_work_count{operation=\"admin\"} 1152"));
}
#[test]
fn churn_registry_storage_tracks_live_sessions_and_registration_never_waits() {
    let (obs, _, mut worker, _) = setup();
    for _ in 0..3000 {
        let mut app = app();
        assert!(obs.attach(&mut app));
        worker.sessions(&obs.control, false);
    }
    worker.sessions(&obs.control, false);
    assert!(worker.owners.is_empty());
    let guard = obs.control.sessions.lock().unwrap();
    let mut app = app();
    assert!(!obs.attach(&mut app));
    drop(guard);
    command(&mut app, 2);
    assert_eq!(obs.control.registration_drops.load(Ordering::Relaxed), 1);
    assert!(obs.control.sessions.lock().unwrap().capacity() <= 64);
    assert!(worker.owners.capacity() <= 64);
}
#[test]
fn config_is_opt_in_separate_and_loopback_only() {
    assert!(!ObservationConfig::default().enabled);
    assert!(Observability::start(ObservationConfig::default())
        .unwrap()
        .is_none());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("observation.ron");
    std::fs::write(&path, "(enabled: true, metrics_listen: \"0.0.0.0:9109\")").unwrap();
    assert!(ObservationConfig::from_path(&path).is_err());
    std::fs::write(&path, "(enabled: true, snapshot_ms: 0)").unwrap();
    assert!(ObservationConfig::from_path(&path).is_err());
    std::fs::write(&path, "(enabled: true, unknown: 1)").unwrap();
    assert!(ObservationConfig::from_path(&path).is_err());
    std::fs::write(&path, "(enabled: true)").unwrap();
    let config = ObservationConfig::from_path(&path).unwrap();
    assert_eq!(config.snapshot_ms, 200);
}

// Test-only process heap accounting. The allocator does only integer atomics;
// it does not register metrics, format strings, invoke the collector or log.
#[cfg(not(feature = "runtime-measurement"))]
struct CountedAllocator;
#[cfg(not(feature = "runtime-measurement"))]
static LIVE_BYTES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[cfg(not(feature = "runtime-measurement"))]
unsafe impl std::alloc::GlobalAlloc for CountedAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let pointer = std::alloc::System.alloc(layout);
        if !pointer.is_null() {
            LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: std::alloc::Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        std::alloc::System.dealloc(pointer, layout);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: std::alloc::Layout, size: usize) -> *mut u8 {
        let result = std::alloc::System.realloc(pointer, layout, size);
        if !result.is_null() {
            LIVE_BYTES.fetch_add(size, Ordering::Relaxed);
            LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        result
    }
}
#[cfg(not(feature = "runtime-measurement"))]
#[global_allocator]
static ALLOCATOR: CountedAllocator = CountedAllocator;

#[cfg(feature = "runtime-measurement")]
#[global_allocator]
static ALLOCATOR: crate::runtime_measurement::CountingAllocator<'static, std::alloc::System> =
    crate::runtime_measurement::process_allocator();
fn live_bytes() -> usize {
    #[cfg(feature = "runtime-measurement")]
    {
        crate::runtime_measurement::allocator::process_live_bytes()
    }
    #[cfg(not(feature = "runtime-measurement"))]
    {
        LIVE_BYTES.load(Ordering::Relaxed)
    }
}

#[test]
#[ignore = "heap accounting runs alone with --ignored --test-threads=1"]
fn exporter_heap_is_bounded_without_scrapes_and_retires_on_idle() {
    let (obs, receiver, mut worker, handle) = setup();
    let mut app = app();
    obs.attach(&mut app);
    command(&mut app, 1);
    worker.sessions(&obs.control, false);
    let mut summary = receiver.try_recv().unwrap();
    let operations: Vec<_> = core::Operation::labels()
        .into_iter()
        .map(|(op, _)| op)
        .collect();
    handle.run_upkeep();
    drop(handle.render());
    for _ in 0..100 {
        worker.gate.poll();
    }
    let baseline = live_bytes();
    let blocker = crossbeam_epoch::pin();
    for _ in 0..WORKER_BATCH {
        summary.sequence += 1;
        assert!(worker.consume(summary));
    }
    handle.run_upkeep();
    worker.gate.after_upkeep();
    let held_start = live_bytes();
    for _ in 0..10000 {
        assert!(!worker.gate.poll());
    }
    let held_end = live_bytes();
    assert_eq!(
        held_start, held_end,
        "a pinned reader suspends allocation of new histogram batches"
    );
    drop(blocker);
    let mut snapshots = [0; 4];
    for batch in 0..2000 {
        let mut ready = false;
        for _ in 0..1000 {
            if worker.gate.poll() {
                ready = true;
                break;
            }
        }
        assert!(
            ready,
            "isolated collector must reclaim after readers quiesce"
        );
        for i in 0..WORKER_BATCH {
            summary.sequence += 1;
            summary.operation = operations[(batch * WORKER_BATCH + i) % operations.len()];
            assert!(worker.consume(summary));
        }
        handle.run_upkeep();
        worker.gate.after_upkeep();
        if (batch + 1) % 500 == 0 {
            for _ in 0..100 {
                worker.gate.poll();
            }
            snapshots[batch / 500] = live_bytes();
        }
    }
    for _ in 0..1000 {
        handle.run_upkeep();
        worker.gate.poll();
    }
    let idle = live_bytes();
    // One raw block per histogram per partial batch, plus full blocks; this
    // conservative byte bound includes header/alignment and deferred batches.
    let raw_block_upper = 16 * usize::BITS as usize + 64;
    let deferred_bound = (core::OPERATION_COUNT * 4 + WORKER_BATCH * 4) * raw_block_upper * 4;
    assert!(snapshots.iter().max().unwrap() - snapshots.iter().min().unwrap() <= deferred_bound);
    assert!(idle <= baseline + deferred_bound);
    println!("S2_HEAP {{\"baseline_live\":{baseline},\"held_start\":{held_start},\"held_end\":{held_end},\"samples\":{},\"plateaus\":{snapshots:?},\"idle_live\":{idle},\"deferred_bound\":{deferred_bound}}}", (2000+1)*WORKER_BATCH*4);
}

#[test]
fn a_contended_atomic_read_keeps_the_last_commit_and_selection_until_a_real_end() {
    let (obs, _, worker, handle) = setup();
    let mut owner = SessionOwner::default();
    worker.snapshot(
        &obs.control,
        vec![owner.sample(Some(sample(1, 101, 4))).unwrap()],
    );
    obs.select(Some(1)).unwrap();
    let previous = owner.sample(None).unwrap();
    assert_eq!(previous.current.usage, Usage([4; USAGE_FIELDS]));
    assert_eq!(previous.interval_peak, previous.current.usage);
    worker.snapshot(&obs.control, vec![previous]);
    assert_eq!(obs.control.detail.lock().unwrap().selected_session, Some(1));
    assert!(handle
        .render()
        .contains("core_runtime_selected_active{slot=\"0\"} 1"));
    worker.snapshot(&obs.control, vec![]);
    assert!(obs
        .control
        .detail
        .lock()
        .unwrap()
        .selected_session
        .is_none());
}

#[test]
fn cmrp_registry_cap_raw_ring_retired_records_and_queue_drops_preserve_gameplay() {
    let (obs, receiver, mut worker, handle) = setup();
    let mut applications = Vec::new();
    for _ in 0..MAX_OBSERVED_SESSIONS {
        let mut application = app();
        assert!(obs.attach(&mut application));
        applications.push(application);
    }
    let mut detached = app();
    let mut baseline = app();
    assert!(!obs.attach(&mut detached));
    assert_eq!(obs.control.registration_drops.load(Ordering::Relaxed), 1);
    for i in 0..core::QUEUE_CAPACITY + 19 {
        command(&mut applications[0], i as u32);
        command(&mut detached, i as u32);
        command(&mut baseline, i as u32);
        if i == core::QUEUE_CAPACITY - 2 {
            for application in [&mut applications[0], &mut detached, &mut baseline] {
                let error = application
                    .execute(
                        Uuid::nil(),
                        game_core::application::ApplicationCommand::from_document(
                            serde_json::json!({"type":"withdraw_unit","employee_uuid":Uuid::nil()})
                                .into(),
                        )
                        .unwrap(),
                        game_core::application::ApplicationExecutionContext::default(),
                    )
                    .unwrap_err();
                assert_eq!(
                    error.kind(),
                    game_core::application::ApplicationErrorKind::InvalidAction
                );
            }
        }
    }
    for kind in ApplicationSnapshotKind::ALL {
        assert_eq!(
            detached.snapshot(*kind).unwrap(),
            baseline.snapshot(*kind).unwrap()
        );
        assert_eq!(
            applications[0].snapshot(*kind).unwrap(),
            baseline.snapshot(*kind).unwrap()
        );
    }
    assert_eq!(receiver.counters().dropped, 20);
    let sessions = worker.sessions(&obs.control, true);
    assert_eq!(sessions.len(), MAX_OBSERVED_SESSIONS);
    assert_eq!(worker.owners.len(), MAX_OBSERVED_SESSIONS);
    worker.snapshot(&obs.control, sessions);
    // Terminal/refused operations queued before retirement must remain raw
    // diagnostics even when their weak lease has already expired.
    drop(applications);
    assert!(worker.sessions(&obs.control, true).is_empty());
    for _ in 0..core::QUEUE_CAPACITY / WORKER_BATCH {
        worker.batch(&receiver, &handle, &obs.control);
    }
    let detail = obs.control.detail.lock().unwrap();
    assert_eq!(detail.raw_operations.len(), RING_CAPACITY);
    assert_eq!(detail.raw_operations_received, core::QUEUE_CAPACITY as u64);
    assert_eq!(
        detail.raw_operations_overwritten,
        (core::QUEUE_CAPACITY - RING_CAPACITY) as u64
    );
    assert_eq!(detail.queue_drops, 20);
    assert_eq!(detail.registration_drops, 1);
    assert!(detail
        .raw_operations
        .iter()
        .all(|s| s.started_identity.is_some()));
    assert!(detail
        .raw_operations
        .iter()
        .any(|s| s.outcome == core::Outcome::Rejected));
    drop(detail);
    assert!(worker.owners.is_empty());
    assert!(obs.control.sessions.lock().unwrap().is_empty());
    assert!(
        obs.attach(&mut detached),
        "dead leases release registration capacity"
    );
}
#[test]
fn cmrp_derived_detail_sessions_and_error_strings_are_bounded() {
    let (obs, _, worker, _) = setup();
    worker.snapshot(
        &obs.control,
        (0..MAX_OBSERVED_SESSIONS + 17)
            .map(|i| sample(i as u64, i as u128 + 1, 1))
            .collect(),
    );
    let detail = obs.control.detail.lock().unwrap();
    assert_eq!(detail.sessions.len(), MAX_OBSERVED_SESSIONS);
    assert_eq!(detail.snapshot_sessions_truncated, 17);
    drop(detail);
    let text = bounded_error("한".repeat(ERROR_BYTE_CAP));
    assert!(text.len() <= ERROR_BYTE_CAP);
    assert!(std::str::from_utf8(text.as_bytes()).is_ok());
    assert_eq!(bounded_error("small"), "small");
}

#[test]
fn relic_observation_actual_host_layout_and_bounded_raw_retention() {
    use std::mem::size_of;
    let (obs, receiver, mut worker, handle) = setup();
    let mut application = app();
    assert!(obs.attach(&mut application));
    for n in 0..RING_CAPACITY + 3 {
        command(&mut application, n as u32);
    }
    // Astra test-author correction to the original Sol fixture: batch returns
    // accepted session-owner samples, not raw rows drained. This fixture has
    // no sampled session owners, so consume returns false for every raw row.
    for _ in 0..(RING_CAPACITY + 3).div_ceil(WORKER_BATCH) {
        assert_eq!(worker.batch(&receiver, &handle, &obs.control), 0);
    }
    let detail = obs.control.detail.lock().unwrap();
    assert_eq!(detail.raw_operations_received, (RING_CAPACITY + 3) as u64);
    assert_eq!(detail.raw_operations.len(), RING_CAPACITY);
    assert_eq!(detail.raw_operations_overwritten, 3);
    let json = serde_json::to_value(&*detail).unwrap();
    assert!(!json["raw_operations"][0]
        .as_object()
        .unwrap()
        .contains_key("max_relic_owner_extent_bytes"));
    println!("relic host layout operation={} committed={} session={} ring_row={} detail={} session_owner={} ring_cap={} raw_backing_capacity={} raw_backing_bytes={}",
        size_of::<core::OperationSummary>(), size_of::<core::CommittedSnapshot>(), size_of::<core::SessionSnapshot>(), size_of::<RingRow>(), size_of::<Detail>(), size_of::<SessionOwner>(), RING_CAPACITY,
        detail.raw_operations.capacity(), detail.raw_operations.capacity().checked_mul(size_of::<core::OperationSummary>()).unwrap());
}
