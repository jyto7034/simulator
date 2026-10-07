#!/usr/bin/env python3
import json
from initialize import HERE,artifact,write

f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
result=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())
def ids(*ns):return [f'PE-CRRP-{n:02}' for n in ns]
allids=ids(*range(1,16))
wps=[
 {'id':'WP-CRRP-RESULT','selectors':{'implementation':ids(1,2,3,10),'supporting':ids(7,12,13,15)},'boundary':'sole combat/employee/grant/node owner→immediate settled result/choice receipt→ack/query/capability','expected_change':'TD01/02: immediate full settlement, defeat consume/no retry, pending fixed candidate/context and at-most-once claim; compact current receipt, no full runtime log persistence','prerequisite':'U03 Complete/High; exact Freeze001 and RQ01/RQ06 current source','validation':'V01,V02,V07,V12; result visible and settled before ack; ChooseOne2/0, actual employee/death/consumable effects','unknown':'exact Rust receipt/helper spelling within reviewed owner; no second grant engine'},
 {'id':'WP-CRRP-RESTORE','selectors':{'implementation':ids(4,5,6,7,8,9),'supporting':ids(1,3,10,11,13,14,15)},'boundary':'node pre-entry/battle-entry/confirmed state→stable Continue and separate single SavePoint codec/read/write','expected_change':'TD03/04: current-state payload, all unfinished-node rollback, success retreat2 overlay only for Continue, explicit load3/monotonic metadata, first-SavePoint-independent Continue','prerequisite':'RESULT receipt/choice meaning; existing RunCheckpoint capture and validators; current schema identity/RNG/resource accounting','validation':'V03–06,V10,V11,V14; every introduced state at each restore boundary; invalid identity/schema rejection','unknown':'immutable cache layout and shared validator extraction within review; no future gameplay state'},
 {'id':'WP-CRRP-DURABLE','selectors':{'implementation':ids(12,13,14),'supporting':ids(1,3,4,6,7,8,9,11,15)},'boundary':'exclusive CoreApplication prepare→actual file commit intent/revision→profile/active publish/recovery; command/tick consumed together','expected_change':'TD05–07: reuse sole transaction owner and existing journal IO; versioned conditional publish including terminal tombstone, current recovery copy, strict original protection and pre/post-intent failure separation','prerequisite':'RESULT/RESTORE durable state; actual V4/V9 and V3/profile source; no remote compatibility claim','validation':'V09–12,V15; subprocess crashes each stage, two-actor choice/stale-write/ABA, accepted recovery pending, strict unsupported and no partial grants','unknown':'concrete journal checkpoints/IO error fixture names in GEX; platform execution Linux only'},
 {'id':'WP-CRRP-HOST','selectors':{'implementation':ids(11),'supporting':ids(1,2,3,4,6,7,9,12,13,14,15)},'boundary':'stock ws/session actor→manual/tick/management→Core result/capability and durability state','expected_change':'TD05/08: all active finishing routes prepared+durable, reconnect pause distinct from Continue, stale writer gate, fault and IO recovery status, canonical DTO/document consumers','prerequisite':'Core prepared APIs and DURABLE coordinated result; existing U03 fault authority/observer','validation':'V08,V12,V15 actual public/socket transport and native fault contrasts; no runtime restart or fake commit','unknown':'host notification details within Core-owned meaning; no Unity UI implementation'},
 {'id':'WP-CRRP-VERIFY','selectors':{'implementation':ids(15),'validation_only':ids(*range(1,15))},'boundary':'integrated Core/host/file adapter; durable actual tests/probes, currentness and reversible freeze; independent audit','expected_change':'own V01–15 fixtures and execution,112 inherited +31 R06 impact, ten dimensions, exact before/after/archive restoration and independent Astra High audit/corrections','prerequisite':'AG-CRRP-RESULT-PERSISTENCE integrated current source; predecessor baseline admitted by exact byte/publication binding','validation':'all15 frozen elements and15 seams; appropriate full Core/server matrix, actual stock ws/subprocess crashes/concurrent choices; audit Complete/High before U05','unknown':'actual measured results remain GEX work; prior R05 approved warning kept, no arbitrary performance threshold'},
]
edges=[('RESULT','RESTORE','persistence_lifetime','result/choice fields must be present in current resume and rewind roots'),('RESULT','DURABLE','transaction','host can accept one already planned settlement only through the sole prepared root'),('RESTORE','DURABLE','schema_data','actual active writer and reader must agree on current/SavePoint schema and monotonic exception'),('DURABLE','HOST','producer_consumer','automatic tick/manual/management must consume revision and pre/post-intent dispositions'),('RESULT','HOST','producer_consumer','display/ack/capability must read already settled receipt'),('RESULT','VERIFY','validation','actual immediate result and choice required'),('RESTORE','VERIFY','validation','real node/checkpoint/Continue roots required'),('DURABLE','VERIFY','validation','actual file stages required for process crash fixture'),('HOST','VERIFY','validation','real websocket actor/consumer required')]
phase='PHASE-CRRP-01';ag='AG-CRRP-RESULT-PERSISTENCE'
plan=artifact('Phase Plan','PP-CRRP-001',{
 'Bound Final Policy Freeze Ref':f['ref'],'Exact Prerequisite Composite Basis':result['current_basis'],
 'Applicable P2G-40 Obligation Key Set':[],'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],
 'Goal Structure Fact':'single_phase','Parallelization':'none approved','work_packages':wps,
 'dependencies':[{'from':'WP-CRRP-'+a,'to':'WP-CRRP-'+b,'type':t,'evidence':why,'broken':'Ignoring this edge breaks '+why} for a,b,t,why in edges],
 'atomic_groups':[{'id':ag,'members':[w['id'] for w in wps[:4]],'evidence':'Minimum simultaneous result/restore/schema/file/consumer cutover: core-only loses accepted results after process death, file-only lacks authoritative receipt/rollback roots, host-only retains old deferred settlement, schema-only breaks reader/writer. No intermediate part can claim S01–08 complete; no second transaction or reward engine.'}],
 'standalone':['WP-CRRP-VERIFY'],'cyclic_sccs':[],
 'phases':[{'id':phase,'primary':allids,'supporting':[],'nodes':[ag,'WP-CRRP-VERIFY'],'exit':'Current actual results/choices/retreat/Continue/SavePoint and file restart semantics agree across Core+host; all acceptance evidence and predecessor impact; exact independent Complete/High before U05'}],
 'audits':[{'phase':phase,'state':'one immediate Core settlement with durable receipt/choice; all unfinished nodes restore same boundary; separate SavePoint load; actual adapter recovers exact accepted intent; stale writers and old formats cannot overwrite current rights',
 'checks':'single authority/commit/grant owner; matching new reader/writer; pre/post intent semantics and tombstone revision; current profile union/loads remain monotonic; U03 fault gates and native/observation contracts; no future schemas/legacy reinterpretation',
 'evidence':'V01–15,112 inherited conditions and separate31 R06,ten dimensions, public/stock ws/file crash tests and exact source restoration',
 'limitation':'local Linux actual only, no remote deployed/Windows crash claim; future U14/15/17/20/21 states and Unity own later extension'}],
 'integration':[{'id':f'IVA-CRRP-V{n:02}','source':f'CODE_REVIEW_001.ko.md V{n:02}; exact Freeze acceptance/review mapping','disposition':'assigned_execution_node','nodes':['WP-CRRP-VERIFY'],'phases':[phase],'locator':f'execution/single_goal/COVERAGE.json V{n:02}','evidence':'nonzero actual Core/public/host/file execution; exact source/data/config/toolchain/input/raw exit/expected vs actual; original negative receipts preserved','prerequisite':'AG-CRRP-RESULT-PERSISTENCE integrated current source'} for n in range(1,16)],
 'Known Unknown/Limitations':'Implementation measurements and exact helper/fixture choices are bounded GEX work; required meaning/source known. No future schemas, deployed proof, new caps/rewards/legacy migration. Original artifacts preserved.'})
write('P2G_40_PHASE_PLAN_001.json',{'artifacts':[plan],'validation_receipt_ref':{'Artifact Type':'Phase Plan Validation Receipt','Artifact ID':'PVR-CRRP-001','Content Digest':None},'validator_method_version':'p2g40-receipt-validator/1.2.0'})
write('P2G_40_DESIGN_001.ko.md','''# U04 단일 전환 단계

Bound Freeze: `FPF-CRRP-001`. Exact selectors/basis는 [Phase Plan](P2G_40_PHASE_PLAN_001.json)이 소유한다.
Repair Basis: obligations/resolutions/supersessions explicit empty. 동결 전 identifier 교정은 완료됐다.
Parallelization: none approved. 구현은 주 작업자가 순차 수행하고 독립 감사만 비참여 Astra High에 맡긴다.

| Work Package | 구현 소유 | 완료 경계 |
| --- | --- | --- |
| RESULT | 공통 즉시 정산·receipt·선택·ack | 전체 자산/직원/노드와 확정 권리가 같은 결과 |
| RESTORE | pre-entry/전투전/현재/SavePoint와 codec | 현재 모든 노드·복원 경계와 새 reader/writer |
| DURABLE | prepared publication·file journal/revision | 접수 전 rollback, 접수 후 같은 결과 복구, stale write 보호 |
| HOST | 실제 명령/tick/관리/재접속·DTO | Core 결과와 영속 상태를 실제 socket 소비 |
| VERIFY | V01–15·선행 영향·실제 crash와 감사 target | exact 자체 검증/복원과 독립 감사 |

앞 네 package는 `AG-CRRP-RESULT-PERSISTENCE` 하나의 최소 전환 단위다. 구현 내부 순서는
RESULT→RESTORE→DURABLE→HOST이고 새 public DTO/reader/writer를 같은 current target에 연결한다.
Core 정산만 완료하면 프로세스 종료에 결과가 사라지고 저장만 바꾸면 기존 ack 정산/노드 복구와 불일치한다.
부분 구현은 중간 작업 상태이며 독립 완료·release라고 게시하지 않는다. 공유 계약은 CODE_REVIEW TD01–08이다.

`PHASE-CRRP-01`이15요소의 유일 primary owner이며 atomic group과 VERIFY 두 final node를 포함한다.
단계 직후에는 즉시 결과·고정 선택·후퇴2·미확정 노드 복원·stable Continue·load3·실제 파일 원자 복구가
함께 유효해야 한다. 선행 U03 fault/관측·native 한도와 원본 보호도 유지한다. 새 schema는 현재 상태에 한정한다.
15개 integration acceptance의 필요한 durable test/probe는 VERIFY가 구현하며 GEX-30이나 감사자에게 미루지 않는다.
단일 단계 구조이므로 P2G-50은 Single Goal을 작성한다. 표준 receipt는 구조만 검증하며 의미적 중간 상태 감사는
이 문서·Phase Plan audits와 코드 검토가 소유한다. 독립 감사 통과 전 U05 종속 구현을 시작하지 않는다.
''')
print('U04 P2G40 candidate:5 WPs,4-member atomic group,2 final nodes,1 Phase,15 assigned seams; standard validation pending')
