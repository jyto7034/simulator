//! One content family per process: observed publication/acquisition cost only.
//! Run under an external finite timeout; this binary cannot select an approval.
use game_core::{authoring::{self, CancellationToken},
    game::data::content_pack::{ContentSelection, InputLimits},
    mod_validation::{Candidate, TrustedBase},
    runtime_safety::{RuntimeSafetySettings, EMBEDDED_RUNTIME_SAFETY_RON}};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{error::Error, fs, io::Read, path::Path, sync::Arc, time::Instant};
use u11_independent_public_probe::{file_state::FileState, normal_preparation::save};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn bounded(path: &Path, cap: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(cap as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > cap { return Err("diagnostic input exceeds finite read bound".into()); }
    Ok(bytes)
}
fn memory() -> Result<Value> {
    let raw = String::from_utf8(bounded(Path::new("/proc/self/status"), 65_536)?)?;
    let field = |name: &str| -> Result<u64> {
        let line = raw.lines().find(|line| line.starts_with(name)).ok_or("Linux RSS field absent")?;
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() != 3 || fields[2] != "kB" { return Err("unexpected Linux RSS unit".into()); }
        Ok(fields[1].parse::<u64>()?.checked_mul(1024).ok_or("RSS byte overflow")?)
    };
    Ok(json!({"current_rss_bytes": field("VmRSS:")?,
        "process_lifetime_peak_rss_bytes": field("VmHWM:")?,
        "peak_scope": "Cumulative process high-water mark, including earlier operations and diagnostic bookkeeping; not per-operation allocations."}))
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 6 {
        return Err("usage: family_usage ROOT REGISTRY_JSON SELECTION_JSON BASE_SELECTION_JSON PROFILE_RON NEW_OUTPUT".into());
    }
    let root = Path::new(&args[0]); let out = Path::new(&args[5]); fs::create_dir(out)?;
    let original = FileState::capture(root, 65_536, 256 * 1024 * 1024)?;
    save(&out.join("source_before.json"), &original.report())?;
    let mut inputs = Vec::new();
    for (index, label) in [(1,"registry"),(2,"selection"),(3,"base_selection"),(4,"profile")] {
        let path = Path::new(&args[index]); let bytes = bounded(path, 1_048_576)?;
        inputs.push((label, bytes.clone()));
        save(&out.join(format!("{label}_input.json")), &json!({"path": path,"bytes": bytes.len(),
            "sha256": format!("{:x}",Sha256::digest(&bytes)),"raw_bytes":bytes}))?;
    }
    let registry = &inputs[0].1;
    let selection: ContentSelection = serde_json::from_slice(&inputs[1].1)?;
    let base_selection: ContentSelection = serde_json::from_slice(&inputs[2].1)?;
    let profile = &inputs[3].1;
    let settings = Arc::new(RuntimeSafetySettings::from_ron("actual family diagnostic profile",profile)?);
    let embedded = std::str::from_utf8(EMBEDDED_RUNTIME_SAFETY_RON)?;
    let observation = embedded.replacen("authoring_execution: (\n        max_work: 10000000,",
        "authoring_execution: (\n        max_work: 100000000,",1);
    let profile_kind = if profile.as_slice() == EMBEDDED_RUNTIME_SAFETY_RON {
        "unchanged_embedded_baseline"
    } else if observation != embedded && profile.as_slice() == observation.as_bytes() {
        "observation_only_authoring_100m"
    } else { return Err("profile is neither exact embedded baseline nor approved single-field observation profile".into()); };
    let baseline = RuntimeSafetySettings::embedded();
    let mut expected_execution = serde_json::to_value(baseline.authoring_execution())?;
    let differences = if profile_kind == "observation_only_authoring_100m" {
        expected_execution["max_work"] = json!(100_000_000u64);
        json!([{"field":"authoring_execution.max_work","before":10_000_000u64,"after":100_000_000u64}])
    } else { json!([]) };
    assert_eq!(serde_json::to_value(settings.authoring_execution())?, expected_execution);
    assert_eq!(settings.nominal_native_runtime().max_work, baseline.nominal_native_runtime().max_work);
    save(&out.join("context.json"), &json!({"schema":"u11.family-cost-diagnostic/v1",
        "profile_kind":profile_kind,"exact_profile_semantic_diff":differences,
        "all_other_profile_bytes_unchanged":true,"runtime_safety_raw_sha256":settings.raw_sha256(),
        "runtime_safety_effective_sha256":settings.effective_sha256(),
        "authoring_execution_limits":settings.authoring_execution(),
        "build_debug_assertions":cfg!(debug_assertions),
        "timing_scope":"Actual build only; debug timings are not optimized release calibration.",
        "numeric_acceptance_criteria_selected":false,"default_limit_change_claimed":false,
        "external_timeout_required_seconds":300}))?;
    let before = memory()?; let start = Instant::now();
    let publication = authoring::validate_with_runtime_safety(root,&selection,&InputLimits::default(),
        &CancellationToken::default(),settings.clone());
    let elapsed = start.elapsed().as_nanos(); let after = memory()?;
    let publication = match publication {
        Ok(content) => json!({"status":"completed","usage_available":true,
            "usage":content.validation_usage(),"content_identity":content.token().identity(),
            "source_receipt":content.token().source_receipt()}),
        Err(error) => json!({"status":"failed","usage_available":false,"usage":null,"error":error}),
    };
    save(&out.join("publication_result.json"),&json!({"result":publication,"elapsed_ns":elapsed,
        "memory_before":before,"memory_after":after}))?;
    let before = memory()?; let start = Instant::now();
    let base = TrustedBase::acquire(root,base_selection,&InputLimits::default(),settings,&CancellationToken::default());
    let elapsed = start.elapsed().as_nanos(); let after = memory()?;
    let base_observation = match &base {
        Ok(base) => json!({"status":"completed","usage_available":false,
            "usage_unavailable_reason":"TrustedBase does not expose completed acquisition usage.",
            "content_identity":base.generation().identity(),"source_receipt":base.generation().source_receipt()}),
        Err(error) => json!({"status":"failed","usage_available":false,"usage":null,"error":error}),
    };
    save(&out.join("trusted_base_result.json"),&json!({"result":base_observation,"elapsed_ns":elapsed,
        "memory_before":before,"memory_after":after}))?;
    let before = memory()?; let start = Instant::now();
    let candidate = match &base {
        Ok(base) => Some(Candidate::acquire(root,registry,base,&InputLimits::default(),&CancellationToken::default())),
        Err(_) => None,
    };
    let elapsed = start.elapsed().as_nanos(); let after = memory()?;
    let candidate_observation = match &candidate {
        Some(Ok(candidate)) => json!({"status":"completed","usage_available":true,
            "usage":candidate.authoring_content().validation_usage(),"candidate":candidate.view()}),
        Some(Err(error)) => json!({"status":"failed","usage_available":false,"usage":null,"error":error}),
        None => json!({"status":"not_executed","usage_available":false,"usage":null,
            "reason":"Actual TrustedBase prerequisite failed."}),
    };
    save(&out.join("candidate_result.json"),&json!({"result":candidate_observation,"elapsed_ns":elapsed,
        "memory_before":before,"memory_after":after}))?;
    if let Some(Ok(candidate)) = candidate {
        let mut expected = serde_json::to_value(&selection)?;
        let mut actual = serde_json::to_value(&candidate.required_configuration().selection)?;
        for document in [&mut expected,&mut actual] {
            document["packs"].as_array_mut().ok_or("selection packs absent")?.sort_by_key(Value::to_string);
        }
        assert_eq!(actual,expected,"explicit isolated selection must equal actual registry closure");
    }
    let after = FileState::capture(root,65_536,256*1024*1024)?;
    save(&out.join("source_after.json"),&after.report())?; original.ensure_unchanged(&after)?;
    save(&out.join("RESULT.json"),&json!({"diagnostic_completed":true,"source_unchanged":true,
        "publication":publication,"trusted_base":base_observation,"candidate":candidate_observation,
        "numeric_acceptance_criteria_selected":false,"approval_claimed":false}))?;
    Ok(())
}
