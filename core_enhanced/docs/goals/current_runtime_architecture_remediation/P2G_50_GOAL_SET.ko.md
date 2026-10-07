# Core Runtime Architecture Remediation P2G-50 Goal Set

상태: `P2G-50 complete / current Goal Set / Authored / Not Activated / Implementation Authorized: No`

이 문서는 current `PP-CRAR-P2G40-001`의 8개 ordered Phase를 재설계하지 않고 Master coordination member
1개와 Phase Goal member 8개로 materialize한 complete Goal Set이다. P2G-60 handoff, Goal activation,
production mutation과 구현은 수행하거나 승인하지 않는다.

## Stage Invocation and Entry

- User Request Ref: `USR-CRAR-010`
- Exact user statement: `응, 그럼 P2G-50 을 수행해줘.`
- Selected Stage: `P2G-50`
- Contract Basis: `core_enhanced/docs/policy_to_goal_pipeline.md`, `sha256:874742806deb79a39b13972d80ea08e5f6340f3b6fe24a9b7b42d2ffb8544c1c`
- Final Policy Freeze Ref: `FPF-CRAR-P2G30-001 / current`
- Phase Plan Ref: `PP-CRAR-P2G40-001 / current`
- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-001 / passed`
- Exact Upstream Hashes: Freeze `2a0f70f8359f7b7353bf7e1ca217528ec5d293aff1e9d46c04698e4d4f96bfa3`; Plan `ba2f1b20d372a579f43b05952f147458b92b9a7afda96cc61952cb4d0585cd81`; Receipt `0662850628f62a7e4336cba3640e8c873b02cd0f0e46c7020c7a95382aa4298a`
- Receipt Validation: `p2g40-receipt-validator/1.0.0 / passed / failures 0`
- Goal Structure Fact: `multi_phase / 8 ordered Phases`
- Validation Baseline Reuse Directive: `N/A — 별도 user-approved reuse directive 없음`
- Correction Predecessor: `N/A — initial Goal Set`
- Applicable Change/Obligation Basis: `explicit empty`
- Entry Result: `passed`; Freeze members `10`, Final Execution Nodes `9`, Phase assignments `8`, integration ownership rows `12`가 current Plan/Receipt에서 complete

## Goal Set

- Goal Set Ref: `GS-CRAR-P2G50-001`
- Predecessor Goal Set Ref: `N/A`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-001 + RB-CRAR-P2G20-001 + RSB-CRAR-P2G20-001 + FPF-CRAR-P2G30-001 + PP-CRAR-P2G40-001 + PVR-CRAR-P2G40-001 + explicit-empty Change/Obligation set`
- Goal Structure Fact: `multi_phase`
- Phase Plan Ref: `PP-CRAR-P2G40-001`
- Final Policy Freeze Ref: `FPF-CRAR-P2G30-001`
- Goal Quality Contract Version: `2`
- Complete Goal Member Set:
  `GOAL-CRAR-MASTER`,
  `GOAL-CRAR-P01-PRESERVATION-BASELINE`,
  `GOAL-CRAR-P02-VALIDATED-DATA`,
  `GOAL-CRAR-P03-APPLICATION-AUTHORITY`,
  `GOAL-CRAR-P04-BATTLE-KERNEL`,
  `GOAL-CRAR-P05-SESSION-HISTORY-RESULT`,
  `GOAL-CRAR-P06-CHECKPOINT-MIGRATION`,
  `GOAL-CRAR-P07-PROTOCOL-CUTOVER`,
  `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION`
- Applicable P2G-50 Obligation Key Set: `explicit empty`
- P2G-50 Resolution Record Ref Set: `explicit empty`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Authoring Status: `Authored`
- Activation Status: `Not Activated`
- Implementation Authorized: `No`

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| `PHASE-CRAR-01-PRESERVATION-BASELINE` | `GOAL-CRAR-P01-PRESERVATION-BASELINE` | `WP-CRAR-01` | exactly one Goal |
| `PHASE-CRAR-02-VALIDATED-DATA` | `GOAL-CRAR-P02-VALIDATED-DATA` | `WP-CRAR-04` | exactly one Goal |
| `PHASE-CRAR-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` | `AG-CRAR-01-APPLICATION-AUTHORITY` | exactly one Goal |
| `PHASE-CRAR-04-BATTLE-KERNEL` | `GOAL-CRAR-P04-BATTLE-KERNEL` | `WP-CRAR-05` | exactly one Goal |
| `PHASE-CRAR-05-SESSION-HISTORY-RESULT` | `GOAL-CRAR-P05-SESSION-HISTORY-RESULT` | `WP-CRAR-06` | exactly one Goal |
| `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | `GOAL-CRAR-P06-CHECKPOINT-MIGRATION` | `WP-CRAR-08` | exactly one Goal |
| `PHASE-CRAR-07-PROTOCOL-CUTOVER` | `GOAL-CRAR-P07-PROTOCOL-CUTOVER` | `AG-CRAR-02-PROTOCOL-CUTOVER` | exactly one Goal |
| `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` | `WP-CRAR-07`, `WP-CRAR-11` | exactly one Goal |

## Validation Baseline Reuse Disposition

- User-Approved Directive: `N/A`
- Evidence Reuse Authorization: `none`; 이전 성공 결과는 각 Goal completion evidence로 자동 승계하지 않음
- Phase-Local Rule: 각 Goal은 자신의 changed input-to-seam impact map에 해당하는 focused validation을 current target에서 non-zero로 실행해야 함
- Full Validation Ownership: Phase 1은 production mutation 전 preservation/performance baseline을 고정하고 Phase 8은 final integrated full Core/server/Unity validation을 실행한다. 이는 evidence reuse 승인이나 중간 Goal validation 생략을 뜻하지 않음
- Authoring-Time Execution: `none`; P2G-50은 실행 권한을 부여하지 않음

## Master Goal Member — `GOAL-CRAR-MASTER`

- Goal ID: `GOAL-CRAR-MASTER`
- Goal Kind: `master`
- Complete Phase Goal ID Set:
  `GOAL-CRAR-P01-PRESERVATION-BASELINE`, `GOAL-CRAR-P02-VALIDATED-DATA`,
  `GOAL-CRAR-P03-APPLICATION-AUTHORITY`, `GOAL-CRAR-P04-BATTLE-KERNEL`,
  `GOAL-CRAR-P05-SESSION-HISTORY-RESULT`, `GOAL-CRAR-P06-CHECKPOINT-MIGRATION`,
  `GOAL-CRAR-P07-PROTOCOL-CUTOVER`, `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION`
- Derived Sequential/Parallel Relations: `P01 → P02 → P03 → P04 → P05 → P06 → P07 → P08`; parallel execution `none approved`
- Coordination Dependencies/Conditions: 각 predecessor Phase Goal의 completion matrix와 Intermediate-State Audit evidence가 current여야 다음 Goal을 선택하며, dependency를 사용자 승인만으로 우회하지 않는다.
- Assigned Phase / Final Execution Node / Implementation Work Ownership: `None`

### Derived Cross-Phase Invariants

| Invariant | Exact source | Coordination requirement |
| --- | --- | --- |
| 승인된 현재 Rulebook 기능과 named boss/movement/skill/battle/roguelike 의미를 제거하거나 승인 없이 algorithm rewrite하지 않음 | `FPF-CRAR-P2G30-001: PE-CRAR-001, PE-CRAR-002`; `PP-CRAR-P2G40-001` | P01 oracle을 모든 cutover가 소비하고 P08이 10/10 final differential을 재검증 |
| 모든 rejected mutation은 exact pre-command authority/durable equality이며 route마다 commit owner가 하나뿐임 | `PE-CRAR-003`, `PE-CRAR-004`; `IVA-CRAR-02..03` | P03 application authority를 P04~P07이 우회하지 않고 P08이 cross-route matrix를 재검증 |
| production data와 battle mutation은 validated publication/private finite kernel 경계를 통해서만 진입 | `PE-CRAR-005`, `PE-CRAR-006`; `IVA-CRAR-04..05` | P02 construction boundary를 P03/P04가 소비하고 public fixture/kernel bypass를 재도입하지 않음 |
| live reconnect/history와 archive는 bounded이며 durable result/receipt와 gameplay authority를 공유하지 않음 | `PE-CRAR-009`; `IVA-CRAR-06` | P05 lifetime owners를 P07 wire adapter가 보존하고 P08이 overflow/expiry continuation을 재검증 |
| V4는 certification까지 semantic importer/oracle이고 새 save와 external protocol은 각각 V5와 protocol v1만 생성 | `PE-CRAR-007`, `PE-CRAR-010`; `IVA-CRAR-07..08` | P06 save migration 뒤 P07 server/Unity hard cutover를 수행하며 dual write/dual wire path를 만들지 않음 |
| 동일 workload 핵심 지표는 pre-cutover baseline보다 악화되지 않고 permanent numeric guard를 가짐 | `PE-CRAR-008`; `IVA-CRAR-09` | P01이 baseline을 고정하고 P08이 event-heavy/64-session final comparison을 소유 |

### Integration Acceptance Coordination

| Acceptance | Disposition | Owning Goal / Final Execution Node | Locator or evidence contract |
| --- | --- | --- | --- |
| `IVA-CRAR-01` feature inventory와 semantic preservation | `assigned_execution_node` | `GOAL-CRAR-P01-PRESERVATION-BASELINE` / `WP-CRAR-01`; `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-11` | durable manifest, route/test index와 old/new differential |
| `IVA-CRAR-02` rejection equality/retry | `assigned_execution_node` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` / `AG-CRAR-01-APPLICATION-AUTHORITY`; `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-11` | six-command, StartNewGame, post-commit/durable-intent matrix |
| `IVA-CRAR-03` lifecycle/route single owner | `assigned_execution_node` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` / `AG-CRAR-01-APPLICATION-AUTHORITY` | owner registry/API audit와 invalid-phase fixtures |
| `IVA-CRAR-04` validated publication/capability isolation | `assigned_execution_node` | `GOAL-CRAR-P02-VALIDATED-DATA` / `WP-CRAR-04` | loader invalid matrix, compile-fail/API audit, AdminCommand tests |
| `IVA-CRAR-05` private finite kernel/determinism | `assigned_execution_node` | `GOAL-CRAR-P04-BATTLE-KERNEL` / `WP-CRAR-05` | bypass audit, NaN/Inf matrix, boss/movement/skill differential |
| `IVA-CRAR-06` bounded history/result/archive | `assigned_execution_node` | `GOAL-CRAR-P05-SESSION-HISTORY-RESULT` / `WP-CRAR-06`; `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-11` | window/resync/property, restart/idempotency/archive-expiry fixtures |
| `IVA-CRAR-07` V4 import/V5 write/continuation | `assigned_execution_node` | `GOAL-CRAR-P06-CHECKPOINT-MIGRATION` / `WP-CRAR-08` | canonical V4, V5 round-trip/checksum, continuation/removal guard |
| `IVA-CRAR-08` protocol v1 semantic parity/current consumer | `assigned_execution_node` | `GOAL-CRAR-P07-PROTOCOL-CUTOVER` / `AG-CRAR-02-PROTOCOL-CUTOVER`; `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-11` | server goldens와 Unity EditMode/PlayMode contract/reconnect fixtures |
| `IVA-CRAR-09` performance non-regression/numeric caps | `assigned_execution_node` | `GOAL-CRAR-P01-PRESERVATION-BASELINE` / `WP-CRAR-01`; `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-07` | event-heavy boss/64-session interleaved benchmark receipts |
| `IVA-CRAR-10` final 10/10 acceptance와 legacy absence | `assigned_execution_node` | `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` / `WP-CRAR-11` | final evidence index, API/static audits, end-to-end suites |
| `IVA-CRAR-11` repository-wide regression runners | `existing_read_only` | `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION`; implementation node `N/A` | task-owned target dir의 Core/server tests와 Unity runners; exit 0/zero failure |
| `IVA-CRAR-12` P2G-20 lightweight performance probe | `audit_local_read_only` | `GOAL-CRAR-P01-PRESERVATION-BASELINE`; implementation node `N/A` | reviewed `/tmp/core_arch_perf`; ordinary current actual만 관찰 |

- Baseline Coordination: 별도 reuse directive가 없으므로 Master는 성공 evidence 재사용을 승인하지 않는다. P01 baseline identity와 P08 comparison input의 동일성만 coordination한다.
- Overall Completion Conditions: P01~P08이 순서대로 complete이고 12개 integration mapping이 current evidence를 가지며 P08 final 10/10 acceptance, numeric guards와 full Core/server/Unity validation이 성공한다.
- Applicable Integration Risk / Reviewer Requirement: transaction, lifecycle, data publication, kernel, persistence, migration, protocol과 performance 경계 review가 필요하다. reviewer 지정·승인과 Goal Gate 실행은 후속 execution pipeline 소유다.

## Phase Goal Member — `GOAL-CRAR-P01-PRESERVATION-BASELINE`

- Goal ID: `GOAL-CRAR-P01-PRESERVATION-BASELINE`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-01-PRESERVATION-BASELINE`
- Assigned Final Execution Node Set: `WP-CRAR-01`
- Covered Policy Element Revision Ref Set: `PER-CRAR-001-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-001-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER의 exact acceptance in `FPF-CRAR-P2G30-001`; 기능 inventory 100%와 cutover 전후 승인되지 않은 removal/semantic delta 0
- Objective / Contract-Visible Result: 현재 Rulebook 기능, route, test와 persistence/protocol/rejection/performance behavior를 stable feature manifest와 재현 가능한 pre-cutover oracle로 고정한다.
- User-Visible Problem: 전면 경계 재작성 도중 기존 boss, movement, skill/battle, roguelike 기능이 사라지거나 의미가 바뀌어도 이를 체계적으로 검출할 단일 보존 oracle이 없다.
- User-Visible Completion Condition Set: 모든 current 기능이 stable ID와 executable evidence에 연결되고 unmapped 기능이 0이며, Little Red·projectile·effect event-heavy 및 64-session/8-way workload의 pre-cutover 결과가 재현 가능하다.
- In-Scope: Rulebook heading/behavior route/domain test inventory, rejection/save/protocol goldens, event-heavy/concurrent performance fixture와 baseline receipt
- Out-of-Scope: production behavior 변경, 결함 구현, 최종 non-regression PASS, 정책에 없는 새 기능 또는 balance 변경
- Reviewed Current Behavior / Delta: 광범위한 current test seam은 있으나 formal feature manifest, slice differential gate와 event-heavy/concurrent same-run baseline이 없다. 이를 behavior 변경 없이 추가한다.
- Goal-Local Source of Truth / Owning Layer: `game_rulebook.md`의 current policy, Core/server/Unity current executable route와 tests; manifest/oracle owner는 test-support documentation layer이며 gameplay authority를 소유하지 않는다.
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-008`; manifest 누락 0, oracle이 production behavior를 변경하지 않음, baseline workload/toolchain/input identity 고정
- Boundary: documentation/test manifest, Core behavior fixtures, server/Unity protocol goldens, V4 corpus와 task-owned benchmark harness/output; production runtime mutation 없음
- Consumer Verification Boundary: workspace에서 확인된 Core tests와 server/Unity fixtures는 required; canonical Rulebook 기능은 manifest로 확인; Released/Deployed consumer는 존재하지 않으며 외부 unknown consumer를 완료 claim에 포함하지 않는다.
- Dependencies / Prerequisites: exact current Freeze/Plan/Receipt; predecessor Phase Goal 없음; production mutation 전에 완료 evidence 고정
- Inherited Baseline / Impact Map: shared reuse directive는 N/A. manifest, fixture, benchmark input/toolchain이 바뀌면 해당 oracle evidence 전체를 invalid하고 current pre-cutover target에서 다시 실행한다.
- Assigned Durable Integration Seam: `IVA-CRAR-01` baseline portion, `IVA-CRAR-09` pre-cutover portion; `IVA-CRAR-12` audit-local observation을 보조 evidence로만 사용
- Structural Completion / Legacy and Fallback Disposition: oracle 전용 production branch/fallback을 만들지 않고 current legacy surface는 제거 대상 ID로 명시만 한다. manifest 밖 feature와 snapshot 기대값 무근거 갱신은 허용하지 않는다.
- Canonical Documentation Obligation: `docs/game_rulebook.md`와 새 stable feature manifest의 양방향 locator를 유지하고 stale contract는 normative source로 승격하지 않는다.
- Migration/Compatibility/Rollout/Rollback: code/data migration 없음; baseline artifact 오류는 production rollback이 아니라 invalid evidence 폐기와 재생성으로 복구; release rollout N/A
- Known Unknown / Blocker / Stop: `GAP-CRAR-P2G20-001`; event-heavy boss 또는 64-session workload를 deterministic/repeatable하게 만들 수 없으면 Phase 2로 진행하지 않는다.
- Risk / Reviewer: inventory omission과 benchmark representativeness review required; reviewer 지정·승인 미수행

### P01 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| feature inventory | Rulebook→route→test reverse coverage audit | 100% mapped, duplicate/unknown/unapproved removal 0 |
| behavior/rejection/save/protocol oracle | current deterministic fixture/golden execution | non-zero execution과 exact evidence key 기록 |
| performance baseline | event-heavy boss와 64-session/8-way interleaved harness | raw samples, toolchain/input hash와 repeatability evidence 존재 |
| production non-mutation | scoped status/hash audit | runtime/data/server/Unity behavior delta 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | representative normal routes의 current oracle | 각 stable feature ID가 성공 결과와 연결됨 |
| `invalid_input` | `required` | current rejection matrix snapshot | invalid command/data의 error와 pre-state evidence 고정 |
| `failure_rollback` | `required` | six-command/StartNewGame current failure observation | partial state/event/durable delta를 oracle이 식별 가능 |
| `retry_idempotency` | `required` | same-basis retry fixture | retry 결과/side effect identity가 evidence에 포함 |
| `ordering_concurrency` | `required` | event-heavy same-bucket 및 64-session/8-way harness | ordering/workload input이 deterministic하고 재현 가능 |
| `boundary_empty_max` | `required` | empty inventory check와 1K/100K/history workload | unmapped 0, 최소/최대 workload evidence 존재 |
| `disconnect_restart_persistence` | `required` | reconnect, V4 continuation과 restart fixtures | current lifetime/save semantics가 cutover 전 고정 |
| `live_data` | `required` | current production RON loader corpus와 Rulebook mapping | 실제 loader data coverage와 invalid startup 결과 기록 |
| `producer_consumer` | `required` | Core→server→Unity current protocol goldens | mock-only가 아닌 접근 가능한 producer/consumer shape 고정 |
| `legacy_fallback_absence` | `required` | legacy/bypass inventory audit | 새 oracle-only fallback 0, 기존 제거 대상은 stable ID로 식별 |

## Phase Goal Member — `GOAL-CRAR-P02-VALIDATED-DATA`

- Goal ID: `GOAL-CRAR-P02-VALIDATED-DATA`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-02-VALIDATED-DATA`
- Assigned Final Execution Node Set: `WP-CRAR-04`
- Covered Policy Element Revision Ref Set: `PER-CRAR-005-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-005-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER의 exact acceptance; production bare/unvalidated construction 0, invalid input pre-authority recoverable failure, fixture/direct setter production-inaccessible
- Objective / Contract-Visible Result: authoring input을 recoverably validate한 opaque `ValidatedGameData`만 production Core를 구성하고 test fixture와 typed admin capability를 분리한다.
- User-Visible Problem: invalid 또는 fixture용 데이터가 production authority에 들어가 런 중 실패하거나 테스트 편의 API가 실제 게임 상태를 우회 변경할 수 있다.
- User-Visible Completion Condition Set: valid live data는 동일하게 load되고 invalid data는 authority 생성 전 명시적 오류가 되며, production build에서 bare builder/database, test currency/direct setter에 접근할 수 없다.
- In-Scope: RON loader/factory, `ValidatedGameData`, Core construction, fixture feature/`cfg(test)`, typed `AdminCommand`, compile-fail/public API audit
- Out-of-Scope: gameplay data 내용/balance 변경, lifecycle/transaction kernel, hot-reload 정책 신설, save/protocol schema 변경
- Reviewed Current Behavior / Delta: official validated loader와 public bare builder/database/Core constructor/direct setter가 공존한다. official path를 유일 production publication으로 만들고 capability를 분리한다.
- Goal-Local Source of Truth / Owning Layer: `src/game/data` validation/publication layer가 data validity authority, application construction root가 유일 consumer; admin mutation authority는 typed application command layer
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-005`, `PP-CRAR-P2G40-001 PHASE-02`; invalid input은 authority/mutation 전 failure, production fixture capability 0, valid load semantic parity
- Boundary: authoring RON→validation→opaque publication→Core constructor, test fixture builder와 admin command API; persistence/wire 미변경
- Consumer Verification Boundary: current Core construction call sites와 live RON loader는 required; test modules/fixtures는 test-only consumer; server/Unity는 validated Core projection의 간접 consumer이며 data constructor API를 직접 소비하지 않는다.
- Dependencies / Prerequisites: `GOAL-CRAR-P01-PRESERVATION-BASELINE` complete 및 current live-data oracle
- Inherited Baseline / Impact Map: reuse directive N/A. loader/data schema/fixture/admin API 변경은 live-load, invalid matrix, construction/API compile checks를 current target에서 재실행한다.
- Assigned Durable Integration Seam: `IVA-CRAR-04`
- Structural Completion / Legacy and Fallback Disposition: bare builder/database/Core construction과 production direct setter를 제거하고 alias, hidden fallback, test feature의 production enablement를 0으로 만든다.
- Canonical Documentation Obligation: `docs/data_loading_contract.md`의 publication/validation/capability 경계를 actual API와 일치하도록 갱신한다.
- Migration/Compatibility/Rollout/Rollback: live RON semantic migration 없음; invalid current data가 발견되면 fallback/default 없이 publication을 중단하고 authoring data를 고친다; release rollout N/A
- Known Unknown / Blocker / Stop: current production caller가 opaque validated type으로 표현할 수 없는 unchecked mutation에 의존하면 우회 API를 만들지 않고 upstream behavior evidence를 재검토한다.
- Risk / Reviewer: data validation, Rust visibility/feature gating과 admin authority review required

### P02 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| valid production publication | all current live RON load + Core construction | semantic parity와 single opaque input path |
| invalid authoring | malformed/cross-reference/range matrix | recoverable error, authority 생성·mutation 0 |
| capability isolation | compile-fail, public API와 feature audit | production bare builder/direct setter 접근 0 |
| typed admin mutation | AdminCommand focused tests | validation/transaction 경계를 우회하지 않음 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | current live RON load/Core construction | 모든 valid data가 opaque publication으로 동일 의미 생성 |
| `invalid_input` | `required` | malformed/reference/range invalid matrix | recoverable pre-authority failure와 경로가 명시됨 |
| `failure_rollback` | `required` | validation/publication fault injection | partial database/Core publication 0 |
| `retry_idempotency` | `required` | 동일 valid/invalid input 반복 load | valid semantic identity, invalid authority delta 0 |
| `ordering_concurrency` | `N/A` | — | 이 Goal은 startup 단일 publication만 소유하며 concurrent hot reload 정책을 추가하지 않음 |
| `boundary_empty_max` | `required` | empty catalog, missing ref, numeric min/max corpus | 승인된 empty만 통과하고 limit 위반은 명시적 failure |
| `disconnect_restart_persistence` | `N/A` | — | save/session lifetime을 변경하지 않고 startup publication까지만 소유 |
| `live_data` | `required` | production loader가 읽는 실제 RON 전체 | 실제 data success와 invalid startup rejection 모두 증명 |
| `producer_consumer` | `required` | actual loader→ValidatedGameData→Core round-trip | mock builder만으로 완료하지 않음 |
| `legacy_fallback_absence` | `required` | public API/feature/call-site audit | bare/unvalidated/fixture/direct-setter production path 0 |

## Phase Goal Member — `GOAL-CRAR-P03-APPLICATION-AUTHORITY`

- Goal ID: `GOAL-CRAR-P03-APPLICATION-AUTHORITY`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-03-APPLICATION-AUTHORITY`
- Assigned Final Execution Node Set: `AG-CRAR-01-APPLICATION-AUTHORITY` (`WP-CRAR-02`, `WP-CRAR-03`)
- Covered Policy Element Revision Ref Set: `PER-CRAR-002-R3-CODE-REVIEWED`, `PER-CRAR-003-R3-CODE-REVIEWED`, `PER-CRAR-004-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-003-R1`, `DR-CRAR-004-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER 3개의 exact Freeze acceptance; preservation-guarded boundary rewrite, 모든 `Err` exact equality, route별 one owner/no dual write
- Objective / Contract-Visible Result: closed lifecycle와 route registry를 단일 `validate → stage → commit → publish` command executor에 결속해 모든 mutating command의 authority와 failure semantics를 하나로 만든다.
- User-Visible Problem: 실패한 명령이 일부 상태·event·revision을 남기고 lifecycle option 조합과 broad router가 잘못된 단계의 mutation 또는 competing writer를 허용한다.
- User-Visible Completion Condition Set: 모든 mutating route가 정확히 한 owner/commit port를 가지며 invalid phase와 모든 `Err`는 command 전 authoritative/durable state와 exact-equal하고 retry 결과가 같은 basis에서 일관된다.
- In-Scope: `GameCoreState` lifecycle sum type/context, application services, route-owner registry, staged draft/commit token, accepted/recovery-pending outcome, six battle commands와 StartNewGame matrix
- Out-of-Scope: battle algorithm rewrite, private kernel internals, protocol v1, history/checkpoint format과 새 gameplay 기능
- Reviewed Current Behavior / Delta: parallel optionals와 broad router가 lifecycle을 convention으로 유지하고 six battle commands는 commit 후 cursor projection error, StartNewGame은 partial mutation 가능성이 있다. root authority와 outcome typing을 atomic group으로 교체한다.
- Goal-Local Source of Truth / Owning Layer: application command executor와 closed lifecycle state가 mutating route authority; domain services는 staged draft만 만들고 durable publication은 typed commit port만 소유
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-002..004`, `AG-CRAR-01-APPLICATION-AUTHORITY`; `Err ⇒ exact pre-command equality`, route owner exactly one, old/new simultaneous writer 0, gameplay algorithm differential pass
- Boundary: Core world/application state, mutating command routing, event/revision/cursor/durable-intent staging과 server caller outcome projection
- Consumer Verification Boundary: Core direct callers/tests와 server player actor handlers는 required accessible consumers; Unity는 protocol adapter 후행 consumer라 current command result shape parity만 oracle로 보존하고 v1 cutover는 P07이 소유한다.
- Dependencies / Prerequisites: P01, P02 complete; Atomic Group member를 분리 실행하지 않음
- Inherited Baseline / Impact Map: reuse directive N/A. lifecycle/router/executor/state/event/revision/durable-intent 변경은 six-command, StartNewGame, invalid-phase, retry와 server accepted-outcome seams을 모두 current target에서 revalidate한다.
- Assigned Durable Integration Seam: `IVA-CRAR-02`, `IVA-CRAR-03`
- Structural Completion / Legacy and Fallback Disposition: broad old router, parallel optional lifecycle writer, post-commit `Err`, direct mutation/commit bypass와 route별 dual writer를 제거한다. 미래 전용 transaction trait만 남기는 완료는 금지한다.
- Canonical Documentation Obligation: `docs/core_runtime_contract.md`에 command outcome, lifecycle owner, prevalidation/stage/commit/publish와 durable-intent 경계를 actual API 기준으로 갱신한다.
- Migration/Compatibility/Rollout/Rollback: route별 hard switch를 Atomic Group 안에서 완료하며 dual-run rollback은 금지; 실패 시 group 전체 변경을 pre-cutover 상태로 되돌리고 partial owner selection을 publish하지 않음
- Known Unknown / Blocker / Stop: 어떤 route가 staged draft/closed lifecycle로 표현되지 않거나 post-commit 실패를 `Err`로 유지해야 한다면 bypass하지 않고 중단한다.
- Risk / Reviewer: root transaction, durable intent, lifecycle typestate와 server outcome review required

### P03 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| exact rejection equality | authoritative snapshot/event/revision/durable-intent matrix | 모든 `Err` before=after exact |
| retry/commit outcome | same-basis retry와 post-commit fault injection | retry deterministic, committed failure는 accepted/recovery-pending |
| single route authority | registry reverse coverage와 API/call-site audit | mutating route 100% mapped, missing/duplicate/bypass 0 |
| lifecycle closure | invalid transition/constructor tests | impossible state와 post-mutation phase rejection 0 |
| algorithm preservation | P01 slice differential | approved boundary 밖 semantic delta 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | all mutating route accepted matrix | stage/commit/publish 결과와 previous user-visible semantics 일치 |
| `invalid_input` | `required` | invalid command/lifecycle/target matrix | prevalidation error와 exact authoritative equality |
| `failure_rollback` | `required` | validation/staging/commit-boundary fault injection | commit 전 failure delta 0, commit 후 `Err` 0 |
| `retry_idempotency` | `required` | same revision/durable basis retry and duplicate | 동일 authoritative result, duplicate durable intent 없음 |
| `ordering_concurrency` | `required` | competing route/revision and same-boundary event tests | one commit order, stale revision pre-reject, dual writer 0 |
| `boundary_empty_max` | `required` | empty lifecycle payload, route registry completeness, command limits | missing owner/empty invalid payload 명시적 rejection |
| `disconnect_restart_persistence` | `required` | durable-intent fault/restart and accepted recovery fixtures | rejected intent 0, committed outcome recoverable/idempotent |
| `live_data` | `N/A` | — | validated data를 소비하지만 이 Goal은 RON schema/publication을 변경하지 않음 |
| `producer_consumer` | `required` | Core executor→server actor actual outcome round-trip | direct caller와 server가 same accepted/error semantics 사용 |
| `legacy_fallback_absence` | `required` | router/owner/direct mutation call-site audit | old broad writer, parallel optionals, bypass commit/fallback 0 |

## Phase Goal Member — `GOAL-CRAR-P04-BATTLE-KERNEL`

- Goal ID: `GOAL-CRAR-P04-BATTLE-KERNEL`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-04-BATTLE-KERNEL`
- Assigned Final Execution Node Set: `WP-CRAR-05`
- Covered Policy Element Revision Ref Set: `PER-CRAR-006-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-006-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER exact acceptance; production-public lifecycle bypass mutation 0, non-finite exact rejection, finite ordering/determinism 보존
- Objective / Contract-Visible Result: battle authority를 private `BattleKernel`과 scoped command/query ports로 닫고 position/speed/time input을 finite validated types로 제한한다.
- User-Visible Problem: public unit/event fields와 event injection이 battle lifecycle을 우회할 수 있고 NaN/Inf가 ordering/heap을 오염시켜 전투가 비결정적이거나 복구 불가능해질 수 있다.
- User-Visible Completion Condition Set: production API로 kernel authority를 직접 변경할 수 없고 모든 NaN/±Inf 입력은 mutation 전에 거부되며 finite boss/movement/skill/battle 결과는 preservation oracle과 일치한다.
- In-Scope: `BattleKernel` visibility, command/query/mutation ports, setup/command/event/position/speed/time finite types와 constructors, heap/log ordering, deterministic corpus
- Out-of-Scope: gameplay formula·AI·movement algorithm 재설계, cross-platform bitwise physics guarantee, history/protocol/save format
- Reviewed Current Behavior / Delta: public kernel/unit fields와 event injection surface가 있고 float event ordering은 finite premise를 신뢰한다. finite algorithm은 유지하면서 construction/mutation authority만 닫는다.
- Goal-Local Source of Truth / Owning Layer: private `BattleKernel`이 battle state/event authority, application lifecycle owner가 유일 command producer; validated finite newtype constructor가 numeric boundary authority
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-006`, `PP-CRAR-P2G40-001 PHASE-04`; public bypass 0, non-finite pre-mutation equality, finite determinism/oracle parity
- Boundary: combat setup, battle commands/events, unit state, movement/target/skill/boss runtime input과 query projection; application→kernel port
- Consumer Verification Boundary: Core world/application and battle tests are required actual consumers; server/Unity는 public projection consumer이며 kernel fields 직접 접근을 허용하지 않는다. inaccessible external consumer는 없음.
- Dependencies / Prerequisites: P01~P03 complete; validated data와 application executor를 우회하지 않음
- Inherited Baseline / Impact Map: reuse directive N/A. kernel visibility/ports/finite constructors/order changes는 setup, command, movement, targeting, skill, boss, event-log와 non-finite matrix를 current target에서 revalidate한다.
- Assigned Durable Integration Seam: `IVA-CRAR-05`
- Structural Completion / Legacy and Fallback Disposition: public mutable field/event injection/raw float construction aliases를 제거하고 test helper도 production capability를 갖지 않는다. old/new kernel dual path 0.
- Canonical Documentation Obligation: `docs/core_runtime_contract.md`와 `docs/skill_target_contract.md`의 battle authority/numeric input 경계를 actual private ports와 finite types에 맞춘다.
- Migration/Compatibility/Rollout/Rollback: process-local battle state migration 없음; port cutover는 current battle construction과 함께 완료하고 public legacy adapter를 rollback용으로 남기지 않음
- Known Unknown / Blocker / Stop: named finite algorithm이 private port로 이동하며 semantic parity를 유지하지 못하거나 non-finite가 validated boundary 이전에 authority에 들어오면 중단한다.
- Risk / Reviewer: battle lifecycle, unsafe/public API, float total ordering, boss/movement/skill regression review required

### P04 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| private authority | compile-fail/public API/call-site audit | production mutation bypass 0 |
| non-finite rejection | setup/command/event/position/speed/time NaN/±Inf matrix | error와 exact pre-command authority equality |
| finite determinism | fixed seed/order boss/movement/skill corpus | repeated result/event order identical |
| gameplay preservation | P01 differential | approved algorithm change 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | finite setup/command/boss/movement/skill scenarios | current user-visible battle semantics 보존 |
| `invalid_input` | `required` | exhaustive NaN/±Inf and invalid authority input matrix | mutation 전 typed error와 exact equality |
| `failure_rollback` | `required` | port validation and staged command fault injection | partial unit/event/heap/log mutation 0 |
| `retry_idempotency` | `required` | rejected command retry and committed command ID tests | rejection deterministic, duplicate commit 없음 |
| `ordering_concurrency` | `required` | same-time events/projectiles/movement ordering corpus | finite total order와 repeat determinism 유지 |
| `boundary_empty_max` | `required` | empty battlefield/targets, numeric extrema와 unit/event limits | panic/non-finite/overflow 없이 명시 결과 |
| `disconnect_restart_persistence` | `N/A` | — | kernel은 process-local active battle만 소유하고 reconnect/history/save lifetime은 후행 Goal 소유 |
| `live_data` | `required` | actual validated battle/boss/skill data setup | mock-only가 아닌 live definitions가 finite port를 통과 |
| `producer_consumer` | `required` | application command→private kernel→public projection round-trip | caller가 public field를 우회하지 않음 |
| `legacy_fallback_absence` | `required` | API/call-site/test-feature audit | public mutation/raw float/event injection/dual kernel path 0 |

## Phase Goal Member — `GOAL-CRAR-P05-SESSION-HISTORY-RESULT`

- Goal ID: `GOAL-CRAR-P05-SESSION-HISTORY-RESULT`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-05-SESSION-HISTORY-RESULT`
- Assigned Final Execution Node Set: `WP-CRAR-06`
- Covered Policy Element Revision Ref Set: `PER-CRAR-009-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-008-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER exact acceptance; bounded active memory/reconnect work, window delta identity, overflow/expiry authority/result 독립, archive 없이 continuation
- Objective / Contract-Visible Result: reconnect history를 indexed bounded window로 만들고 active-session, durable result/receipt와 optional debug archive의 lifetime/authority를 분리한다.
- User-Visible Problem: unbounded event log와 linear reconnect scan이 장시간/다중 전투에서 메모리와 복구 시간을 키우고 terminal full log가 gameplay result authority와 결합한다.
- User-Visible Completion Condition Set: history는 `8192 events OR 4 MiB OR 180 battle seconds` 중 먼저 닫히는 cap 안이며 window 내 reconnect는 동일 delta, floor 이전 cursor는 full resync를 받고 overflow/archive expiry 뒤에도 result와 continuation이 변하지 않는다.
- In-Scope: ring/deque sequence index, cursor floor/full resync, active-session retention, durable `BattleResultRecord/Receipt`, default-off `64 MiB/24h` archive port와 expiry
- Out-of-Scope: full deterministic replay 보장, archive product UI, protocol v1 envelope, checkpoint schema, performance 최종 PASS
- Reviewed Current Behavior / Delta: unbounded `Vec`와 linear scan, active clone의 history 포함, terminal compressed full log가 공존한다. indexed bounded window와 독립 durable record로 교체한다.
- Goal-Local Source of Truth / Owning Layer: active battle session이 bounded reconnect window authority, durable result store가 terminal outcome/receipt authority, external archive는 diagnostic-only non-authority
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-009`, `IVA-CRAR-06`; caps hard bound, sequence identity, cursor-before-floor full resync, overflow/expiry가 authority/result/continuation을 바꾸지 않음
- Boundary: kernel event sequence→active history index→server reconnect projection, terminal result/receipt persistence와 external archive port
- Consumer Verification Boundary: current server reconnect/actor state와 Core result consumer는 required; Unity current/full-resync behavior는 P07 actual consumer cutover에서 재검증; archive consumer는 optional이며 부재가 완료를 막지 않는다.
- Dependencies / Prerequisites: P01~P04 complete; final kernel event sequence와 application accepted-commit semantics current
- Inherited Baseline / Impact Map: reuse directive N/A. history/index/cursor/result/archive/store 변경은 delta/full-resync, caps/scaling, result restart/idempotency, overflow/expiry continuation seams을 current target에서 revalidate한다.
- Assigned Durable Integration Seam: `IVA-CRAR-06` Phase-local portion
- Structural Completion / Legacy and Fallback Disposition: unbounded Vec/full-log authority와 linear tail scan을 제거한다. archive/full replay fallback을 gameplay continuation 또는 result source로 사용하지 않는다.
- Canonical Documentation Obligation: `docs/core_runtime_contract.md`와 `game_server/docs/unity_bridge_protocol.md`의 reconnect cursor/full-resync/result receipt lifetime을 actual boundary에 맞춘다.
- Migration/Compatibility/Rollout/Rollback: active in-memory session migration N/A; durable result에는 explicit version/identity를 사용하고 archive는 default-off; current adapter는 Phase 완료 시 reader/writer 함께 유효해야 함
- Known Unknown / Blocker / Stop: cap 내에서 required reconnect semantic을 제공할 수 없거나 durable result가 full archive에 의존하면 중단한다. final latency/memory PASS는 P08 owner다.
- Risk / Reviewer: retention math, cursor identity, persistence idempotency, reconnect transport와 archive authority review required

### P05 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| bounded window | count/byte/battle-time property and long-run tests | 세 cap 중 first-close, active memory/work bounded |
| reconnect identity | within-window delta and before-floor full-resync fixtures | exact sequence/no gap/no duplicate, bounded lookup |
| durable result/receipt | terminal/retry/restart/idempotency tests | archive/history와 독립된 exactly-one result |
| overflow/expiry continuation | forced eviction/archive absent/expiry scenarios | authority/result/next gameplay 동일 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | normal delta reconnect와 terminal receipt | identical delta/result/continuation |
| `invalid_input` | `required` | malformed/future/stale cursor와 archive config matrix | bounded full-resync 또는 explicit error, authority delta 0 |
| `failure_rollback` | `required` | result store/archive write fault injection | gameplay result atomic; archive failure non-authoritative |
| `retry_idempotency` | `required` | duplicate reconnect/result commit/receipt request | no duplicate event/result/durable write |
| `ordering_concurrency` | `required` | concurrent event append/reconnect/terminal settlement | monotonic sequence, one terminal owner, no torn window |
| `boundary_empty_max` | `required` | empty history, exact floor, 8192/4MiB/180s and archive caps | off-by-one/overflow 없이 first-close와 full resync 정확 |
| `disconnect_restart_persistence` | `required` | disconnect/reconnect, process restart와 result reload | active battle lifetime과 durable result lifetime 분리 |
| `live_data` | `N/A` | — | 이 Goal은 RON/data publication을 변경하지 않으며 runtime event/result만 소유 |
| `producer_consumer` | `required` | kernel sequence→server reconnect/result actual adapter | mock history끼리의 일치로 완료하지 않음 |
| `legacy_fallback_absence` | `required` | storage/call-site/complexity audit | unbounded/full-log authority, linear tail scan과 archive fallback 0 |

## Phase Goal Member — `GOAL-CRAR-P06-CHECKPOINT-MIGRATION`

- Goal ID: `GOAL-CRAR-P06-CHECKPOINT-MIGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-06-CHECKPOINT-MIGRATION`
- Assigned Final Execution Node Set: `WP-CRAR-08`
- Covered Policy Element Revision Ref Set: `PER-CRAR-010-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-009-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER exact acceptance; canonical V4 semantic import, V5 target-only writes, continuation differential와 certification 전 reader-removal rejection
- Objective / Contract-Visible Result: Core checkpoint와 server store를 V5 writer로 전환하고 explicit V4 importer/oracle과 migration certification gate를 유지한다.
- User-Visible Problem: runtime state 구조 변경이 기존 V4 checkpoint의 identity/state/continuation 의미를 잃거나 새 코드가 여러 format을 암묵적으로 쓰고 old reader를 너무 일찍 제거할 수 있다.
- User-Visible Completion Condition Set: canonical V4 save가 같은 identity/state/continuation으로 import되고 새 저장은 V5만 생성하며 V4/V5 continuation differential이 통과하고 certification 전 V4 reader 제거가 실패한다.
- In-Scope: Core V5 codec/state identity, V4 importer, server envelope/checksum/store, canonical corpus, continuation oracle와 removal guard
- Out-of-Scope: released/production save migration, V4 writer 유지, active battle persistence, protocol v1 wire schema와 manual save-conversion UI
- Reviewed Current Behavior / Delta: Core payload/server envelope는 exact V4이고 strict reader/rejection/continuation tests가 있다. 이를 oracle로 보존하면서 V5 target writer와 explicit importer를 추가한다.
- Goal-Local Source of Truth / Owning Layer: Core V5 codec/state schema가 target semantic authority, server store는 version/checksum envelope authority, V4 importer는 certification까지 compatibility boundary
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-010`, `IVA-CRAR-07`; V4 semantic identity, V5-only write, corruption rejection, exact continuation, premature reader removal gate
- Boundary: Core state export/import, save identity/version/checksum, server active-run store envelope와 canonical V4/V5 fixtures
- Consumer Verification Boundary: Core world continuation tests와 server store are required accessible producer/consumer; Released/Deployed save population은 evidence-backed N/A; Unity는 save payload를 직접 읽지 않고 protocol outcome만 소비한다.
- Dependencies / Prerequisites: P01~P03 complete; P04/P05와 independent semantics지만 ordered Phase Plan에 따라 current state/result types를 사용
- Inherited Baseline / Impact Map: reuse directive N/A. codec/state/envelope/store/fixture 변경은 V4 import, V5 round-trip/corruption, continuation, target-only writer와 reader-removal guard를 current target에서 revalidate한다.
- Assigned Durable Integration Seam: `IVA-CRAR-07`
- Structural Completion / Legacy and Fallback Disposition: V4 writer, implicit version guessing, union write와 silent default fallback을 제거한다. V4 reader는 승인된 migration boundary이므로 certification 전 legacy로 간주해 제거하지 않는다.
- Canonical Documentation Obligation: `docs/core_runtime_contract.md`와 server persistence contract/documentation에 V4 importer/V5 writer/checksum/certification owner를 기록한다.
- Migration/Compatibility/Rollout/Rollback: pre-release development V4 corpus만 applicable; rollback은 V5 write를 V4 write로 되돌리는 dual path가 아니라 implementation change rollback이며 이미 생성한 V5 fixture는 target reader로 보존
- Known Unknown / Blocker / Stop: V4 identity/continuation을 explicit importer로 보존하지 못하거나 target state가 의미 결정을 요구하면 migration default를 만들지 않고 중단한다.
- Risk / Reviewer: serialization identity, checksum/version dispatch, migration corpus와 persistence atomicity review required

### P06 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| V4 semantic import | canonical corpus identity/state assertions | 100% import와 semantic loss 0 |
| V5 target-only writer | write/version/checksum inspection | 새 write V5 only, V4 write/guessing 0 |
| continuation parity | V4-import vs V5-native old/new differential | subsequent command/result identity 유지 |
| removal certification | build/test negative gate | certification evidence 전 V4 reader removal 거부 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | V4 import and V5 native round-trip/continue | identity/state/continuation 의미 일치 |
| `invalid_input` | `required` | bad version/checksum/truncation/shape corpus | explicit recoverable failure, state/usage delta 0 |
| `failure_rollback` | `required` | partial store write/import failure injection | no torn save or partially loaded authority |
| `retry_idempotency` | `required` | repeated import/export/load and duplicate request | deterministic bytes/semantic state, usage double-consume 0 |
| `ordering_concurrency` | `required` | competing save/load and stale revision fixtures | single store authority와 deterministic conflict rejection |
| `boundary_empty_max` | `required` | empty/minimal/max-size payload and field limits | supported boundary round-trip, oversize/corrupt explicit rejection |
| `disconnect_restart_persistence` | `required` | server restart/store reload/explicit load continuation | durable envelope와 Core state 정확 복원 |
| `live_data` | `N/A` | — | checkpoint migration은 RON authoring/publication을 변경하지 않음 |
| `producer_consumer` | `required` | Core codec→server envelope/store→Core import round-trip | producer/actual store consumer same version/identity |
| `legacy_fallback_absence` | `required` | writer/reader/version-dispatch audit | V4 write, guessing/default/dual write 0; approved V4 reader만 gate 하에 존재 |

## Phase Goal Member — `GOAL-CRAR-P07-PROTOCOL-CUTOVER`

- Goal ID: `GOAL-CRAR-P07-PROTOCOL-CUTOVER`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-07-PROTOCOL-CUTOVER`
- Assigned Final Execution Node Set: `AG-CRAR-02-PROTOCOL-CUTOVER` (`WP-CRAR-09`, `WP-CRAR-10`)
- Covered Policy Element Revision Ref Set: `PER-CRAR-007-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-007-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER exact acceptance; 모든 external shape의 versioned exhaustive mapping/parity, server/Unity same-version hard cutover, current legacy-free consumer failure 0
- Objective / Contract-Visible Result: Core domain과 독립된 protocol v1 DTO/envelope와 exhaustive server mapper를 만들고 Unity의 raw message consumers를 같은 atomic group에서 typed v1로 hard cutover한다.
- User-Visible Problem: unversioned Core domain serde와 Unity raw `JObject` interpretation이 결합되어 domain refactor가 wire를 조용히 깨거나 server와 client가 서로 다른 의미를 해석할 수 있다.
- User-Visible Completion Condition Set: command/result/error/snapshot/reconnect/save outcome이 v1 exhaustive mapper와 semantic parity fixture를 가지며 server와 Unity current consumer가 같은 version으로 동작하고 legacy wire/parser branch가 없어도 failure 0이다.
- In-Scope: protocol v1 envelope/DTO/version owner, server mapper/ws command/error/result/snapshot/reconnect adapters, Unity `GameWsClient`/command types/snapshot store/battle state와 raw consumer migration, golden fixtures
- Out-of-Scope: dual-stack release compatibility, production deployed clients, gameplay/domain algorithm과 balance, Unity visual presentation redesign
- Reviewed Current Behavior / Delta: Core domain serde가 unversioned server messages에 직접 들어가고 Unity central raw parser/store에는 62개 reverse-reference consumer가 있다. producer/consumer를 one atomic schema cutover로 분리한다.
- Goal-Local Source of Truth / Owning Layer: protocol v1 schema와 exhaustive mapper가 wire semantic authority, Core domain은 producer source이지만 wire owner가 아니며 Unity typed adapter가 current consumer authority
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-007`, `AG-CRAR-02-PROTOCOL-CUTOVER`, `IVA-CRAR-08`; same version, exhaustive mapping, semantic parity, current consumer failure/legacy path 0
- Boundary: Core public projection→server v1 DTO/envelope/ws→Unity typed deserializer/store/battle readers and command→server error/result round-trip
- Consumer Verification Boundary: server and all accessible Unity runtime/tests are required current consumers; canonical Unity contract는 stale input이므로 actual code와 새 v1 fixture에 맞춰 갱신해야 함; Released/Deployed consumers는 존재하지 않음.
- Dependencies / Prerequisites: P01~P06 complete; Atomic Group producer/consumer를 분리 실행하지 않음
- Inherited Baseline / Impact Map: reuse directive N/A. schema/mapper/ws/fixture/Unity parser-store-consumer 변경은 all command/result/error/snapshot/reconnect/save parity와 EditMode/PlayMode tests를 current paired builds에서 revalidate한다.
- Assigned Durable Integration Seam: `IVA-CRAR-08` Phase-local portion
- Structural Completion / Legacy and Fallback Disposition: direct domain serde, unversioned envelope, raw central interpretation, legacy server/Unity branches와 schema alias/dual stack을 atomic group 안에서 제거한다.
- Canonical Documentation Obligation: `game_server/docs/unity_bridge_protocol.md`와 `/mnt/f/unity projects/ark/docs/unity_core_contract.md`를 protocol v1 actual schema/owner에 맞춰 갱신하고 stale 표기를 해소한다.
- Migration/Compatibility/Rollout/Rollback: pre-release coordinated hard cutover; released/deployed compatibility N/A. rollback은 server와 Unity group 전체를 같은 pre-cutover revision으로 되돌릴 때만 유효하며 mixed version/dual stack 금지
- Known Unknown / Blocker / Stop: 62개 current Unity consumer 중 하나라도 typed exhaustive mapping/runner evidence를 확보하지 못하면 legacy fallback 없이 중단한다.
- Risk / Reviewer: cross-repository schema, mapper exhaustiveness, websocket ordering/reconnect와 Unity consumer review required

### P07 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| exhaustive v1 mapping | compile-exhaustive mapper and golden corpus | external variant 100%, unmapped/default 0 |
| semantic parity | old oracle↔v1 Core/server/Unity round-trip | command/result/error/snapshot 의미 delta 0 |
| coordinated consumer cutover | paired server + Unity EditMode/PlayMode/reconnect suites | same version과 current consumer failure 0 |
| legacy removal | server/Unity call-site/schema audit | direct domain/raw/legacy/dual stack path 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | all normal command/result/snapshot/reconnect round-trips | paired server/Unity v1 user-visible parity |
| `invalid_input` | `required` | unknown version/type, malformed fields와 invalid command corpus | explicit versioned error, Core authority delta 0 |
| `failure_rollback` | `required` | disconnect/serialization/send failure around commit | pre-commit failure no mutation, committed outcome recoverable |
| `retry_idempotency` | `required` | duplicate command/reconnect/result delivery | one authoritative commit and deterministic projection |
| `ordering_concurrency` | `required` | revision ordering, reconnect during events와 concurrent messages | monotonic revision/sequence, stale input rejection |
| `boundary_empty_max` | `required` | empty collections, missing optionals, max snapshot/history payload | exact typed shape, limit error/full-resync without fallback |
| `disconnect_restart_persistence` | `required` | Unity reconnect, server actor restart/result/save outcome fixtures | bounded resync와 durable outcome semantics 유지 |
| `live_data` | `N/A` | — | wire schema/consumer를 변경하며 RON authoring/publication은 변경하지 않음 |
| `producer_consumer` | `required` | actual Core producer→server mapper/ws→Unity runtime consumer | mock-only golden이 아닌 paired consumer pass |
| `legacy_fallback_absence` | `required` | both repository static/call-site/schema audit | unversioned/direct domain/raw JObject/legacy/alias/dual stack 0 |

## Phase Goal Member — `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION`

- Goal ID: `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION`
- Goal Kind / Phase ID: `phase / PHASE-CRAR-08-PERFORMANCE-INTEGRATION`
- Assigned Final Execution Node Set: `WP-CRAR-07`, `WP-CRAR-11`
- Covered Policy Element Revision Ref Set: `PER-CRAR-008-R3-CODE-REVIEWED`
- Covered Decision Revision Ref Set: `DR-CRAR-002-R1`
- Derived Observable Policy Acceptance Coverage: 위 PER exact acceptance와 `WP-CRAR-11` validation-only로 모든 Freeze PER의 final acceptance; same-workload non-regression, numeric cap/permanent guard, 10/10 semantic completion
- Objective / Contract-Visible Result: pre-cutover baseline과 final target을 동일 workload에서 비교해 성능 guard를 고정하고 10개 정책 acceptance, full Core/server/Unity validation과 legacy/bypass absence를 통합 증명한다.
- User-Visible Problem: 구조 결함을 고친 뒤 기능은 맞아도 battle/session/reconnect/serialization이 무거워지거나 일부 legacy authority와 consumer가 남아 전체 rewrite가 불완전할 수 있다.
- User-Visible Completion Condition Set: event-heavy/64-session workload의 핵심 지표가 baseline보다 악화되지 않고 모든 numeric cap이 permanent CI guard로 통과하며 10/10 policy acceptance, feature inventory 100%, full Core/server/Unity suites와 legacy/bypass/dual path 0이 증명된다.
- In-Scope: touched-state staging/index optimization, interleaved benchmark/CI guard, allocation/RSS/history/serialization measurement, final acceptance evidence index, full regression/integration and static/API/docs audit
- Out-of-Scope: product SLA 약속, production deployment/promotion, 임시 정책 기능, balance/content 최적화와 승인되지 않은 algorithm rewrite
- Reviewed Current Behavior / Delta: ordinary work는 가볍지만 tail scan/full-session staging이 retained history에 비례하고 event-heavy/concurrent baseline은 P2G-20 gap이다. final implementation에 indexed/touched-state optimization과 permanent evidence를 추가한다.
- Goal-Local Source of Truth / Owning Layer: benchmark contract/CI guard가 engineering cap authority, exact Final Freeze acceptance와 P01 feature manifest가 semantic completion authority; gameplay runtime은 수치를 만족하기 위해 의미를 바꾸지 않음
- Goal-Local Invariant Set / Source Refs: `PE-CRAR-008`, `IVA-CRAR-01/02/06/08/09/10/11`; current-relative no-regression, TD-010 numeric caps, 10/10 acceptance와 legacy/bypass 0
- Boundary: Core command/tick/snapshot/checkpoint/history, server session/serialization/result, Unity typed consumer, benchmark harness/CI와 final documentation/evidence index
- Consumer Verification Boundary: all accessible Core/server/Unity current consumers and test runners are required; Released/Deployed environment는 N/A; inaccessible unknown consumer를 PASS로 추정하지 않는다.
- Dependencies / Prerequisites: P01~P07 complete와 각 intermediate audit current; `WP-CRAR-07` performance evidence 후 `WP-CRAR-11` final completion claim
- Inherited Baseline / Impact Map: reuse directive N/A. P01 raw workload identity만 comparison basis로 사용한다. benchmark/harness/schema/history/serialization/test 변경은 관련 sample과 acceptance evidence를 invalid하고 current integrated target에서 재실행한다.
- Assigned Durable Integration Seam: `IVA-CRAR-01`, `IVA-CRAR-02`, `IVA-CRAR-06`, `IVA-CRAR-08` final portions, `IVA-CRAR-09`, `IVA-CRAR-10`; `IVA-CRAR-11` existing runner 실행 owner
- Structural Completion / Legacy and Fallback Disposition: 최종 static/API/call-site audit에서 unvalidated/broad-router/public-kernel/unbounded-history/V4-writer/unversioned-wire 및 test-only production behavior를 0으로 만든다. benchmark 우회나 slow fallback을 두지 않는다.
- Canonical Documentation Obligation: `game_rulebook.md`, Core/data/skill target contracts, server bridge와 Unity contract를 final actual authority/version/lifetime/limits에 맞추고 stale/contradictory normative statement를 0으로 만든다.
- Migration/Compatibility/Rollout/Rollback: V4 reader certification과 protocol v1 pre-release hard cutover 조건을 재검증; deployment/production rollout은 수행하지 않음. cap 실패 시 semantic 완화를 승인하지 않고 owning Goal로 돌아가 최적화 후 전체 affected evidence 재실행
- Known Unknown / Blocker / Stop: event-heavy/concurrent baseline과 final comparison이 동일 evidence key로 성립하지 않거나 Unity/full runner를 실행하지 못하거나 핵심 지표가 baseline/cap을 넘으면 complete로 판정하지 않는다.
- Risk / Reviewer: performance statistics/measurement noise, cross-phase acceptance, legacy removal, migration/protocol consumer review required

### P08 Validation Plan / Completion Matrix

| Acceptance | Direct validation | Completion condition |
| --- | --- | --- |
| relative performance | P01 vs final same-run interleaved workload | 핵심 latency/memory/history/serialization regression 0 |
| absolute engineering caps | permanent benchmark/CI guard | command/tick `<0.5ms p95`; snapshot `<0.5ms/<64KiB`; checkpoint `<0.1ms/<32KiB`; last-10 `<=10us p95`, 1K→100K `<=2x`; staging `<0.5ms p95` |
| semantic completion | 10/10 acceptance evidence index와 P01 differential | missing/failed/unknown acceptance 0, inventory 100% |
| integrated consumers | full Core/server/Unity suites | non-zero execution, zero failure on current integrated target |
| structural completion | final API/static/call-site/docs audit | legacy/bypass/dual writer/fallback/stale normative contract 0 |

### Validation Dimension Matrix

| Dimension | Disposition | Direct seam/evidence contract | Completion requirement or N/A reason |
| --- | --- | --- | --- |
| `success` | `required` | full representative gameplay and all current suites | 10/10 acceptance와 user-visible feature parity |
| `invalid_input` | `required` | consolidated data/command/non-finite/version/cursor/save invalid matrix | exact rejection, authority delta 0, no panic/default |
| `failure_rollback` | `required` | cross-boundary fault injection for command/store/ws/archive | no partial authority; committed result recoverable |
| `retry_idempotency` | `required` | duplicate command/reconnect/result/save/load matrix | exactly-one durable/authority effect across all boundaries |
| `ordering_concurrency` | `required` | event-heavy same-bucket and 64-session/8-way workload | deterministic order, no race/dual writer and cap PASS |
| `boundary_empty_max` | `required` | all Phase min/empty/max/overflow cases plus performance scale | hard caps와 exact boundary semantics 모두 PASS |
| `disconnect_restart_persistence` | `required` | reconnect, process restart, V4/V5 continuation와 durable result | lifetime/authority/migration invariants 통합 PASS |
| `live_data` | `required` | full actual production RON load and representative gameplay | invalid startup rejection와 valid content semantics 유지 |
| `producer_consumer` | `required` | actual Core→server→Unity end-to-end and store round-trips | accessible current consumers 100%, mock-only completion 금지 |
| `legacy_fallback_absence` | `required` | final API/static/call-site/schema/docs audit | 모든 제거 대상 legacy/bypass/alias/dual path/fallback 0 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref Set | Primary Phase ID | Executable Goal ID |
| --- | --- | --- | --- | --- |
| `PE-CRAR-001` | `PER-CRAR-001-R3-CODE-REVIEWED` | `DR-CRAR-001-R1` | `PHASE-CRAR-01-PRESERVATION-BASELINE` | `GOAL-CRAR-P01-PRESERVATION-BASELINE` |
| `PE-CRAR-002` | `PER-CRAR-002-R3-CODE-REVIEWED` | `DR-CRAR-003-R1` | `PHASE-CRAR-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-003` | `PER-CRAR-003-R3-CODE-REVIEWED` | `DR-CRAR-004-R1` | `PHASE-CRAR-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-004` | `PER-CRAR-004-R3-CODE-REVIEWED` | `DR-CRAR-003-R1` | `PHASE-CRAR-03-APPLICATION-AUTHORITY` | `GOAL-CRAR-P03-APPLICATION-AUTHORITY` |
| `PE-CRAR-005` | `PER-CRAR-005-R3-CODE-REVIEWED` | `DR-CRAR-005-R1` | `PHASE-CRAR-02-VALIDATED-DATA` | `GOAL-CRAR-P02-VALIDATED-DATA` |
| `PE-CRAR-006` | `PER-CRAR-006-R3-CODE-REVIEWED` | `DR-CRAR-006-R1` | `PHASE-CRAR-04-BATTLE-KERNEL` | `GOAL-CRAR-P04-BATTLE-KERNEL` |
| `PE-CRAR-007` | `PER-CRAR-007-R3-CODE-REVIEWED` | `DR-CRAR-007-R1` | `PHASE-CRAR-07-PROTOCOL-CUTOVER` | `GOAL-CRAR-P07-PROTOCOL-CUTOVER` |
| `PE-CRAR-008` | `PER-CRAR-008-R3-CODE-REVIEWED` | `DR-CRAR-002-R1` | `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | `GOAL-CRAR-P08-PERFORMANCE-INTEGRATION` |
| `PE-CRAR-009` | `PER-CRAR-009-R3-CODE-REVIEWED` | `DR-CRAR-008-R1` | `PHASE-CRAR-05-SESSION-HISTORY-RESULT` | `GOAL-CRAR-P05-SESSION-HISTORY-RESULT` |
| `PE-CRAR-010` | `PER-CRAR-010-R3-CODE-REVIEWED` | `DR-CRAR-009-R1` | `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | `GOAL-CRAR-P06-CHECKPOINT-MIGRATION` |

## Goal Authoring Summary

- Goal Set Ref: `GS-CRAR-P2G50-001`
- Goal Quality Contract Version: `2`
- Master Goal Members: `1`
- Phase Goal Members: `8`
- Phase-to-Goal Mapping: `8/8 exactly one`
- Final Execution Node Mapping: `9/9 exact`
- Freeze PER / Decision Traceability: `10/10 PER`, `9/9 Decision revisions`; `DR-CRAR-003-R1`은 승인 원문대로 `PE-CRAR-002`와 `PE-CRAR-004`를 함께 결정
- Validation Dimension Rows: `80/80`; each Phase Goal has exact 10 dimensions
- Integration Acceptance Mapping: `12/12`
- Shared Validation Baseline Reuse Directive: `N/A`; evidence reuse authorization 없음
- Authoring/Activation/Authorization: `Authored / Not Activated / No`

## P2G-50 Structure Validation

- Validator: `p2g50-goal-set-validator/1.1.0`
- Quality Contract: `2`
- Upstream Input: exact current Freeze/Plan/passed P2G-40 Receipt
- Result: `passed`
- Failure Summary: `[]`
- Game Test/Production Mutation: `none`; 구조 검증만 수행

## P2G-50 Complete Stage Result

- Owner Stage: `P2G-50`
- Required Result Artifact/Relation Set:
  - Goal Set Ref: `GS-CRAR-P2G50-001`
  - Complete Goal Member Set: Master `1` + Phase Goals `8`
  - Phase-to-Goal Member Mapping: `8/8`
  - P2G-50 Resolution Record Ref Set: `explicit empty`
  - Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Cross-Reference/Coverage Evidence: P2G-50 validator `passed`, traceability `10/10`, integration mapping `12/12`, dimension rows `80/80`
- Exit Condition Validation: `passed`
- Current Selection Result: `GS-CRAR-P2G50-001` selected as current complete Goal Set; partial candidate publication 없음

## Required Stop

- Completed Stage: `P2G-50` only
- Recommended Next Stage: `P2G-60 — exact current Freeze/Plan/Goal Set durable handoff`
- New User Request Required: `Yes`
- Handoff Artifact: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production Mutation: `none`
- Test Execution: `none`; Goal Set authoring/structure validation only
