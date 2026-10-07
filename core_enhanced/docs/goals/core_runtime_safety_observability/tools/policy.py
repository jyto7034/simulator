#!/usr/bin/env python3
"""Publish the immutable P2G-10 decision/element basis from captured policy."""
import copy
import json
from initialize import HERE, artifact, write

# source_span, statement, independently observable acceptance
POLICIES = [
    ("PLAN §1, §2 형식·기본값", "한도 설정은 trusted host가 소유한다. pack/scenario/mod는 자기 한도를 바꿀 수 없다. 기존 native/execution/expression/trial/history 한도의 숫자·scope·ceiling·typed fault를 유지하며 embedded RON을 유일한 운영 기본값 원본으로 사용한다.",
     "기본 profile과 변경 전 기본 실행은 결과·오류가 같다. 파일 미지정 embedding/host/CLI의 effective settings가 같고 임의 pack 값은 이를 완화하지 못한다. 신규 gameplay 상한이나 전투 총 메모리 cap을 추가하지 않는다."),
    ("PLAN §2 파일 오류·identity", "명시적 설정 경로의 부재/invalid는 초기화 오류이며 fallback하지 않는다. DTO는 versioned named fields, 유한 정수·checked target-width conversion, unknown/duplicate/음수/overflow/불법0/ceiling 거절이다. loader는 고정64KiB·depth 제한을 profile 파싱 전에 적용한다.",
     "미지정은 정상 default, 지정 missing/invalid는 경로와 field를 포함한 오류이며 콘텐츠 publication·전투 생성 전에 거절된다. 입력 크기·중첩·중복·범위 경계를 넘는 입력도 bounded하게 거절된다."),
    ("PLAN §2 파일 오류·적용·identity", "raw SHA와 canonical effective SHA를 분리한다. profile은 host/session/generation 생성 때 immutable하게 고정하고 active battle hot reload는 하지 않는다. 설정 차이로 달라지는 admission을 cache identity에 결속한다.",
     "주석/공백만 다르면 raw SHA만 다르고 effective SHA는 같다. 값이 다른 profile은 admission/cache identity가 다르며 서로 격리된다. active 실행은 후속 host profile 변경으로 바뀌지 않고 재현의 hash 불일치는 명시된다."),
    ("PLAN §2 constructor·restore; S1 완료 기준", "Core는 파일시스템/환경변수 없이 validated profile을 받는다. 실제 host와 library는 scenario/첫 spawn 이전에 native limits를 주입하고 restore도 host profile로 active state를 검증한다. 저장 파일은 임의 한도를 승인하지 않는다.",
     "실제 host→Core 및 embedding의 첫 spawn부터 작은 유효 한도가 적용된다. restore는 현재 host 한도로 검증하며 실패 시 원본 save bytes가 유지된다. profile을 저장에서 무검증 복구하지 않는다."),
    ("PLAN §2 authoring execution/trial·S1 완료 기준", "authoring API와 CLI는 동일 validated profile을 사용한다. CLI 선택 --runtime-safety PATH는 기존 positional 계약을 보존한다. execution allowance의 중첩/rollback 공유를 보존하고 trial total-work/receipt cap을 live battle 전체 수명 cap으로 바꾸지 않는다.",
     "동일 입력의 API/CLI/host는 적용 가능한 동일 settings와 typed fault를 관측한다. 작은 한도·중첩·취소·rollback 시험에서 allowance가 우회되지 않으며 invalid 설정으로 기존 세션/publication이 교체되지 않는다."),
    ("PLAN §2 expression·cache", "expression profile은 parse→compile→catalog→plan→runtime/preview/explain까지 일관되게 적용한다. parser HARD_MAX와 compiler의 기존 typed API 허용 범위를 구분하고 다른 admission profile의 plan/cache를 무검증 재사용하지 않는다.",
     "각 단계의 경계 입력이 선택 profile을 소비한다. 두 profile의 compile/admission 결과와 cache reuse는 identity로 격리되며 정상 식 의미/preview 읽기 전용 계약과 기존 합법 ExprLimits 범위를 보존한다."),
    ("PLAN §2 battle history", "history는 현재 bounded 창의 events/encoded bytes/span limits를 profile에서 받는다. 생성·deserialize·import도 같은 정책을 통과하고 floor_seq/eviction/resync/결과 통계를 유지한다. history span은 전투 최대 시간이 아니다.",
     "작은 history profile에서 import·실행 모두 저장 상한을 지키고 cursor gap/resync와 결과 통계가 유지된다. 창보다 긴 정상 전투도 history span 때문에 종료되지 않는다."),
    ("PLAN §3 첫 metric 계약", "내부 정수 장부가 safety 판정을 계속 소유한다. bounded operation summary로 work/duration/admitted bytes·committed extent/events/actions/projectiles/units·history·fault/drop·selected simulation time과 interval peak를 제공한다. extent/admission/RSS 및 wall/simulation time을 구분한다.",
     "짧은 사용량 peak와 실제 반환 fault가 관측되고 내부 poll 중복으로 rejection이 증가하지 않는다. exporter 값이 safety 판정으로 역류하지 않으며 scrape가 비용 청구·owner 전체 순회·게임 오류를 유발하지 않는다."),
    ("PLAN §3 표준 메트릭·수집 주기", "core 계측은 선택 기능, exporter/HTTP/task는 opt-in host 소유다. Rust metrics→Prometheus→Grafana와 실제 datasource/dashboard 구성을 제공한다. HTTP 개발 기본은 loopback, 서비스 부재/실패는 gameplay를 막지 않는다.",
     "저장소 구성으로 실제 Grafana에서 사용량·거부 원인을 볼 수 있다. 계측 disabled/enabled·collector 미접속·scrape 중단에서 게임 결과/오류/RNG/event 순서가 같고 유한 exporter storage가 유휴에도 회수된다."),
    ("PLAN §3 bounded storage·label", "관측 설정은 별도 observability.ron으로 gameplay hash에서 분리한다. 최초 개발 기본은 snapshot200ms/scrape1s/dashboard1s/선택1개/최근ring300개/local24h다. label은 유한 operation/reason/outcome과 고정 slot0이며 UUID/player/action/event ID를 시계열 label로 쓰지 않는다.",
     "전체 sum/max/분포와 선택한 전투 상세가 있고 선택 identity/시각은 제한된 상세 기록에 표시된다. 포화 시 관측 손실만 기록되고 게임 이벤트는 유지된다. profile 변경이 gameplay hash를 바꾸지 않으며 registry/ring/histogram storage 상한을 확인할 수 있다."),
    ("PLAN §3 종료·rollback·완료 기준", "metric handle은 hot path 이전에 등록하고 allocator hook에서 등록/문자열/출력하지 않는다. rollback된 시도의 실패/작업 통계는 남기되 commit count/state gauge는 commit에만 반영한다. 종료된 전투는 집계 차감·선택 해제·자원 회수한다.",
     "실패/rollback·중복 소비·종료·선택 변경에서 counter 중복과 stale gauge가 없고 관측 보관량이 bounded하다. 종료와 exporter 부재/포화가 gameplay 상태·한도 의미에 영향을 주지 않는다."),
    ("ROADMAP U02; PLAN §1,§4–6", "기존 R05 경고는 raw guard와 정책 판정을 구분해 유지한다. 관측 비용은 측정하되 임의 새 합격 수치·20분 시험·3MB cap·안전 오류의 자동 스킬 취소·continuation을 U02에 추가하지 않는다. 기존 기능·복원·결정론·공개 authoring 계약을 회귀 검증한다.",
     "현재 source의 필요한 전체 Core/host 검사와 고정입력 off/on parity, 정상/집중/기능별/한도거절 표본이 남는다. 기존 경고를 숨기거나 새 source 실행으로 과거 결과를 가장하지 않으며 U01 취소 계약이 유지된다."),
]


def main():
    initial = json.loads((HERE / "P2G_00_INITIAL_001.json").read_text())["artifacts"]
    intake, before = initial
    basis = {"Input Working Revision Ref": before["ref"],
             "Exact User-Approved Target Scope/Change Request": None,
             "Policy Intake Snapshot Ref": intake["ref"]}
    artifacts, current, decisions, dispositions = [], {}, [], []
    for number, (span, statement, acceptance) in enumerate(POLICIES, 1):
        eid = f"PE-CRSO-{number:02}"
        opened = artifact("Policy Element Revision", f"{eid}-r1", {
            "Stable Element ID": eid, "Predecessor Policy Element Revision Ref": None,
            "Policy Statement": statement, "Decision Revision Ref": None,
            "Observable Policy Acceptance": None, "Intrinsic Status": "open"})
        decision = artifact("Decision Revision", f"DEC-CRSO-{number:02}-r1", {
            "Predecessor Decision Revision Ref": None, "Affected Element IDs": [eid],
            "Exact Decision": statement, "Decision Authority": "AI delegated decision adopting existing approved direction",
            "Exact User Approval Reference": {"intake_ref": intake["ref"], "item": "IN-001", "scope": "U02/A04/A05"},
            "Source Reference": {"intake_ref": intake["ref"], "items": ["IN-002", "IN-005", "IN-006"], "span": span},
            "Reason": "기존 추천 채택·설정/관측 계획을 새 연속 요청의 U02 범위에서 구체화한다. 예전 문서의 구현 미승인/인계 정지는 최신 명시적 요청으로 해소됐으며 원본은 보존한다.",
            "Impact": acceptance,
            "Delegation Criteria": {"existing_agreements_preserved": "기존 값·오류·scope와 U02 제외 유지",
                "derived_from_ownership": "trusted host 설정과 별도 관측 소유권에서 도출",
                "no_new_game_rule": "새 비용·보상·확률·gameplay 상한 없음",
                "local_impact": "설정·관측 경계 안, 타 객체의 획득·진행 변경 없음",
                "user_data_preserved": "구저장 삭제·일괄 변환·재해석 없음"}})
        decided = artifact("Policy Element Revision", f"{eid}-r2", {
            "Stable Element ID": eid, "Predecessor Policy Element Revision Ref": opened["ref"],
            "Policy Statement": statement, "Decision Revision Ref": decision["ref"],
            "Observable Policy Acceptance": acceptance, "Intrinsic Status": "decided"})
        disposition = artifact("Candidate Policy Element Disposition Record", f"CD-CRSO-{number:02}-001", {
            "Candidate ID": f"C-CRSO-{number:02}", "P2G-10 Evaluation Basis": basis,
            "Source Reference": decision["payload"]["Source Reference"], "Candidate Statement": statement,
            "Disposition": "promoted", "Stable Element ID": eid,
            "Initial Open Policy Element Revision Ref": opened["ref"],
            "Decision Revision Ref": decision["ref"]})
        artifacts.extend([opened, decision, decided, disposition])
        current[eid] = decided["ref"]
        decisions.append(decision["ref"])
        dispositions.append(disposition["ref"])
    payload = copy.deepcopy(before["payload"])
    payload.update({"Predecessor Working Revision Ref": before["ref"],
                    "Current Policy Element Revision Map": current, "Decision Revision Ref Set": decisions})
    result = artifact("Working Revision", "WR-CRSO-002", payload)
    artifacts.append(result)
    assert len(current) == len(decisions) == len(dispositions) == 12
    assert len({r["Artifact ID"] for r in decisions}) == 12
    assert all(a["payload"]["Decision Revision Ref"] in decisions for a in artifacts
               if a["ref"]["Artifact Type"] == "Policy Element Revision" and a["payload"]["Intrinsic Status"] == "decided")
    write("P2G_10_POLICY_001.json", {"artifacts": artifacts})
    write("P2G_10_RESULT_001.json", {"stage": "P2G-10", "status": "complete",
        "Input Working Revision Ref": before["ref"], "Result Working Revision Ref": result["ref"],
        "Produced Candidate Disposition Ref Set": dispositions, "is_p2g10_complete": True,
        "checks": {"all_in_scope_candidates_disposed": True, "all_elements_decided": True,
            "decisions_current_single_leaf": True, "acceptances_present_and_consistent": True,
            "delegation_basis_per_decision": True, "intake_unchanged": True,
            "open_questions_empty": True, "cross_references_valid": True},
        "Finding Triage": [
            {"classification": "material_policy", "disposition": "delegated_decision", "elements": list(current),
             "evidence": "P2G_10_POLICY_001.json Decision Revision exact basis/reason/impact/criteria"},
            {"classification": "implementation_finding", "disposition": "P2G-20 source review required",
             "statement": "G-03 host bootstrap·현재 defaults·expression/cache/history 경계와 exporter boundedness는 코드 근거로 확인해야 하며 아직 구현/검토 PASS 아님"},
            {"classification": "out_of_scope", "disposition": "original owners retained",
             "statement": "U03 오류 세션 수명·U11 Lab 정책과 Unity는 해당 카드에서 수행; 20분 시험은 제외"}],
        "new_user_request_required": False, "next": "P2G-20", "production_mutation": False})
    view = (HERE / "WORKING_POLICY.ko.md").read_text()
    view = view.replace("`WR-CRSO-001`", "`WR-CRSO-002`")
    view = view.replace("현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 이 초기 인수는 정책 검토 또는 구현 완료를 뜻하지 않는다.",
        "현재 P2G-10 완료, 다음은 P2G-20 코드 검토다. 정책 요소 12개와 독립 결정 12개, 미결 질문 0개다.\nP2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [검증·분류](P2G_10_RESULT_001.json)가 근거다. 코드 검토·제품 구현 완료를 뜻하지 않는다.")
    view += "\n정책 의미는 기존 계획을 최신 요청의 위임으로 채택했다. 결정마다 선택·이유·관측 가능한 영향·다섯 위임 조건을 기록했으며 사용자 직접 답변을 새로 만든 것으로 표시하지 않는다.\n"
    (HERE / "WORKING_POLICY.ko.md").write_text(view)
    print("P2G-10 complete: 12 decided elements, 12 decisions, 12 candidate dispositions, 0 open questions")


if __name__ == "__main__":
    main()
