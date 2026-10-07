# Phase 01 CRA-P09-C08 Notes

Status: complete
Implementation Complete: true

## Scope Boundary

This is a safety-net correction, not a runtime validation change. Production typed deserialization/publication
continues to own rejection, while this Phase makes tests prove the exact rejection cause. CRA-P09-C01 and all other
publication/targeting behavior remain future Phases.

## Dependency Decision

There is no predecessor Phase. The current Child is validly active because the Master exists and the user's
2026-08-09 execution request explicitly activated it. Phase 02 remains pending until this Child is `Complete / High`.

## Evidence Quality

The audit directly observed `missing field delay_ms` being accepted by catch-any-panic. Current inspection locates
the exact authority in `core_enhanced/src/game/data/skill_data.rs` and shows the fixture also omits `name`, `kind`,
`focus_time_ms` and `effects`. The live baseline run must identify which unrelated field wins on this tree; the
correction must make every unrelated requirement valid rather than merely adding the historically named one.

## Policy And Compatibility

No user-visible/gameplay choice is open. CRA-POL-006 is a guardrail only: no RON, DTO/wire, V3, public API or
external consumer migration is authorized. If exact error assertion would require changing one of those contracts,
this Phase stops for policy rather than weakening its proof.

## Dirty Work Rule

All current changes are user-owned. Before editing any discovered target, capture its status, existing diff and
content fingerprint. Preserve unrelated lines and do not reset, restore, stash, clean or broadly format the tree.

The target file was already modified by a user-owned `SkillCastSelection` change (167 insertions/5 deletions before
Phase mutation). Its newly added `selection` fields overlap fixture neighborhoods but have distinct meaning from
C08. They are to be preserved; this Phase will not alter selection semantics and can safely add the missing valid
boilerplate and exact assertions around them.

## Completion Hand-Off

The deliverable to Phase 15/CRA-P09-C01 is a trustworthy negative-schema safety net: repaired controls succeed and
each restored defect fails for exactly its named reason. The Phase 01 completion audit, not a green test alone,
decides whether this hand-off exists.

## Final Implementation Boundary

- Test authority changed only inside `skill_data.rs`'s `#[cfg(test)]` module.
- Fourteen RON negatives are paired with complete controls and exact complete error strings.
- All programmatic negative validations in the target module now inspect the exact object/path/cause rather than a
  generic constructor panic; the file contains no `catch_unwind` or bare `.is_err()` assertion.
- The pre-existing `SkillCastSelection` runtime/deserialization work remains user-owned and unchanged by Phase 01.
- No canonical update is warranted because the production contract did not change; only its safety net became
  truthful.

The initial helper failure is retained as useful evidence: exact-one fixture mutation is intentional and prevents a
future replacement from silently changing both the cast-level and step-level target at once.
