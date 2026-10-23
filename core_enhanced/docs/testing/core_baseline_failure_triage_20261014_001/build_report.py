"""Build a complete, auditable per-test disposition from preserved observations."""
from pathlib import Path
import json,re,collections,hashlib,tarfile
O=Path(__file__).resolve().parent
R=O.parents[3]
I=json.loads((O/'INTAKE.json').read_text())['tests']
G=json.loads((O/'CURRENT_FAILURES.json').read_text())['groups']
D=json.loads((O/'DIAGNOSTIC_FAILURES.json').read_text())
dfail={r['test']:r['failure'] for r in D}
log=(O/'diagnostic138_002.stdout.log').read_text()
passed=set(re.findall(r'^test (.*?) \.\.\. ok',log,re.M))
reasons={n:g['reason'] for g in G for n in g['tests']}
# Existing coverage is source-inspected; it is not declared newly passing.
coverage={
 'trial':(['core_enhanced/src/authoring/tests.rs:313','core_enhanced/src/authoring/tests.rs:639','core_enhanced/src/game/data/content_pack/tests.rs:703'], '부분 중복. explain/metadata 단위 검사는 실제 TrialSession 접수·실행·영수증·clock·fresh process 및 원본 세션 보존을 대체하지 않는다.'),
 'causal':(['core_enhanced/src/game/world/tests/u10_causal/audit_joint_cases.rs:391','core_enhanced/src/game/world/tests/u10_causal/audit_joint_cases.rs:732','core_enhanced/src/game/battle/core/relic_causal_tests.rs:257'],'부분 중복. 실제 기본공격/장비반격/침식 조합 및 하위 BattleCore 검사는 이 39개의 각 합류·반복·복제본·지연 수명 oracle를 대체하지 않는다.'),
 'resource':(['core_enhanced/src/game/world/tests/resource_lifetimes.rs:1201','core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:175','core_enhanced/src/game/world/tests/u06_starter.rs:751'],'부분 중복. 작성 오류·대여 복제본·장비 호환성 검사는 실제 비어 있지 않은 typed resource의 attempt/run/delivery/chain 수명 및 회수·교환·retry를 대체하지 않는다.'),
 'codec':(['core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:119','core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:175','core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:213','core_enhanced/src/game/world/tests/u12_lifecycle/codec.rs:297'],'부분 중복. current Continue·profile·구형 거절은 존재하지만 정확한 envelope/keyset/checksum, reservation 및 외부 generation/서로 다른 소유자의 보존 검사를 모두 대체하지 않는다.'),
 'starter':(['core_enhanced/src/game/world/tests/u06_starter.rs:122','core_enhanced/src/game/world/tests/u06_starter.rs:159','core_enhanced/src/game/world/tests/u06_starter.rs:208'],'높은 부분 중복. 6개 역할별 세트/복제본 역방향 소유/Continue, 채용 rollback/10명 상한은 존재한다. 기존 queue의 순서·세션 유지·완료 시점이나 snapshot 필드 검사는 별도 확인이 필요하다.'),
 'facility':(['core_enhanced/src/game/world/tests/u13_facility.rs:93','core_enhanced/src/game/world/tests/u13_facility.rs:140','core_enhanced/src/game/world/tests/u13_facility.rs:165','core_enhanced/src/game/world/tests/u14_deterrence.rs:81'],'부분 중복. 정상 방 이동·SavePoint·미지원 활동 거절·preview refresh는 존재한다. 개별 event history/monolith/전체 snapshot 보존은 완전 대체되지 않는다.'),
 'combat':(['core_enhanced/src/game/world/tests/combat.rs:3577','core_enhanced/src/game/world/tests/combat.rs:6031','core_enhanced/src/application/fault_tests.rs:389'],'부분 중복. cursor/retained history/fault 검사는 각각의 paused deploy·raw authored actor·VN·field work·종료 결과 oracle와 다른 경계다.'),
 'fault':(['core_enhanced/src/application/fault_tests.rs:288','core_enhanced/src/application/fault_tests.rs:346','core_enhanced/src/application/fault_tests.rs:389','core_enhanced/src/application/fault_tests.rs:836'],'부분 중복. 다른 fault origin·prepared host tick은 존재하지만 Elapsed work fault 이후 64회 모든 공개 mutation gate/first cause/roots를 검사하는 본 사례를 완전히 대체하지 않는다.'),
 'resolver':(['core_enhanced/src/authoring/resolve.rs:646','core_enhanced/src/authoring/resolve.rs:946','core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:75'],'서로 보완. mandatory loan list의 명령 거절과 JSON pointer 불변성/aggregate node exact-limit은 다른 계약이다.'),
 'api':(['core_enhanced/src/application.rs:1846','core_enhanced/src/game/world/application.rs','core_enhanced/src/game/behavior.rs'],'숫자는 같아도 application↔domain bijection, route/scope/authority, 단일 application owner를 각각 검사한다. 단순 중복 삭제 대상이 아니다.'),
 'content':(['core_enhanced/src/authoring/tests.rs:193','core_enhanced/src/game/data/content_pack/tests.rs:866','core_enhanced/src/game/data/content_pack/tests.rs:1102'],'부분 중복. production diagnostic/외부 generation 보존은 존재하지만 schema 정확성·unknown field·개별 시작 실패 경계나 파일 입력 종류를 모두 대체하지 않는다.'),
}
def purpose(n):
 if 'u10_causal::' in n and 'game::world' in n:return 'causal'
 if 'resource_lifetimes::' in n:return 'resource'
 if 'authoring::trial::' in n or 'authoring_generation_lifetime::' in n:return 'trial'
 if 'fault_tests::' in n:return 'fault'
 if 'authoring::resolve' in n:return 'resolver'
 if 'command_kind_is_' in n or 'every_action_kind' in n or 'every_behavior_kind' in n:return 'api'
 if 'checkpoint_validation::' in n or 'active_run_checkpoint_round_trip' in n or 'durable_continue_and_savepoint' in n:return 'codec'
 if 'recruitment' in n or 'recruit_employee' in n or 'starter_employee_roster' in n:return 'starter'
 if 'game::world::tests::combat::' in n or 'game::battle::' in n:return 'combat'
 if 'game::world' in n and ('facility::' in n or 'snapshot' in n or 'event_history' in n or 'shop_pool' in n or 'admin::' in n):return 'facility'
 return 'content'

def diagnose(n,reason):
 if reason=='role_weapon':return ('stale_fixture','F01','시험 fixture가 Healer/Tank를 포함한 모든 시작 직원을 sword 세트로 덮어써 현재 역할-무기 검증에서 거절된다.','현재 역할에 맞는 장비·active fragment를 준비하고 각 시험이 요구하는 실제 source/owner를 유지한다.')
 if reason=='missing_difficulty':return ('stale_fixture','F02','공통 helper가 첫 허용 slot을 Combat으로 바꾸지만 원래 SavePoint 등 비전투 node의 combat_difficulty를 만들지 않는다.','현행 작성된 Combat slot/난이도 및 방 activity metadata를 일치시키는 준비를 사용한다.')
 if reason=='removed_standard':return ('stale_fixture','F03','Trial preparation이 제거된 Standard 모드를 명령 JSON으로 사용한다.','현행 Tutorial/Expedition의 실제 시작→선발→이동→전투 경로와 binding/receipt index를 함께 갱신한다.')
 if reason=='loan_list':return ('stale_fixture','F04','현행 명령의 필수 loan_copy_ids가 테스트 JSON에 없다.','명시 []와 이에 따른 resolved aggregate node 수를 함께 교정한다. 정확한 한도+한도-1 검사는 유지한다.')
 if reason=='socket_env':return ('environment','E01','UnixListener::bind가 OS EPERM으로 실패하며 socket 입력을 loader에 전달하기 전 중단된다.','UTF-8/directory/socket 입력 검사를 유지하되 socket 부분은 지원 환경에서 실행한다. 환경 실패를 제품 PASS로 세지 않는다.')
 if 'Number(15)' in reason:return ('stale_expectation','X01','현재 writer version 15를 정상 저장하면서 positive test가 12를 기대한다.','현재 positive codec을 갱신한다. 실제 구형 writer bytes에 대한 명시 거절 검사는 별도로 보존한다.')
 if 'left: 80' in reason:return ('stale_expectation','X02','현행 명령 80개에 대해 이전 79개를 고정했다.','현행 전체 집합과 route/owner 계약을 각각 유지한다.')
 if 'left: 46' in reason:return ('stale_expectation','X03','지원 definition kind가 46개인데 45개를 기대한다.','현행 support metadata 집합과 exact output boundary를 유지한다.')
 if 'effective_skill_definition_copy' in reason:return ('stale_expectation','X04','현행 admission site는 effective_skill_definition_copy이며 테스트 진단 문자열만 구형이다.','현재 site를 기대하되 접수 전 identity/context 불변 및 exact ceiling은 유지한다.')
 if 'schema version should load' in reason:return ('stale_expectation','X05','현재 LIVE_RON_SCHEMA_VERSION은 6인데 정상 입력에 5, 진단 기대에 exactly 5를 사용한다.','현재 버전 성공과 이전 버전 거절을 함께 유지한다.')
 if 'legacy_fallback' in reason:return ('stale_fixture','F05','unknown field를 검사하려는 RON이 현재 필수 필드도 누락하여 앞선 parser 오류로 종료한다.','유효한 current policy에서 목표 unknown field 하나만 추가한다.')
 if 'NotImplemented' in reason:return ('stale_fixture','F06','공통 helper가 노드 kind/payload만 바꾸고 시설 activity의 지원 상태를 맞추지 않아 실제 guard에서 거절된다.','현행 지원 활동과 맞는 노드를 준비한다. 미지원 guard를 제품에서 우회하지 않는다.')
 if 'invalid starter employee loadout reference' in reason:return ('stale_fixture','F07','시험용 장비 catalog가 현재 candidate_2의 test_starter_shield를 누락한다.','role-compatible 전체 시작 catalog를 유지하며 목표 VN/generation 실패만 주입한다.')
 if 'ActionCapabilityDto' in reason:return ('stale_expectation','X06','기대 capability 목록에 현행 RequestRelicEffectivePlan 및 terminal StartNewGame 항목이 빠져 있다.','현재 상태별 capability 전체 집합으로 갱신하며 본래 battle/result/state oracle를 보존한다.')
 if 'headquarters' in n or 'session_is_fixed_and_recruitment' in n:return ('stale_expectation','X07','한 명 채용 후 즉시 session 종료·잔여 후보 queue 반환을 기대한다. 현행 규칙은 최대 두 명 채용 또는 명시 종료다.','첫 채용 후 session 유지, 둘째/명시 종료 후 queue 순서 및 node 완료를 분리 검증한다.')
 if 'start_new_game_creates_starter' in n:return ('stale_expectation','X08','모든 시작 직원이 sword를 받는다는 전제가 현재 역할별 세트와 충돌한다.','U06 역할별 시작 검사에 잔여 roster eligibility/ledger consistency/roster_order 비노출 검사를 합친 뒤 중복 본문을 제거할 수 있다.')
 if 'durable_continue_recomputes' in n:return ('stale_fixture','F08','현행 map에서 구형 standard_branching slot 이름을 찾는다. 후반에는 저장 preview를 위조하고 checksum을 재계산한다.','정상 Continue 후 재계산 검사는 현행 지도에서 유지/통합한다. 저장 위조 전용 분기는 로드맵 §1.9에 따라 제거한다.')
 if 'standard_intro_a' in reason or 'standard_branching_a' in reason or 'right: "Standard"' in reason:return ('stale_expectation','X09','현재 Expedition/facility를 준비하면서 Standard 또는 구형 template ID를 기대한다.','현재 authored map/진행/방 상태를 기준으로 의미 있는 snapshot과 복원 assertion을 보존한다.')
 if 'real_trial_replacement_failures' in n:return ('stale_fixture','F09','선발 직후 Available Combat이 있다는 가정으로 position().unwrap()한다. 현행 Expedition은 선행 SavePoint/시설 진행이 있다.','공개 준비 단계와 receipt/binding index를 현행 경로로 갱신한다. generation lifetime 검사는 유지한다.')
 if 'run_checkpoint_preserves_encounter' in n:return ('stale_fixture','F10','임의 event node/VN roots를 주입하고 내부 save helper만 호출한다. 현재 공개 LoadRunCheckpoint 자격이 없어 목표 graph 복원에 도달하지 않는다.','지원 event graph와 실제 SavePoint/load 가능 상태를 준비한다. 정상 보존만 검사하며 임의 저장 위조 검사는 추가하지 않는다.')
 if 'relic_event_history_survives' in n:return ('stale_fixture','F11','helper를 반복 호출하면 같은 첫 slot/node를 Maintenance로 덮어써 event와 maintenance가 동일해진다.','서로 다른 현행 작성 slot을 지정하고 event node identity를 보존한다.')
 if 'live_shop_pool' in n:return ('stale_fixture','F12','run floor_index만 3으로 바꾸고 실제 방 layout/map은 이전 상태로 남겨 저장 layout 검증에서 거절된다.','실제 4층 진행 또는 일관된 authored fixture로 상점 선택/monolith 저장 목적을 유지한다.')
 if 'native_fault_preserves' in n:return ('stale_fixture','F13','시험 전용 max_work=1000이 battle entry snapshot도 수용하지 못해 의도한 Elapsed fault 주입 전에 실패한다.','입장을 허용하되 60초 advance에서 실제 work fault가 발생하는 시험 한도를 선정한다. 제품 기본 한도는 변경하지 않는다.')
 if 'live_ron_defense_route' in n:return ('stale_expectation','X10','장기 전투 종료 후 bounded event log에 초기 UnitSpawned가 영구 잔류한다고 기대한다. 전투는 실제 종료했다.','초기/진행 events_delta에서 spawn을 관측하고 최종 retained log에서는 종료·보존 한도를 검사한다.')
 if 'paused_live_battle' in n or 'track_and_rejoin_raw_live' in n:return ('unresolved','Q01','첫 combat-eligible 직원과 첫 Ground cell을 무조건 결합한다. 현재 역할별 deployment affinity 불일치가 유력하며 진단에서 검증한다.','역할에 맞는 직원/배치 cell을 선택하고 원래 paused/raw-source oracle를 그대로 재실행한다.')
 if 'r12_world_two_employees' in n:return ('unresolved','Q02','보호막 소비·parry는 통과했지만 400ms+200×50ms 안에 전투가 끝나지 않는다. 시간 가정 문제와 종료 결함을 구분해야 한다.','더 긴 유한 관측에서 정상 종료 및 결과·fresh restore를 확인한다. 종료 조건을 완화하거나 강제 종료로 바꾸지 않는다.')
 raise RuntimeError((n,reason))

rows=[]
for i,x in enumerate(I,1):
 n=x['test'];k=purpose(n);cl,cause,why,action=diagnose(n,reasons[n]);paths,overlap=coverage[k]
 disposition='retain_update'
 if cause=='E01':disposition='retain_environment'
 if cause.startswith('Q'):disposition='retain_pending_diagnosis'
 if cause=='X08':disposition='merge_then_remove_duplicate'
 if cause=='F08':disposition='retain_normal_remove_tamper_branch'
 rows.append({'id':f'T{i:03}','test':n,'source':x['source_candidates'][0],'purpose_family':k,'classification':cl,'cause_id':cause,'cause':why,'recommendation':disposition,'next_action':action,'value':'높음: 현재 지원 계약의 독립 관측 지점 또는 상태/소유/실패 경계를 보호함','existing_coverage':paths,'replacement_assessment':overlap,'diagnostic002':{'status':'pass' if n in passed else 'fail','remaining_failure':dfail.get(n)},'scope_note':'내부 fixture 자체는 삭제 사유가 아니다. 지원 경로에서 가능한 정상 상태·실제 장애를 준비하는 경우는 유지. 저장 권위 위조만으로 성립하는 분기는 제외.'})
assert len(rows)==len({x['test'] for x in rows})==138
(O/'MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print(collections.Counter(x['classification'] for x in rows));print(collections.Counter(x['cause_id'] for x in rows))

# Targeted diagnostic 004: current product bytes, only test preparation/observation changed.
probe=(O/'diagnostic5_004.stdout.log').read_text()
probe_pass=set(re.findall(r'^test (.*?) \.\.\. ok',probe,re.M))
for x in rows:
 n=x['test']
 if n in json.loads((O/'diagnostic003_selection.json').read_text()):
  x['diagnostic004']={'status':'pass' if n in probe_pass else 'fail','log':'diagnostic5_004.stdout.log','details':'diagnostic5_004.stderr.log'}
 if x['cause_id']=='Q01':
  assert n in probe_pass
  x.update(classification='stale_fixture',cause_id='F14',cause='관측에서 첫 직원 field_medic의 Ground affinity=false였다. 같은 준비에서 Ground-compatible 직원을 선택하면 원래 paused/raw actor assertion 전체가 통과한다.',recommendation='retain_update')
 if x['cause_id']=='Q02':
  assert n in probe_pass
  x.update(classification='stale_expectation',cause_id='X11',cause='보호막 소비·parry가 통과한 뒤 구형 10.4초 관측 상한에서만 중단됐다. 100.4초 이내의 유한 관측으로 원래 정상 종료·결과 처리·fresh restore assertion 전부 통과했다.',recommendation='retain_update',next_action='전투 종료까지 유한 관측하는 현행 fixture를 사용한다. 100.4초는 이번 진단 상한이며 제품 성능 합격선/종료 시간 계약이 아니다.')
 if x['cause_id']=='X10':
  x['cause']+=' 진단에서 floor_seq=540, retained entries=780, 첫 잔류 시각=12,150ms를 확인했다.'
 if x['cause_id']=='F13':
  x['cause']+=' 시험 한도 3000에서 입장→실제 60초 Elapsed work fault→모든 gate/first cause/roots assertion이 통과했다.'
 x['evidence_level']='isolated_full_test_pass' if n in passed|probe_pass else ('environment_failure_before_product' if x['classification']=='environment' else 'current_first_failure_explained_by_source_and_contract')
 x['product_behavior_limit']='이 진단은 해당 준비/기대값 때문에 실패함을 판정한다. 다른 입력/미도달 후속 경로의 제품 무결함을 주장하지 않는다.'
(O/'MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
counts=dict(collections.Counter(x['classification'] for x in rows)); decisions=dict(collections.Counter(x['recommendation'] for x in rows))
summary={'tests':138,'current_original':{'passed':0,'failed':138,'ignored':0,'filtered':1965},'classification':counts,'recommendation':decisions,'confirmed_product_defects':0,'isolated_full_test_pass_unique':len(passed|probe_pass),'test_side_not_yet_full_pass':137-len(passed|probe_pass),'environment_cases':1,'original_source_changes':0,'removed_original_tests':0}
(O/'RESULT.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')

cause_sources={
 'F01':['core_enhanced/src/game/data/content_pack/tests.rs:44','core_enhanced/src/game/data/validation.rs:517'],
 'F02':['core_enhanced/src/game/world/tests/mod.rs:1628','core_enhanced/src/game/world/tests/u10_causal.rs:227'],
 'F03':['core_enhanced/src/authoring/trial.rs:1110','core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:47'],
 'F04':['core_enhanced/src/authoring/resolve.rs:946','core_enhanced/src/game/world/tests/u12_lifecycle/start_identity.rs:75'],
 'F05':['core_enhanced/src/game/data/run_policy_data.rs:1407'],
 'F06':['core_enhanced/src/game/world/tests/mod.rs:1628','core_enhanced/src/game/world/facility_rooms.rs:80'],
 'F07':['core_enhanced/src/game/data/mod.rs:1143'],
 'F08':['core_enhanced/src/game/world/tests/combat.rs:937','core_enhanced/src/game/world/tests/combat.rs:1077','core_enhanced/docs/roadmaps/core_first/ROADMAP.ko.md:345'],
 'F09':['core_enhanced/src/game/data/content_pack/tests.rs:1572'],
 'F10':['core_enhanced/src/game/world/tests/snapshots_and_start.rs:814','core_enhanced/src/game/world/node_flow.rs:1405'],
 'F11':['core_enhanced/src/game/world/tests/u08_event_owner.rs:307','core_enhanced/src/game/world/tests/mod.rs:1628'],
 'F12':['core_enhanced/src/game/world/tests/node_sessions.rs:1046'],
 'F13':['core_enhanced/src/application/fault_tests.rs:113'],
 'F14':['core_enhanced/src/game/world/combat.rs:228'],
 'X01':['core_enhanced/src/game/world/active_run.rs:39'],
 'X02':['core_enhanced/src/application.rs:1846'],
 'X03':['core_enhanced/src/authoring/tests.rs:233'],
 'X04':['core_enhanced/src/game/battle/core/commands.rs'],
 'X05':['core_enhanced/src/game/data/live_ron.rs:2584'],
 'X06':['core_enhanced/src/game/world/action_capability.rs'],
 'X07':['core_enhanced/src/game/recruitment.rs','core_enhanced/docs/game_rules/run-and-map.md:185'],
 'X08':['core_enhanced/src/game/world/tests/snapshots_and_start.rs:1079','core_enhanced/src/game/world/tests/u06_starter.rs:48'],
 'X09':['core_enhanced/src/game/world/tests/mod.rs:550','core_enhanced/src/game/world/tests/u13_facility.rs:93'],
 'X10':['core_enhanced/src/game/world/tests/combat.rs:2781','core_enhanced/src/game/battle/event_log.rs:783'],
 'X11':['core_enhanced/src/game/world/tests/r09_combat.inc.rs:334'],
 'E01':['core_enhanced/src/game/data/content_pack/snapshot/tests.rs:232']}
for x in rows:x['cause_sources']=cause_sources[x['cause_id']]
(O/'MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
labels={'stale_fixture':'구형 준비 입력','stale_expectation':'구형 기대값','environment':'환경'}
displ={'retain_update':'유지·보정','retain_environment':'유지·환경 분리','merge_then_remove_duplicate':'통합 후 중복 제거','retain_normal_remove_tamper_branch':'정상 경로 유지·변조 분기 제거'}
lines=['# Core 기존 실패 138개 진단 및 처분 판단','',
'현재 실패 원인은 **테스트 준비/기대값 노후화 137개, 실행 환경 1개**다. 제품 결함으로 확정된 사례는 0개다. 이는 제품에 결함이 없다는 선언이 아니다. 격리 복사본의 시험 준비·기대값만 보정하여 65개는 본래 assertion 전체를 통과했고, 나머지 테스트 측 72개는 현재 실패 원인을 설명했지만 모든 후속 동작의 통과까지 확인하지 않았다. 원본 테스트는 여전히 138개 실패다.','',
'이번 범위는 원인·가치·기존 검증 중복·유지/제거 판단이다. 원본 제품 코드와 테스트는 수정하거나 삭제하지 않았다. 검증 대상 동결 source 713개는 작업 종료 시 모두 동일하다.','',
'| 원인 | 개수 | 판정 |','| --- | ---: | --- |',
'| 준비 입력 노후화 | 109 | 장비 역할, 필수 명령 필드, 전투 난이도/활동, 현재 시작·지도·저장 경로와 불일치 |',
'| 기대값 노후화 | 28 | 버전/명령 수/진단 문자열/채용 완료 시점/보존 로그/시간 가정이 구형 |',
'| 환경 | 1 | Unix socket 생성이 EPERM. 제품 loader 호출 전 실패 |','',
'| 권고 | 테스트 수 | 실행 여부 |','| --- | ---: | --- |',
'| 유지하고 현행 계약으로 보정 | 135 | 미적용 |',
'| 기존 검사에 잔여 항목을 합친 뒤 중복 본문 제거 | 1 | 미적용 |',
'| 정상 경로 유지, 저장 위조 전용 분기 제거 | 1 | 미적용 |',
'| 환경 조건을 분리하여 유지 | 1 | 미적용 |','',
'전체 테스트를 즉시 삭제할 대상으로 확정한 것은 없다. 단순히 이름에 legacy/v12/Standard가 있거나 같은 모듈에 새 테스트가 있다는 이유로 삭제하지 않는다.','',
'## 제거·통합 판단','',
'1. `start_new_game_creates_starter_employee_roster_and_equipment`: U06의 `u06_starter_six_normal_sets_grant_exact_copies_and_continue_bindings`가 여섯 역할별 장비·파편과 역방향 소유·Continue를 더 넓게 검사한다. 기존 테스트에 남은 combat eligibility 수, inventory ledger consistency, run snapshot의 `roster_order` 비노출 assertion을 적절한 현행 검사에 합친 뒤 중복 본문을 제거하는 것이 좋다. 지금 그대로 삭제하면 잔여 검증을 잃는다.',
'2. `durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache`: 정상 Continue 후 전투 setup 재계산은 유지한다. 저장 JSON의 enemy를 바꾸고 checksum을 재계산하는 후반 분기는 [로드맵 §1.9](../../roadmaps/core_first/ROADMAP.ko.md#19-검증-범위-제한--외부-임의-변조-방어-제외)의 제외 범위다. 이 분기를 살리기 위한 교정은 하지 않는다. U14 preview refresh와 부분 중복하지만 새 인스턴스 Continue→실제 전투 시작까지 완전히 같지는 않아 정상 분기를 즉시 삭제하지 않는다.',
'3. 명령 수 검사 3개는 각각 application/domain bijection, route/scope/authority, 단일 application owner를 검사한다. 숫자가 같아도 대체 관계가 아니다.',
'4. U10 인과 체인 39개와 외부 RON resource lifetime 18개는 유지 가치가 높다. 최근 통합 검사나 하위 단위 검사가 각 복제본·합류·지연·회수·실제 public commit 경계를 모두 대체하지 않는다.',
'5. 정상 저장 version 검사 8개도 유지한다. U12의 current/old-version 검사는 정확한 keyset, reservation, 외부 generation 및 독립 owner 보존을 전부 대신하지 못한다.','',
'## 실행 근거와 한계','',
'- 원본 `current138_001`: 0 PASS / 138 FAIL / 0 ignored / 1965 filtered, test 18.63초. 과거 실패 목록과 정확히 같은 138개를 현재 source에서 재현했다.',
'- `diagnostic138_001`: 격리 복사본의 docs include fixture 9개가 누락되어 빌드 실패했다. 시험 실행 0이며 PASS 증거가 아니다. 정확한 현재 bytes를 복사하고 [추가입력](DIAGNOSTIC_EXTRA_INPUTS.json)을 보존했다.',
'- `diagnostic138_002`: 61 PASS / 77 FAIL / 0 ignored / 1965 filtered, test 39.24초. 제품 실행 코드는 그대로이며 [시험 변경](DIAGNOSTIC_001.diff)만 사용했다. 39개 U10 인과 체인·8개 정상 codec·7개 전투 준비·3개 명령 수·definition count·site·resolver·member-scope 검사가 통과했다.',
'- `diagnostic5_003`: 진단 코드가 sibling private method를 호출하여 빌드 실패했다. 시험 실행 0. 진단을 공개된 effective profile의 affinity 관측으로 바꾸고 [후속 변경](DIAGNOSTIC_004.diff)을 보존했다.',
'- `diagnostic5_004`: 4 PASS / 1 FAIL / 0 ignored / 2098 filtered, test 9.88초. Ground와 맞는 직원 선택으로 배치 2개, 3000의 시험 전용 work 한도로 fault gate 1개, 100.4초 이내 유한 관측으로 보호막/parry→정상 종료→fresh restore 1개가 통과했다. 제품 기본 한도나 종료 조건은 바꾸지 않았다.',
'- 남은 로그 사례는 실제 종료 후 보존 buffer만 검사한다. 관측 floor_seq=540, 780 entries, 첫 잔류 시각 12,150ms로 초기 spawn이 제거된 상태다. `UnitSpawned`를 초기/진행 events_delta에서 검사하도록 보정해야 한다.',
'- Trial 쪽은 장비 준비를 고치자 다음 Standard 모드 오류가 드러났다. R07은 시설 graph connection fixture 불일치가 드러났고, resource lifetime 18개는 자체 장비 덮어쓰기가 남았다. 따라서 공통 helper 한 줄 변경으로 138개를 해결했다고 주장하지 않는다.',
'- resolver 한도 검사는 필수 빈 loan list를 추가하면 JSON node 수도 늘어난다. old 51/50 경계를 그대로 두고 통과시키면 안 되며 current exact N/N−1을 계산해야 한다.',
'- [개별 원문·후속 실패·대체 범위](MATRIX.json), [현재 실패 원문](current138_001.stdout.log), [격리 실험 원문](diagnostic138_002.stdout.log), [추가 관측](diagnostic5_004.stderr.log), [종료 코드](RESULT.json), [source 동일성](SOURCE_RECHECK.json). 기존 coverage의 존재와 검사 범위는 소스를 확인했다. 아래 추가 재실행이 명시된 사례 외에는 이번 실행의 PASS로 승격하지 않았다.','',
'## 교정 순서','',
'1. 현행 작성된 전투 노드의 난이도·지원 activity·실제 방 상태를 함께 보존하는 공통 준비를 정리한다. 격리 실험의 slot 우선 선택은 원인 확인용이며 공식 경로를 구형 Tutorial 지도 설치로 치환하는 최종안이 아니다.',
'2. 외부 RON의 역할별 시작 세트와 resource fixture 자체의 장비 덮어쓰기를 교정한다. 각 lifetime/skill/source 계약이 유지되는 장비를 선택한다.',
'3. Trial은 현행 시작과 시설 진행을 명시적인 공개 명령으로 준비하고 binding index·none slot·receipt 한도·clock 관측을 함께 갱신한다.',
'4. 버전·명령 집합·지원 metadata·채용 session·snapshot 기대값을 현행 계약으로 맞춘다. 이전 저장 거절과 현재 정상 저장을 혼동하지 않는다.',
'5. 개별 배치·로그·전투 관측·fault 입력을 보정하고 위 두 통합/부분제거 권고를 적용한다. 보정 후 처음 도달하는 assertion에서 실패가 생기면 제품 결함 여부를 새로 판정한다. 현재 분류를 근거로 자동으로 기대값을 바꾸지 않는다.','',
'## 원인별 근거','',
'| ID | 수 | 설명 | 소스 근거 |','| --- | ---: | --- | --- |']
for c in sorted({x['cause_id'] for x in rows}):
 items=[x for x in rows if x['cause_id']==c];x=items[0]
 lines.append(f"| {c} | {len(items)} | {x['cause']} | "+'<br>'.join('`'+p+'`' for p in cause_sources[c])+' |')
lines+=['','## 기존 검증과 대체 가능성','','기존 검증을 발견했다는 사실만으로 완전 대체로 판정하지 않았다. 각 테스트의 정확한 source/후속 실패/권고는 MATRIX에 연결된다.','','| 계열 | 기존 검증 위치 | 판단 |','| --- | --- | --- |']
for k,(paths,note) in coverage.items():lines.append('| '+k+' | '+'<br>'.join('`'+p+'`' for p in paths)+' | '+note+' |')
lines+=['','## 138개 개별 판정','','격리 통과는 원본에 교정을 적용했다는 뜻이 아니다. “원인 확인”은 최초 실패의 원인을 확인했다는 뜻이며 모든 후속 assertion의 통과를 뜻하지 않는다.','','| ID | 테스트 | 원인 | 권고 | 검증 깊이 |','| --- | --- | --- | --- | --- |']
for x in rows:
 depth='격리 전체 통과' if x['evidence_level']=='isolated_full_test_pass' else ('환경 실패' if x['classification']=='environment' else '현재 실패 원인 확인')
 lines.append(f"| {x['id']} | `{x['test']}` | {x['cause_id']} | {displ[x['recommendation']]} | {depth} |")
(O/'REPORT.ko.md').write_text('\n'.join(lines)+'\n')
print(json.dumps(summary,ensure_ascii=False,indent=2))

alternatives=(O/'current_alternatives12_001.stdout.log').read_text()
alt_pass=set(re.findall(r'^test (.*?) \.\.\. ok',alternatives,re.M))
assert len(alt_pass)==12 and '12 passed; 0 failed; 0 ignored' in alternatives
summary['current_alternative_tests']={'passed':12,'failed':0,'ignored':0,'filtered':2091,'test_seconds':11.84,'log':'current_alternatives12_001.stdout.log','selection':'current_alternatives_selection.json'}
(O/'RESULT.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
for x in rows:
 if x['recommendation']=='merge_then_remove_duplicate':x['value']='중간: 핵심 장비/소유 검사는 U06와 중복. 남은 eligibility/ledger/snapshot assertion은 통합 후 보존.'
 elif x['recommendation']=='retain_normal_remove_tamper_branch':x['value']='정상 Continue 재계산은 높음. 저장 위조 전용 분기는 현행 검증 범위 밖.'
 elif x['classification']=='environment':x['value']='중간: 지원 외부 콘텐츠의 파일 종류/인코딩 오류를 구분. OS fixture 생성 가능 여부는 제품 계약과 분리.'
 elif x['purpose_family']=='api':x['value']='중간 이상: 새 명령 추가 시 route/owner 누락을 검출. 3개 층이 달라 단순 중복이 아님.'
 x['existing_coverage_execution']='관련 12개 원본 테스트 별도 재실행 PASS. 정확한 목록은 current_alternatives_selection.json; 그 밖에는 소스 존재/범위 확인만 수행.'
(O/'MATRIX.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
p=O/'REPORT.ko.md';s=p.read_text();s=s.replace('## 교정 순서','원본 코드에서 대체 후보 12개를 별도로 재실행하여 **12 PASS / 0 FAIL / 0 ignored**를 확인했다(test 11.84초). U06 역할별 세트·채용 3개, U12 codec 4개와 시작 계약 2개, U13 방/미지원 활동 2개, U14 preview 1개다. [정확한 선택](current_alternatives_selection.json)과 [실행 원문](current_alternatives12_001.stdout.log)을 보존했다. 통과해도 위에서 지적한 고유 assertion까지 자동으로 대체되지는 않는다.\n\n## 교정 순서')
s=s.replace('[종료 코드](RESULT.json)','[실행별 종료 코드](EXECUTION_INDEX.json)');p.write_text(s)
jobs=[]
for path in sorted(O.glob('*.result.json')):
 j=json.loads(path.read_text());t=(O/j['stdout']).read_text();j['test_summary']=re.findall(r'test result:.*',t);j['build_only_failure']=j['exit_code']!=0 and not j['test_summary'];j['record']=path.name;jobs.append(j)
(O/'EXECUTION_INDEX.json').write_text(json.dumps(jobs,indent=2)+'\n')
