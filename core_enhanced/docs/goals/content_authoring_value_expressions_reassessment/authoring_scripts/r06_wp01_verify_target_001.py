#!/usr/bin/env python3
"""Read-only currentness verifier for frozen WP01 target 001."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/segments/wp01_content_input_compile_001/target/target_001"
)


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def verify_row(entry: dict[str, object], current: bool = True) -> None:
    path = ROOT / str(entry["path"])
    if not path.is_file() or path.stat().st_size != entry["bytes"] or digest(path) != entry["sha256"]:
        qualifier = "current " if current else ""
        raise SystemExit(f"{qualifier}row mismatch: {entry['path']}")


def verify_archive(
    archive_entry: dict[str, object], expected_rows: list[dict[str, object]]
) -> None:
    verify_row(archive_entry)
    expected = {str(entry["path"]): entry for entry in expected_rows}
    with tarfile.open(ROOT / str(archive_entry["path"]), "r:gz") as archive:
        members = {member.name: member for member in archive.getmembers() if member.isfile()}
        if set(members) != set(expected):
            raise SystemExit(f"archive member set mismatch: {archive_entry['path']}")
        for path, entry in expected.items():
            payload = archive.extractfile(members[path])
            data = payload.read() if payload is not None else b""
            if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise SystemExit(f"archive payload mismatch: {path}")


def main() -> None:
    manifest_path = TARGET / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (TARGET / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit("target detached hash mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001":
        raise SystemExit("unexpected target record")
    if manifest["state"] != "partial_implementation_validated_stage_review_request_pending":
        raise SystemExit("unexpected target state")
    if any(manifest["completion_claims"].values()):
        raise SystemExit("forbidden completion claim in partial target")

    baseline_path = ROOT / manifest["baseline"]["manifest"]
    if digest(baseline_path) != manifest["baseline"]["manifest_sha256"]:
        raise SystemExit("baseline manifest mismatch")
    verify_row(manifest["baseline"]["archive"])
    predecessor_path = ROOT / manifest["predecessor"]["manifest"]
    if digest(predecessor_path) != manifest["predecessor"]["manifest_sha256"]:
        raise SystemExit("predecessor manifest mismatch")
    checkpoint_path = ROOT / manifest["stage_checkpoint"]["manifest"]
    if digest(checkpoint_path) != manifest["stage_checkpoint"]["manifest_sha256"]:
        raise SystemExit("stage checkpoint manifest mismatch")

    source = manifest["current_source"]
    for entry in source["files"]:
        verify_row(entry)
    verify_archive(source["archive"], source["files"])
    verify_row(source["recoverable_delta"])

    for entry in manifest["records"] + manifest["evidence"]:
        verify_row(entry)
    verify_archive(manifest["records_archive"], manifest["records"])
    verify_archive(manifest["evidence_archive"], manifest["evidence"])

    labels = set()
    required_count = 0
    expected_failure_count = 0
    invalid_attempt_count = 0
    for receipt in manifest["validation_receipts"]:
        label = receipt["label"]
        if label in labels:
            raise SystemExit(f"duplicate validation label: {label}")
        labels.add(label)
        verify_row(receipt["receipt"])
        verify_row(receipt["output"])
        role = receipt["role"]
        if role.endswith("success"):
            if receipt["exit_code"] != 0:
                raise SystemExit(f"nonzero success receipt: {label}")
            required_count += role == "required_direct_success"
        elif role == "expected_recorded_failure":
            if receipt["exit_code"] == 0:
                raise SystemExit(f"expected failure unexpectedly succeeded: {label}")
            expected_failure_count += 1
        elif role == "invalid_zero_test_attempt_not_counted":
            if receipt["exit_code"] != 0:
                raise SystemExit(f"invalid attempt exit mismatch: {label}")
            invalid_attempt_count += 1
        else:
            raise SystemExit(f"unknown validation role: {role}")
    if required_count != 11 or expected_failure_count != 2 or invalid_attempt_count != 1:
        raise SystemExit("validation role counts changed")

    for entry in manifest["preserved_fixtures"]:
        verify_row(entry)
    for entry in manifest["status"].values():
        if isinstance(entry, dict) and "path" in entry:
            verify_row(entry)

    print(
        json.dumps(
            {
                "result": "CURRENTNESS_VERIFIED",
                "record": manifest["record"],
                "manifest_sha256": manifest_hash,
                "source_files": len(source["files"]),
                "changes": source["change_counts"],
                "records": len(manifest["records"]),
                "evidence": len(manifest["evidence"]),
                "validation_receipts": len(manifest["validation_receipts"]),
                "scope": "read-only target currentness only; not R06/WP/Phase/Goal or review PASS",
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
