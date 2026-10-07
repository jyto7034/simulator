# Core Runtime Architecture Remediation P2G-40 Phase Plan

상태: `P2G-40 complete / current Phase Plan / Goal not authored / implementation not authorized`

이 문서는 `FPF-CRAR-P2G30-001`의 10개 동결 정책 요소를 구현 가능한 Work Package, typed dependency,
Atomic Group과 ordered Phase로 변환한다. 정책 의미를 변경하지 않으며 P2G-50 Goal 작성과 구현은 수행하지 않는다.

## Stage Invocation and Entry

- User Request Ref: `USR-CRAR-009`
- Exact user statement: `응, 다음 p2g-40 을 수행해줘.`
- Selected Stage: `P2G-40`
- Contract basis: `core_enhanced/docs/policy_to_goal_pipeline.md`, `sha256:874742806deb79a39b13972d80ea08e5f6340f3b6fe24a9b7b42d2ffb8544c1c`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-001`
- Current Review Batch Ref: `RB-CRAR-P2G20-001`
- Current Reviewed Source Basis Ref: `RSB-CRAR-P2G20-001`
- Entry currentness evidence: `PCE-CRAR-P2G40-001` — simulator HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, Unity HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`, reviewed source `38/38 SHA-256 exact match`; stale Freeze/source/binding finding `0`
- Policy meaning change: `none`
- Production mutation: `none`

## Phase Plan

- Phase Plan Ref: `PP-CRAR-P2G40-001`
- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-001`
- Lifecycle Disposition: `current`
- Exact Prerequisite Composite Basis:
  `WR-CRAR-P2G20-COMPLETE-001 + RB-CRAR-P2G20-001 + RSB-CRAR-P2G20-001 + CRB-CRAR-P2G20-001..010 + FPF-CRAR-P2G30-001`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-001`
- Repair Basis: `explicit empty — no Post-Freeze Change, declaration, active obligation, Resolution Record or Obligation Supersession Record`
- Applicable P2G-40 Obligation Key Set: `explicit empty`
- P2G-40 Resolution Record Ref Set: `explicit empty`
- Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Goal Structure Fact: `multi_phase`
- Parallelization: `none approved`
- Known Unknown/Limitations: `GAP-CRAR-P2G20-001`은 `WP-CRAR-01`이 pre-cutover event-heavy boss/64-session concurrent baseline을 고정하고 `WP-CRAR-07`이 post-cutover comparison과 permanent guard를 소유한다. current performance PASS, Unity runtime PASS, Released/Deployed compatibility는 주장하지 않는다.
- Validator/Method Version: `p2g40-receipt-validator/1.0.0`

## Work Package Set

모든 selector는 위 Bound Freeze에 상대적인 stable Element ID다. `implementation`은 정책 realization,
`supporting`은 다른 요소의 경계 지원, `validation_only`는 의미 변경 없는 검증 coverage를 뜻한다.

| Work Package ID | Covered Stable Element ID Selector Set and coverage role | Execution node disposition | Implementation boundary | Expected change | Prerequisites | Validation/completion | Applicable unknown |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `WP-CRAR-01` | implementation: `PE-CRAR-001`; supporting: `PE-CRAR-002`, `PE-CRAR-008`; validation_only: `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | Rulebook heading, behavior route와 executable domain test를 stable feature ID inventory로 고정하고 rejection/save/protocol/performance pre-cutover oracle 및 differential harness를 만든다. | 승인된 현재 기능 전부를 machine-checkable manifest에 연결하고 Little Red·projectile·effect event-heavy fixture와 64-session/8-way interleaved baseline을 cutover 전에 보존한다. | Bound Freeze와 current reviewed source; production algorithm 변경 없음 | inventory route/test coverage 100%, unknown/unmapped feature 0, current behavior/rejection/V4/protocol corpus snapshot, repeatable baseline evidence 생성 | `GAP-CRAR-P2G20-001`; 이 WP가 baseline을 확보하지만 최종 non-regression PASS는 `WP-CRAR-07` 소유 |
| `WP-CRAR-02` | implementation: `PE-CRAR-003`; supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-008` | member of `AG-CRAR-01-APPLICATION-AUTHORITY` | Core application root의 mutating command를 `validate → stage → commit → publish` executor, staged draft, typed commit token과 post-commit accepted/recovery-pending outcome으로 통일한다. | `Err`는 authoritative state/event/revision/durable-intent를 쓰지 않고, commit 뒤 projection/transport 실패는 `Err`가 아니라 accepted outcome으로 분리된다. | `WP-CRAR-01`, `WP-CRAR-04`; `WP-CRAR-03`과 상호 authority/transaction prerequisite | CA-003 six-command와 StartNewGame rejection/retry matrix에서 exact pre-command equality; post-commit failure typing; executor bypass route 0 | none |
| `WP-CRAR-03` | implementation: `PE-CRAR-002`, `PE-CRAR-004`; supporting: `PE-CRAR-003`, `PE-CRAR-005`, `PE-CRAR-010` | member of `AG-CRAR-01-APPLICATION-AUTHORITY` | parallel optional lifecycle을 closed sum type/context로 교체하고 각 mutating route를 하나의 application service/commit port에 등록한다. | boundary slice별 old/new hard switch, one owner/no dual write, invalid lifecycle pre-rejection을 강제하되 named gameplay algorithms는 그대로 이동한다. | `WP-CRAR-01`, `WP-CRAR-04`; `WP-CRAR-02`와 상호 authority/transaction prerequisite | route-owner registry 완전성, impossible state construction 차단, invalid phase exact equality, old/new simultaneous writer 0, slice differential pass | none |
| `WP-CRAR-04` | implementation: `PE-CRAR-005`; supporting: `PE-CRAR-002`, `PE-CRAR-004` | standalone | authoring input→recoverable validation→opaque `ValidatedGameData` publication→Core construction 경계와 test/admin capability를 분리한다. | bare database/builder/public direct setter를 production API에서 제거하고 fixture builder는 `cfg(test)` 또는 explicit fixture feature, 운영 변경은 typed `AdminCommand`로 제한한다. | `WP-CRAR-01` inventory/oracle | invalid RON/authoring recoverable failure, production bare construction compile-fail, fixture/test currency production-inaccessible, existing valid data load parity | none |
| `WP-CRAR-05` | implementation: `PE-CRAR-006`; supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-008` | standalone | private `BattleKernel`, scoped command/query/mutation ports, finite position/speed/time newtypes와 fallible constructors를 battle boundary에 적용한다. | public unit/event authority mutation과 raw non-finite 유입을 차단하고 finite algorithms/order/determinism은 preservation oracle에 맞춰 유지한다. | `WP-CRAR-01`, `WP-CRAR-04`, `AG-CRAR-01-APPLICATION-AUTHORITY` | public bypass compile-fail/API audit, NaN/±Inf exhaustive pre-mutation rejection equality, finite determinism corpus와 boss/movement/skill differential pass | none |
| `WP-CRAR-06` | implementation: `PE-CRAR-009`; supporting: `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-007`, `PE-CRAR-008` | standalone | active-session reconnect history, cursor/index, terminal durable `BattleResultRecord/Receipt`와 optional external debug archive의 lifetime owner를 분리한다. | history를 `8192 events OR 4 MiB OR 180 battle seconds`에서 먼저 닫히는 bounded window로 만들고 cursor-before-floor full resync, default-off archive `64 MiB/24h` cap을 구현한다. | `WP-CRAR-01`, `AG-CRAR-01-APPLICATION-AUTHORITY`, `WP-CRAR-05` | window delta identity, overflow/expiry authority equality, bounded lookup/work, reconnect full resync, result/receipt durability·idempotency, archive absent/expired continuation | `WP-CRAR-07`의 same-run performance gate 전에는 latency/memory PASS를 주장하지 않음 |
| `WP-CRAR-07` | implementation: `PE-CRAR-008`; validation_only: `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-009` | standalone | `WP-CRAR-01` pre-cutover baseline과 동일 workload를 interleaved 실행하는 permanent benchmark/CI guard, touched-state staging과 indexed tail 최적화를 소유한다. | 핵심 지표 no-regression과 ordinary command/tick `<0.5ms p95`, snapshot `<0.5ms p95/<64KiB`, checkpoint export `<0.1ms p95/<32KiB`, last-10 `<=10us p95` 및 1K→100K `<=2x`, staging `<0.5ms p95` cap을 강제한다. | `WP-CRAR-01`, `WP-CRAR-05`, `WP-CRAR-06`, `AG-CRAR-02-PROTOCOL-CUTOVER` | event-heavy boss와 64-session/8-way workload에서 same-run confidence evidence, allocation/RSS/history/serialization numeric guard, cap breach CI failure | baseline noise는 interleaved samples/confidence로 표시; product SLA가 아니라 engineering guard |
| `WP-CRAR-08` | implementation: `PE-CRAR-010`; supporting: `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-009` | standalone | Core checkpoint codec/state identity와 server store envelope/checksum에 V5 target writer, explicit V4 importer와 certification gate를 둔다. | 새 write는 V5만 생성하고 canonical V4 corpus는 semantic import/continuation oracle로 유지하며 인증 전 V4 reader removal을 거부한다. | `WP-CRAR-01`, `WP-CRAR-04`, `AG-CRAR-01-APPLICATION-AUTHORITY` | V4→V5 identity/state/continuation differential, V5 round-trip/corruption rejection, target-only write, premature reader removal negative gate | Released/Deployed saves는 evidence-backed N/A; canonical development V4 corpus만 migration authority |
| `WP-CRAR-09` | implementation: `PE-CRAR-007`; supporting: `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-009`, `PE-CRAR-010` | member of `AG-CRAR-02-PROTOCOL-CUTOVER` | Core domain에서 독립된 protocol v1 envelope/DTO, exhaustive server mapper와 command/error/result/snapshot/reconnect adapters를 만든다. | unversioned direct domain serde를 제거하고 semantic parity fixture를 생산하며 server는 v1만 publish/consume한다. | `WP-CRAR-05`, `WP-CRAR-06`, `WP-CRAR-08`; `WP-CRAR-10`과 상호 schema/consumer prerequisite | exhaustive mapper compile coverage, wire golden/parity/round-trip, unknown version rejection, legacy server path 0 | released compatibility 없음; atomic group 밖 dual stack 금지 |
| `WP-CRAR-10` | implementation: `PE-CRAR-007`; validation_only: `PE-CRAR-001` | member of `AG-CRAR-02-PROTOCOL-CUTOVER` | Unity `GameWsClient`, command/result types, snapshot store, battle state와 62개 raw-consumer surface를 typed protocol v1 adapter로 전환한다. | raw `JObject` central interpretation과 legacy message branch를 제거하고 server fixture와 동일 version/semantic mapping을 사용한다. | `WP-CRAR-01`; `WP-CRAR-09`와 상호 schema/consumer prerequisite | EditMode/PlayMode fixture, reconnect/revision/result/error parity, current consumer failure 0, legacy Unity parser/branch 0 | Unity runner availability는 실행 시 증명해야 하며 current PASS 아님 |
| `WP-CRAR-11` | validation_only: `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-006`, `PE-CRAR-007`, `PE-CRAR-008`, `PE-CRAR-009`, `PE-CRAR-010` | standalone | 모든 cutover 후 legacy/bypass/fallback surface 제거 audit, preservation manifest reconciliation과 cross-Core/server/Unity integration suite를 소유한다. | 승인되지 않은 의미 변경, competing authority, unreadable persistence, unversioned consumer와 미소유 validation seam을 0으로 만든다. | 모든 preceding Final Execution Node, 특히 `WP-CRAR-07` | 10/10 acceptance evidence, full Core/server/Unity suites, feature manifest 100%, legacy/bypass/dual writer 0, docs/fixtures currentness | Released/production promotion과 임시 기능 정책 구현은 범위 밖 |

## Dependency Graph

| Prerequisite Work Package | Dependent Work Package | Dependency Type | Evidence/reason | Broken prerequisite, valid state or completion claim if ignored |
| --- | --- | --- | --- | --- |
| `WP-CRAR-01` | `WP-CRAR-04` | `policy`, `validation` | validated publication 전 current feature/data load oracle를 고정해야 함 | valid data 의미가 rewrite 과정에서 조용히 바뀜 |
| `WP-CRAR-01` | `WP-CRAR-02` | `policy`, `validation` | rejection/accepted behavior baseline이 transaction 교체의 oracle | exact equality가 기능 손실을 가릴 수 있음 |
| `WP-CRAR-01` | `WP-CRAR-03` | `policy`, `validation` | route inventory가 lifecycle owner registry의 완전성 기준 | 미등록 route 또는 승인되지 않은 알고리즘 변경 발생 |
| `WP-CRAR-04` | `WP-CRAR-02` | `authority`, `schema_data` | application root는 opaque validated publication만 받아야 함 | 새 executor가 bare invalid data로 authority를 구성 가능 |
| `WP-CRAR-04` | `WP-CRAR-03` | `authority`, `schema_data` | lifecycle construction context도 validated data boundary를 소비 | closed state가 invalid data authority를 감쌀 수 있음 |
| `WP-CRAR-02` | `WP-CRAR-03` | `transaction`, `authority` | route owner는 typed command executor/commit port를 통해서만 mutate | owner registry와 실제 commit root가 불일치 |
| `WP-CRAR-03` | `WP-CRAR-02` | `authority`, `transaction` | executor는 closed lifecycle과 route-specific authority 없이는 prevalidate 불가 | invalid phase가 staging 또는 commit 이후 발견됨 |
| `WP-CRAR-01` | `WP-CRAR-05` | `policy`, `validation` | finite battle algorithm 보존 corpus가 kernel privacy cutover 기준 | gameplay drift를 authority 강화로 오판 |
| `WP-CRAR-04` | `WP-CRAR-05` | `schema_data`, `authority` | kernel setup input은 ValidatedGameData에서만 파생 | invalid authored float/state가 kernel 생성 뒤 도착 |
| `WP-CRAR-02` | `WP-CRAR-05` | `transaction` | battle command port가 root commit outcome 규약을 소비 | kernel error와 application commit 결과가 분리됨 |
| `WP-CRAR-03` | `WP-CRAR-05` | `authority`, `producer_consumer` | battle lifecycle owner가 private kernel의 유일 producer/consumer | public bypass 또는 competing battle authority 발생 |
| `WP-CRAR-01` | `WP-CRAR-06` | `validation`, `persistence_lifetime` | reconnect/result baseline과 event identity가 retention rewrite oracle | bounded window가 delta/result 의미를 바꿈 |
| `WP-CRAR-02` | `WP-CRAR-06` | `transaction`, `persistence_lifetime` | history/result publication은 accepted commit 뒤에만 발생 | rejected command event나 durable intent가 남음 |
| `WP-CRAR-03` | `WP-CRAR-06` | `authority`, `persistence_lifetime` | active session/terminal result의 lifetime owner가 lifecycle state에 결속 | archive/history/result가 authority를 공유하거나 잃음 |
| `WP-CRAR-05` | `WP-CRAR-06` | `producer_consumer`, `schema_data` | kernel sequence/event output이 indexed window와 receipt input | event ordering/cursor identity가 재구성 불가 |
| `WP-CRAR-01` | `WP-CRAR-08` | `migration_compatibility`, `validation` | V4 canonical corpus와 continuation oracle을 먼저 고정 | migration 손실을 새 정상으로 승인 |
| `WP-CRAR-04` | `WP-CRAR-08` | `schema_data`, `migration_compatibility` | importer 결과는 current validated data context에서 복원 | V4 identity가 invalid/bare data 경로를 재개방 |
| `WP-CRAR-02` | `WP-CRAR-08` | `transaction`, `persistence_lifetime` | checkpoint write/usage는 accepted commit과 durable intent 규약을 따름 | rejected load/save가 state나 usage를 부분 변경 |
| `WP-CRAR-03` | `WP-CRAR-08` | `authority`, `migration_compatibility` | closed lifecycle이 V4/V5 상태 복원 target을 정의 | parallel optional 또는 impossible state를 import |
| `WP-CRAR-05` | `WP-CRAR-09` | `schema_data`, `producer_consumer` | private kernel projection이 protocol DTO source | wire가 다시 kernel/domain fields에 직접 결합 |
| `WP-CRAR-06` | `WP-CRAR-09` | `schema_data`, `persistence_lifetime` | bounded delta/full resync/result receipt가 protocol mapping 입력 | reconnect cursor와 durable result semantic parity 깨짐 |
| `WP-CRAR-08` | `WP-CRAR-09` | `schema_data`, `migration_compatibility` | save/load result/version errors가 protocol v1에 exhaustive mapping | client가 V4/V5 migration outcome을 해석 불가 |
| `WP-CRAR-09` | `WP-CRAR-10` | `schema_data`, `producer_consumer`, `legacy_removal` | server v1 fixture와 mapper가 Unity consumer contract 생산 | unreadable wire 또는 legacy fallback 필요 |
| `WP-CRAR-10` | `WP-CRAR-09` | `producer_consumer`, `schema_data` | 실제 Unity exhaustive consumer mapping이 server schema 완전성을 닫음 | server-only schema가 current consumer surface를 누락 |
| `WP-CRAR-01` | `WP-CRAR-07` | `external_evidence`, `validation` | same-run pre-cutover baseline과 exact workload가 non-regression 비교 기준 | 성능 PASS의 비교 basis가 없음 |
| `WP-CRAR-05` | `WP-CRAR-07` | `producer_consumer`, `validation` | private kernel/touched-state staging의 final hot path가 측정 대상 | 중간 API 성능을 최종 성능으로 오판 |
| `WP-CRAR-06` | `WP-CRAR-07` | `persistence_lifetime`, `validation` | bounded history/index/full resync의 final complexity를 측정 | retained-history 독립성과 scaling cap 검증 불가 |
| `WP-CRAR-09` | `WP-CRAR-07` | `schema_data`, `validation` | final server serialization path가 snapshot/result 비용 측정 대상 | domain-only benchmark가 wire regression을 놓침 |
| `WP-CRAR-10` | `WP-CRAR-07` | `producer_consumer`, `validation` | current Unity consumer까지 성공해야 protocol workload가 대표성 가짐 | 서버 최적화가 client parse/shape 실패를 숨김 |
| `WP-CRAR-01` | `WP-CRAR-11` | `validation` | 최종 inventory reconciliation은 initial oracle을 소비 | 10개 결함 수정 중 기능 제거를 탐지 못함 |
| `WP-CRAR-04` | `WP-CRAR-11` | `validation`, `legacy_removal` | bare/fixture capability removal 최종 audit | production bypass가 남은 채 완료 주장 |
| `WP-CRAR-02` | `WP-CRAR-11` | `validation`, `legacy_removal` | rejection equality와 post-commit outcome 최종 matrix | 일부 route가 old error semantics 유지 |
| `WP-CRAR-03` | `WP-CRAR-11` | `validation`, `legacy_removal` | route owner/no dual-write registry 최종 audit | competing authority를 놓침 |
| `WP-CRAR-05` | `WP-CRAR-11` | `validation`, `legacy_removal` | public mutation/non-finite boundary 최종 audit | kernel bypass가 남음 |
| `WP-CRAR-06` | `WP-CRAR-11` | `validation`, `persistence_lifetime` | reconnect/result/archive cross-consumer 검증 | expiry가 continuation/result를 훼손 |
| `WP-CRAR-08` | `WP-CRAR-11` | `validation`, `migration_compatibility` | V4 certification/reader-removal gate 최종 확인 | unreadable save 또는 조기 reader 제거 |
| `WP-CRAR-09` | `WP-CRAR-11` | `validation`, `legacy_removal` | server version/legacy path audit | direct domain serde가 남음 |
| `WP-CRAR-10` | `WP-CRAR-11` | `validation`, `legacy_removal` | Unity current consumer/legacy branch audit | same-version 완료 claim이 거짓 |
| `WP-CRAR-07` | `WP-CRAR-11` | `validation`, `external_evidence` | 최종 completion은 permanent numeric guard PASS를 요구 | 기능만 맞고 성능이 퇴행한 상태를 완료로 승인 |

## Cyclic SCC Disposition Set

| SCC member Work Packages | Disposition | Boundary/edge reconstruction | Evidence | Recomputed cycle result |
| --- | --- | --- | --- | --- |
| `WP-CRAR-02`, `WP-CRAR-03` | `coimplement_atomic` | `AG-CRAR-01-APPLICATION-AUTHORITY` | typed commit executor는 closed lifecycle/route owner가 필요하고 route owner는 같은 commit port 없이는 authority가 아님 | condensed acyclic |
| `WP-CRAR-09`, `WP-CRAR-10` | `coimplement_atomic` | `AG-CRAR-02-PROTOCOL-CUTOVER` | pre-release hard cutover에서 server producer와 Unity consumer는 같은 schema/version/legacy-removal 경계로만 유효 | condensed acyclic |

## Atomic Group Set

| Atomic Group ID | Member Work Packages | Indivisible boundary and evidence |
| --- | --- | --- |
| `AG-CRAR-01-APPLICATION-AUTHORITY` | `WP-CRAR-02`, `WP-CRAR-03` | 모든 mutating route의 lifecycle prevalidation, staging, typed commit과 single owner를 한 root에서 동시에 전환해야 missing/dual authority가 없음 |
| `AG-CRAR-02-PROTOCOL-CUTOVER` | `WP-CRAR-09`, `WP-CRAR-10` | server v1 producer와 Unity v1 consumer, semantic fixture와 legacy removal을 같이 cutover해야 reader/writer 불일치나 dual stack이 없음 |

Standalone Final Execution Nodes: `WP-CRAR-01`, `WP-CRAR-04`, `WP-CRAR-05`, `WP-CRAR-06`, `WP-CRAR-07`, `WP-CRAR-08`, `WP-CRAR-11`.

## Ordered Phase Set

| Phase ID | Primary Stable Element ID Selectors | Supporting policy relations | Assigned final execution nodes | Dependencies | Exit acceptance |
| --- | --- | --- | --- | --- | --- |
| `PHASE-CRAR-01-PRESERVATION-BASELINE` | `PE-CRAR-001` | `PE-CRAR-002`, `PE-CRAR-008` cutover oracle와 pre-cutover baseline; 나머지 acceptance의 initial evidence | `WP-CRAR-01` | `FPF-CRAR-P2G30-001` | stable feature inventory 100%, current behavior/rejection/V4/protocol oracle와 event-heavy/64-session baseline이 재현 가능하고 production behavior 변화 0 |
| `PHASE-CRAR-02-VALIDATED-DATA` | `PE-CRAR-005` | `PE-CRAR-002`, `PE-CRAR-004` construction boundary | `WP-CRAR-04` | `PHASE-CRAR-01-PRESERVATION-BASELINE` | production은 opaque validated publication만 소비하고 invalid authoring은 pre-authority recoverable failure이며 test/admin capability가 분리됨 |
| `PHASE-CRAR-03-APPLICATION-AUTHORITY` | `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004` | `PE-CRAR-005`, `PE-CRAR-008`, `PE-CRAR-010` root construction/performance/persistence support | `AG-CRAR-01-APPLICATION-AUTHORITY` | `PHASE-CRAR-01-PRESERVATION-BASELINE`, `PHASE-CRAR-02-VALIDATED-DATA` | 모든 mutating route가 closed lifecycle의 single owner와 typed executor를 사용하고 모든 `Err` exact equality, dual writer와 algorithm drift 0 |
| `PHASE-CRAR-04-BATTLE-KERNEL` | `PE-CRAR-006` | `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-005`, `PE-CRAR-008` | `WP-CRAR-05` | `PHASE-CRAR-01-PRESERVATION-BASELINE`, `PHASE-CRAR-02-VALIDATED-DATA`, `PHASE-CRAR-03-APPLICATION-AUTHORITY` | private kernel/scoped ports와 finite types가 적용되고 public bypass 0, non-finite exact rejection, finite boss/movement/skill determinism parity |
| `PHASE-CRAR-05-SESSION-HISTORY-RESULT` | `PE-CRAR-009` | `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-007`, `PE-CRAR-008` | `WP-CRAR-06` | `PHASE-CRAR-03-APPLICATION-AUTHORITY`, `PHASE-CRAR-04-BATTLE-KERNEL` | bounded indexed reconnect, cursor-floor full resync, durable result/receipt와 optional expiring archive의 lifetime/authority가 분리되고 continuation이 보존됨 |
| `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | `PE-CRAR-010` | `PE-CRAR-002`, `PE-CRAR-004`, `PE-CRAR-009` | `WP-CRAR-08` | `PHASE-CRAR-01-PRESERVATION-BASELINE`, `PHASE-CRAR-02-VALIDATED-DATA`, `PHASE-CRAR-03-APPLICATION-AUTHORITY` | V5 target-only writer, V4 semantic importer/oracle, continuation differential과 certification 전 reader-removal negative gate가 통과 |
| `PHASE-CRAR-07-PROTOCOL-CUTOVER` | `PE-CRAR-007` | `PE-CRAR-001`, `PE-CRAR-002`, `PE-CRAR-003`, `PE-CRAR-004`, `PE-CRAR-009`, `PE-CRAR-010` | `AG-CRAR-02-PROTOCOL-CUTOVER` | `PHASE-CRAR-04-BATTLE-KERNEL`, `PHASE-CRAR-05-SESSION-HISTORY-RESULT`, `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | server/Unity가 protocol v1 semantic parity로 함께 hard cutover되고 current consumer failure, unversioned domain serde와 legacy wire path가 0 |
| `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | `PE-CRAR-008` | 모든 Freeze selector의 final differential, legacy removal과 cross-boundary validation | `WP-CRAR-07`, `WP-CRAR-11` | `PHASE-CRAR-01-PRESERVATION-BASELINE`부터 `PHASE-CRAR-07-PROTOCOL-CUTOVER`까지 전체 | same-run 핵심 지표 non-regression/numeric caps와 10/10 policy acceptance, full Core/server/Unity suites, feature inventory 100%, legacy/bypass/dual path 0 |

## Intermediate-State Audit

| Phase ID | State immediately after completion | Applicable validity checks | Completion-claim evidence | Result/limitation |
| --- | --- | --- | --- | --- |
| `PHASE-CRAR-01-PRESERVATION-BASELINE` | production behavior는 그대로이며 manifest, oracle, benchmark input만 추가됨 | feature/route/test coverage, deterministic fixture identity, baseline reproducibility, no source behavior delta | inventory audit, current regression corpus, V4/protocol goldens, event-heavy/concurrent baseline receipt | `valid`; 결함 수정이나 performance PASS는 아직 주장하지 않음 |
| `PHASE-CRAR-02-VALIDATED-DATA` | 기존 application/battle 알고리즘은 유지하되 모든 production construction이 opaque validated data를 사용 | reader/publication match, invalid pre-authority failure, fixture/admin capability isolation, no bare fallback | load/validation parity, invalid matrix, compile-fail/API audit | `valid`; transaction/lifecycle 결함은 아직 current지만 data boundary 자체에 dual publication path 없음 |
| `PHASE-CRAR-03-APPLICATION-AUTHORITY` | world/application mutating routes가 closed lifecycle와 one executor/owner를 사용; battle internals는 아직 기존 구현 | one authority, exact `Err` equality, accepted post-commit outcome, route completeness, no old/new simultaneous writer | six-command/StartNewGame/retry/durable-intent matrix와 owner registry audit | `valid`; slice별 hard cutover가 모두 끝나 old root fallback은 없고 battle kernel privacy는 다음 Phase 범위 |
| `PHASE-CRAR-04-BATTLE-KERNEL` | battle authority가 private kernel/scoped ports/finite inputs로 닫혔고 protocol/history는 current adapters를 유지 | public mutation absence, finite construction, pre-mutation non-finite rejection, algorithm parity | compile-fail/API audit, exhaustive non-finite matrix, boss/movement/skill deterministic differential | `valid`; current protocol adapter는 public DTO projection만 소비하고 kernel bypass가 아님 |
| `PHASE-CRAR-05-SESSION-HISTORY-RESULT` | current transport adapter가 bounded delta/full resync와 durable result projection을 사용; archive는 default-off | memory/work cap, cursor floor, result authority, expiry/overflow continuation, no archive-as-authority | retention/resync/property tests, persistence idempotency/restart tests | `valid`; protocol v1은 아직 아니지만 current adapter의 reader/writer shape는 함께 갱신되어 unreadable 중간 상태가 없음 |
| `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | Core/server가 V5만 쓰고 V4를 명시적으로 import; 현재 protocol consumer에는 save semantic result를 기존 adapter가 제공 | writer/reader version, checksum, identity/state/continuation, reader retention gate | canonical V4 import/V5 round-trip/old-new continuation differential | `valid`; V4 writer와 dual-format write는 없고 V4 reader 제거는 certification까지 금지 |
| `PHASE-CRAR-07-PROTOCOL-CUTOVER` | server와 Unity가 v1만 사용하고 legacy wire reader/writer/parser가 제거됨 | same version, exhaustive mapper, producer/consumer goldens, reconnect/result/save errors, rollback/legacy absence | Core/server mapper tests, Unity EditMode/PlayMode contract and reconnect suites | `valid`; pre-release coordinated hard cutover이며 released client compatibility나 dual stack을 주장하지 않음 |
| `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | 최종 architecture와 validation guard가 모두 current이며 legacy/bypass/fallback surface가 없음 | all acceptance, preservation differential, numeric performance, migration, protocol, persistence, cross-runtime consumers | permanent benchmark gate, 10/10 acceptance evidence index, full suites와 repository audits | `valid`; production promotion과 임시 정책 기능 구현은 범위 밖 |

## Integration Validation Ownership Map

| Integration acceptance / source | Seam disposition | Read-only locator, invocation and evidence contract | Owning Final Execution Node / Phase | Availability prerequisite | Evidence / limitation |
| --- | --- | --- | --- | --- | --- |
| `IVA-CRAR-01` — 전체 기능 inventory와 slice/final semantic preservation | `assigned_execution_node` | durable feature manifest, behavior route/test index와 old/new differential fixtures | `WP-CRAR-01` / `PHASE-CRAR-01-PRESERVATION-BASELINE`; `WP-CRAR-11` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | current Rulebook/code/tests; final rerun은 모든 cutover | baseline 고정과 final 100% reconciliation을 분리 소유 |
| `IVA-CRAR-02` — 모든 rejection의 authoritative/durable exact equality와 retry | `assigned_execution_node` | six-command, StartNewGame, post-commit projection와 durable-intent matrix | `AG-CRAR-01-APPLICATION-AUTHORITY` / `PHASE-CRAR-03-APPLICATION-AUTHORITY`; `WP-CRAR-11` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | validated data와 application root | diagnostic state는 authority snapshot 밖에서 별도 관찰 |
| `IVA-CRAR-03` — closed lifecycle, route single owner와 no dual write | `assigned_execution_node` | durable owner-registry completeness/API audit와 invalid-phase tests | `AG-CRAR-01-APPLICATION-AUTHORITY` / `PHASE-CRAR-03-APPLICATION-AUTHORITY` | Phase 2 publication | route별 hard switch; fallback writer 불허 |
| `IVA-CRAR-04` — production validated publication과 test/admin capability isolation | `assigned_execution_node` | loader invalid matrix, compile-fail/public API audit, typed AdminCommand tests | `WP-CRAR-04` / `PHASE-CRAR-02-VALIDATED-DATA` | current live RON corpus | fixture feature가 production artifact에 없는지를 함께 증명 |
| `IVA-CRAR-05` — private kernel, finite boundary와 deterministic gameplay | `assigned_execution_node` | public bypass audit, NaN/Inf matrix, boss/movement/skill differential corpus | `WP-CRAR-05` / `PHASE-CRAR-04-BATTLE-KERNEL` | preservation oracle, validated data, application authority | bitwise cross-platform physics replay는 frozen scope 밖 |
| `IVA-CRAR-06` — bounded reconnect/history와 durable result/receipt/archive independence | `assigned_execution_node` | window/overflow/cursor/full-resync property tests, restart/idempotency/archive expiry fixtures | `WP-CRAR-06` / `PHASE-CRAR-05-SESSION-HISTORY-RESULT`; `WP-CRAR-11` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | final kernel sequence/events; protocol integration은 Phase 7 | archive 부재가 gameplay continuation을 막지 않음 |
| `IVA-CRAR-07` — V4 semantic import, V5 target-only write와 continuation | `assigned_execution_node` | canonical V4 fixture, V5 round-trip/checksum, old/new continuation differential, removal guard | `WP-CRAR-08` / `PHASE-CRAR-06-CHECKPOINT-MIGRATION` | Phase 3 lifecycle state | released save population 없음; canonical corpus가 authority |
| `IVA-CRAR-08` — versioned command/result/error/snapshot semantic parity와 current Unity consumer | `assigned_execution_node` | server mapper goldens + Unity EditMode/PlayMode contract/reconnect fixtures | `AG-CRAR-02-PROTOCOL-CUTOVER` / `PHASE-CRAR-07-PROTOCOL-CUTOVER`; `WP-CRAR-11` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | final Core DTO/history/save semantics | 같은 atomic group에서 legacy server/Unity path를 함께 제거 |
| `IVA-CRAR-09` — current-relative non-regression과 permanent numeric caps | `assigned_execution_node` | event-heavy boss/64-session interleaved harness, allocation/RSS/history/serialization receipts | `WP-CRAR-01` / `PHASE-CRAR-01-PRESERVATION-BASELINE`; `WP-CRAR-07` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | baseline before Phase 2; final after Phase 7 | `GAP-CRAR-P2G20-001`을 baseline과 comparison 두 owner로 완결 |
| `IVA-CRAR-10` — 10개 acceptance와 cross-runtime legacy/bypass absence | `assigned_execution_node` | final evidence index, static/API audits and end-to-end Core/server/Unity suites | `WP-CRAR-11` / `PHASE-CRAR-08-PERFORMANCE-INTEGRATION` | all final execution nodes | production promotion/임시 정책 구현 제외 |
| `IVA-CRAR-11` — repository-wide existing regression runners | `existing_read_only` | `/mnt/f/work/simulator`에서 task-owned `CARGO_TARGET_DIR=/tmp/<id>`로 `cargo test -p game_core`, `cargo test -p game_server`; Unity project의 existing EditMode/PlayMode test runner; exit 0/zero failure | `N/A` | assigned focused tests와 runner availability | runner 자체만 existing seam이며 최종 실행/evidence는 `WP-CRAR-11`이 소유 |
| `IVA-CRAR-12` — P2G-20 current lightweight performance characterization | `audit_local_read_only` | `/tmp/core_arch_perf`의 reviewed probe를 동일 toolchain/input으로 release 실행하고 raw sample을 task-owned `/tmp`에 보존 | `N/A` | reviewed probe hash/current source basis | ordinary current actual만 관찰; event-heavy/concurrent baseline 또는 final PASS를 대신하지 않음 |

## Phase Plan Validation Receipt Summary

- Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-001`
- Evaluated Phase Plan Ref: `PP-CRAR-P2G40-001`
- Bound Final Policy Freeze Ref: `FPF-CRAR-P2G30-001`
- Validator/Method Version: `p2g40-receipt-validator/1.0.0`
- Immutable Machine Detail: `P2G_40_PHASE_PLAN_VALIDATION_RECEIPT.json`
- Overall Result: `passed`

## P2G-40 Complete Stage Result

- Owner Stage: `P2G-40`
- Required Result Artifact/Relation Set:
  - Phase Plan Ref: `PP-CRAR-P2G40-001`
  - Phase Plan Validation Receipt Ref: `PVR-CRAR-P2G40-001`
  - P2G-40 Resolution Record Ref Set: `explicit empty`
  - Applicable Obligation Supersession Record Ref Set: `explicit empty`
- Cross-Reference/Coverage Evidence: `PCE-CRAR-P2G40-001 + PVR-CRAR-P2G40-001`
- Exit Condition Validation: `passed`
- Current Selection Result: `PP-CRAR-P2G40-001` selected as current ordered Phase Plan

## P2G-40 Exit Validation

1. exact Phase Plan/Freeze input에 대한 repository standard Receipt가 `passed`이고 failure가 empty다.
2. stable selector 10개가 Freeze member 10개로 exact resolve되고 primary Phase assignment는 missing/duplicate/extra 0이다.
3. Work Package 11개는 Atomic Group member 4개와 standalone 7개로 exactly partition된다.
4. dependency graph의 cyclic SCC 2개가 모두 `coimplement_atomic` disposition되고 9-node condensation graph는 acyclic이다.
5. Final Execution Node 9개는 8개 ordered Phase에 exactly once 배정된다.
6. Intermediate-State Audit 8개와 Integration Validation Ownership row 12개가 complete다.
7. 병렬화는 승인하지 않았고 구조 사실은 8-Phase `multi_phase`다.
8. `GAP-CRAR-P2G20-001`은 pre-cutover baseline과 final comparison owner 모두를 가진다.
9. applicable Change/obligation/resolution/supersession set은 모두 explicit empty이며 새 material policy는 없다.

```text
is_p2g40_complete(PP-CRAR-P2G40-001, PVR-CRAR-P2G40-001) = true
```

## Required Stop

- Completed Stage: `P2G-40` only
- Recommended Next Stage: `P2G-50 — current Freeze/Plan/Receipt를 exact basis로 Goal Set 작성`
- New User Request Required: `Yes`
- Goal Set: `not created`
- Goal Activation: `Not Activated`
- Implementation Authorized: `No`
- Production mutation: `none`
- Test execution: `none`; Receipt generator/validator는 문서 구조, selector와 graph만 검증
