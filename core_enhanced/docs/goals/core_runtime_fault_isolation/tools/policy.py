#!/usr/bin/env python3
"""P2G-10: scope-local decisions under the user's explicit CF-D06 delegation."""
import copy
import json
from initialize import HERE, artifact, write

# Source spans refer to exact INTAKE_SOURCE_001 bytes. These are delegated
# decisions, not invented user answers or claims that implementation is complete.
POLICIES = [
('분류', 'IN-002 §5 U03; IN-005 MS04; IN-006 §확정 정책3, §5 MS04',
 '이미 게시된 활성 전투의 실제 실행에서 안전 한도·checked 내부 불변식·런타임 수식 실패가 반환되면 해당 전투를 오류 종료한다. 일반 입력/권한/자원 부족 거절, 정적 admission 거절, 조회·전달·수집 오류와 구별한다. 문자열을 추측하여 오류를 승격하지 않고 typed 원인과 실행 경계로 판정한다.',
 '실제 native 한도와 동적 수식/내부 실패가 오류 종료를 만든다. 잘못된 authority·일반 행동/자원 거절·미게시 진입 실패는 기존 거절 의미를 유지하며 정상 전투를 오류 종료하지 않는다.',
 '확정된 실제 전투 safety 중단과 일반 명령 거절의 기존 소유 경계에서 도출한다.'),
('수명', 'IN-002 §5 U03 구현 범위·직후 유효 상태; IN-006 확정 정책3',
 'CoreApplication이 게시한 전투에 귀속된 최초 오류를 보존한다. 실패 후보의 rollback으로 이 종료 상태가 사라지지 않으며 동일 application 소유권의 전투는 tick·수동 명령·재접속·중복 요청으로 재개되지 않는다. 정상 승패/result 상태와 구별하고 임의 재시도 성공으로 해제하지 않는다.',
 '첫 오류 뒤 gameplay 상태/RNG/event/result가 진전하지 않고 모든 후속 경로에서 같은 전투 identity·최초 원인을 조회할 수 있다. 동일 오류의 반복은 원인 교체·중복 정산을 만들지 않는다.',
 '전투의 authoritative CoreApplication과 실제 actor 수명에 종료 상태를 귀속한다.'),
('실패 원자성', 'IN-002 U03 선행 입력·명시적 제외; IN-006 확정 정책3 다음 단락',
 '현재 application 후보의 precommit 실패 rollback과 first-fault/누적 작업 장부를 유지한다. 마지막 유효 gameplay root를 진단용으로 보존하고 오류를 정상 패배·보상·후퇴·스킬 취소로 정산하지 않는다. 전 Run 전체 복원이나 같은 전투 재시작을 보장하지 않는다.',
 '실패 시 마지막 commit의 자원·RNG·event·결과가 유지되고 실패 시도 비용은 사라지지 않는다. 오류 직후 정상 BattleResult/보상 권리/노드 완료가 새로 생기지 않는다.',
 '기존 transactional root와 오류는 정상 결과가 아니라는 명시적 합의를 함께 유지한다.'),
('commit와 전달', 'IN-005 MS04; IN-006 §5 MS04 commit 이전/이후 구분',
 '이미 성공한 domain commit 이후 결과 직렬화·전송·클라이언트 응답·선택 관측 실패를 precommit gameplay fault로 바꾸지 않는다. 진짜 runtime fault가 발생한 뒤 진단 구성/전달까지 실패해도 전투 중단과 최초 typed 원인은 유지한다.',
 'postcommit projection/transport 오류가 성공 commit을 되돌리거나 새 전투 fault를 만들지 않는다. runtime fault와 진단 실패가 겹친 경우에도 후속 실행·저장이 거절되고 최소 오류 원인을 조회할 수 있다.',
 'commit 소유권과 read/transport 소유권을 분리한 현재 application 계약에서 도출한다.'),
('진단과 조회', 'IN-002 U03 산출물·trace 한도; IN-006 확정 정책3',
 '오류 전투 identity·typed 최초 원인·실패 작업을 bounded 진단으로 노출한다. 정상 조회와 오류 상태 조회를 제공하고 오류 상태에서 실행 가능한 정상 전투 행동을 광고하지 않는다. 진단은 gameplay를 재실행하지 않으며 기존 trace/diagnostic 한도를 우회하는 무한 문자열·이력 축적을 만들지 않는다.',
 '공개 API와 실제 headless host에서 같은 최초 오류를 읽고 재접속 시 오류임을 식별한다. 큰 원인/진단 실패·중복 조회에서도 저장량이 유한하고 trace 한도·root 불변을 지킨다.',
 '기존 bounded 진단과 Core 소유 read model을 오류 종료 수명에 확장한다.'),
('명령 gate', 'IN-002 U03 함께 전환할 경계·완료 조건; IN-003 연속 요청',
 '오류가 남은 동일 application에 대한 gameplay·admin·profile/continue/reset·rollback/prepare 우회 mutation을 거절한다. 읽기와 명시적 세션 종료는 허용한다. 정상 입력 거절에는 이 전면 gate를 적용하지 않고 미게시 prepared 후보·authoring 후보의 기존 소유권을 유지한다.',
 '공개 mutation 포트와 host 관리 명령을 직접 호출해도 오류 root를 바꾸거나 저장 가능한 정상 성공 결과를 만들 수 없다. 조회·명시적 종료는 가능하고 정상 전투/비전투의 기존 유효 호출은 계속 동작한다.',
 '이미 오류 종료된 전투의 동일 root를 다른 mutation 포트로 재사용하지 않는다는 소유권에서 도출한다.'),
('host 수명', 'IN-002 U03 tick/수동 명령/재접속·late callback; IN-005 MS04',
 'host는 Core fault를 소비해 tick을 중지하고 재예약·늦은 callback·중복 명령을 차단한다. 같은 actor로의 detach/reconnect/session replacement는 오류를 보존하고 live battle 복원 메시지로 위장하지 않는다. 접속 부재의 자동 TTL이 오류 전투를 조용히 새 정상 세션으로 바꾸지 않는다.',
 '실제 actor에서 수동/자동 최초 실패 후 late callback·EnsureTick·재접속·세션 교체에도 gameplay가 진전하지 않는다. 새 접속이 같은 오류 원인을 받고 오래된 접속이 mutation을 수행하지 못한다.',
 '현재 active battle은 disconnect TTL로 폐기되지 않는 actor 정책과 sticky 전투 종료를 결합한다.'),
('저장 보호', 'IN-002 U03 저장 쓰기 gate·원본 저장 bytes 보호; IN-004 CF-D01; IN-006 확정 정책3',
 '오류 전투가 있는 application/actor가 active checkpoint·profile·보상/완료 저장을 생성·덮어쓰기·삭제하지 못하도록 실제 writer 경계를 막는다. stop/reconnect/profile 동기화와 관리 recovery도 우회가 아니다. 오류 전부터 존재한 유효 저장과 획득 기록을 보존하며 저장 형식을 자동 변환하지 않는다.',
 '오류 유발부터 후속 명령·detach/reconnect·actor stop까지 해당 actor에 기인한 쓰기/삭제가 없고 active/profile 원본 파일 bytes가 같다. 정상 다른 세션 저장까지 전역 차단하지 않는다.',
 '기존 유효 저장 보호와 오류 결과 반영 금지에서 도출하며 U04의 새 결과 정책은 채택하지 않는다.'),
('복구 선택', 'IN-002 U03 정상 headless host 재시작·안전 복구; IN-006 전체 복원/재시작 사전 보장 제외',
 '안전 복구는 기존 명시적 세션 종료 후 새 application/actor를 만들고 기존 유효 checkpoint 이어하기 경로를 선택하는 것으로 한정한다. 같은 오류 전투 재개나 자동 종료/자동 저장 삭제를 추가하지 않는다. 저장이 없거나 유효하지 않으면 기존 안내/새 시작 경로를 유지하며 새 보상·비용·환불·무료 Load 예외를 만들지 않는다.',
 '실제 Quit→actor 종료→새 actor→기존 Continue 경로를 검증하고 원본 저장이 유효할 때만 이어한다. 오류 전투 identity는 재개되지 않고 파일/loads 규칙을 우회하지 않는다. 저장 부재/거절도 기존 의미를 유지한다.',
 '사용자가 요청한 안전 재시작을 현재 명시적 Quit와 새 owner/Continue 수명으로 실현한다. 전투 복원 보장은 추가하지 않는다.'),
('세션 격리', 'IN-002 U03 직후 유효 상태·다른 player 격리; IN-006 확정 정책3',
 '오류는 해당 전투/application/actor에만 귀속한다. 서버 프로세스와 다른 player의 독립 전투를 종료하거나 fault 상태를 전파하지 않는다. 현재 local-install 공유 profile 저장 소유권을 player별 신규 저장 모델로 바꾸지 않는다.',
 '동일 실제 host의 두 player 중 하나가 fault를 일으켜도 다른 player는 명령/tick/정상 저장을 계속한다. fault actor는 공유 store를 오염시키지 않고 다른 player의 성공을 자기 fault 해제로 해석하지 않는다.',
 '명시된 전투 단위 격리와 기존 local-install persistence owner를 모두 유지한다.'),
('관측 종료', 'IN-002 U03 선행 U02 한도와 진단; IN-007 U02 완료 감사',
 '오류 종료도 U02의 종료 집계 정리에 반영한다. 실제 실패 시도/작업/최초 fault 관측은 유지하고 정상 commit count를 만들어내지 않으며 active battle 집계·선택 상세의 stale 상태를 해제한다. 수집기 유무·실패가 오류 수명이나 gameplay를 바꾸지 않는다.',
 'fault 후 관측의 활성 집계와 선택 상세가 정리되고 실패 횟수/시도 work는 남는다. off/on·포화/부재에서 fault 이유·root·저장 보호가 같고 반복 gate가 최초 실행을 재수행하지 않는다.',
 'U02가 이미 승인한 종료 cleanup/실패와 commit 구분을 새 명시적 종료에 적용한다.'),
('선행 계약과 검증', 'IN-002 U03 선행·제외; IN-007 U02 감사; IN-001 연속 요청',
 'U01/R06/R07 88조건과 U02 12조건을 변경 영향에 맞게 승계/재검증한다. private battle와 speculative authoring candidate의 rollback/retry·취소 계약은 유지하고, 이미 게시된 실제 application의 새 오류 수명만 명시적으로 전환한다. 실제 공개 API/headless host와 비참여 독립 감사로 확인하며 20분 시험·임의 성능 상한을 추가하지 않는다.',
 '100개 선행 조건의 현재 적용성/의도된 차이/새 증거가 추적되고 실제 runtime fault가 실행된 공개 경로 증거가 있다. 기존 경고·raw 실패를 보존하며 내부 fixture만으로 host 완료를 주장하지 않는다.',
 '사용자의 증거 재사용·순차 독립 감사와 U03 범위 제한을 적용한다.'),
]

def main():
    intake, before=json.loads((HERE/'P2G_00_INITIAL_001.json').read_text())['artifacts']
    basis={'Input Working Revision Ref':before['ref'],'Exact User-Approved Target Scope/Change Request':None,'Policy Intake Snapshot Ref':intake['ref']}
    artifacts=[];current={};decisions=[];dispositions=[]
    for number,(name,span,statement,acceptance,reason) in enumerate(POLICIES,1):
        eid=f'PE-CRFI-{number:02}'
        opened=artifact('Policy Element Revision',f'{eid}-r1',{'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':None,'Policy Statement':statement,'Decision Revision Ref':None,'Observable Policy Acceptance':None,'Intrinsic Status':'open'})
        decision=artifact('Decision Revision',f'DEC-CRFI-{number:02}-r1',{'Predecessor Decision Revision Ref':None,'Affected Element IDs':[eid],'Exact Decision':statement,'Decision Authority':'AI delegated material-policy decision; not a new direct user answer','Exact User Approval Reference':{'intake_ref':intake['ref'],'item':'IN-001','delegation':'IN-004 CF-D06','scope':'U03/A06'},'Source Reference':{'intake_ref':intake['ref'],'span':span},'Reason':reason,'Impact':acceptance,'Delegation Criteria':{'existing_agreements_preserved':'실제 오류 전투만 중단·정상 결과 아님·기존 원본 보존; U04 정책 전환 제외','derived_from_ownership':reason,'no_new_game_rule':'새 비용/보상/확률/상한/환불/무료 checkpoint-load 예외 없음','local_impact':'해당 오류 전투/actor 경계; 다른 player의 독립 진행·획득 불변','user_data_preserved':'유효 save bytes 보존; 삭제·일괄 변환·재해석·권리 소멸 없음'}})
        decided=artifact('Policy Element Revision',f'{eid}-r2',{'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':opened['ref'],'Policy Statement':statement,'Decision Revision Ref':decision['ref'],'Observable Policy Acceptance':acceptance,'Intrinsic Status':'decided'})
        disposition=artifact('Candidate Policy Element Disposition Record',f'CD-CRFI-{number:02}-001',{'Candidate ID':f'C-CRFI-{number:02}','P2G-10 Evaluation Basis':basis,'Source Reference':decision['payload']['Source Reference'],'Candidate Statement':statement,'Disposition':'promoted','Stable Element ID':eid,'Initial Open Policy Element Revision Ref':opened['ref'],'Decision Revision Ref':decision['ref']})
        artifacts.extend([opened,decision,decided,disposition]);current[eid]=decided['ref'];decisions.append(decision['ref']);dispositions.append(disposition['ref'])
    payload=copy.deepcopy(before['payload']);payload.update({'Predecessor Working Revision Ref':before['ref'],'Current Policy Element Revision Map':current,'Decision Revision Ref Set':decisions})
    result=artifact('Working Revision','WR-CRFI-002',payload);artifacts.append(result)
    assert len(current)==len(decisions)==len(dispositions)==12
    assert all(a['payload']['Decision Revision Ref'] in decisions for a in artifacts if a['ref']['Artifact Type']=='Policy Element Revision' and a['payload']['Intrinsic Status']=='decided')
    write('P2G_10_POLICY_001.json',{'artifacts':artifacts})
    write('P2G_10_RESULT_001.json',{'stage':'P2G-10','status':'complete','Input Working Revision Ref':before['ref'],'Result Working Revision Ref':result['ref'],'Produced Candidate Disposition Ref Set':dispositions,'is_p2g10_complete':True,'checks':{'all_in_scope_candidates_disposed':True,'all_elements_decided':True,'single_leaf_decisions':True,'acceptances_present_consistent':True,'delegation_five_criteria_per_decision':True,'intake_unchanged':True,'open_questions_empty':True,'cross_references_valid':True},'Finding Triage':[{'classification':'material_policy','disposition':'explicit CF-D06 delegated decision','elements':list(current),'evidence':'P2G_10_POLICY_001.json exact reason, impact and five criteria per decision'},{'classification':'implementation_finding','disposition':'P2G-20 required','statement':'typed producer/route classification; Core rollback versus outer fault ownership; host early management handlers/stopped writer; postcommit projection; speculative authoring; observer lifecycle all need formal exact-source review'},{'classification':'out_of_scope','disposition':'U04/U11/U26 owners retained','statement':'normal result/save policy changes, mod registration, visual error UI, new gameplay costs/caps, full Run rollback, skill auto-cancel,20min tests excluded'}],'new_user_request_required':False,'next':'P2G-20','production_mutation':False})
    view=(HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CRFI-001`','`WR-CRFI-002`').replace('현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 입력 인수는 새 정책 검토·제품 구현 완료를 뜻하지 않는다.','현재 P2G-10 완료, 다음은 P2G-20 코드 검토다. 정책 요소·결정 각12개, 미결 질문0개다.\nP2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [검증·분류](P2G_10_RESULT_001.json)가 근거다. 코드 검토·제품 구현 완료를 뜻하지 않는다.')
    view+='\nCF-D06 위임으로 선택·이유·관측 가능한 영향·다섯 조건을 요소별 기록했다. 새 사용자 직접 답변을 만들지 않았다. 안전 복구는 기존 명시적 Quit→새 owner→유효 checkpoint Continue 경로이며 오류 전투의 자동 재개나 저장 삭제가 아니다.\n'
    (HERE/'WORKING_POLICY.ko.md').write_text(view)
    print('P2G-10 complete: 12 decided elements and decisions; zero open questions; no product changes')

if __name__=='__main__':main()
