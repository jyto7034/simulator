# DefenseRoute 전술 지휘 Phase 9 — Experiment Notes

Status: complete

## Baseline

- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Predecessor: Phase 8 `Complete / High`
- Existing user dirty is preserved; `game_server/docs/unity_bridge_protocol.md` remains untouched.
- Unity runtime/contract/tests are not inspected or changed.

## Policy extraction

- `DR-17` and the frozen design evidence agree on a narrow first-consumer-only object contract.
- Assignment is implicit final-occupancy reconciliation; there are no assign/cancel commands or work costs.
- Progress is object-owned Preserve-only battle time; Hard CC/Pause have no stored catch-up state.
- Only assignment start/stop and completion are semantic events; checkpoint owns current progress and mapping.
- First content is exactly one `suppress_freischutz` emergency repair object at `(2, 6)`, 10 seconds, protected HP +100.

## Discovery

- Immutable definitions belong to `PveTacticalPlanData` → `TacticalPlan`, not tactical points or static objects.
- BattleCore event time and generation-tagged scheduled events provide the exact settlement/stale-event seam.
- Deploy/withdraw/death and both tactical position commands already have centralized final commit points suitable for one
  field-work reconcile.
- Setup/checkpoint live DTOs are produced in World state; game_server is a direct current-shape adapter.

## Implementation evidence

- `field_work.rs` owns runtime status, progress, assignment, contributor count, generations and completion.
- `UnitExclusiveActivity` is an actual tactical-state field. FieldWorking forces Defense, suppresses attack/skill/block/focus/rest,
  while relocation/swap/withdraw remain eligible.
- Progress uses battle-time delta × current eligible workers. Hard CC only changes derived contributor count; Pause never invokes
  simulation advance. Neither path stores missed work.
- Completion changes status before applying the repair and increments generation, so full-HP and stale scheduled paths cannot
  repeat the effect/event.
- Freischutz setup/checkpoint/recovery and server JSON carry stable object/cell/worker identity and exact typed effect.

## Validation results

- `cargo test -p game_core field_work -- --nocapture` — 7 focused tests passed.
- `cargo test -p game_core --quiet` — 910 passed total after final focused additions
  (library 862 + integration 3 + 4 + 24 + 13 + 4).
- `cargo test -p game_server --quiet` — 56 passed.
- `cargo test -p game_core --test ron_loading --quiet` — 24 passed.
- `cargo check --workspace --all-targets` — pass, warning 없음.
- `cargo fmt --all -- --check` — pass.
- `git diff --check` — pass.
- Production legacy/placeholder and exact-one-live-object searches — pass.
- Failed/ignored — 0.

## Audit notes

- Setup version is 2 and event-log version is 31; there is no durable active-battle save migration.
- No assign/cancel command, Reset mode, additional live object, compatibility reader or generic framework exists.
- Game rulebook, runtime contract and data-loading contract match the implementation. The only accessible server mapping is direct
  DTO transport; existing dirty `game_server/docs/unity_bridge_protocol.md` was deliberately preserved untouched.
- Unity runtime/contract/UI/tests remained uninspected and unchanged as explicitly out of scope.
- Phase 9 has no known correctness gap; final DR-01..DR-17 traceability remains the parent Master audit responsibility.
