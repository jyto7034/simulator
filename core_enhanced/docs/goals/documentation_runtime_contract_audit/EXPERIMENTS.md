# Documentation Runtime Contract Audit Experiments

This file records audit attempts, failed assumptions, fixes, and verification results.

## 2026-06-21 - Goal Document Creation

Action:

- Created `docs/goals/documentation_runtime_contract_audit/PLAN.md`.
- Created `docs/goals/documentation_runtime_contract_audit/EXPERIMENTS.md`.
- Created `docs/goals/documentation_runtime_contract_audit/EXPERIMENT_NOTES.md`.

Evidence used:

- Read `docs/README.md`.
- Read `docs/code_documentation_sync_guidelines.md`.
- Read `docs/game_rulebook.md`.
- Read `docs/skill_target_contract.md`.
- Read `docs/refactor_preparation_plan.md`.
- Read external `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`.
- Read external `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Listed current `docs/goals/*/PLAN.md` files while updating the docs index.

Result:

- Audit goal workspace exists.
- No runtime audit has been performed yet in this goal.
- No code, data, DTO, or external Unity contract files were changed by this goal creation step.

Verification:

- Not run. This step only creates documentation.

## 2026-06-21 - Canonical Transport Document Inventory

Action:

- Re-read `docs/README.md`, `docs/code_documentation_sync_guidelines.md`, `docs/game_rulebook.md`, `docs/codex_goal_command.md`, and `docs/refactor_preparation_plan.md`.
- Searched for stale references to the old split battle setup/update contracts.

Evidence used:

- `rg -n "core_unity_battle_setup_snapshot_contract|core_unity_battle_update_contract|core_unity_battle_transport_contract|unity_core_contract|unity_client_implementation_goal|stale copy|supersede|Supersedes|canonical" docs "/mnt/f/unity projects/ark/docs"`
- External `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md` states that it supersedes the old split setup/update documents.
- External `/mnt/f/unity projects/ark/docs/unity_core_contract.md` points live battle runtime transport to the integrated transport document.

Fixes:

- Updated `docs/game_rulebook.md` to point battle setup/update/checkpoint/resync/final snapshot flow at external `core_unity_battle_transport_contract.md`.
- Updated `docs/codex_goal_command.md` so new transport changes prefer the integrated battle transport document.
- Updated `docs/code_documentation_sync_guidelines.md` so the integrated battle transport document is the canonical transport doc and split docs are only migration context.
- Updated `docs/refactor_preparation_plan.md` so absorbed transport policy goes to the integrated external contract.

Result:

- Current top-level local docs no longer treat `core_unity_battle_setup_snapshot_contract.md` or `core_unity_battle_update_contract.md` as active canonical sources.
- `docs/README.md` still lists the split documents, but explicitly as superseded migration context.

## 2026-06-21 - Unity-Facing Battle Transport Runtime Audit

Action:

- Compared external transport docs with Rust DTOs and game_server message mapping.

Evidence used:

- `src/game/behavior.rs`
  - `LiveBattleSetupSnapshotDto`
  - `LiveBattleUpdateDto`
  - `LiveBattleStateCheckpointDto`
  - `LiveBattleUnitCheckpointDto`
  - `LiveBattleUnitHudDto`
- `src/game/world/state.rs`
  - `battle_setup_snapshot_dto`
  - `battle_update_dto_after`
  - `battle_update_dto_from_events`
  - `live_deployment_dto`
  - `live_unit_hud_dto`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`
  - top-level `battle_setup_snapshot`, `battle_update`, `battle_resync`
  - `request_battle_resync { known_seq, need_setup }`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs`
  - battle side-message command mapping with `send_state_snapshot: false`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs`
  - final `battle_update` followed by `combat_result` `state_snapshot`

Result:

- Runtime DTOs match the integrated battle transport direction:
  - battle setup is top-level `battle_setup_snapshot`.
  - live updates are top-level `battle_update` with `events_delta` and `checkpoint`.
  - catch-up resync is top-level `battle_resync` with nested `update`.
  - battle command results are accepted/error feedback, not state mutation source.
  - battle side-message commands do not get a trailing full `state_snapshot`.
  - final live tick pushes the last `battle_update` before the `combat_result` snapshot.
  - future `known_seq` is rejected with `InvalidBattleResyncSeq`.
- One nuance remains recorded in notes: setup-loss recovery may still produce a normal command response plus node-confirm snapshot. That does not currently contradict the canonical docs, but should stay visible to Unity implementers.

## 2026-06-21 - Game Rulebook Runtime/Data Audit

Action:

- Compared `docs/game_rulebook.md` against current combat mode, survive timer, redeploy policy, battle records, and generated route policy.

Evidence used:

- `src/game/combat_preview/types.rs`: `CombatMissionVariant` is `Defense | Boss`.
- `src/game/data/validation.rs`: `survive_timer_ms` must be positive and only on explicit Defense encounters.
- `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron`: live encounters use `node_type: Defense`; one encounter uses `survive_timer_ms: Some(45000)`.
- `src/game/world.rs`: battle deployment policy uses withdraw redeploy cooldown `30_000`, defeat redeploy cooldown `90_000`, redeploy cost multiplier `150`.
- `src/game/world/state.rs`: completed battle records are exported to `battle_records/run_<run_seed>/<battle_uuid>.json`.
- `src/game/combat_preview/mod.rs`: generated DefenseRoute path generation rejects non-contiguous fallback routes.

Result:

- `docs/game_rulebook.md` matches current runtime policy for official live combat:
  - `DefenseRoute` is the official live combat axis.
  - `Encirclement`, `Recovery`, `DefendAndEscape`, and `SplitRoom` are not active mission variants.
  - survival is `Defense + survive_timer_ms`, with wave data coming from encounter/wave pool data.
  - redeploy cooldowns match runtime.
  - battle records are persisted as JSON exports.
- No gameplay-policy question was found in this pass.

## 2026-06-21 - Skill Target Contract Runtime/Data Audit

Action:

- Compared `docs/skill_target_contract.md` against ability schema, skill data validation, abnormality/basic attack validation, corroded employee profile data, and battle targeting code.

Evidence used:

- `src/game/ability.rs`: `DeliveryDef::{Instant, Projectile, TileArea}`, `ProjectileHitPolicy::{TargetLocked, DirectionalCollision}`, `SkillTileAreaDeliveryDef`.
- `src/game/data/skill_data.rs`: `WholeFieldValidTiles` validation, `TileArea` validation, projectile validation, range preset resolution.
- `src/game/data/abnormality_data.rs`: basic attack delivery rejects `TileArea` and rejects `DirectionalCollision`.
- `src/game/data/corroded_employee_data.rs`: `profile_role`, range preset pool, normal corroded employee validation.
- `/mnt/f/work/simulator/game_resources/data/enemies/corroded_employees.ron`: `melee_front_1`, `ranged_center_3x3`, `ranged_center_5x5` presets and live profile mapping.
- `src/game/battle/core/basic_attack.rs` and `src/game/battle/core/targeting.rs`: tile-based basic attack eligibility and ranged reposition policy.

Fixes:

- Updated `docs/skill_target_contract.md` to separate current live weapon archetypes from future candidates:
  - current live RON: `Sword`, `Spear`, `Shield`, `Bow`, `Gun`, `Staff`
  - `GrenadeLauncher` remains enum-only/misused future candidate, not current live content
  - `Axe`, `Crossbow`, `Shotgun` require a later enum/RON/Unity contract goal

Result:

- Skill/range contract mostly matches runtime:
  - official live DefenseRoute range source is `TileRangePattern`.
  - Unity preview source is final `range_previews` cells.
  - `range_units` is not official basic attack eligibility, though runtime still keeps it for movement/approach and future continuous mechanics.
  - basic attack projectile delivery is target-locked; skill projectiles can use target-locked or directional collision policies.
  - normal corroded employees cannot use `WholeFieldValidTiles`.
- No code fix was performed in this audit pass.

## 2026-06-21 - Content Docs Versus Live Data Audit

Action:

- Compared `docs/skills/lobotomy_content_catalog.md` and `docs/skills/abnormality_skill_design_notes.ko.md` against live abnormality, encounter, and enemy RON files.

Evidence used:

- `/mnt/f/work/simulator/game_resources/data/abnormalities/base.ron`
- `/mnt/f/work/simulator/game_resources/data/abnormalities/legacy_random_event_abnormalities.ron`
- `/mnt/f/work/simulator/game_resources/data/pve/encounters.ron`
- `/mnt/f/work/simulator/game_resources/data/enemies/corroded_employees.ron`
- `/mnt/f/work/simulator/game_resources/data/skill_fragments/base.ron`
- `/mnt/f/work/simulator/game_resources/data/equipments/base.ron`
- `docs/skills/lobotomy_content_catalog.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/skills/skill_fragment_system.md`
- `docs/skills/skill_fragment_wiki.md`
- `docs/skills/ego_equipment_wiki.md`
- `docs/skills/tool_abnormality_wiki.md`

Fixes:

- Updated `docs/skills/lobotomy_content_catalog.md` so current weapon archetype examples match live RON and future candidates are not presented as current defaults.
- Updated `docs/skills/lobotomy_content_catalog.md` so current skill-system capability says tile-pattern areas, not geometric Circle/Line/Box/Rectangle/Cone AoE.

Result:

- The two major abnormality content docs are design/content source documents, not proof that every listed mechanic is already implemented.
- Live abnormality and encounter RON broadly follows the current confirmed roster.
- `skill_fragment_wiki.md` and `ego_equipment_wiki.md` intentionally index the current 22 live placeholder fragment/equipment candidates, not the full future roster.
- `tool_abnormality_wiki.md` correctly treats tool abnormalities as design/policy candidates outside combat unit, normal fragment, and placeholder E.G.O indexes.
- `legacy_random_event_abnormalities.ron` remains as an explicit legacy data file. It was not modified by this audit because deleting/replacing content is outside scope and should be done through a content/data goal.

## 2026-06-21 - Verification

Action:

- Ran focused compile/test checks for the audited areas.

Commands and results:

- `cargo check -p game_core`
  - Passed.
- `cargo test -p game_server player_game_actor -- --nocapture`
  - Passed: 22 tests, 0 failed.
  - Covered top-level battle setup/update/resync serialization, command-result side-message mapping, final battle update before combat result snapshot, future resync cursor rejection, setup-loss recovery to node confirm, and battle playback commands without trailing full snapshot.
- `cargo test -p game_core live_defense -- --nocapture`
  - Passed: 9 tests, 0 failed.
  - Covered live DefenseRoute setup/update behavior, playback speed/pause, future known_seq rejection, setup-loss recovery, redeploy locks, retreat, and live skill activation.
- `cargo test -p game_core ron_loading -- --nocapture`
  - First attempt did not run the intended tests because `ron_loading` was treated as a test-name filter; all tests in that target were filtered out.
- `cargo test -p game_core --test ron_loading -- --nocapture`
  - Passed: 16 tests, 0 failed.
  - Covered live abnormality roster validation, live PVE references, map content pools, reward restrictions, projectile delivery authoring, and live preview warning tags.

Result:

- No runtime regression was found by the focused checks.
- The first RON command was a command-selection mistake, not a product failure; the corrected `--test ron_loading` command passed.
