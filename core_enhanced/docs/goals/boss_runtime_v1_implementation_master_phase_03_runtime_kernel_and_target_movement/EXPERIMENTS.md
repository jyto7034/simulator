# Boss Runtime v1 Phase 03 — Experiments

## 2026-08-09 — activation baseline

- P02 independently completed `Complete / High` with `ready_for_p03`; P03 is the next required implementation Phase.
- The sorted precreation repository status contains 251 entries with SHA-256
  `ec91f011a9f4292d7854bc1c6f860f5f4fa176adb883593bc075695c725dcdd6`.
- Potential overlapping owner hashes before P03 production mutation:
  - `battle/core/mod.rs`: `ccc2d6a79004e9808ae94926d1dd883573d90bb2c0d5e4f25a01c4c5bc345bad`
  - `battle/core/sim.rs`: `9eaf60134e1ec9341ada0c6d6777043efea6389f75acf6570ec3e3168fde3d6b`
  - `battle/core/basic_attack.rs`: `29ae1dea51badd60d0420efdff69356024dd5e710a590239c5400c40053586e9`
  - `battle/core/build.rs`: `1125b3846c5b9e48579befe0b34521f6da36070cc83bc7553edf4e92cbed2122`
  - `movement/planner.rs`: `3d746b0939ee76bad7a8293a38ffce8fe2898c503beff66c86ec28914dfc9d9d`
  - `movement/engine.rs`: `8e3dd85166c692c7ab290265cb009c7bc8640ca5402d6ed14e596ffe2d70cd59`
  - `movement/path.rs`: `c75566b9973c279715e6135f61ae5ec8d006a0b501f46d8b069ac47c807d9d23`
  - `battle/enums.rs`: `a5cae50ca9936d2e9b83b13df58089bef67122cee07f9bcea78278282d48788c`
  - `battle/scenario.rs`: `6a65d9f6026950fbebc34e8070196c289c355b1d12c14950faf88ed4a11e6d8d`
  - `data/boss_runtime_data.rs`: `e02303b46e7e5cf70bfcdb6a092eb3e38746ebea9ba8e14eca7c43f3787afd86`
  - `core_runtime_contract.md`: `14ca6144d93b04f95a2c880e594e790b2f74c827d17f7259b2c71392dd982524`
- Most potential owners were already modified in the user baseline. P03 will prefer new Boss-domain modules and will
  treat the hashes plus recorded symbols—not status shape alone—as overlap evidence.
- No P03 production file has changed and no runner has executed at this checkpoint.

## 2026-08-09 — entry-boundary source finding

- `BattleCore::process_event_inner(SpawnGroup)` spawns one authored group and queues each unit's initial `AttackStart`
  at the same battle time. The t0 bucket then waits until no `SpawnGroup` remains before battle-start hooks and newly
  queued same-time events run.
- Therefore the safe activation seam is after the last t0 spawn group and before the next same-time bucket. P03 can
  resolve every bound `ScenarioUnitRef` to runtime identity there and gate queued actor actions before they execute.
- Subsequent wave spawns require the same runtime reconciliation after their `SpawnGroup` event; unresolved actor
  slots remain inactive rather than falling back to classification-based Boss behavior.
- `BattleCore` is Clone because P01 stages world advance. P03 can stage a whole same-time Boss microstep chain on a
  cloned Core and publish only after every transition/admission invariant passes, while existing leaf modules remain
  the mutation authority inside the staged copy.

## 2026-08-09 — movement/target seam finding

- Ordinary planner goals are built in canonical runtime-unit-ID order after block-state refresh. An explicit Boss
  directive can be consulted at that point, before the existing owner/route fallback, without touching non-opt-in
  goals.
- The continuous movement engine already returns typed body/stop/target outputs and remains the spatial executor, but
  it does not own behavior chaining. P03 must translate authoritative position/stop/path/timeout facts into one
  State-owned Task result.
- `BattlefieldLayout` exposes exact valid/walkable tiles and sorted static obstacles. A deterministic static-grid BFS
  can own reachability/path cost and canonical tie paths without changing Rapier/collision ownership.
- `RuntimeUnit.current_target` is shared basic/block leaf state and is unsuitable as the Boss lock. The lock must live
  under the Boss Pattern/resident-State activation and only be passed to leaf actions as an explicit captured target.

## 2026-08-09 — pre-mutation non-opt-in characterization

Three fully-qualified exact tests ran before P03 production mutation:

- ordinary route forward-progress planner: `1 passed`, `0 failed`, `0 ignored`, `892 filtered out`;
- Boss-class encounter with no binding resolved-setup identity: `1/0/0/892`;
- Event-started combat victory/world transition: `1/0/0/892`.

These are characterization anchors, not P03 implementation evidence. All three must remain green after the runtime is
integrated, and the full Core/workspace gates remain required.

## 2026-08-09 — first runtime compile checkpoint

- The first `cargo check -p game_core --lib` after adding the internal runtime shell failed before tests: one sibling
  import crossed the private `movement::path` module, tuple `.min()` accidentally required `Position: Ord`, and the
  task poller attempted to update the actor route cursor while its active State was mutably borrowed. There were also
  six unused imports.
- These are scoped compile/integration defects, not runtime test failures. The fix is to re-export the existing route
  helper at the Core boundary, compare only primitive path-key fields, defer the cursor write until the State borrow
  ends and remove the imports; the failed check is not counted as pass evidence.
- After those corrections, `cargo check -p game_core --lib` passed. A follow-up review also made the wake identity
  operational rather than decorative: only a current actor-slot/task activation wake requests a Boss evaluation
  boundary, and the existing all-variant total-order fixture now includes the twentieth internal event variant.

## 2026-08-09 — first runtime test compile checkpoint

- The first Boss runtime module test command did not run assertions: two atomic-overflow assertions used
  `assert_eq!(Result<_, GameError>, ...)`, but the intentionally non-`PartialEq` `GameError` type makes that test code
  fail to compile. Production code compiled. The assertions are changed to `matches!(..., Err(InvalidAction))`; this
  failed invocation is not counted as a test result.

## 2026-08-09 — first runtime assertion result

- The compiled six-test runtime module produced `5 passed`, `1 failed`, `0 ignored`, `893 filtered out`.
- The target-invalidation assertion expected the Pattern to remain in its `rejoin` State. The runtime correctly took
  the authored `TargetInvalidated → rejoin`, found the actor already on its forward route, returned `Reached`, emitted
  the named outcome and routed to the resident `route_combat` control State within the same bounded timestamp chain.
  The assertion is corrected to require that final authored control node, a cleared lock and no implicit replacement
  target. No production behavior is changed for this fixture correction.
- The corrected module rerun passed `6 passed`, `0 failed`, `0 ignored`, `893 filtered out`. Current coverage includes
  opt-in construction, exact no-binding authority, lexicographic insertion independence, immediate invalidation,
  unreachable-path rejection and whole-Core microstep/activation-overflow rollback.

## 2026-08-09 — internal wake total-order checkpoint

- Exact command: `cargo test -p game_core --lib game::battle::enums::tests::battle_event_cmp_equal_iff_eq_for_every_live_variant -- --exact`
- Result: `1 passed`, `0 failed`, `0 ignored`, `898 filtered out`. The new Boss wake participates in the complete
  live-variant equality/order matrix; no catch-all equality was reintroduced.
- Review immediately after this checkpoint found two lifecycle semantics that need explicit closure before expanding
  the matrix: `EnemyUnit` must exclude the actor and same-owner units independently of optional query filters, and
  FixedSequence cursor state is Phase-local under BP-33. Phase exit now clears selector cursors while the BP-27
  battle-scoped cooldown ledger remains intact. Focused tests are added before either change is treated as evidence.

## 2026-08-09 — lifecycle/deadline/target expansion checkpoint

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Result: `10 passed`, `0 failed`, `0 ignored`, `893 filtered out` after formatting. No zero-match filter.
- The four added fixtures prove:
  - `EnemyUnit` excludes self/same-owner candidates even when the optional filter list is empty.
  - At the exact deadline, a reached movement domain fact wins over `TimedOut`; `Wait` completes exactly at its
    absolute battle-time deadline, and the completed activation's wake becomes stale/non-triggering.
  - Same-Phase reentry allocates fresh Phase/behavior/control identities, clears the activation-owned target and
    Phase-local FixedSequence cursor, and preserves the already-consumed battle-scoped cooldown ready time.
  - `Immediate`, `AfterCurrentState` and `AfterCurrentPattern` all reach the target Phase, while only
    `AfterCurrentPattern` admits the intermediate rejoin Task (observable as its additional scheduled wake). This
    distinguishes authored boundary timing rather than merely checking the common final Phase index.
- Successful initial Pattern admission is also now asserted to consume the `tracking` cooldown exactly once at
  battle time zero (`ready_at_ms == 2_000`). The existing full-snapshot overflow cases continue to prove failed
  scheduler admission leaves the cooldown ledger untouched as part of the unchanged transaction snapshot.

## 2026-08-09 — movement/cancellation expansion first result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Result: `12 passed`, `1 failed`, `0 ignored`, `893 filtered out`.
- Both new movement cases passed: the opt-in actor traversed through the existing continuous movement leaf and
  completed the authored Move/Rejoin path, while an external movement lock kept its position fixed through `4_999`
  and the exact `5_000` Boss deadline selected the authored timeout recovery.
- The cancellation fixture failed before exercising cancellation because its scheduled `AttackResolve` callback was
  absent. This is a fixture admission/setup issue at `unreleased attack callback`, not evidence of a production
  cancellation failure. The exact ordinary-attack admission guard is inspected and the fixture will be corrected
  before rerun; the failed invocation is retained here and is not counted as pass evidence.

## 2026-08-09 — movement/cancellation corrected result

- Inspection showed the fixture actor was a fixed DefenseRoute melee enemy. Existing target authority intentionally
  permits that actor to attack only the route-end target, while the fixture had killed that target before selecting
  the remaining player. The cancellation path had therefore never admitted an attack. The fixture now uses the
  existing ranged persisted-target branch; no production attack-selection rule was changed.
- Exact corrected test command:
  `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime::tests::cancellation_drops_unreleased_attack_but_preserves_released_projectile -- --exact`
- Corrected exact result: `1 passed`, `0 failed`, `0 ignored`, `905 filtered out`.
- Full module rerun command: `cargo test -p game_core --lib game::battle::core::boss_runtime`
- Full module result: `13 passed`, `0 failed`, `0 ignored`, `893 filtered out`.
- The cancellation fixture now proves both sides of the ownership boundary with existing leaf code: State exit bumps
  the auto-attack epoch so the unreleased `AttackResolve` produces neither projectile nor damage, while a projectile
  released before cancellation remains in Core projectile authority and damages the target after its actor-owned
  work is cancelled.

## 2026-08-09 — arbitration/multi-actor/cursor expansion first result

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Result: `15 passed`, `1 failed`, `0 ignored`, `893 filtered out`.
- Authored-priority permutation and monotonic route-cursor tests passed. The two-actor `AfterCurrentPattern` fixture
  remained in Phase 0 after the second actor's target approach, although actor 0 correctly waited and Phase did not
  transition early.
- Inspection of the fixture geometry found actor 1 approached its selected target at `(2, 2)` but its authored route
  rejoin point is `(2, 1)`. A Boss boundary alone does not execute a movement leaf tick, so actor 1 correctly had not
  completed the whole Pattern. The fixture will place the actor on `(2, 1)`, which is both target-adjacent and on the
  authored route, to isolate the all-actors boundary behavior. No production behavior is changed; this failed run is
  retained and not counted as pass evidence.

## 2026-08-09 — arbitration/multi-actor/cursor corrected result

- Corrected exact command:
  `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime::tests::after_pattern_phase_transition_waits_for_every_bound_actor -- --exact`
- Exact result: `1 passed`, `0 failed`, `0 ignored`, `908 filtered out`.
- Full module rerun: `cargo test -p game_core --lib game::battle::core::boss_runtime`
- Full result: `16 passed`, `0 failed`, `0 ignored`, `893 filtered out`.
- The expanded matrix proves that the higher authored transition priority wins under both Task-result storage
  permutations, `AfterCurrentPattern` holds Phase 0 after actor 0 finishes and commits Phase 1 only after actor 1 also
  finishes, and a route cursor advanced to index 3 does not regress when the actor is externally relocated back near
  index 1.

## 2026-08-09 — no-eligible/named-outcome/checked-time expansion

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Result: `17 passed`, `0 failed`, `0 ignored`, `893 filtered out`.
- Phase reentry at battle time `100` now continues one staged step and proves the still-cooling sole FixedSequence
  candidate selects the authored `no_eligible` control route without changing its battle-scoped ready time.
- A fixture with every hostile candidate inactive proves `AcquireTarget` returns `NoEligibleTarget`, follows the
  Pattern's named `target_missing` outcome and reaches the authored route control without engine target fallback.
- The atomic failure matrix now also places a no-cooldown PatternCall immediately before a parameterized Move task and
  evaluates it at `u64::MAX`. The checked absolute deadline overflows after intermediate staged lifecycle work; the
  full Core/runtime/queue/log/cooldown snapshot remains exactly unchanged, covering deadline failure separately from
  activation-ID and microstep overflow.

## 2026-08-09 — canonical runtime contract checkpoint

- `docs/core_runtime_contract.md` no longer describes the runtime scheduler as unimplemented. Its Boss section now
  records only the P02/P03 behavior present in current source: opt-in construction/reset/t0 activation, fresh scoped
  IDs and Phase timing, FixedSequence/cooldown/no-eligible semantics, whole-chain atomic staging, five supported Tasks,
  checked battle-time wakes, target/query/lock invalidation, movement/cursor authority and unreleased-versus-released
  leaf cancellation.
- The same section explicitly leaves official live reference Boss publication and public Boss lifecycle Wire outside
  the implemented boundary. No game rule, public DTO, server mapping, save reader or Unity consumer was changed by
  this canonical update.

## 2026-08-09 — identity/tie/relocation expansion

- Exact command: `cargo fmt --all && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Result: `19 passed`, `0 failed`, `0 ignored`, `893 filtered out`.
- Equal PathCost and HpRatio candidates are now forced onto the same position/health and the lower spawn-order unit is
  selected before runtime-ID tie; reversing HashMap removal/reinsertion order preserves the same identity.
- Relocating the locked unit while it remains active/hostile preserves the exact lock and does not seed the
  TargetInvalidated latch, keeping target identity separate from movement feasibility.
- A future-timestamp Boss spawn proves the encounter/behavior exists without a unit at t0, then binds the exact
  `ScenarioUnitRef` and activates its Pattern after the spawn at time 10 but before the actor's newly queued same-time
  auto action. No Boss `AttackStart` semantic event is emitted while that Pattern owns movement.

## 2026-08-09 — existing movement integration gate

- Exact command: `cargo check -p game_core --lib && cargo test -p game_core --lib game::battle::core::movement`
- Compile result: pass.
- Movement result: `63 passed`, `0 failed`, `0 ignored`, `849 filtered out`.
- The gate includes direct and Rapier movement parity, static/void obstacles, canonical input ordering, airborne
  policy, DefenseRoute block/attack authority, empty/unit-specific routes and the pre-P03 no-retarget-after-progress
  characterization. The Boss directive/cursor seam did not regress any existing movement owner.

## 2026-08-09 — full Core broad gate

- Exact formatting/test command: `cargo fmt --all -- --check && cargo test -p game_core --lib`.
- Formatting result: pass. The verbose test invocation completed successfully but its captured output was truncated,
  so it is not used alone for the exact count.
- Exact count confirmation command: `cargo test -p game_core --lib -- --quiet`.
- Core result: `912 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- This broad gate includes P01 lifecycle/ordering/atomicity, P02 Boss compiler/setup/live-bundle tests, all existing
  world combat transaction and Pause cases, and the 19 P03 runtime tests in one Core binary.

## 2026-08-09 — optional strict Clippy diagnostic

- Diagnostic command ended with `cargo clippy -p game_core --lib -- -D warnings` failure: `81` warnings were promoted
  to errors. The output spans pre-existing/current-baseline unusual literal grouping, large enum variants, derivable
  impls, argument-count, type-complexity and style lints across ability/map/world/reward/Core modules.
- Two P03 private helpers (`admit_state_tasks`, `admit_pattern`) are included only by the argument-count lint; there is
  no compile/test/runtime failure and the required Phase gate does not declare repository-wide warning-free Clippy.
  Converting the broad baseline in this Boss Phase would implement unrelated refactor candidates, so no Clippy-driven
  production rewrite is authorized here.
- `git diff --check` in the same diagnostic invocation emitted no error. The leading production-source scan used a
  duplicated cwd-relative path and did not execute as intended; it is rerun separately and is not treated as evidence.

## 2026-08-09 — server broad gate

- Before this gate the only logically unreachable Pattern target arm and the post-check pending-Phase extraction were
  changed from panic-style assumptions to `GameError::InvalidAction`; corrupt runtime stays on the system-error/atomic
  rollback channel. No valid behavior changed.
- Exact command: `cargo fmt --all -- --check && cargo test -p game_server -- --quiet`.
- Formatting result: pass.
- Server results: library `56 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`; binary and doc tests
  each ran `0` tests successfully. The existing server mapping requires no Boss field because P03 adds no public DTO.

## 2026-08-09 — workspace all-targets broad gate

- Exact command: `cargo test --workspace --all-targets -- --quiet`.
- Exit: `0`.
- Results: Core library `912 passed`; Core main and five fixture-generator binaries `0`; integration targets
  `3 + 4 + 24 + 13 + 4` passed; server library `56 passed`; server main and three examples `0`. Every listed target
  reports `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- This closes the required P03 broad command set together with the separately recorded formatting, Boss module,
  movement, full Core and server gates.

## 2026-08-09 — pre-exit audit finding P03-F01

- Reviewing BP-23/BP-25 and the actual `ContinueState` runtime path found a material foundation gap before completion
  audit: `consume_condition_source` consumes TaskResult but not `EventOccurred(Pulse)`, so a winning Pulse
  `ContinueState` can observe the same pulse again inside the same-time chain. In addition, P02 compilation currently
  permits unguarded `ContinueState` with persistent `Always`, StatePredicate or `UntilStateExit` conditions because
  ContinueState is not represented as a graph adjacency edge.
- The runtime microstep limit would roll the Core back rather than partially mutate, but that is a system-error safety
  net, not valid authored behavior or the required publication rejection. P03 cannot claim event-memory/loop closure
  while this remains.
- Scoped correction: consume only Pulse on a winning ContinueState; preserve `UntilStateExit` latches; require a
  bounded loop guard when a persistent condition targets ContinueState. Add compiler/runtime focused tests and rerun
  all P02/P03 and broad gates. This is a P03-discovered strengthening of the compiled input invariant, not a new
  selector/task/schema or unrelated refactor campaign.

## 2026-08-09 — P03-F01 first correction test result

- Command:
  `cargo fmt --all && cargo test -p game_core --lib game::data::boss_runtime_data::tests::persistent_continue_state_requires_a_bounded_loop_guard -- --exact && cargo test -p game_core --lib game::battle::core::boss_runtime`
- The exact compiler test ran `0 passed`, `1 failed`, `0 ignored`, `914 filtered out`; command chaining correctly
  prevented the runtime suite from running after that failure.
- The assertion expected the new diagnostic but another existing validation fired first. This is a test-construction
  ordering issue until the actual compiler error is captured; it is not counted as correction pass evidence. The
  assertion is amended to print the concrete cause, then the fixture will be isolated from unrelated exhaustive-result
  validation.
- Diagnostic rerun identified the concrete earlier failure: replacing the authored `Reached` transition made the
  Move Task result mapping non-exhaustive. The fixture now appends a distinct higher-priority persistent ContinueState
  edge while retaining every finite result arm, so it reaches the intended validation without weakening another
  compiler invariant.

## 2026-08-09 — P03-F01 corrected focused result

- Exact command:
  `cargo fmt --all && cargo test -p game_core --lib game::data::boss_runtime_data::tests::persistent_continue_state_requires_a_bounded_loop_guard -- --exact && cargo test -p game_core --lib game::battle::core::boss_runtime`
- Compiler exact result: `1 passed`, `0 failed`, `0 ignored`, `914 filtered out`.
- Boss runtime module result: `21 passed`, `0 failed`, `0 ignored`, `894 filtered out`.
- Runtime coverage now proves a ContinueState winner consumes a Pulse but retains its State-local UntilStateExit latch,
  and a guarded persistent ContinueState takes its normal edge exactly twice before choosing the authored exhaustion
  target. Compilation rejects the same persistent edge without a bounded loop guard.
- P03-F01 status: corrected in focused scope; broad regression gates must be rerun on the changed compiler/runtime
  before the finding is closed.

## 2026-08-09 — P03-F01 broad closure

- Exact current-tree command: `cargo fmt --all -- --check && git diff --check && cargo test -p game_core --lib -- --quiet`.
- Formatting/diff result: pass. Core result: `915 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- Exact current-tree command: `cargo test --workspace --all-targets -- --quiet`.
- Workspace exit: `0`. Current results: Core `915`; integration targets `3 + 4 + 24 + 13 + 4`; server `56`;
  Core/server binaries, five Core generators and three server examples `0`; every target reports zero failure/ignore.
- P03-F01 status: closed. The compiler/runtime/canonical are aligned and both focused and latest broad evidence are
  post-correction results; the earlier 912-count broad record remains historical pre-finding evidence only.

## 2026-08-09 — pre-completion dirty/status comparison

- Current exact command uses `git status --short --untracked-files=all | LC_ALL=C sort`; before creation of a
  completion-audit artifact it reports `255` entries with SHA-256
  `501afe2b5e4ca16d52337a63c6d267ea66024ce87d5dc05cb7166d1e4c9d5983`.
- Removing exactly the three required P03 child documents and new `battle/core/boss_runtime.rs` produces `251`
  entries with SHA-256 `ec91f011a9f4292d7854bc1c6f860f5f4fa176adb883593bc075695c725dcdd6`, byte-identical to the
  recorded P03 precreation status hash/count. No unrelated status path was added, removed or normalized by P03.
- Existing dirty owners intentionally keep their pre-P03 status shape. P03 symbol/hunk changes are limited to the
  Core runtime field/reset/boundary, action gates, movement directive/cursor, internal event wake/order, the
  P03-F01 compiler invariant and current canonical. `movement/path.rs` ended with its exact pre-P03 hash after an
  initially considered visibility widening was removed.
- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`; P03 ran no checkout/reset/clean/delete and created no
  generated repository residue.

## 2026-08-09 — pre-exit audit finding P03-F02

- Comparing Boss State cancellation with the existing Hard-CC unreleased-action owner found that Boss pending-skill
  cleanup releases resonance/basic/movement locks solely by deadline equality. The existing authoritative path also
  checks the cancelled skill's `focus_permissions`, because a focus that permits attack or movement does not own those
  locks even if another source happens to share the deadline.
- This can shorten an unrelated same-deadline basic/movement lock when RouteCombat exits during a pending cast. It is
  an ownership defect in P03 cancellation, not a reason to alter the existing skill subsystem.
- Scoped correction: take the pending cast and its deadline under the unit borrow, resolve the current skill's captured
  focus permissions from existing GameData, release resonance plus only the locks that focus actually owned, set basic
  windup-owned locks to the cancellation battle time rather than zero, and preserve released deliveries. Focused and
  broad gates are rerun before completion audit.

## 2026-08-09 — P03-F02 corrected focused result

- Exact command: `cargo fmt --all && cargo fmt --all -- --check && cargo test -p game_core --lib game::battle::core::boss_runtime`.
- Formatting result: pass. Boss runtime module result: `22 passed`, `0 failed`, `0 ignored`, `894 filtered out`.
- The added actual-path regression fixture installs a valid skill whose focus permits both movement and basic attack,
  seeds all three locks at the same deadline, then cancels the pending Boss cast. Cancellation clears pending cast and
  next-action state, releases the focus-owned resonance lock at battle time 10, and preserves the movement/basic locks
  at 110 because that focus did not own them.
- The existing unreleased-basic/released-projectile test now also proves windup-owned movement/basic locks return to
  the exact cancellation battle time rather than zero; the stale resolve remains cancelled and an already released
  projectile still completes.
- P03-F02 status: corrected in focused scope; broad Core/workspace regression evidence must be refreshed before the
  finding is closed.

## 2026-08-09 — P03-F02 broad closure

- Exact current-tree command: `cargo fmt --all -- --check && git diff --check && cargo test -p game_core --lib -- --quiet`.
- Formatting/diff result: pass. Core result: `916 passed`, `0 failed`, `0 ignored`, `0 measured`, `0 filtered out`.
- Exact current-tree command: `cargo test --workspace --all-targets -- --quiet`.
- Workspace exit: `0`. Current results: Core `916`; integration targets `3 + 4 + 24 + 13 + 4`; server `56`;
  Core/server binaries, five Core generators and three server examples `0`; every target reports zero failure/ignore.
- P03-F02 status: closed. The focused ownership regression, canonical contract, full Core and all workspace targets
  now agree; the earlier 915-count broad record is historical pre-F02 evidence only.

## 2026-08-09 — completion-audit dirty/status closure

- After creating the required P03 completion audit, exact sorted status is `256` entries with SHA-256
  `e0e7f5cbf7d39f41a2ad8e2a97c4070f3a726d268a87242df096c373c5ce30fc`.
- Removing exactly the four P03 Goal artifacts (`PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`,
  `COMPLETION_AUDIT.md`) and new `battle/core/boss_runtime.rs` yields `251` entries with SHA-256
  `ec91f011a9f4292d7854bc1c6f860f5f4fa176adb883593bc075695c725dcdd6`, exactly the precreation baseline.
- Completion audit verdict: `Complete / High`; no open P03 finding or stop condition remains. Handoff is
  `ready_for_p04`.
