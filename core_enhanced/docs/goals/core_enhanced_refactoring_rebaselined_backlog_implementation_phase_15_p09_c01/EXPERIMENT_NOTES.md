# Phase 15 CRA-P09-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P09-C01, Master Phase 15 of 44.
- Direct linear predecessor Phase 14 is `Complete / High`; exact-cause prerequisite Phase 01 is complete.
- Phases 16~44 remain pending. Phase 15 mutation authority is limited to projectile/basic/weapon publication,
  direct tests and nearest canonical evidence.
- All three Child documents and the Phase 14 audit must be fully read before production/status/data inspection.

## Approved Policy Boundary

CRA-POL-003 approves strict rejection before publication:

- projectile speed and every present optional count/time/limit are positive;
- basic attacks and weapon profiles accept only target-locked fields their runtime executes;
- field-specific typed object/path/cause is retained;
- no partial startup, normalization, instant fallback or permissive legacy path;
- RON and programmatic construction share the contract and current live data remains valid.

CRA-POL-006 still prohibits changing the current public/RON/save/wire shape without separate approval.

## Authority Notes

- Semantic validity belongs before database publication, not in a runtime zero-flight or ignored-field fallback.
- Serde can enforce representation syntax, but programmatic and RON candidates must converge at one semantic gate.
- Basic/weapon compatibility must be derived from fields actually executed by their runtime, not merely from variant
  names. Default-valued unsupported fields may be representationally indistinguishable; reject every observable
  nondefault unsupported policy with exact provenance.
- Runtime guards may remain defensive integrity checks but must not authorize invalid published data.

## Dirty Worktree Risk

The workspace contains extensive user-owned data, validation, ability, battle, server, test and canonical changes.
Fingerprint prospective targets before editing. Do not reset, restore, clean, stash, delete or broadly format; stop
if an exact Phase 15 hunk cannot be separated.

## Testing Notes

- Use otherwise-valid fixtures repaired under P09-C08 so each negative proves the intended exact failure.
- Exercise both RON deserialization/publication and programmatic database construction; a serde-only test is not
  parity proof.
- Cover valid Instant and positive default TargetLocked delivery beside every invalid basic/weapon policy row.
- Preserve positive ceiling arithmetic and actual skill/basic runtime behavior, not only validator return values.
- Assert candidate-level atomic failure/provenance; catch-any-panic, zero-test filters and helper-only tables are not
  proof.

## Out-Of-Scope Follow-Up Handling

Keep P09-C06 query consolidation under Phase 16, usefulness/preview under Phases 17~18, committed damage under Phase
28 and shared motion extraction under Phase 29. Do not reopen Phase 14 terminal delivery or prebuild those owners.

## Implemented Authority

- `DeliveryDef` owns positive projectile scalar validation and the strict basic-attack consumer matrix through one
  typed field-suffix/cause error.
- Skill, abnormality/corroded and equipment databases attach their own object/index prefix before publication.
- Official Skill RON parses/resolves a raw candidate without stringifying semantic errors; outer domain validation
  owns typed publication. Direct `SkillDatabase` deserialization remains fail-closed through the same validator.
- Raw partial structs are candidates, not publications. A raw collision with `max_hits: Some(0)` can deserialize but
  fails its semantic contract and cannot pass any official database publication gate.

## Final Evidence Snapshot Before Review

- Historical exact reds: four nonzero filters failed for the intended acceptance/path mismatches.
- Corrected exact matrices: four filters passed; RON cases also passed through `GameDataBase::load_live_sources`.
- Debug Core 978/978; release Core 978/978; debug/release skill runtime 14/14; debug/release RON 25/25;
  live skill 4/4; live item skill 4/4; server 59/59.
- Workspace all-target check, formatter and scoped diff check pass.
- Live skill/abnormality/equipment RON files are byte-identical to activation. No DTO/checkpoint/wire/event shape or
  runtime projectile behavior changed.
- Status growth 443→445 is limited to previously clean abnormality and corroded validator sources. The pre-existing
  dirty ability/skill/equipment/live loader/canonical hunks remain preserved.

## Review Gate

Production correction and validation are complete, but Phase 16 remains unauthorized until the full completion
guide is reread and an independent audit records `Complete / High` with no unresolved high-impact improvement.
