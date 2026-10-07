#!/usr/bin/env python3
"""Capture P2G-00 inputs only; never replace an existing revision."""
import hashlib
import json
import pathlib
import subprocess
import tarfile

ROOT = pathlib.Path(__file__).resolve().parents[5]
HERE = pathlib.Path(__file__).resolve().parents[1]
DOC_ID = "WPD-CRSO-20261004"
REQUEST = """core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md의 활성 범위를
P2G와 GEX에 따라 순차적으로 구현해줘.

U00 인수와 U01 F01 교정부터 시작하고, 내가 중단을 요청하거나
로드맵의 승인 범위를 완료할 때까지 다음 작업으로 계속 진행해줘.

이번 요청은 각 작업에 필요한 다음 과정을 모두 포함해:
- P2G 정책 정리·코드 검토·계획·Goal·인계
- GEX 구현·자체 검증·독립 감사
- 감사 finding 교정·재감사
- 선행 조건 충족 후 다음 작업 선택과 실행

단계 전환이나 감사 인계마다 재승인을 요청하지 마.
필요한 독립 감사에는 구현에 참여하지 않은 별도 에이전트(GPT-6 Astra / High)를 활용하고,
감사 통과 전에는 그 결과에 의존하는 후속 구현을 시작하지 마.

구현 중 필요한 정책은 기존 명시적 합의와 승인 범위 안에서
재량으로 판단해줘. 결정·이유·영향·위임 근거를 해당 정책 문서에
남기고, 필요한 P2G 갱신·재검증 후 구현을 이어가줘.

기존 계획과 유효한 증거를 재사용하고, 로드맵 전체를 미리
상세 설계하거나 완료된 단계를 불필요하게 반복하지 마.

작업별 결과·검증·남은 문제와 다음 위치를 계속 기록하고,
의미 있는 진척은 간결하게 보고하되 보고만 하고 멈추지는 마.
기존 사용자 변경과 과거 감사 원본은 보존해줘.

내 중단 요청, 승인 범위 완료, 또는 현재 권한과 환경으로
해결할 수 없는 실제 장애에서만 멈춰줘.
중단 시 완료·미완료 상태와 정확한 재개 위치를 남겨줘."""


def write(name, data):
    with (HERE / name).open("x", encoding="utf-8") as out:
        if isinstance(data, str):
            out.write(data)
        else:
            json.dump(data, out, ensure_ascii=False, indent=2)
            out.write("\n")


def artifact(kind, identity, payload):
    encoded = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
    return {"ref": {"Artifact Type": kind, "Artifact ID": identity,
                    "Content Digest": "sha256:" + hashlib.sha256(encoded).hexdigest()},
            "payload": payload}


def main():
    assert ROOT.name == "simulator", ROOT
    assert not (HERE / "WORKING_POLICY.ko.md").exists()
    for doc in (ROOT / "core_enhanced/docs/goals").glob("*/WORKING_POLICY*.md"):
        assert DOC_ID not in doc.read_text(), doc
    paths = [
        "core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md",
        "core_enhanced/docs/roadmaps/core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md",
        "core_enhanced/docs/roadmaps/core_first/execution/DECISIONS_20261004.ko.md",
        "core_enhanced/docs/plans/runtime_safety_observability/PLAN.ko.md",
        "core_enhanced/docs/plans/runtime_safety_observability/README.ko.md",
        "core_enhanced/docs/plans/runtime_safety_observability/REVIEW.ko.md",
    ]
    rows = [{"Intake Item ID": "IN-001", "Intake Kind": "user request",
             "Exact Content or Content Reference": REQUEST,
             "User-Provided Locator": "2026-10-04 current conversation, continuous implementation request",
             "User-Stated Constraint": "승인 범위·기존 합의 안 재량, 독립 감사 이전 종속 구현 금지, 기존 원본 보존",
             "Capture Limitation": "None"}]
    archive = HERE / "INTAKE_SOURCE_001.tar.gz"
    assert not archive.exists()
    with tarfile.open(archive, "w:gz") as tar:
        for index, path in enumerate(paths, 2):
            raw = (ROOT / path).read_bytes()
            rows.append({"Intake Item ID": f"IN-{index:03}", "Intake Kind": "exact repository reference",
                         "Exact Content or Content Reference": {"repository": str(ROOT), "path": path,
                            "sha256": hashlib.sha256(raw).hexdigest()},
                         "User-Provided Locator": "사용자가 지정한 활성 로드맵과 그 U02/S02 참조",
                         "User-Stated Constraint": "선택 범위 U02/A04/A05; 나머지 로드맵은 후속 소유자 유지",
                         "Capture Limitation": "local bytes captured; source semantics and runtime behavior not certified by intake"})
            tar.add(ROOT / path, arcname=path)
    intake = artifact("Policy Intake Snapshot", "PIS-CRSO-001", {
        "Working Policy Document ID": DOC_ID, "Snapshot Kind": "initial",
        "Predecessor Policy Intake Snapshot Ref": None, "Created By Stage": "P2G-00", "Intake Rows": rows})
    approval = {"user_request": "Policy Intake Snapshot.Intake Rows[IN-001]",
                "adopted_scope": "IN-002 exact ROADMAP §U02, A04/A05 and S02 reference",
                "execution_record": paths[1],
                "boundary": "P2G through Handoff; separately authorized GEX only after predecessor audit passes"}
    scopes = [
        ("I01", "in", "U02/A04: 기존 한도의 trusted host RON profile과 Core/authoring 실제 주입·identity·설정 오류 경계"),
        ("I02", "in", "U02/A05: bounded counters/high-water·종료 회수·관측 off/on 결정론·실제 Grafana 구성과 진단"),
        ("I03", "in", "선택 범위 P2G 및 GEX 자체 검증·독립 감사·교정; 기존 R05 경고와 유효 증거의 정확한 인수"),
        ("O01", "out", "U02 명시적 제외: 새 gameplay 발동 상한·임의 성능 합격 수치·20분 시험·mod가 profile 값을 바꾸는 권한"),
        ("O02", "out", "U02 카드: continuation/soft quota는 필수 아님; 후속 U03의 세션 terminal fault 및 U11의 새 Lab 허가 정책은 해당 owner 유지"),
        ("O03", "out", "활성 로드맵 Core 우선 범위: Unity 구현과 개별 최종보스 고유 구현; 기존 사용자 변경·감사 원본의 덮어쓰기"),
    ]
    working = artifact("Working Revision", "WR-CRSO-001", {
        "Predecessor Working Revision Ref": None, "Predecessor Final Policy Freeze Ref": None,
        "Approved Metadata and Scope": {
            "Working Policy Document ID": DOC_ID,
            "Policy Topic": "U02 실행 한도 설정과 제한된 비용의 런타임 관측",
            "Requested Result": "bounded_pipeline P2G-00→60, then separately authorized GEX and independent audit",
            "Target Project/Repository": {"project": "simulator", "workspace": str(ROOT),
                "head_at_intake": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()},
            "Approval Reference": approval,
            "Scope Items": [{"Scope Item ID": key, "Approved Disposition": disposition,
                "Statement": statement, "Exact User Approval Reference": approval} for key, disposition, statement in scopes]},
        "Policy Intake Snapshot Ref": intake["ref"], "Current Policy Element Revision Map": {},
        "Decision Revision Ref Set": [], "Open Policy Question Ref Set": [],
        "Post-Freeze Change/Closure Basis": {"Post-Freeze Change Record Ref Set": [],
            "Authoritative Change Obligation Declaration Ref Set": []}})
    bundle = {"artifacts": [intake, working]}
    write("P2G_00_INITIAL_001.json", bundle)
    assert working["payload"]["Policy Intake Snapshot Ref"] == intake["ref"]
    assert not working["payload"]["Current Policy Element Revision Map"]
    write("P2G_00_RESULT_001.json", {"stage": "P2G-00", "status": "complete",
        "created_refs": [a["ref"] for a in bundle["artifacts"]],
        "checks": {"path_unoccupied": True, "identity_unique": True, "conforming_path": True,
                   "raw_intake_archived": True, "scope_approval_per_item": True,
                   "empty_initial_policy_sets": True, "no_later_stage_artifacts": True},
        "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
        "new_user_request_required": False, "next": "P2G-10", "production_mutation": False})
    write("WORKING_POLICY.ko.md", """# U02 실행 한도 설정과 제한된 비용의 런타임 관측

Document Identity: `WPD-CRSO-20261004` · Target Documentation Root: `core_enhanced/docs`.
Policy Directory: `core_runtime_safety_observability` · Current Working Revision: `WR-CRSO-001`.
정확한 Ref와 승인 in/out 범위는 [initial revision](P2G_00_INITIAL_001.json)이 소유한다.

현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.
P2G-10 Completion Predicate: N/A. 이 초기 인수는 정책 검토 또는 구현 완료를 뜻하지 않는다.

사용자의 2026-10-04 연속 요청에 따라 U02/A04/A05를 선택했다. P2G-60 인계까지 필요한 단계를
연속 진행하며 기존 요청에 따라 GEX와 독립 감사까지 수행한다. 단계별 재승인은 요구하지 않는다.
U01의 독립 감사가 통과하기 전에는 그 결과에 의존하는 제품 구현을 시작하지 않는다.
현재 작업은 U01 동결 target·records·references를 변경하지 않는 신규 입력 인수다.

원문은 [입력 아카이브](INTAKE_SOURCE_001.tar.gz), 초기 검증은 [결과](P2G_00_RESULT_001.json)에 보존했다.
기존 계획을 재사용하되 실제 source 확인·정책 의미 판정·완료 기준은 각 후속 owner에서 검증한다.
중단 요청·승인 범위 완료·현재 권한과 환경에서 해결할 수 없는 실제 장애가 실행 종료 경계다.
""")
    print("P2G-00 complete: 2 exact artifacts, 7 raw intake rows, 6 approved scope items")


if __name__ == "__main__":
    main()
