#!/usr/bin/env python3
"""Read-only currentness and restoration verifier for the frozen segment target."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/segments/wp02_effect_execution_prefix_001/"
    "target/effect_execution_prefix_001"
)


def identity(payload: bytes) -> dict[str, object]:
    return {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}


def assert_row(row: dict[str, object]) -> None:
    payload = (ROOT / row["path"]).read_bytes()
    assert identity(payload) == {"bytes": row["bytes"], "sha256": row["sha256"]}, row["path"]


def compare_tree(root: Path, rows: list[dict[str, object]], side: str) -> None:
    for row in rows:
        expected = row[side]
        payload = (root / row["path"]).read_bytes()
        assert identity(payload) == expected, (side, row["path"])


def run(*args: str, cwd: Path) -> None:
    subprocess.run(args, cwd=cwd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    arguments = parser.parse_args()
    manifest_payload = (TARGET / "MANIFEST.json").read_bytes()
    expected_hash = (TARGET / "MANIFEST.sha256").read_text().split()[0]
    assert hashlib.sha256(manifest_payload).hexdigest() == expected_hash
    manifest = json.loads(manifest_payload)
    assert manifest["state"] == "partial_implementation_self_validation_complete_stage_review_pending"
    assert manifest["terminal_status"] == "부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기"
    assert manifest["claims"]["stage_review_target_frozen"] is True
    assert manifest["claims"]["stage_review_pending"] is True
    for forbidden in [
        "review_pass_declared", "wp02_complete", "r06_complete", "phase_complete",
        "goal_complete", "publication_unblocked", "limits_relaxed",
    ]:
        assert manifest["claims"][forbidden] is False

    assert_row(manifest["predecessor"])
    assert_row(manifest["implementation_baseline"])
    for row in manifest["artifacts"]:
        assert_row(row)
    for row in manifest["basis"]["files"]:
        assert_row(row)
    for row in manifest["evidence"]["files"]:
        assert_row(row)
    assert_row(manifest["handoff"])

    source_rows = manifest["source"]["files"]
    current_mismatch = []
    for row in source_rows:
        payload = (ROOT / row["path"]).read_bytes()
        if identity(payload) != row["after"]:
            current_mismatch.append(row["path"])
    assert not current_mismatch, current_mismatch
    assert sum(row["changed"] for row in source_rows) == manifest["source"]["changed_file_count"]

    before_root = Path(tempfile.mkdtemp(prefix="r06-wp02-verify-before-"))
    after_root = Path(tempfile.mkdtemp(prefix="r06-wp02-verify-after-"))
    with tarfile.open(ROOT / manifest["source"]["before_archive"]["path"], "r:gz") as archive:
        archive.extractall(before_root, filter="data")
    with tarfile.open(ROOT / manifest["source"]["after_archive"]["path"], "r:gz") as archive:
        archive.extractall(after_root, filter="data")
    compare_tree(before_root, source_rows, "before")
    compare_tree(after_root, source_rows, "after")

    forward = ROOT / manifest["source"]["forward_delta"]["path"]
    reverse = ROOT / manifest["source"]["restore_delta"]["path"]
    run("git", "apply", "--check", str(forward), cwd=before_root)
    run("git", "apply", str(forward), cwd=before_root)
    compare_tree(before_root, source_rows, "after")
    run("git", "apply", "--check", str(reverse), cwd=before_root)
    run("git", "apply", str(reverse), cwd=before_root)
    compare_tree(before_root, source_rows, "before")
    shutil.rmtree(before_root)
    shutil.rmtree(after_root)

    result = {
        "record": "R06_WP02_EFFECT_EXECUTION_TARGET_VERIFICATION_001",
        "result": "PASS",
        "manifest_sha256": expected_hash,
        "artifacts_verified": len(manifest["artifacts"]),
        "basis_files_verified": len(manifest["basis"]["files"]),
        "evidence_files_verified": len(manifest["evidence"]["files"]),
        "source_files_current": len(source_rows),
        "changed_files": manifest["source"]["changed_file_count"],
        "forward_and_restore_delta_verified": True,
        "review_pass_declared": False,
        "wp_phase_goal_complete": False,
    }
    payload = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if arguments.output:
        output = Path(arguments.output)
        if output.exists():
            raise SystemExit(f"refusing to overwrite verifier output: {output}")
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
