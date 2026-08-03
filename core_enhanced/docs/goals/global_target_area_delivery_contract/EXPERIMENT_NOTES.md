# Global Target Area Delivery Contract Notes

Goal: G09

Status: complete

## Guardrails

- Preserve G10's exact per-skill selection choices; G09 changes reach and footprint ownership only.
- Preserve G11's Player-only preview surfaces and opponent preview absence.
- Do not add movement-era tracking/homing/orientation policy where current deployed units do not
  move and current required footprints are symmetric.
- Do not add a loadable field-wave variant. Record the accepted future behavior only in canonical
  architecture documentation.
- Preserve dirty worktrees and all closed-goal changes.

## Immediate Trace

- Classify each `range_policy`, `defense_tile_range`, `WholeFieldValidTiles`, cast-target, step-target,
  `CastTarget`/`CastTargetStart`, and `TileArea` occurrence by acquisition, footprint, runtime anchor,
  projection, or stale documentation role.
- Inventory live Global and TileArea combinations and identify the stationary final-boss skill.
- Trace whether queued cast state stores only unit identity or already captures a tile, and whether
  death/withdrawal cleanup invalidates that state before delayed delivery.
- Trace catalog/snapshot DTO construction and every Unity parser/fixture before selecting the final
  typed reach/footprint migration.

## Final Findings

- Global is now cast acquisition reach only. It maps to the existing whole-valid-tile selector
  internally but does not supply delivery geometry.
- Every live cast explicitly authors `reach` and `air_capable`; Pattern enemy casts resolve one
  explicit target range source, while Global forbids one.
- `TileArea.footprint` is the only area hit shape. Step range/air authority and the mixed
  WholeField/TileArea schema are absent from production types and live data.
- Cast state already had a stored anchor seam; runtime resolution was adjusted so dead/withdrawn
  reused targets can reach it. The final exact test proves both disappearance paths and event/hit
  parity for a stationary caster with no facing.
- All current non-caster-centered footprints are rotation-invariant because no explicit
  orientation contract exists. No movement-era or field-wave variant was introduced.
- Catalog v2 is consumed by a strict Unity reader; Unity display remains downstream of Core final
  cells/affected tiles.
- Final verdict: Complete / High / no immediate correction.
