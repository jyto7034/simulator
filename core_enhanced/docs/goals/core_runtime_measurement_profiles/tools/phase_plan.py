"""U07-M actual dependency plan; one integrated auditable phase."""
import json
from initialize import HERE, artifact, write
f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
basis=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())['current_basis']
def ids(*ns): return [f'PE-CMRP-{n:02}' for n in ns]
rows=[
 ('PROFILE',ids(1,2),ids(3,6,9,10),'trusted strict settings→effective owner limits→private operation authority',
  'TD01/02 optional revision1 finite emergency<=32MiB; default bytes/hash/8MiB strict API; active/pending/deferred/projection checks and explicit small overrides agree',
  'U07 exact baseline; current immutable GameData settings and native budget owner',
  'default canonical bytes/hash and PC first; malformed/content/save attempt; warning/cap exact/one-over, other guards, outside snapshot, nested/deferred and local override',
  'exact private helper names/types are implementation choices; no default numeric relaxation'),
 ('ACCOUNTING',ids(3,4),ids(1,2,5,8,9),'existing native Budget/extent→fixed distinct attempted/accepted/committed measurements',
  'TD02/03 preserve cumulative nonrefund/sticky fault and old admitted field; fixed seed/growth/request/firstsite metrics and separate owner extent',
  'PROFILE provides immutable warning/effective cap; existing scope/rollback remains authoritative',
  'failed seed, growth rejection, nested/candidate discard/next operation, exact/overflow/latefault; committed current not failed candidate; feature off parity',
  'outside-native capture remains outside native metric scope, covered by process allocator; no retroactive whole-core heap claim'),
 ('COLLECTOR',ids(5),ids(4,6,7,8,9),'Core bounded summaries→host bounded registry/ring→process allocator/RSS/raw export',
  'TD04 existing queue1024/batch128/ring300 reused; registry256; optional host measurement module/allocator from process start; bounded Linux RSS/missing; identity finite and no metric labels',
  'ACCOUNTING exposes fixed summaries without altering gameplay; existing host attachment and weak sessions',
  'none/detached/full/off parity; registry/ring/sample/file bounds/drop; cross-window free/realloc/failure/concurrency; weak retirement; raw summary counts',
  'helper decomposition and tighter finite bounds allowed; processwide attribution/measurement overhead always explicit'),
 ('PLAYBACK',ids(6,7,8),ids(1,2,3,4,5,9,10),'trusted replay manifest→normal CoreApplication and actual host→current results/identity/lifecycle samples',
  'TD05 bounded measurement binary uses external pack/public commands/ticks/checkpoints; expected profile gate before mutation; real server WebSocket+detail sample; repetitions/profile isolation and recovery',
  'PROFILE/ACCOUNTING/COLLECTOR define supported authority/metrics; current U03/U04 fault/current-host restore contract',
  'normal battle/U07 objects/warning/hardfailure raw; on/off equality, mismatch0execution, liveclone/currenthost checkpoint, Pause/reconnect/repeated retirement/concurrent two profiles',
  'exact public manifest/fixture commands selected by implementation; no private-only evidence or universal final tuning claim'),
 ('VERIFY',ids(9,10),ids(*range(1,9)),'current integrated evidence→whole-goal audit→future tuning handoff',
  'TD06 current10acceptance/10dimensions/181predecessors+31R06; default PC/R05 original guards; dev separate comparisons; selection/impact/adaptation/coverage/freeze/independent audit',
  'All prior nodes execute normal consumers and leave raw current receipts',
  'required affected Core/host/public/native/fault/allocator/save/numeric checks; exact inputs/restoration/currentness; AstraHigh CompleteHigh/open0; U30-C/P procedure',
  'measured runtime and final platforms/values not known in planning; final deployment adequacy remains later owner')]
wps=[{'id':'WP-CMRP-'+k,'selectors':{'implementation':im,('validation_only' if k=='VERIFY' else 'supporting'):su},'boundary':b,'expected_change':c,'prerequisite':pr,'validation':v,'unknown':u} for k,im,su,b,c,pr,v,u in rows]
edges=[('PROFILE','ACCOUNTING','producer_consumer','fixed measurement uses actual selected warning and cap'),('ACCOUNTING','COLLECTOR','producer_consumer','host consumes native summary semantics'),('COLLECTOR','PLAYBACK','producer_consumer','normal execution measurements require bounded actual collectors')]+[(k,'VERIFY','validation',k+' must have current normal execution evidence') for k,*_ in rows[:-1]]
phase='PHASE-CMRP-01'
p=artifact('Phase Plan','PP-CMRP-001',{'Bound Final Policy Freeze Ref':f['ref'],'Exact Prerequisite Composite Basis':basis,'Applicable P2G-40 Obligation Key Set':[],'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],
 'Goal Structure Fact':'single_phase','Parallelization':'none approved','work_packages':wps,
 'dependencies':[{'from':'WP-CMRP-'+a,'to':'WP-CMRP-'+b,'type':t,'evidence':e,'broken':'Ignoring edge breaks '+e} for a,b,t,e in edges],
 'atomic_groups':[],'standalone':[w['id'] for w in wps],'cyclic_sccs':[],
 'phases':[{'id':phase,'primary':ids(*range(1,11)),'supporting':[],'nodes':[w['id'] for w in wps],'exit':'10acceptances/10dimensions/181predecessors+31R06 current and independently Complete/High/open0 before U08'}],
 'audits':[{'phase':phase,'state':'trusted finite development profile and normal bounded Core/host measurement/reproduction work; deployment defaults and fault/save meanings remain',
  'checks':'no untrusted cap control, bypass/reset/refund/sticky-fault suppression, metric conflation, unbounded collector/file, stale profile replay or historical oracle rewriting',
  'evidence':'V01–10/10dimensions/181+31,current actual source/input/commands/raw and forward/reverse restoration, default/dev separate, actual host samples',
  'limitation':'local Linux preparation; optional observer overhead; no Unity/all-content/final budget adequacy; inherited R05 peak-only warning and20minstress exclusion'}],
 'integration':[{'id':f'IVA-CMRP-V{n:02}','source':f'CODE_REVIEW_001.ko.md Element Coverage {n:02}; exact Freeze acceptance','disposition':'assigned_execution_node','nodes':['WP-CMRP-VERIFY'],'phases':[phase],'locator':f'execution/single_goal/COVERAGE.json V{n:02}','evidence':'nonzero normal/native/host command and source/config/input/toolchain/raw receipt with expected-versus-observed; exact valid reuse or focused current revalidation','prerequisite':'All implementation nodes integrated'} for n in range(1,11)],
 'Shared Contract Handoff':'Freeze owns10 meanings. TD01–06 owns authority/limits/metrics/host/runner/validation. Each node includes its own inseparable producer-consumer and failure/clone boundary. Existing immutable settings, native Budget, Copy observation channel and CoreApplication APIs separate sequential nodes; final preparation completion requires every node.',
 'Known Unknown/Limitations':'No required source/material policy gap. Exact private helper/manifest/fixture/helper decomposition and observed costs remain GEX decisions. Five nodes are coherent implementation transitions inside one auditable phase, not independently complete products. No atomic group is needed across stable existing settings/budget/channel/application seams; internal paired producer/consumer changes belong to their node.'})
assert set().union(*(set(w['selectors']['implementation']) for w in wps))==set(ids(*range(1,11)))
write('P2G_40_PHASE_PLAN_001.json',{'artifacts':[p],'validation_receipt_ref':{'Artifact Type':'Phase Plan Validation Receipt','Artifact ID':'PVR-CMRP-001','Content Digest':None},'validator_method_version':'p2g40-receipt-validator/1.2.0'})
write('P2G_40_DESIGN_001.ko.md','''# U07-M 실행 경계

PROFILE→ACCOUNTING→COLLECTOR→PLAYBACK→VERIFY의5노드를 하나의 PHASE-CMRP-01에서 순차 수행한다. 설정/예산/관측채널/공개application의 기존 안정된 경계를 사용한다. 각 node 내부의 producer/consumer·실패·계량 변경은 같이 수행하고 중간 API를 최종 지원 완료로 광고하지 않는다.

Frozen10조건과 TD01–06을 보존한다. default8MiB/원래hash·작은override를 지키고 trusted finite개발 profile만 추가한다. 누적admission/실제allocator/RSS는 다른 지표다. 정상Core와 실제host의bounded표본·재현·lifecycle를 완성하고 current 통합·독립감사 뒤U08로 진행한다. 초기 선행 baseline은 exact U07/audit002를 재사용하며 단계마다 전체시험을 반복하지 않는다. 기존20분stress 제외와R05peak경고만 상속한다.

병렬 제품 agent는 승인하지 않는다. 지정Sol/xHigh가 순차 구현/시험하고 비참여Astra/High가 exact target 동결 뒤 전체감사한다. 최종 플랫폼·최대부하·배포값은U30-C/P 소유다. P2G 문서 작성 자체는 제품 구현/완료 판정이 아니다.
''')
print('P2G40 candidate:5 nodes/0 atomic groups/1 phase/10 durable seams')
