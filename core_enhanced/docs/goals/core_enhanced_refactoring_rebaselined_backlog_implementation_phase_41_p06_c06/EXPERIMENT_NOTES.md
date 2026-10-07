# Phase 41 CRA-P06-C06 Experiment Notes

Status: complete
Implementation Complete: true

## Structural Decision

- This is one public ghost-input cleanup, not a scenario scheduling redesign.
- Historical `once: true` counts locate constructors but do not prove the current field is unread or universally
  constant. Current code/data/serde/external inventory owns the disposition.
- Target authority is current enqueue/order plus runtime identity/deduplication, with no replacement repeat flag.

## Direct Dependency

- Phase 40 `Complete / High` is the exact linear predecessor; its audit hash is `ea94b13a...`.
- Closed P06 lifecycle/order/atomicity behavior and Boss scheduling remain inherited invariants.
- Phase 42 event-log I/O, Phase 43 range/delivery and Phase 44 package cleanup remain separate.

## Policy And Compatibility

- The user approved all remaining registered cleanup, including verified public/serde breaking removal without shim
  or deprecation after each Phase's inventory and behavior guards.
- Approval is not blanket schema deletion. Any current reader, `false` value, omitted/default meaning, repeated
  schedule, serialized data contract or external consumer is evidence to stop or retain.
- Repeating scenario events are an excluded feature, not a compatibility fallback.

## Dirty Worktree

- Pre-Child expanded status is 585 rows at `54b165d7...`; existing rows are user/completed-campaign work.
- Phase 41 path did not exist. The three initialized documents are the only expected pre-activation additions.
- Candidate carriers may already be dirty from completed Phases, but current paths/hashes/diffs are deliberately not
  opened until activation and prospective fingerprinting.
- Protected marker `game_server/logs/app.log.2026-08-09` must remain 57 bytes at `614d2fbd...`.

## Current Evidence Gate

After sole activation, filename-only discovery must fix prospective scenario/data/test/consumer fingerprints before
content inspection. Current discovery then reconstructs exact definition/derives/defaults, every constructor and
actual value, all reads/matches, serde/live data and public consumers, followed by nonzero retained behavior guards.

## Current Disposition

- `ScenarioEvent` is public and potentially publishable but only `Debug, Clone`; it has no serde/default/data shape.
- The pre-assertion inventory was 23/23 `once: true`, with no `false`, read, assignment, match, default or external
  consumer. The retained event-validation assertion temporarily raised the mechanical write count to 24.
- One-time admission is owned by the sole `start_battle_execution` enqueue call. One-time group materialization is
  owned by `ScenarioRuntimeState.spawned_groups` and is now directly asserted.
- User-approved immediate public cleanup was applied after the retained validation/dedupe plus delayed/forced/order
  baselines passed with the old field present. The member and all 24 writes are absent without replacement; both
  complete Core profiles pass 1,059/0/0 and the public/data/save/server/Unity boundaries remain unchanged.

## Out-Of-Scope Handling

Do not add repeat/periodic scheduling, change event ordering or group identity, absorb event-log file I/O cleanup, or
modify range/package surfaces. Record independent findings as later-Phase inputs only.
