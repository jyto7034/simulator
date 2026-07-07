# Combat Roll Identity RNG Decoupling Experiments

Use this file to record implementation attempts, failures, fixes, and validation results.

## Trial Log

- 2026-07-06: Pre-goal code read found the active coupling in `src/game/battle/core/damage_runtime.rs`. `damage_roll_percent_with_event_log_seq(...)` uses `event_log_seq` as seed material and `DamageSourceSnapshot` stores `crit_roll_event_log_seq`.
- 2026-07-06: Introduced `CombatRollKind` / `CombatRollIdentity`, replaced `crit_roll_event_log_seq`, and changed damage crit roll seeding to use battle seed plus roll identity.
- 2026-07-06: Initial roll identity used deterministic source material but did not strongly distinguish repeated source instances. Hardened the implementation by adding `BattleCore.damage_source_seq` as a damage-source-instance id counter independent from event logs and RNG draw order.
- 2026-07-06: Added regression tests for event-log-independent crit rolls and delayed target/hit-index materialization.

## Validation Log

- 2026-07-06: `cargo test -p game_core damage_crit_roll_is_independent_from_event_log_sequence --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core delayed_damage_materialization_uses_hit_index_identity_not_event_log_sequence --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core crit --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core projectile --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo test -p game_core game::battle --lib -- --test-threads=1` passed.
- 2026-07-06: `cargo check -p game_core` passed.
- 2026-07-06: `cargo fmt --check` passed.
- 2026-07-06: `rg -n "crit_roll_event_log_seq|damage_roll_percent_with_event_log_seq" src tests -g '*.rs'` returned no matches.
- 2026-07-06: `rg -n "event_log_seq" src/game/battle/core/damage_runtime.rs src/game/battle/damage.rs -g '*.rs'` returned no matches.

## Failed Attempts

- 2026-07-06: Running multiple cargo commands in parallel caused temporary Cargo file-lock waits. No code issue; subsequent validation was run sequentially.
- 2026-07-06: After making source snapshot builders `&mut self`, two tests had nested mutable borrows because they built snapshots inline inside `push` / `process_commands` calls. Fixed by creating local `source_snapshot` variables first.
- 2026-07-06: The first event-log independence test compared two snapshots created sequentially in one core. After adding explicit source instance allocation, that correctly produced different source identities. Fixed the test to compare two identical cores where only one core has an extra event log entry before snapshot creation.
