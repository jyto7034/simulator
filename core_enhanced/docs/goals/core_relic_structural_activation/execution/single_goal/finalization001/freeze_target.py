#!/usr/bin/env python3
"""Bind U09 evidence and execute exact restoration only in temporary trees.

Assigned Sol executor only. This is a thin U09 binding of the reviewed U08
rows/archive/extract/pack/git-diff/git-apply mechanics, not U08's main workflow.
An authored script proves no restoration. A frozen target proves no independent
Complete/High/open0 verdict. Never run through a collector writing into scope.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

if not __debug__:
    raise RuntimeError("Restoration checks require Python without -O")
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / "Cargo.toml").is_file()
            and (p / "core_enhanced/src/lib.rs").is_file()
            and (p / "game_server/Cargo.toml").is_file())
E = HERE.parent
GOAL = E.parent.parent
PRIOR = ROOT / "core_enhanced/docs/goals/core_relic_numeric_variants/execution/single_goal/target/u08_current_013"
PRIOR_AUDIT = ROOT / "core_enhanced/docs/goals/core_relic_numeric_variants/post_audit/u08_current_013_pass002"
BASELINE = E / "baseline/PREPARATION_ENTRY_001.json"
SCANNER = E / "test_parts/runtime_validation001/performance001/run.py"
DOCS = {
    "core_enhanced/docs/core_runtime_contract.md": E / "product_parts/bundle_f001/u09_dev_profile001/core_runtime_contract.before.md",
    "core_enhanced/docs/skills/skill_authoring_model.ko.md": E / "product_parts/bundle_f001/u09_canonical_docs001/skill_authoring_model.before.md",
    "core_enhanced/docs/game_rules/relics.md": E / "product_parts/bundle_f001/u09_canonical_docs001/relics.before.md",
}
FUTURE = {"frozen_restore", "independent_completion"}


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write(path, value):
    with Path(path).open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def sha(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def digest(value):
    # Preserve the established source-manifest serializer and row order.
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def resolve(name):
    path = Path(name)
    path = (path if path.is_absolute() else ROOT / path).resolve()
    assert path.is_relative_to(ROOT), path
    return path


def ref(path):
    path = resolve(path)
    assert path.is_file() and not path.is_symlink(), path
    return {"path": str(path.relative_to(ROOT)), "sha256": sha(path), "bytes": path.stat().st_size}


def check_ref(row):
    assert ref(row["path"]) == {key: row[key] for key in ("path", "sha256", "bytes")}, row["path"]
    return resolve(row["path"])


def safe_rows(entries):
    assert entries and len({r["path"] for r in entries}) == len(entries)
    for row in entries:
        path = Path(row["path"])
        assert not path.is_absolute() and ".." not in path.parts and str(path) == row["path"]
        assert set(row) == {"path", "sha256", "bytes"} and row["bytes"] >= 0


def rows(root):
    return [{"path": str(p.relative_to(root)), "sha256": sha(p), "bytes": p.stat().st_size}
            for p in sorted(root.rglob("*")) if p.is_file()]


def archive_exact(path, expected_rows):
    safe_rows(expected_rows)
    expected = {r["path"]: r for r in expected_rows}
    seen = set()
    with tarfile.open(path) as archive:
        for member in archive:
            assert member.isfile() and member.name in expected and member.name not in seen, member.name
            row = expected[member.name]
            assert member.size == row["bytes"]
            h = hashlib.sha256()
            with archive.extractfile(member) as stream:
                while chunk := stream.read(1024 * 1024):
                    h.update(chunk)
            assert h.hexdigest() == row["sha256"], member.name
            seen.add(member.name)
    assert seen == set(expected)


def extract(path, destination, expected):
    archive_exact(path, expected)
    with tarfile.open(path) as archive:
        for member in archive:
            output = destination / member.name
            assert output.resolve().is_relative_to(destination.resolve())
            output.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, output.open("xb") as target:
                shutil.copyfileobj(source, target)


def pack(destination, base, entries):
    with tarfile.open(destination, "x:gz") as archive:
        for row in entries:
            path = base / row["path"]
            assert sha(path) == row["sha256"] and path.stat().st_size == row["bytes"]
            archive.add(path, arcname=row["path"], recursive=False)
    archive_exact(destination, entries)


def pointer(value, path):
    for segment in path.strip("/").split("/") if path else []:
        segment = segment.replace("~1", "/").replace("~0", "~")
        value = value[int(segment)] if isinstance(value, list) else value[segment]
    return value


def refs_in(value):
    if isinstance(value, dict):
        if {"path", "sha256", "bytes"} <= value.keys():
            yield {key: value[key] for key in ("path", "sha256", "bytes")}
        for child in value.values():
            yield from refs_in(child)
    elif isinstance(value, list):
        for child in value:
            yield from refs_in(child)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-manifest", required=True)
    parser.add_argument("--generated", required=True)
    parser.add_argument("--handoff", required=True)
    parser.add_argument("--target", required=True)
    args = parser.parse_args()
    source_path, generated, handoff_path, target = map(resolve,
        (args.source_manifest, args.generated, args.handoff, args.target))
    assert target.parent == E / "target" and not target.exists(), "Fresh U09 target only"
    assert generated.is_relative_to(HERE) and handoff_path.is_relative_to(E)
    # An active parent logger inside RECORDS would make the target self-changing.
    pid = os.getppid()
    while pid > 1:
        directory = Path("/proc") / str(pid)
        try:
            command = (directory / "cmdline").read_bytes().split(b"\0")
            assert not any(x.endswith(b"/run.py") for x in command), "Run freeze directly, unwrapped"
            status = (directory / "status").read_text()
            pid = int(next(x.split(":", 1)[1] for x in status.splitlines() if x.startswith("PPid:")))
        except (FileNotFoundError, ProcessLookupError):
            break
    spec = importlib.util.spec_from_file_location("u09_frozen_identity", SCANNER)
    scanner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(scanner)
    current = read(source_path)
    safe_rows(current["files"])
    assert digest(current["files"]) == current["sha256"]
    assert scanner.identity() == current, "Fixed source identity drift, including membership"
    handoff = read(handoff_path)
    assert handoff["source_sha256"] == current["sha256"]
    assert handoff["active_input_writers"] == handoff["active_functional_runners"] == handoff["active_performance_runners"] == 0
    assert handoff["reviewer"].strip() and handoff["semantic_currentness_review"].strip()
    outputs = {name: read(generated / name) for name in
               ("EVIDENCE_SELECTION.json", "PREDECESSOR_IMPACT.json", "COVERAGE.json")}
    assert {r["path"] for r in handoff["generated"]} == {str((generated / n).relative_to(ROOT)) for n in outputs}
    for row in handoff["generated"] + handoff["documents"]:
        check_ref(row)
    assert {str((E / name).relative_to(ROOT)) for name in
            ("PLAN.md", "EXPERIMENT_NOTES.md", "EXPERIMENTS.md", "STATE_INVENTORY.md")} <= {r["path"] for r in handoff["documents"]}
    for value in outputs.values():
        assert value["source_sha256"] == current["sha256"] and value["source"] == ref(source_path)
    selection, impact, coverage = outputs.values()
    pending = {key for key, value in selection["evidence"].items() if value["status"] == "pending"}
    assert pending <= FUTURE | {"predecessor_impact"}, pending
    assert all(set(row["pending_or_unresolved_evidence"]) <= FUTURE for row in impact["rows"])
    assert len(impact["rows"]) == 254 and len(coverage["acceptances"]) == 34
    assert len(coverage["integration_seams"]) == 17 and len(coverage["required_dimensions"]) == 10
    baseline = read(BASELINE)
    original = baseline["source"]
    safe_rows(original["files"])
    assert len(original["files"]) == 438 and digest(original["files"]) == original["sha256"]
    baseline_archive = check_ref(baseline["reused_preimage"]["archive"])
    archive_exact(baseline_archive, original["files"])
    before_by_path = {r["path"]: r for r in original["files"]}
    doc_preimages = []
    for destination, preimage in DOCS.items():
        observed = ref(preimage)
        row = dict(observed, path=destination)
        assert destination not in before_by_path or before_by_path[destination] == row
        before_by_path[destination] = row
        doc_preimages.append({"destination": destination, "preimage": observed, "current": ref(destination)})
    assert "config/runtime_safety_authoring_dev.ron" not in before_by_path
    assert "config/runtime_safety_authoring_dev.ron" in {r["path"] for r in current["files"]}
    before = [before_by_path[p] for p in sorted(before_by_path, key=Path)]
    scope = set(before_by_path) | {r["path"] for r in current["files"]}
    after = [ref(p) for p in sorted(scope, key=Path) if (ROOT / p).is_file()]
    external, references = {}, {}

    def add_reference(row):
        check_ref(row)
        table = external if row["path"].endswith(".tar.gz") else references
        entry = {"archive": row, "verification": "historical_blob_bytes_bound; not a restore preimage"} if table is external else row
        if row["path"] in table:
            previous = table[row["path"]]
            assert (previous["archive"] if table is external else previous) == row
        else:
            table[row["path"]] = entry

    # Retain the immediate predecessor and its existing exact external bindings.
    prior_closure = read(PRIOR_AUDIT / "AUDIT_CLOSURE.json")
    closure_refs = {}
    for row in refs_in(prior_closure):
        assert row["path"] not in closure_refs or closure_refs[row["path"]] == row
        closure_refs[row["path"]] = row
    archived_closure_refs = {}
    prior_specs = [("SOURCE_BEFORE.tar.gz", "MANIFEST.json", "/before_files"),
                   ("SOURCE_AFTER.tar.gz", "MANIFEST.json", "/source_files"),
                   ("RECORDS.tar.gz", "RECORDS.json", ""), ("REFERENCES.tar.gz", "REFERENCES.json", "")]
    inherited = read(PRIOR / "EXTERNAL_ARCHIVES.json")
    for item in inherited + [{"archive": ref(PRIOR / archive), "members_manifest": ref(PRIOR / manifest), "members_pointer": field}
                             for archive, manifest, field in prior_specs]:
        archive = check_ref(item["archive"])
        manifest = check_ref(item["members_manifest"])
        entries = pointer(read(manifest), item["members_pointer"])
        archive_exact(archive, entries)
        external[item["archive"]["path"]] = dict(item, exact_member_bytes_verified=True, member_count=len(entries))
        add_reference(item["members_manifest"])
        for row in entries:
            if row == closure_refs.get(row["path"]):
                archived_closure_refs.setdefault(row["path"], []).append(item["archive"])
    # The predecessor target was frozen before its independent audit. Preserve
    # that subsequent closure explicitly; a pre-audit archive cannot contain it.
    assert prior_closure["target"] == ref(PRIOR / "MANIFEST.json")
    assert prior_closure["product_source_sha256"] == original["sha256"]
    assert prior_closure["product_source_files"] == len(original["files"]) == 438
    assert prior_closure["verdict"] == "Complete" and prior_closure["confidence"] == "High"
    assert prior_closure["open_findings_count"] == 0 and prior_closure["open_findings"] == []
    assert prior_closure["complete_high_open0"] is True
    assert prior_closure["independence"]["participated_in_plan_design_product_or_test_authoring"] is False
    assert prior_closure["independence"]["participated_in_supplemental_probe_or_verifier_authoring"] is False
    for row in closure_refs.values():
        # Preserve the closure's expected digest, not just a newly observed
        # digest. Config/correction refs already present as exact archive
        # members remain there; only otherwise missing refs are copied anew.
        check_ref(row)
        if row["path"] not in archived_closure_refs:
            add_reference(row)
    prior_audit_files = [path for path in sorted(PRIOR_AUDIT.iterdir()) if path.is_file()]
    assert len(prior_audit_files) == 16, "Exact prior pass002 closure/supporting file membership"
    for path in prior_audit_files:
        add_reference(ref(path))
    for path in (BASELINE, source_path, handoff_path, PRIOR / "MANIFEST.json", PRIOR / "EXTERNAL_ARCHIVES.json"):
        add_reference(ref(path))
    add_reference(baseline["reused_preimage"]["archive"])
    external[str(baseline_archive.relative_to(ROOT))].update(exact_member_bytes_verified=True,
        members_manifest=ref(BASELINE), members_pointer="/source/files", member_count=438)
    for value in list(outputs.values()) + [handoff]:
        for row in refs_in(value):
            add_reference(row)
    for row in doc_preimages:
        add_reference(row["preimage"])
        add_reference(row["current"])
    # All U09 history is preserved, including failed receipts and authored tools.
    # Existing archives remain external; neither prior nor current archive bytes
    # are recursively copied into the new RECORDS archive.
    records = []
    for path in sorted(GOAL.rglob("*")):
        relative = path.relative_to(GOAL)
        if not path.is_file() or {"target", "post_audit", "__pycache__"} & set(relative.parts):
            continue
        row = ref(path)
        if path.name.endswith(".tar.gz"):
            add_reference(row)
        else:
            records.append(row)
    target.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix="u09-isolated-restore-") as directory:
        root = Path(directory)
        b, a = root / "before", root / "after"
        b.mkdir()
        a.mkdir()
        extract(baseline_archive, b, original["files"])
        for destination, preimage in DOCS.items():
            path = b / destination
            path.parent.mkdir(parents=True, exist_ok=True)
            if path.exists():
                assert sha(path) == sha(preimage)
            else:
                shutil.copyfile(preimage, path)
        assert rows(b) == before
        for row in after:
            destination = a / row["path"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / row["path"], destination)
        assert rows(a) == after
        pack(target / "SOURCE_BEFORE.tar.gz", b, before)
        pack(target / "SOURCE_AFTER.tar.gz", a, after)
        for label, left, right in (("FORWARD", "before", "after"), ("REVERSE", "after", "before")):
            result = subprocess.run(["git", "diff", "--no-index", "--binary", "--src-prefix=a/", "--dst-prefix=b/", "--", left, right], cwd=root, capture_output=True)
            assert result.returncode in (0, 1), result.stderr.decode(errors="replace")
            with (target / (label + ".patch")).open("xb") as stream:
                stream.write(result.stdout)
        for label, expected in (("FORWARD", after), ("REVERSE", before)):
            patch = target / (label + ".patch")
            assert patch.stat().st_size > 0
            result = subprocess.run(["git", "apply", "--binary", "-p2", "--whitespace=nowarn", str(patch)], cwd=b, capture_output=True)
            with (target / ("RESTORE_" + label + ".log")).open("xb") as stream:
                stream.write(result.stdout + result.stderr)
            assert result.returncode == 0, result.stderr.decode(errors="replace")
            assert rows(b) == expected, "Isolated restoration membership or bytes mismatch"
    write(target / "RESTORE_VERIFICATION.json", {"before_sha256": digest(before), "after_sha256": digest(after),
        "forward_membership_and_bytes_match": True, "reverse_membership_and_bytes_match": True,
        "isolated_only": True, "workspace_reverse_patch_applied": False, "baseline": ref(BASELINE),
        "canonical_document_preimages": doc_preimages, "new_config_absent_before_present_after": True,
        "patch_method": "git diff --no-index --binary; git apply -p2 in isolated before directory"})
    record_names = {r["path"] for r in records}
    reference_rows = [references[p] for p in sorted(references, key=Path) if p not in record_names]
    pack(target / "RECORDS.tar.gz", ROOT, records)
    pack(target / "REFERENCES.tar.gz", ROOT, reference_rows)
    write(target / "RECORDS.json", records)
    write(target / "REFERENCES.json", reference_rows)
    write(target / "EXTERNAL_ARCHIVES.json", [external[p] for p in sorted(external)])
    def git(*args):
        return subprocess.check_output(["git", *args], cwd=ROOT)
    tracked = set(git("ls-files", "-z").decode().split("\0"))
    dirty = set(git("diff", "--name-only", "-z", "HEAD").decode().split("\0"))
    membership = {}
    for group, entries in (("source", after), ("records", records), ("references", reference_rows),
                           ("external_archive", [v["archive"] for v in external.values()])):
        for row in entries:
            item = membership.setdefault(row["path"], dict(row, groups=[],
                kind="tracked/dirty" if row["path"] in dirty else "tracked/clean" if row["path"] in tracked else "untracked"))
            assert item["sha256"] == row["sha256"] and item["bytes"] == row["bytes"]
            item["groups"].append(group)
    write(target / "MEMBERSHIP.json", [membership[p] for p in sorted(membership)])
    with (target / "GIT_STATUS.txt").open("xb") as stream:
        stream.write(git("status", "--porcelain=v2", "--untracked-files=all"))
    assert scanner.identity() == current
    for row in after + records + reference_rows + [v["archive"] for v in external.values()]:
        check_ref(row)
    write(target / "MANIFEST.json", {"schema": "U09_FROZEN_SELF_VERIFICATION_TARGET_V1", "goal": "GOAL-CRSA-01", "goal_set": "GS-CRSA-003",
        "state": "Frozen self-verification target; independent nonparticipant audit pending",
        "source_sha256": current["sha256"], "source_manifest": ref(source_path),
        "before_files": before, "source_files": after, "before_sha256": digest(before), "restoration_source_sha256": digest(after),
        "changed": [r["path"] for r in after if before_by_path.get(r["path"]) != r],
        "added": sorted({r["path"] for r in after} - set(before_by_path)), "deleted": sorted(set(before_by_path) - {r["path"] for r in after}),
        "selection": ref(generated / "EVIDENCE_SELECTION.json"), "coverage": ref(generated / "COVERAGE.json"),
        "predecessor_impact": ref(generated / "PREDECESSOR_IMPACT.json"), "documentation_handoff": ref(handoff_path),
        "predecessor_independent_closure": ref(PRIOR_AUDIT / "AUDIT_CLOSURE.json"),
        "predecessor_closure_references": [{"reference": closure_refs[path],
            "already_preserved_in_exact_external_archives": archived_closure_refs.get(path, []),
            "direct_reference": path in references} for path in sorted(closure_refs)],
        "repository": str(ROOT), "vcs_revision": git("rev-parse", "HEAD").decode().strip(),
        "archives_and_patches": [{"path": p.name, "sha256": sha(p), "bytes": p.stat().st_size} for p in sorted(target.iterdir()) if p.is_file()],
        "independent_completion": "pending; no self-awarded Complete/High/open0",
        "scope": "Baseline438 plus three canonical document preimages -> union with fixed current source scanner. U09 goal materials in RECORDS; referenced external artifacts in REFERENCES; archives bound externally. Audit outputs bind after independent audit without mutating this target.",
        "excluded": ["Build outputs", "__pycache__", "target directories", "post_audit directories"],
        "historical_archive_limit": "Only explicitly member-verified archives claim an exact independent member manifest. Other historical archive blobs are byte-bound external records, never source restoration preimages."})
    print(json.dumps({"target": str(target.relative_to(ROOT)), "source_sha256": current["sha256"],
                      "restoration_verified": True, "independent_completion": "pending"}))


if __name__ == "__main__":
    main()
