# Experiment Notes

## Policy Notes

- `block` in the confirmed policy means engagement/intercept logic that helps decide next action, not a tile-occupancy rule.
- Opponent unit overlap is allowed, so spawn validation must not introduce nearest-open behavior.
- `allowed_actions` is retained as a Unity-facing affordance projection. It is not the security source; `execute_with_source_command_id` and handler validation remain authoritative.
- `RequestDeploymentRangePreview` remains included in in-battle allowed actions so Unity can disable/enable the button without attempting the command.
- `MovementStopped` already carries reason data for arrival, obstacle, and no-goal stops.
- Follow-up policy supersedes the old compatibility note: `Battlefield` single-tile `occupant` projection and related single-owner tile semantics should be removed. Unit position/body state is the source of truth; any tile membership acceleration must be multi-occupant and derived.
- `BehaviorResult` broad split requires shared command/result DTO work and belongs to `core_policy_unity_server_contract`.
- Timeline naming normalization also belongs to `core_policy_unity_server_contract` because it crosses Unity/server naming and payload contracts.
- Same-timestamp `AttackResolve` draining now covers battle-end confirmation ordering. Changing whether a dead basic-attack attacker still applies already-started damage would be a separate combat snapshot policy and was not changed here.

## Follow-Up Candidates

- In `core_policy_unity_server_contract`, split overloaded `BehaviorResult` contracts and normalize timeline delta/event naming together with typed DTO migration.
- If future design wants basic attack damage to snapshot at windup/release even when the attacker dies before resolve, define that as a separate policy from same-timestamp queue draining.
