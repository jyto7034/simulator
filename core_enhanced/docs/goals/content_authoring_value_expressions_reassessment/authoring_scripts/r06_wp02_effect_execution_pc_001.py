#!/usr/bin/env python3
"""Run the unchanged 36-condition PC fixture against the current WP02 source."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
SEGMENT = TOPIC / "execution/single_goal/segments/wp02_effect_execution_prefix_001"
SERIES = os.environ.get("R06_WP02_PC_SERIES", "pc_current_001")
if SERIES not in {"pc_current_001", "pc_current_002"}:
    raise SystemExit(f"invalid PC series: {SERIES}")
OUTPUT = ROOT / SEGMENT / "evidence/performance" / SERIES
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001/MANIFEST.json"
BASE_SANDBOX = Path("/tmp/r06-cap-after-source")
SANDBOX = Path(f"/tmp/r06-wp02-effect-{SERIES}-source")
TARGET = Path("/tmp/r06-cap-after-target")
FIXTURE = ROOT / "core_enhanced/docs/tmp/r06_pc_baseline_20260926/probe.rs"
SUMMARIZER = ROOT / TOPIC / (
    "execution/single_goal/corrections/wp02_cap_01_02_001/verification/pc_summarize.mjs"
)
PREVIOUS_SUMMARY = ROOT / TOPIC / (
    "execution/single_goal/corrections/wp02_cap_01_02_001/verification/pc_after_summary.json"
)

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

HOOK = (
    '\n    #[path = "'
    + FIXTURE.as_posix()
    + '"]\n    mod load_baseline;\n'
).encode()
ADAPTER = b'''#[cfg(test)]
pub(crate) fn baseline_usage() -> Option<(u64, usize, usize)> {
    ACTIVE.with(|slot| slot.borrow().as_ref().map(|budget| {
        (budget.work, budget.clone_slots, budget.retained_admission.unwrap_or(0))
    }))
}
'''


def sha(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def file_row(path: Path) -> dict[str, object]:
    payload = path.read_bytes()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "bytes": len(payload),
        "sha256": sha(payload),
    }


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def summary_key(case: dict[str, object]) -> tuple[object, ...]:
    return (
        case["family"],
        case["n"],
        case.get("chunk"),
        case.get("sources"),
        case.get("reads"),
        case.get("grouped"),
    )


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite PC evidence: {OUTPUT}")
    if not BASE_SANDBOX.is_dir() or not TARGET.is_dir():
        raise SystemExit("preserved isolated PC source/target sandbox is unavailable")
    if SANDBOX.exists():
        raise SystemExit(f"refusing to overwrite PC source sandbox: {SANDBOX}")
    OUTPUT.mkdir(parents=True)

    baseline = json.loads(BASELINE.read_text(encoding="utf-8"))
    shutil.copytree(BASE_SANDBOX, SANDBOX)
    for row in baseline["source"]["files"]:
        relative = Path(row["path"])
        shutil.copy2(ROOT / relative, SANDBOX / relative)

    value_runtime = SANDBOX / "core_enhanced/src/game/battle/core/value_expr_runtime.rs"
    payload = value_runtime.read_bytes()
    assert HOOK not in payload
    assert payload.endswith(b"}\n")
    value_runtime.write_bytes(payload[:-2] + HOOK + b"}\n")

    native_control = SANDBOX / "core_enhanced/src/game/battle/runtime_control.rs"
    native_payload = native_control.read_bytes()
    if ADAPTER not in native_payload:
        marker = b"impl RuntimeFaultCode {"
        assert native_payload.count(marker) == 1
        native_control.write_bytes(native_payload.replace(marker, ADAPTER + b"\n" + marker, 1))

    verified = []
    product_delta = []
    for row in baseline["source"]["files"]:
        relative = Path(row["path"])
        actual = (SANDBOX / relative).read_bytes()
        if relative == Path("core_enhanced/src/game/battle/core/value_expr_runtime.rs"):
            assert actual.count(HOOK) == 1
            actual = actual.replace(HOOK, b"", 1)
        elif relative == Path("core_enhanced/src/game/battle/runtime_control.rs"):
            assert actual.count(ADAPTER) == 1
            actual = actual.replace(ADAPTER + b"\n", b"", 1)
        current = (ROOT / relative).read_bytes()
        assert actual == current, relative
        if sha(current) != row["sha256"]:
            product_delta.append(relative.as_posix())
        verified.append({"path": relative.as_posix(), "bytes": len(current), "sha256": sha(current)})

    raw = OUTPUT / "pc_current_raw.json"
    log = OUTPUT / "pc_current.log"
    command = [
        "cargo", "test", "--manifest-path", str(SANDBOX / "core_enhanced/Cargo.toml"),
        "--offline", "--locked", "--release", "--lib", "pc_load_baseline", "--",
        "--ignored", "--exact",
        "game::battle::core::value_expr_runtime::tests::load_baseline::pc_load_baseline",
        "--nocapture", "--test-threads=1",
    ]
    environment = {
        **os.environ,
        "CARGO_TARGET_DIR": str(TARGET),
        "CARGO_BUILD_JOBS": "4",
        "R06_BASELINE_OUTPUT": str(raw),
        "R06_BASELINE_SAMPLES": "11",
    }
    started = datetime.now(timezone.utc).isoformat()
    completed = subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    log.write_bytes(completed.stdout)
    if completed.returncode != 0 or not raw.is_file():
        raise SystemExit(f"PC fixture failed; see {log.relative_to(ROOT)}")

    shutil.copy2(SUMMARIZER, OUTPUT / "pc_summarize.mjs")
    summarized = subprocess.run(
        ["node", "pc_summarize.mjs", raw.name],
        cwd=OUTPUT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    summary_log = OUTPUT / "pc_summary.log"
    summary_log.write_bytes(summarized.stdout)
    summary_path = OUTPUT / "pc_current_summary.json"
    if summarized.returncode != 0 or not summary_path.is_file():
        raise SystemExit(f"PC summary failed; see {summary_log.relative_to(ROOT)}")

    current_rows = json.loads(summary_path.read_text(encoding="utf-8"))["summary"]
    previous_rows = json.loads(PREVIOUS_SUMMARY.read_text(encoding="utf-8"))["summary"]
    current = {summary_key(row): row for row in current_rows}
    previous = {summary_key(row): row for row in previous_rows}
    assert len(current) == 36 and current.keys() == previous.keys()
    assert all(row["samples"] == 11 and row["allocation_samples"] == 3 for row in current.values())
    assert sum(bool(row["errors"]) for row in current.values()) == 5

    comparisons = []
    for key in sorted(current, key=str):
        before = previous[key]
        after = current[key]
        assert bool(after["errors"]) == bool(before["errors"]), key
        assert after["errors"] == before["errors"], key
        if key[0] in {"literal_battle_public_step", "damage_burst_private_transaction"}:
            assert after["example"]["after_log_sha256"] == before["example"]["after_log_sha256"], key
            assert after["example"]["hp_loss"] == before["example"]["hp_loss"], key
        comparisons.append(
            {
                "case": key,
                "completed": not bool(after["errors"]),
                "faults": after["errors"],
                "previous_requested_bytes_max": before["requested_bytes"]["max"],
                "current_requested_bytes_max": after["requested_bytes"]["max"],
                "previous_native_retained_admission_max": before["native_retained_admission_max"],
                "current_native_retained_admission_max": after["native_retained_admission_max"],
            }
        )

    source_receipt = {
        "source_files_verified": len(verified),
        "product_delta_paths": product_delta,
        "instrumentation_only_paths": [
            "core_enhanced/src/game/battle/core/value_expr_runtime.rs",
            "core_enhanced/src/game/battle/runtime_control.rs",
        ],
        "fixture": file_row(FIXTURE),
        "fixture_expected_sha256": "096f011800275fe93e15ff230e333719b02137b318d8130a9c19b46110d2a11b",
        "fixture_unchanged": sha(FIXTURE.read_bytes())
        == "096f011800275fe93e15ff230e333719b02137b318d8130a9c19b46110d2a11b",
        "files": verified,
    }
    write_json(OUTPUT / "SOURCE_IDENTITY.json", source_receipt)
    comparison = {
        "conditions": 36,
        "time_samples_per_condition": 11,
        "allocation_samples_per_condition": 3,
        "completed_conditions": 31,
        "typed_refusals": 5,
        "typed_refusal_policy": (
            "the five existing 128-Read cumulative retained-admission refusals are preserved; "
            "no limit was raised"
        ),
        "allocation_metric": "sum of requested allocation/reallocation bytes; not live peak or RSS",
        "previous_source": PREVIOUS_SUMMARY.relative_to(ROOT).as_posix(),
        "rows": comparisons,
    }
    write_json(OUTPUT / "COMPARISON.json", comparison)
    receipt = {
        "record": "R06_WP02_EFFECT_EXECUTION_PC_001",
        "started_at_utc": started,
        "finished_at_utc": datetime.now(timezone.utc).isoformat(),
        "result": "DIRECT_MEASUREMENT_COMPLETE",
        "command": command,
        "environment": {
            "CARGO_TARGET_DIR": str(TARGET),
            "CARGO_BUILD_JOBS": "4",
            "R06_BASELINE_SAMPLES": "11",
        },
        "exit_code": completed.returncode,
        "source_identity": file_row(OUTPUT / "SOURCE_IDENTITY.json"),
        "raw": file_row(raw),
        "summary": file_row(summary_path),
        "comparison": file_row(OUTPUT / "COMPARISON.json"),
        "log": file_row(log),
        "summary_log": file_row(summary_log),
        "review_pass_declared": False,
        "wp_phase_goal_complete": False,
    }
    write_json(OUTPUT / "RECEIPT.json", receipt)
    print(json.dumps({
        "result": receipt["result"],
        "conditions": 36,
        "completed": 31,
        "typed_refusals": 5,
        "source_files_verified": len(verified),
    }, indent=2))


if __name__ == "__main__":
    main()
