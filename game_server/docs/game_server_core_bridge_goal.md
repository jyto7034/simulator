# Goal: Update game_server Into a Thin core_enhanced Bridge

## Objective

Update `game_server` so it successfully bridges the Unity client to the latest `core_enhanced` single-player game system.

The server must not own game rules. It should translate WebSocket JSON requests into `game_core::game::behavior::PlayerBehavior`, execute them through `GameCore`, and return `BehaviorResult` plus the latest core snapshot in a stable Unity-facing protocol.

## Current Policy Decisions

- The game is now fully single-player.
- Existing multiplayer, matchmaking, Redis queue, and PvP-related code may be useful later, so do not delete it.
- Deprecated multiplayer paths should be clearly marked and isolated from the active single-player Unity bridge.
- Long-term direction: prefer a thin, explicit adapter over server-side compatibility logic.
- Long-term response shape: keep a stable command-result plus state-snapshot flow for Unity, but let payload content follow the current `core_enhanced` contracts.
- Do not reintroduce phase-era or suppression-era core concepts as compatibility wrappers.

## Success Criteria

The goal is complete only when all required checks below pass.

- `cargo check -p game_server` passes.
- `cargo test -p game_server` passes, or any failures are unrelated and documented.
- `cargo test -p game_core` is run after server compilation succeeds, or a clear blocker is documented.
- The active `/game` WebSocket path supports the latest `core_enhanced` single-player flow:
  - `StartNewGame`
  - `SelectStarterEmployees`
  - `RequestMapData`
  - `SelectMapNode`
  - `UseReconScan`
  - `ConfirmEnterNode`
  - `CancelSelectedNode`
  - `CompleteNode`
  - support node actions
  - reward selection
  - skill fragment equip, unequip, upgrade, awaken, dismantle
  - equipment restore, dismantle, enhance
  - combat replay finish
- The active state snapshot is based on `GameCore::get_run_snapshot_json()`.
- The server no longer calls removed core APIs such as:
  - `get_progression`
  - `get_current_phase_events`
  - `get_active_suppression_replay`
  - phase request or event-selection APIs removed from `core_enhanced`
- The server no longer maps removed `PlayerBehavior` variants:
  - `RequestPhaseData`
  - `SelectEvent`
  - `StartSuppression`
  - `FinishSuppressionReplay`
  - `ClaimCombatReward`
  - `ExitCombatReward`
- The RON game-data loader in `game_server` populates all current `GameDataBaseParts` fields required by `core_enhanced`.
- Deprecated multiplayer/matchmaking routes remain buildable and are labeled as deprecated, but they do not block the active single-player bridge.

## Non-Goals

- Do not redesign `core_enhanced`.
- Do not change game rules to make `game_server` easier to compile.
- Do not delete matchmaking or PvP code unless explicitly requested later.
- Do not make Unity UI changes in this goal.
- Do not add a persistence/database layer for single-player runs in this goal.
- Do not preserve phase-era protocol names if they obscure the current node-map model.

## Primary Files

- `src/main.rs`
- `src/game/player_game_actor/messages.rs`
- `src/game/player_game_actor/handlers.rs`
- `src/game/player_game_actor/state.rs`
- `src/game/player_game_actor/session.rs`
- `src/game/load_balance_actor/*`
- `src/game/match_coordinator/*`
- `src/matchmaking/**/*`
- `../core_enhanced/src/game/behavior.rs`
- `../core_enhanced/src/game/world.rs`
- `../core_enhanced/src/game/world/snapshot.rs`
- `../core_enhanced/src/game/data/mod.rs`
- `../core_enhanced/tests/common/mod.rs`

## Implementation Plan

### Phase 1: Establish the Bridge Contract

- Treat `/game` as the active Unity single-player bridge.
- Keep `/ws/`, matchmaking, match coordinator, and Redis pub/sub as deprecated legacy paths.
- Add clear comments or module-level documentation marking deprecated multiplayer paths.
- Avoid routing new single-player behavior through deprecated matchmaker code.

### Phase 2: Update Client Request Mapping

Update `PlayerGameClientMessage` and `PlayerBehaviorRequest` so the request adapter covers current `PlayerBehavior`.

Add or update request variants for:

- `StartNewGame`
- `SelectStarterEmployees { candidate_ids }`
- `RequestMapData`
- `SelectMapNode { node_id }`
- `UseReconScan`
- `ConfirmEnterNode`
- `CancelSelectedNode`
- `CompleteNode`
- `ChooseSupport { support_type }`
- `SelectSupportTarget { employee_uuid }`
- `SelectMedicalTreatment { treatment }`
- `SelectReward { reward_id }`
- `EquipSkillFragment { employee_uuid, fragment_id }`
- `UnequipSkillFragment { employee_uuid, fragment_id }`
- `UpgradeSkillFragment { target_fragment_id, material_fragment_id }`
- `AwakenSkillFragment { target_fragment_id, material_fragment_ids }`
- `DismantleSkillFragment { fragment_id }`
- `RestoreEquipment { recipe_id }`
- `DismantleEquipment { item_uuid }`
- `EnhanceEquipment { item_uuid }`
- existing inventory, field, bench, shop, reward, and equipment equip actions
- `FinishCombatReplay`

Remove active mappings for deleted phase/suppression actions.

Preferred protocol direction:

- Keep `#[serde(tag = "type")]` request messages.
- Prefer current game-system names over legacy aliases.
- Use snake_case at the JSON boundary if that is already expected by Unity; map internally to Rust enum variants explicitly.

### Phase 3: Update Result Mapping

Update `behavior_result_to_command_result` to cover every current `BehaviorResult`.

Required result coverage:

- `StartNewGame { candidates, required_count }`
- `StarterEmployeesSelected`
- `MapState`
- `NodePreview`
- `ReconScanUsed`
- `NodeEntered`
- `NodeCompleted`
- `SupportState`
- `ActComplete`
- `RunComplete`
- `RunFailed`
- `UnEquipItem`
- `EquipItem`
- `SkillFragmentLoadoutUpdated`
- `SkillFragmentUpgraded`
- `SkillFragmentAwakened`
- `SkillFragmentDismantled`
- `EquipmentRestored`
- `EquipmentDismantled`
- `EquipmentEnhanced`
- `MoveUnit`
- `MoveBenchUnit`
- `ShopState`
- `RerollShop`
- `SellItem`
- `PurchaseItem`
- `RewardGranted`
- `CombatRewardsGranted`
- `CombatResolved`
- `RewardState`
- `Ok`

Rules:

- Do not silently drop `research_deliveries`.
- Compress timelines for WebSocket payloads when returning `CombatResolved`.
- Preserve recursive completion payload for `CombatRewardsGranted`.
- Prefer serializing complete DTOs from core rather than rebuilding them manually in the server.

### Phase 4: Replace Server-Built Snapshots

Replace custom phase-era snapshot assembly with `GameCore::get_run_snapshot_json()`.

The snapshot returned after auth and after each command should come from core and include:

- `game_state`
- `game_state_context`
- `allowed_actions`
- `run_progression`
- `map_progression`
- `map`
- `current_node_session`
- `selected_event`
- `roster`
- `field`
- `bench`
- `inventory`
- `resources`

If an active combat replay exists, use the current core method `get_active_combat_replay()` only as a supplemental compressed timeline field. Do not use suppression-era naming.

### Phase 5: Update Game Data Loading

Bring the server RON loader in line with current `core_enhanced/tests/common/mod.rs`.

The server loader must populate:

- abnormality data
- corroded employee profiles
- corroded wave presets
- starter employee candidates
- equipment data
- artifact data
- shop data
- reward data
- random event data
- pve encounter data
- skill data
- skill fragment data

Rules:

- Use current `GameDataBuilder` or current `GameDataBaseParts`, whichever is clearer after reading the latest core API.
- Remove runtime dependency on deleted `event_pools` data.
- Keep live RON loading deterministic with `include_str!` unless there is a clear reason to switch.
- Do not add fallback data in the server for missing live data. Fix the loader or the shared RON path instead.

### Phase 6: Deprecate Multiplayer Without Deleting It

Mark these paths as deprecated legacy infrastructure:

- `/ws/` matchmaking route
- `MatchCoordinator`
- matchmaker modules
- Redis queue scripts and queue operations
- PvP/match result pubsub paths

Deprecation should be lightweight:

- Module comments or route comments are enough.
- Keep them compiling.
- Do not route single-player `/game` through them.
- Do not perform large refactors of deprecated code unless compilation requires it.

### Phase 7: Tests and Fast Feedback Loop

Run checks in this order:

1. `cargo check -p game_server`
2. `cargo test -p game_server`
3. `cargo test -p game_core`

If server tests are sparse, add focused adapter tests for:

- request JSON deserialization into current `PlayerBehaviorRequest`
- conversion from `PlayerBehaviorRequest` to `PlayerBehavior`
- result mapping for at least:
  - `StartNewGame`
  - `NodePreview`
  - `NodeEntered`
  - `SupportState`
  - `CombatResolved`
  - `CombatRewardsGranted`
- auth attach returns a core run snapshot without phase-era fields

Fast loop preference:

- Use `cargo check -p game_server` after each adapter/data-loader phase.
- Run broader tests only after compilation is stable.

## Completion Checklist

- [x] Active Unity route `/game` is identified as the single-player bridge.
- [x] Deprecated multiplayer routes are labeled but retained.
- [x] Removed phase/suppression request variants are no longer active in the bridge.
- [x] Current `PlayerBehavior` variants are available through WebSocket request mapping.
- [x] Current `BehaviorResult` variants are serialized by `behavior_result_to_command_result`.
- [x] Core snapshot uses `GameCore::get_run_snapshot_json()`.
- [x] Combat replay uses current combat naming and compression.
- [x] Server RON loader builds current `GameDataBase`.
- [x] `cargo check -p game_server` passes.
- [x] `cargo test -p game_server` passes or unrelated failures are documented.
- [x] `cargo test -p game_core` passes or unrelated failures are documented.
- [x] Any protocol-visible renames are documented for Unity.

## Experiment Log Template

Use this section while executing the goal. Add entries instead of relying on memory.

### Experiment: Initial Compile Baseline

- Command: `cargo check -p game_server`
- Result: failed before bridge updates.
- Findings: server still called removed phase/suppression APIs (`get_progression`, `get_current_phase_events`, `get_active_suppression_replay`) and mapped removed `PlayerBehavior`/`BehaviorResult` variants.
- Next action: update request adapter, result adapter, and state snapshot builder.

### Experiment: Request Adapter Update

- Command: `cargo test -p game_server`
- Result: passed after adding adapter tests.
- Findings: snake_case node-flow requests deserialize; removed phase/suppression requests are rejected.
- Next action: keep protocol-visible request names documented for Unity.

### Experiment: Result Adapter Update

- Command: `cargo check -p game_server`
- Result: passed.
- Findings: current `BehaviorResult` variants are exhaustively mapped, including research deliveries and compressed combat timelines.
- Next action: run server tests.

### Experiment: Data Loader Update

- Command: `cargo check -p game_server`
- Result: passed.
- Findings: server RON loader now matches current `GameDataBuilder` fields and no longer reads deleted `event_pools`.
- Next action: run `cargo test -p game_server`.

### Experiment: Final Verification

- Command: `cargo check -p game_server`
- Result: passed with pre-existing warnings.
- Findings: bridge compiles after formatting.
- Next action: none.

- Command: `cargo test -p game_server`
- Result: passed. 9 tests passed, 0 failed, 1 doctest ignored.
- Findings: server adapter and actor tests pass.
- Next action: none.

- Command: `cargo test -p game_core`
- Result: passed. 390 unit tests plus integration/doc tests passed.
- Findings: core regression suite is green.
- Next action: none.

- Command: `rg -n "get_progression|get_current_phase_events|get_active_suppression_replay|RequestPhaseData|SelectEvent|StartSuppression|FinishSuppressionReplay|ClaimCombatReward|ExitCombatReward|PhaseNotReady|SuppressAbnormality|RandomEventState|AdvancePhase|Ordeal" src`
- Result: no matches.
- Findings: removed phase/suppression names are no longer active in `game_server/src`.
- Next action: none.

## Notes for Future Agents

- This repository may contain unrelated dirty worktree changes. Do not revert them.
- Prefer reading `../core_enhanced/docs/current_handoff.md` before making policy assumptions.
- If server and core disagree, trust current `core_enhanced` behavior and update the server bridge.
- If Unity compatibility conflicts with core naming, preserve stable JSON shape but keep internal Rust mapping current.
- Ask before making any gameplay-policy decision. Adapter, serialization, and deprecation choices can be made locally.
