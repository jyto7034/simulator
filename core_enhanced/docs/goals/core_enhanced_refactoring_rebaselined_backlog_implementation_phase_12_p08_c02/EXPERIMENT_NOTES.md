# Phase 12 CRA-P08-C02 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P08-C02, Master Phase 12 of 44.
- Direct predecessor Phase 11 is `Complete / High`; Phase 13 remains pending.
- Mutation authority is limited to runtime stat transition/provenance, scoped callers, exact tests and nearest
  canonical evidence.
- All three Child documents must be read fully before Phase 12 production/status/data inspection.

## Authority Notes

- UnitStats owns arithmetic only. It may compute a clamped before/after snapshot but does not own runtime events,
  lifecycle, graveyard, buff cleanup or kill credit.
- Existing Core HP/death finalization remains authoritative. Reuse its idempotent handoff rather than reproducing
  damage or death logic in the stat layer.
- Persistent battle-entry stat composition remains event-free. This Phase is only runtime mutation.
- Preserve dependent body state updates in the same committed transition.

## CRA-POL-001 Notes

- A runtime MaxHealth clamp to zero is lethal; Active zero HP is never terminal.
- Preserve optional causal unit through StatChanged and HpChanged. Existing source is killer; absent/system source
  means no killer and no OnKill credit.
- Observable order is StatChanged, then HpChanged only for actual current-HP delta, then normal death finalization.
- Exactly-once death and nested trigger safety are correctness requirements, not later cleanup.

## Compatibility Risk

The historical command may be public while causal source is currently lost. Do not assume that adding a public enum
field/variant is allowed. First inventory all constructors/matches and seek the smallest internal typed carrier or
compatible adapter. Stop if correctness would otherwise require an unapproved public/save/wire break or hidden
side-channel state.

## Dirty Worktree Risk

The workspace contains extensive user-owned BattleCore, world combat, item/data, server, test and canonical changes
from the campaign and prior work. Fingerprint each prospective target before editing. Do not reset, restore, clean,
stash or broadly format; stop if the required hunk cannot be separated.

## Testing Notes

- Decision tables must execute production command→event→death paths and assert exact state/events, not a private
  UnitStats helper alone.
- Include flat and percent, nonlethal and lethal, sourced and unsourced causes.
- Sourced lethal proof must observe killer/OnKill; unsourced lethal proof must observe no killer/OnKill.
- Assert graveyard, action/movement and buff cleanup plus one UnitDied. Add nested death-trigger effects to prove no
  duplicate finalization or post-death target mutation.
- Preserve ordinary Attack/movement and positive MaxHealth behavior, Phase 10 extremes, and event-free prebattle
  composition.
- Zero-test binaries and success-only assertions are not proof.

## Out-Of-Scope Follow-Up Handling

Keep equipment publication under Phase 13, committed-damage consolidation under P08-C05 and public DeathHandler
cleanup under P08-C06. Record independent findings without absorbing them.

## Completion Boundary

- Phase-exit audit verdict: `Complete / High`; improvement class `none`.
- Runtime MaxHealth mutation now preserves item/skill causal source, emits conditional HpChanged and enters the
  existing death/kill occurrence authority exactly once without changing public/RON/save/wire shape.
- Phase 13 owns effect/activation reference and target-matrix publication validation. Phase 28 owns committed-damage
  consolidation; Phase 38 owns disconnected public DeathHandler cleanup.
- No unresolved finding or policy question remains in Phase 12.
