//! Isolated publication and complete candidate costs under unchanged embedded limits.
//! Diagnostics only: every failure is retained and no acceptance threshold is selected.
use game_core::{
    authoring::{self, CancellationToken},
    game::data::content_pack::{ContentSelection, InputLimits},
    mod_validation::{Candidate, TrustedBase},
    runtime_safety::{RuntimeSafetySettings, EMBEDDED_RUNTIME_SAFETY_RON},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{error::Error, fs, path::Path};
use u11_independent_public_probe::{file_state::FileState, normal_preparation::save};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: candidate_usage FROZEN_ORIGINAL_FIXTURE_ROOT NEW_OUTPUT_DIRECTORY".into());
    }
    let root = Path::new(&args[0]);
    let out = Path::new(&args[1]);
    fs::create_dir(out)?;
    let original = FileState::capture(root, 65_536, 256 * 1024 * 1024)?;
    save(&out.join("source_before.json"), &original.report())?;
    let settings = RuntimeSafetySettings::embedded_shared();
    save(&out.join("context.json"), &json!({
        "schema": "u11.candidate-cost-diagnostic/v1", "source_root": root,
        "runtime_safety_sha256": format!("{:x}", Sha256::digest(EMBEDDED_RUNTIME_SAFETY_RON)),
        "authoring_execution_limits": settings.authoring_execution(),
        "embedded_limits_unchanged": true, "numeric_acceptance_criteria_selected": false,
        "scope": "Isolated single publications and actual full Candidate acquisition are distinct operations; their costs must not be equated."
    }))?;
    let cases = [
        ("official", vec!["official"]),
        ("a", vec!["official", "a_skills", "a_item"]),
        ("b", vec!["official", "b_skills", "b_item"]),
        ("ab", vec!["official", "a_skills", "a_item", "b_skills", "b_item"]),
    ];
    let mut publications = Vec::new();
    for (name, packs) in cases {
        let selection = ContentSelection {
            packs: packs.into_iter().map(Into::into).collect(), winners: vec![],
        };
        save(&out.join(format!("publication_{name}_input.json")), &selection)?;
        let result = authoring::validate_with_runtime_safety(
            root, &selection, &InputLimits::default(), &CancellationToken::default(), settings.clone(),
        );
        let observation = match result {
            Ok(content) => json!({"status": "completed", "usage_available": true,
                "usage": content.validation_usage(), "content_identity": content.token().identity(),
                "source_receipt": content.token().source_receipt()}),
            Err(error) => json!({"status": "failed", "usage_available": false, "usage": null,
                "usage_unavailable_reason": "The public failure result exposes a typed diagnostic, not completed usage.",
                "error": error}),
        };
        save(&out.join(format!("publication_{name}_result.json")), &observation)?;
        publications.push(json!({"name": name, "result": observation}));
    }
    let base_result = TrustedBase::acquire(root,
        ContentSelection { packs: vec!["official".into()], winners: vec![] },
        &InputLimits::default(), settings, &CancellationToken::default());
    let base_observation = match &base_result {
        Ok(base) => json!({"status": "completed", "content_identity": base.generation().identity(),
            "source_receipt": base.generation().source_receipt(), "usage_available": false,
            "usage_unavailable_reason": "TrustedBase does not publicly expose its acquisition usage."}),
        Err(error) => json!({"status": "failed", "usage_available": false, "error": error}),
    };
    save(&out.join("trusted_base_result.json"), &base_observation)?;
    let mut candidates = Vec::new();
    for name in ["a", "b", "ab"] {
        let path = root.join(format!("registries/{name}.json"));
        let bytes = fs::read(&path)?;
        save(&out.join(format!("candidate_{name}_input.json")), &json!({"registry_path": path,
            "bytes": bytes.len(), "sha256": format!("{:x}", Sha256::digest(&bytes))}))?;
        let observation = match &base_result {
            Ok(base) => match Candidate::acquire(root, &bytes, base,
                &InputLimits::default(), &CancellationToken::default()) {
                Ok(candidate) => json!({"status": "completed", "usage_available": true,
                    "usage": candidate.authoring_content().validation_usage(), "candidate": candidate.view()}),
                Err(error) => json!({"status": "failed", "usage_available": false, "usage": null,
                    "usage_unavailable_reason": "Candidate failure exposes the actual typed error, not completed usage.",
                    "error": error}),
            },
            Err(error) => json!({"status": "not_executed", "usage_available": false, "usage": null,
                "reason": "Actual TrustedBase acquisition failed.", "prerequisite_error": error}),
        };
        save(&out.join(format!("candidate_{name}_result.json")), &observation)?;
        candidates.push(json!({"name": name, "result": observation}));
    }
    let after = FileState::capture(root, 65_536, 256 * 1024 * 1024)?;
    save(&out.join("source_after.json"), &after.report())?;
    original.ensure_unchanged(&after)?;
    save(&out.join("RESULT.json"), &json!({"diagnostic_completed": true,
        "source_unchanged": true, "embedded_limits_unchanged": true,
        "publications": publications, "trusted_base": base_observation, "candidates": candidates,
        "numeric_acceptance_criteria_selected": false, "approval_claimed": false}))?;
    Ok(())
}
