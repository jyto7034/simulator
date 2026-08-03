# Boss Omen Gate And Rollback Policy Completion Review

Goal: G14

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Overall verdict: Complete

Overall long-term fit: High

Immediate correction: none

## Audit Table

| Item | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- |
| DCA-H-14 forced BossOmen victory bypasses Gate | Complete | High | none |
| DCA-H-15 BossOmen RunFailed lacks checkpoint recovery | Complete | High | none |

## DCA-H-14 — Dedicated BossOmen Exit Gate

Master item:

The forced BossOmen node replaced the map terminal, so ordinary combat completion treated victory as
terminal and advanced the Endless Floor directly, bypassing the independent Gate and its recovery.

Policy source:

PD-10 requires the committed boss result to clear the chain and open a dedicated Gate on the same
Floor. Gate confirmation is the sole Floor transition and applies ordinary Gate trauma recovery.

Expected behavior:

Victory commits the G15 result once, completes the forced boss, clears BossOmen chain state, and
exposes exactly one deterministic Gate. Floor index and Gate recovery remain unchanged until the
player selects/confirms that Gate.

Runtime evidence:

- `src/game/boss_omen.rs` owns `complete_forced_boss_with_exit_gate`. It verifies current/terminal
  forced-boss identity, completes that node, masks other available nodes, deterministically allocates
  a new map id/slot, inserts one forward-only `boss_omen_exit_gate`, and clears forced/active state.
- `src/game/world/node_flow.rs` calls that plan only for non-interactive combat-result completion and
  carries the planned BossOmen state beside the staged map/progression. No early run mutation or
  kind-id inference is used at commit.
- Existing `handle_confirmed_gate_transition` remains the only Floor-advance path and reuses
  `StagedSupportEffect::GateTransition` for trauma recovery.
- G15 employee/result/consumable staging remains unchanged and commits before node completion.

Data evidence:

- Existing BossOmen encounter/chain RON and Gate recovery policy values are reused. No new live RON
  field, reward, encounter, or balance value was added.

External contract evidence:

- Existing `NodeCompleted`, map node, `map_navigation`, Select/Confirm, and `FloorAdvanced` shapes are
  reused. Core and external Unity canonical docs now state the two-step result -> Gate -> Floor flow.
- Unity is instructed not to advance locally after boss victory and to treat the Core-projected Gate
  as an ordinary Gate.

Test evidence:

- The focused victory test proves Floor 0 and trauma 90 remain after result, forced/active state is
  clear, only one correctly typed Gate is selectable, and Gate confirmation returns Endless Floor 1
  with trauma 81: 1/1 passed.
- Existing Gate transition, BossOmen generation/source, combat module 55/55, and Core library
  679/679 passed.

Legacy/fallback audit:

- The old focused test expecting immediate `FloorAdvanced` was replaced.
- Forced victory no longer reaches generic terminal completion. No synthetic support effect,
  duplicate Floor generator, kind-name fallback, compatibility node, or ignored regression remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: deterministic Gate creation remains in BossOmen map ownership, while ordinary Gate
  confirmation remains the sole recovery/Floor-transition owner. Internal staged state keeps map and
  chain atomic without a public schema.

Verdict:

- Complete.

Remaining risk:

- G08 must render/confirm the projected Gate without introducing a local BossOmen transition.

## DCA-H-15 — Explicit Checkpoint Recovery From BossOmen RunFailed

Master item:

Forced BossOmen defeat correctly reached RunFailed but the existing checkpoint action was relevant
only in ViewingMap, so accepted remaining-load recovery was invisible.

Policy source:

PD-10 requires defeat to commit and enter RunFailed first. Only an existing SavePoint checkpoint
with remaining load capacity may add one explicit load action. No capacity ends the run, and the
exception does not apply to an ordinary final boss.

Expected behavior:

The same forced BossOmen/current-node predicate and `RunCheckpointState::can_load()` gate action
projection and execution. Accepted load increments once and restores checkpoint map, BossOmen,
roster/resources, and normal ViewingMap flow.

Runtime evidence:

- `src/game/world/node_flow.rs::is_forced_boss_omen_run_failed` requires typed
  `RunFailed(BossDefeated)` plus equality between current progression node and forced BossOmen node.
- `src/game/world/action_capability.rs` exposes existing `LoadRunCheckpoint` only when that predicate
  and checkpoint capacity are true; exhausted forced failure is irrelevant/actionless.
- `handle_load_run_checkpoint` revalidates the same forced-failure/capacity boundary before applying
  the existing checkpoint payload and incrementing `loads_used`.
- G15 still commits the boss result and transitions to RunFailed before capability evaluation; no
  retry/cancel or map recovery-node path was added.

Data evidence:

- Existing SavePoint payload and `RUN_CHECKPOINT_MAX_LOADS` remain authoritative. No save migration
  or new rollback copy was introduced.

External contract evidence:

- Existing RunFailed reason, action capability, `load_run_checkpoint` command, checkpoint factual
  DTO, `MapState`, and ViewingMap snapshot shapes are reused.
- Canonical Unity docs prohibit inferring boss/load eligibility and require the enabled capability.
  G08 owns final button/UX consumption.

Test evidence:

- Qualified forced defeat projects exactly one enabled action, loads the exact saved map, clears the
  post-checkpoint forced boss, returns ViewingMap, and increments load count to one: 1/1 passed.
- Forced failure with exhausted capacity projects no action: 1/1 passed.
- Ordinary standard final boss with a valid checkpoint projects no action: 1/1 passed.

Legacy/fallback audit:

- There is no automatic rollback, generic BossDefeated recovery, category-only boss check, static
  Unity affordance, duplicate command, or disabled compatibility action in terminal no-load state.
- ViewingMap's existing checkpoint capability behavior is preserved.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one typed run-state predicate composes the existing checkpoint fact and command instead of
  creating a BossOmen-specific save schema or result branch.

Verdict:

- Complete.

Remaining risk:

- Current checkpoint loading preserves the pre-existing run-local battle-record exception; G16/PD-12
  owns removal of gameplay battle records and is not a G14 recovery authority.

## Cross-Component Findings

- G14 consumes G15 rather than bypassing it: result effects commit once, and only the post-result map
  completion differs for forced BossOmen victory.
- Dedicated Gate recovery reuses ordinary Gate support logic, so balance and Floor generation have
  no parallel BossOmen implementation.
- RunFailed recovery is distinct from G15's no-deployable map restriction. The former is an explicit
  checkpoint action after terminal boss commit; the latter permits only typed map recovery nodes.
- Standard final boss and exhausted-capacity negative controls prevent the forced predicate from
  becoming a generic failure escape.
- Server and Unity receive existing shapes; no live-data/save/public-contract schema changed.

## Validation Results

- Focused G14 forced BossOmen matrix: 3/3 passed.
- Combat world module: 55/55 passed.
- Core library: 679 passed, 0 failed/ignored.
- Core all-targets: 720 passed across nonzero targets (`679 + 3 + 3 + 18 + 13 + 4`).
- game_server check: passed; all-targets 28 passed, 0 failed/ignored.
- Unity MCP EditMode job `1ca3cf6a6ef14c289af5842f9656dc3f`: 189 passed, 0 failed/skipped.
- Unity post-run error console after clearing TestRunner output: 0 entries.
- Formatting, scoped Core/Unity diff checks, direct-advance/static-recovery legacy searches, and
  relevant ignored-test searches passed.

Failed experiments retained in `EXPERIMENTS.md`:

- Both accepted transition tests initially failed on the intended regressions.
- First Gate implementation exposed unrelated manually-available test-map nodes and was corrected.
- Two broad-context test edits were removed and reapplied with function-specific anchors.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G14. G08 is the dependent Unity consumption goal, not a G14 correction.

## Policy Questions

None.

## Review Conclusion

DCA-H-14 and DCA-H-15 are Complete with High long-term fit and no immediate correction. G08 may
begin against the existing Gate/checkpoint/capability shapes while preserving G14/G15 authority.
