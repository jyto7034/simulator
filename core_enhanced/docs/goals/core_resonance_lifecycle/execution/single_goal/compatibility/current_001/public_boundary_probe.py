"""Compile public release capabilities and their deliberate privacy failures."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
sys.path.insert(0, '/mnt/f/work/simulator/core_enhanced/docs/goals/core_resonance_lifecycle/tools')
from initialize import ROOT
HERE = Path('/mnt/f/work/simulator/core_enhanced/docs/goals/core_resonance_lifecycle/execution/single_goal/compatibility/current_001')

label = sys.argv[1] if len(sys.argv) > 1 else "001"
assert label.isdigit()
out = HERE / ("evidence/public_boundary_" + label)
out.mkdir(exist_ok=False)
template = ROOT / "core_enhanced/tools/authoring_trial_api_probe/Cargo.toml"
configuration = template.read_text().replace('path = "../.."', 'path = ' + json.dumps(str(ROOT / "core_enhanced")))
cases = {
    "valid": (0, None, None, '''
use game_core::{application::CoreApplication, runtime_safety::*};
fn check(app: &mut CoreApplication) {
    let _settings = app.runtime_safety().effective_sha256();
    let _ = app.capture_persistence_rollback();
    let settings = RuntimeSafetySettings::from_ron("host", EMBEDDED_RUNTIME_SAFETY_RON).unwrap();
    settings.verify_effective_sha256(settings.effective_sha256()).unwrap();
}
'''),
    "rollback_private": (101, "E0616", "revision", 'fn check(t: &game_core::application::ApplicationPersistenceRollback) { let _ = t.revision; }'),
    "rollback_clone": (101, "E0277", "Clone", 'fn need<T: Clone>() {} fn check() { need::<game_core::application::ApplicationPersistenceRollback>(); }'),
    "rollback_serialize": (101, "E0277", "Serialize", 'fn check(t: &game_core::application::ApplicationPersistenceRollback) { let _ = serde_json::to_string(t); }'),
    "rollback_deserialize": (101, "E0277", "Deserialize", 'fn check() { let _ = serde_json::from_str::<game_core::application::ApplicationPersistenceRollback>("{}"); }'),
    "rollback_reuse": (101, "E0382", "moved", 'fn check(app: &mut game_core::application::CoreApplication, t: game_core::application::ApplicationPersistenceRollback) { let _ = app.restore_persistence_rollback(t); let _ = app.restore_persistence_rollback(t); }'),
    "settings_private": (101, "E0616", "native_runtime", 'fn check(s: &mut game_core::runtime_safety::RuntimeSafetySettings) { s.native_runtime.max_work = 999; }'),
}
rows = []
inputs = set()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(name, manifest, expected, code=None, diagnostic=None):
    command = ["cargo", "check", "--release", "--offline", "--locked", "--manifest-path", str(manifest)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True)
    log = out / (name + ".log")
    log.write_bytes(result.stdout + result.stderr)
    assert result.returncode == expected, (name, result.returncode, result.stderr[-2000:])
    if code:
        assert f"error[{code}]".encode() in result.stderr and diagnostic.encode() in result.stderr, name
        assert b"no matching package" not in result.stderr and b"could not resolve" not in result.stderr
    rows.append(dict(name=name, command=command, exit_code=result.returncode, expected=expected, code=code, diagnostic=diagnostic, log_sha256=sha(log)))
    for path in manifest.parent.rglob("*"):
        if path.is_file() and "target" not in path.relative_to(manifest.parent).parts:
            inputs.add(path)
    (out / "PROGRESS.json").write_text(json.dumps(rows, indent=2) + "\n")


for name, (expected, code, diagnostic, source) in cases.items():
    project = out / name
    (project / "src").mkdir(parents=True)
    (project / "Cargo.toml").write_text(configuration)
    (project / "Cargo.lock").write_bytes(template.with_name("Cargo.lock").read_bytes())
    (project / "src/main.rs").write_text(source + "\nfn main() {}\n")
    check(name, project / "Cargo.toml", expected, code, diagnostic)

for family, code, diagnostic in (
    ("application_authority_api_audit", "E0603", "GameCore"),
    ("validated_data_api_audit", "E0432", "GameDataBuilder"),
    ("history_result_api_audit", "E0432", "CombatResultEventLogAttachmentDto"),
):
    base = ROOT / "core_enhanced/tools" / family
    check(family + "_valid", base / "valid/Cargo.toml", 0)
    check(family + "_invalid", base / "invalid/Cargo.toml", 101, code, diagnostic)
inputs.add(ROOT / "core_enhanced/tools/external_pack_probe.rs")
inputs.add(ROOT / "core_enhanced/tools/battle_kernel_api_audit/valid/src/main.rs")
(out / "SUMMARY.json").write_text(json.dumps(dict(passed=True, cases=rows, input_files={str(path.relative_to(ROOT)): sha(path) for path in sorted(inputs)}), indent=2) + "\n")
print(json.dumps(dict(passed=True, cases=len(rows))))
