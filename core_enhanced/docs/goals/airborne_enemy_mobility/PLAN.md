# Airborne Enemy Mobility Plan

## Objective

Implement airborne enemies as a core mobility policy, with authored `mobility_kind` as the source of truth for terrain immunity, unblockability, targetability, preview warnings, and Unity-facing display.

## Completion Conditions

- Enemy metadata/profile/runtime carries `mobility_kind: Ground | Airborne`.
- Airborne enemies map to `MovementTerrainPolicy::Airborne`.
- Airborne enemies follow authored route waypoints without static obstacle/void/walkability correction and do not move beyond route end.
- Airborne enemies cannot enter block state.
- Airborne enemy basic attack chooses protected target in range first, otherwise nearest live player combat unit.
- Single-target basic attacks/skills require explicit air capability to target airborne enemies.
- Area skill hits include airborne enemies by projected 2D position.
- Combat preview can emit `air_enemy_possible`.
- Unity-facing snapshot/contract exposes enough mobility data for display without Unity recomputation.
- If code review exposes a policy that must be decided with the user, stop the goal.

## Status

- Complete.

## Completion Summary

- Added `MobilityKind::{Ground, Airborne}` as the enemy/profile/runtime source of truth.
- `AbnormalityMetadata.mobility_kind` defaults to `ground`; `target_traits: [Airborne]` is rejected so the legacy/duplicate source does not return.
- `UnitCombatProfile`, `RuntimeUnit`, `UnitSnapshot`, and `TimelineEvent::UnitSpawned` carry `mobility_kind`.
- Airborne enemy runtime units map to `MovementTerrainPolicy::Airborne`, skip block matching, and derive display `UnitTargetTrait::Airborne` from mobility.
- Airborne enemy route behavior keeps authored route movement, attacks protected target in range first, then nearest live player combatant.
- Single-target basic attacks and single-target skill targeting use `air_capable`; persisted/current targets are revalidated at resolution.
- Tile/area skill delivery continues to include airborne units by projected 2D tile position.
- `CombatPreview` emits `air_enemy_possible` from enemy mobility.
- Live representative data marks `Punishing Bird` as `mobility_kind: airborne`.
- Canonical Unity docs in `F:\unity projects\ark\docs` were updated with `mobility_kind` and `air_capable` DTO fields.

## Verification

- `cargo test -p game_core airborne -- --nocapture`: passed.
- `cargo test -p game_core movement -- --nocapture`: passed.
- `cargo test -p game_core blocking -- --nocapture`: passed, but the filter currently matches only 2 tests; airborne block coverage is in the movement/airborne test groups.
- `cargo test -p game_core combat_preview -- --nocapture`: passed.
- `cargo test -p game_core --test ron_loading live_abnormality_ron_reads_airborne_mobility_kind -- --nocapture`: passed.
- `cargo check -p game_core`: passed.
- `cargo check -p game_server`: passed.
- `cargo test -p game_core --test ron_loading -- --nocapture`: failed on pre-existing/next-goal live equipment `weapon_profile` validation (`justitia` and related weapon fixtures), not on airborne RON schema.
- `cargo test -p game_core -- --nocapture`: failed on the same weapon-profile fixture/data debt plus the same full live RON validation path. This should be handled by the Weapon Archetype and Targeting Profile goal rather than papered over here.
