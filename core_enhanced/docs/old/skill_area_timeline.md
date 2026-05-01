# Skill Area Timeline Contract

## Goal

Unity must be able to visualize skill boundaries without reconstructing server
targeting logic from `skill_id`, `step_id`, or client-side catalogs.

The server timeline is the source of truth for:

- when an area is declared
- which gameplay area instance it represents
- the resolved shape and dimensions
- resolved origin, center, and direction hint in simulation units
- gameplay duration versus presentation duration
- which target filter/tick policy produced later effects

## Event Flow

Area-delivered skills now emit this sequence:

1. `AbilityStepTriggered`
2. `SkillAreaDeclared`
3. `HpChanged` / `StatChanged` / other outcomes

For area damage, outcome events use `SkillAreaDeclared` as their parent cause
when possible. This lets playback associate damage text with the declared
boundary instance directly instead of inferring from `skill_id`.

## `SkillAreaDeclared`

Fields:

- `area_id`: stable per-delivery UUID for correlating area visuals and outcomes.
- `skill_id`, `step_id`: debug/catalog references, not required for geometry.
- `caster_instance_id`: unit that declared the area.
- `target`: optional original step target, either unit or tile.
- `shape`: Unity-friendly tagged shape DTO.
- `origin`: sweep start for line/rectangle/cone shapes.
- `center`: center point for circle/box shapes.
- `direction_hint`: absolute aim point. Directional visuals should use
  `direction_hint - origin`.
- `start_time_ms`: declaration time.
- `duration_ms`: gameplay lifetime. Instant areas remain `0`.
- `display_duration_ms`: visual lifetime. Instant areas get a short non-zero
  default so debug visuals are visible without changing simulation rules.
- `warning_ms`: warning/telegraph time. Currently `0`; reserved for data-driven
  telegraphs.
- `tick_interval_ms`, `tick_policy`, `hit_targets`, `include_caster`: gameplay
  semantics exposed for debugging and deterministic playback.

## Shape DTO

Shapes serialize as internally tagged JSON:

```json
{ "type": "circle", "radius_units": 1100000 }
```

Supported shape types:

- `circle`
- `line`
- `box`
- `rectangle`
- `cone`

## Unity Playback Rule

Unity should create or update area visuals from `SkillAreaDeclared` only.
`HpChanged` is for HP bars and damage text. `skill_id` and `step_id` may select
polished VFX later, but they must not be required to derive area geometry.
