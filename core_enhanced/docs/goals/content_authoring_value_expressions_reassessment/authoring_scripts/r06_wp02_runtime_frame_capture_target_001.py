#!/usr/bin/env python3
"""Freeze R06 WP02 runtime frame/capture segment target 001."""

from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
SEGMENT = EXECUTION / "segments/wp02_runtime_frame_capture_001"
BASELINE = ROOT / SEGMENT / "baseline/pre_implementation_001"
PREDECESSOR = ROOT / EXECUTION / (
    "corrections/wp01_target_001_f01_f02/target/correction_001"
)
OUTPUT = ROOT / SEGMENT / "target/target_001"
HANDOFF = ROOT / SEGMENT / "REVIEW_HANDOFF.md"
HANDOFF_HASH = HANDOFF.with_suffix(".md.sha256")
SOURCE_ARCHIVE = OUTPUT / "SOURCE_AFTER_WP02_RUNTIME_FRAME_CAPTURE_001.tar.gz"
RECORD_ARCHIVE = OUTPUT / "RECORDS_AT_TARGET_001.tar.gz"
EVIDENCE_ARCHIVE = OUTPUT / "EVIDENCE_AT_TARGET_001.tar.gz"
SOURCE_DELTA = OUTPUT / "SOURCE_DELTA.patch"

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
RECORDS = [
    SEGMENT / "PLAN.md",
    SEGMENT / "EXPERIMENTS.md",
    SEGMENT / "EXPERIMENT_NOTES.md",
    TOPIC / "authoring_scripts/r06_wp02_runtime_frame_capture_run_validation.py",
    TOPIC / "authoring_scripts/r06_wp02_runtime_frame_capture_target_001.py",
    TOPIC / "authoring_scripts/r06_wp02_verify_runtime_frame_capture_target_001.py",
]
REQUIRED_SUCCESS = [
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
]
INVALID_ATTEMPTS = {"wp01_review_probe": 101}
EXPECTED_TEST_SUMMARIES = {
    "focused_runtime_frame_capture": "7 passed; 0 failed",
    "r04_runtime_fact_regression": "11 passed; 0 failed",
    "r05_resource_store_regression": "37 passed; 0 failed",
    "r05_resource_lifetime_regression": "17 passed; 0 failed",
    "wp01_value_expr_regression": "36 passed; 0 failed",
    "wp01_content_pipeline_regression": "59 passed; 0 failed",
    "authoring_budget_regression": "15 passed; 0 failed",
    "checkpoint_save_regression": "18 passed; 0 failed",
    "literal_skill_runtime_regression": "15 passed; 0 failed",
    "wp01_review_probe_after_fixture_fix": "3 passed; 0 failed; 1 ignored",
    "wp01_parser_depth_probe": "1 passed; 0 failed",
}


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        args, cwd=ROOT, check=check, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )


def row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    if not absolute.is_file():
        raise SystemExit(f"missing required file: {relative.as_posix()}")
    return {
        "path": relative.as_posix(),
        "bytes": absolute.stat().st_size,
        "sha256": digest(absolute),
    }


def verify_row(entry: dict[str, object]) -> None:
    current = row(Path(str(entry["path"])))
    if any(current[key] != entry[key] for key in ("path", "bytes", "sha256")):
        raise SystemExit(f"frozen row mismatch: {entry['path']}")


def validate_detached(directory: Path, expected_hash: str) -> dict[str, object]:
    manifest_path = directory / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (directory / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != expected_hash or detached != expected_hash:
        raise SystemExit(f"manifest identity mismatch: {manifest_path}")
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def validate_archive(entry: dict[str, object], expected: list[dict[str, object]]) -> None:
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


def source_paths(scope: list[str]) -> set[str]:
    paths: set[str] = set()
    for raw in scope:
        absolute = ROOT / raw
        if absolute.is_file():
            paths.add(Path(raw).as_posix())
        elif absolute.is_dir():
            paths.update(
                child.relative_to(ROOT).as_posix()
                for child in absolute.rglob("*")
                if child.is_file()
            )
        else:
            raise SystemExit(f"source scope path missing: {raw}")
    return paths


def changed_rows(
    before: list[dict[str, object]], after: list[dict[str, object]]
) -> list[dict[str, object]]:
    old = {str(item["path"]): item for item in before}
    new = {str(item["path"]): item for item in after}
    changes = []
    for path in sorted(set(old) | set(new)):
        left = old.get(path)
        right = new.get(path)
        if left == right:
            continue
        changes.append(
            {
                "path": path,
                "state": "added" if left is None else "deleted" if right is None else "modified",
                "before_sha256": None if left is None else left["sha256"],
                "before_bytes": None if left is None else left["bytes"],
                "after_sha256": None if right is None else right["sha256"],
                "after_bytes": None if right is None else right["bytes"],
            }
        )
    return changes


def write_archive(destination: Path, entries: list[dict[str, object]]) -> dict[str, object]:
    with tarfile.open(destination, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for entry in entries:
            relative = Path(str(entry["path"]))
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    result = row(destination.relative_to(ROOT))
    result["file_count"] = len(entries)
    validate_archive(result, entries)
    return result


def write_delta(before_archive: Path, changes: list[dict[str, object]]) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="r06-wp02-frame-target-") as raw:
        temp = Path(raw)
        extracted = temp / "extracted"
        before = temp / "before"
        after = temp / "after"
        extracted.mkdir()
        before.mkdir()
        after.mkdir()
        with tarfile.open(before_archive, "r:gz") as archive:
            archive.extractall(extracted, filter="data")
        for change in changes:
            relative = Path(str(change["path"]))
            old = extracted / relative
            current = ROOT / relative
            if old.is_file():
                destination = before / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(old, destination)
            if current.is_file():
                destination = after / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(current, destination)
        completed = run(
            "git",
            "diff",
            "--no-index",
            "--binary",
            "--src-prefix=pre_implementation_001/",
            "--dst-prefix=target_001/",
            str(before),
            str(after),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise SystemExit(completed.stderr.decode(errors="replace"))
        SOURCE_DELTA.write_bytes(completed.stdout)
    if SOURCE_DELTA.stat().st_size == 0:
        raise SystemExit("recoverable source delta is empty")
    return row(SOURCE_DELTA.relative_to(ROOT))


def validation_rows(
    current_source: list[dict[str, object]], baseline: dict[str, object]
) -> list[dict[str, object]]:
    validation_root = ROOT / SEGMENT / "evidence/validation"
    expected = set(REQUIRED_SUCCESS) | set(INVALID_ATTEMPTS)
    actual = {
        path.stem
        for path in validation_root.glob("*.json")
        if path.name not in {"VALIDATION_SUMMARY.json", "development_failures.json"}
    }
    if actual != expected:
        raise SystemExit(f"validation receipt set mismatch: {sorted(actual ^ expected)}")
    current_by_path = {str(item["path"]): item for item in current_source}
    results = []
    for label in sorted(expected):
        path = validation_root / f"{label}.json"
        receipt = json.loads(path.read_text(encoding="utf-8"))
        verify_row(receipt["output"])
        if receipt["baseline_manifest_sha256"] != EXPECTED_BASELINE_HASH:
            raise SystemExit(f"validation baseline mismatch: {label}")
        if label in INVALID_ATTEMPTS:
            expected_exit = INVALID_ATTEMPTS[label]
            role = "invalid_fixture_setup_preserved_not_counted"
            if receipt.get("counted_as_validation") is not False:
                raise SystemExit(f"invalid attempt counted as validation: {label}")
        else:
            expected_exit = 0
            role = "required_direct_success"
            if receipt.get("zero_match"):
                raise SystemExit(f"zero-match validation selector: {label}")
            for identity in receipt["source_identity"]:
                if current_by_path.get(identity["path"]) != identity:
                    raise SystemExit(
                        f"validation source identity mismatch: {label} {identity['path']}"
                    )
        if receipt["exit_code"] != expected_exit:
            raise SystemExit(f"unexpected validation exit: {label}")
        results.append(
            {
                "label": label,
                "role": role,
                "exit_code": expected_exit,
                "matched_test_counts": receipt["matched_test_counts"],
                "command": receipt["command"],
                "receipt": row(path.relative_to(ROOT)),
                "output": receipt["output"],
            }
        )
    by_label = {item["label"]: item for item in results}
    for label, expected_text in EXPECTED_TEST_SUMMARIES.items():
        output = (ROOT / str(by_label[label]["output"]["path"])).read_text(
            encoding="utf-8", errors="replace"
        )
        if expected_text not in output:
            raise SystemExit(f"validation summary missing: {label}")
    summary = json.loads(
        (validation_root / "VALIDATION_SUMMARY.json").read_text(encoding="utf-8")
    )
    if summary["required_receipts"] != REQUIRED_SUCCESS or summary["review_pass_declared"]:
        raise SystemExit("validation summary contract mismatch")
    return results


def handoff_text(
    manifest_hash: str, changes: list[dict[str, object]], evidence_count: int
) -> str:
    change_lines = "\n".join(
        f"- `{item['path']}`: `{item['state']}`, `{item['before_sha256']}` → `{item['after_sha256']}`"
        for item in changes
    )
    return f"""# R06 WP02 runtime frame/capture segment 001 재검토 인계

Status: `부분 구현·검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기`

이 인계는 WP02 전체 완료가 아니라 첫 의미 있는 단계검토 경계의 부분 구현과 자체 검증 결과다. 자체 테스트 성공을 재검토 PASS로 표시하지 않으며 다음 구간, GEX-20/30, WP/Phase/Goal 완료를 시작하거나 선언하지 않는다.

## Before / after identity

- 승인 predecessor target: `{(PREDECESSOR / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- predecessor SHA-256: `{EXPECTED_PREDECESSOR_HASH}`
- pre-implementation baseline: `{(BASELINE / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- baseline SHA-256: `{EXPECTED_BASELINE_HASH}`
- after target: `{(OUTPUT / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- after target SHA-256: `{manifest_hash}`

기존 correction target, baseline, 감사 보고서와 그 evidence를 덮어쓰지 않았다. 새 target에는 source actual bytes, 실행 기록 actual bytes, evidence {evidence_count}개 actual bytes와 binary-capable 복원 delta를 별도로 보존했다.

## 구현 책임과 결과

- generation/capture: production `World`의 exact immutable content identity를 `BattleCore`가 공유하며, current catalog의 exact plan만 capture한다. capture는 action attempt, delivery occurrence, authored source/site/fingerprint를 별도 검증한다.
- Snapshot/Live: Snapshot은 release frame을 한 번 보존하고 Live는 consumption/effect-start 인자의 target/time으로 실제 core state를 읽는다. plan/source/frame은 `Arc` 공유이고 전체 state/AST deep clone은 없다.
- 실제 scalar: Attack, HP Current/Max/Missing, BattleElapsed, explicit-deadline RemainingTime과 typed resource Balance/Reserved/Available을 연결했다.
- 계산: 기존 admitted plan limits, bounded evaluator, rounding/checked arithmetic, Damage/Heal narrowing만 사용한다. 별도 계산 의미는 없다.
- resource 권한: mutation consumer와 공통 exact lease resolver를 사용하되 read는 `bind`/`value`뿐이며 지출 capability, reservation, revision을 만들거나 변경하지 않는다. 공개 `apply_operations` 계약도 유지했다.
- typed refusal: 미확정 target, 사라진 owner/unit, generation/action/delivery mismatch, missing deadline, 아직 미연결인 stat/result/aggregate는 0/default 없이 typed fault이고 실제 source `EffectSite`를 유지한다.

## 변경 source

{change_lines}

## 직접 실행 검증

- focused adapter 7/7, R04 runtime facts 11/11, R05 resource store 37/37, R05 lifetime 17/17 PASS.
- WP01 ValueExpr 36/36, content pipeline 59/59, authoring 15/15, checkpoint/save 18/18, literal skill runtime 15/15 PASS.
- 같은 WP01 review probe는 새 임시 external pack에서 3 pass / 0 fail / 1 ignored, depth probe는 1/1 PASS. `typo_parameter`, authored negative aggregate, source diagnostics와 publication gate assertion을 유지했다.
- all-target check와 focused rustfmt는 exit 0. Clippy는 predecessor와 같은 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용해 exit 0이며 기존 warning과 아직 consumer 미연결에 따른 dead-code warning을 raw log에 보존했다.
- tight work budget 및 cancellation은 capture allocation/mutation 전에 거부되고 core transaction snapshot이 불변임을 확인했다.
- 첫 external fixture 실행의 부모 디렉터리 setup 오류(exit 101)는 무효 시도로 삭제하지 않고 보존했다. assertion 변경 없이 setup만 교정한 별도 receipt를 성공 근거로 사용했다.

## 재사용 증거

- predecessor target SHA와 WP01-F01/F02 부분 PASS 감사 보고서는 진입 currentness/계보로만 재사용했다.
- R01/R04/R05 과거 보고서는 계약 조사에만 사용했고 성공 수에 합산하지 않았다. 관련 seam은 이번 source identity에서 직접 재실행했다.

## 잔여 위험과 다음 검토 경계

- adapter는 actual `BattleCore` state에 연결됐지만 production skill effect consumer는 아직 호출하지 않는다. 따라서 authored expression publication은 계속 `content_capability_unsupported`다.
- sequential effect execution, exact completed-prefix result witness, `ResolvedTargets` member frame, callback/instant/projectile/area 실패 전파와 후속 effect 실패 시 candidate 전체 rollback은 미구현이다.
- SpellPower/AttackSpeedPoints runtime producer, preview/explain/usefulness, atomic publication cutover는 미구현이다.
- 이번 target은 위 책임 경계만 재검토한다. 재검토 결과 전에는 이 source·records·evidence·handoff·target을 변경하지 않는다.
"""


def main() -> None:
    if OUTPUT.exists() or HANDOFF.exists() or HANDOFF_HASH.exists():
        raise SystemExit("target or handoff already exists")
    baseline = validate_detached(BASELINE, EXPECTED_BASELINE_HASH)
    predecessor = validate_detached(PREDECESSOR, EXPECTED_PREDECESSOR_HASH)
    if baseline["record"] != "R06_WP02_RUNTIME_FRAME_CAPTURE_BASELINE_001":
        raise SystemExit("unexpected baseline record")
    if predecessor["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_TARGET_001":
        raise SystemExit("unexpected predecessor record")

    validate_archive(baseline["source"]["archive"], baseline["source"]["files"])
    validate_archive(
        baseline["basis_documents"]["archive"], baseline["basis_documents"]["files"]
    )
    verify_row(baseline["predecessor_review"])
    verify_row(baseline["git_status"]["global"])
    verify_row(baseline["git_status"]["scoped"])

    actual_paths = source_paths(baseline["source"]["scope"])
    current_source = [row(Path(path)) for path in sorted(actual_paths)]
    source_changes = changed_rows(baseline["source"]["files"], current_source)
    actual_changes = {item["path"]: item["state"] for item in source_changes}
    if actual_changes != EXPECTED_SOURCE_CHANGES:
        raise SystemExit(f"unexpected source delta: {actual_changes}")
    validations = validation_rows(current_source, baseline)

    evidence_paths = sorted(
        (
            path.relative_to(ROOT)
            for path in (ROOT / SEGMENT / "evidence").rglob("*")
            if path.is_file()
        ),
        key=lambda path: path.as_posix(),
    )
    evidence = [row(path) for path in evidence_paths]
    execution_records = [row(path) for path in RECORDS]

    OUTPUT.mkdir(parents=True, exist_ok=False)
    source_archive = write_archive(SOURCE_ARCHIVE, current_source)
    record_archive = write_archive(RECORD_ARCHIVE, execution_records)
    evidence_archive = write_archive(EVIDENCE_ARCHIVE, evidence)
    source_delta = write_delta(
        ROOT / str(baseline["source"]["archive"]["path"]), source_changes
    )

    scope = baseline["source"]["scope"]
    scoped_status = run(
        "git", "status", "--porcelain=v1", "--untracked-files=all", "--", *scope
    ).stdout
    scoped_path = OUTPUT / "SCOPED_GIT_STATUS_AT_TARGET.txt"
    scoped_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    global_path = OUTPUT / "GLOBAL_GIT_STATUS_AT_TARGET.txt"
    global_path.write_bytes(global_status)

    changes_by_state = {
        state: sum(item["state"] == state for item in source_changes)
        for state in ("added", "modified", "deleted")
    }
    manifest = {
        "record": "R06_WP02_RUNTIME_FRAME_CAPTURE_TARGET_001",
        "state": "partial_implementation_self_validation_complete_target_frozen_review_pending",
        "status_text": "부분 구현·검증 종료 / 단계검토 타깃 동결 / 재검토 요청 대기",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").stdout.decode().strip(),
        "head": run("git", "rev-parse", "HEAD").stdout.decode().strip(),
        "goal": baseline["goal"],
        "handoff": baseline["handoff"],
        "pipeline_revision": baseline["pipeline_revision"],
        "checkpoint_scope": baseline["checkpoint_scope"],
        "completion_claims": {
            "partial_implementation_self_validation_complete": True,
            "stage_review_target_frozen": True,
            "review_pass_declared": False,
            "wp02_complete": False,
            "next_segment_started": False,
            "gex_20_started": False,
            "gex_30_started": False,
            "phase_complete": False,
            "goal_complete": False,
        },
        "predecessor_target": {
            "record": predecessor["record"],
            "manifest": (PREDECESSOR / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": EXPECTED_PREDECESSOR_HASH,
            "entry_currentness_verified_before_mutation": True,
            "frozen_artifacts_unchanged": True,
            "live_source_expectedly_superseded_by_this_segment": True,
        },
        "pre_implementation_baseline": {
            "record": baseline["record"],
            "manifest": (BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": EXPECTED_BASELINE_HASH,
            "source_archive": baseline["source"]["archive"],
            "basis_document_archive": baseline["basis_documents"]["archive"],
        },
        "preservation": {
            "predecessor_target_manifest_unchanged": True,
            "predecessor_review": baseline["predecessor_review"],
            "baseline_global_status": baseline["git_status"]["global"],
            "baseline_scoped_status": baseline["git_status"]["scoped"],
            "original_game_resources_modified": False,
            "existing_dirty_bytes_preserved": True,
        },
        "current_source": {
            "scope": scope,
            "file_count": len(current_source),
            "files": current_source,
            "archive": source_archive,
            "changes_from_baseline": {
                **changes_by_state,
                "files": source_changes,
            },
            "recoverable_delta": source_delta,
            "scoped_status": {
                **row(scoped_path.relative_to(ROOT)),
                "rows": len(scoped_status.splitlines()),
            },
            "global_status": {
                **row(global_path.relative_to(ROOT)),
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before manifest/hash/handoff creation to avoid self-reference.",
            },
        },
        "execution_records": {
            "file_count": len(execution_records),
            "files": execution_records,
            "archive": record_archive,
            "entry_state": "new segment records were absent at baseline",
            "handoff_note": "The detached handoff is generated after the manifest to bind its SHA-256 and is verified separately.",
        },
        "evidence": {
            "file_count": len(evidence),
            "files": evidence,
            "archive": evidence_archive,
        },
        "validation_receipts": validations,
        "validation_summary": {
            "focused_runtime_frame_capture": "7 passed",
            "r04_runtime_facts": "11 passed",
            "r05_resource_store": "37 passed",
            "r05_resource_lifetimes": "17 passed",
            "wp01_value_expr": "36 passed",
            "wp01_content_pipeline": "59 passed",
            "authoring": "15 passed",
            "checkpoint_save": "18 passed",
            "literal_skill_runtime": "15 passed",
            "wp01_review_probe": "3 passed / 0 failed / 1 ignored",
            "parser_depth_probe": "1 passed",
            "all_targets_check": "exit 0",
            "focused_rustfmt": "exit 0",
            "clippy": "exit 0 with only predecessor allows clippy::never_loop and clippy::eq_op",
            "invalid_fixture_attempt": "exit 101 preserved and not counted",
            "direct_vs_reused": "all listed successful receipts are direct on current source; predecessor target/review and R01/R04/R05 reports are lineage-only reused evidence",
        },
        "implemented_responsibilities": [
            "exact generation/catalog plan and action/delivery identity-bound capture",
            "release Snapshot frame versus consumption/effect-start Live frame",
            "actual Attack/HP/time/resource scalar reads",
            "shared bounded evaluator and Damage/Heal narrowing",
            "read-only exact resource lease resolution shared with the mutation consumer",
            "typed refusal without fallback for missing or not-yet-connected contexts",
        ],
        "deferred_responsibilities": [
            "sequential skill effect consumer and completion-order execution",
            "exact completed-prefix result witness and ResolvedTargets member frames",
            "callback/instant/projectile/area error propagation and whole-candidate rollback",
            "SpellPower/AttackSpeedPoints runtime producers",
            "preview/explain/usefulness runtime frames and atomic publication cutover",
        ],
        "publication_gate": {
            "code": "content_capability_unsupported",
            "remains_closed": True,
            "direct_probe_verified": True,
        },
        "residual_risks": [
            "fresh independent stage review is required",
            "the adapter is intentionally not called by the production skill effect consumer yet",
            "transaction failure propagation and result-prefix ordering remain unimplemented",
            "dead-code warnings expose the unconnected consumer seam and were not suppressed",
        ],
        "review_entry": {
            "path": HANDOFF.relative_to(ROOT).as_posix(),
            "detached_sha256": HANDOFF_HASH.relative_to(ROOT).as_posix(),
            "binding": "Generated after MANIFEST.json; verifier checks detached hash and exact manifest hash text.",
        },
        "mutation_boundary": "Do not mutate segment source, records, evidence, handoff, or target after final read-only currentness verification until stage review returns.",
        "recovery": {
            "actual_before_after_source_bytes_preserved": True,
            "actual_record_and_evidence_bytes_preserved": True,
            "recoverable_binary_capable_source_delta_preserved": True,
            "safe_method": "Extract archives into an isolated directory and inspect/apply the scoped delta there; do not overwrite the dirty worktree blindly.",
        },
    }
    manifest_path = OUTPUT / "MANIFEST.json"
    manifest_path.write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    manifest_hash = digest(manifest_path)
    (OUTPUT / "MANIFEST.sha256").write_text(
        f"{manifest_hash}  MANIFEST.json\n", encoding="utf-8"
    )
    HANDOFF.write_text(
        handoff_text(manifest_hash, source_changes, len(evidence)), encoding="utf-8"
    )
    HANDOFF_HASH.write_text(
        f"{digest(HANDOFF)}  {HANDOFF.name}\n", encoding="utf-8"
    )
    print(manifest_hash)


if __name__ == "__main__":
    main()
