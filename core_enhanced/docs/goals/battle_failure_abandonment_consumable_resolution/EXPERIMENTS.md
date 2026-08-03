# Battle Failure, Abandonment, And Consumable Resolution Experiments

## 2026-07-14 — Goal initialized and current transition split traced

Hypothesis:

G26 supplies the employee commit/discard seam, but current retreat/defeat paths still contradict
accepted PD-11 in attempts, consumption, fatal priority, and recovery selection.

Action:

- Created the mandatory G15 documents and recorded all six primary findings.
- Captured Core/server/Unity baselines and inspected current retreat, result, attempt, checkpoint,
  map selection, node completion, support, and recruitment capability paths.

Commands and actual test counts:

- Source/document/diff inspection only; 0 tests executed and no implementation success claimed.

Observed result:

- Battle entry increments attempts and advances UUID state before entering `InBattle`.
- Retreat with attempts remaining returns to NodeConfirm without restoring the increment; exhausted
  retreat creates a Draw, decrements consumables, records/completes the node, or fails a boss run.
- Defeat commits employee state and consumable duration before checking attempts; attempts remaining
  returns to retry before fatal `NoLivingEmployees` is evaluated.
- `NoDeployableEmployees` handling currently uses checkpoint availability rather than the accepted
  immediately-enterable Treatment/Recruitment capability rule.
- Map projection and `SelectMapNode` validation consume the same progression selectability but have
  no recovery restriction state.
- Recruitment is runtime-explicit through HeadquartersContact options/candidates; current Support
  variants expose SavePoint/Rest only and no Treatment capability.

Decision and next step:

Add focused failing exact-restore and fatal-priority tests, then implement the battle-entry snapshot
before changing recovery deferral.

## 2026-07-14 — Exact abandonment and fatal-priority regressions reproduced

Hypothesis:

Current retreat retains the consumed attempt/UUID continuation, and current defeat retry ordering
preempts fatal roster evaluation.

Commands and actual test counts:

- Exact pre-entry projection/UUID restoration focused: 1 executed, 0 passed, 1 failed, 672 filtered.
- Fatal committed defeat priority focused: 1 executed, 0 passed, 1 failed, 672 filtered.

Observed result:

- After retreat, the root snapshot reported `attempts_started: 1`, `remaining_attempts: 2`, and the
  selected combat node remained unavailable instead of exactly matching the pre-entry snapshot.
- A committed all-dead employee condition returned the retry NodePreview path while an attempt
  remained instead of `RunFailed(NoLivingEmployees)`.

Decision and next step:

Implement the complete battle-entry snapshot and single restore path first, rerun the exact focused
test, then correct committed-defeat ordering.

## 2026-07-14 — Exact abandonment, atomic result, and typed recovery implemented

Hypothesis:

One pre-entry snapshot plus one staged committed-result state can remove the split rollback and
partial defeat paths without a public DTO/schema addition.

Action:

- Added `BattleEntrySnapshot` coverage for attempts, run/map/session/content, UUID/RNG continuation,
  roster/inventory/resources/consumables, VN/trigger state, and run checkpoint.
- Made `RetreatBattle` consume that snapshot and return the restored NodeConfirm preview without a
  Draw/result/record/node-completion branch. Event-started combat restores a reentry-ready
  NodeConfirm after the already-committed Event choice and immediate effects.
- Staged employee condition, survival XP, `OnAnyResolution` Event effects, and combat-duration
  decrement in one result state. Fatal `NoLivingEmployees` and boss failure now preempt retry.
- Added internal typed recovery capability/restriction state. Only immediately selectable
  HeadquartersContact nodes with authored candidate count and a nonempty live recruitment catalog
  qualify; Rest, SavePoint, Shop, future nodes, and checkpoint availability do not.
- Shared the restriction across map projection, `map_navigation.selectable_node_ids`, action
  capability, and `SelectMapNode` validation. Recovery-node completion rechecks deployability.
- Removed the unused retreat-only rumor-disproof mutator and replaced attempt-consuming retreat
  tests with exact restore tests.

Changed files:

- `src/game/world/{state.rs,combat.rs,node_flow.rs,helpers.rs,action_capability.rs,snapshot.rs}`.
- `src/game/combat_preview/mod.rs`.
- `src/game/world/tests/{combat.rs,map_flow.rs}`.
- Canonical Core and Unity contract documents plus the explicit Unity live-flow assertion.

Commands and actual test counts:

- Exact pre-entry restore focused: 1/1 passed.
- Fatal committed defeat priority focused: 1/1 passed.
- Typed recovery restriction, victory parity, recovery success/failure, future/Rest exclusion, and
  zero-candidate Headquarters focused tests: 5/5 passed.
- Event-started combat retreat/reentry regression: 1/1 passed.
- Core library final: 678/678 passed, 0 failed/ignored.
- Core all-targets final: 719 passed across nonzero targets
  (`678 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- game_server check: passed; all-targets: 28/28 passed.
- Unity MCP EditMode job `0f3a751a9b474723a9a4259c827098b8`: 189/189 passed.
- Unity explicit live PlayMode job `4ae11b1e0f9d475ba4434daf4a5f4167`: 1/1 passed against
  a locally started admin-enabled game_server.

Observed result:

- Repeated normal and final-boss abandonment reproduces the exact serialized pre-entry state and
  battle UUID, consumes no attempt, and creates no record or result.
- Committed defeat decrements duration once, commits working employee state once, and immediately
  fails an all-dead roster even with attempts remaining.
- No-deployable committed victory/defeat consumes the combat node and exposes only a live,
  immediately selectable recruitment node. Recruiting clears the restriction; resolving the node
  without recovery fails the run. Checkpoint load is action-locked during the restriction.
- Event-started abandonment preserves the committed choice while discarding only the battle attempt.

Failure cause:

- The first broad library run after exact restore found that Event combat had no reentry-form entry
  snapshot. It was corrected to capture committed Event state as a NodeConfirm reentry snapshot.
- Three explicit Unity live-test attempts exposed a pre-existing harness race: it treated the stale
  root `node_confirm` snapshot as current after battle setup and tried `CancelSelectedNode`, then an
  over-strict state-object wait timed out, then a root-context assertion ignored live battle
  transport authority. The harness now waits for/uses `HasActiveBattleRuntime`; the next run passed.

Revert or correction:

- No runtime rollback was needed. The Event snapshot and Unity live harness were corrected, and all
  broad suites were rerun from the final tree.

Decision and next step:

Apply the completion review guide to all six G15 findings. Unlock G14 only for Complete / High with
no immediate correction.

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
