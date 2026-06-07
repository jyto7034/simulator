# Skill Fragment Compatibility Experiments

## 2026-06-07 - Current-State Read

Commands/read scope:

- `docs/core_policy_implementation_master_goal.md`
- `docs/skill_fragment_compatibility_goal.md`
- `docs/goals/core_policy_implementation_master/PLAN.md`
- `src/game/data/skill_fragment_data.rs`
- `src/game/skill_fragment.rs`
- `src/game/employee.rs`
- `src/game/world/maintenance.rs`
- `src/game/world/helpers.rs`
- `src/game/world/snapshot.rs`
- `src/game/combat_player_spawns.rs`
- `src/game/data/equipment_data.rs`
- `../game_resources/data/skill_fragments/base.ron`

Result:

- No code changes performed yet.
- No test commands run yet.
- Implementation is gated by active-fragment/no-weapon and weapon-change invalidation policy decisions.

## 2026-06-07 - Compatibility Schema Foundation

Changes:

- Added compatibility requirements to skill fragment metadata.
- Added a pure evaluator and stable failure codes.
- Updated test fixtures to explicitly use default compatibility requirements.
- Did not wire requirements into equip commands or weapon equip validation because active-fragment/no-weapon and weapon replacement invalidation are still policy gates.

Verification:

- `cargo test -p game_core skill_fragment_data -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.

## 2026-06-07 - Runtime Compatibility Wiring

Policy decisions applied:

- Active skill fragment equip requires the employee to have an effective weapon profile.
- Equipment combination/weapon replacement that invalidates the current active fragment is rejected.
- Snapshot and command failure use the same stable `SkillFragmentCompatibilityFailureCode` values.

Changes:

- Added `SkillFragmentMetadata::compatibility_report`.
- Added `GameError::SkillFragmentIncompatible`.
- Added server error code `skill_fragment_incompatible`.
- `validate_equip_skill_fragment_payload` now rejects incompatible active fragments.
- Equipment equip/combination projects the resulting item slot and rejects changes that invalidate the active fragment.
- Roster snapshot now exposes `skill_fragments.compatibility[*]`.
- Inventory skill fragment snapshot now exposes `requirements`.
- Live skill fragment RON now includes representative requirements for Scorched Spark, Red Shoes, Freischutz, Funeral Butterfly, and Helper fragments.
- Canonical Unity docs now document requirements, compatibility snapshot entries, failure codes, and `skill_fragment_incompatible`.

Verification:

- `cargo test -p game_core skill_fragment_equip -- --nocapture`: passed.
- `cargo test -p game_core equipment_combination_rejects_result_that_invalidates_active_fragment -- --nocapture`: passed.
- `cargo test -p game_core skill_fragment_actions_equip_and_unequip_employee_loadout -- --nocapture`: passed.
- `cargo test -p game_core skill_fragment_dismantle_rejects_equipped_or_last_copy -- --nocapture`: passed.
- `cargo test -p game_core manual_fragment -- --nocapture`: passed.
- `cargo test -p game_core activate_skill_uses_equipped_manual_fragment_in_live_defense -- --nocapture`: passed.
- `cargo test -p game_core skill_fragment_data -- --nocapture`: passed.
- `cargo test -p game_core skill_fragment -- --nocapture`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.

## 2026-06-07 - Effective Profile Snapshot Alignment

Changes:

- Added `effective_combat_profile_for_employee` in the player spawn/profile path.
- Kept `battle_unit_draft_for_employee` requiring combat availability for actual deployment.
- Let snapshot/preview effective profile calculation apply equipped weapon profiles without hiding resting or unavailable employees only because they cannot deploy.
- Updated roster snapshot generation to use the shared effective profile helper.
- Added a focused assertion that an equipped weapon appears in roster snapshot `combat_profile.effective_weapon_profile`.

Verification:

- `cargo test -p game_core equip_item_targets_employee_loadout_after_roster_initialization -- --nocapture`: passed.
- `cargo test -p game_core skill_fragment_data -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.

## 2026-06-07 - Compatibility Static Validation

Changes:

- Added static validation for skill fragment compatibility requirements.
- Rejected duplicate requirement axes.
- Rejected `block_capacity_min = 0`.
- Rejected empty capability tags, duplicate tags, and tags that are both required and incompatible.
- Connected validation to `SkillFragmentDatabase::validate_indexes`.

Verification:

- `cargo test -p game_core skill_fragment_data -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
