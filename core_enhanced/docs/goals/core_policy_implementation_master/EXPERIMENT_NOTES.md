# Core Policy Implementation Master Experiment Notes

## 2026-06-07

- The worktree contains many pre-existing modified, deleted, and untracked files across core, resources, and server. Do not revert or normalize unrelated changes.
- Local `docs/unity_core_contract.md` and `docs/unity_client_implementation_goal.md` may be stale copies. The external ark docs are the canonical Unity-facing docs.
- Airborne enemies are now a dedicated prerequisite goal. `mobility_kind` is the source of truth; do not introduce a parallel `target_traits: [Airborne]` schema unless a later policy requires broader target traits.
- Weapon `AirFirst` and `air_capable` must use the Airborne Enemy Mobility goal's runtime policy instead of redefining flying/air rules in equipment code.
- Airborne movement must follow authored route waypoint/polyline order while ignoring terrain and walkability. Do not reinterpret "ignores terrain" as direct start-to-end straight flight.
- Airborne route endpoints must be validated against protected-target basic attack range to avoid an enemy waiting forever outside meaningful range.
- Movement Backend Policy Refactor is now the prerequisite for Airborne Enemy Mobility. Rapier/backend physics must stay a ground static obstacle correction helper, not a source of truth for blocking, targetability, route progress, deploy occupancy, or airborne terrain immunity.
- Do not implement airborne terrain immunity as a hidden Rapier exception. Add typed movement/collision policy to movement input/runtime path first.
- Goal 6 implemented `MovementTerrainPolicy` in `MovementUnitInput`; Airborne Enemy Mobility can now attach real enemy mobility data to this path instead of adding Rapier exceptions.
- Runtime board bounds are now core clamp's responsibility. Rapier wall colliders are not synced by runtime tick.
- `RuntimeUnit` still has no authored mobility field. Goal 8 should add the enemy/data/runtime source of truth and map `Airborne` enemies to `MovementTerrainPolicy::Airborne`.
- Inserted Rapier Backend Quality Refactor as the new goal 7 before Airborne Enemy Mobility. The previous Airborne goal shifts to goal 8.
- Rapier backend quality work should harden/refactor backend internals without changing gameplay policy. If review suggests removing Rapier entirely, stop and ask the user.
- Goal 7 completed without requiring Rapier removal or new user policy decisions.
- Rapier ground correction now ignores unit colliders and board wall colliders, and KCC slide is disabled so authored static obstacle blockers surface as blocked/corrected movement instead of backend-created detours.
- Goal 10 Skill Fragment Compatibility started. Created `docs/goals/skill_fragment_compatibility/*` work memory.
- Skill Fragment Compatibility code read found a real policy gate: active fragment equip with no weapon, and weapon replacement that invalidates an already equipped active fragment. Recommended policy is to disallow no-weapon active fragment equip and block incompatible weapon replacement until the active fragment is unequipped/replaced.
- Skill Fragment Compatibility also found a required structural cleanup: actual battle deployment applies equipment weapon profiles via `BattleUnitDraft::combat_profile`, but Safezone/roster snapshot and fragment validation paths do not share that effective-profile calculation. Goal 10 should consolidate or share this path before adding compatibility checks.
- Goal 10 policy gates resolved by user: weapons are default-provided, active fragment equip fails closed if no effective weapon profile exists, incompatible weapon replacement is blocked, and snapshot/command use stable reason codes.
- Goal 11 found no current data/runtime support for permanent type immunity metadata. Do not add boss immunity exceptions until a concrete mechanic needs them and the user confirms the schema.
- Goal 11 consolidated existing AD/AP thresholds into `combat_balance`; future preview warnings, damage feedback, and validation should use that module instead of local numeric literals.
- Master checklist is complete as of AD/AP Balance Validation.
