#!/usr/bin/env python3
"""Run the existing standalone Core API probes with explicit R02 pack input."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, required=True)
    parser.add_argument("--pack-root", type=Path, required=True)
    parser.add_argument("--pack", required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repository.resolve(strict=True)
    pack_root = args.pack_root.resolve(strict=True)
    output = args.output_dir.absolute()
    output.mkdir()
    cases = [(name, "valid", []) for name in [
        "validated_data_api_audit", "application_authority_api_audit",
        "battle_kernel_api_audit", "core_public_projection_api_audit", "history_result_api_audit",
    ]] + [
        ("validated_data_api_audit", "invalid", ["GameDataBuilder", "GameDataBaseParts"]),
        ("application_authority_api_audit", "invalid", ["GameCore", "state", "private"]),
        ("battle_kernel_api_audit", "invalid", ["core", "private"]),
        ("core_public_projection_api_audit", "invalid", ["PlayerBehavior", "BehaviorResult", "private"]),
        ("history_result_api_audit", "invalid", ["CombatResultEventLogAttachmentDto", "GameCore"]),
        ("runtime_architecture_final_validation", "default_fixture_api_audit",
         ["load_live_embedded", "replace_active_battle_event_log_fixture"]),
    ]
    records = []
    for name, kind, diagnostics in cases:
        positive = kind == "valid"
        manifest = repo / "core_enhanced/tools" / name / kind / "Cargo.toml"
        command = ["cargo", "run" if positive else "check", "--release", "--offline", "--locked",
                   "--manifest-path", str(manifest), "--target-dir", str(repo / "target/capg-api")]
        if positive:
            command += ["--", str(pack_root), args.pack]
        started = time.perf_counter()
        result = subprocess.run(command, cwd=repo, capture_output=True, text=True, timeout=120)
        log = output / f"{name}_{kind}.log"
        log.write_text(result.stdout + result.stderr)
        record = {"name": name, "kind": kind, "command": command, "exit_code": result.returncode,
                  "elapsed_seconds": time.perf_counter() - started, "log": str(log),
                  "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest(),
                  "required_diagnostics": diagnostics}
        records.append(record)
        (output / "commands.json").write_text(json.dumps(records, indent=2) + "\n")
        assert (result.returncode == 0) == positive, (name, kind, result.stderr)
        if not positive:
            assert "error[E" in result.stderr, (name, "not a Rust API rejection", result.stderr)
            for diagnostic in diagnostics:
                assert diagnostic in result.stderr, (name, diagnostic, result.stderr)
            if kind != "default_fixture_api_audit":
                assert "no function or associated item named `load_live_embedded`" not in result.stderr
        print(json.dumps({"name": name, "kind": kind, "expected_result_verified": True}), flush=True)
    receipt = {"passed": True, "positive_runtime_consumers": 5, "negative_compile_consumers": 6,
               "commands": records, "pack_root": str(pack_root), "pack": args.pack,
               "scope": "Existing standalone assertion bodies retained; negative result requires the original forbidden API diagnostic, not a loader or Cargo failure."}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"passed": True, "commands": len(records), "receipt": str(output / "receipt.json")}))


if __name__ == "__main__":
    main()
