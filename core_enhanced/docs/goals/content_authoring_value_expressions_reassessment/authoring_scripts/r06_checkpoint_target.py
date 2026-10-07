#!/usr/bin/env python3
"""Freeze the first R06 staged-review target and its recoverable source delta."""

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
INITIAL = ROOT / EXECUTION / "baseline/initial_001"
OUTPUT = ROOT / EXECUTION / "checkpoints/checkpoint_001"
ARCHIVE = OUTPUT / "SOURCE_AFTER_001.tar.gz"
DELTA = OUTPUT / "CHECKPOINT_DELTA.patch"

RECORDS = [
    EXECUTION / "PLAN.md",
    EXECUTION / "EXPERIMENTS.md",
    EXECUTION / "EXPERIMENT_NOTES.md",
    EXECUTION / "CHECKPOINT_REVIEW_HANDOFF.md",
    TOPIC / "authoring_scripts/r06_execution_baseline.py",
    TOPIC / "authoring_scripts/r06_run_validation.py",
    TOPIC / "authoring_scripts/r06_checkpoint_target.py",
]


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


def collect_scope(scope: list[str]) -> list[Path]:
    files: set[Path] = set()
    for raw in scope:
        relative = Path(raw)
        absolute = ROOT / relative
        if absolute.is_file() or absolute.is_symlink():
            files.add(relative)
        elif absolute.is_dir():
            for child in absolute.rglob("*"):
                if child.is_file() or child.is_symlink():
                    files.add(child.relative_to(ROOT))
    return sorted(files, key=lambda path: path.as_posix())


def file_row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    return {
        "path": relative.as_posix(),
        "sha256": digest(absolute),
        "bytes": absolute.stat().st_size,
    }


def record_rows(paths: list[Path]) -> list[dict[str, object]]:
    return [file_row(path) for path in paths]


def evidence_rows() -> list[dict[str, object]]:
    evidence = ROOT / EXECUTION / "evidence"
    paths = sorted(
        (path.relative_to(ROOT) for path in evidence.rglob("*") if path.is_file()),
        key=lambda path: path.as_posix(),
    )
    return record_rows(paths)


def write_delta(initial_archive: Path, changes: list[dict[str, object]]) -> bytes:
    with tempfile.TemporaryDirectory(prefix="r06-checkpoint-") as raw_temp:
        temp = Path(raw_temp)
        before = temp / "initial"
        after = temp / "checkpoint"
        before.mkdir()
        after.mkdir()
        with tarfile.open(initial_archive, "r:gz") as archive:
            archive.extractall(before, filter="data")
        for change in changes:
            relative = Path(str(change["path"]))
            current = ROOT / relative
            if current.is_file() or current.is_symlink():
                destination = after / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(current, destination, follow_symlinks=False)
        for change in changes:
            relative = Path(str(change["path"]))
            initial = before / relative
            if initial.exists() and not (after / relative).exists():
                continue
            if initial.exists():
                isolated = temp / "initial_changed" / relative
                isolated.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(initial, isolated, follow_symlinks=False)
        before_changed = temp / "initial_changed"
        before_changed.mkdir(exist_ok=True)
        completed = run(
            "git",
            "diff",
            "--no-index",
            "--binary",
            "--src-prefix=initial/",
            "--dst-prefix=checkpoint/",
            str(before_changed),
            str(after),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise RuntimeError(completed.stderr.decode(errors="replace"))
        return completed.stdout


def main() -> None:
    initial_manifest_path = INITIAL / "MANIFEST.json"
    initial_manifest = json.loads(initial_manifest_path.read_text(encoding="utf-8"))
    scope = list(initial_manifest["scope"])
    current_paths = collect_scope(scope)
    current_rows = {row["path"]: row for row in map(file_row, current_paths)}
    initial_rows = {row["path"]: row for row in initial_manifest["files"]}

    changes: list[dict[str, object]] = []
    for path in sorted(set(initial_rows) | set(current_rows)):
        before = initial_rows.get(path)
        after = current_rows.get(path)
        if before == after:
            continue
        state = "added" if before is None else "deleted" if after is None else "modified"
        changes.append({
            "path": path,
            "state": state,
            "before_sha256": None if before is None else before["sha256"],
            "before_bytes": None if before is None else before["bytes"],
            "after_sha256": None if after is None else after["sha256"],
            "after_bytes": None if after is None else after["bytes"],
        })

    OUTPUT.mkdir(parents=True, exist_ok=True)
    with tarfile.open(ARCHIVE, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in current_paths:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)

    delta = write_delta(ROOT / initial_manifest["archive"]["path"], changes)
    DELTA.write_bytes(delta)

    scope_args = scope
    scoped_status = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--", *scope_args
    ).stdout
    scoped_status_path = OUTPUT / "SCOPED_GIT_STATUS_AFTER.txt"
    scoped_status_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    global_status_path = OUTPUT / "GLOBAL_GIT_STATUS_AFTER.txt"
    global_status_path.write_bytes(global_status)

    validations = []
    validation_root = ROOT / EXECUTION / "evidence/validation"
    for receipt_path in sorted(validation_root.glob("*.json")):
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        validations.append({
            "label": receipt["label"],
            "exit_code": receipt["exit_code"],
            "command": receipt["command"],
            "receipt": receipt_path.relative_to(ROOT).as_posix(),
            "receipt_sha256": digest(receipt_path),
            "output": receipt["output"],
        })

    initial_manifest_hash = digest(initial_manifest_path)
    branch = run("git", "branch", "--show-current").stdout.decode().strip()
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    manifest = {
        "record": "R06_STAGE_REVIEW_TARGET_001",
        "state": "partial_implementation_validated_review_pending",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": branch,
        "head": head,
        "handoff": "HA-CAVER-P2G60-001",
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "pipeline_revision": "2026-09-13-gex-r7",
        "checkpoint_reason": (
            "The in-memory typed expression compile/evaluate kernel is a reviewable, "
            "buildable responsibility boundary before authored catalog and runtime cutover."
        ),
        "completion_claims": {
            "goal_complete": False,
            "phase_complete": False,
            "work_package_complete": False,
            "gex_10_exit_claimed": False,
            "gex_20_started": False,
        },
        "initial_baseline": {
            "record": initial_manifest["record"],
            "manifest": initial_manifest_path.relative_to(ROOT).as_posix(),
            "manifest_sha256": initial_manifest_hash,
            "detached_hash_matches": (INITIAL / "MANIFEST.sha256")
            .read_text(encoding="utf-8")
            .split()[0]
            == initial_manifest_hash,
            "source_archive": initial_manifest["archive"],
        },
        "current_source": {
            "scope": scope,
            "file_count": len(current_rows),
            "files": list(current_rows.values()),
            "archive": {
                "path": ARCHIVE.relative_to(ROOT).as_posix(),
                "sha256": digest(ARCHIVE),
                "bytes": ARCHIVE.stat().st_size,
            },
            "changes_from_initial": {
                "added": sum(change["state"] == "added" for change in changes),
                "modified": sum(change["state"] == "modified" for change in changes),
                "deleted": sum(change["state"] == "deleted" for change in changes),
                "files": changes,
            },
            "recoverable_delta": {
                "path": DELTA.relative_to(ROOT).as_posix(),
                "sha256": digest(DELTA),
                "bytes": DELTA.stat().st_size,
                "note": "SOURCE_BEFORE_001 plus SOURCE_AFTER_001 are authoritative actual-byte recovery inputs; this patch is a readable binary-capable comparison.",
            },
            "scoped_status": {
                "path": scoped_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(scoped_status_path),
                "rows": len(scoped_status.splitlines()),
            },
            "global_status": {
                "path": global_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(global_status_path),
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before this manifest/hash pair was written to avoid self-reference.",
            },
        },
        "execution_records": record_rows(RECORDS),
        "evidence": evidence_rows(),
        "validation_receipts": validations,
        "review_entry": (EXECUTION / "CHECKPOINT_REVIEW_HANDOFF.md").as_posix(),
        "mutation_boundary": (
            "After successful generation and read-only verification, do not mutate the "
            "checkpoint source, execution records, or evidence before staged review."
        ),
        "recovery": {
            "safe_method": "Extract both archives into isolated directories and inspect CHECKPOINT_DELTA.patch; do not overwrite the dirty worktree blindly.",
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
