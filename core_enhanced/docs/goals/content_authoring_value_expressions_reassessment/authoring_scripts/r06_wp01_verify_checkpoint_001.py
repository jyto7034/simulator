#!/usr/bin/env python3
"""Read-only verification for the frozen WP01 checkpoint 001."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/segments/wp01_content_input_compile_001/checkpoints/checkpoint_001"
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


def main() -> None:
    manifest_path = TARGET / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (TARGET / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit("checkpoint detached hash mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["record"] != "R06_WP01_CONTENT_INPUT_CHECKPOINT_001":
        raise SystemExit("unexpected checkpoint record")
    verify_row(manifest["baseline"]["archive"])
    if digest(ROOT / manifest["baseline"]["manifest"]) != manifest["baseline"]["manifest_sha256"]:
        raise SystemExit("baseline manifest mismatch")

    source = manifest["current_source"]
    archive_path = ROOT / source["archive"]["path"]
    verify_row(source["archive"])
    expected = {entry["path"]: entry for entry in source["files"]}
    with tarfile.open(archive_path, "r:gz") as archive:
        members = {member.name: member for member in archive.getmembers() if member.isfile()}
        if set(members) != set(expected):
            raise SystemExit("checkpoint archive member set mismatch")
        for path, entry in expected.items():
            payload = archive.extractfile(members[path])
            data = payload.read() if payload is not None else b""
            if len(data) != entry["bytes"] or hashlib.sha256(data).hexdigest() != entry["sha256"]:
                raise SystemExit(f"checkpoint archive payload mismatch: {path}")
            verify_row(entry)
    verify_row(source["recoverable_delta"])
    for entry in manifest["records"] + manifest["evidence"]:
        verify_row(entry)
    for receipt in manifest["validation_receipts"]:
        if receipt["exit_code"] != 0:
            raise SystemExit(f"nonzero required receipt: {receipt['label']}")
        verify_row(receipt["receipt"])
        verify_row(receipt["output"])
    for entry in manifest["preserved_fixtures"]:
        verify_row(entry)
    print(
        json.dumps(
            {
                "result": "PASS",
                "record": manifest["record"],
                "manifest_sha256": manifest_hash,
                "source_files": len(expected),
                "changes": manifest["current_source"]["change_counts"],
                "validation_receipts": len(manifest["validation_receipts"]),
                "scope": "checkpoint continuation gate only; not R06/WP/Phase/Goal PASS",
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
