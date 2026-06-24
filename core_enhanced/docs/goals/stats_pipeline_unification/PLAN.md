# Stats Pipeline Unification Plan

## Objective

Unify the battle-entry employee stat pipeline so equipment, artifacts, skill fragments, growth stacks, equipment enhancement, and consumable modifiers have one explicit runtime entry point and an audited order.

## Current Scope

In scope:

- Re-read runtime code before relying on stale design notes.
- Identify all current battle-entry stat mutation paths.
- Preserve current user-visible behavior unless code inspection proves a better long-term structure.
- Add focused tests that lock the intended order and resulting battle profile behavior.
- Keep buff runtime separate from pre-battle consumable and loadout modifiers.

Out of scope:

- Live RON schema changes.
- Balance value changes.
- Unity-facing DTO shape changes.
- Save migration.
- Runtime buff system refactor.

## Plan

1. Inspect `src/game/battle/types.rs`, `src/game/battle/core/build.rs`, `src/game/employee.rs`, `src/game/skill_fragment.rs`, `src/game/stats.rs`, and focused tests.
2. Write down the current source-of-truth order in `EXPERIMENT_NOTES.md`.
3. Add or adjust focused tests for current battle-entry stats behavior.
4. Introduce a single stats pipeline entry point if the current code lacks one.
5. Route existing battle setup code through the pipeline.
6. Remove duplicated/legacy helper paths made obsolete by the pipeline.
7. Run focused tests and record failures/fixes in `EXPERIMENTS.md`.

## Completion Conditions

- There is a single clear battle-entry final stat calculation entry point.
- The modifier order is explicit in code and tested.
- Equipment/artifact, skill fragment, growth/enhancement, and consumable modifier behavior remains current-policy compliant.
- Buff runtime is not merged with pre-battle stats.
- Verification commands pass.
- No compatibility layer, fallback path, or dual stat pipeline is introduced.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Completion Evidence

- `src/game/battle/stat_pipeline.rs` owns employee battle profile modifiers and draft final stats calculation.
- `Employee::combat_profile_for_battle` delegates to `stat_pipeline::employee_combat_profile_for_battle`.
- `BattleUnitDraft::effective_stats` delegates to `stat_pipeline::effective_stats_for_draft`.
- Modifier order is documented in code comments and locked by order-sensitive tests.
- Verification passed:
  - `cargo test -p game_core battle::stat_pipeline::tests -- --nocapture`
  - `cargo test -p game_core employee::tests -- --nocapture`
  - `cargo test -p game_core battle::types::tests -- --nocapture`
  - `cargo test -p game_core world::tests::equipment -- --nocapture`
  - `cargo test -p game_core --test live_item_skill_activation`
  - `cargo fmt`
  - `cargo check -p game_core`

## Stop Conditions

- The correct modifier order requires a new design decision.
- A live RON schema or save migration becomes necessary.
- Unity-facing DTO shape must change.
- Balance values need to change.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
