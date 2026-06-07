# Consumable Modifier Re-entry Duration Experiments

## Log

- Read consumable metadata, employee modifier storage, use command, combat result, retreat, and snapshot paths.
- Existing tests already covered immediate item removal, dead target rejection, alive-but-combat-unavailable target acceptance, and overwrite without refund.
- Added focused retreat/re-entry duration test.

## Failed Experiment

- Initial focused test failed because all retreat paths skipped `decrement_consumables_after_combat_node`.
- Fix: keep duration unchanged for retreat with remaining attempts, but call duration decrement when a retreat exhausts the final attempt and consumes the node.

## Verification

- `cargo test -p game_core consumable_modifier_survives_retreat_reentry_and_expires_when_abnormality_is_resolved -- --nocapture`: passed after fix.
- `cargo test -p game_core consumable -- --nocapture`: passed.
- `cargo test -p game_core retreat -- --nocapture`: passed.
- `cargo check -p game_server`: passed.

Note: the `consumable` test filter includes expected `should_panic` validation output for invalid consumable metadata.
