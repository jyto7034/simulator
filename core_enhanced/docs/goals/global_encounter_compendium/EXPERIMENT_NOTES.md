# Global Encounter Compendium Notes

Goal: G29

Status: complete

## Fixed Guardrails

- IDs actually spawned, not previewed or merely authored.
- Commit with node consumption only; no retry or retreat discovery.
- Game-global monotonic union; never checkpoint/run replacement.
- Persist IDs and resolve display data from current live catalogs.
- No battle records, traces, history fields, counters, timestamps, or compatibility aliases.

## Immediate Trace

- Locate the smallest top-level state/persistence owner that is outside run/checkpoint rollback.
- Trace `BattleUnitSourceIdentity` from every abnormality/corroded spawn into active attempt state.
- Identify the shared player-victory and final consuming-defeat staged commit point.
- Determine whether an existing root/profile DTO and Unity surface already represents compendium
  knowledge; do not invent a competing client authority.

## Chosen Technical Boundary

- Add one serializable, strict, sorted-ID profile state outside every run/checkpoint/retreat owner.
- Derive attempt discoveries from opponent `UnitSpawned` events at result processing; do not add a
  second mutable collection to `BattleCore` or `ActiveBattleSession`.
- Validate imported profile IDs against current strict catalogs and merge by union only. Do not
  expose a replacement load method or compatibility aliases.
- Commit the infallible union only after a consuming completion/RunFailed transition succeeds;
  retry and retreat paths never call it.
- Project resolved current catalog metadata in one root `encounter_compendium` DTO and add one strict
  Unity read model. No standalone compendium command or Unity-local discovery calculation.

## Closed Result

- Completion review: Complete / High; no immediate correction.
- The profile stores only strict sorted ID sets in `GameCoreState`, outside RunState, checkpoint, and
  battle-entry rollback. Core exposes validate-and-union import/export for the server profile seam.
- Server lifetime persistence is monotonic across player-actor recreation. The current server has no
  durable player database; the serializable Core profile is the explicit boundary for any future
  process-durable adapter and does not justify inventing one in this goal.
- Unity reads only the required root projection, rejects missing/unknown fields and duplicate IDs,
  and never derives discovery from preview or battle events.
- G21 may begin. Preserve this global profile boundary while moving research delivery to the
  authoritative planning-return seam.
