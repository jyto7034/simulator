# Combat Setup Usage Audit Notes

## Initial Assumptions

- This goal is an audit/cleanup goal, not a combat setup policy change.
- Files with no internal callers and no public external behavior should be removed instead of preserved as legacy scaffolding.
- If removing a file implies schema, DTO, or gameplay policy changes, stop and ask.

## File Audit

| File | Runtime callers | Decision | Reason |
| --- | --- | --- | --- |
| `balance.rs` | `battle/damage.rs`, `combat_preview/threat.rs`, combat preview tests | Keep | Shared coarse AD/AP warning and damage-feedback thresholds. Not a stub. |
| `battlefield_plan.rs` | `events/combat.rs` battle startup and tests | Keep | Converts combat preview dimensions/tiles/obstacles into `BattleFieldSpec`; validates scenario obstacle overlap. |
| `defense_object.rs` | `events/combat.rs` protect-unit scenario assembly | Keep | Creates scenario-only defense objective units for protect missions. |
| `enemy_spawns.rs` | `events/combat.rs` battle scenario assembly | Keep | Converts preview waves into deterministic enemy spawn groups and movement plans. |
| `mission_policy.rs` | `events/combat.rs`, `combat_preview`, `world/combat.rs`, data validation, reward policy, integration tests | Keep | Central table/function policy for supported combat node types, mission variants, default objectives, and reward tags. |
| `player_spawns.rs` | `events/combat.rs`, `world/combat.rs`, `world/helpers.rs`, `world/snapshot.rs` | Keep | Converts employee loadouts and permanent artifacts into battle unit drafts and effective combat profiles. |
| `rewards.rs` | `events/combat.rs` reward resolution | Keep | Thin but live combat reward boundary; validates forbidden rewards and mission reward policy. |
| `scenario_groups.rs` | `events/combat.rs` scenario assembly | Keep | Thin but live helper that pairs spawn groups with spawn events consistently. |

## Follow-Up Candidates

- `events/combat.rs` is the actual orchestration hub and remains large. A future goal can split battle-start assembly into a dedicated builder once behavior tests are strong enough.
- `rewards.rs` and `scenario_groups.rs` are intentionally thin. They should stay only while they remove duplication at the combat setup boundary; if a later refactor inlines the single call site, remove them rather than preserving compatibility wrappers.
- Current `combat_setup` modules are exported from `game/mod.rs` as public modules. Tightening visibility would be cleaner, but integration tests and cross-module use should be audited separately because it can become an API-surface change.
