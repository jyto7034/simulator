use actix::Actor;
use std::time::{Duration, Instant};
use tracing::{info, warn};

use crate::observer_actor::{message::StartObservation, ObserverActor, Phase, PhaseCondition};
use crate::player_actor::PlayerActor;
use crate::swarm::schedule::spawn_schedule_constant;
use crate::swarm::seed::uuid_for;
use crate::swarm::behavior_mix::{behavior_for_index, BehaviorMixConfig};
use crate::swarm::template::SwarmTemplate;
use crate::swarm::generate::generate;
use crate::swarm::manifest::{SwarmRunSummary, save_swarm_summary};
use crate::scenario_actor::ScenarioRunnerActor;
use uuid::Uuid;

pub mod config;
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
    let seed = cfg
        .seed
        .or_else(|| std::env::var("SWARM_SEED").ok().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(42);
    let mix: BehaviorMixConfig = if let Some(m) = cfg.behavior_mix.clone() {
        m
    } else if let Some(path) = cfg.template_path.clone() {
        match std::fs::read_to_string(&path) {
            Ok(toml) => match toml::from_str::<SwarmTemplate>(&toml) {
                Ok(tpl) => generate(seed, &tpl).behavior_mix,
                Err(e) => {
                    warn!("Failed to parse template at {}: {}. Using default mix.", path, e);
                    BehaviorMixConfig {
                        slow_ratio: 0.1,
                        slow_delay_seconds: 5,
                        spiky_ratio: 0.05,
                        spiky_delay_ms: 150,
                        timeout_ratio: 0.02,
                        quit_before_ratio: 0.03,
                        quit_during_loading_ratio: 0.03,
                        invalid_mode_unknown_weight: 1.0,
                        invalid_mode_missing_weight: 0.0,
                        invalid_mode_early_loading_complete_weight: 0.0,
                        invalid_mode_duplicate_enqueue_weight: 0.0,
                        invalid_mode_wrong_session_id_weight: 0.0,
                        invalid_ratio: 0.0,
                    }
                }
            },
            Err(e) => {
                warn!("Failed to read template at {}: {}. Using default mix.", path, e);
                BehaviorMixConfig {
                    slow_ratio: 0.1,
                    slow_delay_seconds: 5,
                    spiky_ratio: 0.05,
                    spiky_delay_ms: 150,
                    timeout_ratio: 0.02,
                    quit_before_ratio: 0.03,
                    quit_during_loading_ratio: 0.03,
                    invalid_mode_unknown_weight: 1.0,
                    invalid_mode_missing_weight: 0.0,
                    invalid_mode_early_loading_complete_weight: 0.0,
                    invalid_mode_duplicate_enqueue_weight: 0.0,
                    invalid_mode_wrong_session_id_weight: 0.0,
                    invalid_ratio: 0.0,
                }
            }
        }
    } else {
        BehaviorMixConfig {
            slow_ratio: 0.1,
            slow_delay_seconds: 5,
            spiky_ratio: 0.05,
            spiky_delay_ms: 150,
            timeout_ratio: 0.02,
            quit_before_ratio: 0.03,
            quit_during_loading_ratio: 0.03,
            invalid_mode_unknown_weight: 1.0,
            invalid_mode_missing_weight: 0.0,
            invalid_mode_early_loading_complete_weight: 0.0,
            invalid_mode_duplicate_enqueue_weight: 0.0,
            invalid_mode_wrong_session_id_weight: 0.0,
            invalid_ratio: 0.0,
        }
    };

    // Generate run_id and export to Observer
    let run_id = uuid::Uuid::new_v4().to_string();
    std::env::set_var("OBSERVER_RUN_ID", &run_id);

    // Reset match server test run_id for test-scoped metrics
    if let Some(base) = cfg.match_server_base.clone() {
        let http = if base.starts_with("ws://") { base.replacen("ws://", "http://", 1) } else if base.starts_with("wss://") { base.replacen("wss://", "https://", 1) } else { base.clone() };
        let url = format!("{}/admin/test/reset?run_id={}", http.trim_end_matches('/'), run_id);
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => info!("[run_id] reset sent: {}", run_id),
            Ok(resp) => warn!("[run_id] reset failed: status={} url={}", resp.status(), url),
            Err(e) => warn!("[run_id] reset error: {} url={}", e, url),
        }
    } else {
        warn!("match_server_base not set; skipping /admin/test/reset call");
    }

    // Log Grafana dashboard URL for this run (from now to now+duration)
    let start_ms = chrono::Utc::now().timestamp_millis();
    let end_ms = start_ms + (cfg.duration_secs as i64 * 1000);
    let grafana_url = format!(
        "http://localhost:3000/d/sim-swarm/simulator-matchmaking?from={}&to={}&var-run_id={}&refresh=5s",
        start_ms, end_ms, run_id
    );
    info!("Open Grafana for this run: {}", grafana_url);

    // Spawn observers per shard and orchestrate players
    let mut observer_addrs: Vec<actix::Addr<ObserverActor>> = Vec::with_capacity(cfg.shards as usize);
    for shard in 0..cfg.shards {
        let test_name = format!("SwarmShard-{}", shard);
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
        let schedule_ms = spawn_schedule_constant(shard_seed, "swarm", player_count, cps.max(0.1), 200);

        // Prepare player id list and kick off observation (ids pre-registered for shard filtering)
        let mut ids: Vec<Uuid> = Vec::with_capacity(player_count as usize);
        for i in 0..player_count {
            ids.push(uuid_for(shard_seed, "player", i));
        }
        observer_addr.do_send(StartObservation { player_ids: ids.clone() });

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
            std::env::set_var("TEST_CLIENT_BEHAVIOR_MIX", serde_json::to_string(&mix).unwrap_or("{}".into()));

        for (i, ms) in schedule_ms.iter().enumerate() {
            let when_ms = *ms;
            let obs = observer_addr.clone();
            let pid = ids[i];
            let mix_clone = mix.clone();
            actix::spawn(async move {
                let target = t0 + Duration::from_millis(when_ms);
                let now = Instant::now();
                if target > now {
                    tokio::time::sleep(target - now).await;
                }
                let behavior = behavior_for_index(shard_seed, i as u64, &mix_clone);
                let actor = PlayerActor::new(obs, Box::new(behavior), pid, true);
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
            let summary = SwarmRunSummary {
                timestamp: ts,
                seed,
                run_id: run_id.clone(),
                config: cfg.clone(),
                metrics_url: metrics_url.clone(),
                slo: report,
            };
            let out_path = cfg
                .result_path
                .clone()
                .unwrap_or_else(|| format!(
                    "logs/swarm_summary_{}_{}.json",
                    ts.format("%Y%m%d_%H%M%S"),
                    &run_id
                ));
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
    if overall_ok { Ok(()) } else { Err(anyhow::anyhow!("SLO failed")) }
}
pub mod concrete;
pub mod generate;
pub mod manifest;
pub mod seed;
pub mod template;
pub mod schedule;
pub mod swarm;
pub mod behavior_mix;
