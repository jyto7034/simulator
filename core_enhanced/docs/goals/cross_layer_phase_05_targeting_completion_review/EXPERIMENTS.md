# Cross-Layer Phase 05 Targeting Completion Review Experiments

Review: R05

Updated: 2026-07-14

## Entry gate

- G11, G10, G09, G13, G18, and G20 each report Complete / High.
- R04 is Complete / High; Phase 6 remains blocked by this independent review.

## Dirty baseline

- Simulator status fingerprint:
  `5bf6f4b0ef5161b89dd2a0beaf7daf2f1c862469a2568ef7aad47981bb0eb776`.
- Unity status fingerprint:
  `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Preserve all existing changes. R05 is review-only and does not silently implement corrections.

## Initial independent observations

- Live skill RON contains 25 explicit cast definitions with explicit Pattern reach and air authority,
  21 concrete TileArea footprints, and four current RetargetOnStep uses.
- Runtime types have no step range/air authority; cast selection and delivery footprint are distinct.
- The exact Global disappearance regression composes stored anchor, no-facing symmetric geometry,
  bystander targeting, event projection, death, and withdrawal in one scenario.
- Unity strictly rejects opponent preview compatibility and skill catalog v1/legacy step fields.
- Manifest v2 derives owners from live data and explicitly records one unreferenced skill rather than
  calling membership behavior coverage.

## Focused validation

- Core composition: 13 exact one-test gates plus manifest/index audit 4/4 passed. Covered Player-only
  preview, four-facing pending preview, skill/weapon separation, Global disappearance/no-facing,
  edge/void, air, strict loader, catalog v2, reuse range, live references, and evidence semantics.
- Unity focused EditMode: 7/7; focused PlayMode HUD/pending-preview presentation: 2/2.
- Legacy/source audit found no production skill weapon inheritance, step range/air authority, mixed
  WholeField/TileArea, implicit reach/air, enemy preview compatibility, wave placeholder, current
  catalog v1/stale examples/billboard/Facility overclaim, behavior-coverage overclaim, or relevant
  ignored test. Live counts are 25 explicit casts/reach/air, 21 TileArea/footprints, 4 retargets.

## Broad validation and review finding

- Core all-targets: 751 passed, 0 failed, 0 ignored.
- game_server all-targets: 30 passed, 0 failed, 0 ignored.
- Unity EditMode: 215/215.
- Unity PlayMode: failed 8 of 56 after the focused PlayMode run. The first failure was
  `Bootstrap_RendersEmployeeSelectionMockAndSelectionState`: expected Ada, actual Cyra from retained
  pointer hover despite the existing warp/yield isolation line. The exact test rerun failed the same
  way. Seven subsequent failures are cascade evidence from the first test's aborted cleanup.
- Verdict: runtime/data/DTO/canonical/evidence composition is Complete / High, but the required broad
  phase validation is Partially complete. Route the deterministic pointer/cleanup seam to G31 and
  rerun R05; do not unlock Phase 6.

## Correction rerun after G31

- Independently repeated 13 focused Core one-test gates plus manifest/index 4/4: all passed.
- Independently repeated focused Unity EditMode 7/7 and the contaminating PlayMode sequence 2/2.
- Broad Core: 751 passed, 0 failed, 0 ignored. Broad game_server: 30 passed, 0 failed, 0 ignored.
- Broad Unity EditMode: 215/215.
- Full PlayMode immediately after focused sequence: 52 passed, 0 failed, four explicit external-
  server skips. Immediate second full run without manual cleanup: same 52/0/4 result.
- Formatting, Core/server/Unity scoped diff, new-file whitespace, ignore/retry/order/warp, legacy
  source, and final console audits passed.
- Correction verdict: Complete / High / no immediate correction. Phase 5 passes.
