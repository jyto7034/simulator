use crate::swarm::behavior_mix::BehaviorMixConfig;
use crate::swarm::manifest::BehaviorOutcomeCounts;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SloThresholds {
    pub p95_match_time_secs: f64,
    pub p95_loading_secs: f64,
    pub max_violations: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SloReport {
    pub p95_match_time_secs: Option<f64>,
    pub p95_loading_secs: Option<f64>,
    pub violations_total: u64,
    // Additional counters for dashboard/summary
    pub enqueued_total: u64,
    pub matched_players_total: u64,
    pub loading_completed_total: u64,
    pub dedicated_alloc_success_total: u64,
    // Behavior ratios summary
    pub behavior_summary: Option<BehaviorSummary>,
    // Outcome counts calculated from metrics
    pub outcome_counts: BehaviorOutcomeCounts,

    pub passed: bool,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BehaviorSummary {
    pub expected_normal_ratio: f64,
    pub expected_abnormal_ratio: f64,
    pub observed_normal_ratio: Option<f64>,
    pub observed_abnormal_ratio: Option<f64>,
}

/// Build metrics URL from base like ws://host:port → http://host:port/metrics unless overridden.
pub fn metrics_url_from_base(base: &str) -> String {
    let http = if base.starts_with("ws://") {
        base.replacen("ws://", "http://", 1)
    } else if base.starts_with("wss://") {
        base.replacen("wss://", "https://", 1)
    } else {
        base.to_string()
    };
    format!("{}/metrics", http.trim_end_matches('/'))
}

pub async fn fetch_metrics(url: &str) -> anyhow::Result<String> {
    let resp = reqwest::get(url).await?;
    let text = resp.text().await?;
    Ok(text)
}

#[derive(Debug, Clone)]
struct Histogram {
    buckets: Vec<(f64, u64)>, // (le, count)
    count: u64,
}

fn parse_histograms(
    scrape: &str,
    metric_name: &str,
    label_filter: Option<(&str, &str)>,
) -> Histogram {
    let bucket_re = Regex::new(&format!(
        r"^{}\_bucket\{{(?P<labels>[^}}]*)\}}\s+(?P<value>[-0-9\.eE]+)$",
        regex::escape(metric_name)
    ))
    .unwrap();
    let count_re = Regex::new(&format!(
        r"^{}\_count\{{(?P<labels>[^}}]*)\}}\s+(?P<value>[-0-9\.eE]+)$",
        regex::escape(metric_name)
    ))
    .unwrap();

    let mut agg: HashMap<String, u64> = HashMap::new();
    let mut total_count: u64 = 0;

    for line in scrape.lines() {
        if let Some(caps) = bucket_re.captures(line) {
            let labels = caps.name("labels").map(|m| m.as_str()).unwrap_or("");
            if !label_match(labels, label_filter) {
                continue;
            }
            if let Some(le_str) = extract_label(labels, "le") {
                let val: u64 = caps
                    .name("value")
                    .and_then(|m| m.as_str().parse::<f64>().ok())
                    .unwrap_or(0.0) as u64;
                *agg.entry(le_str).or_insert(0) += val;
            }
        } else if let Some(caps) = count_re.captures(line) {
            let labels = caps.name("labels").map(|m| m.as_str()).unwrap_or("");
            if !label_match(labels, label_filter) {
                continue;
            }
            let val: u64 = caps
                .name("value")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0) as u64;
            total_count += val;
        }
    }

    let mut buckets: Vec<(f64, u64)> = agg
        .into_iter()
        .map(|(k, v)| {
            let le = if k == "+Inf" || k == "Inf" {
                f64::INFINITY
            } else {
                k.parse::<f64>().unwrap_or(f64::INFINITY)
            };
            (le, v)
        })
        .collect();
    buckets.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    Histogram {
        buckets,
        count: total_count,
    }
}

fn label_match(labels: &str, filter: Option<(&str, &str)>) -> bool {
    if let Some((k, v)) = filter {
        if let Some(val) = extract_label(labels, k) {
            return val == v;
        }
        return false;
    }
    true
}

fn extract_label(labels: &str, key: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"{}\s*=\s*\"([^\"]*)\""#, regex::escape(key))).ok()?;
    re.captures(labels)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn pxx_from_hist(hist: &Histogram, quantile: f64) -> Option<f64> {
    if hist.count == 0 || hist.buckets.is_empty() {
        return None;
    }
    let target = (hist.count as f64 * quantile).ceil() as u64;
    for (le, cnt) in hist.buckets.iter() {
        if *cnt >= target {
            return Some(*le);
        }
    }
    hist.buckets.last().map(|(le, _)| *le)
}

fn sum_counters(scrape: &str, metric_name: &str, label_filter: Option<(&str, &str)>) -> u64 {
    // Support both labeled and unlabeled metrics:
    // metric_name{labels} value  (labeled)
    // metric_name value          (unlabeled)
    let labeled_re = Regex::new(&format!(
        r"^{}\{{(?P<labels>[^}}]*)\}}\s+(?P<value>[-0-9\.eE]+)$",
        regex::escape(metric_name)
    ))
    .unwrap();
    let unlabeled_re = Regex::new(&format!(
        r"^{}\s+(?P<value>[-0-9\.eE]+)$",
        regex::escape(metric_name)
    ))
    .unwrap();

    let mut sum = 0u64;
    for line in scrape.lines() {
        if let Some(caps) = labeled_re.captures(line) {
            let labels = caps.name("labels").map(|m| m.as_str()).unwrap_or("");
            if !label_match(labels, label_filter) {
                continue;
            }
            let val: u64 = caps
                .name("value")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0) as u64;
            sum += val;
        } else if let Some(caps) = unlabeled_re.captures(line) {
            // For unlabeled metrics, only match if no filter is specified
            if label_filter.is_some() {
                continue;
            }
            let val: u64 = caps
                .name("value")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0) as u64;
            sum += val;
        }
    }
    sum
}

pub async fn evaluate_slo(
    metrics_url: &str,
    game_mode: Option<&str>,
    th: &SloThresholds,
) -> anyhow::Result<SloReport> {
    let text = fetch_metrics(metrics_url).await?;
    let filter = game_mode.map(|m| ("game_mode", m));
    let match_hist = parse_histograms(&text, "match_time_seconds", filter);
    let load_hist = parse_histograms(&text, "loading_duration_seconds", filter);
    let p95_match = pxx_from_hist(&match_hist, 0.95);
    let p95_load = pxx_from_hist(&load_hist, 0.95);

    let violations = sum_counters(&text, "state_violations_total", None);
    // Additional totals for summary/panels
    let enqueued_total = sum_counters(&text, "enqueued_total_by_mode", filter);
    let matched_players_total = sum_counters(&text, "matched_players_total_by_mode", filter);
    let loading_completed_total = sum_counters(&text, "loading_completed_total_by_mode", filter);
    let dedicated_alloc_success_total =
        sum_counters(&text, "dedicated_allocation_success_total_by_mode", filter);

    let mut passed = true;
    let mut details = Vec::new();
    if let Some(v) = p95_match {
        if v > th.p95_match_time_secs {
            passed = false;
            details.push(format!(
                "p95 match_time {:.2}s > {:.2}s",
                v, th.p95_match_time_secs
            ));
        }
    } else {
        details.push("p95 match_time unavailable".into());
    }
    if let Some(v) = p95_load {
        if v > th.p95_loading_secs {
            passed = false;
            details.push(format!(
                "p95 loading {:.2}s > {:.2}s",
                v, th.p95_loading_secs
            ));
        }
    } else {
        details.push("p95 loading unavailable".into());
    }

    // Derive behavior expected/observed ratios if we can infer BehaviorMixConfig
    let behavior_summary = {
        let mix_env = std::env::var("TEST_CLIENT_BEHAVIOR_MIX").ok();
        if let Some(mix_json) = mix_env {
            if let Ok(mix) = serde_json::from_str::<BehaviorMixConfig>(&mix_json) {
                let expected_abnormal = (mix.timeout_ratio
                    + mix.quit_before_ratio
                    + mix.quit_during_loading_ratio
                    + mix.invalid_ratio)
                    .clamp(0.0, 1.0);
                let expected_normal = (1.0 - expected_abnormal).max(0.0);
                // Observed abnormal ratio inferred from (enqueued - matched)
                let observed_abnormal = if enqueued_total > 0
                    && matched_players_total <= enqueued_total
                {
                    Some(
                        ((enqueued_total - matched_players_total) as f64 / enqueued_total as f64)
                            .clamp(0.0, 1.0),
                    )
                } else {
                    None
                };
                let observed_normal = observed_abnormal.map(|a| (1.0 - a).max(0.0));

                // Sanity rules
                if loading_completed_total > 0 && dedicated_alloc_success_total == 0 {
                    details.push(
                        "Sanity: loading_completed_total > 0 but dedicated_alloc_success_total = 0"
                            .into(),
                    );
                    passed = false;
                }
                if let Some(a) = observed_abnormal {
                    let diff = (a - expected_abnormal).abs();
                    if diff > 0.2 {
                        // deviation threshold
                        details.push(format!("Observed abnormal ratio deviates from expected by >0.2: obs={:.2}, exp={:.2}", a, expected_abnormal));
                        passed = false;
                    }
                    if a > 0.6 && (dedicated_alloc_success_total as f64) > a * enqueued_total as f64
                    {
                        details.push(format!("Suspicious: observed_abnormal_ratio={:.2} but dedicated_success_total={} > abnormal_count~{:.0}", a, dedicated_alloc_success_total, a * enqueued_total as f64));
                        passed = false;
                    }
                }

                Some(BehaviorSummary {
                    expected_normal_ratio: expected_normal,
                    expected_abnormal_ratio: expected_abnormal,
                    observed_normal_ratio: observed_normal,
                    observed_abnormal_ratio: observed_abnormal,
                })
            } else {
                None
            }
        } else {
            None
        }
    };
    if violations > th.max_violations {
        passed = false;
        details.push(format!("violations {} > {}", violations, th.max_violations));
    }

    // Calculate outcome counts here with game mode filter awareness
    let outcome_counts = calculate_outcome_counts_filtered(
        enqueued_total,
        matched_players_total,
        loading_completed_total,
        dedicated_alloc_success_total,
        violations,
        &text,
        filter,
    );

    Ok(SloReport {
        p95_match_time_secs: p95_match,
        p95_loading_secs: p95_load,
        violations_total: violations,
        enqueued_total,
        matched_players_total,
        loading_completed_total,
        dedicated_alloc_success_total,
        behavior_summary,
        outcome_counts,
        passed,
        details,
    })
}

/// Calculate outcome counts with game mode filter awareness
fn calculate_outcome_counts_filtered(
    enqueued_total: u64,
    matched_players_total: u64,
    loading_completed_total: u64,
    dedicated_alloc_success_total: u64,
    violations_total: u64,
    metrics_text: &str,
    _filter: Option<(&str, &str)>,
) -> BehaviorOutcomeCounts {
    // Parse specific error types from metrics
    // Note: loading_session_timeout_players_total has no game_mode label, so use None filter
    let timeout_players = sum_counters(metrics_text, "loading_session_timeout_players_total", None);
    let matchmaking_errors = sum_counters(metrics_text, "matchmaking_errors_total", None);

    // Calculate outcome counts based on the flow
    let successful_matches = dedicated_alloc_success_total;
    let loading_timeouts = timeout_players;
    let connection_failures = if enqueued_total > matched_players_total {
        // Some players didn't even get matched - likely connection issues
        let failed_to_match = enqueued_total - matched_players_total;
        // Subtract known errors to avoid double counting
        failed_to_match.saturating_sub(matchmaking_errors + violations_total)
    } else {
        0
    };
    let invalid_requests = violations_total;
    let other_failures = if matched_players_total > loading_completed_total {
        // Players matched but didn't complete loading (excluding known timeouts)
        (matched_players_total - loading_completed_total).saturating_sub(loading_timeouts)
    } else {
        0
    };

    BehaviorOutcomeCounts {
        successful_matches,
        loading_timeouts,
        quit_before_match: 0,   // This would need specific tracking
        quit_during_loading: 0, // This would need specific tracking
        connection_failures,
        invalid_requests,
        other_failures,
    }
}

/// Calculate outcome counts based on metrics from the match server (legacy function)
pub fn calculate_outcome_counts(
    enqueued_total: u64,
    matched_players_total: u64,
    loading_completed_total: u64,
    dedicated_alloc_success_total: u64,
    violations_total: u64,
    metrics_text: &str,
) -> BehaviorOutcomeCounts {
    // Parse specific error types from metrics
    let timeout_players = sum_counters(metrics_text, "loading_session_timeout_players_total", None);
    let matchmaking_errors = sum_counters(metrics_text, "matchmaking_errors_total", None);

    // Calculate outcome counts based on the flow
    let successful_matches = dedicated_alloc_success_total;
    let loading_timeouts = timeout_players;
    let connection_failures = if enqueued_total > matched_players_total {
        // Some players didn't even get matched - likely connection issues
        let failed_to_match = enqueued_total - matched_players_total;
        // Subtract known errors to avoid double counting
        failed_to_match.saturating_sub(matchmaking_errors + violations_total)
    } else {
        0
    };
    let invalid_requests = violations_total;
    let other_failures = if matched_players_total > loading_completed_total {
        // Players matched but didn't complete loading (excluding known timeouts)
        (matched_players_total - loading_completed_total).saturating_sub(loading_timeouts)
    } else {
        0
    };

    BehaviorOutcomeCounts {
        successful_matches,
        loading_timeouts,
        quit_before_match: 0,   // This would need specific tracking
        quit_during_loading: 0, // This would need specific tracking
        connection_failures,
        invalid_requests,
        other_failures,
    }
}
