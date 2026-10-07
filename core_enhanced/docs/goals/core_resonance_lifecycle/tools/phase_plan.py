#!/usr/bin/env python3
import json
from initialize import HERE,artifact,write
f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0];result=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())
def ids(*n):return [f'PE-CRSL-{x:02}' for x in n]
allids=ids(*range(1,15))
data=[
('DEFINITION',ids(1,2,12),ids(3,5,8,10,13),'actual fragment/profile + raw/compiled skill + live25 and typed RON fixtures','TD01/02/05: employee base/fragment max and explicit use descriptor; remove employee/Rest old fields; preserve valid deployment effect and opponent cadence','Current150 source and frozen employee/autonomous scope; U01/U04 audit closed','V01/02/12 strict loader and current all-skill applicability','exact type/field spelling and public descriptor compilation within TD01–11'),
('ADMISSION',ids(4,7,8,9,10,11),ids(1,2,3,6,12,13),'public command→latest validate→single consume/credential→pending start/corrosion→target anchor/cancel','TD03/04/08: consumed-once action credential, rejection unchanged, empty tile, direct target invalidity, paused actual timing','DEFINITION resource/skill identity; sole CoreApplication candidate and existing action source','V04/07–11 latest public input, Pause/resume, target/preview, corrosion timing/cost','exact credential handle private representation; no new input execution timing'),
('LIFETIME',ids(3,5,6),ids(2,4,7,11,12,13),'use owner→cast/legacy and structured flow/finite buff/channel/creation end→gain gate and battle-second pulse','TD02/05–07: separate control/use/effect lifetime, finite leases and early ends; future-control cancellation with released payload retained','DEFINITION descriptor and ADMISSION accepted-use identity; R04/R05 action/source and U01 cancel boundaries','V02–07/11/12 mixed timed effects, slow projectile, combo/repeat, independent summon/DoT, CC/silence, no bank','exact pulse scheduling/order and bounded tracking implementation in GEX; no inferred independent-child lock'),
('PROJECTION',ids(13),ids(1,2,3,4,5,6,7,8,9,10,11,12),'authoritative readiness/gain reasons→Core query/host wire/reconnect and private rollback/confirmed Continue','TD09: source-derived DTO/canonical with typed overlap reasons, controlled clone and retained extent; U04 pre-entry/savepoint unchanged','ADMISSION/LIFETIME current kernel facts and DEFINITION strict schema','V04/09/12/13 public host normal command/query, same actor Pause reconnect, rollback/Continue/old originals','exact additive DTO fields and any genuinely affected save codec; no new live-battle disk snapshot'),
('VERIFY',ids(14),ids(*range(1,14)),'integrated Core/host/current live definitions→durable semantic fixtures,127 inherited+31 R06 and independent audit','V01–14 executable completion matrix, before/after rollback/source archive, applicable full suite and currentness','AG-CRSL-RESONANCE integrated current source; exact U04 baseline reused only where unaffected','all14 acceptance sets,10dimensions,live25/public typed fixture/runtime/stock host paths; independent CompleteHigh before U06','measured results pending; existing R05 exception/20-minute exclusion preserved without enlarging')]
wps=[{'id':'WP-CRSL-'+k,'selectors':{'implementation':im,'supporting':su} if k!='VERIFY' else {'implementation':im,'validation_only':su},'boundary':b,'expected_change':c,'prerequisite':p,'validation':v,'unknown':u} for k,im,su,b,c,p,v,u in data]
edges=[('DEFINITION','ADMISSION','schema_data','resource and use descriptor must match validated accepted input'),('DEFINITION','LIFETIME','schema_data','runtime end owner must execute actual compiled lifetime contract'),('ADMISSION','LIFETIME','persistence_lifetime','consume and lock require the same use identity'),('ADMISSION','PROJECTION','producer_consumer','readiness and pending projection must consume admitted facts'),('LIFETIME','PROJECTION','producer_consumer','gain reasons require actual overlapping use/CC owners')]+[(k,'VERIFY','validation','integrated '+k+' must exist for durable acceptance execution') for k,*_ in data[:-1]]
phase='PHASE-CRSL-01';ag='AG-CRSL-RESONANCE'
p=artifact('Phase Plan','PP-CRSL-001',{'Bound Final Policy Freeze Ref':f['ref'],'Exact Prerequisite Composite Basis':result['current_basis'],'Applicable P2G-40 Obligation Key Set':[],'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],
 'Goal Structure Fact':'single_phase','Parallelization':'none approved','work_packages':wps,
 'dependencies':[{'from':'WP-CRSL-'+a,'to':'WP-CRSL-'+b,'type':t,'evidence':why,'broken':'Ignoring this edge breaks '+why} for a,b,t,why in edges],
 'atomic_groups':[{'id':ag,'members':[w['id'] for w in wps[:-1]],'evidence':'Minimal simultaneous resource/schema/admission/start/end/gain/query transition: consumption alone rejects pending at full gate; end alone double resets or unlocks early; schema alone fails actual current skills; projection alone reports unsupported semantics. All share current employee use identity and strict producer/consumer cutover.'}],
 'standalone':['WP-CRSL-VERIFY'],'cyclic_sccs':[],
 'phases':[{'id':phase,'primary':allids,'supporting':[],'nodes':[ag,'WP-CRSL-VERIFY'],'exit':'All current skill admission/use-end/gain and employee charge rules valid across loader/Core/host/restore;14 acceptance sets and127 inherited/31 R06 impact proven; independent CompleteHigh before U06'}],
 'audits':[{'phase':phase,'state':'one accepted-use resource owner; explicit compatible raw/compiled data; distinct action/effect/use lifetimes and accepted target anchor; current corrosion start timing; projection and same actor/Continue match','checks':'no full-after-consume recheck, reset/refund/reuse, unconditional child lifetime lock, all positive gain bypass, employee old field fallback or silent enemy cadence change; supported reader/writer/rollback agree','evidence':'V01–14+10 dimensions+127 inherited/31 R06,actual public fixtures/stock host and exact archive restoration','limitation':'workspace/local Linux only; Unity and remote deployment excluded; U19 new damage and future actors not preimplemented'}],
 'integration':[{'id':f'IVA-CRSL-V{n:02}','source':f'CODE_REVIEW_002.ko.md Element Coverage {n:02}; exact Freeze member acceptance','disposition':'assigned_execution_node','nodes':['WP-CRSL-VERIFY'],'phases':[phase],'locator':f'execution/single_goal/COVERAGE.json V{n:02}','evidence':'actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts','prerequisite':ag+' integrated source'} for n in range(1,15)],
 'Known Unknown/Limitations':'Exact bounded tracking/schema/probe design is GEX-owned; no required policy/source gap or implicit future gameplay. Prior R05 exception remains source-bound.'})
write('P2G_40_PHASE_PLAN_001.json',{'artifacts':[p],'validation_receipt_ref':{'Artifact Type':'Phase Plan Validation Receipt','Artifact ID':'PVR-CRSL-001','Content Digest':None},'validator_method_version':'p2g40-receipt-validator/1.2.0'})
write('P2G_40_DESIGN_001.ko.md','''# U05 단일 공명 전환 단계

Bound Freeze: FPF-CRSL-001. Exact selectors와 prerequisite는 Phase Plan001 소유다.
Repair Basis: obligations/resolutions/supersessions explicit empty. Parallelization: none approved.

DEFINITION→ADMISSION→LIFETIME→PROJECTION은 동일 사용 자격·수명/작성 계약을 함께 전환하는
AG-CRSL-RESONANCE다. 소비만 이동하면 start gate가 잔량0을 거절하고 종료만 바꾸면 재소비/잠금과
query가 불일치한다. 전종 새 schema/실행과 restore가 함께 있어야 current valid 상태다.
각 내부 WP는 독립 완료 단계가 아니다. VERIFY가14개 durable integration seam과 선행127조건/별도31R06,
10dimensions를 실제 실행하고 exact source/archive를 독립 감사에 넘긴다. 구현은 root가 순차 수행하고
비참여 GPT-6 Astra High 독립 감사·finding 교정을 통과한 뒤만 U06으로 간다.

공유 계약의 의미 owner는 Freeze14요소, 구조 source는 CODE_REVIEW_002 TD01–11이다.
PHASE-CRSL-01이 유일 primary14요소를 소유하고 AG+VERIFY 두 final node를 포함한다.
즉시 소비/무환불/실제 종료/양수gate/빈 타일과 current corrosion 타이밍이 정상 public Core/host에서
함께 유효해야 한다. 직원 base20% 이후 유효 배치 효과와 별도 적 autonomous 주기를 유지한다.
지원 live snapshot과 U04 confirmed Continue는 다른 수명이며 미래 disk battle snapshot은 추가하지 않는다.
표준 receipt는 구조를 검증하고 이 문서/plan audits는 단계 직후 의미적 유효 상태를 검토한다.
''')
print('U05 P2G40 candidate:5 WPs,atomic4,2 final nodes,1 Phase,14 durable seams;validation pending')
