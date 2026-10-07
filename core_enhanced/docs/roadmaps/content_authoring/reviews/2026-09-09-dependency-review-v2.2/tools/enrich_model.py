from pathlib import Path
import json
ROOT=Path(__file__).resolve().parents[1]
P=ROOT/'roadmap_v2.2-review/GOALS.json'
M=json.loads(P.read_text()); U={u['id']:u for u in M['units']}
M['schema']='content-authoring-roadmap-review/1'
M['format_note']='기존 roadmap GOALS.json의 CA 배열과 다른 versioned 연구 모델이다. 새 별도 경로에서만 사용한다. 기존 소비 도구/정식 P2G/GEX 입력과의 호환성은 인증하지 않았으며 원본 파일을 교체하지 않는다.'
M['source_ledger']='../evidence/source_ledger.json'
M['current_formal_basis']={'working_policy':'WPD-CAMEC-20260909','working_revision':'WR-CAMEC-P2G20-001','freeze':'FPF-CAMEC-P2G30-001','phase_plan':'PP-CAMEC-P2G40-002','goal_set':'GS-CAMEC-P2G50-001','goal':'GOAL-CAMEC-01','handoff':'HOF-CAMEC-P2G60-001','status':'Authored / Not Activated / Implementation Authorized: No','status_document':'v1.9 / record 0010 / NEXT-0009','qualification':'原本에서 읽은 현재 선택. 이번 연구는 formal current/stale 또는 GEX eligibility를 발행하지 않는다.'}
M['current_formal_basis']['qualification']=M['current_formal_basis']['qualification'].replace('原本','원본')
extra={'R00':'S065 S066 S078 S088 S089 S092 S093','R01':'S065 S066 S070 S079 S091 S092 S093 S094 S095 S097','R02':'S068 S069 S071 S085 S091 S094','R03':'S068 S069 S070 S071 S072 S085','R04':'S064 S070 S071 S072 S082','R05':'S082 S091','R06':'S061 S062 S063 S064 S091 S095','R07':'S064 S075 S091','R08':'S084 S094','R09':'S059 S060 S061 S075 S077 S091','R10':'S070 S079 S081 S082 S097','R11':'S063 S064 S076 S082 S087','R12':'S076 S082 S087','R13':'S076 S082 S087','R14':'S062 S063 S064 S076 S083 S091','R15':'S076 S081 S083 S087','R16':'S071 S072 S082 S083','R17':'S076 S082 S087','R18':'S084 S094 S096','R19':'S062 S063 S073 S074 S091 S094','R20':'S077 S091 S094','R21':'S075 S076 S078 S079 S087 S096','R22':'S084 S085','R23':'S071 S072 S082 S085','R24':'S088 S089'}
for id,u in U.items():
 u['sources']=sorted(set(u['sources']+extra.get(id,'').split()))
 u['execution_state']='inherited_investigation_and_formal_authoring' if id=='R00' else 'formal_authored_not_activated' if id=='R01' else 'proposed_not_implemented'
 u['research_status']='정적 변경 경계 검토; source_ledger의 범위와 LIMITATIONS 적용'
 u['actual_runtime_receipt']=None
 u['implementation_authorized']=False
 u['stage']='M 재분할; CA 대응표 참조'
 u['verification_dependencies']=[]
 u['card']=f'goals/{id}.ko.md'
 u['order_index']=int(id[1:])
U['R07']['atomic_changes']+=' intrinsic/passive+active 조합의 slot·공명·등록을 같은 profile binding에서 구별하며, 고급 channel/override 기믹은 R11이 확장한다.'
U['R13']['valid_after']='아군/적 actor와 정지 함정·벽이 각 기능에 맞게 생성된다. 작은→큰 진화는 논리적 형태 전환이다. node ID 유지/교체는 기술 설계에서 선택하되 source 권한과 stale link/event 무효화를 함께 보장한다.'
U['R14']['atomic_changes']+=' 관리 aggregate와 Composer의 책임은 분리한다. 유물 판매 금지, 직원 내 중복/비호환, 무료 장착·이전, 기존 장비 가루로 분쇄를 보존한다.'
U['R14']['extra']+=' 전역 definition catalog는 계속 원형 설명을 제공한다. 개인별 effective preview/설명은 binding context를 받는 별도 projection이며 전역 catalog를 개인 plan으로 덮지 않는다.'
U['R18']['extra']=U['R18']['extra'].replace('R13/R12/R11','R13/R11').replace('실제로 소비하는 R13/R11 산출물만 기능상 필수다.','실제로 소비하는 actor adapter와 profile/lease 산출물만 기능상 필수다. 공유 보호막을 쓰는 보스만 R12의 조건부 소비자다.')
U['R09']['extra']+=' 룰북의 Pause 중 일반 cast 시작/자원 소비 즉시 반영은 유지한다. 침식 판정은 별도 실제 cast-start 결정 지점을 설계해야 하며 단순히 focus 종료/release와 같다고 확정하지 않는다. 해당 시점·비용 의미가 충돌하면 R09의 P2G-10 정책 owner에 반환한다; 현재 최소 Goal의 선행은 아니다.'
# Typed predecessor relations. Verification evidence is not mislabeled as implementation dependency.
VPAIRS={('R00','R01'),('R09','R20'),('R17','R20'),('R18','R20'),('R19','R20'),('R15','R21'),('R16','R21'),('R17','R21'),('R18','R21'),('R20','R21'),('R21','R22'),('R22','R23'),('R23','R24')}
# Per-producer output names; the target is still the concrete consumer below, not the entire future unit.
OUTPUTS={'R00':'승인/미정이 분리된 요구·정식 최소 Goal 기준선','R01':'검증된 최신 최소 schema·effect 주소·descriptor·SourceMap','R02':'외부 source selection·검증된 immutable generation과 fingerprint','R03':'공개 로딩/API/CLI·readonly 설명·headless 실행 포트','R04':'typed facts/receipts·현재 전달체 ActionHandle·source-copy binding 포트','R05':'owner×lifetime resource/lease·원자적 소비/복원 포트','R06':'typed value/result/resource 식·집계·스탯 입력','R07':'실제 파편 사본/형태·장착 binding·정상 지급/저장','R08':'BossTask→Action request/result/cancel/방향 adapter','R09':'침식 suppression·강제 시계·직원 대가/Breakdown의 단독 검증','R10':'실행 가능한 typed Flow/target/carrier와 immutable plan 참조','R11':'source별 reversible profile/lease·membership·종료 포트','R12':'공용 pool/HP비용·방어 중재의 필요한 최소 포트','R13':'actor/object identity·원자적 생성·진화/삭제 수명','R14':'유물 사본 binding·실제 numeric effective plan/설명/저장','R15':'유한 grant/활성 closure·typed structural recipe 증거','R16':'branch-local ancestry·지연 전달·suppression dispatcher','R17':'관계/원천별 감염·집계·consume/cleanup 증거','R18':'보스 형태·신호·다중 actor의 통합 증거','R19':'정상 등장/획득 선택/평가/교체의 통합 증거','R20':'복합 결과·후퇴·저장·재접속의 현재 revision 증거','R21':'대표 공식 실제 RON·전체 baseline disposition·지원 문서','R22':'엔진 동결 후 독립 작성/실행 및 data-only 증거','R23':'현재 revision의 결정론·fault·성능·회귀 증거'}
CONSUMERS={
'R01':'현재 SkillDatabase 검증→ValidatedGameData→sim/build_skill_step_commands와 runtime 검사',
'R02':'live_ron source 확보→data/mod publication→CoreApplication/new-run briefing→checkpoint restore',
'R03':'Describe/Validate/Explain/Simulate와 Cargo 명시 binary/외부 RON 실행',
'R04':'damage_runtime/commands/death_runtime/triggers 및 skill_runtime의 결과·수명 포트',
'R05':'resource owner key·cost request→attempt/entry/checkpoint/result commit',
'R06':'stat_pipeline/typed evaluator의 live/snapshot/resource/result 입력',
'R07':'SkillFragmentInventory/Loadout→Maintenance/grant→player_spawns/profile→save/표시',
'R08':'BossTaskData/compiled task→공통 내부 action admission→state activation result',
'R09':'manual/ordinary cast 접수와 실제 실행 연결→withdraw/kill→attempt roster→node/save/clock',
'R10':'sim/skill_runtime의 continuation/result slot·반복·전달 분리',
'R11':'buff/stat/basic profile와 activation/membership/lease 해제',
'R12':'incoming damage→pool/HP/survival atomic commit 및 수혜자 membership',
'R13':'SpawnKernel adapter→Trap/Actor/FieldObject 생성·HP비용·진화·source별 수명',
'R14':'loadout Composer→cast/queued step/projectile/area/개인 preview·평가 입력',
'R15':'typed selector/closure/recipe→기존 지원 primitive의 변형된 plan 재검증',
'R16':'trigger dispatcher→proc/extra attack→projectile/area/timer→후속 reaction',
'R17':'object/status source membership→relation/GroupByTile→detonation/consume',
'R18':'boss state profile/selector→field-work signal→multi-actor/wave 관찰',
'R19':'pve reward-source/encounter plan·commit→grant/save→Certified 모델',
'R20':'복합 run/battle entry/attempt 결과와 restore/reconnect 검증',
'R21':'공개 최신 RON/production loader를 통한 C01–C07 및 전체 COV 인수',
'R22':'별도 작성자의 frozen-engine 외부 pack 제작·검증·실행',
'R23':'독립 제작 workload를 포함한 fault/결정론/성능 통합 시험',
'R24':'독립 감사자의 coverage/currentness/실제 결과 판정'}
NEW=[]
def N(a,b,kind,reason,condition='',consumer=None):
 src=sorted(set(U[a]['sources']+U[b]['sources']))
 row={'id':f'REL-{len(NEW)+1:03}','from':a,'to':b,'type':kind,'producer_output':OUTPUTS.get(a,U[a]['name']),'consumer':consumer or CONSUMERS.get(b,U[b]['atomic_changes']),'reason':reason,'condition':condition,'sources':src,'evidence_kind':'관찰 근거를 명시한 목표 전환/검증 설계; 이번 게임 실행 아님'}
 NEW.append(row)
for id,u in U.items():
 deps=list(u.get('dependencies',u['implementation_dependencies'])); u['implementation_dependencies']=[]
 for p in deps:
  typ='검증선행' if (p,id) in VPAIRS else '기능상필수'
  u['verification_dependencies' if typ=='검증선행' else 'implementation_dependencies'].append(p)
  N(p,id,typ,'후행이 실제 소비하는 산출은 producer_output에 한정한다. 후행 단위의 직후 유효 상태와 owner 시험에 필요한 입력이다.' if typ=='기능상필수' else '후행의 완료 주장을 판정할 실제 선행 증거이며 후행 kernel의 코드 호출 선행과 구별한다.')
# Precise nontrivial edge contracts, not generic whole-unit prerequisites.
DETAILS={('R04','R05'):('자원 transaction에 필요한 것은 CostReceipt·source owner identity다. 모든 ActionHandle waiter 구현을 전제로 하지 않는다.','resource request owner/cost receipt→ResourceStore'),('R04','R08'):('기존 Instant/Projectile/Area의 ActionHandle과 실패/취소 포트만 필요하다. full Flow·shield·설치물을 기다리지 않는다.','BossTask의 Accepted/Released/EffectsSettled waiter'),('R07','R09'):('실제로 선택된 파편의 definition/origin/form binding이 최종 보스 파편 제외와 cast 대체의 입력이다.','SkillFragmentLoadout→실제 시전 침식 적격 판정'),('R12','R13'):('R13에 포함한 meatwall의 HP비용/생성 원자성만 선행이다. 비용 없는 actor/trap은 독립이다.','cost reserve/commit/abort→SpawnKernel 생성 transaction'),('R07','R13'):('파편별 trap 소유·진화에 copy binding이 필요하다. 기존 보스 actor에는 필수 아님.','Laetitia 원천 사본→small/big 진화 자격'),('R10','R14'):('선택한 기본안에서 current/structured Flow 모두의 개인별 불변 plan을 소비한다. 관리 aggregate 자체는 독립이다.','cast/queued step/impact consumer의 effective plan 참조'),('R11','R12'):('공유 보호막의 수혜자 add/remove와 source 해제 포트만 필요하다. Hello·재입력 전체는 선행이 아니다.','pool membership와 source activation lease'),('R15','R16'):('필요한 것은 생성 effect의 안정 주소·유한 closure다. 모든 shield/channel recipe 완료를 요구하지 않는다.','reaction definition과 (copy,effect) 재방문 key'),('R13','R18'):('필요한 것은 사용하는 actor의 생성/수명 adapter다. 모든 trap/wall recipe가 보스 기본 호출의 선행은 아니다.','multi-actor binding·소환 batch 수명'),('R11','R18'):('보스 형태에 쓰는 reversible profile/lease만 필요하다. 전체 channel 기믹을 기다리지 않는다.','state profile overlay와 state exit cleanup'),('R03','R21'):('최종 공식 인수는 private builder로 대신하지 못하므로 공개 source→실행 포트가 필요하다.','공식 pack 로딩·실행·receipt')}
for e in NEW:
 if (e['from'],e['to']) in DETAILS:e['reason'],e['consumer']=DETAILS[(e['from'],e['to'])]
for a,b,why in [('R03','R04','초기 공개 RON 관찰 경로를 준비해 재작업을 줄인다. 기존 skill_runtime_contract 등의 kernel seam으로 기능 개발은 가능하다.'),('R03','R05','ResourceStore 자체의 코드 선행이 아니라 외부 제작 인수의 편의·일관성이다.'),('R06','R07','profile 파일 충돌을 줄이려고 성장 입력 뒤에 사본 전환한다. 사본 저장/선택 자체는 Expr 전체와 독립이다.'),('R07','R08','보수적 순차 기본안이다. R08의 실제 필수는 R01/R04뿐이다.'),('R09','R10','침식 suppression/clock 경계를 먼저 닫아 후속 기능 추가가 이를 회귀 인수하게 한다. 미정이면 지정 분기만 유예한다.'),('R13','R14','공용 types/skill runtime/owner-state 변경을 순차 통합해 개인 plan 전환 충돌을 줄인다.')]:N(a,b,'진행상권장',why)
N('R12','R18','조건부','공유 보호막을 직접 사용하는 보스에 한해 pool 계약이 필요하다. 거리 무효화 predicate와 R08 기본 호출은 전체 shield의 선행을 요구하지 않는다.','해당 boss가 shared shield capability를 채택한 경우')
N('R09','R16','조건부','R09가 먼저 구현됐다면 새 모든 subscriber가 침식 suppression 회귀를 인수한다. R09가 유예되면 나중 R09가 새 subscriber를 인수한다.','침식 또는 새 chain의 실제 도입 순서에 따른 재검증 owner')
N('R23','R22','검증선행','최적화가 의미·지원 범위·API를 바꾼 경우 영향받은 독립 제작 증거를 다시 획득한다. 구현 순환/항상 실행 선행이 아니다.','변경 영향이 있는 경우에만 재검증')
NEW[-1]['conditional_revalidation']=True
# Atomic groups are intra-unit components, not cyclic entire-CA arrows.
AT=[]
def A(id,owner,boundary,why,files,src):
 AT.append({'id':id,'owner':owner,'type':'동시전환','components':boundary,'reason':why,'paths_and_symbols':files,'sources':src.split(),'commit_policy':'최소 공개 유효 상태의 동시 전환. git commit 수나 이 단위의 모든 feature가 한 transaction이라는 뜻이 아니다.'})
A('AT01','R01','typed metadata/compiler + latest RON/catalog + 직접 runtime 소비자','구형/신형 의미가 섞인 publication 또는 빈 metadata만의 성공을 막는다. 기존 AG-CAMEC-01을 그대로 재사용한다.','ability.rs; skill_data.rs; live_ron.rs; sim.rs; skill_runtime; game_resources/data/skills; P2G_40_PHASE_PLAN_002.json','S010 S013 S016 S017 S018 S019 S020 S021 S079')
A('AT02','R02','외부 snapshot/resolve/compile/publication + new-run 데이터 선택 + fingerprint save/restore','GameCore state-only commit으로 세대 선택을 잃거나 다른 내용의 동일 ID 저장을 복원하지 않게 한다.','application.rs; data/{live_ron,mod}; world.rs; world/{application,active_run,state,node_flow}','S012 S013 S014 S015 S037 S038 S039 S040')
A('AT03','R04','Damage/Heal/Death/Cost receipt 생성 + 실제 mutation + 기존 optional trigger 소비','HP 사실/처치 사실과 후속 반응의 의미를 동시에 바꿔야 한다. mandatory wave는 유지한다.','damage_runtime; commands; death_runtime; triggers; wave_runtime; recording/event_log','S023 S024 S025 S026 S053 S071 S072')
A('AT04','R04','persistent equipment copy → draft → RuntimeItem source binding','base UUID/배치 인덱스로 영구 소유 사본을 추정할 수 없다.','resources/inventory; combat_setup/player_spawns; battle/core/build; core/types','S034 S064 S022 S027')
A('AT05','R05','새 Run 자원 writer + overlay/result/entry + checkpoint reader/validator','다음 저장 단계가 없어도 새 상태가 유효하게 왕복해야 한다.','world/{state,active_run,application,combat,node_flow}; 새 resource store','S037 S038 S039 S040 S044')
A('AT06','R06','Attack/SpellPower·무기 성장 입력 + profile/stat 계산 + 해당 projection/save 검증','기존 무기 기여 이중 합산 또는 실제 계산과 표시 입력의 분리를 방지한다. 숫자 평가 가중 결정과는 별개다.','stats; stat_pipeline; inventory; player_spawns; range_preview; combat_power; snapshot','S030 S034 S043 S062 S063 S064 S091')
A('AT07','R07','파편 copy aggregate + grant/equip/select/awake/dismantle/recovery + profile + snapshot/checkpoint','종류 장부와 새 사본 장부를 이중 truth로 남기거나 실제 장착/저장에서 사본성을 잃지 않게 한다.','skill_fragment; skill_fragment_data; maintenance; helpers; grant_state; reward; player_spawns; snapshot/state/active_run','S031 S032 S035 S036 S038 S039 S041 S042 S064 S091')
A('AT08','R08','BossTask raw/compile + 권한 있는 Action admission + result/cancel/facing 소비','enum 존재·Accepted 응답만으로 실행/완료를 가장하지 않는다.','boss_runtime_data; boss_runtime; sim; skill_runtime/cast; build','S047 S048 S018 S019 S022')
A('AT09','R09','침식 시전 대체/kill/suppression + Withdrawn/attempt trauma + 강제 clock + 결과·Breakdown·복원','마지막 적 처치로 전투가 즉시 끝나도 대가를 잃지 않으며, optional trigger만 차단하고 진행은 유지한다.','sim; commands; death_runtime; build; world/combat; employee; employee_psychology; node_flow; run_policy_data; active_run/snapshot','S018 S024 S025 S022 S044 S059 S060 S040 S061 S039 S091 S046 S077')
A('AT10','R10','새 Flow AST/compile + continuation/result/각 carrier 소비 + actual fixture','새 결과 slot/완료 milestone을 정의만 하고 구형 step executor에 방치하지 않는다.','ability; skill_data; sim; skill_runtime/{cast,projectile,area}; core/types','S016 S017 S018 S019 S020 S021 S027')
A('AT11','R11','source별 modifier/lease 설치 + 모든 종료/복귀 경로 + effective profile','반대 숫자 덧셈이나 오래된 원형 snapshot으로 다른 source 변경까지 덮어 복귀하지 않게 한다.','buffs; stat_pipeline; build; sim; scope별 state','S029 S030 S022 S018 S082')
A('AT12','R12','pool debit + actual HP/survival commit + receipts + membership cleanup','한 번의 피해가 여러 수혜자의 복제 잔량에 적용되거나 pool만 차감된 실패를 막는다.','damage_runtime; damage; commands; resource/membership store','S023 S028 S024 S082')
A('AT13','R13','해당 생성 request의 비용·배치·UUID·생성 + 진화/삭제 시 참조 무효화','생성 실패에 비용만 소모하거나 구형 node timer가 새 node에 적용되지 않게 한다.','spawn_kernel; summon_runtime; object store/typed adapters','S049 S050 S051 S087')
A('AT14','R14','numeric effective plan/compiler + cast/queued/impact source binding + 개인 preview/평가 입력','지연 적중이 전역 SkillId 원형을 다시 읽어 유물 변형을 잃지 않게 한다. 유물 관리-only는 독립 단위로 분리 가능하다.','sim; skill_runtime/{cast,projectile,area}; range_preview; player_spawns; snapshot; stat_pipeline','S018 S019 S020 S021 S063 S064 S091 S030')
A('AT15','R16','reaction admission/visited 검사 + 모든 지연 payload ancestry + root producer','timer나 새 projectile이 ancestry를 세탁하지 못하게 한다.','triggers; sim; types; projectile; area; recording; separate ChainContext','S026 S018 S027 S020 S021 S071 S072')
A('AT16','R19','복수 파편/선택 schema + 보상 plan/commit + 참조/중복 검증 + 정상 grant/save','현 단일 source.fragment_id 가정 중 실제 변경한 부분은 모든 reader/writer에 같은 의미여야 한다.','pve_data; encounter_progression; skill_fragment_data; reward; validation; world snapshot/save','S073 S074 S042 S036 S067 S038 S091')
M['new_relationships']=NEW;M['atomic_groups']=AT
for u in U.values():
 u['atomic_groups']=[a['id'] for a in AT if a['owner']==u['id']]
 u['recommended_after']=[e['from'] for e in NEW if e['to']==u['id'] and e['type']=='진행상권장']
 u['dependencies']=u['implementation_dependencies']+u['verification_dependencies']
 u['relationship_ids']=[e['id'] for e in NEW if e['to']==u['id']]
# Exact CA IDs remain the coverage backbone, never newly activated formal goals.
ca_names=['기준 상태·전체 요구사항·정책 추적표','작성 단위·타입·실행 지원 계약','최신 schema 전환·외부 다중 팩 게시','공통 검증·설명·headless 시험 실행 포트','자원·상태 소유와 수명 kernel','계산식·집계·스탯 성장','의미 있는 전투 사건·피해 해결 결과','대상·전달·단계·분기·반복 실행','가역 modifier·오라·집중·기본 공격 교체','공유 보호막·생존·방어 중재·제어','파편 사본·개화·형태와 무기 성장 연결','유물 소유·장착·회수·분쇄','효과 선택·변형·유효 효과 구성','연쇄 실행·경로별 재방문 차단','소환·설치물·전장 객체','관계·감염·공간 집계','보스 제작·실행 확장','등록·획득·평가 연결','침식','저장·결과·수명 통합','대표 공식 RON 및 표현력 검증','독립 제작 인수','안전·결정론·성능','최종 감사·인계']
ca_desc=[
'CA00 요구/COV 조사 성과를 인수한다. 새 정책 결정·전수 게임 실행 증거로 승격하지 않는다.',
'최소 formal Goal은 유지. 확장된 handle/Flow/state 의미는 후속 R로 분리한다.',
'최소 schema와 catalog의 embedded 전환은 R01, 외부 입력/세대/저장은 R02가 소유한다.',
'공개 제작 포트는 먼저 권장하되 kernel 기능 선행으로 과장하지 않는다. 각 새 opcode의 descriptor/negative test는 해당 owner가 동반한다.',
'현재 owner/source 일부를 R04에서 정하고 자원/store/lifetime은 R05로 분리한다.',
'R06에서 계산·집계·무기 성장. 파편 일반 rank 제거는 R07. 전투력 최종 모델은 R19.',
'R04에서 사건/결과/출처를 앞으로 이동하고 기존 basic trigger 의미·죽음 진행 처리를 보존한다.',
'현재 전달체의 handle은 R04, 일반 target/carrier/Flow는 R10. 보스 최소 adapter는 R08.',
'R11: source별 ledger/lease/membership·기본 공격 override·재입력/종료. 현재 직접 modifier 변경을 가역이라고 부르지 않는다.',
'R12: pool/HP/제어/생존 중재. R08 보스 호출과 R09 침식의 전역 선행으로 두지 않는다.',
'R07 사본화와 모든 장착/지급/저장; 무기 성장 입력은 R06. 유물 aggregate와 필수 합병하지 않는다.',
'관리 도메인은 독립성을 유지한다. 기본안은 R14에서 최소 수치 유물 실행까지 같은 작업 단위로 인수한다.',
'R14 numeric 실행 연결→R15 grant/구조 합성→R17 관계 특화 recipe. 선택 효과와 carrier를 구별한다.',
'R16: 즉시·지연 모두 branch-local chain. R09 suppression은 R16의 선행이 아니라 도입 순서에 따른 회귀 인수.',
'R13: 기존 SpawnKernel 재사용, boss/opponent 상위 결속 해제. actor/trap/fieldobject의 capability를 분리한다.',
'R17: source별 관계/감염/타일 집계. 정밀 line-crossing 알고리즘은 이번 source 상세 미검토로 남긴다.',
'R08에서 최소 호출을 조기 분리하고 R18에서 형태·signal·selector·다중 actor를 확장한다.',
'R07/R14가 최초 정상 획득/장착, R06이 평가 입력, R19가 복수 선택/등록/평가 모델 통합을 닫는다.',
'R09으로 조기 이동. 현재 적 kill·optional 억제·직원 대가·강제 clock·Breakdown을 함께 소유한다.',
'R02/R05/R07/R09/R14 및 새 persistent-state owner에 저장을 분산하고 R20은 복합 검증이다.',
'R21에서 C01–C07 실제 최신 RON과 전체 COV 표현력 disposition을 별도 인수한다.',
'R22는 독립 작성자와 frozen-engine data-only 증거가 필요하다. 이번 연구에서 수행하지 않았다.',
'각 기능 도입부터 예산/fault/결정론 관찰, R23에서 복합 측정/회귀를 닫는다.',
'R24의 독립 감사는 미래 작업이다. 이번 문서 정합성 검사는 대체물이 아니다.']
M['ca_mapping']=[]
for i,name in enumerate(ca_names):
 ca=f'CA{i:02}';owners=[u['id'] for u in U.values() if ca in u['old_ca']]
 M['ca_mapping'].append({'id':ca,'label':name,'label_note':'CA00–CA13은 원문 제목; 이후는 식별용 요약 제목이며 원문 의미는 S054를 참조','units':owners,'disposition':ca_desc[i],'current_status':'기존 조사/정식 설계 성과 인수, 게임 미구현' if i<2 else '변경 경계 정적 조사; 향후 구현·게임 검증 미실행','review_limits':'소스 ledger의 읽은 범위. 미정 정책·전체 COV 행/geometry/성능 검증은 LIMITATIONS 참조','sources':sorted(set(s for u in U.values() if ca in u['old_ca'] for s in u['sources']))})
# Shared work explicitly maps CA04/06/07/22 obligations even when no new monolithic CA remains.
CAADD={'CA04':['R04'],'CA06':['R09','R12','R16'],'CA07':['R08'],'CA08':['R18'],'CA09':['R13'],'CA12':['R17'],'CA13':['R04'],'CA14':['R18'],'CA16':['R19'],'CA17':['R03'],'CA19':['R04','R06','R10','R11','R12','R13','R16','R17','R18','R19'],'CA22':['R03','R06','R08','R09','R10','R11','R12','R13','R14','R15','R16','R17','R18']}
# Use a separate supporting-owner field to avoid conflating trace support with total CA implementation.
for c in M['ca_mapping']:c['supporting_owners']=CAADD.get(c['id'],[])
MS=[range(0,2),range(2,4),range(4,8),range(8,11),range(11,14),range(14,17),range(17,20),range(20,22),range(22,24)]
M['milestone_mapping']=[{'id':f'M{i}','old_ca':[f'CA{x:02}' for x in ca],'units':sorted({r for c in M['ca_mapping'] if int(c['id'][2:]) in ca for r in c['units']}),'status':'기존 완료는 R00 인수, 새 순서는 분해/재배치 권고. M 자체의 구현 완료를 선언하지 않음'} for i,ca in enumerate(MS)]
P.write_text(json.dumps(M,ensure_ascii=False,indent=2)+'\n')
print('units',len(U),'old edges',len(M['old_dependency_edges']),'new typed relations',len(NEW),'atomic groups',len(AT))
