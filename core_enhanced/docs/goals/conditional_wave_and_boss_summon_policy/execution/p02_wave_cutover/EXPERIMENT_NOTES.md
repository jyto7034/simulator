# P02 Wave Cutover Current Notes

## Current Summary

- Current pass: `GEX-20 complete / Audit Pass 1`
- Implementation: `typed Wave schema/live data/runtime/Kernel cutover implemented and focused current validation passed`
- Dependency: `P01 Complete / High, current`
- Policy question: `none`
- Blocker: `none`
- Completion: `Complete / High`
- Baseline: inherited initial full baseline; no phase-start rerun
- Dirty attribution: current Wave/Boss/data files are user-owned current actual matching the P2G review; only future
  post-intake deltas are attributable to P02

## Preserved Boundaries

- P02 must remove old live Wave activation rather than leave a fallback.
- Summon, named outcome and permanent compendium semantics remain later phases.
- Same-time policy uses simulation buckets, not wall-clock order.
- Required point trigger has a deadline; generated Wave and summon-batch vocabularies stay type-specific.
- P01 Kernel and process-local session semantics remain unchanged unless a directly invalidated seam is found.

## Technical Decisions Within Approved Policy

- Wave overlap uses Kernel `AllowOccupied([Any])` because approved `SP-02` explicitly permits fixed authored Wave cells to
  overlap existing occupants; occupancy is not reinterpreted as a summon policy.
- `RuntimeUnitLifecycle::Despawned` is introduced as an observation fact only in P02. The producer/removal command and
  summoned-unit cleanup side effects remain P03-owned.
- Wave activation cause is held in the active-battle trigger ledger rather than added to the Unity-facing battle event DTO;
  Unity projection remains P04/out of scope.
- Actor actual death is an epoch edge written by the authoritative death finalizer. A binding snapshots the current epoch,
  so pre-bind death and restoration cannot satisfy `ActorDefeated`, while a real post-bind death remains latched.
- The adapter reads Boss actor-slot binding only. Wave does not inspect encounter phase, control node, task or pattern state.
- `radius_cells` uses Chebyshev tile distance, matching the existing Boss grid stop predicate; it observes the actor's
  projected tile and never emits a movement command.
- `AtTimeMs` fixtures use the explicit constructor only for Rust test construction; RON remains fully explicit and has no
  compatibility/default reader.

## Corrected Existing Fixture

- `elite_survival_timer_encounter` reused a generic test Wave with `required_for_victory: true`. New validation correctly
  rejected it. The fixture now explicitly overrides the gate to `false`; production Clouded Monk survival rows received the
  same approved meaning during live migration.

## Final Current Focused Evidence

- Wave runtime: 10 passed / 0 failed.
- strict PVE schema and graph: 14 passed / 0 failed.
- current live PVE reference load: 1 passed / 0 failed.
- current `game_server` compile seam: passed.
- Prior focused evidence listed in `EXPERIMENTS.md` remains current because its production inputs were not subsequently
  changed.
