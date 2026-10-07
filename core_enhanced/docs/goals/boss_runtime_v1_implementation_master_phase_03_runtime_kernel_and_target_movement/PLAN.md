# Boss Runtime v1 — Phase 03 Runtime Kernel And Target/Movement Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — only this child scope
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Master Phase: P03
Activated: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

P02의 immutable compiled definition과 typed setup을 직접 소비하는 Core-owned Boss Runtime kernel을 구현한다.
Encounter/actor/Phase/control-node/Pattern/State/Task lifecycle, deterministic arbitration, checked battle-time
deadline, target lock/query와 explicit movement intent/result adapter를 기존 BattleCore event/movement leaf 위에
통합한다. Boss opt-in이 없는 battle path는 정확히 기존 동작을 유지한다.

## Direct dependency and handoff

- P02: `../boss_runtime_v1_implementation_master_phase_02_catalog_and_typed_setup/COMPLETION_AUDIT.md`
  - Verdict: `Complete / High`
  - Handoff: `ready_for_p03`
- P01 correctness substrate remains authoritative for terminal order, total `BattleEvent` order and staged world
  advance.
- Refactor sub-track: no new CRA candidate is implemented in P03. Deferred and independent candidates remain outside
  this Goal.
- Runtime policy: BP-02~BP-27 and BP-31~BP-34 runtime portions; BP-01/P02 opt-in boundary and BP-30 existing-event-only
  privacy are preserved.

Current dirty files are the user-owned baseline. Findings, failing characterization, implementation decisions and
actual validation results are written after each bounded unit rather than reconstructed at Phase end.

## In scope

- Core-owned `BossEncounterRuntime` plus one `BossActorBehaviorRuntime` per resolved actor slot
- Monotonic typed activation identities for Phase, control node, Pattern, State and Task; one owner/one exit
- Fresh Phase reentry, child-before-parent cancellation, stale wake/result no-op and battle-scoped selector cooldown
- Same-snapshot condition/result candidate collection, fixed cross-scope order and unique authored priority arbitration
- Explicit transition plans and bounded same-time microstep chains committed atomically
- FixedSequence selector execution for the supported compiled v1 subset; unsupported selectors stay publication errors
- Closed Task execution for `RouteCombat`, `AcquireTarget`, `MoveToTarget`, `RejoinForwardRoute` and `Wait`
- Checked admission-time completion/timeout deadlines using authoritative battle time; exact-boundary domain-result
  precedence and Pause safety
- Activation-owned target lock, immediate lifecycle invalidation, explicit reacquisition and typed lexicographic query
- Deterministic static-grid reachability/path cost and stable unit tie-break
- Boss movement directive before ordinary planner fallback, forward-only rejoin cursor and typed movement result mapping
- Opt-in actor action/movement gating while existing movement, blocking, basic attack, skill, effect, projectile, damage
  and death modules retain leaf execution authority
- Core runtime canonical updates describing only implemented behavior

## Out of scope

- Official production Boss definition/binding, identity, balance, assets or live encounter opt-in
- P04 authored `TrackAndRejoin` bundle/server vertical-slice fixture
- PriorityEligible/WeightedEligible execution, future `RepathToProtectedObjective` or new mechanic/task vocabulary
- New public Boss event/DTO/checkpoint/save, game_server mapping or Unity consumer work
- Generic script VM, dynamic task registry, deep override, hot reload/LKG or cross-generation runtime migration
- CRA-P07-C03/C04/P09-C05 cleanup and every independent/deferred refactor candidate
- Boss-owned physics, collision, damage, attack, skill, projectile or death implementations

## Milestone P03-M1 — runtime entry and non-opt-in characterization

1. Fix the exact precreation status/hash and hashes of every potential runtime/leaf owner.
2. Characterize non-opt-in initial event, movement/attack and terminal traces before production mutation.
3. Add a Core construction fixture proving P02 setup maps actor slots to spawned unit identity without raw database
   lookup.
4. Determine and document the exact authoritative boundary where t0 spawn completion activates Boss Runtime before
   newly queued actor actions execute.

## Milestone P03-M2 — lifecycle, selector and atomic scheduler

1. Add encounter/actor runtime structures with fresh monotonic activation IDs and immutable definition/frame capture.
2. Implement Phase/control-node/Pattern/State/Task entry/exit, FixedSequence selection, outcome routing and battle-scoped
   cooldown ownership.
3. Collect Phase, control and Pattern transition candidates from one snapshot; apply terminal/death > Phase > control
   > Pattern > task admission scope order and authored priority within a node.
4. Stage cancellation, lock/intent cleanup, next entry, deadlines, cooldown and queued wake effects in one transition
   plan; validate before commit.
5. Bound same-time microsteps and prove overflow, reentry, checked-ID/deadline failure and stale epochs leave the
   pre-chain authoritative snapshot unchanged.

## Milestone P03-M3 — target and static-grid query authority

1. Evaluate the closed target query against one canonical snapshot with explicit filters and lexicographic keys.
2. Use deterministic static-grid reachability/path cost with canonical neighbor order; unit final tie is spawn sequence
   then runtime identity.
3. Store target locks under Pattern or resident-State activation ownership and preserve them until explicit decision
   point, invalidation or owner exit.
4. Produce one `TargetInvalidated` pulse/latch/result from death/withdraw/despawn without hidden grace or automatic
   retarget.
5. Prove insertion permutation, equal metric, relocation-with-same-identity, unreachable, no-target and immediate
   invalidation behavior.

## Milestone P03-M4 — movement/task leaf integration

1. Place an explicit Boss movement directive ahead of ordinary planner fallback without changing the non-opt-in goal
   path.
2. Execute Move/Rejoin through the existing continuous movement backend and adapt position/stop/path/target/deadline
   facts to exactly one closed Task result.
3. Maintain a monotonic authored-route cursor and choose only reachable forward rejoin points; never silently repath
   to the objective.
4. Gate opted-in actor auto attack/cast starts by the active Task capability and cancel only unreleased State-owned
   work on exit; released projectile/area/damage continues under existing lifecycle.
5. Prove Pause, hard-CC stall, exact completion-vs-timeout boundary, path unavailable, timeout, interruption and
   route-cursor permutations.

## Preserved invariants

- `scenario.boss_runtime == None` constructs no Boss runtime and uses the byte/semantic-equivalent existing path.
- Runtime consumes the P02 `Arc` definitions/frames retained by scenario setup and never reopens raw Boss catalog RON.
- Existing Core event queue, battle clock, Pause, terminal result, world transaction and revision remain authority.
- Existing movement/block/attack/skill/effect/projectile/damage/death code remains the only leaf executor.
- Target identity, path feasibility and action eligibility are separate failure axes; none silently becomes another.
- Losing transition candidates and failed admissions mutate no runtime, task, intent, leaf action, cooldown, RNG, event
  or revision state.
- State/Pattern/Phase cancellation never emits an authored Task result or Pattern outcome.
- Internal graph/target/task identity is not public Wire, save or Unity state.
- No missing/invalid state becomes fallback, default behavior or hidden retry.

## DTO, data, save and deployment

- DTO/Wire migration: none. Existing leaf presentation events/checkpoint remain sufficient in P03.
- Data migration: none. P03 consumes P02 compiled definitions; production Boss collections remain empty.
- Save/checkpoint migration: none. Runtime lives only in the in-memory `BattleCore` clone/stage boundary and is not a
  durable run payload or public checkpoint field.
- Deployment: code-only Core change behind explicit setup opt-in. Existing in-memory battles are not converted and no
  hot reload path is added.

## Focused validation

- Non-opt-in Core trace/goal/event/terminal parity before and after integration
- Runtime construction and immutable setup consumption
- Phase/control/Pattern/State/Task activation, reentry, cancellation and stale-epoch matrix
- Same-snapshot/cross-scope/priority/permutation arbitration and atomic failure snapshots
- FixedSequence/cooldown/no-eligible/outcome routing and checked overflow cases
- Target query/filter/key/tie/reachability/lock/invalidation/event-memory cases
- Static-grid path/rejoin forward cursor and movement result/deadline/Pause cases
- Unreleased cancellation versus released delivery continuation

Every exact command and actual pass/fail/ignored/filtered count is recorded. A zero-match filter is failure.

## Broad validation

After focused closure:

```text
cargo fmt --all -- --check
cargo test -p game_core --lib game::battle::core::boss_runtime
cargo test -p game_core --lib game::battle::core::movement
cargo test -p game_core --lib
cargo test -p game_server
cargo test --workspace --all-targets
```

If implementation places the runtime tests under a different exact module path, record the discovered target and run
it in addition to—not instead of—the required full Core/server/workspace gates.

## Completion conditions

1. P02 setup constructs the two-layer runtime with immutable definitions/frames and exact bound actor identity.
2. Phase/control/Pattern/State/Task activations have single ownership, fresh reentry and stale-epoch no-op semantics.
3. Same-boundary candidates arbitrate deterministically by lifecycle scope then authored node priority.
4. Transition/admission and same-time microstep overflow are atomic across runtime, intent, leaf, cooldown and event
   state.
5. FixedSequence, named outcomes, no-eligible paths and cooldown consumption obey the compiled contract.
6. Checked battle-time deadlines are Pause-safe and produce exactly one exhaustive result with correct boundary order.
7. Target acquisition is explicit, locked, lexicographic and deterministic; invalidation is immediate and never
   silently retargets.
8. Move/Rejoin use the existing movement leaf, return closed results and maintain a monotonic forward route cursor.
9. Non-opt-in battles are unchanged; no official live content, public Wire or unrelated refactor enters P03.
10. Runtime canonical, focused and broad tests pass with actual counts and dirty user work is preserved.
11. Independent completion audit returns `Complete / High` and handoff `ready_for_p04`.

## Stop conditions

- The supported P02 compiled subset cannot express a required P03 lifecycle without changing approved gameplay policy.
- Correct multi-actor phase boundary semantics require a user-visible policy choice not fixed by BP-01~BP-34.
- Existing leaf cancellation cannot preserve released delivery without a public contract migration.
- An internal runtime requires a public/checkpoint/save schema to remain authoritative in the current deployment.
- Current user edits change inside an overlapping runtime owner and cannot be reconciled at symbol/hunk scope.
- Correct deterministic path/query implementation requires dynamic navigation semantics outside the approved static
  v1 field model.

No current stop condition is active.
