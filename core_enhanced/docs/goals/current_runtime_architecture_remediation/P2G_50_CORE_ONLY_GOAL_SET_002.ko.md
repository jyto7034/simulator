# Core Runtime Architecture Remediation — Successor Core-Only P2G-50 Goal Set

상태: `P2G-50 complete / current Goal Set / Authored / Not Activated / Implementation Authorized: No`

이 문서는 current `FPF-CRAR-P2G30-002`, `PP-CRAR-P2G40-002`와 passed
`PVR-CRAR-P2G40-002`를 exact basis로 사용해 8개 ordered Phase를 Master 1개와 Phase Goal 8개로 변환한
complete Goal Set이다. Goal activation, handoff, implementation과 validation execution은 수행하지 않는다.

## Stage Invocation and Entry

- User Stage Request Ref: `USR-CRAR-018`
- Exact User Statement: `P2G-50 을 수행해줘.`
- Selected Stage: `P2G-50` only
- Final Policy Freeze Ref: `FPF-CRAR-P2G30-002`; lifecycle `current`
- Phase Plan Ref: `PP-CRAR-P2G40-002`; lifecycle `current`
- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-002`; overall `passed`, failures `0`
- Upstream Exact Hashes: Freeze
  `c55dbe40a2e5ec40fb3a62ff6d02dbff6d359395a048cb08e795f22e5d25452a`; Plan
  `6d2ffd859e833df39b74de4ac93df5d32e01425b3172ebee499fc1b54e83e25b`; Receipt
  `d5d467b8314a5d0e078d29180292985ab92756f931aba100789abdce1557e9fb`
- Upstream Structure Fact: `multi_phase`; ordered Phases `8`; Final Execution Nodes `9`
- Upstream Coverage: Freeze selectors `10/10`; Phase primary owners `10/10`; integration seams `12/12`
- Applicable Active Leaf: `(COD-CRAR-001, OBL-CRAR-001-P2G50)`
- Earlier Effective Resolutions:
  `{RR-CRAR-CHG001-P2G10-001, RR-CRAR-CHG001-P2G20-001, RR-CRAR-CHG001-P2G30-001,
  RR-CRAR-CHG001-P2G40-001}`
- Correction Predecessor Goal Set Ref: `GS-CRAR-P2G50-001`; lifecycle `stale by COD-CRAR-001`. Its bound Phase Plan
  and Final Execution Node structure changed, so `PP-CRAR-P2G40-002` first completed the required Phase replanning;
  this Goal Set now replaces the exact invalidated predecessor rather than treating it as a current planning basis.
- User-Approved Validation Baseline Reuse Directive: `N/A`
- External Inspection/Claim: `none`; game_server/Unity는 source, consumer, compatibility 또는 validation target이 아님
- Production Code/Data/Test Mutation and Game Test Execution: `none`

Entry result: `eligible_successor_repair`.

## Goal Set

- Goal Set Ref: `GS-CRAR-P2G50-002`
- Predecessor Goal Set Ref: `GS-CRAR-P2G50-001`
- Lifecycle Disposition: `current`
- Goal Quality Contract Version: `2`
- Goal Structure Fact: `multi_phase`
- Exact Prerequisite Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-002 + FPF-CRAR-P2G30-002 + PP-CRAR-P2G40-002 +
  PVR-CRAR-P2G40-002 + Declaration Set {COD-CRAR-001}`
- Final Policy Freeze Ref: `FPF-CRAR-P2G30-002`
- Phase Plan Ref: `PP-CRAR-P2G40-002`
- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-002`
- Applicable P2G-50 Obligation Key Set: `{(COD-CRAR-001, OBL-CRAR-001-P2G50)}`
- P2G-50 Resolution Record Ref Set: `{RR-CRAR-CHG001-P2G50-001}`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Complete Goal Member Set:
  `{GOAL-CRAR-C2-MASTER, GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY,
  GOAL-CRAR-C2-P02-VALIDATED-DATA, GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY,
  GOAL-CRAR-C2-P04-BATTLE-KERNEL, GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT,
  GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION, GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION,
  GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION}`
- Authoring Status: `Authored`
- Activation Status: `Not Activated`
- Implementation Authorized: `No`
- Durable Handoff: `not created; P2G-60 owner`

Goal member identity는 `(GS-CRAR-P2G50-002, Goal ID)`다. Master member는 execution node나 implementation을
소유하지 않으며 아래 eight Phase members만 executable member다.

### Phase-to-Goal Member Mapping

| Phase ID | Executable Goal ID | Exact Assigned Final Execution Node Set |
| --- | --- | --- |
| `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` | `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY` | `WP-CRAR-C2-01` |
| `PHASE-CRAR-C2-02-VALIDATED-DATA` | `GOAL-CRAR-C2-P02-VALIDATED-DATA` | `WP-CRAR-C2-04` |
| `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` | `AG-CRAR-C2-01-APPLICATION-AUTHORITY` |
| `PHASE-CRAR-C2-04-BATTLE-KERNEL` | `GOAL-CRAR-C2-P04-BATTLE-KERNEL` | `WP-CRAR-C2-05` |
| `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT` | `GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT` | `WP-CRAR-C2-06` |
| `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` | `GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION` | `WP-CRAR-C2-07` |
| `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` | `GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION` | `WP-CRAR-C2-08` |
| `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` | `WP-CRAR-C2-09`, `WP-CRAR-C2-10` |

## Validation Baseline Reuse Disposition

- User-Approved Directive: `N/A`
- Disposition: cross-Goal 성공 evidence의 자동 reuse/skipping을 승인하지 않는다.
- Initial Observation Owner: `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY`; behavior oracle와 performance
  baseline을 production mutation 전에 고정하지만 full-suite reuse 권한을 만들지 않는다.
- Final Full Validation Owner: `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION`; current integrated Core에서
  complete full validation을 새로 실행한다.
- Phase-Local Rule: 각 Goal은 자신이 바꾼 production/test/fixture/tool/config input에 영향받는 focused seam을
  current target에서 non-zero로 실행하고 rebind한다. 단순 Phase 시작만으로 evidence가 current가 되지는 않는다.

## Master Goal Member — `GOAL-CRAR-C2-MASTER`

- Goal ID: `GOAL-CRAR-C2-MASTER`
- Goal Kind: `master`
- Assigned Phase / Final Execution Node / Implementation Work Ownership: None
- Complete Phase Goal ID Set:
  `{GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY, GOAL-CRAR-C2-P02-VALIDATED-DATA,
  GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY, GOAL-CRAR-C2-P04-BATTLE-KERNEL,
  GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT, GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION,
  GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION, GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION}`
- Coordination Relation: P01→P02→P03→P04→P05→P06→P07→P08 strict sequential; parallelization 없음.
- Baseline Coordination: P01 current oracle/baseline identity와 P08 final comparison/full validation ownership을
  mapping하지만 Master가 command를 실행하거나 evidence reuse를 승인하지 않는다.
- Overall Completion Conditions: eight Phase Goals가 순서대로 complete이고 Final Freeze selector `10/10`, Final
  Execution Node `9/9`, integration seam `12/12`가 current evidence로 닫히며 P08 final 10/10 acceptance와 Core-only
  full validation이 성공한다.
- Applicable Integration Risk / Reviewer Requirement: transaction/lifecycle, validated data, kernel, history lifetime,
  checkpoint migration, public projection과 final performance 경계 review가 필요하다. reviewer 지정·승인 및 Goal
  Gate 실행은 후속 Goal Execution Pipeline 소유다.

### Derived Cross-Phase Invariants

| Invariant | Exact source | Coordination condition |
| --- | --- | --- |
| Rulebook 기능 의미 보존 | `FPF-CRAR-P2G30-002` PE-001/002; Plan IVA-C2-01 | every Phase before/after differential과 final inventory 100% |
| rejected command authority equality | PE-003/004; IVA-C2-02/03 | P03 이후 모든 downstream mutation이 one executor/owner 사용 |
| validated input before authority | PE-005; IVA-C2-04 | P02 publication만 P03~P08 construction input |
| finite private battle authority | PE-006; IVA-C2-05 | P04 이후 public projection/history가 scoped port만 소비 |
| bounded diagnostic history independent of continuation | PE-009; IVA-C2-06 | P05 cap/result split을 checkpoint/projection/performance가 보존 |
| V4 semantic import and V5 target-only writes | PE-010; IVA-C2-07 | P06 이후 reader-removal certification 전 V4 reader 유지 |
| transport-neutral Core projection | PE-007; IVA-C2-08 | P07 public surface에 domain mutation handle 없음 |
| functional work precedes blocking performance work | PE-008; `USR-CRAR-016`; IVA-C2-09 | P01 관찰, P02~P07 non-blocking, P08만 optimization/PASS 소유 |

### Integration Acceptance Coordination

| Integration Acceptance | Seam Disposition | Owning Goal / Final Execution Node | Evidence contract |
| --- | --- | --- | --- |
| `IVA-CRAR-C2-01` feature inventory/final semantic preservation | `assigned_execution_node` | `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY` / `WP-CRAR-C2-01`; `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-10` | durable manifest, route/test/data index, slice/final differential |
| `IVA-CRAR-C2-02` rejection equality/retry | `assigned_execution_node` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` / `AG-CRAR-C2-01-APPLICATION-AUTHORITY`; `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-10` | six-command/StartNewGame/post-commit/durable-intent matrix |
| `IVA-CRAR-C2-03` lifecycle owner/no dual write | `assigned_execution_node` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` / `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | owner registry/API audit/invalid-phase tests |
| `IVA-CRAR-C2-04` validated publication/capability isolation | `assigned_execution_node` | `GOAL-CRAR-C2-P02-VALIDATED-DATA` / `WP-CRAR-C2-04` | actual loader invalid matrix, compile-fail/public API audit |
| `IVA-CRAR-C2-05` private finite kernel/gameplay parity | `assigned_execution_node` | `GOAL-CRAR-C2-P04-BATTLE-KERNEL` / `WP-CRAR-C2-05` | bypass audit, NaN/Inf matrix, boss/movement/skill differential |
| `IVA-CRAR-C2-06` bounded history/result independence | `assigned_execution_node` | `GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT` / `WP-CRAR-C2-06`; `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-10` | cap/overflow/cursor/resync/continuation-without-history fixtures |
| `IVA-CRAR-C2-07` V4 import/V5 write/continuation | `assigned_execution_node` | `GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION` / `WP-CRAR-C2-07` | canonical V4, V5 round-trip/checksum, continuation/removal guard |
| `IVA-CRAR-C2-08` Core public projection parity | `assigned_execution_node` | `GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION` / `WP-CRAR-C2-08`; `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-10` | exhaustive Core mapping, round-trip/public API audit |
| `IVA-CRAR-C2-09` performance non-regression/numeric guards | `assigned_execution_node` | `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY` / `WP-CRAR-C2-01`; `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-09` | event-heavy/64-session, heap/allocation/history-query/serialization/checkpoint receipts |
| `IVA-CRAR-C2-10` final ten acceptance/legacy absence | `assigned_execution_node` | `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-10` | final evidence index, static/API audits, full Core suite |
| `IVA-CRAR-C2-11` existing Core regression runners | `existing_read_only` | `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION`; implementation node `N/A` | `cargo test -p game_core --all-targets`, manifest verifier; exit 0/zero failure |
| `IVA-CRAR-C2-12` P2G-20 performance characterization | `audit_local_read_only` | `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY`; implementation node `N/A` | exact reviewed tool/input/profile, temporary raw output; final PASS 대체 불가 |

## Phase Goal Member — `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY`

- Goal ID: `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY`
- Assigned Final Execution Node Set: `WP-CRAR-C2-01`
- Covered Policy Element Revision Ref Set: `PER-CRAR-001-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-001-R1`
- Derived Observable Policy Acceptance Coverage: exact PE-001 acceptance from current Freeze; feature inventory 100% and unapproved feature removal/semantic delta 0.
- Objective / Contract-Visible Result: current Rulebook/Core behavior를 stable manifest, slice differential oracle와 reproducible pre-change performance observation으로 고정한다.
- User-Visible Problem: 구조 재작성 중 boss, movement, skill/battle 또는 roguelike 기능 손실을 체계적으로 검출할 durable oracle이 부족하다.
- User-Visible Completion Condition Set: 57 features가 actual route/test/data evidence에 연결되고 unknown 0이며 deterministic corpus와 latency/size/direct heap-allocation baseline이 재현된다.
- In-Scope: Rulebook→route→test/data manifest, rejection/checkpoint/projection corpus, event-heavy/64-session baseline probe.
- Out-of-Scope: production gameplay behavior 변경, 결함 구현, 성능 최적화 또는 performance PASS.
- Reviewed Current Behavior / Delta: current manifest/tests와 latency/size samples는 있으나 slice differential과 durable heap/allocation seam이 부족하다.
- Goal-Local Source of Truth / Owning Layer: `game_rulebook.md`, actual Core routes/tests/RON; oracle/test-support layer는 gameplay authority가 아니다.
- Goal-Local Invariant Set / Source Refs: PE-001/002/008, Phase C2-01; manifest 누락 0, behavior delta 0, exact workload/tool/input identity.
- Boundary: docs/test manifest, Core fixtures and audit tool; production runtime algorithm/data mutation 없음.
- Consumer Verification Boundary: actual Core command/query/load/checkpoint/public-projection call sites와 tests만 required; external consumers는 제외.
- Dependencies / Prerequisites: exact current Freeze/Plan/Receipt; predecessor executable Goal 없음.
- Inherited Baseline / Impact Map: reuse directive N/A. manifest/fixture/probe/toolchain 변경 시 해당 evidence를 invalid하고 current pre-change target에서 다시 실행한다.
- Assigned Durable Integration Seam: IVA-C2-01 baseline, IVA-C2-09 baseline, IVA-C2-12 auxiliary observation.
- Structural Completion / Legacy and Fallback Disposition: oracle-only production branch 0; 기존 legacy/bypass는 removal target으로 식별하되 여기서 제거하지 않는다.
- Canonical Documentation Obligation: `docs/game_rulebook.md`와 feature manifest의 bidirectional locator를 current로 유지한다.
- Migration/Compatibility/Rollout/Rollback: runtime migration 없음; invalid evidence는 폐기/recreate하며 production fallback을 만들지 않는다.
- Known Unknown / Blocker / Stop: `GAP-CRAR-P2G20-002` baseline seam을 재현하지 못하면 production mutation 전 중단한다.
- Risk / Reviewer: inventory omission과 workload representativeness review required; reviewer 승인 미수행.

### P01 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| feature preservation | Rulebook→route→test/data reverse coverage | 100% mapped, duplicate/unknown/unapproved removal 0 |
| deterministic oracle | current command/rejection/checkpoint/projection corpus | non-zero execution과 exact evidence identity |
| observation baseline | event-heavy/64-session plus heap/allocation probe | repeatable raw samples; 최적화/PASS claim 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | representative current feature routes | every stable feature has successful executable evidence |
| `invalid_input` | `required` | current invalid command/data corpus | rejection/error/pre-state evidence recorded |
| `failure_rollback` | `required` | current fault/rejection observation | partial authority delta를 검출할 oracle 존재 |
| `retry_idempotency` | `required` | same-basis retry fixtures | result/side-effect identity reproducible |
| `ordering_concurrency` | `required` | event-heavy ordering and 64-session workload | deterministic input/output ordering |
| `boundary_empty_max` | `required` | empty inventory and history/workload limits | unmapped 0 and boundary evidence present |
| `disconnect_restart_persistence` | `required` | history/checkpoint continuation corpus | current lifetime semantics fixed before cutover |
| `live_data` | `required` | actual Core RON loader corpus | all actual inputs mapped and load result recorded |
| `producer_consumer` | `required` | actual Core route→result/read-model paths | mock-only evidence 금지 |
| `legacy_fallback_absence` | `required` | legacy/bypass inventory audit | new oracle fallback 0 and removal targets identified |

## Phase Goal Member — `GOAL-CRAR-C2-P02-VALIDATED-DATA`

- Goal ID: `GOAL-CRAR-C2-P02-VALIDATED-DATA`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-02-VALIDATED-DATA`
- Assigned Final Execution Node Set: `WP-CRAR-C2-04`
- Covered Policy Element Revision Ref Set: `PER-CRAR-005-R6-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-005-R2`
- Derived Observable Policy Acceptance Coverage: exact PE-005 acceptance; production bare/unvalidated construction 0 and invalid authoring pre-authority failure.
- Objective / Contract-Visible Result: opaque validated publication만 production Core를 구성하고 fixture/direct mutation을 test-only capability로 격리한다.
- User-Visible Problem: invalid/fixture data가 runtime authority에 들어가거나 편의 API가 production state를 우회 변경할 수 있다.
- User-Visible Completion Condition Set: actual valid RON은 동일 의미로 load되고 invalid input은 authority 생성 전 오류이며 production bare/fixture/direct paths가 0이다.
- In-Scope: Core loader/validation/publication/factory, visibility/feature gating, typed Core admin command.
- Out-of-Scope: gameplay balance/data 내용 변경, server admin API, transaction/lifecycle/checkpoint/protocol 변경.
- Reviewed Current Behavior / Delta: fallible validated loader와 public builder/database/bare fields가 공존하므로 official publication을 유일 path로 만든다.
- Goal-Local Source of Truth / Owning Layer: Core data validation/publication layer; application construction root가 유일 consumer.
- Goal-Local Invariant Set / Source Refs: PE-005, Phase C2-02; invalid pre-authority failure, valid semantic parity, production test capability 0.
- Boundary: authoring RON→validation→opaque publication→Core construction and test-only fixture boundary.
- Consumer Verification Boundary: actual Core constructors/call sites/live RON required; external server/Unity/admin consumer는 검사하지 않음.
- Dependencies / Prerequisites: P01 complete and current live-data oracle.
- Inherited Baseline / Impact Map: reuse N/A. loader/schema/fixture/constructor changes invalidate live-load, invalid matrix and API visibility seams.
- Assigned Durable Integration Seam: IVA-C2-04.
- Structural Completion / Legacy and Fallback Disposition: bare constructor/builder/direct setter/fixture production path and silent default fallback 0.
- Canonical Documentation Obligation: `docs/data_loading_contract.md`를 actual Core publication boundary와 맞추되 Rulebook 우선순위를 보존한다.
- Migration/Compatibility/Rollout/Rollback: content migration 없음; invalid actual input은 defaulting 없이 publication을 중단하고 authoring source를 수정한다.
- Known Unknown / Blocker / Stop: unchecked production dependency가 발견되면 우회 API를 추가하지 않고 source boundary를 재검토한다.
- Risk / Reviewer: data validation and Rust visibility/feature-gating review required.

### P02 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| valid publication | all actual RON load and Core construction | semantic parity and one opaque path |
| invalid authoring | malformed/ref/range matrix | recoverable error, authority delta 0 |
| capability isolation | compile-fail/public API/feature audit | production bypass 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | actual valid RON load/Core construction | identical semantic publication |
| `invalid_input` | `required` | malformed/reference/range matrix | explicit recoverable pre-authority failure |
| `failure_rollback` | `required` | validation/publication fault injection | partial database/Core publication 0 |
| `retry_idempotency` | `required` | repeat identical valid/invalid load | valid identity; invalid authority delta 0 |
| `ordering_concurrency` | `N/A` | startup publication has no hot-reload owner | concurrent reload policy is not introduced |
| `boundary_empty_max` | `required` | empty/missing/min/max catalogs | approved empty only; range overflow rejected |
| `disconnect_restart_persistence` | `N/A` | Goal ends at startup publication | session/save lifetime unchanged |
| `live_data` | `required` | production loader actual RON set | actual success and invalid-startup rejection |
| `producer_consumer` | `required` | actual loader→publication→Core | mock builder cannot satisfy completion |
| `legacy_fallback_absence` | `required` | call-site/API/feature audit | bare/fixture/direct/default fallback 0 |

## Phase Goal Member — `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY`

- Goal ID: `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-03-APPLICATION-AUTHORITY`
- Assigned Final Execution Node Set: `AG-CRAR-C2-01-APPLICATION-AUTHORITY` (`WP-CRAR-C2-02`, `WP-CRAR-C2-03`)
- Covered Policy Element Revision Ref Set: `PER-CRAR-002-R6-CODE-REVIEWED`, `PER-CRAR-003-R3-CODE-REVIEWED`, `PER-CRAR-004-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-003-R1`, `DR-CRAR-004-R1`
- Derived Observable Policy Acceptance Coverage: exact PE-002/003/004 acceptance; Core-only staged rewrite, every `Err` exact equality and one route owner/no dual write.
- Objective / Contract-Visible Result: closed lifecycle와 single `validate→stage→prepare→commit→publish` executor로 all mutating routes를 atomic하게 닫는다.
- User-Visible Problem: failed command가 partial state/event/revision을 남기고 invalid lifecycle 조합이나 competing writer가 잘못된 mutation을 허용한다.
- User-Visible Completion Condition Set: every mutating route has exactly one owner/commit port; invalid phase와 every `Err` preserve exact authority; retry is deterministic.
- In-Scope: Core lifecycle sum type/context, route registry, application executor, staged draft/commit token, failure outcome typing.
- Out-of-Scope: gameplay algorithm rewrite, battle kernel privacy, external transport/consumer, history/checkpoint format.
- Reviewed Current Behavior / Delta: parallel optionals/broad router와 mutation 뒤 fallible projection paths를 one atomic authority로 교체한다.
- Goal-Local Source of Truth / Owning Layer: application executor plus closed lifecycle state; typed commit port만 authority replace/publish 소유.
- Goal-Local Invariant Set / Source Refs: PE-002..004, AG-C2-01; `Err` pre-state equality, owner exactly one, old/new writer 0.
- Boundary: world/application state, command routing, event/revision/durable-intent staging and Core outcome projection.
- Consumer Verification Boundary: actual Core callers/routes/tests required; external handlers/Unity consumers excluded.
- Dependencies / Prerequisites: P01/P02 complete; Atomic Group members cannot execute/publish separately.
- Inherited Baseline / Impact Map: reuse N/A. lifecycle/router/executor/state/event changes invalidate all command rejection/retry/owner seams.
- Assigned Durable Integration Seam: IVA-C2-02 and IVA-C2-03.
- Structural Completion / Legacy and Fallback Disposition: broad old router, parallel lifecycle writer, post-commit `Err`, direct commit bypass and dual writer 0.
- Canonical Documentation Obligation: stale `docs/core_runtime_contract.md`는 authority로 사용하지 않고 implementation 뒤 actual lifecycle/transaction contract로 갱신한다.
- Migration/Compatibility/Rollout/Rollback: Atomic Group hard cutover only; failure selects no partial member and returns source to pre-group authority.
- Known Unknown / Blocker / Stop: route가 staged/closed lifecycle로 표현 불가하거나 post-commit failure를 `Err`로 유지해야 하면 중단한다.
- Risk / Reviewer: transaction, durable intent and lifecycle typestate review required.

### P03 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| exact rejection | command/start-new-game/fault matrix | every `Err` before=after exact |
| retry/outcome | same-basis retry and post-commit injection | deterministic retry; committed failure typed accepted |
| authority closure | route registry/API/call-site audit | mapped 100%, missing/duplicate/bypass 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | every mutating command success matrix | exact commit/publish once |
| `invalid_input` | `required` | invalid command/lifecycle matrix | rejection before authority access |
| `failure_rollback` | `required` | fault injection at all fallible stages | pre-commit exact equality; no partial publish |
| `retry_idempotency` | `required` | same-basis retry/duplicate commands | deterministic result and no duplicate commit |
| `ordering_concurrency` | `required` | competing transition and event order tests | single serialized authority order |
| `boundary_empty_max` | `required` | empty/missing state and command limits | explicit prevalidation, no panic/default |
| `disconnect_restart_persistence` | `required` | durable-intent/checkpoint restart matrix | rejected intent absent; committed intent recoverable |
| `live_data` | `N/A` | validated data is inherited from P02 | loader/schema is not changed here |
| `producer_consumer` | `required` | actual Core caller→executor→result paths | all routes consume same outcome contract |
| `legacy_fallback_absence` | `required` | router/writer/API audit | old writer/bypass/fallback 0 |

## Phase Goal Member — `GOAL-CRAR-C2-P04-BATTLE-KERNEL`

- Goal ID: `GOAL-CRAR-C2-P04-BATTLE-KERNEL`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-04-BATTLE-KERNEL`
- Assigned Final Execution Node Set: `WP-CRAR-C2-05`
- Covered Policy Element Revision Ref Set: `PER-CRAR-006-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-006-R1`
- Derived Observable Policy Acceptance Coverage: exact PE-006 acceptance; no production-public mutation bypass, non-finite pre-rejection and finite deterministic parity.
- Objective / Contract-Visible Result: battle authority를 private scoped kernel/ports와 fallible finite input types로 닫는다.
- User-Visible Problem: public fields/raw floats/event injection이 lifecycle과 finite invariants를 우회해 battle corruption 또는 nondeterminism을 만들 수 있다.
- User-Visible Completion Condition Set: public mutation bypass 0, every NaN/Inf input rejects before authority mutation, finite boss/movement/skill behavior remains identical.
- In-Scope: Core battle kernel/unit/event visibility, command/query ports, finite position/speed/time types, deterministic algorithms.
- Out-of-Scope: balance/gameplay rewrite, external physics consumer, bitwise cross-platform replay guarantee.
- Reviewed Current Behavior / Delta: public kernel/unit fields/raw float inputs를 scoped private authority and finite constructors로 교체한다.
- Goal-Local Source of Truth / Owning Layer: private battle kernel is mutation owner; application executor is sole command producer.
- Goal-Local Invariant Set / Source Refs: PE-006, Phase C2-04; finite inputs, scoped mutation, deterministic ordering, event authority private.
- Boundary: application command port→kernel command/query/event/read model; external transport 없음.
- Consumer Verification Boundary: actual Core combat setup, boss/movement/skill callers and tests required; external clients excluded.
- Dependencies / Prerequisites: P01~P03 complete and validated publication/application authority current.
- Inherited Baseline / Impact Map: reuse N/A. kernel/type/event changes invalidate non-finite, ordering, gameplay differential and public API seams.
- Assigned Durable Integration Seam: IVA-C2-05.
- Structural Completion / Legacy and Fallback Disposition: public field/event injection/raw float bypass and duplicate mutation API 0.
- Canonical Documentation Obligation: actual skill targeting/battle boundary를 `docs/skill_target_contract.md`와 reconcile하고 stale runtime claims를 갱신한다.
- Migration/Compatibility/Rollout/Rollback: persistence/wire migration 없음; incomplete privacy cutover는 current Phase output으로 선택하지 않는다.
- Known Unknown / Blocker / Stop: approved finite algorithm parity가 불가능하면 normalization/defaulting 없이 중단한다.
- Risk / Reviewer: numerical validation, ordering and boss/movement/skill semantic review required.

### P04 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| private authority | compile-fail/public API/call-site audit | public mutation/event injection 0 |
| finite boundary | NaN/±Inf matrix | pre-mutation error and exact authority equality |
| gameplay parity | boss/movement/skill differential corpus | approved finite semantics/determinism unchanged |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | normal boss/movement/skill corpus | deterministic finite results parity |
| `invalid_input` | `required` | NaN/±Inf and invalid command matrix | explicit pre-mutation error |
| `failure_rollback` | `required` | kernel command fault/reject snapshot | unit/event/order authority unchanged |
| `retry_idempotency` | `required` | same-basis rejected/accepted command retry | deterministic result/no duplicate event |
| `ordering_concurrency` | `required` | same-bucket events and competing commands | stable total order and one mutation owner |
| `boundary_empty_max` | `required` | empty units/targets and numeric extremes | explicit safe/reject semantics, no overflow |
| `disconnect_restart_persistence` | `N/A` | kernel owns no save/reconnect lifetime | checkpoint/history are later Goals |
| `live_data` | `required` | actual validated skill/boss/movement data | production setup exercises private ports |
| `producer_consumer` | `required` | application executor→kernel→read model/event | mock-only kernel tests insufficient |
| `legacy_fallback_absence` | `required` | visibility/call-site audit | public/raw/direct mutation aliases 0 |

## Phase Goal Member — `GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT`

- Goal ID: `GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT`
- Assigned Final Execution Node Set: `WP-CRAR-C2-06`
- Covered Policy Element Revision Ref Set: `PER-CRAR-009-R6-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-008-R2`
- Derived Observable Policy Acceptance Coverage: exact PE-009 acceptance; bounded Core history, gap-free window query and result/continuation independence.
- Objective / Contract-Visible Result: diagnostic active history를 count/byte/battle-time cap으로 닫고 result/continuation authority에서 분리한다.
- User-Visible Problem: long battles/history queries can grow without bound and optional diagnostics can incorrectly retain or control gameplay continuation.
- User-Visible Completion Condition Set: first of 8,192 events/4 MiB/180 battle-seconds closes window, old cursor requests typed resync, and history loss never changes result/continuation.
- In-Scope: Core event window/index/cursor/floor/resync requirement, result/continuation lifetime split.
- Out-of-Scope: battle timeout, external reconnect/receipt/archive, server persistence and client protocol.
- Reviewed Current Behavior / Delta: unbounded Vec and cloned linear tail become bounded indexed window with authority-independent result.
- Goal-Local Source of Truth / Owning Layer: Core active-session history owner; continuation/result aggregate is separate authority.
- Goal-Local Invariant Set / Source Refs: PE-009, TD-018/Phase C2-05; cap-first semantics, gap/duplicate 0, overflow diagnostic-only.
- Boundary: kernel event sequence→bounded window/query/resync/result projection; no external store.
- Consumer Verification Boundary: actual Core history query/public projection/checkpoint continuation callers required; external consumers excluded.
- Dependencies / Prerequisites: P01/P03/P04 complete.
- Inherited Baseline / Impact Map: reuse N/A. history/index/result changes invalidate window/cursor/overflow/continuation and Phase-8 scaling seams.
- Assigned Durable Integration Seam: IVA-C2-06.
- Structural Completion / Legacy and Fallback Disposition: unbounded active log, full-tail clone fallback and history-as-result-authority 0.
- Canonical Documentation Obligation: Core history/result lifetime and 180 battle-seconds가 timeout이 아님을 actual runtime contract에 기록한다.
- Migration/Compatibility/Rollout/Rollback: durable external archive N/A; incomplete cap cutover or dual unbounded log를 publish하지 않는다.
- Known Unknown / Blocker / Stop: cap 적용이 result/continuation semantics를 바꾸면 diagnostic loss로 숨기지 않고 중단한다.
- Risk / Reviewer: sequence/cursor/lifetime/overflow review required.

### P05 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| bounded lifetime | three-cap boundary/overflow property tests | retained history never exceeds active limit semantics |
| query/resync | in-window and before-floor cursor matrix | exact delta or typed snapshot resync |
| result independence | continuation with absent/expired/overflowed history | identity and next decisions unchanged |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | in-window append/query/result corpus | gap-free exact deltas and same continuation |
| `invalid_input` | `required` | invalid/future cursor and malformed sequence | typed error/resync, authority unchanged |
| `failure_rollback` | `required` | allocation/query/result projection fault injection | committed gameplay result not reverted by diagnostic failure |
| `retry_idempotency` | `required` | repeated cursor query/resync/result reads | identical projection and no mutation |
| `ordering_concurrency` | `required` | same-sequence/competing append ordering | unique monotonic sequence, duplicate 0 |
| `boundary_empty_max` | `required` | empty window and exact cap±1 cases | correct floor/cap behavior, no unbounded fallback |
| `disconnect_restart_persistence` | `required` | continuation/checkpoint without retained history | gameplay continuation survives history absence |
| `live_data` | `N/A` | no authoring data schema is changed | runtime event/history state only |
| `producer_consumer` | `required` | actual kernel events→window→Core projection | sequence/result mapping parity |
| `legacy_fallback_absence` | `required` | owner/query/call-site audit | unbounded Vec/full-clone/history authority 0 |

## Phase Goal Member — `GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION`

- Goal ID: `GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION`
- Assigned Final Execution Node Set: `WP-CRAR-C2-07`
- Covered Policy Element Revision Ref Set: `PER-CRAR-010-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-009-R1`
- Derived Observable Policy Acceptance Coverage: exact PE-010 acceptance; canonical V4 semantic import, V5-only new writes and reader-removal certification gate.
- Objective / Contract-Visible Result: Core checkpoint writer를 V5로 전환하고 explicit V4 importer와 continuation certification을 유지한다.
- User-Visible Problem: checkpoint format rewrite can silently lose identity/state/continuation or remove the only V4 reader before old saves are certified.
- User-Visible Completion Condition Set: canonical V4 imports with same meaning, new writes are V5 only, V5 round-trips, corrupt input rejects before mutation and V4 reader removal gate fails until certification.
- In-Scope: Core checkpoint codec/version/checksum/importer/canonical corpus/continuation tests.
- Out-of-Scope: external store envelope, released/deployed save population, dual-format writes or rollout.
- Reviewed Current Behavior / Delta: strict V4 reader/writer exists; add V5 target writer and explicit V4 importer while retaining read certification.
- Goal-Local Source of Truth / Owning Layer: Core checkpoint codec plus canonical semantic fixture; closed lifecycle/continuation aggregate is imported authority.
- Goal-Local Invariant Set / Source Refs: PE-010, Phase C2-06; identity/state/continuation preservation, target-only write, pre-mutation strict reject.
- Boundary: Core state/continuation↔versioned checkpoint bytes; no external server store.
- Consumer Verification Boundary: actual Core capture/load/continue callers/tests required; released saves and external consumers N/A.
- Dependencies / Prerequisites: P01~P03 and P05 complete.
- Inherited Baseline / Impact Map: reuse N/A. codec/state/corpus changes invalidate V4 import, V5 round-trip, corruption, continuation and removal-gate seams.
- Assigned Durable Integration Seam: IVA-C2-07.
- Structural Completion / Legacy and Fallback Disposition: V4 writer/dual-write/best-effort parse/schema alias 0; V4 reader remains explicit until certification.
- Canonical Documentation Obligation: checkpoint V4-import/V5-write and certification boundary를 actual Core runtime contract에 갱신한다.
- Migration/Compatibility/Rollout/Rollback: canonical development corpus migration only; failure keeps V4 reader and rejects current Goal completion.
- Known Unknown / Blocker / Stop: Released/Deployed actual이 나타나면 current N/A를 사용하지 않고 별도 evidence/policy review 전 중단한다.
- Risk / Reviewer: codec, identity/continuation, strict rejection and reader-retention review required.

### P06 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| V4 semantic import | canonical corpus import/continue/recapture | identity/state/continuation exact |
| V5 write/read | target-only byte/version/round-trip tests | new writes V5 only and semantic parity |
| strict reject/removal gate | corrupt/unknown/missing plus deletion negative gate | pre-mutation reject and premature removal denied |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | V4 import and V5 round-trip corpus | exact semantic continuation |
| `invalid_input` | `required` | corrupt/unknown/missing/checksum matrix | strict pre-mutation rejection |
| `failure_rollback` | `required` | parse/validate/restore fault injection | current authority unchanged on failure |
| `retry_idempotency` | `required` | repeat import/load/write from same basis | same state/bytes semantics, no duplicate usage |
| `ordering_concurrency` | `required` | competing capture/load and revision ordering | explicit ordering/stale-basis rejection |
| `boundary_empty_max` | `required` | empty optional state, max collection/size corpus | bounded valid round-trip or explicit reject |
| `disconnect_restart_persistence` | `required` | capture→restart→load→continue | continuation identical across process boundary |
| `live_data` | `required` | checkpoint restore against actual validated data | identity/reference restoration valid |
| `producer_consumer` | `required` | actual Core writer→bytes→reader→runtime | mock codec alone insufficient |
| `legacy_fallback_absence` | `required` | version/call-site/removal audit | V4 writer/dual-write/best-effort alias 0; reader gate retained |

## Phase Goal Member — `GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION`

- Goal ID: `GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION`
- Assigned Final Execution Node Set: `WP-CRAR-C2-08`
- Covered Policy Element Revision Ref Set: `PER-CRAR-007-R6-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-007-R2`
- Derived Observable Policy Acceptance Coverage: exact PE-007 acceptance; transport-neutral Core projection exposes no domain mutation authority and passes Core semantic parity.
- Objective / Contract-Visible Result: Core application command/result/error/snapshot/read-model을 domain aggregate/mutation handle에서 분리한다.
- User-Visible Problem: public serde/DTO surface가 domain state와 mutation authority에 결합돼 boundary changes가 runtime invariants를 우회하게 한다.
- User-Visible Completion Condition Set: all Core meanings map exhaustively through transport-neutral application ports, parity passes and public domain mutation/legacy projection paths are 0.
- In-Scope: Core public application types/ports/mappers/read models/API visibility/parity corpus.
- Out-of-Scope: server wire version/schema, Unity consumer, network transport and external compatibility.
- Reviewed Current Behavior / Delta: domain/application/transport-shaped types coexist; create Core-owned projection boundary and remove public mutation handles.
- Goal-Local Source of Truth / Owning Layer: application projection layer reads closed lifecycle/kernel/history/checkpoint ports; domain stays private authority.
- Goal-Local Invariant Set / Source Refs: PE-007, Phase C2-07; exhaustive semantic mapping, mutation handle 0, external claim 0.
- Boundary: Core application command/result/error/snapshot/read-model only; external transport adapter N/A.
- Consumer Verification Boundary: actual Core public API call sites/tests required; game_server/Unity consumers explicitly not required or claimed.
- Dependencies / Prerequisites: P03~P06 complete.
- Inherited Baseline / Impact Map: reuse N/A. public type/mapper/read-model changes invalidate exhaustive mapping, parity, visibility and downstream performance serialization seams.
- Assigned Durable Integration Seam: IVA-C2-08.
- Structural Completion / Legacy and Fallback Disposition: public aggregate/mutation handle/direct domain serde/legacy Core projection aliases 0.
- Canonical Documentation Obligation: transport-neutral Core projection contract를 actual API로 문서화하고 stale Unity contract는 수정/검사하지 않는다.
- Migration/Compatibility/Rollout/Rollback: external protocol migration N/A; Core hard cutover만 current 선택하며 dual public API를 유지하지 않는다.
- Known Unknown / Blocker / Stop: public semantic mapping을 external schema 추정으로 채워야 한다면 범위를 넓히지 않고 중단한다.
- Risk / Reviewer: API ownership, exhaustive mapping and mutation visibility review required.

### P07 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| exhaustive projection | command/result/error/snapshot mapping coverage | every Core variant mapped exactly once |
| semantic parity | old Core meaning vs new application-port corpus | approved observable outcomes identical |
| authority privacy | public API/serde/call-site audit | mutation handle/direct domain projection 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | normal command/result/snapshot parity | exact transport-neutral result meaning |
| `invalid_input` | `required` | unknown/invalid command and error mapping | typed Core rejection with no mutation |
| `failure_rollback` | `required` | projection failure after/before commit matrix | no committed state represented as `Err`; no partial projection authority |
| `retry_idempotency` | `required` | repeated query/result/error projection | identical read model, no mutation |
| `ordering_concurrency` | `required` | revision/event/result ordering corpus | explicit stable ordering/revision semantics |
| `boundary_empty_max` | `required` | empty/large snapshot and result collections | bounded explicit representation/no truncation ambiguity |
| `disconnect_restart_persistence` | `required` | resync/checkpoint error/result projection | Core continuation meanings preserved |
| `live_data` | `N/A` | projection does not load authoring data | P02 owns live RON publication |
| `producer_consumer` | `required` | actual Core service→public port caller round-trip | external consumers not substituted or claimed |
| `legacy_fallback_absence` | `required` | API/serde/call-site audit | public domain mutation/direct serde/dual projection 0 |

## Phase Goal Member — `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION`

- Goal ID: `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION`
- Assigned Final Execution Node Set: `WP-CRAR-C2-09`, `WP-CRAR-C2-10`
- Covered Policy Element Revision Ref Set: `PER-CRAR-008-R6-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-002-R1`
- Derived Observable Policy Acceptance Coverage: exact PE-008 acceptance plus validation-only final coverage of every Freeze element; like-for-like non-regression, numeric guards and full 10/10 acceptance.
- Objective / Contract-Visible Result: all functional Core boundaries are complete before measuring, then optimize only proven hotspots and publish final Core acceptance evidence.
- User-Visible Problem: premature optimization can distort incomplete architecture, while unmeasured final regressions/legacy bypass can make a functionally correct rewrite operationally worse.
- User-Visible Completion Condition Set: P01 baseline and final architecture compare like-for-like with no statistically supported regression, final caps/guards pass, full Core suite and all ten acceptance rows are current.
- In-Scope: final profiler/benchmark, required hotspot optimization, heap/allocation/retained-memory/history-query/serialization/checkpoint guards, final full Core validation/evidence index/legacy audit.
- Out-of-Scope: optimization during P01~P07, arbitrary extra caps, external server/client metrics, production deployment and deferred new features.
- Reviewed Current Behavior / Delta: current representative timings are adequate but direct heap baseline/guards and final architecture comparison do not exist.
- Goal-Local Source of Truth / Owning Layer: final current Core executable target; benchmark/profiler and full validation evidence are verification owners, not gameplay authority.
- Goal-Local Invariant Set / Source Refs: PE-008 and all final validation selectors, Phase C2-08; functions first, exact deterministic output, no regression, history caps, legacy/bypass 0.
- Boundary: final Core runtime/read model/checkpoint paths plus task-owned validation/benchmark tooling; external metrics absent.
- Consumer Verification Boundary: actual Core public API and full `game_core` tests/manifest/benchmark only; game_server/Unity not inspected or required.
- Dependencies / Prerequisites: P01~P07 complete; internal order WP-C2-09 performance stabilization then WP-C2-10 final reconciliation.
- Inherited Baseline / Impact Map: reuse directive N/A. final source/tool/input/config changes invalidate affected benchmark/full validation evidence and require current non-zero rerun.
- Assigned Durable Integration Seam: IVA-C2-01/02/06/08 final portions, IVA-C2-09/10 and existing runner IVA-C2-11.
- Structural Completion / Legacy and Fallback Disposition: all superseded Core paths, duplicate authority, unbounded history, direct mutation and benchmark-only production branches 0.
- Canonical Documentation Obligation: final actual Core architecture/performance budget and feature manifest를 current로 갱신하며 stale external contracts를 authoritative로 만들지 않는다.
- Migration/Compatibility/Rollout/Rollback: V4/V5 Core certification evidence included; external rollout/deployment N/A; performance optimization regression은 last known functionally complete Phase-7 state로 source revert 후 재검증한다.
- Known Unknown / Blocker / Stop: `GAP-CRAR-P2G20-002`가 final measurement/guard로 닫히지 않거나 cap failure가 남으면 performance PASS와 Goal completion을 주장하지 않는다.
- Risk / Reviewer: benchmarking statistics, allocation attribution, final cross-boundary acceptance and legacy-removal review required.

### P08 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| final performance | interleaved repeated event-heavy/64-session plus profiler | no supported regression, `<25 ms`/`<10 ms`, memory/query/serialization guards pass |
| functional integration | full Core suite, manifest and ten-acceptance evidence index | non-zero current execution, 10/10 and 57/57 |
| structural closure | API/call-site/legacy/bypass audit | duplicate authority/fallback/unbounded history 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | full normal Core suite and benchmark | all acceptance and final success paths pass |
| `invalid_input` | `required` | aggregate invalid data/command/float/checkpoint matrix | explicit rejection and authority equality |
| `failure_rollback` | `required` | aggregate transaction/kernel/checkpoint fault evidence | no partial state/event/durable publication |
| `retry_idempotency` | `required` | retry/query/import/result aggregate corpus | deterministic/idempotent approved meanings |
| `ordering_concurrency` | `required` | event ordering and 64-session repeated workload | stable ordering, no duplicate/missing events |
| `boundary_empty_max` | `required` | empty/max/history cap/snapshot/checkpoint corpus | every approved limit explicit and passing |
| `disconnect_restart_persistence` | `required` | bounded history, resync, V4/V5 continuation | restart/continuation identity preserved |
| `live_data` | `required` | actual RON full load under final Core | all actual data valid through opaque publication |
| `producer_consumer` | `required` | actual Core service→public projection round-trip | Core semantic parity; external claim 0 |
| `legacy_fallback_absence` | `required` | repository API/call-site/static audit | superseded Core paths/dual writer/fallback 0 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref Set | Primary Phase ID | Executable Goal ID |
| --- | --- | --- | --- | --- |
| `PE-CRAR-001` | `PER-CRAR-001-R3-CODE-REVIEWED` | `DR-CRAR-001-R1` | `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` | `GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY` |
| `PE-CRAR-002` | `PER-CRAR-002-R6-CODE-REVIEWED` | `DR-CRAR-003-R1` | `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-003` | `PER-CRAR-003-R3-CODE-REVIEWED` | `DR-CRAR-004-R1` | `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-004` | `PER-CRAR-004-R3-CODE-REVIEWED` | `DR-CRAR-003-R1` | `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-005` | `PER-CRAR-005-R6-CODE-REVIEWED` | `DR-CRAR-005-R2` | `PHASE-CRAR-C2-02-VALIDATED-DATA` | `GOAL-CRAR-C2-P02-VALIDATED-DATA` |
| `PE-CRAR-006` | `PER-CRAR-006-R3-CODE-REVIEWED` | `DR-CRAR-006-R1` | `PHASE-CRAR-C2-04-BATTLE-KERNEL` | `GOAL-CRAR-C2-P04-BATTLE-KERNEL` |
| `PE-CRAR-007` | `PER-CRAR-007-R6-CODE-REVIEWED` | `DR-CRAR-007-R2` | `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` | `GOAL-CRAR-C2-P07-CORE-PUBLIC-PROJECTION` |
| `PE-CRAR-008` | `PER-CRAR-008-R6-CODE-REVIEWED` | `DR-CRAR-002-R1` | `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | `GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` |
| `PE-CRAR-009` | `PER-CRAR-009-R6-CODE-REVIEWED` | `DR-CRAR-008-R2` | `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT` | `GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT` |
| `PE-CRAR-010` | `PER-CRAR-010-R3-CODE-REVIEWED` | `DR-CRAR-009-R1` | `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` | `GOAL-CRAR-C2-P06-CHECKPOINT-MIGRATION` |

## Resolution Record — `RR-CRAR-CHG001-P2G50-001`

- Obligation Key: `(COD-CRAR-001, OBL-CRAR-001-P2G50)`
- Owner Stage: `P2G-50`
- Exact Resolved Prerequisite Basis:
  `FPF-CRAR-P2G30-002 + PP-CRAR-P2G40-002 + PVR-CRAR-P2G40-002 +
  RR-CRAR-CHG001-P2G40-001 + PFCR-CRAR-001 + COD-CRAR-001`
- Resolution Artifact Ref: `GS-CRAR-P2G50-002`
- Exact Result Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-002 + FPF-CRAR-P2G30-002 + PP-CRAR-P2G40-002 +
  PVR-CRAR-P2G40-002 + GS-CRAR-P2G50-002 + Declaration Set {COD-CRAR-001}`
- Resolution Evidence: Quality Contract v2 validator passed, Phase mapping 8/8, node mapping 9/9, traceability 10/10,
  integration 12/12 and validation dimensions 80/80
- Selection: `effective with the same complete P2G-50 result`

## Goal Authoring Summary

- Goal Set Ref: `GS-CRAR-P2G50-002`
- Goal Quality Contract Version: `2`
- Master / Phase Goal Members: `1 / 8`
- Phase-to-Goal / Final Execution Node Mapping: `8/8 / 9/9`
- Freeze PER / Decision Traceability: `10/10 / 9/9`
- Validation Dimension Rows: `80/80`
- Integration Acceptance Mapping: `12/12`
- Shared Validation Baseline Reuse Directive: `N/A`; automatic evidence reuse/skipping authorization 없음
- Authoring/Activation/Authorization: `Authored / Not Activated / No`

## P2G-50 Structure Validation

- Validator: `p2g50-goal-set-validator/1.2.0`
- Quality Contract: `2`
- Upstream Input: exact current Freeze/Plan/passed P2G-40 Receipt
- Result: `passed`
- Counts: Freeze `10`, Phases `8`, Final Execution Nodes `9`, Goal Members `9`, executable `8`, mapping `8`,
  traceability `10`, integration `12`, validation dimension rows `80`
- Failure Summary: `[]`
- Game Test/Production Mutation: `none`; structure validation only

## P2G-50 Complete Stage Result

- Owner Stage: `P2G-50`
- Required Result Artifact/Relation Set:
  - Goal Set Ref: `GS-CRAR-P2G50-002`
  - Complete Goal Member Set: Master `1` + Phase Goals `8`
  - Phase-to-Goal Member Mapping: `8/8`
  - P2G-50 Resolution Record Ref Set: `{RR-CRAR-CHG001-P2G50-001}`
  - Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Cross-Reference/Coverage Evidence: validator passed, traceability 10/10, integration 12/12, dimensions 80/80
- Exit Condition Validation: `passed`
- Current Selection Result: complete Goal Set and Resolution Record selected together; partial candidate publication `0`

## Required Stop

- Completed Stage: `P2G-50` only
- Earliest Unresolved Active Obligation: `(COD-CRAR-001, OBL-CRAR-001-P2G60)`
- Recommended Next Stage: `P2G-60 — exact current Freeze/Plan/Goal Set durable handoff`
- New User Request Required: `Yes — explicit P2G-60 request`
- Handoff Artifact: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production Mutation: `none`
- Test Execution: `none`; Goal authoring/structure validation only
