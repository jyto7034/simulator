# Core Enhanced Refactoring Post-Boss Reaudit Phase 01 Experiments

Status: complete

## 2026-08-09 — Phase initialization

### Checks

- Master P00 status is `complete — Complete / High`; P01 is `ready`; P02 remains `pending`.
- P00 completion audit transitions to complete and permits P01 activation.
- Registered P01 path was absent.
- Campaign-excluded status remains the P00 272-entry hash `8d03ae2f...`.

### Test result

No runner executed; pass/fail/skip counts are `not_applicable` before Phase documents exist.

### Decision

Initialize the three P01 documents as `investigation_only`, activate only P01, then read current and historical
candidate evidence.

## 2026-08-09 — Current publication and candidate reaudit

### Hypothesis

Current source should show whether C01/C02 are fully closed beyond their Boss subset and whether the new Boss catalog
changes or closes C03's private mapping problem.

### Performed checks

- Read all original P01 Goal/artifact documents as historical detail.
- Traced current server loader, macro inventory, parser/domain validation, candidate assembly, manifested preview/
  setup validation and Arc publication.
- Compared all current embedded/direct RON paths with all physical production RON files.
- Read current C01/C02 negative tests, official live RON suite, canonical and consumer references.
- Searched current workspace/external Unity for builder/parts/config consumers.

### Exact commands and results

```text
comm -3 <normalized source include paths> <normalized physical production RON paths>
# no output
<source include count>
# 24
rg --files game_resources/data -g '*.ron' | wc -l
# 24

cargo test -p game_core embedded_source_inventory_matches_every_physical_gameplay_ron_exactly -- --nocapture
# 1 passed, 0 failed, 0 ignored; package filter nonmatches not counted

cargo test -p game_core live_candidate_rejects_preview_failure_unique_to_manifested_difficulty -- --nocapture
# 1 passed, 0 failed, 0 ignored

cargo test -p game_core live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup -- --nocapture
# 1 passed, 0 failed, 0 ignored

cargo test -p game_core --test ron_loading -- --nocapture
# 25 passed, 0 failed, 0 ignored
```

### Actual result

- C01 target is fully present: path/count derive from one macro, exact 24-file test exists, canonical says 23
  non-manifest sources, no fallback loader exists.
- C02 target is fully present: baseline plus every profile/band/role/stage and five seeds use difficulty-aware
  preview, same-preview warning and full setup feasibility before Arc return.
- C03 remains: repeated parsed/parts/database/builder mapping continues. Boss raw compile is performed once for PVE
  reference validation and again for publication, strengthening the existing private conversion root cause.
- No direct public parts consumer was found in workspace, but builder has many fixture/test consumers and external
  Rust API use remains unknown.
- No config consumer was found; deletion remains unverified.

### Test result

Meaningful current executions: 28 passed, 0 failed, 0 ignored/skipped. Package-wide name filters also reported
zero-test bin targets and filtered nonmatches; they are not counted as executions or skips.

### Decision

- C01 `fully_implemented`, Boss coimplementation confirmed over full scope.
- C02 `fully_implemented`, Boss prerequisite claim confirmed over full scope.
- C03 `still_valid`, work type unchanged; Boss compile duplication is merged evidence, not a new ID.
- New `CRR-P01-*` candidates: none.

## 2026-08-09 — Artifact and preservation validation

### Performed checks

- Wrote `CANDIDATE_REAUDIT.md`, `TEST_REAUDIT.md` and `BOUNDARY_AUDIT.md`.
- Checked three exact CRA summary rows, no issued CRR ID, Master active registry, trailing whitespace, production
  content manifest and campaign-excluded status.

### Actual result

- Three assigned CRA rows each have one current disposition.
- No `CRR-P01-Cyy` ID exists; the document explicitly records zero new candidates.
- Only P01 is active.
- Trailing whitespace output is empty.
- Campaign-excluded 272-entry status hash remains `8d03ae2f...`.
- Relevant 96-file production/current-method manifest remains `9638cfa2...`.

### Decision

Proceed to independent P01 completion audit. No broad runner is needed beyond the 28 current focused/integration
executions because no production contract changed and the dispositions have direct assertion evidence.

## 2026-08-09 — Phase exit audit

- Independent `COMPLETION_AUDIT.md` rechecked P01-C01~C08 against original contracts and current evidence.
- All eight rows pass; primary verdict `Complete`, fit `High`.
- Final hashes remain exact P00 baseline outside audit artifacts.
- P01 transitions to complete and hands P02 readiness to the Master.
