# Boss Omen Gate And Rollback Policy Experiments

## 2026-07-14 — Goal initialized

Hypothesis:

The completed G15 result transaction can be reused while replacing only the forced-BossOmen
post-result node completion and RunFailed capability seams.

Action:

- Created the isolated G14 documents after G15 passed Complete / High.
- Recorded accepted PD-10 transitions and the standard-final-boss negative boundary.
- Performed initial source discovery of BossOmen forced-node state, combat completion, Gate
  transition, run checkpoint load, action capability, and existing tests.

Commands and actual test counts:

- Documentation and source discovery only; 0 new G14 tests executed and no implementation success
  claimed.

Observed result:

- Forced BossOmen nodes replace `map.terminal_node_id`; ordinary combat-result node completion thus
  treats victory as terminal and plans direct Floor advance/run completion.
- Existing node completion clears forced-boss state inside that direct terminal transition.
- Defeat already commits through G15 and enters `RunFailed(BossDefeated)`, but
  `LoadRunCheckpoint` capability is relevant only in ViewingMap, so RunFailed exposes no accepted
  recovery even when a checkpoint can load.
- Existing Gate confirmation already applies `GateTransition` and advances the Floor, making it the
  intended reusable recovery/progression path.

Repository baseline after G14 document initialization:

- Core scoped view: `main` / `c2c5dc5`, dirty fingerprint `6cddbef7...`; G14 documents are the only
  intentional G14 additions so far, while runtime overlaps are predecessor/user-owned.
- game_server scoped view: `main` / `c2c5dc5`, dirty fingerprint `0a9b6db5...`; no G14 change.
- Unity scoped view: `master` / `7cd8df1`, dirty fingerprint `e6d09e52...`; includes the completed
  G15 canonical/live-test synchronization and unrelated pre-existing changes, no G14 change.

Decision and next step:

Capture repository fingerprints and add focused failing transition tests before implementation.

## 2026-07-14 — Accepted transition regressions reproduced

Hypothesis:

Forced BossOmen victory still uses ordinary terminal completion, and qualified BossOmen RunFailed
still suppresses the existing checkpoint action.

Commands and actual test counts:

- Forced victory dedicated-Gate focused test: 1 executed, 0 passed, 1 failed, 678 filtered.
- Forced defeat valid-checkpoint focused test: 1 executed, 0 passed, 1 failed, 678 filtered.
- Ordinary standard final-boss negative control with a valid checkpoint: 1/1 passed, 678 filtered.

Observed result:

- Victory returned `FloorAdvanced` instead of the required same-Floor `NodeCompleted` plus Gate.
- BossOmen RunFailed had no `LoadRunCheckpoint` capability despite a valid checkpoint.
- An ordinary standard final boss with a valid checkpoint still exposed no action, confirming the
  special predicate must be forced-BossOmen-specific.

Decision and next step:

Implement the dedicated Gate planning seam first, then share the forced-BossOmen RunFailed
checkpoint predicate between capability projection and execution.

## 2026-07-14 — Dedicated Gate and qualified RunFailed load implemented

Hypothesis:

A deterministic same-Floor Gate plan and one forced-BossOmen failure predicate are sufficient; no
new public command/DTO/save field or duplicated Gate recovery path is needed.

Action:

- Added deterministic `complete_forced_boss_with_exit_gate`: complete the forced terminal boss,
  mask every other available node, create one forward-only `boss_omen_exit_gate`, make it terminal
  and selectable, and clear the completed chain.
- Extended internal staged node completion to carry an optional planned BossOmen state so the Gate
  map and cleared chain commit together without label inference or early mutation.
- Reused ordinary Gate confirmation and `GateTransition` recovery for Floor advance.
- Added `is_forced_boss_omen_run_failed` and used it in both checkpoint action capability and load
  handler preflight. Standard final boss and exhausted/missing checkpoint controls remain actionless.
- Updated Core and external Unity canonical contracts.

Changed files:

- `src/game/boss_omen.rs`.
- `src/game/world/{node_flow.rs,action_capability.rs}`.
- `src/game/world/tests/combat.rs`.
- Core/Unity canonical documents and G14/Master status documents.

Commands and actual test counts:

- Forced BossOmen focused matrix: 3/3 passed.
- Combat world module: 55/55 passed.
- Core library: 679/679 passed, 0 failed/ignored.
- Core all-targets: 720 passed across nonzero targets
  (`679 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- game_server check: passed; all-targets: 28/28 passed.
- Unity MCP EditMode job `1ca3cf6a6ef14c289af5842f9656dc3f`: 189/189 passed.
- Unity post-run error console after clearing TestRunner output: 0 entries.

Observed result:

- Victory remains on Endless Floor 0, keeps trauma unchanged, clears forced/active chain state, and
  exposes only the dedicated Gate. Confirming it applies normal 10% Gate trauma recovery and returns
  `FloorAdvanced` for Floor 1.
- Qualified forced-BossOmen defeat exposes exactly one enabled checkpoint action; loading returns the
  saved ViewingMap/map/BossOmen state and increments `loads_used` once.
- Forced defeat with exhausted capacity and ordinary final boss defeat with a valid checkpoint both
  expose no action.

Failure cause:

- The first Gate implementation left unrelated manually-available test-map nodes selectable. The
  forced boss contract already requires exclusivity, so Gate planning now masks all other available
  nodes before inserting the exit Gate; the focused test then passed.
- Two test setup edits initially matched overly broad repeated setup text and landed in unrelated
  tests. They were removed and reapplied with function-specific anchors before any broad validation.

Decision and next step:

Perform the guide-based completion review for DCA-H-14 and DCA-H-15. Unlock G08 only for Complete /
High with no immediate correction.

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
