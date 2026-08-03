# Weapon Block Capacity Authoring Contract

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G04

Finding: `DCA2-GAME-001`

Policy: Master P01

## Objective

Make the equipped weapon's explicit RON-authored `block_capacity` the sole base authority for a
player employee's battle block capacity, with no class, armor, aptitude or archetype fallback.

## Approved Policy

- Every current weapon RON entry explicitly authors the value.
- Review guideline: Shield 3; Sword/Spear 1; Bow/Gun/Staff 0; no weapon 0.
- Individual deviations require a separate content/balance rationale; this child does not invent
  them.
- Armor and employee `CombatAptitude` do not contribute.
- Runtime must not infer the value from weapon archetype/range role.
- Temporary skill/buff block modifiers are not introduced by this goal.

## In Scope

- Required strict weapon profile field and validation.
- Migration of every current live weapon entry with explicit reviewed values.
- Equipped weapon to battle draft/runtime final block-capacity projection.
- Unarmed zero behavior and elimination of duplicate employee/armor authority if present.
- DTO/snapshot/preview consistency if the existing public surface exposes block capacity.
- Focused representative tests for all six current archetypes, unarmed, armor independence and
  weapon replacement.
- Gameplay/data/external wire canonical wording owned by this behavior.

## Out Of Scope

- New weapon content or per-item balance deviations from the approved guideline.
- Skill/buff block modifiers and block-combat redesign.
- GrenadeLauncher, removed by G06.
- Unrelated equipment stat pipeline changes.

## Implementation Plan

1. [x] Trace current block-capacity authority from employee/loadout through battle draft/runtime/DTO.
2. [x] Add a required strict weapon field with no serde/runtime fallback.
3. [x] Author every current weapon RON value from the approved archetype guideline.
4. [x] Project equipped weapon value exactly and keep unarmed at zero.
5. [x] Add representative schema/gameplay/projection regressions and correct nearest canonical docs.
6. [x] Run focused data/equipment, server, Core broad and static validation.

## Completion Conditions

- Every live weapon has an explicit value; missing field loading fails.
- Runtime uses the authored field exactly and contains no archetype fallback.
- Shield/Sword/Spear/Bow/Gun/Staff and unarmed representative assertions pass.
- Armor-only changes do not change block capacity; weapon replacement does.
- Any exposed DTO uses the same Core-computed value.
- Relevant focused and broad tests pass.

All completion conditions satisfied on 2026-07-18.

## Result

- `WeaponCombatProfile.block_capacity` is required during deserialization and is serialized through
  the existing inventory/roster weapon-profile DTO surface.
- All 29 live weapons explicitly author the approved value.
- Employee default/unarmed capacity is 0; `apply_weapon_profile` copies the authored value exactly.
- Armor and employee profile do not contribute; weapon replacement updates the next authoritative
  profile/draft.
- No runtime archetype fallback was added.

## Stop Conditions

- A live weapon requires a value different from the approved guideline without existing rationale.
- Existing shipped compatibility requires a missing-field default or duplicate authority.
