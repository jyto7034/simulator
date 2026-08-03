# Phase 05 Targeting Completion Review

Reviewed: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Initial verdict: Partially complete

Initial long-term fit: High for runtime/data/contracts; validation gate incomplete

Initial improvement class: immediate correction recommended

Initial Phase 5 exit gate: failed

Current verdict after G31 correction rerun: Complete

Current long-term fit: High

Current improvement class: none

Current Phase 5 exit gate: passed

## Item Audit

| Item | Child | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- | --- |
| Enemy preview removal / Player preview preservation | G11 | Complete | High | None |
| Skill selection / basic-attack weapon separation | G10 | Complete | High | None |
| Global reach / captured TileArea delivery | G09 | Complete | High | None |
| Battlefield HUD / reserved Facility canonical | G13 | Complete | High | None |
| Final skill examples / catalog v2 wording | G18 | Complete | High | None |
| Manifest/index evidence governance | G20 | Complete | High | None |
| Cross-component runtime/data/DTO composition | G11/G10/G09/G13/G18/G20 | Complete | High | None |
| Required broad Unity validation composition | R05/G31 | Complete after correction rerun | High | None |

## Integrated Runtime And Data Evidence

- The one Core checkpoint builder gives final range previews only to Player actors; Unity requires
  them for Player and rejects them for non-player actors. Pending four-facing preview remains a
  separate read-only Player query.
- Basic attacks consume weapon `targeting_profile`; skills consume required skill-owned
  `EnemySingle.rule`. Exact conflicting-profile tests prove the sources remain separate.
- All 25 live skills explicitly author Pattern reach and air authority; all 21 TileArea deliveries
  own concrete footprints; all four RetargetOnStep uses match canonical examples.
- The exact Global scenario proves far selection with no facing, captured target-centered geometry,
  bystander hit, withdrawal/death persistence, and event/execution parity. Independent edge/void,
  air, reuse-range, and usefulness gates pass.
- Catalog v2 separates cast reach/range/air from delivery footprint. Server forwards the typed shape
  unchanged; Unity rejects v1, legacy step authority, mixed Global range, and missing footprint.
- Current canonical HUD, Facility, examples, and manifest evidence-strength language match actual
  production. Manifest/index tests derive owners/unreferenced status and all 30 fragment ids.

## Legacy And Cross-Component Audit

- No production/current canonical compatibility path was found for enemy previews, weapon-inherited
  skill selection, step range/air, mixed WholeField/TileArea, implicit reach/air, field wave schema,
  catalog v1, stale retarget ids, camera billboard HUD, produced Facility, or manifest behavior
  overclaim.
- Relevant Core/server tests are not ignored. Unity's four full-suite skips are explicit external
  live-server environment cases, not Phase 5 contract suppressions.
- No child introduced a competing range source: selection, acquisition, footprint, final preview,
  and presentation each have one owner.

## Validation Results

- Focused Core: 13 exact one-test gates plus manifest/index 4/4, all passed.
- Focused Unity: EditMode 7/7 and PlayMode 2/2, all passed.
- Broad Core: 751 passed, 0 failed, 0 ignored.
- Broad server: 30 passed, 0 failed, 0 ignored.
- Broad Unity EditMode: 215/215.
- Broad Unity PlayMode: failed 8 of 56. First failure expected starter Ada but rendered pointer-hovered
  Cyra; exact rerun also failed. Seven later failures are consistent with first-test teardown abort.

## R05-H-01 — Sequence-Dependent PlayMode Pointer Isolation

Expected behavior: the first starter-selection mock test must begin with deterministic non-hovered
selection regardless of which pointer-sensitive tests ran before it, and failed assertions must not
leak root/UI state to later tests.

Evidence: the current warp-to-zero plus one-frame yield is insufficient after the R05 focused
PlayMode order. The failure reproduces both in full suite and exact rerun.

Long-term direction review:

- Fit: High for the production contract; Low for the current test isolation technique.
- Improvement class: immediate correction recommended.
- Required correction: G31 must establish deterministic Input System pointer state and guaranteed
  teardown, then prove exact, focused→full, repeated full, and console-clean behavior.

Verdict: Partially complete.

Correction: G31 replaced the method-local queued warp with immediate class-level Input System state
neutralization and guaranteed baseline-root teardown. G31 passed exact, contaminating-order, repeated
full, and EditMode review without production changes or workarounds.

Current verdict: Complete.

## Correction Rerun Evidence

- Focused Core: 13 exact one-test gates plus manifest/index 4/4, all passed.
- Focused Unity: EditMode 7/7 and contaminating PlayMode 2/2, all passed.
- Broad Core: 751 passed, 0 failed, 0 ignored.
- Broad server: 30 passed, 0 failed, 0 ignored.
- Broad Unity EditMode: 215/215.
- Full Unity PlayMode immediately after focused sequence: 52 passed, 0 failed, four explicit
  environment skips.
- Immediate second full PlayMode without manual cleanup: 52 passed, 0 failed, four explicit skips.
- Formatting, diff, legacy/compatibility, ignore/retry/order/warp, scoped whitespace, and final
  console audits passed.

The correction is confined to test fixture input/object lifecycle. Production targeting, live RON,
Core/server DTOs, Unity consumers, hover UX, and canonical policy did not change.

## Immediate Correction Candidates

None after G31.

## Follow-Up Refactor Candidates

- `unknown_distortion_strike` remains explicitly unreferenced under G20; future content may assign or
  remove it. This is not a Phase 5 correctness blocker.

## Policy Questions

None. G31 is test determinism/isolation, not gameplay or UX policy.

## Review Conclusion

Initial R05 was Partially complete and correctly blocked Phase 6. G31 is Complete / High, and the
independent appended correction rerun passes every focused/broad/order/repetition gate. Phase 5 is
now Complete / High with no immediate correction. Phase 6 may begin.
