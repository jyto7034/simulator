//! Explicit diagnostic of the existing concurrent workload. Keep timers out of
//! the default guard. Fixture constructors and operations come from that guard;
//! this orchestration preserves its lane/session order without adding barriers.
//! Wall times include scheduling; overlapping lane times are not additive.
use super::*;

#[derive(Serialize)]
struct LaneTiming {
    lane: usize,
    entry_offset_ns: u128,
    pin_ns: u128,
    initialize_ns: u128,
    construct_sum_ns: u128,
    start_sum_ns: u128,
    step_ns: u128,
    encode_and_cleanup_ns: u128,
    encode_sum_ns: u128,
    end_offset_ns: u128,
}

#[derive(Serialize)]
struct InspectionSample {
    sample: usize,
    elapsed_ns: u128,
    spawn_loop_end_offset_ns: u128,
    joined_offset_ns: u128,
    sorted_offset_ns: u128,
    event_count: usize,
    serialized_bytes: usize,
    combined_event_log_sha256: String,
    lanes: Vec<LaneTiming>,
}

fn inspect_sample(data: ValidatedGameData, sample: usize) -> InspectionSample {
    let started = Instant::now();
    let (lane_results, lanes, spawn_end, joined, sorted) = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for lane in 0..LANE_COUNT {
            let lane_data = data.clone();
            handles.push(scope.spawn(move || {
                let entered = Instant::now();
                pin_current_thread(lane * 2);
                let pinned = Instant::now();
                let mut construct_sum_ns = 0;
                let mut start_sum_ns = 0;
                let mut sessions = (0..SESSION_COUNT / LANE_COUNT)
                    .map(|slot| {
                        let session = lane * (SESSION_COUNT / LANE_COUNT) + slot;
                        let constructing = Instant::now();
                        let mut core = active_session_core(lane_data.clone(), session);
                        let constructed = Instant::now();
                        let execution = core
                            .start_battle_execution()
                            .expect("start concurrent session battle");
                        let initialized = Instant::now();
                        construct_sum_ns += constructed.duration_since(constructing).as_nanos();
                        start_sum_ns += initialized.duration_since(constructed).as_nanos();
                        (session, core, execution)
                    })
                    .collect::<Vec<_>>();
                let initialized = Instant::now();
                for _ in 0..SESSION_STEP_COUNT {
                    for (_, core, execution) in &mut sessions {
                        black_box(
                            core.step_battle_execution_by(execution, SESSION_STEP_MS)
                                .expect("step concurrent session"),
                        );
                    }
                }
                let stepped = Instant::now();
                let mut encode_sum_ns = 0;
                let results = sessions
                    .into_iter()
                    .map(|(session, core, _)| {
                        let encoding = Instant::now();
                        let bytes = serde_json::to_vec(&core.event_log)
                            .expect("serialize concurrent event log");
                        encode_sum_ns += encoding.elapsed().as_nanos();
                        (session, core.event_log.entries.len(), bytes)
                    })
                    .collect::<Vec<_>>();
                let encoded = Instant::now();
                let timing = LaneTiming {
                    lane,
                    entry_offset_ns: entered.duration_since(started).as_nanos(),
                    pin_ns: pinned.duration_since(entered).as_nanos(),
                    initialize_ns: initialized.duration_since(pinned).as_nanos(),
                    construct_sum_ns,
                    start_sum_ns,
                    step_ns: stepped.duration_since(initialized).as_nanos(),
                    encode_and_cleanup_ns: encoded.duration_since(stepped).as_nanos(),
                    encode_sum_ns,
                    end_offset_ns: encoded.duration_since(started).as_nanos(),
                };
                (results, timing)
            }));
        }
        let spawn_end = started.elapsed().as_nanos();
        let mut lanes = Vec::with_capacity(LANE_COUNT);
        let mut results = handles
            .into_iter()
            .flat_map(|handle| {
                let (results, timing) = handle.join().expect("concurrent lane");
                lanes.push(timing);
                results
            })
            .collect::<Vec<_>>();
        let joined = started.elapsed().as_nanos();
        results.sort_by_key(|(session, _, _)| *session);
        let sorted = started.elapsed().as_nanos();
        (results, lanes, spawn_end, joined, sorted)
    });
    let elapsed_ns = started.elapsed().as_nanos();
    let event_count = lane_results.iter().map(|(_, count, _)| count).sum();
    let serialized_bytes = lane_results.iter().map(|(_, _, bytes)| bytes.len()).sum();
    let mut digest = Sha256::new();
    for (session, _, bytes) in &lane_results {
        digest.update(session.to_le_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    InspectionSample {
        sample, elapsed_ns, spawn_loop_end_offset_ns: spawn_end,
        joined_offset_ns: joined, sorted_offset_ns: sorted,
        event_count, serialized_bytes,
        combined_event_log_sha256: format!("{:x}", digest.finalize()),
        lanes,
    }
}

pub(super) fn run(output: PathBuf) {
    assert!(!cfg!(debug_assertions), "use release for phase inspection");
    assert!(!cfg!(feature = "allocation-observer"), "use plain release");
    let logical_cpus = std::thread::available_parallelism().map(usize::from).unwrap_or(1);
    assert!(logical_cpus >= 31, "same affinity workload requires 31 CPUs");
    let (data, conversion_ns) = preservation_data();
    pin_current_thread(30);
    set_allocation_tracking(false);
    // Untouched workload supplies an independent output reference, not a new
    // latency distribution. A separate traced warm-up precedes the 21 samples.
    let reference = concurrent_sample(data.clone(), 0);
    let _ = inspect_sample(data.clone(), 0);
    let samples = (1..=LATENCY_SAMPLE_COUNT)
        .map(|sample| inspect_sample(data.clone(), sample)).collect::<Vec<_>>();
    let parity = samples.iter().all(|sample| {
        sample.event_count == reference.event_count
            && sample.serialized_bytes == reference.serialized_bytes
            && sample.combined_event_log_sha256 == reference.combined_event_log_sha256
    });
    let receipt = serde_json::json!({
        "schema_version":1, "workload_version":"concurrent-phase-inspection/1",
        "source_workload_version":WORKLOAD_VERSION,
        "performance_pass_claimed":false, "baseline_comparison_applicable":false,
        "allocation_observer_enabled":false, "output_matches_uninstrumented_reference":parity,
        "tool_source_sha256":sha256(include_bytes!("final_performance.rs")),
        "inspection_source_sha256":sha256(include_bytes!("concurrent_inspection.rs")),
        "session_count":SESSION_COUNT, "lane_count":LANE_COUNT,
        "step_count_per_session":SESSION_STEP_COUNT, "step_ms":SESSION_STEP_MS,
        "environment":{"profile":"release", "os":std::env::consts::OS,
            "arch":std::env::consts::ARCH,"logical_cpus":logical_cpus,
            "affinity":"coordinator=30; lanes=0,2,4,6,8,10,12,14"},
        "scope":"Diagnostic wall-clock intervals, not CPU times or a latency gate. Lane phases overlap. Initialization includes construction/start plus collection overhead. Encoding/cleanup includes core destruction. Digest is after elapsed timer, as in default. No barriers, worker pooling, product changes, or workload reduction.",
        "fixture_conversion_elapsed_ns":conversion_ns,
        "uninstrumented_reference":reference,
        "traced_p95_ns":nearest_rank_p95(samples.iter().map(|s|s.elapsed_ns)),
        "raw_samples":samples,
    });
    fs::write(output, serde_json::to_vec_pretty(&receipt).unwrap()).expect("write concurrent inspection");
    assert!(parity, "inspection must preserve uninstrumented workload outputs");
}
