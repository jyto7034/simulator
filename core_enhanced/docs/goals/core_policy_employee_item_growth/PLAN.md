# Employee, Item, Growth Policy Implementation

## Objective

Implement long-term employee growth, loadout, fragment, equipment binding, and trauma policies while keeping the future trust system deferred.

## Policies Covered

- `remove unused growth ids`
- `trust feature surface`
- `growth curve and XP policy source`
- `remove legacy employee grade`
- `trauma applies to final battle max HP`
- `starter employee loadout source`
- `skill fragment equip limit policy`
- `skill fragment dismantle auto unequip`
- `bound equipment interaction lock`

## Plan

1. Read employee runtime state, growth data, starter employee setup, loadout/equipment code, and live employee/item/fragment RON.
2. Remove unused growth identifiers and legacy grade/star concepts.
3. Move growth curve, level, and XP policy to data-driven sources.
4. Preserve trust surfaces for future official trust work, but do not implement new trust policy in this goal.
5. Apply trauma to final battle max HP after equipment/level/temp HP modifiers are included.
6. Move starter employee equipment/fragments into live RON and remove code injection.
7. Enforce fragment equip copy accounting and `GlobalExclusive` or equivalent explicit high-tier exclusivity from data.
8. Allow fragment dismantle with deterministic auto-unequip and expose enough data for Unity warning UI.
9. Enforce bound equipment interaction locks until explicit unbind policy/event.
10. Update tests for growth loading, loadout invariants, fragment equip/dismantle, bound equipment, and battle HP initialization.

## Completion Conditions

- Employee grade is removed from runtime, live data, DTOs, and tests unless a new explicit concept replaces it by policy.
- Growth curve/XP policy is data-driven.
- Trust code is not accidentally deleted or expanded beyond confirmed policy.
- Starter loadouts are live-data authored.
- Fragment equip limits are enforced from canonical owned counts and explicit per-fragment policy.
- Bound equipment cannot be altered through normal interaction paths.
- Trauma affects final battle max HP as confirmed.

## Completion Report

Completed on 2026-06-22 by implementation/verification.

### Implemented

- Removed legacy employee `grade`/star state from runtime employee profiles, starter/recruitment live RON, DTO snapshots, and tests.
- Moved XP thresholds, post-battle survival XP, and level-to-battle-tier mapping into `RunPolicyData.growth` loaded from live RON.
- Removed inert `PveWinStack` and `QuestRewardStack` growth ids.
- Kept trust implementation deferred without deleting or expanding trust policy surfaces.
- Confirmed trauma/run HP is applied to final battle max HP after equipment, fragment, level, and consumable modifiers are assembled.
- Moved starter baseline skill fragments to live starter employee loadout data; production code no longer injects the starter fragment helper.
- Added explicit `SkillFragmentEquipLimit` live data and runtime enforcement for `OwnedCopies` and `GlobalExclusive`.
- Changed skill-fragment dismantle repair to unequip only the excess active loadouts after copy count changes.
- Locked bound equipment out of normal equip/unequip/dismantle/enhance/sell/combination-consumption paths while still exposing DTO state for UI warnings.

### Removed Legacy

- `EmployeeGrade`
- `GrowthId::PveWinStack`
- `GrowthId::QuestRewardStack`
- Hard-coded `Employee::experience_required_for_next_level`
- Production availability of built-in starter fragment injection

### Fixed Contracts

- Employee battle tier is derived from `RunPolicyData.growth.battle_tier_by_level`.
- XP grants use the canonical grant path and policy-backed level thresholds.
- Live skill fragments must declare `equip_limit`.
- Bound equipment preview state is informational; command validation remains authoritative.
- Starter employee baseline fragments are live-data authored.

### Validation

- `cargo test bound_equipment_cannot_be_dismantled_or_enhanced_in_maintenance --lib`
- `cargo fmt`
- `rg -n "EmployeeGrade|\\.grade\\b|\\bgrade\\b|PveWinStack|QuestRewardStack|post_battle\\.survival_xp" src tests ../game_resources/data ../game_server/src -S`
- `git diff --check`
- `cargo test --test ron_loading`
- `cargo check -p game_server`
- `cargo test --lib -- --test-threads=1`

### Remaining Risk

- Trust system policy remains intentionally deferred. Future trust implementation should preserve this goal's growth/loadout contracts unless a new policy decision explicitly changes them.
