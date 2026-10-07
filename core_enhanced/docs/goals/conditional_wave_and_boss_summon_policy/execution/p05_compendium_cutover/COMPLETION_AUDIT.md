# P05 Compendium Cutover Completion Audit

- Goal ID: `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`
- Phase ID: `PHASE-CWBS-05-COMPENDIUM-CUTOVER`
- Audit Pass: `1`
- Audit Date: `2026-08-17`
- Audited Target: hashes in `EXPERIMENTS.md / Current GEX-10 target identities`
- Dependencies:
  - `GOAL-CWBS-P01-RUNTIME-FOUNDATION / Complete / High / Audit Pass 3`
  - `GOAL-CWBS-P04-OUTCOME-CUTOVER / Complete / High / Audit Pass 1`
- Verdict: `Complete`
- Confidence: `High`
- Correction Required: `none`

## Fresh-pass method

This audit began after the P05 implementation record and target hashes were frozen. It independently traced the current
successful-spawn producer, P01 ledger lifetime, P04 result transition, Victory and Defeat consumption branches, permanent
profile/snapshot shape, server envelope/reset path and SavePoint payload. It did not infer completion from the GEX-10
summary and did not repeat the run-wide full baseline.

## Dependency and target integrity

- P01 still owns a cloneable process-local `BattleCore`/active session and server detach/attach keeps the same battle
  paused without TTL. P04 still commits exactly one stable Victory/Defeat result and consumes it transactionally.
- Recomputed target hashes match the GEX-10 identities. Scoped `git diff --check` and final
  `cargo check -p game_core -p game_server` pass. The first server test compile ICE was confined to obsolete task-local
  incremental metadata; a clean incremental rebuild passed without source correction.
- Existing dirty work remains preserved; the retired deleted store was not restored and no external profile,
  production promotion, Unity surface or deployment was touched.

## Requirement-by-requirement audit

| Freeze acceptance | Current authoritative evidence | Audit result |
| --- | --- | --- |
| successful actual opponent spawn is the candidate source | The shared post-commit spawn recorder adds only opponent `Abnormality`/`CorrodedEmployee` identities using authoritative event sequence. Preview/reference, player and summoned-unit sources do not enter the ledger; failed/canceled/empty requests never call the recorder | pass |
| candidate survives death/despawn/reconnect, not process death | Ledger state is independent of unit lifecycle and remains inside cloneable active `BattleCore`; no removal path erases it. P01 detach/attach preserves that same paused active battle. It is transferred only to process-local result state and is absent from durable payloads | pass |
| final Victory commits every candidate atomically/idempotently | Result transition freezes the full ledger. Only the Victory planner validates it; all Defeat branches contain no compendium call. Both Victory branches merge only after fallible completion work, and consumed result state prevents duplicate completion. Primary/death/reward credit is never consulted | pass |
| Defeat/Retreat/process death do not register | Retryable and exhausted Defeat fixtures retain the previous set exactly; Retreat restores the entry snapshot and drops the active attempt. Process restart restores only the last SavePoint and cannot contain candidates | pass |
| one new permanent profile schema | Core persisted state contains only `registered_ids`; snapshot rows contain one `registered` boolean. New/reset profiles are empty and Core/server round-trip/backup/reset fixtures pass | pass |
| no legacy union/migration or durable battle state | Server version 3 has no legacy decoder/migration function, rejects version 2 and ignores retired v1 files. Old four-set fields are strict unknowns. `RunCheckpointPayload` contains no active battle, result or candidate field | pass |

## Focused validation evidence

- Core/server test-target compilation: pass; server pass follows one task-local incremental cache recovery
- Core compendium schema/profile: `6 / 0`
- successful spawn and ordered/idempotent attempt ledger: `2 / 0`
- final Victory, every Defeat/Retreat and failed completion: `3 / 0`
- server profile store v3/round-trip/reset/no-migration: `11 / 0`
- server compendium bootstrap/attach/reset/failure handling: `6 / 0`
- shared local-install profile and no-TTL paused reconnect: `2 / 0`
- fixture generation and retired-symbol audit: pass
- production Core/server check: pass, zero warnings

The exact commands, counts and the compiler-cache incident are recorded in `EXPERIMENTS.md`. The inherited full baseline
remains `game_core 1,080 + game_server 72 = 1,152 / 0 failed`; current final full counts are intentionally deferred to
P06 under the Master validation contract.

## Scope and forward ownership

- P05 does not add a client-visible pending-candidate list; Unity UI is outside scope. Same-process retention is an
  authority/lifetime property of the active session, not durable resync persistence.
- No repository-owned development profile artifact existed. Version-3 empty reset/regeneration is executable, while
  migration or mutation of an external profile directory is neither required nor authorized.
- P06 owns the final placeholder/reference, canonical documentation and integrated full-suite audit. Production content
  promotion and deployment remain outside scope.

## Verdict

`GOAL-CWBS-P05-COMPENDIUM-CUTOVER` is complete at High confidence. `PER-CWBS-BL-05-CODE-006` has current producer,
lifetime, transaction, schema/store and executable evidence; no P05 stop condition is active and no correction Goal is
required. P06 may begin GEX-00 against this audited target.
