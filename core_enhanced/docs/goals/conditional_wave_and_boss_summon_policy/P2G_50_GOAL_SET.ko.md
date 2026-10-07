# 조건부 Wave·보스 소환 P2G-50 Goal Set

상태: `P2G-50 complete / current Goal Set / Authored / Not Activated / Implementation Authorized: No`

이 문서는 current `PP-CWBS-P2G40-002`의 6개 ordered Phase를 재설계하지 않고 Master coordination member
1개와 Phase Goal member 6개로 materialize한 complete Goal Set이다. P2G-60 handoff, Goal activation,
production mutation과 구현은 수행하거나 승인하지 않았다.

## Stage Invocation and Entry

- User Request Ref: `USR-CWBS-013`
- Exact user statement: `그럼 50 으로 넘어가자.`
- Selected Stage: `P2G-50`
- Contract Basis: `core_enhanced/docs/policy_to_goal_pipeline.md`,
  `sha256:3ff93f532d186dcb6bf0d40ba6e9438d6a257ff7aa770c85c55510b37f379b63`
- Final Policy Freeze Ref: `FPF-CWBS-P2G30-001 / current`
- Phase Plan Ref: `PP-CWBS-P2G40-002 / current`
- Phase Plan Validation Receipt Ref: `PVR-CWBS-P2G40-002 / passed`
- Receipt validation: `p2g40-receipt-validator/1.0.0 / passed / failures 0`
- Goal Structure Fact: `multi_phase / 6 ordered Phases`
- Validation Baseline Reuse Directive: `VBRD-CWBS-001 / USR-CWBS-009..011`
- Correction predecessor: `N/A`
- Applicable Change/Obligation basis: `explicit empty`
- Entry Result: `passed`; 25 Freeze members, 7 Final Execution Nodes, 6 Phase assignments와 10 integration
  ownership rows가 current Plan/Receipt에서 complete

## Goal Set

- Goal Set Ref: `GS-CWBS-P2G50-001`
- Predecessor Goal Set Ref: `N/A`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CWBS-P2G20-001 + RB-CWBS-P2G20-001 + RSB-CWBS-P2G20-001 + FPF-CWBS-P2G30-001 + PP-CWBS-P2G40-002 + PVR-CWBS-P2G40-002 + VBRD-CWBS-001 + explicit-empty Change/Obligation set`
- Goal Structure Fact: `multi_phase`
- Phase Plan Ref: `PP-CWBS-P2G40-002`
- Final Policy Freeze Ref: `FPF-CWBS-P2G30-001`
- Complete Goal Member Set:
  `GOAL-CWBS-MASTER`, `GOAL-CWBS-P01-RUNTIME-FOUNDATION`, `GOAL-CWBS-P02-WAVE-CUTOVER`,
  `GOAL-CWBS-P03-SUMMON-CUTOVER`, `GOAL-CWBS-P04-OUTCOME-CUTOVER`,
  `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`, `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Applicable P2G-50 Obligation Key Set: `explicit empty`
- P2G-50 Resolution Record Ref Set: `explicit empty`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Authoring Status: `Authored`
- Activation Status: `Not Activated`
- Implementation Authorized: `No`

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` | `AG-CWBS-01-RUNTIME-FOUNDATION` | exactly one Goal |
| `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` | `AG-CWBS-02-WAVE-CUTOVER`, `WP-CWBS-03` | exactly one Goal |
| `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` | `AG-CWBS-03-SUMMON-CUTOVER` | exactly one Goal |
| `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` | `AG-CWBS-04-OUTCOME-CUTOVER` | exactly one Goal |
| `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | `GOAL-CWBS-P05-COMPENDIUM-CUTOVER` | `WP-CWBS-10` | exactly one Goal |
| `PHASE-CWBS-06-REFERENCE-INTEGRATION` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` | `WP-CWBS-11` | exactly one Goal |

## Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: `VBRD-CWBS-001 / USR-CWBS-009..011`
- Initial Full Baseline Owner / Timing: `GOAL-CWBS-P01-RUNTIME-FOUNDATION`, production mutation 전 정확히 한 번
- Initial Full Command Set:
  `CARGO_TARGET_DIR=/tmp/<goal-owned> cargo test -p game_core`와
  `CARGO_TARGET_DIR=/tmp/<goal-owned> cargo test -p game_server`
- Baseline Evidence Key: exact command/test selector, fixture/input identity, relevant production source/data,
  test/fixture/probe/config, Rust toolchain, `Cargo.lock`/dependency identity, non-zero execution result, owning Goal과
  execution target revision
- Reuse Rule: 위 relevant input identity와 evidence contract가 unchanged인 성공 evidence만 재사용
- Phase-Start Rule: Phase 시작 자체는 invalidation이 아니며 Phase마다 전수 command를 반복하지 않음
- Production Input Invalidation: production source/data/config 변경의 영향 map에 속하는 seam은 test 파일이
  unchanged여도 invalid; 해당 focused validation을 current target에서 실행해 다시 bind
- Test/Fixture/Probe Change: 변경된 seam evidence는 invalid; 변경된 검증을 current target에서 non-zero 실행해
  새 baseline 고정
- Shared Boundary Change: harness/schema/persistence/transport/dependency 변경은 아래 Phase별 impact map이 가리키는
  baseline set 전체를 invalid
- Unaffected Seam Rule: impact map 밖의 unchanged seam은 재실행하지 않음
- Final Full Baseline Owner / Timing: `GOAL-CWBS-P06-REFERENCE-INTEGRATION`, 모든 구현과 focused rebaseline 뒤
  current integrated target에서 위 full command set을 정확히 한 번 실행
- Retry Rule: 실패, zero execution 또는 실행 뒤 relevant input 변경은 성공 baseline이 아니며 필요한 재실행을
  금지하지 않음
- Normal Full-Validation Count: `initial 1 + final 1`; invalidation/failure retry는 별도
- Authoring-Time Execution: `none`; 이 계약은 후속 authorized execution의 evidence reuse를 정의할 뿐 P2G-50에서
  test를 실행하지 않음

## Master Goal Member — `GOAL-CWBS-MASTER`

- Goal ID: `GOAL-CWBS-MASTER`
- Goal Kind: `master`
- Complete Phase Goal ID Set: `GOAL-CWBS-P01-RUNTIME-FOUNDATION`, `GOAL-CWBS-P02-WAVE-CUTOVER`,
  `GOAL-CWBS-P03-SUMMON-CUTOVER`, `GOAL-CWBS-P04-OUTCOME-CUTOVER`,
  `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`, `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Derived Sequential/Parallel Relations: `P01 -> P02 -> P03 -> P04 -> P05 -> P06`; parallel execution
  `none approved`
- Coordination Dependencies/Conditions: predecessor Goal의 completion evidence와 해당 Intermediate-State Audit이
  유효해야 다음 Goal을 선택할 수 있으며 dependency를 사용자 승인만으로 우회하지 않음
- Assigned Phase / Final Execution Node / Implementation Work Ownership: `None`

### Derived Cross-Phase Invariants

| Invariant | Exact source | Coordination requirement |
| --- | --- | --- |
| spawn request는 all-or-none이고 같은-time terminal보다 먼저 정산 | `FPF-CWBS-P2G30-001: SP-01..02, WV-03`; `PP-CWBS-P2G40-002` | P01 Kernel을 P02/P03이 단일 consumer path로 사용하고 P04 terminal이 settlement 뒤 실행 |
| active-battle state는 reconnect에 유지되지만 durable SavePoint에는 저장하지 않음 | `PS-01`; `IVA-CWBS-07` | P01 authority와 P06 regression 모두 `RunCheckpointPayload` non-change를 보존 |
| Wave, summon과 encounter Boss authority/provenance를 혼합하지 않음 | `SM-01..04`, `BR-04..05`, `WV-05` | P02 actor adapter와 P03 summon domain이 authority-free consumer boundary를 유지 |
| named outcome 하나만 commit되고 node/reward/progression이 atomic/idempotent하게 소비 | `BL-01..04`; `IVA-CWBS-04..05` | P04가 direct winner, cap/Draw와 flat dual reward authority를 함께 제거 |
| compendium은 final Victory에서만 single registered set을 쓰며 legacy union migration 없음 | `BL-05`; `IVA-CWBS-06` | P05가 P04 committed outcome을 유일한 writer input으로 사용 |
| placeholder는 nonproduction이고 Unity, production balance/promotion은 범위 밖 | `RF-01`; `IVA-CWBS-08..09` | P06이 exclusion과 canonical alignment만 완료하고 production 승격을 주장하지 않음 |

### Integration Acceptance Coordination

| Acceptance | Disposition | Owning Goal / Final Execution Node | Locator or evidence contract |
| --- | --- | --- | --- |
| `IVA-CWBS-01` Kernel all-or-none/provenance | `assigned_execution_node` | P01 / `AG-CWBS-01-RUNTIME-FOUNDATION`; final P06 / `WP-CWBS-11` | generic Kernel fixture와 final Wave/summon cross-consumer fixture |
| `IVA-CWBS-02` strict Wave trigger/dependency/order/gate | `assigned_execution_node` | P02 / `AG-CWBS-02-WAVE-CUTOVER` | durable RON/compiler/runtime live fixtures |
| `IVA-CWBS-03` summon/BossTask/placement/AI/removal | `assigned_execution_node` | P03 / `AG-CWBS-03-SUMMON-CUTOVER` | durable Boss Runtime RON + battle fixture |
| `IVA-CWBS-04` full bucket then Defeat-first outcome | `assigned_execution_node` | P04 / `AG-CWBS-04-OUTCOME-CUTOVER` | deterministic simulation-time paired outcome fixture |
| `IVA-CWBS-05` atomic outcome consumption | `assigned_execution_node` | P04 / `AG-CWBS-04-OUTCOME-CUTOVER` | Core/server round-trip, duplicate consume/reconnect, exhaustive policy fixtures |
| `IVA-CWBS-06` Victory-only single compendium set | `assigned_execution_node` | P05 / `WP-CWBS-10` | Core snapshot/commit + server profile/reset fixtures |
| `IVA-CWBS-07` reconnect/restart/no active-battle SavePoint | `assigned_execution_node` | P06 / `WP-CWBS-11` | no-refire/reselect/reset + `RunCheckpointPayload` exclusion fixture |
| `IVA-CWBS-08` excluded nonproduction placeholder | `assigned_execution_node` | P06 / `WP-CWBS-11` | placeholder RON run + exclusion assertions |
| `IVA-CWBS-09` canonical docs/data alignment | `assigned_execution_node` | P06 / `WP-CWBS-11` | durable invariant test or repository audit with final loaders |
| `IVA-CWBS-10` repository-wide regression | `existing_read_only` | P06 executes final baseline; implementation node `N/A` | full `game_core` and `game_server` command set, non-zero/zero-failure evidence |

- Baseline Reuse Coordination: P01 initial baseline evidence를 각 Goal의 impact map으로 전달하고 invalidated seam만
  focused rebaseline; P06이 final full baseline을 소유. Master는 command를 실행하거나 test를 구현하지 않음.
- Overall Completion Conditions: P01..P06이 순서대로 complete이고 10 integration acceptance mapping이 current
  evidence를 가지며 P06 final full baseline이 성공하고 final canonical/data audit가 통과함
- Applicable Integration Risk / Reviewer Requirement: cross-Phase transaction, persistence, schema cutover와
  outcome authority review가 필요. reviewer 지정·승인과 Goal Gate 실행은 후속 pipeline 소유.

## Phase Goal Member — `GOAL-CWBS-P01-RUNTIME-FOUNDATION`

- Goal ID: `GOAL-CWBS-P01-RUNTIME-FOUNDATION`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-01-RUNTIME-FOUNDATION`
- Assigned Final Execution Node Set: `AG-CWBS-01-RUNTIME-FOUNDATION` (`WP-CWBS-01`, `WP-CWBS-02`)
- Covered Policy Element Revision Ref Set:
  `PER-CWBS-SP-01-CODE-003`, `PER-CWBS-SP-02-CODE-003`, `PER-CWBS-PS-01-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-SP-01-001`, `DR-CWBS-SP-02-001`, `DR-CWBS-PS-01-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 3개의 exact rows in
  `FPF-CWBS-P2G30-001#Observable-Policy-Acceptance-Set`; 변형·추가 없음
- Objective / Contract-Visible Result: Wave/summon 공용 atomic Spawn Kernel, deterministic placement/provenance,
  request/lifecycle ledger와 process-local reconnect authority를 제공하면서 SavePoint shape를 바꾸지 않음
- In-Scope: preflight→single commit→event emission, Kill/Despawn primitive, request/counter/batch/signal/outcome
  state의 active-battle authority와 reconnect projection
- Out-of-Scope: Wave/summon concrete schema consumer, terminal cutover, durable active-battle save,
  `RunCheckpointPayload` 변경, Unity와 production content
- Reviewed Current Behavior / Delta: current Wave member insert는 순차 mutation이고 generic ledger/despawn가 없지만
  `ActiveBattleSession` reconnect는 process-local authority이며 SavePoint는 active battle을 제외함;
  `RE-CWBS-SPAWN-001`, `RE-CWBS-PERSIST-001`, `CRA-CWBS-SP-01..02-001`, `CRA-CWBS-PS-01-001`
- Boundary: BattleCore spawn transaction/placement/lifecycle, ActiveBattleSession state, checkpoint DTO/event와
  server reconnect projection; durable SavePoint payload 제외
- Dependencies / Prerequisites: exact Goal Set basis와 initial full baseline success; predecessor Phase Goal 없음
- Inherited Baseline / Impact Map: production mutation 전 full baseline을 고정. Kernel, lifecycle, state,
  serialization/reconnect source 또는 관련 fixture 변경은 Kernel all-or-none/order/duplicate/occupied-cell,
  no-refire/reset과 SavePoint-shape focused seams만 invalid/rebaseline
- Assigned Durable Integration Seam: `IVA-CWBS-01`의 generic Kernel portion; `IVA-CWBS-07` final integration은 P06 소유
- Migration/Compatibility/Rollout/Rollback: SavePoint migration 없음; old Wave consumer는 P02 전까지 유효하게 유지;
  release rollout `N/A`
- Known Unknown / Blocker / Stop: atomic preflight와 commit 사이 competing writer를 단일 transaction으로 닫을 수
  없거나 battle state를 SavePoint에 넣어야만 한다면 중단하고 upstream premise 검토
- Risk / Reviewer: transaction/lifetime/serialization boundary review required; reviewer identity 지정은 미수행

### P01 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| all-or-none deterministic spawn | focused Kernel tests | partial mutation, duplicate settlement와 nondeterministic order 없음 |
| Kill/Despawn lifecycle 분리 | focused lifecycle/log tests | Despawn가 death/on-kill/reward를 만들지 않음 |
| reconnect authority / SavePoint non-change | Core/server focused serialization tests | same session no-refire; restart는 last SavePoint이며 active battle 없음 |
| valid intermediate state | Phase Plan P01 audit | old Wave behavior를 유지한 채 foundation이 독립 유효 |

## Phase Goal Member — `GOAL-CWBS-P02-WAVE-CUTOVER`

- Goal ID: `GOAL-CWBS-P02-WAVE-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-02-WAVE-CUTOVER`
- Assigned Final Execution Node Set: `AG-CWBS-02-WAVE-CUTOVER` (`WP-CWBS-04`, `WP-CWBS-05`) and `WP-CWBS-03`
- Covered Policy Element Revision Ref Set: `PER-CWBS-WV-01-CODE-003`, `PER-CWBS-WV-02-CODE-003`,
  `PER-CWBS-WV-03-CODE-003`, `PER-CWBS-WV-04-CODE-003`, `PER-CWBS-WV-05-CODE-003`,
  `PER-CWBS-WV-06-CODE-003`, `PER-CWBS-WV-07-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-WV-01-001`, `DR-CWBS-WV-02-001`, `DR-CWBS-WV-03-001`,
  `DR-CWBS-WV-04-001`, `DR-CWBS-WV-05-001`, `DR-CWBS-WV-06-001`, `DR-CWBS-WV-07-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 7개의 exact Freeze acceptance rows; 변형·추가 없음
- Objective / Contract-Visible Result: live Wave를 strict typed trigger/dependency/deadline schema와 one-shot runtime
  evaluator/Kernel consumer로 atomic cutover하고 encounter-independent Boss actor behavior adapter를 준비
- In-Scope: PVE RON schema/compiler/publication validation, live Wave rows, trigger/group/deadline/cancel/gate runtime,
  deterministic bucket order와 actor behavior adapter
- Out-of-Scope: summon domain/BossTask Summon, named outcome cutover, Unity, production balance/promotion
- Reviewed Current Behavior / Delta: current `PveWaveData`는 `time_ms`만 가지며 required default와 sequential insert,
  dependency/cycle/actor trigger가 없음; behavior runtime은 encounter에 중첩됨;
  `RE-CWBS-WAVE-001`, `RE-CWBS-SPAWN-001`, `RE-CWBS-BOSS-001`, `CRA-CWBS-WV-01..07-001`, `CRA-CWBS-BR-04-001`
- Boundary: PVE Wave data/compiler/live RON, simulation evaluator/group state, Spawn Kernel consumer, terminal settlement
  seam과 authority-free actor behavior adapter
- Dependencies / Prerequisites: `GOAL-CWBS-P01-RUNTIME-FOUNDATION` complete와 its audit/baseline evidence current
- Inherited Baseline / Impact Map: Wave schema/RON/compiler/runtime, event ordering, group lifecycle와 adapter changes는
  parse/publication, live load, trigger/order/gate/cancel 및 Little Red regression seams을 invalid/rebaseline;
  unrelated P01 evidence는 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-02`; P03 `IVA-CWBS-03`의 adapter prerequisite만 제공
- Migration/Compatibility/Rollout/Rollback: current time-only/default-dependent live rows를 strict schema와 함께
  cutover; legacy/default fallback 금지; production promotion `N/A`
- Known Unknown / Blocker / Stop: live row가 closed trigger set으로 표현 불가하거나 required reachability가
  성립하지 않으면 fallback을 만들지 않고 중단
- Risk / Reviewer: schema/runtime reader-writer cutover와 same-time event ordering review required

### P02 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| strict publication | malformed/default/cycle/unreachable fixtures + live RON load | invalid graph 거부, all live rows explicit |
| one-shot ordered Wave runtime | time/priority/threshold/group/deadline/cancel tests | no refire, primary/deadline cause와 terminal-before settlement 정확 |
| actor behavior adapter | Little Red/route/task/removal regression | encounter authority 누출 없이 기존 behavior 보존 |
| valid intermediate state | Phase Plan P02 audit | no legacy activation path; Wave 독립 완료 |

## Phase Goal Member — `GOAL-CWBS-P03-SUMMON-CUTOVER`

- Goal ID: `GOAL-CWBS-P03-SUMMON-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-03-SUMMON-CUTOVER`
- Assigned Final Execution Node Set: `AG-CWBS-03-SUMMON-CUTOVER` (`WP-CWBS-06`, `WP-CWBS-07`)
- Covered Policy Element Revision Ref Set: `PER-CWBS-SM-01-CODE-003`, `PER-CWBS-SM-02-CODE-003`,
  `PER-CWBS-SM-03-CODE-003`, `PER-CWBS-SM-04-CODE-003`, `PER-CWBS-BR-01-CODE-003`,
  `PER-CWBS-BR-02-CODE-003`, `PER-CWBS-BR-03-CODE-003`, `PER-CWBS-BR-04-CODE-003`,
  `PER-CWBS-BR-05-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-SM-01-001`, `DR-CWBS-SM-02-001`, `DR-CWBS-SM-03-001`,
  `DR-CWBS-SM-04-001`, `DR-CWBS-BR-01-001`, `DR-CWBS-BR-02-001`, `DR-CWBS-BR-03-001`,
  `DR-CWBS-BR-04-001`, `DR-CWBS-BR-05-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 9개의 exact Freeze acceptance rows; 변형·추가 없음
- Objective / Contract-Visible Result: Wave enemy와 분리된 summon definition/provenance/counter/batch domain과
  BossTask Summon의 deterministic placement/wait/result/signal/removal path를 완결
- In-Scope: summon catalog/scaling, per-summoner max_alive/max_total, batch UUID/handle/signal, placement candidates,
  bounded wait, Persist/Despawn/Kill, behavior adapter consumption
- Out-of-Scope: recursive summon, summon victory gate/reward/compendium/Boss encounter authority, named outcome,
  Unity와 production opt-in
- Reviewed Current Behavior / Delta: current unit catalog에는 SummonedUnit이 없고 BossTask에는 Summon/result/batch/
  signal/placement wait가 없으며 behavior runtime만 재사용 가능; `RE-CWBS-BOSS-001`, `RE-CWBS-SPAWN-001`,
  `RE-CWBS-PERSIST-001`, `CRA-CWBS-SM-01..04-001`, `CRA-CWBS-BR-01..05-001`
- Boundary: summon data/compiler, Kernel preflight/commit, ActiveBattle counters/batch/handle/signal, Boss Runtime task,
  battlefield placement와 removal lifecycle
- Dependencies / Prerequisites: P01 and P02 complete; P02 adapter current
- Inherited Baseline / Impact Map: summon schema/catalog/scaling, counters, Boss task/compiler, placement/wait,
  behavior/removal changes는 isolation/limit/atomic batch/four-result/handle/signal/lifecycle seams을 rebaseline;
  unchanged Wave validation은 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-03`
- Migration/Compatibility/Rollout/Rollback: new opt-in definition only; legacy unit/Wave data alias 없음;
  production content promotion `N/A`
- Known Unknown / Blocker / Stop: atomic batch와 counter commit, exact handle scope 또는 bounded same-time wait를 한
  transaction/lifetime으로 닫지 못하면 중단
- Risk / Reviewer: producer-consumer schema, placement determinism, lifecycle/attribution review required

### P03 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| summon domain isolation/scaling/limits | catalog/runtime focused tests | no inherited hidden stats, reward/compendium/authority leakage, success-only counters |
| BossTask four results and atomic placement | deterministic battle fixtures | Spawned/LimitReached/NoSpawnSpace/Canceled, no partial/eviction/fallback |
| exact batch handle/signal/removal | lifecycle/reconnect tests | exact scope, clear once, Kill/Despawn semantics와 cancel order 정확 |
| valid intermediate state | Phase Plan P03 audit | summon opt-in path 완결, old terminal still valid |

## Phase Goal Member — `GOAL-CWBS-P04-OUTCOME-CUTOVER`

- Goal ID: `GOAL-CWBS-P04-OUTCOME-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-04-OUTCOME-CUTOVER`
- Assigned Final Execution Node Set: `AG-CWBS-04-OUTCOME-CUTOVER` (`WP-CWBS-08`, `WP-CWBS-09`)
- Covered Policy Element Revision Ref Set: `PER-CWBS-BL-01-CODE-003`, `PER-CWBS-BL-02-CODE-003`,
  `PER-CWBS-BL-03-CODE-003`, `PER-CWBS-BL-04-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-BL-01-001`, `DR-CWBS-BL-02-001`, `DR-CWBS-BL-03-001`,
  `DR-CWBS-BL-04-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 4개의 exact Freeze acceptance rows; 변형·추가 없음
- Objective / Contract-Visible Result: same-time settlement 뒤 named terminal outcome 하나를 Defeat-first로 commit하고
  result/node/reward/progression/server/BossOmen이 outcome ID를 atomic/idempotent하게 소비
- In-Scope: terminal schema/compiler/evaluator, neutral producer signal, priority/tie-break, outcome state/latch,
  exhaustive node policy와 result/server/reward/progression consumer cutover
- Out-of-Scope: compendium schema replacement, new production encounter outcomes, Unity
- Reviewed Current Behavior / Delta: current runtime은 one WinCondition, direct winner producer, 60s system cap와
  implicit Draw/Opponent paths, outcome ID 없는 result와 alternate flat reward authorities를 사용;
  `RE-CWBS-TERMINAL-001`, `RE-CWBS-REWARD-001`, `CRA-CWBS-BL-01..04-001`
- Boundary: PVE terminal data/compiler, battle settlement/evaluator, committed outcome projection, node policy,
  transactional result/server/reward/progression/BossOmen consumers
- Dependencies / Prerequisites: P01..P03 complete; Wave/summon settlement signals current
- Inherited Baseline / Impact Map: terminal schema/evaluator/event order, result DTO/server, node/reward/progression와
  reconnect changes는 bucket/Defeat-first/tie-break/pause/Retreat/ID round-trip/no-repay/Exit Gate seams을 rebaseline
- Assigned Durable Integration Seam: `IVA-CWBS-04`, `IVA-CWBS-05`
- Migration/Compatibility/Rollout/Rollback: direct winner, system cap/implicit Draw와 flat dual reward authority를 한
  atomic cutover에서 제거; compatibility fallback 없음
- Known Unknown / Blocker / Stop: any live node outcome policy를 exhaustive하게 publish할 수 없거나 one-transaction
  consumption이 불가능하면 중단
- Risk / Reviewer: terminal authority와 reward/progression transaction review required

### P04 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| stable single outcome selection | paired same-time fixtures | full bucket 후 Defeat-first, priority/id tie-break, no cap/implicit Draw |
| atomic exhaustive consumption | Core/server/node/reward fixtures | outcome ID round-trip, exactly one policy, no dual reward/reselection/repay |
| Retreat/no-terminal continuation | scenario fixtures | authored terminal 전 계속, Retreat는 별도 결과 |
| valid intermediate state | Phase Plan P04 audit | all live consumers named outcome 사용; compendium old shape는 verdict seam만 소비 |

## Phase Goal Member — `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`

- Goal ID: `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-05-COMPENDIUM-CUTOVER`
- Assigned Final Execution Node Set: `WP-CWBS-10`
- Covered Policy Element Revision Ref Set: `PER-CWBS-BL-05-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-BL-05-001`
- Derived Observable Policy Acceptance Coverage: exact `PER-CWBS-BL-05-CODE-003` acceptance row in Freeze
- Objective / Contract-Visible Result: four discovered/defeated sets와 legacy union migration을 single registered set,
  empty new schema와 final Victory-only writer로 교체
- In-Scope: Core compendium candidate/snapshot/result writer, server profile schema/store/reset, outcome-bound commit
- Out-of-Scope: legacy profile union migration, Defeat/Retreat registration, Unity UI, production promotion
- Reviewed Current Behavior / Delta: current Core/server는 four sets, defeat-side discovery writer, snapshot two booleans와
  v1→v2 union migration을 사용; `RE-CWBS-COMPENDIUM-001`, `CRA-CWBS-BL-05-001`
- Boundary: battle attempt candidate collection, final Victory transaction, Core snapshot/new run, server profile
  serialization/reset
- Dependencies / Prerequisites: P04 complete and committed outcome seam current
- Inherited Baseline / Impact Map: compendium schema/writer/snapshot/store/reset changes는 Victory union,
  Defeat/Retreat unchanged, no legacy union, round-trip/reset seams을 rebaseline
- Assigned Durable Integration Seam: `IVA-CWBS-06`
- Migration/Compatibility/Rollout/Rollback: blank replacement schema; existing development profiles 재생성;
  reader/writer dual path와 union migration 금지
- Known Unknown / Blocker / Stop: any required consumer가 four-set shape를 계속 요구하면 alias를 두지 않고 중단
- Risk / Reviewer: schema/store atomic cutover and no-migration review required

### P05 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| Victory-only all actual candidates | Core battle/result tests | primary/death 무관 final Victory union, Defeat/Retreat unchanged |
| single new profile schema | Core/server round-trip/reset | one registered set, empty reset, no legacy union or four-set writer |
| valid intermediate state | Phase Plan P05 audit | Core/server authority 일치, SavePoint unaffected |

## Phase Goal Member — `GOAL-CWBS-P06-REFERENCE-INTEGRATION`

- Goal ID: `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-06-REFERENCE-INTEGRATION`
- Assigned Final Execution Node Set: `WP-CWBS-11`
- Covered Policy Element Revision Ref Set: `PER-CWBS-RF-01-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-RF-01-001`
- Derived Observable Policy Acceptance Coverage: exact `PER-CWBS-RF-01-CODE-003` acceptance row in Freeze
- Objective / Contract-Visible Result: excluded nonproduction placeholder로 representative vertical path를 증명하고
  all-phase integration, reconnect/no-SavePoint, canonical/data alignment와 final full baseline을 완료
- In-Scope: placeholder RON/reference IDs, required exclusion flags/registries, final durable integration fixtures,
  canonical docs/data audit, full Core/server validation
- Out-of-Scope: actual lore/balance/reward, normal BossOmen pool/cycle, production promotion, Unity,
  Released/Deployed rollout claim
- Reviewed Current Behavior / Delta: WhiteNight가 live BossOmen/final reward source이고 Little Red만 Boss Runtime에
  bind되며 separate excluded placeholder와 target canonical 설명이 없음;
  `RE-CWBS-REFERENCE-001`, `RE-CWBS-CANONICAL-001`, `CRA-CWBS-RF-01-001`
- Boundary: nonproduction PVE/Boss/omen/reward data, exclusion registries, final Core/server fixtures와 canonical docs
- Dependencies / Prerequisites: P01..P05 complete, all focused rebaseline evidence current
- Inherited Baseline / Impact Map: placeholder/exclusion/docs/integration harness changes는 corresponding reference,
  reconnect, cross-consumer와 canonical seams을 rebaseline; 그 뒤 current integrated target에서 final full command
  set을 실행하며 prior unaffected evidence를 final audit input으로 사용
- Assigned Durable Integration Seam: `IVA-CWBS-01` final Wave/summon portion, `IVA-CWBS-07`, `IVA-CWBS-08`,
  `IVA-CWBS-09`; `IVA-CWBS-10` existing read-only final runner execution owner
- Migration/Compatibility/Rollout/Rollback: placeholder is excluded/nonproduction; promotion requires separate content
  authoring/balance review; no release/deploy claim
- Known Unknown / Blocker / Stop: exclusion을 normal production registries에서 완전하게 증명할 수 없거나 final full
  validation이 zero/fail이면 complete하지 않음
- Risk / Reviewer: end-to-end state/persistence/data/consumer and exclusion/canonical review required

### P06 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| representative placeholder/exclusions | placeholder RON run + registry/reward/progression assertions | normal pool/cycle/reward/record/progression 유입 없음 |
| cross-phase runtime integrity | Wave/summon/terminal/compendium fixtures | Kernel provenance, settlement, outcome, registration chain 일치 |
| reconnect/no-SavePoint | Core/server fixture | same-session no replay; restart last SavePoint/no active battle; payload unchanged |
| canonical/data alignment | final repository audit | final loaders, docs와 implementation authority 일치 |
| final full baseline | full `game_core` + `game_server` commands | non-zero, zero failures, post-run relevant input unchanged |

## Goal Authoring Summary

| Check | Result |
| --- | --- |
| Goal Structure | `multi_phase`: Master 1 + Phase Goal 6 |
| Phase Mapping | 6/6 exactly once; missing/duplicate/extra 0 |
| Final Execution Node Mapping | 7/7 exactly once; Atomic Group split 0 |
| Freeze Primary Coverage | 25/25 PER and DR rows mapped; missing/duplicate/extra 0 |
| Integration Acceptance | 10/10 mapped to read-only seam or owning Goal/node |
| Baseline Directive | initial owner P01, final owner P06, Phase-start full rerun prohibited |
| Parallelization | `none approved` preserved |
| Activation / Authorization | `Not Activated / No` |

## Policy–Phase–Goal Traceability

각 acceptance는 표시된 exact PER가 소유한 `FPF-CWBS-P2G30-001#Observable-Policy-Acceptance-Set` row다.

| Stable Element | Policy Element Revision | Decision Revision | Primary Phase | Goal ID |
| --- | --- | --- | --- | --- |
| `PE-CWBS-SP-01` | `PER-CWBS-SP-01-CODE-003` | `DR-CWBS-SP-01-001` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-SP-02` | `PER-CWBS-SP-02-CODE-003` | `DR-CWBS-SP-02-001` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-PS-01` | `PER-CWBS-PS-01-CODE-003` | `DR-CWBS-PS-01-001` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-WV-01` | `PER-CWBS-WV-01-CODE-003` | `DR-CWBS-WV-01-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-02` | `PER-CWBS-WV-02-CODE-003` | `DR-CWBS-WV-02-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-03` | `PER-CWBS-WV-03-CODE-003` | `DR-CWBS-WV-03-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-04` | `PER-CWBS-WV-04-CODE-003` | `DR-CWBS-WV-04-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-05` | `PER-CWBS-WV-05-CODE-003` | `DR-CWBS-WV-05-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-06` | `PER-CWBS-WV-06-CODE-003` | `DR-CWBS-WV-06-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-07` | `PER-CWBS-WV-07-CODE-003` | `DR-CWBS-WV-07-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-SM-01` | `PER-CWBS-SM-01-CODE-003` | `DR-CWBS-SM-01-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-02` | `PER-CWBS-SM-02-CODE-003` | `DR-CWBS-SM-02-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-03` | `PER-CWBS-SM-03-CODE-003` | `DR-CWBS-SM-03-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-04` | `PER-CWBS-SM-04-CODE-003` | `DR-CWBS-SM-04-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-01` | `PER-CWBS-BR-01-CODE-003` | `DR-CWBS-BR-01-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-02` | `PER-CWBS-BR-02-CODE-003` | `DR-CWBS-BR-02-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-03` | `PER-CWBS-BR-03-CODE-003` | `DR-CWBS-BR-03-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-04` | `PER-CWBS-BR-04-CODE-003` | `DR-CWBS-BR-04-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-05` | `PER-CWBS-BR-05-CODE-003` | `DR-CWBS-BR-05-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BL-01` | `PER-CWBS-BL-01-CODE-003` | `DR-CWBS-BL-01-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-02` | `PER-CWBS-BL-02-CODE-003` | `DR-CWBS-BL-02-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-03` | `PER-CWBS-BL-03-CODE-003` | `DR-CWBS-BL-03-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-04` | `PER-CWBS-BL-04-CODE-003` | `DR-CWBS-BL-04-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-05` | `PER-CWBS-BL-05-CODE-003` | `DR-CWBS-BL-05-001` | `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | `GOAL-CWBS-P05-COMPENDIUM-CUTOVER` |
| `PE-CWBS-RF-01` | `PER-CWBS-RF-01-CODE-003` | `DR-CWBS-RF-01-001` | `PHASE-CWBS-06-REFERENCE-INTEGRATION` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` |

## P2G-50 Complete Stage Result

- Goal Set Ref: `GS-CWBS-P2G50-001`
- P2G-50 Resolution Record Ref Set: inherited `explicit empty`
- Applicable Obligation Supersession Record Ref Set: inherited `explicit empty`
- Complete Result Validation: `passed`
- Current Selection: `GS-CWBS-P2G50-001 / current`

## P2G-50 Exit Validation

1. `multi_phase` Goal Set은 Master 1개와 6 Phase Goal member를 포함하고 member ID가 모두 unique하다.
2. 6 ordered Phases와 7 Final Execution Nodes는 정확히 한 Phase Goal에 mapping되며 missing/duplicate/extra가 없다.
3. 25 Freeze members와 Decision/Acceptance coverage는 primary Phase를 통해 정확히 한 Phase Goal에 도달한다.
4. 각 Phase Goal의 scope, dependency, boundary, intermediate completion과 validation plan은
   `PP-CWBS-P2G40-002`를 재설계하지 않고 그대로 materialize한다.
5. 10 integration acceptance는 exact read-only seam 또는 owning Phase Goal/Final Execution Node에 mapping된다.
6. `VBRD-CWBS-001`은 common payload에 있고 initial full baseline owner는 P01, final owner는 P06이다. Phase 시작
   자체로 full validation을 반복하지 않으며 changed production/test/shared input은 impact map에 따라 invalid된다.
7. Master는 coordination만 소유하며 Phase, Final Execution Node, code/data change 또는 validation 실행을 소유하지 않는다.
8. Goal Set은 `Authored / Not Activated / Implementation Authorized: No`이고 P2G-60 handoff를 소유하지 않는다.
9. applicable Change, P2G-50 obligation, resolution과 supersession set은 `explicit empty`다.

Result: `is_p2g50_complete(GS-CWBS-P2G50-001) = true`.

다음 단계는 사용자의 별도 요청이 있을 때 P2G-60만 수행한다.
