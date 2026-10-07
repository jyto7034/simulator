#!/usr/bin/env python3
"""Freeze the stage-review target for WP02 effect execution segment 001."""

from __future__ import annotations

from datetime import datetime, timezone
import difflib
import gzip
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
SEGMENT = EXECUTION / "segments/wp02_effect_execution_prefix_001"
TARGET = ROOT / SEGMENT / "target/effect_execution_prefix_001"
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001/MANIFEST.json"
PREDECESSOR = ROOT / EXECUTION / (
    "corrections/wp02_cap_01_02_001/target/cap_01_02_correction_002/MANIFEST.json"
)
PREDECESSOR_SHA = "6ee8ea5a889428eef4a0986cd83037ef8c51de6c751868ef9657adb11d3ce789"

EXPECTED_CHANGED = [
    "core_enhanced/src/application.rs",
    "core_enhanced/src/game/battle/core/commands.rs",
    "core_enhanced/src/game/battle/core/damage_runtime.rs",
    "core_enhanced/src/game/battle/core/mod.rs",
    "core_enhanced/src/game/battle/core/resource_runtime.rs",
    "core_enhanced/src/game/battle/core/runtime_facts.rs",
    "core_enhanced/src/game/battle/core/sim.rs",
    "core_enhanced/src/game/battle/core/skill_runtime/area.rs",
    "core_enhanced/src/game/battle/core/skill_runtime/projectile.rs",
    "core_enhanced/src/game/battle/core/target_usefulness.rs",
    "core_enhanced/src/game/battle/core/types.rs",
    "core_enhanced/src/game/battle/core/value_expr_runtime.rs",
    "core_enhanced/src/game/battle/damage.rs",
    "core_enhanced/src/game/battle/enums.rs",
    "core_enhanced/src/game/behavior.rs",
    "core_enhanced/src/game/value_expr/catalog.rs",
    "core_enhanced/src/game/world/snapshot.rs",
]


def identity(payload: bytes) -> dict[str, object]:
    return {"bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}


def file_row(path: Path) -> dict[str, object]:
    payload = path.read_bytes()
    return {"path": path.relative_to(ROOT).as_posix(), **identity(payload)}


def write_json(path: Path, value: object) -> dict[str, object]:
    payload = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
    path.write_bytes(payload)
    return identity(payload)


def write_archive(path: Path, entries: dict[str, bytes]) -> dict[str, object]:
    with path.open("xb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", filename="", mtime=0) as zipped:
            with tarfile.open(fileobj=zipped, mode="w") as archive:
                for name, payload in sorted(entries.items()):
                    member = tarfile.TarInfo(name)
                    member.size = len(payload)
                    member.mode = 0o644
                    member.mtime = 0
                    archive.addfile(member, io.BytesIO(payload))
    return identity(path.read_bytes())


def run(*args: str, cwd: Path = ROOT) -> bytes:
    return subprocess.run(
        args, cwd=cwd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    ).stdout


def patch_bytes(before: dict[str, bytes], after: dict[str, bytes], reverse: bool = False) -> bytes:
    chunks = []
    for path in EXPECTED_CHANGED:
        first, second = (after[path], before[path]) if reverse else (before[path], after[path])
        chunks.extend(
            difflib.unified_diff(
                first.decode("utf-8").splitlines(keepends=True),
                second.decode("utf-8").splitlines(keepends=True),
                fromfile=f"a/{path}",
                tofile=f"b/{path}",
                n=3,
            )
        )
    return "".join(chunks).encode()


def compare_tree(root: Path, expected: dict[str, bytes]) -> list[str]:
    return [
        path
        for path, payload in expected.items()
        if not (root / path).is_file() or (root / path).read_bytes() != payload
    ]


def verify_round_trip(
    before_archive: Path,
    forward_patch: Path,
    reverse_patch: Path,
    before: dict[str, bytes],
    after: dict[str, bytes],
) -> dict[str, object]:
    temporary = Path(tempfile.mkdtemp(prefix="r06-wp02-effect-delta-"))
    with tarfile.open(before_archive, "r:gz") as archive:
        archive.extractall(temporary, filter="data")
    initial_mismatch = compare_tree(temporary, before)
    assert not initial_mismatch, initial_mismatch
    run("git", "apply", "--check", str(forward_patch), cwd=temporary)
    run("git", "apply", str(forward_patch), cwd=temporary)
    forward_mismatch = compare_tree(temporary, after)
    assert not forward_mismatch, forward_mismatch
    run("git", "apply", "--check", str(reverse_patch), cwd=temporary)
    run("git", "apply", str(reverse_patch), cwd=temporary)
    reverse_mismatch = compare_tree(temporary, before)
    assert not reverse_mismatch, reverse_mismatch
    shutil.rmtree(temporary)
    return {
        "before_archive_matches_manifest": True,
        "forward_git_apply_check": True,
        "forward_result_matches_current_after_bytes": True,
        "reverse_git_apply_check": True,
        "reverse_result_matches_before_bytes": True,
        "changed_files": len(EXPECTED_CHANGED),
    }


def main() -> None:
    if TARGET.exists():
        raise SystemExit(f"refusing to overwrite target: {TARGET}")
    predecessor_payload = PREDECESSOR.read_bytes()
    assert identity(predecessor_payload)["sha256"] == PREDECESSOR_SHA
    baseline_payload = BASELINE.read_bytes()
    baseline = json.loads(baseline_payload)

    source_paths = [row["path"] for row in baseline["source"]["files"]]
    before_archive_source = ROOT / baseline["source"]["archive"]["path"]
    before: dict[str, bytes] = {}
    with tarfile.open(before_archive_source, "r:gz") as archive:
        for member in archive.getmembers():
            if member.isfile():
                extracted = archive.extractfile(member)
                assert extracted is not None
                before[member.name] = extracted.read()
    assert sorted(before) == sorted(source_paths)
    after = {path: (ROOT / path).read_bytes() for path in source_paths}
    changed = sorted(path for path in source_paths if before[path] != after[path])
    assert changed == EXPECTED_CHANGED, {"expected": EXPECTED_CHANGED, "actual": changed}

    final_validation = json.loads(
        (ROOT / SEGMENT / "evidence/validation_final/VALIDATION_SUMMARY.json").read_text()
    )
    assert final_validation["result"] == "DIRECT_VALIDATION_COMPLETE"
    assert not final_validation["failed"]
    pc = json.loads(
        (ROOT / SEGMENT / "evidence/performance/pc_current_002/RECEIPT.json").read_text()
    )
    assert pc["result"] == "DIRECT_MEASUREMENT_COMPLETE"
    r05 = json.loads(
        (ROOT / SEGMENT / "evidence/performance/r05_guard_001/RECEIPT.json").read_text()
    )
    assert r05["result"] == "DIRECT_GUARD_PASS"

    TARGET.mkdir(parents=True)
    before_archive = TARGET / "SOURCE_BEFORE_EFFECT_EXECUTION_001.tar.gz"
    before_archive.write_bytes(before_archive_source.read_bytes())
    after_archive = TARGET / "SOURCE_AFTER_EFFECT_EXECUTION_001.tar.gz"
    after_archive_identity = write_archive(after_archive, after)
    forward_patch = TARGET / "SOURCE_DELTA.patch"
    forward_patch.write_bytes(patch_bytes(before, after))
    reverse_patch = TARGET / "SOURCE_RESTORE.patch"
    reverse_patch.write_bytes(patch_bytes(before, after, reverse=True))
    restoration = verify_round_trip(
        before_archive, forward_patch, reverse_patch, before, after
    )
    restoration_identity = write_json(TARGET / "RESTORATION_VERIFICATION.json", restoration)

    basis_paths = [
        TOPIC / "CODE_REVIEW_003.ko.md",
        TOPIC / "P2G_40_PHASE_PLAN_001.json",
        EXECUTION / "segments/wp02_runtime_frame_capture_001/PLAN.md",
        PREDECESSOR.relative_to(ROOT),
        EXECUTION / "reviews/wp02_cap_01_02_correction_002_review_001/REVIEW.md",
        SEGMENT / "PLAN.md",
        SEGMENT / "IMPLEMENTATION_RECORD.ko.md",
        SEGMENT / "DEVELOPMENT_FAILURES.json",
        SEGMENT / "REVIEW_HANDOFF.ko.md",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_baseline_001.py",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_validate_001.py",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_pc_001.py",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_r05_guard_001.py",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_freeze_001.py",
        TOPIC / "authoring_scripts/r06_wp02_effect_execution_verify_target_001.py",
    ]
    basis = {path.as_posix(): (ROOT / path).read_bytes() for path in basis_paths}
    evidence_root = ROOT / SEGMENT / "evidence"
    evidence_paths = sorted(path for path in evidence_root.rglob("*") if path.is_file())
    evidence = {path.relative_to(ROOT).as_posix(): path.read_bytes() for path in evidence_paths}
    bundle_entries = {**basis, **evidence}
    evidence_archive_identity = write_archive(
        TARGET / "EVIDENCE_AND_HANDOFF_001.tar.gz", bundle_entries
    )

    scoped_status = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--",
        *EXPECTED_CHANGED, SEGMENT.as_posix(),
    )
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all")
    (TARGET / "SCOPED_GIT_STATUS_AT_TARGET.txt").write_bytes(scoped_status)
    (TARGET / "GLOBAL_GIT_STATUS_AT_TARGET.txt").write_bytes(global_status)

    source_rows = []
    for path in source_paths:
        source_rows.append(
            {
                "path": path,
                "before": identity(before[path]),
                "after": identity(after[path]),
                "changed": path in EXPECTED_CHANGED,
            }
        )
    artifacts = [
        file_row(before_archive),
        {"path": after_archive.relative_to(ROOT).as_posix(), **after_archive_identity},
        file_row(forward_patch),
        file_row(reverse_patch),
        {
            "path": (TARGET / "RESTORATION_VERIFICATION.json").relative_to(ROOT).as_posix(),
            **restoration_identity,
        },
        {
            "path": (TARGET / "EVIDENCE_AND_HANDOFF_001.tar.gz").relative_to(ROOT).as_posix(),
            **evidence_archive_identity,
        },
        file_row(TARGET / "SCOPED_GIT_STATUS_AT_TARGET.txt"),
        file_row(TARGET / "GLOBAL_GIT_STATUS_AT_TARGET.txt"),
    ]
    manifest = {
        "record": "R06_WP02_EFFECT_EXECUTION_PREFIX_TARGET_001",
        "state": "partial_implementation_self_validation_complete_stage_review_pending",
        "frozen_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "predecessor": {
            "path": PREDECESSOR.relative_to(ROOT).as_posix(),
            **identity(predecessor_payload),
            "review_scope": "CAP correction only; not WP02 completion",
        },
        "implementation_baseline": {
            "path": BASELINE.relative_to(ROOT).as_posix(),
            **identity(baseline_payload),
            "source_files": len(source_paths),
            "unexpected_entry_drift": [],
        },
        "scope": {
            "implemented": [
                "per-delivery sequential effect evaluate-all-targets/apply/settle",
                "exact completed-prefix witness with actual result projections",
                "immediate/projectile/area typed error propagation and candidate rollback",
            ],
            "remaining": [
                "cross-step/cross-delivery action-level prefix witness",
                "release-before-completed action result Snapshot carrier",
                "ResolvedTargets aggregate frame connection",
                "public expression publication",
            ],
        },
        "claims": {
            "partial_implementation_self_validation_complete": True,
            "stage_review_target_frozen": True,
            "stage_review_pending": True,
            "review_pass_declared": False,
            "wp02_complete": False,
            "r06_complete": False,
            "phase_complete": False,
            "goal_complete": False,
            "publication_unblocked": False,
            "limits_relaxed": False,
        },
        "source": {
            "file_count": len(source_rows),
            "changed_file_count": len(EXPECTED_CHANGED),
            "changed_paths": EXPECTED_CHANGED,
            "files": source_rows,
            "before_archive": file_row(before_archive),
            "after_archive": {
                "path": after_archive.relative_to(ROOT).as_posix(),
                **after_archive_identity,
            },
            "forward_delta": file_row(forward_patch),
            "restore_delta": file_row(reverse_patch),
            "restoration_verification": restoration,
        },
        "verification": {
            "final_validation": file_row(
                ROOT / SEGMENT / "evidence/validation_final/VALIDATION_SUMMARY.json"
            ),
            "all_targets": {"lib_passed": 1432, "integration_passed": 52, "failed": 0},
            "focused": {
                "effect_execution": 23,
                "delivery_error_propagation": 3,
                "projectile_snapshot_lifetime": 1,
                "runtime_facts": 13,
                "resources": 37,
                "literal_runtime": 15,
            },
            "pc_fixture": file_row(
                ROOT / SEGMENT / "evidence/performance/pc_current_002/RECEIPT.json"
            ),
            "r05_guard": file_row(
                ROOT / SEGMENT / "evidence/performance/r05_guard_001/RECEIPT.json"
            ),
            "development_failures": file_row(ROOT / SEGMENT / "DEVELOPMENT_FAILURES.json"),
            "direct_vs_reused": {
                "direct": [
                    "all final validation commands",
                    "unchanged 36-condition PC fixture",
                    "R05 allocation-observer guard",
                    "delta round-trip/currentness",
                ],
                "reused": [
                    "predecessor approval and immutable implementation-entry baseline",
                ],
                "not_run": [
                    "other OS/allocator measurements",
                    "public expression publication",
                    "WP03/WP04 validation",
                ],
            },
        },
        "basis": {
            "files": [{"path": path, **identity(payload)} for path, payload in sorted(basis.items())],
        },
        "evidence": {
            "file_count": len(evidence),
            "files": [
                {"path": path, **identity(payload)} for path, payload in sorted(evidence.items())
            ],
            "archive": {
                "path": (TARGET / "EVIDENCE_AND_HANDOFF_001.tar.gz").relative_to(ROOT).as_posix(),
                **evidence_archive_identity,
            },
        },
        "artifacts": artifacts,
        "status": {
            "scoped": file_row(TARGET / "SCOPED_GIT_STATUS_AT_TARGET.txt"),
            "global": file_row(TARGET / "GLOBAL_GIT_STATUS_AT_TARGET.txt"),
        },
        "handoff": file_row(ROOT / SEGMENT / "REVIEW_HANDOFF.ko.md"),
        "terminal_status": "부분 구현·자체 검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기",
    }
    manifest_identity = write_json(TARGET / "MANIFEST.json", manifest)
    (TARGET / "MANIFEST.sha256").write_text(
        manifest_identity["sha256"] + "  MANIFEST.json\n", encoding="utf-8"
    )
    print(json.dumps({
        "target": TARGET.relative_to(ROOT).as_posix(),
        "manifest_sha256": manifest_identity["sha256"],
        "source_files": len(source_paths),
        "changed_files": len(EXPECTED_CHANGED),
        "restoration_verified": True,
        "state": manifest["state"],
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
