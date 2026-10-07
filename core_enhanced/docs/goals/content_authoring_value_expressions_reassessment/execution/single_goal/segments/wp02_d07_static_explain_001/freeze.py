"""Freeze D07-A and verify current bytes plus isolated reversible deltas."""

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
TARGET = HERE / "target/d07_static_explain_001"
BASE_PATH = HERE / "baseline/entry_001/MANIFEST.json"
MODIFIED = {
    "core_enhanced/src/authoring/mod.rs",
    "core_enhanced/src/authoring/query.rs",
    "core_enhanced/src/authoring/tests.rs",
    "core_enhanced/src/authoring/trial.rs",
    "core_enhanced/src/game/value_expr/catalog.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
}
ADDED = {"core_enhanced/src/authoring/amount.rs"}
EXPECTED_AFTER_DIGEST = "5279b8333c6be8f90992ac94a47f09beb083bf82bfcdf02ebbe3e0d179c8f530"
EXPECTED_PRODUCT_DIGEST = "17e41fa7625d001242fcfcec65834ab533fea43ea74e3f89f7ce78f3bae5acfd"
REQUIRED_RECORDS = {
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/CODE_REVIEW_003.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/handoffs/WP02_D07_IMPLEMENTATION_REQUEST_20260927.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/handoffs/WP02_D07_CODE_EVIDENCE_20260927.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/corrections/wp02_d04_owner_f01_001/target/"
    "d04_owner_f01_correction_001/MANIFEST.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d04_owner_f01_correction_001_review_001/REVIEW.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/README.ko.md",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/RESULT.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/ALL_TARGETS.log",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/ALL_TARGETS.time",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/SANDBOX_ATTEMPT.json",
    "core_enhanced/docs/goals/content_authoring_value_expressions_reassessment/"
    "execution/single_goal/reviews/wp02_d07_test_timing_001/SANDBOX_ATTEMPT.log",
}
FINAL_RECEIPTS = {
    "format_final_004.json": (0, 0, 0),
    "check_all_final_001.json": (0, 0, 0),
    "authoring_focused_final_001.json": (0, 110, 0),
    "value_expr_final_001.json": (0, 88, 0),
    "all_targets_parallel_final_001.json": (0, 1533, 0),
    "clippy_final_001.json": (0, 0, 0),
    "package_current_cli_fixture_001.json": (0, 0, 0),
    "cli_effect_explain_current_final_001.json": (0, 0, 0),
    "trial_delegation_002.json": (0, 1, 0),
    "probe_static_explain_usage_final_004.json": (0, 0, 0),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def meta(path):
    data = path.read_bytes()
    return {
        "path": str(path.relative_to(ROOT)),
        "bytes": len(data),
        "sha256": sha(data),
    }


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write("\n")


def check(entry):
    assert meta(ROOT / entry["path"]) == entry, entry["path"]


baseline = json.loads(BASE_PATH.read_text())
for entry in (
    baseline["predecessor"],
    baseline["review"],
    baseline["request"],
    baseline["archive"],
    baseline["git_status"],
):
    check(entry)
before_files = [entry["path"] for entry in baseline["source"]["files"]]
added_files = baseline["source"]["absent_before"]
assert len(before_files) == len(set(before_files)) == 337
assert set(added_files) == ADDED
after_files = before_files + added_files
product_files = before_files[: baseline["source"]["product_file_count"]] + added_files


def archived_before():
    expected = set(before_files)
    contents = {}
    with tarfile.open(ROOT / baseline["archive"]["path"]) as archive:
        members = archive.getmembers()
        assert {member.name for member in members} == expected
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert ".." not in Path(member.name).parts
            source = archive.extractfile(member)
            assert source is not None
            contents[member.name] = source.read()
    return contents


def archived_after(path):
    contents = {}
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        assert {member.name for member in members} == set(after_files)
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert ".." not in Path(member.name).parts
            source = archive.extractfile(member)
            assert source is not None
            contents[member.name] = source.read()
    return contents


def identity(contents, selected):
    rows = [(name, len(contents[name]), sha(contents[name])) for name in selected]
    return sha(json.dumps(rows, separators=(",", ":")).encode())


def isolated_patch(origin, destination, patch_name, origin_files, destination_files):
    with tempfile.TemporaryDirectory(prefix="wp02-d07-static-explain-restore-") as temp:
        root = Path(temp)
        for name in origin_files:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(origin[name])
        subprocess.run(
            [
                "patch",
                "--batch",
                "--fuzz=0",
                "-p1",
                "-i",
                str(TARGET / patch_name),
            ],
            cwd=root,
            check=True,
            capture_output=True,
        )
        for name in destination_files:
            assert (root / name).read_bytes() == destination[name], name
        for name in set(origin_files) - set(destination_files):
            assert not (root / name).exists(), name


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
        assert {member.name for member in members} == set(record_by_path)
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert ".." not in Path(member.name).parts
            source = evidence.extractfile(member)
            assert source is not None
            data = source.read()
            assert {
                "path": member.name,
                "bytes": len(data),
                "sha256": sha(data),
            } == record_by_path[member.name]

    before = archived_before()
    after = archived_after(ROOT / manifest["after_archive"]["path"])
    assert identity(before, before_files) == manifest["source"]["before_digest"]
    assert identity(after, after_files) == manifest["source"]["after_digest"]
    assert identity(after, product_files) == manifest["source"]["product_after_digest"]
    assert [entry["path"] for entry in manifest["source"]["files"]] == after_files

    baseline_rows = {
        entry["path"]: {"bytes": entry["bytes"], "sha256": entry["sha256"]}
        for entry in baseline["source"]["files"]
    }
    modified = []
    added = []
    for entry in manifest["source"]["files"]:
        name = entry["path"]
        assert {"bytes": len(after[name]), "sha256": sha(after[name])} == entry["after"]
        assert (ROOT / name).read_bytes() == after[name], name
        if name in before:
            assert entry["before"] == baseline_rows[name]
            assert {"bytes": len(before[name]), "sha256": sha(before[name])} == entry["before"]
            if before[name] != after[name]:
                modified.append(name)
        else:
            assert entry["before"] is None
            added.append(name)
    assert set(modified) == MODIFIED
    assert set(added) == ADDED
    assert modified == manifest["source"]["modified_paths"]
    assert added == manifest["source"]["added_paths"]
    assert manifest["source"]["deleted_paths"] == []

    isolated_patch(before, after, "SOURCE_DELTA.patch", before_files, after_files)
    isolated_patch(after, before, "SOURCE_RESTORE.patch", after_files, before_files)

    return {
        "state": "CURRENTNESS_AND_ISOLATED_BIDIRECTIONAL_RESTORATION_VERIFIED",
        "before_source_count": len(before_files),
        "after_source_count": len(after_files),
        "product_source_count": len(product_files),
        "modified_paths": modified,
        "added_paths": added,
        "record_count": len(manifest["records"]),
    }


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in ("freeze", "verify"):
        raise SystemExit("use freeze or verify")
    if sys.argv[1] == "verify":
        manifest = json.loads((TARGET / "MANIFEST.json").read_text())
        print(
            json.dumps(
                {"manifest": meta(TARGET / "MANIFEST.json"), "verification": verify(manifest)},
                indent=2,
            )
        )
        raise SystemExit(0)

    assert not TARGET.exists(), TARGET
    before = archived_before()
    after = {name: (ROOT / name).read_bytes() for name in after_files}
    modified = [name for name in before_files if before[name] != after[name]]
    added = [name for name in added_files if name in after]
    assert set(modified) == MODIFIED, modified
    assert set(added) == ADDED, added
    assert identity(before, before_files) == baseline["source"]["digest"]
    assert identity(after, after_files) == EXPECTED_AFTER_DIGEST
    assert identity(after, product_files) == EXPECTED_PRODUCT_DIGEST

    for name, expected in FINAL_RECEIPTS.items():
        receipt = json.loads((HERE / "evidence" / name).read_text())
        assert (
            receipt["exit_code"],
            receipt["test_passed"],
            receipt["test_failed"],
        ) == expected, name
        assert receipt["source_before"]["digest"] == EXPECTED_AFTER_DIGEST, name
        assert receipt["source_after"]["digest"] == EXPECTED_AFTER_DIGEST, name

    TARGET.mkdir(parents=True)
    for reverse, filename in (
        (False, "SOURCE_DELTA.patch"),
        (True, "SOURCE_RESTORE.patch"),
    ):
        origin, result = (after, before) if reverse else (before, after)
        existing = modified
        delta = "".join(
            "".join(
                difflib.unified_diff(
                    origin[name].decode().splitlines(keepends=True),
                    result[name].decode().splitlines(keepends=True),
                    fromfile="a/" + name,
                    tofile="b/" + name,
                )
            )
            for name in existing
        )
        for name in added:
            if reverse:
                old_lines = after[name].decode().splitlines(keepends=True)
                new_lines = []
                from_file, to_file = "a/" + name, "/dev/null"
            else:
                old_lines = []
                new_lines = after[name].decode().splitlines(keepends=True)
                from_file, to_file = "/dev/null", "b/" + name
            delta += "".join(
                difflib.unified_diff(
                    old_lines,
                    new_lines,
                    fromfile=from_file,
                    tofile=to_file,
                )
            )
        (TARGET / filename).write_text(delta)

    with tarfile.open(TARGET / "SOURCE_AFTER.tar.gz", "w:gz") as archive:
        for name in after_files:
            archive.add(ROOT / name, arcname=name, recursive=False)

    local_record_paths = {
        path
        for path in HERE.rglob("*")
        if path.is_file()
        and TARGET not in path.parents
        and "__pycache__" not in path.parts
        and path.suffix != ".pyc"
    }
    external_record_paths = {ROOT / name for name in REQUIRED_RECORDS}
    record_paths = sorted(local_record_paths | external_record_paths)
    records = [meta(path) for path in record_paths]
    with tarfile.open(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz", "w:gz") as archive:
        for path in record_paths:
            archive.add(path, arcname=str(path.relative_to(ROOT)), recursive=False)

    manifest = {
        "record": "R06_WP02_D07_STATIC_EXPLAIN_001",
        "state": "partial_implementation_self_validation_complete_review_pending",
        "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baseline": meta(BASE_PATH),
        "predecessor": baseline["predecessor"],
        "review": baseline["review"],
        "request": baseline["request"],
        "source": {
            "before_file_count": len(before_files),
            "after_file_count": len(after_files),
            "product_file_count": len(product_files),
            "before_digest": identity(before, before_files),
            "after_digest": identity(after, after_files),
            "product_after_digest": identity(after, product_files),
            "modified_paths": modified,
            "added_paths": added,
            "deleted_paths": [],
            "files": [
                {
                    "path": name,
                    "before": (
                        {"bytes": len(before[name]), "sha256": sha(before[name])}
                        if name in before
                        else None
                    ),
                    "after": {"bytes": len(after[name]), "sha256": sha(after[name])},
                }
                for name in after_files
            ],
        },
        "after_archive": meta(TARGET / "SOURCE_AFTER.tar.gz"),
        "artifacts": [meta(path) for path in sorted(TARGET.iterdir())],
        "records": records,
        "claims": {
            "d07_a_self_validation_complete": True,
            "independent_review_pass": False,
            "d07_complete": False,
            "wp02_complete": False,
            "r06_complete": False,
            "publication_unblocked": False,
            "actual_frame_numeric_preview": False,
            "twenty_minute_test_run": False,
            "pc36_r05_remeasured": False,
        },
    }
    verification = verify(manifest)
    save(TARGET / "RESTORATION_VERIFICATION.json", verification)
    manifest["artifacts"].append(meta(TARGET / "RESTORATION_VERIFICATION.json"))
    save(TARGET / "MANIFEST.json", manifest)
    print(
        json.dumps(
            {"manifest": meta(TARGET / "MANIFEST.json"), "verification": verification},
            indent=2,
        )
    )
