# Architecture Findings

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- ID 형식: `ARCH-REVIEW-###`

## 기록 원칙

- 구조 pressure 가능성을 처음 식별하면 `Hypothesis`로 즉시 등록한다.
- 단순 file size, import count, clone, public field, lint 또는 AI 생성 사실만으로 finding을 만들지 않는다.
- representative change scenario/operation, dependency/data flow, compiler/test protection과 실제 영향이 필요하다.
- 보존 자산과 migration seam은 finding과 별도 positive catalog에 기록한다.
- 반증된 후보와 superseded history를 삭제하지 않는다.

## Finding template

```markdown
## ARCH-REVIEW-###. <제목>

- 최초 기록 시각:
- 마지막 갱신 시각:
- 기준선:
- criterion ID:
- 상태: Hypothesis | Active | Rejected | Superseded
- 심각도:
- 확신도:
- 관련 domain/subsystem:
- 평가한 change scenario 또는 운영 요구:
- current owner/type/module:
- production entry와 dependency/data flow:
- raw observation:
- 실제 변경 증폭 또는 제한:
- compiler/test가 보호하는 범위:
- 사용자·개발·운영 영향:
- 보존해야 할 자산:
- 반대 증거와 불확실성:
- 근본 원인:
- 연관 finding:
- 가능한 개선 대안:
- migration seam 또는 blocker:
- rewrite decision에 미치는 영향:
- 필요한 검증:
- 연결된 experiment/evidence:
```

## Initially Registered Hypotheses

이 절은 최초 등록 위치를 보존한다. 각 finding의 현재 `상태`가 권위이며 ARCH-REVIEW-001은 후속 시나리오로
`Active`에 승격되었다.

## ARCH-REVIEW-001. Application root is the main change-amplification boundary

- 최초 기록 시각: `2026-08-22T06:53:30+09:00`
- 마지막 갱신 시각: `2026-08-22T07:31:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A01, A02, A03, A05
- 상태: Active
- 심각도: `S1 High`
- 확신도: `Confirmed`
- 관련 domain/subsystem: application/world root
- 평가한 change scenario 또는 운영 요구: REQ-001~007 and S01~S08, especially Relic/growth and a new roguelike node
- current owner/type/module: `GameCore`, `GameCoreState`, `world.rs`
- production entry와 dependency/data flow: `execute*` → capability gate → central command routing → world handler → shared state roots
- raw observation: `GameCoreState` contains numerous run, profile, session, inventory, roster, battle and checkpoint roots;
  `GameCore` centrally routes every `PlayerBehavior` family. Monolith changes cross encounter/map/shop/VN/combat/snapshot/
  capability modules. A genuinely new node category crosses category, payload, active content, phase/context, routing,
  capability, snapshot/checkpoint and tests. `RunState` also `Deref`s to `EncounterProgressionState`, hiding explicit owner access.
- 실제 변경 증폭 또는 제한: S04/S08 demonstrate semantic fan-out beyond legitimate enum exhaustiveness. New run mechanics
  and node content repeatedly require edits to root switches and parallel state representations. Cohesive domain engines still
  isolate algorithms: movement, Boss graph, fragment compilation, facility graph and economy planning need not be replaced.
- compiler/test가 보호하는 범위: private `GameCore.state`, typed behaviors/mode and focused run-start tests protect external
  access and REQ-001 semantics.
- 사용자·개발·운영 영향: an AI-assisted solo developer must load a broad `world` context and remember session/transaction/
  persistence conventions for each feature; omission risk grows even when the domain algorithm is small.
- 보존해야 할 자산: typed root entry, capability preflight, `GameMode`/`RunProgression`, broad characterization tests.
- 반대 증거와 불확실성: a single application façade remains appropriate. Private state, exhaustive routes,
  `with_state_transaction`, staged node completion and broad characterization tests are strong safety assets. Static reference
  counts alone were not used as proof.
- 근본 원인: application orchestration, aggregate storage, session phase, projection and domain handlers share the same
  `world` type graph without narrow mutation ports.
- 연관 finding: ARCH-REVIEW-003, ARCH-REVIEW-004, ARCH-REVIEW-005
- 가능한 개선 대안: retain `GameCore` as façade; extract application use-case modules, explicit `RunContext`/mutation ports,
  one active-node sum type and a common command transaction/publish boundary.
- migration seam 또는 blocker: existing `execute_*_behavior` family routing is a potential seam.
- rewrite decision에 미치는 영향: supports a bounded application-layer redesign behind the existing façade. It argues against
  preserving current `world` internals unchanged, but preserved domain engines argue against a whole-core big bang.
- 필요한 검증: a follow-up implementation should build the command classification/conformance suite and demonstrate one
  vertical-slice extraction behind the unchanged façade.
- 연결된 experiment/evidence: E-006~E-019, S04/S08, `DOMAIN_MODEL_MAP.md`, `AGGREGATE_STATE_MAP.md`,
  `TESTABILITY_MAP.md`

## ARCH-REVIEW-002. Full active-battle cloning may make correctness cost scale with retained runtime state

- 최초 기록 시각: `2026-08-22T07:01:30+09:00`
- 마지막 갱신 시각: `2026-08-22T07:31:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A05, A07, A09
- 상태: Active
- 심각도: `S2 Medium`
- 확신도: `Confirmed`
- 관련 domain/subsystem: active battle application transaction
- 평가한 change scenario 또는 운영 요구: every server tick and tactical command; long battle/many entities
- current owner/type/module: `ActiveBattleSession`, `with_staged_battle_advance_state`,
  `with_staged_tactical_world_mutation`
- production entry와 dependency/data flow: server tick or PlayerBehavior → clone active session → mutate/project → replace
- raw observation: `ActiveBattleSession: Clone` contains a cloned `BattleCore`; `BattleCore` contains queues, maps, logs,
  projectiles/areas/movement, boss/wave/summon states and a Rapier world. Advance additionally clones VN/UUID/root state.
- 실제 변경 증폭 또는 제한: E-020 measured near-linear history/entity scaling. At 100K retained events the lower-bound
  `BattleCore` clone was 1.879ms p95 and a last-10 query was 96.916us p95; the log JSON was 12.78MB. Ten/100/1,000 retained
  units produced 2.930/20.341/207.524us p95 staged one-ms work.
- compiler/test가 보호하는 범위: staging gives strong rollback; tests verify overflow/cursor/failure immutability.
- 사용자·개발·운영 영향: correctness cost and reconnect work grow with unrelated retained history/state, reducing tick
  headroom and making an unbounded long battle an operational resource risk.
- 보존해야 할 자산: transaction semantics, typed command seam, projection-before-publish, immutability tests.
- 반대 증거와 불확실성: current ordinary live fixture is light (one-ms tick 10.848us p95); no allocation, concurrent-session
  or event-heavy boss workload was measured. The measured clone is a lower bound for `ActiveBattleSession`.
- 근본 원인: rollback implemented by complete state copy rather than mutation journal/owned transaction stage.
- 연관 finding: ARCH-REVIEW-001
- 가능한 개선 대안: retain clone with measured budget; split append-only/shared data; kernel-owned transactional command;
  journal/swap only touched roots.
- migration seam 또는 blocker: two existing staging helpers centralize the mechanism.
- rewrite decision에 미치는 영향: may justify battle application-boundary rewrite, not whole-core rewrite.
- 필요한 검증: allocation/peak memory and event-heavy boss/projectile/effect workload; product retention/tick/concurrency cap.
- 연결된 experiment/evidence: E-008/E-009/E-020, S09, `PERFORMANCE_BUDGET.md`, GAP-001

## ARCH-REVIEW-003. Battle kernel dependency boundary is porous

- 최초 기록 시각: `2026-08-22T07:04:30+09:00`
- 마지막 갱신 시각: `2026-08-22T07:31:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A03, A04, A05, A06
- 상태: Active
- 심각도: `S2 Medium`
- 확신도: `Confirmed`
- 관련 domain/subsystem: battle kernel, data/application boundary
- 평가한 change scenario 또는 운영 요구: boss, movement and skill/battle extension
- current owner/type/module: `battle/core/**`, `GameDataBase`, `behavior.rs`
- production entry와 dependency/data flow: CombatExecutor passes full database into BattleCore; kernel imports authoring types,
  application error/reason and projection DTOs.
- raw observation: `BattleCore` stores `Arc<GameDataBase>` and exposes mutable units/battlefield/data/log/sequence fields plus
  public `enqueue_event`; core modules import `GameError`, `ActionDisabledReasonCode`, `LiveBattleSkillReadinessDto` and authored
  profiles. A float-bearing public `BattleEvent` manually asserts `Eq` and uses a Debug-string final order key while finite input
  remains a contract premise rather than an input type.
- 실제 변경 증폭 또는 제한: S01~S03 show that authored boss/skill types, application readiness/errors/projections and the
  full catalog enter kernel-adjacent changes. Isolated tests can build empty databases, but realistic kernel behavior often
  constructs a broad catalog/application fixture. The ordinary typed algorithm seams remain usable.
- compiler/test가 보호하는 범위: crate-private setup plan and typed live commands are positive seams; many internals are
  private. Current live inputs and ordering tests are strong, so no production nondeterminism was inferred from public reachability.
- 사용자·개발·운영 영향: potential AI context growth, wider compile/change surface and harder isolated kernel tests.
- 보존해야 할 자산: deterministic kernel algorithms, typed commands/outcomes, tests, compiled scenario.
- 반대 증거와 불확실성: single-crate direct types avoid mapping overhead and may be pragmatic for one team.
- 근본 원인: authoring, domain, application and projection models share one crate/type graph.
- 연관 finding: ARCH-REVIEW-001
- 가능한 개선 대안: private authority façade and finite validated event input first; battle-scoped compiled catalog/context;
  kernel-local error/query models; optional crate split only after boundary extraction.
- migration seam 또는 blocker: `ResolvedCombatSetupPlan`, `BattleScenario`, `BattleLiveCommand` are existing seams.
- rewrite decision에 미치는 영향: supports incremental boundary extraction or subsystem rewrite, not big-bang evidence.
- 필요한 검증: compile a battle-scoped catalog/context and kernel-local result model in a follow-up vertical slice; crate split
  is optional until module visibility proves the boundary.
- 연결된 experiment/evidence: E-009~E-012/E-018/E-019, S01~S03, `DEPENDENCY_DIRECTION_MAP.md`,
  `TESTABILITY_MAP.md`, `DATA_CONTRACT_EVOLUTION_MAP.md`

## Active Findings

## ARCH-REVIEW-004. Mutating battle use cases have no uniform transaction boundary

- 최초 기록 시각: `2026-08-22T07:00:07+09:00`
- 마지막 갱신 시각: `2026-08-22T07:00:07+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A02, A05, A07
- 상태: Active
- 심각도: `S2 Medium`
- 확신도: `Confirmed`
- 관련 domain/subsystem: active-battle application transaction, skill/focus/playback commands
- 평가한 change scenario 또는 운영 요구: S03 player-selected activation and rejected-command immutability
- current owner/type/module: `GameCore::execute_with_transport_context`, `ActiveBattleSession`, `world/combat.rs`
- production entry와 dependency/data flow: behavior authority/preflight → direct handler → live `BattleCore`/session mutation →
  revision commit → cursor DTO → outer VN/capability attachment.
- raw observation: deploy/withdraw/stance/relocate/swap use `with_staged_tactical_world_mutation`; begin/cancel skill focus,
  activate skill and playback handlers mutate the live session. `handle_activate_skill` can pass prevalidation, append cast state
  and event, reconcile, increment revision, then return `InvalidBattleResyncSeq` from `battle_update_dto_after` for a future
  cursor. There is no enclosing rollback.
- 실제 변경 증폭 또는 제한: every new player-visible battle mutation must remember which bespoke staging convention applies
  and must manually order every fallible postcondition before mutation. The type system does not enforce this policy.
- compiler/test가 보호하는 범위: revision overflow is prechecked; skill tests protect wrong payload/no-target/focus rejection
  and success. They do not cover post-mutation cursor/projection failure for direct handlers. Staged tactical tests do cover the
  equivalent rollback class.
- 사용자·개발·운영 영향: a rejected command can be reported as failure after authoritative battle state changed, causing
  reconnect surprise or retry duplication. The host may avoid malformed cursors in normal flow, but architecture must not rely
  on that convention for failure atomicity.
- 보존해야 할 자산: battle authority revision, typed behaviors/commands, staged helper semantics, focused rejection tests and
  cast epoch/idempotency logic.
- 반대 증거와 불확실성: successful direct activation has a short path, and most predictable errors are validated before
  mutation. No runtime failure injection measured frequency or additional late-failure classes.
- 근본 원인: transaction ownership is handler-local convention rather than a single application mutation port.
- 연관 finding: ARCH-REVIEW-001, ARCH-REVIEW-002
- 가능한 개선 대안: route every mutating battle behavior through a generic staged-session transaction; prebuild projection
  before publish; later replace full clone with a kernel/application transaction implementation behind the same port.
- migration seam 또는 blocker: `with_staged_tactical_world_mutation` already proves the desired publish-after-projection shape;
  generalize it to focus/playback/skill and advance paths before changing rollback internals.
- rewrite decision에 미치는 영향: supports a bounded active-battle application-layer rewrite/refactor; does not support a
  whole-core big bang.
- 필요한 검증: reproduce future-cursor failure on direct activation/focus/playback, inventory all mutating battle handlers, and
  characterize host cursor guarantees.
- 연결된 experiment/evidence: E-010, S03, `AGGREGATE_STATE_MAP.md`

## ARCH-REVIEW-005. Application transaction atomicity is a handler convention, not a command invariant

- 최초 기록 시각: `2026-08-22T07:14:33+09:00`
- 마지막 갱신 시각: `2026-08-22T07:14:33+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A02, A05, A07, A08, A11
- 상태: Active
- 심각도: `S1 High`
- 확신도: `Confirmed`
- 관련 domain/subsystem: application command boundary across event, reward, shop, maintenance, node and battle
- 평가한 change scenario 또는 운영 요구: run start, S04~S08 plus Rulebook atomic node/result/failure requirements
- current owner/type/module: `GameCore::execute_with_transport_context`, domain handlers, `with_state_transaction`,
  `EconomyGrantState`, battle staging helpers
- production entry와 dependency/data flow: capability preflight → selected domain handler → handler-chosen clone/plan/direct
  mutation → result/projection/VN attachment; only some handlers enter an enclosing root transaction.
- raw observation: node confirm/complete, HQ terminal choices, sweep, checkpoint load and combat-result completion explicitly use
  `with_state_transaction`. Shop/reward/event/maintenance commands use several narrower/manual stages. Event StartCombat commits
  its receipt/session before calling fallible `commit_planned_live_battle_start`. Consumable use publishes inventory/roster before
  calling fallible recovery/failure resolution. Maintenance dismantle paths sequence mutation and a later grant commit. The root
  dispatcher can also perform fallible battle projection/VN attachment after the domain handler returned success. Current
  StartNewGame assigns run-start roots and pending mode before fallible RunStart trigger activation without a root transaction.
- 실제 변경 증폭 또는 제한: every new command must audit all fallible work and choose among whole-root rollback, economy
  staging, tactical session staging, clone-and-publish or validation ordering. The type/API does not make partial publication
  unrepresentable or force projection/durable plan creation before commit.
- compiler/test가 보호하는 범위: typed payloads, preflight, deterministic receipts, narrow staged structs and many
  no-mutation tests cover known failures. They do not provide a universal conformance suite for every mutating `PlayerBehavior`
  or future late-failure injection.
- 사용자·개발·운영 영향: a newly added feature can return an error after one authoritative root changed, making retry,
  reconnect or host durable publication ambiguous. Even where current prevalidation makes a late error unlikely, maintaining
  that equivalence is an undocumented proof obligation.
- 보존해야 할 자산: `with_state_transaction` semantics, staged economy/battle helpers, typed commands/results, deterministic
  receipts and existing rejection tests.
- 반대 증거와 불확실성: many current late calls are effectively infallible after matching preview/validation under valid live
  data, and the host may wrap exported active-run operations. No injected production failure rate was measured.
- 근본 원인: transaction scope is selected inside individual handlers after routing instead of being a property of the
  application command/use-case port.
- 연관 finding: ARCH-REVIEW-001, ARCH-REVIEW-004
- 가능한 개선 대안: classify commands as Query/RunMutation/BattleMutation/ProfileMutation; execute each mutation through one
  transaction object that builds domain effects, projection and durable publication intent before atomic publish. Keep
  permanent profile merge semantics explicit rather than cloning it blindly.
- migration seam 또는 blocker: `with_state_transaction`, `EconomyGrantState` and `with_staged_tactical_world_mutation` already
  demonstrate the target patterns. Generalize them behind the existing `execute*` façade before changing domain algorithms.
- rewrite decision에 미치는 영향: supports rewriting the application command/transaction layer in place. It does not justify
  replacing map, battle, movement, boss, reward or fragment domain implementations.
- 필요한 검증: inventory all mutating command routes, inject late projection/commit failures, define durable-host handoff and
  build one transaction-conformance suite.
- 연결된 experiment/evidence: E-010, E-015~E-017, S03/S06~S08, `AGGREGATE_STATE_MAP.md`

## ARCH-REVIEW-006. Validated live data and fixture-built data share one runtime-accepted type

- 최초 기록 시각: `2026-08-22T07:38:00+09:00`
- 마지막 갱신 시각: `2026-08-22T07:38:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion ID: A03, A06, A08, A11
- 상태: Active
- 심각도: `S2 Medium`
- 확신도: `Confirmed`
- 관련 domain/subsystem: static data publication, runtime construction and test fixtures
- 평가한 change scenario 또는 운영 요구: every application/battle fixture and new boss/skill/relic data schema
- current owner/type/module: `GameDataBuilder`, `GameDataBase::new`, `GameCore::new`, official live-RON validated candidate
- production entry와 dependency/data flow: either strict `load_live_embedded` or public builder/parts construction produces the
  same bare `GameDataBase`; `GameCore::new(Arc<GameDataBase>, seed)` cannot distinguish provenance/type-state.
- raw observation: official loading privately parses/validates/compiles/generated-tests before publication. Public
  `GameDataBuilder::build/build_arc` calls public `GameDataBase::new`; index/reference/policy paths include panic/`expect`, and
  fixture-minimal databases can omit application-required content while remaining accepted by the Core constructor.
- 실제 변경 증폭 또는 제한: a test or alternate caller can enter runtime with a meaning not producible by the official
  loader, moving static-data failure into later application commands. AI-authored tests can accidentally validate a parallel
  fixture contract instead of the production compiler contract.
- compiler/test가 보호하는 범위: selected contract says builders are fixtures; direct production consumers observed in this
  Goal use the official loader, and live-RON tests strongly protect publication. Rust types/visibility do not enforce that rule.
- 사용자·개발·운영 영향: current production incidence is low, but invalid fixture/runtime states enlarge failure and review
  space and obscure which invariants a subsystem test may assume.
- 보존해야 할 자산: strict loader, source-aware errors, private validated candidate, generated setup gate and useful minimal
  fixture builders.
- 반대 증거와 불확실성: no current official server startup was audited in this Goal; E-019 shows the official Core API, not
  misuse frequency. The builder is intentionally valuable for isolated tests.
- 근본 원인: validated publication state and fixture construction are erased into the same bare runtime type.
- 연관 finding: ARCH-REVIEW-003, ARCH-REVIEW-005
- 가능한 개선 대안: expose `Arc<ValidatedGameData>`/consumer-scoped compiled contexts to production constructors; keep an
  explicitly named unchecked fixture context under test support, and make fallible builder validation return typed errors.
- migration seam 또는 blocker: `ValidatedGameplayRon` and private `from_validated_live_catalogs` already encode the official
  type-state; introduce a public opaque validated wrapper without changing authored data algorithms.
- rewrite decision에 미치는 영향: supports a small data-construction/boundary rewrite, not replacement of validators/catalogs.
- 필요한 검증: compile-time production constructor rejection for fixture data and parity tests for scoped compiled contexts.
- 연결된 experiment/evidence: E-019/E-023, ROUTE-018, `DATA_CONTRACT_EVOLUTION_MAP.md`

## Rejected Candidates

아직 없음.

## Superseded Findings

아직 없음.

## Root-Cause Consolidation Log

E-022 independent lock 뒤 E-023 cross-check까지 마친 후 다음처럼 통합했다.

- RC-A — application orchestration, phase/session state, transaction selection and projection share the broad world root:
  ARCH-REVIEW-001/004/005. Remedy is a root-enforced use-case/transaction boundary plus typed session aggregates.
- RC-B — validated authoring, domain kernel and external protocol lack compiler-enforced type/dependency boundaries:
  ARCH-REVIEW-003/006, with ARCH-REVIEW-001 protocol fan-out. Remedy is compiled scoped contexts, private kernel authority and
  versioned adapters before optional crate split.
- RC-C — correctness staging and reconnect presentation reuse complete retained battle state/history:
  ARCH-REVIEW-002, with ARCH-REVIEW-004 transaction semantics. Remedy is a preserved transaction port backed by touched-state
  staging and a bounded indexed delta/archive design.

No root cause demonstrates that the domain algorithms, live-data pipeline, checkpoint model and tests must be replaced together.
