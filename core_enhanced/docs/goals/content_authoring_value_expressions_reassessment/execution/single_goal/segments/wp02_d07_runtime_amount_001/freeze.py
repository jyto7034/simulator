"""Freeze D07-B and verify current bytes plus isolated reversible deltas."""

import datetime
import difflib
import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

ROOT = Path("/mnt/f/work/simulator")
HERE = Path(__file__).resolve().parent
TARGET = HERE / "target/d07_runtime_amount_001"
BASE_PATH = HERE / "baseline/entry_001/MANIFEST.json"
EXPECTED_AFTER_DIGEST = "ad8021359785e365c3f9560268bcef2a95205495f9e33a22314c3f97743480c4"
EXPECTED_PRODUCT_DIGEST = "6b5e0291b1b6f936c277e2138bc2c2adbbeaaa2b3df5c1b4d5bf31b5acff6807"
ADDED = {
    "core_enhanced/src/authoring/amount_query.rs",
    "core_enhanced/src/game/battle/core/amount_observation.rs",
}
MODIFIED = {
    'core_enhanced/src/application.rs',
    'core_enhanced/src/authoring/clone_shape.rs',
    'core_enhanced/src/authoring/control.rs',
    'core_enhanced/src/authoring/definition.rs',
    'core_enhanced/src/authoring/mod.rs',
    'core_enhanced/src/authoring/trial.rs',
    'core_enhanced/src/bin/author_content.rs',
    'core_enhanced/src/game/battle/core/authoring_observation.rs',
    'core_enhanced/src/game/battle/core/boss_runtime.rs',
    'core_enhanced/src/game/battle/core/commands.rs',
    'core_enhanced/src/game/battle/core/death_runtime.rs',
    'core_enhanced/src/game/battle/core/field_work.rs',
    'core_enhanced/src/game/battle/core/mod.rs',
    'core_enhanced/src/game/battle/core/movement/blocking.rs',
    'core_enhanced/src/game/battle/core/movement/engine.rs',
    'core_enhanced/src/game/battle/core/queue.rs',
    'core_enhanced/src/game/battle/core/runtime_facts.rs',
    'core_enhanced/src/game/battle/core/sim.rs',
    'core_enhanced/src/game/battle/core/target_usefulness.rs',
    'core_enhanced/src/game/battle/core/targeting.rs',
    'core_enhanced/src/game/battle/core/value_expr_runtime.rs',
    'core_enhanced/src/game/battle/core/wave_runtime.rs',
    'core_enhanced/src/game/battle/damage.rs',
    'core_enhanced/src/game/battle/enums.rs',
    'core_enhanced/src/game/battle/event_log.rs',
    'core_enhanced/src/game/battle/mod.rs',
    'core_enhanced/src/game/battle/runtime_control.rs',
    'core_enhanced/src/game/battle/runtime_facts.rs',
    'core_enhanced/src/game/battle/tile_range.rs',
    'core_enhanced/src/game/data/content_pack/raw.rs',
    'core_enhanced/src/game/data/live_ron.rs',
    'core_enhanced/src/game/data/skill_data.rs',
    'core_enhanced/src/game/events/combat.rs',
    'core_enhanced/src/game/mod.rs',
    'core_enhanced/src/game/resource/runtime.rs',
    'core_enhanced/src/game/value_expr/mod.rs',
    'core_enhanced/src/game/world.rs',
    'core_enhanced/src/game/world/active_run.rs',
    'core_enhanced/src/game/world/authoring_observation.rs',
    'core_enhanced/src/game/world/combat.rs',
    'core_enhanced/src/game/world/node_flow.rs',
    'core_enhanced/src/game/world/state.rs',
    'core_enhanced/src/game/world/tests/checkpoint_validation.rs',
    'core_enhanced/src/game/world/tests/combat.rs',
    'core_enhanced/src/authoring/amount.rs',
}
REQUIRED_RECORDS = {
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/CODE_REVIEW_003.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_IMPLEMENTATION_REQUEST_20260927.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_B_IMPLEMENTATION_REQUEST_20260927.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_B_CODE_EVIDENCE_20260927.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_B_DOCUMENT_VALIDATION_20260927.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_d07_static_explain_001/target/d07_static_explain_001/MANIFEST.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_static_explain_001_review_001/REVIEW.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/README.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/RESULT.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/ALL_TARGETS.log",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/ALL_TARGETS.time",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/SANDBOX_ATTEMPT.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_test_timing_001/SANDBOX_ATTEMPT.log",
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def meta(path):
    data = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)), "bytes": len(data), "sha256": sha(data)}


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write("\n")


def check(entry):
    assert meta(ROOT / entry["path"]) == entry, entry["path"]


baseline = json.loads(BASE_PATH.read_text())
for entry in (baseline["predecessor"], baseline["review"], baseline["request"], baseline["archive"], baseline["git_status"]):
    check(entry)
before_files = [entry["path"] for entry in baseline["source"]["files"]]
added_files = baseline["source"]["absent_before"]
assert len(before_files) == len(set(before_files)) == 338
assert set(added_files) == ADDED
after_files = before_files + added_files
# D07-A inherited 333 product paths, appended amount.rs after four tool/lock paths.
assert before_files[333:337] == [
    "core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs",
    "core_enhanced/tools/runtime_architecture_preservation/src/main.rs",
    "core_enhanced/tools/runtime_architecture_final_validation/Cargo.lock",
    "core_enhanced/tools/runtime_architecture_preservation/Cargo.lock",
]
assert before_files[337] == "core_enhanced/src/authoring/amount.rs"
product_files = before_files[:333] + [before_files[337]] + added_files


def archive_contents(path, expected):
    contents = {}
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        assert {m.name for m in members} == set(expected)
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert ".." not in Path(member.name).parts
            source = archive.extractfile(member)
            assert source is not None
            contents[member.name] = source.read()
    return contents


def archived_before():
    return archive_contents(ROOT / baseline["archive"]["path"], before_files)


def identity(contents, selected):
    rows = [(name, len(contents[name]), sha(contents[name])) for name in selected]
    return sha(json.dumps(rows, separators=(",", ":")).encode())


def isolated_patch(origin, destination, patch_name, origin_files, destination_files):
    with tempfile.TemporaryDirectory(prefix="wp02-d07-runtime-amount-restore-") as temp:
        root = Path(temp)
        for name in origin_files:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(origin[name])
        subprocess.run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(TARGET / patch_name)], cwd=root, check=True, capture_output=True)
        for name in destination_files:
            assert (root / name).read_bytes() == destination[name], name
        for name in set(origin_files) - set(destination_files):
            assert not (root / name).exists(), name


def validate_receipts():
    final = json.loads((HERE / "evidence/FINAL_VALIDATION.json").read_text())
    assert final["source"] == {"file_count": 340, "digest": EXPECTED_AFTER_DIGEST, "product_file_count": 336, "product_digest": EXPECTED_PRODUCT_DIGEST}
    validations = final["validations"]
    for key, passed in (("authoring", 112), ("value_expr", 91), ("all_targets_parallel", 1538), ("focused_d07_b", 5), ("pc36", 1)):
        assert validations[key]["exit_code"] == 0 and validations[key]["test_passed"] == passed and validations[key]["test_failed"] == 0, key
    for key in ("format", "check_all_targets", "clippy"):
        assert validations[key]["exit_code"] == 0, key
    assert validations["r05"]["exit_code"] == 101
    assert validations["r05"]["policy_classification"] == "warning_concurrent_peak_only"
    assert validations["cli"]["trace_on"]["exit_code"] == 0
    assert validations["cli"]["trace_off"]["exit_code"] == 0
    assert validations["cli"]["amount_documents"] == {"byte_equal": True, "sha256": "8d555df63d8410237e0a9796d7cadb038197d5e31c3dcf22f158c8c3bf6ceaa0", "count_each": 4}
    assert set(validations["cli"]["rejections"].values()) == {2}
    pc = json.loads((HERE / "evidence/PC_SUMMARY.json").read_text())
    assert (pc["distinct_conditions"], pc["successful_conditions"], pc["typed_refusal_conditions"]) == (36, 31, 5)
    assert pc["same_gameplay_source_hit_order_rng_and_typed_refusal"]
    r05 = json.loads((HERE / "evidence/R05_SUMMARY.json").read_text())
    assert r05["classification"] == "warning_concurrent_peak_only" and not r05["other_hard_guard_failures"]
    assert r05["event_heavy"]["allocation_retained_peak_guard_pass"]
    assert not r05["concurrent_sessions"]["allocation_retained_peak_guard_pass"]
    v17 = json.loads((HERE / "evidence/V17_RUNTIME_QUERY_CORRESPONDENCE.json").read_text())
    for moment in ("snapshot", "live"):
        assert v17[moment]["same_typed_amount"] and v17[moment]["same_trace"] and v17[moment]["runtime_query_usage_equal"]


def verify(manifest):
    for key in ("baseline", "predecessor", "review", "request", "after_archive"):
        check(manifest[key])
    for entry in manifest["artifacts"] + manifest["records"]:
        check(entry)
    record_by_path = {entry["path"]: entry for entry in manifest["records"]}
    assert len(record_by_path) == len(manifest["records"])
    assert REQUIRED_RECORDS <= set(record_by_path)
    with tarfile.open(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz") as evidence:
        members = evidence.getmembers()
        assert {m.name for m in members} == set(record_by_path)
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute() and ".." not in Path(member.name).parts
            data = evidence.extractfile(member).read()
            assert {"path": member.name, "bytes": len(data), "sha256": sha(data)} == record_by_path[member.name]
    before = archived_before()
    after = archive_contents(ROOT / manifest["after_archive"]["path"], after_files)
    assert identity(before, before_files) == manifest["source"]["before_digest"]
    assert identity(after, after_files) == manifest["source"]["after_digest"] == EXPECTED_AFTER_DIGEST
    assert identity(after, product_files) == manifest["source"]["product_after_digest"] == EXPECTED_PRODUCT_DIGEST
    baseline_rows = {entry["path"]: {"bytes": entry["bytes"], "sha256": entry["sha256"]} for entry in baseline["source"]["files"]}
    modified, added = [], []
    for entry in manifest["source"]["files"]:
        name = entry["path"]
        assert {"bytes": len(after[name]), "sha256": sha(after[name])} == entry["after"]
        assert (ROOT / name).read_bytes() == after[name], name
        if name in before:
            assert entry["before"] == baseline_rows[name]
            if before[name] != after[name]: modified.append(name)
        else:
            assert entry["before"] is None
            added.append(name)
    assert set(modified) == MODIFIED and set(added) == ADDED
    assert modified == manifest["source"]["modified_paths"] and added == manifest["source"]["added_paths"]
    assert manifest["source"]["deleted_paths"] == []
    isolated_patch(before, after, "SOURCE_DELTA.patch", before_files, after_files)
    isolated_patch(after, before, "SOURCE_RESTORE.patch", after_files, before_files)
    validate_receipts()
    return {"state": "CURRENTNESS_AND_ISOLATED_BIDIRECTIONAL_RESTORATION_VERIFIED", "before_source_count": 338, "after_source_count": 340, "product_source_count": 336, "modified_paths": modified, "added_paths": added, "record_count": len(manifest["records"])}


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in ("freeze", "verify"):
        raise SystemExit("use freeze or verify")
    if sys.argv[1] == "verify":
        manifest = json.loads((TARGET / "MANIFEST.json").read_text())
        print(json.dumps({"manifest": meta(TARGET / "MANIFEST.json"), "verification": verify(manifest)}, indent=2))
        raise SystemExit(0)
    assert not TARGET.exists(), TARGET
    validate_receipts()
    before = archived_before()
    after = {name: (ROOT / name).read_bytes() for name in after_files}
    modified = [name for name in before_files if before[name] != after[name]]
    added = [name for name in added_files if name in after]
    assert set(modified) == MODIFIED, modified
    assert set(added) == ADDED, added
    assert identity(before, before_files) == baseline["source"]["digest"]
    assert identity(after, after_files) == EXPECTED_AFTER_DIGEST
    assert identity(after, product_files) == EXPECTED_PRODUCT_DIGEST
    TARGET.mkdir(parents=True)
    for reverse, filename in ((False, "SOURCE_DELTA.patch"), (True, "SOURCE_RESTORE.patch")):
        origin, result = (after, before) if reverse else (before, after)
        delta = "".join("".join(difflib.unified_diff(origin[name].decode().splitlines(keepends=True), result[name].decode().splitlines(keepends=True), fromfile="a/" + name, tofile="b/" + name)) for name in modified)
        for name in added:
            if reverse:
                old_lines, new_lines, from_file, to_file = after[name].decode().splitlines(keepends=True), [], "a/" + name, "/dev/null"
            else:
                old_lines, new_lines, from_file, to_file = [], after[name].decode().splitlines(keepends=True), "/dev/null", "b/" + name
            delta += "".join(difflib.unified_diff(old_lines, new_lines, fromfile=from_file, tofile=to_file))
        (TARGET / filename).write_text(delta)
    with tarfile.open(TARGET / "SOURCE_AFTER.tar.gz", "w:gz") as archive:
        for name in after_files: archive.add(ROOT / name, arcname=name, recursive=False)
    local_records = {p for p in HERE.rglob("*") if p.is_file() and TARGET not in p.parents and "__pycache__" not in p.parts and p.suffix != ".pyc"}
    external_records = {ROOT / name for name in REQUIRED_RECORDS}
    record_paths = sorted(local_records | external_records)
    records = [meta(path) for path in record_paths]
    with tarfile.open(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz", "w:gz") as archive:
        for path in record_paths: archive.add(path, arcname=str(path.relative_to(ROOT)), recursive=False)
    manifest = {
        "record": "R06_WP02_D07_RUNTIME_AMOUNT_001",
        "state": "partial_implementation_self_validation_complete_review_pending",
        "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baseline": meta(BASE_PATH), "predecessor": baseline["predecessor"], "review": baseline["review"], "request": baseline["request"],
        "source": {"before_file_count": 338, "after_file_count": 340, "product_file_count": 336, "before_digest": identity(before, before_files), "after_digest": identity(after, after_files), "product_after_digest": identity(after, product_files), "modified_paths": modified, "added_paths": added, "deleted_paths": [], "files": [{"path": name, "before": ({"bytes": len(before[name]), "sha256": sha(before[name])} if name in before else None), "after": {"bytes": len(after[name]), "sha256": sha(after[name])}} for name in after_files]},
        "after_archive": meta(TARGET / "SOURCE_AFTER.tar.gz"),
        "artifacts": [], "records": records,
        "claims": {"d07_b_self_validation_complete": True, "stage_review_target_frozen": True, "independent_review_pass": False, "d07_complete": False, "wp02_complete": False, "r06_complete": False, "publication_unblocked": False, "future_amount_prediction_supported": False, "twenty_minute_test_run": False, "pc36_remeasured": True, "r05_remeasured": True, "r05_policy_warning_only": True},
    }
    # Restoration is verified before it becomes a listed immutable artifact.
    verification = verify({**manifest, "artifacts": [meta(p) for p in sorted(TARGET.iterdir())]})
    save(TARGET / "RESTORATION_VERIFICATION.json", verification)
    manifest["artifacts"] = [meta(p) for p in sorted(TARGET.iterdir())]
    save(TARGET / "MANIFEST.json", manifest)
    print(json.dumps({"manifest": meta(TARGET / "MANIFEST.json"), "verification": verification}, indent=2))
