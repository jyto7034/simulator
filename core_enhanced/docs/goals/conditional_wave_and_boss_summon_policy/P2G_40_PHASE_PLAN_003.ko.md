# 조건부 Wave·보스 소환 P2G-40 Successor Phase Plan 003

상태: `P2G-40 complete / current successor Phase Plan / Goal not authored / implementation not authorized`

이 문서는 `FPF-CWBS-P2G30-002`에 동결된 변경 정책을 구현 가능한 Work Package, dependency, Atomic Group과
ordered Phase로 변환한다. stale predecessor `PP-CWBS-P2G40-002`의 6-Phase macro topology는 여전히 유효해
보존하되, changed 7 elements의 trigger epoch, occupancy set, removal-first, candidate ledger와 session lifecycle
경계를 새 code review에 맞게 갱신했다. P2G-50과 구현은 수행하지 않았다.

## Stage Invocation and Entry

- User Request Ref: `USR-CWBS-040`
- Exact user statement: `그럼 40 진행해줘.`
- Selected Stage: `P2G-40 successor repair`
- Contract basis: `core_enhanced/docs/policy_to_goal_pipeline.md`,
  `sha256:7b86983112a3d87b56249ac32613c9b93e1e0daa50fed43ae9089926793d0746`
- Predecessor Phase Plan Ref: `PP-CWBS-P2G40-002`
- Predecessor disposition: `stale by COD-CWBS-001 / retained as immutable history`
- Bound Final Policy Freeze Ref: `FPF-CWBS-P2G30-002`
- Current Review Batch Ref: `RB-CWBS-P2G20-002`
- Current Reviewed Source Basis Ref: `RSB-CWBS-P2G20-002`
- Entry evidence: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; P2G-30
  `FCE-CWBS-SOURCE-002` manifest `41/41 SHA-256 + status exact match`; Freeze/source/binding stale finding `none`
- Repair scope: `PFCR-CWBS-001` direct changed elements
  `SP-01, SP-02, WV-05, SM-04, BR-03, BL-05, PS-01`; new material policy `none`
- Production mutation: `none`

## Phase Plan

- Phase Plan Ref: `PP-CWBS-P2G40-003`
- Phase Plan Validation Receipt Ref: `PVR-CWBS-P2G40-003`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CWBS-P2G20-002 + RB-CWBS-P2G20-002 + RSB-CWBS-P2G20-002 + ordered 25 CRB-CWBS-*-002 + FPF-CWBS-P2G30-002 + Declaration Set {COD-CWBS-001}`
- Bound Final Policy Freeze Ref: `FPF-CWBS-P2G30-002`
- Repair Basis: `PFCR-CWBS-001 + COD-CWBS-001`; P2G-10, P2G-20, P2G-30 prerequisite obligations are
  effectively resolved and the exact P2G-40 owner obligation is resolved only with this complete result
- Applicable P2G-40 Obligation Key Set: `(COD-CWBS-001, OBL-CWBS-001-P2G40)`
- P2G-40 Resolution Record Ref Set: `RR-CWBS-CHG001-P2G40-001`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Goal Structure Fact: `multi_phase`
- Parallelization: `none approved`
- Known Unknown/Limitations: required Freeze claim에 blocking unknown 없음; reachability 미검사는 accepted map-authoring
  constraint다. Unity, production content authoring/balance와 Released/Deployed rollout은 approved out-of-scope다.

## Work Package Set

모든 selector는 위 Bound Freeze에 상대적인 exact Stable Element ID다. role은
`implementation`, `supporting`, `validation_only`로 구분한다.

| Work Package ID | Covered Stable Element ID Selector Set and coverage role | Execution node disposition | Implementation boundary | Expected change | Prerequisites | Validation/completion | Applicable unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `WP-CWBS-01` | implementation: `PE-CWBS-SP-01`, `PE-CWBS-SP-02`; supporting: `PE-CWBS-WV-04`, `PE-CWBS-SM-04`, `PE-CWBS-BR-03`, `PE-CWBS-BR-05`, `PE-CWBS-BL-05` | member of `AG-CWBS-01-RUNTIME-FOUNDATION` | Core atomic Spawn Kernel, placement preflight, request identity/one-shot ledger, trigger registration epoch와 removal cause ordering primitive | Wave/summon 공용 preflight/commit, exact-once settlement, value-bearing occupancy input과 removal-before-allocation seam 추가 | Bound Freeze; `WP-CWBS-02`와 상호 transaction/lifetime prerequisite | all-or-none, deterministic order, duplicate settlement, occupancy-set validation, same-time removal, no partial/eviction focused tests | none |
| `WP-CWBS-02` | implementation: `PE-CWBS-PS-01`; supporting: `PE-CWBS-SP-01`, `PE-CWBS-WV-03`, `PE-CWBS-WV-05`, `PE-CWBS-SM-03`, `PE-CWBS-BR-02`, `PE-CWBS-BL-04`, `PE-CWBS-BL-05` | member of `AG-CWBS-01-RUNTIME-FOUNDATION` | `BattleCore`/`ActiveBattleSession` authoritative ledgers와 server session detach/attach/tick/timeout boundary; `RunCheckpointPayload` 비변경 | active-battle ledger와 candidate set을 process-local로 고정하고 disconnect auto-Pause, active-battle no-TTL, snapshot 후 explicit Resume, explicit-load-only usage를 구현 | `WP-CWBS-01`과 상호 prerequisite | disconnect clock stop, no timeout, reconnect paused/no-refire/reset, process restart rollback, SavePoint shape와 load-use tests | none |
| `WP-CWBS-03` | supporting: `PE-CWBS-BR-04` | standalone | Boss actor runtime에서 encounter-independent per-actor behavior adapter 추출 | summon이 existing task/query/movement를 쓰되 encounter phase authority는 갖지 않음 | `WP-CWBS-01` | Little Red regression, route/task behavior와 removal cancellation fixture | none |
| `WP-CWBS-04` | implementation: `PE-CWBS-WV-01`, `PE-CWBS-WV-02`, `PE-CWBS-WV-05`, `PE-CWBS-WV-06`, `PE-CWBS-WV-07`; supporting: `PE-CWBS-SP-02` | member of `AG-CWBS-02-WAVE-CUTOVER` | PVE Wave schema/compiler/publication validator, actor-slot binding declaration과 current live Wave rows | time-only/default-dependent authoring을 strict typed trigger/dependency/deadline 및 actor binding contract로 cutover | `WP-CWBS-01`; `WP-CWBS-05`와 상호 schema/consumer prerequisite | malformed/default/cycle/source/deadline validation 실패, exact actor slot resolution과 live RON load 성공 | none |
| `WP-CWBS-05` | implementation: `PE-CWBS-WV-01`, `PE-CWBS-WV-03`, `PE-CWBS-WV-04`, `PE-CWBS-WV-05`, `PE-CWBS-WV-07`; supporting: `PE-CWBS-SP-01`, `PE-CWBS-PS-01` | member of `AG-CWBS-02-WAVE-CUTOVER` | simulation trigger registry/evaluator, binding epoch, prospective death observer, group state, Kernel consumer와 terminal settlement seam | direct Wave insert를 typed one-shot evaluator로 교체하고 HP/point first evaluation, post-bind actual death, restore no-refire를 구현 | `WP-CWBS-01`, `WP-CWBS-02`; `WP-CWBS-04`와 상호 prerequisite | registration/binding, already-true HP/point, pre-bind death exclusion, restore, time/priority/group/deadline/required-gate tests | none |
| `WP-CWBS-06` | implementation: `PE-CWBS-SM-01`, `PE-CWBS-SM-02`, `PE-CWBS-SM-03`, `PE-CWBS-SM-04`; supporting: `PE-CWBS-BR-02`, `PE-CWBS-BR-03`, `PE-CWBS-BR-05` | member of `AG-CWBS-03-SUMMON-CUTOVER` | summon definition, provenance, per-summoner counters, batch/handle state와 Persist/Despawn removal lifecycle | SummonedUnit domain과 atomic limit/batch semantics, prior-commit-only lifetime, unresolved removal-first cancellation 추가 | `WP-CWBS-01`, `WP-CWBS-02`; `WP-CWBS-07`과 상호 prerequisite | isolation/scaling/attribution/limit, all removal causes, no UUID/total on cancel, batch-clear exactly-once tests | none |
| `WP-CWBS-07` | implementation: `PE-CWBS-BR-01`, `PE-CWBS-BR-02`, `PE-CWBS-BR-03`, `PE-CWBS-BR-04`, `PE-CWBS-BR-05`; supporting: `PE-CWBS-SM-01`, `PE-CWBS-SM-02`, `PE-CWBS-SM-03`, `PE-CWBS-SM-04`, `PE-CWBS-SP-02` | member of `AG-CWBS-03-SUMMON-CUTOVER` | Boss Runtime schema/compiler/executor, closed occupancy set, deterministic placement waiter, signal binding과 behavior adapter consumer | BossTask Summon, scoped batch handle, Around/Zone order, bounded FIFO/ID wait, space-before-timeout와 removal-first settlement 연결 | `WP-CWBS-01`, `WP-CWBS-02`, `WP-CWBS-03`; `WP-CWBS-06`과 상호 prerequisite | four results, invalid occupancy combinations, all-occupant/distinct-cell preflight, multiple waiters, no reachability/fallback/partial tests | none; 고립 가능성은 content author 책임 |
| `WP-CWBS-08` | implementation: `PE-CWBS-BL-01`, `PE-CWBS-BL-02`, `PE-CWBS-BL-03`, `PE-CWBS-BL-04`; supporting: `PE-CWBS-WV-01`, `PE-CWBS-WV-03`, `PE-CWBS-BR-05` | member of `AG-CWBS-04-OUTCOME-CUTOVER` | terminal schema/compiler, neutral signal-condition evaluator와 single commit authority | direct winner/global cap/implicit Draw를 named outcome evaluator로 교체 | `WP-CWBS-01`, `WP-CWBS-02`, `WP-CWBS-05`, `WP-CWBS-07`; `WP-CWBS-09`와 상호 prerequisite | full bucket, Defeat-first, tie-break, pause, Retreat/no-terminal continuation tests | none |
| `WP-CWBS-09` | implementation: `PE-CWBS-BL-04`; supporting: `PE-CWBS-BL-05` | member of `AG-CWBS-04-OUTCOME-CUTOVER` | result/public projection, node outcome policy, reward/progression와 BossOmen consumer | committed outcome ID를 exhaustive exactly-one policy로 atomic/idempotent consume | `WP-CWBS-02`; `WP-CWBS-08`과 상호 prerequisite | policy publication, ID round-trip, no repay, Defeat no-positive-effect, Exit Gate tests | none |
| `WP-CWBS-10` | implementation: `PE-CWBS-BL-05`; supporting: `PE-CWBS-PS-01`, `PE-CWBS-SP-01` | standalone | successful opponent spawn→attempt candidate ledger→final outcome transaction→Core/server single-set profile store | four sets/result-derived writer를 process-local ordered candidates와 final-Victory-only atomic/idempotent commit, empty new schema로 cutover | `WP-CWBS-01`, `WP-CWBS-02`, `WP-CWBS-09` | death/despawn/reconnect retention, Victory all-candidate commit, Defeat/Retreat/process-death discard, no legacy union, round-trip/reset tests | none |
| `WP-CWBS-11` | implementation: `PE-CWBS-RF-01` | standalone | excluded placeholder data, canonical docs와 end-to-end integration suites | nonproduction representative path와 final cross-phase validation/canonical publication | `WP-CWBS-05`, `WP-CWBS-07`, `WP-CWBS-09`, `WP-CWBS-10` | placeholder exclusions, reconnect/no-SavePoint, full Core/server suites와 docs/data audit | production promotion은 별도 review 필요; current completion blocker 아님 |

## Dependency Graph

| Prerequisite Work Package | Dependent Work Package | Dependency Type | Evidence/reason | Broken prerequisite, valid state or completion claim if ignored |
| --- | --- | --- | --- | --- |
| `WP-CWBS-01` | `WP-CWBS-02` | `transaction`, `persistence_lifetime` | committed request identity/lifecycle/candidate cause가 authoritative active state에 기록됨 | reconnect duplicate 방지와 exactly-once가 깨짐 |
| `WP-CWBS-02` | `WP-CWBS-01` | `authority`, `transaction` | ledger writer/commit point는 Kernel transaction이고 session이 clock authority를 소유 | state와 producer/pause authority가 분리됨 |
| `WP-CWBS-01` | `WP-CWBS-03` | `producer_consumer` | actor removal/cancel이 shared lifecycle 원인을 소비 | removed summon의 pending task 종료가 불명확함 |
| `WP-CWBS-01` | `WP-CWBS-04` | `schema_data`, `validation` | authored placement가 Kernel atomic/distinct contract에 맞아야 함 | publication이 실행 불가능한 Wave를 허용함 |
| `WP-CWBS-01` | `WP-CWBS-05` | `producer_consumer`, `transaction` | Wave runtime이 Kernel request/registration ledger를 소비 | partial spawn과 terminal-before-settlement가 재발함 |
| `WP-CWBS-02` | `WP-CWBS-05` | `persistence_lifetime`, `authority` | trigger/binding/request settlement가 active battle에 남음 | resync가 Wave를 재등록·재발동함 |
| `WP-CWBS-01` | `WP-CWBS-06` | `transaction`, `persistence_lifetime` | summon counter와 batch commit이 atomic spawn/removal order에 결속 | 실패 request가 total을 쓰거나 partial batch 발생 |
| `WP-CWBS-02` | `WP-CWBS-06` | `authority`, `persistence_lifetime` | counter/batch/provenance가 process-local active battle authority에 있음 | phase/reconnect에서 counter/handle 손실 |
| `WP-CWBS-01` | `WP-CWBS-07` | `producer_consumer`, `transaction` | BossTask가 Kernel result/occupancy/removal settlement를 소비 | task result가 spawn transaction과 분리됨 |
| `WP-CWBS-02` | `WP-CWBS-07` | `authority`, `persistence_lifetime` | scoped handle/wait/signal cursor가 active state를 사용 | reconnect가 wait/rebinding을 재실행함 |
| `WP-CWBS-03` | `WP-CWBS-07` | `producer_consumer` | summon actor가 authority-free adapter를 소비 | encounter authority 누출 또는 AI 부재 |
| `WP-CWBS-04` | `WP-CWBS-05` | `schema_data`, `producer_consumer` | typed compiler가 actor-slot/trigger runtime input 생산 | 새 data unreadable/ignored |
| `WP-CWBS-05` | `WP-CWBS-04` | `validation`, `schema_data` | validator가 runtime closed binding semantics만 publish | 실행 불가능 condition 또는 fallback 필요 |
| `WP-CWBS-06` | `WP-CWBS-07` | `producer_consumer`, `schema_data` | summon domain이 BossTask input/result/lifetime 정의 | unproduced definition/handle/signal 참조 |
| `WP-CWBS-07` | `WP-CWBS-06` | `authority`, `transaction` | BossTask binding이 batch authority와 removal-first의 concrete consumer | 독립 validation과 exactly-once 불성립 |
| `WP-CWBS-05` | `WP-CWBS-08` | `transaction`, `producer_consumer` | Wave events가 terminal 전 같은 bucket에서 settle | required Wave가 terminal 뒤 spawn |
| `WP-CWBS-07` | `WP-CWBS-08` | `transaction`, `producer_consumer` | summon removal/signal이 terminal 전에 settle | stale batch/signal로 outcome 선택 |
| `WP-CWBS-02` | `WP-CWBS-08` | `authority`, `persistence_lifetime` | pending/committed outcome과 latch가 active authority에 있음 | reconnect outcome 재선택/중복 signal |
| `WP-CWBS-08` | `WP-CWBS-09` | `producer_consumer`, `schema_data` | committed outcome이 result/node policy 입력 | ID 없는 winner 또는 incomplete policy |
| `WP-CWBS-09` | `WP-CWBS-08` | `transaction`, `authority` | evaluator commit과 result consumption은 한 transaction | outcome/reward 이중 authority |
| `WP-CWBS-01` | `WP-CWBS-10` | `transaction`, `producer_consumer` | successful opponent spawn commit이 candidate ledger source | result-time reconstruction 또는 candidate 누락 |
| `WP-CWBS-02` | `WP-CWBS-10` | `authority`, `persistence_lifetime` | candidate ledger가 reconnect에는 남고 process death에는 폐기됨 | durable profile 조기 반영 또는 reconnect candidate 손실 |
| `WP-CWBS-09` | `WP-CWBS-10` | `authority`, `migration_compatibility` | compendium은 committed final Victory만 사용 | Defeat 등록 또는 competing writer |
| `WP-CWBS-10` | `WP-CWBS-11` | `validation`, `legacy_removal` | final integration이 single-set/no-migration을 포함 | legacy reader/writer 누락 |
| `WP-CWBS-05` | `WP-CWBS-11` | `validation` | placeholder가 conditional Wave path 사용 | Wave 대표 path 부재 |
| `WP-CWBS-07` | `WP-CWBS-11` | `validation` | placeholder가 Boss Summon path 사용 | Summon 대표 path 부재 |
| `WP-CWBS-09` | `WP-CWBS-11` | `validation`, `release_rollout` | placeholder/docs가 final outcome authority와 일치 | stale docs 또는 reward/progression 유입 |

## Cyclic SCC Disposition Set

| SCC member Work Packages | Disposition | Boundary/edge reconstruction | Evidence | Recomputed cycle result |
| --- | --- | --- | --- | --- |
| `WP-CWBS-01`, `WP-CWBS-02` | `coimplement_atomic` | `AG-CWBS-01-RUNTIME-FOUNDATION` | spawn commit/ledger/session transaction과 process-local lifetime이 상호 완성 | condensed acyclic |
| `WP-CWBS-04`, `WP-CWBS-05` | `coimplement_atomic` | `AG-CWBS-02-WAVE-CUTOVER` | strict actor-slot reader/writer schema와 binding/runtime을 분리할 수 없음 | condensed acyclic |
| `WP-CWBS-06`, `WP-CWBS-07` | `coimplement_atomic` | `AG-CWBS-03-SUMMON-CUTOVER` | domain producer와 BossTask consumer/batch/removal authority가 상호 완성 | condensed acyclic |
| `WP-CWBS-08`, `WP-CWBS-09` | `coimplement_atomic` | `AG-CWBS-04-OUTCOME-CUTOVER` | evaluator/result/node/reward가 single outcome transaction | condensed acyclic |

## Atomic Group Set

| Atomic Group ID | Member Work Packages | Indivisible boundary and evidence |
| --- | --- | --- |
| `AG-CWBS-01-RUNTIME-FOUNDATION` | `WP-CWBS-01`, `WP-CWBS-02` | Kernel commit, registration/removal order, active-battle ledgers와 session pause/lifetime authority가 한 transaction boundary |
| `AG-CWBS-02-WAVE-CUTOVER` | `WP-CWBS-04`, `WP-CWBS-05` | strict Wave/actor schema, compiler, live RON과 binding/evaluator runtime의 reader-writer cutover |
| `AG-CWBS-03-SUMMON-CUTOVER` | `WP-CWBS-06`, `WP-CWBS-07` | summon data/counter/batch/lifetime producer와 BossTask placement/wait/removal consumer cutover |
| `AG-CWBS-04-OUTCOME-CUTOVER` | `WP-CWBS-08`, `WP-CWBS-09` | terminal evaluator/result/node policy/reward consumer의 single-authority cutover |

Standalone Final Execution Nodes: `WP-CWBS-03`, `WP-CWBS-10`, `WP-CWBS-11`.

## Ordered Phase Set

| Phase ID | Primary Stable Element ID Selectors | Supporting policy relations | Assigned final execution nodes | Dependencies | Exit acceptance |
| --- | --- | --- | --- | --- | --- |
| `PHASE-CWBS-01-RUNTIME-FOUNDATION` | `PE-CWBS-SP-01`, `PE-CWBS-SP-02`, `PE-CWBS-PS-01` | `PE-CWBS-WV-04`, `PE-CWBS-WV-05`, `PE-CWBS-SM-04`, `PE-CWBS-BR-03`, `PE-CWBS-BR-05`, `PE-CWBS-BL-05` shared state/lifecycle support | `AG-CWBS-01-RUNTIME-FOUNDATION` | `FPF-CWBS-P2G30-002` | Kernel/ledger/removal order와 auto-Pause/no-TTL/explicit Resume/process-death rollback이 focused tests로 유효하고 `RunCheckpointPayload`는 전투 상태를 포함하지 않음 |
| `PHASE-CWBS-02-WAVE-CUTOVER` | `PE-CWBS-WV-01`, `PE-CWBS-WV-02`, `PE-CWBS-WV-03`, `PE-CWBS-WV-04`, `PE-CWBS-WV-05`, `PE-CWBS-WV-06`, `PE-CWBS-WV-07` | `PE-CWBS-BR-04` adapter 준비; Phase 1 Kernel/state와 trigger epoch 소비 | `AG-CWBS-02-WAVE-CUTOVER`, `WP-CWBS-03` | `PHASE-CWBS-01-RUNTIME-FOUNDATION` | strict Wave schema/live RON/runtime cutover, binding first-evaluation/prospective death/restore no-refire, no legacy/default path; adapter regression pass |
| `PHASE-CWBS-03-SUMMON-CUTOVER` | `PE-CWBS-SM-01`, `PE-CWBS-SM-02`, `PE-CWBS-SM-03`, `PE-CWBS-SM-04`, `PE-CWBS-BR-01`, `PE-CWBS-BR-02`, `PE-CWBS-BR-03`, `PE-CWBS-BR-04`, `PE-CWBS-BR-05` | Phase 1 Kernel/state/removal과 Phase 2 adapter 소비 | `AG-CWBS-03-SUMMON-CUTOVER` | `PHASE-CWBS-02-WAVE-CUTOVER` | catalog/counter/batch/value occupancy/placement/bounded waiter/behavior/removal-first path 완결; no reachability inference/fallback |
| `PHASE-CWBS-04-OUTCOME-CUTOVER` | `PE-CWBS-BL-01`, `PE-CWBS-BL-02`, `PE-CWBS-BL-03`, `PE-CWBS-BL-04` | Wave/summon producers와 active signal/outcome state 소비 | `AG-CWBS-04-OUTCOME-CUTOVER` | `PHASE-CWBS-01-RUNTIME-FOUNDATION`, `PHASE-CWBS-02-WAVE-CUTOVER`, `PHASE-CWBS-03-SUMMON-CUTOVER` | live encounter/node/result/server/reward/progression을 named outcome authority로 atomic cutover; pending Victory는 producer settlement 뒤 commit |
| `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | `PE-CWBS-BL-05` | Phase 1 attempt candidate lifetime, Phase 4 final Victory와 spawn identity seam 소비; `PE-CWBS-PS-01` supporting validation | `WP-CWBS-10` | `PHASE-CWBS-01-RUNTIME-FOUNDATION`, `PHASE-CWBS-04-OUTCOME-CUTOVER` | candidate retention/discard와 final-Victory-only Core/server single registered set commit, empty new schema, no union migration/Defeat writer |
| `PHASE-CWBS-06-REFERENCE-INTEGRATION` | `PE-CWBS-RF-01` | all Freeze selectors의 integration validation과 canonical publication | `WP-CWBS-11` | `PHASE-CWBS-02-WAVE-CUTOVER`, `PHASE-CWBS-03-SUMMON-CUTOVER`, `PHASE-CWBS-04-OUTCOME-CUTOVER`, `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | excluded placeholder, reconnect/no-SavePoint, full Core/server suites와 canonical/data audit pass; no production promotion |

## Intermediate-State Audit

| Phase ID | State immediately after completion | Applicable validity checks | Completion-claim evidence | Result/limitation |
| --- | --- | --- | --- | --- |
| `PHASE-CWBS-01-RUNTIME-FOUNDATION` | generic Kernel/ledger/session lifecycle 존재; old Wave/compendium consumers 유지; candidate schema는 process-local dormant | single transaction/clock authority, auto-Pause/no-TTL, explicit Resume, process rollback, SavePoint exclusion, old behavior regression | Kernel/ledger/session detach-attach/load-use tests | `valid`; Wave target과 permanent compendium commit은 주장하지 않고 later consumer가 dormant seam을 사용 |
| `PHASE-CWBS-02-WAVE-CUTOVER` | live Wave strict cutover; summon path는 아직 없음; adapter 준비 | reader/writer match, registration/binding epoch, prospective death, no restore refire, order/gate, Boss regression | Wave publication/runtime/live RON와 adapter tests | `valid`; Wave는 독립 완료, adapter는 concrete Phase 3 prerequisite |
| `PHASE-CWBS-03-SUMMON-CUTOVER` | summon/BossTask fixture와 opt-in path 완결; old terminal 유지 | producer/consumer schema, occupancy validation, waiter arbitration, counter/batch lifetime, removal-first, reward isolation | summon compiler/runtime/placement/removal/AI tests | `valid`; summon은 승리 gate가 아니고 current battle-end hook에서 정리 |
| `PHASE-CWBS-04-OUTCOME-CUTOVER` | live result/server named outcome 사용; compendium profile shape는 기존 | single terminal authority, full bucket settlement, exhaustive data, atomic consumption, no cap/dual reward | outcome/result/server/BossOmen/live data tests | `valid`; pending Victory는 정산 후 commit하고 old compendium writer는 Phase 5까지 유지 |
| `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | attempt candidate source와 single new compendium schema가 final Victory transaction에 연결 | one profile authority, reconnect retention/process-death discard, no four-set/union migration, Defeat/Retreat unchanged, SavePoint unaffected | Core/server candidate/commit/round-trip/reset tests | `valid`; 개발 profile 재생성은 승인된 정책 |
| `PHASE-CWBS-06-REFERENCE-INTEGRATION` | placeholder/exclusions/docs/end-to-end가 final implementation과 일치 | cross-phase state/persistence/data/consumer, exclusions, docs currentness, no promotion | assigned integration suites와 existing full-suite runners | `valid`; Released/Deployed, Unity, production balance claim 없음 |

## Integration Validation Ownership Map

| Integration acceptance / source | Seam disposition | Read-only locator, invocation and evidence contract | Owning Final Execution Node / Phase | Availability prerequisite | Evidence / limitation |
| --- | --- | --- | --- | --- | --- |
| `IVA-CWBS-01` — shared Kernel atomicity/provenance/exact-once | `assigned_execution_node` | durable Kernel and final cross-consumer fixtures | `AG-CWBS-01-RUNTIME-FOUNDATION` / `PHASE-CWBS-01-RUNTIME-FOUNDATION`; `WP-CWBS-11` / `PHASE-CWBS-06-REFERENCE-INTEGRATION` | Phase 1; final after Phase 3 | generic transaction과 actual Wave/summon 연결을 분리 검증 |
| `IVA-CWBS-02` — trigger registration/binding, first evaluation, prospective death, restore | `assigned_execution_node` | durable PVE compiler + deterministic simulation observer fixtures | `AG-CWBS-02-WAVE-CUTOVER` / `PHASE-CWBS-02-WAVE-CUTOVER` | Phase 1 | already-true HP/point와 pre-bind death를 함께 검증 |
| `IVA-CWBS-03` — value-bearing occupancy and deterministic placement | `assigned_execution_node` | durable publication-invalid matrix + multi-occupant/cell-order battle fixtures | `AG-CWBS-03-SUMMON-CUTOVER` / `PHASE-CWBS-03-SUMMON-CUTOVER` | Phase 1 Kernel | empty/duplicate/redundant/Any-mixed 거부와 no eviction/partial 검증 |
| `IVA-CWBS-04` — bounded waiter, space-before-timeout and removal-first | `assigned_execution_node` | deterministic same-bucket multiple-request/removal battle fixture | `AG-CWBS-03-SUMMON-CUTOVER` / `PHASE-CWBS-03-SUMMON-CUTOVER` | Phase 1 clock/removal seam | FIFO/ID, cancel/limit recheck, no UUID/usage on canceled commit |
| `IVA-CWBS-05` — full bucket then one Defeat-first outcome | `assigned_execution_node` | deterministic simulation-time paired outcome fixture | `AG-CWBS-04-OUTCOME-CUTOVER` / `PHASE-CWBS-04-OUTCOME-CUTOVER` | Phases 2–3 | wall-clock timing은 evidence 아님 |
| `IVA-CWBS-06` — outcome/node/reward/server/BossOmen atomic consumption | `assigned_execution_node` | Core/server round-trip, duplicate consume/reconnect, exhaustive live policy fixtures | `AG-CWBS-04-OUTCOME-CUTOVER` / `PHASE-CWBS-04-OUTCOME-CUTOVER` | Phase 1 state and Phase 4 data | no flat reward fallback |
| `IVA-CWBS-07` — attempt candidates and final-Victory single registered set | `assigned_execution_node` | successful-spawn identity + Core active state + server profile/reset fixtures | `WP-CWBS-10` / `PHASE-CWBS-05-COMPENDIUM-CUTOVER` | Phase 1 ledger, Phase 4 verdict | death/despawn/reconnect retain; Defeat/Retreat/process death discard; no legacy union |
| `IVA-CWBS-08` — disconnect auto-Pause/no TTL/snapshot then explicit Resume | `assigned_execution_node` | server actor detach/attach/tick/timeout + Core revisioned pause fixtures | `AG-CWBS-01-RUNTIME-FOUNDATION` / `PHASE-CWBS-01-RUNTIME-FOUNDATION` | active battle fixture | client command 없는 pause와 timeoutless same-process retention 검증 |
| `IVA-CWBS-09` — process death last SavePoint and explicit-load-only usage | `assigned_execution_node` | ActiveBattleSession exclusion + active-run/profile store restart/load fixtures | `AG-CWBS-01-RUNTIME-FOUNDATION` / `PHASE-CWBS-01-RUNTIME-FOUNDATION`; `WP-CWBS-11` / `PHASE-CWBS-06-REFERENCE-INTEGRATION` | Phase 1; final integrated rerun | OS durability를 active battle persistence로 오인하지 않음 |
| `IVA-CWBS-10` — excluded nonproduction placeholder | `assigned_execution_node` | placeholder RON run + exclusion assertions | `WP-CWBS-11` / `PHASE-CWBS-06-REFERENCE-INTEGRATION` | Phases 2–5 | lore/balance/promotion 제외 |
| `IVA-CWBS-11` — canonical docs/data authority alignment | `assigned_execution_node` | durable invariant test or repository audit with final loaders | `WP-CWBS-11` / `PHASE-CWBS-06-REFERENCE-INTEGRATION` | all nodes | Unity/Released/Deployed 제외 |
| `IVA-CWBS-12` — repository-wide regression runner | `existing_read_only` | `CARGO_TARGET_DIR=/tmp/<task-owned> cargo test -p game_core` and `cargo test -p game_server`; non-zero, zero-failure evidence | `N/A` | assigned target tests complete | runner availability only; baseline reuse/execution은 P2G-50 Goal 계약이 소유 |

## P2G-40 Resolution Record — `RR-CWBS-CHG001-P2G40-001`

- Resolution Record Ref: `RR-CWBS-CHG001-P2G40-001`
- Obligation Key: `(COD-CWBS-001, OBL-CWBS-001-P2G40)`
- Owner Stage: `P2G-40`
- Resolved prerequisite basis:
  `FPF-CWBS-P2G30-002 + RR-CWBS-CHG001-P2G10-001 + RR-CWBS-CHG001-P2G20-001 + RR-CWBS-CHG001-P2G30-001 + PFCR-CWBS-001 + COD-CWBS-001 + USR-CWBS-040`
- Resolution artifacts: `PP-CWBS-P2G40-003 + PVR-CWBS-P2G40-003`
- Result Composite Basis projection:
  `WR-CWBS-P2G20-002 + FPF-CWBS-P2G30-002 + PP-CWBS-P2G40-003 + Goal Set N/A + Declaration Set {COD-CWBS-001}`
- Closed invalidation selector: stale `PP-CWBS-P2G40-002`, `PVR-CWBS-P2G40-002`과 predecessor planning selection
- Downstream obligation closure change: `none`; Obligation Supersession Record Ref Set은 explicit empty
- Record selection: `effective only with the same passed Receipt and complete P2G-40 result`

## Phase Plan Validation Receipt Summary

- Phase Plan Validation Receipt Ref: `PVR-CWBS-P2G40-003`
- Evaluated Phase Plan Ref: `PP-CWBS-P2G40-003`
- Bound Final Policy Freeze Ref: `FPF-CWBS-P2G30-002`
- Validator/Method Version: `p2g40-receipt-validator/1.0.0`
- Immutable Machine Detail: `P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_003.json`
- Overall Result: `passed`

## P2G-40 Complete Stage Result

- Owner Stage: `P2G-40`
- Required Result Artifact/Relation Set:
  - Phase Plan Ref: `PP-CWBS-P2G40-003`
  - Phase Plan Validation Receipt Ref: `PVR-CWBS-P2G40-003`
  - P2G-40 Resolution Record Ref Set: `{RR-CWBS-CHG001-P2G40-001}`
  - Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Cross-Reference/Coverage Evidence: `PCE-CWBS-P2G40-003-SOURCE + PVR-CWBS-P2G40-003`
- Exit Condition Validation: `passed`
- Current Selection Result: `PP-CWBS-P2G40-003` selected as current ordered Phase Plan;
  `PP-CWBS-P2G40-002` retained as stale immutable history

## P2G-40 Exit Validation

1. Receipt가 exact `PP-CWBS-P2G40-003`와 `FPF-CWBS-P2G30-002`을 가리키고 `passed`다.
2. 25 stable selectors는 Freeze member 25개로 exact resolve되고 primary Phase missing/duplicate/extra가 없다.
3. 11 Work Packages는 8 Atomic Group members와 3 standalone nodes로 complete partition된다.
4. 27 dependency edges의 cyclic SCC 4개가 모두 disposition되고 7-node condensation graph는 acyclic이다.
5. 7 Final Execution Nodes는 6 Phases에 exactly once 배정되고 Phase dependency/order와 일치한다.
6. 6 Intermediate-State Audit rows와 12 Integration Validation Ownership rows가 complete다.
7. Parallelization은 승인하지 않았고 Goal Structure Fact는 6-Phase `multi_phase`다.
8. SavePoint는 active battle/candidate ledger를 저장하지 않으며 성공한 explicit load만 usage를 소비한다.
9. disconnect auto-Pause/no-TTL/explicit Resume, trigger binding epoch, removal-first와 candidate lifetime이 exact owner
   node/Phase에 배정됐다.
10. applicable declaration은 valid/current이고 `RR-CWBS-CHG001-P2G40-001`이 O40을 effectively resolve한다.

```text
is_p2g40_complete(PP-CWBS-P2G40-003, PVR-CWBS-P2G40-003) = true
effective_resolution(COD-CWBS-001, OBL-CWBS-001-P2G40)
  = RR-CWBS-CHG001-P2G40-001
```

## Required Stop

- Completed Stage: `P2G-40` only
- Earliest unresolved active obligation: `(COD-CWBS-001, OBL-CWBS-001-P2G50)`
- Recommended Next Stage: `P2G-50 — current Freeze/Plan/Receipt를 exact basis로 successor Goal Set 작성`
- New User Request Required: `Yes`
- Goal Set: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production mutation: `none`
- Test execution: `none`; Receipt generator/validator는 문서 구조와 graph만 검증
