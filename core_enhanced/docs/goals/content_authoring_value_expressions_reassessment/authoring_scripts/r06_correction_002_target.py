#!/usr/bin/env python3
"""Freeze the R06 correction-001 R01-R03 successor target for re-review."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
from datetime import datetime, timezone


ROOT = Path(__file__).resolve().parents[5]
TOPIC = Path("core_enhanced/docs/goals/content_authoring_value_expressions_reassessment")
EXECUTION = TOPIC / "execution/single_goal"
CORRECTION = EXECUTION / "corrections/correction_001_r01_r03"
BASELINE = ROOT / CORRECTION / "baseline/pre_fix_002"
PREDECESSOR = ROOT / EXECUTION / "corrections/checkpoint_001_f01_f04/target/correction_001"
PREDECESSOR_BASELINE = ROOT / EXECUTION / "corrections/checkpoint_001_f01_f04/baseline/pre_fix_001"
OUTPUT = ROOT / CORRECTION / "target/correction_002"
HANDOFF = ROOT / CORRECTION / "CORRECTION_REVIEW_HANDOFF.md"
HANDOFF_HASH = ROOT / CORRECTION / "CORRECTION_REVIEW_HANDOFF.sha256"

SOURCE_ARCHIVE = OUTPUT / "SOURCE_AFTER_CORRECTION_002.tar.gz"
SOURCE_DELTA = OUTPUT / "SOURCE_CORRECTION_002_DELTA.patch"
DOCUMENT_ARCHIVE = OUTPUT / "DOCUMENTS_AFTER_CORRECTION_002.tar.gz"
DOCUMENT_DELTA = OUTPUT / "DOCUMENT_CORRECTION_002_DELTA.patch"
EVIDENCE_ARCHIVE = OUTPUT / "EVIDENCE_AFTER_CORRECTION_002.tar.gz"

EXPECTED_SOURCE_CHANGES = {
    "core_enhanced/src/game/value_expr/compile.rs",
    "core_enhanced/src/game/value_expr/eval.rs",
    "core_enhanced/src/game/value_expr/mod.rs",
    "core_enhanced/src/game/value_expr/tests.rs",
}

DOCUMENTS = [
    CORRECTION / "PLAN.md",
    CORRECTION / "CORRECTION_NOTES.md",
    CORRECTION / "EXPERIMENTS.md",
]

RECORDS = [
    *DOCUMENTS,
    TOPIC / "authoring_scripts/r06_correction_002_baseline.py",
    TOPIC / "authoring_scripts/r06_run_correction_002_validation.py",
    TOPIC / "authoring_scripts/r06_recover_historical_correction_plan.py",
    TOPIC / "authoring_scripts/r06_correction_002_target.py",
    TOPIC / "authoring_scripts/r06_verify_correction_002_target.py",
]

SUCCESS_LABELS = {
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
    "pre_fix_reaudit_probe": {
        "exit_code": 101,
        "classification": "pre-fix R01 reproduction: 8 pass / 2 fail / 1 ignored",
    },
    "pre_fix_deep_shape_probe": {
        "exit_code": -6,
        "classification": "pre-fix R02 reproduction: 2 MiB worker stack overflow/SIGABRT with AST drop disabled",
    },
    "intermediate_unit_invalid_resource_fixture": {
        "exit_code": 101,
        "classification": "test-fixture defect: ResourceId exceeded its 128-byte contract; corrected to 100 bytes before passing rerun",
    },
}


def run(*args: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        args,
        cwd=ROOT,
        check=check,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def byte_digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def file_row(relative: Path) -> dict[str, object]:
    absolute = ROOT / relative
    if not absolute.is_file():
        raise SystemExit(f"missing required file: {relative.as_posix()}")
    return {
        "path": relative.as_posix(),
        "sha256": digest(absolute),
        "bytes": absolute.stat().st_size,
    }


def rows(paths: list[Path]) -> list[dict[str, object]]:
    return [file_row(path) for path in paths]


def verify_row(expected: dict[str, object]) -> None:
    current = file_row(Path(expected["path"]))
    if any(current[key] != expected[key] for key in ("path", "sha256", "bytes")):
        raise SystemExit(f"frozen file changed: {expected['path']}")


def validate_detached(directory: Path) -> tuple[dict[str, object], str]:
    manifest_path = directory / "MANIFEST.json"
    manifest_hash = digest(manifest_path)
    detached = (directory / "MANIFEST.sha256").read_text(encoding="utf-8").split()[0]
    if manifest_hash != detached:
        raise SystemExit(f"detached hash mismatch: {manifest_path.relative_to(ROOT)}")
    return json.loads(manifest_path.read_text(encoding="utf-8")), manifest_hash


def validate_archive(archive_path: Path, expected: list[dict[str, object]]) -> None:
    with tarfile.open(archive_path, "r:gz") as archive:
        members = {member.name: member for member in archive.getmembers() if member.isfile()}
        if set(members) != {entry["path"] for entry in expected}:
            raise SystemExit(f"archive member set mismatch: {archive_path.relative_to(ROOT)}")
        for entry in expected:
            extracted = archive.extractfile(members[entry["path"]])
            if extracted is None:
                raise SystemExit(f"archive member unreadable: {entry['path']}")
            data = extracted.read()
            if len(data) != entry["bytes"] or byte_digest(data) != entry["sha256"]:
                raise SystemExit(f"archive member mismatch: {entry['path']}")


def write_archive(path: Path, entries: list[dict[str, object]]) -> dict[str, object]:
    with tarfile.open(path, "w:gz", format=tarfile.PAX_FORMAT) as archive:
        for entry in entries:
            relative = Path(entry["path"])
            archive.add(ROOT / relative, arcname=relative.as_posix(), recursive=False)
    validate_archive(path, entries)
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "sha256": digest(path),
        "bytes": path.stat().st_size,
        "file_count": len(entries),
    }


def source_paths(scope: list[str]) -> set[str]:
    found: set[str] = set()
    for item in scope:
        absolute = ROOT / item
        if absolute.is_file():
            found.add(Path(item).as_posix())
        elif absolute.is_dir():
            found.update(
                path.relative_to(ROOT).as_posix()
                for path in absolute.rglob("*")
                if path.is_file()
            )
        else:
            raise SystemExit(f"source scope path missing: {item}")
    return found


def changed_rows(
    before: list[dict[str, object]], current: list[dict[str, object]]
) -> list[dict[str, object]]:
    after_by_path = {entry["path"]: entry for entry in current}
    changes = []
    for old in before:
        new = after_by_path[old["path"]]
        if old == new:
            continue
        changes.append(
            {
                "path": old["path"],
                "state": "modified",
                "before_sha256": old["sha256"],
                "before_bytes": old["bytes"],
                "after_sha256": new["sha256"],
                "after_bytes": new["bytes"],
            }
        )
    return changes


def write_delta(
    before_archive: Path,
    changes: list[dict[str, object]],
    output: Path,
    before_prefix: str,
    after_prefix: str,
) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="r06-correction-002-") as raw_temp:
        temporary = Path(raw_temp)
        extracted = temporary / "extracted"
        before_changed = temporary / "before"
        after_changed = temporary / "after"
        extracted.mkdir()
        before_changed.mkdir()
        after_changed.mkdir()
        with tarfile.open(before_archive, "r:gz") as archive:
            archive.extractall(extracted, filter="data")
        for change in changes:
            relative = Path(change["path"])
            old_destination = before_changed / relative
            old_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(extracted / relative, old_destination)
            new_destination = after_changed / relative
            new_destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, new_destination)
        completed = run(
            "git",
            "diff",
            "--no-index",
            "--binary",
            f"--src-prefix={before_prefix}/",
            f"--dst-prefix={after_prefix}/",
            str(before_changed),
            str(after_changed),
            check=False,
        )
        if completed.returncode not in (0, 1):
            raise RuntimeError(completed.stderr.decode(errors="replace"))
        output.write_bytes(completed.stdout)
    return file_row(output.relative_to(ROOT))


def validation_rows() -> list[dict[str, object]]:
    validation_root = ROOT / CORRECTION / "evidence/validation"
    receipt_paths = sorted(validation_root.glob("*.json"))
    expected_labels = SUCCESS_LABELS | set(EXPECTED_FAILURES)
    actual_labels = {path.stem for path in receipt_paths}
    if actual_labels != expected_labels:
        raise SystemExit(
            f"validation receipt set mismatch: missing={sorted(expected_labels - actual_labels)} "
            f"unexpected={sorted(actual_labels - expected_labels)}"
        )
    receipts = []
    for path in receipt_paths:
        receipt = json.loads(path.read_text(encoding="utf-8"))
        label = receipt["label"]
        verify_row(receipt["output"])
        if label in SUCCESS_LABELS:
            exit_code = 0
            classification = "pass"
        else:
            exit_code = EXPECTED_FAILURES[label]["exit_code"]
            classification = EXPECTED_FAILURES[label]["classification"]
        if receipt["exit_code"] != exit_code:
            raise SystemExit(f"unexpected validation exit: {label}")
        receipts.append(
            {
                "label": label,
                "exit_code": exit_code,
                "classification": classification,
                "command": receipt["command"],
                "receipt": path.relative_to(ROOT).as_posix(),
                "receipt_sha256": digest(path),
                "output": receipt["output"],
            }
        )
    return receipts


def handoff_text(manifest_hash: str, baseline_hash: str, changes: list[dict[str, object]]) -> str:
    change_lines = "\n".join(
        f"- `{entry['path']}`: `{entry['before_sha256']}` → `{entry['after_sha256']}`"
        for entry in changes
    )
    return f"""# R06 correction 001 R01–R03 재검토 인계

Status: `교정본 재검토 준비 완료 / 재검토 요청 대기`

이번 인계는 R01–R03 교정, 자체 검증, successor target 동결까지만 다룬다. 다음 R06 기능, GEX-20/30, WP/Phase/Goal 완료를 선언하지 않는다.

## 동결 경계

- 수정 전 baseline: `R06_CHECKPOINT_001_CORRECTION_BASELINE_002`
- baseline manifest: `{(BASELINE / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- baseline SHA-256: `{baseline_hash}`
- 수정 후 target: `R06_CHECKPOINT_001_CORRECTION_TARGET_002`
- target manifest: `{(OUTPUT / 'MANIFEST.json').relative_to(ROOT).as_posix()}`
- target SHA-256: `{manifest_hash}`
- source before/after actual-byte archive와 binary-capable source delta, 변경 문서 before/after actual-byte archive와 delta, validation evidence archive를 target에 보존했다.

## R01

compile/evaluate가 member scan과 sort work helper를 공유한다. AuthoredMembers의 raw count, identity/owner/value payload, sort/group/reduction 및 중첩 반복을 checked arithmetic으로 plan에 합성한다. 정적 plan은 같은 `static_work` meter에서 실행 가능하며, compile 시 알 수 없는 frame/resolved payload는 `runtime_dynamic_work`와 승인 ceiling으로 분리했다.

검증은 재감사의 1-member 6/7 실패가 7/7 성공으로 바뀐 것, authored 15개 조합의 bound==actual, empty/long/group/nested 및 bound-1/bound/bound+1을 포함한다.

## R02

recursive `segment_shape`를 dependency 순서의 iterative bottom-up 분석으로 교체하고 scratch storage를 할당 전에 admission했다. 동시 value stack과 task frame 의미를 유지했으며 새 hard cap이나 worker stack 증가는 없다.

기본 depth 64/65, 확장 depth 512, 재감사 depth 2,048/2 MiB worker subprocess를 검사했다. deep probe는 `ManuallyDrop`으로 AST 파괴 재귀를 분리하며 수정 후 정상 typed 결과로 종료한다.

## R03

요구된 과거 PLAN을 실제 bytes로 복구했다: `evidence/r03/HISTORICAL_PLAN_PRE_FIX_001.md`, 4,460 bytes, SHA-256 `661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7`. 원본은 Codex session JSONL line 1075의 apply-patch result이며 raw event와 recovery record도 보존했다.

predecessor baseline은 수정하지 않았다. 당시 실제 PLAN bytes를 archive하지 못한 역사적 결함과 successor에서의 원본 복구를 별도 세대로 표시한다. verifier도 current live PLAN과 recovered historical snapshot을 분리한다.

## 변경 파일

제품 source는 다음 네 파일만 변경됐다.

{change_lines}

기록·보존 도구 변경은 `{CORRECTION.as_posix()}`와 동일 goal의 `r06_correction_002_*`, `r06_run_correction_002_validation.py`, `r06_recover_historical_correction_plan.py`로 한정했다.

## 검증

이번 pass 실행:

- value_expr unit 22/22, predecessor correction probe 7/7
- 재감사 probe 10/10와 격리 deep probe 1/1
- all-target check PASS; all-target test lib 1,382 + integration 52 PASS
- focused rustfmt PASS
- Clippy PASS with predecessor baseline allow `clippy::never_loop`, `clippy::eq_op`
- pre-fix archive와 corrected source fingerprint 3/3 byte-identical

재사용:

- predecessor target source/record/evidence/archive currentness hash 검증
- predecessor의 Unix socket EPERM 환경 분류; 이번 전체 test는 승인된 환경에서 직접 PASS

보존된 실패는 pre-fix R01 exit 101, pre-fix R02 SIGABRT(-6), 구현 중 invalid ResourceId test-fixture exit 101이다. 마지막 실패는 유효한 100-byte fixture로 교정한 뒤 unit과 전체 suite를 통과했다.

## 남은 제약

- runtime-dynamic payload는 compile 시 알 수 없어 runtime meter가 제한한다.
- retained-byte 모델은 보수적 kernel-owned allocation model이며 RSS 보장이 아니다.
- predecessor baseline이 생성 당시 PLAN 실제 bytes를 누락한 사실은 유지된다. successor evidence가 원본을 복구했으며 과거 baseline을 소급 PASS로 바꾸지 않는다.
- 실제 lease 검증과 다음 R06 기능은 범위 밖이다.

이 target과 인계서를 재검토 입력으로 사용한다. 재검토 결과가 돌아오기 전까지 correction source·records·evidence를 변경하지 않는다.
"""


def main() -> None:
    if OUTPUT.exists() or HANDOFF.exists() or HANDOFF_HASH.exists():
        raise SystemExit("correction-002 target or handoff already exists; frozen output will not be overwritten")

    baseline, baseline_hash = validate_detached(BASELINE)
    predecessor, predecessor_hash = validate_detached(PREDECESSOR)
    old_baseline, old_baseline_hash = validate_detached(PREDECESSOR_BASELINE)
    if predecessor_hash != "87feab8cdfc20b43d7c49c2f6238bd1e684cb7e1daf4f993efe2f193f01dc45e":
        raise SystemExit("unexpected predecessor target hash")
    if baseline_hash != "96eb47695ea346627b5c0c6aedbd75953cf496353ad00b03b04d37bd80232bda":
        raise SystemExit("unexpected correction-002 baseline hash")

    for entry in predecessor["execution_records"]:
        verify_row(entry)
    for entry in predecessor["evidence"]:
        verify_row(entry)
    for key in ("archive", "recoverable_delta", "scoped_status", "global_status"):
        verify_row(predecessor["current_source"][key])
    validate_archive(
        ROOT / predecessor["current_source"]["archive"]["path"],
        predecessor["current_source"]["files"],
    )
    for entry in baseline["review_inputs"]["files"]:
        verify_row(entry)
    for section, archive_key in (
        ("source", "archive"),
        ("mutable_documents", "archive"),
        ("predecessor_execution_records", "archive"),
        ("review_inputs", "archive"),
    ):
        entry = baseline[section]
        verify_row(entry[archive_key])
        validate_archive(ROOT / entry[archive_key]["path"], entry["files"])

    expected_source_paths = {entry["path"] for entry in baseline["source"]["files"]}
    actual_source_paths = source_paths(baseline["source"]["scope"])
    if actual_source_paths != expected_source_paths:
        raise SystemExit(
            f"source file set drift: added={sorted(actual_source_paths - expected_source_paths)} "
            f"deleted={sorted(expected_source_paths - actual_source_paths)}"
        )
    current_source = rows([Path(path) for path in sorted(actual_source_paths)])
    source_changes = changed_rows(baseline["source"]["files"], current_source)
    if {entry["path"] for entry in source_changes} != EXPECTED_SOURCE_CHANGES:
        raise SystemExit(f"unexpected source delta: {source_changes}")

    current_documents = rows(DOCUMENTS)
    document_changes = changed_rows(baseline["mutable_documents"]["files"], current_documents)
    if {entry["path"] for entry in document_changes} != {path.as_posix() for path in DOCUMENTS}:
        raise SystemExit("all three mutable correction documents must have explicit before/after bytes")

    validations = validation_rows()
    evidence_paths = sorted(
        (path.relative_to(ROOT) for path in (ROOT / CORRECTION / "evidence").rglob("*") if path.is_file()),
        key=lambda path: path.as_posix(),
    )
    evidence = rows(evidence_paths)
    execution_records = rows(RECORDS)

    entry_currentness_path = CORRECTION / "evidence/currentness/ENTRY_CURRENTNESS.json"
    entry_currentness = json.loads((ROOT / entry_currentness_path).read_text(encoding="utf-8"))
    if entry_currentness["current_target_reference_hashes_passed"] is not True:
        raise SystemExit("entry currentness did not verify current predecessor target")
    if len(entry_currentness["errors"]) != 1 or entry_currentness["errors"][0]["expected"] != "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7":
        raise SystemExit("entry currentness contains unexplained drift")

    recovery_path = CORRECTION / "evidence/r03/R03_HISTORICAL_PLAN_RECOVERY.json"
    recovery = json.loads((ROOT / recovery_path).read_text(encoding="utf-8"))
    recovered = recovery["recovered_copy"]
    verify_row(recovered)
    if recovered["bytes"] != 4460 or recovered["sha256"] != "661dd456d923e323cfd5704dd520e322251ba9a603303a4b73c295e6c13412c7":
        raise SystemExit("R03 recovered PLAN identity mismatch")
    raw_event = recovery["provenance"]["raw_event"]
    verify_row(raw_event)
    event = json.loads((ROOT / raw_event["path"]).read_text(encoding="utf-8"))
    old_plan_path = str(ROOT / Path(recovery["predecessor_reference"]["path"]))
    content = event["payload"]["item"]["changes"][old_plan_path]["content"].encode()
    if len(content) != 4460 or byte_digest(content) != recovered["sha256"]:
        raise SystemExit("R03 provenance does not reproduce recovered PLAN bytes")
    if (ROOT / recovered["path"]).read_bytes() != content:
        raise SystemExit("R03 recovered copy differs from provenance content")
    if old_baseline["correction_plan"]["sha256"] != recovered["sha256"]:
        raise SystemExit("R03 recovery does not bind the predecessor baseline reference")

    OUTPUT.mkdir(parents=True, exist_ok=False)
    source_archive = write_archive(SOURCE_ARCHIVE, current_source)
    document_archive = write_archive(DOCUMENT_ARCHIVE, current_documents)
    evidence_archive = write_archive(EVIDENCE_ARCHIVE, evidence)
    source_delta = write_delta(
        ROOT / baseline["source"]["archive"]["path"],
        source_changes,
        SOURCE_DELTA,
        "pre_fix_002",
        "correction_002",
    )
    document_delta = write_delta(
        ROOT / baseline["mutable_documents"]["archive"]["path"],
        document_changes,
        DOCUMENT_DELTA,
        "pre_fix_002",
        "correction_002",
    )

    scope = baseline["source"]["scope"]
    scoped_status = run("git", "status", "--porcelain=v1", "--untracked-files=all", "--", *scope).stdout
    scoped_status_path = OUTPUT / "SCOPED_GIT_STATUS_AFTER_CORRECTION_002.txt"
    scoped_status_path.write_bytes(scoped_status)
    global_status = run("git", "status", "--porcelain=v1", "--untracked-files=all").stdout
    global_status_path = OUTPUT / "GLOBAL_GIT_STATUS_AFTER_CORRECTION_002.txt"
    global_status_path.write_bytes(global_status)

    manifest = {
        "record": "R06_CHECKPOINT_001_CORRECTION_TARGET_002",
        "state": "corrected_re_review_requested",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "repository": str(ROOT),
        "branch": run("git", "branch", "--show-current").stdout.decode().strip(),
        "head": run("git", "rev-parse", "HEAD").stdout.decode().strip(),
        "handoff": "HA-CAVER-P2G60-001",
        "goal": "GS-CAVER-P2G50-001 / GOAL-CAVER-01 / PHASE-CAVER-01",
        "pipeline_revision": "2026-09-13-gex-r7",
        "correction_scope": ["R01", "R02", "R03", "same-cause preservation tools"],
        "completion_claims": {
            "correction_re_review_ready": True,
            "next_r06_feature_started": False,
            "gex_20_started": False,
            "gex_30_started": False,
            "work_package_complete": False,
            "phase_complete": False,
            "goal_complete": False,
        },
        "predecessor_target": {
            "record": predecessor["record"],
            "manifest": (PREDECESSOR / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": predecessor_hash,
            "detached_hash_matches": True,
        },
        "predecessor_baseline": {
            "record": old_baseline["record"],
            "manifest": (PREDECESSOR_BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": old_baseline_hash,
            "historical_plan_reference_intentionally_not_rewritten": True,
        },
        "pre_fix_baseline": {
            "record": baseline["record"],
            "manifest": (BASELINE / "MANIFEST.json").relative_to(ROOT).as_posix(),
            "manifest_sha256": baseline_hash,
            "detached_hash_matches": True,
            "source_archive": baseline["source"]["archive"],
            "document_archive": baseline["mutable_documents"]["archive"],
        },
        "entry_currentness": {
            "record": file_row(entry_currentness_path),
            "new_drift": [],
            "known_historical_plan_mismatch_only": True,
        },
        "preservation": {
            "predecessor_target_unchanged": True,
            "predecessor_baseline_unchanged": True,
            "predecessor_execution_records": predecessor["execution_records"],
            "predecessor_evidence": predecessor["evidence"],
            "review_inputs": baseline["review_inputs"]["files"],
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
                **file_row(scoped_status_path.relative_to(ROOT)),
                "rows": len(scoped_status.splitlines()),
            },
            "global_status": {
                **file_row(global_status_path.relative_to(ROOT)),
                "rows": len(global_status.splitlines()),
                "capture_note": "Captured before manifest/hash/handoff to avoid self-reference.",
            },
        },
        "documents": {
            "file_count": len(current_documents),
            "files": current_documents,
            "archive": document_archive,
            "changes_from_pre_fix": {
                "added": 0,
                "modified": len(document_changes),
                "deleted": 0,
                "files": document_changes,
            },
            "recoverable_delta": document_delta,
        },
        "findings": {
            "R01": "shared compile/evaluate authored preprocessing cost, checked nested composition, and explicit static versus runtime-dynamic work",
            "R02": "allocation-admitted iterative bottom-up segment shape with unchanged stack/task semantics and no new depth cap",
            "R03": "exact 4,460-byte historical PLAN recovered with raw provenance while predecessor records remain unchanged",
        },
        "r03_recovery": {
            "record": file_row(recovery_path),
            "historical_plan": recovered,
            "raw_provenance": raw_event,
            "predecessor_baseline_gap_retained_as_historical_fact": True,
            "successor_recovery_verified": True,
        },
        "execution_records": execution_records,
        "evidence": {
            "file_count": len(evidence),
            "files": evidence,
            "archive": evidence_archive,
        },
        "validation_receipts": validations,
        "validation_summary": {
            "pre_fix_reaudit_probe": "8 pass / 2 expected R01 failures / 1 ignored",
            "pre_fix_deep_shape_probe": "expected SIGABRT(-6) at depth 2,048 on 2 MiB worker",
            "post_fix_value_expr_unit": "22 passed",
            "post_fix_correction_probe": "7 passed",
            "post_fix_reaudit_probe": "10 passed / 1 ignored; isolated deep probe passed",
            "post_fix_all_targets": "1,382 lib plus 52 integration tests passed",
            "fingerprints": "3/3 pre-fix and corrected byte values identical",
            "clippy": "passed with predecessor baseline allows clippy::never_loop and clippy::eq_op",
        },
        "stage_review": [
            "pipeline, re-audit report, probes/logs/currentness, predecessor handoff and target read",
            "entry currentness separated the known historical PLAN mismatch from current target drift; no new drift",
            "pre-fix actual source and mutable-document bytes archived before implementation",
            "R01 and R02 failures reproduced before correction",
            "R01/R02 code corrected and boundary/regression/fingerprint/all-target validation passed",
            "R03 exact historical bytes recovered with raw provenance without rewriting predecessor evidence",
            "successor target frozen; re-review requested and mutation stopped",
        ],
        "review_entry": {
            "path": HANDOFF.relative_to(ROOT).as_posix(),
            "detached_sha256": HANDOFF_HASH.relative_to(ROOT).as_posix(),
            "binding": "Generated after MANIFEST.json so the handoff can contain the exact target hash; verifier checks its detached hash.",
        },
        "mutation_boundary": "Do not mutate correction source, records, evidence, handoff, or target after read-only verification until re-review returns.",
        "recovery": {
            "actual_before_after_source_bytes_preserved": True,
            "actual_before_after_document_bytes_preserved": True,
            "actual_validation_evidence_bytes_preserved": True,
            "recoverable_source_and_document_deltas_preserved": True,
            "safe_method": "Extract archives into isolated directories and inspect the binary-capable deltas; do not overwrite the dirty worktree blindly.",
        },
    }
    manifest_path = OUTPUT / "MANIFEST.json"
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    manifest_hash = digest(manifest_path)
    (OUTPUT / "MANIFEST.sha256").write_text(f"{manifest_hash}  MANIFEST.json\n", encoding="utf-8")

    HANDOFF.write_text(handoff_text(manifest_hash, baseline_hash, source_changes), encoding="utf-8")
    HANDOFF_HASH.write_text(
        f"{digest(HANDOFF)}  {HANDOFF.name}\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
