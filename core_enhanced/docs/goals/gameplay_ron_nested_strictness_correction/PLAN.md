# Gameplay RON Nested Strictness Correction

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G01

Finding: `DCA2-DATA-002`

## Objective

Make every demonstrated live-reachable nested gameplay RON schema fail closed on unknown fields,
with production-shaped regressions proving transitive strictness through actual outer databases.

## Source Of Truth

- Master G01 scope and accepted no-compatibility invariant
- `docs/data_loading_contract.md`
- current serde types and `GameDataBase::load_live_embedded`
- production `game_resources/data/**`
- `docs/goals/documentation_code_alignment_reaudit/EXPERIMENT_NOTES.md` schema probe evidence

## In Scope

- Re-audit `MovementDef`, `ResonanceDef`, `TileRangePattern`, `DeliveryDef`, `SkillEffectDef`,
  `StatModifier`, `MapNodePayload` and battlefield database wrapper reachability.
- Add strict unknown-field rejection to every demonstrated shared nested gameplay type.
- Add production-shaped negative tests through Abnormality, Skill, Equipment, Map Node and
  Battlefield databases as applicable.
- Correct current live/fixture RON only if strict parsing exposes an actually obsolete field whose
  current intended meaning is unambiguous.

## Out Of Scope

- New gameplay fields, compatibility aliases or migration fallback
- Unrelated data/schema refactors
- G04 weapon block field
- P1 Unity presentation and other Master children

## Implementation Plan

1. Inspect current derives/serde attributes and all production reachability for the named types.
2. Record a focused pre-change reproduction for unknown nested fields.
3. Apply fail-closed serde attributes at the owning type boundary.
4. Add production-shaped negative tests, including the battlefield wrapper.
5. Run focused data/loader tests, strict live loading and Core broad tests.
6. Independently search for demonstrated types still lacking strictness and write completion record.

## Test Requirements

- Named negative tests for each demonstrated type/path
- `cargo test -p game_core game::data --lib -- --test-threads=1`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core --all-targets -- --test-threads=1`

## Completion Conditions

- Every demonstrated stale nested field fails through its actual outer database.
- Current embedded live bundle passes exact loading/validation.
- No alias/default silently preserves the tested unknown fields.
- Focused and broad test gates pass with exact counts recorded.
- Master G01 can be marked complete from current evidence.

## Stop Conditions

- A current live field fails strict parsing and its replacement/removal meaning is ambiguous.
- Strictness requires a public/save/wire compatibility policy not already approved.
- User-owned dirty changes cannot be safely preserved in an overlapping type.
