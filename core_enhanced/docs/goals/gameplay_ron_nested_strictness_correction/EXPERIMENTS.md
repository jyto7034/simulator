# Gameplay RON Nested Strictness Correction Experiments

Status: complete

## 2026-07-18 — Child Initialization

- Read the parent Master G01 scope, dependencies, invariants and completion gates.
- Confirmed the child path was available.
- Recorded broad pre-existing dirty changes across the relevant Core/data surfaces.
- Created the mandatory child documents before code search or mutation.
- No production/test/canonical file changed and no test ran during initialization.

## 2026-07-18 — Strictness Implementation And Validation

- Added `deny_unknown_fields` to the seven demonstrated shared nested gameplay types and the
  battlefield wrapper DTO.
- Added production-shaped source-set regressions for abnormality movement/resonance, skill
  range/delivery/effect, equipment stat modifier and map Support payload.
- Added a direct battlefield wrapper unknown-field regression; confirmed the production file root
  is already a strict vector of strict definitions.
- Focused unknown-source tests: 5 passed.
- Battlefield wrapper test: 1 passed.
- Data-focused baseline after attribute change: 125 passed before new regressions; new regressions
  are included in the final 737-test lib count.
- `ron_loading`: 20 passed.
- Core all-targets: 781 passed, 0 failed, 0 ignored.
- No live RON content change was required.

Result: G01 complete; `DCA2-DATA-002` corrected.
