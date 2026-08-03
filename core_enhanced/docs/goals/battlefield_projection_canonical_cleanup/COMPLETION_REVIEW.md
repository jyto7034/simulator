# Battlefield Projection Canonical Cleanup Completion Review

Goal: G13

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| HUD render space | lower world anchor is projected into dedicated screen-space overlay UI | Complete | High | None |
| Billboard removal | current canonical text forbids world canvas/mesh and camera billboard alignment | Complete | High | None |
| Facility reachability | current production actor/setup lists exclude FacilityEntity | Complete | High | None |
| Reserved wire shape | Rust/Unity enum/parser shape remains documented as future and unproduced | Complete | High | None |
| Scope control | no code, live data, schema, or behavior changed | Complete | High | None |

## Expected Source Of Truth

Core checkpoint HUD metadata owns bar mode/threat facts. Unity owns presentation: it obtains the
actor's lower world anchor, projects it through the camera, and renders bars in a dedicated
screen-space overlay. PVE validation/setup own production reachability; a wire enum variant alone
does not make FacilityEntity live content.

## Evidence

- `docs/core_runtime_contract.md` now matches external Unity canonical and the existing HUD renderer.
- The exact Unity PlayMode test proves `RenderMode.ScreenSpaceOverlay` and lower-anchor screen
  projection.
- `PveWaveEnemyData::FacilityEntity` remains rejected by final validation, combat setup has no
  FacilityEntity profile path, and the focused live PVE test passes while explicitly panicking if
  one appears.
- External transport separates defined enum shapes from the four currently produced actor source
  kinds and labels Facility setup/live identity as wire-reserved future surface.

## Validation Review

- Focused Core live PVE reference: 1 passed, 0 failed, 0 ignored.
- Exact Unity HUD PlayMode: 1 passed, 0 failed, 0 skipped; final console zero errors.
- Immediately preceding broad gate on the same production sources: Core 750, server 30, Unity
  EditMode 215, PlayMode 52 passed with four explicit environment skips.
- Current-canonical stale-wording searches and scoped Core/Unity whitespace checks passed.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: canonical ownership now describes actual projection/reachability without weakening strict
  validation or creating compatibility behavior.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G13. FacilityEntity activation, if ever desired, requires a separately authorized
profile/runtime/validation/Unity implementation goal.

## Policy Questions

None.

## Review Conclusion

G13 is Complete / High with no immediate correction. G18 may begin.
