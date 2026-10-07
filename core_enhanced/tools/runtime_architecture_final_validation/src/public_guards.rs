use std::{
    fs,
    hint::black_box,
    path::PathBuf,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "allocation-observer")]
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use game_core::{
    application::{
        ApplicationAdminCommand, ApplicationCommand, ApplicationDocument,
        ApplicationExecutionContext, ApplicationSnapshotKind, CoreApplication,
    },
    game::battle::{
        event_log::{BattleEventCause, BattleEventLog, BattleEventLogEntry, BattleLogEvent},
        types::BattleWinner,
    },
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SAMPLE_COUNT: usize = 101;
const ALLOCATION_SAMPLE_COUNT: usize = 5;

#[cfg(feature = "allocation-observer")]
struct ObservedSystemAllocator;

#[cfg(feature = "allocation-observer")]
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static DEALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static REALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "allocation-observer")]
static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "allocation-observer")]
fn record_allocation(size: usize) {
    if !TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        return;
    }
    let current = LIVE_BYTES.fetch_add(size as u64, Ordering::SeqCst) + size as u64;
    PEAK_LIVE_BYTES.fetch_max(current, Ordering::SeqCst);
    ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[cfg(feature = "allocation-observer")]
fn record_deallocation(size: usize) {
    if !TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
        return;
    }
    LIVE_BYTES.fetch_sub(size as u64, Ordering::SeqCst);
    DEALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
}

#[cfg(feature = "allocation-observer")]
unsafe impl GlobalAlloc for ObservedSystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            record_allocation(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record_deallocation(layout.size());
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() && TRACK_ALLOCATIONS.load(Ordering::Relaxed) {
            match new_size.cmp(&layout.size()) {
                std::cmp::Ordering::Greater => {
                    let added = new_size - layout.size();
                    let current =
                        LIVE_BYTES.fetch_add(added as u64, Ordering::SeqCst) + added as u64;
                    PEAK_LIVE_BYTES.fetch_max(current, Ordering::SeqCst);
                }
                std::cmp::Ordering::Less => {
                    LIVE_BYTES.fetch_sub((layout.size() - new_size) as u64, Ordering::SeqCst);
                }
                std::cmp::Ordering::Equal => {}
            }
            REALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        new_ptr
    }
}

#[cfg(feature = "allocation-observer")]
#[global_allocator]
static GLOBAL_ALLOCATOR: ObservedSystemAllocator = ObservedSystemAllocator;

#[cfg(feature = "allocation-observer")]
fn set_allocation_tracking(enabled: bool) {
    TRACK_ALLOCATIONS.store(enabled, Ordering::Relaxed);
}

#[cfg(not(feature = "allocation-observer"))]
fn set_allocation_tracking(_enabled: bool) {}

#[derive(Clone, Copy)]
struct AllocationSnapshot {
    live_bytes: u64,
    allocation_count: u64,
    deallocation_count: u64,
    reallocation_count: u64,
}

impl AllocationSnapshot {
    #[cfg(feature = "allocation-observer")]
    fn begin() -> Self {
        let live_bytes = LIVE_BYTES.load(Ordering::SeqCst);
        PEAK_LIVE_BYTES.store(live_bytes, Ordering::SeqCst);
        Self {
            live_bytes,
            allocation_count: ALLOCATION_COUNT.load(Ordering::SeqCst),
            deallocation_count: DEALLOCATION_COUNT.load(Ordering::SeqCst),
            reallocation_count: REALLOCATION_COUNT.load(Ordering::SeqCst),
        }
    }

    #[cfg(not(feature = "allocation-observer"))]
    fn begin() -> Self {
        Self {
            live_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
            reallocation_count: 0,
        }
    }

    #[cfg(feature = "allocation-observer")]
    fn finish(self) -> HeapAllocationObservation {
        let end_live_bytes = LIVE_BYTES.load(Ordering::SeqCst);
        let peak_live_bytes = PEAK_LIVE_BYTES.load(Ordering::SeqCst);
        HeapAllocationObservation {
            baseline_live_bytes: self.live_bytes,
            peak_live_bytes,
            peak_delta_bytes: peak_live_bytes.saturating_sub(self.live_bytes),
            end_live_bytes,
            retained_delta_bytes: i128::from(end_live_bytes) - i128::from(self.live_bytes),
            allocation_count: ALLOCATION_COUNT.load(Ordering::SeqCst) - self.allocation_count,
            deallocation_count: DEALLOCATION_COUNT.load(Ordering::SeqCst) - self.deallocation_count,
            reallocation_count: REALLOCATION_COUNT.load(Ordering::SeqCst) - self.reallocation_count,
        }
    }

    #[cfg(not(feature = "allocation-observer"))]
    fn finish(self) -> HeapAllocationObservation {
        HeapAllocationObservation {
            baseline_live_bytes: self.live_bytes,
            peak_live_bytes: self.live_bytes,
            peak_delta_bytes: 0,
            end_live_bytes: self.live_bytes,
            retained_delta_bytes: 0,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            reallocation_count: self.reallocation_count,
        }
    }
}

#[derive(Serialize)]
struct HeapAllocationObservation {
    baseline_live_bytes: u64,
    peak_live_bytes: u64,
    peak_delta_bytes: u64,
    end_live_bytes: u64,
    retained_delta_bytes: i128,
    allocation_count: u64,
    deallocation_count: u64,
    reallocation_count: u64,
}

#[derive(Serialize)]
struct Receipt {
    schema_version: u32,
    workload_version: &'static str,
    generated_at_unix_ms: u128,
    tool_source_sha256: String,
    allocation_observer_enabled: bool,
    latency_gate_applicable: bool,
    allocation_gate_applicable: bool,
    run_snapshot: LatencyGuard,
    checkpoint_export: LatencyGuard,
    bounded_history_query: HistoryGuard,
    transaction_staging: StagingGuard,
}

#[derive(Serialize)]
struct LatencyGuard {
    sample_count: usize,
    nearest_rank_p95_ns: u128,
    absolute_cap_ns: u128,
    absolute_cap_pass: bool,
    encoded_bytes: usize,
    encoded_byte_cap: usize,
    encoded_byte_cap_pass: bool,
    exact_repeatable: bool,
    raw_elapsed_ns: Vec<u128>,
}

#[derive(Serialize)]
struct HistoryGuard {
    small_retained_entries: usize,
    large_retained_entries: usize,
    result_entries: usize,
    small_nearest_rank_p95_ns: u128,
    large_nearest_rank_p95_ns: u128,
    scaling_limit_numerator: u128,
    scaling_limit_denominator: u128,
    scaling_pass: bool,
    exact_repeatable: bool,
    all_caps_respected: bool,
    small_raw_elapsed_ns: Vec<u128>,
    large_raw_elapsed_ns: Vec<u128>,
}

#[derive(Serialize)]
struct StagingGuard {
    low_retained_entries: usize,
    high_retained_entries: usize,
    low_nearest_rank_p95_ns: u128,
    high_nearest_rank_p95_ns: u128,
    scaling_limit_numerator: u128,
    scaling_limit_denominator: u128,
    additive_scheduler_slack_ns: u128,
    retained_history_independent_pass: bool,
    allocation_retained_history_independent_pass: bool,
    rejection_rollback_exact: bool,
    low_raw_elapsed_ns: Vec<u128>,
    high_raw_elapsed_ns: Vec<u128>,
    low_allocation_raw_samples: Vec<HeapAllocationObservation>,
    high_allocation_raw_samples: Vec<HeapAllocationObservation>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn nearest_rank_p95(values: &[u128]) -> u128 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (95 * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn command(value: Value) -> ApplicationCommand {
    ApplicationCommand::from_document(ApplicationDocument::new(value))
        .expect("public guard command must validate")
}

fn admin_command(value: Value) -> ApplicationAdminCommand {
    ApplicationAdminCommand::from_document(ApplicationDocument::new(value))
        .expect("public guard admin command must validate")
}

fn live_run_with_checkpoint() -> (CoreApplication, Uuid) {
    let game_data = game_core::game::data::ValidatedGameData::load_live_embedded()
        .expect("embedded live data must validate");
    let starter_ids = ["maintenance_tech", "records_clerk", "field_medic"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let player_id = Uuid::from_u128(0xCA08_0001);
    let mut application = CoreApplication::new(game_data, 0xCA08_0002);
    let start = application
        .execute(
            player_id,
            command(json!({
                "type": "start_new_game",
                "game_mode": "Standard",
                "replace_active_run": false,
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("public guard run must start");
    let visual_novel = &start.payload().as_value()["visual_novel"];
    let session_id = visual_novel["session_id"]
        .as_str()
        .expect("run start Visual Novel session")
        .to_owned();
    let current_scene_id = visual_novel["current_scene_id"]
        .as_str()
        .expect("run start Visual Novel scene")
        .to_owned();
    application
        .execute(
            player_id,
            command(json!({
                "type": "advance_visual_novel",
                "session_id": session_id,
                "current_scene_id": current_scene_id,
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("public guard briefing must close");
    application
        .execute(
            player_id,
            command(json!({
                "type": "select_starter_employees",
                "employee_definition_ids": starter_ids,
            })),
            ApplicationExecutionContext::default(),
        )
        .expect("public guard starters must commit");
    application
        .execute_admin(admin_command(json!({ "type": "admin_enter_save_point" })))
        .expect("public guard SavePoint must open");
    application
        .execute(
            player_id,
            command(json!({ "type": "complete_node" })),
            ApplicationExecutionContext::default(),
        )
        .expect("public guard SavePoint must complete");
    assert!(application
        .export_active_run_checkpoint()
        .expect("public guard checkpoint export")
        .is_some());
    (application, player_id)
}

fn enter_available_combat(
    application: &mut CoreApplication,
    player_id: Uuid,
) -> Result<(), String> {
    let snapshot = application
        .snapshot(ApplicationSnapshotKind::Run)
        .map_err(|error| format!("Run snapshot failed: {}", error.code()))?;
    let value = snapshot.read_model().as_value();
    let selectable = value["map_navigation"]["selectable_node_ids"]
        .as_array()
        .ok_or_else(|| "missing selectable node ids".to_string())?;
    let combat_node_id = value["map"]["nodes"]
        .as_array()
        .ok_or_else(|| "missing map nodes".to_string())?
        .iter()
        .find(|node| {
            node["category"] == "Combat"
                && selectable
                    .iter()
                    .any(|selectable_id| selectable_id == &node["id"])
        })
        .and_then(|node| node["id"].as_str())
        .ok_or_else(|| "no selectable combat node".to_string())?
        .to_owned();
    application
        .execute(
            player_id,
            command(json!({ "type": "select_map_node", "node_id": combat_node_id })),
            ApplicationExecutionContext::default(),
        )
        .map_err(|error| format!("combat selection failed: {}", error.code()))?;
    application
        .execute(
            player_id,
            command(json!({ "type": "confirm_enter_node" })),
            ApplicationExecutionContext::default(),
        )
        .map_err(|error| format!("combat entry failed: {}", error.code()))?;
    Ok(())
}

fn rejected_staging_samples(
    application: &mut CoreApplication,
    player_id: Uuid,
) -> (Vec<u128>, bool) {
    let before = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("pre-rejection Run snapshot");
    let rejected = command(json!({ "type": "confirm_enter_node" }));
    let mut elapsed = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let error = application
            .execute(
                player_id,
                rejected.clone(),
                ApplicationExecutionContext::default(),
            )
            .expect_err("in-battle map confirmation must reject");
        elapsed.push(started.elapsed().as_nanos());
        black_box(error);
    }
    let after = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("post-rejection Run snapshot");
    (elapsed, before == after)
}

fn rejected_staging_allocation_samples(
    application: &mut CoreApplication,
    player_id: Uuid,
) -> Vec<HeapAllocationObservation> {
    let rejected = command(json!({ "type": "confirm_enter_node" }));
    let sample_count = if cfg!(feature = "allocation-observer") {
        ALLOCATION_SAMPLE_COUNT
    } else {
        0
    };
    let mut observations = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        set_allocation_tracking(true);
        let snapshot = AllocationSnapshot::begin();
        let error = application
            .execute(
                player_id,
                rejected.clone(),
                ApplicationExecutionContext::default(),
            )
            .expect_err("in-battle map confirmation must reject");
        black_box(&error);
        drop(error);
        let observation = snapshot.finish();
        set_allocation_tracking(false);
        observations.push(observation);
    }
    observations
}

fn maximum_allocation_operations(samples: &[HeapAllocationObservation]) -> u64 {
    samples
        .iter()
        .map(|sample| sample.allocation_count + sample.reallocation_count)
        .max()
        .unwrap_or(0)
}

fn maximum_peak_delta(samples: &[HeapAllocationObservation]) -> u64 {
    samples
        .iter()
        .map(|sample| sample.peak_delta_bytes)
        .max()
        .unwrap_or(0)
}

fn maximum_retained_delta(samples: &[HeapAllocationObservation]) -> i128 {
    samples
        .iter()
        .map(|sample| sample.retained_delta_bytes)
        .max()
        .unwrap_or(0)
}

fn event_entry(seq: u64) -> BattleEventLogEntry {
    BattleEventLogEntry {
        time_ms: seq,
        seq,
        cause: BattleEventCause::default(),
        source_command_id: None,
        skill_effect_source: None,
        event: BattleLogEvent::BattleEnd {
            winner: BattleWinner::Draw,
        },
    }
}

fn query_samples(log: &BattleEventLog, after_seq: u64) -> (Vec<u128>, String) {
    let expected = log.entries_after_seq(after_seq);
    assert_eq!(expected.len(), 10);
    let expected_bytes = serde_json::to_vec(&expected).unwrap();
    let mut elapsed = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let result = black_box(log.entries_after_seq(after_seq));
        elapsed.push(started.elapsed().as_nanos());
        assert_eq!(serde_json::to_vec(&result).unwrap(), expected_bytes);
    }
    (elapsed, sha256(&expected_bytes))
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: public_guards <output-receipt.json>");
    let (mut application, player_id) = live_run_with_checkpoint();

    let reference_snapshot = application
        .snapshot(ApplicationSnapshotKind::Run)
        .expect("reference Run snapshot");
    let reference_snapshot_bytes = serde_json::to_vec(&reference_snapshot).unwrap();
    let mut snapshot_elapsed = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let snapshot = black_box(
            application
                .snapshot(ApplicationSnapshotKind::Run)
                .expect("measured Run snapshot"),
        );
        let bytes = black_box(serde_json::to_vec(&snapshot).unwrap());
        snapshot_elapsed.push(started.elapsed().as_nanos());
        assert_eq!(bytes, reference_snapshot_bytes);
    }

    let reference_checkpoint = application
        .export_active_run_checkpoint()
        .expect("reference checkpoint export")
        .expect("completed SavePoint checkpoint");
    let reference_checkpoint_bytes = reference_checkpoint.core_payload_json().to_vec();
    let mut checkpoint_elapsed = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let checkpoint = black_box(
            application
                .export_active_run_checkpoint()
                .expect("measured checkpoint export")
                .expect("completed SavePoint checkpoint"),
        );
        checkpoint_elapsed.push(started.elapsed().as_nanos());
        assert_eq!(checkpoint.core_payload_json(), reference_checkpoint_bytes);
    }

    let small = BattleEventLog::from_entries((1..=1_024).map(event_entry).collect());
    let large = BattleEventLog::from_entries((1..=8_192).map(event_entry).collect());
    let (small_elapsed, small_hash) = query_samples(&small, 1_014);
    let (large_elapsed, large_hash) = query_samples(&large, 8_182);
    assert_ne!(
        small_hash, large_hash,
        "query fixtures must retain distinct seqs"
    );
    let small_p95 = nearest_rank_p95(&small_elapsed);
    let large_p95 = nearest_rank_p95(&large_elapsed);
    let scaling_pass = large_p95 <= small_p95.saturating_mul(4).saturating_add(10_000);

    enter_available_combat(&mut application, player_id).expect("enter public guard combat");
    assert_eq!(
        application
            .replace_active_battle_event_log_fixture(128)
            .expect("install low-history fixture"),
        128
    );
    let (low_staging_elapsed, low_rollback_exact) =
        rejected_staging_samples(&mut application, player_id);
    let low_staging_allocations = rejected_staging_allocation_samples(&mut application, player_id);
    assert_eq!(
        application
            .replace_active_battle_event_log_fixture(8_192)
            .expect("install high-history fixture"),
        8_192
    );
    let (high_staging_elapsed, high_rollback_exact) =
        rejected_staging_samples(&mut application, player_id);
    let high_staging_allocations = rejected_staging_allocation_samples(&mut application, player_id);
    let low_staging_p95 = nearest_rank_p95(&low_staging_elapsed);
    let high_staging_p95 = nearest_rank_p95(&high_staging_elapsed);
    let retained_history_independent_pass = high_staging_p95
        <= low_staging_p95
            .saturating_mul(3)
            .div_ceil(2)
            .saturating_add(50_000);
    let allocation_retained_history_independent_pass = !cfg!(feature = "allocation-observer")
        || (maximum_allocation_operations(&high_staging_allocations)
            <= maximum_allocation_operations(&low_staging_allocations)
            && maximum_peak_delta(&high_staging_allocations)
                <= maximum_peak_delta(&low_staging_allocations)
            && maximum_retained_delta(&high_staging_allocations)
                <= maximum_retained_delta(&low_staging_allocations));

    let snapshot_p95 = nearest_rank_p95(&snapshot_elapsed);
    let checkpoint_p95 = nearest_rank_p95(&checkpoint_elapsed);
    let receipt = Receipt {
        schema_version: 2,
        workload_version: "crar-public-performance-guards/2",
        generated_at_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        tool_source_sha256: sha256(include_bytes!("public_guards.rs")),
        allocation_observer_enabled: cfg!(feature = "allocation-observer"),
        latency_gate_applicable: !cfg!(feature = "allocation-observer"),
        allocation_gate_applicable: cfg!(feature = "allocation-observer"),
        run_snapshot: LatencyGuard {
            sample_count: SAMPLE_COUNT,
            nearest_rank_p95_ns: snapshot_p95,
            absolute_cap_ns: 500_000,
            absolute_cap_pass: snapshot_p95 < 500_000,
            encoded_bytes: reference_snapshot_bytes.len(),
            encoded_byte_cap: 64 * 1_024,
            encoded_byte_cap_pass: reference_snapshot_bytes.len() < 64 * 1_024,
            exact_repeatable: true,
            raw_elapsed_ns: snapshot_elapsed,
        },
        checkpoint_export: LatencyGuard {
            sample_count: SAMPLE_COUNT,
            nearest_rank_p95_ns: checkpoint_p95,
            absolute_cap_ns: 100_000,
            absolute_cap_pass: checkpoint_p95 < 100_000,
            encoded_bytes: reference_checkpoint_bytes.len(),
            encoded_byte_cap: 32 * 1_024,
            encoded_byte_cap_pass: reference_checkpoint_bytes.len() < 32 * 1_024,
            exact_repeatable: true,
            raw_elapsed_ns: checkpoint_elapsed,
        },
        bounded_history_query: HistoryGuard {
            small_retained_entries: small.entries.len(),
            large_retained_entries: large.entries.len(),
            result_entries: 10,
            small_nearest_rank_p95_ns: small_p95,
            large_nearest_rank_p95_ns: large_p95,
            scaling_limit_numerator: 4,
            scaling_limit_denominator: 1,
            scaling_pass,
            exact_repeatable: true,
            all_caps_respected: small.entries.len() == 1_024
                && large.entries.len() == 8_192
                && large.encoded_entry_array_bytes()
                    <= game_core::game::battle::event_log::BATTLE_HISTORY_MAX_ENCODED_BYTES
                && large.retained_time_span_ms()
                    <= game_core::game::battle::event_log::BATTLE_HISTORY_MAX_SPAN_MS,
            small_raw_elapsed_ns: small_elapsed,
            large_raw_elapsed_ns: large_elapsed,
        },
        transaction_staging: StagingGuard {
            low_retained_entries: 128,
            high_retained_entries: 8_192,
            low_nearest_rank_p95_ns: low_staging_p95,
            high_nearest_rank_p95_ns: high_staging_p95,
            scaling_limit_numerator: 3,
            scaling_limit_denominator: 2,
            additive_scheduler_slack_ns: 50_000,
            retained_history_independent_pass,
            allocation_retained_history_independent_pass,
            rejection_rollback_exact: low_rollback_exact && high_rollback_exact,
            low_raw_elapsed_ns: low_staging_elapsed,
            high_raw_elapsed_ns: high_staging_elapsed,
            low_allocation_raw_samples: low_staging_allocations,
            high_allocation_raw_samples: high_staging_allocations,
        },
    };
    fs::write(&output, serde_json::to_vec_pretty(&receipt).unwrap())
        .expect("write public guard receipt");
    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    if receipt.latency_gate_applicable {
        assert!(receipt.run_snapshot.absolute_cap_pass);
        assert!(receipt.run_snapshot.encoded_byte_cap_pass);
        assert!(receipt.checkpoint_export.absolute_cap_pass);
        assert!(receipt.checkpoint_export.encoded_byte_cap_pass);
        assert!(receipt.bounded_history_query.scaling_pass);
        assert!(receipt.bounded_history_query.all_caps_respected);
        assert!(
            receipt
                .transaction_staging
                .retained_history_independent_pass
        );
    }
    if receipt.allocation_gate_applicable {
        assert!(
            receipt
                .transaction_staging
                .allocation_retained_history_independent_pass
        );
    }
    assert!(receipt.transaction_staging.rejection_rollback_exact);
}
