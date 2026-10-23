//! G11 current-input vault cost and atomicity oracles.
//!
//! Fixtures are runtime inputs, not a compile-time dependency on documentation.
//! The normal profile is an actual public Return. Empty/expanded are explicit I
//! fixtures. These tests do not grant normal-host approval or a performance cap.
use super::*;
use crate::authoring::control::{
    self, CancellationToken, ExecutionLimits, ExecutionPhase, ExecutionUsage,
    NativeStorageProvenance,
};
use crate::game::data::{content_pack, ValidatedGameData};
use crate::game::encounter_compendium::PlayerProfileState;
use crate::game::world::state::{BattleEntrySnapshot, RunCheckpointPayload};
use crate::game::world::PreparedApplicationCommit;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DEFAULT_INPUTS: &str = "docs/goals/core_expedition_lifecycle/execution/single_goal/test_parts/vault_cost001/inputs/manifest.json";
const RESULT_SHA: &str = "fea92c4bc849ef19f534a593cf755faa553393deb9f8e579ff202d54b0ac47a9";

struct Case {
    name: String,
    profile: PlayerProfileState,
}

struct Inputs {
    data: ValidatedGameData,
    cases: Vec<Case>,
    seed: u64,
    player: Uuid,
    manifest_sha256: String,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn file_sha256(path: &Path) -> String {
    let mut reader = std::fs::File::open(path).expect("G11 receipt file exists");
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 16384];
    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    format!("{:x}", hash.finalize())
}

fn verify_receipt(repo: &Path, receipt: &Value) -> PathBuf {
    let path = repo.join(receipt["repo_relative_path"].as_str().unwrap());
    assert_eq!(
        std::fs::metadata(&path).unwrap().len(),
        receipt["byte_count"].as_u64().unwrap()
    );
    assert_eq!(file_sha256(&path), receipt["sha256"].as_str().unwrap());
    path
}

fn inputs() -> &'static Inputs {
    static INPUTS: OnceLock<Inputs> = OnceLock::new();
    INPUTS.get_or_init(|| {
        let core_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo = core_root.parent().unwrap();
        let path = match std::env::var_os("U12_G11_INPUTS") {
            Some(path) => {
                let path = PathBuf::from(path);
                assert!(
                    path.is_absolute(),
                    "explicit G11 manifest path must be absolute"
                );
                path
            }
            None => core_root.join(DEFAULT_INPUTS),
        };
        let bytes = std::fs::read(&path).expect("prepared G11 repository fixtures must exist");
        let digest = sha256(&bytes);
        if let Ok(expected) = std::env::var("U12_G11_INPUTS_SHA256") {
            assert_eq!(digest, expected, "runner binds exact fixture bytes");
        }
        let manifest: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(manifest["schema_version"], 1);
        assert_eq!(manifest["approval_claimed"], false);
        assert_eq!(manifest["source_result"]["sha256"], RESULT_SHA);
        verify_receipt(repo, &manifest["source_result"]);
        verify_receipt(repo, &manifest["source_trace"]);
        let selection_path = verify_receipt(repo, &manifest["selection"]);
        let safety_path = verify_receipt(repo, &manifest["runtime_safety"]);
        let settings = Arc::new(
            crate::runtime_safety::RuntimeSafetySettings::from_ron(
                "G11 current external official default safety input",
                &std::fs::read(safety_path).unwrap(),
            )
            .unwrap(),
        );
        assert!(settings.development_measurement().is_none());
        assert_eq!(
            settings.effective_sha256(),
            manifest["runtime_safety_effective_sha256"]
                .as_str()
                .unwrap()
        );
        let selection = serde_json::from_slice(&std::fs::read(selection_path).unwrap()).unwrap();
        let data = content_pack::load_with_runtime_safety(
            &repo.join(manifest["content_root_repo_relative"].as_str().unwrap()),
            &selection,
            &content_pack::InputLimits::default(),
            settings,
        )
        .expect("current external official Validated inputs");
        assert_eq!(
            serde_json::to_value(data.identity()).unwrap(),
            manifest["content_identity"]
        );
        assert_eq!(
            serde_json::to_value(data.source_receipt()).unwrap(),
            manifest["source_receipt"]
        );
        let mut cases = Vec::new();
        for receipt in manifest["cases"].as_array().unwrap() {
            let raw = std::fs::read(
                path.parent()
                    .unwrap()
                    .join(receipt["path"].as_str().unwrap()),
            )
            .unwrap();
            assert_eq!(raw.len() as u64, receipt["byte_count"].as_u64().unwrap());
            assert_eq!(sha256(&raw), receipt["sha256"].as_str().unwrap());
            let profile: PlayerProfileState = serde_json::from_slice(&raw).unwrap();
            profile.validate_persisted().unwrap();
            assert_eq!(
                profile.lifecycle.vault.equipment.len() as u64,
                receipt["equipment_copies"].as_u64().unwrap()
            );
            assert_eq!(
                profile.lifecycle.vault.fragments.len() as u64,
                receipt["fragment_copies"].as_u64().unwrap()
            );
            assert_eq!(profile.lifecycle.expedition_sequence, 1);
            assert_eq!(
                profile.lifecycle.first_return_story,
                FirstReturnStory::Pending
            );
            assert!(profile.lifecycle.active_expedition.is_none());
            cases.push(Case {
                name: receipt["name"].as_str().unwrap().into(),
                profile,
            });
        }
        assert_eq!(
            cases
                .iter()
                .map(|case| case.name.as_str())
                .collect::<Vec<_>>(),
            ["empty", "normal", "expanded"]
        );
        println!(
            "U12_G11_INPUT {}",
            json!({"path":path,"sha256":digest,
            "source_result_sha256":RESULT_SHA,"approval_claimed":false,
            "native_feature":cfg!(feature="runtime-observation")})
        );
        Inputs {
            data,
            cases,
            seed: manifest["seed"].as_u64().unwrap(),
            player: Uuid::parse_str(manifest["player_uuid"].as_str().unwrap()).unwrap(),
            manifest_sha256: digest,
        }
    })
}

fn idle(case: &Case) -> GameCore {
    let input = inputs();
    let mut core = GameCore::new(input.data.clone(), input.seed);
    core.install_authoritative_player_profile(case.profile.clone())
        .unwrap();
    assert_eq!(core.state.player_profile, case.profile);
    assert!(core.export_active_run_checkpoint().unwrap().is_none());
    core
}

fn running(case: &Case) -> GameCore {
    let mut core = idle(case);
    // Actual public Tutorial start preserves the full permanent profile. No
    // combat, timing, resource funding, or content is altered by this fixture.
    start_new_game_with_mode_and_default_starters(&mut core, inputs().player, GameMode::Tutorial);
    assert_eq!(core.state.player_profile, case.profile);
    assert!(core.export_active_run_checkpoint().unwrap().is_some());
    core
}

fn fresh_fixture(case: &Case, start_run: bool) -> (GameCore, NativeStorageProvenance) {
    let limits = inputs().data.runtime_safety().authoring_execution();
    // The core is constructed here, inside the fresh scope. Its returned ledger
    // belongs to this construction family, including its clones and captures.
    let result = control::run_with_provenance(
        &limits,
        &CancellationToken::default(),
        ExecutionPhase::TrialPreparation,
        NativeStorageProvenance::fresh(),
        || if start_run { running(case) } else { idle(case) },
    );
    println!(
        "U12_G11_FAMILY {}",
        json!({"case":case.name,"start_run":start_run,
            "result":result.as_ref().map(|(_, usage, _)| usage)
                .map_err(|failure| json!(failure))}),
    );
    let (core, _, provenance) = result.expect("preserve default fixture construction failure");
    (core, provenance)
}

fn unknown_provenance_rejects_entry(case: &str, source: &GameCore) {
    let before = roots(source);
    let result = control::run_with_provenance(
        &source.runtime_safety().authoring_execution(),
        &CancellationToken::default(),
        ExecutionPhase::TrialPreparation,
        NativeStorageProvenance::default(),
        || BattleEntrySnapshot::capture(&source.state),
    );
    let failure = result
        .err()
        .expect("unknown Monolith history must remain rejected");
    println!(
        "U12_G11_UNKNOWN_PROVENANCE {}",
        json!({"case":case,"operation":"BattleEntrySnapshot::capture",
            "failure":failure,"roots_unchanged":roots(source)==before}),
    );
    assert_eq!(failure.code, "authoring_clone_limit");
    assert_eq!(failure.site, "owned_clone_storage");
    assert_eq!(failure.requested, Some(usize::MAX as u64));
    assert_eq!(roots(source), before);
}

fn roots(core: &GameCore) -> Value {
    let checkpoint = core.export_active_run_checkpoint().unwrap();
    json!({"roots":core_roots(core), "uuid_manager":core.state.uuid_manager,
        "checkpoint":checkpoint.map(|c|json!({"bytes":c.core_payload_json,
            "loads_used":c.loads_used,"run_peak_certified_power":c.run_peak_certified_power}))})
}

enum Operation {
    CaptureProfile,
    CaptureEntry,
    CaptureCheckpoint,
    RestoreSharedEntry(Arc<BattleEntrySnapshot>),
    Continue {
        bytes: Arc<[u8]>,
        loads: u8,
        peak: u64,
    },
    PrepareImport(PlayerProfileState),
    PrepareAbandon,
}

enum Output {
    Profile(PlayerProfileState),
    Entry(BattleEntrySnapshot),
    Checkpoint(RunCheckpointPayload),
    Import(PreparedApplicationCommit<()>),
    Command(PreparedApplicationCommit<BehaviorResult>),
    Unit,
}

impl Operation {
    fn run(self, core: &mut GameCore) -> Result<Output, GameError> {
        match self {
            Self::CaptureProfile => core.capture_player_profile().map(Output::Profile),
            Self::CaptureEntry => BattleEntrySnapshot::capture(&core.state).map(Output::Entry),
            Self::CaptureCheckpoint => RunCheckpointPayload::capture(
                &core.state,
                core.state.run.as_ref().unwrap(),
                core.content_identity(),
            )
            .map(Output::Checkpoint),
            Self::RestoreSharedEntry(entry) => {
                BattleEntrySnapshot::restore_shared(entry, &mut core.state)?;
                Ok(Output::Unit)
            }
            Self::Continue { bytes, loads, peak } => {
                core.continue_from_active_run_checkpoint(inputs().player, &bytes, loads, peak)?;
                Ok(Output::Unit)
            }
            Self::PrepareImport(profile) => core
                .prepare_authoritative_profile_install(profile)
                .map(Output::Import),
            Self::PrepareAbandon => core
                .prepare_with_transport_context(
                    inputs().player,
                    PlayerBehavior::AbandonRun,
                    None,
                    0,
                    None,
                    None,
                )
                .map(Output::Command),
        }
    }
}

impl Output {
    // Expected-value serialization is deliberately outside every budget scope.
    fn fingerprint(self, source: &GameCore) -> Value {
        match self {
            Self::Profile(profile) => json!({"profile":profile}),
            Self::Checkpoint(checkpoint) => json!({"checkpoint":checkpoint}),
            Self::Import(mut prepared) => json!({"draft":roots(prepared.draft()),
                "publication_profile":prepared.take_publication_profile().unwrap()}),
            Self::Command(mut prepared) => json!({"draft":roots(prepared.draft()),
                "publication_profile":prepared.take_publication_profile().unwrap()}),
            Self::Unit => Value::Null,
            Self::Entry(entry) => {
                let mut restored = source.clone();
                entry.restore(&mut restored.state);
                roots(&restored)
            }
        }
    }
}

fn authoring_boundary(
    case: &str,
    label: &str,
    source: &GameCore,
    provenance: NativeStorageProvenance,
    make: impl Fn() -> Operation,
) -> ExecutionUsage {
    let limits = source.runtime_safety().authoring_execution();
    let token = CancellationToken::default();
    let phase = ExecutionPhase::TrialPreparation;
    let before = roots(source);
    let mut successful = source.clone();
    let operation = make(); // Operand allocation is fixture preparation.
    let default = control::run_with_provenance(&limits, &token, phase, provenance, || {
        operation.run(&mut successful)
    });
    let default_ok = default
        .as_ref()
        .map(|(value, _, _)| value.is_ok())
        .unwrap_or(false);
    let default_roots_unchanged = roots(&successful) == before;
    // First default result is visible even when a fixture hits its existing cap.
    println!(
        "U12_G11_DEFAULT {}",
        json!({"case":case,"operation":label,
        "budget":"authoring","manifest_sha256":inputs().manifest_sha256,
        "roots_unchanged":default_roots_unchanged,
        "result":default.as_ref().map(|(r,u,_)|json!({"domain_ok":r.is_ok(),"usage":u,
            "domain_error":r.as_ref().err().map(|error| format!("{error:?}"))}))
            .map_err(ToString::to_string)})
    );
    if !default_ok {
        assert!(
            default_roots_unchanged,
            "default budget refusal must preserve full R"
        );
    }
    let (output, usage, _) =
        default.expect("preserve default failure; do not resize fixture or allowance");
    let output = output.expect("default operation succeeds before deriving a boundary");
    let expected_output = output.fingerprint(source);
    let expected_roots = roots(&successful);
    assert!(
        usage.work > 1 && usage.clone_slots_peak > 1,
        "{label}: positive measured operation required"
    );
    for work in [true, false] {
        let mut exact = limits.clone();
        if work {
            exact.max_work = usage.work;
        } else {
            exact.max_clone_slots = usage.clone_slots_peak;
        }
        let mut candidate = source.clone();
        let operation = make();
        let (output, measured, _) =
            control::run_with_provenance(&exact, &token, phase, provenance, || {
                operation.run(&mut candidate)
            })
            .unwrap();
        assert_eq!(
            measured, usage,
            "{case}/{label}: exact successful operation usage"
        );
        assert_eq!(output.unwrap().fingerprint(source), expected_output);
        assert_eq!(roots(&candidate), expected_roots);
        let mut short = exact;
        if work {
            short.max_work -= 1;
        } else {
            short.max_clone_slots -= 1;
        }
        let mut rejected = source.clone();
        for attempt in 0..2 {
            let operation = make();
            let failure = control::run_with_provenance(&short, &token, phase, provenance, || {
                operation.run(&mut rejected)
            })
            .err()
            .expect("one-short must reject");
            assert_eq!(
                failure.code,
                if work {
                    "authoring_work_limit"
                } else {
                    "authoring_clone_limit"
                }
            );
            assert_eq!(
                roots(&rejected),
                before,
                "{case}/{label} retry {attempt}: full R"
            );
        }
        let operation = make();
        let (retry, _, _) =
            control::run_with_provenance(&limits, &token, phase, provenance, || {
                operation.run(&mut rejected)
            })
            .unwrap();
        assert_eq!(retry.unwrap().fingerprint(source), expected_output);
        assert_eq!(roots(&rejected), expected_roots);
    }
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    let mut candidate = source.clone();
    let entered = std::cell::Cell::new(false);
    let operation = make();
    let failure = control::run_with_provenance(&limits, &cancelled, phase, provenance, || {
        entered.set(true);
        operation.run(&mut candidate)
    })
    .err()
    .expect("pre-cancelled preparation rejects");
    assert_eq!(failure.code, "authoring_cancelled");
    assert!(!entered.get());
    assert_eq!(roots(&candidate), before);
    assert_eq!(roots(source), before);
    println!(
        "U12_G11_BOUNDARY {}",
        json!({"case":case,"operation":label,
        "budget":"authoring","usage":usage,"axes":["work","clone_slots_peak"],
        "at_limit":true,"one_short_repeats":2,"pre_cancelled":true,"retry":true})
    );
    usage
}

#[cfg(feature = "runtime-observation")]
fn native_boundary(
    case: &str,
    label: &str,
    source: &GameCore,
    make: impl Fn() -> Operation,
) -> Option<(u64, u64)> {
    use crate::game::battle::runtime_control::{self, RuntimeFaultCode};
    let limits = source.runtime_safety().native_runtime();
    let before = roots(source);
    let mut successful = source.clone();
    let operation = make();
    // This private diagnostic wrapper is Admin in the existing observation
    // vocabulary; its case label names the exact operation, not a new product API.
    let window = crate::runtime_observation::begin_lab_window().unwrap();
    let result = runtime_control::run(limits, || operation.run(&mut successful));
    let measured = window.finish().expect("no dropped native observation");
    let default_roots_unchanged = roots(&successful) == before;
    println!(
        "U12_G11_DEFAULT {}",
        json!({"case":case,"operation":label,
        "budget":"native","scope":"one explicit runtime_control::run diagnostic",
        "result_ok":result.is_ok(),"error":result.as_ref().err().map(|error| format!("{error:?}")),
        "roots_unchanged":default_roots_unchanged,
        "work":measured.work,"clone_slots":measured.clone_slots,"retained":measured.retained})
    );
    if result.is_err() {
        assert!(
            default_roots_unchanged,
            "default native refusal must preserve full R"
        );
    }
    let expected_output = result
        .expect("preserve first default native failure")
        .fingerprint(source);
    let expected_roots = roots(&successful);
    assert!(
        measured.work > 1 && measured.clone_slots > 1,
        "{label}: measured native operation required"
    );
    for work in [true, false] {
        let mut exact = limits;
        if work {
            exact.max_work = measured.work;
        } else {
            exact.max_clone_slots = usize::try_from(measured.clone_slots).unwrap();
        }
        let mut candidate = source.clone();
        let operation = make();
        let output = runtime_control::run(exact, || operation.run(&mut candidate)).unwrap();
        assert_eq!(output.fingerprint(source), expected_output);
        assert_eq!(roots(&candidate), expected_roots);
        let mut short = exact;
        if work {
            short.max_work -= 1;
        } else {
            short.max_clone_slots -= 1;
        }
        let mut rejected = source.clone();
        for attempt in 0..2 {
            let operation = make();
            let failure = runtime_control::run(short, || operation.run(&mut rejected))
                .err()
                .unwrap();
            assert!(
                matches!(failure, GameError::BattleRuntimeFault(ref fault)
                if fault.code == if work { RuntimeFaultCode::WorkLimit } else { RuntimeFaultCode::CloneLimit }),
                "{case}/{label}: {failure:?}"
            );
            assert_eq!(
                roots(&rejected),
                before,
                "{case}/{label} retry {attempt}: full R"
            );
        }
        let operation = make();
        let output = runtime_control::run(limits, || operation.run(&mut rejected)).unwrap();
        assert_eq!(output.fingerprint(source), expected_output);
        assert_eq!(roots(&rejected), expected_roots);
    }
    assert_eq!(roots(source), before);
    println!(
        "U12_G11_BOUNDARY {}",
        json!({"case":case,"operation":label,
        "budget":"native","work":measured.work,"clone_slots":measured.clone_slots,
        "at_limit":true,"one_short_repeats":2,"retry":true,
        "vault_owner_extent":"not represented by battle/pending/relic owner extent"})
    );
    Some((measured.work, measured.clone_slots))
}

#[cfg(not(feature = "runtime-observation"))]
fn native_boundary(
    case: &str,
    label: &str,
    _: &GameCore,
    _: impl Fn() -> Operation,
) -> Option<(u64, u64)> {
    println!(
        "U12_G11_NATIVE_UNMEASURED {}",
        json!({"case":case,"operation":label,
        "reason":"runtime-observation feature disabled; native boundary evidence requires the separate feature build"})
    );
    None
}

fn both(
    case: &str,
    label: &str,
    source: &GameCore,
    provenance: NativeStorageProvenance,
    make: impl Fn() -> Operation,
) -> (ExecutionUsage, Option<(u64, u64)>) {
    let usage = authoring_boundary(case, label, source, provenance, &make);
    let native = native_boundary(case, label, source, make);
    (usage, native)
}

#[test]
fn u12_vault_profile_capture_charges_owned_graph() {
    let mut prior = None;
    let mut prior_native = None;
    for case in &inputs().cases {
        let source = idle(case);
        assert_eq!(source.capture_player_profile().unwrap(), case.profile);
        let (usage, native) = both(
            &case.name,
            "capture_player_profile",
            &source,
            NativeStorageProvenance::default(),
            || Operation::CaptureProfile,
        );
        if let Some((work, slots)) = prior {
            assert!(
                usage.work > work && usage.clone_slots_peak > slots,
                "larger owned vault graphs must be included in the capture accounting"
            );
        }
        prior = Some((usage.work, usage.clone_slots_peak));
        if let (Some((work, slots)), Some((prior_work, prior_slots))) = (native, prior_native) {
            assert!(
                work > prior_work && slots > prior_slots,
                "larger owned vault graphs must be included in native scan/copy accounting"
            );
        }
        prior_native = native;
    }
}

#[test]
fn u12_vault_entry_checkpoint_restore_preserves_profile_boundary() {
    let mut capture_baseline = None;
    for case in &inputs().cases {
        let (source, provenance) = fresh_fixture(case, true);
        let before = roots(&source);
        unknown_provenance_rejects_entry(&case.name, &source);
        let entry_usage = both(
            &case.name,
            "BattleEntrySnapshot::capture",
            &source,
            provenance,
            || Operation::CaptureEntry,
        )
        .0;
        let checkpoint_usage = both(
            &case.name,
            "RunCheckpointPayload::capture",
            &source,
            provenance,
            || Operation::CaptureCheckpoint,
        )
        .0;
        if let Some((entry, checkpoint)) = &capture_baseline {
            assert_eq!(
                &entry_usage, entry,
                "entry owned payload excludes the permanent profile"
            );
            assert_eq!(
                &checkpoint_usage, checkpoint,
                "checkpoint owned payload excludes the permanent profile"
            );
        } else {
            capture_baseline = Some((entry_usage, checkpoint_usage));
        }
        let entry = Arc::new(BattleEntrySnapshot::capture(&source.state).unwrap());
        let mut destination = source.clone();
        destination.state.enkephalin.amount += 1;
        let profile = destination.state.player_profile.clone();
        both(
            &case.name,
            "BattleEntrySnapshot::restore_shared",
            &destination,
            provenance,
            || Operation::RestoreSharedEntry(Arc::clone(&entry)),
        );
        BattleEntrySnapshot::restore_shared(Arc::clone(&entry), &mut destination.state).unwrap();
        assert_eq!(destination.state.player_profile, profile);
        assert_eq!(roots(&destination), before);
        let checkpoint = source.export_active_run_checkpoint().unwrap().unwrap();
        let (target, target_provenance) = fresh_fixture(case, false);
        both(
            &case.name,
            "Continue_checkpoint_import",
            &target,
            target_provenance,
            || Operation::Continue {
                bytes: checkpoint.core_payload_json.clone(),
                loads: checkpoint.loads_used,
                peak: checkpoint.run_peak_certified_power,
            },
        );
        let mut resumed = idle(case);
        resumed
            .continue_from_active_run_checkpoint(
                inputs().player,
                &checkpoint.core_payload_json,
                checkpoint.loads_used,
                checkpoint.run_peak_certified_power,
            )
            .unwrap();
        assert_eq!(resumed.state.player_profile, case.profile);
        assert_eq!(roots(&resumed), before);
        assert_eq!(roots(&source), before);
    }
}

#[test]
fn u12_vault_prepare_import_budget_failures_preserve_roots() {
    for case in &inputs().cases {
        let (source, provenance) = fresh_fixture(case, true);
        let mut accepted = case.profile.clone();
        accepted.lifecycle.first_return_story = FirstReturnStory::Consumed;
        both(
            &case.name,
            "typed_authoritative_profile_install_prepare",
            &source,
            provenance,
            || Operation::PrepareImport(accepted.clone()),
        );
        both(
            &case.name,
            "AbandonRun_prepare_with_publication_profile",
            &source,
            provenance,
            || Operation::PrepareAbandon,
        );
    }
}

#[test]
fn u12_vault_prepared_discard_and_commit_keep_exact_profile() {
    for case in &inputs().cases {
        let mut core = running(case);
        let before = roots(&core);
        let mut accepted = case.profile.clone();
        accepted.lifecycle.first_return_story = FirstReturnStory::Consumed;
        let mut prepared = core
            .prepare_authoritative_profile_install(accepted.clone())
            .unwrap();
        assert_eq!(prepared.take_publication_profile().unwrap(), accepted);
        assert_eq!(prepared.draft().state.player_profile, accepted);
        drop(prepared);
        assert_eq!(
            roots(&core),
            before,
            "discard owns only the unpublished candidate"
        );
        let prepared = core
            .prepare_authoritative_profile_install(accepted.clone())
            .unwrap();
        core.commit_prepared_application(prepared);
        assert_eq!(core.state.player_profile, accepted);
        let mut expected = before.clone();
        expected["roots"]["profile"] = serde_json::to_value(&accepted).unwrap();
        assert_eq!(roots(&core), expected, "only the accepted profile changes");
        let before_abandon = roots(&core);
        let prepared = core
            .prepare_with_transport_context(
                inputs().player,
                PlayerBehavior::AbandonRun,
                None,
                0,
                None,
                None,
            )
            .unwrap();
        drop(prepared);
        assert_eq!(roots(&core), before_abandon);
        let mut prepared = core
            .prepare_with_transport_context(
                inputs().player,
                PlayerBehavior::AbandonRun,
                None,
                0,
                None,
                None,
            )
            .unwrap();
        assert_eq!(prepared.take_publication_profile().unwrap(), accepted);
        core.commit_prepared_application(prepared);
        assert_eq!(core.state.player_profile, accepted);
        assert!(core.export_active_run_checkpoint().unwrap().is_none());
    }
}
