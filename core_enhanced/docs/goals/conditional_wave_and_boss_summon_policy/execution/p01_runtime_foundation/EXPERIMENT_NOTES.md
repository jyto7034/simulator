# P01 Runtime Foundation Current Notes

## Current Summary

- Current pass: `GEX-20 pass 3 complete`
- Implementation: `complete / high`
- Direct dependency: `none`
- Policy question: `none`
- Evidence gap: `none for P01`; final cross-consumer/full-suite evidence remains assigned to P06
- Blocker: `none`
- Initial full baseline: `passed / 1,152 tests / 0 failed`
- Dirty attribution: pre-existing P01-surface changes are preserved as current actual; only post-baseline content deltas are
  attributed to this Goal.

## Current Decisions and Boundaries

- Kernel/session foundation is implemented as one atomic Phase node; WP-01 and WP-02 are not split into independent passes.
- Active battle/request/candidate state is process-local, retained across same-process reconnect and discarded on process death.
- SavePoint payload does not contain battle state.
- Disconnect causes authoritative auto-Pause with no active-battle TTL; reconnect requires explicit Resume after snapshot.
- Successful explicit load alone consumes usage; reconnect/continue/process rollback does not.
- Old Wave and compendium consumers remain until their later Phase cutovers.

## Implemented Boundary

- Kernel accepts both Wave and Summon provenance without source-specific commit rules.
- `RequireUnoccupied` and exact `AllowOccupied` classes reject empty, duplicate, `Any`-mixed and
  `Summoner + SameFaction` policy shapes. All occupants of every distinct cell must pass; there is no numeric placement cap.
- Preflight occurs before mutation; mutation errors restore BattleCore while retaining one terminal request settlement.
- Trigger state and request settlement survive BattleCore clone restore; summoner removal can terminally cancel a bound summon
  before execution.
- Disconnect pause and request/candidate ledgers are process memory only. Checkpoint schema/load counter code was not modified.
- P01 does not claim actual Wave/Summon consumer cutover, terminal-result cutover or permanent compendium commit.
