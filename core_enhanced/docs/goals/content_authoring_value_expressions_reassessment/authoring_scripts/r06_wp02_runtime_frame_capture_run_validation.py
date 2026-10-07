#!/usr/bin/env python3
"""Run and preserve direct validation for WP02 runtime frame/capture segment 001."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
SEGMENT = TOPIC / "execution/single_goal/segments/wp02_runtime_frame_capture_001"
OUTPUT = ROOT / SEGMENT / "evidence/validation"
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001/MANIFEST.json"
PROBE = ROOT / TOPIC / (
    "execution/single_goal/corrections/wp01_target_001_f01_f02/"
    "evidence/probes/review_probes.rs"
)

SOURCE_PATHS = [
    Path("core_enhanced/src/game/battle/core/mod.rs"),
    Path("core_enhanced/src/game/battle/core/runtime_facts.rs"),
    Path("core_enhanced/src/game/battle/core/resource_runtime.rs"),
    Path("core_enhanced/src/game/battle/core/value_expr_runtime.rs"),
    Path("core_enhanced/src/game/data/mod.rs"),
    Path("core_enhanced/src/game/data/content_pack/mod.rs"),
    Path("core_enhanced/src/game/events/combat.rs"),
    Path("core_enhanced/src/game/resource/store.rs"),
    Path("core_enhanced/src/game/value_expr/compile.rs"),
    Path("core_enhanced/src/game/world/combat.rs"),
]

REQUIRED_RECEIPTS = [
    "focused_runtime_frame_capture",
    "r04_runtime_fact_regression",
    "r05_resource_store_regression",
    "r05_resource_lifetime_regression",
    "wp01_value_expr_regression",
    "wp01_content_pipeline_regression",
    "authoring_budget_regression",
    "checkpoint_save_regression",
    "literal_skill_runtime_regression",
    "probe_library_build",
    "wp01_review_probe_compile",
    "wp01_review_probe_after_fixture_fix",
    "wp01_parser_depth_probe",
    "all_targets_check",
    "focused_rustfmt_check",
    "clippy_with_predecessor_allows",
]


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    return {
        "path": relative.as_posix(),
        "bytes": absolute.stat().st_size,
        "sha256": digest(absolute),
    }


def source_identity() -> list[dict[str, object]]:
    return [row(path) for path in SOURCE_PATHS]


def run(
    label: str,
    command: list[str],
    *,
    cwd: Path = ROOT / "core_enhanced",
    environment: dict[str, str] | None = None,
    require_nonzero_tests: bool = False,
) -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    started = datetime.now(timezone.utc).isoformat()
    completed = subprocess.run(
        command,
        cwd=cwd,
        env=None if environment is None else {**os.environ, **environment},
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    output = completed.stdout.decode("utf-8", errors="replace")
    log = OUTPUT / f"{label}.log"
    log.write_text(output, encoding="utf-8")
    tests = [int(value) for value in re.findall(r"running (\d+) tests?", output)]
    zero_match = require_nonzero_tests and (not tests or max(tests) == 0)
    receipt = {
        "label": label,
        "role": "required_direct_validation",
        "started_at_utc": started,
        "finished_at_utc": datetime.now(timezone.utc).isoformat(),
        "cwd": cwd.relative_to(ROOT).as_posix() if cwd != ROOT else ".",
        "command": command,
        "exit_code": completed.returncode,
        "matched_test_counts": tests,
        "nonzero_test_selection_required": require_nonzero_tests,
        "zero_match": zero_match,
        "source_identity": source_identity(),
        "baseline_manifest_sha256": digest(BASELINE),
        "output": row(log.relative_to(ROOT)),
    }
    (OUTPUT / f"{label}.json").write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    if completed.returncode != 0 or zero_match:
        raise SystemExit(f"{label} failed; see {log.relative_to(ROOT)}")


def main() -> None:
    run(
        "focused_runtime_frame_capture",
        [
            "cargo", "test", "-p", "game_core", "--lib",
            "game::battle::core::value_expr_runtime", "--offline", "--locked",
            "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "r04_runtime_fact_regression",
        [
            "cargo", "test", "-p", "game_core", "--lib",
            "game::battle::core::runtime_facts", "--offline", "--locked",
            "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "r05_resource_store_regression",
        [
            "cargo", "test", "-p", "game_core", "--lib", "game::resource",
            "--offline", "--locked", "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "r05_resource_lifetime_regression",
        [
            "cargo", "test", "-p", "game_core", "--lib",
            "game::world::tests::combat::resource_lifetimes", "--offline", "--locked",
            "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "wp01_value_expr_regression",
        [
            "cargo", "test", "-p", "game_core", "--lib", "game::value_expr::",
            "--offline", "--locked", "--", "--test-threads=1", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "wp01_content_pipeline_regression",
        [
            "cargo", "test", "-p", "game_core", "game::data::content_pack::",
            "--lib", "--offline", "--locked", "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "authoring_budget_regression",
        [
            "cargo", "test", "-p", "game_core", "authoring::tests::", "--lib",
            "--offline", "--locked", "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "checkpoint_save_regression",
        [
            "cargo", "test", "-p", "game_core", "checkpoint_validation", "--lib",
            "--offline", "--locked", "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "literal_skill_runtime_regression",
        [
            "cargo", "test", "-p", "game_core", "--test", "skill_runtime_contract",
            "--offline", "--locked", "--", "--nocapture",
        ],
        require_nonzero_tests=True,
    )
    run(
        "probe_library_build",
        ["cargo", "build", "-p", "game_core", "--lib", "--offline", "--locked"],
    )

    fixture = Path(tempfile.mkdtemp(prefix="r06-wp02-runtime-frame-probe-"))
    fixture_root = fixture / "pack-fixture"
    fixture_root.mkdir(parents=True)
    binary = fixture / "review_probes"
    run(
        "wp01_review_probe_compile",
        [
            "rustc", "--edition=2021", "--test", str(PROBE.relative_to(ROOT)),
            "--extern", "game_core=target/debug/libgame_core.rlib",
            "-L", "dependency=target/debug/deps", "-o", str(binary),
        ],
        cwd=ROOT,
    )
    run(
        "wp01_review_probe_after_fixture_fix",
        [str(binary), "--test-threads=1", "--nocapture"],
        cwd=ROOT,
        environment={"R06_WP01_REVIEW_FIXTURE_ROOT": str(fixture_root)},
        require_nonzero_tests=True,
    )
    run(
        "wp01_parser_depth_probe",
        [
            str(binary), "--ignored", "--exact", "parser_default_depth_on_worker_stack",
            "--test-threads=1", "--nocapture",
        ],
        cwd=ROOT,
        environment={"R06_WP01_REVIEW_FIXTURE_ROOT": str(fixture_root)},
        require_nonzero_tests=True,
    )
    run(
        "all_targets_check",
        ["cargo", "check", "-p", "game_core", "--all-targets", "--offline", "--locked"],
    )
    run(
        "focused_rustfmt_check",
        [
            "rustfmt", "--edition", "2021", "--check", "--config", "skip_children=true",
            *[path.as_posix() for path in SOURCE_PATHS],
        ],
        cwd=ROOT,
    )
    run(
        "clippy_with_predecessor_allows",
        [
            "cargo", "clippy", "-p", "game_core", "--lib", "--tests", "--",
            "-A", "clippy::never_loop", "-A", "clippy::eq_op",
        ],
    )

    summary = {
        "record": "R06_WP02_RUNTIME_FRAME_CAPTURE_VALIDATION_001",
        "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        "result": "DIRECT_VALIDATION_COMPLETE",
        "required_receipts": REQUIRED_RECEIPTS,
        "invalid_attempts_not_counted": ["wp01_review_probe"],
        "development_failures": row((OUTPUT / "development_failures.json").relative_to(ROOT)),
        "external_pack_fixture": str(fixture),
        "original_game_resources_modified": False,
        "review_pass_declared": False,
    }
    (OUTPUT / "VALIDATION_SUMMARY.json").write_text(
        json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
