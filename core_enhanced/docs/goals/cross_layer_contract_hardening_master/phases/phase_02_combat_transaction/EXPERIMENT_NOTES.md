# Phase 02 Combat Transaction And Boss Recovery Notes

Status: complete

## Completed Review

- R02 `cross_layer_phase_02_combat_completion_review`: Complete / High / no immediate correction.

## G26 Handoff

- Completion review: `../../../combat_employee_incapacitation_resolution/COMPLETION_REVIEW.md`.
- Attempt employee condition is already transactional; G15 composes full-world rollback around it.
- Do not reapply participant penalties at result time or restore fixed death HP behavior.
- Employee result rows and reward participant context consume aggregated result stats, not raw
  runtime participant instances.

## Source-Of-Truth Boundary

- BattleCore owns runtime-instance lifecycle, deterministic simulation facts, and replay/event
  history.
- The world layer owns attempt-local employee condition and the commit/discard transaction.
- G15 owns complete pre-entry restoration and result/failure transition ordering.
- G14 owns BossOmen Gate/recovery semantics; G08 owns Unity command and presentation consumption.

## G15 Handoff

- Completion review: `../../../battle_failure_abandonment_consumable_resolution/COMPLETION_REVIEW.md`.
- Every committed result keeps one attempt and one combat-duration decrement; abandonment exactly
  restores the pre-entry world snapshot.
- NoLiving/boss failure preempts retry. Nonboss NoDeployable deferral uses only immediate typed
  recovery nodes and blocks checkpoint load while that restriction is active.
- G14 may expose checkpoint load only from the committed BossOmen `RunFailed` state; it must not
  turn it back into a pre-failure retry/cancel branch.

## G14 Handoff

- Completion review: `../../../boss_omen_gate_and_rollback_policy/COMPLETION_REVIEW.md`.
- Forced BossOmen victory commits a same-Floor dedicated Gate; only ordinary Gate confirmation
  applies recovery and advances the Floor.
- Qualified forced-BossOmen RunFailed exposes exactly the existing checkpoint action when capacity
  is valid. Standard final boss and exhausted capacity expose no action.
- G08 consumes existing Core map/action/command shapes and must not infer boss/recovery eligibility.

## G08 Handoff

- Completion review: `../../../unity_recovery_and_roster_command_flows/COMPLETION_REVIEW.md`.
- Unity now has typed setup recovery, checkpoint load, and roster movement command/UI paths.
- Incomplete battle runtime preserves only revisioned query/recovery scopes until setup is rebuilt;
  gameplay does not fall back to root capability.
- Roster movement is projected/revalidated by Core in ViewingMap/NodeConfirm and consumes
  authoritative `roster_order` identities.

## Direct Predecessor Invariants

- Exact battle UUID/revision authority is checked before mutation.
- Core action capability and concrete payload validation remain authoritative.
- Missing/stale authority rejects without mutation and returns current resync.

## Open Cross-Child Risks

- G26 must expose a clean commit/discard seam without prematurely implementing G15's whole-world
  rollback table.
- Result aggregation must not collapse runtime replay history or turn XP/MVP into per-death effects.
- Later G15 rollback must include RNG, attempts, consumptions, node state, and all other gameplay
  state; an employee-only undo is not Phase-complete.

## Phase Closure

- The previously open risks are closed by the current combined trace and validation matrix.
- Phase 2 completion review: `COMPLETION_REVIEW.md`.
- Phase 3 may consume this boundary but must not redefine employee transaction, abandonment,
  BossOmen recovery, or battle authority semantics.
