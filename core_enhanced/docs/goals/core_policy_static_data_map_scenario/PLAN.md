# Static Data, Map, Scenario Policy Implementation

## Objective

Move static world, map, and scenario behavior toward explicit live data and validation. Remove implicit runtime fallbacks where the policy requires authored data.

## Policies Covered

- `RUN_SYSTEM_POLICY data source`
- `battle_records export`
- `map authoring/generation policy source`
- `NodeSessionKind`
- `MapViewDto progression projection`
- `map node tags`
- `encounter-less combat preview and empty wave fallback`
- `battlefield archetype random selection source`
- `SplitRoom content policy`
- `FacilityEntity future schema`
- `authored defense route required`
- `starter employee loadout source`
- `remove legacy random event RON`
- `server startup live preview validation`

## Plan

1. Read runtime loaders, map generation/runtime code, scenario loaders, and live RON files before editing.
2. Identify duplicated source-of-truth state between map node structs, DTO projection, runtime session state, and live data.
3. Move policy-like lists, weights, enabled flags, and route requirements into authored data or versioned manifests.
4. Remove `NodeSessionKind`/map tags and replace them with typed fields or derived runtime state.
5. Require authored defense routes and reject invalid scenario content through validation.
6. Remove legacy random event RON from live loader paths.
7. Ensure generated combat previews are validated during startup or content audit.
8. Update tests around live RON loading, map DTOs, scenario validation, and generated preview validation.

## Completion Conditions

- Runtime does not infer map/scenario policy from removed tags or code-only lists.
- Empty combat waves and missing authored routes fail validation where policy requires failure.
- Starter loadouts are loaded from live data and no code injection remains.
- Legacy random event RON is deleted or isolated outside live loading.
- `MapViewDto` exposes progression from canonical per-node state.
- Map generation category weights, safe replacement categories, and row repair rules are sourced from live `map/generation_policy.ron`, not hard-coded generator constants.
- `battle_records/run_<seed>` JSON export remains an always-on debug artifact and is not promoted to gameplay source-of-truth, replay input, or golden-data contract.
- Focused tests and final relevant validation commands are recorded in `EXPERIMENTS.md`.
