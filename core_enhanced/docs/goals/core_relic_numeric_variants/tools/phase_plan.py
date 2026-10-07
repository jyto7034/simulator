"""Fresh U08 atomic runtime cutover and one independently auditable phase."""
import json
from initialize import HERE, artifact, write

def main():
    f=json.loads((HERE/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]
    basis=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())['current_basis']
    def ids(*ns):return [f'PE-CRNV-{n:02}' for n in ns]
    rows=[
      ('DEFINITIONS',ids(1,5,9,14),ids(2,4,6,7,8,10,11,12,15),'strict relic raw schema→immutable catalog/variant/effect selectors→registered supported capability',
       'TD01/03/04:stable definition/effect IDs,normal/corrupt numeric effects+typed penalties,duplicate/incompatibility and authored dust yield;strict current validation and public content identity',
       'Current121sources;existing content_pack winner/compiler/EffectSite authority;R14/P08 adopted meanings',
       'newID RON valid/invalid references,typed same-effect and exact ID/form selectors,missing/invalidvariant,unsupported closure explicit diagnostic;shared definition unchanged',
       'exact Rust module/DTO layout flexible;official named penalties/supply values remain unagreed'),
      ('OWNERS',ids(1,2,3,4,10,11,13),ids(5,7,9,12,15),'run copies/three slots/resources→staged normal grant/loadout/corruption/dismantle/Event→commit',
       'TD01/02:paired slot/copy transition,new run equipmentdust/catalyst owner preserving legacy dust,once-copy variant,normal Event authority and sale refusal;actual new-death recovery',
       'DEFINITIONS strict generation;NUMERIC projectedplan validity and PERSISTENCE same-rootcapture/restore must cut over together',
       '3/4slot,copydoubleequip,duplicate/incompatibility/dormancy permutations,free transfer,battle refusal;cost0/1/cancel/retry/stale/latefailure and death recovery',
       'new generic run-resource placement flexible but no legacyasset reclassification or new gameplay stock cap'),
      ('NUMERIC',ids(5,6,7,8,9,13),ids(1,2,3,10,12,14,15),'current loadout/provider→immutable individual numeric plan→accepted capture/carrier→all actual amount and query consumers',
       'TD03/04:base catalog ptr_eq/generation/action preserved;copy+effect overlay and separate basic-attack typed address;flatthen samekindpercent with checked rounding;literal observationoff and Expr/basic/auto/queued/projectile/area;rawsource and authored penalty execution',
       'DEFINITIONS typed addresses and OWNERS currentcopy/variant graph;PERSISTENCE rebuilt trusted bindings;existing Snapshot/Live remains',
       'twoemployees sameSkillId,180/fraction/overflow/originoracle,slotorder,tag/form changes,sourceending delayed result,stale provider,R09suppression,preview replay and emptycost',
       'overlay representation/private helper exactness is implementation choice;authority weakening or changed formula returns owner'),
      ('PERSISTENCE',ids(12,13,15),ids(1,2,3,4,8,10,11),'all run/candidate/save/entry roots→current-content validation→restore/death/newrun and measured lifetime',
       'TD05:GameCoreState/EconomyGrant/checkpoint/entry/active_run clone/writer/reader fields together;copy/variant/catalyst/dust persistent,derived plan rebuild,legacy bytes preservation and admission/retained/CloneShape',
       'OWNERS canonical graph and DEFINITIONS references;NUMERIC validates restored effective binding;U04/U07/U07M preservation',
       'before/after checkpoint,freshCore,currentreconnect,Continue/retreat,invalid save original protection,death/withdraw/redeploy keepsvariant,newrun reset,clone/latefault/repeated retention',
       'exact codec successor or safe empty-member adaptation must preserve old supported inputs and reject present malformed graphs'),
      ('PUBLIC',ids(14),ids(*range(1,16)),'completed runtime API→normal external RON/trial/CLI and concrete integration fixtures/docs',
       'TD06:representative new numeric/variant RON and third-party recombination,normal grant/equip/Event/catalyst/dismantle/save/query scenarios;durable tests/probes;capability and authors guide',
       'AG-CRNV-RUNTIME fully connects schema/owners/numeric/persistence so public support has actual consumers',
       'normal expedition/trial/public release paths with nonzero actions and exact expected results;unsupported authoring specific diagnostics;trace on/off samegameplay',
       'fixture names/numbers are test inputs,not final official content balance or whole shop/Unity completion'),
      ('VERIFY',ids(15,16),ids(*range(1,15)),'current source/evidence/impact/adaptation→coverage/frozen exact target→whole-goal independent audit',
       'TD06:16acceptance groups/10dimensions/191predecessors+31R06;final affected Core/host/public/fault/cost/PC/R05;restoration/currentness and independent CompleteHighopen0 beforeU09',
       'RUNTIME/PUBLIC normal consumers complete and required raw execution inputs/receipts available',
       'actual nonzero selectors and current source/toolchain/input;original failures/hardguards preserved;PC31/5;R05peak-onlywarning unchanged;forward/reverse exact restoration',
       'measured costs/latefindings remain executionwork;no Unity/deployment/finalbudget adequacy claim')]
    wps=[{'id':'WP-CRNV-'+key,'selectors':{'implementation':im,('validation_only' if key=='VERIFY' else 'supporting'):sup},'boundary':boundary,'expected_change':change,'prerequisite':prereq,'validation':validation,'unknown':unknown} for key,im,sup,boundary,change,prereq,validation,unknown in rows]
    edges=[
      ('DEFINITIONS','OWNERS','schema_data','copy/variant transactions must resolve current immutable definitions'),
      ('DEFINITIONS','NUMERIC','schema_data','effective numeric plans require exact compiled effect addresses and supported selectors'),
      ('DEFINITIONS','PERSISTENCE','schema_data','saved definitions/variants must be validated against current content'),
      ('OWNERS','NUMERIC','producer_consumer','personal plan needs authoritative copy/slot/variant bindings'),
      ('NUMERIC','OWNERS','transaction','equip/corruption commit requires projected numeric-plan validity,not post-commit failure'),
      ('OWNERS','PERSISTENCE','persistence_lifetime','new mutable roots must participate in every save/entry/candidate capture'),
      ('PERSISTENCE','OWNERS','authority','restored graph validation and complete candidate roots are required before normal mutations can be valid'),
      ('NUMERIC','PERSISTENCE','producer_consumer','restore rebuilds derived numeric authority from validated persistent bindings'),
    ]
    edges += [(key,'PUBLIC','producer_consumer',key+' actual runtime behavior is required before exposing normal supported RON/query/trial fixtures') for key,*_ in rows[:4]]
    edges += [(key,'VERIFY','validation',key+' current behavior is required by whole-goal validation') for key,*_ in rows[:-1]]
    phase='PHASE-CRNV-01';ag='AG-CRNV-RUNTIME';members=[w['id'] for w in wps[:4]]
    p=artifact('Phase Plan','PP-CRNV-001',{
      'Bound Final Policy Freeze Ref':f['ref'],'Exact Prerequisite Composite Basis':basis,
      'Applicable P2G-40 Obligation Key Set':[],'P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],
      'Goal Structure Fact':'single_phase','Parallelization':'none approved','work_packages':wps,
      'dependencies':[{'from':'WP-CRNV-'+a,'to':'WP-CRNV-'+b,'type':t,'evidence':e,'broken':'Ignoring this dependency breaks '+e} for a,b,t,e in edges],
      'atomic_groups':[{'id':ag,'members':members,'evidence':'Minimum published normal relic runtime cutover:definitions without actual numeric/transaction consumers are unsupported parse-only;copy mutations without full save/candidate roots lose variant/cost;numeric needs authoritative copy graph and mutations must preflight that plan;restored graph must rebuild the same authority. Internal module work can be sequenced,but no subset is advertised as independently supported runtime.'}],
      'standalone':['WP-CRNV-PUBLIC','WP-CRNV-VERIFY'],
      'cyclic_sccs':[{'members':['WP-CRNV-NUMERIC','WP-CRNV-OWNERS','WP-CRNV-PERSISTENCE'],'disposition':'coimplement_atomic','evidence':'Projected plan validation,canonical copy graph and complete candidate/restore roots form one authority/transaction cutover;all contained in AG-CRNV-RUNTIME.'}],
      'phases':[{'id':phase,'primary':ids(*range(1,17)),'supporting':[],'nodes':[ag,'WP-CRNV-PUBLIC','WP-CRNV-VERIFY'],'exit':'All16acceptance groups/10dimensions/191predecessors+31R06 current;normal numeric/copy/variant/saving works and independent CompleteHigh/open0 beforeU09'}],
      'audits':[{'phase':phase,'state':'External authored numeric relics/variants can be normally granted,equipped,converted and used by individual actual casts;copy/cost/slot/save/recovery meanings agree without U09/U16/Unity',
        'checks':'No shared definition mutation,unconditional closure,plan authority bypass,literal observer-only implementation,rawsource replacement,Event free bypass,cost/variant partialcommit,legacydust conversion or temporary-state purification',
        'evidence':'16 acceptance groups/10 dimensions/191predecessors+31R06;current source and exact public/native input/toolchain/raw receipts;PC/R05 original guard comparison;forward/reverse restoration',
        'limitation':'Local Core/host scope;U09closure/U10causal/U15expanded economy/U16whole activities/U21official content/U26UI remain future;existing R05peakwarning/20min exclusion preserved'}],
      'integration':[{'id':f'IVA-CRNV-V{n:02}','source':f'CODE_REVIEW_001.ko.md Element Coverage {n:02};exact Freeze acceptance',
        'disposition':'assigned_execution_node','nodes':['WP-CRNV-VERIFY'],'phases':[phase],
        'locator':f'execution/single_goal/COVERAGE.json V{n:02}',
        'evidence':'Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS',
        'prerequisite':'AG-CRNV-RUNTIME and PUBLIC all required consumers integrated'} for n in range(1,17)],
      'Shared Contract Handoff':'Freeze owns16 meanings;TD01–06 own proposed state/transaction/plan/source/save/validation. Runtime atomic group includes inseparable schema/run/numeric/save producers and consumers. PUBLIC supplies concrete normal fixtures/docs over completed runtime;VERIFY owns durable final integration evidence and audit handoff.',
      'Known Unknown/Limitations':'No required source or policy gap. Exact helpers/container layout/schema suffix/fixture values and measured costs remain GEX detailed choices under TD01–06. Group internal order DEFINE→OWNERS→PLAN→LIFECYCLE is not independent product completion. One final phase is valid even if U09 never runs.'})
    assert len(wps)==6 and set().union(*(set(w['selectors']['implementation']) for w in wps))==set(ids(*range(1,17)))
    write('P2G_40_PHASE_PLAN_001.json',{'artifacts':[p],'validation_receipt_ref':{'Artifact Type':'Phase Plan Validation Receipt','Artifact ID':'PVR-CRNV-001','Content Digest':None},'validator_method_version':'p2g40-receipt-validator/1.2.0'})
    write('P2G_40_DESIGN_001.ko.md','''# U08 실행 경계

Bound Freeze FPF-CRNV-001. Changes/obligations/resolutions/supersessions: explicit empty. Parallelization: none approved.

DEFINITIONS·OWNERS·NUMERIC·PERSISTENCE의4WP는 AG-CRNV-RUNTIME으로 함께 전환한다. 수치 계획은 현재 사본/variant를 소비하고 전환 명령은 예상 계획을 검증해야 한다. save/entry/candidate writer-reader가 새 roots를 함께 포함하지 않으면 비용과 변형이 분리된다. schema를 등록하면서 실제 소비를 뒤로 미루는 중간 공개 상태도 지원하지 않는다. 이것이 최소 정상 runtime cutover의 근거이며 단순한 파일 충돌 때문에 묶은 것이 아니다.

PUBLIC은 완성된 runtime 위의 대표 외부 RON/제3자 조합·normal trial/CLI/명령·저장/조회 fixture와 문서를 제공한다. VERIFY는16수용군/10차원/191선행+31R06 현재 증거·정역복원·독립감사를 닫는다. condens된3실행node가 하나의 PHASE-CRNV-01을 구성한다. 내부순서 DEFINE→OWNERS→PLAN→LIFECYCLE→PUBLIC→VERIFY는 구현 순서이며 독립완료 phase가 아니다.

Phase 완료 직후 단독 수치/침식 유물을 정상 Core에서 사용·이전·회수·저장할 수 있고 U09 closure나 U16 전체 상점이 없어도 유효하다. 미지원 authoring은 명확히 거절한다. U09/U10/U15/U16/U21/UI를 이번 범위에 선구현하지 않는다. legacy fragment dust·기본8MiB·기존 source/평가/복원과 PC31/5·원래R05guard를 보존한다.

지정Sol/xHigh가 상세구현·시험·교정을 맡고 비참여Astra/High가 exact frozen target을 전체감사한다. root는 owner 설계/조정을 맡으며 구현/최종감사를 겸하지 않는다. Complete/High/open0 뒤 기존 연속 권한으로U09에 진입한다.
''')
    print('U08 P2G40 candidate:6WP/1atomic group/3execution nodes/1phase/16integration seams')

if __name__=='__main__':main()
