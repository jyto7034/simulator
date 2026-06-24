# Unity Range Preview Final Tiles Contract Experiments

## Experiment Log

### 2026-06-20 - Pre-goal code and data reading

Commands/context:

- `rg -n "apply_to_profile|BasicAttackModifier|ActiveSkill|defense_tile_range|effective_combat_profile_for_employee" src/game docs -S`
- `sed -n '760,900p' src/game/skill_fragment.rs`
- `sed -n '1,140p' src/game/battle/stat_pipeline.rs`
- `sed -n '430,590p' src/game/ability.rs`
- `rg -n "cast_targeting\\s*:\\s*Explicit|target:\\s*CastTarget|tile_origin:\\s*Anchor|tracking:\\s*GroundFixed" /mnt/f/work/simulator/game_resources/data/skills/base.ron -C 4`
- `sed -n '590,745p' src/game/battle/core/sim.rs`

Result:

- Success.
- Confirmed weapon profiles can override basic attack `defense_tile_range`.
- Confirmed current `BasicAttackModifier` fragments do not change range.
- Confirmed active skill fragments grant `skill_id`; active skill range comes from skill definitions.
- Confirmed core can represent `SkillCastTarget::Tile`, but current live skills do not require a player manual tile target.
- Confirmed the current default player basic attack pattern does not include the anchor tile, which conflicts with the newly selected fallback policy.

Decision:

- Proceed with a final-cells DTO contract.
- Keep manual tile targeting as a future explicit mode only.

### 2026-06-20 - External Unity contract audit

Commands/context:

- `sed -n '1018,1065p' '/mnt/f/unity projects/ark/docs/unity_core_contract.md'`
- `sed -n '1888,2005p' '/mnt/f/unity projects/ark/docs/unity_core_contract.md'`
- `sed -n '380,465p' '/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md'`

Result:

- Found stale Unity-facing contract text that still described `skill_catalog.skills[*]` and `defense_tile_range` as the range preview source of truth.
- Updated the external canonical Unity docs so `range_previews` final cells are the live overlay source, while `defense_tile_range` remains core/RON authoring data.

## Failed Attempts

- No failed implementation attempts yet.

## Validation Runs

- `cargo check -p game_core`
  - Result: passed.
  - Purpose: verify new DTO/helper wiring compiles before adding focused tests.
- `cargo test -p game_core range_preview`
  - Result: passed. 5 tests passed.
  - Purpose: verify final-cell range preview resolver behavior:
    - default basic attack fallback includes own tile and forward tile
    - facing rotation works
    - out-of-bounds cells are filtered
    - weapon profile range overrides fallback
    - auto-unit active skill preview remains separate and reports `RequiresRuntimeTarget`
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock`
  - Result: passed. 1 test passed.
  - Purpose: verify pending deployment preview helper and live checkpoint `range_previews` agree for a deployed unit.
- `cargo test -p game_core`
  - Result: passed. 479 unit tests, 36 integration tests, and doc tests passed.
  - Purpose: broad validation for game_core gameplay flow, Unity-facing DTO tests, live RON loading, skill catalog audits, and skill suite coverage.
- `rg -n "range_source_of_truth|range_previews|Unity는 스킬 이름|defense_tile_range" '/mnt/f/unity projects/ark/docs/unity_core_contract.md' '/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md'`
  - Result: passed.
  - Purpose: verify external Unity contract docs now describe `range_previews.final_cells` as the overlay source of truth and no longer tell Unity to calculate range preview from `defense_tile_range`.
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run`
  - Result: passed after rerunning from `/mnt/f/work/simulator/game_server`; the first workspace-root attempt failed because `config/development` is resolved relative to the game_server directory.
  - Purpose: start a live game_server for Unity WebSocket probe verification.
- `WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
  - Result: passed.
  - Purpose: verify the real `/game` WebSocket flow emits deployed player checkpoint units with `range_previews`; observed fallback basic attack cells `{x:6,y:6}` and `{x:7,y:6}` for a right-facing deployment, with active skill preview separated and marked `no_skill`.
