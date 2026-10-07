"""Freeze this WP02 checkpoint and verify current bytes plus reversible deltas."""

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
TARGET = HERE / "target/d04_owner_relation_001"
BASES = (
    HERE / "baseline/entry_001/MANIFEST.json",
    HERE / "baseline/tool_consumers_001/MANIFEST.json",
    HERE / "baseline/tool_locks_001/MANIFEST.json",
)
EXPECTED = {
    "core_enhanced/src/game/battle/core/boss_runtime.rs",
    "core_enhanced/src/game/battle/core/build.rs",
    "core_enhanced/src/game/battle/core/commands.rs",
    "core_enhanced/src/game/battle/core/field_work.rs",
    "core_enhanced/src/game/battle/core/mod.rs",
    "core_enhanced/src/game/battle/core/movement/engine.rs",
    "core_enhanced/src/game/battle/core/movement/planner.rs",
    "core_enhanced/src/game/battle/core/runtime_facts.rs",
    "core_enhanced/src/game/battle/core/summon_runtime.rs",
    "core_enhanced/src/game/battle/core/tactical_stance.rs",
    "core_enhanced/src/game/battle/core/targeting.rs",
    "core_enhanced/src/game/battle/core/types.rs",
    "core_enhanced/src/game/battle/core/value_expr_runtime.rs",
    "core_enhanced/src/game/battle/core/wave_runtime.rs",
    "core_enhanced/src/game/battle/stat_pipeline.rs",
    "core_enhanced/src/game/battle/types.rs",
    "core_enhanced/src/game/combat_setup/boss_runtime.rs",
    "core_enhanced/src/game/combat_setup/defense_object.rs",
    "core_enhanced/src/game/combat_setup/enemy_spawns.rs",
    "core_enhanced/src/game/combat_setup/player_spawns.rs",
    "core_enhanced/src/game/events/combat.rs",
    "core_enhanced/src/game/resources/item_slot.rs",
    "core_enhanced/src/game/value_expr/compile.rs",
    "core_enhanced/src/game/value_expr/eval.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/parse.rs",
    "core_enhanced/src/game/value_expr/tests.rs",
    "core_enhanced/src/game/world/tests/combat.rs",
    "core_enhanced/tests/live_item_skill_activation.rs",
    "core_enhanced/tests/skill_runtime_contract.rs",
    "core_enhanced/tests/skill_test/common/scenario.rs",
    "core_enhanced/tools/runtime_architecture_final_validation/src/final_performance.rs",
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


baselines = [json.loads(path.read_text()) for path in BASES]
check(baselines[0]["predecessor"])
for baseline in baselines:
    check(baseline["archive"])

files = [
    entry["path"]
    for baseline in baselines
    for entry in baseline["source"]["files"]
]
assert len(files) == len(set(files)) == 337


def archived_before():
    contents = {}
    for baseline in baselines:
        expected = {entry["path"] for entry in baseline["source"]["files"]}
        with tarfile.open(ROOT / baseline["archive"]["path"]) as archive:
            members = archive.getmembers()
            assert {member.name for member in members} == expected
            for member in members:
                assert member.isfile() and not Path(member.name).is_absolute()
                assert ".." not in Path(member.name).parts
                source = archive.extractfile(member)
                assert source is not None
                contents[member.name] = source.read()
    assert set(contents) == set(files)
    return contents


def archived_after(path):
    with tarfile.open(path) as archive:
        members = archive.getmembers()
        assert {member.name for member in members} == set(files)
        assert all(
            member.isfile()
            and not Path(member.name).is_absolute()
            and ".." not in Path(member.name).parts
            for member in members
        )
        result = {}
        for name in files:
            source = archive.extractfile(name)
            assert source is not None
            result[name] = source.read()
        return result


def identity(contents, selected=files):
    rows = [(name, len(contents[name]), sha(contents[name])) for name in selected]
    return sha(json.dumps(rows, separators=(",", ":")).encode())


def verify(manifest):
    for entry in manifest["baselines"]:
        check(entry)
    check(manifest["predecessor"])
    check(manifest["after_archive"])
    for entry in manifest["artifacts"] + manifest["records"]:
        check(entry)

    record_by_path = {entry["path"]: entry for entry in manifest["records"]}
    assert len(record_by_path) == len(manifest["records"])
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
    assert identity(before) == manifest["source"]["before_digest"]
    assert identity(after) == manifest["source"]["after_digest"]
    assert identity(after, files[:333]) == manifest["source"]["product_after_digest"]
    assert [entry["path"] for entry in manifest["source"]["files"]] == files

    baseline_rows = {
        entry["path"]: {"bytes": entry["bytes"], "sha256": entry["sha256"]}
        for baseline in baselines
        for entry in baseline["source"]["files"]
    }
    changed = []
    for entry in manifest["source"]["files"]:
        name = entry["path"]
        assert entry["before"] == baseline_rows[name]
        assert {"bytes": len(before[name]), "sha256": sha(before[name])} == entry["before"]
        assert {"bytes": len(after[name]), "sha256": sha(after[name])} == entry["after"]
        assert (ROOT / name).read_bytes() == after[name], name
        if before[name] != after[name]:
            changed.append(name)
    assert set(changed) == EXPECTED
    assert changed == manifest["source"]["changed_paths"]

    for origin, destination, patch_name in (
        (before, after, "SOURCE_DELTA.patch"),
        (after, before, "SOURCE_RESTORE.patch"),
    ):
        with tempfile.TemporaryDirectory(prefix="wp02-owner-restore-") as temp:
            root = Path(temp)
            for name, data in origin.items():
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
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
            for name in files:
                assert (root / name).read_bytes() == destination[name], name

    return {
        "state": "CURRENTNESS_AND_BIDIRECTIONAL_RESTORATION_VERIFIED",
        "source_count": len(files),
        "product_source_count": 333,
        "changed_paths": changed,
        "record_count": len(manifest["records"]),
    }


if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in ("freeze", "verify"):
        raise SystemExit("use freeze or verify")
    if sys.argv[1] == "verify":
        manifest = json.loads((TARGET / "MANIFEST.json").read_text())
        print(
            json.dumps(
                {
                    "manifest": meta(TARGET / "MANIFEST.json"),
                    "verification": verify(manifest),
                },
                indent=2,
            )
        )
        raise SystemExit(0)

    assert not TARGET.exists(), TARGET
    before = archived_before()
    after = {name: (ROOT / name).read_bytes() for name in files}
    changed = [name for name in files if before[name] != after[name]]
    assert set(changed) == EXPECTED, changed

    TARGET.mkdir(parents=True)
    for reverse, filename in (
        (False, "SOURCE_DELTA.patch"),
        (True, "SOURCE_RESTORE.patch"),
    ):
        origin, result = (after, before) if reverse else (before, after)
        delta = "".join(
            "".join(
                difflib.unified_diff(
                    origin[name].decode().splitlines(keepends=True),
                    result[name].decode().splitlines(keepends=True),
                    fromfile="a/" + name,
                    tofile="b/" + name,
                )
            )
            for name in changed
        )
        (TARGET / filename).write_text(delta)

    with tarfile.open(TARGET / "SOURCE_AFTER.tar.gz", "w:gz") as archive:
        for name in files:
            archive.add(ROOT / name, arcname=name, recursive=False)

    records = [
        meta(path)
        for path in sorted(HERE.rglob("*"))
        if path.is_file() and TARGET not in path.parents
    ]
    with tarfile.open(TARGET / "EVIDENCE_AND_HANDOFF.tar.gz", "w:gz") as archive:
        for entry in records:
            archive.add(ROOT / entry["path"], arcname=entry["path"], recursive=False)

    manifest = {
        "record": "R06_WP02_D04_OWNER_RELATION_001",
        "state": "partial_implementation_self_validation_complete_review_pending",
        "created_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "baselines": [meta(path) for path in BASES],
        "predecessor": baselines[0]["predecessor"],
        "source": {
            "file_count": len(files),
            "product_file_count": 333,
            "before_digest": identity(before),
            "after_digest": identity(after),
            "product_after_digest": identity(after, files[:333]),
            "changed_paths": changed,
            "added_paths": [],
            "deleted_paths": [],
            "files": [
                {
                    "path": name,
                    "before": {"bytes": len(before[name]), "sha256": sha(before[name])},
                    "after": {"bytes": len(after[name]), "sha256": sha(after[name])},
                }
                for name in files
            ],
        },
        "after_archive": meta(TARGET / "SOURCE_AFTER.tar.gz"),
        "artifacts": [meta(path) for path in sorted(TARGET.iterdir())],
        "records": records,
        "claims": {
            "independent_review_pass": False,
            "wp02_complete": False,
            "r06_complete": False,
            "publication_unblocked": False,
            "twenty_minute_test_run": False,
            "concurrent_peak_warning_only": True,
        },
    }
    verification = verify(manifest)
    save(TARGET / "RESTORATION_VERIFICATION.json", verification)
    manifest["artifacts"].append(meta(TARGET / "RESTORATION_VERIFICATION.json"))
    save(TARGET / "MANIFEST.json", manifest)
    print(
        json.dumps(
            {
                "manifest": meta(TARGET / "MANIFEST.json"),
                "verification": verification,
            },
            indent=2,
        )
    )
