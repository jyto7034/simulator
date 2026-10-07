# U09 유한 활성·typed 속성·구조 변형 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRSA-002
- Predecessor Goal Set Ref: GS-CRSA-001 (immutable historical shared-shield basis; successor repairs CHG-CRSA-001)
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRSA-006; FPF-CRSA-002; PP-CRSA-002; passed PVR-CRSA-002; CHG-CRSA-001/COD-CRSA-001; O10–40 effectively resolved. Exact refs/digests and resolutions are P2G_30_RESULT_002.json/P2G_40_RESULT_002.json. O50 current repair; O60 downstream.
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRSA-002
- Final Policy Freeze Ref: FPF-CRSA-002
- Complete Goal Member Set: GOAL-CRSA-01
- Applicable P2G-50 Obligation Key Set: (COD-CRSA-001, O50); exact Container Ref in POST_FREEZE_CHANGE_001.json; no supersession.
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

CHG-CRSA-001의 O50 수리로 개인 방어막 변경을 반영한 successor Goal이며 기존 Goal 원문은 보존한다. 구체 합의와 검토는 WR006/FPF002/CODE_REVIEW002가 소유한다. 기존 generic shared 기능은 회귀 대상이며 포큐버스의 새 개인 보호와 구분한다.

이 문서는 Goal 작성이다. P2G-60 인계와 GEX 진입은 연속 사용자 요청의 별도 권한으로 진행한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRSA-01 | GOAL-CRSA-01 | AG-CRSA-RUNTIME, WP-CRSA-PUBLIC, WP-CRSA-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRSA-001 intake IN001 최신 연속 사용자 요청 및 IN003 continuous 지침. 유효한 기존 증거는 exact 입력 또는 명시적 비영향 근거로 재사용하고 역할·세션·단계 전환만으로 재실행하지 않는다.
- Baseline Evidence Key: owning Goal, 실제 command/test selector, fixture/input, relevant source/data/test/config, toolchain/lock/dependency, raw 결과·receipt와 claim을 함께 결속한다.
- Reuse Rule: U08 post_audit/u08_current_013_pass002/AUDIT_CLOSURE.json 및 u08_closure_20261007_001/PUBLICATION_RECORD.json의 인수 범위를 시작점으로 삼는다. TEST_SEAMS_REVIEW_001 regression_handoff의 223 condition ID와 별도 R06 31행을 실제 delta마다 분류한다. 이전 성공만으로 변경된 seam을 PASS하지 않는다.
- Production Input Invalidation / Focused Revalidation Map: schema/condition/grant/typed edits→V01–07/13; plan/source/occurrence/Flow/consumer→V05–12/15 및 R06/U05/U07/U08/PC; configuration/query/save/lifetime→V06/12–15 및 U04/U07M/U08; metrics/layout/clone/admission→V15/16 및 PC/R05/U07M. 전체 통합과 감사는 V17.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 변경된 해당 seam은 새 고유 label로 nonzero 실행하고 원 실패·oracle·input을 보존한다. fixture 규모·기대값·cap·PC31성공5거절·원 error guard를 성공을 위해 완화하지 않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 실제 producer/consumer와 source/layout/native clone/host transport 영향 집합을 함께 갱신한다. test 파일 불변은 제품 변경 비영향 근거가 아니다. successor evidence를 사용하면 소비 comparator도 현재 결속을 읽는다.
- Phase-Start Rule: Phase 시작만으로 재실행하지 않는다. 변경·실패·미해결 위험에 맞춘 focused revalidation 뒤 현재 최종 필수 검증을 닫는다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRSA-01이 제품 mutation 전에 독립 완료 U08의 선정 원본58 receipts와 후속 actual public evidence/current audit013을 exact/non-impact 기준으로 인수한다. TEST_SEAMS_REVIEW_001 및 U08 final selection이 command/selector 수의 권위다. 새 증거처럼 재기록하지 않으며 실제 누락·변경이 확인된 seam만 추가 실행한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRSA-01/WP-CRSA-VERIFY, 제품 Sol 통합 runner가 current Core default/all-targets/test-fixtures/runtime-observation, check/event_log_checks/release bins/clippy/fmt, 영향 host default/observation/measurement/heap와 public CLI/native/fault/save/source/PC/R05를 원래 selector와 한도로 실행한다. 정확 command는 GEX 기존 유효 runner에서 확인해 고유 label로 고정한다. 모든 기능 worker 종료 후 성능을 단독 실행한다.
- Normal Full-Validation Count: initial 유효 선행 baseline 1회 인수, final current 필수 set 1회 완료. 사용자 증거 재사용 지시를 적용하며 역할 전환으로 initial을 반복하지 않는다. 실패/입력 변경/불완전 실행은 필요한 재실행 사유다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: compile/0선택/skip/중단/환경 실패는 기능 PASS가 아니다. R05 기존 peak-only 경고, native historical binary SHA와 shared-host 한계를 명시하고 새 회귀와 구분한다. 20분 stress 제외를 임의로 확대하지 않는다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRSA-V01 / FPF-CRSA-002 PE-CRSA-01, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T01,T10 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V01 (to implement); durable independent test/probe and raw comparator per T01,T10; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V02 / FPF-CRSA-002 PE-CRSA-02, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T01 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V02 (to implement); durable independent test/probe and raw comparator per T01; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V03 / FPF-CRSA-002 PE-CRSA-03, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T01,T02,T10 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V03 (to implement); durable independent test/probe and raw comparator per T01,T02,T10; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V04 / FPF-CRSA-002 PE-CRSA-04, both exact acceptance sentences; successor TEST_PLAN_DELTA_002 P01–P05/PE04_T04; historical group mapping T02,T03,T04 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V04 (to implement); successor personal-grant independent test/probe and raw comparator per T02,T03,T04; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V05 / FPF-CRSA-002 PE-CRSA-05, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T02,T03,T12 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V05 (to implement); durable independent test/probe and raw comparator per T02,T03,T12; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V06 / FPF-CRSA-002 PE-CRSA-06, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T09 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V06 (to implement); durable independent test/probe and raw comparator per T09; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V07 / FPF-CRSA-002 PE-CRSA-07, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T02,T04 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V07 (to implement); durable independent test/probe and raw comparator per T02,T04; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V08 / FPF-CRSA-002 PE-CRSA-08, both exact acceptance sentences; successor TEST_PLAN_DELTA_002 P01–P05/PE04_T04; historical group mapping T05 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V08 (to implement); successor personal-grant independent test/probe and raw comparator per T05; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V09 / FPF-CRSA-002 PE-CRSA-09, both exact acceptance sentences; successor TEST_PLAN_DELTA_002 P01–P05/PE04_T04; historical group mapping T06 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V09 (to implement); successor personal-grant independent test/probe and raw comparator per T06; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V10 / FPF-CRSA-002 PE-CRSA-10, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T07,T09,T12 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V10 (to implement); durable independent test/probe and raw comparator per T07,T09,T12; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V11 / FPF-CRSA-002 PE-CRSA-11, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T08 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V11 (to implement); durable independent test/probe and raw comparator per T08; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V12 / FPF-CRSA-002 PE-CRSA-12, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T09,T10 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V12 (to implement); durable independent test/probe and raw comparator per T09,T10; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V13 / FPF-CRSA-002 PE-CRSA-13, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T02,T10 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V13 (to implement); durable independent test/probe and raw comparator per T02,T10; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V14 / FPF-CRSA-002 PE-CRSA-14, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T09,T11 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V14 (to implement); durable independent test/probe and raw comparator per T09,T11; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V15 / FPF-CRSA-002 PE-CRSA-15, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping T12 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V15 (to implement); durable independent test/probe and raw comparator per T12; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V16 / FPF-CRSA-002 PE-CRSA-16, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping E01 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V16 (to implement); durable independent test/probe and raw comparator per E01; actual input/source/toolchain/raw result와 독립 expected/observed. |
| IVA-CRSA-V17 / FPF-CRSA-002 PE-CRSA-17, both exact acceptance sentences; TEST_SEAMS_REVIEW_001 acceptance_mapping E02 | assigned_execution_node | GOAL-CRSA-01 / WP-CRSA-VERIFY | execution/single_goal/COVERAGE.json V17 (to implement); durable independent test/probe and raw comparator per E02; actual input/source/toolchain/raw result와 독립 expected/observed. |

## Single Goal Member `GOAL-CRSA-01`

- Goal ID: GOAL-CRSA-01
- Goal Kind: single
- Phase ID: PHASE-CRSA-01
- Assigned Final Execution Node Set: AG-CRSA-RUNTIME, WP-CRSA-PUBLIC, WP-CRSA-VERIFY
- Covered Policy Element Revision Ref Set: PER-CRSA-01-REVIEWED-001, PER-CRSA-02-REVIEWED-001, PER-CRSA-03-REVIEWED-001, PER-CRSA-04-REVIEWED-001, PER-CRSA-05-REVIEWED-001, PER-CRSA-06-REVIEWED-001, PER-CRSA-07-REVIEWED-001, PER-CRSA-08-REVIEWED-001, PER-CRSA-09-REVIEWED-001, PER-CRSA-10-REVIEWED-001, PER-CRSA-11-REVIEWED-001, PER-CRSA-12-REVIEWED-001, PER-CRSA-13-REVIEWED-001, PER-CRSA-14-REVIEWED-001, PER-CRSA-15-REVIEWED-001, PER-CRSA-16-REVIEWED-001, PER-CRSA-17-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CRSA-01-001, DR-CRSA-02-001, DR-CRSA-03-001, DR-CRSA-04-001, DR-CRSA-05-001, DR-CRSA-06-001, DR-CRSA-07-001, DR-CRSA-08-001, DR-CRSA-09-001, DR-CRSA-10-001, DR-CRSA-11-001, DR-CRSA-12-001, DR-CRSA-13-001, DR-CRSA-14-001, DR-CRSA-15-001, DR-CRSA-16-001, DR-CRSA-17-001
- Derived Observable Policy Acceptance Coverage: FPF-CRSA-002의17 member/34 exact 문장 전체. PE04/08/09는 successor 결정/검토와 TEST_PLAN_DELTA_002의 P01–P05/PE04_T04를 사용하고, 나머지14정책은 기존 revision/acceptance를 그대로 보존한다. TEST_SEAMS_REVIEW_001의 옛 shield 수용문장을 현재 Freeze 대신 쓰지 않는다.
- Objective / Contract-Visible Result: 정상 외부 RON 유물이 현재 직원의 실제 효과에서 유한 최소 활성과 typed 합성·구조 recipe를 만들고, 개인별 시전·보호·일반공격·회복·공개 설명·저장 재구성에서 같은 결과를 낸다. U10 reactive나 Unity 없이 단독 사용 가능하다.
- User-Visible Problem: 현재 유물은 damage 수치만 지원하고 generated 효과의 조건 전파, 공개 typed 속성 편집 및 단일 대상 보호/태아 지원/승려 damage-only area 변형을 개인 실행 구조로 제공하지 못한다. 원형 SkillDef 재조회와 radius 기반 sustained만으로 새 의미를 구현할 수 없다.
- User-Visible Completion Condition Set: V01효과 단위 조건;V02최소 활성/재계산;V03유한 grant/정적 거절;V04공개 typed support;V05결정적 산술/충돌;V06개인 plan과 실제 소비;V07원자 recipe/preserve;V08고정 recipient와 효과별 거리/identity;V09시전시 개인 방어막/단일 강화/원자 전체 재시전;V10regular-only 태아;V11HpLost 기준1회 Self heal;V12source/에고 권한;V13정상 외부 RON/설명;V14저장·사본·수명;V15admission/rollback/관측;V16선행 회귀/비용;V17whole-goal 독립 완료. 각 exact 두 문장은 Freeze가 소유한다.
- In-Scope / Out-of-Scope: U09 승인 유한 condition/grant/typed numeric/atomic structural recipe와 세 대표 사례, 공통 개인 plan/consumer/query/save/admission 및 정상 공개 검증. U10 reactive/미래 카드, 임의 private-field editing, 무한 grant, 공식 미정 S식·기간·침식 패널티·전체 encounter, Unity/개별 최종보스/새 live-battle disk resume는 제외한다. 시험용 수치는 공식 밸런스가 아니다.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_002 TD07–12와 새 RSB002/RB002가 current review다. CODE_REVIEW_001의 unchanged source facts/비shield TD와 DESIGN001 비shield 제안을 재사용한다. 기존 개인 Shield pool/expiry/priority, source, World staged application/Continue, scoped Flow/cold owner 기반을 확장해 cast-release recipient집합·source량1회·개인 group/recast 원자성·typed capability/query를 구현한다. DESIGN_DELTA002와 TEST_PLAN_DELTA002는 구체 제안/독립 oracle이며 실행 증거가 아니다.
- Goal-Local Source of Truth / Owning Layer: immutable GameData의 authored definition/compiled capability가 원형 권위; RelicInventory와 equipment/fragment copy/form 및 generation이 현재 구성 권위; 순수 plan builder가 해당 직원의 파생 graph/값/provenance를 소유한다. World PreparedCommit이 구성·비용 원자성, battle candidate가 mutable pool/member, accepted/released owner가 불변 plan 수명, 현재 content/generation 검사가 restore 권위를 소유한다. readonly query/provenance는 실행 권한이 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CRSA-002 PE01–17과 PP-CRSA-002: same-effect AND/정확 identity; 현재seed 최소유한 closure; Replace→Add→Scale checked/절삭1회; 원형불변/개인 source·copy격리; recipe preserve검증; 포큐버스 release집합 고정·개인잔량/이탈유지·latejoin/재충전없음·same-provider-copy-effect 전체교체 원자성; source량/modifier1회와 recipient 미증폭; fetus만 원 channel/range복귀; genericshared/recharge권한보존; regular-only새coefficient/primaryactor보존; Monk exact effect-occurrenceHpLost>0일때Selfheal1회; 원자 restore/사전 admission; 기존PC/R05oracle/cap유지.
- Code/Data/Schema/Store/Transport/Consumer Boundary: strict relic/skill authoring+ValueExpr/Flow→finite closure/typed effectiveplan→World prospective loadout/query→normal readiness/preview/paused/manual/basic→pending/Flow/released projectile/area/delayed→개인grant release batch/finitepool/exactgroup·fixedchannel→scopedsettledresults→checkpoint/Continue/hostpublication·nativeentity/extent/CloneShape. 네 core WP를 함께 전환한다. 개인 group key는 원본과 생성 effect identity를 모두 손실 없이 담고 source_creation은 새 명시 authored capability로 검증한다.
- Consumer Verification Boundary: workspace에서 확인한 CoreApplication/GameCore command/query, normal Event/reward/maintenance/trial와 release CLI, game_server host serialization/Continue 및 실제 battle 소비를 검증한다. TEST_SEAMS_REVIEW reuse_inventory가 실제 helper/adapter를 식별한다. R11/R12 private grant/roster/slot 주입은 component만 증명하며 정상 RON 획득 증거를 대체하지 않는다. Unity 등 접근하지 않은 external consumer의 배포/완료는 주장하지 않는다. latest fact 하나는 동시 hit 이력 전체가 아니므로 actor별 pre/post HP/shield/aggregate·보존 trace를 결합한다.
- Goal-Local Dependencies / Execution Prerequisites: U08 current audit013 및 publication delta, exact Freeze/Phase/passed Receipt와 후속 P2G-60 인계/GEX 진입. SCHEMA→PLAN↔CONSUMERS↔LIFECYCLE의 순환을 AG에서 닫고 PUBLIC→VERIFY를 완료한다. 역할은 Root Astra/High 계획·조정, Sol/xhigh 제품·통합 실행, 별도 Astra/High 시험·focused 실행; 서로 다른 파일만 병행하며 registry/test module 포함 파일 소유를 순차 이관하고 실행권을 한 명에게 준다.
- Validation Plan / Completion Matrix: 기존 T01–12/E01/E02의 비변경 seam과 TEST_PLAN_DELTA002 P01–P05/cross-group을 사용한다. 작은API/fixture연결→finiteclosure/typedrecipe/rollback→개인shield집합·량·흡수·priority·수명·atomicrecast/recharge분리→태아regular와Monkscopedsettled→모든consumer/paused/released/source→정상외부RON/grant/query→save/lifetime/budget→현source회귀/PC/R05→exacttarget/독립전체감사. Guest타입을만들어존재한다고가정하지않고실제ownedEmployee자격을검증한다. 실패before는해당transaction직전,기존commit비용소급환불없음. 실제nonzeroevidence를COVERAGE로연결한다.
- Structural Completion / Legacy and Fallback Disposition: 새로운 structural support에 parser/query-only 경로, global definition mutation, 역사 seed/self bootstrap, 임의 iteration partial success, private field patch, SkillId-only 공유 cache, global 원형 재선택, source/credit 치환, mutable pool의 rollback 밖 공유, derived save trust, 관측 off 기능 누락이 없다. 기존 numeric 호환 경로는 유지하되 합성을 두 번 하지 않고 새 단일 authority와 연결한다. legacy 비변형 aura의 적용 범위를 식별·회귀 확인하여 regular-only 새 구조 때문에 묵시 변경하지 않는다. future-only abstraction이나 fixture 전용 제품 동작을 추가하지 않는다.
- Canonical Documentation Obligation: 기존 docs/game_rules/relics.md 및 실제 content authoring/ability/command/query canonical 문서에 도달 가능한 typed capability, 조건·유한 grant, 합성·구조 preserve, effective explanation과 정상 RON 예를 반영한다. 실제 파일은 현재 nearest owner를 확인해 선택한다. 공식 미정 balance를 fixture로 확정하지 않는다. 독립 완료 뒤 roadmap status와 후속 U10 경계를 환류한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공통 baseline 계약과 223 고유 선행 조건+별도31 R06를 인수한다. 각 current delta가 affect하는 source/amount/queue/Flow/basic/sustained/save/clone/schema/public seam은 재검증한다. exact input 또는 명시적 비영향에 한해서 기존 evidence를 재사용한다. PC576 비시간 결과/36 workload/31성공5거절, R05 원 cap과 peak-only 경고 및 U07M 계량 경계를 보존한다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CRSA-V01부터 IVA-CRSA-V17 모두 GOAL-CRSA-01/WP-CRSA-VERIFY. PUBLIC은 정상 외부 fixture와 문서, VERIFY는 독립 test/probe/comparator·expected/observed·조건 연결을 작성한다. 제품 Sol은 제품 결함과 통합 원시 실행만 소유하고 테스트 기대값을 수정하지 않는다. 감사자는 이 구현을 대신하지 않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: legacy numeric RON/공개 envelope/원 source·SnapshotLive 보존. 새 compiled plan은 저장 권위로 추가하지 않고 검증된 current copy/form/variant/generation에서 restore 전에 재구성한다. 새 state는 candidate/clone/host lifetime에 함께 반영한다. strict malformed present 입력은 무변경 거절하며 원 저장본 보존; versioned global catalog의 의미를 조용히 변경하지 않는다. exact forward/reverse restoration은 격리 target에서 수행하고 repository reset/clean/stash는 사용하지 않는다.
- Goal-Local Known Unknown / Blocker / Stop Condition: required source/material policy gap0. 구체 enum/DTO/version/cache/fingerprint/정적 proof와 lowering 선택은 TD01–06 안에서 Sol이 상세 설계한다. 첫 API 연결을 시험 담당자가 확인한다. 정책/권한/상태소유/수명/저장/완료기준/회귀oracle/한도 변화가 필요하면 근거와 대안을 Root에 반환하고 earliest P2G owner에서 처리한다. 일반 compile/import/fixture 오류는 담당자가 같은 기대값으로 자율 교정한다. 승인 범위 확대나 실제 불가 장애가 아니면 진행한다.
- Applicable Risk / Reviewer Requirement and Evidence: closure 순환/상태 누출, grant source 위조, 같은 SkillId 혼합, target identity 이전, pool refill/개인 보호 우선순위 전환, Triggered 확산, effect-local heal 중복, delayed owner 소실과 누락 계량이 주요 위험이다. 계획·제품·시험에 참여하지 않은 Astra/High가 현재 Goal 전체17군/34문장/10차원/223+31·세 최종 node·원시 결과·제품설계·복원을 검토해야 한다. Complete/High/open0와 출구 currentness를 인수하기 전 U10 의존 구현으로 넘기지 않는다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | T01/T03–11 normal RON/grant/equip/cast/query, actor별 actual | 유한 활성·세 구조 결과·공개 설명·저장 재구성이 동일 의미 |
| invalid_input | required | T02/T04/T10 invalid cycle/property/recipe/reference/source | 위치·원인 typed 진단과 게시/상태 무변경 |
| failure_rollback | required | T02/T11/T12 early/late plan/allocation/restore fault | copy/slot/cost/UUID/RNG/plan/pool/원 저장본 부분 commit 없음 |
| retry_idempotency | required | T05/T06/T11 recast/retry/range out-back/source end/restore | source 정책대로 교체·비용 처리, 중복 pool/refill/과거 seed 없음 |
| ordering_concurrency | required | T01/T03/T08/T09 슬롯순열/다중 actor/동시hit/paused/queued | 동일 집합 동일 합성, owner 격리, 실제 settled heal 한 번 |
| boundary_empty_max | required | T01/T02/T03/T12 empty/missing/finite bound/overflow/budget | 빈 구성 빠른 경로와 안정 거절, partial closure 성공·한도 완화 없음 |
| disconnect_restart_persistence | required | T09/T11 fresh Core/reconnect/entry/current/Continue | 사본/generation 검증·derived 재구성·released lifetime·회수 보존 |
| live_data | required | T10 production loader new-ID external RON and invalid startup | 정상 획득·실제 시전·read-only 설명까지 nonzero, parse-only 금지 |
| producer_consumer | required | T04–12 plan→preview/accepted/Flow/released/basic/personal-grant/generic-shared/query/host | 원 source/qualification/cost/시간/pool/settled 의미가 실제 소비에 연결 |
| legacy_fallback_absence | required | E01/E02 current source/strict loader/PC/R05/U08 impact | 원형 재조회 우회·이중 authority·fixture 전용 동작 없음, legacy 허용 의미 보존 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRSA-01 | PER-CRSA-01-REVIEWED-001 | DR-CRSA-01-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-02 | PER-CRSA-02-REVIEWED-001 | DR-CRSA-02-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-03 | PER-CRSA-03-REVIEWED-001 | DR-CRSA-03-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-04 | PER-CRSA-04-REVIEWED-001 | DR-CRSA-04-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-05 | PER-CRSA-05-REVIEWED-001 | DR-CRSA-05-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-06 | PER-CRSA-06-REVIEWED-001 | DR-CRSA-06-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-07 | PER-CRSA-07-REVIEWED-001 | DR-CRSA-07-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-08 | PER-CRSA-08-REVIEWED-001 | DR-CRSA-08-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-09 | PER-CRSA-09-REVIEWED-001 | DR-CRSA-09-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-10 | PER-CRSA-10-REVIEWED-001 | DR-CRSA-10-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-11 | PER-CRSA-11-REVIEWED-001 | DR-CRSA-11-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-12 | PER-CRSA-12-REVIEWED-001 | DR-CRSA-12-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-13 | PER-CRSA-13-REVIEWED-001 | DR-CRSA-13-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-14 | PER-CRSA-14-REVIEWED-001 | DR-CRSA-14-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-15 | PER-CRSA-15-REVIEWED-001 | DR-CRSA-15-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-16 | PER-CRSA-16-REVIEWED-001 | DR-CRSA-16-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
| PE-CRSA-17 | PER-CRSA-17-REVIEWED-001 | DR-CRSA-17-001 | PHASE-CRSA-01 | GOAL-CRSA-01 |
