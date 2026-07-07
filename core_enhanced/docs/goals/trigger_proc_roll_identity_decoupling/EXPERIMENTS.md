# Trigger Proc Roll Identity Decoupling Experiments

Use this file to record implementation attempts, failures, fixes, and validation results.

## Trial Log

- 2026-07-06: Pre-goal code read found that proc rolls are not directly coupled to `event_log_seq`. The active helper in `src/game/battle/core/sim.rs` uses `battle seed + source + ability_id + binding_index + current_time_ms + trigger_count`.
- 2026-07-06: Identified the main design issue: `trigger_count` is successful proc state, not proc candidate identity. Failed proc candidates do not advance it, so simultaneous candidates can become hard to reason about.
- 2026-07-06: Identified likely implementation boundary: `activation_commands_from_bindings(...)` currently erases richer trigger occurrence context when creating `BattleCommand::TriggerAbility`.
- 2026-07-06: Added `ProcRollIdentity`, carried it through `BattleCommand::TriggerAbility`, and introduced `TriggerAbilityContext` for activation command conversion.
- 2026-07-06: Updated instant basic attack, basic attack projectile impact, death-trigger, and battle-start trigger activation call sites to provide stable occurrence context.
- 2026-07-06: Replaced proc roll seeding with `ProcRollIdentity` and left `AbilityProcState.trigger_count` as successful proc state for cooldown/max-trigger behavior.
- 2026-07-06: Removed duplicated source/ability/binding fields from `TriggeredAbilityProcContext` and now derives proc state keys from the carried `ProcRollIdentity`.
- 2026-07-06: Added focused tests for event-log independence, occurrence identity, command boundary identity preservation, and max trigger preservation.

## Validation Log

- 2026-07-06: `cargo check -p game_core` passed after helper name cleanup.
- 2026-07-06: `cargo test -p game_core proc_roll --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core item_activation_proc --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core trigger --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core projectile --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core death --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core game::battle --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo check -p game_core` passed.
- 2026-07-06: `cargo fmt --check` passed.
- 2026-07-06: `rg -n "trigger_count" src/game/battle/core/sim.rs src/game/battle/core/commands.rs src/game/battle/damage.rs -g '*.rs'` shows `trigger_count` only in proc state read/increment paths.

## Failed Attempts

- 2026-07-06: First `cargo check -p game_core` failed because new helper names duplicated existing inherent `BattleCore` helper names from other modules. Fixed by renaming the new helpers to proc-specific names.
- 2026-07-06: Initial test insertion patch missed the exact context around the existing delayed damage identity test. Re-read the file location and inserted the tests at the correct boundary.
- 2026-07-06: A chained final validation command failed once while opening the shared Cargo target lock after earlier parallel cargo runs. Re-ran the relevant validation commands separately and they passed.
