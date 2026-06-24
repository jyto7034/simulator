# Experiment Notes

## Policy Notes

- Trust system work is deferred. If trust code blocks another confirmed policy, record the exact issue before touching it.
- High-tier fragment exclusivity means even multiple owned copies may have simultaneous equip limit one when the data says so.
- Dismantle is allowed; Unity should warn, runtime should deterministically repair loadout ownership invariants.
- Resolved implementation rule: employee `grade` is removed. Combat draft `Tier` remains a battle-facing value but is derived from `RunPolicyData.growth.battle_tier_by_level`, not employee candidate/runtime grade.
- Resolved implementation rule: post-battle survival XP belongs to `RunPolicyData.growth.post_battle_survival_xp`; post-battle trauma/run-HP loss stays in `post_battle`.
- Resolved implementation rule: `GrowthId` keeps only meaningful runtime stack ids. Removed no-op PVE/quest reward stack ids instead of preserving inert growth reward sources.
- Resolved implementation rule: skill fragment equip limit source is explicit `SkillFragmentMetadata.equip_limit`. `OwnedCopies` consumes one active equip allowance per owned copy; `GlobalExclusive` allows at most one active equip across the roster even when multiple copies are owned.
- Resolved implementation rule: skill-fragment dismantle repairs active loadouts only as much as needed after the copy count changes. Employees retained/unequipped are chosen by stable UUID ordering.
- Resolved implementation rule: starter baseline skill fragments are authored by live starter employee RON. Built-in starter fragment injection remains available only as a `#[cfg(test)]` fixture helper, not a production runtime source.
- Resolved implementation rule: bound equipment remains visible in inventory/maintenance DTOs, but normal runtime interaction paths reject equip, unequip, dismantle, enhance, sell, and combination consumption until an explicit unbind event/policy exists.

## Follow-Up Candidates

- Future official trust implementation design.
