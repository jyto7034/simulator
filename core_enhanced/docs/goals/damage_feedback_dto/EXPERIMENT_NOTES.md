# Damage Feedback DTO Experiment Notes

## 2026-06-07

- `piercing` can be represented as a tag enum now, but should only be emitted when runtime can prove penetration actually lowered a resistance. The current `DamageResult` does not preserve before/after resistance values.
- `shield`, `blocked`, and `resisted_status` do not currently have concrete damage-runtime meaning in the inspected path. Keep them as typed schema candidates only if needed by the contract; do not emit dummy tags.
- `immune` can be emitted for damage results where `raw_damage > 0` and `final_damage == 0`, which is reachable for non-basic sources with `minimum_damage == 0`.
- The canonical Unity contract already documents `feedback_tags`, so no external Unity doc write was needed for this sub-goal.
- The first attempted focused test command used an unmatched test filter (`damage_feedback`) and ran 0 tests; subsequent commands used matching filters.
