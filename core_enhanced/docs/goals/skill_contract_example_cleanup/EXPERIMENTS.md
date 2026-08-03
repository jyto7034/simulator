# Skill Contract Example Cleanup Experiments

Goal: G18

Updated: 2026-07-14

## Entry gate

- G10 and G09: Complete / High; final target selection and reach/footprint ownership are fixed.
- G13: Complete / High; battlefield canonical cleanup no longer overlaps this child.

## Dirty baseline

- Simulator status fingerprint:
  `6b2fadf1959e6c29c636cba5594b2aa8dcc53215a25cc3dc5b25aae7d68f123f`.
- Unity status fingerprint:
  `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Preserve all closed-goal/unrelated hunks; G18 is canonical documentation and verification only.

## Revalidation

- DCA-M-03 changed under G09: reused direct targets no longer recheck cast range, but final direct
  resolution still applies alive/hostile and step-usefulness filters; the canonical bullet omitted
  usefulness.
- DCA-M-04's obsolete TileArea `range_units` example was already replaced during G09 by a concrete
  `TileArea.footprint` example. It needs final review, not another schema change.
- DCA-M-05 remains current: the RetargetOnStep sentence names removed Big Bird/Judgement Bird
  examples rather than the four actual live uses.
- A second stale current example was found: the detailed Unity skill catalog block still declared
  version 1 even though root canonical, Core DTO, server, and strict Unity reader require version 2.
- The `FirstStepTarget` paragraph also described an already-removed variant as future cleanup and
  named obsolete helper parameters.

## Canonical correction

- Direct `ReuseCastTarget` now states the final split precisely: no acquisition-range recheck, but
  alive/hostile, cast air eligibility, and step usefulness still gate direct unit application.
- RetargetOnStep examples now list all four live pairs exactly: Little Red `hunt_finish`, Nothing
  There `goodbye_finish`, Funeral `black_funeral`, and fragment Funeral `fragment_quiet_lament`.
- `FirstStepTarget` is labeled removed legacy and helper arguments use final target/reach/range/air
  vocabulary.
- The detailed Unity skill catalog example now declares version 2 and orders its cast-target fields
  consistently with the typed DTO.
- The G09 concrete TileArea footprint example was rechecked and already closes the old
  `range_units` finding.

## Validation

- Core exact reused-target range regression: 1/1; strict skill loader group: 1/1; live skill
  catalog audit: 3/3.
- Unity skill catalog contract tests: 3/3; final cleared console: zero errors.
- Current-canonical searches found no stale Big Bird/Judgement Bird step ids, catalog version 1, or
  SkillStepDef example containing removed range/air fields. All four replacement id/step pairs exist
  in live RON.
- Core/Unity scoped whitespace checks passed. No production source or live data changed, so the
  immediately preceding broad gate remains Core 750, server 30, Unity EditMode 215, PlayMode 52
  plus four explicit environment skips.
