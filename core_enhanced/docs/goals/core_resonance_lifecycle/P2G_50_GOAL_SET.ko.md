# U05 공명 접수와 실제 사용 수명 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRSL-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRSL-004; FPF-CRSL-001; PP-CRSL-001; PVR-CRSL-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRSL-001
- Final Policy Freeze Ref: FPF-CRSL-001
- Complete Goal Member Set: GOAL-CRSL-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성 자체는 구현 권한이 아니다. 원 사용자 연속 요청은 별도로 P2G 인계 후 GEX 구현·검증·독립 감사·교정을 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRSL-01 | GOAL-CRSL-01 | AG-CRSL-RESONANCE, WP-CRSL-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRSL-002 IN-001 기존 계획·유효 증거 재사용/불필요 반복 금지; 필수 독립 감사는 유지.
- Baseline Evidence Key: command/test selector, fixture/input, relevant source/data/test/config, toolchain/lock/dependency, raw result와 owning Goal.
- Reuse Rule: evidence/PREDECESSOR_BASELINE_001.json의 U04 최종367 product hash exact unchanged를 확인해 선행 성공을 재사용. U05 새 동작의 검증으로 주장하지 않는다.
- Production Input Invalidation / Focused Revalidation Map: stat/profile/data→V01/02/12/13; admission/target/corrosion→V04/07–11; flow/buff/channel/gain→V02–07/12; query/clone/host→V04/09/12/13와 native/R06/observation; docs only→계약/구조/참조.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 바뀐 seam은 current source에서 nonzero 실행하고 새 receipt를 결속한다. 실패 원본은 보존한다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 모든 affected consumer/seam을 invalidation. test bytes 불변만으로 runtime 변경 전 결과를 재사용하지 않는다.
- Phase-Start Rule: 단계 시작은 재실행 이유가 아니다. 실제 changed input-to-seam이 재검증을 결정한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRSL-01, production mutation 전 U04 CompleteHigh baseline Core1734/1734/1743,host89/97+heap1,fmt/check/event-log/clippy/release,16 public consumers/PC576 및 continuation/rewind/fault actual evidence를 exact currentness로 인수한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRSL-01 WP-CRSL-VERIFY, integrated current Core default/test-fixtures/runtime-observation,host default/observation와 heap,fmt/check/event-log/clippy/release,normal public/host use-lifetime matrix 및 affected inherited/R06 seams.
- Normal Full-Validation Count: initial은 exact unchanged U04 baseline1회를 재사용, final integrated set은 한 번 성공 고정. 실패/입력 변경에 필요한 재실행은 허용한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/zero/입력 변경은 pass가 아니다. 교정 후 해당 set 재결속. 기존 R05 concurrent peak 승인 예외·20분 시험 제외는 보존하고 새 실패를 덮지 않는다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRSL-V01 / CODE_REVIEW_002.ko.md Element Coverage 01; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V01; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V02 / CODE_REVIEW_002.ko.md Element Coverage 02; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V02; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V03 / CODE_REVIEW_002.ko.md Element Coverage 03; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V03; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V04 / CODE_REVIEW_002.ko.md Element Coverage 04; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V04; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V05 / CODE_REVIEW_002.ko.md Element Coverage 05; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V05; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V06 / CODE_REVIEW_002.ko.md Element Coverage 06; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V06; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V07 / CODE_REVIEW_002.ko.md Element Coverage 07; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V07; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V08 / CODE_REVIEW_002.ko.md Element Coverage 08; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V08; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V09 / CODE_REVIEW_002.ko.md Element Coverage 09; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V09; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V10 / CODE_REVIEW_002.ko.md Element Coverage 10; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V10; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V11 / CODE_REVIEW_002.ko.md Element Coverage 11; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V11; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V12 / CODE_REVIEW_002.ko.md Element Coverage 12; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V12; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V13 / CODE_REVIEW_002.ko.md Element Coverage 13; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V13; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |
| IVA-CRSL-V14 / CODE_REVIEW_002.ko.md Element Coverage 14; exact Freeze member acceptance | assigned_execution_node | GOAL-CRSL-01 / WP-CRSL-VERIFY | execution/single_goal/COVERAGE.json V14; actual nonzero execution, exact source/data/config/input/toolchain/command/raw result and expected vs observed; preserve failed receipts |

## Single Goal Member `GOAL-CRSL-01`

- Goal ID: GOAL-CRSL-01
- Goal Kind: single
- Phase ID: PHASE-CRSL-01
- Assigned Final Execution Node Set: AG-CRSL-RESONANCE, WP-CRSL-VERIFY
- Covered Policy Element Revision Ref Set: PER-CRSL-01-REVIEWED-002, PER-CRSL-02-REVIEWED-002, PER-CRSL-03-REVIEWED-002, PER-CRSL-04-REVIEWED-002, PER-CRSL-05-REVIEWED-002, PER-CRSL-06-REVIEWED-002, PER-CRSL-07-REVIEWED-002, PER-CRSL-08-REVIEWED-002, PER-CRSL-09-REVIEWED-002, PER-CRSL-10-REVIEWED-002, PER-CRSL-11-REVIEWED-002, PER-CRSL-12-REVIEWED-002, PER-CRSL-13-REVIEWED-002, PER-CRSL-14-REVIEWED-002
- Covered Decision Revision Ref Set: DR-CRSL-01-002, DR-CRSL-02-002, DR-CRSL-03-001, DR-CRSL-04-001, DR-CRSL-05-001, DR-CRSL-06-001, DR-CRSL-07-001, DR-CRSL-08-001, DR-CRSL-09-001, DR-CRSL-10-001, DR-CRSL-11-001, DR-CRSL-12-002, DR-CRSL-13-001, DR-CRSL-14-001
- Derived Observable Policy Acceptance Coverage: FPF-CRSL-001 exact14 member acceptance 전부; 아래 설명은 구현 인계이며 의미를 수정하지 않는다.
- Objective / Contract-Visible Result: 아군 직원의 배치 기본20%+유효 명시 효과, 전투초당1 충전, 유효 접수 즉시 전량1회 소비와 actual use-end 잠금을 같은 Core 계약으로 구현한다. 타일 지정/preview/CC와 현재 침식 caller·public query/작성·복원이 일치한다.
- User-Visible Problem: 현재 공격/피격/Rest가 공명을 추가하고 접수 시 남은 잔량을 start에서 재검사한 뒤 CastEnd에 소비한다. 단순 시간 lock은 combo·timedbuff·channel·independent projectile의 실제 사용 종료를 나타내지 못하고 빈 타일 접수를 막는다.
- User-Visible Completion Condition Set: V01–14: floor/fragment max+InitialSkillCharge, 직원 자동 충전/Rest제거, all-positive gate, valid/invalid/duplicate/Pause 접수, instant/projectile/combo/timedbuff/channel/creation/permanent 종료, CC/silence/target cancel, empty tile/focus/anchor, old corrosion 소비후start, official25+public typed fixture, query/reconnect/Continue 및 선행 계약.
- In-Scope / Out-of-Scope: DEFINITION/ADMISSION/LIFETIME/PROJECTION의 최소 atomic 전환과 VERIFY. 제외는 새 침식 피해(U19), 미래 actor/장비/유물/보스 기능, 미합의 최종보스·Unity UI·remote deployment, 독립 child 전체 생존 잠금, unrelated cleanup, 기존 저장 원본 자동삭제/변환, 임의 새로운 적 자동 주기,20분 시험.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_002 RQ01–07/TD01–11; old base/profile+consumable composition과 shared autonomous consumer를 분리, immediate consume credential, separate use owner/finite leases/flow future actions, employee pulse와 common gain gate, PlayerTile admission, actual kernel-derived query/clone와 strict data cutover.
- Goal-Local Source of Truth / Owning Layer: skill/fragment public authored definition과 compiler가 수명/max 의미를 소유하고 BattleCore의 동일 transaction/use identity가 validate·consume·start/end·gain을 소유한다. World/host는 승인 상태를 투영/게시하고 target/lock/gameplay를 재계산하지 않는다. U04는 durable pre-entry/confirmed/SavePoint를 계속 소유한다.
- Goal-Local Invariant Set / Source Refs: FPF-CRSL-001의14요소; consume-once/no refund/no credential reuse; all-positive HardCC/use gate; independent delivery와 actual use 별도; timedbuff 실제 종료; accepted anchor/no manual retarget; current corrosion timing; employee base+valid deploy effects; preserved opponent autonomous cadence; U01/U04/R04/R05 원본/권리 보존.
- Code/Data/Schema/Store/Transport/Consumer Boundary: raw SkillDef/fragment max→strict validated+compiled use descriptor→actual selected-copy profile→spawn/redeploy/pulse; public command transaction→credential→SkillStartGate/AutoCastStart/corrosion→cast/flow/buff/channel/creation→gain/query. Controlled clone/extent/same actor reconnect와 durable Continue/current schema를 함께 확인. TD01–11이 owner/order/failure/alternatives를 설명하며 GEX가 exact enum/state key/events/hooks와 validation 기대값을 구체화한다.
- Consumer Verification Boundary: workspace required CoreApplication normal command/query, authoritative kernel, public RON/catalog/authoring fixtures, actual host actor/stock websocket DTO, clone/Continue/file reader. Canonical-only Unity 화면과 제공되지 않은 remote deployed instance는 구현/검증 claim에서 제외한다. 실제 required local source150개 확인.
- Goal-Local Dependencies / Execution Prerequisites: U01와 U04 독립 CompleteHigh/closure 및 current367 baseline, exact Freeze/Plan/receipt/Goal/Handoff. GEX00에서 existing dirty와 baseline을 보존. DEFINITION→ADMISSION→LIFETIME→PROJECTION 내부 순서; 해당 통합체는 부분 독립 완료 아님. 구현 전 descriptor/current25와 typed capabilities, private credential identity, actual future-control/finite leases, queue order/clone/observer/native bound 준비 점검을 기록한다.
- Validation Plan / Completion Matrix: CODE_REVIEW Element Coverage01–14 및 integration mapping 전부를 actual nonzero state/trace로 닫는다. 성공·거부·취소·동일시각·Fault rollback·Pause/stale replay·early lease end·empty tile·actual enemy preservation·strict removed fields·host query/reconnect·U04 restore와 unsupported originals를 포함한다. 관련 full Core/host/release/native/observer와127 inherited/별도31R06 impact를 source-bound 결과로 고정한다.
- Structural Completion / Legacy and Fallback Disposition: employee attack/damage/Rest bonus와 end-reset/postlock, consumed full recheck, manual empty-tile usefulness blocker를 제거. actual opponent autonomous source는 명시 분리된 유효 계약으로 보존. single use owner와 descriptor를 current live/fixture/public path에 적용하고 미지원 조합/제거 필드를 silent default로 보정하지 않는다. test-only success branch나 UI rule clone·future unused abstraction 없음.
- Canonical Documentation Obligation: skill_target_contract.md,core_runtime_contract.md,data_loading_contract.md와 실제 host unity_bridge_protocol의 affected admission/use/gain/data/query 계약을 함께 갱신. game_rulebook current combat/resonance 관련 설명도 실제 owner 위치를 맞춘다. 과거 감사 원본은 불변이며 ROADMAP completion은 독립 감사 뒤 게시한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공통 baseline reuse 계약. 선행127조건과 별도31R06 각각 reuse/changed-intent/affected/new evidence를 남긴다. old gain/reset/castend assertions는 승인 정책 변경으로 설명하고 무관한 action facts/resources/native bounds/observer/fault/cancel/restore를 재검증한다. 값이 바뀌었다는 이유만으로 이전 실패를 숨기지 않는다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CRSL-V01–V14는 WP-CRSL-VERIFY/GOAL-CRSL-01 소유. 실제 public fixture/probe·edge execution은 구현자 책임이며 감사자나 미래 GEX 단계로 미루지 않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: raw/compiled/official/test data와 public consumer를 같은 cutover에 전환. affected persistent field가 있으면 strict reader/writer/unsupported 원본보호 동시 변경, 없으면 지원 형식 그대로. private live candidate rollback은 use/gain/event/source를 함께 복원; same actor 재접속과 새 process 미완료 pre-entry Continue를 구분. exact actual before/after source archive와 fuzz0 forward/reverse 복원 증거.
- Goal-Local Known Unknown / Blocker / Stop Condition: exact descriptor/enum/container/hook/pulse/fixture seed는 TD01–11 내 GEX 상세 선택. required policy/source gap0; 아직 구현 결과는 없다. 새 material 의미/필수 source/phase 오류는 가장 이른 P2G owner로 immutable successor 처리. 사용자 stop·승인 범위 완료·현 권한/환경으로 해결 불가 실제 장애에서만 중단한다.
- Applicable Risk / Reviewer Requirement and Evidence: 구현 비참여 GPT-6 Astra/High가14 acceptance와10dimensions/127 inherited/31R06, actual public data/host/currentness/restore를 독립 감사. consumed credential 재사용, last flow leaf vs future control, early buff removal/overlap, CC released payload 보존, old corrosion timing, employee/opponent schema 구별, native count/clone/observer 성능이 주요 위험. finding 교정/재감사 CompleteHigh 전 U06 시작 금지. 이 작성은 reviewer activation/GEX 실행 자체가 아니다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01–14 normal public lifecycle | 실제 consume/charge/end/query가 policy와 일치 |
| invalid_input | required | V04/07–10/12 stale target/kind/resource/removed RON | 거절은 공명/focus/playback/state revision 무변경; 정확한 diagnostics |
| failure_rollback | required | V04/06/11/13 controlled cancellation/runtime fault | 소비·자격·event·facts의 원자 rollback과 no-refund gameplay cancel 구분 |
| retry_idempotency | required | V04/09/11 duplicate input/event/reinput | one use/one consume; stale credential/event 재실행 없음 |
| ordering_concurrency | required | V02–07/11–13 same-time expiry/CC/pulse/host | 결정적 단일 owner 순서; overlapping locks를 독립 유지 |
| boundary_empty_max | required | V01/02/05/08/12 max1/4/5/99/100,empty tile/permanent | floor/cap/no bank,empty allowed vs invalid,permanent/independent child no infinite lock |
| disconnect_restart_persistence | required | V04/09/13 actual same actor/process/store | Pause reconnect pending 유지·재소비 없음; new process U04 pre-entry/SavePoint 보호 |
| live_data | required | V01/05/10/12 official25 and public typed RON | 실제 loader→execute 전종 descriptor 적용과 strict old-field errors |
| producer_consumer | required | V04/11–13 public Core/host/catalog/query | admission/credential/lifetime projection과 실제 소비자 왕복 일치 |
| legacy_fallback_absence | required | V01–04/08/12 source+negative execution | employee old gain/reset/lock/empty-tile fallback 없음; enemy explicit owner 보존 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRSL-01 | PER-CRSL-01-REVIEWED-002 | DR-CRSL-01-002 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-02 | PER-CRSL-02-REVIEWED-002 | DR-CRSL-02-002 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-03 | PER-CRSL-03-REVIEWED-002 | DR-CRSL-03-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-04 | PER-CRSL-04-REVIEWED-002 | DR-CRSL-04-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-05 | PER-CRSL-05-REVIEWED-002 | DR-CRSL-05-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-06 | PER-CRSL-06-REVIEWED-002 | DR-CRSL-06-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-07 | PER-CRSL-07-REVIEWED-002 | DR-CRSL-07-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-08 | PER-CRSL-08-REVIEWED-002 | DR-CRSL-08-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-09 | PER-CRSL-09-REVIEWED-002 | DR-CRSL-09-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-10 | PER-CRSL-10-REVIEWED-002 | DR-CRSL-10-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-11 | PER-CRSL-11-REVIEWED-002 | DR-CRSL-11-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-12 | PER-CRSL-12-REVIEWED-002 | DR-CRSL-12-002 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-13 | PER-CRSL-13-REVIEWED-002 | DR-CRSL-13-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
| PE-CRSL-14 | PER-CRSL-14-REVIEWED-002 | DR-CRSL-14-001 | PHASE-CRSL-01 | GOAL-CRSL-01 |
