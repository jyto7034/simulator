# Weapon Archetype Targeting Experiment Notes

## Findings

- Do not weaken `EquipmentDatabase::validate_weapon_profiles`; the failures are useful and point to unmigrated weapon data.
- Test helpers should become profile-aware instead of each test opting out with `weapon_profile: None`.
- Live RON must carry explicit weapon profiles. A hidden default would make Unity previews and balance auditing harder.
- Player basic attack target hints/current target persistence are now fallback behavior after weapon-profile selection, except melee blocked targets remain first priority.
- `SplashClusterFirst` is only direct-target selection. It scores nearby valid enemies around each candidate; it does not itself apply area damage.
- Skill tests should follow explicit skill data. `Spider Bud` uses `EnemySingle(rule: Nearest)`, so the old current-target test expectation was stale.
- The next Skill Fragment Compatibility goal can use `effective_weapon_profile` as the runtime/UI source of truth.
