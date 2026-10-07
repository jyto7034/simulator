#!/usr/bin/env python3
"""Freeze the first WP01 content-input staged-review checkpoint."""

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
OUTPUT = ROOT / SEGMENT / "checkpoints/checkpoint_001"
ARCHIVE = OUTPUT / "SOURCE_AT_CHECKPOINT_001.tar.gz"
DELTA = OUTPUT / "SOURCE_DELTA.patch"

RECORDS = [
    SEGMENT / "PLAN.md",
    SEGMENT / "EXPERIMENTS.md",
    SEGMENT / "EXPERIMENT_NOTES.md",
    SEGMENT / "checkpoints/checkpoint_001/CHECKPOINT_HANDOFF.md",
    TOPIC / "authoring_scripts/r06_wp01_run_validation.py",
    TOPIC / "authoring_scripts/r06_wp01_checkpoint_001.py",
    TOPIC / "authoring_scripts/r06_wp01_verify_checkpoint_001.py",
]


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        args, cwd=ROOT, check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )


def byte_digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


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


def write_archive(paths: list[Path]) -> dict[str, object]:
    with tarfile.open(ARCHIVE, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in paths:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    with tarfile.open(ARCHIVE, "r:gz") as archive:
        members = [member for member in archive.getmembers() if member.isfile()]
        if [member.name for member in members] != [path.as_posix() for path in paths]:
            raise SystemExit("checkpoint archive member set/order mismatch")
        for relative, member in zip(paths, members):
            payload = archive.extractfile(member)
            data = payload.read() if payload is not None else b""
            current = (ROOT / relative).read_bytes()
            if data != current:
                raise SystemExit(f"checkpoint archive byte mismatch: {relative}")
    return {
        "path": ARCHIVE.relative_to(ROOT).as_posix(),
        "bytes": ARCHIVE.stat().st_size,
        "sha256": digest(ARCHIVE),
        "file_count": len(paths),
    }


def write_delta(before_archive: Path, changes: list[dict[str, object]]) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="r06-wp01-stage-") as raw:
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
            "--dst-prefix=checkpoint_001/",
            str(before),
            str(after),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise SystemExit(completed.stderr.decode(errors="replace"))
        DELTA.write_bytes(completed.stdout)
    return row(DELTA.relative_to(ROOT))


def main() -> None:
    if (OUTPUT / "MANIFEST.json").exists() or ARCHIVE.exists():
        raise SystemExit("checkpoint 001 is already frozen")
    baseline_path = BASELINE / "MANIFEST.json"
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    detached = (BASELINE / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if digest(baseline_path) != detached:
        raise SystemExit("baseline detached hash mismatch")

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

    source_archive = write_archive(paths)
    delta = write_delta(ROOT / baseline["source"]["archive"]["path"], changes)
    validation_dir = ROOT / SEGMENT / "evidence/validation"
    evidence = [
        row(path.relative_to(ROOT))
        for path in sorted(validation_dir.glob("stage_001_*"))
        if path.is_file()
    ]
    receipts = []
    for receipt_path in sorted(validation_dir.glob("stage_001_*.json")):
        receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
        receipts.append(
            {
                "label": receipt["label"],
                "exit_code": receipt["exit_code"],
                "command": receipt["command"],
                "receipt": row(receipt_path.relative_to(ROOT)),
                "output": receipt["output"],
            }
        )
    if {receipt["label"] for receipt in receipts} != {
        "stage_001_value_expr_unit",
        "stage_001_all_targets_check",
        "stage_001_focused_fmt",
    } or any(receipt["exit_code"] != 0 for receipt in receipts):
        raise SystemExit("checkpoint validation receipt set/result mismatch")

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
        "record": "R06_WP01_CONTENT_INPUT_CHECKPOINT_001",
        "state": "partial_checkpoint_frozen_stage_review_pending",
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
        "checkpoint_reason": "Actual skill RON bounded parse through immutable compile catalog is buildable and testable before external-pack regression expansion.",
        "completion_claims": {
            "r06_complete": False,
            "wp01_complete": False,
            "phase_complete": False,
            "goal_complete": False,
            "gex_20_or_30_started": False,
            "wp02_runtime_connected": False,
        },
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
        "records": [row(path) for path in RECORDS],
        "evidence": evidence,
        "validation_receipts": receipts,
        "status": {
            "scoped": row(scoped_path.relative_to(ROOT)),
            "global": row(global_path.relative_to(ROOT)),
            "note": "Status captured before manifest/hash creation; unrelated dirty work remains attributed to the user.",
        },
        "preserved_fixtures": baseline["preserved_preimplementation_fixtures"],
        "review_entry": (SEGMENT / "checkpoints/checkpoint_001/CHECKPOINT_HANDOFF.md").as_posix(),
        "mutation_boundary": "Checkpoint archive, manifest, records and stage_001 evidence are immutable after freeze; later WP01 source changes require a successor target.",
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
