# Unity Range Preview Final Tiles Contract Plan

## Objective

Refactor the core-to-Unity range preview contract so Unity no longer combines intermediate fields such as `defense_tile_range`, `effective_weapon_profile`, `effective_basic_attack`, or skill catalog range metadata to calculate attack overlays.

Core must resolve the final preview cells and expose them through Unity-facing DTOs:

- Basic attack preview cells.
- Active skill preview cells, kept separate from basic attack.
- Availability/debug metadata that explains why a range is available or empty.

`defense_tile_range` remains the RON/source-data authoring format inside core. It is not the Unity-facing source of truth for drawing overlays.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code.
2. Live RON/data.
3. Unity-facing snapshot/command/update contracts.
4. Latest policy docs.

Do not trust this plan over runtime code. If code reading shows a better long-term shape, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Observed Structure

Pre-goal code reading found:

- `UnitCombatProfile::employee_default()` currently defines a fallback basic attack `defense_tile_range` with one forward tile but `include_anchor_tile: false`.
- The target policy now requires fallback basic attack range to include the caster's own tile and one forward tile when no weapon or skill-specific range is available.
- `UnitCombatProfile::apply_weapon_profile()` copies `weapon_profile.defense_tile_range` into `basic_attack.defense_tile_range`, so equipped weapons can change the basic attack range.
- `SkillFragmentEffectDef::BasicAttackModifier` currently changes attack and attack interval only. It does not change range.
- `SkillFragmentEffectDef::ActiveSkill` grants `profile.skill_id` and `SkillActivationMode::Manual`; active skill range comes from the skill definition, not by mutating basic attack range.
- `SkillDef.cast_target_definition()` exposes cast range from either the first step or explicit cast targeting.
- Skill steps may use `DeliveryDef::TileArea`; actual affected tiles are based on `defense_tile_range` plus area origin/anchor policy.
- `ActivateSkill` already accepts `target: Option<SkillCastTarget>`, and `SkillCastTarget` can be `Unit` or `Tile`.
- Live skill RON currently uses unit/self/previous-target style targeting. Manual tile targeting is possible in engine shape, but should not become a required Unity UX path until a skill explicitly needs it.

## In Scope

- Define a Unity-facing range preview DTO shape where final cells are authoritative.
- Implement core-side range resolution helpers for:
  - deployed live units by `unit_instance_id`
  - pending deployment previews by `employee_uuid + position + facing`
- Keep basic attack and active skill previews separate.
- Apply the default basic attack fallback policy: own tile plus one forward tile.
- Include facing-aware tile projection.
- Filter final cells against the active battlefield's valid tile set and any current blocking/terrain policy that runtime targeting already enforces.
- Preserve `defense_tile_range` as internal/RON authoring data.
- Update Unity-facing contract docs so Unity reads final cells only.
- Add tests around user-visible behavior and Unity-facing DTO shape.

## Out Of Scope

- Full manual tile-targeting UX.
- New ground-targeted skills.
- New skill balance, weapon balance, or range pattern authoring beyond the default fallback correction.
- Unity rendering implementation.
- Removing `defense_tile_range` from RON or skill authoring.
- Compatibility layers that keep old Unity range calculation paths alive indefinitely.

## Target Policy

### Contract Split

Internal/source data:

- `defense_tile_range`
- weapon profiles
- skill catalog definitions
- active skill step definitions

Core runtime:

- resolves those inputs with current position, facing, battlefield validity, weapon/equipment state, active skill fragment state, and fallback policy

Unity-facing DTO:

- final cell lists only
- availability/reason metadata
- optional debug source metadata

Unity must not combine internal range fields to compute attack overlays.

### Preview Categories

Basic attack and active skill preview must be separate:

```json
{
  "range_previews": {
    "basic_attack": {
      "available": true,
      "reason": null,
      "cells": [
        { "x": 3, "y": 4 },
        { "x": 4, "y": 4 }
      ],
      "source": "FallbackBasicAttack",
      "source_id": null
    },
    "active_skill": {
      "available": true,
      "reason": null,
      "skill_id": "fragment_freischutz_black_round",
      "targeting_kind": "auto_unit",
      "requires_manual_target": false,
      "cast_cells": [],
      "effect_preview_cells": [],
      "source": "SkillDefinition",
      "source_id": "fragment_freischutz_black_round"
    }
  }
}
```

The exact Rust type names may differ after reading the DTO code, but the final contract must keep these meanings.

### Basic Attack Fallback

When no weapon-specific range and no other basic attack range source exists, player basic attack preview uses:

- caster's own tile
- one tile in the current facing direction

This fallback must be implemented in core and tested through Unity-facing DTO output.

### Active Skill Preview

Active skill preview is based on the currently equipped/available active skill fragment:

- `targeting_kind: "self"` for self skills
- `targeting_kind: "auto_unit"` for automatic unit-targeting skills
- `targeting_kind: "manual_tile"` only for future skills that explicitly require manual tile targeting

Current implementation should leave manual tile targeting possible in schema/policy, but should not require Unity to implement it for current live skills.

### Cast Cells And Effect Cells

Definitions:

- `cast_cells`: cells the player may select as a cast anchor when a skill explicitly requires manual tile targeting.
- `effect_preview_cells`: cells expected to be affected by the skill preview. For auto-targeted or target-dependent skills, this may be empty or marked unavailable until a concrete target exists.

Do not pretend target-dependent effect areas are fixed when core cannot resolve them without an actual runtime target.

### Availability / Reason

Every preview block should tell Unity whether it can be drawn:

- `available: true` means `cells`, `cast_cells`, or `effect_preview_cells` contain meaningful preview data.
- `available: false` with `reason` explains why no preview exists.

Suggested reasons:

- `NoSkill`
- `MissingRangeData`
- `RequiresRuntimeTarget`
- `InvalidPlacement`
- `NoValidCells`

Use exact enum/string names after checking existing DTO style.

### Debug Source Metadata

Unity must not calculate with debug source fields, but source metadata is useful for logs:

- `source`: `FallbackBasicAttack`, `WeaponProfile`, `SkillDefinition`, `Unavailable`
- `source_id`: weapon id, skill id, or null

## Initial Code Reading Targets

Start by reading:

- `src/game/battle/types.rs`
  - `UnitCombatProfile::employee_default`
  - `UnitCombatProfile::apply_weapon_profile`
- `src/game/skill_fragment.rs`
  - `SkillFragmentLoadout::apply_to_profile`
- `src/game/ability.rs`
  - `SkillCastTargetingDef`
  - `SkillTarget`
  - `SkillStepDef`
  - `DeliveryDef::TileArea`
- `src/game/battle/tile_range.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/sim.rs`
  - manual skill validation
  - explicit `SkillCastTarget::Tile` handling
- `src/game/world/snapshot.rs`
  - current Unity-facing employee/combat profile fields
- `src/game/behavior.rs`
  - battle setup/update DTO types
- `src/game/world/combat.rs`
  - deploy/activate skill command handling
- live RON:
  - `/mnt/f/work/simulator/game_resources/data/equipments/base.ron`
  - `/mnt/f/work/simulator/game_resources/data/skills/base.ron`
  - `/mnt/f/work/simulator/game_resources/data/skill_fragments`

## Implementation Sketch

1. Audit existing range calculations and DTO consumers.
2. Design shared range resolution helper(s) that accept:
   - battlefield/template validity
   - origin cell
   - facing
   - range pattern source
   - source metadata
3. Implement fallback basic attack pattern as core policy.
4. Add DTO types for range previews.
5. Add range previews to the relevant Unity-facing surfaces:
   - pending deployment preview/state if such a DTO already exists
   - live deployed unit checkpoint or unit state DTO
   - selected/current battle state DTO if that is the existing Unity surface
6. Keep old intermediate fields only if they are still needed for catalog/debug, but document that Unity overlay rendering must not consume them.
7. Update contract docs and README references if needed.
8. Add behavior tests before broad checks.

## Tests To Add Or Update

Focused tests should cover:

- No weapon/default player basic attack preview returns own tile plus one forward tile.
- Facing rotates the preview cells correctly.
- Invalid/out-of-bounds cells are removed.
- Weapon range overrides the fallback basic attack preview.
- Active skill preview is separate from basic attack preview.
- Active skill without fixed effect preview reports a clear unavailable/requires-target reason rather than an empty ambiguous range.
- Existing live RON skill/equipment data loads and validates.
- Unity-facing JSON/DTO shape contains final cells and does not require `effective_weapon_profile` to draw default range.

Replace legacy tests that assert Unity should calculate from `defense_tile_range`; do not preserve them as ignored tests.

## Completion Criteria

- [x] Core exposes final range preview cells in Unity-facing DTOs.
- [x] Unity-facing contract docs identify final range cells as the overlay source of truth.
- [x] `defense_tile_range` remains internal/RON authoring data, not Unity overlay authority.
- [x] Basic attack fallback policy is implemented and tested.
- [x] Active skill preview is represented separately from basic attack.
- [x] Manual tile targeting remains a future explicit mode, not a current required UX path.
- [x] No compatibility layer or dual range-calculation contract is introduced without documented user approval.
- [x] Focused tests and a final broad validation command have been run.

## Required Completion Report

When implementation finishes, report:

- Changed DTOs and their JSON meaning.
- Removed or downgraded legacy Unity range dependencies.
- New fixed policies.
- Tests added/updated.
- Remaining risks.
- Exact validation commands run.
