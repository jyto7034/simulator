# Boss Omen Gate And Rollback Policy

Goal ID: G14

Status: complete

Implementation complete: true

Updated: 2026-07-14

Primary findings: DCA-H-14, DCA-H-15

## Required Resume Read Set

1. `../cross_layer_contract_hardening_master/STATUS.md`.
2. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/PLAN.md`.
3. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/EXPERIMENT_NOTES.md`.
4. This `PLAN.md`.
5. This Goal's `EXPERIMENTS.md`.
6. This Goal's `EXPERIMENT_NOTES.md`.
7. `../battle_failure_abandonment_consumable_resolution/COMPLETION_REVIEW.md`.

Read Master PD-10 or audit provenance only when a concrete transition detail conflicts with live
runtime evidence.

## Objective

Make the BossOmen chain capstone use the completed G15 combat transaction without bypassing normal
Floor transition or hiding accepted checkpoint recovery. Victory commits the boss result, clears
the completed chain, and opens one dedicated exit Gate; only Gate confirmation applies normal Gate
trauma recovery and advances the Floor. Defeat commits the result and enters RunFailed first; only
a valid remaining SavePoint checkpoint load is exposed from that RunFailed state.

## Accepted Transition Table

| Exit | Boss result | BossOmen state | Map/Floor | Recovery/action |
| --- | --- | --- | --- | --- |
| Forced BossOmen victory | commit G15 result once | consume/clear completed chain and forced boss | complete boss, create one immediately selectable dedicated Gate on the same Floor | Gate is sole progression action |
| Dedicated Gate confirm | no combat replay | cleared chain remains cleared | apply normal Gate transition recovery and advance Floor | normal next-Floor map |
| Forced BossOmen defeat with checkpoint capacity | commit G15 result once, enter RunFailed | failed run state remains authoritative until load | no retry/cancel/map recovery node | expose existing `LoadRunCheckpoint` capability only |
| Accepted checkpoint load | no result rollback logic in combat | restore checkpoint BossOmen/map state | increment load count and return to checkpoint map | normal ViewingMap actions resume |
| Forced BossOmen defeat without valid load | commit once, enter RunFailed | failed state retained | no transition | no actions; run remains ended |
| `RetreatBattle` before result | exact G15 restore | restore pre-entry BossOmen/map state | no boss/node/Gate mutation | NodeConfirm; no attempt consumed |

The forced BossOmen capstone is not the final boss of an Endless run. A standard final-floor boss
does not receive this dedicated Gate/checkpoint exception unless it is also the active forced
BossOmen node under the accepted state.

## Repository Baseline

All repositories are heavily dirty and contain completed predecessor work. Existing changes are
user-owned; do not stash, reset, restore, clean, or baseline-commit them.

G14 begins immediately after G15 Complete / High. Capture exact fingerprints in `EXPERIMENTS.md`
before implementation edits.

## In Scope

- Distinguish active forced BossOmen capstone completion from ordinary terminal boss completion.
- Stage/commit a dedicated Gate node and map progression without direct Floor advance on victory.
- Reuse normal Gate confirmation and `GateTransition` recovery rather than duplicating recovery.
- Expose existing `LoadRunCheckpoint` capability from qualified BossOmen RunFailed only when the
  checkpoint exists and remaining-load capacity is valid.
- Revalidate the same predicate on command execution; accepted load increments count exactly once.
- Preserve G15 result atomicity, exact abandonment, and fatal priority.
- Update canonical Core/Unity contract semantics and focused/broad tests.

## Explicit Non-Scope

- G08 Unity recovery/roster UX beyond consuming the existing action/capability shape.
- A new checkpoint DTO, save schema, command variant, failure reason, balance value, or Gate reward.
- General recovery from arbitrary RunFailed reasons or non-BossOmen bosses.
- BossOmen content redesign, Event vertical slice work, battle-record/compendium work, or deferred
  G05/G22 content.

## Planned Implementation

1. Add failing focused tests for forced-boss victory direct advance and missing RunFailed checkpoint
   capability, plus no-checkpoint and ordinary-final-boss negative controls.
2. Introduce one explicit forced-BossOmen completion predicate from run state/current node.
3. Plan the post-victory same-Floor map by completing the boss, clearing the chain, and inserting
   exactly one dedicated Gate as the only selectable successor.
4. Let existing Gate confirm apply `StagedSupportEffect::GateTransition` and advance the Floor.
5. Share one qualified RunFailed checkpoint predicate across action projection and execution.
6. Prove load count, restored map/BossOmen/checkpoint state, and no retry/cancel leakage.
7. Synchronize contracts, run broad Core/server/Unity validation, and perform the completion review.

## User-Visible Completion Conditions

- Forced BossOmen victory first returns a committed combat result whose completion remains on the
  current Floor with exactly one selectable Gate.
- Floor index and Gate trauma recovery do not change until Gate confirmation.
- Gate confirmation advances once through the ordinary Gate transition path.
- Forced BossOmen defeat always reaches RunFailed before recovery; a valid checkpoint adds only the
  existing load action, while no valid load exposes no action.
- Loading restores the checkpoint and consumes one load. It does not resurrect the failed battle
  session or preserve post-checkpoint boss/result mutations.
- Standard final boss behavior and G15 abandonment/recovery semantics do not regress.

## Planned Validation

1. Focused forced-boss victory/Gate/defeat/load matrix.
2. G15 exact abandonment, fatal defeat, result consumption, and typed recovery regressions.
3. Existing BossOmen generation/source/forced-node and SavePoint load suites.
4. Core library and all-targets; game_server check/all-targets.
5. Unity MCP EditMode and applicable PlayMode using existing capability/command DTOs.
6. Formatting, scoped diff, direct-Floor-advance/static-recovery/legacy searches.
7. Guide-based completion review before G08 unlock.

## Stop Conditions

Stop if implementation requires a new public command/DTO/save schema, a new Gate reward or recovery
meaning, general RunFailed recovery beyond PD-10, or an unrelated unclassifiable dirty overlap.
