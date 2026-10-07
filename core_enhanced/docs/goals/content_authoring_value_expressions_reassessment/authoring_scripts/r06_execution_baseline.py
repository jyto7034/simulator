#!/usr/bin/env python3
"""Freeze a recoverable pre-implementation baseline for the R06 goal run."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
OUTPUT = ROOT / TOPIC / "execution/single_goal/baseline/initial_001"
ARCHIVE = OUTPUT / "SOURCE_BEFORE_001.tar.gz"
CLOSE_MANIFEST = ROOT / TOPIC / "baseline/close_002/MANIFEST.json"

SCOPE = [
    Path("Cargo.lock"),
    Path("Cargo.toml"),
    Path("core_enhanced/Cargo.toml"),
    Path("core_enhanced/src"),
    Path("core_enhanced/tests"),
    Path("game_resources/data"),
    Path("game_server/src"),
    Path("game_server/docs/unity_bridge_protocol.md"),
    Path("game_server/config/development.toml"),
    Path("game_server/config/production.toml"),
    Path("core_enhanced/docs/core_runtime_contract.md"),
    Path("core_enhanced/docs/data_loading_contract.md"),
    Path("core_enhanced/docs/skill_target_contract.md"),
    Path("core_enhanced/docs/game_rulebook.md"),
    TOPIC / "PLAN_HANDOFF.ko.md",
    TOPIC / "CODE_REVIEW_003.ko.md",
    TOPIC / "P2G_50_GOAL_SET_001.ko.md",
    TOPIC / "P2G_60_HANDOFF_001.json",
    Path("core_enhanced/docs/goal_execution_pipeline(1).md"),
    Path("core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_CLOSURE_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/AUDIT_TARGET_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_minimal_effect_contract/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md"),
    Path("core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/AUDIT_CLOSURE_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/AUDIT_TARGET_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_runtime_facts/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md"),
    Path("core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/AUDIT_CLOSURE_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/AUDIT_TARGET_RESUME_001.json"),
    Path("core_enhanced/docs/goals/content_authoring_resource_lifetimes/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md"),
]


def run(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(args, cwd=ROOT, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def files_in_scope() -> list[Path]:
    files: set[Path] = set()
    for relative in SCOPE:
        absolute = ROOT / relative
        if absolute.is_file() or absolute.is_symlink():
            files.add(relative)
        elif absolute.is_dir():
            for child in absolute.rglob("*"):
                if child.is_file() or child.is_symlink():
                    files.add(child.relative_to(ROOT))
    return sorted(files, key=lambda path: path.as_posix())


def git_scope_args() -> list[str]:
    return [path.as_posix() for path in SCOPE]


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    files = files_in_scope()
    with tarfile.open(ARCHIVE, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in files:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)

    status = run("git", "status", "--porcelain=v1", "--untracked-files=all", "--", *git_scope_args()).stdout
    (OUTPUT / "SCOPED_GIT_STATUS.txt").write_bytes(status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    (OUTPUT / "GLOBAL_GIT_STATUS.txt").write_bytes(global_status)
    patch = run("git", "diff", "--binary", "--", *git_scope_args()).stdout
    (OUTPUT / "PREEXISTING_TRACKED_DIFF.patch").write_bytes(patch)

    close = json.loads(CLOSE_MANIFEST.read_text(encoding="utf-8"))
    close_rows = {row["path"]: row for row in close["files"]}
    comparison = []
    for path, row in sorted(close_rows.items()):
        absolute = ROOT / path
        if absolute.is_file():
            current = digest(absolute)
            state = "unchanged" if current == row["sha256"] else "changed"
        else:
            current = None
            state = "missing"
        comparison.append({
            "path": path,
            "planning_close_sha256": row["sha256"],
            "current_sha256": current,
            "state": state,
        })

    branch = run("git", "branch", "--show-current").stdout.decode().strip()
    head = run("git", "rev-parse", "HEAD").stdout.decode().strip()
    rustc = run("rustc", "--version").stdout.decode().strip()
    cargo = run("cargo", "--version").stdout.decode().strip()
    manifest = {
        "record": "R06_IMPLEMENTATION_BASELINE_INITIAL_001",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": branch,
        "head": head,
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "handoff": "HA-CAVER-P2G60-001",
        "pipeline_revision": "2026-09-13-gex-r7",
        "scope": [path.as_posix() for path in SCOPE],
        "file_count": len(files),
        "files": [
            {
                "path": relative.as_posix(),
                "sha256": digest(ROOT / relative),
                "bytes": (ROOT / relative).stat().st_size,
            }
            for relative in files
        ],
        "archive": {
            "path": ARCHIVE.relative_to(ROOT).as_posix(),
            "sha256": digest(ARCHIVE),
            "bytes": ARCHIVE.stat().st_size,
        },
        "scoped_status": {
            "path": (OUTPUT / "SCOPED_GIT_STATUS.txt").relative_to(ROOT).as_posix(),
            "sha256": digest(OUTPUT / "SCOPED_GIT_STATUS.txt"),
            "rows": len(status.splitlines()),
        },
        "global_status": {
            "path": (OUTPUT / "GLOBAL_GIT_STATUS.txt").relative_to(ROOT).as_posix(),
            "sha256": digest(OUTPUT / "GLOBAL_GIT_STATUS.txt"),
            "rows": len(global_status.splitlines()),
        },
        "tracked_diff": {
            "path": (OUTPUT / "PREEXISTING_TRACKED_DIFF.patch").relative_to(ROOT).as_posix(),
            "sha256": digest(OUTPUT / "PREEXISTING_TRACKED_DIFF.patch"),
            "bytes": len(patch),
        },
        "planning_close_comparison": {
            "source_manifest": (TOPIC / "baseline/close_002/MANIFEST.json").as_posix(),
            "unchanged": sum(row["state"] == "unchanged" for row in comparison),
            "changed": sum(row["state"] == "changed" for row in comparison),
            "missing": sum(row["state"] == "missing" for row in comparison),
            "rows": comparison,
        },
        "toolchain": {"rustc": rustc, "cargo": cargo},
        "recovery": {
            "archive_contains_actual_preimplementation_bytes": True,
            "tracked_deletions_and_binary_changes_are_recorded_by": "PREEXISTING_TRACKED_DIFF.patch plus SCOPED_GIT_STATUS.txt",
            "restore_warning": "Extract only into an isolated directory; do not overwrite the dirty worktree blindly.",
        },
    }
    manifest_path = OUTPUT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (OUTPUT / "MANIFEST.sha256").write_text(f"{digest(manifest_path)}  MANIFEST.json\n", encoding="utf-8")


if __name__ == "__main__":
    main()
