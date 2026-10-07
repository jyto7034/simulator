# U07 독립 생성물·진화·분절 상태 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CIOB-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CIOB-003; FPF-CIOB-001; PP-CIOB-001; PVR-CIOB-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CIOB-001
- Final Policy Freeze Ref: FPF-CIOB-001
- Complete Goal Member Set: GOAL-CIOB-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성은 구현 권한이 아니다. 원 연속 요청은 P2G 인계 후 GEX 구현·자체 검증·비참여 독립 감사와 교정을 별도로 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CIOB-01 | GOAL-CIOB-01 | WP-CIOB-DEFINITIONS, WP-CIOB-CREATION, WP-CIOB-SPATIAL, WP-CIOB-SEGMENTS, WP-CIOB-PUBLIC, WP-CIOB-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CIOB-001의 현 연속 요청/운영 갱신 인수; 기존 계획·유효 증거를 재사용하고 필수 통합·독립 감사를 생략하지 않는다.
- Baseline Evidence Key: source/data/schema/test/config/input/toolchain/lock/dependency와 실제 command/selector·raw receipt·owner Goal·claim을 함께 식별한다.
- Reuse Rule: PREDECESSOR_INTAKE_001.json의 U06 final target/audit002와 별도 현황 문서 delta를 인수한다. 이전 실행은 이전 source 실행으로 표시하고 현재 claim 영향이 없을 때만 재사용한다.
- Production Input Invalidation / Focused Revalidation Map: definitions/opcode→V01/04/05/06/07/18; creation/source/kernel→V01–04/07–09/14/16/17; spatial→V03/05/06/12/13/16; shield/cost→V02/10/11/13/14; query/host/serializer→V15/17/18; all new state→V19 and inherited native/fault/observer/restore.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 변경한 fixture의 old input/oracle/raw를 보존하고 의도된 새 의미와 legacy adapter를 구분한다. selector nonzero와 현재 source를 검증한다. 단순 expected rewrite로 PC 숫자/상한을 완화하지 않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 실제 affected producer/consumer를 함께 재검증한다. 테스트 파일 불변만으로 새 enum/layout/runtime 영향이 없다고 판단하지 않는다.
- Phase-Start Rule: 단계 시작 자체는 full suite 반복 근거가 아니다. source/claim 영향, 새 변경·실패·미해결 위험만 추가 실행 이유다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CIOB-01 mutation 전 U06 Complete/High product b33a8a019087f3b6766c8cf611636f46246fa3d895464b5a439f6d176694a401의 Core1794/1794/1803·host92/100+heap1,fmt/check/event-log/clippy/release/public schema/PC576·native/fault/restore를 exact 현재성·영향 분석으로 재사용한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CIOB-01 WP-CIOB-VERIFY가 integrated current Core default/test-fixtures/runtime-observation,host default/observation/heap,fmt/check/event-log/clippy/release, 공개 object pack/trial/host, affected persistence/fault/PC/native/allocator/observer와161 inherited+31R06을 닫는다.
- Normal Full-Validation Count: initial 불변 선행 baseline1회 재사용, final integrated 성공 set1회. 교정/실패/입력변경에 따른 필요한 재실행은 허용한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/0개/실행 뒤 입력 변경은 pass가 아니다. 원본 tool/product error를 남기고 교정 후 현재 set에 결속한다. R05 peak-only 경고와20분stress 제외만 상속하며 새 예외를 추가하지 않는다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CIOB-V01 / CODE_REVIEW_001.ko.md Element Coverage 01; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V01; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V02 / CODE_REVIEW_001.ko.md Element Coverage 02; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V02; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V03 / CODE_REVIEW_001.ko.md Element Coverage 03; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V03; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V04 / CODE_REVIEW_001.ko.md Element Coverage 04; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V04; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V05 / CODE_REVIEW_001.ko.md Element Coverage 05; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V05; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V06 / CODE_REVIEW_001.ko.md Element Coverage 06; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V06; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V07 / CODE_REVIEW_001.ko.md Element Coverage 07; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V07; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V08 / CODE_REVIEW_001.ko.md Element Coverage 08; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V08; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V09 / CODE_REVIEW_001.ko.md Element Coverage 09; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V09; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V10 / CODE_REVIEW_001.ko.md Element Coverage 10; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V10; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V11 / CODE_REVIEW_001.ko.md Element Coverage 11; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V11; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V12 / CODE_REVIEW_001.ko.md Element Coverage 12; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V12; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V13 / CODE_REVIEW_001.ko.md Element Coverage 13; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V13; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V14 / CODE_REVIEW_001.ko.md Element Coverage 14; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V14; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V15 / CODE_REVIEW_001.ko.md Element Coverage 15; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V15; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V16 / CODE_REVIEW_001.ko.md Element Coverage 16; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V16; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V17 / CODE_REVIEW_001.ko.md Element Coverage 17; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V17; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V18 / CODE_REVIEW_001.ko.md Element Coverage 18; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V18; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V19 / CODE_REVIEW_001.ko.md Element Coverage 19; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V19; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |
| IVA-CIOB-V20 / CODE_REVIEW_001.ko.md Element Coverage 20; exact Freeze acceptance | assigned_execution_node | GOAL-CIOB-01 / WP-CIOB-VERIFY | execution/single_goal/COVERAGE.json V20; actual nonzero execution and exact source/data/config/toolchain/input/command/raw receipt with expected-versus-observed; original failures and numeric oracles retained |

## Single Goal Member `GOAL-CIOB-01`

- Goal ID: GOAL-CIOB-01
- Goal Kind: single
- Phase ID: PHASE-CIOB-01
- Assigned Final Execution Node Set: WP-CIOB-DEFINITIONS, WP-CIOB-CREATION, WP-CIOB-SPATIAL, WP-CIOB-SEGMENTS, WP-CIOB-PUBLIC, WP-CIOB-VERIFY
- Covered Policy Element Revision Ref Set: PER-CIOB-01-REVIEWED-001, PER-CIOB-02-REVIEWED-001, PER-CIOB-03-REVIEWED-001, PER-CIOB-04-REVIEWED-001, PER-CIOB-05-REVIEWED-001, PER-CIOB-06-REVIEWED-001, PER-CIOB-07-REVIEWED-001, PER-CIOB-08-REVIEWED-001, PER-CIOB-09-REVIEWED-001, PER-CIOB-10-REVIEWED-001, PER-CIOB-11-REVIEWED-001, PER-CIOB-12-REVIEWED-001, PER-CIOB-13-REVIEWED-001, PER-CIOB-14-REVIEWED-001, PER-CIOB-15-REVIEWED-001, PER-CIOB-16-REVIEWED-001, PER-CIOB-17-REVIEWED-001, PER-CIOB-18-REVIEWED-001, PER-CIOB-19-REVIEWED-001, PER-CIOB-20-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CIOB-01-001, DR-CIOB-02-001, DR-CIOB-03-001, DR-CIOB-04-001, DR-CIOB-05-001, DR-CIOB-06-001, DR-CIOB-07-001, DR-CIOB-08-001, DR-CIOB-09-001, DR-CIOB-10-001, DR-CIOB-11-001, DR-CIOB-12-001, DR-CIOB-13-001, DR-CIOB-14-001, DR-CIOB-15-001, DR-CIOB-16-001, DR-CIOB-17-001, DR-CIOB-18-001, DR-CIOB-19-001, DR-CIOB-20-001
- Derived Observable Policy Acceptance Coverage: FPF-CIOB-001 exact20 member acceptance 전체; CODE_REVIEW_001 Element Coverage01–20과 integration mapping은 실제 검증 경계를 소유한다.
- Objective / Contract-Visible Result: 정상 작성/명령으로 진영을 따르는 독립 actor·함정·유한 field·HP벽과 같은 타일 진화, 단일 remaining 분절 보호 pool 및 분리/HP비용+벽 생성을 실행한다. 실제 행동·원자성·source/copy·수명·조회·복원이 일치하고 공식 친구/얼음벽/고기벽 소비와 독립 감사를 완료한다.
- User-Visible Problem: 현재 생성은 보스 적 actor에 묶여 있고 스킬의 빈 타일 생성·친구 진화·HP벽 접촉·분절 pool 분리/비용 consumer가 없다. 기존 shield와 unit만으로는 객체별 권한/회복/사망/승패 구분을 표현하지 못한다.
- User-Visible Completion Condition Set: V01–20 전체: 종류별 실제 능력/진영/권한분리, stable request와 전체 비용 원자성, 배치/상한, actor 행동, field tick, 친구 공격·진화, source독립/TTL/cleanup, segmented stack/detach/fallback, 벽 contact·HP·경로복귀, maxHP/order, 공명접수/취소, query/Pause/clone/restore, strict 외부작성/공식소비, 비용/독립감사.
- In-Scope / Out-of-Scope: U07/B13·C01과6 final nodes를 한Phase에서 순차 구현한다. Unity·개별 최종보스·full 관계망/감염/시체 소유권 producer·거미줄 자유프로그래밍·전체 조우/보상 공급·전용 장비 전체·미지원live-battle disk포맷·remote deploy·unrelated cleanup·20분stress는 제외한다. 공통 공개 runtime/공식 기본 consumer 미완성은 후속으로 넘기지 않는다.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001 Current Actual RQ01–08/TD01–08/IF01–04. strict object definitions/typed op→cast tile/kernel candidate→independent owner/actor/HP adapters; trap/evolution/contact/field; sustained remaining segmented descriptor→detach/HealthCost wall transaction; shared query/normal application/host/clone/reader-writer. 기존 Boss summon/TileArea/damage/Shield/U05/U04 owner 유지.
- Goal-Local Source of Truth / Owning Layer: raw/compiled catalog는 정의/종류/상한, SpawnKernel은 request settlement/원자성, BattleCore object owner는 logical ID/form/source/epoch/TTL, RuntimeUnit adapter는 actor/wall HP truth, sustained record는 pool remaining, World candidate는 정상입력/공명/게시, ActiveBattleSession은clock/query, host는정상투영/영속연결이다.
- Goal-Local Invariant Set / Source Refs: Freeze20 exact. 모든object를employee/combatant로강제하지않음; sourceSide와no wave/reward authority; cost+spawn전부commit; parent lease revival없음; top=P-(ceil(P/H)-1)*H/stack파생; mitigation한번/no falseHP-loss; fallbacknonlethal; wallnoheal/deathproc/reroute; firstcause/stale무해; Pausebattleclock; read-onlypreview/currentrevalidation; nativeguard/oracle유지.
- Code/Data/Schema/Store/Transport/Consumer Boundary: TD01–08. strict RON→reference/compile/source→normal effect/site/tile→sharedkernel→object/runtimeadapter/pool→movement/damage/lifecycle→facts/query→CoreApplication/trial/host. 각새field/queue/source는clone/retained/work에처음부터포함. serializer reader/writer와currentdata 동시전환. exactRust이름/초기수치/queue형태는GEX상세화.
- Consumer Verification Boundary: required 정상 외부 RON/API/CLI validate/explain/trial, 실제 CoreApplication ActivateSkill과 shared preview/query, live battle native, host actor/직렬화/stockport, clone/fault/reconnect와supportedContinue/SavePoint/originalprotection.146개capture의requiredowner정역검토. Unity렌더링/remote실행은주장하지않는다.
- Goal-Local Dependencies / Execution Prerequisites: U06 final002 CompleteHigh·PREDECESSOR_INTAKE 현재성, exactFreeze/Plan/receipt/Goal/Handoff. DEFINITIONS→CREATION→SPATIAL/SEGMENTS→PUBLIC→VERIFY 및 plan11edges. 새state의clone/extent/queuefault를먼저연결하고없는API를완료로간주하지않는다.
- Validation Plan / Completion Matrix: V01–20 exactacceptance+10차원+161선행/별도31R06. 작은strict/shape/atomicfault→기능별실제native→정상친구/벽/pool/actor/fieldpublicpack→최종Core/host/native/cost/observer/restore. 성공/불법입력/duplicate/stale/latefailure/CC/Pause/TTL/사망/철수/clone/order/upperbound의actualnonzero와raw를결속한다.
- Structural Completion / Legacy and Fallback Disposition: boss-only Opponent/admission을범용생성정책으로사용하지않음; trap을가짜employee나무한basicattack으로구현하지않음; wall을시설목표/staticreroute/Heal/deathproc로처리하지않음; poolstack이중truth/recentactivation추정/흡수의가짜HP-loss/비용partialcommit/lease부활없음. 기존정상boss/TileArea/shield/HealthCost/kill/피해/저장행동은회귀보존.
- Canonical Documentation Obligation: core_runtime_contract/skill_target_contract/data_loading_contract/공개authoring설명/host unity_bridge_protocol을실제포트와맞춘다. 최신game_rulebook/game_rules의독립객체·분절/벽/source경계와authored초기값을연결하고AI위임과직접합의를구분한다. 과거감사/동결bytes보존,공유완료상태게시/U08선택은독립감사뒤.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: PREDECESSOR_INTAKE와공통baseline계약에서161조건+별도31R06각각reuse/affected/intentionalchange/currentevidence를명시한다. 새enum/layout/runtime은PC/heap/5관측모드/clone/source/restore영향을분석하고원래숫자oracle를보존한다. 일반치유와wall제외및기존summon권한회귀도현재검증한다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CIOB-V01–V20 모두 GOAL-CIOB-01/WP-CIOB-VERIFY. 실제normalcommand/query/authoring/host/nativeprobe는구현자가완료하고감사자에게테스트구현을미루지않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: 각WP의schema/runtime/query/clone를동시전환하고unsupported는명시거절/원본보존한다. optionalabsent직렬화/fixture/checkpointcanonical영향을검증한다. source/copy/epoch/pool/settlement/queue의liveclone는독립이고같은actor재접속과processpre-entryContinue는분리한다. exactbefore/afterarchive/fuzz0정역복원증거를남긴다.
- Goal-Local Known Unknown / Blocker / Stop Condition: TD01–08안의exacttype/container/geometry/lease/DTO/초기balance는GEX상세이며policy/sourcegap0. 새material의미/필수source/단계경계문제는최초P2Gowner에서불변successor로수리한다. 사용자중단/승인범위완료/사용자권한환경의실제장애만중단조건이다.
- Applicable Risk / Reviewer Requirement and Evidence: 구현비참여GPT-6AstraHigh가20acceptance/10차원/161선행+31R06과actualsource/currentness/복원/nativecost/host를독립감사한다. 주요위험은보스/직원/시설권한누출, source/lease재사용, partialcost/이중pooltruth, staleepoch/TTL경쟁, wallheal/deathproc/경로변경, unboundedowner/누락된clone이다. finding교정·재감사CompleteHigh/open0전U08종속구현금지.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01–20 normalpublic creation/pool/officialconsumer | actual actor/trap/field/wall/evolution/HPcost results and query agree |
| invalid_input | required | V02/03/07/10/11/14/18 strict and currentpreflight | typed refuse/no unintended cost/object/ID/source/queue mutation |
| failure_rollback | required | V02/07/10/11/14/17/19 latebatch and controlledfault | whole candidate HP/pool/object/epoch/facts unchanged; accepted resonance stays perU05 |
| retry_idempotency | required | V02/07/09/14 duplicate/conflict/staleepoch | one settlement/cost/generation/removal; retry afterfault valid |
| ordering_concurrency | required | V06/09/12/13/16 same-time damage/TTL/evolve/source/victory | deterministic stable order, firstcause, no extra attack/despawn |
| boundary_empty_max | required | V03/06/10/11/13/19 no target/one-over/P150/1/0/lethal/overflow | exact authored/host bounds distinguished, partial top and no freeHP |
| disconnect_restart_persistence | required | V08/09/17 liveclone/reconnect/freshContinue/SavePoint | source/copy/epoch/pool/timer preserved at supported boundary and original protection |
| live_data | required | V01/04/05/06/12/18 actual RON/fragment/normal trial | real small/big friend, ice/flesh wall, independent actor/field and pool producer |
| producer_consumer | required | V02/10/11/15/17/18/19 source→cost/spawn→query/host/clone | singleauthority/sharedvalidator/executor and real admission/accounting |
| legacy_fallback_absence | required | V01/04/08/10/12/13/18/19 oldboss/shield/damage/Heal/restore regressions | no fakeemployee/goal/HP-loss/lease/reroute/fallback success; old valid contracts retained |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CIOB-01 | PER-CIOB-01-REVIEWED-001 | DR-CIOB-01-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-02 | PER-CIOB-02-REVIEWED-001 | DR-CIOB-02-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-03 | PER-CIOB-03-REVIEWED-001 | DR-CIOB-03-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-04 | PER-CIOB-04-REVIEWED-001 | DR-CIOB-04-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-05 | PER-CIOB-05-REVIEWED-001 | DR-CIOB-05-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-06 | PER-CIOB-06-REVIEWED-001 | DR-CIOB-06-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-07 | PER-CIOB-07-REVIEWED-001 | DR-CIOB-07-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-08 | PER-CIOB-08-REVIEWED-001 | DR-CIOB-08-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-09 | PER-CIOB-09-REVIEWED-001 | DR-CIOB-09-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-10 | PER-CIOB-10-REVIEWED-001 | DR-CIOB-10-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-11 | PER-CIOB-11-REVIEWED-001 | DR-CIOB-11-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-12 | PER-CIOB-12-REVIEWED-001 | DR-CIOB-12-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-13 | PER-CIOB-13-REVIEWED-001 | DR-CIOB-13-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-14 | PER-CIOB-14-REVIEWED-001 | DR-CIOB-14-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-15 | PER-CIOB-15-REVIEWED-001 | DR-CIOB-15-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-16 | PER-CIOB-16-REVIEWED-001 | DR-CIOB-16-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-17 | PER-CIOB-17-REVIEWED-001 | DR-CIOB-17-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-18 | PER-CIOB-18-REVIEWED-001 | DR-CIOB-18-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-19 | PER-CIOB-19-REVIEWED-001 | DR-CIOB-19-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
| PE-CIOB-20 | PER-CIOB-20-REVIEWED-001 | DR-CIOB-20-001 | PHASE-CIOB-01 | GOAL-CIOB-01 |
