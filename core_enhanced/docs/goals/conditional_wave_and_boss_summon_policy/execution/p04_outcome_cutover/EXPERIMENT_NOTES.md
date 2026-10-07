# P04 Outcome Cutover Current Notes

## Current Summary

- Current pass: `GEX-20 complete / Audit Pass 1`
- Dependency: `P03 Complete / High / Audit Pass 1, current`
- Policy question: `none`
- Blocker: `none`
- Completion: `Complete / High`
- Baseline: inherited initial full baseline; no phase-start rerun
- Dirty attribution: current terminal/result/data files are pre-existing current actual; only post-intake deltas are
  attributable to P04

## Preserved Boundaries

- Typed Wave, Boss and summon producers expose neutral facts/signals; they do not choose a verdict.
- The whole same-time bucket settles before outcome evaluation. Defeat outranks Victory, then priority descending and
  stable outcome ID ascending choose one candidate.
- No global battle-duration cap or implicit Draw/Defeat remains. With no authored terminal match the battle continues.
- Retreat is separate and non-consuming. Post-battle run-failure evaluation is not a terminal condition.
- Node outcome policy is exhaustive and exactly one per outcome. Defeat has no positive reward/progression.
- SavePoint remains battle-free; same-process reconnect retains the already committed result rather than reselecting.
- P05 alone owns the compendium profile cutover and final Victory-only registration.

## Initial Technical Direction Within Approved Policy

- Introduce stable typed outcome ID/verdict/condition definitions in the battle scenario and compile them strictly from
  PVE live data.
- Replace winner computation with a deterministic outcome evaluator called only at completed bucket boundaries.
- Keep `BattleWinner` as a compatibility projection derived from the committed verdict while making outcome ID the
  authoritative result identity; remove Draw from authored terminal production.
- Move encounter and fixed-node rewards into outcome-specific policies, resolve the effective exhaustive policy before
  battle start, and carry only the selected policy into result consumption.
- Reuse the existing `with_state_transaction` result completion boundary for atomic/idempotent reward, progression and
  BossOmen effects rather than creating a second settlement store.

## GEX-10 Decisions and Findings

- Outcome ID is authoritative; verdict is a closed Victory/Defeat classification and winner remains only its derived
  compatibility projection. No authored or runtime Draw producer remains.
- Empty-player predeployment is not Defeat. `AllPlayerCombatantsDefeated` means an existing player combatant set whose
  members are all `Dead`; `Withdrawn` is deliberately not defeated because Retreat/redeployment remain separate paths.
- Encounter outcome policies are the default authority and fixed map-node policies are exhaustive per-outcome overrides.
  Runtime never merges a flat reward list after selecting an outcome.
- Outcome policy is resolved before battle and selected by the latched ID after finalization. A missing/mismatched policy
  is static-data failure, not fallback or reselection.
- `serde(default)` permits typed parsing to produce a precise publication error for missing outcome collections; strict
  validation still requires nonempty outcomes, a Victory and exact policy coverage before live publication.
- Existing world transaction staging is sufficient for atomic reward/progression/BossOmen consumption. No new receipt or
  settlement ledger was introduced.
- P05 still owns only compendium profile replacement and final Victory-only registration. P04 temporarily preserves the
  current compendium structure while switching its success input from winner to verdict.

## GEX-20 Result

- Audit Pass 1: `Complete / High`.
- Correction Goal: `none`.
- No P04 stop condition is active. P05 may begin against the frozen hashes in `EXPERIMENTS.md`.
