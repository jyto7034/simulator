#!/usr/bin/env python3
"""Read-only currentness verification for the WP01 F01/F02 correction target."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
CORRECTION = EXECUTION / "corrections/wp01_target_001_f01_f02"
TARGET = ROOT / CORRECTION / "target/correction_001"
EXPECTED_BASELINE_HASH = "a431db07b318c56f572b1799f9ce186903ecc667d3bddb898fbf78f8c7eb635e"
EXPECTED_PREDECESSOR_HASH = "6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de"
EXPECTED_SOURCE_CHANGES = {
    "core_enhanced/src/game/data/content_pack/tests.rs",
    "core_enhanced/src/game/value_expr/catalog.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/parse.rs",
}
REQUIRED_SUCCESS = {
    "post_fix_focused_fmt",
    "post_fix_value_expr_unit",
    "post_fix_content_pipeline_regression",
    "post_fix_authoring_regression",
    "post_fix_skill_runtime_regression",
    "post_fix_checkpoint_save_regression",
    "post_fix_all_targets_check",
    "post_fix_clippy_with_baseline_allows",
    "post_fix_final_probe_build",
    "post_fix_final_review_probe_compile",
    "post_fix_final_review_probe",
    "post_fix_parser_depth_probe",
}
EXPECTED_FAILURES = {
    "pre_fix_review_probe_reproduced": 101,
    "pre_fix_review_probe": 101,
}


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
    archive_path = ROOT / str(entry["path"])
    if digest(archive_path) != entry["sha256"] or archive_path.stat().st_size != entry["bytes"]:
        raise SystemExit(f"archive identity mismatch: {entry['path']}")
    with tarfile.open(archive_path, "r:gz") as archive:
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
    if manifest["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_TARGET_001":
        raise SystemExit("unexpected target record")
    if manifest["status_text"] != "교정·자체 검증 종료 / 교정 타깃 동결 / 재검토 요청 대기":
        raise SystemExit("unexpected target status")

    baseline = validate_detached(manifest["pre_fix_baseline"], EXPECTED_BASELINE_HASH)
    predecessor = validate_detached(
        manifest["predecessor_target"], EXPECTED_PREDECESSOR_HASH
    )
    if baseline["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_BASELINE_001":
        raise SystemExit("unexpected baseline record")
    if predecessor["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001":
        raise SystemExit("unexpected predecessor record")

    for section in (
        "source",
        "existing_execution_documents",
        "review_inputs",
        "predecessor_target_records",
    ):
        value = baseline[section]
        verify_archive(value["archive"], value["files"])
    for section in (
        "predecessor_target_records",
        "predecessor_target_evidence",
        "review_inputs",
        "existing_execution_documents",
    ):
        for entry in manifest["preservation"][section]:
            verify_row(entry)

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
    changes = source["changes_from_pre_fix"]
    if (
        changes["added"] != 0
        or changes["deleted"] != 0
        or changes["modified"] != 4
        or {entry["path"] for entry in changes["files"]} != EXPECTED_SOURCE_CHANGES
        or any(entry["state"] != "modified" for entry in changes["files"])
    ):
        raise SystemExit("unexpected source change set")
    baseline_by_path = {entry["path"]: entry for entry in baseline["source"]["files"]}
    current_by_path = {entry["path"]: entry for entry in source["files"]}
    for change in changes["files"]:
        old = baseline_by_path[change["path"]]
        new = current_by_path[change["path"]]
        if (
            change["before_sha256"] != old["sha256"]
            or change["before_bytes"] != old["bytes"]
            or change["after_sha256"] != new["sha256"]
            or change["after_bytes"] != new["bytes"]
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
    if not REQUIRED_SUCCESS <= set(receipts):
        raise SystemExit("required validation receipt missing")
    for label in REQUIRED_SUCCESS:
        if receipts[label]["exit_code"] != 0 or receipts[label]["role"] != "required_direct_success":
            raise SystemExit(f"required validation result mismatch: {label}")
    for label, exit_code in EXPECTED_FAILURES.items():
        if receipts[label]["exit_code"] != exit_code:
            raise SystemExit(f"preserved failure mismatch: {label}")
    for receipt in receipts.values():
        verify_row(receipt["receipt"])
        verify_row(receipt["output"])

    claims = manifest["completion_claims"]
    if (
        claims["review_pass_declared"]
        or claims["wp01_complete"]
        or claims["wp02_started"]
        or claims["next_r06_feature_started"]
        or claims["gex_20_started"]
        or claims["gex_30_started"]
        or claims["phase_complete"]
        or claims["goal_complete"]
    ):
        raise SystemExit("forbidden completion/progression claim")

    handoff = manifest["review_entry"]
    handoff_path = ROOT / str(handoff["path"])
    handoff_hash_path = ROOT / str(handoff["detached_sha256"])
    handoff_hash = handoff_hash_path.read_text(encoding="utf-8").split()[0]
    if digest(handoff_path) != handoff_hash:
        raise SystemExit("handoff detached hash mismatch")
    handoff_text = handoff_path.read_text(encoding="utf-8")
    if manifest_hash not in handoff_text or manifest["status_text"] not in handoff_text:
        raise SystemExit("handoff is not bound to target identity/status")

    print(
        json.dumps(
            {
                "result": "CURRENTNESS_VERIFIED",
                "record": manifest["record"],
                "manifest_sha256": manifest_hash,
                "source_files": source["file_count"],
                "source_changes": changes["modified"],
                "records": records["file_count"],
                "evidence": evidence["file_count"],
                "validation_receipts": len(receipts),
                "pre_fix_expected_failure_preserved": True,
                "invalid_setup_attempt_preserved_not_counted": True,
                "review_pass_declared": False,
                "status": manifest["status_text"],
            },
            ensure_ascii=False,
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
