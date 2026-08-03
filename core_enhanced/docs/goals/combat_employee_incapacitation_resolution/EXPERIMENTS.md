# Combat Employee Incapacitation Resolution Experiments

## 2026-07-14 — Goal initialized and dirty baseline captured

Hypothesis:

The accepted PD-16 contract can be implemented inside Core's world/battle boundary without a new
DTO, RON, save, Unity, or balance decision.

Action:

- Created the mandatory G26 documents and recorded CPR-H-01 and MSTR-01.
- Captured Core, game_server, and Unity branch/HEAD/status fingerprints.
- Searched the current runtime for fixed-death-redeploy, participant result, incapacitation,
  survival XP, employee change, and MVP paths.
- Inspected scoped diffs enough to identify completed Phase 1 changes in target files and the still
  active legacy fixed-60% and delayed result-time penalty paths.

Changed files:

- G26 and Phase 2 documentation only.

Commands and actual test counts:

- Repository/source/diff inspection only; 0 tests executed and no implementation success claimed.

Observed result:

- `src/game/world/state.rs` still chooses `BattleDeployCurrentHpPolicy::death_redeploy()`.
- `src/game/battle/core/sim.rs` still defines the 60% death-redeploy override.
- `src/game/world/combat.rs` still applies `apply_incapacitation` while completing participant
  results, so current redeployment cannot observe attempt-local trauma/Run HP/life-state changes.
- `src/game/battle/result_stats.rs` already merges some facts by employee but overwrites terminal
  participant fields in iteration order; the exact employee aggregation must be verified by tests.
- Target files contain completed predecessor changes. They are classifiable and must be preserved.

Decision and next step:

Trace the exact state structures and mutation ordering, then add focused failing behavior tests
before implementation.

## 2026-07-14 — Fixed-60% policy regression reproduced

Hypothesis:

The current death-redeploy path ignores employee condition-derived Battle HP and overwrites the
fresh draft with the legacy fixed 60% value.

Action:

- Changed the existing authoritative death reconciliation/redeploy test to require the pre-entry
  persistent roster to remain unchanged, an idempotent attempt-local projection, and redeploy HP
  derived through `Employee::battle_start_hp_for_max`.

Commands and actual test counts:

- `cargo test -q -p game_core live_deployment_reconciles_defeated_player_unit_during_authoritative_advance --lib -- --test-threads=1`:
  1 executed, 0 passed, 1 failed, 669 filtered out.

Observed result:

- The redeployed unit had 60 HP while the current employee condition pipeline expected 100 HP.
- The assertion failed specifically at the legacy fixed-60% override, proving MSTR-01 remains live.

Failure cause:

- `LiveBattleRedeployState` still stores `BattleDeployCurrentHpPolicy::death_redeploy()` and
  `BattleCore` applies `PercentOfMax(60)` after building the employee-derived draft.

Decision and next step:

Introduce the attempt-local employee condition owner and remove the death HP override without a
fallback, then rerun the same nonzero focused test.

## 2026-07-14 — Attempt-local condition and employee aggregation implemented

Hypothesis:

One `BattleEmployeeAttemptState` inside the active session can preserve BattleCore instance history
while making world-owned employee condition the single redeploy and result source of truth.

Action:

- Added a battle-entry roster clone plus processed `UnitInstanceId` set to the active session.
- Reconciled dead deployed employee instances into the working roster through the existing trust,
  consumable mitigation, death prevention, Run HP, trauma, injury, and life-state pipeline.
- Changed death redeploy locks to carry no HP override; withdraw locks retain their explicit carried
  HP override.
- Made battle capability, previews, commands, battle updates, roster snapshots, setup recovery, and
  VN transport consume the working roster during an active attempt.
- Carried the final working roster into `CombatBattleState`, then committed it once during result
  settlement without replaying incapacitation penalties.
- Aggregated terminal result fields chronologically by employee while preserving all instance event
  metrics; employee rows, MVP representative identity, survival XP, reward participant context, and
  outcome rows are now employee-level.
- Updated canonical runtime/rulebook wording and removed the fixed-60% runtime path.

Changed files:

- Core battle sim/result stats, world active/result state and combat flow, battle projections,
  snapshots, focused tests, and canonical Core/rulebook documents.
- No live RON, public serialized field, game_server mapper, or Unity reader/command file changed.

Commands and actual test counts:

- Death/redeploy/idempotence/repeated-death/abandonment focused: 1/1 passed.
- Employee-level chronological result aggregation: 1/1 passed.
- Existing result aggregation regression: 1/1 passed.
- Victory condition commit plus once-per-employee survival XP: 1/1 passed.
- Defeat condition commit without duplicate penalty: 1/1 passed.
- Result stats module: 4/4 passed.
- Live deployment group: 2/2 passed.
- Combat result group: 4/4 passed at that point.
- Trauma consumable, death prevention, and trust trauma modifier composition: 3/3 passed.
- Battle-start HP policy tests: 4/4 passed.

Observed result:

- Three distinct tactical deaths produce three penalty bundles; a repeated reconcile of one dead
  instance produces no duplicate.
- Fresh redeployment uses `battle_start_hp_for_max` over current working Run HP/trauma.
- Working persistent death removes the employee from redeploy projection while the entry roster
  remains untouched until commit.
- `RetreatBattle` discards the working condition; victory and defeat each commit it once.
- Existing roster JSON shape projects the working condition during battle and restored entry state
  after abandonment.
- Two runtime instances produce one employee result row and one MVP representative using the last
  deployment's terminal state, with cumulative incapacitation and metrics.

Decision and next step:

Run broad validation and hygiene, then perform the G26 completion review.

## 2026-07-14 — Broad validation failure corrected

Hypothesis:

The first full Core library run would expose call-site or legacy test assumptions not covered by the
focused filters.

Commands and actual test counts:

- First `cargo test -q -p game_core --lib -- --test-threads=1`: 670 executed, 669 passed, 1 failed.

Observed result:

- The withdraw redeploy test failed because a new tactical-death assertion block had been inserted
  into the adjacent withdraw test rather than the death test. Runtime behavior was not the cause.

Correction:

- Moved the assertion block into the authoritative death reconciliation test.
- Reran withdraw and death focused tests: 2/2 passed.
- Reran Core library after subsequent focused additions: 671/671 passed.

Decision and next step:

Retain the failed broad run as test-placement evidence and use only the corrected nonzero run.

## 2026-07-14 — Broad package and hygiene validation

Commands and actual test counts:

- `cargo test -q -p game_core --all-targets -- --test-threads=1`: 712 passed across nonzero
  binaries (671 library + 3 + 3 + 18 + 13 + 4), 0 failed/ignored; one zero-test binary was not
  counted as evidence.
- `cargo check -q -p game_core -p game_server`: passed.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`: 28 passed, 0 failed/ignored;
  three zero-test targets were not counted as evidence.
- `cargo fmt --all -- --check`: passed.
- Scoped tracked `git diff --check`: passed.
- Canonical/runtime fixed-60%, relevant ignored-test, and new-document trailing-space searches:
  passed with no matches.

Failed experiments retained:

- `cargo test ... apply_incapacitation ...` matched 0 tests and is not evidence. Listing tests found
  the exact consumable/death-prevention names; both corrected one-test filters passed.
- The first hygiene search was run from the simulator parent with Core-relative paths. `rg` reported
  missing paths, so that command is not evidence. The same searches were rerun from the Core root
  and passed.

External-state observation:

- game_server dirty fingerprint remained `0a9b6db...` and Unity tracked dirty fingerprint remained
  `733c1986...`; G26 made no server or Unity edits.
- Final scoped G26 implementation/document fingerprint before review: `1a4219be...` (to be refreshed
  after the completion documents are finalized).

Decision and next step:

Perform the guide-based completion review. Do not unlock G15 until the verdict is Complete / High
with no immediate correction.

## Experiment Template

### YYYY-MM-DD — Short title

Hypothesis:

Action:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Revert or correction:

Decision and next step:
