# P05 Compendium Cutover Current Notes

## Current Summary

- Current pass: `GEX-20 complete / Audit Pass 1`
- Dependencies: `P01 Complete / High / Pass 3`; `P04 Complete / High / Pass 1`
- Policy question: `none`
- Blocker: `none`
- Completion: `Complete / High`
- Baseline: inherited initial full baseline; no phase-start rerun
- Dirty attribution: targeted targets are accepted current dirty actual; the deleted old compendium store remains
  retired in favor of the current consolidated player-profile store

## Preserved Boundaries

- Successful opponent spawn commit is the candidate source. Result-time event-log reconstruction is not target behavior.
- Candidate identity survives death, despawn and same-process reconnect but is neither permanent nor SavePoint-durable.
- Only the final committed Victory registers candidates. Defeat, Retreat and process death do not.
- Registration is whole-ledger, atomic and idempotent; primary target and combat credit are irrelevant.
- The permanent profile authority is one canonical registered set with an empty replacement/reset schema and no legacy
  four-set union migration.
- P01 process/session and SavePoint contracts and P04 outcome/result transaction remain authoritative.

## Initial Technical Direction Within Approved Policy

- Locate the P01 active battle/session ledger seam and attach ordered identity-deduplicated compendium candidates to that
  process-local state rather than to durable run checkpoint/profile data.
- Record eligible opponent canonical identity at the successful spawn commit shared by initial and later spawned units.
- Replace the current compendium result writer with a Victory-only staged registration in the existing world completion
  transaction; clear/discard the attempt ledger on every terminal consumption path.
- Replace Core/server four-set DTO/store/reset shapes directly. Remove legacy readers/migrations instead of keeping an
  alias or union bridge.
- Rebaseline only source/store/result/session tests invalidated by this cutover, then perform a fresh GEX-20 audit.

## GEX-10 Decisions and Findings

- The existing P01 `BattleCore` ledger is the single attempt-candidate authority. No duplicate world/server ledger was
  introduced.
- `CombatBattleState` carries the frozen candidates only between finalization and result consumption. It remains
  process-local and outside both `RunCheckpointPayload` and active-run persistence.
- Canonical permanent identity is the raw catalog ID in one set. Candidate kind remains attempt-local for source-domain
  checking and snapshot catalogs independently query the one set.
- Registration is intentionally not derived from event log, death, primary target, reward policy or progression. The
  old event log remains presentation/stat evidence only.
- Core registration executes only after every fallible completion/recovery decision in each Victory branch. This keeps
  the permanent merge as the final step despite the wider monotonic profile rollback rule.
- Server schema generation is version 3. Old v2 envelopes are unsupported and legacy compendium files are ignored; this
  is the approved pre-release blank replacement, not a migration failure fallback.
- Explicit full-profile reset still deletes retired artifacts as cleanup, but no normal load reads, unions or rewrites
  them.
- No repository-owned development profile artifact existed to regenerate. No external profile directory was touched.

## GEX-20 Result

- Audit Pass 1: `Complete / High`.
- Correction Goal: `none`.
- No P05 stop condition is active. P06 may begin against the frozen hashes in `EXPERIMENTS.md`.
