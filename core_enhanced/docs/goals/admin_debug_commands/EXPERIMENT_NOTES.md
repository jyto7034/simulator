# Admin Debug Commands Experiment Notes

## 2026-06-07 - Design Boundary

Admin/debug commands can contradict normal game rules. That is acceptable only if they are treated as dev/test fixture builders:

- They must not be reachable in production.
- They must not be documented as player gameplay features.
- They should not be mixed into `PlayerBehaviorRequest`.
- They should create valid runtime states that normal commands can continue from.

## 2026-06-07 - Policy Risk

Commands like `admin_finish_battle`, `admin_skip_node`, or `admin_force_node_kind` can bypass failure/reward/attempt policies. Keep them out of the first implementation unless the user explicitly approves them.

## 2026-06-07 - Useful First Target

The first useful target is non-combat contract testing:

- Enter Maintenance without waiting for random map generation.
- Grant an equipment item and equipment dust.
- Grant a skill fragment and fragment dust.
- Make a Medical target by setting HP/trauma.
- Enter Shop/Reward/HQ directly.

This supports Unity UI and Python smoke tests without changing the actual game loop.

## 2026-06-07 - Top-Level Message Chosen

`admin_command` is intentionally a top-level WebSocket request:

- It avoids presenting fixture commands as normal gameplay behavior.
- It makes access-gate logic local to the WebSocket/session boundary.
- It lets Unity and probe scripts search for all admin usage by one message type.

## 2026-06-07 - Access Gate Policy

The gate is stricter than a token-only check:

- Production always rejects admin commands.
- Development/test also require `ENABLE_ADMIN_COMMANDS`.
- `ADMIN_COMMAND_TOKEN` is optional, but if set the request must include the same token.

This keeps accidental local command exposure from becoming the default behavior.

## 2026-06-07 - Fixture Node Policy

Admin node entry does not mutate a random existing map node into another type. It appends a deterministic admin-only fixture node to the current run map and moves the current node pointer there.

Reason:

- The command creates a valid runtime continuation point.
- It avoids corrupting authored map content.
- Normal commands like `CompleteNode`, `ExitShop`, and `ExitReward` can still operate on the session.

## 2026-06-07 - Snapshot Shape Correction

Medical `target_candidates` are UUID strings in both `BehaviorResult::SupportState` and
`state_snapshot.selected_event`. Do not document or implement Unity code as if target candidates
are rich employee DTO objects. Unity should join these UUIDs against `state.roster.employees` when
it needs display names, HP, trauma, or portrait data.
