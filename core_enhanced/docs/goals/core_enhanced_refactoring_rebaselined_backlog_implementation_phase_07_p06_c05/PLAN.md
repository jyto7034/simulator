# Phase 07 Goal — CRA-P06-C05 Use One Saturating MVP Score For Selection And Publication

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P06-C05
Master Phase: 07 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 06's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Make MVP selection and the published MVP DTO consume one explicitly saturating signed score evaluation. Every
accepted run-policy weight set, including extreme positive/negative values whose mathematical total exceeds `i64`,
must produce the same deterministic score for comparison and publication without debug panic or release wrapping.
Do not change weight authoring policy, live balance, metric meaning, eligibility, highlight/title or tie breaking.

This Phase is complete only when current result aggregation, policy weights, comparator, DTO construction,
highlight/title and tie paths are identified; extreme accepted-weight tests reproduce the comparator/publication
divergence or overflow; one shared evaluation returns the saturated total and highlight used by both selection and
publication; current live-policy goldens, incapacitated/redeployment aggregation and UUID ties remain exact;
focused/broad validation passes; and a phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found that MVP comparison sums already saturated per-metric products with ordinary `.sum()`,
while DTO publication independently accumulates with `saturating_add`. Because accepted weights are signed `i64`,
extreme valid policy can panic in debug or wrap in release and select a different employee than the published score
would justify.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 07 registry row.
- Direct predecessor: Phase 06 CRA-P05-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_06_p05_c01/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_06_battle_lifecycle_result_and_boss_scheduling/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_06_battle_lifecycle_result_and_boss_scheduling/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_06_battle_lifecycle_result_and_boss_scheduling/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

No direct policy ID is assigned. Arithmetic consistency is policy-free; changing accepted weight signs,
duplicates, ranges or live values requires a separate policy and is out of scope.

## Priority And Current Disposition

- Priority axes: `2 / 2 / 1 / 2 / 3 / 2 / unknown`; known subtotal `12/18`.
- Final disposition at activation: `still_valid`, correctness correction, extreme-value tests first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historical current authority: comparator and DTO independently calculate total score with different overflow
  semantics; run-policy weights allow the extreme signed values that expose the split.
- Target authority: one pure, saturating score evaluation owns total and highlight. Comparator selects by that total
  and DTO publishes the exact same evaluation result.
- Current function/type names, metric iteration order and title resolution must be rediscovered after the required
  document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify aggregation row, metric getters, policy weight storage/validation, comparator/tie and DTO/title paths;
- capture expanded status, scoped diffs and hashes for every prospective source/test/canonical target;
- add extreme positive and negative multi-weight cases, including intermediate/final overflow in both directions;
- assert selected employee, published employee, total score and highlight are derived from one evaluation;
- preserve deterministic UUID fallback, live-policy score/title goldens, incapacitated eligibility and aggregation
  across multiple deployed instances of one employee;
- remove only duplicate/divergent total arithmetic and update the nearest canonical after behavior is real.

## Out Of Scope

- rejecting, clamping or normalizing authored weights at publication;
- deciding whether negative or duplicate metric weights are desirable;
- changing live RON weights, metric definitions, score titles/highlight priority or eligibility;
- changing result aggregation, representative instance identity, DTO/wire field shape or server transport;
- adjacent event-log validator, ScenarioEvent or debug I/O cleanup;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. Current live policy produces identical MVP employee, score, title, highlight and deterministic ties.
2. Employee rows aggregate all deployments/redeployments before scoring; MVP uses the same aggregated row.
3. Existing incapacitation eligibility and representative instance identity remain unchanged.
4. Signed metric products and their total use explicit `i64` saturation in a deterministic iteration order.
5. Comparator and DTO cannot observe different score calculations.
6. RON, policy schema, DTO/wire/public Rust and server shapes remain unchanged.
7. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded dirty status and prospective target status/diff/hash before inspecting current source/tests/canonical.
Treat all existing changes as user-owned and edit only new extreme proofs, the shared score seam and nearest
canonical.

Use test-first proof in this order:

1. Run current result-stats and run-policy focused tests with nonzero counts.
2. Trace aggregation→candidate comparison→tie→DTO/title and enumerate all score arithmetic.
3. Add positive-overflow and negative-underflow candidate matrices that are accepted by the current policy type.
4. Demonstrate current comparator panic/wrap/divergence without weakening assertions for build mode.
5. Introduce or reuse one evaluation result carrying saturated total and highlight; route comparison and DTO through
   it and remove duplicate total arithmetic.
6. Re-run exact extremes, live goldens, tie, redeployment/incapacitation, result and owning-package suites.

## Migration, Deployment And Compatibility

- No data, schema, save/checkpoint, DTO/wire or version migration.
- No live balance change and no new policy rejection.
- Extreme inputs that were already accepted become defined and consistent; ordinary inputs remain byte/value
  equivalent.
- Server/Unity consume the same result DTO fields and require no rollout change.

## Validation Plan

- Baseline: result-stats unit family and run-policy validation with nonzero counts.
- First proof: exact positive/negative extreme matrix and comparator/DTO identity.
- Focused final: extremes, live policy golden, ties, eligibility, redeployment aggregation and title/highlight.
- Broad: result aggregation/world combat-result families, live RON validation, Core package and server compile where
  the shared DTO is consumed.
- Static: all score arithmetic/callers, `.sum()`/`saturating_add` inventory, policy-field and serialized-shape audit,
  scoped status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P07-C01 | current aggregation/scoring/comparator/DTO authorities identified | complete call/type/arithmetic inventory | completed |
| P07-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | completed |
| P07-C03 | extreme accepted weights reproduce unsafe/divergent scoring | positive/negative test-first matrix | completed |
| P07-C04 | one saturating evaluation owns selection and publication | comparator/DTO exact identity | completed |
| P07-C05 | live/tie/eligibility/redeployment behavior is unchanged | focused regression matrix | completed |
| P07-C06 | RON/schema/DTO/wire/public compatibility is unchanged | static and live validation | completed |
| P07-C07 | focused and affected broad validation passes | nonzero counts and static checks | completed |
| P07-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | completed — Complete / High |

## Stop Conditions

Stop and do not activate Phase 08 if current comparator and DTO already use one saturating total; reproducing the
problem requires changing accepted authoring policy or live balance; one result cannot preserve current highlight/
tie meaning; serialized/public result shape must change; dirty overlap is inseparable; a focused runner selects zero
tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.

## Completion Result

- Phase exit audit: `Complete / High`.
- Improvement class: `none`.
- One private saturating evaluation now owns candidate comparison and published score/title/highlight.
- Complete Core validation: 944 passed, 0 failed; no data, DTO, wire or persistence migration.
- Master may mark Phase 07 complete and make Phase 08 ready.
