# Skill Contract Example Cleanup Completion Review

Goal: G18

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Item | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| Reuse semantics | no cast-range recheck; live/hostile/air/usefulness execution gates are explicit | Complete | High | None |
| TileArea example | concrete delivery footprint with no removed step range field | Complete | High | None |
| Retarget examples | all four named skill/step pairs exist in current live RON | Complete | High | None |
| Legacy inference | removed FirstStepTarget is no longer described as future work | Complete | High | None |
| Unity catalog example | detailed JSON uses final version 2 cast/step shape | Complete | High | None |

## Expected Source Of Truth

Rust runtime and strict live RON own targeting behavior/schema. Typed Core snapshot serialization
owns Unity JSON shape. Canonical examples explain those sources and never preserve obsolete fields,
ids, defaults, or versions for copying convenience.

## Evidence

- The reuse runtime first preserves a cast target without acquisition-range revalidation, then
  direct target resolution applies alive/hostile and usefulness rules; cast-level air eligibility
  remains authoritative. TileArea instead resolves at the captured tile after disappearance.
- Live RON contains exactly the documented RetargetOnStep pairs:
  `little_red_hunt_the_prey.hunt_finish`, `nothing_there_goodbye.goodbye_finish`,
  `funeral_butterfly_eulogy_volley.black_funeral`, and
  `fragment_funeral_butterfly_eulogy.fragment_quiet_lament`.
- The current TileArea RON example owns `footprint` inside delivery and contains no `range_units`,
  step range policy, step defense range, or step air authority.
- Root and detailed Unity examples now both use catalog v2, matching Core and the strict Unity
  reader that rejects v1.

## Validation Review

- Focused Core: reuse range regression 1/1, strict skill loader 1/1, live catalog audit 3/3.
- Focused Unity catalog contract: 3/3; final console zero errors.
- Current-canonical stale-id/version/removed-step-field searches and scoped whitespace checks passed.
- No production source/data changed; immediately preceding broad evidence remains Core 750, server
  30, Unity EditMode 215, PlayMode 52 plus four explicit environment skips.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: examples are downstream of final typed/runtime sources and no longer encode historical
  behavior as a competing contract.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G18. G20 owns whether live catalog evidence links to concrete behavior scenarios.

## Policy Questions

None.

## Review Conclusion

G18 is Complete / High with no immediate correction. G20 may begin.
