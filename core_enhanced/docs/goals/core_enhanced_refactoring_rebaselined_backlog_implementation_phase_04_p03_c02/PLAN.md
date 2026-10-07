# Phase 04 Goal — CRA-P03-C02 Atomic Enhancement Material Aggregation

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P03-C02
Master Phase: 04 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 03's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Make equipment enhancement material admission and consumption atomic when a recipe contains repeated material IDs.
The command must use a checked per-material aggregate plan: insufficient aggregate balance or arithmetic overflow
fails before inventory/equipment/diff mutation, while exact aggregate balance consumes the total once and performs
the existing single enhancement with unchanged result semantics.

This Phase is complete only when current recipe publication, command validation, execution, authoritative inventory
and result-diff paths are identified; duplicate-row insufficient and exact-aggregate tests are written first and
reproduce the defect; the smallest domain-owned aggregate/staging correction is implemented; current single-row
behavior, bound/max/error/result contracts and live data remain compatible; focused/broad validation passes; and a
phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found that validation compares each recipe row with the same pre-command material balance, but
execution consumes rows sequentially from live inventory and raises the equipment level only afterward. A recipe
such as `(dust, 2), (dust, 2)` with balance 2 passes admission, consumes 2, then fails, leaving a failed command with
partially mutated inventory and no successful enhancement result.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 04 registry row.
- Direct predecessor: Phase 03 CRA-P02-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_03_p02_c01/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Historical findings define the defect class and proof, not the current implementation seam.

## Priority And Current Disposition

- Priority axes: `1 / 3 / 1 / 2 / 3 / 3 / unknown`; known subtotal `13/18`.
- Final disposition at activation: `still_valid`, correctness, test first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historically identified publication authority validates nonempty positive recipe rows and referenced materials but
  permits repeated IDs.
- Historically identified runtime validation reads each row against the same pre-command balance; execution consumes
  live material rows sequentially before increasing the equipment level.
- Target authority: one checked per-material aggregate plan shared by validation and execution/staging so the
  authoritative inventory and equipment commit only after the entire command can succeed.
- Exact live functions, recipe/data types, result-diff producer, callers and owning targets must be rediscovered
  before mutation and recorded in `EXPERIMENTS.md`.

## In Scope

- discover current enhancement recipe publication/validation, command handler, material inventory, equipment level
  and diff construction paths;
- capture status, scoped diff and hashes for every affected source/test/live-data/canonical target before editing;
- add a duplicate-row insufficient-aggregate test that proves unchanged material inventory, equipment state and
  outward diff/result on failure;
- add an exact-aggregate test that proves total consumption and exactly one existing enhancement result;
- use checked aggregation so duplicate sum overflow is rejected before mutation;
- preserve current single-row recipes and result/error/bound/max-level semantics;
- update the nearest canonical only after implemented behavior is real.

## Out Of Scope

- changing recipe quantities, prices, balance, level caps or production content;
- changing equipment ownership/loadout bindings, enhancement stat formula or Boss setup consumption;
- redesigning generic grant/economy state, inventory public APIs or later C03/C04 refactors;
- rejecting all duplicate rows if that would prevent the required exact-aggregate success contract;
- changing V3/RON/wire/public API shape, command/result tags or diff ordering beyond aggregation of one logical cost;
- unrelated cleanup or dirty-worktree normalization.

## Preserved Invariants And Policy

1. Failed enhancement leaves material balances, equipment level/state and returned/observable diff unchanged.
2. Duplicate rows for one material are one logical checked total; exact aggregate balance succeeds once.
3. Single-row live recipes retain cost, level increment, errors, bound/max gates and result-diff semantics.
4. Recipe publication remains strict for empty costs, zero quantities and missing material references.
5. Addition/consumption arithmetic is checked; overflow is not saturated or partially consumed.
6. CRA-POL-006 applies globally: current V3/RON/wire/public API contracts remain unless separately migrated.
7. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct phase policy: none beyond global CRA-POL-006. Open policy decisions: none at activation. If exact aggregation
requires choosing new balance/content, outward error semantics or public shape, stop rather than guess.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded status and exact target status/diff/hash before current source/data/test inspection. Treat all existing
changes as user-owned and edit only the enhancement aggregation proof/authority/canonical seam.

Use test-first proof in this order:

1. Run current enhancement tests with a nonzero count.
2. Build a valid recipe with duplicate material rows and an otherwise eligible owned equipment instance.
3. Prove insufficient aggregate currently mutates material state or otherwise violates exact atomic rejection.
4. Prove exact aggregate must consume the full total and increment exactly once.
5. Add checked aggregate/staging at the narrowest existing domain authority.
6. Re-run focused equipment, live RON/publication and owning-package validation plus static/dirty review.

## Migration, Deployment And Compatibility

- Live RON/data: no schema or intended content change; duplicate rows use their summed current meaning.
- Save/checkpoint: existing inventory/equipment fields and Phase 02 invariant graph stay unchanged.
- DTO/wire/public Rust API: preserve commands, results, errors, nesting and field types.
- Deployment: no migration. Existing duplicate recipes, if any, become atomic and aggregate-correct.
- Temporary compatibility: none.

## Validation Plan

- Baseline: current enhancement-focused tests, actual nonzero count.
- First proof: duplicate insufficient aggregate with full pre/post state and no diff/result mutation; duplicate exact
  aggregate with exact total consumption and one level/result.
- Overflow: checked duplicate-total overflow fails before mutation if reachable through the current recipe type.
- Focused regression: complete equipment enhancement test family and relevant data/publication validation.
- Broad regression: owning Core library/package and live RON tests selected from current Cargo topology.
- Static: caller/validator/consumer inventory, live recipe search, scoped status/diff/hashes, compatibility/canonical
  comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P04-C01 | current publication/validation/execution/diff authorities identified | current call/data inventory and E01 | complete |
| P04-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete |
| P04-C03 | duplicate insufficient aggregate defect reproduced | test-first full no-mutation failure evidence | complete |
| P04-C04 | exact aggregate succeeds once and consumes total | positive duplicate aggregate assertions | complete |
| P04-C05 | overflow and all failure paths are pre-mutation | checked arithmetic and negative evidence | complete |
| P04-C06 | live single-row/data/public contracts remain exact | live search, existing tests and compatibility audit | complete |
| P04-C07 | focused and affected broad validation passes | nonzero runner counts and static checks | complete |
| P04-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete |

## Stop Conditions

Stop and do not activate Phase 05 if the current implementation already aggregates atomically; production duplicate
recipe meaning conflicts with exact-aggregate success; the required result/error semantics are undecidable; public
schema/API migration or balance change is required; dirty overlap is inseparable; a focused runner selects zero
tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
