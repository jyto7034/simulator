# Global Target Area Delivery Contract Experiments

Goal: G09

Updated: 2026-07-14

## Entry gate

- R04: Complete / High.
- G11: Complete / High; opponent preview state is absent.
- G10: Complete / High; skill-owned selection and basic-attack weapon authority are fixed.
- Accepted PD-06 authorizes the one-way reach/footprint cutover while deferring field-wave runtime.

## Dirty baseline

- Core workspace status fingerprint:
  `ea5575b7d6e080772e290472ee5a2ab4ad7c4aeb5e765b0fb7fec0477a5d8e03`.
- game_server workspace-view status fingerprint:
  `ea5575b7d6e080772e290472ee5a2ab4ad7c4aeb5e765b0fb7fec0477a5d8e03`.
- Unity status fingerprint:
  `cd6eeed715cde88a4f1e429accbff76ccf3a2d8a75d31f3c0c712ecb2b7db28a`.
- Core scoped tracked diff fingerprint:
  `25bfac9b19264c24a813e84c947482955e0f7975f41efb44e411d7c749d9df75`;
  server tracked diff fingerprint:
  `d2eca044c3a8a6b3c7c26fd38bad04c0abd4e52c7e859f08caaab82fb91c63db`;
  Unity tracked diff fingerprint:
  `112b49da6f51b28e4e11606f541d5c7930449023880f65749b54e0ea73fbc407`.
- All worktrees remain heavily dirty. Preserve every unrelated and closed-goal hunk; no stash,
  reset, restore, clean, or baseline commit is authorized.

## Schema and live-content cutover

- Added typed `SkillTargetReach::{Pattern, Global}` cast authority and moved all skill acquisition
  range/air eligibility off `SkillStepDef`.
- Made live `reach` and `air_capable` presence-required. The final strictness audit found and
  removed an interim default seam; all 25 live skills now author both fields explicitly.
- Moved the required concrete `TileRangePattern` into `DeliveryDef::TileArea.footprint`; raw step
  loading rejects removed range/air fields, missing footprints, mixed reach sources, invalid
  anchor/origin pairs, and facing-dependent target/impact footprints.
- Migrated all 21 live TileArea steps. Current symmetric areas use explicit 3x3 footprints, while
  WhiteNight's three existing wide areas use symmetric 5x5 footprints to preserve their scenario
  hit coverage.

## Runtime and cross-layer cutover

- Cast state captures the selected target tile. Reused TileArea delivery retains that identity and
  falls back to the stored anchor after target death or withdrawal without reapplying cast reach.
- Runtime execution, usefulness, event projection, edge/void clipping, and facingless symmetric
  projection share the same footprint primitives.
- Core catalog version 2 separates cast `reach`/`target_range` from TileArea `footprint`; server
  forwarding remains typed and unchanged in ordering.
- Added a strict Unity catalog-v2 reader and EditMode contract tests. Unity consumes final Core
  affected tiles and never reconstructs geometry from the footprint.
- Canonical Core/server/Unity documents describe the split and record the future expanding field
  wave as documentation-only, with no loadable schema/runtime placeholder.

## Validation and correction loop

- Exact Core coverage proves a no-facing stationary Global caster, far target, unchanged captured
  center after both withdrawal and death, eligible bystander hits, projection/execution parity,
  edge/void clipping, cast-level air eligibility, and no reused-target range recheck.
- Strict tests reject missing `reach`/`air_capable`, missing/conflicting cast range, missing
  footprint, legacy step range/air authority, invalid area origin, and asymmetric facingless area.
- A first broad pass after making reach/air presence-required exposed six direct `SkillDef` RON test
  fixtures that still omitted the fields. Migrating those fixtures restored the intended exact
  contract; the final broad run passed.
- The first Unity PlayMode run exposed pointer-position leakage in the earliest hover-sensitive
  starter-card test. That test now warps the Input System pointer to a deterministic neutral origin;
  its exact rerun and the full PlayMode suite passed.
- Final Core all-targets: 750 passed (707 library + 3 + 3 + 20 + 13 + 4), 0 failed, 0 ignored.
- Final game_server all-targets: 30 passed, 0 failed, 0 ignored.
- Final Unity EditMode: 215/215. Final Unity PlayMode: 52 passed, 0 failed, four explicit
  environment-dependent live-server skips. Cleared console: zero errors.
- Core/server formatting and diff checks, scoped Unity/new-file whitespace checks, live-field
  counts (25/25/25), ignored-test audit, and legacy/future-variant searches passed.
