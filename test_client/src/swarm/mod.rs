use actix::Actor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::observer_actor::{message::StartObservation, ObserverActor, Phase, PhaseCondition};
use crate::player_actor::PlayerActor;
use crate::scenario_actor::ScenarioRunnerActor;
use crate::swarm::behavior_mix::{behavior_for_index, BehaviorMixConfig};
use crate::swarm::manifest::{save_swarm_summary, SwarmRunSummary};
use crate::swarm::schedule::spawn_schedule_constant;
use crate::swarm::seed::uuid_for;
use redis::{AsyncCommands, AsyncIter};
use uuid::Uuid;

pub mod config;
fn resolve_seed_and_mix(cfg: &config::SwarmConfig) -> (u64, BehaviorMixConfig) {
    let seed = cfg
        .seed
        .or_else(|| {
            std::env::var("SWARM_SEED")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
        })
        .unwrap_or(42);
    let mix = if let Some(m) = cfg.behavior_mix.clone() {
        m
    } else {
        panic!()
    };
    (seed, mix)
}

pub mod slo;

pub async fn run_swarm(cfg: config::SwarmConfig) -> anyhow::Result<()> {
    info!(
        "Starting swarm: shards={}, players_per_shard={}, duration_secs={}",
        cfg.shards, cfg.players_per_shard, cfg.duration_secs
    );

    // Start a single runner to satisfy ObserverActor's dependency
    let runner = ScenarioRunnerActor::new(vec![]).start();

    // Prefer streaming only violations by default to reduce noise
    if std::env::var("OBSERVER_STREAM_KIND").is_err() {
        std::env::set_var("OBSERVER_STREAM_KIND", "state_violation");
    }

    // Decide deterministic seed and behavior mix
    let (seed, mix) = resolve_seed_and_mix(&cfg);

    // (Legacy run_id/Grafana/reset removed)

    // Spawn observers per shard and orchestrate players
    let mut observer_addrs: Vec<actix::Addr<ObserverActor>> =
        Vec::with_capacity(cfg.shards as usize);
    for shard in 0..cfg.shards {
        let test_name = format!("SwarmShard-{}", shard);
        let test_session_id = Uuid::new_v4().to_string();
        info!("Generated test_session_id for shard {}: {}", shard, test_session_id);
        let players_schedule = std::collections::HashMap::<
            Uuid,
            std::collections::HashMap<Phase, PhaseCondition>,
        >::new();
        let players_phase = std::collections::HashMap::<Uuid, Phase>::new();
        // Expect base server URL like ws://host:port
        let base_url = cfg
            .events_base_url()
            .unwrap_or_else(|| "ws://127.0.0.1:8080".to_string());

        let observer = ObserverActor::new(
            base_url,
            test_name,
            test_session_id.clone(),
            runner.clone(),
            players_schedule,
            players_phase,
        );
        let observer_addr = observer.start();
        observer_addrs.push(observer_addr.clone());

        // Compute deterministic player ids and schedule
        let shard_seed = seed + shard as u64;
        let player_count = cfg.players_per_shard as u64;
        let cps = if cfg.duration_secs > 0 {
            (player_count as f64) / (cfg.duration_secs as f64)
        } else {
            player_count as f64
        };
        let schedule_ms =
            spawn_schedule_constant(shard_seed, "swarm", player_count, cps.max(0.1), 200);

        // Prepare player id list and kick off observation (ids pre-registered for shard filtering)
        let mut ids: Vec<Uuid> = Vec::with_capacity(player_count as usize);
        for i in 0..player_count {
            ids.push(uuid_for(shard_seed, "player", i));
        }
        observer_addr.do_send(StartObservation {
            player_ids: ids.clone(),
        });

        // Spawn players according to schedule
        let t0 = Instant::now();
        // Set global config for server URL/game_mode consumed by PlayerActors
        let ws_url = format!(
            "{}/ws/",
            cfg.events_base_url()
                .unwrap_or_else(|| "ws://127.0.0.1:8080".to_string())
                .trim_end_matches('/')
        );
        std::env::set_var("TEST_CLIENT_WS_URL", ws_url);
        if let Some(gm) = cfg.game_mode.clone() {
            std::env::set_var("TEST_CLIENT_GAME_MODE", gm);
        }
        // Export behavior mix for SLO summary to consume
        // Optional per-shard burst barrier
        let burst_n = if let Some(r) = cfg.burst_ratio {
            if r > 0.0 {
                ((player_count as f64) * r).ceil().max(1.0) as usize
            } else {
                0
            }
        } else {
            0
        };
        let burst_barrier = if burst_n > 0 {
            Some(Arc::new(tokio::sync::Barrier::new(burst_n)))
        } else {
            None
        };

        for (i, ms) in schedule_ms.iter().enumerate() {
            let when_ms = *ms;
            let obs = observer_addr.clone();
            let pid = ids[i];
            let mix_clone = mix.clone();
            let burst_barrier = burst_barrier.clone();
            let test_session_id_clone = test_session_id.clone();
            actix::spawn(async move {
                let target = t0 + Duration::from_millis(when_ms);
                let now = Instant::now();
                // If within burst, wait at barrier for simultaneous start; otherwise honor schedule
                if let Some(bar) = burst_barrier.clone() {
                    if i < burst_n {
                        bar.wait().await;
                    } else {
                        if target > now {
                            tokio::time::sleep(target - now).await;
                        }
                    }
                } else {
                    if target > now {
                        tokio::time::sleep(target - now).await;
                    }
                }
                let behavior = behavior_for_index(shard_seed, i as u64, &mix_clone);
                let actor = PlayerActor::new(obs, Box::new(behavior), pid, test_session_id_clone, true);
                actor.start();
            });
        }
    }

    // Hold the system for the configured duration
    let dur = Duration::from_secs(cfg.duration_secs as u64);
    info!("Swarm run window: {:?}", dur);
    tokio::time::sleep(dur).await;

    warn!("Swarm window elapsed. Evaluating SLO...");
    let metrics_url = cfg
        .match_server_base
        .as_ref()
        .map(|b| slo::metrics_url_from_base(b))
        .unwrap_or_else(|| "http://127.0.0.1:8080/metrics".into());
    let thresholds = slo::SloThresholds {
        p95_match_time_secs: std::env::var("SLO_P95_MATCH_TIME_SECS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(10.0),
        p95_loading_secs: std::env::var("SLO_P95_LOADING_SECS")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(20.0),
        max_violations: std::env::var("SLO_MAX_VIOLATIONS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0),
    };
    let mut overall_ok = true;
    match slo::evaluate_slo(&metrics_url, cfg.game_mode.as_deref(), &thresholds).await {
        Ok(report) => {
            if report.passed {
                info!(
                    "SLO PASS: violations={}, p95_match={:?}s, p95_loading={:?}s",
                    report.violations_total, report.p95_match_time_secs, report.p95_loading_secs
                );
            } else {
                warn!("SLO FAIL: {:?}", report.details);
                overall_ok = false;
            }
            // Persist summary
            let ts = chrono::Utc::now();
            let still_queued_at_end = compute_still_queued().await.unwrap_or_default();

            // Use outcome counts calculated in SLO report
            let outcome_counts = report.outcome_counts.clone();

            let summary = SwarmRunSummary {
                timestamp: ts,
                seed,
                config: cfg.clone(),
                metrics_url: metrics_url.clone(),
                slo: report,
                outcome_counts,
                still_queued_at_end,
            };
            let out_path = cfg.result_path.clone().unwrap_or_else(|| {
                format!("logs/swarm_summary_{}.json", ts.format("%Y%m%d_%H%M%S"),)
            });
            if let Err(e) = save_swarm_summary(std::path::Path::new(&out_path), &summary) {
                warn!("Failed to save swarm summary to {}: {}", out_path, e);
            } else {
                info!("Saved swarm summary to {}", out_path);
            }
        }
        Err(e) => warn!("SLO evaluation failed: {}", e),
    }

    // Gracefully stop observers to close /events/stream sockets
    for obs in observer_addrs.into_iter() {
        obs.do_send(crate::observer_actor::message::StopObservation);
    }
    // Short delay to allow server to register WS closes
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    actix::System::current().stop();
    if overall_ok {
        Ok(())
    } else {
        Err(anyhow::anyhow!("SLO failed"))
    }
}

/// Scan Redis for queue:* keys and compute remaining players per mode.
async fn compute_still_queued() -> anyhow::Result<crate::swarm::manifest::RemainingQueueSummary> {
    let cfg = env::SimulatorConfig::global();
    let r = &cfg.database.redis;
    let url = r.url();
    let client = redis::Client::open(url.as_str())?;
    let mut conn = client.get_async_connection().await?;
    let mut iter: AsyncIter<String> = conn.scan_match("queue:*").await?;
    let mut keys: Vec<String> = Vec::new();
    while let Some(k) = iter.next_item().await {
        keys.push(k);
    }
    use std::collections::HashMap;
    let mut by_mode: HashMap<String, u64> = HashMap::new();
    let mut total: u64 = 0;
    for key in keys.into_iter() {
        let size: i64 = conn.scard(&key).await.unwrap_or(0);
        let mode = key.split(':').nth(1).unwrap_or("unknown").to_string();
        let u = size.max(0) as u64;
        *by_mode.entry(mode).or_insert(0) += u;
        total += u;
    }
    Ok(crate::swarm::manifest::RemainingQueueSummary { total, by_mode })
}
pub mod behavior_mix;
pub mod concrete;
pub mod generate;
pub mod manifest;
pub mod schedule;
pub mod seed;
pub mod swarm;
pub mod template;
