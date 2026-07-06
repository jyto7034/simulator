# Node Map Facility Template Contract Plan

## Objective

Implement the long-term core-to-Unity Node Map contract described in `docs/node_map_facility_template_contract.md`.

The game should present map progression as facility exploration on a fixed authored template, not as a freeform Slay-the-Spire-style abstract tree. Core owns the run graph, node assignment, traversal rules, current location, and selectability. Unity owns the local facility art, slot coordinates, corridor rendering, and presentation effects.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime core map generation, map progression, node selection, and snapshot code.
2. Live RON/data used by map generation, node kinds, act/floor setup, and any template assignment data.
3. Unity-facing snapshot/command contracts and live WebSocket JSON.
4. Latest policy docs, especially `docs/node_map_facility_template_contract.md`.

Do not trust this plan or the policy document over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Policy To Implement

- Node Map is facility-template based.
- Core sends `map.map_template_id`; Unity uses it to select local map art/template.
- Core sends every run graph node in `map.nodes`, including concealed, obscured, unavailable, locked, completed, and available nodes.
- Core sends `slot_id` for every node.
- Unity never guesses slot from order, name, kind, category, or fallback coordinates.
- Core sends graph edges as node-id pairs plus explicit `direction`.
- Valid edge directions are `Bidirectional` and `ForwardOnly`.
- There is no implicit edge direction default.
- `map_progression.current_node_id` is the current exploration location.
- Current location is not a node state. Do not add `Current` to `map.nodes[*].state`.
- Entrance is a normal node, starts `Completed`, and is the initial `current_node_id`.
- `node_confirm` is preview/confirmation only. It must not move `current_node_id`; actual movement happens after `confirm_enter_node` succeeds.
- Canonical Unity-facing selection rule:

```text
node.state == "Available" && node.visibility != "Concealed"
```

- `Available` is derived by core progression from current location, completed transit nodes, locked conditions, and explicit edge direction.
- Completed nodes are transit only; their content cannot be replayed.
- Locked nodes are non-selectable until their condition is satisfied.
- Locked nodes are leaf rooms and must not have child/progression nodes behind them.
- `visibility` is separate from progression state:
  - `Concealed`: DTO contains the node, Unity hides it, not selectable.
  - `Obscured`: Unity may show an unknown room; if `Available`, it is selectable.
  - `Revealed`: Unity may show normal kind/category/state/preview details.
- Long-term progress states:
  - `Available`
  - `Completed`
  - `Locked`
  - `Unavailable`
- Legacy `Hidden`/`Revealed` progress states should be migrated into `visibility` plus one of the progress states above.
- `map_progression.available_node_ids` and `map_progression.completed_node_ids` must not be exposed as competing Unity-facing selection sources.
- Internal core may keep equivalent sets for progression calculation.

## In Scope

- Audit current map DTO types, snapshot builders, progression state, and node selection validation.
- Add/adjust Unity-facing DTO fields:
  - `map.map_template_id`
  - `map.nodes[*].id`
  - `map.nodes[*].slot_id`
  - `map.nodes[*].kind_id`
  - `map.nodes[*].category`
  - `map.nodes[*].state`
  - `map.nodes[*].visibility`
  - `map.edges[*].from_node_id`
  - `map.edges[*].to_node_id`
  - `map.edges[*].direction`
- Ensure `viewing_map` and `node_confirm` snapshots both include the complete facility map DTO.
- Ensure `node_confirm` preserves current location and only records selected preview separately.
- Implement or wire map template id selection.
- Implement or wire node-to-slot assignment.
- Validate slot assignment:
  - missing/empty `slot_id` is an error
  - duplicate `slot_id` in one map is an error
  - `slot_id` not present in selected template is an error, if template slot catalog is available to core
- Validate edge endpoints and direction:
  - endpoints reference existing node ids
  - every edge has explicit `direction`
  - direction is known
- Make selection/action validation follow `state == Available && visibility != Concealed`, not `available_node_ids` from DTO.
- Make progression availability follow explicit edge direction and completed-node transit.
- Enforce locked leaf policy or fail data validation if a locked node has child/progression nodes behind it.
- Update relevant canonical docs/probe docs after runtime shape is final.
- Update live smoke/probe to assert the new map DTO shape for `viewing_map` and `node_confirm`.
- Add focused tests for Unity-facing JSON shape, progression/selectability, slot/edge validation, and node_confirm/current-location behavior.

## Out Of Scope

- Unity rendering implementation.
- Unity facility template art or slot coordinate authoring.
- New map art, room illustrations, icons, animations, fog effects, or hover UX.
- New gameplay rewards or node kinds.
- Full condition system design for every future locked room.
- One-way edge gameplay content beyond the explicit DTO and traversal rule.
- Backward compatibility layer that lets Unity infer positions from old row/column tree DTOs.
- Keeping dual public selection sources in Unity-facing snapshots.

## Target DTO Shape

```json
{
  "game_state": "viewing_map",
  "game_state_context": { "type": "viewing_map" },
  "map_progression": {
    "current_node_id": "act1_entrance"
  },
  "map": {
    "map_template_id": "act_01_floor_a",
    "nodes": [
      {
        "id": "act1_entrance",
        "slot_id": "entrance_lab",
        "kind_id": "entrance",
        "category": "Start",
        "state": "Completed",
        "visibility": "Revealed"
      },
      {
        "id": "act1_lab_a",
        "slot_id": "archive_room",
        "kind_id": "combat_low",
        "category": "Combat",
        "state": "Available",
        "visibility": "Obscured"
      }
    ],
    "edges": [
      {
        "from_node_id": "act1_entrance",
        "to_node_id": "act1_lab_a",
        "direction": "Bidirectional"
      }
    ]
  }
}
```

Exact Rust type names may change after code reading, but the public JSON meaning must stay stable.

## Initial Code Reading Targets

Read before implementation:

- `src/game/map/types.rs`
  - map node DTOs, edge DTOs, node states, progression structs
- `src/game/map/progression.rs`
  - availability calculation, selection validation, completed/current-node behavior
- `src/game/world/snapshot.rs`
  - `viewing_map` and `node_confirm` snapshot construction
- `src/game/world/state.rs`
  - command handling around map node selection and node entry
- `src/game/world/helpers.rs`
  - game state transitions into node confirm and node entry
- `src/game/behavior.rs`
  - select/confirm behavior DTOs and result contracts
- live RON/data under `game_resources/data`
  - map generation inputs, node kinds, any act/floor/template data
- game server command/snapshot transport paths if JSON shape is injected or transformed outside core
- external Unity probe docs and smoke probe only after the core DTO shape is final

## Implementation Sketch

1. Audit current runtime and data:
   - document current DTO fields and progression state names in `EXPERIMENT_NOTES.md`
   - identify whether current map is generated as row/column tree, graph, or facility-compatible graph
   - identify whether template/slot data exists or must be introduced
2. Define battle-independent map DTO/domain types:
   - progress state enum
   - visibility enum
   - edge direction enum
   - node slot/template identifiers
3. Migrate snapshot DTO shape:
   - add `map_template_id`
   - add `slot_id`
   - add `visibility`
   - add edge `direction`
   - rename/shape edges to `from_node_id` and `to_node_id` if needed
4. Remove Unity-facing duplicate selection projections:
   - stop exposing `available_node_ids` / `completed_node_ids` publicly
   - keep internal equivalents if useful
5. Implement current-location semantics:
   - entrance starts as `Completed`
   - `current_node_id` starts at entrance
   - `node_confirm` does not move current location
   - `confirm_enter_node` success moves current location
6. Implement progression/selectability:
   - selectability uses `state == Available && visibility != Concealed`
   - availability follows edge direction
   - completed nodes can be used as transit
   - completed node content is not replayable
   - locked nodes are non-selectable and leaf-only
7. Implement validation:
   - all nodes present
   - unique ids
   - unique slot ids
   - valid slot ids against template catalog when available
   - valid edge endpoints
   - explicit valid edge direction
   - locked leaf rule
8. Update tests:
   - DTO shape tests for `viewing_map` and `node_confirm`
   - current-node transition tests
   - selectability tests for `Concealed`, `Obscured`, `Revealed`, `Locked`, `Completed`
   - edge direction traversal tests
   - completed transit tests
   - slot/edge validation tests
   - live RON loading tests if schema/data changes
9. Update docs and probe:
   - sync canonical docs after final JSON shape is proven by tests
   - update smoke probe to assert real live JSON
   - verify `viewing_map` and `node_confirm` snapshots

## Tests To Add Or Update

Focused tests should cover:

- `viewing_map` snapshot contains `map_template_id`, all nodes, slot ids, visibility, and directed edges.
- `node_confirm` snapshot contains the same map DTO and does not change `current_node_id`.
- Selecting a node requires `state == Available && visibility != Concealed`.
- `Obscured + Available` is selectable.
- `Concealed + Available` is not selectable.
- `Locked` is not selectable.
- `Completed` is not selectable for replay.
- Completed nodes can be traversed as transit when calculating reachable available nodes.
- `ForwardOnly` edge allows traversal only from `from_node_id` to `to_node_id`.
- `Bidirectional` edge allows traversal both ways.
- Missing/duplicate/unknown slot ids fail validation.
- Missing/unknown edge endpoints fail validation.
- Missing/unknown edge direction fails validation.
- Locked nodes with child/progression nodes behind them fail validation.
- Public snapshot does not expose `map_progression.available_node_ids` or `completed_node_ids`.
- Live RON/data loads with the new schema.
- WebSocket smoke probe confirms real JSON shape.

Do not preserve tests that require Unity to infer slot positions from row/column order or rely on `map_progression.available_node_ids` as the public selection source.

## Stop Conditions

Stop, mark the goal complete for policy handoff, and ask the user before proceeding if implementation discovers:

- Current map generation fundamentally cannot assign every node to a stable facility slot without a new authored template/data model.
- Unity's local template slot catalog is unavailable to core and there is no agreed place to validate unknown slot ids.
- Existing save data needs migration for map graph/current-node/template/slot state.
- A locked room with child nodes is required by existing content and cannot be removed without a gameplay decision.
- `Hidden`/`Revealed` legacy state migration would change user-visible progression semantics.
- A compatibility layer, dual DTO shape, or fallback slot inference appears necessary.
- Existing Unity requires `available_node_ids` to remain public during transition.
- A new condition system is required to decide locked-node availability.

## Completion Criteria

This goal is complete when:

- Core runtime emits the target Node Map DTO in `viewing_map` and `node_confirm`.
- Node selection and entry follow the documented current-location, visibility, state, transit, locked, and edge-direction rules.
- Slot and edge validation catches contract-breaking data.
- Legacy Unity-facing map selection projections are removed or no longer exposed.
- Tests cover user-visible DTO shape and progression behavior.
- Live RON/data loading is validated if data/schema changes.
- Live WebSocket/probe validation confirms the real JSON shape.
- Relevant docs and probes are synchronized.
- `EXPERIMENTS.md` records every failed attempt, focused test, cargo check, broad test, live RON check, and probe result.

## Completion Status

Completed on 2026-06-25.

- Core now emits `map_template_id`, per-node `slot_id`, per-node `visibility`, and explicit `from_node_id` / `to_node_id` / `direction` edges.
- Public snapshots no longer expose map selection through `map.current_node_id`, `map.available_node_ids`, `map.completed_node_ids`, `map_progression.available_node_ids`, or `map_progression.completed_node_ids`.
- `map_progression.current_node_id` is the current exploration location. `node_confirm` keeps this location stable while previewing the selected node.
- Runtime selectability is fixed to `state == Available && visibility != Concealed`.
- Explicit edge direction, completed-node transit, locked-node leaf validation, and deterministic first-template slot validation are covered by tests.
- `RunMap.edges` is now the only map graph connection source of truth. Legacy per-node `outgoing` connections were removed from `MapNode`, generator, progression, and fixtures.
- The first implementation uses `map_template_id = "act_01_floor_a"` and deterministic `slot_id = depth_XX_lane_YY`. A richer authored facility slot catalog remains a follow-up candidate.
- External Unity smoke docs and `unity_ws_smoke_probe.py` were synchronized and live-probed against the running server.
