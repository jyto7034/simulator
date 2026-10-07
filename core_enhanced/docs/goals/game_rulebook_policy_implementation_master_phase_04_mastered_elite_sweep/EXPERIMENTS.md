# Game Rulebook Policy Implementation Master Phase 04 — Experiments

## 2026-08-12 — Phase Activation And Document Initialization

### Hypothesis

P03 `Complete / High` closes P04's serialization dependency, and the continuing Master execution request permits
P04 to become the sole active production Phase after the required child documents are created.

### Performed work

- Verified P02/P03 complete/true status and P03 `COMPLETION_AUDIT.md` verdict `Complete / High`.
- Verified the Master registry had no active production Phase and listed P04 as the only ready candidate.
- Read `goal_authoring_and_execution_guide.md` completely before initialization.
- Confirmed the P04 child path did not exist, then created `PLAN.md`, `EXPERIMENTS.md`,
  `EXPERIMENT_NOTES.md` before any P04 production investigation.

### Exact commands

```text
read goal_authoring_and_execution_guide.md completely
test ! -e <P04 child directory>
mkdir -p <P04 child directory>
```

### Result

- P04 required documents: 3/3.
- Path collision: none.
- Tests/probes: 0; pass/fail/skip 0/0/0 in this activation unit.
- Production changes: 0; child/Master Goal documents only.
- Current inherited P03 exit evidence: Core 1065, game_server 71, 0 failed/ignored.

### Interpretation

P04 may become the sole active Phase. Before production mutation it must inventory the current typed NodeConfirm,
mastery/source/cycle/reward/XP/duration/server paths, classify dirty target hunks and establish a focused current
baseline. Unity remains excluded.

## 2026-08-12 — Production Authority, Dirty Ownership And Baseline Characterization

### Hypothesis

P04 can remain a battle-independent two-step result while reusing the existing ordered grant transaction, exact
encounter ledger/cycle, active-consumable decrement and staged node-completion authorities. A stored completed plan,
rather than a fake battle or a confirmation-time replan, can satisfy reconnect and all-or-nothing requirements.

### Performed work

- Read the current `ActionKind`/`PlayerBehavior` routing, common capability projection, `NodeConfirm` context,
  selected-event projection and command-result projection.
- Read the Endless Elite assignment, mastery/source/cycle reducer, combat reward resolution, ordered economy grant,
  survival XP, consumable duration and node completion plan/commit seams.
- Confirmed the approved two-step boundary and living combat-available armed prerequisite from `SW-01~05`, then
  corrected the Phase plan before production mutation.
- Inspected every intended existing target path with `git diff --`; all existing changes are inherited P01~P03/user
  work. P04 owns only later added hunks and the new sweep module/tests.
- Ran focused pre-mutation characterization.

### Exact commands and results

```text
cargo test -p game_core encounter_progression -- --quiet
  6 passed; 0 failed; 0 ignored

cargo test -p game_core action_capability -- --quiet
  1 passed; 0 failed; 0 ignored

cargo test -p game_core dismantle -- --quiet
  7 passed; 0 failed; 0 ignored

cargo test -p game_server behavior_contract -- --quiet
  0 selected; 71 filtered; 0 failed
```

### Authority inventory

- Eligibility identity: current `NodeConfirm` node payload + Endless mode + Elite source + completed source ledger +
  Monolith victory count 3 + current `mastered_elite_cycle.assigned_source_ids` + pure roster
  `Employee::is_battle_deployable` check.
- Base rewards: the current node's authored combat override when present, otherwise encounter reward UUIDs, followed
  by the current node reward multiplier and existing `ClaimAll` validation.
- Sweep XP: existing `run_policy.growth.post_battle_survival_xp`. The generic `AliveRoster` executor divides an
  amount across recipients, so P04 uses the same ordered grant authority with one `SelectedEmployee` effect per
  living employee to preserve the approved per-employee full amount; Dead is excluded.
- Exclusive source branch: source-complete + owned count 0 grants the same fragment; owned count 1+ grants only the
  authored repeat effect. The source ledger remains unchanged.
- Grant roots: `EconomyGrantState`; duration: the existing employee modifier decrement operation; node/session/map:
  staged node completion converted to the combat-result-safe completion type without battle recovery effects.
- Reconnect: a new battle-independent `GameState`/selected-event result stores the already-computed commit plan;
  confirmation consumes that exact plan and does not replan.
- Persistence: no new Active Run V4 or PlayerProfile V2 field. The staged pre-confirmation result is transient actor
  state; committed effects are already represented by existing V4 roots.

### Dirty ownership classification

- Existing modified files include `behavior.rs`, `resources/selection.rs`, `world.rs`, `action_capability.rs`,
  `combat.rs`, `node_flow.rs`, `snapshot.rs`, `state.rs` and several game_server files. Their pre-P04 hunks are
  inherited and must not be rewritten.
- `encounter_progression.rs` is untracked in the outer worktree but is P02-owned completed work; P04 may add only its
  explicit sweep-plan reducer hunk.
- The P04 Goal directory is P04-owned. No Unity path was read, tested or changed.

### Interpretation

The authority graph is closed and no policy stop condition is present. Production mutation may begin with target-first
tests and a separate sweep module plus minimal typed routing/projection hooks. The server behavior filter is not a real
test target; final validation must use the full non-zero game_server suite and its concrete serialization tests.

## 2026-08-12 — Typed Synthetic Sweep Implementation

### Hypothesis

A separate non-battle world module can stage the exact reward/progression plan on cloned roots and expose it through
the existing generic command transport. Confirmation can consume that immutable plan in one transaction without a
new V4/profile field or a server-owned reducer.

### Performed work

- Added `SweepMasteredElite {}` and `CompleteMasteredEliteSweep {}` as strict empty typed commands, their action
  descriptors, NodeConfirm direct/sweep projection, staged context/result DTOs and generic command-result mapping.
- Added one shared Core evaluator for current Endless/Combat/Available/Elite/mastery-3/source-complete/current-
  assignment/exact-session/deployable-roster eligibility.
- Added `world/mastered_elite_sweep.rs`: stage computes base rewards, exclusive source branch, per-living-employee
  full XP, duration decrement and node completion on cloned roots; live roots are restored before publishing the
  transient result. Confirm revalidates UUID, exact node/session/encounter, mastery/source/assignment and roster,
  then consumes the stored plan without replanning.
- Added progression reducers for owned-0 reacquire versus owned-1+ repeat and assignment consumption without mastery
  or source reset. Reused the same combat reward preview/resolution and ordered `EconomyGrantState` authority without
  creating `ActiveBattle` or `CombatBattleState`.
- Kept staged plan outside Active Run V4/checkpoint and PlayerProfile V2. Checkpoint restore clears transient staged
  state and rolls committed run-local effects back through existing V4 roots.
- Added Core integration, reducer, fragment-lifetime and game_server serde/transport tests. No live RON content row
  was added or changed for P04.

### Target-first and focused results

```text
cargo test -p game_core mastered_elite -- --quiet
  6 passed; 0 failed; 0 ignored

cargo test -p game_core final_copy_dismantle_and_reacquisition -- --quiet
  1 passed; 0 failed; 0 ignored

cargo test -p game_core behavior::tests -- --quiet
  11 passed; 0 failed; 0 ignored

cargo test -p game_server mastered_elite -- --quiet
  1 passed; 0 failed; 0 ignored
```

### Protected behavior

- Direct battle remains `ConfirmEnterNode`; Standard/unmastered/non-Elite nodes do not offer sweep.
- Stage produces a stable selected-event result and changes no authoritative reward/progression/profile/battle root.
- Confirm applies base rewards once; owned 0 reacquires exactly one fragment, owned 1+ grants repeat only.
- Every living roster employee receives the full configured XP, Dead receives none, and active duration decreases
  once. Assignment/source/node/session/navigation commit atomically.
- Reward overflow and stale confirmation rollback; reconnect reprojects the stored result; a committed result cannot
  replay.
- Condition, BossOmen, monolith, profile, compendium, event log, result stats and battle trace stay unchanged.
- Checkpoint load rolls the committed run-local sweep back while incrementing monotonic `loads_used` normally.
- Final-copy dismantle and repeated reacquisition preserve upgrade/awakening progress, mastery and source completion.

### Interpretation

The implementation satisfies the approved two-step contract without a fake battle, compatibility field or new
content/schema. A second-pass completion audit remains required before Phase status changes.

## 2026-08-12 — Final Validation And Phase Exit Audit

### Performed work

- Re-read the complete completion-review guide and audited all 16 completion conditions against current code,
  producer/server transport, strict versions, tests and canonical documents.
- Tightened confirmation-time stale validation to exact encounter payload, available Combat node, exact session,
  mastery 3, completed/currently assigned reward source and deployable roster; reran focused and broad suites.
- Updated `game_rulebook.md` and `core_runtime_contract.md` to describe the implemented battle-independent staged
  result and exact atomic commit boundary.
- Ran formatting, whitespace, fake-battle/legacy/version and consumer static searches. Unity remained uninspected and
  unmodified by explicit scope.

### Final results

```text
cargo test -p game_core -- --quiet
  1074 passed; 0 failed; 0 ignored
  (1023 lib + 4 live item + 4 live catalog + 25 live RON + 14 skill runtime + 4 skill suite)

cargo test -p game_server -- --quiet
  72 passed; 0 failed; 0 ignored

cargo fmt --all -- --check
  passed

git diff --check
  passed
```

### Result

- Completion conditions: 16/16 pass.
- Baseline failures: none; the activation-time zero-selected server filter was replaced by a real focused producer
  test and the full non-zero suite.
- Audit verdict: `Complete / High`.
- Remaining scope boundary: Unity integration and final content/balance remain explicitly unverified/out of scope;
  no compatibility fallback was added.
