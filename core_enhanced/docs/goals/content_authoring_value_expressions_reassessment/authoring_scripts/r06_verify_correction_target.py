#!/usr/bin/env python3
"""Read-only verification for the frozen checkpoint-001 correction target."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/checkpoint_001_f01_f04/target/correction_001"
)


def byte_digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def verify_row(row: dict[str, object]) -> None:
    path = ROOT / row["path"]
    if not path.is_file():
        raise SystemExit(f"missing frozen file: {row['path']}")
    if digest(path) != row["sha256"] or path.stat().st_size != row["bytes"]:
        raise SystemExit(f"frozen file mismatch: {row['path']}")


def main() -> None:
    manifest_path = TARGET / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (TARGET / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit("target detached hash mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["record"] != "R06_CHECKPOINT_001_CORRECTION_TARGET_001":
        raise SystemExit("unexpected target record")
    if manifest["state"] != "corrected_re_review_requested":
        raise SystemExit("target is not in re-review-requested state")

    for key in ("parent_target", "pre_fix_baseline"):
        entry = manifest[key]
        path = ROOT / entry["manifest"]
        if digest(path) != entry["manifest_sha256"]:
            raise SystemExit(f"{key} manifest mismatch")
        detached_path = path.with_name("MANIFEST.sha256")
        if detached_path.read_text(encoding="utf-8").split()[0] != entry["manifest_sha256"]:
            raise SystemExit(f"{key} detached hash mismatch")

    for row in manifest["preservation"]["review_evidence"]:
        verify_row(row)
    for row in manifest["execution_records"]:
        verify_row(row)
    for row in manifest["evidence"]:
        verify_row(row)
    for key in ("archive", "recoverable_delta", "scoped_status", "global_status"):
        verify_row(manifest["current_source"][key])

    source_rows = manifest["current_source"]["files"]
    if len(source_rows) != manifest["current_source"]["file_count"]:
        raise SystemExit("source row count mismatch")
    for row in source_rows:
        verify_row(row)

    archive_path = ROOT / manifest["current_source"]["archive"]["path"]
    with tarfile.open(archive_path, "r:gz") as archive:
        members = {member.name: member for member in archive.getmembers() if member.isfile()}
        expected_names = {row["path"] for row in source_rows}
        if set(members) != expected_names:
            raise SystemExit("archive member set mismatch")
        for row in source_rows:
            extracted = archive.extractfile(members[row["path"]])
            if extracted is None:
                raise SystemExit(f"archive member unreadable: {row['path']}")
            data = extracted.read()
            if len(data) != row["bytes"] or byte_digest(data) != row["sha256"]:
                raise SystemExit(f"archive member mismatch: {row['path']}")

    changes = manifest["current_source"]["changes_from_pre_fix"]
    expected_changes = {
        "core_enhanced/src/game/value_expr/compile.rs",
        "core_enhanced/src/game/value_expr/eval.rs",
        "core_enhanced/src/game/value_expr/mod.rs",
        "core_enhanced/src/game/value_expr/tests.rs",
    }
    if changes["added"] != 0 or changes["deleted"] != 0 or changes["modified"] != 4:
        raise SystemExit("unexpected source change counts")
    if {row["path"] for row in changes["files"]} != expected_changes:
        raise SystemExit("unexpected source change paths")

    for receipt in manifest["validation_receipts"]:
        receipt_path = ROOT / receipt["receipt"]
        if digest(receipt_path) != receipt["receipt_sha256"]:
            raise SystemExit(f"receipt mismatch: {receipt['label']}")
        verify_row(receipt["output"])
    required_successes = {
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
    receipt_by_label = {row["label"]: row for row in manifest["validation_receipts"]}
    for label in required_successes:
        if receipt_by_label[label]["exit_code"] != 0:
            raise SystemExit(f"required validation did not pass: {label}")

    claims = manifest["completion_claims"]
    if not claims["correction_re_review_ready"]:
        raise SystemExit("correction re-review readiness not recorded")
    prohibited = (
        "next_r06_feature_started",
        "gex_20_started",
        "gex_30_started",
        "work_package_complete",
        "phase_complete",
        "goal_complete",
    )
    if any(claims[key] for key in prohibited):
        raise SystemExit("out-of-scope progress/completion claim found")

    print(f"record={manifest['record']}")
    print(f"manifest_sha256={manifest_hash}")
    print(f"source_files={len(source_rows)} changed_files={changes['modified']}")
    print(f"review_evidence={len(manifest['preservation']['review_evidence'])}")
    print(f"validation_receipts={len(manifest['validation_receipts'])}")
    print("state=CORRECTED_RE_REVIEW_REQUESTED")
    print("verification=PASS")


if __name__ == "__main__":
    main()
