# Core Runtime Architecture Remediation — Successor Core-Only P2G-40 Phase Plan

상태: `P2G-40 complete / current Phase Plan / Goal not authored / implementation not authorized`

이 문서는 current Freeze `FPF-CRAR-P2G30-002`의 10개 정책 요소를 `core_enhanced` 안에서 구현할 Work
Package, typed dependency, Atomic Group과 ordered Phase로 변환한다. 기능·구조 교정이 끝나기 전에는 성능
최적화나 성능 합격 판정을 요구하지 않고 관찰값만 보존한다. P2G-50 Goal 작성과 구현은 수행하지 않는다.

## Stage Invocation and Entry

- User Performance Priority Ref: `USR-CRAR-016`
- Exact user performance statement:

  ```text
  응. 성능을 너무 빡빡하게 하면 안 돼.
  성능은 일단 기능이 먼저 모두 구현되고 나서 개선해야하는 부분이지, 구현도 안 됐는데 성능을 개선하려고 하면 안돼.
  차라리 스킵을 하는건 어때?
  ```

- User Stage Request Ref: `USR-CRAR-017`
- Exact user stage statement: `P2G-40 을 수행해줘.`
- Selected Stage: `P2G-40` only
- Contract Basis: `docs/policy_to_goal_pipeline.md`; SHA-256
  `874742806deb79a39b13972d80ea08e5f6340f3b6fe24a9b7b42d2ffb8544c1c`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-002`
- Bound Freeze Lifecycle: `current`
- Current Working Revision / Review Batch / Source Basis:
  `WR-CRAR-P2G20-COMPLETE-002` / `RB-CRAR-P2G20-002` / `RSB-CRAR-P2G20-002`
- Entry Currentness Evidence: `PCE-CRAR-P2G40-002` — observation `2026-08-22T22:25:00+09:00`;
  repository HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; P2G-20 durable reviewed inputs
  `51/51` SHA-256 exact-match; Freeze SHA-256
  `c55dbe40a2e5ec40fb3a62ff6d02dbff6d359395a048cb08e795f22e5d25452a`; stale
  Freeze/source/binding finding `0`
- Earlier Owner Obligation Closure:
  `{RR-CRAR-CHG001-P2G10-001, RR-CRAR-CHG001-P2G20-001, RR-CRAR-CHG001-P2G30-001}` effective
- Active Owner Leaf: `(COD-CRAR-001, OBL-CRAR-001-P2G40)`
- Interpretation of `USR-CRAR-016`: implementation sequencing constraint. It defers optimization and blocking
  performance acceptance until the final functional architecture exists; it does not remove or weaken the frozen final
  PE-008 acceptance.
- Policy Meaning Change: `none`
- External Inspection/Claim: `none`; game_server와 Unity는 source/compatibility/validation target이 아님
- Production Code/Data/Test Mutation: `none`
- Documentation Tooling Change: standard P2G-40 validator `1.1.0` accepts both historical
  `Ordered code_reviewed` and current `Ordered code-reviewed` immutable Freeze field spellings

Entry result: `eligible_successor_repair`.

## Phase Plan

- Phase Plan Ref: `PP-CRAR-P2G40-002`
- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-002`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-002 + RB-CRAR-P2G20-002 + RSB-CRAR-P2G20-002 +
  CRB-CRAR-P2G20-011..020 + FPF-CRAR-P2G30-002 + Declaration Set {COD-CRAR-001}`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-002`
- Repair Basis:
  `PFCR-CRAR-001 + COD-CRAR-001 + effective prerequisite resolutions RR-CRAR-CHG001-P2G10-001,
  RR-CRAR-CHG001-P2G20-001, RR-CRAR-CHG001-P2G30-001 + active OBL-CRAR-001-P2G40`
- Applicable P2G-40 Obligation Key Set: `{(COD-CRAR-001, OBL-CRAR-001-P2G40)}`
- P2G-40 Resolution Record Ref Set: `{RR-CRAR-CHG001-P2G40-001}`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Goal Structure Fact: `multi_phase`
- Parallelization: `none approved`
- Performance Sequencing: Phase 1은 비교 가능한 baseline과 durable observation seam만 고정하고 최적화하지
  않는다. Phase 2~7 performance observation은 non-blocking이다. 기능·구조 완료 뒤 Phase 8만 최적화와 frozen
  performance acceptance를 blocking으로 소유한다.
- Structural Safety Exception: bounded history, indexed cursor와 retained-authority 분리는 성능 튜닝이 아니라
  PE-009의 correctness/operational-boundary이므로 Phase 5에서 구현한다.
- Known Unknown/Limitations: `GAP-CRAR-P2G20-002` direct heap/allocation baseline은
  `WP-CRAR-C2-01`이 pre-change observation seam과 baseline을 만들고 `WP-CRAR-C2-09`가 final like-for-like
  result와 guard를 소유한다. Phase 8 전에는 performance PASS를 주장하지 않는다. external compatibility,
  Released/Deployed actual과 production promotion은 N/A/out-of-scope다.
- Validator/Method Version: `p2g40-receipt-validator/1.1.0`

## Work Package Set

모든 selector는 위 Bound Freeze에 상대적인 stable Element ID다. `implementation`은 policy realization,
`supporting`은 다른 요소의 구현 경계 지원, `validation_only`는 의미 변경 없는 검증 coverage를 뜻한다.

| Work Package ID | Covered Stable Element ID Selector Set and coverage role | Execution node disposition | Implementation boundary | Expected change | Prerequisites | Validation/completion | Applicable unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `WP-CRAR-C2-01` | implementation: `PE-CRAR-001`; supporting: `PE-CRAR-002`, `PE-CRAR-008`; validation_only: `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | current Rulebook 기능, Core production route와 executable oracle을 stable feature ID로 연결하고 slice differential, rejection/checkpoint/public-projection corpus와 repeatable performance observation seam을 고정한다. | 57-feature manifest를 route/test/data oracle에 연결하고 event-heavy/64-session latency·encoded-size baseline을 durable하게 재현하며 direct heap/allocation observation을 추가한다. production algorithm은 변경하지 않는다. | Bound Freeze와 current Core source/data/test basis | feature/route/test coverage 100%, unknown/unmapped feature 0, deterministic corpus identity, baseline 재현 receipt와 source behavior delta 0 | `GAP-CRAR-P2G20-002`의 pre-change 측정 owner. 측정 존재만 blocking이며 수치 개선이나 performance PASS는 아님 |
| `WP-CRAR-C2-02` | implementation: `PE-CRAR-003`; supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-008` | member of `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | Core mutating command를 `validate → stage → prepare fallible result → commit → infallible publish` executor, staged draft와 typed commit token으로 통일한다. | 모든 `Err`는 authoritative state/event/revision/durable-intent를 쓰지 않고, commit 뒤 projection 실패는 accepted/recovery-pending outcome으로 분리한다. | `WP-CRAR-C2-01`, `WP-CRAR-C2-04`; `WP-CRAR-C2-03`과 상호 transaction/authority prerequisite | CA-003 six-command와 StartNewGame rejection/retry matrix exact equality, post-commit outcome typing, executor bypass 0 | none |
| `WP-CRAR-C2-03` | implementation: `PE-CRAR-002`, `PE-CRAR-004`; supporting: `PE-CRAR-003`, `PE-CRAR-005`, `PE-CRAR-010` | member of `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | parallel optional lifecycle을 closed sum type/context로 교체하고 모든 mutating route를 하나의 application service/commit port와 authority owner에 등록한다. | vertical slice별 old/new hard switch, one owner/no dual write와 invalid lifecycle pre-rejection을 강제하며 gameplay algorithm은 oracle 그대로 이동한다. | `WP-CRAR-C2-01`, `WP-CRAR-C2-04`; `WP-CRAR-C2-02`와 상호 authority/transaction prerequisite | route-owner registry 완전성, impossible-state construction 차단, invalid-phase exact equality, simultaneous writer 0, slice differential pass | none |
| `WP-CRAR-C2-04` | implementation: `PE-CRAR-005`; supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-006` | standalone | authoring input→recoverable validation→opaque `ValidatedGameData` publication→Core construction 경계와 fixture/test/admin capability를 분리한다. | production bare database/builder/direct setter를 제거하고 fixture는 test-only capability, 운영 변경은 typed Core command로 제한한다. | `WP-CRAR-C2-01` inventory/oracle | actual RON load parity, invalid authoring pre-authority failure, production bare construction compile-fail/API audit, fixture capability production-inaccessible | external server admin surface는 검사하거나 completion 조건으로 삼지 않음 |
| `WP-CRAR-C2-05` | implementation: `PE-CRAR-006`; supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-008` | standalone | private battle kernel, scoped command/query/mutation port와 fallible finite position/speed/time newtype을 Core battle boundary에 적용한다. | public unit/event authority mutation과 raw non-finite 입력을 차단하며 finite boss/movement/skill algorithm, ordering과 determinism은 보존한다. | `WP-CRAR-C2-01`, `WP-CRAR-C2-04`, `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | public bypass compile-fail/API audit, NaN/±Inf pre-mutation equality, finite boss/movement/skill differential and determinism pass | runtime public-caller frequency는 completion claim이 아님 |
| `WP-CRAR-C2-06` | implementation: `PE-CRAR-009`; supporting: `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-007`, `PE-CRAR-008`, `PE-CRAR-010` | standalone | active Core history/index/cursor lifetime을 authoritative result/continuation/checkpoint와 분리한다. | history를 `8,192 events`, `4 MiB encoded bytes`, `180 battle-seconds` 중 먼저 도달한 cap에서 닫고 old cursor에는 typed snapshot-resync requirement를 반환하며 result/continuation은 diagnostic log를 retain하지 않는다. | `WP-CRAR-C2-01`, `AG-CRAR-C2-01-APPLICATION-AUTHORITY`, `WP-CRAR-C2-05` | in-window gap/duplicate 0, boundary/overflow/old-cursor matrix, result/continuation equality, history absence/expiry independence, bounded lookup/work | 180 battle-seconds는 history window이지 battle timeout이 아님; latency/memory PASS는 Phase 8 소유 |
| `WP-CRAR-C2-07` | implementation: `PE-CRAR-010`; supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-009` | standalone | Core checkpoint codec/state identity에 V5 target writer, explicit V4 importer와 certification gate를 둔다. | 새 write는 V5만 생성하고 canonical V4 corpus는 identity/state/continuation import oracle로 유지하며 인증 전 V4 reader removal을 거부한다. | `WP-CRAR-C2-01`, `WP-CRAR-C2-04`, `AG-CRAR-C2-01-APPLICATION-AUTHORITY`, `WP-CRAR-C2-06` | V4→V5 semantic differential, V5 round-trip/corruption reject, target-only write, reader-removal negative gate | Released/Deployed save population은 evidence-backed N/A; canonical development corpus만 authority |
| `WP-CRAR-C2-08` | implementation: `PE-CRAR-007`; supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-006`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | Core domain aggregate/mutation handle에서 transport-neutral Core `ApplicationCommand`, result/error, snapshot/read-model projection을 분리한다. | public projection이 domain mutation authority를 노출하지 않고 모든 Core command/result/error/snapshot semantic mapping을 exhaustive하게 닫으며 legacy Core public mutation projection을 제거한다. | `AG-CRAR-C2-01-APPLICATION-AUTHORITY`, `WP-CRAR-C2-05`, `WP-CRAR-C2-06`, `WP-CRAR-C2-07` | Core mapping exhaustiveness, semantic parity/round-trip, public API audit, legacy Core projection/bypass 0 | game_server/Unity wire schema, consumer와 compatibility PASS는 범위 밖 |
| `WP-CRAR-C2-09` | implementation: `PE-CRAR-008`; validation_only: `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | 모든 Core functional boundary가 완성된 뒤에만 same-run benchmark, profiler, retained-memory/allocation/history-query/serialization/checkpoint 측정과 필요한 hotspot optimization을 수행한다. | Phase 1 baseline과 like-for-like 비교해 통계적으로 지지되는 regression을 제거하고 event-heavy release p95 `<25 ms`, 64-session p95 `<10 ms`, exact deterministic output, PE-009 cap과 evidence-derived permanent Core guard를 닫는다. | `WP-CRAR-C2-01`, `WP-CRAR-C2-05`, `WP-CRAR-C2-06`, `WP-CRAR-C2-07`, `WP-CRAR-C2-08` | interleaved repeated comparison, direct heap/allocation result, retained-memory-independent staging, history/query/serialization/checkpoint guard, final cap PASS | Phase 1~7 관찰값은 non-blocking. arbitrary 추가 absolute cap을 미리 만들지 않으며 최적화는 이 WP 전 금지 |
| `WP-CRAR-C2-10` | validation_only: `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-008`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | 모든 cutover 뒤 Core legacy/bypass/fallback 제거 audit, preservation manifest reconciliation과 10개 acceptance evidence index를 소유한다. | 승인되지 않은 gameplay meaning change, competing authority, unreadable checkpoint, unbounded active history, domain mutation exposure와 미소유 validation seam을 0으로 만든다. | 모든 preceding Final Execution Node, 특히 `WP-CRAR-C2-09` | 10/10 acceptance, full Core suites, feature manifest 100%, legacy/bypass/dual writer 0, final evidence/index currentness | external consumers, production promotion과 deferred feature-policy implementation은 범위 밖 |

## Dependency Graph

| Prerequisite Work Package | Dependent Work Package | Dependency Type | Evidence/reason | Broken prerequisite, valid state or completion claim if ignored |
| --- | --- | --- | --- | --- |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-04` | `policy`, `validation` | validated publication 전 actual data/feature oracle와 invalid corpus를 고정해야 함 | valid data 의미가 경계 변경 중 조용히 바뀜 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-02` | `policy`, `validation` | rejection/accepted behavior baseline이 transaction 교체 oracle | exact equality가 기능 손실을 가릴 수 있음 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-03` | `policy`, `validation` | production route inventory가 lifecycle owner registry 완전성 기준 | 미등록 route 또는 gameplay drift 발생 |
| `WP-CRAR-C2-04` | `WP-CRAR-C2-02` | `authority`, `schema_data` | application executor는 opaque validated publication만 받아야 함 | invalid data로 authority가 구성될 수 있음 |
| `WP-CRAR-C2-04` | `WP-CRAR-C2-03` | `authority`, `schema_data` | closed lifecycle construction도 validated context를 소비해야 함 | illegal state가 invalid data authority를 감쌀 수 있음 |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-03` | `transaction`, `authority` | route owner는 typed executor/commit port를 통해서만 mutate | owner registry와 실제 commit root 불일치 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-02` | `authority`, `transaction` | executor는 closed lifecycle/route authority 없이는 prevalidate 불가 | invalid phase가 staging/commit 뒤 발견됨 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-05` | `policy`, `validation` | finite battle algorithm preservation corpus가 privacy cutover 기준 | gameplay drift를 authority 강화로 오판 |
| `WP-CRAR-C2-04` | `WP-CRAR-C2-05` | `schema_data`, `authority` | kernel setup input은 ValidatedGameData에서만 파생 | invalid authored state가 kernel 생성 뒤 도착 |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-05` | `transaction` | battle command port가 root commit outcome 규약을 소비 | kernel error와 application commit 결과 분리 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-05` | `authority`, `producer_consumer` | battle lifecycle owner가 private kernel의 유일 owner | public bypass/competing battle authority 발생 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-06` | `validation`, `persistence_lifetime` | event identity와 result/continuation oracle이 retention rewrite 기준 | bounded window가 authoritative continuation 의미를 바꿈 |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-06` | `transaction`, `persistence_lifetime` | history/result publication은 accepted commit 뒤에만 발생 | rejected command event/durable intent가 남음 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-06` | `authority`, `persistence_lifetime` | active history와 terminal continuation lifetime이 closed state에 결속 | diagnostic history가 authority를 공유하거나 잃음 |
| `WP-CRAR-C2-05` | `WP-CRAR-C2-06` | `producer_consumer`, `schema_data` | kernel sequence/event output이 indexed window 입력 | ordering/cursor identity 재구성 불가 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-07` | `migration_compatibility`, `validation` | V4 canonical corpus/continuation oracle을 먼저 고정 | migration loss를 새 정상으로 승인 |
| `WP-CRAR-C2-04` | `WP-CRAR-C2-07` | `schema_data`, `migration_compatibility` | importer 결과는 current validated data context에서 복원 | V4가 bare data 경로를 재개방 |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-07` | `transaction`, `persistence_lifetime` | checkpoint write/import는 commit/durable-intent 규약을 따름 | rejected load/save가 authority를 부분 변경 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-07` | `authority`, `migration_compatibility` | closed lifecycle이 V4/V5 복원 target을 정의 | impossible state를 import |
| `WP-CRAR-C2-06` | `WP-CRAR-C2-07` | `persistence_lifetime`, `migration_compatibility` | checkpoint가 continuation state와 diagnostic history 분리를 보존해야 함 | 새 save가 unbounded history를 다시 retain |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-08` | `transaction`, `producer_consumer` | public result/error는 executor outcome 의미에서 파생 | commit 전후 의미가 projection에서 뒤바뀜 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-08` | `authority`, `producer_consumer` | public command route는 single lifecycle owner를 소비 | projection이 competing mutation route가 됨 |
| `WP-CRAR-C2-05` | `WP-CRAR-C2-08` | `schema_data`, `producer_consumer` | private kernel read model이 public snapshot source | projection이 kernel field에 직접 재결합 |
| `WP-CRAR-C2-06` | `WP-CRAR-C2-08` | `schema_data`, `persistence_lifetime` | bounded delta/resync/result semantics가 public mapping 입력 | cursor/result meaning parity가 깨짐 |
| `WP-CRAR-C2-07` | `WP-CRAR-C2-08` | `schema_data`, `migration_compatibility` | checkpoint/import outcome을 transport-neutral error/result로 닫아야 함 | public caller가 migration outcome을 해석 불가 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-09` | `external_evidence`, `validation` | pre-change like-for-like baseline과 exact workload가 final 비교 basis | performance PASS 비교 authority 없음 |
| `WP-CRAR-C2-05` | `WP-CRAR-C2-09` | `producer_consumer`, `validation` | final private kernel/staging hot path가 측정 대상 | 중간 API 성능을 final로 오판 |
| `WP-CRAR-C2-06` | `WP-CRAR-C2-09` | `persistence_lifetime`, `validation` | bounded history/index의 final complexity를 측정 | retained-history independence/scaling 검증 불가 |
| `WP-CRAR-C2-07` | `WP-CRAR-C2-09` | `migration_compatibility`, `validation` | final checkpoint codec이 serialization 비용 측정 대상 | legacy codec 비용을 final로 오판 |
| `WP-CRAR-C2-08` | `WP-CRAR-C2-09` | `producer_consumer`, `validation` | final public projection이 snapshot/result serialization target | domain-only benchmark가 public cost를 놓침 |
| `WP-CRAR-C2-01` | `WP-CRAR-C2-10` | `validation` | final inventory reconciliation은 initial oracle을 소비 | 기능 제거/meaning drift 탐지 불가 |
| `WP-CRAR-C2-04` | `WP-CRAR-C2-10` | `validation`, `legacy_removal` | bare/fixture capability final audit | production bypass가 남음 |
| `WP-CRAR-C2-02` | `WP-CRAR-C2-10` | `validation`, `legacy_removal` | rejection equality/post-commit outcome final matrix | 일부 route가 old error semantics 유지 |
| `WP-CRAR-C2-03` | `WP-CRAR-C2-10` | `validation`, `legacy_removal` | route owner/no-dual-write final audit | competing authority를 놓침 |
| `WP-CRAR-C2-05` | `WP-CRAR-C2-10` | `validation`, `legacy_removal` | public mutation/non-finite boundary final audit | kernel bypass가 남음 |
| `WP-CRAR-C2-06` | `WP-CRAR-C2-10` | `validation`, `persistence_lifetime` | bounded history/result continuation final matrix | expiry가 continuation을 훼손 |
| `WP-CRAR-C2-07` | `WP-CRAR-C2-10` | `validation`, `migration_compatibility` | V4 import/reader-removal gate final 확인 | unreadable save/조기 reader 제거 |
| `WP-CRAR-C2-08` | `WP-CRAR-C2-10` | `validation`, `legacy_removal` | Core public projection/bypass final audit | domain mutation surface가 public에 남음 |
| `WP-CRAR-C2-09` | `WP-CRAR-C2-10` | `validation`, `external_evidence` | final completion은 frozen performance acceptance evidence 요구 | 기능만 맞고 전체 acceptance는 불완전 |

## Cyclic SCC Disposition Set

| SCC member Work Packages | Disposition | Boundary/edge reconstruction | Evidence | Recomputed cycle result |
| --- | --- | --- | --- | --- |
| `WP-CRAR-C2-02`, `WP-CRAR-C2-03` | `coimplement_atomic` | `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | typed commit executor는 closed lifecycle/route owner가 필요하고 route owner는 같은 commit port 없이는 단일 authority가 아님 | condensed acyclic |

## Atomic Group Set

| Atomic Group ID | Member Work Packages | Indivisible boundary | Atomic completion condition | Failure/rollback disposition |
| --- | --- | --- | --- | --- |
| `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | `WP-CRAR-C2-02`, `WP-CRAR-C2-03` | command transaction과 lifecycle/route authority를 같은 hard cutover에서 닫는다. | executor/closed state/owner registry/rejection matrix가 함께 current이고 old writer가 0 | complete result 전에는 group member를 current Phase output으로 선택하지 않는다. partial cutover를 폐기하고 pre-group authority를 유지한다. |

Standalone Final Execution Nodes: `WP-CRAR-C2-01`, `WP-CRAR-C2-04`, `WP-CRAR-C2-05`, `WP-CRAR-C2-06`, `WP-CRAR-C2-07`, `WP-CRAR-C2-08`, `WP-CRAR-C2-09`, `WP-CRAR-C2-10`

Partition result: Work Packages `10`; Atomic Group members `2`; standalone `8`; missing/duplicate/overlap `0`.
Final Execution Node Set은 Atomic Group `1` + standalone `8` = `9`다. Condensation graph는 acyclic이다.

## Ordered Phase Set

| Phase ID | Primary Stable Element ID Selector Set | Supporting/validation selector relation | Assigned Final Execution Nodes | Phase prerequisites | Observable completion boundary |
| --- | --- | --- | --- | --- | --- |
| `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` | `PE-CRAR-001` | supporting: `PE-CRAR-002`, `PE-CRAR-008`; validation: 나머지 Freeze selectors | `WP-CRAR-C2-01` | Bound Freeze/current source basis | feature/route/test/data oracle 100%, deterministic corpus와 latency/size/heap-allocation baseline이 durable하게 재현되며 production behavior delta 0. 성능 개선/PASS는 주장하지 않음 |
| `PHASE-CRAR-C2-02-VALIDATED-DATA` | `PE-CRAR-005` | supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-006` | `WP-CRAR-C2-04` | `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` | production construction은 opaque validated publication만 사용하고 invalid input은 authority 생성 전 recoverable failure이며 fixture/direct capability가 production에서 접근 불가 |
| `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004` | supporting: `PE-CRAR-005`, `PE-CRAR-008`, `PE-CRAR-010` | `AG-CRAR-C2-01-APPLICATION-AUTHORITY` | Phase 1~2 | every mutating route가 closed lifecycle의 single owner와 single commit executor를 사용하고 모든 `Err` exact equality와 no-dual-write가 증명됨 |
| `PHASE-CRAR-C2-04-BATTLE-KERNEL` | `PE-CRAR-006` | supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-008` | `WP-CRAR-C2-05` | Phase 1~3 | Core battle authority가 private scoped kernel/finite input으로 닫히고 bypass 0, non-finite pre-rejection과 finite boss/movement/skill parity가 통과 |
| `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT` | `PE-CRAR-009` | supporting: `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-007`, `PE-CRAR-008`, `PE-CRAR-010` | `WP-CRAR-C2-06` | Phase 1, 3, 4 | bounded indexed history, cursor-floor resync와 result/continuation authority 분리가 current이고 history 부재/overflow가 gameplay continuation을 바꾸지 않음 |
| `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` | `PE-CRAR-010` | supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-009` | `WP-CRAR-C2-07` | Phase 1~3, 5 | V5 target-only write, V4 semantic importer, continuation differential과 certification 전 reader-removal negative gate가 통과 |
| `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` | `PE-CRAR-007` | supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-006`, `PE-CRAR-009`, `PE-CRAR-010` | `WP-CRAR-C2-08` | Phase 3~6 | transport-neutral Core command/result/error/snapshot projection이 semantic parity를 통과하고 public domain mutation handle과 legacy Core projection이 0 |
| `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | `PE-CRAR-008` | validation: 모든 Freeze selectors의 final differential/legacy removal | `WP-CRAR-C2-09`, `WP-CRAR-C2-10` | Phase 1~7 전체 | 먼저 final architecture를 측정하고 필요한 hotspot만 최적화한 뒤 frozen performance acceptance와 10/10 정책 acceptance, full Core suite, feature inventory 100%, legacy/bypass/dual path 0을 함께 증명 |

Phase 1~7의 benchmark observation은 기록/경보만 제공하며 해당 Phase completion을 성능 수치로 막지 않는다.
Phase 5의 history cap은 PE-009 functional invariant다. Phase 8 내부 순서는 `WP-CRAR-C2-09` performance
stabilization 뒤 `WP-CRAR-C2-10` final evidence reconciliation이다.

## Intermediate-State Audit

| Phase ID | State immediately after completion | Applicable validity checks | Completion-claim evidence | Result/limitation |
| --- | --- | --- | --- | --- |
| `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY` | production behavior unchanged; durable oracle와 baseline observation만 추가 | feature/route/test coverage, deterministic fixture/baseline identity, no source behavior delta | manifest verifier, current regression corpus, baseline receipt | `valid`; GAP-002 pre-change seam은 생기지만 performance PASS/optimization은 없음 |
| `PHASE-CRAR-C2-02-VALIDATED-DATA` | existing gameplay algorithms consume one opaque validated publication | publication/reader match, invalid pre-authority failure, no production bare/fixture fallback | load parity, invalid matrix, compile-fail/API audit | `valid`; transaction/lifecycle 결함은 아직 current지만 data authority에는 dual path 없음 |
| `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | world/application mutating routes use closed lifecycle and one executor/owner; battle internal은 current adapter 뒤에 있음 | one authority, exact `Err`, accepted post-commit outcome, route completeness, old writer 0 | command rejection/retry/durable-intent matrix and owner audit | `valid`; incomplete slice는 Phase completion으로 선택하지 않고 old/new simultaneous writer를 허용하지 않음 |
| `PHASE-CRAR-C2-04-BATTLE-KERNEL` | battle authority closed behind private kernel/scoped ports/finite input; history/public projection은 adapter 사용 | public mutation absence, finite construction, non-finite rejection, algorithm parity | compile-fail/API audit, non-finite matrix, boss/movement/skill differential | `valid`; adapter는 read-model/event port만 소비하며 kernel bypass가 아님 |
| `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT` | Core active history bounded; result/continuation/checkpoint authority는 diagnostic history와 분리 | cap/floor, gap-free window, result authority, overflow/expiry continuation, no fallback log authority | retention/resync/property/continuation tests | `valid`; 성능 수치는 non-blocking이나 unbounded lifetime은 제거됨 |
| `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` | Core writes V5 only, imports V4 explicitly and does not persist diagnostic history as continuation authority | version/read-write identity, checksum/reject, continuation, reader retention gate | canonical V4 import/V5 round-trip/old-new continuation differential | `valid`; dual-format write 없음, V4 reader는 certification까지 유지 |
| `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION` | Core exposes transport-neutral application projection and no public domain mutation authority | exhaustive mapping, semantic parity, public visibility, legacy projection/bypass absence | Core parity/round-trip/API audit | `valid`; external consumers의 현재 호환성이나 cutover는 주장하지 않음 |
| `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | final Core architecture, performance guards와 acceptance evidence가 current이고 legacy/bypass가 없음 | 10/10 policy acceptance, feature preservation, atomicity, history, checkpoint, projection, final performance | full Core suite, benchmark/profiler receipts, final evidence index/static audits | `valid`; external compatibility, release/deployment와 deferred feature implementation은 범위 밖 |

각 Phase는 위 evidence가 모두 통과한 complete result로만 선택한다. incomplete source state를 current Phase
completion으로 간주하지 않으며 승인되지 않은 runtime fallback 또는 dual writer를 다음 Phase의 전제로 남기지 않는다.

## Integration Validation Ownership Map

| Integration acceptance / source | Seam disposition | Read-only locator, invocation and evidence contract | Owning Final Execution Node / Phase | Availability prerequisite | Evidence / limitation |
| --- | --- | --- | --- | --- | --- |
| `IVA-CRAR-C2-01` — 전체 기능 inventory와 slice/final semantic preservation | `assigned_execution_node` | durable feature manifest, route/test/data index와 old/new differential fixtures | `WP-CRAR-C2-01` / `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY`; `WP-CRAR-C2-10` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | current Rulebook/Core source/tests/data; final rerun은 모든 cutover | baseline 고정과 final reconciliation을 분리 소유 |
| `IVA-CRAR-C2-02` — rejection authority/durable equality와 retry | `assigned_execution_node` | six-command, StartNewGame, post-commit projection and durable-intent matrix | `AG-CRAR-C2-01-APPLICATION-AUTHORITY` / `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY`; `WP-CRAR-C2-10` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | validated data/application root | diagnostic observation은 authority snapshot 밖에서 분리 |
| `IVA-CRAR-C2-03` — closed lifecycle, route single owner/no dual write | `assigned_execution_node` | durable owner registry, visibility audit, invalid-phase tests | `AG-CRAR-C2-01-APPLICATION-AUTHORITY` / `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` | Phase 2 publication | route별 hard switch; fallback writer 불허 |
| `IVA-CRAR-C2-04` — production validated publication/test capability isolation | `assigned_execution_node` | actual loader invalid matrix, compile-fail/public API audit, typed Core command tests | `WP-CRAR-C2-04` / `PHASE-CRAR-C2-02-VALIDATED-DATA` | current actual RON corpus | external admin API는 completion row가 아님 |
| `IVA-CRAR-C2-05` — private kernel/finite boundary/deterministic gameplay | `assigned_execution_node` | public bypass audit, NaN/Inf matrix, boss/movement/skill differential corpus | `WP-CRAR-C2-05` / `PHASE-CRAR-C2-04-BATTLE-KERNEL` | preservation oracle, validated data, application authority | bitwise cross-platform replay는 scope 밖 |
| `IVA-CRAR-C2-06` — bounded history/result/continuation independence | `assigned_execution_node` | cap/overflow/cursor/resync property tests and continuation-without-history fixtures | `WP-CRAR-C2-06` / `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT`; `WP-CRAR-C2-10` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | final kernel event sequence | external reconnect/receipt/archive owner 없음 |
| `IVA-CRAR-C2-07` — V4 import/V5 target-only write/continuation | `assigned_execution_node` | canonical V4 fixture, V5 round-trip/checksum, continuation differential/removal guard | `WP-CRAR-C2-07` / `PHASE-CRAR-C2-06-CHECKPOINT-MIGRATION` | Phase 3 lifecycle and Phase 5 lifetime split | released save population 없음; canonical corpus authority |
| `IVA-CRAR-C2-08` — transport-neutral Core public projection semantic parity | `assigned_execution_node` | exhaustive command/result/error/snapshot mappings, round-trip and public API audit | `WP-CRAR-C2-08` / `PHASE-CRAR-C2-07-CORE-PUBLIC-PROJECTION`; `WP-CRAR-C2-10` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | final Core lifecycle/kernel/history/checkpoint semantics | game_server/Unity claims 0 |
| `IVA-CRAR-C2-09` — current-relative non-regression/numeric Core guards | `assigned_execution_node` | event-heavy/64-session interleaved harness, allocation/retained-memory/history-query/serialization/checkpoint receipts | `WP-CRAR-C2-01` / `PHASE-CRAR-C2-01-PRESERVATION-OBSERVABILITY`; `WP-CRAR-C2-09` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | baseline before Phase 2; final after Phase 7 | Phase 1~7 non-blocking; final Phase only blocking; GAP-002 exact two-owner closure |
| `IVA-CRAR-C2-10` — ten acceptance and Core legacy/bypass absence | `assigned_execution_node` | final evidence index, static/API audits and full Core suite | `WP-CRAR-C2-10` / `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` | all Final Execution Nodes | external consumers/production promotion/deferred features excluded |
| `IVA-CRAR-C2-11` — existing Core regression runners | `existing_read_only` | repository root에서 task-owned `CARGO_TARGET_DIR=/tmp/<id>`로 `cargo test -p game_core --all-targets`; manifest verifier; exit 0/zero failure | `N/A` | focused assigned tests와 exact Core source/data | runner는 seam이며 final evidence execution은 WP-C2-10 소유 |
| `IVA-CRAR-C2-12` — P2G-20 current lightweight performance characterization | `audit_local_read_only` | reviewed `crar-preservation-baseline/1` tool/input을 동일 toolchain/profile로 실행하고 raw output은 task-owned temporary path에 보존 | `N/A` | exact reviewed tool/input/source identity | 현재 observation 재현용; durable allocation seam과 final PASS는 WP-C2-01/09 소유 |

## Resolution Record — `RR-CRAR-CHG001-P2G40-001`

- Obligation Key: `(COD-CRAR-001, OBL-CRAR-001-P2G40)`
- Owner Stage: `P2G-40`
- Exact Resolved Prerequisite Basis:
  `WR-CRAR-P2G20-COMPLETE-002 + RB-CRAR-P2G20-002 + FPF-CRAR-P2G30-002 +
  RR-CRAR-CHG001-P2G10-001 + RR-CRAR-CHG001-P2G20-001 + RR-CRAR-CHG001-P2G30-001 +
  PFCR-CRAR-001 + COD-CRAR-001`
- Resolution Artifact Ref: `PP-CRAR-P2G40-002 + PVR-CRAR-P2G40-002`
- Exact Result Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-002 + FPF-CRAR-P2G30-002 + PP-CRAR-P2G40-002 +
  PVR-CRAR-P2G40-002 + Goal Set N/A + Declaration Set {COD-CRAR-001}`
- Resolution Evidence: `PCE-CRAR-P2G40-002`, Work Package/graph/Atomic Group/Phase/audit/integration maps,
  repository standard passed Receipt와 exit validation
- Selection: `effective with the same complete P2G-40 result`

## Phase Plan Validation Receipt Summary

- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-002`
- Evaluated Phase Plan Ref: `PP-CRAR-P2G40-002`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-002`
- Validator/Method Version: `p2g40-receipt-validator/1.1.0`
- Immutable Machine Detail: `P2G_40_CORE_ONLY_PHASE_PLAN_VALIDATION_RECEIPT_002.json`
- Overall Result: `passed`
- Expected Machine Counts: Freeze members `10`, Work Packages `10`, dependency edges `39`, cyclic SCCs `1`,
  Atomic Groups `1`, Final Execution Nodes `9`, Phases/audit rows `8/8`, integration rows `12`

## P2G-40 Complete Stage Result

- Owner Stage: `P2G-40`
- Required Result Artifact/Relation Set:
  - Phase Plan Ref: `PP-CRAR-P2G40-002`
  - Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-002`
  - P2G-40 Resolution Record Ref Set: `{RR-CRAR-CHG001-P2G40-001}`
  - Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Cross-Reference/Coverage Evidence: `PCE-CRAR-P2G40-002 + PVR-CRAR-P2G40-002`
- Exit Condition Validation: `passed`
- Current Selection Result: Plan, passed Receipt와 Resolution Record를 함께 current로 선택; partial candidate
  publication `0`

## P2G-40 Exit Validation

| # | Requirement | Evidence | Result |
| --- | --- | --- | --- |
| 1 | exact Phase Plan/Freeze Receipt passed, exit 0 and failures empty | repository standard generator/default validator | pass |
| 2 | every Freeze selector resolves exactly and primary owner covers 10/10 | Receipt selector/primary coverage | pass |
| 3 | all Work Packages partition into standalone or one non-overlapping Atomic Group | 10 = 8 standalone + 2 group members | pass |
| 4 | every cyclic SCC has disposition and condensation is acyclic | one SCC → AG-C2-01 | pass |
| 5 | every Final Execution Node assigned exactly once without group split | nine nodes / eight Phases | pass |
| 6 | every Phase immediate state and completion claim is valid | Intermediate-State Audit 8/8 | pass |
| 7 | Phase order, completion and supporting relations are explicit | ordered Phase Set | pass |
| 8 | all cross-Phase/integration acceptance owns a valid seam | Integration map 12/12 | pass |
| 9 | claimed parallel combination has positive evidence | none claimed/approved | pass |
| 10 | Goal Structure Fact derived without authoring Goals | eight Phases → `multi_phase` | pass |
| 11 | published Plan binds exact current Freeze and is current-selected with Receipt | PP/PVR-C2-002 complete result | pass |
| 12 | declaration valid and P2G-40 obligation effectively resolved | COD-001 / RR-P2G40-001 | pass |
| 13 | stage and authorization boundary preserved | P2G-50/Goal/implementation not performed | pass |

```text
is_p2g40_complete(PP-CRAR-P2G40-002, PVR-CRAR-P2G40-002) = true
effective_resolution(COD-CRAR-001, OBL-CRAR-001-P2G40)
  = RR-CRAR-CHG001-P2G40-001
```

## Required Stop

- Completed Stage: `P2G-40` only
- Earliest Unresolved Active Obligation: `(COD-CRAR-001, OBL-CRAR-001-P2G50)`
- Recommended Next Stage: `P2G-50 — current Freeze/Plan/Receipt를 exact basis로 Goal Set 작성`
- New User Request Required: `Yes — explicit P2G-50 request`
- Goal Set: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production Code/Data/Test Mutation: `none`
- Test Execution: `none`; Receipt generator/validator는 문서 구조, selector와 graph만 검증
