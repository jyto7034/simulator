# Documentation–Code Alignment Corrections Master Experiments

Status: complete

Document class: non-canonical orchestration execution log

Started: 2026-07-18

Production implementation and child-level validation belong in each activated child’s execution log.
This file records Master initialization, routing changes, phase transitions and final combined review.

## 2026-07-18 — Master Initialization

### Objective

Move the completed reaudit’s non-P1 corrections and newly accepted policy decisions into bounded,
independently verifiable child tracks without modifying production artifacts during planning.

### Inputs reviewed

- Completed `documentation_code_alignment_reaudit` plan, report, notes and experiments.
- Current documentation governance and correction/master conventions.
- Existing goal directory names and target path collision.
- User decisions made after the audit for block, fragments, abnormality range, WhiteNight,
  Maintenance and GrenadeLauncher.

### Decisions recorded

- P01 weapon-authored block capacity with explicit live RON values and archetype guidelines.
- P02 formal active-skill fragment policy; BasicAttackModifier placeholders remain temporary.
- P03 formal authored abnormality range; current generic values remain temporary.
- P04 WhiteNight remains future/undecided and unimplemented.
- P05 Maintenance is explicit recipe-gated with no fallback.
- P06 GrenadeLauncher is removed from current schema and retained only as a future concept.

### Routing result

- Seven planned child goals own 12 included findings exactly once.
- `DCA2-WIRE-001` P1 Unity presentation remains excluded by user instruction.
- `DCA2-DOC-002` external duplicate-tree deletion remains excluded pending exact authorization.
- Child directories were not created before activation.

### Files changed

- `docs/goals/documentation_code_alignment_corrections/PLAN.md`
- `docs/goals/documentation_code_alignment_corrections/EXPERIMENT_NOTES.md`
- `docs/goals/documentation_code_alignment_corrections/EXPERIMENTS.md`

No production code, live RON, tests, durable canonical or external Unity file was modified.

### Tests

Not run. This turn initializes orchestration Markdown only. Runtime tests are required by the
activated child that changes their owning behavior.

### Next action

Activate one independent technical child (G01, G02 or G03), or G06 if prioritizing current schema
cleanup before weapon block authoring.

## 2026-07-18 — G01 Completed

- Activated `gameplay_ron_nested_strictness_correction` after creating its required documents.
- Corrected transitive strictness for all seven demonstrated nested gameplay types and protected the
  battlefield wrapper against future direct widening.
- Production-shaped unknown-field regressions and current embedded live loading passed.
- Final child broad result: Core all-targets 781 passed, 0 failed, 0 ignored.
- `DCA2-DATA-002` is complete under G01.

## 2026-07-18 — G02 Completed

- Activated `battle_setup_catalog_ref_correction` after creating its required documents.
- Corrected setup ref collection at the Core producer: candidates must be actual abnormality kind,
  non-empty and resolvable in the loaded catalog; sorting and deduplication remain authoritative.
- Added exact live-data regressions for actual, mixed and corroded-only encounter shapes.
- Core combat tests passed 60/60 and game_server tests passed 30/30.
- The registered real WebSocket smoke passed and observed
  `battle_setup_summary.catalog_refs.abnormality_ids: []` for a corroded-only encounter.
- `DCA2-WIRE-003` is complete under G02; excluded P1 Unity presentation remained untouched.

## 2026-07-18 — G03 Completed

- Activated `endless_completion_regression` after creating its required documents.
- Replaced an indirect single-response test with a direct final-response construction and exact
  `CombatRewardsGranted -> RunComplete` assertion.
- The test also proves Core enters `GameState::RunComplete`, all initialized responses are complete,
  and a pre-queued Safe Zone delivery remains pending.
- No production code changed. Core library validation passed 738/738 and formatting passed.
- `DCA2-TEST-001` is complete under G03.

## 2026-07-18 — G06 Completed

- Activated `grenade_launcher_schema_removal` before G04 as planned.
- Removed the enum variant and exhaustive fixture arm; live RON required no migration.
- Added a fail-closed RON regression and corrected current external enum/visual inventories while
  preserving explicit future-only design notes.
- Core library passed 739/739; `ron_loading` passed 20/20; format/diff checks passed.
- `DCA2-BATTLE-002` and P06 are complete under G06.

## 2026-07-18 — G04 Completed

- Activated `weapon_block_capacity_authoring_contract` after G06 fixed the current archetype set.
- Added a required weapon profile field, authored all 29 live weapon values, set unarmed capacity to
  zero and copied the exact equipped weapon value into the authoritative combat profile.
- Added missing-field, live-catalog, six-archetype, unarmed, armor-independence, replacement and
  snapshot evidence; corrected nearest Core/external canonical wording.
- Final child gates: equipment 27/27, ron_loading 21/21, Core lib 741/741, game_server 30/30,
  formatting/diff checks green.
- `DCA2-GAME-001` and P01 are complete under G04.

## 2026-07-18 — G05 Completed

- Activated `placeholder_content_policy_canonicalization` after G04 finalized adjacent equipment
  wording.
- Separated formal policy from current placeholders for fragments and abnormality ranges, marked
  WhiteNight's detailed design future/undecided, and made Maintenance recipe-only with current
  `standard_armor` coverage explicit in Core and external canonical docs.
- No production code, test or live RON changed under this child; vocabulary and diff checks passed.
- Four G05 findings and P02–P05 are complete.

## 2026-07-18 — G07 Completed

- Corrected the durable gameplay source count to the loader/test-owned value 21.
- Replaced current-domain `G15`/`G24` vocabulary without rewriting historical goal records.
- Updated the external current map-category and targeting-profile inventories to the final Core enum
  shape, removing `SplashClusterFirst` and adding `Maintenance`, `Event` and `Gate`.
- The focused embedded-source test passed 1/1 and exact stale-term/inventory searches were clean.
- Three G07 findings are complete; no production code, live RON or test changed under this child.

## 2026-07-18 — Combined Completion Review

- Re-read the completion-review guide and sampled every child directly from runtime, live RON,
  server pass-through, current Core/external canonical and behavior-pinning tests.
- Reconciled the finding registry: all 12 included findings remain assigned once and complete.
- Found no cross-component source-of-truth conflict, compatibility fallback, ignored regression or
  stale current enum authority requiring another correction child.
- Confirmed the P1 Unity presentation finding and external duplicate documentation tree remain
  excluded and were not mutated.

### Broad validation

- `cargo fmt --all -- --check`: passed.
- Core all-targets: 786 passed, 0 failed/ignored.
- `cargo check -p game_server`: passed.
- game_server: 30 passed, 0 failed/ignored.
- Unity full EditMode job `36c79d3dae8f4eeb91ec19ad6d056c2d`: 241 passed, 0 failed/skipped.
- Core and scoped external diff checks passed. Whole external diff checking retains unrelated
  pre-existing CRLF warnings in `.gitignore`/IDE files.
- Exact current/future/removed enum, source-count and historical-label searches passed.

### Verdict

- [COMPLETION_REVIEW.md](COMPLETION_REVIEW.md): Complete / High.
- Immediate corrections: none.
- Required follow-up refactors: none.
- Open policy questions: none.
