# Experiment Notes

## Fixed Policy Summary

- Runtime must carry explicit gameplay `GameMode`.
- `Standard` is a 3-Act structure: Act 1-2 use Gate transition; Act 3 final boss victory clears the run.
- `Endless` is floor-based and uses Gate as next-floor source.
- Gate is an independent node; Elite/Boss nodes do not double as Gate.
- Gate confirmation is Unity-local. Core receives only the confirmed transition command.
- Gate transition reward: living employees recover trauma by `gate_transition_trauma_recovery_percent`, default 10. No HP healing, no checkpoint save.
- Start is a real `Completed + Revealed` node and the initial current node.
- Core exposes `map_navigation.current_node_id` and `map_navigation.selectable_node_ids`.
- Unity-facing DTO does not expose `map.edges` or `travel_path`.
- Completed nodes are transit-only and never rerun their original content/effect.
- Combat/elite/normal boss nodes have three total attempts. Exhaustion makes the node Completed/transit-only without success reward.
- Final boss defeat immediately fails the run. Final boss retreat before defeat is valid while attempts remain; third retreat/exhaustion fails the run.

## Initial Implementation Biases

- Prefer a typed `GameMode` in the map/run domain instead of stringly mode flags in world state.
- Prefer a `MapNavigationDto` next to `map` over stuffing `selectable` booleans into each node. This preserves the split between static-ish node display data and current-location-dependent navigation state.
- Keep internal `RunMap.edges`; remove only Unity-facing `MapViewDto.edges`.
- Do not keep public `available_node_ids`/`completed_node_ids` unless code proves another non-Unity consumer requires them. If another consumer exists, rename/scope it so it is not a Unity selection source.
- Do not add a `gate_confirm` game state. If current node flow wants a preview for Gate, it must still treat the Unity confirmation popup as local and only mutate core state after confirmed command.

## Known Policy/Data Questions To Watch

- How should runtime identify a final boss versus a normal boss? Current code has `CombatNodeType::Boss`, but the policy now distinguishes normal boss and final boss.
- Should `Gate` be a new `MapNodeCategory::Gate`, a `MapNodePayload`, or both? Prefer first-class category unless code shows category expansion is too disruptive.
- Does `ConfirmEnterNode` naturally cover confirmed Gate transition, or is a clearer `EnterGate`/`AdvanceFloor` command needed?
- How should `GameMode` be selected at new game start if the current command has no mode argument? Defaulting to `Standard` is likely correct, but if starter selection/new-game UX already has a policy hook, use that.
- Does checkpoint payload need versioning/migration after adding `GameMode`? If existing serialized checkpoint data is loaded in production, stop and ask.

## Implementation Judgments

- `GameMode` was added to `RunProgression` with `Standard` as the new-game default. Current checkpoint code stores `RunProgression` by in-memory clone and reloads the same type, so no external serialized checkpoint migration decision was found in the reviewed code.
- `Gate` is best represented as `MapNodeCategory::Gate`. A separate payload is not needed for the current policy because Gate behavior is category-driven and the first visual presentation is owned by Unity.
- `ConfirmEnterNode` can model confirmed Gate transition without adding a `gate_confirm` state. Unity can keep its confirmation popup local, then send the existing confirmed command; core validates selectability, applies Gate reward, and advances Act/Floor.
- Generated map terminal identity was split from boss identity. `RunMap` now carries an internal terminal endpoint id, while the Unity-facing map DTO does not expose boss or terminal ids. Standard Act 1-2 use an independent Gate terminal, and the final Standard Act uses a Boss terminal.

## Follow-Up Candidates Outside This Goal

- Authored semantic slot ids beyond `depth_XX_lane_YY`.
- Conditional/Locked/secret/Concealed Gate content.
- Full Endless boss omen generation.
- Unity Gate confirmation UI implementation.
- Visual route/path highlighting, if the design later reintroduces movement path presentation.
