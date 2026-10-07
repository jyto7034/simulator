"""Publish validated P2G40 result and author U07-M quality2 Goal candidate."""
import json
from initialize import HERE, write
pa=json.loads((HERE/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0];p=pa['payload']
v=json.loads((HERE/'P2G_40_VALIDATION_001.json').read_text());assert v['result']=='passed' and not v['failures']
basis=json.loads((HERE/'P2G_30_RESULT_001.json').read_text())['current_basis'];basis['Phase Plan Ref']=pa['ref']
write('P2G_40_RESULT_001.json',{'stage':'P2G-40','status':'complete','Phase Plan Ref':pa['ref'],'Phase Plan Validation Receipt Ref':'PVR-CMRP-001','P2G-40 Resolution Record Ref Set':[],'Applicable Obligation Supersession Record Ref Set':[],'current_basis':basis,'All Applicable Change Obligations Resolved':True,'raw_exit':0,'validation':v,'semantic_design':'TD01–06/P2G_40_DESIGN_001.ko.md; five sequential implementation transitions in one integrated phase','next':'P2G-50'})
a=json.loads((HERE/'P2G_20_REVIEW_001.json').read_text())['artifacts'];es=sorted([x for x in a if x['ref']['Artifact Type']=='Policy Element Revision'],key=lambda x:x['payload']['Stable Element ID'])
nodes=', '.join(p['phases'][0]['nodes'])
s=f'''# U07-M 실측 기반 실행 예산 조정 준비 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CMRP-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CMRP-003; FPF-CMRP-001; PP-CMRP-001; PVR-CMRP-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CMRP-001
- Final Policy Freeze Ref: FPF-CMRP-001
- Complete Goal Member Set: GOAL-CMRP-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

이 문서는 Goal 작성이다. 원 연속 요청은 P2G 인계 후 GEX 구현·자체검증·독립감사와 교정을 별도로 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CMRP-01 | GOAL-CMRP-01 | {nodes} | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CMRP-001 현 연속 요청/효율화 지침/FOLLOWUP A1–A8; 유효한 이전 증거 재사용과 필수 current 통합/독립감사.
- Baseline Evidence Key: source/data/schema/test/config/input/toolchain/lock/dependency, 실제 command/selector·raw receipt·owner Goal·claim을 함께 결속한다.
- Reuse Rule: PREDECESSOR_INTAKE_001의 U07 target002/audit002와 두현황문서 delta를 인수한다. exact current 또는 실제 변경이 해당claim에 영향없다는 근거가 있을 때만 재사용한다.
- Production Input Invalidation / Focused Revalidation Map: settings/entry/cap→V01/02/03/06/07/defaultPC; budget/extent→V02/03/04/05/08/native/fault; observation/allocator/host→V04/05/07/08/heap; playback/identity/save→V01/06/07/08/public/restore; 전체→V09/10.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 원래 input/oracle/raw와 실패를 보존한다. 새 trusted 개발 profile은 별도 identity/label로 비교한다. default PC31성공/5기존거절 및 numeric hardguard를 comparator로 완화하지 않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: affected producer/consumer를 함께 현재검증한다. test파일 불변만으로 common runtime/layout 영향이 없다고 보지 않는다. 새개발profile의 의도된 허용차이와 default회귀를 분리한다.
- Phase-Start Rule: 단계 시작 자체는 전체시험 반복 사유가 아니다. 새변경·실패·미해결 위험에 필요한 focused/current실행만 추가한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CMRP-01이 mutation전 U07 CompleteHigh product886d427882319b3147260f394f1204acf903af654f772709bdaaa045f1c62146의 integrated008 Core1865/1865/1874,host93/101+heap1;fmt/check/event_log/clippy,public003/trace004/release004,numeric003/fault003/PC005576/R05005를 exact reuse/impact로 인수한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CMRP-01 WP-CMRP-VERIFY가 current Coredefault/fixtures/observation,hostdefault/observation/measurement/heap,fmt/check/event_log/clippy/release/public/native/fault/allocator/persistence/numeric/PC/R05와181선행+31R06을 닫는다. 기능workers 종료 뒤 성능측정한다.
- Normal Full-Validation Count: 초기 exact 선행baseline1회 재사용, 최종 current 성공set1회. 실패·교정·입력변경은 필요한 재실행 사유이며 원본은 보존한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/0선택/실행뒤입력변경을 pass로 쓰지 않는다. original receipt보존→교정→새 label로 현재 결속. inherited R05 concurrent peak-only warning과20분stress 제외만 상속한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
'''
for r in p['integration']:s+=f"| {r['id']} / {r['source']} | {r['disposition']} | GOAL-CMRP-01 / WP-CMRP-VERIFY | {r['locator']}; {r['evidence']} |\n"
s+='\n## Single Goal Member `GOAL-CMRP-01`\n\n'
fields={
'Goal ID':'GOAL-CMRP-01','Goal Kind':'single','Phase ID':'PHASE-CMRP-01','Assigned Final Execution Node Set':nodes,
'Covered Policy Element Revision Ref Set':', '.join(e['ref']['Artifact ID'] for e in es),
'Covered Decision Revision Ref Set':', '.join(e['payload']['Decision Revision Ref']['Artifact ID'] for e in es),
'Derived Observable Policy Acceptance Coverage':'FPF-CMRP-001 exact10 member acceptance 전체; CODE_REVIEW_001 Element Coverage01–10과 integration mapping이 실행경계를 소유한다.',
'Objective / Contract-Visible Result':'기존 배포 기본을 보존하는 trusted finite 개발 측정 profile을 정상Core/host에서 선택하고, warning/cap·누적admission·actualallocator/RSS·identity·결과를 boundedraw/요약/재현명령으로 제공한다. 필요한 lifecycle/보호/독립감사와 후속조정 인계를 닫는다.',
'User-Visible Problem':'현재8MiB engine ceiling 때문에 정상 비용을 넘어선 측정창이 없고, operation admission과 actualheap/RSS·정상host 표본을 함께 해석할 실행경로가 없다. hash helper는 실제 replay에서 소비되지 않는다.',
'User-Visible Completion Condition Set':'V01기본/신뢰선택;V02warning/emergency일치;V03누적/원자fault;V04지표분리;V05bounded비간섭;V06identity/복원;V07normalCore/host실측;V08반복/회수/isolation;V09현재검증/독립완료;V10후속조정절차.',
'In-Scope / Out-of-Scope':'등록U07-M A1–A8,5node/1phase. Unity·개별최종보스·새remote서비스/dashboard·모든콘텐츠/최종배포값증명·자동/무한완화·과거실패면제·새livebattle savecodec·unrelatedcleanup·20분stress는 제외한다.',
'Reviewed Current Behavior / Implementation Delta':'CODE_REVIEW_001 RQ01–06/TD01–06/IF01–04; strict settings→privateeffective entry/directowner/projection; fixed attempted/accepted/committed metrics→boundedhost/allocator/RSS→public replaygate/normalhost/lifecycle. 기존 Budget/application/save owner 재사용.',
'Goal-Local Source of Truth / Owning Layer':'RuntimeSafetySettings는 trusted immutable nominal/dev/identity; RuntimeFactState/RunResourceResult는 effective ownerlimits; TLSBudget은 operation누적/stickyfault; observer는 fixedmetrics/weaklease; hostmeasurement는processheap/RSS/file; currentGameData는restore권한.',
'Goal-Local Invariant Set / Source Refs':'TD01–06와freeze10. default8MiB/hash/oracle유지; explicittrustedfiniteonly; lowoverride존중; nestednoreset/refund; softwarningnohardfault; failedcandidate≠committedextent; admission≠heap≠RSS; noobservergameplayfeedback; mismatchbeforemutation/currenthostrestore; historicalbytespreserved.',
'Code/Data/Schema/Store/Transport/Consumer Boundary':'runtime_safety.rs strict optional DTO와raw/effectiveidentity→GameDataArc→BattleCore/Worldprivateentry/directlimits→runtime_controlmetrics→runtime_observationCopyqueue→hostregistry/ring/measurementfeature→boundedmanifest/export. 밖에서 호출되는 snapshot localallocationcap과 비nativecapture scope는TD02대로 처리한다.',
'Consumer Verification Boundary':'실제externalpack+공개CoreApplicationcommand/admin/tick/checkpoint API, 실제game_serverWebSocket/actor/attach/detail, nativefocusedbudget·defaultPC/R05, 저장/복원/fault와collector/allocator/RSS. private fixture만으로 normalhost완료를 주장하지 않는다.',
'Goal-Local Dependencies / Execution Prerequisites':'exactU07target002/audit002/현재publicationdelta와FPF/PP/PVR/Goal/Handoff. PROFILE→ACCOUNTING→COLLECTOR→PLAYBACK→VERIFY 및7edges. 새producer/consumer와failure/accounting을node안에서연결한다.',
'Validation Plan / Completion Matrix':'V01–10/10차원/181선행+31R06. strict/privatecap→nativecumulative/rollback/defaultPC→collector/allocator/RSS→normalCore/actualhost/lifecycle→최종current통합→sourceboundfreeze/독립감사. exact/oneover/nonzero와raw기대대조를 남긴다.',
'Structural Completion / Legacy and Fallback Disposition':'silentdefaultfallback/globalENGINE_MAX상향/content-savecap선택/postfaultcontinue/allowancerefund/상한없는registry/file/가짜RSS/실패candidate의current보고/출력hash만의replay승인없음. outside-nativecapture에몰래새guard추가하지않으며default유효계약과현재hostContinue유지.',
'Canonical Documentation Obligation':'core_runtime_contract 및새runtime_measurement사용문서/hostobservabilityREADME에측정scope·상한·지원/missing·replay/현재hostrestore·overhead·재현명령·default/dev차이·AI위임임시32MiB근거를기록. 독립감사뒤roadmap상태갱신. U08/U11/U30-C/P 책임명시.',
'Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan':'위baseline계약에따라181조건+31R06각각exactreuse/affected/의도된dev차이/currentevidence를추적. defaultsourcecost/publicsave/fault와observer/hostowner변경은현재검증. 원래PC/R05오라클과실패는불변이며개발표본을섞지않는다.',
'Assigned Durable Integration Validation Seam Coverage':'IVA-CMRP-V01–V10 전부 GOAL-CMRP-01/WP-CMRP-VERIFY. 구현자가 nonzero normal/native/host 증거를 완성하며 감사자에게 제품시험구현을 넘기지 않는다.',
'Applicable Migration/Compatibility/Rollout/Rollback':'기본설정bytes/hash와strict8MiB public API·저장schema보존; optionaldevfield만추가; ownerlimit/entry/directcheck동시전환. observer feature/data는gameplaystate외부. 측정binaryoptfeature와create-newboundedoutput. 정역복원archive/diff/currentness; noreset/clean/stash.',
'Goal-Local Known Unknown / Blocker / Stop Condition':'정책/sourcegap0. helper/manifest/fixtures와실측비용은GEX내부결정,최종플랫폼/정상최대/배포수치는U30-C/P. material계약/owner/완료기준변경은최초P2Gowner로반환해불변successor. 사용자중지/승인전체범위완료/해결불가능한권한환경장애에서만중단.',
'Applicable Risk / Reviewer Requirement and Evidence':'비참여GPT-6AstraHigh whole-goal10acceptance/10차원/181+31/sourcecurrentness/복원/normalhost/guard/identity/metrics감사. 주요위험은cap누락·override무시·누적환급·계량혼합·collector무한성·hash소비부재·관측feedback·defaultPC회귀. CompleteHigh/open0전U08금지.'}
for k,value in fields.items():s+=f'- {k}: {value}\n'
s+='\n### Validation Dimension Matrix\n\n| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |\n| --- | --- | --- | --- |\n'
dims=[('success','V01/02/04/07 normalCore/host','명시profile의정상결과와nonzero분리측정'),('invalid_input','V01/02/06 strictprofile/manifest','invalid/oversized/untrusted/mismatch실행전거절'),('failure_rollback','V03/05/08 latecap/collector/export','candidate/save원본보호와관측실패비간섭'),('retry_idempotency','V03/06/08 rollback/replay','operation누적비환급/새operationseed·stale/duplicate기존의미'),('ordering_concurrency','V03/05/08 nested/deferred/multisession','동일seed순서보존/profileisolation/processwideheap범위'),('boundary_empty_max','V02/04/05 exact/oneover/overflow/empty/full','finiteguard·sample/drop/missing·noallocfailedadmission'),('disconnect_restart_persistence','V03/06/08 Pause/reconnect/clone/Continue','U03fault/currenthostrestore와originalprotection'),('live_data','V01/07 externalpack/normalhost','실제지원전투/U07objects/selectedtrustedsettings샘플'),('producer_consumer','V02/04/06/07 allentry/metrics/hash','profileowner→admission→summary→boundedraw와정상replaygate'),('legacy_fallback_absence','V01/02/03/05/09 default/historical','기본hash/8MiB/PCguards·no silent fallback/relaxation')]
for d,seam,condition in dims:s+=f'| {d} | required | {seam} | {condition} |\n'
s+='\n## Policy–Phase–Goal Traceability\n\n| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |\n| --- | --- | --- | --- | --- |\n'
for e in es:s+=f"| {e['payload']['Stable Element ID']} | {e['ref']['Artifact ID']} | {e['payload']['Decision Revision Ref']['Artifact ID']} | PHASE-CMRP-01 | GOAL-CMRP-01 |\n"
assert len(es)==len(dims)==10
write('P2G_50_GOAL_SET.ko.md',s)
print('P2G40 complete; P2G50 quality2 candidate authored, validation pending')
