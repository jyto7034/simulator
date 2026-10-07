#!/usr/bin/env python3
"""Run the inherited R05 allocation guard against the current WP02 source."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
SEGMENT = TOPIC / "execution/single_goal/segments/wp02_effect_execution_prefix_001"
OUTPUT = ROOT / SEGMENT / "evidence/performance/r05_guard_001"
BASELINE = ROOT / "core_enhanced/evidence/P01_PERFORMANCE_BASELINE.json"
MANIFEST = ROOT / SEGMENT / "baseline/pre_implementation_001/MANIFEST.json"
TARGET = "/tmp/r06-wp02-r05-guard-target"
CAP = 9_030_188

CHANGED_SOURCE_PATHS = [
    Path("core_enhanced/src/application.rs"),
    Path("core_enhanced/src/game/battle/core/commands.rs"),
    Path("core_enhanced/src/game/battle/core/damage_runtime.rs"),
    Path("core_enhanced/src/game/battle/core/mod.rs"),
    Path("core_enhanced/src/game/battle/core/resource_runtime.rs"),
    Path("core_enhanced/src/game/battle/core/runtime_facts.rs"),
    Path("core_enhanced/src/game/battle/core/sim.rs"),
    Path("core_enhanced/src/game/battle/core/skill_runtime/area.rs"),
    Path("core_enhanced/src/game/battle/core/skill_runtime/projectile.rs"),
    Path("core_enhanced/src/game/battle/core/target_usefulness.rs"),
    Path("core_enhanced/src/game/battle/core/types.rs"),
    Path("core_enhanced/src/game/battle/core/value_expr_runtime.rs"),
    Path("core_enhanced/src/game/battle/damage.rs"),
    Path("core_enhanced/src/game/battle/enums.rs"),
    Path("core_enhanced/src/game/behavior.rs"),
    Path("core_enhanced/src/game/value_expr/catalog.rs"),
    Path("core_enhanced/src/game/world/snapshot.rs"),
]


def sha(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def row(path: Path) -> dict[str, object]:
    payload = path.read_bytes()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "bytes": len(payload),
        "sha256": sha(payload),
    }


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def peak(workload: dict[str, object]) -> int:
    return max(
        sample["heap_allocation"]["peak_delta_bytes"]
        for sample in workload["allocation_raw_samples"]
    )


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite R05 guard evidence: {OUTPUT}")
    OUTPUT.mkdir(parents=True)
    receipt_path = OUTPUT / "r05_guard_current.json"
    log = OUTPUT / "r05_guard_current.log"
    command = [
        "cargo", "run", "--release", "--offline", "--locked",
        "--manifest-path", "tools/runtime_architecture_final_validation/Cargo.toml",
        "--target-dir", TARGET,
        "--features", "allocation-observer",
        "--bin", "final_performance", "--",
        str(BASELINE.relative_to(ROOT / "core_enhanced")),
        str(receipt_path),
    ]
    environment = {**os.environ, "CARGO_BUILD_JOBS": "4"}
    started = datetime.now(timezone.utc).isoformat()
    completed = subprocess.run(
        command,
        cwd=ROOT / "core_enhanced",
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    log.write_bytes(completed.stdout)
    if completed.returncode != 0 or not receipt_path.is_file():
        raise SystemExit(f"R05 guard failed; see {log.relative_to(ROOT)}")

    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    event_peak = peak(receipt["event_heavy"])
    concurrent_peak = peak(receipt["concurrent_sessions"])
    assertions = {
        "allocation_observer_enabled": receipt["allocation_observer_enabled"] is True,
        "allocation_gate_applicable": receipt["allocation_gate_applicable"] is True,
        "event_repeatable": receipt["event_heavy"]["repeatable"] is True,
        "concurrent_repeatable": receipt["concurrent_sessions"]["repeatable"] is True,
        "event_deterministic_parity": receipt["event_heavy"]["deterministic_baseline_parity"] is True,
        "concurrent_deterministic_parity": receipt["concurrent_sessions"]["deterministic_baseline_parity"] is True,
        "event_allocation_guard": receipt["event_heavy"]["allocation_retained_peak_guard_pass"] is True,
        "concurrent_allocation_guard": receipt["concurrent_sessions"]["allocation_retained_peak_guard_pass"] is True,
        "concurrent_original_cap": concurrent_peak <= CAP,
    }
    assert all(assertions.values()), assertions
    source = [row(ROOT / path) for path in CHANGED_SOURCE_PATHS]
    summary = {
        "record": "R06_WP02_EFFECT_EXECUTION_R05_GUARD_001",
        "started_at_utc": started,
        "finished_at_utc": datetime.now(timezone.utc).isoformat(),
        "result": "DIRECT_GUARD_PASS",
        "command": command,
        "environment": {"CARGO_BUILD_JOBS": "4"},
        "exit_code": completed.returncode,
        "assertions": assertions,
        "original_concurrent_cap_bytes": CAP,
        "event_peak_delta_bytes_max": event_peak,
        "concurrent_peak_delta_bytes_max": concurrent_peak,
        "concurrent_headroom_bytes": CAP - concurrent_peak,
        "baseline": row(BASELINE),
        "implementation_baseline_manifest": row(MANIFEST),
        "changed_source": source,
        "guard_receipt": row(receipt_path),
        "log": row(log),
        "limits_changed": False,
        "review_pass_declared": False,
        "wp_phase_goal_complete": False,
    }
    write_json(OUTPUT / "RECEIPT.json", summary)
    print(json.dumps({
        "result": summary["result"],
        "event_peak_delta_bytes_max": event_peak,
        "concurrent_peak_delta_bytes_max": concurrent_peak,
        "original_concurrent_cap_bytes": CAP,
        "concurrent_headroom_bytes": CAP - concurrent_peak,
    }, indent=2))


if __name__ == "__main__":
    main()
