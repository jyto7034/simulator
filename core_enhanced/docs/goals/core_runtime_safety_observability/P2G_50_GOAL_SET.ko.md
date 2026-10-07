# U02 설정과 관측 — Master/Phase Goal Set

## Goal Set

- Goal Set Ref: GS-CRSO-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRSO-005; FPF-CRSO-001; PP-CRSO-001; PVR-CRSO-001
- Goal Structure Fact: multi_phase
- Phase Plan Ref: PP-CRSO-001
- Final Policy Freeze Ref: FPF-CRSO-001
- Complete Goal Member Set: GOAL-CRSO-MASTER, GOAL-CRSO-S1, GOAL-CRSO-S2
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G는 구현 권한을 부여하지 않는다. 사용자의 2026-10-04 연속 요청이 별도로 GEX 실행·독립 감사·교정을 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRSO-S1 | GOAL-CRSO-S1 | WP-CRSO-CONFIG, WP-CRSO-CORE, WP-CRSO-PORTS, WP-CRSO-S1VERIFY | 1 |
| PHASE-CRSO-S2 | GOAL-CRSO-S2 | WP-CRSO-OBSERVE, WP-CRSO-EXPORT, WP-CRSO-S2VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRSO-001 IN-001 기존 계획과 유효 증거 재사용·불필요 반복 금지. 정해진 initial/final 횟수의 별도 user directive는 N/A. 각 Phase의 독립 완료와 필수 검사 우선.
- Baseline Evidence Key: command/selectors, fixture/input, relevant source/data/test/config, rustc/Cargo.lock identity, raw result and owning Goal.
- Reuse Rule: exact relevant input identity와 evidence contract가 그대로인 경우에만 재사용. U01 감사된 Core baseline은 GEX-10 currentness로 비교하고 source 변경 이후 성공으로 가장하지 않는다.
- Production Input Invalidation / Focused Revalidation Map: config/parser→V01–03; generation/expression→V03–06; application/restore→V04/V08/V12; history→V07; host/CLI→V05/V08; observer/exporter/dependency→V09–12 및 연결된 S1 seam.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 해당 seam을 current source에서 nonzero 실행. shared harness/schema/persistence/transport/dependency 변경은 영향받는 전체 set 무효화.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 공통 경계 변경은 해당 소비자를 포함한 관련 seam 전체를 무효화하고 focused 재검증 후 final baseline을 고정한다.
- Normal Full-Validation Count: exact-source initial Core baseline 1회 인수, S1과 S2 독립 완료 source별 final 1회씩. 사용자의 별도 initial1+final1 횟수 고정 directive는 없으며 각 Phase의 명시적 완료 검사와 영향 기반 재사용을 적용한다. 실패/실행 후 변경의 재시도는 별도다.
- Phase-Start Rule: Phase 시작 자체는 baseline 무효화 사유가 아니다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRSO-S1 GEX-10; audited U01 exact source core baseline 인수와 이미 실패한 server baseline 보존, 부족한 실제 경계만 추가 실행.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRSO-S2가 최종 통합 baseline을 소유한다. 각 독립 Phase의 verification node가 final core default/test-fixtures 전체, 변경 server, format/check/clippy와 공개 integration을 실행한다. S2는 final current U02 source를 소유한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 성공으로 세지 않고 원인 교정 후 영향 seam을 재실행한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRSO-V01-S1 / CODE_REVIEW_001.ko.md V01; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V01; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V02-S1 / CODE_REVIEW_001.ko.md V02; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V02; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V03-S1 / CODE_REVIEW_001.ko.md V03; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V03; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V04-S1 / CODE_REVIEW_001.ko.md V04; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V04; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V05-S1 / CODE_REVIEW_001.ko.md V05; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V05; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V06-S1 / CODE_REVIEW_001.ko.md V06; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V06; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V07-S1 / CODE_REVIEW_001.ko.md V07; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V07; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V08-S1 / CODE_REVIEW_001.ko.md V08; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V08; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V09-S2 / CODE_REVIEW_001.ko.md V09; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S2 / WP-CRSO-S2VERIFY | execution/s2/COVERAGE.json V09; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V10-S2 / CODE_REVIEW_001.ko.md V10; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S2 / WP-CRSO-S2VERIFY | execution/s2/COVERAGE.json V10; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V11-S2 / CODE_REVIEW_001.ko.md V11; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S2 / WP-CRSO-S2VERIFY | execution/s2/COVERAGE.json V11; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V12-S1 / CODE_REVIEW_001.ko.md V12; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S1 / WP-CRSO-S1VERIFY | execution/s1/COVERAGE.json V12; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |
| IVA-CRSO-V12-S2 / CODE_REVIEW_001.ko.md V12; Frozen acceptance selectors | assigned_execution_node | GOAL-CRSO-S2 / WP-CRSO-S2VERIFY | execution/s2/COVERAGE.json V12; actual nonzero test/CLI/service execution; exact source/config/input/seed/lock/toolchain; expected and observed outputs; raw failure preserved |

## Master Goal Member `GOAL-CRSO-MASTER`

- Goal ID: GOAL-CRSO-MASTER
- Goal Kind: master
- Complete Phase Goal ID Set: GOAL-CRSO-S1, GOAL-CRSO-S2
- Derived Sequential/Parallel Relations: S1 독립 감사 통과 후 S2. 병렬 구현 없음.
- Coordination Dependencies/Conditions: PP-CRSO-001 S1VERIFY→OBSERVE edge; user independent-audit prerequisite.
- Derived Cross-Phase Invariants and Source Refs: FPF-CRSO-001 전체; settings hash/semantic identity 분리, 원래 allowance/rollback/first fault, 불변 generation, 진단과 gameplay 분리.
- Derived Integration Acceptance and Source Refs: PP-CRSO-001 integration 전체 13개, 위 common mapping에서 실행 owner 파생.
- Integration Validation Seam Disposition / Locator or Owning Phase Goal and Final Execution Node: common integration mapping; Master는 구현 seam을 소유하지 않는다.
- Baseline Reuse Coordination / Initial and Final Full Baseline Owning Phase Goal: common reuse contract. S1 기존 근거 인수, S2 final integrated baseline; 서로 다른 current source를 혼동하지 않는다.
- Overall Completion Conditions: 두 Phase가 각각 구현·실행 증거·복원 증명과 구현 비참여 GPT-6 Astra/High 독립 감사에 통과하고 U02 전체 12 frozen acceptance가 current evidence로 추적된다. 경고·남은 문제와 다음 U03 위치를 기록한다.
- Applicable Integration Risk / Reviewer Requirement and Evidence: settings가 observer에 의해 바뀌거나 export failure가 gameplay budget/RNG에 영향을 주지 않는지 별도 독립 reviewer가 확인한다.
- Assigned Phase / Final Execution Node / Implementation Work Ownership: None

## Phase Goal Member `GOAL-CRSO-S1`

- Goal ID: GOAL-CRSO-S1
- Goal Kind: phase
- Phase ID: PHASE-CRSO-S1
- Assigned Final Execution Node Set: WP-CRSO-CONFIG, WP-CRSO-CORE, WP-CRSO-PORTS, WP-CRSO-S1VERIFY
- Covered Policy Element Revision Ref Set: PER-CRSO-01-REVIEWED-002, PER-CRSO-02-REVIEWED-002, PER-CRSO-03-REVIEWED-002, PER-CRSO-04-REVIEWED-002, PER-CRSO-05-REVIEWED-002, PER-CRSO-06-REVIEWED-002, PER-CRSO-07-REVIEWED-002
- Covered Decision Revision Ref Set: DR-CRSO-01-002, DR-CRSO-02-002, DR-CRSO-03-002, DR-CRSO-04-002, DR-CRSO-05-002, DR-CRSO-06-002, DR-CRSO-07-002
- Derived Observable Policy Acceptance Coverage: FPF-CRSO-001의 위 exact member acceptance; 아래 completion은 그 수락 기준의 실행 해설이며 변경하지 않는다.
- Objective / Contract-Visible Result: trusted host가 선택한 validated RON profile이 실제 server·library·authoring CLI의 첫 실행부터 일관되게 적용되며 기존 기본 동작을 보존한다.
- User-Visible Problem: 한도 기본값이 여러 Rust owner에 분산되고 host에서 주입할 경로가 없으며 server는 현재 공개 API와 맞지 않아 컴파일되지 않는다.
- User-Visible Completion Condition Set: 기본값 parity; invalid 지정 파일은 publication 이전 거절; 두 profile 격리; 첫 spawn·restore·parse/compile/runtime/query·history 적용; 실제 server와 CLI contract 및 저장 실패 복원 통과.
- In-Scope / Out-of-Scope: TD01–04, CONFIG/CORE/PORTS/S1VERIFY. 제외: metrics/Grafana 구현, U03 새 fault 정책, hot reload, 새 cap, save 삭제/자동 변환.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001 RQ01–04: 숫자 default/ceiling 분리, generation/compiler profile 연결, 첫 BattleCore 생성 경로, history deserialize/import, server private API→public CoreApplication.
- Goal-Local Source of Truth / Owning Layer: runtime_safety immutable validated settings; generation owns admission identity; existing native/authoring ledgers retain safety authority; application owns commits; host owns file loading/persistence.
- Code/Data/Schema/Store/Transport/Consumer Boundary: TD01–04 exact reviewed55 source. config/runtime_safety.ron→loader→ContentGeneration/catalog/plan→CoreApplication/BattleCore; authoring API/CLI and server actor transport. Opaque non-battle persistence capability must bind exact session/revision and preserve monotonic profile.
- Goal-Local Dependencies / Execution Prerequisites: U01 Complete/High exact target currentness; GEX-10 before archive and default inventory; detailed constructor/restore/opaque transaction design before mutation. No metrics dependency in this Phase.
- Validation Plan / Completion Matrix: V01–08,V12; defaults/nested fault/restore/query/history and full changed host/CLI. Source/config/seed/hash and raw receipts; baseline server failure retained as before evidence.
- Structural Completion / Legacy and Fallback Disposition: Rust operational numeric defaults come from embedded RON once; separate ceilings remain explicit. All actual consumers use the chosen profile. Server no private GameCore/BehaviorResult import and no invalid-file fallback.
- Canonical Documentation Obligation: config schema/default scope/ceiling, error path/field, raw/effective identity and CLI/server opt-in documented; README and roadmap status updated only after independent closure.
- Applicable Migration/Compatibility/Rollout/Rollback: capture dirty source/data before edits; bounded parse before publication; failed reload keeps old session; restore does not rewrite save; host persistence failure uses session/revision-bound non-battle rollback with monotonic profile/peak preservation.
- Goal-Local Known Unknown / Blocker / Stop Condition: concrete API names and measured costs remain GEX details; missing required evidence or incompatible policy returns to owning P2G stage. Stop only user stop/scope done/real unresolvable blocker.
- Goal-Local Invariant Set / Source Refs: FPF-CRSO-001 primary/supporting members and PP-CRSO-001 intermediate-state audit; original limits/scope/first fault/RNG/event order, immutable generation and diagnostic-only observation.
- Consumer Verification Boundary: workspace-confirmed: Core/application/authoring CLI/server source. S2 Prometheus/Grafana is required external service evidence to be obtained during GEX; no deployed remote product claim.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: common reuse contract above and WP-specific V mapping. Only exact unchanged evidence reused; every affected seam executes on final source.
- Assigned Durable Integration Validation Seam Coverage: PP-CRSO-001 integration rows ending S1, owned by WP-CRSO-S1VERIFY; no unowned future validation.
- Applicable Risk / Reviewer Requirement and Evidence: separate implementation-nonparticipant GPT-6 Astra/High completion audit of frozen source/records/references and actual restore; findings corrected and re-audited before dependent work.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01,V03–08 actual host/API/CLI | default/explicit valid profile completes with expected identity and gameplay |
| invalid_input | required | V02 bytes/depth/schema/field/range tests | typed path/field rejection before publication, no fallback |
| failure_rollback | required | V04,V05,V08 injected failures | same allowance and state; opaque restore exact authority; previous session/save preserved |
| retry_idempotency | required | V04/V08 repeated fault/restore/capability reuse | no refreshed allowance or duplicate restore; retry after corrected input succeeds |
| ordering_concurrency | required | V03/V04 two immutable profiles, nested calls | no global cross-profile effect; original RNG/event order |
| boundary_empty_max | required | V02/V06/V07 zero/max/overflow/parser/history | bounded rejection and exact original legal typed ranges |
| disconnect_restart_persistence | required | V04/V08 reconnect/checkpoint/persistence failures | cursor and monotonic data preserved under current host settings |
| live_data | required | V01/V05 production embedded and external content pack | same real loader/compiler profile, no fixture-only evidence |
| producer_consumer | required | V03–08 full pipeline | profile identity reaches every required consumer before first spawn |
| legacy_fallback_absence | required | V01/V02/V08 source and failure path | one operational default source; no private Core re-export or silent invalid-profile fallback |

## Phase Goal Member `GOAL-CRSO-S2`

- Goal ID: GOAL-CRSO-S2
- Goal Kind: phase
- Phase ID: PHASE-CRSO-S2
- Assigned Final Execution Node Set: WP-CRSO-OBSERVE, WP-CRSO-EXPORT, WP-CRSO-S2VERIFY
- Covered Policy Element Revision Ref Set: PER-CRSO-08-REVIEWED-002, PER-CRSO-09-REVIEWED-002, PER-CRSO-10-REVIEWED-002, PER-CRSO-11-REVIEWED-002, PER-CRSO-12-REVIEWED-002
- Covered Decision Revision Ref Set: DR-CRSO-08-002, DR-CRSO-09-002, DR-CRSO-10-002, DR-CRSO-11-002, DR-CRSO-12-002
- Derived Observable Policy Acceptance Coverage: FPF-CRSO-001의 위 exact member acceptance; 아래 completion은 그 수락 기준의 실행 해설이며 변경하지 않는다.
- Objective / Contract-Visible Result: bounded runtime observation shows attempted work, committed state, brief peaks and rejection causes in actual Prometheus/Grafana while gameplay is unchanged.
- User-Visible Problem: existing ledgers guard runtime work but do not provide external trend/distribution and lifecycle-safe aggregate observation.
- User-Visible Completion Condition Set: correct attempted/committed accounting, peak and rejection-once; finite queue/labels/histograms/ring and idle cleanup; termination removes gauges; actual Grafana displays data; off/on and collector failure parity.
- In-Scope / Out-of-Scope: TD05–07 OBSERVE/EXPORT/S2VERIFY. Exclude new gameplay safety limits, 20minute tests, per-battle RSS claim, skill auto-cancel, continuation, game-internal graph UI.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001 RQ05–07: existing cost/extent seams plus optional fixed operation summary and host worker, standard metrics/exporter and Grafana provisioning.
- Goal-Local Source of Truth / Owning Layer: existing integer ledgers remain safety authority; outer operation observer owns attempted stats outside rollback, application commit owns gauges; host owns finite collector/exporter tasks and HTTP.
- Code/Data/Schema/Store/Transport/Consumer Boundary: Core feature and constant-cost summary→bounded nonblocking queue→finite host batch metrics calls→synchronous upkeep→Prometheus/Grafana. Active-session aggregates removed on end/drop; slot0/ring300 detail.
- Goal-Local Dependencies / Execution Prerequisites: S1 independent Complete/High and exact source currentness; detail queue/batch/finite-label proof and operation nesting/commit hooks before mutation; exact dependency source/MSRV/license/lock pinned.
- Validation Plan / Completion Matrix: V09–12 plus invalidated S1 seams. Real off/on paired battles, short peak, injected rollback, collector absence/scrape stop/worker stall, heap/storage bounds, actual Grafana datasource/dashboard and final core/host regressions.
- Structural Completion / Legacy and Fallback Disposition: No owner full scan on scrape or exporter feedback into safety; optional feature/HTTP; finite handles registered before hot path; direct unbounded histogram hot path absent; profile hash independent of observability.ron.
- Canonical Documentation Obligation: metric names/units/accounted versus RSS, attempted versus committed, wall versus simulation, finite storage/cleanup rules, local launch and Grafana dashboard/selection boundaries documented.
- Applicable Migration/Compatibility/Rollout/Rollback: failed operation stats retained once while committed gauges stay unchanged; observation drop never drops game events; service failure does not block gameplay; termination cleans aggregates and selected identity.
- Goal-Local Known Unknown / Blocker / Stop Condition: measured overhead/storage/service evidence pending actual GEX. Queue/batch details may vary only within bounded contract. Source/consumer policy mismatch returns to P2G owner; no fabricated external graph pass.
- Goal-Local Invariant Set / Source Refs: FPF-CRSO-001 primary/supporting members and PP-CRSO-001 intermediate-state audit; original limits/scope/first fault/RNG/event order, immutable generation and diagnostic-only observation.
- Consumer Verification Boundary: workspace-confirmed: Core/application/authoring CLI/server source. S2 Prometheus/Grafana is required external service evidence to be obtained during GEX; no deployed remote product claim.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: common reuse contract above and WP-specific V mapping. Only exact unchanged evidence reused; every affected seam executes on final source.
- Assigned Durable Integration Validation Seam Coverage: PP-CRSO-001 integration rows ending S2, owned by WP-CRSO-S2VERIFY; no unowned future validation.
- Applicable Risk / Reviewer Requirement and Evidence: separate implementation-nonparticipant GPT-6 Astra/High completion audit of frozen source/records/references and actual restore; findings corrected and re-audited before dependent work.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V09/V11 actual operations and graph | work, extent, faults, peaks and time units agree with actual ledgers |
| invalid_input | required | V10 observer config and malformed service config | observability errors isolated from valid gameplay; safe configuration diagnostics |
| failure_rollback | required | V09/V10 fault injection and disabled comparison | attempt once persists, committed state unchanged, no gameplay blocking |
| retry_idempotency | required | V09 duplicated summary/termination/reselection | no duplicate fault or stale/double-subtracted gauge |
| ordering_concurrency | required | V09/V10 nested operation/two sessions/worker stall | finite aggregate ownership, exact same gameplay events/RNG |
| boundary_empty_max | required | V10 no sessions/queue full/ring rollover/finite buckets | storage reaches fixed bounds then drops observation only |
| disconnect_restart_persistence | required | V10/V11 exporter restart and battle end/drop | collector absence maintenance works; no stale battle gauge or gameplay save mutation |
| live_data | required | V10/V12 normal/concentrated/function/refusal content | actual runtime summary from source-bound battles; synthetic fixtures identified |
| producer_consumer | required | V11 real exporter→Prometheus→Grafana | datasource queries show expected samples and rejection reasons |
| legacy_fallback_absence | required | V10 feature disabled/exporter failed | no new hidden required service or metrics-driven safety decision |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRSO-01 | PER-CRSO-01-REVIEWED-002 | DR-CRSO-01-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-02 | PER-CRSO-02-REVIEWED-002 | DR-CRSO-02-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-03 | PER-CRSO-03-REVIEWED-002 | DR-CRSO-03-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-04 | PER-CRSO-04-REVIEWED-002 | DR-CRSO-04-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-05 | PER-CRSO-05-REVIEWED-002 | DR-CRSO-05-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-06 | PER-CRSO-06-REVIEWED-002 | DR-CRSO-06-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-07 | PER-CRSO-07-REVIEWED-002 | DR-CRSO-07-002 | PHASE-CRSO-S1 | GOAL-CRSO-S1 |
| PE-CRSO-08 | PER-CRSO-08-REVIEWED-002 | DR-CRSO-08-002 | PHASE-CRSO-S2 | GOAL-CRSO-S2 |
| PE-CRSO-09 | PER-CRSO-09-REVIEWED-002 | DR-CRSO-09-002 | PHASE-CRSO-S2 | GOAL-CRSO-S2 |
| PE-CRSO-10 | PER-CRSO-10-REVIEWED-002 | DR-CRSO-10-002 | PHASE-CRSO-S2 | GOAL-CRSO-S2 |
| PE-CRSO-11 | PER-CRSO-11-REVIEWED-002 | DR-CRSO-11-002 | PHASE-CRSO-S2 | GOAL-CRSO-S2 |
| PE-CRSO-12 | PER-CRSO-12-REVIEWED-002 | DR-CRSO-12-002 | PHASE-CRSO-S2 | GOAL-CRSO-S2 |
