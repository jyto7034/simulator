# Core Enhanced Refactoring Audit Phase 08 — Experiment Notes

Status: complete

## Boundary decision

- Primary authority: persistent authored inputs → runtime stat/effect state → damage/HP/resonance → death and world
  employee-attempt reconciliation.
- Phase 7 owns tactical transition timing; Phase 9 owns target/delivery; Phase 10 owns projections.
- Shared large files are read only at stat/effect/death mutation slices.

## Dirty precautions

- Current dirty tree is the baseline; production files may already be modified by the user.
- Baseline hashes/status must be captured before detailed Phase 8 reads.
- Only Master/Phase Goal artifacts are audit-owned.

## CRA-14 policy

- Formula tests count only when assertions observe authoritative inputs, intermediate contract or exact final state.
- Target/projectile fixtures count only for effect semantics actually asserted; delivery correctness remains Phase 9.
- `remove` requires no unique contract and an equal/stronger surviving safety net.
- Runner green counts do not close static gaps.

## Resolved authority notes

- Persistent-to-runtime stat order is explicit in `stat_pipeline.rs`; no competing builder was found, but arithmetic
  saturation and runtime max-HP mutation are incomplete boundaries.
- `calculate_damage` is the formula authority. Hit orchestration and trigger dispatch are duplicated around it, and
  the calculated `UnitDied` command is legacy-shaped no-op data.
- Buff/control/proc/resonance deadlines use battle time and reject stale expiry/tick identity. The confirmed trigger
  failure is occurrence coverage: battle-start effects are omitted while battle-start activations run.
- Core death runtime is authoritative; top-level `battle/death.rs` is a disconnected partial model.
- Published effect reference and compatibility validation is incomplete. OnHit damage-effect target meaning remains
  a policy question and is not treated as verified behavior.

## Handoff condition

Phase 9 stays inactive until Phase 8 has complete boundary/candidate/test artifacts, no-mutation proof and an
independent `Complete / High` audit.
