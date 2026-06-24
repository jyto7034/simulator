# Data Validation Refactor Plan

## Objective

Move `src/game/data/mod.rs` cross-reference validation into `src/game/data/validation.rs` and remove the empty `src/game/battle/replay/` directory.

## Current Scope

In scope:

- Re-read current `data/mod.rs`; do not trust stale line numbers in `component_design_review.md`.
- Move cross-reference validation helpers to a dedicated validation module.
- Keep validation behavior and error messages equivalent unless current code proves a better long-term structure.
- Remove the empty `battle/replay/` directory after confirming it has no files or references.
- Run focused checks for compile and live RON/data validation.

Out of scope:

- Changing validation policy.
- Changing live RON schema.
- Changing `GameDataBase` public behavior.
- Unity-facing DTO changes.
- Renaming timeline/event-log fields.

## Plan

1. Identify current validation functions in `src/game/data/mod.rs`.
2. Classify which functions are cross-reference validation versus local DB helpers.
3. Add `src/game/data/validation.rs` and move cross-reference validation there.
4. Wire `GameDataBase::new` to call validation module entry points.
5. Remove empty `src/game/battle/replay/`.
6. Run focused tests:
   - `cargo check -p game_core`
   - `cargo test -p game_core --test ron_loading`
   - `cargo test -p game_core --test live_skill_catalog_audit`

## Completion Conditions

- `data/mod.rs` no longer owns cross-reference validation helper implementations.
- `data/validation.rs` owns cross-reference validation and is called from `GameDataBase::new`.
- Validation order and failure semantics are preserved.
- Empty `src/game/battle/replay/` directory is gone.
- Focused verification commands pass.
- No compatibility layer, fallback path, or dual validation path is introduced.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Evidence

- `src/game/data/mod.rs` declares `mod validation;` and delegates cross-reference validation to `validation::validate_data_references`.
- `src/game/data/validation.rs` owns skill fragment, unit skill, reward, PvE enemy, shop item, and generated combat preview validation helpers.
- `src/game/battle/replay/` was confirmed empty and removed.
- Verification passed:
  - `cargo check -p game_core`
  - `cargo test -p game_core --test ron_loading`
  - `cargo test -p game_core --test live_skill_catalog_audit`
  - `cargo fmt`
  - `cargo check -p game_core` after formatting

## Stop Conditions

- Moving validation requires changing live RON schema.
- Moving validation requires changing validation policy.
- Moving validation requires changing public GameDataBase or Unity-facing behavior.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
