# Boss Runtime v1 P00 — Refactor Execution Registry

Status: complete

Last Updated: 2026-08-09

## Registry rules

- 이 registry는 refactor audit의 production 권한이 아니라 Boss Master 내부의 실행 시점/비실행 disposition이다.
- `implement_in_boss_phase`만 해당 Boss child가 구현한다.
- `conditional_public_cutover`는 실제 public contract가 생길 때만 구현한다.
- `defer_after_slice`와 `independent_goal_only`는 Boss v1 완료를 막지 않는다. 별도 승인·Goal 없이 이번
  campaign에 흡수하지 않는다.
- 모든 current path는 사용자 소유 dirty baseline이다. 후보를 구현할 때도 기존 hunk를 보존한다.

## Exact candidate set and timing

| Timing | Candidate IDs | Final P00 disposition |
| --- | --- | --- |
| P01 prerequisite | CRA-P06-C01, CRA-P06-C02, CRA-P06-C03 | `implement_in_boss_phase` |
| P02 foundation | CRA-P01-C01, CRA-P05-C04 | `implement_in_boss_phase` |
| P05 live gate | CRA-P01-C02, CRA-P05-C02 | `implement_in_boss_phase` |
| P06 public gate | CRA-P10-C03 | `conditional_public_cutover` |
| after P04 seam proof | CRA-P07-C03, CRA-P07-C04, CRA-P09-C05 | `defer_after_slice`; separate Goal only if then approved/needed |
| independent | CRA-P07-C01, CRA-P07-C02, CRA-P08-C01, CRA-P08-C05, CRA-P09-C03, CRA-P09-C06, CRA-P10-C02, CRA-P10-C04 | `independent_goal_only`; not Boss gates |

## P01 prerequisite candidates

### CRA-P06-C01 — terminal/max-time authority

- Work/current evidence: correctness correction; direct next-event and elapsed/until stepping still disagree at the
  max-time cap, and execution state does not own an immutable finish outcome.
- Authority/invariant: `BattleExecutionState` becomes the only terminal outcome/cap authority; preserve configured
  cap, same-time drain, Draw fallback, one `BattleEnd`, Pause/speed semantics and no post-cap events.
- Overlap/timing: direct prerequisite for Boss terminal arbitration; P01 milestone 1. Dirty owners are
  `battle/core/sim.rs` and related `battle/core/mod.rs` tests.
- Entry/exit: start with event-after-cap and exact-cap failure tests; exit when direct/until/by/world-server paths
  return the same state-owned result and second finalize/replacement rejects.
- Validation/gate: focused Core terminal matrix plus world tick-to-`CombatResult`; no policy/external gate.
- Disposition: `implement_in_boss_phase:P01`.

### CRA-P06-C02 — total deterministic BattleEvent order

- Work/current evidence: correctness correction; current `Ord` still returns `Equal` for unequal variants/payloads.
- Authority/invariant: one stable semantic key covers every variant and distinguishing payload while preserving
  current time/priority bands, spawn-before-action and resolve-before-terminal behavior.
- Overlap/timing: Boss same-time scope arbitration prerequisite; P01 milestone 2 after the terminal contract is fixed.
  Dirty owner is `battle/enums.rs` and scheduler integration tests.
- Entry/exit: review same-priority cross-variant semantics, add `cmp == Equal iff Eq` table first; exit when all heap
  insertion permutations and simultaneous manual/manual plus auto/manual completions drain identically.
- Validation/gate: enum unit/property tables, heap permutations, existing attack-before-end regression and repeated
  log/result equality. A genuinely ambiguous semantic cross-variant priority requires a narrow policy decision;
  current table itself is preserved and no such blocker is presently identified.
- Disposition: `implement_in_boss_phase:P01`.

### CRA-P06-C03 — atomic world battle advance

- Work/current evidence: correctness correction; active Core mutates before fallible VN/reconcile/projection/cursor/
  transition work, and future cursor validation occurs after revision commit.
- Authority/invariant: one world advance pipeline prevalidates or stages every fallible part and publishes one
  revision/result transition; preserve raw-vs-pause tick distinction, setup-on-first-update, trigger thresholds,
  employee reconciliation and debug-trace non-authority.
- Overlap/timing: Boss transition commit prerequisite; P01 milestone 3 after C01/C02. Shares possible low-level
  preflight primitives with CRA-P07-C02 but must not implement the five player-command transactions.
- Entry/exit: first add exact pre/post state tests for future cursor and injected late failures; exit when elapsed and
  next-bucket entrypoints call one orchestration and every error preserves Core/time/queue/log/resource/employee/VN/
  revision/cursor/result state.
- Validation/gate: focused world failure matrix, success parity, server tick/cursor tests and smallest relevant server
  suite. `BattleCore` is not full-Clone, so implementation must explicitly design stage/preflight/rollback; no policy
  decision is required for canonical no-mutation.
- Disposition: `implement_in_boss_phase:P01`.

## P02 foundation candidates

### CRA-P01-C01 — embedded source inventory authority

- Work/current evidence: behavior-preserving refactor; actual source authority is 1 manifest + 22 files, while manual
  count/canonical copies can drift. Adding Boss data would amplify this gap.
- Authority/invariant: `GameplayRonSourceSet::embedded`/parser remains the single compile-time inventory authority;
  preserve the current 23 normalized paths, manifest version, typed errors and no filesystem hot load.
- Overlap/timing: coimplement while adding the separate Boss catalog in P02 so the new source cannot become another
  untracked count. Does not authorize dynamic reload/LKG.
- Entry/exit: exact-set characterization before catalog addition; exit when inventory/count/reconciliation derives
  mechanically from one source and an extra/missing live file fails a checked test.
- Validation/gate: live-data unit tests and `ron_loading`; no policy gate unless someone proposes deploy-time loading.
- Disposition: `implement_in_boss_phase:P02`.

### CRA-P05-C04 — one typed combat-setup plan

- Work/current evidence: behavior-preserving refactor; world, reward, preview, encounter and scenario assembly still
  join manually and `ActiveBattleSession` is filled field-by-field.
- Authority/invariant: one private resolved setup/builder owns the aggregate after domain authorities resolve their
  values; preserve group/event/reward order, seed/UUID, errors, live deployment policy and world lifecycle ownership.
- Overlap/timing: P02 uses this seam to resolve optional Boss encounter/actor binding and immutable compiled frames
  before Core construction. P05 later broadens feasibility over every live difficulty; P02 does not claim that gate.
- Entry/exit: characterize no-binding normal/Boss-class/survival/authored-plan/Event setups; exit when world commits one
  typed validated setup and runtime does no raw Boss catalog lookup.
- Validation/gate: golden scenario/setup/first checkpoint/reward, deterministic enemy identity and late-failure
  no-world-mutation tests. Keep public `CombatExecutor` test/consumer seam; no gameplay policy gate.
- Disposition: `implement_in_boss_phase:P02`.

## P05 official live publication candidates

### CRA-P01-C02 — every manifested difficulty validation

- Work/current evidence: correctness correction; startup validation still covers baseline generation while runtime
  consumes manifested difficulty-specific tuning.
- Authority/invariant: one finite authored difficulty/stage representative matrix calls the runtime-equivalent
  generator for every encounter and approved seeds; preserve tuning, determinism, source-aware errors and immutable
  all-or-nothing startup.
- Overlap/timing: P05 gate before the first official Boss opt-in; its matrix is shared with CRA-P05-C02.
- Entry/exit: negative fixture where baseline passes and a later stage fails; exit when every manifested live
  difficulty is validated with diagnostic profile/stage context.
- Validation/gate: data/preview/RON integration plus measured finite-matrix startup cost. No policy gate unless the
  existing canonical validation breadth is intentionally weakened.
- Disposition: `implement_in_boss_phase:P05`.

### CRA-P05-C02 — full setup feasibility at publication

- Work/current evidence: correctness correction; generated preview publication can accept content that later fails
  spawn-cell capacity, tactical object/reference or final scenario validation during battle entry.
- Authority/invariant: a pure publication-time setup feasibility validator uses the P01-C02 difficulty/seed matrix and
  the same empty-player scenario predicates; preserve runtime defensive checks, UUID/state non-consumption and
  current content/balance.
- Overlap/timing: P05 official Boss definition/binding gate after P02's pure typed builder exists.
- Entry/exit: add capacity, invalid work/tactical cell, broken objective/ref and difficulty-added wave failures; exit
  when the entire candidate bundle rejects before publication and one full live start still passes.
- Validation/gate: structured source-aware negative cases, full RON suite and DefenseRoute + reference Boss start
  smokes. No new content policy is required.
- Disposition: `implement_in_boss_phase:P05`.

## Conditional first-public-wire candidate

### CRA-P10-C03 — Core DTO/server transport parity

- Work/current evidence: behavior-preserving refactor; server still reauthors normal update/setup fields while resync
  nests the Core DTO, amplifying any first public Boss lifecycle change.
- Authority/invariant: Core owns semantic DTO content; server owns only message selection, resync envelope and send
  ordering. Preserve exact top-level JSON, tags, resync nesting and response/side-message order.
- Overlap/timing: only P06, if the approved live Boss needs a new public semantic lifecycle/facing/telegraph. If P06
  is explicitly `not_applicable_to_v1`, this candidate remains outside Boss completion.
- Entry/exit: exact old/new serde parity tests first; exit when normal/setup/resync serialize one Core authority and
  Unity fixtures/consumer use the same generation.
- Validation/gate: exact JSON goldens, actor ordering and synchronized Unity contract/live-flow evidence. Any shape or
  nesting change is a separate consumer migration decision; exact parity itself needs none.
- Disposition: `conditional_public_cutover:P06`.

## Deferred until the vertical slice proves the seam

### CRA-P07-C03 — single private Rapier production backend

- Work/current evidence: cleanup; production defaults to Rapier while Direct and backend selection remain test-heavy
  alternate authority. Current source still contains both and test-only selectors.
- Authority/invariant: eventual private Rapier path must preserve deterministic ordering, obstacle/air/overlap/block/
  clamp behavior and authored-route progress.
- Overlap/timing: changing the movement test authority before P04 would enlarge the Boss movement risk. Reassess only
  after the tracking slice runs against the actual production backend.
- Entry/exit/validation: transfer every unique Direct assertion first, inventory public Rust consumers, then remove
  alternate backend with repeated default-run log equality.
- Gate/disposition: external API removal approval may be required; `defer_after_slice`, separate Goal, not a Boss gate.

### CRA-P07-C04 — remove ghost movement tick state

- Work/current evidence: cleanup; unread input/body fields and never-produced outputs still exist.
- Authority/invariant: eventual minimal internal movement view preserves body position/radius/speed, current goals,
  terrain/locks/lifecycle, segments, facing and all public event/DTO behavior.
- Overlap/timing: follows CRA-P07-C03; P03 may adapt outputs for Boss Task results but must not perform broad ghost
  field deletion under runtime implementation.
- Entry/exit/validation: compile/search each field/variant, external consumer inventory and golden moving/stopped/
  blocked logs/checkpoints after one backend remains.
- Gate/disposition: public Rust/serde removal may need approval; `defer_after_slice`, separate Goal, not a Boss gate.

### CRA-P09-C05 — continuous projectile motion kernel

- Work/current evidence: behavior-preserving refactor; basic and skill projectile paths still duplicate sweep,
  interpolation, rounding and next-event logic.
- Authority/invariant: eventual pure motion query must preserve separate projectile IDs/payload/filters/terminal rules,
  exact timing, collision and public effects/events.
- Overlap/timing: `TrackAndRejoin` needs movement, not projectile unification. Reassess only if P04/P05 actual
  vocabulary exposes projectile-bound duplication.
- Entry/exit/validation: after damage boundary and projectile terminal prerequisites, run geometry/time goldens and
  repeated public event parity; keep lifecycle adapters separate.
- Gate/disposition: coordinate public record visibility and CRA-P08-C05; `defer_after_slice`, separate Goal, not a Boss
  gate.

## Independent candidates — explicitly outside Boss completion

### CRA-P07-C01 — tactical tile-effect final-state derivation

- Current evidence/authority: correctness bug remains possible because stance, relocation and field-work each mutate
  Focus/block/Rest while snapshot derives intended state. One final-state reconciler is the target authority.
- Invariant/overlap: preserve FieldWorking/Ready/Defense semantics and unrelated modifiers. Boss movement may call the
  existing leaf, but the bug is not caused by Boss opt-in and does not gate the P04 reference slice.
- Entry/exit/validation/gate: Any-profile worker relocation/swap failure tests, exact modifier/Rest epoch assertions;
  no policy gate. `independent_goal_only`.

### CRA-P07-C02 — atomic tactical player-command mutation

- Current evidence/authority: five world command handlers can still validate cursor/reconcile after mutation. Target
  is a command-scoped preflight/commit boundary, distinct from battle tick advance.
- Invariant/overlap: preserve each command reason/cost/cooldown/salt/mapping and exactly one revision. P01 C03 may
  share a neutral early validator, but must not merge player-command and Boss tick transaction lifetimes.
- Entry/exit/validation/gate: per-command future-cursor and injected reconcile failure equality; no policy gate.
  `independent_goal_only`.

### CRA-P08-C01 — one typed trigger occurrence dispatcher

- Current evidence/authority: battle-start, hit and death trigger assembly remains split; battle-start still collects
  activations separately from effects. Target is one typed occurrence dispatcher.
- Invariant/overlap: preserve item/unit order, release semantics, effect-before-activation and death contexts. Boss
  tasks use leaf effects but do not require this unrelated live equipment correction.
- Entry/exit/validation/gate: parasite-seed battle-start and all occurrence ordering tests; no policy gate.
  `independent_goal_only`.

### CRA-P08-C05 — committed damage resolution consolidation

- Current evidence/authority: instant, basic projectile and general damage paths still contain overlapping formula/
  trigger/application logic and an ignored `UnitDied` command. Target is one committed-hit execution summary.
- Invariant/overlap: preserve snapshot/live-defense timing, roll identity, resonance, causes and death order. Boss
  runtime retains damage as a leaf and does not need its internal refactor.
- Entry/exit/validation/gate: instant/projectile drift/death/trigger golden parity after prerequisite characterization;
  public carrier removal needs external inventory. `independent_goal_only`.

### CRA-P09-C03 — delivery-aware skill usefulness

- Current evidence/authority: TileArea usefulness can still evaluate only an anchor while runtime owns the actual
  affected set. Target is an executable delivery-target plan shared by acquisition/autocast/step gating.
- Invariant/overlap: preserve current live choices and filters. Boss target queries are a separate semantic schema and
  must not inherit skill “usefulness” as path/selector authority.
- Entry/exit/validation/gate: footprint/edge/filter/multi-step cases; multi-step any-vs-next usefulness is an unresolved
  policy decision. `independent_goal_only`.

### CRA-P09-C06 — typed tile reach query/projection

- Current evidence/authority: runtime membership, manual reach, preview and area materialization still duplicate
  policy/pattern/facing/clipping adapters. Target is one typed contains/materialize query.
- Invariant/overlap: preserve pattern geometry, stable valid-cell order and intentional absence of obstacle filtering.
  This is tile reach, not Boss static-grid navigation/path cost; P03 does not depend on it.
- Entry/exit/validation/gate: full policy/pattern/facing/edge parity and membership-set equivalence; no policy gate.
  `independent_goal_only`.

### CRA-P10-C02 — typed command-result projection

- Current evidence/authority: `BehaviorResult`, domain result enums and JSON/result-type mappings remain duplicated.
  Target is one typed wire-result projection.
- Invariant/overlap: preserve every result string/payload, nested completion and battle side-message split. Boss public
  lifecycle transport does not require refactoring all command results.
- Entry/exit/validation/gate: all-variant exact JSON goldens and external API inventory; current `RunComplete` shape is
  frozen, any wrapping is migration. `independent_goal_only`.

### CRA-P10-C04 — typed host snapshot overlay

- Current evidence/authority: server still mutates Core snapshot JSON to add persistence/recovery status. Target is a
  typed host overlay while Core retains gameplay snapshot authority.
- Invariant/overlap: preserve flattened root keys, capability order/reasons and host recovery behavior. It neither
  gates internal Boss runtime nor P06 battle DTO parity.
- Entry/exit/validation/gate: persistence-state snapshot table, duplicate/order and Core-root preservation; no policy
  gate if current flattened JSON remains exact. `independent_goal_only`.

## Registry conclusion

- Required implementation inside the approved Boss campaign: 7 unconditional candidates in P01/P02/P05, plus
  CRA-P10-C03 only when P06 activates.
- Deferred/independent candidates are fully dispositioned but not authorized production work. The Boss Master does
  not mean “implement the complete 51-candidate refactor catalog” or even all 19 relevant candidates.
- P01 owns one child with three reviewable milestones because C01 defines the finish contract consumed by C03 and
  C02 fixes the scheduler substrate in the same dirty Core surface. Splitting them into parallel children would add
  temporary competing terminal/transaction authorities.
