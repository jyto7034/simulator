#!/usr/bin/env python3
"""P2G-00: immutable raw intake and approved U04 scope only."""
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[5]
HERE = Path(__file__).resolve().parents[1]
DOC_ID = "WPD-CRRP-20261004"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def artifact(kind, identity, payload):
    raw = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return {"ref": {"Artifact Type": kind, "Artifact ID": identity,
                    "Content Digest": "sha256:" + hashlib.sha256(raw).hexdigest()}, "payload": payload}


def write(name, value):
    with (HERE / name).open("x") as stream:
        if isinstance(value, str):
            stream.write(value)
        else:
            json.dump(value, stream, ensure_ascii=False, indent=2)
            stream.write("\n")


def main():
    assert ROOT.name == "simulator"
    assert not (HERE / "WORKING_POLICY.ko.md").exists()
    goals = ROOT / "core_enhanced/docs/goals"
    checked = []
    for directory in goals.iterdir():
        if not directory.is_dir() or directory == HERE:
            continue
        for path in directory.iterdir():
            if path.is_file() and (path.name.startswith("WORKING_POLICY") or
                                   path.name.startswith("P2G_") and path.suffix == ".json"):
                assert DOC_ID not in path.read_text(), path
                checked.append(str(path.relative_to(ROOT)))
    previous = goals / "core_runtime_fault_isolation"
    audit = previous / "post_audit/u03_20261004_003"
    close = json.loads((audit / "AUDIT_CLOSE.json").read_text())
    assert close["verdict"] == "Complete" and close["confidence"] == "High"
    assert not close["open_findings"] and not close["remaining_required_work_in_scope"]
    closure = ROOT / "core_enhanced/docs/roadmaps/core_first/execution/u03_closure_20261004_001"
    publication = json.loads((closure / "PUBLICATION_RECORD.json").read_text())
    assert publication["audit_closure_sha256"] == sha(audit / "AUDIT_CLOSE.json")
    prior = json.loads((previous / "P2G_00_INITIAL_001.json").read_text())["artifacts"][0]
    request = prior["payload"]["Intake Rows"][0]["Exact Content or Content Reference"]
    paths = [
        "core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md",
        "core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md",
        "core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md",
        "core_enhanced/docs/game_rulebook_next.md",
        "core_enhanced/docs/game_rules_next/results-and-save.md",
        "core_enhanced/docs/game_rules_next/open-decisions.md",
        "core_enhanced/docs/goals/core_runtime_fault_isolation/post_audit/u03_20261004_003/COMPLETION_AUDIT.md",
        "core_enhanced/docs/goals/core_runtime_fault_isolation/post_audit/u03_20261004_003/END_CURRENTNESS.json",
        "core_enhanced/docs/goals/core_runtime_fault_isolation/post_audit/u03_20261004_003/AUDIT_CLOSE.json",
        "core_enhanced/docs/goals/core_runtime_fault_isolation/execution/single_goal/corrections/f02_origin_001/target/u03_003/MANIFEST.json",
        "core_enhanced/docs/roadmaps/core_first/execution/u03_closure_20261004_001/RESULT.ko.md",
        "core_enhanced/docs/roadmaps/core_first/execution/u03_closure_20261004_001/PUBLICATION_RECORD.json",
    ]
    rows = [{"Intake Item ID": "IN-001", "Intake Kind": "user request",
             "Exact Content or Content Reference": request,
             "User-Provided Locator": "2026-10-04 continuous Core-first implementation request, preserved verbatim in preceding intake",
             "User-Stated Constraint": "순차 P2G/GEX·독립 Astra High 감사와 교정, 승인 범위 내 정책 재량, 선행 통과 전 종속 구현 금지, 원본 보존, 중간 재승인 없이 계속",
             "Capture Limitation": "None; original request copied unchanged, not a new user decision"}]
    with tarfile.open(HERE / "INTAKE_SOURCE_001.tar.gz", "x:gz") as archive:
        for index, name in enumerate(paths, 2):
            path = ROOT / name
            rows.append({"Intake Item ID": f"IN-{index:03}", "Intake Kind": "exact repository reference",
                         "Exact Content or Content Reference": {"repository": str(ROOT), "path": name, "sha256": sha(path)},
                         "User-Provided Locator": "ROADMAP U04/S01-S08, results-and-save, CF-D01/02/03/06/07 and closed U03 prerequisite",
                         "User-Stated Constraint": "현재 결과·선택·후퇴·종료복구·이어하기·SavePoint와 실제 reader/writer를 함께 전환; 미래 상태 선추가·구저장 삭제 금지",
                         "Capture Limitation": "Exact local bytes captured; intake does not certify policy meaning or current runtime behavior"})
            archive.add(path, arcname=name, recursive=False)
    intake = artifact("Policy Intake Snapshot", "PIS-CRRP-001", {
        "Working Policy Document ID": DOC_ID, "Snapshot Kind": "initial",
        "Predecessor Policy Intake Snapshot Ref": None, "Created By Stage": "P2G-00", "Intake Rows": rows})
    approval = {"user_request": "Policy Intake Snapshot.IN-001",
                "adopted_scope": "IN-002 ROADMAP §5 U04/S01-S08 and §6 current-state restoration boundary; IN-006 results-and-save",
                "delegation": "IN-004 DECISIONS_20261004 CF-D06; original continuous request",
                "boundary": "U03 independent Complete/High consumed; scoped P2G00-60 then separately authorized GEX implementation/audit/correction"}
    scope = [
        ("I01", "in", "승패 성립 즉시 현재 직원·손실·자원·XP·보상 권리·노드 소비를 확정·저장하고 결과 확인은 표시만 닫는다", "U04/S01,S08"),
        ("I02", "in", "선택형 보상의 권리·실제 후보·선택 대기를 보존하고 중복 지급·재추첨을 막는다; 후보 부족/0개의 기존 확정 정책을 따른다", "U04/S02; CF-D02/03"),
        ("I03", "in", "성공한 수동후퇴를 노드당2회로 제한하고 세 번째 도전·이미 소비한 횟수·미확정 노드 종료복구를 구분한다", "U04/S03,S04"),
        ("I04", "in", "현재 확정 진행의 stable Continue와 단일 최신 SavePoint의 명시적 load3회·누적 횟수·동일 프로세스 재접속을 분리한다", "U04/S05,S06,S07"),
        ("I05", "in", "Core 결과·실제 정산·영속 확인/재처리·query/ack·tick/명령·active/profile 저장·V9 후속 reader/writer와 crash/concurrent choice 검증을 함께 전환한다", "U04 together-boundaries and completion; ROADMAP §6.1/6.2"),
        ("I06", "in", "구 개발/테스트 저장의 legacy 의미 호환은 요구하지 않으며 원본 보존·명시적 미지원 처리를 적용한다", "CF-D01; results-and-save 구저장 지원 범위"),
        ("I07", "in", "선행 U01/U02/U03의 실제 완료 증거와 오류 격리·원본 보호를 인수하고 영향 조건 재검증·독립 감사·교정을 수행한다", "continuous request; U04 prerequisites"),
        ("O01", "out", "최신 지도·억제력·창고·전조 등 아직 도입하지 않은 미래 schema 선추가 및 그 owner의 gameplay 구현", "U04 explicit exclusions; ROADMAP §6.1/6.2"),
        ("O02", "out", "과거 저장 자동 삭제·덮어쓰기·필드 무시로 새 형식 취급, 최초 실제 저장 구현을 U28로 미루기", "U04 explicit exclusions; CF-D01"),
        ("O03", "out", "공개/숨겨진 최종보스의 미합의 고유 구현과 Unity 후속 화면 구현", "ROADMAP USER-01/02 and U04 Core scope"),
        ("O04", "out", "후속 U05 이후 전투·유물·모드 정책의 일괄 상세 설계 또는 무관한 제품 정리", "continuous request scope-local sequencing; ROADMAP §5 owners"),
    ]
    working = artifact("Working Revision", "WR-CRRP-001", {
        "Predecessor Working Revision Ref": None, "Predecessor Final Policy Freeze Ref": None,
        "Approved Metadata and Scope": {"Working Policy Document ID": DOC_ID,
            "Policy Topic": "U04 결과 확정·선택 권리·후퇴·이어하기·SavePoint 수명 전환",
            "Requested Result": "bounded_pipeline P2G-00→60, then authorized GEX implementation/verification/independent audit/correction",
            "Target Project/Repository": {"project": "simulator", "workspace": str(ROOT),
                "head_at_intake": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()},
            "Approval Reference": approval,
            "Scope Items": [{"Scope Item ID": key, "Approved Disposition": disposition, "Statement": statement,
                             "Exact User Approval Reference": {**approval, "specific_locator": locator}}
                            for key, disposition, statement, locator in scope]},
        "Policy Intake Snapshot Ref": intake["ref"], "Current Policy Element Revision Map": {},
        "Decision Revision Ref Set": [], "Open Policy Question Ref Set": [],
        "Post-Freeze Change/Closure Basis": {"Post-Freeze Change Record Ref Set": [], "Authoritative Change Obligation Declaration Ref Set": []}})
    assert working["payload"]["Policy Intake Snapshot Ref"] == intake["ref"]
    write("P2G_00_INITIAL_001.json", {"artifacts": [intake, working]})
    write("P2G_00_RESULT_001.json", {
        "stage": "P2G-00", "status": "complete", "recorded_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "created_refs": [intake["ref"], working["ref"]], "identity_checked_paths": checked,
        "checks": {"path_unoccupied": True, "identity_unique": True, "conforming_path": True,
                   "scope_approval_per_item": True, "raw_intake_archived": True, "empty_initial_policy_sets": True,
                   "no_later_stage_artifacts": True, "U03_independent_audit_closed": True},
        "archive_sha256": sha(HERE / "INTAKE_SOURCE_001.tar.gz"),
        "predecessor_audit_close_sha256": sha(audit / "AUDIT_CLOSE.json"),
        "next": "P2G-10 under original continuous authorization", "new_user_request_required": False,
        "production_mutation": False})
    write("WORKING_POLICY.ko.md", """# U04 결과 확정·선택 권리·후퇴·이어하기·SavePoint 수명 전환

Document Identity: `WPD-CRRP-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_result_persistence` · Current Working Revision: `WR-CRRP-001`.
승인 범위와 exact refs는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.
P2G-10 Completion Predicate: N/A. 입력 인수는 정책 동결·제품 구현·독립 감사 완료가 아니다.

U03 전체 독립 Complete / High 종료와 상태-only 완료 인수를 확인한 뒤 U04/S01–S08을 선택했다.
사용자의 연속 요청과 CF-D06 정책 재량 위임을 인수한다. 기존 명시 합의·승인 범위 안에서 필요한
결정·이유·영향·위임 근거를 남기고 P2G/GEX·비참여 Astra High 독립 감사·교정을 이어간다.

현재 존재하는 결과·선택·후퇴·종료복구·Continue·SavePoint 및 실제 파일 reader/writer가 범위다.
후속 owner의 미래 schema와 미합의 최종보스 고유 구현을 이 단계로 앞당기지 않는다.
구저장 지원 범위와 후보 부족/0개 정책은 기존 명시 결정 원문을 인수하며 새 사용자 답변으로 만들지 않는다.

[입력 원본 archive](INTAKE_SOURCE_001.tar.gz)와 [초기 검증](P2G_00_RESULT_001.json)을 보존한다.
정책 의미·코드 현재성·구체 구현 경계는 이후 owning 단계에서 판정한다.
""")
    print("U04 P2G-00 complete: 2 immutable artifacts, 13 raw intake rows, 11 approved scope items; no product mutation")


if __name__ == "__main__":
    main()
