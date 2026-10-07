#!/usr/bin/env python3
"""P2G-10 U04: immutable scope-local decisions; no production mutation."""
import copy
import json
from initialize import HERE, artifact, write

# Names, exact captured source spans, decision, observable acceptance, rationale.
POLICIES = [
    ("즉시 결과 확정", "IN-006:7-19; IN-002 U04/S01,S08",
     "현재 전투의 승패가 성립한 동일 처리에서 직원 상태·손실·소모품 기간·자원·XP·확정 보상 또는 선택 권리·격파 기록·노드 소비를 함께 확정한다. 결과 확인은 이미 확정된 결과를 표시 종료하고 다음 진행으로 옮기는 동작이다. 명령과 자동 tick 모두 같은 의미를 따른다.",
     ["승리·패배 직후 확인 없이 종료하고 새 프로세스로 이어해도 동일 직원·자원·XP·보상·진행이 남는다.", "반복 조회·결과 확인·재접속·재처리로 정산이 다시 실행되지 않는다. 선택 후보에 속한 효과는 선택 전 무조건 지급하지 않는다."],
     "결과 확정 시점은 명시 합의다. 현재 보상 effect가 XP 등을 함께 담으므로 무조건 정산과 아직 선택하지 않은 조건부 지급 권리를 구별한다."),
    ("패배 우선과 실패 완료", "IN-006:18-48; IN-002 U04/S01",
     "동시 승패는 패배 우선이다. 확정 패배도 노드를 소비하고 재도전을 제공하지 않는다. 현재 일반전·엘리트·일반 보스·Event 전투의 실패 완료는 통과 가능하되 승리 보상·격파·해금을 만들지 않는다. 현재 FinalBoss·생존자/복구 경로 없음에 따른 런 실패와 체크포인트 복구 선택은 기존 소유 규칙을 적용한다.",
     ["첫 도전 패배 후에도 같은 노드 전투에 다시 진입할 수 없고 실패 완료와 성공 완료·격파를 구분한다.", "패배 XP는 현행 authored 규칙대로 한 번만 적용하며 승리 자산·격파를 지급하지 않는다. 런 실패 후보에는 남은 유효 SavePoint 복구 선택이 우선한다."],
     "남은 도전은 미확정 전투에서 후퇴한 경우에만 허용한다는 명시 합의를 기존 실패/복구 owner에 적용한다."),
    ("선택 권리와 고정 후보", "IN-006:13-17; IN-004 CF-D02/03",
     "승리의 선택형 보상은 결과에 귀속된 권리·실제 후보·선택 대기와 원래 지급 문맥을 고정한다. 후보 부족 시 실제 가능한 후보만 제시하고 0개는 보상 없음 안내 후 생략한다. 중복 보유·희귀도 제한이나 대체 보상을 추가하지 않는다. 선택은 한 번만 지급하며 동일 권리의 중복·경쟁 선택은 추가 지급을 만들지 않는다.",
     ["후보 3개 요청/실제2개와 실제0개에서 각각 2개 선택 또는 없음 안내가 유지되며 재실행·재접속으로 재추첨하지 않는다.", "살아남은 전투 참가자 등 원래 보상 문맥이 Continue 뒤에도 같고, 같은 권리에 대한 두 선택 중 최대 하나만 자산을 얻는다."],
     "CF-D02/03의 직접 결정과 단일 결과 소유권에서 도출한다. 선택 대상 효과의 수령 시점만 선택 이후이며 승패·노드 소비는 이미 확정이다."),
    ("후퇴 횟수", "IN-006:22-36,60-83; IN-002 U04/S03",
     "현재 후퇴 허용 전투는 노드당 성공한 수동 후퇴만 최대2회 소비한다. 첫 진입과 두 번의 후퇴 뒤 마지막 도전을 허용하며 두 번째 후퇴 자체는 노드를 소비하지 않는다. 거절·취소 입력과 개별 직원 철수는 차감하지 않는다. 명시된 강제 조우 후퇴 금지는 유지한다.",
     ["첫·둘째 후퇴는 각각 한 번만 차감하고 세 번째 도전의 후퇴는 거절된다. 재방문·재접속·프로세스 재시작에도 사용량이 남는다.", "확정 승패 후에는 남은 후퇴 횟수와 무관하게 후퇴할 수 없으며 거절 입력은 상태·횟수를 바꾸지 않는다."],
     "최대2회는 새 상한이 아니라 이미 승인한 S03이며 현재 attempts_started를 단순히 패배 재시도 횟수로 유지해서는 안 된다."),
    ("후퇴 복원", "IN-006:60-83; IN-002 §6.1/6.2",
     "성공한 후퇴는 현재 전투 진입 전의 직원·소모품·인벤토리·재화·노드/진행·체크포인트·난수를 함께 복원하되 사용한 후퇴 횟수를 보존한다. XP·보상·완료·추가 자원 비용을 만들지 않는다. 이미 확정된 현재 전투 전 상태는 보존하며 미래 최초 입장 비용·부화·유물 도입은 해당 owner가 이 경계를 확장한다.",
     ["전투에서만 발생한 손실·RNG·자산 변화가 함께 되돌고 후퇴 차감은 복원되지 않는다.", "복원 실패는 후퇴 성공/차감을 만들지 않으며 이미 확정된 전투 전 행동을 보상 환불로 바꾸지 않는다."],
     "현재 존재하는 상태에 승인 복원 경계를 적용한다. 아직 없는 억제력·부화 구현을 선추가하지 않는다."),
    ("미확정 노드 종료 복구", "IN-006:275-294; IN-002 U04/S04",
     "프로세스 종료 후 미확정 전투·Event·상점·정비·보상 등 모든 현재 노드는 입장 전 상태로 함께 복원한다. 노드 안의 미확정 소비와 결과·난수 배정을 함께 되돌리고 이미 사용한 수동 후퇴는 반환하지 않는다. 종료 복구는 후퇴도 SavePoint load도 아니며 둘 다 추가 소비하지 않는다. 승패가 나거나 비전투 완료가 확정된 노드는 유지한다.",
     ["각 현재 노드 유형에서 미완료 중 종료하면 비용·획득·직원·노드·RNG가 같은 입장 전 시점으로 복원된다.", "2회 후퇴 뒤 미결 전투 종료와 재진입에도 후퇴 사용량2, load 사용량 불변이며 마지막 도전의 후퇴 불가가 유지된다.", "결과 확인 전 종료와 완료된 비전투 종료는 완료·보상·소비를 되돌리지 않는다."],
     "승패 전 종료의 명시 예외와 모든 미확정 노드의 공통 원자 복원 규칙을 인수한다."),
    ("stable Continue", "IN-006:275-310; IN-002 U04/S05,S07",
     "Continue는 SavePoint 유무와 관계없이 최신 확정 진행과 미확정 노드의 입장 전 복구 경계를 사용한다. 같은 원정·직원·확정 자산·선택 권리를 이어하며 새 런 예산이나 기록을 만들지 않는다. 런 완료·확정 실패·자발 포기는 Continue를 무효화한다. 새 런으로 교체하는 기존 명시 확인을 유지한다.",
     ["첫 SavePoint 전에도 확정 시작/진행을 Continue할 수 있고 최신 완료 노드가 마지막 SavePoint보다 뒤여도 보존된다.", "Continue의 load 소비는0이며 이미 종료된 런은 이어할 수 없다. 교체 확인 없이 기존 유효 런을 덮어쓰지 않는다."],
     "Continue와 과거 체크포인트 복구가 다른 행위라는 명시 수명에서 도출한다."),
    ("SavePoint 작성", "IN-006:267-273; IN-002 U04/S06",
     "현재 SavePoint 완료는 생존 직원의 현재 트라우마15%를 올림해 줄이고 노드 완료와 단일 최신 체크포인트 저장을 함께 확정한다. Critical·Breakdown을 포함하고 Dead를 제외하며 신체 부상·신뢰·기억을 바꾸지 않는다. 새 저장으로 누적 load 사용량을 초기화하지 않는다.",
     ["살아 있는 트라우마1/99/100 및 Critical·Breakdown에서 올림 회복하고 Dead는 회복하지 않는다.", "SavePoint 재작성과 재시작 후에도 성공 load 사용량이 유지되고 회복·노드 완료·저장 중 일부만 확정되지 않는다."],
     "이미 정해진 회복과 원자 작성 의미를 그대로 적용하며 미래 지정 방 배치 변경은 지도 owner에 남긴다."),
    ("명시적 체크포인트 load", "IN-006:296-335; IN-002 U04/S06",
     "명시적 SavePoint load는 Core가 허용한 때만 런당 최대3회 성공한다. 원정 자산·소비 결과·직원 생사·RNG·노드 진행·후퇴 사용량은 체크포인트 당시 값으로 함께 복원한다. 누적 load 횟수와 확정 영구 기록은 되돌리지 않으며 저장 덮어쓰기·재시작·백업 복구로 load 사용량이 줄지 않는다.",
     ["후퇴 사용량이 다른 과거 체크포인트를 불러오면 해당 시점 값으로 복원되지만 load 누적은 하나 증가한다.", "3회 후 재저장·재시작·현재 파일 장애와 백업 경로에도 4회째 load가 허용되지 않는다.", "체크포인트 뒤의 소비와 그 결과가 함께 되돌고 동일 복구를 반복해 자산을 누적 반환하지 않는다."],
     "후퇴 사용량을 영구 max로 보존하면 승인한 과거 복구를 깨므로 Continue 예외와 명시적 rewind를 구분한다."),
    ("직원 손실과 소모품", "IN-006:18-20,204-224; IN-002 U04/S08",
     "현재 전투 개체의 실제 전투불능마다 부상+1과 authored 트라우마를 시도에 한 번 누적하고 결과에서 직원에 한 번 반영한다. 재배치된 직원의 통계는 합치고 마지막 개체가 최종 HP/생존을 대표한다. 실제 사망의 현행 장비·공유 파편 회수와 소비 아이템 효과 제거, 사망 방지 예외 및 전투 종료 기간 소비를 같은 결과에 포함한다.",
     ["같은 전투불능의 재관찰/결과 확인/Continue는 부상·트라우마를 중복 반영하지 않고 다른 실제 전투불능은 각각 반영한다.", "사망 직원의 현재 자산 회수·소모품 종료가 결과 직후 보존되며 사망 방지 때는 회수하지 않는다. 일반 회복으로 Dead가 살아나지 않는다."],
     "기존 employee attempt 정산을 확정 경계로 이동하는 정책이며 새로운 부상 엔진이나 미래 유물 상태를 추가하지 않는다."),
    ("동일 프로세스 재접속", "IN-006:292-294; IN-002 U04/S07; IN-010 U03 exact target",
     "동일 서버 actor의 연결 끊김은 기존 자동 정지·재접속 규칙을 유지하여 진행 중 전투를 보존한다. 프로세스 종료/새 owner의 Continue와 구별한다. U03 runtime fault는 정상 승패·후퇴·저장 결과가 아니며 해당 fault actor의 mutation/writer gate와 최초 원인을 유지한다.",
     ["연결만 끊었다 붙이면 같은 전투와 정지 상태가 남고 후퇴·load·노드 복원이 실행되지 않는다.", "runtime fault는 결과/보상/저장을 생성하지 않고 U03 Quit→새 actor→유효 Continue만 기존 복구 경로로 사용할 수 있다."],
     "U03 선행 오류 격리와 명시 disconnect 예외를 보존한다."),
    ("원자 영속 확정과 재처리", "IN-006:9-15,337-344; IN-002 U04 함께 전환할 경계/완료 조건",
     "영속 host에서 결과·선택·후퇴·완료·load의 성공은 해당 원정 상태와 영구 기록을 장애 후 일관되게 복구할 수 있을 때만 게시한다. 확정 전 쓰기 실패는 성공을 만들지 않고, 이미 durable하게 접수된 확정의 후속 쓰기/응답 실패는 같은 확정을 재처리한다. 일부 자산만 남기거나 동일 결과를 두 번 지급하지 않는다. I/O 장애를 정상 패배 또는 U03 전투 실행 fault로 바꾸지 않는다.",
     ["실제 파일 adapter의 각 게시 단계에서 프로세스를 종료하고 다시 열면 이전 유효 경계 또는 접수된 전체 결과 중 하나만 나타난다.", "active와 profile이 따로 갱신되는 중단·결과 ack 전 종료·선택 재요청에서 권리/지급/노드가 일치한다.", "확정 뒤 응답 실패를 재시도해도 성공 지급은 한 번이며 복구 필요 상태는 명시적으로 표시된다."],
     "처리 실패의 부분 결과와 중복 지급을 금지한 명시 합의에서 도출한다. 저장 수단은 P2G-20에서 현재 공통 owner를 검토한다."),
    ("저장 identity와 경쟁 쓰기", "IN-006:11-17,296-310; IN-002 U04 결과ID/중복/실제저장소",
     "결과/선택 중복 여부는 현재 원정과 현재 복구 분기의 권리에 귀속한다. 명시적 과거 load로 정당하게 되돌린 뒤 다시 성립한 결과를 과거 battle UUID만으로 영원히 차단하지 않는다. 공유하는 현재 단일 active 저장에 대한 오래된 상태의 쓰기는 최신 확정 자산·권리를 덮어쓸 수 없다. 경쟁 입력은 같은 권리를 한 번만 확정하고 충돌은 명시한다.",
     ["같은 저장 revision에서 시작한 경쟁 선택은 하나만 확정하며 패자는 최신 파일을 덮어쓰지 않는다.", "체크포인트 rewind 뒤 동일 결정적 battle identity가 재생돼도 되돌린 현재 자산에는 새 정당한 결과가 한 번 적용된다.", "다른 활성 원정/오래된 actor가 현재 저장을 암묵 교체하지 않고 기존 명시 교체 동의만 새 런 교체를 허용한다."],
     "현재 local-install 단일 active 저장과 개별 원정 권리 소유권을 유지한다. 새 player별 DB·보상 규칙·다른 actor의 강제 종료는 도입하지 않는다."),
    ("미지원 저장 보존", "IN-004 CF-D01; IN-006:337-344",
     "전환 후 지원하지 않는 구 active/profile 형식은 원본을 보존하고 명시적으로 거절한다. 변환·의미 호환을 필수로 요구하지 않으며 자동 삭제·덮어쓰기·필드 무시·다른 백업으로 조용히 대체하지 않는다. 새 지원 형식의 읽기/쓰기와 장애 복구는 함께 구현한다. 이미 양립하는 영구 profile 의미를 불필요하게 폐기하지 않는다.",
     ["구 V9 Core/V4 active 및 미지원 profile fixture의 bytes가 load·reset·new-run·복구 시도 후에도 보존되며 미지원 사유가 노출된다.", "새 형식 정상 저장·Continue·SavePoint·쓰기 장애 복구를 실제 adapter로 검증한다. 원본 분리 없이 새 형식으로 덮어쓰지 않는다."],
     "CF-D01은 구형 변환 의무를 없애되 원본 보호를 명시했다. 어떤 현행 profile schema가 계속 양립하는지는 코드 검토의 기술 판정이다."),
    ("현재 상태 범위와 선행 계약", "IN-002 U04 명시적 제외/§6.1/6.2; IN-001; IN-007..013 U03 closure",
     "현재 구현된 combat/Event/비전투 완료·자산·직원·checkpoint/Continue와 실제 reader/writer만 함께 전환한다. 기존 mastered-elite sweep의 확인 전 계산은 실제 BattleVerdict가 아닌 미확정 preview이므로 확인으로 완료할 때 저장하고 미완료 종료는 입장 전 복구한다. 기존 generic final-boss 경로는 유지하되 미합의 고유 콘텐츠와 미래 지도·억제력·창고·도구·유물 schema는 해당 후속 owner에 남긴다. 선행112조건과 별도 R06 계약을 영향에 맞게 인수/재검증한다.",
     ["현재 모든 producer·public Core consumer·host tick/명령·file reader/writer의 결과/복구 계약이 일치한다. sweep 미확정 preview 종료와 실제 완료를 구분한다.", "선행 증거의 현재 적용성·의도된 변경·새 검증이 추적되고 변경되지 않은 원본/사용자 변경을 보존한다.", "자체 시험과 구현 비참여 Astra High 독립 감사가 닫힌 뒤에만 U05 후속 구현으로 이동한다."],
     "로드맵의 현재 상태 제한과 실제 승패 확정이라는 조건을 현재 sweep preview에 적용한다. sweep 제거는 U17 소유다."),
]


def main():
    intake, before = json.loads((HERE / "P2G_00_INITIAL_001.json").read_text())["artifacts"]
    basis = {"Input Working Revision Ref": before["ref"],
             "Exact User-Approved Target Scope/Change Request": None,
             "Policy Intake Snapshot Ref": intake["ref"]}
    artifacts, current, decisions, dispositions = [], {}, [], []
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        eid = f"PE-CRRP-{n:02}"
        opened = artifact("Policy Element Revision", f"PER-CRRP-{n:02}-OPEN-001", {
            "Stable Element ID": eid, "Predecessor Policy Element Revision Ref": None,
            "Policy Statement": statement, "Decision Revision Ref": None,
            "Observable Policy Acceptance": [], "Intrinsic status": "open"})
        decision = artifact("Decision Revision", f"DR-CRRP-{n:02}-001", {
            "Predecessor Decision Revision Ref": None, "Affected Element IDs": [eid],
            "Exact Decision": statement,
            "Decision Authority": "AI delegated material-policy decision applying existing explicit agreements; not a new user answer",
            "Exact User Approval Reference": {"intake_ref": intake["ref"], "item": "IN-001",
                "delegation": "IN-004 CF-D06", "scope": "U04/S01-S08 and current-state boundary"},
            "Source Reference": {"intake_ref": intake["ref"], "span": span},
            "Reason": reason, "Impact": acceptance,
            "Delegation Criteria": {
                "existing_agreements_preserved": "즉시 정산/노드 소비·후퇴2/load3·종료 예외·CF-D01/02/03·U03 fault 계약 유지",
                "derived_from_ownership": reason,
                "no_new_game_rule": "승인한 수명/한도를 적용하며 새 보상·비용·확률·환불·자산 이전 예외 없음",
                "local_impact": "현재 원정 결과·선택·복원과 해당 저장 경계; 독립 진행·영구 기록 보호",
                "user_data_preserved": "구형 파일 보존/명시 거절, 유효 권리 보존, 사용자 변경과 감사 원본 보존"}})
        decided = artifact("Policy Element Revision", f"PER-CRRP-{n:02}-DECIDED-001", {
            "Stable Element ID": eid, "Predecessor Policy Element Revision Ref": opened["ref"],
            "Policy Statement": statement, "Decision Revision Ref": decision["ref"],
            "Observable Policy Acceptance": acceptance, "Intrinsic status": "decided"})
        disposition = artifact("Candidate Policy Element Disposition Record", f"CD-CRRP-{n:02}-001", {
            "Candidate ID": f"C-CRRP-{n:02}", "P2G-10 Evaluation Basis": basis,
            "Source Reference": decision["payload"]["Source Reference"], "Candidate Statement": statement,
            "Disposition": "promoted", "Stable Element ID": eid,
            "Initial Open Policy Element Revision Ref": opened["ref"], "Decision Revision Ref": decision["ref"]})
        artifacts.extend([opened, decision, decided, disposition])
        current[eid] = decided["ref"]
        decisions.append(decision["ref"])
        dispositions.append(disposition["ref"])
    for n, (candidate, span, owner) in enumerate([
        ("미래 억제력/지도/창고/부화/도구/유물 상태 선추가", "IN-006:86-203,226-246,303-335; IN-002 U04 제외", "U14/U15/U17/U20/U21 상태 도입 owner가 복원 계약 확장"),
        ("mastered elite sweep 제거", "IN-002 U17", "U17; U04는 현재 미확정 preview/확정 완료 경계만 적용"),
        ("Unity 결과/오류 화면 구현과 미합의 최종보스 고유 콘텐츠", "IN-002 USER-01/02,U26,U04 Core 범위", "각 명시 후속/제외 owner"),
    ], 1):
        disposition = artifact("Candidate Policy Element Disposition Record", f"CD-CRRP-OUT-{n:02}-001", {
            "Candidate ID": f"C-CRRP-OUT-{n:02}", "P2G-10 Evaluation Basis": basis,
            "Source Reference": {"intake_ref": intake["ref"], "span": span}, "Candidate Statement": candidate,
            "Disposition": "out_of_scope", "Evidence/Decision Reference": owner,
            "Stable Element ID": None, "Initial Open Policy Element Revision Ref": None})
        artifacts.append(disposition)
        dispositions.append(disposition["ref"])
    payload = copy.deepcopy(before["payload"])
    payload.update({"Predecessor Working Revision Ref": before["ref"],
                    "Current Policy Element Revision Map": current, "Decision Revision Ref Set": decisions})
    result = artifact("Working Revision", "WR-CRRP-002", payload)
    artifacts.append(result)
    for a in artifacts:
        assert artifact(a["ref"]["Artifact Type"], a["ref"]["Artifact ID"], a["payload"]) == a
    assert len({a["ref"]["Artifact ID"] for a in artifacts}) == len(artifacts)
    assert len(current) == len(decisions) == 15 and len(dispositions) == 18
    assert all(a["payload"]["Decision Revision Ref"] in decisions and isinstance(a["payload"]["Observable Policy Acceptance"], list)
               for a in artifacts if a["ref"]["Artifact Type"] == "Policy Element Revision" and a["payload"]["Intrinsic status"] == "decided")
    write("P2G_10_POLICY_001.json", {"artifacts": artifacts})
    write("P2G_10_RESULT_001.json", {
        "stage": "P2G-10", "status": "complete", "Input Working Revision Ref": before["ref"],
        "Result Working Revision Ref": result["ref"], "Produced Candidate Disposition Ref Set": dispositions,
        "Result Current Candidate Policy Element Disposition Effective View": dispositions,
        "is_p2g10_complete": True,
        "checks": {"18_candidates_one_disposition_each": True, "15_elements_decided": True,
                   "single_leaf_decisions": True, "acceptances_present_consistent": True,
                   "delegation_five_criteria_per_decision": True, "intake_unchanged": True,
                   "open_questions_empty": True, "cross_references_and_digests_valid": True},
        "Finding Triage": [
            {"classification": "material_policy", "disposition": "15 explicit CF-D06 delegated decisions applying existing agreements",
             "elements": list(current), "evidence": "P2G_10_POLICY_001.json exact decisions/reasons/impacts/five delegation criteria"},
            {"classification": "implementation_finding", "disposition": "P2G-20 formal source binding required",
             "statement": "즉시 정산 vs ack, actual file atomicity/recovery, current node rollback, choice participant context, concurrent stale writer, V9/V4 boundary, public/host consumers reviewed next; no implementation claim"},
            {"classification": "out_of_scope", "disposition": "three excluded candidates recorded with future owners",
             "statement": "future-state schemas, sweep removal, Unity/excluded unique content not preimplemented"}],
        "Open Policy Question Log": [], "Evidence Gap Log": [],
        "new_user_request_required": False, "next": "P2G-20", "production_mutation": False})
    view = (HERE / "WORKING_POLICY.ko.md").read_text().replace("`WR-CRRP-001`", "`WR-CRRP-002`")
    view = view.replace("현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 입력 인수는 정책 동결·제품 구현·독립 감사 완료가 아니다.",
        "현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다. 정책 요소·결정 각15개, candidate18개(승격15/범위외3), 미결 질문0개다.\nP2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [판정·분류](P2G_10_RESULT_001.json)가 근거다. 코드 검토·동결·제품 구현 완료를 뜻하지 않는다.")
    view += "\n각 결정은 기존 합의·CF-D06 위임의 이유·영향·다섯 조건을 기록한다. Continue와 명시적 SavePoint rewind의 후퇴 사용량 수명을 구분하고, 단일 active 저장의 경쟁 입력이 현재 권리를 덮어쓰지 못하게 한다. 저장 구현 수단과 정확한 변경 경계는 P2G-20이 소유한다.\n"
    (HERE / "WORKING_POLICY.ko.md").write_text(view)
    print("U04 P2G-10 complete: 15 decided elements/decisions, 18 dispositions, zero questions; no production mutation")


if __name__ == "__main__":
    main()
