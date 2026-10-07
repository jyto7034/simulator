"""P2G-10 approved U07-M meanings and explicitly delegated local decisions."""
import copy
import json
from initialize import HERE, artifact, write

POLICIES = [
('배포 기본과 trusted 선택', 'FOLLOWUP4.1/A1',
 '기존 배포 기본 profile의8MiB 포함 모든 기본 한도·기본 거절 의미를 보존한다. 개발 측정 profile은 trusted host가 세션/실행 경계에서 명시 선택하는 유한 설정이다. 콘텐츠·저장·일반 게임 명령은 모드나 한도를 올리지 못한다. 잘못된/과대/미지원 설정은 구체적으로 거절하고 기본값으로 조용히 대체하지 않는다. 플레이 중 자동 전환하지 않는다.',
 ['기존 설정과 생략 입력의 기본 hash·성공/거절이 유지되고 명시 개발 설정만 다른 effective identity와 동작을 갖는다.', '잘못된/과대/비신뢰 입력 완화 시도를 정상 공개 로더/host에서 거절하고 원본을 보존한다.'],
 '후속 요청이 허용한 명시 개발 경로만 추가한다. 기존8MiB 실패나 모든 정상 실행을 새 한도로 재해석하지 않는다.'),
('경고선·유한 비상 상한', 'FOLLOWUP4.1/A2',
 '배포 profile의 정상 retained 예산을 개발 실행의 경고 기준으로 유지하고, 별도로 검증된 유한 비상 cap만 실제 admission 상한으로 적용한다. 코드 허용 상한·선택 cap·경고선·단위·계량 범위를 분리한다. 경고를 hard fault로 먼저 걸었다가 무시하는 방식은 금지한다. 모든 관련 native retained owner/operation/result 경계가 선택 profile에 일치한다. work·clone·객체/입력·expression scratch·overflow 등 별개 보호는 유지한다.',
 ['기본/개발 각각 경고선·cap의 exact/one-over와 owner/operation/result 경계에서 의도된 성공·경고·typed 거절 차이를 입증한다.', '다른 제한과 overflow가 개발 모드에서도 거절되며 유한 비상 cap 이상의 요청은 실행되지 않는다.'],
 '개발 cap은 측정 창 확보용 engineering 선택이며 최종 배포 적정값이 아니다. 후속 계약의 명시 허용 차이와 기본 보존을 구분한다.'),
('누적 allowance·원자 fault', 'FOLLOWUP4.1/A3',
 'operation 시작 보유 extent와 이후 양의 retained growth를 누적한다. 중첩·candidate discard·rollback·해제가 같은 operation의 allowance를 초기화하거나 환급하지 않는다. 새로운 public operation만 현재 owner extent로 시작한다. 비상 cap 또는 다른 hard fault는 기존 sticky typed 오류·candidate 원자 복원·U03 세션 종료/저장 보호를 따른다. 개발 경고만 발생한 정상 실행은 fault 상태가 아니다.',
 ['동일operation 중첩/실패candidate/재시도 누적값과 exact/부족cap, 다음operation 새 시작을 관찰한다.', '늦은 emergency failure에 상태·queue·facts·RNG·저장 원본이 보호되고 U03 정상 fault/재접속/Continue 정책을 유지한다.'],
 '관측용 모드가 실행 자격이나 복원 정책을 우회하지 않게 기존 예산 owner를 재사용한다.'),
('서로 다른 메모리 지표', 'FOLLOWUP4.2/A4,A5',
 '시작/current/최대 accounted retained extent, 누적 admitted growth와 최대 요청/초과 site, allocator live/peak/종료 후 delta와 할당 수, 지원 OS의 process RSS/peak를 구분한다. 각 값은 범위·단위·방법·지원 여부·누락/관측 overhead를 가진다. 서로 더하거나 대체해 가짜 단일 메모리 peak를 만들지 않는다. 실제 heap/RSS는 관측만 하며 게임 admission이나 결정론의 입력이 아니다.',
 ['정상 샘플에 누적 예산과 실제 allocator peak가 독립 필드·범위로 존재하고 시작/종료·실패 최대 요청도 추적된다.', '지원환경 RSS는 실제 OS 측정이고 미지원/실패는 명시적 missing이며 allocator 수치로 채우지 않는다.'],
 '현재 retained admission과 heap peak의 의미 차이를 보존한다. 측정 최대를 이론적 최대나 RSS로 과장하지 않는다.'),
('bounded 수집과 관측 비간섭', 'FOLLOWUP4.2/A4,A5',
 '기존 runtime observation/host 수집을 우선 재사용하여 샘플·버퍼·파일·출력·집계와 cardinality를 유한하게 제한한다. 표본 수·드롭·누락·선택 범위를 표시한다. 평균·상위구간·최대·조건별 요약은 표본 정의와 함께 제공한다. 관측 on/off·collector 부재/포화/실패가 gameplay·영속 결과를 바꾸지 않는다. 수집 속도·RSS·벽시계로 runtime 예산을 조정하지 않는다.',
 ['normal/off/detached/full 관측 모드에서 같은 profile·seed·입력의 gameplay/저장 결과가 동일하다.', '포화·종료·파일상한에서 bounded 크기와 drop/missing이 확인되며 원시 표본과 요약의 수량/분포가 일치한다.'],
 '진단 파이프의 실패를 게임 오류로 승격하지 않는다. 새 원격서비스나 dashboard를 전제로 삼지 않는다.'),
('profile identity·복원 권한', 'FOLLOWUP4.1/4.2/A6',
 '선택 profile의 raw/effective hash·revision을 현재 엔진/콘텐츠/실행 identity와 연결한다. 같은 profile/seed/input은 관측 모드와 무관하게 같은 결과다. replay/승인에서 다른 profile을 같은 실행으로 인수하지 않는다. 저장 데이터가 host profile을 선택하지 않으며 기존 Continue/pre-entry 복원은 현재 trusted host profile로 재검증하고 원본을 보존한다. live clone은 선택된 immutable profile과 실행 의미를 유지한다. 새로운 live-battle disk codec은 도입하지 않는다.',
 ['생략/명시기본 canonical identity와 개발 profile mismatch 거절을 실제 승인/trial/replay 소비 경계에서 검증한다.', 'clone·같은host reconnect·현재host의 Continue/SavePoint 복원에서 profile 선택 권한과 원본 보호가 유지된다.'],
 '기존 U02 effective hash와 U04 현재host 복원 계약을 따른다. 저장값으로 개발 모드가 켜지는 권한 경로를 추가하지 않는다.'),
('정상 Core/host 실측과 재현', 'FOLLOWUP4.2/4.3/A4',
 '현재 지원 정상 CoreApplication과 실제 host 경로에서 bounded 샘플·원시 결과·요약·재현 명령을 제공한다. engine/content/profile/build·관측모드·기기/OS·seed·입력·전투 규모/조합·동시 세션·측정 창·성공/거절/중단을 연결한다. private 직접호출만으로 정상 경로 완료를 주장하지 않는다. U07 원래PC 입력/실패는 비교 가능한 이력으로 보존하고 default/development 결과를 혼합하지 않는다.',
 ['정상 전투와 생성물 소비의 실제 CoreApplication/host 표본을 동일 명령/seed/profile로 재현하고 nonzero 측정·정상 결과를 연결한다.', '기본/개발의 경고초과·hard거절 표본과 원래PC 비교를 분리하며 측정창 밖 상태나 작은 표본의 한계를 명시한다.'],
 '이번 준비 단계에서 현재 실제 수집 경로를 완성하되 모든 콘텐츠와 Unity 플레이 표본을 요구하지 않는다.'),
('반복 실행·수명·회수', 'FOLLOWUP4.3/A6',
 '현재 지원 전투·생성/해제의 반복과 종료 후 회수를 관찰하고 정상 일시 peak와 지속 누적/누락을 구분한다. Pause·재접속·원자 rollback·동시 세션의 범위와 profile별 isolation을 검증한다. 수집기 수명은 battle/profile 자산을 불필요하게 붙잡지 않는다. 기존20분 stress 제외를 유지하며 짧고 명시된 반복/동시 표본으로 준비 경계를 닫는다.',
 ['반복 생성/만료/전투종료 뒤 accounted owner와 실제 allocator 종료delta/host session retention을 측정해 회수 근거를 남긴다.', '동시 다른 세션·관측포화·collector 종료·clone에서 예산 또는 profile이 다른 operation으로 새지 않는다.'],
 '정상 반복 비용의 측정과 장시간/전종 최악부하 증명을 구분한다.'),
('현재 검증·전체 독립 완료', 'FOLLOWUPA7/current continuous request',
 '10정책 요소·10필수 검증차원·181선행 조건·별도31R06의 현재 적용성을 추적한다. 기본 profile 계약은 보존하고 개발 모드에 명시 승인된 허용 범위 차이는 새 증거로 구분한다. 작은 profile/native/collector 경계부터 검증하고 영향받는 최종 Core/host·공개/저장/fault/allocator/PC/R05를 닫는다. 과거 실패/target/감사·수치oracle는 보존한다. exact 동결 뒤 비참여 Astra/High 전체 감사와 finding교정을 Complete/High/open0으로 닫기 전 U08을 시작하지 않는다.',
 ['각 조건·차원에 nonzero selector 또는 근거 있는 exact 재사용, default/dev 별도 결과·baseline·원래실패가 연결된다.', '정역 복원 가능한 current target과 독립 전체 Complete/High/open0가 존재하며 역할·쓰기가 분리된다.'],
 '단계/문서 통과를 제품 완료로 표시하지 않는다. 승인된 새 devprofile을 과거8MiB 실패 면제나 default회귀 은폐에 사용하지 않는다.'),
('후속 수치 조정 소유', 'FOLLOWUP3/4.3/A8',
 'U07-M은 측정·설정 경로 준비 완료까지만 소유한다. U08 이후 도입 owner는 계량과 표본을 유지하고 U11은 profile/승인 identity를 인수한다. U30-C는 목표 Core환경·정상 최대/동시부하와 여유로 후보를 정하고 U30-P는 실제 Unity 플레이로 배포값을 재검토한다. 변경은 새 profile/revision과 전후 raw·이유·영향 검증·독립 인수에 결속한다. 평균만으로 hardcap을 정하거나 heappeak를 누적admission값에 대입하지 않는다.',
 ['재현·수치조정 절차에 profile version/원시전후/영향/후속owner와 미확정 플랫폼·부하·최종값이 명시된다.', 'U07-M 완료 후 U08이 진행 가능하며 최종 Unity 측정을 현재 순환 선행조건으로 삼지 않는다.'],
 '등록된 후속 작업의 단계 소유와 Core 우선 순서를 그대로 구체화한다.'),
]


def main():
    intake, before = json.loads((HERE/'P2G_00_INITIAL_001.json').read_text())['artifacts']
    assert json.loads((HERE/'P2G_00_RESULT_001.json').read_text())['status'] == 'complete'
    basis = {'Input Working Revision Ref': before['ref'], 'Exact User-Approved Target Scope/Change Request': None,
             'Policy Intake Snapshot Ref': intake['ref']}
    arts, current, decisions, dispositions = [], {}, [], []
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        eid = f'PE-CMRP-{n:02}'
        opened = artifact('Policy Element Revision', f'PER-CMRP-{n:02}-OPEN-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': None,
            'Policy Statement': statement, 'Decision Revision Ref': None,
            'Observable Policy Acceptance': [], 'Intrinsic status': 'open'})
        decision = artifact('Decision Revision', f'DR-CMRP-{n:02}-001', {
            'Predecessor Decision Revision Ref': None, 'Affected Element IDs': [eid],
            'Decision Name': name, 'Exact Decision': statement,
            'Decision Authority': 'AI delegated application/detail of registered U07-M and continuous request; not a fabricated new user answer',
            'Exact User Approval Reference': {'intake_ref': intake['ref'], 'items': ['IN-001','IN-002','IN-003','IN-004'], 'specific_span': span},
            'Source Reference': {'intake_ref': intake['ref'], 'item': 'IN-004', 'span': span},
            'Reason': reason, 'Impact': acceptance,
            'Alternatives not selected': 'Silent default cap raise; unlimited mode; post-fault continue; content/save-controlled settings; observer-based gameplay; remote service dependency; final tuning claim from small samples.',
            'Delegation Criteria': {'existing_agreements_preserved': 'Deployment defaults and exact historical verdicts stay intact. Explicit trusted development extension is the registered successor scope.',
                'derived_from_ownership': reason, 'no_unapproved_global_rule': 'No new gameplay cap/rule, save codec, remote service, default budget or final target-platform commitment.',
                'local_impact': 'Trusted profile/native accounting/normal observation and reproduction; U30-C/P own final tuning.',
                'user_data_preserved': 'No reset/clean/stash, original save rewriting or historical failure/target edits.'},
            'Revisability': 'Engineering temporary values are recorded in code-informed design; policy meaning changes return to earliest P2G owner and preserve prior revision.'})
        decided = artifact('Policy Element Revision', f'PER-CMRP-{n:02}-DECIDED-001', {
            'Stable Element ID': eid, 'Predecessor Policy Element Revision Ref': opened['ref'],
            'Policy Statement': statement, 'Decision Revision Ref': decision['ref'],
            'Observable Policy Acceptance': acceptance, 'Intrinsic status': 'decided'})
        disposition = artifact('Candidate Policy Element Disposition Record', f'CD-CMRP-{n:02}-001', {
            'Candidate ID': f'C-CMRP-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': decision['payload']['Source Reference'], 'Candidate Statement': statement,
            'Disposition': 'promoted', 'Stable Element ID': eid,
            'Initial Open Policy Element Revision Ref': opened['ref'], 'Decision Revision Ref': decision['ref']})
        arts.extend([opened, decision, decided, disposition]); current[eid] = decided['ref']
        decisions.append(decision['ref']); dispositions.append(disposition['ref'])
    for n, (statement, locator) in enumerate([
        ('전체 콘텐츠·실제 Unity 플레이 집계와 최종 배포 수치 확정', 'FOLLOWUP3/A8 U30-C/U30-P'),
        ('새 원격 수집 서비스/dashboard와 무제한 또는 자동완화 모드', 'FOLLOWUP4.1/4.2 explicit exclusions'),
        ('현재 U07의8MiB 실패 면제·과거판정 소급변경·개별최종보스/Unity 선구현', 'FOLLOWUP1 and current request'),
    ], 1):
        item = artifact('Candidate Policy Element Disposition Record', f'CD-CMRP-OUT-{n:02}-001', {
            'Candidate ID': f'C-CMRP-OUT-{n:02}', 'P2G-10 Evaluation Basis': basis,
            'Source Reference': {'intake_ref': intake['ref'], 'span': locator},
            'Candidate Statement': statement, 'Disposition': 'out_of_scope',
            'Evidence/Decision Reference': locator, 'Stable Element ID': None,
            'Initial Open Policy Element Revision Ref': None})
        arts.append(item); dispositions.append(item['ref'])
    payload = copy.deepcopy(before['payload']); payload.update({
        'Predecessor Working Revision Ref': before['ref'], 'Current Policy Element Revision Map': current,
        'Decision Revision Ref Set': decisions})
    result = artifact('Working Revision', 'WR-CMRP-002', payload); arts.append(result)
    assert len(current) == len(decisions) == 10 and len(dispositions) == 13
    for a in arts:
        assert artifact(a['ref']['Artifact Type'], a['ref']['Artifact ID'], a['payload']) == a
    write('P2G_10_POLICY_001.json', {'artifacts': arts})
    write('P2G_10_RESULT_001.json', {'stage': 'P2G-10', 'status': 'complete',
        'Input Working Revision Ref': before['ref'], 'Result Working Revision Ref': result['ref'],
        'Produced Candidate Disposition Ref Set': dispositions,
        'Result Current Candidate Policy Element Disposition Effective View': dispositions,
        'is_p2g10_complete': True,
        'checks': {'all13_candidates_dispositioned': True, 'all10_elements_decided': True,
            'observable_acceptances_consistent': True, 'explicit_delegation_per_decision': True,
            'open_questions_empty': True, 'raw_intake_unchanged': True, 'digest_refs_checked': True},
        'Finding Triage': [
            {'classification': 'material_policy', 'disposition': '10 adopted/delegated leaf concerns cover FOLLOWUP A1–A8 without choosing final budget/platform values.'},
            {'classification': 'implementation_finding', 'disposition': 'Preliminary read-only code orientation identified trusted settings, scattered native retained checks, existing bounded observation and profile consumers. Formal source necessity/currentness and implementation design remain P2G20.'},
            {'classification': 'out_of_scope', 'disposition': '3 candidates retained with exact future/exclusion owner.'}],
        'Open Policy Question Log': [], 'Evidence Gap Log': [], 'production_mutation': False,
        'new_user_request_required': False, 'next': 'P2G-20'})
    view = (HERE/'WORKING_POLICY.ko.md').read_text().replace('`WR-CMRP-001`', '`WR-CMRP-002`')
    view = view.replace('현재 P2G-00 완료. 초기 정책 요소·결정·질문 집합은 비어 있으며 다음은 P2G-10이다.\n초기화는 정책 의미·코드 검토·제품 구현·실측 완료가 아니다. P2G-10 Completion Predicate: N/A.',
        '현재 P2G-10 완료, 다음은 P2G-20 정식 코드 검토다.10정책·13candidate(승격10/범위외3), 미결질문0.\nP2G-10 Completion Predicate: true. [정책/결정](P2G_10_POLICY_001.json)과 [검증](P2G_10_RESULT_001.json)이 근거이며 제품 구현/실측 완료는 아니다.')
    for n, (name, span, statement, acceptance, reason) in enumerate(POLICIES, 1):
        view += f'\n## PE-CMRP-{n:02} — {name}\n\n{statement}\n\n근거: {span}. AI 위임 결정 이유: {reason}\n\n'
        view += '\n'.join('- ' + value for value in acceptance) + '\n'
    (HERE/'WORKING_POLICY.ko.md').write_text(view)
    print('U07-M P2G10 complete:10 decided elements/13 dispositions; no product mutation')


if __name__ == '__main__':
    main()
