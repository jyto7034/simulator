#!/usr/bin/env python3
"""Run direct validation for the frozen WP02 sequential-effect segment."""

from __future__ import annotations

from collections import Counter
from datetime import datetime, timezone
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
SEGMENT = TOPIC / "execution/single_goal/segments/wp02_effect_execution_prefix_001"
SERIES = os.environ.get("R06_WP02_VALIDATION_SERIES", "validation")
if SERIES not in {"validation", "validation_final"}:
    raise SystemExit(f"invalid validation series: {SERIES}")
OUTPUT = ROOT / SEGMENT / "evidence" / SERIES
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001/MANIFEST.json"
BASELINE_ARCHIVE = ROOT / SEGMENT / (
    "baseline/pre_implementation_001/SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz"
)
TARGET_DIR = "/tmp/r06-wp02-validation-target"

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


def digest_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def digest(path: Path) -> str:
    return digest_bytes(path.read_bytes())


def identity(path: Path) -> dict[str, object]:
    payload = path.read_bytes()
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "bytes": len(payload),
        "sha256": digest_bytes(payload),
    }


def source_identity() -> list[dict[str, object]]:
    return [identity(ROOT / path) for path in CHANGED_SOURCE_PATHS]


def write_json(path: Path, value: object) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )


def run(
    label: str,
    command: list[str],
    *,
    require_nonzero_tests: bool = False,
) -> dict[str, object]:
    started = datetime.now(timezone.utc).isoformat()
    environment = {
        **os.environ,
        "CARGO_TARGET_DIR": TARGET_DIR,
        "CARGO_BUILD_JOBS": "4",
    }
    completed = subprocess.run(
        command,
        cwd=ROOT / "core_enhanced",
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    output = completed.stdout.decode("utf-8", errors="replace")
    log = OUTPUT / f"{label}.log"
    log.write_text(output, encoding="utf-8")
    counts = [int(value) for value in re.findall(r"running (\d+) tests?", output)]
    zero_match = require_nonzero_tests and (not counts or max(counts) == 0)
    receipt = {
        "label": label,
        "role": "required_direct_validation",
        "started_at_utc": started,
        "finished_at_utc": datetime.now(timezone.utc).isoformat(),
        "cwd": "core_enhanced",
        "command": command,
        "environment": {
            "CARGO_TARGET_DIR": TARGET_DIR,
            "CARGO_BUILD_JOBS": "4",
        },
        "exit_code": completed.returncode,
        "matched_test_counts": counts,
        "nonzero_test_selection_required": require_nonzero_tests,
        "zero_match": zero_match,
        "source_identity": source_identity(),
        "baseline_manifest_sha256": digest(BASELINE),
        "validator_source_sha256": digest(Path(__file__)),
        "output": identity(log),
    }
    write_json(OUTPUT / f"{label}.json", receipt)
    return receipt


def format_edits(before: bytes, after: bytes) -> Counter[tuple[object, ...]]:
    before_lines = before.decode("utf-8").splitlines(keepends=True)
    after_lines = after.decode("utf-8").splitlines(keepends=True)
    edits: Counter[tuple[object, ...]] = Counter()
    for tag, first_start, first_end, second_start, second_end in difflib.SequenceMatcher(
        None, before_lines, after_lines, autojunk=False
    ).get_opcodes():
        if tag != "equal":
            edits[(
                tag,
                tuple(before_lines[first_start:first_end]),
                tuple(after_lines[second_start:second_end]),
            )] += 1
    return edits


def focused_rustfmt() -> dict[str, object]:
    baseline_root = Path(tempfile.mkdtemp(prefix="r06-wp02-format-baseline-"))
    with tarfile.open(BASELINE_ARCHIVE, "r:gz") as archive:
        archive.extractall(baseline_root, filter="data")
    rows = []
    unmatched_total = 0
    for relative in CHANGED_SOURCE_PATHS:
        baseline = (baseline_root / relative).read_bytes()
        current = (ROOT / relative).read_bytes()
        with tempfile.TemporaryDirectory(prefix="r06-wp02-rustfmt-") as temporary:
            temporary = Path(temporary)
            baseline_file = temporary / "baseline.rs"
            current_file = temporary / "current.rs"
            baseline_file.write_bytes(baseline)
            current_file.write_bytes(current)
            for candidate in (baseline_file, current_file):
                subprocess.run(
                    [
                        "rustfmt",
                        "--edition",
                        "2021",
                        "--config",
                        "skip_children=true",
                        str(candidate),
                    ],
                    check=True,
                    cwd=ROOT,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
            baseline_edits = format_edits(baseline, baseline_file.read_bytes())
            current_edits = format_edits(current, current_file.read_bytes())
        unmatched = current_edits - baseline_edits
        unmatched_count = sum(unmatched.values())
        unmatched_total += unmatched_count
        rows.append(
            {
                "path": relative.as_posix(),
                "baseline_preexisting_edit_groups": sum(baseline_edits.values()),
                "current_edit_groups": sum(current_edits.values()),
                "new_or_modified_region_edit_groups": unmatched_count,
            }
        )
    log = OUTPUT / "focused_rustfmt_check.log"
    log.write_text(
        "\n".join(
            f"{row['path']} baseline={row['baseline_preexisting_edit_groups']} "
            f"current={row['current_edit_groups']} "
            f"unmatched={row['new_or_modified_region_edit_groups']}"
            for row in rows
        )
        + "\n",
        encoding="utf-8",
    )
    receipt = {
        "label": "focused_rustfmt_check",
        "role": "required_direct_validation",
        "method": (
            "rustfmt current and preserved baseline copies independently; every current "
            "format edit group must be an exact member of the baseline pre-existing debt multiset"
        ),
        "command": [
            "rustfmt",
            "--edition",
            "2021",
            "--config",
            "skip_children=true",
            "<each changed source copy>",
        ],
        "exit_code": 0 if unmatched_total == 0 else 1,
        "new_or_modified_region_edit_groups": unmatched_total,
        "rows": rows,
        "source_identity": source_identity(),
        "baseline_manifest_sha256": digest(BASELINE),
        "validator_source_sha256": digest(Path(__file__)),
        "output": identity(log),
    }
    write_json(OUTPUT / "focused_rustfmt_check.json", receipt)
    return receipt


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit(f"refusing to overwrite validation evidence: {OUTPUT}")
    OUTPUT.mkdir(parents=True)
    receipts = [
        run(
            "focused_effect_execution",
            [
                "cargo", "test", "-p", "game_core", "--lib", "--offline", "--locked",
                "game::battle::core::value_expr_runtime", "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "delivery_error_propagation",
            [
                "cargo", "test", "-p", "game_core", "--lib", "--offline", "--locked",
                "expression_error_propagates", "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "projectile_snapshot_lifetime",
            [
                "cargo", "test", "-p", "game_core", "--lib", "--offline", "--locked",
                "released_projectile_keeps_snapshot_expression_after_source_withdrawal",
                "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "runtime_fact_regression",
            [
                "cargo", "test", "-p", "game_core", "--lib", "--offline", "--locked",
                "game::battle::core::runtime_facts", "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "resource_regression",
            [
                "cargo", "test", "-p", "game_core", "--lib", "--offline", "--locked",
                "game::resource", "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "literal_skill_runtime_regression",
            [
                "cargo", "test", "-p", "game_core", "--test", "skill_runtime_contract",
                "--offline", "--locked", "--", "--nocapture",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "all_targets_check",
            [
                "cargo", "check", "-p", "game_core", "--all-targets", "--offline", "--locked",
            ],
        ),
        run(
            "all_targets_test",
            [
                "cargo", "test", "-p", "game_core", "--all-targets", "--offline", "--locked",
            ],
            require_nonzero_tests=True,
        ),
        run(
            "clippy_with_predecessor_allows",
            [
                "cargo", "clippy", "-p", "game_core", "--all-targets", "--offline", "--locked",
                "--", "-A", "clippy::never_loop", "-A", "clippy::eq_op",
            ],
        ),
        focused_rustfmt(),
    ]
    failed = [
        receipt["label"]
        for receipt in receipts
        if receipt["exit_code"] != 0 or receipt.get("zero_match", False)
    ]
    summary = {
        "record": "R06_WP02_EFFECT_EXECUTION_VALIDATION_001",
        "completed_at_utc": datetime.now(timezone.utc).isoformat(),
        "result": "DIRECT_VALIDATION_COMPLETE" if not failed else "DIRECT_VALIDATION_FAILED",
        "receipts": [receipt["label"] for receipt in receipts],
        "failed": failed,
        "changed_source_files": len(CHANGED_SOURCE_PATHS),
        "validator_source_sha256": digest(Path(__file__)),
        "review_pass_declared": False,
        "wp_phase_goal_complete": False,
    }
    write_json(OUTPUT / "VALIDATION_SUMMARY.json", summary)
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
