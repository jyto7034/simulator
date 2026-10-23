from pathlib import Path
import json,re,hashlib,shutil,difflib,os
R=Path('/mnt/f/work/simulator')
O=R/'core_enhanced/docs/testing/integration_failure_triage_20261014_001'
C=R/'core_enhanced/docs/testing/core_baseline_failure_correction_20261014_001'
original=json.loads((C/'INTEGRATION_REMAINDER.json').read_text())
index=json.loads((C/'EXECUTION_INDEX.json').read_text())
rows=[]
def row(key,label,purpose,overlap,unique,candidates=(),recommend='유지·준비 교정'):
    match=[x for x in original if key in x['test']]; assert len(match)==1,key
    x=dict(match[0]); x.update(label=label,purpose=purpose,overlap_assessment=overlap,unique_assertions=unique,candidate_test_names=list(candidates),recommendation=recommend)
    rows.append(x)
row('every_live_starter','시작·영입 장비 명세','실제 RON의 12명 훈련 장비·액티브 파편·보상풀 분리 검사',
    'U06는 지급 사본을 현재 정의와 비교한다. 따라서 잘못된 정의와 잘못된 지급이 함께 바뀌면 통과할 수 있어, 명시적 12명 명세 검사 전체를 대체하지 못한다.',
    '12명 정확한 장비/파편 ID, baseline 파편 없음, 일반 보상풀에서 훈련 무기 제외',
    ['u06_starter_six_normal_sets_grant_exact_copies_and_continue_bindings','u06_starter_recruit_normal_loadout_and_post_grant_failure_rollback'], '유지·기대값/문서 정합화')
row('external_fixture_registers','외부 등록·장착 기본 확인','외부 pack 등록 뒤 유물 1개 지급/장착 검사',
    '성공 이전 테스트가 동일 외부 content(), 공개 grant/equip, 정확한 copy/owner/slot/count를 이미 검사한다. 파일 존재·pack 수 및 최초 슬롯0 확인은 명시적으로 옮겨야 한다.',
    'root/u08_public/relics.ron 존재, selection.packs=2, 최초 단일 사본의 owner/slot0',
    ['u08_public_successful_transfer_moves_exact_copy_and_only_its_employee_contributions'], '고유 검증 통합 후 제거')
row('two_employees_same_skill','동일 스킬·직원별 조회','동일 스킬을 가진 직원 둘의 유물 기여와 조회 순수성 검사',
    '실전 HP 바인딩 검사와 query owner의 예산·원자성 검사는 일부 겹치지만 public DTO의 직원/세대/범위와 출력 비노출까지 대체하지 못한다.',
    '100→180/100 직원별 분리, copy 식별, prospective scope/generation, 반복 결과, public roots 불변, runtime ID/ordinal/epoch 미노출',
    ['literal_observation_off_same_skill_two_employees_keep_distinct_actual_bindings','relic_query_outer_native_retained_admission_precedes_targets_penalties_and_generation'])
row('real_fragment_form','파편·폼·무기 공급자','실제 장착 파편·폼·무기별 효과 적용과 휴면 사유 검사',
    '실전 rebind 검사는 결과 피해를 다룬다. 공개 plan의 provider copy와 equipment/form/fragment 휴면 사유는 별도 계약이다.',
    '최종 절삭133, 기본공격+9, 다른 공급자 차용 금지, 폼/파편 교체 시100과 정확한 dormant reason',
    ['typed_form_and_weapon_changes_rebind_actual_skill_and_basic_damage_in_fresh_battles'])
row('split_tags','태그·실행 맥락·치유 구분','태그 교집합 및 런타임 대상 필요/치유 기본공격의 조회 표현 검사',
    '실전 피해·순수 숫자 연산으로는 아직 실행하지 않은 대상의 unresolved 상태나 Heal의 Damage target 제외를 검증할 수 없다.',
    '분리 태그3/3·교집합34, dormant tags, runtime_context_required, 치유 무기에는 basic_attack_damage 없음')
row('corrupt_penalty','타락 페널티·인증 전투력','타락 후 기본/유효 능력치 및 roster·인증 전투력 일치 검사',
    '실전 저항/피해와 인증 기본값 검사에 일부 겹치지만 공개 plan과 roster의 동일 투영 및 읽기 순수성은 남는다.',
    '기본값 불변, 방어-15/마저-10, roster 유효값 일치, peak 불변, 피해210, 반복 query roots 불변',
    ['physical_magic_modifiers_and_resistance_consume_relic_once_and_negative_penalties_in_real_hp','all_three_copy_slot_permutations_preserve_actual_damage_penalties_and_certified_base'])
row('canonical_copy_order','정렬·반올림·오버플로','장착 슬롯 순서와 무관한 공개 plan 및 최종 overflow 표현 검사',
    '순수 산술 및 실전 슬롯순열은 일부 중복이다. plan 전체 정렬과 공개 final/overflow fault, 실패 query 원자성은 대체하지 못한다.',
    '중복 Normal 합산280, 슬롯 재배치 후 plan 전체 동일, final overflow, roots 불변, 해제 후 u32::MAX 복구',
    ['all_three_copy_slot_permutations_preserve_actual_damage_penalties_and_certified_base','relic_numeric_once_additive_fraction_and_final_narrow'])
row('missing_weapon','무기 부재 조회','무기 해제 시 유물 보존과 능력치/피해 공급자 부재 검사',
    '자원 owner 검사나 정상 장착 수치 검사는 무기 부재 DTO 계약의 대체가 아니다.',
    'Normal/Corrupt 양쪽에서 copy/slot/variant/material 보존, roster effective_stats 없음, 공개 plan의 없는 무기 처리와 읽기 불변')
row('normal_reward_grants','무료 타락·다중 선택','정상 보상 유물들의 무료 타락 후보·선택·재시도 검사',
    '외부 trial 및 내부 event owner가 일부 보상/이력/예산을 다루지만 이 테스트의 다중 후보 필터·두 선택·정확 재시도·완료 후 권한 상실 전체를 대체하지 못한다.',
    '촉매0 무료 선택, 이미타락/미지원 제외, 후보 정렬·수치·페널티, 장착 유지, 두 이력, 재시도 멱등성, 다른copy/비활성 이벤트 거절',
    ['u08_external_ron_normal_grants_savepoint_event_replay_and_fresh_process','relic_event_history_survives_a_later_legal_dismantle_of_its_selected_copy'])
row('empty_candidates','빈 후보·거절 이력','후보 없는 두 장면에서 거절과 과거 재시도 검사',
    '내부 decline owner의 admission·메모리 계측과 부분 중복이지만 공개 후보/장면/정확 이력·잘못된 copy 재시도는 별도다.',
    '빈 후보에서도 can_decline, null 선택 이력 두 개, 정확 재시도, 잘못된 UUID 거절·roots 보존, 자원0',
    ['relic_event_decline_only_activates_independent_history_owners_and_admits_before_allocation'])
row('wrong_authority','분기 권한·잘못된 명령','EmployeeCheck 확정 분기와 정상 명령 경계의 잘못된 식별자 거절 검사',
    '저장 문서 위조 거절 검사와 대체 관계가 아니다. 사용자 입력 명령의 node/event/scene/copy 검증은 유효한 공개 계약이다.',
    'Check 장면 타락 금지, committed outcome과 분기 일치, 네 잘못된 식별자 각각 mutation 없음, 유효 선택의 source_scene 기록')
row('shop_rejects_all','판매 금지·변형/장착 조합','유물 판매 금지와 구체 오류 및 상태 불변 검사',
    '일반 shop 정책이나 유물 owner 자원 검증만으로 정확한 relic_not_sellable 및 네 조합을 대체하지 못한다.',
    'Normal/Corrupt×장착/미장착 4조합, unsellable UUID 정렬, quantity0 우선, 미지 UUID offer_unavailable, 전체 roots 보존')
row('disabled_shop','판매 상태·정책 우선순위','유물 판매에서도 상점 밖/판매 비활성 정책을 우선 적용',
    '판매 허용 상점의 relic_not_sellable 테스트와 입력 상태 및 기대 오류가 다르다.',
    '상점 밖 not_in_shop_state, 비활성 selling_unavailable, 두 실패 모두 roots 불변')
row('empty_relic_state','빈 목록 JSON 생략','유물이 없을 때 파생 판매불가 목록의 JSON 필드 생략',
    '비어 있는 자료구조/clone 비용 및 비어 있지 않은 판매불가 목록 검사와 다르다. 작다는 이유만으로 중복 삭제하지 않는다.',
    'request_relic_state 결과에서 not_sellable_copy_uuids 키 자체가 없음')
row('successful_transfer','정상·타락 유물 이전','공개 EquipRelic로 정확한 사본을 다른 직원에게 이전',
    '이름이 비슷한 normal_grant_and_three_slot_transfer...는 중복 정책에 의한 이전 거절을 검사한다. 성공 이전을 대체하지 못한다.',
    '두 variant의 copy/resources 보존, 유일 slot 소유, 이전 직원100/새 직원180 또는210, penalty 귀속, copy만으로 역조회 unequip',
    ['normal_grant_and_three_slot_transfer_use_current_variant_duplicate_policy'])
row('abandon_then','포기·새 런 초기화','포기 후 새 런에서 유물·슬롯·촉매·분진 초기화와 정상 Continue 검사',
    'U12 loans/abandon은 vault 자산·직원 종료를 다룬다. 활성 유물 네 root 초기화, 구 copy 거절 및 새 런 복원은 별도다.',
    '비어있지 않은 유물/두 슬롯/촉매4/분진23→새런0, 이전 UUID 거절, checkpoint roundtrip',
    ['u12_lifecycle_zero_one_two_loans_move_exact_copies_and_abandon_loses_only_active_assets'])
row('unfinished_event','미완료 이벤트 Continue','이벤트 전 유료 타락 비용은 유지하며 미완료 무료 타락/이력만 되감고 재진입·완료·복원',
    'U04는 미완료 이벤트 전투 복원, 기존 U08 trial은 SavePoint Load 및 완료 이벤트 history의 fresh Continue다. 미완료 이벤트 Continue와 사전 유료 타락 비용 조합을 대체하지 못한다.',
    '진입전 checkpoint 바이트 동일, 선택 유물180 복원·기지급 타락210 유지, 촉매2/분진11 유지, 원앱 불변, 재진입 이력 초기화, 완료의 두 이력 보존',
    ['u04_unfinished_event_combat_restores_before_event_instead_of_after_choice','durable_continue_and_savepoint_keep_independent_variant_cost_and_slot_owners','u08_external_ron_normal_grants_savepoint_event_replay_and_fresh_process'])
assert len(rows)==17
sourcefiles=list((R/'core_enhanced/src').rglob('*.rs'))+list((R/'core_enhanced/tests').rglob('*.rs'))
candidate_names={n for x in rows for n in x['candidate_test_names']}
locations={}
for p in sourcefiles:
    data=p.read_text()
    for name in candidate_names:
        m=re.search(r'fn\s+'+re.escape(name)+r'\s*\(',data)
        if m: locations[name]={'path':str(p.relative_to(R)),'line':data[:m.start()].count('\n')+1}
for x in rows:
    x['source']=next({'path':str(p.relative_to(R)),'line':i} for p in sourcefiles if '/tests/' in str(p) for i,line in enumerate(p.read_text().splitlines(),1) if 'fn '+x['test'].split('::')[-1]+'(' in line)
    x['existing_coverage']=[]
    for n in x['candidate_test_names']:
        assert n in locations,n
        passed=[]
        for run in index:
            for test_name,status in run['test_results'].items():
                if test_name.endswith(n) and status=='ok':
                    passed.append({'run':run['label'],'test':test_name,'stdout':run['stdout']})
        x['existing_coverage'].append({'test':n,**locations[n],'prior_pass_evidence':passed})
    x['next_preparation']='공통 support의 Standard→Expedition 및 select_starter_employees.loan_copy_ids=[] 정합화'
    if x['label']=='시작·영입 장비 명세': x['next_preparation']='containment_trainee shield/last_line: 현재 데이터·validator와 구 룰북 간 정합화. hq_containment_aide greatsword/breakthrough 유지.'
    if x['label']=='포기·새 런 초기화': x['next_preparation']+='; 성공 재시작 replace_active_run=false, 활성 런 거절은 false로도 별도 확인하여 금지 true 때문에만 통과하지 않게 함; 실제 PlayerProfile 설치 후 Continue'
    if x['label']=='미완료 이벤트 Continue': x['next_preparation']+='; 현재 Expedition layout/실제 Event 진입 가능한 authored fixture; fresh/after_completion에 해당 PlayerProfile 설치'
    x['duplication_decision_basis']='소스 assertion/공개 경로 비교; 기존 후보의 PASS는 동등성 증명이 아님'
    x['probe_runs']=[]
    for result in sorted(O.glob('diagnostic*.result.json')):
        run=json.loads(result.read_text()); output=(O/run['stdout']).read_text()
        m=re.search(r'^test '+re.escape(x['test'])+r' \.\.\. (ok|FAILED)',output,re.M)
        if m:x['probe_runs'].append({'run':result.stem,'status':m.group(1),'stdout':run['stdout'],'stderr':run['stderr']})
(O/'MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print('wrote',len(rows),'rows')

def link(label,p):return '['+label+']('+os.path.relpath(p,O)+')'
text='''# 추가 통합 실패 17개 — 목적·원인·중복 판정

2026-10-14 사용자 요청에 따른 분류다. 권고는 **16개 유지·교정, 1개는 고유 assertion 통합 후 제거**다. 고유 검증을 버리고 즉시 삭제할 수 있는 완전 중복은 0개다. 이번에는 원본 테스트·제품 코드·공식 데이터의 교정이나 삭제를 적용하지 않았다.

최초 실패는 **폐기된 Standard 시작 준비 16개, 시작 장비 기대 불일치 1개**다. 격리 사본에서 현재 준비/기대값만 최소 보정한 결과 **15개는 본래 assertion 전체 통과**, 생명주기 2개는 추가 준비 문제에서 중단했다. 이 2개의 이후 동작은 미검증이며, 확인된 제품 결함은 0개지만 제품 무결함을 뜻하지 않는다.

| 실제 정비 유형 | 수 | 권고 |
|---|---:|---|
| 공통 시작 준비 노후화 | 14 | 고유 목적 13개 유지, 기본 등록·장착 확인 1개 통합 |
| 생명주기 준비 복합 노후화 | 2 | 목적 유지, 재시작·프로필·시설 fixture 교정 |
| 명시 장비 기대와 현행 데이터/구 문서 불일치 | 1 | 명세 검사 유지, 기대값·문서 정합화 |

중복은 이름이나 기능 영역만으로 판단하지 않았다. 입력 상태·공개 진입 경로·검증 값·실패 때 보존 대상까지 비교했다. 아래 기존 후보는 이전 실행의 실제 통과 근거를 연결했지만, 그 통과 자체가 완전 대체의 증거는 아니다. 개별 정확한 테스트명·위치·원 실패·후보 실행 근거는 [MATRIX.json](MATRIX.json)에 있다.

## 개별 판정

| # | 목적 | 다른 검사와 겹쳐도 남는 고유 검증 | 권고 |
|---:|---|---|---|
'''
for i,x in enumerate(rows,1):
    text+=f"| {i} | {link(x['label'],R/x['source']['path'])} | {x['unique_assertions']} | {x['recommendation']} |\n"
text+='''
## 통합 후보 1개와 유지 판단의 핵심

`prospective::u08_public_external_fixture_registers_new_definitions_before_app_setup`은 동일 외부 fixture를 사용하는 `lifecycle::u08_public_successful_transfer_moves_exact_copy_and_only_its_employee_contributions`에 통합할 수 있다. 후자는 Normal/Corrupt 두 변형의 공개 grant/equip, 정확한 사본·소유자·슬롯·개수, 직원별 기여 및 역방향 해제까지 검사하며 격리002에서 통과했다.

단, 단순 삭제는 권고하지 않는다. `content()`에 외부 relic 파일 존재와 pack 수 2의 검증을 옮기고, 성공 이전 테스트의 최초 장착에 단일 copy/owner/slot0의 정확한 검증을 옮겨야 한다. 이후 기존 slot2→새 직원 slot1 이전 검증도 유지한다. 이 조치가 반영되고 통과한 뒤 독립 smoke 테스트만 제거하는 권고이며, 현재는 적용하지 않았다.

- 시작 장비 테스트는 U06와 다르다. U06의 지급 사본↔현재 정의 일치는 현재 정의 자체의 정확성을 고정하지 않는다. 이 테스트의 12명 명시 ID와 일반 보상풀 분리는 유지할 가치가 있다.
- 수치 plan 6개는 내부 산술·실전 피해·query 예산 검사와 일부 겹친다. 공개 DTO의 provider·휴면 사유·정렬·unresolved/overflow·무기 부재·조회 불변을 함께 대체하는 기존 검사는 확인되지 않았다.
- 이벤트의 잘못된 node/event/scene/copy 명령은 정상 공개 입력 경계 검사다. 외부 저장 바이트 위조 검사와 혼동해 제거하지 않는다.
- `normal_grant_and_three_slot_transfer_use_current_variant_duplicate_policy`는 본문상 중복 정책에 의한 이전 **거절**을 검사한다. 성공 이전 테스트의 대체가 아니다.
- 작은 빈 목록 테스트도 `not_sellable_copy_uuids`가 빈 배열이라는 뜻이 아니라 **JSON 키 자체의 생략**을 확인하므로 유지한다.
- 기존 U08 외부 trial은 SavePoint Load와 완료 이벤트 history의 fresh Continue를 검사한다. 미완료 이벤트 Continue에서 사전 유료 타락 비용을 유지하면서 임시 타락/이력만 되감는 검사를 완전히 대체하지 않는다.

## 원인과 교정 범위

공통 시작 준비는 U12의 `Tutorial | Expedition` 계약에 맞춰 `Standard`를 `Expedition`으로 바꾸고, 시작 직원 선택에 필수 `loan_copy_ids: []`를 제공해야 한다. 격리002에서 이 보정만으로 공통 준비 대상 14개의 본래 assertion이 모두 통과했다.

포기 후 새 런 테스트는 모드/필수 필드 보정 후에도 성공 경로의 `replace_active_run: true`에서 `invalid_action`으로 중단했다. 현행 계약은 true 자체를 금지한다. 성공 재시작은 false로 바꾸고, 활성 런 거절도 false로 별도 확인해야 활성 런 조건을 실제로 시험한다. 이후 새 앱 Continue 전에는 해당 런의 authoritative PlayerProfile을 설치해야 한다. 프로필 문제는 소스/계약 확인이며 이번 실행은 해당 지점까지 도달하지 않았다.

미완료 이벤트 Continue 테스트는 공통 준비 보정 후 `event_node()`가 구형 `records_annex` 슬롯을 찾지 못해 중단했다. fixture가 구형 facility/node assignment 파일을 덮어쓰지만 현행 Expedition layout에서 요구 이벤트가 만들어지지 않는다. 현재 시설 작성 방식으로 실제 진입 가능한 Event 경로를 구성하고 두 fresh 앱에 해당 PlayerProfile을 설치해야 한다. 모드를 Tutorial로 바꾸거나 admin 진입으로 대체해 저장 가능한 실제 경로 검증을 약화하면 안 된다. 이 테스트의 이벤트 진입 이후 assertion은 이번에 검증하지 않았다.

시작 장비는 `containment_trainee`가 현재 Tank이며 `standard_training_shield`/`training_last_line`을 갖는다. 현재 validator도 시작 Tank의 Shield를 요구한다. 반면 구형 장비 규칙표는 탱커2 Greatsword/Axe·breakthrough로 남아 있고 테스트도 대검을 기대한다. 따라서 현재 데이터에 맞춘 기대값 보정과 구 규칙표의 정합화 항목으로 분류한다. 영입 Dealer인 `hq_containment_aide`의 greatsword/breakthrough까지 일괄 변경하면 안 된다. 역할에 따른 시작 지급 제한을 일반 장비 이전 제한으로 확장해서도 안 된다.

근거: '''
refs=[('U12 준비 계약','core_enhanced/docs/goals/core_expedition_lifecycle/execution/single_goal/product_parts/PREPARATION_001.ko.md'),('공식 직원 정의','game_resources/data/employees/definitions.ron'),('시작 장비 validator','core_enhanced/src/game/data/validation.rs'),('구 장비 규칙표','core_enhanced/docs/game_rules/equipment-and-fragments.md')]
text+=', '.join(link(n,R/p) for n,p in refs)+'.\n'
text+='''
## 격리 실행과 한계

| 실행 | 변경/범위 | 관측 |
|---|---|---|
| 직전 integration_021/023 | 원본17, 변경 없는 입력의 원 실패 재사용 | Standard16, shield/greatsword1 |
| diagnostic001 | 격리 사본의 mode·필수 선택 필드·시작 장비 기대만 보정 | 장비1 통과. U08 16은 진단용 resources 심볼릭 링크가 안전한 content loader에 거절됨 |
| diagnostic002 | resource를 실제 디렉터리로 복사하고 동일 U08 바이너리 실행 | 14 통과, 재시작 true/구형 이벤트 슬롯 2 실패 |

001의 content_acquisition은 진단 환경 구성 오류이며 원본 제품/테스트 결함으로 집계하지 않는다. 원본 데이터에 대한 심볼릭 링크를 실제 복사로 바꾼 뒤 002로 바로 재실행했고, 이때 U08 소스와 바이너리는 동일하다. 원 테스트의 assertion은 제거하지 않았다.

원본에서17개가 해결됐거나 전체 통합 테스트가 green이라고 주장하지 않는다. 추가 생명주기 교정과 통합 제거는 다음 구현 작업이며, 이번 산출물은 분류와 근거다. 기존138 교정 및 U14 동결 이력은 그대로 유지하며 U15 착수나 별도 Goal 완료 판정을 하지 않는다.

실행 명령·exit code는 [001](diagnostic001.result.json)과 [002](diagnostic002.result.json), 실제 assertion 결과는 [001 로그](diagnostic001.stdout.log)와 [002 로그](diagnostic002.stdout.log), 격리 변경은 [diff](diagnostic001.diff), 원본 불변 확인은 [INPUT_VERIFICATION.json](INPUT_VERIFICATION.json)에 있다. 재현용 사본은 `probe/`에 보존하며 Cargo.toml의 path dependency와 resources 위치는 현재 workspace 기준이다.

## 비교 근거 상세

'''
for i,x in enumerate(rows,1):
    text+=f"{i}. **{x['label']}** — `{x['test']}`\n\n   {x['overlap_assessment']}\n\n"
    for c in x['existing_coverage']:
        text+='   - '+link(c['test'],R/c['path'])+f" (line {c['line']})"
        passes=c['prior_pass_evidence']
        if passes:
            p=next((p for p in passes if p['run']=='full_core_016'),passes[-1])
            text+='; '+link(p['run']+' PASS',C/p['stdout'])
        elif c['test'].startswith('u08_public_successful_transfer'):
            text+='; [격리002 PASS](diagnostic002.stdout.log)'
        text+='\n'
    text+='\n'
(O/'REPORT.ko.md').write_text(text)
