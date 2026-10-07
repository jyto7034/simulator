# Boss Runtime v1 — P01 G0 Implementation Handoff

Handoff Date: 2026-08-09  
Source Phase: P00 execution preflight  
Target Phase: P01 battle lifecycle correctness  
Verdict: `ready_for_g0`  
Recommended Child Shape: one child Goal, three sequential review milestones

## Why one child owns all three corrections

C01, C02 and C03 share the live execution state, event queue semantics, world advance orchestration and
`world/tests/combat.rs`. The current versions of those owners are also user-dirty. Three concurrently active children
would create ambiguous ownership and could stage C03 around terminal/order semantics that are about to change.

P01 therefore owns one atomic correctness Phase with review stops after C01, C02 and C03. It must patch only named
symbols/hunks on top of the current dirty baseline. It must not add Boss schema, Boss runtime, live RON, public Wire
or Unity behavior.

## Milestone 1 — CRA-P06-C01 terminal authority

### Mutation owners

- `core_enhanced/src/game/battle/core/sim.rs`
  - `BattleExecutionState`
  - `BattleStepOutcome` / state-owned terminal outcome
  - `finish_signal`
  - direct, until and by stepping
  - `finalize_battle_execution`
- `core_enhanced/src/game/battle/core/mod.rs` test module
- `core_enhanced/src/game/world/tests/combat.rs` only for production-world terminal acceptance

### Required behavior

- The authoritative execution clock never advances beyond `max_battle_time_ms`.
- Direct, until-time and elapsed-time stepping expose the same terminal result at the cap.
- A win condition satisfied at or before the cap wins; otherwise the established result is Draw.
- Events strictly after the cap are not processed.
- A state-owned terminal outcome is produced once; finalization consumes that exact outcome once and records exactly
  one `BattleEnd` at the terminal time.
- Re-step after terminal and second finalization remain rejected.
- Same-time bucket draining and AttackResolve-before-final-BattleEnd logging remain unchanged.

### Tests to add before implementation

- `direct_until_and_by_steps_share_terminal_outcome_at_runtime_cap`
- `exact_cap_satisfied_condition_wins_and_unsatisfied_condition_draws`
- `terminal_state_clamps_clock_skips_late_event_and_finalizes_once`
- `live_world_tick_reaches_combat_result_at_runtime_cap_without_revision_churn`

## Milestone 2 — CRA-P06-C02 total deterministic event order

### Mutation owners

- `core_enhanced/src/game/battle/enums.rs`
  - `BattleEvent::priority`
  - one explicit stable variant discriminator
  - one total semantic tie-break path
  - `Ord` / `PartialOrd`
- `core_enhanced/src/game/battle/core/mod.rs` only for simultaneous cast completion and repeated-result integration
  tests

### Narrow semantic decision

Keep every current priority band unchanged. Within the shared priority-8 band, `AutoCastEnd` precedes
`ManualCastEnd`, matching current declaration/match order and making the previously incidental relationship explicit.
All other cross-variant ties use the explicit stable variant discriminator. Pointer identity, insertion order, hash
iteration and RNG are prohibited ordering inputs.

The comparator must first preserve `Eq`: equal values compare `Equal`. For unequal values it must compare time,
priority, variant discriminator and stable live identifiers/generation/cast/step fields, followed by a deterministic
complete-payload fallback. The fallback must normalize equality-equivalent floating values and must not be a hash
whose collision can produce `Equal`.

### Tests to add before implementation

- `battle_event_cmp_equal_iff_eq_for_every_live_variant`
- `battle_event_heap_is_identical_for_all_relevant_insert_permutations`
- `simultaneous_manual_cast_end_is_stable_across_insertion_orders`
- `simultaneous_auto_and_manual_cast_end_uses_documented_variant_order`
- retain `same_timestamp_attack_resolve_is_drained_before_battle_end`

The table fixture must instantiate all 19 current variants and include unequal same-variant payload pairs, not merely
one instance per priority.

## Milestone 3 — CRA-P06-C03 atomic world advance

### Mutation owners

- `core_enhanced/src/game/world/combat.rs`
  - one internal staged advance pipeline
  - `active_battle_advanced_result`
  - elapsed-time and next-event-bucket adapters
  - finished-result publish ordering
- `core_enhanced/src/game/world/state.rs`
  - cursor/revision prevalidation and the minimum scoped transaction snapshot/staging seam
- `core_enhanced/src/game/world/visual_novel_triggers.rs` only if trigger evaluation must target staged state
- `core_enhanced/src/game/battle/core/mod.rs` only for the minimum clone/snapshot seam proven necessary
- `core_enhanced/src/game/world/tests/combat.rs`

### Required transaction order

1. Validate active authority, future cursor and revision capacity against live state before mutation.
2. Create a scoped staged active-battle/VN/UUID state or an exact rollback snapshot. Do not clone unrelated durable
   run state unless evidence proves it is mutated by this operation.
3. Run Core step/finalization on staged state.
4. Evaluate BattleStarted/BattleTimeReached triggers and reconcile employee/tactical state on staged state.
5. Build the exact DTO and optional completed battle from staged state.
6. Publish the staged active/VN/UUID state and exactly one revision, then perform the infallible state transition.

Both `advance_active_battle_by_internal` and `advance_active_battle_to_next_event_bucket` must call the same pipeline;
their pause-aware/raw-delta entry semantics remain distinct outside it.

### Concrete failure injection

- Cursor: call mutating `advance_active_battle_by_after_seq` with `after_seq = latest_seq + 1` both before ordinary
  events and on a finishing advance.
- Revision: set `battle_state_revision = u64::MAX` before elapsed and bucket advance.
- VN trigger: install a `BattleTimeReached` trigger whose referenced VN is missing before battle construction, then
  cross its threshold during advance.
- Reconcile: construct an unprocessed incapacitated deployed employee whose attempt roster entry is absent, then
  advance through reconciliation.
- Projection/DTO: cursor and revision injections cover the two current fallible projection boundaries. Do not add an
  artificial production failure mode only for testing.

Every failure assertion must compare pre/post battle clock, event log/seq, event queue observable snapshot,
execution terminal/finalized state, employee attempt/live deployment/tactical resource, VN active/queue/consumed
sets, UUID continuation, battle revision, `GameState`, active content and active-battle presence.

### Tests to add before implementation

- `mutating_advance_rejects_future_cursor_without_any_state_change`
- `finishing_advance_rejects_future_cursor_without_consuming_terminal_state`
- `elapsed_and_bucket_advance_reject_revision_overflow_without_any_state_change`
- `advance_rolls_back_core_and_world_when_visual_novel_trigger_fails`
- `advance_rolls_back_core_and_world_when_employee_reconcile_fails`
- `elapsed_and_bucket_advance_publish_equivalent_success_state`

## Exact validation commands

Characterization commands use fully qualified paths; the unqualified `--exact` form was proven to match zero tests.

```text
cargo test -p game_core --lib game::battle::core::tests::<exact_test_name> -- --exact --nocapture
cargo test -p game_core --lib game::battle::enums::tests::<exact_test_name> -- --exact --nocapture
cargo test -p game_core --lib game::world::tests::combat::<exact_test_name> -- --exact --nocapture
```

Milestone and Phase gates:

```text
cargo test -p game_core --lib game::battle::core::tests -- --nocapture
cargo test -p game_core --lib game::battle::enums::tests -- --nocapture
cargo test -p game_core --lib game::world::tests::combat -- --nocapture
cargo test -p game_core --lib
cargo test -p game_server
cargo test --workspace --all-targets
```

Record actual pass/fail/ignored/filtered counts. A zero-match result is a failed validation. Run formatting only on
P01-touched Rust files or use `cargo fmt --all -- --check` at exit; never bulk-rewrite unrelated dirty files.

## Baseline evidence

On 2026-08-09 the fully-qualified representative C01, C02 and read-only cursor tests each ran once and passed
(`3 passed`, `0 failed`, `0 ignored` in aggregate). Those tests do not characterize the defects above; they only prove
the package, module paths and focused runner shape are viable.

## Entry gate

P01 may activate when P00 is `Complete / High` and the Master registry marks P00 complete/P01 ready. The current dirty
tree is accepted as the baseline; no concurrent active production Goal owns these symbols. Re-run the scoped status
and active-Goal query immediately before creating P01.

## Exit gate

- All listed behavior and no-mutation assertions pass.
- Existing pause, stale-authority, revision, cursor, same-time attack/end and finished-result regressions pass.
- Smallest sufficient broad Core/server/workspace validation passes with exact counts.
- Canonical documents the corrected runtime only after the code is implemented.
- P01 completion audit is `Complete / High`; only then may P02 activate.
