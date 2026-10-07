#!/usr/bin/env python3
"""Freeze correction-002 source and mutable-document bytes before implementation."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
PREDECESSOR = ROOT / EXECUTION / "corrections/checkpoint_001_f01_f04/target/correction_001"
PREDECESSOR_BASELINE = ROOT / EXECUTION / "corrections/checkpoint_001_f01_f04/baseline/pre_fix_001"
REVIEW = ROOT / EXECUTION / "reviews/correction_001_review_001"
CORRECTION = EXECUTION / "corrections/correction_001_r01_r03"
OUTPUT = ROOT / CORRECTION / "baseline/pre_fix_002"

MUTABLE_DOCUMENTS = [
    CORRECTION / "PLAN.md",
    CORRECTION / "CORRECTION_NOTES.md",
    CORRECTION / "EXPERIMENTS.md",
]


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def run(*args: str) -> bytes:
    return subprocess.run(
        args, cwd=ROOT, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    ).stdout


def row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    if not absolute.is_file():
        raise SystemExit(f"missing required file: {relative.as_posix()}")
    return {
        "path": relative.as_posix(),
        "sha256": digest(absolute),
        "bytes": absolute.stat().st_size,
    }


def validate_detached(directory: Path) -> tuple[dict[str, object], str]:
    manifest_path = directory / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (directory / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit(f"detached hash mismatch: {manifest_path.relative_to(ROOT)}")
    return json.loads(manifest_path.read_text(encoding="utf-8")), manifest_hash


def write_archive(path: Path, paths: list[Path]) -> dict[str, object]:
    with tarfile.open(path, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in paths:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "sha256": digest(path),
        "bytes": path.stat().st_size,
        "file_count": len(paths),
    }


def main() -> None:
    if (OUTPUT / "MANIFEST.json").exists():
        raise SystemExit("pre-fix correction-002 baseline already exists")
    predecessor, predecessor_hash = validate_detached(PREDECESSOR)
    old_baseline, old_baseline_hash = validate_detached(PREDECESSOR_BASELINE)
    if predecessor_hash != "87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e":
        raise SystemExit("unexpected predecessor target hash")

    source_rows = []
    for expected in predecessor["current_source"]["files"]:
        current = row(Path(expected["path"]))
        if current != expected:
            raise SystemExit(f"new source drift: {expected['path']}")
        source_rows.append(current)

    record_rows = []
    for expected in predecessor["execution_records"]:
        current = row(Path(expected["path"]))
        if current != expected:
            raise SystemExit(f"new predecessor record drift: {expected['path']}")
        record_rows.append(current)
    evidence_rows = []
    for expected in predecessor["evidence"]:
        current = row(Path(expected["path"]))
        if current != expected:
            raise SystemExit(f"new predecessor evidence drift: {expected['path']}")
        evidence_rows.append(current)

    old_plan_reference = old_baseline["correction_plan"]
    live_old_plan = row(Path(old_plan_reference["path"]))
    known_historical_mismatch = {
        "path": old_plan_reference["path"],
        "historical_expected_sha256": old_plan_reference["sha256"],
        "historical_expected_bytes": old_plan_reference["bytes"],
        "current_sha256": live_old_plan["sha256"],
        "current_bytes": live_old_plan["bytes"],
        "classification": "known R03 historical snapshot gap; current live record matches predecessor target",
    }
    if (
        old_plan_reference["sha256"]
        != "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7"
        or old_plan_reference["bytes"] != 4460
    ):
        raise SystemExit("unexpected historical PLAN reference")

    mutable_rows = [row(path) for path in MUTABLE_DOCUMENTS]
    review_paths = sorted(
        (path.relative_to(ROOT) for path in REVIEW.rglob("*") if path.is_file()),
        key=lambda path: path.as_posix(),
    )
    review_rows = [row(path) for path in review_paths]

    OUTPUT.mkdir(parents=True, exist_ok=False)
    source_archive = write_archive(
        OUTPUT / "SOURCE_BEFORE_CORRECTION_002.tar.gz",
        [Path(entry["path"]) for entry in source_rows],
    )
    document_archive = write_archive(
        OUTPUT / "MUTABLE_DOCUMENTS_BEFORE_CORRECTION_002.tar.gz", MUTABLE_DOCUMENTS
    )
    predecessor_records_archive = write_archive(
        OUTPUT / "PREDECESSOR_RECORDS_AT_ENTRY.tar.gz",
        [Path(entry["path"]) for entry in record_rows],
    )
    review_archive = write_archive(
        OUTPUT / "REVIEW_INPUTS_AT_ENTRY.tar.gz", review_paths
    )

    scoped_status = run(
        "git",
        "status",
        "--porcelain=v1",
        "--untracked-files=all",
        "--",
        *predecessor["current_source"]["scope"],
    )
    scoped_status_path = OUTPUT / "SCOPED_GIT_STATUS_BEFORE_CORRECTION_002.txt"
    scoped_status_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all")
    global_status_path = OUTPUT / "GLOBAL_GIT_STATUS_BEFORE_CORRECTION_002.txt"
    global_status_path.write_bytes(global_status)

    manifest = {
        "record": "R06_CHECKPOINT_001_CORRECTION_BASELINE_002",
        "state": "pre_implementation_frozen",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").decode().strip(),
        "head": run("git", "rev-parse", "HEAD").decode().strip(),
        "handoff": "HA-CAVER-P2G60-001",
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "pipeline_revision": "2026-09-13-gex-r7",
        "correction_scope": ["R01", "R02", "R03", "same-cause preservation tools"],
        "predecessor_target": {
            "record": predecessor["record"],
            "manifest": (PREDECESSOR / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": predecessor_hash,
            "detached_hash_matches": True,
            "current_source_records_evidence_match": True,
        },
        "predecessor_baseline": {
            "record": old_baseline["record"],
            "manifest": (PREDECESSOR_BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": old_baseline_hash,
            "detached_hash_matches": True,
        },
        "entry_currentness": {
            "new_drift": [],
            "known_historical_mismatch": known_historical_mismatch,
        },
        "source": {
            "scope": predecessor["current_source"]["scope"],
            "file_count": len(source_rows),
            "files": source_rows,
            "archive": source_archive,
        },
        "mutable_documents": {
            "file_count": len(mutable_rows),
            "files": mutable_rows,
            "archive": document_archive,
        },
        "predecessor_execution_records": {
            "file_count": len(record_rows),
            "files": record_rows,
            "archive": predecessor_records_archive,
        },
        "predecessor_evidence": {
            "file_count": len(evidence_rows),
            "files": evidence_rows,
            "note": "Current bytes verified against predecessor target; original evidence remains in place.",
        },
        "review_inputs": {
            "file_count": len(review_rows),
            "files": review_rows,
            "archive": review_archive,
        },
        "status": {
            "scoped": {
                "path": scoped_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(scoped_status_path),
                "bytes": scoped_status_path.stat().st_size,
                "rows": len(scoped_status.splitlines()),
            },
            "global": {
                "path": global_status_path.relative_to(ROOT).as_posix(),
                "sha256": digest(global_status_path),
                "bytes": global_status_path.stat().st_size,
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before manifest/hash pair to avoid self-reference.",
            },
        },
        "recovery": {
            "actual_source_bytes_preserved": True,
            "actual_mutable_document_bytes_preserved": True,
            "actual_predecessor_record_bytes_preserved": True,
            "actual_review_input_bytes_preserved": True,
            "historical_r03_plan_bytes_preserved": False,
            "safe_method": "Extract archives into isolated directories; do not overwrite the dirty worktree blindly.",
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
