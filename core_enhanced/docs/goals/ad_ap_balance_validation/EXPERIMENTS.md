# AD AP Balance Validation Experiments

## 2026-06-07 - Goal Start

Commands/read scope:

- `docs/ad_ap_balance_validation_goal.md`
- `docs/goals/core_policy_implementation_master/PLAN.md`
- `docs/core_policy_decisions_2026_06.md`

Result:

- No code changes performed yet.
- Work memory created.

## 2026-06-07 - Runtime Inspection

Commands/read scope:

- `src/game/battle/damage.rs`
- `src/game/battle/types.rs`
- `src/game/combat_preview.rs`
- `src/game/data/pve_data.rs`
- `src/game/data/corroded_employee_data.rs`
- `src/game/data/corroded_wave_data.rs`
- `src/game/data/mod.rs`
- `tests/ron_loading.rs`
- `../game_resources/data/abnormalities/base.ron`
- `../game_resources/data/enemies/corroded_employees.ron`

Findings:

- Defense/magic resist use the existing mitigation formula and do not create zero damage by themselves when `minimum_damage > 0`.
- Existing damage feedback considers damage mitigated when final damage is 60% or less of raw damage.
- Existing combat preview already has high defense and high magic resist thresholds at 50.
- Preview threat warnings are generated from spawned preview enemies, plus at most one seeded rumor tag.
- Current data has node type `Boss`, but no explicit permanent immunity or boss immunity exception metadata.

## 2026-06-07 - Implementation

Changed:

- Added `src/game/combat_balance.rs` as the shared AD/AP policy module.
- Moved high defense, high magic resist, fast breakthrough, and mitigated-feedback thresholds behind named policy helpers.
- Updated damage feedback to use the shared mitigation feedback policy.
- Updated combat preview to compute required briefing warning tags through the shared policy.
- Added `GameDataBase::new` preview warning contract validation over representative seeds.
- Added focused tests for stat-only non-immunity and live preview warning consistency.

Verification:

- `cargo fmt`
- `cargo check -p game_core`
- `cargo test -p game_core resistance_alone_cannot_create_permanent_type_immunity_when_minimum_damage_exists -- --nocapture`
- `cargo test -p game_core combat_preview_serializes_typed_threat_warnings -- --nocapture`
- `cargo test -p game_core live_combat_previews_include_required_ad_ap_threat_warning_tags -- --nocapture`
