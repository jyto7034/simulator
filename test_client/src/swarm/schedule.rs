use crate::swarm::seed::rng_for;
use rand::Rng;

pub type SpawnScheduleMs = Vec<u64>;

/// Generate a constant-rate spawn schedule with optional per-player jitter (in ms).
///
/// Deterministic: fully determined by (global_seed, namespace, player_count, cps, jitter_ms).
/// - global_seed: test seed
/// - namespace: optional disambiguator (e.g., group name). Use "spawn" if unsure.
/// - player_count: total players to spawn
/// - cps: target average creations per second
/// - jitter_ms: uniform random jitter added to each spawn time in [0, jitter_ms]
pub fn spawn_schedule_constant(
    global_seed: u64,
    namespace: &str,
    player_count: u64,
    cps: f64,
    jitter_ms: u64,
) -> SpawnScheduleMs {
    assert!(cps > 0.0, "cps must be > 0");

    let mut schedule = Vec::with_capacity(player_count as usize);
    // Base deterministic times (no jitter): floor(1000 * i / cps)
    for i in 0..player_count {
        let base_ms = ((i as f64) * 1000.0 / cps).floor() as u64;
        // Per-index RNG to avoid cross-correlation and to isolate jitter stream
        let mut rng = rng_for(global_seed, &format!("spawn/{}/{}", namespace, i));
        let jitter = if jitter_ms == 0 {
            0
        } else {
            rng.gen_range(0..=jitter_ms)
        };
        schedule.push(base_ms + jitter);
    }
    schedule
}

/// Utility: convert a schedule (ms offsets) into per-second buckets counts for visualization.
/// Returns a vector y[t] with length = duration_secs, where y[t] is the number of spawns in second t.
pub fn bucketize_per_second(schedule_ms: &SpawnScheduleMs, duration_secs: u64) -> Vec<u64> {
    let mut buckets = vec![0u64; duration_secs as usize];
    for &ms in schedule_ms.iter() {
        let sec = (ms / 1000) as usize;
        if sec < buckets.len() {
            buckets[sec] += 1;
        }
    }
    buckets
}
