#!/usr/bin/env python3
"""Read-only verification for the frozen R06 correction-002 target."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import tarfile


ROOT = Path(__file__).resolve().parents[5]
TARGET = ROOT / (
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/correction_001_r01_r03/target/correction_002"
)
EXPECTED_CHANGES = {
    "core_enhanced/src/game/value_expr/compile.rs",
    "core_enhanced/src/game/value_expr/eval.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/tests.rs",
}
REQUIRED_SUCCESSES = {
    "pre_fix_build",
    "pre_fix_reaudit_probe_compile",
    "post_fix_build",
    "post_fix_value_expr_unit",
    "post_fix_correction_probe_compile",
    "post_fix_correction_probe",
    "post_fix_reaudit_probe_compile",
    "post_fix_reaudit_probe",
    "post_fix_deep_shape_probe",
    "post_fix_fingerprint_compat",
    "post_fix_all_targets_check",
    "post_fix_focused_fmt",
    "post_fix_clippy_with_baseline_allows",
    "post_fix_all_targets_test",
}
EXPECTED_FAILURES = {
    "pre_fix_reaudit_probe": 101,
    "pre_fix_deep_shape_probe": -6,
    "intermediate_unit_invalid_resource_fixture": 101,
}


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


def validate_detached(entry: dict[str, object]) -> dict[str, object]:
    path = ROOT / entry["manifest"]
    if digest(path) != entry["manifest_sha256"]:
        raise SystemExit(f"referenced manifest mismatch: {entry['manifest']}")
    detached = path.with_name("MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if detached != entry["manifest_sha256"]:
        raise SystemExit(f"referenced detached hash mismatch: {entry['manifest']}")
    return json.loads(path.read_text(encoding="utf-8"))


def verify_archive(row: dict[str, object], expected: list[dict[str, object]]) -> None:
    verify_row(row)
    with tarfile.open(ROOT / row["path"], "r:gz") as archive:
        members = {member.name: member for member in archive.getmembers() if member.isfile()}
        if set(members) != {entry["path"] for entry in expected}:
            raise SystemExit(f"archive member set mismatch: {row['path']}")
        for entry in expected:
            extracted = archive.extractfile(members[entry["path"]])
            if extracted is None:
                raise SystemExit(f"archive member unreadable: {entry['path']}")
            data = extracted.read()
            if len(data) != entry["bytes"] or byte_digest(data) != entry["sha256"]:
                raise SystemExit(f"archive member mismatch: {entry['path']}")


def scoped_paths(scope: list[str]) -> set[str]:
    found: set[str] = set()
    for item in scope:
        path = ROOT / item
        if path.is_file():
            found.add(Path(item).as_posix())
        elif path.is_dir():
            found.update(
                child.relative_to(ROOT).as_posix()
                for child in path.rglob("*")
                if child.is_file()
            )
        else:
            raise SystemExit(f"source scope path missing: {item}")
    return found


def verify_changes(
    changes: dict[str, object],
    before: list[dict[str, object]],
    after: list[dict[str, object]],
    expected_paths: set[str],
) -> None:
    if changes["added"] != 0 or changes["deleted"] != 0:
        raise SystemExit("unexpected added/deleted files in frozen delta")
    if changes["modified"] != len(expected_paths):
        raise SystemExit("unexpected modified file count in frozen delta")
    if {entry["path"] for entry in changes["files"]} != expected_paths:
        raise SystemExit("unexpected modified paths in frozen delta")
    before_by_path = {entry["path"]: entry for entry in before}
    after_by_path = {entry["path"]: entry for entry in after}
    for change in changes["files"]:
        old = before_by_path[change["path"]]
        new = after_by_path[change["path"]]
        if (
            change["before_sha256"] != old["sha256"]
            or change["before_bytes"] != old["bytes"]
            or change["after_sha256"] != new["sha256"]
            or change["after_bytes"] != new["bytes"]
        ):
            raise SystemExit(f"delta hash binding mismatch: {change['path']}")


def main() -> None:
    manifest_path = TARGET / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (TARGET / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit("target detached hash mismatch")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["record"] != "R06_CHECKPOINT_001_CORRECTION_TARGET_002":
        raise SystemExit("unexpected target record")
    if manifest["state"] != "corrected_re_review_requested":
        raise SystemExit("target is not in re-review-requested state")

    predecessor = validate_detached(manifest["predecessor_target"])
    if manifest["predecessor_target"]["manifest_sha256"] != "87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e":
        raise SystemExit("unexpected predecessor target identity")
    old_baseline = validate_detached(manifest["predecessor_baseline"])
    baseline = validate_detached(manifest["pre_fix_baseline"])
    if manifest["pre_fix_baseline"]["manifest_sha256"] != "96eb47695ea346627b5c0c6aedbd75953cf496353ad00b03b04d37bd80232bda":
        raise SystemExit("unexpected correction-002 baseline identity")

    for entry in manifest["preservation"]["predecessor_execution_records"]:
        verify_row(entry)
    for entry in manifest["preservation"]["predecessor_evidence"]:
        verify_row(entry)
    for entry in manifest["preservation"]["review_inputs"]:
        verify_row(entry)
    if predecessor["record"] != "R06_CHECKPOINT_001_CORRECTION_TARGET_001":
        raise SystemExit("unexpected predecessor record")
    if old_baseline["record"] != "R06_CHECKPOINT_001_CORRECTION_BASELINE_001":
        raise SystemExit("unexpected historical baseline record")
    if baseline["record"] != "R06_CHECKPOINT_001_CORRECTION_BASELINE_002":
        raise SystemExit("unexpected pre-fix baseline record")

    source = manifest["current_source"]
    if source["file_count"] != len(source["files"]):
        raise SystemExit("source file count mismatch")
    if scoped_paths(source["scope"]) != {entry["path"] for entry in source["files"]}:
        raise SystemExit("current source set differs from frozen target")
    for entry in source["files"]:
        verify_row(entry)
    verify_archive(source["archive"], source["files"])
    verify_row(source["recoverable_delta"])
    verify_row(source["scoped_status"])
    verify_row(source["global_status"])
    verify_changes(
        source["changes_from_pre_fix"],
        baseline["source"]["files"],
        source["files"],
        EXPECTED_CHANGES,
    )

    documents = manifest["documents"]
    if documents["file_count"] != len(documents["files"]):
        raise SystemExit("document file count mismatch")
    for entry in documents["files"]:
        verify_row(entry)
    verify_archive(documents["archive"], documents["files"])
    verify_row(documents["recoverable_delta"])
    document_paths = {entry["path"] for entry in documents["files"]}
    verify_changes(
        documents["changes_from_pre_fix"],
        baseline["mutable_documents"]["files"],
        documents["files"],
        document_paths,
    )

    for entry in manifest["execution_records"]:
        verify_row(entry)
    evidence = manifest["evidence"]
    if evidence["file_count"] != len(evidence["files"]):
        raise SystemExit("evidence file count mismatch")
    for entry in evidence["files"]:
        verify_row(entry)
    verify_archive(evidence["archive"], evidence["files"])

    entry_currentness_row = manifest["entry_currentness"]["record"]
    verify_row(entry_currentness_row)
    entry_currentness = json.loads((ROOT / entry_currentness_row["path"]).read_text(encoding="utf-8"))
    if entry_currentness["current_target_reference_hashes_passed"] is not True:
        raise SystemExit("entry currentness failed current target checks")
    errors = entry_currentness["errors"]
    if len(errors) != 1 or errors[0]["expected"] != "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7":
        raise SystemExit("entry currentness contains an unexplained mismatch")

    r03 = manifest["r03_recovery"]
    verify_row(r03["record"])
    verify_row(r03["historical_plan"])
    verify_row(r03["raw_provenance"])
    if r03["historical_plan"]["bytes"] != 4460 or r03["historical_plan"]["sha256"] != "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7":
        raise SystemExit("historical PLAN identity mismatch")
    recovery = json.loads((ROOT / r03["record"]["path"]).read_text(encoding="utf-8"))
    event = json.loads((ROOT / r03["raw_provenance"]["path"]).read_text(encoding="utf-8"))
    historical_path = str(ROOT / Path(recovery["predecessor_reference"]["path"]))
    content = event["payload"]["item"]["changes"][historical_path]["content"].encode()
    if content != (ROOT / r03["historical_plan"]["path"]).read_bytes():
        raise SystemExit("historical PLAN differs from raw provenance")
    if old_baseline["correction_plan"]["sha256"] != byte_digest(content) or len(content) != 4460:
        raise SystemExit("historical PLAN does not satisfy predecessor reference")
    if baseline["recovery"]["historical_r03_plan_bytes_preserved"] is not False:
        raise SystemExit("pre-fix-002 baseline historical fact was rewritten")

    receipts = {entry["label"]: entry for entry in manifest["validation_receipts"]}
    if set(receipts) != REQUIRED_SUCCESSES | set(EXPECTED_FAILURES):
        raise SystemExit("validation receipt set mismatch")
    for label in REQUIRED_SUCCESSES:
        if receipts[label]["exit_code"] != 0:
            raise SystemExit(f"required validation did not pass: {label}")
    for label, exit_code in EXPECTED_FAILURES.items():
        if receipts[label]["exit_code"] != exit_code:
            raise SystemExit(f"expected failure identity changed: {label}")
    for receipt in receipts.values():
        receipt_path = ROOT / receipt["receipt"]
        if digest(receipt_path) != receipt["receipt_sha256"]:
            raise SystemExit(f"validation receipt mismatch: {receipt['label']}")
        verify_row(receipt["output"])

    handoff = manifest["review_entry"]
    handoff_path = ROOT / handoff["path"]
    handoff_hash_path = ROOT / handoff["detached_sha256"]
    handoff_hash = handoff_hash_path.read_text(encoding="utf-8").split()[0]
    if digest(handoff_path) != handoff_hash:
        raise SystemExit("review handoff detached hash mismatch")
    handoff_content = handoff_path.read_text(encoding="utf-8")
    if manifest_hash not in handoff_content or "교정본 재검토 준비 완료" not in handoff_content:
        raise SystemExit("review handoff is not bound to this target")

    claims = manifest["completion_claims"]
    if not claims["correction_re_review_ready"]:
        raise SystemExit("correction re-review readiness is not recorded")
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
    print(f"source_files={len(source['files'])} changed_files={source['changes_from_pre_fix']['modified']}")
    print(f"documents={len(documents['files'])} evidence={len(evidence['files'])}")
    print(f"validation_receipts={len(receipts)}")
    print("r03_historical_plan=RECOVERED_AND_GENERATION_SEPARATED")
    print("state=CORRECTED_RE_REVIEW_REQUESTED")
    print("verification=PASS")


if __name__ == "__main__":
    main()
