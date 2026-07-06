# Behavior Domain Dispatch Validation Colocation

## Objective

Follow up `docs/goals/behavior_execution_boundary_refactor` by moving behavior validation closer to behavior dispatch at the domain boundary.

The previous goal made the top-level execution pipeline explicit:

```text
gate -> validate -> dispatch -> postprocess
```

This goal should keep that clarity while reducing the duplicate full-`PlayerBehavior` match problem. Today `validate_behavior_payload(...)` matches most behavior variants in one place, and `dispatch_behavior(...)` matches the same enum again. Long-term, a maintainer adding a shop behavior should mostly read the shop behavior entry, not a global validator and a global dispatcher.

## Source Of Truth Order

1. Runtime behavior in `src/game/world.rs` and domain handler modules.
2. Existing tests for allowed actions, payload validation, source command ids, roster sync, and domain behavior.
3. Unity/server command contracts.
4. Canonical docs.
5. Audit and completed goal notes.

## Scope

- Introduce domain-level behavior entries such as shop, reward, battle, event, equipment/maintenance, support/headquarters, and map/navigation where the existing code naturally supports them.
- Move domain-specific payload validation next to the matching domain dispatch.
- Keep the top-level gate and postprocess stages explicit.
- Keep user-visible behavior and command DTOs stable.
- Preserve battle `source_command_id` behavior for deploy/withdraw/skill activation.

## Non-Goals

- Do not redesign `PlayerBehavior`.
- Do not introduce a large command framework, trait registry, macro dispatcher, or runtime plugin system.
- Do not change Unity-facing command DTOs, server messages, allowed-action policy, or roster sync timing.
- Do not move every domain at once if a domain boundary is unclear.
- Do not start the P-008 battle core decomposition from this goal.

## Desired Direction

Prefer a structure close to:

```text
execute_with_source_command_id(...)
-> derive action kind
-> gate_behavior_action(...)
-> execute_validated_behavior_domain(...)
   -> execute_shop_behavior(...)
      -> validate shop payload
      -> dispatch shop handler
   -> execute_battle_behavior(...)
      -> validate battle payload
      -> dispatch battle handler
   -> execute_reward_behavior(...)
      -> validate reward payload
      -> dispatch reward handler
   -> execute_general_behavior(...)
      -> validate/dispatch behavior that has no clean domain yet
-> postprocess_behavior_execution(...)
```

It is acceptable to keep a small general bucket for behaviors whose domain ownership is not yet clean. Do not force an awkward boundary just to remove every match arm.

## Plan

1. Before any code edit, re-read:
   - `docs/goals/behavior_execution_boundary_refactor/PLAN.md`
   - `src/game/world.rs`
   - `src/game/world/helpers.rs`
   - current domain handler files under `src/game/world/`
   - `src/game/behavior.rs`
   - tests covering allowed actions, invalid actions, payload validation, battle source command ids, roster sync, shop/reward/event/equipment/combat behavior.
2. Map current `PlayerBehavior` variants into domain buckets in `EXPERIMENT_NOTES.md`.
3. Identify which validators can safely move next to dispatch without changing behavior.
4. Implement in small steps:
   - keep top-level action gate unchanged;
   - replace global validation + global dispatch with domain execution entries where safe;
   - leave unclear behavior in a general bucket with a note.
5. After each domain move, run focused tests for that domain.
6. Add or adjust tests only for user-visible behavior, command result/error behavior, source command id preservation, and roster sync timing.
7. Run broader validation after all chosen domain moves.

## Completion Conditions

- Top-level behavior execution still reads as explicit gate/domain execution/postprocess stages.
- Validation and dispatch for moved domains live near each other.
- Duplicate whole-enum matching is reduced meaningfully.
- Battle `source_command_id` behavior is preserved and tested.
- Roster sync still occurs once after successful command execution.
- Existing command results and errors are preserved.
- No command DTO, server message, allowed-action, or gameplay policy change is introduced.
- Any behavior left in a general bucket has a reason recorded in `EXPERIMENT_NOTES.md`.

## Implementation Result (2026-07-06)

Status: complete.

Implemented the domain colocation refactor without changing command DTOs or gameplay policy:

- Replaced the top-level global validation call plus `dispatch_behavior(...)` with `execute_validated_behavior_domain(...)`.
- Added domain execution entries:
  - `execute_map_run_behavior(...)`
  - `execute_support_headquarters_behavior(...)`
  - `execute_equipment_maintenance_behavior(...)`
  - `execute_shop_behavior(...)`
  - `execute_reward_behavior(...)`
  - `execute_event_behavior(...)`
  - `execute_battle_behavior(...)`
- Removed the old full-enum `validate_behavior_payload(...)` wrapper from `helpers.rs`.
- Kept validation calls next to the domain dispatch for variants that need explicit payload prevalidation:
  - starter employee selection;
  - map node selection;
  - equip item;
  - select reward.
- Left handlers that already validate internally unchanged.
- Kept battle `source_command_id` handoff isolated in the battle domain entry.
- No general bucket is used because every current `PlayerBehavior` variant has a clear domain owner.

## Validation Commands

Adjust filters after reading actual test names:

- `cargo test -p game_core allowed_actions --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::snapshots_and_start --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::node_sessions --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::equipment --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::map_flow --lib -- --test-threads=1`
- `cargo test -p game_core live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

## Stop Conditions

Complete the goal and report questions if:

- A domain boundary is ambiguous and choosing it would affect future policy.
- A behavior depends on validation running globally before dispatch in a non-obvious way.
- Source command id semantics need a new policy.
- Roster sync timing would need to change.
- A clean implementation requires changing command DTOs or server contracts.
- The refactor begins to require P-008 battle core decomposition.
