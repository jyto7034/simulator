# Grenade Launcher Schema Removal

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G06

Finding: `DCA2-BATTLE-002`

Policy: Master P06

## Objective

Remove `GrenadeLauncher` from every current runtime/data/wire authority while retaining it only as
an explicitly future, non-current weapon concept.

## Approved Policy

- Remove the current enum/schema variant; do not retain a compatibility alias or reserved current
  variant.
- Current parsing must reject the removed value.
- Preserve a future concept note only; a later implementation must jointly approve range,
  targeting, delivery and client presentation before reintroducing schema.

## In Scope

- Core enum variants, match arms, tests and fixtures that treat GrenadeLauncher as current.
- Current Core/external Unity enum inventories and targeting canonical affected by removal.
- Strict removed-value negative loading/serialization evidence.
- A clearly labeled future/non-current concept note.

## Out Of Scope

- Implementing grenade weapon content under another name.
- Compatibility aliases, save migration or deployed-client compatibility work.
- Weapon block values owned by G04.
- General targeting redesign.

## Implementation Plan

1. [x] Inventory exact current code/data/test/document references and dirty overlap.
2. [x] Remove current schema authority and repair exhaustive matches/fixtures.
3. [x] Add removed-value rejection evidence.
4. [x] Correct current canonical inventories and preserve explicit future concepts.
5. [x] Run focused loading, Core broad and relevant external/static validation.

## Completion Conditions

- Production/current contract searches contain no GrenadeLauncher authority.
- Current parser rejects `GrenadeLauncher` rather than aliasing/falling back.
- Future documentation is explicitly non-current and non-implemented.
- Relevant focused and broad tests pass.

All completion conditions satisfied on 2026-07-18.

## Result

- `WeaponArchetype::GrenadeLauncher` no longer exists; live data already used only the six retained
  archetypes.
- RON deserialization of `GrenadeLauncher` has a named fail-closed regression.
- Test helper exhaustive classification contains only the retained variants.
- Current Unity contract and current visual brief inventories no longer present the value as usable.
- Core and Unity visual/content docs retain only explicit future/non-current concept wording.

## Stop Conditions

- A current live RON entry or deployed compatibility contract requires the variant.
- Removal would require inventing replacement gameplay behavior.
