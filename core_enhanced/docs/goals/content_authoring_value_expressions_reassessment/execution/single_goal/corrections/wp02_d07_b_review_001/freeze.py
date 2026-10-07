"""D07-B F01 checkpoint: preserve/replay exact before and after bytes.

Reuses the preceding checkpoint's tar + unified diff + isolated patch method.
No product mutation is performed by verify.
"""

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
BASE = HERE / "baseline/entry_001"
TARGET = HERE / "target/d07_b_f01_correction_001"
BASELINE = json.loads((BASE / "MANIFEST.json").read_text())
BEFORE = [row["path"] for row in BASELINE["source"]["files"]]
AFTER = BEFORE + BASELINE["source"]["absent_before"]
MODIFIED = ["core_enhanced/src/bin/author_content.rs"]
ADDED = ["core_enhanced/tests/author_content_cli_arguments.rs"]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def meta(path):
    data = path.read_bytes()
    return {"path": str(path.relative_to(ROOT)), "bytes": len(data), "sha256": sha(data)}


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write("\n")


def read_archive(path, expected):
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        assert len(members) == len(expected)
        assert {m.name for m in members} == set(expected)
        result = {}
        for member in members:
            assert member.isfile() and not Path(member.name).is_absolute()
            assert ".." not in Path(member.name).parts
            result[member.name] = archive.extractfile(member).read()
        return result


def identity(contents, names):
    rows = [(p, len(contents[p]), sha(contents[p])) for p in names]
    return sha(json.dumps(rows, separators=(",", ":")).encode())


def restore(origin, destination, patch):
    with tempfile.TemporaryDirectory(prefix="d07-b-f01-restore-") as directory:
        root = Path(directory)
        for name, data in origin.items():
            p = root / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
        subprocess.run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(patch)],
                       cwd=root, check=True, capture_output=True)
        actual = {str(p.relative_to(root)) for p in root.rglob("*") if p.is_file()}
        assert actual == set(destination)
        for name, data in destination.items():
            assert (root / name).read_bytes() == data, name


def verify(manifest):
    for item in manifest["artifacts"] + manifest["records"]:
        assert meta(ROOT / item["path"]) == item, item["path"]
    before = read_archive(BASE / "SOURCE_BEFORE.tar.gz", BEFORE)
    after = read_archive(TARGET / "SOURCE_AFTER.tar.gz", AFTER)
    assert identity(before, BEFORE) == BASELINE["source"]["digest"]
    assert identity(after, AFTER) == manifest["source"]["after_digest"]
    for row in BASELINE["source"]["files"]:
        assert len(before[row["path"]]) == row["bytes"]
        assert sha(before[row["path"]]) == row["sha256"]
    assert [p for p in BEFORE if before[p] != after[p]] == MODIFIED
    assert set(after) - set(before) == set(ADDED)
    for p in AFTER:
        assert (ROOT / p).read_bytes() == after[p], p
    for row in manifest["source"]["files"]:
        assert row["after"] == {"bytes": len(after[row["path"]]), "sha256": sha(after[row["path"]])}
        old = before.get(row["path"])
        assert row["before"] == (None if old is None else {"bytes": len(old), "sha256": sha(old)})
    # Detect new/missing product source/test members, not only changes to listed files.
    membership = {str(p.relative_to(ROOT)) for folder in ("core_enhanced/src", "core_enhanced/tests")
                  for p in (ROOT / folder).rglob("*") if p.is_file()}
    assert membership == {p for p in AFTER if p.startswith(("core_enhanced/src/", "core_enhanced/tests/"))}
    record_names = [row["path"] for row in manifest["records"]]
    records = read_archive(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz", record_names)
    for row in manifest["records"]:
        assert len(records[row["path"]]) == row["bytes"] and sha(records[row["path"]]) == row["sha256"]
    restore(before, after, TARGET / "SOURCE_DELTA.patch")
    restore(after, before, TARGET / "SOURCE_RESTORE.patch")
    final = json.loads((HERE / "evidence/FINAL_VALIDATION.json").read_text())
    assert final["source_digest"] == manifest["source"]["after_digest"]
    for name in final["fresh_validations"]:
        receipt = json.loads((HERE / "evidence" / (name + ".json")).read_text())
        assert receipt["exit_code"] == 0
        assert receipt["source_before"]["digest"] == receipt["source_after"]["digest"] == final["source_digest"]
        assert sha((HERE / "evidence" / (name + ".log")).read_bytes()) == receipt["log_sha256"]
    test = json.loads((HERE / "evidence/f01_after_regression_001.json").read_text())
    assert (test["test_passed"], test["test_failed"]) == (4, 0)
    cli = json.loads((HERE / "evidence/cli_matrix_001/SUMMARY.json").read_text())
    assert cli["successful_commands"] == 8 and cli["amount_output_matches_predecessor"]
    return {"state": "CURRENTNESS_AND_ISOLATED_BIDIRECTIONAL_RESTORATION_VERIFIED",
            "before_source_count": len(BEFORE), "after_source_count": len(AFTER),
            "modified_paths": MODIFIED, "added_paths": ADDED,
            "source_test_membership_verified": True, "record_count": len(record_names)}


def freeze():
    assert not TARGET.exists()
    before = read_archive(BASE / "SOURCE_BEFORE.tar.gz", BEFORE)
    after = {name: (ROOT / name).read_bytes() for name in AFTER}
    assert [p for p in BEFORE if before[p] != after[p]] == MODIFIED
    assert BASELINE["source"]["absent_before"] == ADDED
    TARGET.mkdir(parents=True)
    for reverse, filename in [(False, "SOURCE_DELTA.patch"), (True, "SOURCE_RESTORE.patch")]:
        origin, result = (after, before) if reverse else (before, after)
        delta = []
        for name in MODIFIED + ADDED:
            delta.extend(difflib.unified_diff(
                origin.get(name, b"").decode().splitlines(keepends=True),
                result.get(name, b"").decode().splitlines(keepends=True),
                fromfile="a/" + name if name in origin else "/dev/null",
                tofile="b/" + name if name in result else "/dev/null"))
        (TARGET / filename).write_text("".join(delta))
    with tarfile.open(TARGET / "SOURCE_AFTER.tar.gz", "w:gz") as archive:
        for p in AFTER:
            archive.add(ROOT / p, arcname=p, recursive=False)
    predecessor_path = ROOT / BASELINE["predecessor"]["path"]
    predecessor = json.loads(predecessor_path.read_text())
    assert sha(predecessor_path.read_bytes()) == BASELINE["predecessor"]["sha256"]
    # Preserve exact reused evidence, inputs, instructions and earlier findings.
    external = {ROOT / row["path"] for row in predecessor["records"]}
    external.add(predecessor_path)
    base = HERE.parents[1]
    external.update((base / "handoffs").glob("WP02_D07_B_CORRECTION_*20260928*"))
    external.update(p for p in (base / "reviews/wp02_d07_runtime_amount_001_review_001").iterdir() if p.is_file())
    local = {p for p in HERE.rglob("*") if p.is_file() and TARGET not in p.parents
             and p not in {HERE / "FREEZE_RESULT.json", HERE / "VERIFY_RESULT.json"}
             and "__pycache__" not in p.parts and p.suffix != ".pyc"}
    paths = sorted(local | external)
    records = [meta(p) for p in paths]
    with tarfile.open(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz", "w:gz") as archive:
        for p in paths:
            archive.add(p, arcname=str(p.relative_to(ROOT)), recursive=False)
    manifest = {
        "record": "R06_WP02_D07_B_F01_CORRECTION_001",
        "state": "partial_correction_self_validation_complete_review_pending",
        "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "predecessor": meta(predecessor_path),
        "source": {"before_file_count": len(BEFORE), "after_file_count": len(AFTER),
                   "before_digest": identity(before, BEFORE), "after_digest": identity(after, AFTER),
                   "modified_paths": MODIFIED, "added_paths": ADDED, "deleted_paths": [],
                   "files": [{"path": p,
                              "before": None if p not in before else {"bytes": len(before[p]), "sha256": sha(before[p])},
                              "after": {"bytes": len(after[p]), "sha256": sha(after[p])}} for p in AFTER]},
        "records": records, "artifacts": [meta(p) for p in sorted(TARGET.iterdir())],
        "claims": {"f01_self_validated": True, "f02_complete": False, "f03_complete": False,
                   "d07_b_complete": False, "independent_review_pass": False,
                   "r06_complete": False, "production_publication_unblocked": False}}
    verification = verify(manifest)
    save(TARGET / "RESTORATION_VERIFICATION.json", verification)
    manifest["artifacts"] = [meta(p) for p in sorted(TARGET.iterdir())]
    save(TARGET / "MANIFEST.json", manifest)
    return manifest, verification


if __name__ == "__main__":
    if sys.argv[1:] == ["freeze"]:
        manifest, result = freeze()
    elif sys.argv[1:] == ["verify"]:
        manifest = json.loads((TARGET / "MANIFEST.json").read_text())
        result = verify(manifest)
    else:
        raise SystemExit("use freeze or verify")
    print(json.dumps({"manifest": meta(TARGET / "MANIFEST.json"), "verification": result}, indent=2))
