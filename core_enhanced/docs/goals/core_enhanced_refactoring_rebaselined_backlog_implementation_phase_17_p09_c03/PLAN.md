# Phase 17 Goal — CRA-P09-C03 Evaluate Usefulness Over Actual Delivery Targets

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C03
Master Phase: 17 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 16's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Implement CRA-POL-004 with one delivery-aware usefulness authority. Overall cast admission must accept a skill when
any executable step has a useful effect over that step's actual materialized/clipped/filter-applied delivery target
set. After admission, each executing step must evaluate only its own current actual delivery set. Selection,
admission and execution must therefore agree for anchor-excluding TileArea footprints and multi-step skills without
changing target ordering, release timing or no-target lifecycle.

This Phase is complete only when the current usefulness producer/caller graph is identified; test-first decision
tables reproduce anchor-only and first-hostile-step defects; direct and TileArea delivery sets include the exact
anchor/materialization/clipping/hit-filter semantics runtime will execute; automatic and manual admission use the
any-step rule; execution gates only the current step; useful-later/useless-first and useful-first/useless-later cases
retain lifecycle/order; live choices and data remain valid; public/RON/save/wire shapes remain exact; focused and
broad validation pass; and the independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical and re-audited evidence found that EnemySingle TileArea acquisition checks usefulness on the selected
anchor unit even though runtime delivery later materializes a footprint, clips it to the valid board and applies
`hit_targets`/caster filters. A schema-valid footprint that excludes its anchor can therefore accept a useful anchor
while affecting nobody, or reject a useless anchor even though a useful neighbor is actually hit. The current
first-hostile-step approximation also cannot implement the approved multi-step rule.

Current live EnemySingle TileArea footprints include their anchors, so this is a published-contract hardening
correction rather than an asserted current live balance incident. Valid live choices must remain stable.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 17 registry row.
- Direct linear predecessor: Phase 16 CRA-P09-C06, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_16_p09_c06/COMPLETION_AUDIT.md`.
- Required spatial seam: crate-private typed tile reach query/projection from Phase 16.
- Approved policy: CRA-POL-004 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Cross-cutting compatibility policy: CRA-POL-006.
- Downstream dependency: Phase 18 CRA-P09-C04 depends on this Phase and Phase 16 before correcting preview meaning.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

CRA-POL-004 is explicit: cast admission is any-step over actual delivery targets; execution/no-op gating is
current-step only. A no-op current step cannot retroactively cancel a cast admitted because a later step is useful.
No policy question remains open unless current runtime cannot express these semantics without changing an unrelated
lifecycle or target-filter contract.

## Priority And Current Disposition

- Priority axes: `3 / 2 / 2 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Final disposition at activation: `still_valid`, correctness correction, actual-set tests first.
- Current live-tree disposition: discovery pending until all three Child documents and the complete final Phase 16
  audit are fully read and the scoped dirty baseline refreshes historical source/test/canonical targets.

## Current And Target Authority

- Historical current authority: usefulness helpers choose one representative/first hostile target-bearing step;
  EnemySingle TileArea acquisition evaluates the anchor, while area runtime alone owns actual affected target sets.
- Target authority: one typed executable-usefulness plan/query resolves each step's exact delivery targets using the
  same geometry/filter semantics as runtime. Cast admission folds those per-step results with `any`; step execution
  consumes only the current step result.
- Exact current type/function/file names and the smallest viable ownership location must be rediscovered after the
  Child read/dirty gate. Historical names are evidence, not mutation instructions.

## In Scope

- inventory every skill usefulness producer and consumer at automatic selection, manual validation, cast admission,
  retargeting, step execution and TileArea no-op boundaries;
- capture expanded dirty status, scoped diffs and fingerprints before production edits;
- add decision-table tests first for anchor-excluding EnemySingle footprints, clipped/empty cells, useless
  anchor/useful affected neighbor, useful anchor/empty or filtered area, ally/enemy/all filters and caster inclusion;
- cover useful-later/useless-first and useful-first/useless-later steps through admission and execution;
- resolve actual direct/projectile/TileArea delivery target sets without applying effects during targeting;
- share the Phase 16 typed footprint projection and the runtime's stable target/filter rules;
- preserve stable ordering/ties, current target choices for live skills, release timing and no-target/no-op cleanup;
- update the nearest targeting canonical only after implementation is real.

## Out Of Scope

- Phase 18 CRA-P09-C04 effect-preview footprint/union correction or any DTO content/shape change;
- applying damage, heal, buff or other Phase 8 effects during targeting/usefulness evaluation;
- changing effect usefulness classification beyond CRA-POL-004's delivery-set aggregation;
- changing target rules, air eligibility, cast reach, TileArea geometry, filters, tracking, projectile motion,
  delivery terminal lifecycle, damage/effect ordering or balance values;
- public Rust, live RON, save/checkpoint, event, DTO/wire/server/Unity schema migration;
- Boss navigation/target locking, Phase 29 motion extraction or unrelated cleanup/refactor;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. Overall admission is any-step and execution/no-op is current-step exactly as CRA-POL-004 specifies.
2. Actual delivery targets match runtime materialization, valid-board clipping, hit filters and caster inclusion.
3. A useful anchor cannot validate an empty/filtered footprint; a useless anchor cannot invalidate useful affected targets.
4. Stable unit ordering, target rules, ties, current live choices and air/lifecycle eligibility remain exact.
5. A no-op current step does not cancel later scheduled useful work or alter release/terminal timing.
6. Usefulness simulation does not apply effects or mutate authoritative battle state.
7. Cast acquisition reach and delivery footprint remain distinct Phase 16 query inputs.
8. Public Rust, RON, V3 checkpoint, DTO/wire/server/Unity shapes remain exact under CRA-POL-006.
9. Phase 18 preview policy remains unimplemented until its own activation.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 16 completion audit. Capture expanded status and prospective target diff/hash before inspecting production
source/tests/live data. Treat every existing hunk as user-owned until reconciled.

Use test-first proof in this order:

1. Trace cast admission versus step execution usefulness and the exact runtime delivery-target producers.
2. Run current selector/manual-cast/TileArea/multi-step/live baselines.
3. Add anchor-exclusion/filter/clipping and two-order multi-step decision tables; prove nonzero intended failures.
4. Introduce the smallest typed actual-delivery usefulness plan/query and migrate admission/execution one boundary at a time.
5. Remove only anchor/first-step approximations proven obsolete; keep effect execution downstream.
6. Re-run focused, complete Core, affected live/server/release and static/dirty/compatibility checks.

## Migration, Deployment And Compatibility

- CRA-POL-004 changes schema-valid runtime behavior but requires no schema or content migration.
- Current live definitions should keep the same choices because their EnemySingle TileArea footprints include anchors;
  any current-live choice drift is a stop condition unless directly required by the approved actual-set rule.
- Public preview/event/save/wire shapes remain unchanged. Phase 18/P10 own later projection/transport corrections.
- Unity/runtime probes are omitted unless inspection finds a serialized shape or message-order change, which triggers
  a policy stop.

## Validation Plan

- Baseline: usefulness/target selector, manual activation, TileArea runtime, multi-step skill runtime and live RON.
- First red: actual-set decision table for anchor exclusion, clipping and filters plus multi-step admission/execution
  table in both useful-step orders.
- Focused final: same decision tables through production selection, admission and execution; Phase 16 query parity.
- Broad: complete affected Core integration families, Core lib, game_server targets and release.
- Static: usefulness producer/caller inventory, anchor/first-step shortcut search, public/serde/save/wire comparison,
  scoped status/diff/hashes, canonical comparison, formatter and diff check.
- Zero-test filters, helper-only predicates, compile-only success and snapshot-only updates do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P17-C01 | complete usefulness/caller graph identified | admission/selection/step/delivery target inventory | complete |
| P17-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P17-C03 | historical anchor and first-step defects reproduced | nonzero intended-red actual-set/multi-step tables | complete |
| P17-C04 | one actual-delivery usefulness authority exists | source/caller and shortcut-removal audit | complete |
| P17-C05 | cast admission follows any-step actual-set rule | automatic/manual positive and negative tables | complete |
| P17-C06 | execution follows current-step actual-set rule | useful-later/useless-first lifecycle/order proof | complete |
| P17-C07 | filters/clipping/order/live behavior remain exact | footprint/filter/edge/live positive matrix | complete |
| P17-C08 | public/data/save/wire compatibility remains exact | shape/live-content/consumer audit | complete |
| P17-C09 | focused and broad validation passes | nonzero Core/live/server/release/static counts | complete |
| P17-C10 | Phase 18 seam/canonical and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 18 if actual runtime delivery targets cannot be evaluated without applying effects;
current lifecycle semantics conflict with CRA-POL-004; automatic and manual admission require an unapproved split;
live choices drift beyond the approved actual-set correction; target ordering/filter semantics would change; a
public/RON/save/wire migration is required; dirty overlap is inseparable; a required runner selects zero tests; or
completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
