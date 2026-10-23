# Goal: Update game_server Into a Thin core_enhanced Bridge

## Objective

Update `game_server` so it successfully bridges the Unity client to the latest `core_enhanced` single-player game system.

The server must not own game rules. It parses WebSocket command documents through `ApplicationCommand::from_document`, executes or prepares them through `CoreApplication` with `ApplicationExecutionContext`, and publishes Core's typed `ApplicationResult` meaning and payload. Changes published through paired active-run/profile persistence pass through the host's accepted application intent before Core commit. State reads use `CoreApplication::snapshot(ApplicationSnapshotKind::Run)` in the stable Unity-facing protocol.

## Current Policy Decisions

- The game is now fully single-player.
- Existing multiplayer, matchmaking, Redis queue, and PvP-related code may be useful later, so do not delete it.
- Deprecated multiplayer paths should be clearly marked and isolated from the active single-player Unity bridge.
- Long-term direction: prefer a thin, explicit adapter over server-side compatibility logic.
- Long-term response shape: keep a stable command-result plus state-snapshot flow for Unity, but let payload content follow the current `core_enhanced` contracts.
- Do not reintroduce phase-era or suppression-era core concepts as compatibility wrappers.

## Tutorial and Expedition consumer

The bridge accepts Core's Tutorial/Expedition modes, required loan_copy_ids, three distinct employees from six candidates (two per role),
paid HQ candidates, saved Gate/exhaustion continuation choices and first-return acknowledgement. Fresh Expedition
does not depend on a tutorial completion flag. The profile stores exact expedition sequence/binding/vault/story
beside monotonic knowledge. Paired publication uses profile4/active7/journal3; profile-only lifecycle mutations reject
before Core commit. Removed psychology fields and old modes/versions are rejected with original bytes protected.
This describes the current implementation contract; U12 execution and independent review remain in the Goal records.

## Success Criteria

### Current Relic boundary

Relic support follows Core's published RON catalog and generic application command/result boundary.
`CatalogKind::Relics` is one of the 26 catalogs under schema6; its strict definitions provide Normal effects
and optional replacement Corrupt effects/penalties. The official catalog currently contains no balance
definitions. Host/Unity code does not invent a second catalog, capability, ownership table or modifier
registry. See [data loading](../../core_enhanced/docs/data_loading_contract.md#relics-catalog-and-immutable-variants).

The bridge passes the Relic equip/unequip, state/numeric-plan queries, corruption/dismantle previews
and mutations, and free Event selection through Core's current action/context checks. It preserves
owned copy UUIDs, three-slot employee loadouts, typed catalyst/equipment-dust counters, exact result
kinds and explicit unresolved/rejected numeric projections. Ordinary corruption spends one typed
catalyst; the authored Event path spends none and retains an idempotent source-scene/copy receipt.
Relics are not Shop sale items. These are dedicated Core projections, not new host-owned inventory
snapshot fields. The [Unity protocol](unity_bridge_protocol.md#relic-commands-and-projections) lists
the actual wire command fields and response boundaries.

Durable publication preserves the current and independent SavePoint Relic graphs, including copy
variant/placement, materials, UUID stream and Event history. ActiveRun envelope V7, outer continuation
V12 and standalone Core checkpoint payload V13 are distinct formats. Content generation and original
damage-source bindings remain Core-owned; reconnect/Continue cannot manufacture them from a UI numeric
plan. Existing basic aura precedes one Relic composition. Strong retained owners and test-only Weak
lifetime checks do not establish host RSS or deployment capacity. No new live battle disk codec is
introduced by this boundary.

The goal is complete only when all required checks below pass.

- The server build and selected Core/server checks cover the changed command, query, paired storage and recovery paths.
- Relevant failures, selected/executed counts, skips and raw evidence are recorded; unrelated full suites are not automatic requirements.
- Preparation/build-only and prior target evidence do not establish current Tutorial/Expedition behavior.
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
  - live battle commands and `CompleteCombatResult` acknowledgement
- The active state snapshot comes from `CoreApplication::snapshot(ApplicationSnapshotKind::Run)`.
  The server serializes the typed `RunSnapshotDto` and adds only its host persistence/recovery overlay.
- The local active-run store uses strict envelope version 7 around external Core continuation document version 12.
  The standalone Core checkpoint payload encoder uses version 13; its document is not a normal Continue input.
  The opaque Core graph owns its power-model/Floor-anchor generation; the server envelope preserves monotonic load
  count and run peak. Completed publication writes current and backup for the same latest confirmed generation.
  Backup recovery and explicit SavePoint rewind do not decrease those high-water values.
- The server snapshot wrapper preserves Core's typed `encounter_compendium` root field without a
  second field list or metadata mapper. A local-install singleton persistence runtime restores and publishes
  the monotonic discovered/defeated sets independently of transport player UUID and active-run/checkpoint
  state, and projects recovery/dirty/fatal status to Unity. Before building the reconnect snapshot,
  `AttachSession` installs a readable authoritative profile through `CoreApplication::install_authoritative_player_profile`:
  the installed profile carries its exact lifecycle sequence/binding/vault/story and published monotonic knowledge.
  Recovery-pending, protected and fatal profiles are not installed as empty durable authority; their explicit
  recovery status remains available in the snapshot.
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
- Do not add account/cloud/database synchronization. Local active-run and encounter-compendium files are the
  approved host persistence boundaries.
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

Keep `PlayerGameClientMessage::Command` parsing the behavior document into the current public
`ApplicationCommand`; the server does not translate it into a domain `PlayerBehavior` enum.

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
- `CompleteCombatResult`

Remove active mappings for deleted phase/suppression actions.

Preferred protocol direction:

- Keep `#[serde(tag = "type")]` request messages.
- Prefer current game-system names over legacy aliases.
- Use snake_case at the JSON boundary if that is already expected by Unity; map internally to Rust enum variants explicitly.

### Phase 3: Update Result Mapping

Keep `behavior_result_to_command_result` consuming `ApplicationResult` and preserving its
public meaning, payload and disposition. Core owns domain `BehaviorResult` conversion.

Required result coverage:

- `RunStartFlowStarted { game_mode, visual_novel }`
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
- `RefreshShop`
- `SellItem`
- `PurchaseItem`
- `RewardGranted`
- `CombatRewardsGranted`
- `RewardState`
- `Ok`

Rules:

- Preserve `research_deliveries` only on authoritative planning-return completions:
  `NodeCompleted` and `FloorAdvanced`. Entry/state results do not carry this field.
- Preserve the same field inside recursive completion payloads such as
  `CombatRewardsGranted`, `EmployeeRecruited`, and `EmergencySuppliesGranted`.
- Send the successful command result before the trailing root snapshot so Unity can present the
  exact one-shot delivery fact without inferring it from inventory differences.
- Preserve live battle setup/update/resync side messages. Production terminal results have no event-log
  attachment; `compressed_event_log` is omitted. Diagnostic timelines remain outside this boundary.
- `complete_combat_result` acknowledges Core's stored compact receipt and advances to the next phase;
  it does not repeat the settlement or its unconditional grants.
- Preserve recursive completion payload for `CombatRewardsGranted`.
- Prefer serializing complete DTOs from core rather than rebuilding them manually in the server.

### Phase 4: Replace Server-Built Snapshots

Read the typed `RunSnapshotDto` through `CoreApplication::snapshot(ApplicationSnapshotKind::Run)`
and serialize it as a whole, preserving Core's published roots and action capabilities without a second
server-owned field list. Add only the host persistence/recovery overlay. Read-only projections and live
battle side-message results do not request a trailing full state snapshot.

An active battle uses Core's setup/update/resync projections. A finished host tick publishes the final
battle update before the `combat_result` snapshot; the client then acknowledges the stored receipt
with `complete_combat_result`. Neither the root snapshot nor terminal result attaches a replay log.

### Phase 5: Update Game Data Loading

Bring the server RON loader in line with current `core_enhanced/tests/common/mod.rs`.

The server loader must populate:

- abnormality data
- corroded employee profiles
- corroded wave presets
- starter employee candidates
- equipment data
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

- request JSON deserialization into current `ApplicationCommand`
- parsing command documents through `ApplicationCommand::from_document`
- result mapping for at least:
  - `RunStartFlowStarted`
  - `NodePreview`
  - `NodeEntered`
  - `SupportState`
  - live battle setup/update/resync projections
  - `CombatRewardsGranted`
- auth attach returns a core run snapshot without phase-era fields

Fast loop preference:

- Use `cargo check -p game_server` after each adapter/data-loader phase.
- Run broader tests only after compilation is stable.

## Completion Checklist

- [x] Active Unity route `/game` is identified as the single-player bridge.
- [x] Deprecated multiplayer routes are labeled but retained.
- [x] Removed phase/suppression request variants are no longer active in the bridge.
- [x] Current public `ApplicationCommand` documents are available through WebSocket request parsing.
- [x] `behavior_result_to_command_result` preserves typed `ApplicationResult` meaning, payload and disposition.
- [x] Core snapshot uses `CoreApplication::snapshot(ApplicationSnapshotKind::Run)` and typed `RunSnapshotDto`.
- [x] Live battle setup/update/resync and compact-receipt acknowledgement follow the current Core boundary.
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
