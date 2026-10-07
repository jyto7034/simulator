"""Read-only review of frozen sources, restoration, and reusable evidence."""
from pathlib import Path
import datetime
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
CFG = json.loads((HERE / "isolate.json").read_text())
ROOT = Path(CFG["root"])
TARGET = Path(CFG["target_manifest"])
CORRECTION = TARGET.parents[2]
EXPECTED_TARGET = "abe5a96830d8090f77e63fde7c6d7ff79675f701a382437d2492e0c66e9aa60a"
sha = lambda data: hashlib.sha256(data).hexdigest()


def check(row):
    data = (ROOT / row["path"]).read_bytes()
    assert sha(data) == row["sha256"], row["path"]
    assert len(data) == row["bytes"], row["path"]


def read_archive(path, expected):
    with tarfile.open(path) as archive:
        assert len(archive.getmembers()) == len(expected)
        assert set(archive.getnames()) == set(expected)
        data = {}
        for name, digest in expected.items():
            member = archive.getmember(name)
            assert member.isfile() and not Path(name).is_absolute() and ".." not in Path(name).parts
            content = archive.extractfile(member).read()
            assert sha(content) == digest, name
            data[name] = content
        return data


assert sha(TARGET.read_bytes()) == EXPECTED_TARGET
m = json.loads(TARGET.read_text())
check(m["baseline"])
check(m["predecessor"])
for row in m["artifacts"] + m["records"]:
    check(row)
expected = {f["path"]: f["after"]["sha256"] for f in m["source"]["files"]}
assert len(expected) == len(m["source"]["files"])
for name, digest in expected.items():
    assert sha((ROOT / name).read_bytes()) == digest, name
rs = {str(p.relative_to(ROOT)) for p in (ROOT / "core_enhanced/src").rglob("*.rs")}
assert not rs - expected.keys(), sorted(rs - expected.keys())
b = json.loads((ROOT / m["baseline"]["path"]).read_text())
check(b["source"]["archive"])
before = read_archive(ROOT / b["source"]["archive"]["path"],
                      {f["path"]: f["before"]["sha256"] for f in m["source"]["files"]})
after = read_archive(ROOT / m["after_archive"]["path"], expected)
read_archive(TARGET.parent / "EVIDENCE_AND_HANDOFF.tar.gz",
             {f["path"]: f["sha256"] for f in m["records"]})
for origin, destination, patch in [(before, after, "SOURCE_DELTA.patch"),
                                   (after, before, "SOURCE_RESTORE.patch")]:
    with tempfile.TemporaryDirectory(prefix="r06-eep-review002-restore-", dir="/tmp") as tmp:
        for name, content in origin.items():
            p = Path(tmp) / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(content)
        subprocess.run(["patch", "--batch", "--fuzz=0", "-p1", "-i", str(TARGET.parent / patch)],
                       cwd=tmp, check=True, capture_output=True)
        assert all((Path(tmp) / name).read_bytes() == content for name, content in destination.items())

e = CORRECTION / "evidence"
receipts = {}
for name in ["all_tests_002", "review_probes_002", "check_final", "clippy_final", "changed_format", "pc_001", "r05_guard_001"]:
    d = json.loads((e / (name + ".json")).read_text())
    assert d["exit_code"] == 0
    for side in ["source_before", "source_after"]:
        assert len(d[side]) == len(expected)
        assert {f["path"]: f["sha256"] for f in d[side]} == expected
    log = (e / (name + ".log")).read_text()
    results = re.findall(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", log)
    assert all(row[0] == "ok" and row[2] == "0" for row in results)
    receipts[name] = dict(command=d.get("command"), cwd=d["cwd"], start=d["start"], end=d["end"],
                          exit_code=d["exit_code"], source_identity_matches=True, log_sha256=sha(log.encode()),
                          passed=sum(int(row[1]) for row in results), test_summaries=results,
                          provenance="implementation evidence reused; not executed by this review")
assert receipts["all_tests_002"]["passed"] == 1487
assert receipts["review_probes_002"]["passed"] == 7
identity = json.loads((e / "isolate_final_identity.json").read_text())
assert {f["path"]: f["sha256"] for f in identity["source"]} == expected
for name, digest in identity["hooks"].items():
    assert sha((e / name).read_bytes()) == digest
original = (HERE.parent / "wp02_effect_execution_prefix_001_review_001/probes.rs").read_text()
reused = (e / "review_probes.rs").read_text()
assert reused[reused.index("#[test]"):].startswith(original[original.index("#[test]"):])

old_log = HERE.parent / "wp02_eep_f01_f03_correction_001_review_001/product_all_targets_002.log"
parts = re.split(r"---- (.*?) stdout ----\n", old_log.read_text())
readonly, other = [], []
for i in range(1, len(parts), 2):
    (readonly if "Read-only file system" in parts[i + 1].split("failures:")[0] else other).append(parts[i])
assert len(readonly) == 64 and len(other) == 1

result = dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(), target_sha256=EXPECTED_TARGET,
              sources_checked=len(expected), source_membership="all current core_enhanced/src/*.rs recursively included",
              artifacts_checked=len(m["artifacts"]), records_checked=len(m["records"]),
              archive_members_match=True, bidirectional_restoration=True, source_unchanged=True,
              evidence_reuse=receipts, original_five_probe_bodies_unchanged=True,
              isolated_test_hook_digests_match=True,
              previous_failed_run=dict(readonly_failures=readonly, other_failures=other,
                                       treated_as_success=False),
              local_toolchain=dict(cargo=shutil.which("cargo"), rustc=shutil.which("rustc")),
              rust_tests_executed_this_review=0)
out = HERE / (sys.argv[1] + ".json")
with out.open("x") as f:
    json.dump(result, f, indent=2, ensure_ascii=False)
    f.write("\n")
print(json.dumps(dict(report=str(out), sources=len(expected), records=len(m["records"]),
                      bidirectional_restoration=True, reusable_product_tests=1487,
                      reusable_probes=7, rust_tests_executed_this_review=0)))
