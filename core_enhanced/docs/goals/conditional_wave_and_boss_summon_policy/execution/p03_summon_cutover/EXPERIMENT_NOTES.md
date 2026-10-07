# P03 Summon Cutover Current Notes

## Current Summary

- Current pass: `GEX-20 complete / Audit Pass 1`
- Implementation: `complete at audited target`
- Dependency: `P02 Complete / High / Audit Pass 1, current`
- Policy question: `none`
- Blocker: `none`
- Completion: `Complete / High`
- Baseline: inherited initial full baseline; no phase-start rerun
- Dirty attribution: current Boss/Wave/data files are pre-existing current actual; only post-intake deltas are attributable
  to P03

## Preserved Boundaries

- Summoned units are a separate source/provenance domain, not employees, abnormalities, corroded employees or Boss actors.
- Definitions supply base combat values; encounter scaling is applied exactly once and summoner stats/buffs are never read.
- Summon graphs cannot recursively summon. Summons have no encounter phase, Wave, outcome, reward or compendium authority.
- Failed or canceled requests create no batch/UUID and consume neither `max_alive` nor `max_total`.
- `Persist` applies only to an earlier committed batch. `Despawn` and unresolved request cancellation settle with summoner
  removal/state exit before same-time allocation.
- `AllowOccupied` is a closed nonempty value set; every occupant must be allowed, `Any` is exclusive, and
  `Summoner + SameFaction` is redundant/invalid.
- Map isolation is an accepted content-authoring risk; runtime does not infer reachability or choose an implicit fallback.
- SavePoint does not contain battle/summon state; same-process reconnect retains the paused active battle only.

## Initial Technical Direction Within Approved Policy

- Keep summon definitions in the compiled Boss Runtime data boundary while giving them an explicit independent definition
  ID and source identity; physical catalog location does not grant Boss encounter authority.
- Keep counters, batches, scoped handles, signal latches and waiter state process-local under the active battle runtime.
- Add one summon-specific atomic commit primitive that prepares independent drafts and delegates cell commitment to the
  P01 Kernel transaction; do not route through legacy Wave group insertion.
- Reuse the per-actor behavior state machine through an authority tag/adapter path; encounter actor-slot and phase loops
  must exclude summoned actors by construction.

## GEX-10 Decisions and Findings

- The independent summon catalog is a separate live RON source but is compiled into the Boss Runtime database because
  Boss tasks are its only approved producer. Physical ownership does not grant encounter or Boss classification.
- Same-time removal priority is implemented structurally: batch reconciliation runs before waiter/request settlement;
  an already committed batch is therefore observable, while an unresolved request sees inactive summoner/state and is
  canceled before allocation. No extra timestamp marker is required.
- Despawn reuses the established nondeath inactive cleanup for targeting, movement, buffs and casts, then records the
  distinct `Despawned` lifecycle. The P03 acceptance is death/on-kill/reward absence; a new Unity presentation event is
  outside this Phase and was not invented.
- Signal publication is encounter-scoped, not merely catalog-global. A declared producer that is absent from every
  bound encounter behavior is rejected, and each consuming encounter must have a bound producer.
- Signal subscriptions capture the current epoch only after successful batch commit. Earlier epochs are not
  retroactive; the next epoch removes exactly that batch. Neutral signal IDs are latched in the active Boss runtime.
- Summon actor behavior instances share the existing scheduler implementation but are appended after
  `encounter_actor_count`. Phase authority, actor-slot adapters and encounter transitions iterate only the authored
  encounter prefix.
- Around placement intentionally uses current continuous summoner coordinates against tile centers, squared Euclidean
  distance, then `y/x`; SpawnZone preserves authored cell order. Neither path checks reachability.
- Counter total and deterministic UUID continuation advance only after Kernel success. Rollback restores the whole
  staged BattleCore for bind/commit failure; typed preflight failure creates no batch UUID.
- The live summon catalog stays empty. Representative content remains a focused fixture until P06's explicitly excluded
  placeholder; this is not production promotion.

## Validation Interpretation

- New tests changed the focused seams, so their passing final counts are the P03 rebaseline until another relevant test,
  schema, loader or runtime input changes. The run-wide full baseline remains the P01 1,152-pass baseline and is reserved
  for the required final P06 rerun.
- The two initial survival-fixture failures were baseline failures caused by P02's stricter Wave graph contract, not new
  P03 regressions. Both were corrected only in test authoring and then rerun to green.
- No save/checkpoint serializer contains `SummonRuntimeState`; process-local reconnect keeps the existing BattleCore,
  while process restart still falls back to the last SavePoint under the P01 contract.
- P03 does not bump or document an external Unity event contract. `SummonedUnit` source identity is opt-in and current
  live definitions are empty; final canonical/external alignment remains owned by P06.

## Completion

- Fresh `COMPLETION_AUDIT.md / Audit Pass 1` maps all assigned SM/BR acceptance rows to current code/data/tests.
- Verdict: `Complete / High`; correction required: none.
- P04 is dependency-eligible against the exact frozen P03 target.
