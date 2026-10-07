#!/usr/bin/env python3
"""Read-only currentness verification for R06 WP02 frame/capture target 001."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
SEGMENT = EXECUTION / "segments/wp02_runtime_frame_capture_001"
TARGET = ROOT / SEGMENT / "target/target_001"
EXPECTED_BASELINE_HASH = "26b5222aaa514b6622093429bd9994c3bab04f40791e0446bb2c96cd447fe6be"
EXPECTED_PREDECESSOR_HASH = "679336ad025f9dfaa6b44345ca3e494fd694e5903d49786a91d0ff270dbd642f"
EXPECTED_SOURCE_CHANGES = {
    "core_enhanced/src/game/battle/core/mod.rs": "modified",
    "core_enhanced/src/game/battle/core/resource_runtime.rs": "modified",
    "core_enhanced/src/game/battle/core/runtime_facts.rs": "modified",
    "core_enhanced/src/game/battle/core/value_expr_runtime.rs": "added",
    "core_enhanced/src/game/data/content_pack/mod.rs": "modified",
    "core_enhanced/src/game/data/mod.rs": "modified",
    "core_enhanced/src/game/events/combat.rs": "modified",
    "core_enhanced/src/game/resource/store.rs": "modified",
    "core_enhanced/src/game/value_expr/compile.rs": "modified",
    "core_enhanced/src/game/world/combat.rs": "modified",
}
REQUIRED_SUCCESS = {
    "focused_runtime_frame_capture",
    "r04_runtime_fact_regression",
    "r05_resource_store_regression",
    "r05_resource_lifetime_regression",
    "wp01_value_expr_regression",
    "wp01_content_pipeline_regression",
    "authoring_budget_regression",
    "checkpoint_save_regression",
    "literal_skill_runtime_regression",
    "probe_library_build",
    "wp01_review_probe_compile",
    "wp01_review_probe_after_fixture_fix",
    "wp01_parser_depth_probe",
    "all_targets_check",
    "focused_rustfmt_check",
    "clippy_with_predecessor_allows",
}
INVALID_ATTEMPT = "wp01_review_probe"


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    if not absolute.is_file():
        raise SystemExit(f"missing frozen file: {relative.as_posix()}")
    return {
        "path": relative.as_posix(),
        "bytes": absolute.stat().st_size,
        "sha256": digest(absolute),
    }


def verify_row(entry: dict[str, object]) -> None:
    current = row(Path(str(entry["path"])))
    if any(current[key] != entry[key] for key in ("path", "bytes", "sha256")):
        raise SystemExit(f"frozen file mismatch: {entry['path']}")


def validate_detached(entry: dict[str, object], expected_hash: str) -> dict[str, object]:
    path = ROOT / str(entry["manifest"])
    if digest(path) != expected_hash or entry["manifest_sha256"] != expected_hash:
        raise SystemExit(f"referenced manifest mismatch: {entry['manifest']}")
    detached = path.with_name("MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if detached != expected_hash:
        raise SystemExit(f"referenced detached hash mismatch: {entry['manifest']}")
    return json.loads(path.read_text(encoding="utf-8"))


def verify_archive(entry: dict[str, object], expected: list[dict[str, object]]) -> None:
    verify_row(entry)
    with tarfile.open(ROOT / str(entry["path"]), "r:gz") as archive:
        members = [member for member in archive.getmembers() if member.isfile()]
        if [member.name for member in members] != [item["path"] for item in expected]:
            raise SystemExit(f"archive member mismatch: {entry['path']}")
        for member, item in zip(members, expected):
            extracted = archive.extractfile(member)
            data = b"" if extracted is None else extracted.read()
            if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
                raise SystemExit(f"archive payload mismatch: {member.name}")


def scoped_paths(scope: list[str]) -> set[str]:
    paths: set[str] = set()
    for raw in scope:
        absolute = ROOT / raw
        if absolute.is_file():
            paths.add(Path(raw).as_posix())
        elif absolute.is_dir():
            paths.update(
                path.relative_to(ROOT).as_posix()
                for path in absolute.rglob("*")
                if path.is_file()
            )
        else:
            raise SystemExit(f"scope path missing: {raw}")
    return paths


def main() -> None:
    manifest_path = TARGET / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (TARGET / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit("target detached hash mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["record"] != "R06_WP02_RUNTIME_FRAME_CAPTURE_TARGET_001":
        raise SystemExit("unexpected target record")
    expected_status = "부분 구현·검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기"
    if manifest["status_text"] != expected_status:
        raise SystemExit("unexpected target status")

    baseline = validate_detached(
        manifest["pre_implementation_baseline"], EXPECTED_BASELINE_HASH
    )
    predecessor = validate_detached(
        manifest["predecessor_target"], EXPECTED_PREDECESSOR_HASH
    )
    if baseline["record"] != "R06_WP02_RUNTIME_FRAME_CAPTURE_BASELINE_001":
        raise SystemExit("unexpected baseline record")
    if predecessor["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_TARGET_001":
        raise SystemExit("unexpected predecessor record")
    verify_archive(baseline["source"]["archive"], baseline["source"]["files"])
    verify_archive(
        baseline["basis_documents"]["archive"], baseline["basis_documents"]["files"]
    )
    verify_row(manifest["preservation"]["predecessor_review"])
    verify_row(manifest["preservation"]["baseline_global_status"])
    verify_row(manifest["preservation"]["baseline_scoped_status"])

    source = manifest["current_source"]
    if source["file_count"] != len(source["files"]):
        raise SystemExit("current source count mismatch")
    if scoped_paths(source["scope"]) != {entry["path"] for entry in source["files"]}:
        raise SystemExit("current source set mismatch")
    for entry in source["files"]:
        verify_row(entry)
    verify_archive(source["archive"], source["files"])
    verify_row(source["recoverable_delta"])
    if (ROOT / str(source["recoverable_delta"]["path"])).stat().st_size == 0:
        raise SystemExit("recoverable delta is empty")
    verify_row(source["scoped_status"])
    verify_row(source["global_status"])

    changes = source["changes_from_baseline"]
    actual_changes = {entry["path"]: entry["state"] for entry in changes["files"]}
    if (
        actual_changes != EXPECTED_SOURCE_CHANGES
        or changes["added"] != 1
        or changes["modified"] != 9
        or changes["deleted"] != 0
    ):
        raise SystemExit("unexpected source change set")
    baseline_by_path = {entry["path"]: entry for entry in baseline["source"]["files"]}
    current_by_path = {entry["path"]: entry for entry in source["files"]}
    for change in changes["files"]:
        old = baseline_by_path.get(change["path"])
        new = current_by_path.get(change["path"])
        if (
            change["before_sha256"] != (None if old is None else old["sha256"])
            or change["before_bytes"] != (None if old is None else old["bytes"])
            or change["after_sha256"] != (None if new is None else new["sha256"])
            or change["after_bytes"] != (None if new is None else new["bytes"])
        ):
            raise SystemExit(f"change binding mismatch: {change['path']}")

    records = manifest["execution_records"]
    if records["file_count"] != len(records["files"]):
        raise SystemExit("record count mismatch")
    for entry in records["files"]:
        verify_row(entry)
    verify_archive(records["archive"], records["files"])

    evidence = manifest["evidence"]
    if evidence["file_count"] != len(evidence["files"]):
        raise SystemExit("evidence count mismatch")
    for entry in evidence["files"]:
        verify_row(entry)
    verify_archive(evidence["archive"], evidence["files"])

    receipts = {entry["label"]: entry for entry in manifest["validation_receipts"]}
    if set(receipts) != REQUIRED_SUCCESS | {INVALID_ATTEMPT}:
        raise SystemExit("validation receipt set mismatch")
    for label in REQUIRED_SUCCESS:
        receipt = receipts[label]
        if receipt["exit_code"] != 0 or receipt["role"] != "required_direct_success":
            raise SystemExit(f"required validation result mismatch: {label}")
        receipt_payload = json.loads(
            (ROOT / str(receipt["receipt"]["path"])).read_text(encoding="utf-8")
        )
        if receipt_payload.get("zero_match"):
            raise SystemExit(f"zero-match receipt: {label}")
        for identity in receipt_payload["source_identity"]:
            if current_by_path.get(identity["path"]) != identity:
                raise SystemExit(f"receipt source identity mismatch: {label}")
    invalid = receipts[INVALID_ATTEMPT]
    if (
        invalid["exit_code"] != 101
        or invalid["role"] != "invalid_fixture_setup_preserved_not_counted"
    ):
        raise SystemExit("invalid fixture attempt contract mismatch")
    for receipt in receipts.values():
        verify_row(receipt["receipt"])
        verify_row(receipt["output"])

    claims = manifest["completion_claims"]
    if (
        not claims["partial_implementation_self_validation_complete"]
        or not claims["stage_review_target_frozen"]
        or claims["review_pass_declared"]
        or claims["wp02_complete"]
        or claims["next_segment_started"]
        or claims["gex_20_started"]
        or claims["gex_30_started"]
        or claims["phase_complete"]
        or claims["goal_complete"]
    ):
        raise SystemExit("completion/progression claim mismatch")
    if not manifest["publication_gate"]["remains_closed"]:
        raise SystemExit("publication gate was opened")

    handoff = manifest["review_entry"]
    handoff_path = ROOT / str(handoff["path"])
    handoff_hash_path = ROOT / str(handoff["detached_sha256"])
    handoff_hash = handoff_hash_path.read_text(encoding="utf-8").split()[0]
    if digest(handoff_path) != handoff_hash:
        raise SystemExit("handoff detached hash mismatch")
    handoff_text = handoff_path.read_text(encoding="utf-8")
    if manifest_hash not in handoff_text or expected_status not in handoff_text:
        raise SystemExit("handoff is not bound to target identity/status")

    print(
        json.dumps(
            {
                "result": "CURRENTNESS_VERIFIED",
                "record": manifest["record"],
                "manifest_sha256": manifest_hash,
                "source_files": source["file_count"],
                "source_changes": changes["added"] + changes["modified"],
                "records": records["file_count"],
                "evidence": evidence["file_count"],
                "validation_receipts": len(receipts),
                "invalid_setup_attempt_preserved_not_counted": True,
                "publication_gate_closed": True,
                "review_pass_declared": False,
                "status": expected_status,
            },
            ensure_ascii=False,
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
