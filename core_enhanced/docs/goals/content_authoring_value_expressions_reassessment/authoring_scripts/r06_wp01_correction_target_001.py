#!/usr/bin/env python3
"""Freeze the WP01 F01/F02 correction target and generate its handoff."""

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
SEGMENT = EXECUTION / "segments/wp01_content_input_compile_001"
REVIEW = EXECUTION / "reviews/wp01_content_input_compile_001_review_001"
CORRECTION = EXECUTION / "corrections/wp01_target_001_f01_f02"
BASELINE = ROOT / CORRECTION / "baseline/pre_fix_001"
PREDECESSOR = ROOT / SEGMENT / "target/target_001"
OUTPUT = ROOT / CORRECTION / "target/correction_001"
HANDOFF = ROOT / CORRECTION / "CORRECTION_REVIEW_HANDOFF.md"
HANDOFF_HASH = HANDOFF.with_suffix(".md.sha256")
SOURCE_ARCHIVE = OUTPUT / "SOURCE_AFTER_WP01_F01_F02_CORRECTION_001.tar.gz"
RECORD_ARCHIVE = OUTPUT / "RECORDS_AT_CORRECTION_TARGET_001.tar.gz"
EVIDENCE_ARCHIVE = OUTPUT / "EVIDENCE_AT_CORRECTION_TARGET_001.tar.gz"
SOURCE_DELTA = OUTPUT / "SOURCE_DELTA.patch"

EXPECTED_BASELINE_HASH = "a431db07b318c56f572b1799f9ce186903ecc667d3bddb898fbf78f8c7eb635e"
EXPECTED_PREDECESSOR_HASH = "6f55cfa0fae0d88971eba6cb3c47e39ab5d12a2e99b2656a40eb2b51e7cf76de"
EXPECTED_SOURCE_CHANGES = {
    "core_enhanced/src/game/data/content_pack/tests.rs",
    "core_enhanced/src/game/value_expr/catalog.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/parse.rs",
}

RECORDS = [
    CORRECTION / "PLAN.md",
    CORRECTION / "EXPERIMENTS.md",
    CORRECTION / "EXPERIMENT_NOTES.md",
    TOPIC / "authoring_scripts/r06_wp01_correction_run_validation.py",
    TOPIC / "authoring_scripts/r06_wp01_correction_target_001.py",
    TOPIC / "authoring_scripts/r06_wp01_verify_correction_target_001.py",
]

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
EXPECTED_FAILURES = {"pre_fix_review_probe_reproduced": 101}
INVALID_ATTEMPTS = {"pre_fix_review_probe": 101}
SUPPORTING_SUCCESS = {
    "pre_fix_build",
    "pre_fix_review_probe_compile",
    "intermediate_value_expr_unit",
    "intermediate_static_fault_shared_evaluator",
    "intermediate_strict_parse_source_ids",
    "intermediate_external_contract_regression",
    "post_fix_build",
    "post_fix_review_probe_compile",
    "post_fix_review_probe",
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
    old = {item["path"]: item for item in before}
    new = {item["path"]: item for item in after}
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
    paths = [Path(str(entry["path"])) for entry in entries]
    with tarfile.open(destination, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for relative in paths:
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    result = row(destination.relative_to(ROOT))
    result["file_count"] = len(entries)
    validate_archive(result, entries)
    return result


def write_delta(
    before_archive: Path, changes: list[dict[str, object]]
) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="r06-wp01-f01-f02-target-") as raw:
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
            "--src-prefix=pre_fix_001/",
            "--dst-prefix=correction_001/",
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
    validation_root = ROOT / CORRECTION / "evidence/validation"
    receipt_paths = sorted(validation_root.glob("*.json"))
    expected_labels = (
        REQUIRED_SUCCESS
        | SUPPORTING_SUCCESS
        | set(EXPECTED_FAILURES)
        | set(INVALID_ATTEMPTS)
    )
    if {path.stem for path in receipt_paths} != expected_labels:
        raise SystemExit("validation receipt set mismatch")
    current_by_path = {item["path"]: item for item in current_source}
    baseline_by_path = {item["path"]: item for item in baseline["source"]["files"]}
    results = []
    for path in receipt_paths:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        label = receipt["label"]
        verify_row(receipt["output"])
        if label in EXPECTED_FAILURES:
            expected_exit = EXPECTED_FAILURES[label]
            role = "expected_pre_fix_finding_failure"
        elif label in INVALID_ATTEMPTS:
            expected_exit = INVALID_ATTEMPTS[label]
            role = "invalid_fixture_setup_preserved_not_counted"
        elif label in REQUIRED_SUCCESS:
            expected_exit = 0
            role = "required_direct_success"
        else:
            expected_exit = 0
            role = "supporting_direct_success"
        if receipt["exit_code"] != expected_exit:
            raise SystemExit(f"unexpected validation exit: {label}")
        identity_basis = baseline_by_path if label.startswith("pre_fix_") else current_by_path
        if label in REQUIRED_SUCCESS or label.startswith("pre_fix_"):
            for identity in receipt["source_identity"]:
                expected = identity_basis.get(identity["path"])
                if expected is None or identity != expected:
                    raise SystemExit(f"validation source identity mismatch: {label} {identity['path']}")
        results.append(
            {
                "label": label,
                "role": role,
                "exit_code": expected_exit,
                "command": receipt["command"],
                "receipt": row(path.relative_to(ROOT)),
                "output": receipt["output"],
            }
        )
    required_logs = {
        "post_fix_value_expr_unit": "36 passed; 0 failed",
        "post_fix_content_pipeline_regression": "59 passed; 0 failed",
        "post_fix_authoring_regression": "15 passed; 0 failed",
        "post_fix_skill_runtime_regression": "15 passed; 0 failed",
        "post_fix_checkpoint_save_regression": "18 passed; 0 failed",
        "post_fix_final_review_probe": "3 passed; 0 failed; 1 ignored",
        "post_fix_parser_depth_probe": "1 passed; 0 failed",
    }
    by_label = {item["label"]: item for item in results}
    for label, text in required_logs.items():
        output = (ROOT / str(by_label[label]["output"]["path"])).read_text(
            encoding="utf-8", errors="replace"
        )
        if text not in output:
            raise SystemExit(f"validation summary missing: {label}")
    return results


def handoff_text(
    manifest_hash: str, baseline_hash: str, changes: list[dict[str, object]]
) -> str:
    change_lines = "\n".join(
        f"- `{item['path']}`: `{item['before_sha256']}` → `{item['after_sha256']}`"
        for item in changes
    )
    return f"""# R06 WP01 F01/F02 교정 재검토 인계

Status: `교정·자체 검증 종료 / 교정 타깃 동결 / 재검토 요청 대기`

이 인계는 `WP01-F01`, `WP01-F02`의 교정과 자체 검증 결과다. 자체 테스트 성공을 재검토 PASS로 표시하지 않으며 WP02, 다음 R06 기능, GEX-20/30, WP/Phase/Goal 완료를 시작하거나 선언하지 않는다.

## Before / after identity

- Before target: `{(PREDECESSOR / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- Before target SHA-256: `{EXPECTED_PREDECESSOR_HASH}`
- Correction baseline: `{(BASELINE / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- Correction baseline SHA-256: `{baseline_hash}`
- After correction target: `{(OUTPUT / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- After correction target SHA-256: `{manifest_hash}`

기존 target, baseline, review 보고서·probe·실패 evidence는 덮어쓰지 않았다. before/after source actual bytes, 기록/evidence actual bytes와 binary-capable source delta를 새 correction 계보에 보존했다.

## WP01-F01

- 원인: custom `WireValueExpr` enum에 strict unknown-field 설정이 없어 Mul/Ratio/Clamp/Aggregate의 미지원 field가 Serde에서 폐기됐다. `Read`가 사용하는 `ValueRef` struct variant에도 같은 누락이 있었다.
- 변경: `WireValueExpr`와 `ValueRef`에 `deny_unknown_fields`를 적용했다. 이미 strict한 AmountExpr, TypedInteger, AuthoredMember, GroupOwner, ProjectedTile과 상위 effect authoring 타입은 유지했다. 전체 RON 재parse나 사전 AST clone을 추가하지 않았다.
- 검증: 주요 expression struct variant, Stat/HitPoints/Resource/Result/AggregateMember payload, AuthoredMember nested payload와 정상 대조군을 unit regression으로 추가했다. 동일 review probe의 direct parser와 새 외부 pack fixture 모두 typo Mul을 `ron_parse`로 거부하고 source identifiers를 유지했다.

## WP01-F02

- 원인: `validate_result_contract`가 reference set이 비어 있을 때만 평가해 authored source가 직접 공급하는 `AggregateMember`도 dynamic으로 오분류했다.
- 변경: `ResolvedTargets`가 없고 references가 정확히 local `AggregateMember`뿐인 plan을 external-context-free로 분류한다. 해당 plan만 기존 bounded `evaluate`와 기존 Damage/Heal narrowing으로 검증한다. Stat/HitPoints member context, source stat, resource/result/time 및 resolved targets는 static 값으로 대체하지 않는다.
- 검증: authored `-1` Sum Damage와 Heal `i32` 초과는 `value_expr_overflow`, AggregateMember 0 분모는 shared evaluator의 `value_expr_divide_by_zero`로 거부됐다. 정상/중첩 authored aggregate는 통과했고 source stat, member stat, resolved-target aggregate는 compile 후 기존 `content_capability_unsupported` publication gate에 남았다. Damage overflow의 실제 source file/definition/action/effect/field와 외부 pack 식별자를 확인했다.

## 변경 source

{change_lines}

## 직접 실행 검증

- 수정 전: build/probe compile PASS, 원 assertion 재현은 1 pass / 2 expected finding failures / 1 ignored, exit 101.
- 수정 후: ValueExpr 36/36, content pipeline 59/59, authoring 15/15, skill runtime 15/15, checkpoint/save 18/18 PASS.
- 동일 review probe 최종 실행: 3 pass / 0 fail / 1 ignored. 별도 depth probe 1/1 PASS.
- all-target check, focused rustfmt PASS.
- Clippy는 predecessor와 동일한 command-line allow `clippy::never_loop`, `clippy::eq_op`만 사용해 exit 0. 기존 warning은 raw log에 보존했다.
- 첫 새 fixture 실행은 부모 디렉터리 setup 오류로 exit 101이었다. invalid attempt로 삭제하지 않고 보존했으며 결과 수에는 포함하지 않았다.

## 재사용 검증

- 교정 진입에서 predecessor target의 구현자 verifier와 reviewer 독립 verifier를 재사용해 source/archive/reference currentness를 확인했다. 441 reference hash, source 313개, records/evidence archive가 일치했다.
- predecessor의 기존 test receipts는 해시로 보존·확인했으나 이번 직접 실행 test 수에 합산하지 않았다.

## 잔여 위험과 경계

- strict regression은 현재 승인된 authored surface를 다루며 전체 미래 RON 입력 공간의 독립 재감사를 대신하지 않는다.
- `AggregateMember` local 분류는 compiler가 보장하는 lexical aggregate scope와 `resolved_targets` requirement에 의존한다. 해당 compiler/fingerprint/cost 의미는 수정하지 않았다.
- WP02 runtime evaluator는 여전히 미연결이며 authored expression publication은 계속 `content_capability_unsupported`로 차단된다. 이는 이번 correction의 의도된 잔여 경계다.

이 target을 fresh 단계 재검토 입력으로 사용한다. 재검토 결과 전에는 correction source·records·evidence·target을 변경하지 않는다.
"""


def main() -> None:
    if OUTPUT.exists() or HANDOFF.exists() or HANDOFF_HASH.exists():
        raise SystemExit("correction target or handoff already exists")
    baseline = validate_detached(BASELINE, EXPECTED_BASELINE_HASH)
    predecessor = validate_detached(PREDECESSOR, EXPECTED_PREDECESSOR_HASH)

    for section in (
        "source",
        "existing_execution_documents",
        "review_inputs",
        "predecessor_target_records",
    ):
        value = baseline[section]
        validate_archive(value["archive"], value["files"])
    for entry in baseline["predecessor_target_evidence"]["files"]:
        verify_row(entry)
    for entry in baseline["review_inputs"]["files"]:
        verify_row(entry)
    for entry in baseline["existing_execution_documents"]["files"]:
        verify_row(entry)
    if predecessor["record"] != "R06_WP01_CONTENT_INPUT_COMPILE_TARGET_001":
        raise SystemExit("unexpected predecessor record")

    expected_paths = {item["path"] for item in baseline["source"]["files"]}
    actual_paths = source_paths(baseline["source"]["scope"])
    if actual_paths != expected_paths:
        raise SystemExit("source file set changed outside the correction")
    current_source = [row(Path(path)) for path in sorted(actual_paths)]
    source_changes = changed_rows(baseline["source"]["files"], current_source)
    if {item["path"] for item in source_changes} != EXPECTED_SOURCE_CHANGES:
        raise SystemExit(f"unexpected source delta: {source_changes}")
    if any(item["state"] != "modified" for item in source_changes):
        raise SystemExit("source correction must only modify existing files")

    validations = validation_rows(current_source, baseline)
    evidence_paths = sorted(
        (
            path.relative_to(ROOT)
            for path in (ROOT / CORRECTION / "evidence").rglob("*")
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
    scoped_path = OUTPUT / "SCOPED_GIT_STATUS_AFTER_FIX.txt"
    scoped_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    global_path = OUTPUT / "GLOBAL_GIT_STATUS_AFTER_FIX.txt"
    global_path.write_bytes(global_status)

    manifest = {
        "record": "R06_WP01_CONTENT_INPUT_COMPILE_CORRECTION_TARGET_001",
        "state": "correction_self_validation_complete_target_frozen_re_review_pending",
        "status_text": "교정·자체 검증 종료 / 교정 타깃 동결 / 재검토 요청 대기",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").stdout.decode().strip(),
        "head": run("git", "rev-parse", "HEAD").stdout.decode().strip(),
        "goal": baseline["goal"],
        "handoff": baseline["handoff"],
        "pipeline_revision": baseline["pipeline_revision"],
        "correction_scope": ["WP01-F01", "WP01-F02"],
        "completion_claims": {
            "correction_self_validation_complete": True,
            "correction_target_frozen": True,
            "review_pass_declared": False,
            "wp01_complete": False,
            "wp02_started": False,
            "next_r06_feature_started": False,
            "gex_20_started": False,
            "gex_30_started": False,
            "phase_complete": False,
            "goal_complete": False,
        },
        "predecessor_target": {
            "record": predecessor["record"],
            "manifest": (PREDECESSOR / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": EXPECTED_PREDECESSOR_HASH,
            "frozen_artifacts_unchanged": True,
            "live_source_expectedly_superseded_by_this_correction": True,
        },
        "pre_fix_baseline": {
            "record": baseline["record"],
            "manifest": (BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": EXPECTED_BASELINE_HASH,
            "source_archive": baseline["source"]["archive"],
            "existing_execution_document_archive": baseline[
                "existing_execution_documents"
            ]["archive"],
            "review_input_archive": baseline["review_inputs"]["archive"],
        },
        "preservation": {
            "predecessor_target_manifest_unchanged": True,
            "predecessor_target_records": baseline["predecessor_target_records"]["files"],
            "predecessor_target_evidence": baseline["predecessor_target_evidence"]["files"],
            "review_inputs": baseline["review_inputs"]["files"],
            "existing_execution_documents": baseline[
                "existing_execution_documents"
            ]["files"],
            "original_game_resources_modified": False,
        },
        "current_source": {
            "scope": scope,
            "file_count": len(current_source),
            "files": current_source,
            "archive": source_archive,
            "changes_from_pre_fix": {
                "added": 0,
                "modified": len(source_changes),
                "deleted": 0,
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
            "entry_state": "new correction records were absent at baseline",
        },
        "evidence": {
            "file_count": len(evidence),
            "files": evidence,
            "archive": evidence_archive,
        },
        "validation_receipts": validations,
        "validation_summary": {
            "pre_fix_reproduction": "1 passed / 2 expected finding failures / 1 ignored; exit 101",
            "post_fix_value_expr": "36 passed",
            "post_fix_content_pipeline": "59 passed",
            "post_fix_authoring": "15 passed",
            "post_fix_skill_runtime": "15 passed",
            "post_fix_checkpoint_save": "18 passed",
            "post_fix_review_probe": "3 passed / 0 failed / 1 ignored",
            "post_fix_depth_probe": "1 passed",
            "all_targets_check": "exit 0",
            "focused_rustfmt": "exit 0",
            "clippy": "exit 0 with only predecessor allows clippy::never_loop and clippy::eq_op",
            "direct_vs_reused": "all listed correction receipts are direct; predecessor target/review evidence was hash/currentness-reused and not counted",
        },
        "findings": {
            "WP01-F01": {
                "cause": "WireValueExpr and ValueRef struct variants did not reject unknown fields",
                "change": "Serde deny_unknown_fields on both authored wire enums plus direct/external regressions",
                "result": "typo_parameter rejected as ron_parse before publication",
            },
            "WP01-F02": {
                "cause": "static validation equated any reference with external runtime state",
                "change": "only authored-local AggregateMember requirements without ResolvedTargets use the existing bounded evaluator and final narrowing",
                "result": "authored -1 Sum Damage and out-of-range Heal reject; genuinely dynamic controls remain publication-blocked",
            },
        },
        "residual_risks": [
            "fresh independent stage re-review is still required",
            "WP02 runtime evaluation remains unconnected by design",
            "future authored variants must preserve the strict-field and external-context classification contracts",
        ],
        "review_entry": {
            "path": HANDOFF.relative_to(ROOT).as_posix(),
            "detached_sha256": HANDOFF_HASH.relative_to(ROOT).as_posix(),
            "binding": "Generated after MANIFEST.json; verifier checks detached hash and exact manifest hash text.",
        },
        "mutation_boundary": "Do not mutate correction source, records, evidence, handoff, or target after final read-only currentness verification until re-review returns.",
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
        handoff_text(manifest_hash, EXPECTED_BASELINE_HASH, source_changes),
        encoding="utf-8",
    )
    HANDOFF_HASH.write_text(
        f"{digest(HANDOFF)}  {HANDOFF.name}\n", encoding="utf-8"
    )
    print(manifest_hash)


if __name__ == "__main__":
    main()
