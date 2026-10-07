#!/usr/bin/env python3
"""U05 P2G-10: approved policy decisions, without source/runtime mutation."""
import copy
import json
from initialize import HERE, artifact, write

POLICIES = [
('배치 초기량', 'IN-006:140; IN-002 U05/B02',
 '최초 배치와 재배치의 새 전투 개체는 파편별 최대 공명의 20%를 내림한 값으로 시작한다. 철수 전 개체의 공명·사용 잠금을 새 개체로 이전하지 않는다.',
 ['최대 1/4/5/99/100의 최초 배치와 재배치에서 각각 0/0/1/19/20으로 시작한다.', '재배치가 이전 개체의 충전량이나 진행 중 사용 자격을 상속하지 않는다.'],
 'P03의 명시 초기량과 재배치의 새 전투 개체 수명을 적용한다. 전술 이동처럼 같은 개체를 유지하는 동작은 재배치 초기화가 아니다.'),
('전투 시간 자동 충전', 'IN-006:141-147; IN-002 U05/B02',
 '배치되어 살아 있는 직원은 전투 시간 1초마다 공명1을 자동 획득한다. 수치는 초기 조절값이다. 공격·방어·작업·침묵은 이를 막지 않으며 정지·철수·전투불능에는 충전하지 않는다. 초과량과 차단 중 놓친 충전은 비축하거나 나중에 보충하지 않는다. 기본 공격 방출·가한 피해·받은 피해의 기본 획득과 Rest 공명5 보너스는 제거한다. Rest HP 회복과 명시 콘텐츠 자원은 이 제거 대상이 아니다.',
 ['공격 없는 치유 직원도 같은 전투 시간에 충전하며 배속은 wall time만 바꾼다.', 'Pause/철수/전투불능·상한/차단 시간을 지난 뒤 누락분이 한꺼번에 더해지지 않는다.', '기본 공격·피해·Rest는 공명 추가분을 만들지 않고 Rest HP 회복은 유지된다.'],
 '명시된 B02를 현재 직원 충전 owner에 적용한다. 기존 콘텐츠 고유 자원이나 새로운 회복 규칙은 바꾸지 않는다.'),
('양수 획득 공통 gate', 'IN-006:146-147,195; IN-002 U05/B04',
 'Stun·Freeze 또는 실제 스킬 사용 잠금 동안 자동 충전과 파편·유물의 명시 추가 획득을 포함한 모든 양수 공명 획득을 막는다. 침묵만으로 획득을 막지 않는다. 소비·감소·배치 초기화는 양수 획득으로 오인하지 않는다.',
 ['같은 양수 추가 획득을 기본·명시 effect 경로에 적용해도 HardCC/사용 잠금 중 증가하지 않는다.', 'HardCC와 사용 잠금이 겹치면 둘 모두 해소된 뒤에만 증가하며 하나의 종료가 다른 이유를 지우지 않는다.'],
 '모든 양수 획득이라는 명시 범위를 공통 자원 경계에 적용하며 음수 소비와 새 개체 초기량은 별도 행위다.'),
('유효 접수와 소비 자격', 'IN-006:149-150,174-182; IN-002 U05/B03 and start gate',
 '최신 조건을 통과한 최종 입력은 그 접수에서 현재 공명을 전부 정확히 한 번 소비하고 해당 사용에 귀속된 실행 자격과 획득 잠금을 만든다. pending 실행은 이미 소비한 공명이 0이라는 이유로 거절하지 않고 같은 자격을 사용한다. 자격은 다른 사용/개체/스킬로 이전하거나 재사용하지 않는다. 실제 실행 시점은 기존 시전·일시정지 계약을 따른다.',
 ['Pause 중 정상 접수에도 즉시 전량 소비하고 재개 뒤 같은 사용이 추가 소비 없이 한 번 시작한다.', '같은 사용의 중복 입력/시작 콜백이나 이미 pending인 두 번째 입력은 이중 실행·자격 복제·이중 소비를 만들지 않는다.', 'noresonance/잘못된 조건 입력은 공명·사용 자격을 만들거나 바꾸지 않는다.'],
 'U05가 명시한 validate→소비/credential→start의 단일 사용 소유권을 적용한다. 소비 시점을 실행으로 미루거나 실행 자체를 접수 순간으로 앞당기지 않는다.'),
('실제 사용 종료', 'IN-006:152-161; IN-002 U05/B04',
 '즉발은 효과 적용과 사용 동작 종료, 발사체는 발사 동작 종료, 연속 공격은 마지막 공격 동작 종료, 채널은 채널 종료까지 잠근다. 일정 시간 버프는 해당 사용의 버프가 실제로 끝날 때까지 잠근다. 독립 장판·소환물·지속 피해와 영구 버프는 생성·부여 동작 종료까지만 잠근다. 하나의 사용에 여러 해당 의무가 있으면 남아 있는 사용 의무가 모두 끝나야 종료하며, 독립 child 생존이나 발사체 명중 자체를 추가 의무로 만들지 않는다. 사용 종료 뒤 별도 1초 잠금은 없다.',
 ['느린 발사체 명중 전과 독립 소환물/장판/DoT 생존 중에도 발사·생성 동작이 끝나면 충전 가능하다.', '연속 공격 중간/채널 도중/실제 시간제 버프 유지 중에는 충전하지 않는다.', '영구 버프는 무한 잠금을 만들지 않으며 같은 사용의 시간제 버프 의무와 연속 동작이 겹쳐도 조기 해제하지 않는다.'],
 '명시 유형별 종료를 해당 사용의 소유된 의무에 적용한다. 혼합 사용은 동일 소유권의 남은 의무를 보존하며 독립 객체의 수명을 상속하지 않는다.'),
('취소와 제어 상태', 'IN-006:163-167; IN-010/011 U01 cancellation',
 'Stun·Freeze는 시전·연속 공격·채널을 취소하고 환불·자동 재개하지 않는다. 이미 적용한 시간제 버프는 전투 시간에 따라 유지/만료하며 그 사용 잠금도 유지한다. 침묵은 새 시작만 막고 이미 접수한 사용을 소급 취소하지 않는다. 취소된 미방출 작업은 재실행하지 않으며 이미 방출한 독립 효과는 자신의 기존 수명을 유지한다. 개체의 철수·죽음은 기존 수명 정리 계약을 따르며 다른 객체의 방출 효과를 임의 환수하지 않는다.',
 ['HardCC 도중/종료 후 취소된 채널·combo가 재개되지 않고 공명 환불이 없다.', '시간제 버프 중 HardCC가 먼저 끝나도 버프 사용 잠금이 남고, 버프가 먼저 끝나도 HardCC 획득 차단은 남는다.', '접수 후 silence와 접수 전 silence를 구분하며 U01의 이미 방출된 projectile/area 취소 경계를 보존한다.'],
 '명시 CC/silence 차이와 U01의 미방출/방출 소유권을 함께 적용한다. 종료 후 새로운 비용·환불 예외는 만들지 않는다.'),
('필수 대상 상실', 'IN-006:166-167,180-182; IN-009 Skill Execution Lifecycle',
 '시전 중 직접 효과에 필요한 고정 유닛이 무효가 되면 해당 사용을 환불 없이 취소하고 남은 사용 의무가 없으면 충전 잠금을 해제한다. 무효 유닛 직접 효과를 다른 유닛으로 이전하지 않는다. 이미 접수한 타일 anchor를 사용하는 범위 효과에는 원래 anchor 유닛 생존을 필수로 추가하지 않으며, 방출된 효과는 기존 자체 수명을 따른다.',
 ['직접 필수 대상의 사망·철수·적대/공중 자격 상실 뒤 다른 대상에게 사용이 이전되지 않고 재충전 가능 상태를 관측한다.', '유닛 기반 범위는 접수 tile을 유지하며 anchor 유닛의 이동·소멸만으로 geometry를 옮기거나 환불하지 않는다.'],
 'P03의 필수 대상 취소와 같은 장의 고정 tile anchor 규칙을 함께 적용한다. 직접 대상 identity와 이미 캡처한 공간 기준을 구별한다.'),
('명시 대상 종류와 최종 검증', 'IN-006:171-178; IN-009 Skill Cast Selection',
 '자신·자동 유닛·플레이어 유닛·플레이어 타일 선택을 명시하며 누락/다른 종류를 자동 선택으로 보정하지 않는다. 새 접수는 생존/배치·수동 가능 여부·공격 태세·행동 제한·CC/침묵·장착 스킬·가득 찬 공명과 대상의 해당 관계/공중/사거리/효과 자격을 최신 상태에서 검사한다. 자동 유닛의 실제 효과 usefulness 규칙은 유지한다. 수동 타일은 유효 전장 tile과 사거리로 선택하며 적 점유나 현재 hostile 피해 대상을 필수로 요구하지 않는다.',
 ['wrong kind/missing payload/out of range/ineligible unit/noresonance를 명시 거절하고 자원을 소비하지 않는다.', '유효 빈 타일의 수동 타일 사용은 접수되지만 전장 밖/void/사거리 밖 타일은 거절된다.', '자동 유닛 후보는 유효 효과가 없으면 선택하지 않고 기존 결정적 target rule을 유지한다.'],
 '새 P03의 빈 타일 허용은 현재 canonical의 PlayerTile hostile-set 필수와 의도적으로 다르다. U05에서 그 gate만 전환하며 automatic/PlayerUnit usefulness를 일괄 폐기하지 않는다.'),
('미리보기와 선택 상태', 'IN-006:178-184; IN-007 포커스와 배속; IN-009 Skill Cast Selection',
 '미리보기는 읽기 전용이며 최종 사용은 최신 조건을 다시 검사한다. 실패는 선택 상태·배속·공명을 그대로 유지한다. 취소는 비용 없이 선택을 끝내고 기존 포커스 복원 배속으로 돌아간다. 성공은 선택 대상을 고정하고 기존 공통 포커스 종료/배속 복원 계약을 함께 적용한다. 수동 스킬용 추가 대기와 수동 기본 공격은 도입하지 않는다.',
 ['preview 뒤 대상/공명/CC/스킬이 바뀌면 최종 입력을 다시 판정하며 실패 시 focus/playback/resonance/state revision이 바뀌지 않는다.', '미리보기 반복과 취소는 소비하지 않고 성공만 한 번 소비와 포커스 종료를 만든다.'],
 'P03/P05의 명시 비변경/복원 계약을 기존 정상 public 입력에 유지한다.'),
('접수 anchor와 재지정', 'IN-006:180-182; IN-009 SkillTarget/TileArea',
 '성공 시 유닛 identity 또는 타일과 유닛의 접수 당시 tile을 고정한다. 직접 플레이어 대상은 이후 유효성 상실 시 다른 적으로 옮기지 않으며 유닛 기반 범위/타일 범위는 접수 anchor를 유지한다. 플레이어 지정 selection과 단계별 자동 재지정의 결합은 작성 단계에서 거절한다. 비플레이어 selection의 명시 RetargetOnStep은 기존 계약을 보존한다.',
 ['이동/사망/occupant 변경 뒤에도 manual area의 anchor가 접수 tile이고 환불하지 않는다.', 'manual RetargetOnStep RON은 거절하고 기존 automatic retarget skill은 작성 의도대로 실행한다.'],
 'U05/B09는 플레이어 지정 재지정 금지다. 기존 자동 스킬의 명시 step rule까지 제거하는 확장이 아니다.'),
('현재 침식 caller 호환', 'IN-002 U05 prerequisite/completion/residual; IN-006 공명 공통규칙',
 '현재 침식 사용 caller도 유효 접수 시 한 번 소비한 실행 자격으로 기존 시작 gate를 통과해야 한다. 공명 소비와 종료 reset/재소비가 충돌하지 않게 한다. 현재 침식 연출·실행 타이밍과 피해 내용은 유지하며 입력 순간으로 실행을 앞당기지 않는다. 새 적 분류별 침식 피해는 U19가 소유한다.',
 ['정상 현재 침식 사용은 접수 소비 뒤 0 공명에서도 pending start를 통과하며 피해/연출 순서는 기존 재개 시점과 일치한다.', '침식 거절/중복/CC 취소에 추가 소비·환불·이중 실행이 없다.'],
 'U05 카드가 별도 명시한 호환 경계를 적용하며 U19 정책을 앞당겨 구현하지 않는다.'),
('공개 작성과 전종 적용', 'IN-002 U05 together-boundaries/residual; IN-001',
 '현재 모든 skill 및 Flow·시간제 버프·채널·독립 생성의 실제 사용 종료를 공개 작성 계약으로 표현하고 전종 적용성을 확인한다. 제거한 기본 획득/임의 postcast lock 필드를 live RON에서 조용히 무시하지 않는다. query는 사용 잠금과 획득 불가 이유를 실행 사실과 일치하게 관측한다. 서로 독립적인 action control 종료·effect settlement·resource lifetime을 사용 잠금 종료와 합치지 않는다.',
 ['공개 loader→catalog/query→normal command→실제 실행에서 전종 종료 규칙을 추적하고 미분류/암묵 fallback이 없다.', '제거 필드의 작성은 정확한 field 경로 오류로 거절되며 새 정상 작성 fixture는 실행된다.', '발사 뒤 effects unsettled와 사용 잠금 해제가 동시에 사실일 수 있고 query가 이를 모순 없이 표시한다.'],
 'U05의 공개 사용 수명과 전종 점검 요구를 기존 R04/R05/R08–12의 별도 관측/자원 소유권과 함께 적용한다. 구체 schema/자료구조는 P2G-20/40에서 정한다.'),
('복원과 저장 보존', 'IN-012..017 U04 closure; IN-004 CF-D01; IN-002 U05 restore fixtures',
 '동일 actor 재접속은 접수 자격·선택/배속·실제 사용 잠금과 전투 상태를 중복 소비 없이 보존한다. 프로세스 종료의 미확정 전투 Continue와 명시 SavePoint load는 U04의 해당 복구 경계를 그대로 따른다. 지원 형식은 새 상태를 일관되게 복원하고 미지원 구형 저장은 원본 보존과 명시 거절을 적용한다. 숨은 초기화로 권리·진행을 바꾸지 않는다.',
 ['접수/시전/시간제 버프/방출 뒤 같은 actor 재접속 및 지원 snapshot roundtrip에서 자격과 종료가 일치하고 재소비하지 않는다.', '새 프로세스 Continue의 전투 입장 전 복원과 SavePoint rewind가 U04 결과·후퇴/load·자산 계약을 유지한다.', '미지원 형식 fixture의 원본 bytes를 보존하고 자동 덮어쓰기/조용한 reinterpretation을 하지 않는다.'],
 '새 필드를 기존 복구 owner에 포함하되 U04가 구분한 재접속/Continue/rewind 수명을 변경하지 않는다.'),
('선행 인수와 완료 경계', 'IN-001; IN-010..017; IN-002 U05',
 'U01 취소·U04 복원 및 선행127조건과 별도 R06 계약을 영향에 따라 인수/재검증한다. 유효 과거 증거와 사용자 변경·감사 원본을 보존한다. 자체 검증과 구현 비참여 GPT-6 Astra High 독립 감사 및 finding 교정·재감사가 닫히기 전에는 의존 후속 구현을 시작하지 않는다. 이후 미래 actor/장비/보스/Unity 기능은 해당 owner가 이 공통 계약을 소비한다.',
 ['선행 조건마다 현재 적용성·의도된 변경·검증 증거가 추적되고 남은 문제를 숨기지 않는다.', '전종 적용과 정상 공개 headless 경로를 포함한 U05 증거를 독립 감사가 확인한 뒤 다음 owner로 인계한다.'],
 '연속 요청의 감사 gate와 승인 범위를 그대로 적용한다. 기존 증거 재사용은 변경 영향의 currentness를 확인한 경우에만 한다.'),
]

def main():
    intake, before = json.loads((HERE/'P2G_00_INITIAL_001.json').read_text())['artifacts']
    basis = {'Input Working Revision Ref': before['ref'], 'Exact User-Approved Target Scope/Change Request': None,
             'Policy Intake Snapshot Ref': intake['ref']}
    artifacts, current, decisions, dispositions = [], {}, [], []
    for n,(name,span,statement,acceptance,reason) in enumerate(POLICIES,1):
        eid=f'PE-CRSL-{n:02}'
        opened=artifact('Policy Element Revision',f'PER-CRSL-{n:02}-OPEN-001',{
            'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':None,'Policy Statement':statement,
            'Decision Revision Ref':None,'Observable Policy Acceptance':[],'Intrinsic status':'open'})
        decision=artifact('Decision Revision',f'DR-CRSL-{n:02}-001',{
            'Predecessor Decision Revision Ref':None,'Affected Element IDs':[eid],'Decision Name':name,'Exact Decision':statement,
            'Decision Authority':'AI delegated material-policy decision applying adopted explicit agreements; not a new user answer',
            'Exact User Approval Reference':{'intake_ref':intake['ref'],'item':'IN-001','delegation':'IN-004 CF-D06/07','scope':'U05/B02,B03,B04,B09 current-state boundary'},
            'Source Reference':{'intake_ref':intake['ref'],'span':span},'Reason':reason,'Impact':acceptance,
            'Delegation Criteria':{'existing_agreements_preserved':'채택한 P03/P05와 U05의 소비·실제 사용 종료·빈 타일·현재 침식 타이밍을 적용',
                'derived_from_ownership':reason,'no_new_game_rule':'기존 명시 비용/초기량/충전/취소를 적용; 새 비용·보상·확률·환불·상속 예외 없음',
                'local_impact':'현재 전투 개체의 공명·입력·사용 수명과 동일 상태의 관측/복원만 전환; 독립 자산/진행 유지',
                'user_data_preserved':'기존 감사·사용자 변경·저장 원본 보존; 미지원은 명시 거절하고 자동 삭제/변환 없음'}})
        decided=artifact('Policy Element Revision',f'PER-CRSL-{n:02}-DECIDED-001',{
            'Stable Element ID':eid,'Predecessor Policy Element Revision Ref':opened['ref'],'Policy Statement':statement,
            'Decision Revision Ref':decision['ref'],'Observable Policy Acceptance':acceptance,'Intrinsic status':'decided'})
        disposition=artifact('Candidate Policy Element Disposition Record',f'CD-CRSL-{n:02}-001',{
            'Candidate ID':f'C-CRSL-{n:02}','P2G-10 Evaluation Basis':basis,'Source Reference':decision['payload']['Source Reference'],
            'Candidate Statement':statement,'Disposition':'promoted','Stable Element ID':eid,
            'Initial Open Policy Element Revision Ref':opened['ref'],'Decision Revision Ref':decision['ref']})
        artifacts.extend([opened,decision,decided,disposition]);current[eid]=decided['ref'];decisions.append(decision['ref']);dispositions.append(disposition['ref'])
    for n,(statement,owner) in enumerate([
        ('새 적 분류별 침식 피해와 전체 보호/보스 기능','U19; 현재 caller 자격/타이밍만 U05'),
        ('독립 child 전체 생존을 사용 잠금으로 묶거나 미래 actor/장비/유물 기능을 선추가','U05 제외; U07 이후 해당 owner'),
        ('미합의 최종보스·Unity 화면·전체 미래 로드맵 상세 설계','ROADMAP USER-01/02와 이후 owner; 연속 요청 범위')],1):
        d=artifact('Candidate Policy Element Disposition Record',f'CD-CRSL-OUT-{n:02}-001',{
            'Candidate ID':f'C-CRSL-OUT-{n:02}','P2G-10 Evaluation Basis':basis,'Source Reference':{'intake_ref':intake['ref'],'span':'IN-002 U05 exclusions; IN-001'},
            'Candidate Statement':statement,'Disposition':'out_of_scope','Evidence/Decision Reference':owner,
            'Stable Element ID':None,'Initial Open Policy Element Revision Ref':None})
        artifacts.append(d);dispositions.append(d['ref'])
    payload=copy.deepcopy(before['payload']);payload.update({'Predecessor Working Revision Ref':before['ref'],
        'Current Policy Element Revision Map':current,'Decision Revision Ref Set':decisions})
    result=artifact('Working Revision','WR-CRSL-002',payload);artifacts.append(result)
    for a in artifacts:
        assert artifact(a['ref']['Artifact Type'],a['ref']['Artifact ID'],a['payload'])==a
    assert len(current)==len(decisions)==14 and len(dispositions)==17
    assert len({a['ref']['Artifact ID'] for a in artifacts})==len(artifacts)
    write('P2G_10_POLICY_001.json',{'artifacts':artifacts})
    write('P2G_10_RESULT_001.json',{'stage':'P2G-10','status':'complete','Input Working Revision Ref':before['ref'],
        'Result Working Revision Ref':result['ref'],'Produced Candidate Disposition Ref Set':dispositions,
        'Result Current Candidate Policy Element Disposition Effective View':dispositions,'is_p2g10_complete':True,
        'checks':{'17_candidates_one_disposition_each':True,'14_elements_decided':True,'single_leaf_decisions':True,
            'acceptances_present_consistent':True,'delegation_five_criteria_per_decision':True,'intake_unchanged':True,
            'open_questions_empty':True,'cross_references_and_digests_valid':True},
        'Finding Triage':[
            {'classification':'material_policy','disposition':'14 delegated decisions applying adopted P03/P05/U05; exact basis and five criteria per decision','elements':list(current),'evidence':'P2G_10_POLICY_001.json'},
            {'classification':'implementation_finding','disposition':'P2G-20 formal source review required','statement':'Existing canonical PlayerTile usefulness differs from adopted empty-tile rule; old CastEnd resonance reset/action settlement and new use lifetime need distinct ownership. No source facts or implementation success claimed.'},
            {'classification':'out_of_scope','disposition':'Three excluded candidates with future owners','statement':'new corrosion damage, future actors/independent-child lock, unapproved unique content/Unity/future design'}],
        'Open Policy Question Log':[],'Evidence Gap Log':[],'new_user_request_required':False,'next':'P2G-20','production_mutation':False})
    view=(HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CRSL-001`','`WR-CRSL-002`').replace(
        '현재 P2G-00 완료, 다음은 P2G-10이다. initial 정책 요소·결정·미결 질문은 비어 있다.\nP2G-10 Completion Predicate: N/A. 입력 인수는 정책 동결·제품 구현·독립 감사 완료가 아니다.',
        '현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다. 정책 요소·결정 각14개, candidate17개(승격14/범위외3), 미결 질문0개다.\nP2G-10 Completion Predicate: true. [정책·결정](P2G_10_POLICY_001.json)과 [판정](P2G_10_RESULT_001.json)이 근거다. 코드 검토·동결·구현 완료를 뜻하지 않는다.')
    view+='\n차기 명시 빈 타일 허용은 기존 PlayerTile hostile 집합 필수 gate를 전환한다. 사용 잠금은 기존 action control·effect settlement·resource lifetime과 구분한다. 현재 침식 피해/실행 시점을 유지하며 접수 자격만 함께 전환한다. 각 AI 위임 결정은 이유·영향·CF-D06/07 및 다섯 기준을 기록했다.\n'
    (HERE/'WORKING_POLICY.ko.md').write_text(view)
    print('U05 P2G-10 complete: 14 decided elements, 17 dispositions, zero questions; no production mutation')

if __name__=='__main__':
    main()
