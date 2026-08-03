# Weapon Block Capacity Authoring Contract Experiments

Status: complete

## 2026-07-18 — Child Initialization

- Read Master G04 scope, completion gate and approved P01 after G06 completion.
- Confirmed the child path was available and no child dirty state existed.
- Mandatory child documents created before authority/data/test search or mutation.

## 2026-07-18 — Weapon-Only Block Authority

### Authority trace

- Found employee defaults hard-coded at block capacity 1 and weapon application that copied attack
  fields but not block capacity.
- Battle drafts already resolve exactly one equipped weapon and apply its `WeaponCombatProfile`, so
  that method is the narrow single-authority integration point.
- Armor metadata/stat effects and employee aptitude had no block-capacity mutation path.
- Public roster/inventory DTOs already serialize `WeaponCombatProfile`; no new top-level wire field or
  server mapper was required.

### Implementation and migration

- Added required `WeaponCombatProfile.block_capacity` with no serde default.
- Changed employee default/unarmed capacity from 1 to 0.
- `apply_weapon_profile` now copies the authored field exactly.
- Authored all 29 live weapons: Shield 3, Sword/Spear 1, Bow/Gun/Staff 0.
- Added a production-shaped missing-field parse regression and a live full-catalog guideline audit.
- Added an end-to-end loadout/profile/snapshot test covering six archetypes, unarmed employees,
  sequential weapon replacement and armor replacement independence.
- Corrected the gameplay rulebook, content catalog/index and external Unity contract. The external
  contract was patched from its exact dirty version and copied back as a reviewed minimal diff.

### Validation and corrections

- Focused missing-field, live authoring and end-to-end projection tests passed.
- First Core broad run exposed two legitimate old assumptions: employee default expected 1 and a
  skill-fragment compatibility fixture relied on that implicit 1. Updated the former to explicit
  unarmed zero and the latter to explicitly author its test weapon capacity; both focused reruns
  passed.
- Equipment module: 27 passed.
- `ron_loading`: 21 passed.
- Final Core library: 741 passed, 0 failed, 0 ignored.
- game_server: 30 passed.
- `cargo fmt --all -- --check` and Core/external diff checks passed.

### Disposition

- `DCA2-GAME-001` and P01 complete.
