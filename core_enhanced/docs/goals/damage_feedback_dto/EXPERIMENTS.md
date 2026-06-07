# Damage Feedback DTO Experiments

## 2026-06-07

- Inspected `src/game/battle/damage.rs`.
  - `calculate_damage` exposes `raw_damage`, `final_damage`, `damage_type`, `critical`, and `breakdown`.
  - Non-basic damage can produce zero final damage when `minimum_damage` is zero.
- Inspected `src/game/battle/core/commands.rs`.
  - Basic attacks and skill/command damage both pass through `DamageResult` and `apply_damage_result_and_record`.
  - Healing/non-damage HP changes use `apply_hp_delta_and_record`.
- Inspected external canonical `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
  - The documented JSON already includes `feedback_tags`.
- Implemented `DamageFeedbackTag`, `DamageResult.feedback_tags`, and `TimelineEvent::HpChanged.feedback_tags`.
- Added focused tests:
  - `feedback_tags_include_critical`
  - `feedback_tags_include_mitigated_when_final_damage_is_sixty_percent_or_less`
  - `feedback_tags_include_fixed_damage_for_true_damage`
  - `feedback_tags_include_immune_when_raw_damage_is_fully_prevented`
  - `feedback_tags_are_empty_for_plain_damage`
  - `hp_changed_serializes_feedback_tags_as_snake_case_array`
  - `apply_damage_command_records_feedback_tags_on_hp_changed`
- Extended existing battle-command tests to verify basic-attack and non-damage HP changes carry stable empty tag arrays.

## Verification

- `cargo test -p game_core damage_feedback -- --nocapture`
  - Result: command succeeded but ran 0 tests because the filter did not match the new test names.
- `cargo test -p game_core feedback_tags -- --nocapture`
  - Result: passed, 7 tests.
- `cargo test -p game_core advance_basic_attack_projectile_is_idempotent_for_same_projectile_id -- --nocapture`
  - Result: passed, 1 test.
- `cargo test -p game_core apply_hp_delta_records_died_stop_with_latest_continuous_position -- --nocapture`
  - Result: passed, 1 test.
- `cargo test -p game_core damage -- --nocapture`
  - Result: passed, 27 tests. Expected panic output appeared from validation tests that assert panics.
- `cargo check -p game_core`
  - Result: passed.
- `cargo test -p game_core battle -- --nocapture`
  - Result: passed, 181 lib tests plus matching integration tests. Expected panic output appeared from a `should_panic` validation test.
- `rg -n "HpChanged|feedback_tags|TimelineEvent" ../game_server/src -g '*.rs'`
  - Result: no direct server references found.
