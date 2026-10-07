# Phase 03 Goal — CRA-P02-C01 Host Monotonic Metadata On Every Publish

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P02-C01
Master Phase: 03 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 request to perform the Master PLAN remains active;
authorization is limited to this Phase after Phase 02's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Make host `loads_used` and run-peak metadata monotonic across every successful active-run publish. When a readable
current generation has a higher value than a valid incoming checkpoint, the newly installed current generation
must retain the incoming Core payload while storing the component-wise maximum host metadata. Current/backup/temp,
checksum, fsync, tombstone, failure and recovery behavior must remain unchanged.

This Phase is complete only when the current publish/read/rotation authority and every relevant dirty path are
identified; real-file tests first reproduce lower incoming loads, lower incoming peak and both together; the fix
max-merges readable current metadata into the incoming envelope before installation; incoming Core payload bytes
remain exact; focused and affected broad validation passes with nonzero counts; and a phase-exit audit returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical current-tree evidence found that publish writes the incoming envelope unchanged as the next current and
only raises metadata on the previous current before rotating it to backup. A valid writer with stale/lower host
metadata can therefore make the preferred current generation roll back `loads_used` or the run peak even though the
backup still remembers the maximum.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 03 registry row.
- Direct predecessor: Phase 02 CRA-F-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Historical documents identify the candidate and first proof but do not replace current store/source/test discovery.

## Priority And Current Disposition

- Priority axes: `2 / 3 / 2 / 1 / 3 / 3 / unknown`; known subtotal `14/18`.
- Final disposition at activation: `still_valid`, `correctness`, test first.
- Current live-tree disposition: implemented and reviewed `Complete / High`; see `COMPLETION_AUDIT.md`.

## Current And Target Authority

- Historically identified authority: `game_server::persistence::active_run_store::ActiveRunStore` owns host envelope
  version/checksum, current/backup/temp/lock/tombstone lifecycle and monotonic `loads_used`/run peak.
- Core owns the opaque payload and its checkpoint meaning; this Phase must not inspect, rewrite or replace Core
  payload bytes.
- Target authority: publish derives one to-be-installed envelope from the incoming payload plus component-wise
  maxima from any readable current host metadata, then uses the existing validated durable installation path.
- Exact current functions, file ordering, error behavior and owning test targets must be rediscovered before
  mutation and recorded in `EXPERIMENTS.md`.

## In Scope

- discover current publish, encode/checksum, current/backup rotation, recovery and actor publish consumers;
- capture status, scoped diff and hashes for every source/test/canonical target before editing;
- add real-file higher-current to lower-incoming cases for loads only, peak only and both;
- prove each successful publish installs the incoming Core payload with component-wise maximum host metadata;
- preserve current backup recovery and subsequent publication behavior;
- preserve write-failure atomicity, checksum, fsync, tombstone and explicit invalidation contracts;
- update the nearest canonical only if the implemented current contract is not described accurately.

## Out Of Scope

- cross-actor lease, compare-and-swap, logical run identity/conflict detection or multi-writer policy;
- changing Core V3 payload shape, meaning, validation, run anchor or any checkpoint mapping;
- old envelope/payload migration, repair, rebinding, silent corruption recovery or new fallback;
- changing load limits, peak calculation, actor transaction ordering or admin durability;
- redesigning file names, number of generations, lock scope, fsync strategy or public error shapes;
- unrelated server/Core cleanup or later mapping refactors.

## Preserved Invariants And Policy

1. The newly installed Core payload bytes are exactly the valid incoming bytes; host metadata merge never selects or
   reconstructs a previous Core payload.
2. `loads_used` and run peak are independently component-wise monotonic across successful publish, process reopen
   and approved backup recovery.
3. Current/backup/temp/lock/checksum/fsync/tombstone and publish-failure behavior remain current.
4. Old/unsupported/corrupt envelopes remain rejected according to the existing reader; they are not sources for a
   silent merge or migration.
5. CRA-POL-006 applies globally: no V3/RON/wire/public API migration is introduced.
6. No lease/conflict or run-identity semantics are inferred from this local-install monotonic correction.
7. Existing dirty user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct phase policy: none beyond global CRA-POL-006. Open policy decisions: none at activation. If the correction
requires choosing between competing writers/runs or changing failure/durability behavior, stop instead of guessing.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records this as the sole active Phase, read them completely. Then
capture the full expanded status plus exact target status/diff/hash before current production inspection. Treat all
pre-existing changes as user-owned and edit only the store/test/canonical seam proven necessary.

Use test-first proof in this order:

1. Run the existing active-run store module with a nonzero count.
2. Construct a real-file current generation with higher loads and/or peak and a distinct incoming Core byte payload.
3. Publish lower incoming metadata and prove the current read returns incoming Core bytes plus maxima for each of
   loads only, peak only and both.
4. Demonstrate the tests fail against current production for the intended rollback reason.
5. Implement the smallest correction in the existing publish/envelope authority.
6. Re-run focused store, actor consumer and affected package/static validation, then complete the phase-exit audit.

## Migration, Deployment And Compatibility

- Core checkpoint: opaque and byte-preserved; no V3 field/version/meaning change.
- Host envelope: same current version and serialized field set; only valid outgoing values become monotonic.
- Wire/public API: no DTO, command/result, error-code or Rust public-surface migration.
- Deployment: no old-file conversion. Existing readable current metadata participates; unsupported/corrupt current
  continues through the currently approved failure/recovery behavior.
- Temporary compatibility: none permitted or required.

## Validation Plan

- Baseline: full `ActiveRunStore` test module, actual nonzero count.
- First proof: three real-file higher-current to lower-incoming cases (loads, peak, both), each using distinct Core
  payload bytes and exact installed-current assertions.
- Focused regression: complete store module and any directly affected actor handler tests.
- Boundary regression: server library/package target chosen from current Cargo metadata; Core boundary only if source
  or opaque-payload assumptions change.
- Static: store call/rotation inventory, version/field search, scoped diff/status/hashes, canonical comparison,
  formatting and `git diff --check`.
- Zero-test filters are not evidence; runner counts and skips must be recorded exactly.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P03-C01 | current publish/read/rotation and consumer authority identified | current source/call inventory and E01 | pass |
| P03-C02 | dirty targets and concurrent work preserved | pre/post scoped status, diff and hashes | pass |
| P03-C03 | current rollback defect reproduced test-first | three real-file failing cases with exact cause | pass |
| P03-C04 | loads and peak max-merge independently on every publish | installed-current assertions for loads/peak/both | pass |
| P03-C05 | incoming Core payload and generation lifecycle remain exact | distinct payload plus backup/reopen/failure evidence | pass |
| P03-C06 | no unauthorized migration, repair or lease semantics | CRA-POL-006 and legacy/contract audit | pass |
| P03-C07 | focused and affected broad validation passes | runner-reported nonzero counts and static checks | pass |
| P03-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | pass |

## Stop Conditions

Stop this Phase and do not activate Phase 04 if the current store already max-merges every incoming publish; a
readable-current/corrupt-current choice requires new recovery policy; merge requires Core payload interpretation;
lease/conflict semantics are necessary; current durability/error behavior must change; a relevant dirty overlap is
inseparable; a focused runner selects zero tests; or the completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
