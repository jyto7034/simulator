# Node Map Mode Gate Navigation Contract

## Objective

Implement the newly fixed node-map and run-mode policy as a long-term core contract. The implementation must align runtime code, live RON/data, Unity-facing snapshot DTOs, command handling, and tests with `docs/game_rulebook.md` and the external Unity contract.

This goal covers:

- explicit runtime `GameMode` (`Standard`, `Endless`);
- `Gate` as an independent next Act/Floor node;
- Gate transition reward: living employees recover trauma by a RON-configurable percent, default 10%;
- Start as a real `Completed + Revealed` current node;
- Core-computed `map_navigation.current_node_id` and `map_navigation.selectable_node_ids`;
- removal of Unity-facing `map.edges`, `travel_path`, and legacy public available/completed id projections;
- Completed nodes as transit-only nodes;
- combat/elite/boss/final boss attempt policy with final-boss defeat exception.

## Source Of Truth Order

Do not trust documents blindly. Check in this order:

1. Actual runtime code.
2. Live RON/data under `/mnt/f/work/simulator/game_resources`.
3. Unity-facing snapshot/command contract and smoke probe.
4. Latest policy docs.

Canonical policy documents to compare against:

- `docs/game_rulebook.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md`
- `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py`

## Current Runtime Findings

- `RunProgression` currently stores `run_seed`, `act_index`, and `max_acts`; it does not store gameplay mode.
- `MapNodeCategory` currently has no `Gate` category.
- `RunMap.edges` is the internal graph source and should remain internal.
- `MapProgression::view()` currently returns `MapViewDto` with `edges`.
- `MapProgression` currently exposes `available_node_ids` and `completed_node_ids` as fields and snapshot DTO data.
- `MapProgression::is_node_selectable()` currently checks only node `state == Available` and `visibility != Concealed`.
- `reachable_available_node_ids()` already has the useful shape for Completed-node transit, but must be treated as core selection source, not duplicated in Unity.
- `ABNORMALITY_MAX_ATTEMPTS` is already `3`.
- `handle_retreat_battle()` currently rejects `CombatNodeType::Boss` retreat, which conflicts with the new final-boss policy where retreat before defeat is valid.
- Support policy already contains trauma recovery values for SavePoint/Rest, but Gate transition trauma percent is not yet a policy field.

## Implementation Plan

### 1. Inventory And Contract Diff

- Read `src/game/map/types.rs`, `src/game/map/progression.rs`, `src/game/map/generator.rs`, `src/game/world/snapshot.rs`, `src/game/behavior.rs`, `src/game/world/node_flow.rs`, `src/game/world/combat.rs`, and `src/game/data/run_policy_data.rs`.
- Read live map/node RON under `/mnt/f/work/simulator/game_resources/data/map`.
- Read existing tests around map flow, snapshots, retreat, support, and node completion.
- Record any mismatches in `EXPERIMENTS.md`.

### 2. Add Runtime GameMode

- Add a gameplay `GameMode` domain type with at least:
  - `Standard`
  - `Endless`
- Store mode in run state/progression so it is persisted through checkpoint save/load and snapshot generation.
- Default new games to `Standard` unless existing run setup code proves a better explicit default.
- Do not confuse gameplay `GameMode` with game_server environment `RUN_MODE`.
- If adding a mode to existing save/checkpoint data requires a migration decision that is not obvious, stop and report the question.

### 3. Add Gate Node Domain

- Add `Gate` as an independent map node category or equivalent first-class node kind.
- Do not make Elite/Boss double as Gate.
- Live/default Gate nodes must be `Revealed` unless explicit data says otherwise.
- Add or update live RON node definitions for Gate only if needed by this implementation.
- First Gate presentation is stairs. Core should expose category/kind data; Unity owns visual assets.
- Conditional/Locked/secret/Concealed Gate remains future policy. Do not author it in live RON unless explicitly requested.

### 4. Implement Gate Transition

- Add a core command/result path for confirmed Gate entry/transition, or reuse `ConfirmEnterNode` only if code proves Gate can be cleanly modeled as a node entered by the existing flow.
- Do not add a core `gate_confirm` state. Confirmation popup is Unity-local.
- On confirmed Gate transition:
  - validate the Gate is currently selectable by core navigation rules;
  - apply Gate transition trauma recovery to living employees only;
  - do not heal HP;
  - do not save a checkpoint;
  - keep equipment, skill fragments, consumables, resources, HP, trauma long-term state, and inventory except for the explicit trauma recovery;
  - advance Act/Floor according to `GameMode`;
  - mark consumed Gate as completed if it remains in the prior map state before transition.
- Add `gate_transition_trauma_recovery_percent` or equivalent RON policy field with default `10`.

### 5. Replace Unity-Facing Map Selection DTO

- Add Unity-facing `map_navigation` DTO:
  - `current_node_id`
  - `selectable_node_ids`
- Remove Unity-facing `map.edges` from `MapViewDto` or the public snapshot shape.
- Remove Unity-facing `map_progression.available_node_ids`, `map_progression.completed_node_ids`, `map.available_node_ids`, `map.completed_node_ids`, and `map.current_node_id`.
- Keep internal graph/edges in `RunMap`; do not remove core graph data.
- Do not provide `travel_path`; Unity does not animate edge traversal.
- Ensure `Start` is a real `Completed + Revealed` node and `map_navigation.current_node_id` points to it at run/Act/Floor start.

### 6. Core Navigation Rules

- Make core compute selectable nodes using:
  - destination is `Available`;
  - destination is not `Concealed`;
  - destination is reachable from `current_node_id`;
  - intermediate transit nodes are all `Completed`;
  - `Locked`, `Concealed`, `Unavailable`, and unresolved `Available` nodes are not transit nodes;
  - `ForwardOnly` and `Bidirectional` edge direction are honored.
- `SelectMapNode` must reject nodes outside `map_navigation.selectable_node_ids`.
- Completed nodes are transit only. Their original content/effect must never execute again.
- NodeConfirm must not move `current_node_id`; movement/current location commits only on successful `ConfirmEnterNode`/node entry semantics.

### 7. Combat Attempt And Retreat Policy

- General combat, elite, and normal boss:
  - total attempts including first entry: 3;
  - attempt is consumed on battle entry;
  - failure or retreat allows retry if attempts remain;
  - after all attempts are consumed, mark node `Completed`, grant no success/core reward/research progress, and keep it transit-only.
- Final boss:
  - defeat/failure judgment immediately causes run failure;
  - retreat before defeat is valid if attempts remain;
  - retreat consuming the third attempt causes run failure;
  - victory causes run clear.
- Determine how code distinguishes final boss from normal boss. If runtime data does not have a reliable source, stop and report the policy/data question.
- Update `handle_retreat_battle()` and result completion paths without adding one-off special cases that bypass normal node progression invariants.

### 8. Data Validation

- Validate live RON:
  - Gate node definitions exist only as intended;
  - no default live node uses `Concealed`/`Obscured` unless explicit;
  - graph edges are internally valid;
  - `Locked` nodes remain leaf rooms;
  - Start exists and is a valid current node source.
- Validate RON policy field for Gate trauma recovery percent.
- Unknown policy fields must continue to fail validation.

### 9. Tests

Focus tests on player-visible behavior and DTO contracts, not private implementation shape.

Required focused tests:

- New run snapshot exposes `map_navigation.current_node_id` as Start and includes selectable first nodes.
- Unity-facing map snapshot does not expose `map.edges` or legacy public id lists.
- `SelectMapNode` accepts only ids in `map_navigation.selectable_node_ids`.
- Completed nodes are transit-only and allow selecting a farther Available node through them.
- Unresolved Available/Locked/Concealed/Unavailable nodes cannot be transit nodes.
- `ForwardOnly` and `Bidirectional` navigation directions are honored.
- Completed nodes cannot rerun content/effects/rewards.
- Gate transition advances Act/Floor, applies living-employee trauma percent recovery, does not heal HP, and does not save checkpoint.
- Gate cancel remains Unity-local: no core state changes occur without command.
- `GameMode::Standard` produces the documented 3-Act progression semantics.
- Checkpoint save/load preserves gameplay `GameMode` and map navigation state.
- General combat/elite/normal boss attempts: three total attempts, then Completed/transit-only with no success reward on exhaustion.
- Final boss retreat is allowed before defeat while attempts remain.
- Final boss defeat immediately produces run failure.
- Final boss third retreat/exhaustion produces run failure.
- Live RON loading validates Gate policy and navigation contract.

Required integration/probe validation:

- Run focused `cargo test` after each small change.
- Run `cargo check -p game_core`.
- Run `cargo check -p game_server` if behavior/server DTO shape changes.
- Start the actual server and run `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py`.
- Record every failure, fix, and revalidation in `EXPERIMENTS.md`.

## Legacy Removal Policy

- Do not add dual schema compatibility for old Unity-facing `map.edges`.
- Do not keep `available_node_ids` or `completed_node_ids` as public Unity selection sources.
- Do not add Unity fallback aliases for `node_id`, `map_slot_id`, `template_id`, `edges`, or local graph selection.
- Internal `RunMap.edges` remains valid and required.
- If a short transition layer is truly required, stop and ask the user before implementing it. Record reason and removal condition in this file.

## Stop Conditions

Complete the goal with a policy-question report instead of guessing if any of these appear:

- no reliable runtime/data distinction between normal boss and final boss;
- unclear command semantics for Gate transition after reading current node flow;
- live RON needs content deletion/replacement beyond adding explicit Gate support;
- save/checkpoint migration requires choosing how old data maps to `GameMode`;
- Unity-facing DTO change would require a temporary compatibility layer;
- Gate reward timing conflicts with existing support/rest/save behavior;
- attempt exhaustion semantics conflict with existing battle result reward pipeline.

## Completion Criteria

- Runtime stores explicit `GameMode`.
- Gate exists as an independent node category/contract where implemented.
- Gate transition applies configured trauma percent recovery to living employees only.
- Start/current node and selectable node DTO are exposed via `map_navigation`.
- Unity-facing snapshot no longer exposes `map.edges` or legacy public selection id lists.
- Core selection and command validation use core-computed reachability through Completed transit nodes.
- Completed nodes are transit-only and cannot rerun content.
- Combat attempt/retreat/final-boss failure policy matches `docs/game_rulebook.md`.
- Live RON and DTO validation tests cover the new policy.
- External Unity contract/probe stays in sync.
- Final report lists changed contracts, removed legacy DTOs, updated tests, remaining risks, and exact validation commands.
