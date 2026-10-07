#!/usr/bin/env python3
"""Capture the actual-byte baseline for the WP02 sequential-effect segment."""

from __future__ import annotations

from datetime import datetime, timezone
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
SEGMENT = EXECUTION / "segments/wp02_effect_execution_prefix_001"
OUTPUT = ROOT / SEGMENT / "baseline/pre_implementation_001"
PREDECESSOR = ROOT / EXECUTION / (
    "corrections/wp02_cap_01_02_001/target/cap_01_02_correction_002"
)
REVIEW = ROOT / EXECUTION / (
    "reviews/wp02_cap_01_02_correction_002_review_001/REVIEW.md"
)
EXPECTED_PREDECESSOR = "6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789"


def identity(payload: bytes) -> dict[str, object]:
    return {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}


def row(relative: str) -> dict[str, object]:
    payload = (ROOT / relative).read_bytes()
    return {"path": relative, **identity(payload)}


def run(*args: str) -> bytes:
    return subprocess.run(
        args, cwd=ROOT, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    ).stdout


def write_json(path: Path, value: object) -> dict[str, object]:
    payload = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
    path.write_bytes(payload)
    return identity(payload)


def write_archive(path: Path, entries: dict[str, bytes]) -> dict[str, object]:
    with path.open("xb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", filename="", mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode="w") as archive:
                for name, payload in sorted(entries.items()):
                    member = tarfile.TarInfo(name)
                    member.size = len(payload)
                    member.mode = 0o644
                    member.mtime = 0
                    archive.addfile(member, io.BytesIO(payload))
    return identity(path.read_bytes())


def main() -> None:
    assert not OUTPUT.exists(), OUTPUT
    predecessor_path = PREDECESSOR / "MANIFEST.json"
    predecessor_payload = predecessor_path.read_bytes()
    assert hashlib.sha256(predecessor_payload).hexdigest() == EXPECTED_PREDECESSOR
    predecessor = json.loads(predecessor_payload)

    source_rows = predecessor["source"]["files"]
    source = {item["path"]: (ROOT / item["path"]).read_bytes() for item in source_rows}
    mismatches = [
        path
        for path, payload in source.items()
        if identity(payload)
        != {
            "bytes": next(item["bytes"] for item in source_rows if item["path"] == path),
            "sha256": next(item["sha256"] for item in source_rows if item["path"] == path),
        }
    ]
    assert not mismatches, mismatches

    basis_paths = [
        TOPIC / "CODE_REVIEW_003.ko.md",
        TOPIC / "P2G_40_PHASE_PLAN_001.json",
        EXECUTION / "segments/wp02_runtime_frame_capture_001/PLAN.md",
        predecessor_path.relative_to(ROOT),
        REVIEW.relative_to(ROOT),
        SEGMENT / "PLAN.md",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_baseline_001.py",
    ]
    basis = {path.as_posix(): (ROOT / path).read_bytes() for path in basis_paths}

    OUTPUT.mkdir(parents=True)
    source_archive = write_archive(OUTPUT / "SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz", source)
    basis_archive = write_archive(OUTPUT / "BASIS_DOCUMENTS_AT_ENTRY.tar.gz", basis)
    scoped = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--",
        "core_enhanced/src/game/behavior.rs",
        "core_enhanced/src/game/battle/damage.rs",
        "core_enhanced/src/game/battle/runtime_facts.rs",
        "core_enhanced/src/game/battle/core",
    )
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all")
    (OUTPUT / "SCOPED_GIT_STATUS_BEFORE.txt").write_bytes(scoped)
    (OUTPUT / "GLOBAL_GIT_STATUS_BEFORE.txt").write_bytes(global_status)

    manifest = {
        "record": "R06_WP02_EFFECT_EXECUTION_PREFIX_BASELINE_001",
        "state": "pre_implementation_actual_bytes_preserved",
        "captured_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "predecessor": {"path": predecessor_path.relative_to(ROOT).as_posix(), **identity(predecessor_payload)},
        "review": row(REVIEW.relative_to(ROOT).as_posix()),
        "currentness": {
            "approved_source_files": len(source),
            "current_bytes_match_approved_target": True,
            "unexpected_source_drift": [],
        },
        "source": {
            "file_count": len(source),
            "files": [{"path": path, **identity(payload)} for path, payload in sorted(source.items())],
            "archive": {"path": (OUTPUT / "SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz").relative_to(ROOT).as_posix(), **source_archive},
        },
        "basis": {
            "file_count": len(basis),
            "files": [{"path": path, **identity(payload)} for path, payload in sorted(basis.items())],
            "archive": {"path": (OUTPUT / "BASIS_DOCUMENTS_AT_ENTRY.tar.gz").relative_to(ROOT).as_posix(), **basis_archive},
        },
        "status": {
            "scoped": {"path": (OUTPUT / "SCOPED_GIT_STATUS_BEFORE.txt").relative_to(ROOT).as_posix(), **identity(scoped)},
            "global": {"path": (OUTPUT / "GLOBAL_GIT_STATUS_BEFORE.txt").relative_to(ROOT).as_posix(), **identity(global_status)},
        },
        "claims": {
            "implementation_started": False,
            "wp_phase_goal_complete": False,
            "review_pass_declared": False,
        },
    }
    manifest_identity = write_json(OUTPUT / "MANIFEST.json", manifest)
    (OUTPUT / "MANIFEST.sha256").write_text(
        manifest_identity["sha256"] + "  MANIFEST.json\n", encoding="utf-8"
    )
    print(json.dumps({
        "manifest_sha256": manifest_identity["sha256"],
        "source_files": len(source),
        "current_bytes_match_approved_target": True,
    }, indent=2))


if __name__ == "__main__":
    main()
