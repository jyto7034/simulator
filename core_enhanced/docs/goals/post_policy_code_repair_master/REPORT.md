# Post Policy Code Repair Master Report

## Executive Summary

Phase 1 and the non-policy-gated Phase 2 repair work are complete and green.

The broken `game_core` route-fixture test and `game_server` test cfg compile failure were fixed. The empty `unit_test` integration target was removed. Focused coverage was added for Unity-facing deployment/timeline payload fields, false threat rumor retreat/re-entry lifecycle, movement overlap policy, and roster snapshot effective-profile error surfacing.

The stale external resource deletion item was completed after explicit user approval.

## Changes Made

- Fixed `authored_protect_unit_tactical_plan_overrides_default_defense_contract` by making the authored encounter explicitly select a Defense ChokePoint battlefield before using `black_box_breach_main`.
- Refreshed server test fixtures for new core DTO fields:
  - `AbnormalityMetadata.mobility_kind`
  - `AbnormalityMetadata.target_traits`
  - `LiveBattleDeploymentDto.unit_deploy_costs`
- Added server payload assertions for:
  - `deployment.unit_deploy_costs[*].base_deploy_cost`
  - `deployment.unit_deploy_costs[*].effective_deploy_cost`
  - `timeline_delta[*].event.mobility_kind`
  - `timeline_delta[*].event.damage_type`
  - `timeline_delta[*].event.feedback_tags`
- Removed active `Observed` threat warning status/source variants.
- Implemented false rumor policy:
  - 20% chance.
  - At most one rumor.
  - Candidates limited to currently detectable warning tags absent from actual spawn-wave warnings.
  - Retreat/re-entry marks rumor warnings `Disproved`; briefing warnings remain `Unverified`.
- Removed empty `tests/unit_test.rs` and `tests/unit/mod.rs`.
- Updated future-facing goal documents to use `cargo test -p game_core --test <target>` for integration test targets.
- Replaced manual battle/live server `json!` payload assembly with typed serializable payload structs for:
  - `BattleAdvancedPayload`
  - `BattleStatePayload`
  - `BattlePlaybackChangedPayload`
  - `BattleUnitDeployedPayload`
  - `BattleUnitWithdrawnPayload`
  - `BattleSkillActivatedPayload`
- Split generated combat preview contract validation out of `GameDataBase::new` into explicit `GameDataBase::validate_generated_combat_preview_contracts()`.
- Consolidated current/projected employee effective combat profile derivation around `effective_combat_profile_for_employee_with_item_slot`.
- Removed disabled anti-overlap steering configuration and legacy lane-congestion helpers from continuous movement.
- Added roster snapshot `combat_profile.effective_profile_error` for effective profile construction failures.
- Changed combat preview `enemy_briefing.count_hint` to derive from resolved `spawn_waves[*].enemy_entries[*].count`.
- Audited stale `game_resources/data/equipments.ron` references and deleted it after explicit user approval.
- Updated external canonical Unity contract:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`

## Removed Legacy

- Removed the empty `unit_test` integration target instead of keeping a compatibility shell.
- Removed unused `Observed` threat warning contract surface from active core enums.
- Removed dead `SteeringParams` and anti-overlap/lane-congestion steering helper code that was no longer part of the live movement policy.
- Removed steering-only tests that preserved old non-default anti-overlap behavior.
- Removed stale external resource file `/mnt/f/work/simulator/game_resources/data/equipments.ron`; active equipment data remains under `/mnt/f/work/simulator/game_resources/data/equipments/base.ron`.

## New Contracts Fixed

- Authored PVE waves that reference a route must author a battlefield/node contract that actually provides that route.
- Unity-facing deployment payloads expose per-unit base/effective deploy costs.
- Unity-facing timeline payloads preserve airborne mobility and damage feedback fields through the server boundary.
- False rumor warnings are explicit rumors that become `Disproved` after retreat/re-entry.
- `GameDataBase::new` is now structural/static-data construction; generated combat preview warning validation is an explicit audit/test call.
- Projected item-slot validation and runtime/snapshot effective profile construction share the same helper family.
- Moving units may overlap; movement steering now reflects route/goal movement plus board/static-obstacle correction, not unit avoidance.
- Roster snapshots expose effective combat profile failures as `combat_profile.effective_profile_error: { code, message }` instead of silently nulling effective fields.
- Combat preview enemy count hints use resolved spawn wave entries, including generated corroded waves.

## Verification Commands

- `cargo fmt`
- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core authored_protect_unit_tactical_plan_overrides_default_defense_contract -- --nocapture`
- `cargo test -p game_server -- --list`
- `cargo test -p game_server game::player_game_actor::state::tests::live_deployment_command_payloads_preserve_mission_identity -- --nocapture`
- `cargo test -p game_server game::player_game_actor::state::tests::battle_advanced_payload_preserves_timeline_contract_fields -- --nocapture`
- `cargo test -p game_core rumor_threat_warning -- --nocapture`
- `cargo test -p game_core retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted -- --nocapture`
- `cargo test -p game_core retreat_marks_rumor_threat_warning_disproved_on_reentry_preview -- --nocapture`
- `cargo test -p game_core movement -- --nocapture`
- `cargo test -p game_core equipment:: -- --nocapture`
- `cargo test -p game_core employee_roster_snapshot_surfaces_effective_profile_errors -- --nocapture`
- `cargo test -p game_core generated_corroded_wave_source_resolves_during_preview_generation -- --nocapture`
- `rg -n "equipments\\.ron|data/equipments|equipments/base\\.ron|game_resources/data/equipments" "/mnt/f/unity projects/ark/docs" "/mnt/f/unity projects/ark/Assets" -S`
- `rm /mnt/f/work/simulator/game_resources/data/equipments.ron`
- `find /mnt/f/work/simulator/game_resources/data -maxdepth 3 -type f | sed 's#^#/##' | rg "equipments(\\.ron|/)"`
- `cargo test -p game_core --test ron_loading`
- `cargo test -p game_core --test skill_refactor_validation`
- `cargo test -p game_core --test skill_test_suite`
- `cargo test -p game_core --test live_item_skill_activation`
- `cargo test -p game_core --test live_skill_catalog_audit`
- `cargo test -p game_core`
- `cargo test -p game_server`

## Test Counts

- Focused route fixture: `1 passed; 0 failed`.
- Server test list: `11 tests`.
- Focused deployment payload test: `1 passed; 0 failed`.
- Focused timeline payload test: `1 passed; 0 failed`.
- Rumor focused filter: `4 passed; 0 failed`.
- Retreat exhaustion focused test: `1 passed; 0 failed`.
- Retreat rumor re-entry focused test: `1 passed; 0 failed`.
- Movement focused filter: `57 passed; 0 failed`.
- Equipment focused filter: `20 passed; 0 failed`.
- Focused effective profile error snapshot test: `1 passed; 0 failed`.
- Focused generated wave briefing count test: `1 passed; 0 failed`.
- Restricted Unity docs/Assets stale equipment reference search: no matches.
- Equipment resource file inventory after deletion: only `game_resources/data/equipments/base.ron` remains.
- `ron_loading`: `16 passed; 0 failed`.
- `skill_refactor_validation`: `10 passed; 0 failed`.
- `skill_test_suite`: `15 passed; 0 failed`.
- `live_item_skill_activation`: `3 passed; 0 failed`.
- `live_skill_catalog_audit`: `3 passed; 0 failed`.
- Full `game_core`: lib `434 passed; 0 failed`; integration targets `31 passed; 0 failed`; doctests/main binary `0 tests`.
- Full `game_server`: lib `11 passed; 0 failed`; doctests/main binary `0 tests`.

## Remaining Risks

- The workspace still emits `auth_server/Cargo.toml: unused manifest key: env`; it was not addressed in this goal phase.
