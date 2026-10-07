#!/usr/bin/env python3
"""Freeze the checkpoint-001 F01-F04 correction target for re-review."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
CORRECTION = EXECUTION / "corrections/checkpoint_001_f01_f04"
BASELINE = ROOT / CORRECTION / "baseline/pre_fix_001"
PARENT = ROOT / EXECUTION / "checkpoints/checkpoint_001"
OUTPUT = ROOT / CORRECTION / "target/correction_001"
ARCHIVE = OUTPUT / "SOURCE_AFTER_CORRECTION_001.tar.gz"
DELTA = OUTPUT / "CORRECTION_DELTA.patch"
REVIEW = ROOT / EXECUTION / "reviews/checkpoint_001_review_001"

EXPECTED_SOURCE_CHANGES = {
    "core_enhanced/src/game/value_expr/compile.rs",
    "core_enhanced/src/game/value_expr/eval.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/tests.rs",
}

RECORDS = [
    CORRECTION / "PLAN.md",
    CORRECTION / "CORRECTION_NOTES.md",
    CORRECTION / "CORRECTION_REVIEW_HANDOFF.md",
    CORRECTION / "probes/correction_probes.rs",
    CORRECTION / "probes/fingerprint_probe.rs",
    TOPIC / "authoring_scripts/r06_correction_baseline.py",
    TOPIC / "authoring_scripts/r06_run_correction_validation.py",
    TOPIC / "authoring_scripts/r06_verify_fingerprint_compat.py",
    TOPIC / "authoring_scripts/r06_correction_target.py",
    TOPIC / "authoring_scripts/r06_verify_correction_target.py",
]

SUCCESS_LABELS = {
    "pre_fix_build",
    "pre_fix_original_unit",
    "pre_fix_review_probe_compile",
    "post_fix_all_targets_check",
    "post_fix_all_targets_test",
    "post_fix_clippy_with_baseline_allows",
    "post_fix_correction_probe",
    "post_fix_correction_probe_compile",
    "post_fix_fingerprint_compat",
    "post_fix_focused_fmt",
    "post_fix_probe_build",
    "post_fix_value_expr_unit",
}

EXPECTED_FAILURES = {
    "pre_fix_review_probe": {
        "exit_code": 101,
        "classification": "pre-fix reproduction: original review findings reproduced (2 pass / 5 fail)",
    },
    "post_fix_all_targets_test_sandbox": {
        "exit_code": 101,
        "classification": "environment: Unix socket creation returned EPERM; identical approved rerun passed",
    },
    "post_fix_fingerprint_compat_attempt_001": {
        "exit_code": 1,
        "classification": "verifier setup defect before comparison; corrected verifier rerun passed",
    },
}


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        args,
        cwd=ROOT,
        check=check,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def file_row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    if not absolute.is_file():
        raise SystemExit(f"missing required file: {relative.as_posix()}")
    return {
        "path": relative.as_posix(),
        "sha256": digest(absolute),
        "bytes": absolute.stat().st_size,
    }


def rows(paths: list[Path]) -> list[dict[str, object]]:
    return [file_row(path) for path in paths]


def validate_detached(directory: Path) -> tuple[dict[str, object], str]:
    manifest_path = directory / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (directory / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit(f"detached hash mismatch: {manifest_path.relative_to(ROOT)}")
    return json.loads(manifest_path.read_text(encoding="utf-8")), manifest_hash


def validate_review_preservation(baseline: dict[str, object]) -> list[dict[str, object]]:
    preserved = []
    for expected in baseline["review_evidence"]:
        current = file_row(Path(expected["path"]))
        if current != expected:
            raise SystemExit(f"frozen review evidence changed: {expected['path']}")
        preserved.append(current)
    actual_paths = sorted(
        path.relative_to(ROOT).as_posix() for path in REVIEW.rglob("*") if path.is_file()
    )
    expected_paths = sorted(row["path"] for row in baseline["review_evidence"])
    if actual_paths != expected_paths:
        raise SystemExit("frozen review evidence file set changed")
    return preserved


def source_state(baseline: dict[str, object]) -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    current = []
    changes = []
    for before in baseline["source"]["files"]:
        after = file_row(Path(before["path"]))
        current.append(after)
        if before == after:
            continue
        changes.append(
            {
                "path": before["path"],
                "state": "modified",
                "before_sha256": before["sha256"],
                "before_bytes": before["bytes"],
                "after_sha256": after["sha256"],
                "after_bytes": after["bytes"],
            }
        )
    actual_changes = {row["path"] for row in changes}
    if actual_changes != EXPECTED_SOURCE_CHANGES:
        raise SystemExit(
            f"unexpected source delta: expected {sorted(EXPECTED_SOURCE_CHANGES)}, "
            f"got {sorted(actual_changes)}"
        )
    return current, changes


def write_delta(before_archive: Path, changes: list[dict[str, object]]) -> bytes:
    with tempfile.TemporaryDirectory(prefix="r06-correction-target-") as raw_temp:
        temporary = Path(raw_temp)
        extracted = temporary / "extracted"
        before_changed = temporary / "before"
        after_changed = temporary / "after"
        extracted.mkdir()
        before_changed.mkdir()
        after_changed.mkdir()
        with tarfile.open(before_archive, "r:gz") as archive:
            archive.extractall(extracted, filter="data")
        for change in changes:
            relative = Path(change["path"])
            before_source = extracted / relative
            before_destination = before_changed / relative
            before_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(before_source, before_destination)
            after_destination = after_changed / relative
            after_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, after_destination)
        completed = run(
            "git",
            "diff",
            "--no-index",
            "--binary",
            "--src-prefix=pre_fix/",
            "--dst-prefix=corrected/",
            str(before_changed),
            str(after_changed),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise RuntimeError(completed.stderr.decode(errors="replace"))
        return completed.stdout


def validation_rows() -> tuple[list[dict[str, object]], list[dict[str, object]]]:
    validation_root = ROOT / CORRECTION / "evidence/validation"
    receipts = []
    evidence_paths = sorted(
        (path.relative_to(ROOT) for path in validation_root.iterdir() if path.is_file()),
        key=lambda path: path.as_posix(),
    )
    receipt_paths = sorted(validation_root.glob("*.json"))
    actual_labels = {path.stem for path in receipt_paths}
    expected_labels = SUCCESS_LABELS | set(EXPECTED_FAILURES)
    if actual_labels != expected_labels:
        raise SystemExit(
            f"validation receipt set mismatch: missing={sorted(expected_labels - actual_labels)} "
            f"unexpected={sorted(actual_labels - expected_labels)}"
        )
    for receipt_path in receipt_paths:
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        label = receipt["label"]
        output = receipt["output"]
        output_path = ROOT / output["path"]
        if digest(output_path) != output["sha256"] or output_path.stat().st_size != output["bytes"]:
            raise SystemExit(f"validation output mismatch: {label}")
        if label in SUCCESS_LABELS:
            expected_exit = 0
            classification = "pass"
        else:
            expected_exit = EXPECTED_FAILURES[label]["exit_code"]
            classification = EXPECTED_FAILURES[label]["classification"]
        if receipt["exit_code"] != expected_exit:
            raise SystemExit(
                f"unexpected validation exit: {label}: {receipt['exit_code']} != {expected_exit}"
            )
        receipts.append(
            {
                "label": label,
                "exit_code": receipt["exit_code"],
                "classification": classification,
                "command": receipt["command"],
                "receipt": receipt_path.relative_to(ROOT).as_posix(),
                "receipt_sha256": digest(receipt_path),
                "output": output,
            }
        )
    return receipts, rows(evidence_paths)


def main() -> None:
    if (OUTPUT / "MANIFEST.json").exists():
        raise SystemExit("correction target already exists; frozen target will not be overwritten")

    baseline, baseline_hash = validate_detached(BASELINE)
    parent, parent_hash = validate_detached(PARENT)
    if parent_hash != baseline["parent_target"]["manifest_sha256"]:
        raise SystemExit("parent target no longer matches correction baseline")
    if parent["record"] != "R06_STAGE_REVIEW_TARGET_001":
        raise SystemExit("unexpected parent target record")
    review_rows = validate_review_preservation(baseline)
    current_rows, changes = source_state(baseline)
    validations, evidence = validation_rows()

    OUTPUT.mkdir(parents=True, exist_ok=False)
    with tarfile.open(ARCHIVE, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for source_row in current_rows:
            relative = Path(source_row["path"])
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    DELTA.write_bytes(write_delta(ROOT / baseline["source"]["archive"]["path"], changes))

    scope = list(parent["current_source"]["scope"])
    scoped_status = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--", *scope
    ).stdout
    scoped_status_path = OUTPUT / "SCOPED_GIT_STATUS_AFTER_CORRECTION.txt"
    scoped_status_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    global_status_path = OUTPUT / "GLOBAL_GIT_STATUS_AFTER_CORRECTION.txt"
    global_status_path.write_bytes(global_status)

    manifest = {
        "record": "R06_CHECKPOINT_001_CORRECTION_TARGET_001",
        "state": "corrected_re_review_requested",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").stdout.decode().strip(),
        "head": run("git", "rev-parse", "HEAD").stdout.decode().strip(),
        "handoff": "HA-CAVER-P2G60-001",
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "pipeline_revision": "2026-09-13-gex-r7",
        "correction_scope": ["F01", "F02", "F03", "F04", "same-cause kernel paths"],
        "completion_claims": {
            "correction_re_review_ready": True,
            "next_r06_feature_started": False,
            "gex_20_started": False,
            "gex_30_started": False,
            "work_package_complete": False,
            "phase_complete": False,
            "goal_complete": False,
        },
        "parent_target": {
            "record": parent["record"],
            "manifest": (PARENT / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": parent_hash,
            "detached_hash_matches": True,
        },
        "pre_fix_baseline": {
            "record": baseline["record"],
            "manifest": (BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": baseline_hash,
            "detached_hash_matches": True,
            "source_archive": baseline["source"]["archive"],
        },
        "preservation": {
            "parent_checkpoint_unchanged": True,
            "review_evidence_unchanged": True,
            "review_evidence": review_rows,
        },
        "current_source": {
            "scope": scope,
            "file_count": len(current_rows),
            "files": current_rows,
            "archive": {
                "path": ARCHIVE.relative_to(ROOT).as_posix(),
                "sha256": digest(ARCHIVE),
                "bytes": ARCHIVE.stat().st_size,
            },
            "changes_from_pre_fix": {
                "added": 0,
                "modified": len(changes),
                "deleted": 0,
                "files": changes,
            },
            "recoverable_delta": {
                "path": DELTA.relative_to(ROOT).as_posix(),
                "sha256": digest(DELTA),
                "bytes": DELTA.stat().st_size,
                "note": "The before/after archives are authoritative actual-byte inputs; this is a readable binary-capable comparison.",
            },
            "scoped_status": {
                "path": scoped_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(scoped_status_path),
                "bytes": scoped_status_path.stat().st_size,
                "rows": len(scoped_status.splitlines()),
            },
            "global_status": {
                "path": global_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(global_status_path),
                "bytes": global_status_path.stat().st_size,
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before this manifest/hash pair was written to avoid self-reference.",
            },
        },
        "findings": {
            "F01": "pre-clone/pre-sort raw and payload work plus conservative compile/evaluation storage admission",
            "F02": "direct-child node precheck, sequential Add traversal, and streaming fingerprint",
            "F03": "nested simultaneous value-stack requirement with entry and push enforcement; separate task/group protection",
            "F04": "cumulative resources plus TargetEmployee timing capability checks",
        },
        "stage_review": [
            "pre-fix parent/source/review currentness frozen",
            "pre-fix unit pass and original probe 2-pass/5-fail reproduction frozen",
            "F01-F04 and same-cause kernel paths corrected without changing core design",
            "boundary, allocation, regression, all-target, lint, and fingerprint checks passed",
            "corrected target frozen; re-review requested and mutation stopped",
        ],
        "execution_records": rows(RECORDS),
        "evidence": evidence,
        "validation_receipts": validations,
        "validation_summary": {
            "pre_fix_value_expr_unit": "11 passed",
            "pre_fix_review_probe": "2 passed / 5 expected failures",
            "post_fix_value_expr_unit": "18 passed",
            "post_fix_correction_probe": "7 passed",
            "post_fix_all_targets": "1378 lib tests plus 52 integration tests passed",
            "fingerprints": "3/3 pre-fix and corrected byte values identical",
            "sandbox_exception": "one Unix socket EPERM in sandbox; identical approved rerun passed",
        },
        "review_entry": (CORRECTION / "CORRECTION_REVIEW_HANDOFF.md").as_posix(),
        "mutation_boundary": (
            "Do not mutate correction source, records, or evidence after generation and "
            "read-only verification until re-review returns."
        ),
        "recovery": {
            "safe_method": "Extract both correction archives into isolated directories and inspect CORRECTION_DELTA.patch; do not overwrite the dirty worktree blindly.",
            "preexisting_worktree_preserved": True,
        },
    }
    manifest_path = OUTPUT / "MANIFEST.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (OUTPUT / "MANIFEST.sha256").write_text(
        f"{digest(manifest_path)}  MANIFEST.json\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
