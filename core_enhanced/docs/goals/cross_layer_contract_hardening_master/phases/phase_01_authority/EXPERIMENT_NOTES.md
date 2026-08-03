# Phase 01 Authority Experiment Notes

Status: complete

## Corrected Phase Findings Pending R01 Confirmation

- MSTR-05: G30 now composes one Core battle evaluator for projection and execution preflight.
- MSTR-06: G30 now composes missing/stale authority through one current-resync server builder.

G30 reviewed both as Complete / High. R01 must independently confirm the combined Phase before the
findings become Phase-closed.

## Cross-Child Ownership

- G01 remains the authority for revision commit and server delivery cursor/order.
- G06 remains the authority for shared capability DTO, scope, reason vocabulary, and common preflight.
- G30 corrects only the G07 battle-context execution composition and missing/stale server recovery.
- G12 strict live-data publication is unaffected.
- G08 later owns final player-facing recovery UX and must consume this Phase contract.

## Required Long-Term Shape

- One pure Core battle-context evaluation result drives both projection and execution preflight.
- Handler payload validation remains downstream; no handler-by-handler blocking-VN patches.
- One server authority-recovery builder handles missing and stale while preserving distinct errors.
- Server proactive resync is the recovery guarantee; do not add a redundant Unity inference fallback.

## Out-Of-Scope Follow-Up

- Consider staging or rollback inside `BattleCore::deploy_player_unit` so exceptional internal spawn
  failures cannot leave a partial scenario group. Current validated caller inputs make this remote;
  it is not a G30 or Phase 1 exit blocker.
