# Node Map Facility Template Contract Notes

## Working Decisions

- Node Map should feel like exploring rooms inside a fixed facility template, not choosing from a free-floating abstract tree.
- Core owns run graph structure, node assignment, state, visibility, traversal, current location, and selectability.
- Unity owns local template art, slot coordinates, corridor visuals, blocked/contaminated zone art, hover/selection rendering, and preview UI.
- Every run graph node remains present in the DTO even if Unity hides it visually.
- Current location is represented by `map_progression.current_node_id`, not by a `Current` node state.
- Entrance is a normal node, starts completed, and is the initial current node.
- `node_confirm` previews a selected node but does not move current location.
- Actual movement happens only after `confirm_enter_node` succeeds.
- Selection requires `state == Available && visibility != Concealed`.
- `Obscured` hides type/details but can be selected if available.
- `Concealed` is present in data but hidden in Unity and not selectable.
- Completed nodes are transit only and cannot replay content.
- Locked nodes require a condition, are non-selectable while locked, and must be leaf rooms.
- Edge direction is explicit. Every edge must be either `Bidirectional` or `ForwardOnly`.

## Source-Of-Truth Checks To Perform

- Confirm current map DTO shape in runtime code before editing it.
- Confirm whether `Hidden`/`Revealed` are currently node states, visibility concepts, or both.
- Confirm whether current progression uses `available_node_ids` / `completed_node_ids` as runtime internals, Unity-facing fields, or both.
- Confirm how `node_confirm` stores the selected node and whether current node already remains stable.
- Confirm where map graph edges are generated and whether they are currently directed or undirected.
- Confirm whether a template id / slot id source already exists in live data.
- Confirm whether Unity's slot catalog can be validated from core-side data or only by live probe/Unity validation.
- Confirm whether existing save data contains map graph/current-node fields that need migration.

## Code Review Findings Before Implementation

- `src/game/map/types.rs` currently mixes progress and visibility in `MapNodeState`: `Hidden`, `Revealed`, `Available`, `Completed`.
- `MapNodeDto` currently exposes `id`, `depth`, `lane`, `kind_id`, `category`, `state`, and `payload`; it does not expose `slot_id` or `visibility`.
- `MapEdgeDto` currently exposes `from` and `to`; it does not expose `from_node_id`, `to_node_id`, or explicit `direction`.
- `MapViewDto` currently exposes `act_index`, `max_acts`, `nodes`, `edges`, `current_node_id`, and `boss_node_id`; it does not expose `map_template_id`.
- `MapProgression` currently keeps `current_node_id`, `available_node_ids`, and `completed_node_ids` internally. The public snapshot already strips `available_node_ids` and `completed_node_ids` from `map_progression`, but runtime selection still uses the internal `available_node_ids`.
- Current progression starts with `current_node_id: None` and first playable row as available. The target contract requires an entrance node to be the initial `current_node_id`.
- Current completion clears `current_node_id` to `None` after completing a node. The target contract requires current location to remain at the completed node so facility adjacency/transit can be derived from the party's location.
- Current generator is still depth/lane based. It can be bridged into the target DTO by making core assign stable facility slot ids from the generated depth/lane coordinates for the first implementation.
- `../game_resources/data/map` contains node generation and battlefield data, but no dedicated facility node-map template/slot catalog file yet.
- This does not yet require stopping: `MapGenerationConfig::default()` has a bounded shape (`depth_count: 8`, `max_width: 5`), so core can introduce a first `act_01_floor_a` slot catalog covering generated depth/lane slots and validate against it. A richer authored template data model remains a follow-up candidate unless code proves this bridge cannot satisfy runtime/Unity needs.

## Implementation Notes

- Added `MapNodeVisibility` so progression state and discovery/presentation state are no longer mixed in `MapNodeState`.
- Added explicit `RunMap.edges` with `Bidirectional` / `ForwardOnly` direction. DTO edge fields now use `from_node_id`, `to_node_id`, and `direction`; traversal uses the same explicit edge model.
- Post-completion review found that `MapNode.outgoing` still duplicated `RunMap.edges` as an internal graph source. The follow-up correction removed `MapNode.outgoing`; map generation now builds `RunMap.edges` directly and progression reads only edge traversal.
- Added `MapTemplateId` and `MapSlotId`. The first implementation uses `act_01_floor_a` and deterministic slot ids of the form `depth_XX_lane_YY`.
- Added core validation that rejects duplicate node ids, empty/duplicate slot ids, slot ids that do not match the first implementation's depth/lane slot policy, missing edge endpoints, and locked nodes with outgoing edges.
- `MapProgression` still keeps internal available/completed lists for calculation, but public snapshots expose selectability through `map.nodes[*].state`.
- Current location now starts at the completed entrance node and remains at the completed room after node completion. Newly reachable rooms become `Available`; concealed available rooms are revealed as `Obscured`.
- `enter_node` permits idempotent reentry only for the unresolved current room (`current_node_id == node_id`, `Unavailable`, `Revealed`). This preserves setup-loss/retreat recovery without making arbitrary unavailable nodes selectable.

## Questions To Stop For

Stop and ask the user if any of these appear:

- A new authored template/slot data model is needed before core can assign every node to a stable slot.
- Save migration is required.
- Existing content depends on a locked room having child/progression nodes behind it.
- Existing content depends on a concealed node being selectable.
- Unity needs a temporary fallback from node order/category to slot coordinates.
- A transition period with both old and new map DTO shapes appears necessary.
- Map visibility rules require rewards, danger level, or node kind to be hidden separately instead of one `visibility` enum.
- One-way edge behavior needs more than `ForwardOnly` and `Bidirectional`.

## Follow-Up Candidates Outside This Goal

- Unity facility map art polish, fog/discovery animation, route highlighting, and blocked-zone presentation.
- Rich locked-room condition UI and condition-source tooltips.
- Authoring tools for map templates, slot catalogs, and route previews.
- Additional edge types such as temporarily blocked, collapsing, or conditionally one-way corridors.
- Per-node visibility reveal animations or memory of previously seen unknown rooms.
