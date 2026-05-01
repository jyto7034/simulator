# Battlefield Tile Model (WIP)

This document captures the current `Battlefield` tile representation and the planned semantics for
two-stage reservations.

Relevant code: `core/src/game/battle/battlefield.rs`

## Data Model

### Grid

- The battlefield is a fixed `width x height` grid.
- Tiles are stored as a flat `Vec<Tile>` using row-major indexing:
  - `idx = y * width + x`

### Tile

Each tile maintains:

- `occupant: Option<UnitInstanceId>`
  - A tile may contain **at most one** unit id.
  - Friendly pass-through / stacking is not represented at the tile layer; movement must resolve
    conflicts via repath / waiting / (optional) same-tick swap.
- `reservation: Option<Reservation>`
  - Optional reservation metadata for destination planning.

### Reservation (two-stage)

`Reservation` is designed to avoid blocking the board too early when a unit reserves a far-away
destination.

```
Reservation { unit, hard_from_ms }
```

- Before `hard_from_ms`: the reservation is **Soft**
  - Other units may *pass through* the tile.
  - Whether other units may *settle/stop* on the tile should be decided by a separate rule
    (recommended: disallow settling on a reserved tile, even if soft).
- At/after `hard_from_ms`: the reservation is **Hard**
  - The tile becomes blocking for other units (recommended default).
  - The owning unit may still enter/settle the tile.

The current helper API:

- `Reservation::kind_at(now_ms)` -> `Soft | Hard`
- `Reservation::is_soft_at(now_ms)`
- `Reservation::is_hard_at(now_ms)`

## Recommended Rules

### Passability (movement/BFS)

For a mover `U` at time `now_ms`, a tile `T` is passable if:

1. `T.occupant` is empty (unoccupied).
2. `T.reservation` is:
   - `None`, or
   - `Some(r)` and `r.is_soft_at(now_ms)`, or
   - `Some(r)` and `r.is_hard_at(now_ms)` **and** `r.unit == U` (owner can enter).

### Settling (stop/attack position)

A tile can be used as a destination/attack position if:

- It has no occupant, and
- It has no reservation (or only an owner-only hard reservation; depending on design).

## Notes / Next Steps

- If we later need forced stacking / pass-through, we can extend tiles back to multi-occupant or
  add transient movement-layer state instead of encoding overlap in the tile model.
- Reservation timing (`hard_from_ms`) should be derived from ETA to destination, e.g.:
  - `hard_from_ms = max(now_ms, eta_ms - prelock_ms)`
