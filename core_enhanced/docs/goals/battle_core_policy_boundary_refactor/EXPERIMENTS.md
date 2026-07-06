# Experiments

| Date | Trial | Result | Notes |
| --- | --- | --- | --- |
| 2026-07-06 | Goal document creation from audit review. | Planned. | Covers P-008: large battle core files mix multiple abstraction levels. |
| 2026-07-06 | Boundary map before edits. | Success. | Recorded current responsibilities, target modules, and extraction risks in `EXPERIMENT_NOTES.md` before code changes. |
| 2026-07-06 | Extract shared projectile math and basic attack projectile runtime. | Success after import fix. | Moved flight-time math to `projectile_math.rs` and basic attack projectile launch/advance/hit/miss to `basic_attack_projectile.rs`; initial `projectile` test failed only on stale imports, then passed. |
| 2026-07-06 | Extract death finalization runtime. | Success. | Moved death-trigger commands, unit death finalization, dead-unit buff cleanup, graveyard capture, and movement invalidation to `death_runtime.rs`; `death` filter passed. |
| 2026-07-06 | Extract damage runtime. | Success. | Moved damage-source snapshots, crit roll materialization, basic attack damage snapshot calculation, damage result recording, and HP delta recording to `damage_runtime.rs`; `damage` filter passed. |
| 2026-07-06 | Move `ActiveMovementSegment` to movement-owned types. | Success. | Moved movement segment sampling state from `mod.rs` to `movement/types.rs`; `movement` and `game::battle` filters passed after the move. |

## Validation Log

- `cargo test -p game_core projectile --lib -- --test-threads=1`
  - First run failed because tests still imported `projectile_flight_ms` from `commands.rs` and `commands.rs` still needed the `determinism` import for damage roll code.
  - After moving the import to `projectile_math` and restoring the remaining `determinism` import, rerun passed: 20 passed.
- `cargo test -p game_core damage --lib -- --test-threads=1`
  - Passed after damage runtime extraction: 35 passed.
- `cargo test -p game_core death --lib -- --test-threads=1`
  - Passed after death runtime extraction: 6 passed.
- `cargo test -p game_core movement --lib -- --test-threads=1`
  - Passed before and after `ActiveMovementSegment` move: 68 passed.
- `cargo test -p game_core skill --lib -- --test-threads=1`
  - Passed: 84 passed.
- `cargo test -p game_core game::battle --lib -- --test-threads=1`
  - Passed before and after `ActiveMovementSegment` move: 236 passed.
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
  - Passed: 39 passed.
- `cargo check -p game_core`
  - Passed.
- `cargo fmt --check`
  - Passed.

Known unrelated warning remains during test builds:

- `src/game/world/tests/mod.rs` imports `buffs::BuffDatabase` without using it.
