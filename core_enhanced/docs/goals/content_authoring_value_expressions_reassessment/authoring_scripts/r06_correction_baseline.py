#!/usr/bin/env python3
"""Freeze the pre-fix baseline for checkpoint 001 F01-F04 correction."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
EXECUTION = Path(
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal"
)
PARENT = ROOT / EXECUTION / "checkpoints/checkpoint_001"
REVIEW = ROOT / EXECUTION / "reviews/checkpoint_001_review_001"
CORRECTION = EXECUTION / "corrections/checkpoint_001_f01_f04"
OUTPUT = ROOT / CORRECTION / "baseline/pre_fix_001"
ARCHIVE = OUTPUT / "SOURCE_BEFORE_FIX_001.tar.gz"


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def run(*args: str) -> bytes:
    return subprocess.run(
        args,
        cwd=ROOT,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ).stdout


def row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    return {
        "path": relative.as_posix(),
        "sha256": digest(absolute),
        "bytes": absolute.stat().st_size,
    }


def main() -> None:
    parent_manifest_path = PARENT / "MANIFEST.json"
    parent = json.loads(parent_manifest_path.read_text(encoding="utf-8"))
    parent_hash = digest(parent_manifest_path)
    detached = (PARENT / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if parent_hash != detached:
        raise SystemExit("parent target detached hash mismatch")

    current_rows = []
    mismatches = []
    for expected in parent["current_source"]["files"]:
        relative = Path(expected["path"])
        actual = row(relative)
        current_rows.append(actual)
        if actual["sha256"] != expected["sha256"] or actual["bytes"] != expected["bytes"]:
            mismatches.append({"expected": expected, "actual": actual})
    if mismatches:
        raise SystemExit(f"source differs from frozen parent target: {len(mismatches)} files")

    OUTPUT.mkdir(parents=True, exist_ok=True)
    with tarfile.open(ARCHIVE, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for source_row in current_rows:
            relative = Path(source_row["path"])
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)

    scope = list(parent["current_source"]["scope"])
    scoped_status = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--", *scope
    )
    scoped_path = OUTPUT / "SCOPED_GIT_STATUS_BEFORE_FIX.txt"
    scoped_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all")
    global_path = OUTPUT / "GLOBAL_GIT_STATUS_BEFORE_FIX.txt"
    global_path.write_bytes(global_status)

    review_paths = sorted(path for path in REVIEW.rglob("*") if path.is_file())
    review_rows = [row(path.relative_to(ROOT)) for path in review_paths]
    plan = CORRECTION / "PLAN.md"
    manifest = {
        "record": "R06_CHECKPOINT_001_CORRECTION_BASELINE_001",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").decode().strip(),
        "head": run("git", "rev-parse", "HEAD").decode().strip(),
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "handoff": "HA-CAVER-P2G60-001",
        "pipeline_revision": "2026-09-13-gex-r7",
        "correction_scope": ["F01", "F02", "F03", "F04", "same-cause kernel paths"],
        "parent_target": {
            "record": parent["record"],
            "manifest": parent_manifest_path.relative_to(ROOT).as_posix(),
            "manifest_sha256": parent_hash,
            "detached_hash_matches": True,
            "source_file_count": parent["current_source"]["file_count"],
            "current_source_matches_all_files": True,
        },
        "source": {
            "file_count": len(current_rows),
            "files": current_rows,
            "archive": {
                "path": ARCHIVE.relative_to(ROOT).as_posix(),
                "sha256": digest(ARCHIVE),
                "bytes": ARCHIVE.stat().st_size,
            },
            "scoped_status": {
                "path": scoped_path.relative_to(ROOT).as_posix(),
                "sha256": digest(scoped_path),
                "rows": len(scoped_status.splitlines()),
            },
            "global_status": {
                "path": global_path.relative_to(ROOT).as_posix(),
                "sha256": digest(global_path),
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before manifest/hash pair to avoid self-reference.",
            },
        },
        "review_evidence": review_rows,
        "correction_plan": row(plan),
        "recovery": {
            "actual_bytes_preserved": True,
            "safe_method": "Extract into an isolated directory; do not overwrite the dirty worktree blindly.",
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
