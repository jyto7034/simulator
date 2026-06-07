# Skill Fragment Compatibility Plan

## Current State

- Active sub-goal: `docs/skill_fragment_compatibility_goal.md`.
- Parent master goal: `docs/core_policy_implementation_master_goal.md`.
- Canonical Unity docs live outside this repo:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- This goal must not preserve the old "all active skill fragments are freely equipable" policy if it conflicts with weapon-profile compatibility.

## Execution Guardrails

- Do not use `git restore`, `git reset`, or other git file modifier commands.
- Do not copy historical git contents into the current tree with `cp` without explicit user approval.
- Do not revert files to an older state without explaining why and asking the user first.
- Read runtime code and live data before trusting docs.
- If a policy decision is required, stop implementation and ask the user instead of silently choosing a UX/data contract.

## Current Findings

- `SkillFragmentMetadata` now exposes compatibility requirements, but runtime equip paths are not wired yet.
- `SkillFragmentLoadout::equip` validates ownership, metadata presence, active effect presence, and duplicate active slot policy, but not weapon compatibility.
- The actual battle deployment path applies equipment weapon profiles through `BattleUnitDraft::combat_profile`.
- Safezone/roster snapshot and fragment equip validation paths currently do not share that battle deployment effective-profile calculation.
- Therefore this goal needs a shared current/effective combat profile path, or at minimum a shared current weapon profile lookup, so validation and Unity-facing snapshot agree with actual battle runtime.

## Implemented Foundation

- Added `SkillFragmentCompatibilityRequirements` to `SkillFragmentMetadata`.
- Added typed requirement axes for weapon range role, archetype, damage type, targeting profile, air-capable flag, block capacity, required capability tags, and incompatible capability tags.
- Added `SkillFragmentCompatibilityContext`, `SkillFragmentCompatibilityReport`, and stable `SkillFragmentCompatibilityFailureCode` values.
- Added pure compatibility evaluator tests.
- Added static validation for compatibility requirements, including duplicate axes, zero block minimums, empty tags, duplicate tags, and required/incompatible tag conflicts.
- Added a shared effective employee combat profile helper that applies equipped weapon profiles through the same `BattleUnitDraft::combat_profile` path used by battle runtime.
- Updated roster snapshot `effective_*` fields to use the shared effective profile helper, so `effective_weapon_profile` now reflects equipped weapons.
- Added runtime active fragment compatibility validation for `equip_skill_fragment`.
- Added projected equipment-slot validation so equipment combination/weapon changes are rejected when they would invalidate the currently equipped active fragment.
- Added `skill_fragment_incompatible` command error mapping with stable failure codes.
- Added roster snapshot `skill_fragments.compatibility[*]` and inventory fragment `requirements`.
- Updated representative live skill fragment RON with weapon/profile requirements.
- Updated canonical Unity docs in `/mnt/f/unity projects/ark/docs`.

## Policy Questions Blocking Code Changes

1. No-weapon active fragment equip:
   - Recommended policy: disallow active skill fragment equip when the employee has no effective weapon profile.
   - Starter/basic fragment remains a baseline exception because it is not a player-selected active skill fragment.

2. Weapon change that invalidates the active fragment:
   - Recommended policy: block the weapon change and require the player to unequip or replace the active fragment first.
   - Avoid auto-unequip because it hides state changes from the player.
   - Avoid "equipped but inactive" because it makes command success and runtime behavior diverge.

3. Compatibility failure DTO shape:
   - Recommended policy: expose stable reason codes plus display-oriented requirement details in snapshot.
   - Command failure should return the same reason codes or a directly mappable error.

Implementation should continue only after these are confirmed or revised by the user.

## Proposed Implementation Shape

1. Add a nested compatibility requirement struct to skill fragment metadata. - done.
2. Represent requirement axes as typed fields. - done:
   - allowed range roles
   - allowed weapon archetypes
   - allowed damage types
   - allowed targeting profiles
   - required air-capable flag
   - minimum block capacity
   - required capability tags
   - incompatible capability tags
3. Add a compatibility evaluator that returns structured failure reasons. - done as a pure evaluator.
4. Use the same evaluator in command validation, actual equip path, and snapshot DTO generation. - done.
5. Make current effective weapon/profile lookup share as much logic as possible with battle deployment profile construction. - done for snapshot and command validation.
6. Validate compatibility requirements in static data loading. - done for local schema invariants; cross-content/live policy requirements pending.
7. Update live skill fragment RON with a small representative set. - done:
   - broad/generic fragments stay broadly compatible.
   - strong identity fragments can lock to weapon archetypes.
8. Update canonical Unity docs after DTO shape is implemented. - done.
9. Replace tests that encode free active-fragment equip with compatibility tests. - done for affected world tests.

## Verification Targets

- Focused unit tests for compatibility evaluator.
- Command-flow tests for compatible and incompatible equip.
- Snapshot tests proving Unity receives compatibility status and reasons.
- Live RON load/validation test.
- `cargo check -p game_core`.
- `cargo check -p game_server`.
