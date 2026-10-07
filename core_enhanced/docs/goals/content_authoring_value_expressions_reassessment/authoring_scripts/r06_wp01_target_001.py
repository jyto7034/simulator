#!/usr/bin/env python3
"""Freeze the reviewed WP01 content-input/compile implementation segment."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
SEGMENT = TOPIC / "execution/single_goal/segments/wp01_content_input_compile_001"
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001"
CHECKPOINT = ROOT / SEGMENT / "checkpoints/checkpoint_001"
OUTPUT = ROOT / SEGMENT / "target/target_001"
SOURCE_ARCHIVE = OUTPUT / "SOURCE_AT_TARGET_001.tar.gz"
RECORD_ARCHIVE = OUTPUT / "RECORDS_AT_TARGET_001.tar.gz"
EVIDENCE_ARCHIVE = OUTPUT / "EVIDENCE_AT_TARGET_001.tar.gz"
DELTA = OUTPUT / "SOURCE_DELTA.patch"
SELF_CHECK = OUTPUT / "FREEZE_SELF_CHECK.json"

RECORDS = [
    SEGMENT / "PLAN.md",
    SEGMENT / "EXPERIMENTS.md",
    SEGMENT / "EXPERIMENT_NOTES.md",
    SEGMENT / "REVIEW_HANDOFF.md",
    SEGMENT / "checkpoints/checkpoint_001/CHECKPOINT_HANDOFF.md",
    SEGMENT / "checkpoints/checkpoint_001/MANIFEST.json",
    SEGMENT / "checkpoints/checkpoint_001/MANIFEST.sha256",
    SEGMENT / "reviews/checkpoint_001_self_review/REVIEW.md",
    SEGMENT / "reviews/checkpoint_001_self_review/CORRECTION_REVIEW.md",
    TOPIC / "authoring_scripts/r06_wp01_run_validation.py",
    TOPIC / "authoring_scripts/r06_wp01_checkpoint_001.py",
    TOPIC / "authoring_scripts/r06_wp01_verify_checkpoint_001.py",
    TOPIC / "authoring_scripts/r06_wp01_target_001.py",
    TOPIC / "authoring_scripts/r06_wp01_verify_target_001.py",
]

REQUIRED_SUCCESS = {
    "stage_002_value_expr_unit",
    "stage_003_content_pipeline_regression",
    "stage_002_authoring_regression",
    "stage_002_skill_runtime_regression",
    "stage_002_checkpoint_save_regression",
    "stage_002_v7_checksum_regression_rerun",
    "stage_002_focused_fmt",
    "stage_003_all_targets_check",
    "stage_003_clippy_with_baseline_allows",
    "stage_003_allocation_and_locator_probe",
    "stage_003_preserved_fixture_hashes",
}
EXPECTED_FAILURE = {
    "stage_002_touched_legacy_fmt_audit",
    "stage_002_touched_legacy_fmt_reaudit",
}
INVALID_ATTEMPT = {"stage_002_v7_checksum_regression"}


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        args, cwd=ROOT, check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def row(relative: Path) -> dict[str, object]:
    path = ROOT / relative
    return {
        "path": relative.as_posix(),
        "bytes": path.stat().st_size,
        "sha256": digest(path),
    }


def collect(scope: list[str]) -> list[Path]:
    result: set[Path] = set()
    for raw in scope:
        path = ROOT / raw
        if path.is_file():
            result.add(Path(raw))
        elif path.is_dir():
            result.update(
                child.relative_to(ROOT) for child in path.rglob("*") if child.is_file()
            )
        else:
            raise SystemExit(f"scope path disappeared: {raw}")
    return sorted(result, key=lambda value: value.as_posix())


def write_archive(destination: Path, paths: list[Path]) -> dict[str, object]:
    with tarfile.open(destination, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in paths:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    with tarfile.open(destination, "r:gz") as archive:
        members = [member for member in archive.getmembers() if member.isfile()]
        if [member.name for member in members] != [path.as_posix() for path in paths]:
            raise SystemExit(f"archive member set/order mismatch: {destination.name}")
        for relative, member in zip(paths, members):
            payload = archive.extractfile(member)
            data = payload.read() if payload is not None else b""
            if data != (ROOT / relative).read_bytes():
                raise SystemExit(f"archive byte mismatch: {relative}")
    result = row(destination.relative_to(ROOT))
    result["file_count"] = len(paths)
    return result


def write_delta(before_archive: Path, changes: list[dict[str, object]]) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="r06-wp01-target-") as raw:
        temp = Path(raw)
        extracted = temp / "extracted"
        before = temp / "before"
        after = temp / "after"
        extracted.mkdir()
        before.mkdir()
        after.mkdir()
        with tarfile.open(before_archive, "r:gz") as archive:
            archive.extractall(extracted, filter="data")
        for change in changes:
            relative = Path(str(change["path"]))
            old = extracted / relative
            current = ROOT / relative
            if old.is_file():
                target = before / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(old, target)
            if current.is_file():
                target = after / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(current, target)
        completed = run(
            "git",
            "diff",
            "--no-index",
            "--binary",
            "--src-prefix=pre_implementation/",
            "--dst-prefix=target_001/",
            str(before),
            str(after),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise SystemExit(completed.stderr.decode(errors="replace"))
        DELTA.write_bytes(completed.stdout)
    return row(DELTA.relative_to(ROOT))


def receipt_role(label: str) -> str:
    if label in REQUIRED_SUCCESS:
        return "required_direct_success"
    if label in EXPECTED_FAILURE:
        return "expected_recorded_failure"
    if label in INVALID_ATTEMPT:
        return "invalid_zero_test_attempt_not_counted"
    return "supporting_direct_success"


def main() -> None:
    if OUTPUT.exists():
        raise SystemExit("target 001 output already exists")
    OUTPUT.mkdir(parents=True)

    baseline_path = BASELINE / "MANIFEST.json"
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    baseline_detached = (BASELINE / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if digest(baseline_path) != baseline_detached:
        raise SystemExit("baseline detached hash mismatch")

    predecessor_path = ROOT / baseline["predecessor"]["manifest"]
    if digest(predecessor_path) != baseline["predecessor"]["manifest_sha256"]:
        raise SystemExit("predecessor manifest drift")
    checkpoint_path = CHECKPOINT / "MANIFEST.json"
    checkpoint_detached = (CHECKPOINT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if digest(checkpoint_path) != checkpoint_detached:
        raise SystemExit("checkpoint manifest drift")

    paths = collect(list(baseline["source"]["scope"]))
    current_rows = {entry["path"]: entry for entry in map(row, paths)}
    before_rows = {entry["path"]: entry for entry in baseline["source"]["files"]}
    changes = []
    for path in sorted(set(before_rows) | set(current_rows)):
        before = before_rows.get(path)
        after = current_rows.get(path)
        if before == after:
            continue
        changes.append(
            {
                "path": path,
                "state": "added" if before is None else "deleted" if after is None else "modified",
                "before_sha256": None if before is None else before["sha256"],
                "before_bytes": None if before is None else before["bytes"],
                "after_sha256": None if after is None else after["sha256"],
                "after_bytes": None if after is None else after["bytes"],
            }
        )

    source_archive = write_archive(SOURCE_ARCHIVE, paths)
    delta = write_delta(ROOT / baseline["source"]["archive"]["path"], changes)

    validation_dir = ROOT / SEGMENT / "evidence/validation"
    receipts = []
    labels = set()
    for receipt_path in sorted(validation_dir.glob("*.json")):
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        label = receipt["label"]
        if label in labels:
            raise SystemExit(f"duplicate receipt label: {label}")
        labels.add(label)
        role = receipt_role(label)
        exit_code = receipt["exit_code"]
        if role.endswith("success") and exit_code != 0:
            raise SystemExit(f"nonzero success receipt: {label}")
        if role == "expected_recorded_failure" and exit_code == 0:
            raise SystemExit(f"expected failure unexpectedly succeeded: {label}")
        if role == "invalid_zero_test_attempt_not_counted" and exit_code != 0:
            raise SystemExit(f"invalid attempt changed exit semantics: {label}")
        receipts.append(
            {
                "label": label,
                "role": role,
                "exit_code": exit_code,
                "command": receipt["command"],
                "receipt": row(receipt_path.relative_to(ROOT)),
                "output": receipt["output"],
            }
        )
    missing = REQUIRED_SUCCESS - labels
    if missing:
        raise SystemExit(f"missing required success receipts: {sorted(missing)}")
    if not EXPECTED_FAILURE <= labels or not INVALID_ATTEMPT <= labels:
        raise SystemExit("missing failure/invalid-attempt history")

    for fixture in baseline["preserved_preimplementation_fixtures"]:
        path = ROOT / fixture["path"]
        if path.stat().st_size != fixture["bytes"] or digest(path) != fixture["sha256"]:
            raise SystemExit(f"preserved fixture drift: {fixture['path']}")

    self_check = {
        "record": "R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001_FREEZE_CHECK",
        "result": "verified_for_freeze_only_not_review_pass",
        "source_archive_payloads_verified": True,
        "recoverable_delta_written": True,
        "baseline_manifest_sha256": digest(baseline_path),
        "predecessor_manifest_sha256": digest(predecessor_path),
        "checkpoint_manifest_sha256": digest(checkpoint_path),
        "source_file_count": len(current_rows),
        "change_count": len(changes),
        "validation_receipt_count": len(receipts),
        "required_direct_success": sorted(REQUIRED_SUCCESS),
        "expected_recorded_failure": sorted(EXPECTED_FAILURE),
        "invalid_attempt_not_counted": sorted(INVALID_ATTEMPT),
        "preserved_fixture_count": len(baseline["preserved_preimplementation_fixtures"]),
    }
    SELF_CHECK.write_text(
        json.dumps(self_check, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    record_rows = [row(path) for path in RECORDS]
    record_archive = write_archive(RECORD_ARCHIVE, RECORDS)
    evidence_paths = sorted(
        [path.relative_to(ROOT) for path in validation_dir.iterdir() if path.is_file()]
        + [SELF_CHECK.relative_to(ROOT)],
        key=lambda value: value.as_posix(),
    )
    evidence_rows = [row(path) for path in evidence_paths]
    evidence_archive = write_archive(EVIDENCE_ARCHIVE, evidence_paths)

    scoped_status = run(
        "git",
        "status",
        "--porcelain=v1",
        "--untracked-files=all",
        "--",
        *baseline["source"]["scope"],
    ).stdout
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    scoped_path = OUTPUT / "SCOPED_GIT_STATUS.txt"
    global_path = OUTPUT / "GLOBAL_GIT_STATUS.txt"
    scoped_path.write_bytes(scoped_status)
    global_path.write_bytes(global_status)

    manifest = {
        "record": "R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001",
        "state": "partial_implementation_validated_stage_review_request_pending",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").stdout.decode().strip(),
        "head": run("git", "rev-parse", "HEAD").stdout.decode().strip(),
        "goal": baseline["goal"],
        "handoff": baseline["handoff"],
        "pipeline_revision": baseline["pipeline_revision"],
        "predecessor": baseline["predecessor"],
        "baseline": {
            "record": baseline["record"],
            "manifest": baseline_path.relative_to(ROOT).as_posix(),
            "manifest_sha256": digest(baseline_path),
            "archive": baseline["source"]["archive"],
        },
        "stage_checkpoint": {
            "record": "R06_WP01_CONTENT_INPUT_CHECKPOINT_001",
            "manifest": checkpoint_path.relative_to(ROOT).as_posix(),
            "manifest_sha256": digest(checkpoint_path),
            "self_review": (SEGMENT / "reviews/checkpoint_001_self_review/REVIEW.md").as_posix(),
            "correction_review": (SEGMENT / "reviews/checkpoint_001_self_review/CORRECTION_REVIEW.md").as_posix(),
        },
        "completion_claims": {
            "r06_complete": False,
            "wp01_complete": False,
            "wp02_runtime_connected": False,
            "phase_complete": False,
            "goal_complete": False,
            "gex_20_or_30_started": False,
            "review_pass_declared": False,
        },
        "implemented_boundary": "approved authored RON -> bounded parse -> validation/common compile -> immutable catalog -> explicit production publication block",
        "current_source": {
            "scope": baseline["source"]["scope"],
            "file_count": len(current_rows),
            "files": list(current_rows.values()),
            "archive": source_archive,
            "changes": changes,
            "change_counts": {
                "added": sum(change["state"] == "added" for change in changes),
                "modified": sum(change["state"] == "modified" for change in changes),
                "deleted": sum(change["state"] == "deleted" for change in changes),
            },
            "recoverable_delta": delta,
        },
        "records": record_rows,
        "records_archive": record_archive,
        "evidence": evidence_rows,
        "evidence_archive": evidence_archive,
        "validation_receipts": receipts,
        "preserved_fixtures": baseline["preserved_preimplementation_fixtures"],
        "status": {
            "scoped": row(scoped_path.relative_to(ROOT)),
            "global": row(global_path.relative_to(ROOT)),
            "note": "Captured before manifest/hash creation; unrelated dirty work remains user-owned.",
        },
        "review_entry": (SEGMENT / "REVIEW_HANDOFF.md").as_posix(),
        "known_boundary": "Authored expressions have no WP02 runtime consumer and cannot be production-published.",
        "mutation_boundary": "All target source, records, evidence and archives are immutable after this freeze; only external re-review may follow.",
    }
    manifest_path = OUTPUT / "MANIFEST.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    (OUTPUT / "MANIFEST.sha256").write_text(
        f"{digest(manifest_path)}  MANIFEST.json\n", encoding="utf-8"
    )
    print(digest(manifest_path))


if __name__ == "__main__":
    main()
