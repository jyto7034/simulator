# Unity Recovery And Roster Command Flows Experiments

Goal: G08

Updated: 2026-07-14

## Baseline

- Core HEAD: `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Unity HEAD: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Core status fingerprint at goal initialization:
  `af9ec7ddce3a7a23a7064614cf6d5a9abb5f655a1f6a20acdf7317b0cf47230d`.
- Unity status fingerprint at goal initialization:
  `6dbfcbc85767309af2bb0801916f31a988fa3d0009043b1e1c4bd1f069093d6d`.
- Both worktrees are heavily dirty and all unrelated changes are user-owned.

## 2026-07-14 — Finding and predecessor confirmation

Question:

What exact missing surface does DCA-H-13 own after G07 and G14 close the authority policy?

Evidence:

- The audit identifies three missing dedicated Unity flows: `RecoverBattleSetupLoss`,
  `LoadRunCheckpoint`, and `MoveRosterUnit`.
- G07 already projects setup recovery in battle `action_capabilities` and preserves typed current
  battle revision authority.
- G14 exposes checkpoint load from qualified forced-BossOmen `RunFailed` using the existing root
  capability and command, while ordinary final boss/exhausted capacity remain actionless.
- Core already owns the roster payload and returns authoritative `roster_slots`.

Decision:

Keep G08 Unity-only unless discovery proves a defect in an existing public shape. Implement explicit
typed methods and player-facing capability-driven controls; do not alter Core policy or schema.

Next experiment:

Map the current screen composers, gateways, selection state, result application, and tests for the
smallest complete implementation seam. Add failing focused tests before source edits.

## 2026-07-14 — Common capability discovery

Question:

Can Unity expose roster movement strictly from the completed G06 capability contract?

Evidence:

- `MoveRosterUnit` already has a stable `ActionKind`, gameplay scope, typed request, authoritative
  `roster_order` snapshot, and typed `roster_slots` result.
- The common capability projector does not enumerate or evaluate it, and common preflight does not
  own it. Execution therefore currently falls through to the domain handler without a context gate.
- The canonical contract permits roster organization only in `ViewingMap` and `NodeConfirm`.

Result:

This is an existing public-action projection/execution parity defect, not a request for a new schema.
Unity cannot remain capability-driven until the existing action is added to the common projector and
preflight with the same map/preview context rule.

Decision and next step:

Add a focused Core red test for projected/execute parity and out-of-context rejection, then implement
the shared evaluator entry. Preserve read-only setup recovery semantics: it does not carry mutation
authority or increment the battle revision.

Observed red result:

- `cargo test move_roster_unit_`: 1 passed, 2 failed. `ViewingMap` had no projected capability and
  the same command succeeded from `GameOver`, proving both sides of the parity defect.

Implemented result:

- The existing action is now enumerated by the common projector, enabled only for nonempty roster
  organization in `ViewingMap`/`NodeConfirm`, and owned by common preflight for execution gating.
- Focused Core rerun: 3/3 passed. Map-flow regression: 28/28 passed.
- Unity adds exact typed command-bus methods, capability-driven battle setup recovery and checkpoint
  controls, and Node Map detail controls whose adjacent move/swap payloads come from `roster_order`.
- Focused Unity EditMode job `15a9f336e1a948fc81220b33ad284a1f`: 3/3 passed.

Decision and next step:

Update canonical command/presentation wording, then run broader Unity/Core/server regressions and a
live PlayMode command path before completion review.

## 2026-07-14 — Setup-loss capability-source review

Question:

Can the new recovery control still be reached when the immutable setup is the missing local object?

Evidence:

- `GameSnapshotStore.TryGetActionCapability` previously chose battle capabilities only when both
  setup and checkpoint were present.
- With an `in_battle` root context and a valid revisioned update but missing setup, the store fell
  back to the root capability list. That hid `RecoverBattleSetupLoss` precisely in its target state.
- Returning every battle capability in this state would also expose gameplay controls before scene
  reconstruction, so the safe boundary is the revisioned query/recovery scopes only.

Decision and next step:

When context is `InBattle` but the runtime is incomplete, retain only current battle query/recovery
capabilities. Never fall back to gameplay capabilities. Add a focused regression proving recovery
and resync remain reachable while Deploy/Pause/Retreat stay hidden.

Result:

- Focused incomplete-runtime capability/presenter matrix: 3/3 passed.
- Final Unity EditMode job `b856005e883f494c9c3258b5c47616a2`: 193/193 passed.
- Final Unity ordinary PlayMode job `9602b6392ae84599bdff15107a8a17d2`: 51 passed,
  2 explicit live-server tests skipped by design.
- Final explicit live-server job `15366d3ec43b4fabadacb2ccdc14e602`: 1/1 passed against a
  locally started current Rust server. It executed authoritative roster movement, setup-loss
  recovery, deployment/playback controls, and retreat.
- Core all-targets: 722 passed (`681 + 3 + 3 + 18 + 13 + 4`).
- game_server check and all-targets: 28 passed.
- Core formatting, scoped Core/Unity diff checks, command-surface searches, local server shutdown,
  and post-run Unity error console all passed.

Failed experiments retained:

- The first Core focused red run proved missing roster capability and out-of-context execution.
- The first Core library broad run found one obsolete negative capability assertion; it was changed
  to require the new enabled capability.
- A third Node Map panel passed focused presenter tests but broke the fixed two-panel UGUI layout and
  caused PlayMode hit-test cascades. The roster rows/actions moved into the existing detail panel.
- The first two expanded live runs raced a progressing battle after read-only setup recovery and
  correctly received stale mutation authority. The live sequence now pauses first and waits for the
  reconstructed setup/current projection before later mutations.
- Completion review discovery found that incomplete setup caused root capability selection. The
  store now retains only revisioned query/recovery scopes until reconstruction; gameplay stays hidden.

Decision and next step:

Run the guide-based completion review for DCA-H-13 / CPR-H-07. Unlock R02 only for Complete / High
with no immediate correction.

## Experiment Template

### YYYY-MM-DD — Short title

Question:

Evidence:

Result:

Decision and next step:
