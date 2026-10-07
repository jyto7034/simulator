# 조건부 Wave·보스 소환 P2G-50 Successor Goal Set 002

상태: `P2G-50 complete / current Goal Set / Authored / Not Activated / Implementation Authorized: No`

이 문서는 current `PP-CWBS-P2G40-003`의 6개 ordered Phase를 재설계하지 않고 Master coordination member
1개와 Phase Goal member 6개로 materialize한 complete successor Goal Set이다. stale predecessor
`GS-CWBS-P2G50-001`의 stable Goal ID는 동일 Phase 구조에 맞춰 유지한다. P2G-60 handoff, Goal activation,
production mutation과 구현은 수행하거나 승인하지 않았다.

## Stage Invocation and Entry

- User Request Ref: `USR-CWBS-041`
- Exact user statement: `다음 수행해줘.`
- Selected Stage: `P2G-50 successor repair`
- Contract Basis: `core_enhanced/docs/policy_to_goal_pipeline.md`,
  `sha256:7b86983112a3d87b56249ac32613c9b93e1e0daa50fed43ae9089926793d0746`
- Final Policy Freeze Ref: `FPF-CWBS-P2G30-002 / current`
- Phase Plan Ref: `PP-CWBS-P2G40-003 / current`
- Phase Plan Validation Receipt Ref: `PVR-CWBS-P2G40-003 / passed`
- Receipt validation: `p2g40-receipt-validator/1.0.0 / passed / failures 0`
- Exact upstream hashes: Freeze `4c94407a...`, Plan `ae06c903...`, Receipt `0fb94267...`
- Goal Structure Fact: `multi_phase / 6 ordered Phases`
- Validation Baseline Reuse Directive: `VBRD-CWBS-001 / USR-CWBS-009..011`
- Correction predecessor: `GS-CWBS-P2G50-001 / stale by COD-CWBS-001 / immutable history`
- Applicable Change/Declaration: `PFCR-CWBS-001 / COD-CWBS-001`
- Resolved prerequisite obligations: `O10, O20, O30, O40` via their effective `RR-CWBS-CHG001-*` records
- Applicable owner obligation: `(COD-CWBS-001, OBL-CWBS-001-P2G50) / unresolved at entry`
- Entry Result: `passed`; 25 Freeze members, 7 Final Execution Nodes, 6 Phase assignments와 12 integration
  ownership rows가 current Plan/Receipt에서 complete

## Goal Set

- Goal Set Ref: `GS-CWBS-P2G50-002`
- Predecessor Goal Set Ref: `GS-CWBS-P2G50-001 / stale; immutable history`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CWBS-P2G20-002 + RB-CWBS-P2G20-002 + RSB-CWBS-P2G20-002 + FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + PVR-CWBS-P2G40-003 + VBRD-CWBS-001 + Declaration Set {COD-CWBS-001}`
- Goal Structure Fact: `multi_phase`
- Phase Plan Ref: `PP-CWBS-P2G40-003`
- Final Policy Freeze Ref: `FPF-CWBS-P2G30-002`
- Complete Goal Member Set:
  `GOAL-CWBS-MASTER`, `GOAL-CWBS-P01-RUNTIME-FOUNDATION`, `GOAL-CWBS-P02-WAVE-CUTOVER`,
  `GOAL-CWBS-P03-SUMMON-CUTOVER`, `GOAL-CWBS-P04-OUTCOME-CUTOVER`,
  `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`, `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Applicable P2G-50 Obligation Key Set: `(COD-CWBS-001, OBL-CWBS-001-P2G50)`
- P2G-50 Resolution Record Ref Set: `RR-CWBS-CHG001-P2G50-001`
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
  유효해야 다음 Goal을 선택하며 dependency를 사용자 승인만으로 우회하지 않음
- Assigned Phase / Final Execution Node / Implementation Work Ownership: `None`

### Derived Cross-Phase Invariants

| Invariant | Exact source | Coordination requirement |
| --- | --- | --- |
| spawn request는 all-or-none/exact-once이며 registration/binding 후 정산되고 terminal보다 먼저 끝남 | `FPF-CWBS-P2G30-002: SP-01..02, WV-03, WV-05`; `PP-CWBS-P2G40-003` | P01 Kernel/ledger를 P02/P03이 단일 consumer path로 사용하고 P04 terminal은 producer settlement 뒤 실행 |
| active battle은 same-process reconnect에 Pause 상태로 유지되지만 process death/SavePoint에는 남지 않음 | `PS-01`; `IVA-CWBS-08..09` | P01이 auto-Pause/no-TTL/explicit Resume와 explicit-load usage를 소유하고 P06이 integrated restart를 재검증 |
| Wave, summon과 encounter Boss authority/provenance를 혼합하지 않고 summon removal은 unresolved commit보다 우선 | `SM-01..04`, `BR-03..05`, `WV-05` | P02 actor adapter와 P03 summon domain이 authority-free consumer/removal-first boundary를 유지 |
| named outcome 하나만 commit되고 node/reward/progression이 atomic/idempotent하게 소비 | `BL-01..04`; `IVA-CWBS-05..06` | P04가 direct winner, cap/Draw와 flat dual reward authority를 함께 제거 |
| spawn candidates는 attempt-local이고 final Victory만 single registered set에 commit | `BL-05`, `PS-01`; `IVA-CWBS-07` | P01 process-local lifetime과 P05 final transaction이 reconnect retain/process-death discard를 함께 보존 |
| placeholder는 nonproduction이고 Unity, production balance/promotion은 범위 밖 | `RF-01`; `IVA-CWBS-10..11` | P06이 exclusion과 canonical alignment만 완료하고 production 승격을 주장하지 않음 |

### Integration Acceptance Coordination

| Acceptance | Disposition | Owning Goal / Final Execution Node | Locator or evidence contract |
| --- | --- | --- | --- |
| `IVA-CWBS-01` shared Kernel atomicity/provenance/exact-once | `assigned_execution_node` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` / `AG-CWBS-01-RUNTIME-FOUNDATION`; `GOAL-CWBS-P06-REFERENCE-INTEGRATION` / `WP-CWBS-11` | generic Kernel fixture와 final Wave/summon cross-consumer fixture |
| `IVA-CWBS-02` trigger registration/binding/first evaluation/prospective death/restore | `assigned_execution_node` | `GOAL-CWBS-P02-WAVE-CUTOVER` / `AG-CWBS-02-WAVE-CUTOVER` | PVE compiler와 deterministic simulation observer fixtures |
| `IVA-CWBS-03` value occupancy and deterministic placement | `assigned_execution_node` | `GOAL-CWBS-P03-SUMMON-CUTOVER` / `AG-CWBS-03-SUMMON-CUTOVER` | publication-invalid matrix와 multi-occupant/cell-order fixtures |
| `IVA-CWBS-04` bounded waiter/space-before-timeout/removal-first | `assigned_execution_node` | `GOAL-CWBS-P03-SUMMON-CUTOVER` / `AG-CWBS-03-SUMMON-CUTOVER` | same-bucket multiple-request/removal fixture |
| `IVA-CWBS-05` full bucket then Defeat-first outcome | `assigned_execution_node` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` / `AG-CWBS-04-OUTCOME-CUTOVER` | deterministic paired outcome fixture |
| `IVA-CWBS-06` atomic outcome consumption | `assigned_execution_node` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` / `AG-CWBS-04-OUTCOME-CUTOVER` | Core/server round-trip, duplicate consume, exhaustive policy fixtures |
| `IVA-CWBS-07` attempt candidates/final-Victory single set | `assigned_execution_node` | `GOAL-CWBS-P05-COMPENDIUM-CUTOVER` / `WP-CWBS-10` | spawn identity, Core active state와 server profile/reset fixtures |
| `IVA-CWBS-08` disconnect auto-Pause/no TTL/explicit Resume | `assigned_execution_node` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` / `AG-CWBS-01-RUNTIME-FOUNDATION` | server detach/attach/tick/timeout와 Core pause fixtures |
| `IVA-CWBS-09` process death last SavePoint/explicit-load-only usage | `assigned_execution_node` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` / `AG-CWBS-01-RUNTIME-FOUNDATION`; `GOAL-CWBS-P06-REFERENCE-INTEGRATION` / `WP-CWBS-11` | active-run/profile store restart/load fixtures |
| `IVA-CWBS-10` excluded nonproduction placeholder | `assigned_execution_node` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` / `WP-CWBS-11` | placeholder RON run와 exclusion assertions |
| `IVA-CWBS-11` canonical docs/data alignment | `assigned_execution_node` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` / `WP-CWBS-11` | durable invariant test 또는 final loader audit |
| `IVA-CWBS-12` repository-wide regression | `existing_read_only` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION`; implementation node `N/A` | full `game_core`/`game_server` command set, non-zero/zero-failure evidence |

- Baseline Reuse Coordination: P01 initial baseline evidence를 각 Goal의 impact map으로 전달하고 invalidated seam만
  focused rebaseline한다. P06이 final full baseline을 소유하며 Master는 command 실행/test 구현을 소유하지 않음.
- Overall Completion Conditions: P01..P06이 순서대로 complete이고 12 integration acceptance mapping이 current
  evidence를 가지며 P06 final full baseline과 canonical/data audit가 성공함
- Applicable Integration Risk / Reviewer Requirement: cross-Phase transaction, persistence, schema cutover,
  session lifetime와 outcome authority review가 필요. reviewer 지정·승인과 Goal Gate 실행은 후속 pipeline 소유.

## Phase Goal Member — `GOAL-CWBS-P01-RUNTIME-FOUNDATION`

- Goal ID: `GOAL-CWBS-P01-RUNTIME-FOUNDATION`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-01-RUNTIME-FOUNDATION`
- Assigned Final Execution Node Set: `AG-CWBS-01-RUNTIME-FOUNDATION` (`WP-CWBS-01`, `WP-CWBS-02`)
- Covered Policy Element Revision Ref Set:
  `PER-CWBS-SP-01-CODE-006`, `PER-CWBS-SP-02-CODE-006`, `PER-CWBS-PS-01-CODE-006`
- Covered Decision Revision Ref Set: `DR-CWBS-SP-01-002`, `DR-CWBS-SP-02-002`, `DR-CWBS-PS-01-002`
- Derived Observable Policy Acceptance Coverage: 위 PER 3개의 exact rows in
  `FPF-CWBS-P2G30-002#Observable-Policy-Acceptance-Set`; 변형·추가 없음
- Objective / Contract-Visible Result: Wave/summon 공용 atomic Kernel과 trigger/request/candidate ledger를 제공하고
  disconnect auto-Pause, timeoutless same-process session, snapshot 후 explicit Resume, process-death rollback을 완결
- In-Scope: preflight/commit/event ordering, request identity, value occupancy input, registration/removal primitive,
  active-battle ledgers, server detach/attach/tick/timeout와 load-use boundary
- Out-of-Scope: concrete Wave/summon consumer, permanent compendium commit, terminal cutover, durable active-battle save,
  `RunCheckpointPayload` 전투 확장, Unity와 production content
- Reviewed Current Behavior / Delta: current Wave insert는 sequential이고 target Kernel/ledger가 없으며 detach는 Core
  pause 없이 tick만 멈추고 300초 TTL 뒤 actor를 폐기한다. active battle은 process memory이고 SavePoint에서 제외됨;
  `RE-CWBS-SPAWN-002`, `RE-CWBS-SESSION-002`, `RE-CWBS-SAVEPOINT-002`, `CRA-CWBS-SP-01/02/PS-01-002`
- Boundary: BattleCore transaction/placement/lifecycle, ActiveBattleSession ledgers, server actor/session transport와
  active-run/profile store load boundary; durable SavePoint payload 제외
- Dependencies / Prerequisites: exact Goal Set basis와 initial full baseline success; predecessor Phase Goal 없음
- Inherited Baseline / Impact Map: production mutation 전 full baseline 고정. Kernel/state/session/transport/store 또는
  관련 fixture 변경은 atomicity/order/occupancy/removal, auto-Pause/no-TTL/reconnect/load-use/SavePoint seams만
  invalid/rebaseline하고 unaffected baseline은 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-01` generic portion, `IVA-CWBS-08`, `IVA-CWBS-09` Phase-local portion
- Migration/Compatibility/Rollout/Rollback: active battle/ledger의 SavePoint migration 없음; old Wave/compendium
  consumer는 후속 cutover까지 유효하게 유지; release rollout `N/A`
- Known Unknown / Blocker / Stop: Kernel commit과 active state writer를 한 transaction으로 닫을 수 없거나 auto-Pause를
  client 명령/durable battle save에 의존해야 하면 중단하고 upstream premise 검토
- Risk / Reviewer: transaction, session lifetime, transport, persistence boundary review required; reviewer 지정 미수행

### P01 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| atomic exact-once Kernel/occupancy primitive | focused Kernel tests | partial/duplicate/nondeterministic commit, eviction과 invalid occupancy input 없음 |
| trigger/removal/candidate process-local ledger | Core state/lifecycle tests | restore가 registration이 아니고 removal-first 및 reconnect retention 성립 |
| auto-Pause/no TTL/explicit Resume | Core/server detach-attach tests | disconnected clock/settlement 정지, actor 보존, snapshot 뒤 수동 Resume 전 진행 없음 |
| process death/explicit load use | restart/store/load tests | last SavePoint rollback, payload에 battle 없음, successful explicit load만 usage 증가 |
| valid intermediate state | Phase Plan P01 audit | old consumers와 dormant candidate seam을 유지한 채 foundation 독립 유효 |

## Phase Goal Member — `GOAL-CWBS-P02-WAVE-CUTOVER`

- Goal ID: `GOAL-CWBS-P02-WAVE-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-02-WAVE-CUTOVER`
- Assigned Final Execution Node Set: `AG-CWBS-02-WAVE-CUTOVER` (`WP-CWBS-04`, `WP-CWBS-05`) and `WP-CWBS-03`
- Covered Policy Element Revision Ref Set: `PER-CWBS-WV-01-CODE-003`, `PER-CWBS-WV-02-CODE-003`,
  `PER-CWBS-WV-03-CODE-003`, `PER-CWBS-WV-04-CODE-003`, `PER-CWBS-WV-05-CODE-006`,
  `PER-CWBS-WV-06-CODE-003`, `PER-CWBS-WV-07-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-WV-01-001`, `DR-CWBS-WV-02-001`, `DR-CWBS-WV-03-001`,
  `DR-CWBS-WV-04-001`, `DR-CWBS-WV-05-002`, `DR-CWBS-WV-06-001`, `DR-CWBS-WV-07-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 7개의 exact Freeze acceptance rows; 변형·추가 없음
- Objective / Contract-Visible Result: live Wave를 strict trigger/dependency/deadline/actor-slot schema와 one-shot
  binding-aware evaluator/Kernel consumer로 atomic cutover하고 authority-free Boss actor adapter를 준비
- In-Scope: PVE RON/compiler/publication, live rows, registration/binding epoch, HP/point first evaluation,
  prospective actual-death observer, group/deadline/cancel/gate runtime와 actor adapter
- Out-of-Scope: summon domain/BossTask Summon, named outcome cutover, Unity, production promotion
- Reviewed Current Behavior / Delta: scenario Wave는 battle-start/time만 지원하고 Boss defeated는 bind 후
  inactive/absent level fact를 사용한다. generic conditional Wave observer와 target binding epoch가 없음;
  `RE-CWBS-TRIGGER-002`, predecessor Wave evidence, `CRA-CWBS-WV-01..07-002`
- Boundary: PVE schema/compiler/live RON, trigger registry/evaluator, actor-slot binding/death observation, group state,
  Kernel consumer, terminal settlement seam와 behavior adapter
- Dependencies / Prerequisites: P01 complete와 audit/baseline evidence current
- Inherited Baseline / Impact Map: Wave schema/RON/compiler/runtime, binding observer, event order와 adapter changes는
  publication/live-load/already-true/pre-bind-death/restore/order/gate/cancel/Little Red seams을 rebaseline;
  unrelated P01 evidence는 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-02`; P03 adapter prerequisite 제공
- Migration/Compatibility/Rollout/Rollback: live rows를 strict schema와 함께 cutover; legacy/default/retroactive death
  fallback 금지; production promotion `N/A`
- Known Unknown / Blocker / Stop: dependency graph가 closed trigger set으로 표현 불가능하거나 prospective observer를
  authoritative death event에 결속할 수 없으면 fallback 없이 중단
- Risk / Reviewer: schema/runtime reader-writer, binding epoch와 same-time event ordering review required

### P02 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| strict publication/live cutover | malformed/default/cycle/source/deadline fixtures + live load | invalid graph 거부, live rows explicit, no legacy path |
| binding first evaluation | HP/point already-true fixtures | bind boundary에 정확히 1회 충족 |
| prospective defeated/restore | pre-bind death, despawn, reconnect fixtures | 과거 death/despawn/restore가 defeated/refire를 만들지 않음 |
| ordered Wave/gate | time/group/deadline/priority/required tests | full bucket settlement와 terminal ordering 정확 |
| valid intermediate state | Phase Plan P02 audit | Wave 독립 완료, adapter는 concrete P03 prerequisite |

## Phase Goal Member — `GOAL-CWBS-P03-SUMMON-CUTOVER`

- Goal ID: `GOAL-CWBS-P03-SUMMON-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-03-SUMMON-CUTOVER`
- Assigned Final Execution Node Set: `AG-CWBS-03-SUMMON-CUTOVER` (`WP-CWBS-06`, `WP-CWBS-07`)
- Covered Policy Element Revision Ref Set: `PER-CWBS-SM-01-CODE-003`, `PER-CWBS-SM-02-CODE-003`,
  `PER-CWBS-SM-03-CODE-003`, `PER-CWBS-SM-04-CODE-006`, `PER-CWBS-BR-01-CODE-003`,
  `PER-CWBS-BR-02-CODE-003`, `PER-CWBS-BR-03-CODE-006`, `PER-CWBS-BR-04-CODE-003`,
  `PER-CWBS-BR-05-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-SM-01-001`, `DR-CWBS-SM-02-001`, `DR-CWBS-SM-03-001`,
  `DR-CWBS-SM-04-002`, `DR-CWBS-BR-01-001`, `DR-CWBS-BR-02-001`, `DR-CWBS-BR-03-002`,
  `DR-CWBS-BR-04-001`, `DR-CWBS-BR-05-001`
- Derived Observable Policy Acceptance Coverage: 위 PER 9개의 exact Freeze acceptance rows; 변형·추가 없음
- Objective / Contract-Visible Result: summon definition/provenance/counter/batch domain과 BossTask Summon의 closed
  occupancy, deterministic placement, bounded arbitration, result/signal 및 removal-first lifetime path를 완결
- In-Scope: summon catalog/scaling, per-summoner counters, batch UUID/handle/signal, Around/Zone candidates,
  occupancy-set validation, bounded wait, Persist/Despawn/Kill와 actor adapter consumption
- Out-of-Scope: recursive summon, summon victory gate/reward/compendium/Boss authority, reachability inference,
  Unity와 production opt-in
- Reviewed Current Behavior / Delta: scoped repository에 summon task/domain/occupancy/waiter/counter가 없고 Boss
  task/result enum도 summon을 지원하지 않는다. Wave walkability와 actor behavior seam만 재사용 가능;
  `RE-CWBS-SUMMON-002`, `RE-CWBS-SPAWN-002`, `CRA-CWBS-SM-01..04-002`, `CRA-CWBS-BR-01..05-002`
- Boundary: summon data/compiler, Kernel preflight/commit, active counters/batch/handle/signal, Boss task executor,
  battlefield placement/wait arbitration와 removal lifecycle
- Dependencies / Prerequisites: P01 and P02 complete; P02 adapter current
- Inherited Baseline / Impact Map: summon schema/catalog, counters, Boss task/compiler, occupancy/placement/wait와
  removal changes는 isolation/limit/atomic batch/four-result/invalid-set/multi-waiter/signal/lifecycle seams을 rebaseline;
  unchanged Wave evidence는 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-03`, `IVA-CWBS-04`
- Migration/Compatibility/Rollout/Rollback: new opt-in definitions only; legacy aliases/implicit placement/reachability
  fallback 없음; production promotion `N/A`
- Known Unknown / Blocker / Stop: atomic batch/counter, all-occupant preflight, exact handle 또는 removal-first를 한
  settlement boundary로 닫지 못하면 중단. 고립 가능성은 정책대로 content author가 관리
- Risk / Reviewer: schema, placement determinism, waiter arbitration, lifecycle/attribution review required

### P03 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| summon isolation/scaling/limits | catalog/runtime tests | hidden inheritance/authority/reward leakage 없음; success-only counters |
| occupancy/atomic placement | publication matrix + battle fixtures | invalid set 거부, all occupants allowed, distinct cells, no partial/eviction/fallback |
| bounded waiter arbitration | multi-request same-time fixtures | wait-start/FIFO-ID, space-before-timeout, cancel/limit recheck 정확 |
| removal-first/lifetime/batch | death/withdrawal/despawn/state-exit/reconnect tests | unresolved Canceled에 UUID/usage 없음; prior commit만 Persist/Despawn; clear once |
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
- Objective / Contract-Visible Result: producer settlement 뒤 named terminal outcome 하나를 Defeat-first로 commit하고
  result/node/reward/progression/server/BossOmen이 outcome ID를 atomic/idempotent하게 소비
- In-Scope: terminal schema/compiler/evaluator, neutral producer signal, priority/tie-break, pending/committed outcome,
  exhaustive node policy와 result/server/reward/progression consumer cutover
- Out-of-Scope: compendium schema replacement, new production outcomes, Unity
- Reviewed Current Behavior / Delta: current runtime은 direct winner와 implicit terminal paths, outcome ID 없는 result,
  alternate reward authorities를 사용하며 target named settlement가 없음; predecessor terminal/reward evidence와
  `CRA-CWBS-BL-01..04-002`
- Boundary: terminal data/compiler, battle settlement/evaluator, outcome projection, node policy와 transactional
  result/server/reward/progression/BossOmen consumers
- Dependencies / Prerequisites: P01..P03 complete; Wave/summon settlement signals current
- Inherited Baseline / Impact Map: terminal schema/evaluator/order, result DTO/server, node/reward/progression changes는
  Defeat-first/tie-break/pause/Retreat/ID round-trip/no-repay/Exit Gate seams을 rebaseline
- Assigned Durable Integration Seam: `IVA-CWBS-05`, `IVA-CWBS-06`
- Migration/Compatibility/Rollout/Rollback: direct winner/system cap/implicit Draw와 dual reward authority를 한 atomic
  cutover에서 제거; compatibility fallback 없음
- Known Unknown / Blocker / Stop: live node policy를 exhaustive하게 publish할 수 없거나 single transaction consumption이
  불가능하면 중단
- Risk / Reviewer: terminal authority, same-bucket ordering와 reward/progression transaction review required

### P04 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| stable single outcome | paired same-time fixtures | full bucket 뒤 Defeat-first, priority/ID tie-break, no cap/implicit Draw |
| atomic exhaustive consumption | Core/server/node/reward fixtures | outcome ID round-trip, exactly one policy, no reselect/repay/dual writer |
| Retreat/no-terminal continuation | scenario fixtures | authored terminal 전 계속, Retreat 별도 결과 |
| valid intermediate state | Phase Plan P04 audit | all live consumers named outcome 사용; old compendium writer는 P05까지 유지 |

## Phase Goal Member — `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`

- Goal ID: `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-05-COMPENDIUM-CUTOVER`
- Assigned Final Execution Node Set: `WP-CWBS-10`
- Covered Policy Element Revision Ref Set: `PER-CWBS-BL-05-CODE-006`
- Covered Decision Revision Ref Set: `DR-CWBS-BL-05-002`
- Derived Observable Policy Acceptance Coverage: exact `PER-CWBS-BL-05-CODE-006` acceptance row in Freeze
- Objective / Contract-Visible Result: successful opponent spawn candidates를 process-local attempt ledger에 모으고
  final Victory에서만 canonical single registered set에 atomic/idempotent commit
- In-Scope: spawn identity→candidate ledger, reconnect retention/process-death discard, outcome-bound commit, Core/server
  profile schema/store/reset
- Out-of-Scope: legacy union migration, Defeat/Retreat registration, SavePoint battle persistence, Unity UI와 production promotion
- Reviewed Current Behavior / Delta: current four-set compendium은 consumed result spawn log에서 candidates를 재구성하고
  일부 non-Victory path도 discovery를 commit하며 v1→v2 union migration이 존재; `RE-CWBS-COMPENDIUM-002`,
  `RE-CWBS-SAVEPOINT-002`, `CRA-CWBS-BL-05-002`
- Boundary: successful spawn commit, ActiveBattleSession ordered candidate set, final outcome transaction, Core snapshot/new
  run와 server single-set serialization/reset
- Dependencies / Prerequisites: P01 ledger와 P04 committed outcome seam complete/current
- Inherited Baseline / Impact Map: candidate writer/lifetime, compendium schema/result/snapshot/store/reset changes는
  death/despawn/reconnect retention, Victory all-candidate commit, Defeat/Retreat/process-death discard, no-union,
  round-trip/reset seams을 rebaseline
- Assigned Durable Integration Seam: `IVA-CWBS-07`
- Migration/Compatibility/Rollout/Rollback: blank replacement schema와 development profile 재생성; dual path/union 금지
- Known Unknown / Blocker / Stop: required consumer가 four-set shape를 계속 요구하거나 candidate를 durable payload에
  넣어야만 한다면 alias 없이 중단
- Risk / Reviewer: active-state/profile transaction, schema/store no-migration review required

### P05 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| attempt candidate lifetime | spawn/death/despawn/reconnect/restart tests | reconnect retain, Defeat/Retreat/process death discard, SavePoint exclusion |
| final Victory all-candidate commit | Core battle/result tests | death/primary/outcome credit 무관 전체 atomic/idempotent 등록 |
| single new profile schema | Core/server round-trip/reset | one registered set, empty reset, no union/four-set writer |
| valid intermediate state | Phase Plan P05 audit | Core/server authority 일치, SavePoint unaffected |

## Phase Goal Member — `GOAL-CWBS-P06-REFERENCE-INTEGRATION`

- Goal ID: `GOAL-CWBS-P06-REFERENCE-INTEGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CWBS-06-REFERENCE-INTEGRATION`
- Assigned Final Execution Node Set: `WP-CWBS-11`
- Covered Policy Element Revision Ref Set: `PER-CWBS-RF-01-CODE-003`
- Covered Decision Revision Ref Set: `DR-CWBS-RF-01-001`
- Derived Observable Policy Acceptance Coverage: exact `PER-CWBS-RF-01-CODE-003` acceptance row in Freeze
- Objective / Contract-Visible Result: excluded nonproduction placeholder로 representative vertical path를 증명하고
  all-phase integration, session/process rollback, canonical/data alignment와 final full baseline을 완료
- In-Scope: placeholder RON/reference IDs, exclusion flags/registries, final durable integration fixtures,
  canonical docs/data audit와 full Core/server validation
- Out-of-Scope: lore/balance/reward 확정, normal BossOmen pool/cycle, production promotion, Unity와 rollout claim
- Reviewed Current Behavior / Delta: WhiteNight가 live BossOmen/final reward source이고 Little Red만 Boss Runtime에
  bind되며 target excluded placeholder/canonical 설명이 없음; predecessor reference/canonical evidence와
  `CRA-CWBS-RF-01-002`
- Boundary: nonproduction PVE/Boss/omen/reward data, exclusion registries, final Core/server fixtures와 canonical docs
- Dependencies / Prerequisites: P01..P05 complete, all focused rebaseline evidence current
- Inherited Baseline / Impact Map: placeholder/exclusion/docs/integration harness changes는 reference/session/
  cross-consumer/canonical seams을 rebaseline한 뒤 current integrated target에서 final full command set을 실행;
  unaffected evidence는 final audit input으로 재사용
- Assigned Durable Integration Seam: `IVA-CWBS-01` final portion, `IVA-CWBS-09` integrated restart,
  `IVA-CWBS-10`, `IVA-CWBS-11`; `IVA-CWBS-12` existing read-only runner execution owner
- Migration/Compatibility/Rollout/Rollback: placeholder excluded/nonproduction; promotion은 별도 authoring/balance review;
  no release/deploy claim
- Known Unknown / Blocker / Stop: exclusions를 production registries에서 완전히 증명할 수 없거나 final full validation이
  zero/fail이면 complete하지 않음
- Risk / Reviewer: end-to-end transaction/state/persistence/data/consumer와 exclusion/canonical review required

### P06 Completion Matrix

| Acceptance | Validation | Completion condition |
| --- | --- | --- |
| representative placeholder/exclusions | placeholder run + registry/reward/progression assertions | normal pool/cycle/reward/record/progression 유입 없음 |
| cross-phase runtime integrity | Wave/summon/outcome/compendium fixtures | binding, Kernel, removal, settlement, outcome, registration chain 일치 |
| session/process rollback | Core/server fixture | disconnect paused/no timeout; reconnect explicit Resume; restart last SavePoint/no active battle |
| canonical/data alignment | final repository audit | final loaders, docs와 implementation authority 일치 |
| final full baseline | full `game_core` + `game_server` commands | non-zero, zero failures, post-run relevant input unchanged |

## Goal Authoring Summary

| Check | Result |
| --- | --- |
| Goal Structure | `multi_phase`: Master 1 + Phase Goal 6 |
| Phase Mapping | 6/6 exactly once; missing/duplicate/extra 0 |
| Final Execution Node Mapping | 7/7 exactly once; Atomic Group split 0 |
| Freeze Primary Coverage | 25/25 PER and DR rows mapped; missing/duplicate/extra 0 |
| Integration Acceptance | 12/12 mapped to read-only seam or owning Goal/node |
| Baseline Directive | initial owner P01, final owner P06, Phase-start full rerun prohibited |
| Parallelization | `none approved` preserved |
| Activation / Authorization | `Not Activated / No` |

## Policy–Phase–Goal Traceability

각 acceptance는 표시된 exact PER가 소유한 `FPF-CWBS-P2G30-002#Observable-Policy-Acceptance-Set` row다.

| Stable Element | Policy Element Revision | Decision Revision | Primary Phase | Goal ID |
| --- | --- | --- | --- | --- |
| `PE-CWBS-SP-01` | `PER-CWBS-SP-01-CODE-006` | `DR-CWBS-SP-01-002` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-SP-02` | `PER-CWBS-SP-02-CODE-006` | `DR-CWBS-SP-02-002` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-PS-01` | `PER-CWBS-PS-01-CODE-006` | `DR-CWBS-PS-01-002` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `GOAL-CWBS-P01-RUNTIME-FOUNDATION` |
| `PE-CWBS-WV-01` | `PER-CWBS-WV-01-CODE-003` | `DR-CWBS-WV-01-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-02` | `PER-CWBS-WV-02-CODE-003` | `DR-CWBS-WV-02-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-03` | `PER-CWBS-WV-03-CODE-003` | `DR-CWBS-WV-03-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-04` | `PER-CWBS-WV-04-CODE-003` | `DR-CWBS-WV-04-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-05` | `PER-CWBS-WV-05-CODE-006` | `DR-CWBS-WV-05-002` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-06` | `PER-CWBS-WV-06-CODE-003` | `DR-CWBS-WV-06-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-WV-07` | `PER-CWBS-WV-07-CODE-003` | `DR-CWBS-WV-07-001` | `PHASE-CWBS-02-WAVE-CUTOVER` | `GOAL-CWBS-P02-WAVE-CUTOVER` |
| `PE-CWBS-SM-01` | `PER-CWBS-SM-01-CODE-003` | `DR-CWBS-SM-01-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-02` | `PER-CWBS-SM-02-CODE-003` | `DR-CWBS-SM-02-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-03` | `PER-CWBS-SM-03-CODE-003` | `DR-CWBS-SM-03-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-SM-04` | `PER-CWBS-SM-04-CODE-006` | `DR-CWBS-SM-04-002` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-01` | `PER-CWBS-BR-01-CODE-003` | `DR-CWBS-BR-01-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-02` | `PER-CWBS-BR-02-CODE-003` | `DR-CWBS-BR-02-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-03` | `PER-CWBS-BR-03-CODE-006` | `DR-CWBS-BR-03-002` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-04` | `PER-CWBS-BR-04-CODE-003` | `DR-CWBS-BR-04-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BR-05` | `PER-CWBS-BR-05-CODE-003` | `DR-CWBS-BR-05-001` | `PHASE-CWBS-03-SUMMON-CUTOVER` | `GOAL-CWBS-P03-SUMMON-CUTOVER` |
| `PE-CWBS-BL-01` | `PER-CWBS-BL-01-CODE-003` | `DR-CWBS-BL-01-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-02` | `PER-CWBS-BL-02-CODE-003` | `DR-CWBS-BL-02-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-03` | `PER-CWBS-BL-03-CODE-003` | `DR-CWBS-BL-03-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-04` | `PER-CWBS-BL-04-CODE-003` | `DR-CWBS-BL-04-001` | `PHASE-CWBS-04-OUTCOME-CUTOVER` | `GOAL-CWBS-P04-OUTCOME-CUTOVER` |
| `PE-CWBS-BL-05` | `PER-CWBS-BL-05-CODE-006` | `DR-CWBS-BL-05-002` | `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | `GOAL-CWBS-P05-COMPENDIUM-CUTOVER` |
| `PE-CWBS-RF-01` | `PER-CWBS-RF-01-CODE-003` | `DR-CWBS-RF-01-001` | `PHASE-CWBS-06-REFERENCE-INTEGRATION` | `GOAL-CWBS-P06-REFERENCE-INTEGRATION` |

## P2G-50 Resolution Record — `RR-CWBS-CHG001-P2G50-001`

- Resolution Record Ref: `RR-CWBS-CHG001-P2G50-001`
- Obligation Key: `(COD-CWBS-001, OBL-CWBS-001-P2G50)`
- Owner Stage: `P2G-50`
- Resolved prerequisite basis:
  `FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + PVR-CWBS-P2G40-003 + RR-CWBS-CHG001-P2G10-001 + RR-CWBS-CHG001-P2G20-001 + RR-CWBS-CHG001-P2G30-001 + RR-CWBS-CHG001-P2G40-001 + COD-CWBS-001 + USR-CWBS-041`
- Resolution artifact: `GS-CWBS-P2G50-002`
- Result Composite Basis projection:
  `WR-CWBS-P2G20-002 + FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + GS-CWBS-P2G50-002 + Declaration Set {COD-CWBS-001}`
- Closed invalidation selector: stale predecessor `GS-CWBS-P2G50-001`과 old current Goal Set selection
- Downstream obligation closure change: `none`; Obligation Supersession Record Ref Set은 explicit empty
- Record selection: `effective with the same validated complete P2G-50 result`

## Goal Set Structural Validation

- Validator Version: `p2g50-goal-set-validator/1.0.0`
- Inputs: `FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + PVR-CWBS-P2G40-003 + GS-CWBS-P2G50-002`
- Overall Result: `passed`
- Counts: `25 Freeze members / 6 Phases / 7 Final Execution Nodes / 7 Goal members / 6 executable members / 6 mappings / 25 traceability rows / 12 integration rows`
- Failure Summary: `empty`
- Game test execution: `none`

## P2G-50 Complete Stage Result

- Goal Set Ref: `GS-CWBS-P2G50-002`
- P2G-50 Resolution Record Ref Set: `{RR-CWBS-CHG001-P2G50-001}`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Complete Result Validation: `passed`
- Current Selection: `GS-CWBS-P2G50-002 / current`; predecessor `GS-CWBS-P2G50-001 / stale`

## P2G-50 Exit Validation

1. `multi_phase` Goal Set은 Master 1개와 6 Phase Goal member를 포함하고 member ID가 모두 unique하다.
2. 6 ordered Phases와 7 Final Execution Nodes는 정확히 한 Phase Goal에 mapping되며 missing/duplicate/extra가 없다.
3. 25 Freeze members와 Decision/Acceptance coverage는 primary Phase를 통해 정확히 한 Phase Goal에 도달한다.
4. 각 Phase Goal의 scope, dependency, boundary, intermediate completion과 validation plan은
   `PP-CWBS-P2G40-003`을 재설계하지 않고 materialize한다.
5. 12 integration acceptance는 exact read-only seam 또는 owning Phase Goal/Final Execution Node에 mapping된다.
6. `VBRD-CWBS-001`은 common payload에 있고 initial full baseline owner는 P01, final owner는 P06이다. Phase 시작
   자체로 full validation을 반복하지 않으며 changed production/test/shared input은 impact map에 따라 invalid된다.
7. Master는 coordination만 소유하며 Phase, Final Execution Node, code/data change 또는 validation 실행을 소유하지 않는다.
8. 표준 validator는 exit code `0`, result `passed`, failures empty다.
9. Goal Set은 `Authored / Not Activated / Implementation Authorized: No`이고 P2G-60 handoff를 소유하지 않는다.
10. `COD-CWBS-001`은 valid/current이고 `RR-CWBS-CHG001-P2G50-001`이 O50을 effectively resolve한다.

```text
is_p2g50_complete(GS-CWBS-P2G50-002) = true
effective_resolution(COD-CWBS-001, OBL-CWBS-001-P2G50)
  = RR-CWBS-CHG001-P2G50-001
```

## Required Stop

- Completed Stage: `P2G-50` only
- Earliest unresolved active obligation: `(COD-CWBS-001, OBL-CWBS-001-P2G60)`
- Recommended Next Stage: `P2G-60 — current Freeze/Plan/Receipt/Goal Set의 durable handoff 작성`
- New User Request Required: `Yes`
- Handoff: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production mutation: `none`
- Test execution: `none`; baseline contract만 authored
