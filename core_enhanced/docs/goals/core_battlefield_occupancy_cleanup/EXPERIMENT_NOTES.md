# Experiment Notes

## Confirmed Policy

- Remove `Battlefield` single-tile `occupant` projection.
- Remove related single-owner tile semantics.
- Unit position/continuous body state is the source of truth.
- Tile membership acceleration, if needed, must be multi-occupant and derived.
- Do not preserve the old model as debug/display-only `occupant`.

## Policy Questions To Watch

Record `사용자와 정책 논의 필요` and complete the goal if work discovers any item below. Do not pause, block, or leave the goal active; a policy-decision report is the completed deliverable.

- allied overlap policy is not fully settled,
- stacked-unit target priority requires a new rule,
- AoE stacked-unit behavior is ambiguous,
- Unity needs a new stacked-unit display contract,
- a derived index needs lifecycle guarantees that could create a second source of truth.

## Follow-Up Candidates

Record out-of-scope improvements here.

## Implementation Notes

- `Battlefield` now stores unit tile placement only in `unit_pos`.
- `Battlefield::place` allows multiple units on the same valid non-obstacle tile.
- `place_allowing_unit_overlap` was removed because overlap is no longer a special alternate path.
- `Battlefield::units_at(pos)` is a derived multi-occupant query over `unit_pos`, not authoritative storage.
- Static obstacles remain a separate terrain source. Placement still rejects static obstacles, but no unit `occupant.is_some()` style tile ownership check remains.
- The live deployment test now pins the new behavior: a second allied unit can deploy onto the same tile, and withdrawing one unit does not remove the other.
