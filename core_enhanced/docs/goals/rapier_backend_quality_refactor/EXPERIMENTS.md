# Rapier Backend Quality Refactor Experiments

## 2026-06-07

- `cargo test -p game_core rapier -- --nocapture`: passed, 17 passed.
- `cargo test -p game_core movement_backends -- --nocapture`: failed once because `movement_backends_match_board_clamp_policy` used exact equality against `0.35` while Rapier returned `0.35000002`.
- `cargo test -p game_core movement_backends -- --nocapture`: passed after changing the board clamp assertion to use tolerance, 4 passed.
- `cargo test -p game_core movement -- --nocapture`: passed, 55 passed.
- `cargo test -p game_core blocking -- --nocapture`: passed, 2 passed.
- `cargo test -p game_core airborne -- --nocapture`: passed, 5 passed.
- `cargo test -p game_core fixed_defense -- --nocapture`: passed, 17 passed.
- `cargo check -p game_core`: passed.

## Verification Candidates

```text
cargo test -p game_core movement -- --nocapture
cargo test -p game_core blocking -- --nocapture
cargo test -p game_core airborne -- --nocapture
cargo test -p game_core fixed_defense -- --nocapture
cargo check -p game_core
```
