# U08 유물 사본·수치 계획·침식 변형 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRNV-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRNV-003; FPF-CRNV-001; PP-CRNV-001; PVR-CRNV-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRNV-001
- Final Policy Freeze Ref: FPF-CRNV-001
- Complete Goal Member Set: GOAL-CRNV-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성 자체는 구현 권한이 아니다. 기존 연속 사용자 요청은 P2G 인계 뒤 GEX 구현·자체검증·독립감사/교정·후속을 별도로 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRNV-01 | GOAL-CRNV-01 | AG-CRNV-RUNTIME, WP-CRNV-PUBLIC, WP-CRNV-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRNV-001 현 연속 요청/효율화 지침/ROADMAP U08;유효한exact이전증거 재사용과 필수current통합·독립감사.
- Baseline Evidence Key: source/data/schema/test/config/input/toolchain/lock/dependency·실제command/selector/rawreceipt/ownerGoal/claim을 함께 결속한다.
- Reuse Rule: PREDECESSOR_INTAKE_001의 U07-M target001/audit001와 published상태문서2개delta를 인수한다. exactcurrent 또는 해당claim에 영향없다는명시근거가 있어야 재사용한다.
- Production Input Invalidation / Focused Revalidation Map: schema/copy/slots→V01–05/09/14;numeric/capture/basic/source→V05–08/09/13/15/R06/U05/U07/PC;grant/cost/Event→V04/09–12;save/entry/death/clone→V03/10/12/13/15/U04;layout/metrics→V15/U07M;전체→V16.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 원input/oracle/raw/실패를보존한다. 신규fixture와실행은새label. defaultPC31성공5거절과hardguard를comparator로완화하지 않는다. temporarydevprofile결과로default회귀를대체하지않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: affectedproducer/consumer를함께현재검증한다. test파일불변은commonruntime/layout비영향의증거가아니다. source/clone/amount/save확장과host직렬화영향을별도분류한다.
- Phase-Start Rule: 새단계시작만으로전체시험을반복하지않는다. 변경·실패·미해결위험에비례한focused검증과최종current필수set을실행한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRNV-01이제품mutation전U07-M CompleteHigh product f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8의선정54receipts를exact인수한다. Core1875/1875/1892,host93/103/122+heap,normalplayback26/lifecycle10/host4x2,PC002576/36/31성공5거절,R05002원실패101/peak9669160cap9030188의단독경고와finalcheck/fmt/clippy/public/release를현재impact에연결한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRNV-01/WP-CRNV-VERIFY가현재 Coredefault/alltargets/test-fixtures/runtime-observation,check/event_log_checks/releasebins/clippy/fmt,영향hostdefault/observation/measurement/heap와publicCLI/native/fault/save/numeric/PC/R05,U07M관측을닫는다. 기능workers완료뒤성능측정한다.
- Normal Full-Validation Count: 초기exact선행baseline1회재사용,최종current성공set1회. 실패·교정·입력변경은필요재실행사유이며원본보존한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/0선택/실행뒤입력변경은PASS아님. 원receipt→교정→새label·현재결속. R05 concurrentpeak-only기존경고와20분stress제외만상속한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRNV-V01 / CODE_REVIEW_001.ko.md Element Coverage 01;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V01; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V02 / CODE_REVIEW_001.ko.md Element Coverage 02;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V02; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V03 / CODE_REVIEW_001.ko.md Element Coverage 03;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V03; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V04 / CODE_REVIEW_001.ko.md Element Coverage 04;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V04; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V05 / CODE_REVIEW_001.ko.md Element Coverage 05;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V05; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V06 / CODE_REVIEW_001.ko.md Element Coverage 06;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V06; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V07 / CODE_REVIEW_001.ko.md Element Coverage 07;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V07; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V08 / CODE_REVIEW_001.ko.md Element Coverage 08;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V08; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V09 / CODE_REVIEW_001.ko.md Element Coverage 09;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V09; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V10 / CODE_REVIEW_001.ko.md Element Coverage 10;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V10; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V11 / CODE_REVIEW_001.ko.md Element Coverage 11;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V11; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V12 / CODE_REVIEW_001.ko.md Element Coverage 12;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V12; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V13 / CODE_REVIEW_001.ko.md Element Coverage 13;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V13; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V14 / CODE_REVIEW_001.ko.md Element Coverage 14;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V14; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V15 / CODE_REVIEW_001.ko.md Element Coverage 15;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V15; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |
| IVA-CRNV-V16 / CODE_REVIEW_001.ko.md Element Coverage 16;exact Freeze acceptance | assigned_execution_node | GOAL-CRNV-01 / WP-CRNV-VERIFY | execution/single_goal/COVERAGE.json V16; Actual nonzero normal/native/public execution and exact input/source/build/toolchain/raw receipt;expected-versus-observed or justified exact non-impact reuse;no plan-only PASS |

## Single Goal Member `GOAL-CRNV-01`

- Goal ID: GOAL-CRNV-01
- Goal Kind: single
- Phase ID: PHASE-CRNV-01
- Assigned Final Execution Node Set: AG-CRNV-RUNTIME, WP-CRNV-PUBLIC, WP-CRNV-VERIFY
- Covered Policy Element Revision Ref Set: PER-CRNV-01-REVIEWED-001, PER-CRNV-02-REVIEWED-001, PER-CRNV-03-REVIEWED-001, PER-CRNV-04-REVIEWED-001, PER-CRNV-05-REVIEWED-001, PER-CRNV-06-REVIEWED-001, PER-CRNV-07-REVIEWED-001, PER-CRNV-08-REVIEWED-001, PER-CRNV-09-REVIEWED-001, PER-CRNV-10-REVIEWED-001, PER-CRNV-11-REVIEWED-001, PER-CRNV-12-REVIEWED-001, PER-CRNV-13-REVIEWED-001, PER-CRNV-14-REVIEWED-001, PER-CRNV-15-REVIEWED-001, PER-CRNV-16-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CRNV-01-001, DR-CRNV-02-001, DR-CRNV-03-001, DR-CRNV-04-001, DR-CRNV-05-001, DR-CRNV-06-001, DR-CRNV-07-001, DR-CRNV-08-001, DR-CRNV-09-001, DR-CRNV-10-001, DR-CRNV-11-001, DR-CRNV-12-001, DR-CRNV-13-001, DR-CRNV-14-001, DR-CRNV-15-001, DR-CRNV-16-001
- Derived Observable Policy Acceptance Coverage: FPF-CRNV-001 exact16member acceptance전체;CODE_REVIEW ElementCoverage01–16과integrationmapping이실행경계를소유한다.
- Objective / Contract-Visible Result: 외부 RON의 불변 유물 정의를 정상 지급·3슬롯 장착·개인 수치 계획으로 실행하고,사본별1회 침식을 촉매/무료Event에서 원자 처리하여 저장·회수·조회까지 보존한다. U09/U16/UI가 없어도 단독 수치 유물이 정상 작동한다.
- User-Visible Problem: 현재 relic 정의/사본/variant/촉매 owner가 없고 같은 SkillId의 개인 유물 수치와 정상 변환·저장 경로를 제공할 수 없다. 기존 장비 Any3와 sequential modifiers는 유물3슬롯/C04합성을 대체하지 못한다.
- User-Visible Completion Condition Set: V01불변copy;V02직원3slot/제약;V03전환/사망회수;V04정상grant/판매금지/분쇄;V05sameeffect/정확조건;V06flat+percent;V07개인plan모든소비;V08rawsource/권한;V09작성variant/조회/패널티;V10once/실패원자;V11촉매vs무료Event;V12저장동시점;V13수명/비정화;V14외부작성정상경로;V15자원/관측보존;V16현재전체독립완료.
- In-Scope / Out-of-Scope: ROADMAP U08/C02–04와 H04채택R14 최소사본/분쇄/변환/전투/저장. 전체상점·공급활동U16,경제확장U15,closure/구조U09,인과U10,공식콘텐츠미합의값U21,UIU26-B와개별최종보스·새전투중diskcodec·20분stress 제외. 슬롯/중복정책을 재결정하지 않는다.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001 CurrentActualRQ01–07/TD01–06/IF01–04. Existingcopy/pairedloadout/stagedgrant/catalogptr_eq/capture/normalamount/save/entry/death 재사용. 새reliccatalog/runroot/numericbinding·basictypedaddress·정상Event권한·equipmentdust/catalyst·모든restore/query소비를 연결한다.
- Goal-Local Source of Truth / Owning Layer: GameData의 immutable relic catalog가 정의/variant/effect를 소유;run RelicInventory가 copy와3slot/variant를 소유;별도 run crafting resource가 장비가루/침식촉매를 소유;EconomyGrant candidate가 cost/result원자성;Battle 개인 immutable plan/carrier가 release수치/귀속;현재host/content가restore권한을 소유한다.
- Goal-Local Invariant Set / Source Refs: Freeze16/TD01–06:원형불변/copy독립/3slot/같은copy독점/직원별제약;flatthen동종percent/원stat산술보존;catalog권한·rawsource·SnapshotLive보존;literalobserveroff와basic포함;oncecorrupt/no정화;Event권한/0비용vs촉매1;costvariant동시restore;legacydust보존;기본8MiB/PC31·5 유지.
- Code/Data/Schema/Store/Transport/Consumer Boundary: content_pack/raw/validation/registration→relicdefinition/runstate/normalGrant/equip/corrupt/dismantle→playerprofile/build→SkillEffectSite또는BasicDamage typedplan→capture/source/carrier/queue/projectile/area/sim→amountquery/DTO/trial→checkpoint/entry/active_run/death/newrun. 각 신규owner의CloneShape/admission/retained를 포함한다.
- Consumer Verification Boundary: 외부새ID RON→공개CLI/trial/CoreApplication 정상commands/admingrant/Event/maintenance→두직원동일SkillId·basic/auto/queued/projectile/area실제결과→query/amountreplay→freshCore/Continue/retreat/checkpoint. private직접호출은 보조이며 정상경로를 대체하지 않는다.
- Goal-Local Dependencies / Execution Prerequisites: exactU07Mtarget001/audit001와2publishedstatusdelta·P2GFreeze/Phase/Receipt/Handoff. AG-CRNV-RUNTIME의DEFINITIONS/OWNERS/NUMERIC/PERSISTENCE 동시cutover→PUBLIC→VERIFY. copy→plan→projectedtransition과save/restore root의순환은atomicgroup으로 닫는다.
- Validation Plan / Completion Matrix: V01–16/10차원/191선행+31R06. 작은copy/slot/cancel→literal관측off·Expr·기본공격typed주소·180/소수/overflow·sourceauthority→earlyPC→variant/Event/분쇄/savefault→normalRON/CLI/trial→finalcurrentcore/host/public/PC/R05→exactfreeze/독립감사. nonzero실행과expected/observed를 연결한다.
- Structural Completion / Legacy and Fallback Disposition: parse-only유물등록,shared원형변경,종류별corruption장부,장비Any3대체,unconditionalclosure,ptr_eq삭제,sourceRelic치환,observer-only숫자,freeEventflag우회,legacydust환산,save누락,death정화,범용damagehelper전피해증폭과이중합성은 없다. unsupportedauthoring은 위치/원인을 명시한다.
- Canonical Documentation Obligation: 새relic작성/명령/조회·numeric반올림/지원capability·variant/패널티·Eventauthority·사본/저장/분쇄·legacydust구분을 기존canonical문서/사용guide에 반영한다. fixture수치는시험값으로표시한다. 독립완료뒤roadmap환류와U09/U10/U15/U16/U21후속소유를 연결한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 191조건(181선행+10U07M)와별도31R06마다 exactreuse/affected/current증거를 추적한다. source/capture/baseamount/clone/save/grant/commonlayout변경은 실제소비회귀 필요. PC원workload/31성공5거절/errorguard·R05원cap과원실패를보존하고 comparator를완화하지 않는다. profile/관측/allocator영향은U07M도검증한다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CRNV-V01–V16 전부 GOAL-CRNV-01/WP-CRNV-VERIFY 소유. PUBLIC이정상외부입력/시나리오를구현하고VERIFY가finalnonzero실행/원자료/조건mapping을 완성한다. 감사자에게제품시험구현을넘기지않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: 새runroot의capture/restore/serializervalidator를같이전환. savedderivedplan은실행권한이아님. 기존지원빈상태와malformedpresent를구별하고필요한schema이행을명시,구저장원본/legacyasset보존. 기존publicAPI호환wrapper·유물없는JSON/수치경로보존영향검토. forward/reversearchive/diff/currentness,noreset/clean/stash.
- Goal-Local Known Unknown / Blocker / Stop Condition: 정책/sourcegap0. 정확helper/container/API/codec/fixture선택과실측비용은GEX재량. 의미/권한/상태소유/완료조건의변경·공식미합의콘텐츠채택·legacy환산·기본cap완화는earliestP2Gowner로반환. 사용자중지/승인범위완료/현재권한환경에서해결불가능한실제장애외에는계속한다.
- Applicable Risk / Reviewer Requirement and Evidence: 비참여GPT-6AstraHigh가전체16수용군/10차원/191+31·3최종node·정상source/입력·복원·fault·PC/R05·U07M관측을감사. 위험은copy/variant혼합,관측off/basic누락,provider권한우회,지연source소실,Event무료우회,cost/save부분commit,retained회귀. CompleteHigh/open0전U09의존구현금지.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01–14normalgrant/equip/numeric/variant/save | 두직원개인실제결과·공개RON·cost/query일치 |
| invalid_input | required | V02/05/09/10/11/14badcopy/selector/variant/authority | 위치명확한거절·무부분commit |
| failure_rollback | required | V04/10/12/15lategrant/plan/cap/savefault | copy/catalyst/dust/slot/RNG/UUID/Event/저장원본보존 |
| retry_idempotency | required | V03/10/11/13duplicatecorrupt/Event/death | once변환·한번Event·무중복회수/소비 |
| ordering_concurrency | required | V02/06/07/08permutation/twoactor/queued | 동일집합동일계획·개인격리·SnapshotLive/방출권한보존 |
| boundary_empty_max | required | V02/06/09/15empty/3slot/overflow/cap | empty빠른경로·4th거절·유한수치/실행한도·no게임play증폭cap |
| disconnect_restart_persistence | required | V12/13freshCore/reconnect/Continue/retreat | 소비변형동시시점·variant비정화·구저장원본보호 |
| live_data | required | V09/14externalnewID/recombine/normaltrial | 실제등록/명령/효과/패널티·schema만성공금지 |
| producer_consumer | required | V05–08/11/12allsource/plan/Event/save | baseauthority→개인plan→carrier/actual/query와atomicrunroots일치 |
| legacy_fallback_absence | required | V08/14/15/16noobserveronly/rawsourceoverwrite | 기존PC/default/source/legacydust보존·no무조건closure/freeflag우회 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRNV-01 | PER-CRNV-01-REVIEWED-001 | DR-CRNV-01-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-02 | PER-CRNV-02-REVIEWED-001 | DR-CRNV-02-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-03 | PER-CRNV-03-REVIEWED-001 | DR-CRNV-03-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-04 | PER-CRNV-04-REVIEWED-001 | DR-CRNV-04-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-05 | PER-CRNV-05-REVIEWED-001 | DR-CRNV-05-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-06 | PER-CRNV-06-REVIEWED-001 | DR-CRNV-06-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-07 | PER-CRNV-07-REVIEWED-001 | DR-CRNV-07-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-08 | PER-CRNV-08-REVIEWED-001 | DR-CRNV-08-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-09 | PER-CRNV-09-REVIEWED-001 | DR-CRNV-09-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-10 | PER-CRNV-10-REVIEWED-001 | DR-CRNV-10-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-11 | PER-CRNV-11-REVIEWED-001 | DR-CRNV-11-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-12 | PER-CRNV-12-REVIEWED-001 | DR-CRNV-12-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-13 | PER-CRNV-13-REVIEWED-001 | DR-CRNV-13-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-14 | PER-CRNV-14-REVIEWED-001 | DR-CRNV-14-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-15 | PER-CRNV-15-REVIEWED-001 | DR-CRNV-15-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
| PE-CRNV-16 | PER-CRNV-16-REVIEWED-001 | DR-CRNV-16-001 | PHASE-CRNV-01 | GOAL-CRNV-01 |
