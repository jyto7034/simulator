# Abnormality Attempt / Retreat / Re-entry Experiments

## Log

- Read node flow, combat flow, snapshot, action scheduler, server result mapping, and canonical Unity retreat contract.
- Added `RunState` abnormality attempt counters keyed by `MapNodeId`.
- Replaced legacy retreat completion behavior:
  - Attempts 1 and 2 return to `NodeConfirm` with the same node/session available.
  - Attempt 3 retreat completes the node with `outcome.combat.retreated == true`.
  - Retreat no longer decrements consumable modifier durations.
- Added `abnormality_attempt` to `node_confirm` and `in_battle` snapshots.
- Added `can_retreat` to `in_battle` snapshot.
- Replaced legacy retreat test with `retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted`.
- Updated canonical Unity docs:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`

## Verification

- `cargo check -p game_core`: passed.
- `cargo test -p game_core retreat_from_live_defense_battle_reenters_until_attempts_are_exhausted -- --nocapture`: passed.
- `cargo test -p game_core combat -- --nocapture`: passed.
- `cargo check -p game_server`: passed.

Note: the `combat` test filter includes an expected `should_panic` validation test output for zero consumable duration.
