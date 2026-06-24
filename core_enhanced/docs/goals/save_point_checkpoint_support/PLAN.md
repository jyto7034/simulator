# SavePoint Checkpoint Support

## Objective

Remove the legacy Medical support-node flow completely and implement SavePoint as a real run checkpoint system.

This goal is not a rename-only cleanup. `Medical` currently owns target selection, treatment selection, HP recovery, and "available medical node can prevent no-deployable run failure" behavior. The long-term direction is to delete that legacy behavior and replace it with an explicit SavePoint contract:

- `SavePoint` is a support node that completes with `CompleteNode`.
- Completing SavePoint saves one latest run checkpoint.
- Completing SavePoint reduces trauma for living employees only.
- SavePoint never heals HP and never revives or re-enables incapacitated employees by itself.
- Checkpoint load is a separate run recovery feature with a per-run load limit.

## Source Of Truth Order

1. Runtime code and focused tests.
2. Live RON/data.
3. Unity-facing snapshot/command/result DTO contracts.
4. Live server/probe behavior.
5. Current policy documents, especially `docs/game_rulebook.md`.

Do not trust docs alone. The current docs already mention SavePoint, but runtime code still exposes Medical.

## Current Evidence To Re-Read

- `src/game/map/types.rs`
  - `SupportNodeType::Medical`
  - `MedicalTreatmentKind::{EmergencyCare, Counseling, BalancedCare}`
- `src/game/resources/selection.rs`
  - `SupportSessionState` stores support target candidates, selected employee, and selected medical treatment.
  - `requires_employee_target(Medical)` and medical treatment selection make Medical a multi-step node.
- `src/game/world/support.rs`
  - `handle_select_support_target`
  - `handle_select_medical_treatment`
  - `support_medical_heal`
  - `default_support_choices()` returning Medical/Rest
  - `support_state_result()` exposing Medical-specific state.
- `src/game/world/combat.rs`
  - `has_available_medical_support_node()` currently treats a future Medical node as a recovery path when no deployable employees remain.
  - This must not be renamed to SavePoint; it must be replaced by checkpoint-load availability.
- `src/game/behavior.rs`
  - `PlayerBehavior::SelectMedicalTreatment`
  - `ActionKind::SelectMedicalTreatment`
  - Support DTO/result fields for target candidates, selected employee, and selected medical treatment.
- `src/game/managers/action_scheduler.rs`
  - Medical treatment actions are currently schedulable.
- `../game_resources/data/map/node_definitions.ron`
  - `support_medical`, `support_full_choice`, and choices containing `Medical`.
- `../game_resources/data/run/policy.ron`
  - `medical_hp_heal_percent`
  - `medical_trauma_heal`
  - `medical_balanced_hp_heal_percent`
- `src/game/world/tests/support.rs`, `src/game/world/tests/combat.rs`, `src/game/world/tests/snapshots_and_start.rs`
  - Medical behavior tests must be deleted or replaced with SavePoint/checkpoint tests.

## Confirmed Policy

- Core official support choices are `SavePoint` and `Rest`.
- `SupportNodeType::Medical` is removed, not kept as an alias.
- Serialized DTO value `"Medical"` is removed. Official value is `"SavePoint"`.
- Map kind id `support_medical` is replaced by `support_save_point`.
- Support choice lists are `["SavePoint", "Rest"]` where both are available.
- SavePoint has no target selection and no treatment selection.
- SavePoint completes through `CompleteNode`.
- `SelectMedicalTreatment`, `MedicalTreatmentKind`, `selected_medical_treatment`, and Medical-only treatment variants are removed.
- `select_support_target` and support target DTO fields are removed unless re-reading the code proves a non-Medical support node still needs them. Do not keep them speculatively.
- SavePoint completion effect:
  - save current run checkpoint;
  - reduce trauma for living employees only;
  - do not heal HP;
  - do not affect dead, missing, or incapacitated employees.
- SavePoint trauma reduction policy:
  - replace Medical policy fields with `save_point_trauma_reduction_percent`;
  - default value is `15`;
  - live data validation accepts only `10..=20`.
- Run checkpoint policy:
  - one save slot per run;
  - latest SavePoint overwrites the previous checkpoint;
  - load target is the last completed SavePoint checkpoint;
  - load count is maximum 3 per run;
  - checkpoint load availability, not future Medical recovery, is the recovery path for no-deployable run states.
- Checkpoint load restore policy:
  - checkpoint load should feel like returning to the completed SavePoint moment;
  - user-visible run state after the checkpoint is restored to the checkpoint state;
  - load is available only in Safezone, not during combat, combat result, shop, maintenance, support-node handling, or other active node sessions;
  - load usage count is not restored by checkpoint load and is not reset by saving a new SavePoint;
  - debug/operational logs, server logs, battle record exports, and other diagnostic artifacts are preserved;
  - player-facing collectible documents/story unlocks are not implemented yet, but when implemented they should be preserved across checkpoint load rather than rewound;
  - active combat result screen data is not a checkpoint-load case because load is Safezone-only.

## Plan

1. Re-read the runtime support, map progression, run state, roster, reward/resource state, node session, and command scheduling code before editing.
   - Identify every user-visible state field that must be restored by checkpoint load.
   - Identify which fields must not be restored, such as debug-only logs or server transport-only runtime buffers, unless code proves otherwise.
2. Build a focused failing test inventory before the main edit.
   - A Medical support node or serialized Medical DTO should fail once the new policy is implemented.
   - SavePoint should require no target and no treatment.
   - SavePoint should reduce trauma for living employees only and never heal HP.
   - Completing SavePoint should save a checkpoint.
   - Loading a checkpoint should restore the chosen run state and decrement the load counter.
   - Loading should be unavailable when no checkpoint exists or all 3 loads are consumed.
3. Remove Medical type surface.
   - Replace `SupportNodeType::Medical` with `SupportNodeType::SavePoint`.
   - Delete `MedicalTreatmentKind`.
   - Delete `PlayerBehavior::SelectMedicalTreatment` and `ActionKind::SelectMedicalTreatment`.
   - Delete Medical-only command handlers and scheduler entries.
   - Delete Medical-only selected-event and command-result DTO fields.
   - Do not add aliases, dual schema, or fallback deserialization for `Medical`.
4. Simplify support session state around the new contract.
   - Keep support choice selection for full/limited support nodes.
   - Remove target/treatment state unless a real non-Medical support node requires it.
   - Ensure `SavePoint` and `Rest` both complete via `CompleteNode`.
5. Implement SavePoint effect.
   - Apply policy-driven trauma reduction to living employees only.
   - Leave HP unchanged.
   - Save the checkpoint as part of the successful node completion flow.
   - Ensure failed node completion does not leave a partially written checkpoint if map progression cannot commit.
6. Implement run checkpoint storage.
   - Add explicit run checkpoint state to the canonical world/run state, not to Unity-only DTOs.
   - Store the latest completed SavePoint checkpoint.
   - Store remaining load count or used load count in the run state.
   - Prefer structured typed state over `serde_json::Value`.
7. Implement checkpoint load command and state transition.
   - Add a typed behavior/action for loading the run checkpoint if one does not already exist.
   - Expose it only when a checkpoint exists and loads remain.
   - Load should restore the canonical run state to the saved point and consume one load.
   - Do not silently auto-load unless code/policy is re-confirmed; player action is the expected first implementation.
8. Replace no-deployable Medical fallback logic.
   - Delete `has_available_medical_support_node()`.
   - Replace no-deployable recovery checks with checkpoint-load availability.
   - Do not treat future SavePoint nodes as recovery paths, because SavePoint does not heal HP.
9. Update live RON/data and validation.
   - Replace `support_medical` with `support_save_point`.
   - Replace support choices containing `Medical` with `SavePoint`.
   - Remove Medical policy fields.
   - Add `save_point_trauma_reduction_percent`.
   - Add validation for `10..=20`.
10. Update tests to lock user-visible behavior and contracts.
    - Delete Medical tests rather than changing expected strings while preserving Medical flow.
    - Add SavePoint support tests.
    - Add checkpoint save/load tests.
    - Add allowed-actions tests for `LoadRunCheckpoint` availability.
    - Add DTO serialization tests showing no Medical/treatment fields remain.
11. Update internal and Unity-facing documentation.
    - `docs/game_rulebook.md`.
    - Internal DTO/command docs.
    - External Unity contract docs if available in the workspace.
    - Probe docs if WebSocket command/snapshot shape changes.
12. Run focused validation after each small edit, then broad validation.
13. If possible after implementation, start the server and run a WebSocket probe that reaches SavePoint and verifies the official snapshot/command shape.

## Completion Conditions

- Runtime code has no official `Medical` support node, treatment enum, treatment command, or Medical HP-heal path.
- Live RON/data has no official `Medical`, `support_medical`, or Medical policy fields.
- `SavePoint` is an official support type in runtime code, RON, DTOs, and docs.
- `SavePoint` completion saves the latest run checkpoint.
- SavePoint trauma reduction applies only to living employees and never heals HP.
- Checkpoint load state exists in canonical run/world state.
- Checkpoint load is available only when a checkpoint exists and load count remains.
- The load count limit is 3 per run.
- Future SavePoint nodes are not treated as HP recovery paths for no-deployable employees.
- Unity-facing DTOs and allowed actions do not expose Medical-only target/treatment fields.
- Tests cover SavePoint completion, checkpoint save/load, load limit, no HP heal, no Medical DTO surface, and no-deployable checkpoint recovery.
- No compatibility alias, fallback schema, ignored legacy test, or dual Medical/SavePoint support path is introduced.

## Policy Completion Rule

If implementation reveals a policy decision not already covered, complete this goal and report the exact question list instead of guessing. Do this especially for:

- exact checkpoint save scope;
- whether checkpoint load restores rewards/inventory/research changes earned after the checkpoint;
- whether loading a checkpoint consumes a map node attempt or anomaly attempt;
- Unity-facing DTO shape changes beyond removing Medical fields and adding checkpoint-load availability.

When this rule triggers, mark the goal complete with a policy-decision report. Do not leave it paused as a blocked goal.

## Validation Commands

Use focused commands first, then broaden:

- `cargo test -p game_core game::world::tests::support -- --nocapture`
- `cargo test -p game_core game::world::tests::map_flow -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_core`
- If server/transport behavior changes: run `game_server` locally and execute a WebSocket probe against `ws://127.0.0.1:18083/game`.

## Completion Report Template

Status:

Change summary:

Removed legacy:

New contracts fixed:

Tests updated:

Docs/probes updated:

Remaining risk:

Validation commands run:

## Completion Report

Status: implemented and verified.

Change summary:

- Removed the legacy Medical support-node model and made `SavePoint`/`Rest` the official support effects.
- Implemented SavePoint as a `CompleteNode` support node that reduces living employee trauma, never heals HP, and saves the latest run checkpoint after successful node completion.
- Added typed canonical run checkpoint state and `load_run_checkpoint` / `LoadRunCheckpoint`.
- Added `run_checkpoint` snapshot metadata so Unity can display checkpoint existence, used loads, max loads, remaining loads, and load availability without local inference.

Removed legacy:

- Removed `SupportNodeType::Medical`, `MedicalTreatmentKind`, Medical target/treatment commands, Medical HP healing, and Medical target/treatment DTO fields.
- Replaced `support_medical` live data with `support_save_point`.
- Replaced Medical run policy fields with `save_point_trauma_reduction_percent`.
- Removed future-Medical no-deployable recovery and replaced it with checkpoint-load availability.

New contracts fixed:

- Official support types are `SavePoint` and `Rest`.
- SavePoint completes through `complete_node`; checkpoint load uses `load_run_checkpoint`.
- `LoadRunCheckpoint` is exposed only through allowed actions when a checkpoint exists and load count remains.
- Load count is limited to 3 per run and is not restored by loading a checkpoint.
- Checkpoint payload is typed canonical runtime state, not `serde_json::Value`.
- Debug logs, server logs, and battle record exports are preserved across checkpoint load.

Tests updated:

- Added SavePoint tests for living-only trauma reduction, no HP heal, checkpoint save, checkpoint load, load count metadata, load limit, and absent Medical target/treatment DTO fields.
- Added combat recovery coverage for no-deployable roster states through checkpoint-load availability.
- Updated support, map flow, snapshot/start, action scheduler, combat, and RON-loading tests to the SavePoint contract.

Docs/probes updated:

- Updated `docs/game_rulebook.md`.
- Updated `/mnt/f/unity projects/ark/docs/unity_core_contract.md`.
- Updated `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md`.
- Ran the Unity WebSocket smoke probe against a local `game_server`; probe output included `run_checkpoint` in `snapshot_root_keys`.
- Added and ran `/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py`; probe completed a live SavePoint, verified checkpoint metadata, then loaded the checkpoint.

Remaining risk:

- Future player-facing collectible document/story unlock state is not implemented yet. When added, it must remain outside the rewound checkpoint payload per policy.

Validation commands run:

- `cargo fmt`
- `cargo check -p game_core`
- `cargo test -p game_core game::world::tests::support -- --nocapture`
- `cargo test -p game_core game::world::tests::map_flow -- --nocapture`
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_core game::managers::action_scheduler -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core`
- `cargo check -p game_server`
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run` from `/mnt/f/work/simulator/game_server`
- `WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
- `WS_PORT=18083 python3 '/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py'`
