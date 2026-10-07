# Phase 01 CRA-P09-C08 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user explicitly requested execution of the Master PLAN on 2026-08-09; this activates the campaign and Phase
  01 production scope.
- Read the supplied existing-Goal resumption rules and all three required Master documents before production search,
  state inspection, tests or mutation.
- Phase 01 has no direct predecessor completion review. Master initialization plus user activation is its exact gate.
- Read only the evidence-linked audit/final documentation needed to initialize this Child.

### Evidence Carried Into Initialization

- Candidate: CRA-P09-C08, `correctness_correction (test)`, priority `1/3/1/2/3/1/unknown = 11/18`.
- Confirmed historical defect: the inline-range/preset-conflict fixture omits required `delay_ms`, and
  `catch_unwind(...).is_err()` accepts that unrelated panic.
- Required first proof: the repaired fixture parses; restoring only the intended defect fails with the exact typed
  object/path/cause. Evidence-linked bare `is_err` fixtures receive the same treatment.
- Direct predecessor: none. Downstream consumer: CRA-P09-C01 exact publication rejection.
- Policy: no gameplay choice; global CRA-POL-006 forbids collateral RON/wire/save/public migration.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Production path investigation: not started.
- Repository/target dirty-state inspection: not started.
- Test executions: none.
- Production/test/canonical mutation before initialization: none.

### Next Experiment

Record the activation dirty baseline, identify the exact current tests and owning Cargo target, then establish a
nonzero focused baseline before the first test edit.

## 2026-08-09 — E01 Current Target And Dirty Baseline

### Commands

```text
git rev-parse HEAD
git status --porcelain=v1 --untracked-files=all | wc -l
git status --porcelain=v1 --untracked-files=all | sha256sum
rg -n -C 10 'catch_unwind|delay_ms|preset[_ -]?conflict|inline[_ -]?range' core_enhanced --glob '*.rs' --glob '*.ron' --glob '!docs/**'
git status --short -- core_enhanced/src/game/data/skill_data.rs
git diff -- core_enhanced/src/game/data/skill_data.rs
git diff --numstat -- core_enhanced/src/game/data/skill_data.rs
git diff --check -- core_enhanced/src/game/data/skill_data.rs
sha256sum core_enhanced/src/game/data/skill_data.rs
```

### Actual Result

- Activation commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Post-Child-initialization full status: 370 entries; SHA-256
  `7053f818974ea677f7d996c33b93c41b6d4945aba016061903527863ee428e38`.
- Exact target: `core_enhanced/src/game/data/skill_data.rs`; status `M`; content SHA-256
  `8299928cceb348f9e5628c6605cdbce22f086e40996d02e667aba59cc62b9322`.
- Pre-Phase target diff: 167 insertions, 5 deletions; `git diff --check` has no output.
- Confirmed C08 tests: `skill_database_rejects_implicit_and_legacy_targeting_authority` and
  `skill_database_rejects_inline_target_range_and_preset_conflict` in the owning `game_core` library target.
- The conflict fixture currently omits unrelated required `name`, `kind`, `focus_time_ms`, `delay_ms` and `effects`
  while catch-any-panic accepts whichever failure occurs first.
- The implicit/legacy group uses bare `is_err` across fixtures with similarly incomplete unrelated fields.

### Dirty Overlap Decision

The target already contains user-owned `SkillCastSelection` deserialization/validation work, including added
`selection` fields in the affected fixtures. That intent is identifiable and must be retained exactly. Phase 01 can
merge safely by preserving those fields and restricting edits to complete fixture boilerplate, repaired controls and
exact assertions. Any need to change the selection contract itself stops this Phase.

### Next Experiment

Run both current filtered tests without edits, capture nonzero counts and the actual unrelated failure that the
conflict test accepts.

## 2026-08-09 — E02 Pre-Mutation Focused Baseline

### Commands

```text
cargo test -p game_core skill_database_rejects_inline_target_range_and_preset_conflict -- --nocapture
cargo test -p game_core skill_database_rejects_implicit_and_legacy_targeting_authority -- --nocapture
```

### Actual Result

- Conflict filter: 1 library test passed, 0 failed, 0 ignored/measured, 929 filtered; other emitted targets executed
  zero matching tests. The panic hook exposed `Message("missing field `delay_ms`")` at the fixture's `.unwrap()`;
  catch-any-panic nevertheless reported the test as passing.
- Implicit/legacy filter: 1 library test passed, 0 failed, 0 ignored/measured, 929 filtered; other emitted targets
  executed zero matching tests.
- Baseline is green but directly confirms the named false positive; it is not completion evidence.

### Decision

Replace the conflict catch/panic flow with an otherwise-valid parse control and exact returned-error assertion.
Repair the evidence-linked implicit/legacy cases so every negative is derived from complete valid boilerplate and
its error identifies the intended field/variant/validation cause. Do not alter runtime validation.

## 2026-08-09 — E03 Exact-Cause Test Correction

### Implementation

- Added complete self/enemy RON fixture builders, exact-one replacement enforcement and
  `assert_single_ron_defect`.
- Every RON negative now first parses a repaired control and then compares the restored defect's full error string.
  This covers 14 control/defect pairs: missing/unknown cast fields, missing target rule/range, removed legacy cast and
  step authorities, missing TileArea footprint and inline-range/preset conflict.
- Replaced every `catch_unwind`/bare `.is_err()` assertion in the target file with direct exact
  `DataValidationError { object_id, field_path, cause }` or exact RON error checks. The non-RON constructor tests now
  cannot pass on a different validation panic.
- Strengthened the existing unknown-buff cause from substring containment to exact equality.
- Runtime deserialization, validation, live data and public contracts were not changed by this Phase.

### Intermediate Failure And Correction

```text
cargo test -p game_core skill_database_rejects_inline_target_range_and_preset_conflict -- --nocapture
cargo test -p game_core skill_database_rejects_implicit_and_legacy_targeting_authority -- --nocapture
```

- Conflict filter after the first edit: 1 passed, 0 failed, 0 ignored; no panic hook output remained.
- First implicit/legacy rerun: 0 passed, 1 failed, 0 ignored. The test helper correctly rejected a supposedly
  one-time replacement because `target: EnemySingle(rule: Nearest)` occurred in both cast and step targets.
- Correction: scoped the replacement to `selection: automatic_unit, target: EnemySingle(...)` so only the intended
  cast target changes.
- Corrected implicit/legacy rerun: 1 passed, 0 failed, 0 ignored.

### Scope Expansion Decision

The candidate's removal scope says to replace catch-any-panic and bare `is_err` assertions, and the file inventory
showed programmatic validation cases beyond the two RON functions. Leaving them would preserve the same
false-positive mechanism inside the exact authority. They were therefore corrected within CRA-P09-C08, without
changing production behavior or absorbing a later candidate.

## 2026-08-09 — E04 Focused, Broad And Preservation Validation

### Commands

```text
cargo test -p game_core game::data::skill_data::tests --lib -- --nocapture
cargo test -p game_core --lib
cargo check -p game_core --all-targets
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core
rustfmt --edition 2021 --check core_enhanced/src/game/data/skill_data.rs
rg -n 'catch_unwind|\.is_err\(\)' core_enhanced/src/game/data/skill_data.rs
git diff --check -- core_enhanced/src/game/data/skill_data.rs
git status --short -- core_enhanced/src/game/data/skill_data.rs
sha256sum core_enhanced/src/game/data/skill_data.rs
```

### Actual Result

- `skill_data` module: 13 passed, 0 failed, 0 ignored/measured, 917 filtered.
- `game_core --lib`: 930 passed, 0 failed, 0 ignored/measured.
- `game_core` all-target check: pass.
- `ron_loading`: 25 passed, 0 failed, 0 ignored/measured.
- Full `game_core`: 979 physical tests passed (930 library + 3 item activation + 4 catalog + 25 RON + 13 skill
  runtime + 4 skill suite), 0 failed, 0 ignored/measured. Main, five fixture binaries and doc-tests each had zero
  tests and are not counted as behavioral passes.
- File-wide catch-any/bare-error search: no output.
- Target rustfmt check and `git diff --check`: no output.
- Final target status: `M`; SHA-256
  `b70a9c87c56058027af82daace21b7d2090ed90fa1f99b281512a2218063418b`.
- Full worktree status remains 370 entries with the activation SHA-256
  `7053f818974ea677f7d996c33b93c41b6d4945aba016061903527863ee428e38`; Phase edits changed content, not unrelated
  status membership.
- Pre-existing `SkillCastSelection`, compatibility checks and fixture `selection` fields remain present. Phase-owned
  production-file edits are confined to the `#[cfg(test)]` module.

### Omitted Validation

Unity, game_server, WebSocket and performance validation remain omitted: no runtime, RON data, DTO/wire, save,
public API or algorithm changed. The package's live-RON and all-target checks directly cover this test-only boundary.

### Decision

Implementation and required validation satisfy P01-C01~C07. Proceed to the independent completion review; do not
activate Phase 02 unless the verdict is `Complete / High`.

## 2026-08-09 — E05 Completion Audit And Exit

`COMPLETION_AUDIT.md` independently reread the current authority, canonicals, test implementation, dirty evidence
and final runner results. All eight completion rows pass; primary verdict is `Complete`, long-term fit is `High`,
and improvement class is `none`. Phase status transitions to `complete` with `Implementation Complete: true`.
Master may make Phase 02 ready; Phase 01 performs no further mutation unless a correction audit reopens it.
