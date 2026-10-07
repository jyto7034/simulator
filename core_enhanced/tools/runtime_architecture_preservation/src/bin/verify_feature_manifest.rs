use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Manifest {
    schema_version: u32,
    scope: Scope,
    rulebook: Rulebook,
    required_validation_dimensions: Vec<String>,
    performance_observation: PerformanceObservation,
    evidence_bundles: BTreeMap<String, EvidenceBundle>,
    oracle_inputs: Vec<Locator>,
    legacy_removal_targets: Vec<LegacyTarget>,
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct PerformanceObservation {
    tool_path: String,
    receipt_path: String,
    command: String,
    workload_version: String,
    sample_count: usize,
    required_metrics: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BaselineReceipt {
    schema_version: u32,
    workload_version: String,
    tool_source_sha256: String,
    environment: BaselineEnvironment,
    event_heavy: BaselineWorkload,
    concurrent_sessions: BaselineWorkload,
}

#[derive(Debug, Deserialize)]
struct BaselineEnvironment {
    rust_profile: String,
}

#[derive(Debug, Deserialize)]
struct BaselineWorkload {
    repeatable: bool,
    raw_samples: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Scope {
    included: Vec<String>,
    excluded: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Rulebook {
    path: String,
    sha256: String,
    heading_levels: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct EvidenceBundle {
    production_routes: Vec<Locator>,
    executable_tests: Vec<TestLocator>,
}

#[derive(Debug, Deserialize)]
struct Locator {
    path: String,
    contains: String,
}

#[derive(Debug, Deserialize)]
struct TestLocator {
    path: String,
    contains: String,
    command: String,
}

#[derive(Debug, Deserialize)]
struct LegacyTarget {
    id: String,
    source: String,
    locator: Locator,
    #[serde(default)]
    removed: bool,
    replacement_locator: Option<Locator>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    id: String,
    heading: String,
    level: usize,
    classification: String,
    evidence_bundle: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn resolve(root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn check_locator(
    root: &Path,
    locator: &Locator,
    errors: &mut Vec<String>,
    content_identities: &mut BTreeMap<String, String>,
) {
    let lower_path = locator.path.to_ascii_lowercase();
    if lower_path.contains("game_server")
        || lower_path.contains("unity")
        || lower_path.contains("arkclient")
        || Path::new(&locator.path).is_absolute()
    {
        errors.push(format!("out-of-scope external locator: {}", locator.path));
        return;
    }
    let path = resolve(root, &locator.path);
    match fs::read(&path) {
        Ok(bytes) => match std::str::from_utf8(&bytes) {
            Ok(content) if content.contains(&locator.contains) => {
                content_identities.insert(locator.path.clone(), sha256(&bytes));
            }
            Ok(_) => errors.push(format!(
                "locator text missing: {} :: {:?}",
                path.display(),
                locator.contains
            )),
            Err(error) => errors.push(format!(
                "locator is not UTF-8 text: {} :: {error}",
                path.display()
            )),
        },
        Err(error) => errors.push(format!("locator unreadable: {} :: {error}", path.display())),
    }
}

fn check_removed_locator(root: &Path, target: &LegacyTarget, errors: &mut Vec<String>) {
    let lower_path = target.locator.path.to_ascii_lowercase();
    if lower_path.contains("game_server")
        || lower_path.contains("unity")
        || lower_path.contains("arkclient")
        || Path::new(&target.locator.path).is_absolute()
    {
        errors.push(format!(
            "out-of-scope external locator: {}",
            target.locator.path
        ));
        return;
    }

    let path = resolve(root, &target.locator.path);
    match fs::read_to_string(&path) {
        Ok(content) if content.contains(&target.locator.contains) => errors.push(format!(
            "removed legacy locator is still present: {} :: {:?}",
            path.display(),
            target.locator.contains
        )),
        Ok(_) => {}
        Err(error) => errors.push(format!(
            "removed legacy locator unreadable: {} :: {error}",
            path.display()
        )),
    }
}

fn required_positive_u64(
    sample: &serde_json::Value,
    path: &[&str],
    sample_label: &str,
    errors: &mut Vec<String>,
) {
    let value = path
        .iter()
        .try_fold(sample, |current, key| current.get(*key));
    if value.and_then(serde_json::Value::as_u64).unwrap_or(0) == 0 {
        errors.push(format!(
            "{sample_label} requires positive metric {}",
            path.join(".")
        ));
    }
}

fn validate_workload(
    label: &str,
    workload: &BaselineWorkload,
    expected_samples: usize,
    output_hash_field: &str,
    errors: &mut Vec<String>,
) {
    if !workload.repeatable {
        errors.push(format!("{label} receipt is not repeatable"));
    }
    if workload.raw_samples.len() != expected_samples {
        errors.push(format!(
            "{label} sample count mismatch: expected {expected_samples}, actual {}",
            workload.raw_samples.len()
        ));
    }

    let mut output_hashes = BTreeSet::new();
    let mut event_counts = BTreeSet::new();
    let mut encoded_sizes = BTreeSet::new();
    for (index, sample) in workload.raw_samples.iter().enumerate() {
        let sample_label = format!("{label} sample {}", index + 1);
        if sample.get("sample").and_then(serde_json::Value::as_u64) != Some((index + 1) as u64) {
            errors.push(format!("{sample_label} has non-sequential sample identity"));
        }
        required_positive_u64(sample, &["elapsed_ns"], &sample_label, errors);
        required_positive_u64(sample, &["event_count"], &sample_label, errors);
        required_positive_u64(sample, &["serialized_bytes"], &sample_label, errors);
        required_positive_u64(
            sample,
            &["heap_allocation", "allocation_count"],
            &sample_label,
            errors,
        );
        required_positive_u64(
            sample,
            &["heap_allocation", "peak_delta_bytes"],
            &sample_label,
            errors,
        );
        if sample
            .pointer("/heap_allocation/retained_delta_bytes")
            .and_then(serde_json::Value::as_i64)
            .is_none()
        {
            errors.push(format!(
                "{sample_label} lacks signed heap_allocation.retained_delta_bytes"
            ));
        }
        if let Some(value) = sample
            .get(output_hash_field)
            .and_then(serde_json::Value::as_str)
        {
            output_hashes.insert(value);
        }
        if let Some(value) = sample
            .get("event_count")
            .and_then(serde_json::Value::as_u64)
        {
            event_counts.insert(value);
        }
        if let Some(value) = sample
            .get("serialized_bytes")
            .and_then(serde_json::Value::as_u64)
        {
            encoded_sizes.insert(value);
        }
    }
    if output_hashes.len() != 1 || event_counts.len() != 1 || encoded_sizes.len() != 1 {
        errors.push(format!(
            "{label} deterministic output identity is not identical across samples"
        ));
    }
}

fn main() {
    let core_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tool must live under <core>/tools/<name>");
    let manifest_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| core_root.join("docs/architecture/runtime_feature_manifest.v1.json"));
    let manifest_bytes = fs::read(&manifest_path).expect("read feature manifest");
    let manifest: Manifest =
        serde_json::from_slice(&manifest_bytes).expect("parse feature manifest JSON");
    let mut errors = Vec::new();
    let mut content_identities = BTreeMap::new();

    if manifest.schema_version != 1 {
        errors.push(format!(
            "unsupported schema_version {}",
            manifest.schema_version
        ));
    }
    if manifest.scope.included != ["core_enhanced"]
        || manifest.scope.excluded != ["game_server", "Unity", "external client compatibility"]
    {
        errors.push("manifest scope is not the exact core-only scope".to_string());
    }

    let required_dimensions = BTreeSet::from([
        "success",
        "invalid_input",
        "failure_rollback",
        "retry_idempotency",
        "ordering_concurrency",
        "boundary_empty_max",
        "disconnect_restart_persistence",
        "live_data",
        "producer_consumer",
        "legacy_fallback_absence",
    ]);
    let actual_dimensions = manifest
        .required_validation_dimensions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_dimensions != required_dimensions {
        errors.push("required validation dimension set is not exact".to_string());
    }

    let expected_metrics = BTreeSet::from([
        "elapsed_ns",
        "event_count",
        "serialized_bytes",
        "heap_allocation.allocation_count",
        "heap_allocation.peak_delta_bytes",
        "heap_allocation.retained_delta_bytes",
    ]);
    let actual_metrics = manifest
        .performance_observation
        .required_metrics
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_metrics != expected_metrics {
        errors.push("performance observation metric set is not exact".to_string());
    }
    if manifest.performance_observation.command.trim().is_empty() {
        errors.push("performance observation command is empty".to_string());
    }
    let tool_path = resolve(core_root, &manifest.performance_observation.tool_path);
    let receipt_path = resolve(core_root, &manifest.performance_observation.receipt_path);
    let tool_bytes = fs::read(&tool_path).expect("read performance observation tool");
    let receipt_bytes = fs::read(&receipt_path).expect("read performance observation receipt");
    let receipt: BaselineReceipt =
        serde_json::from_slice(&receipt_bytes).expect("parse performance observation receipt");
    if receipt.schema_version != 2
        || receipt.workload_version != manifest.performance_observation.workload_version
    {
        errors.push("performance receipt schema/workload version mismatch".to_string());
    }
    if receipt.tool_source_sha256 != sha256(&tool_bytes) {
        errors.push("performance receipt tool source hash is stale".to_string());
    }
    if receipt.environment.rust_profile != "release" {
        errors.push("performance receipt was not captured in release profile".to_string());
    }
    validate_workload(
        "event-heavy",
        &receipt.event_heavy,
        manifest.performance_observation.sample_count,
        "event_log_sha256",
        &mut errors,
    );
    validate_workload(
        "64-session/8-way",
        &receipt.concurrent_sessions,
        manifest.performance_observation.sample_count,
        "combined_event_log_sha256",
        &mut errors,
    );

    let rulebook_path = resolve(core_root, &manifest.rulebook.path);
    let rulebook_bytes = fs::read(&rulebook_path).expect("read rulebook");
    let actual_rulebook_hash = sha256(&rulebook_bytes);
    if actual_rulebook_hash != manifest.rulebook.sha256 {
        errors.push(format!(
            "rulebook hash mismatch: expected {}, actual {}",
            manifest.rulebook.sha256, actual_rulebook_hash
        ));
    }
    let rulebook_text = String::from_utf8(rulebook_bytes).expect("rulebook must be UTF-8");
    let heading_levels = manifest
        .rulebook
        .heading_levels
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_headings = rulebook_text
        .lines()
        .filter_map(|line| {
            let level = line.bytes().take_while(|byte| *byte == b'#').count();
            if !heading_levels.contains(&level) || line.as_bytes().get(level) != Some(&b' ') {
                return None;
            }
            Some((level, line[level + 1..].trim().to_string()))
        })
        .collect::<Vec<_>>();
    let feature_headings = manifest
        .features
        .iter()
        .map(|feature| (feature.level, feature.heading.clone()))
        .collect::<Vec<_>>();
    if feature_headings != actual_headings {
        errors.push(format!(
            "feature heading coverage/order mismatch: manifest {}, rulebook {}",
            feature_headings.len(),
            actual_headings.len()
        ));
    }

    let mut feature_ids = BTreeSet::new();
    let mut feature_keys = BTreeSet::new();
    for feature in &manifest.features {
        if !feature_ids.insert(feature.id.as_str()) {
            errors.push(format!("duplicate feature id {}", feature.id));
        }
        if !feature_keys.insert((feature.level, feature.heading.as_str())) {
            errors.push(format!("duplicate feature heading {}", feature.heading));
        }
        if feature.classification.trim().is_empty() {
            errors.push(format!("empty classification for {}", feature.id));
        }
        if !manifest
            .evidence_bundles
            .contains_key(&feature.evidence_bundle)
        {
            errors.push(format!(
                "unknown evidence bundle {} for {}",
                feature.evidence_bundle, feature.id
            ));
        }
    }

    for (bundle_id, bundle) in &manifest.evidence_bundles {
        if bundle.production_routes.is_empty() || bundle.executable_tests.is_empty() {
            errors.push(format!("bundle {bundle_id} lacks route or executable test"));
        }
        for locator in &bundle.production_routes {
            check_locator(core_root, locator, &mut errors, &mut content_identities);
        }
        for test in &bundle.executable_tests {
            check_locator(
                core_root,
                &Locator {
                    path: test.path.clone(),
                    contains: test.contains.clone(),
                },
                &mut errors,
                &mut content_identities,
            );
            if test.command.trim().is_empty() {
                errors.push(format!("empty test command in bundle {bundle_id}"));
            }
        }
    }

    for locator in &manifest.oracle_inputs {
        check_locator(core_root, locator, &mut errors, &mut content_identities);
    }
    let mut legacy_ids = BTreeSet::new();
    for target in &manifest.legacy_removal_targets {
        if !legacy_ids.insert(target.id.as_str()) {
            errors.push(format!("duplicate legacy target id {}", target.id));
        }
        if target.source.trim().is_empty() {
            errors.push(format!("legacy target {} has no source", target.id));
        }
        if target.removed {
            check_removed_locator(core_root, target, &mut errors);
            match &target.replacement_locator {
                Some(locator) => {
                    check_locator(core_root, locator, &mut errors, &mut content_identities)
                }
                None => errors.push(format!(
                    "removed legacy target {} has no replacement locator",
                    target.id
                )),
            }
        } else {
            check_locator(
                core_root,
                &target.locator,
                &mut errors,
                &mut content_identities,
            );
        }
    }

    let mut locator_content_digest = Sha256::new();
    for (path, content_sha256) in &content_identities {
        locator_content_digest.update((path.len() as u64).to_le_bytes());
        locator_content_digest.update(path.as_bytes());
        locator_content_digest.update(content_sha256.as_bytes());
    }
    let locator_content_set_sha256 = format!("{:x}", locator_content_digest.finalize());

    let summary = serde_json::json!({
        "schema_version": manifest.schema_version,
        "manifest_sha256": sha256(&manifest_bytes),
        "rulebook_sha256": actual_rulebook_hash,
        "feature_count": manifest.features.len(),
        "rulebook_heading_count": actual_headings.len(),
        "evidence_bundle_count": manifest.evidence_bundles.len(),
        "oracle_input_count": manifest.oracle_inputs.len(),
        "legacy_removal_target_count": manifest.legacy_removal_targets.len(),
        "validation_dimension_count": actual_dimensions.len(),
        "performance_receipt_sha256": sha256(&receipt_bytes),
        "performance_workload_version": receipt.workload_version,
        "event_heavy_sample_count": receipt.event_heavy.raw_samples.len(),
        "concurrent_sample_count": receipt.concurrent_sessions.raw_samples.len(),
        "direct_heap_allocation_observation": true,
        "locator_content_count": content_identities.len(),
        "locator_content_set_sha256": locator_content_set_sha256,
        "unknown_unmapped_duplicate_count": errors.len(),
        "result": if errors.is_empty() { "passed" } else { "failed" },
        "errors": errors,
    });
    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
    if !summary["errors"].as_array().unwrap().is_empty() {
        std::process::exit(1);
    }
}
