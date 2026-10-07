# Phase 13 Goal — CRA-P08-C04 Close Equipment Effect And Activation Publication Validation

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P08-C04
Master Phase: 13 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 12's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make whole-bundle publication the authoring authority for every equipment triggered-effect and ability-activation
edge. Invalid buff/skill references and trigger/effect/target combinations must fail atomically with exact
equipment/path/cause before runtime state is published, while the current official bundle and every approved
Permanent, start, attack/hit and death-family occurrence remain exact.

This Phase is complete only when the current source→parse→equipment/local→cross-catalog→publication path and every
runtime effect/activation consumer are identified; exact missing-buff, missing-skill, invalid-Permanent and exhaustive
occurrence-matrix tests first prove the current holes; one explicit typed compatibility authority matches Phase 11's
dispatcher and CRA-POL-002 retaliation semantics; reference validation traverses every authored equipment edge and
reports deterministic object/field/cause provenance; invalid candidates never partially publish; runtime defensive
lookups remain integrity guards rather than authoring policy; full production RON loads unchanged; public/RON/save/
wire shapes remain compatible; focused/broad validation passes; and the phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audit and reaudit evidence found that equipment validation closes local weapon, recipe and event metadata
but does not join triggered `ApplyBuff` IDs to the buff catalog, activation ability IDs to the skill catalog, or
authored trigger/effect/target rows to what runtime occurrence dispatch can execute. Missing references and dropped
effect combinations can therefore pass publication and become late silent no-ops. A hand-written two-ability live
integration list is not complete database-edge validation.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 13 registry row.
- Direct predecessor: Phase 12 CRA-P08-C02, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_12_p08_c02/COMPLETION_AUDIT.md`.
- Runtime semantic prerequisite: Phase 11 CRA-P08-C01 implemented the typed occurrence dispatcher and
  CRA-POL-002 retaliation contract; its `Complete / High` audit is the executable-matrix boundary.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/BOUNDARY_AUDIT.md`
- Final execution/policy evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

CRA-POL-002 defines OnHit Counterpart BonusDamage as once-per-authored-effect retaliation against the attacker,
owner-attributed and non-recursive; the publication matrix must accept that row as retaliation rather than current-hit
amplification. CRA-POL-006 requires strict whole-bundle failure without unrelated normalization and preserves current
RON/public/save/wire shapes.

## Priority And Current Disposition

- Priority axes: `2 / 2 / 1 / 3 / 3 / 2 / unknown`; known subtotal `13/18`.
- Final disposition at activation: `still_valid`, correctness correction, approved reference/matrix tests first.
- Current live-tree disposition: `still_valid`; implementation and broad validation complete, phase-exit review
  pending.

## Current And Target Authority

- Historical current authority: equipment-local validation checks local metadata, while global cross-catalog
  validation joins other catalogs but omits equipment effect/activation edges; runtime resolves missing references
  late and drops unsupported occurrence combinations defensively.
- Target authority: one publication-time compatibility/reference contract traverses every equipment effect and
  activation edge, resolves catalog IDs and rejects rows not executable by the current typed dispatcher. Runtime
  retains defensive lookups for integrity but does not define authoring validity.
- Current module/type names, loader staging, error carrier, matrix ownership and exact live rows must be rediscovered
  after the required document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify embedded/source manifest, parser, equipment-local validation, cross-catalog join, atomic publication,
  runtime item/effect/activation materializers, official tests and error-provenance carriers;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical/live-data target;
- add exact programmatic and official-source negative cases for missing ApplyBuff buff ID and activation skill ID;
- add invalid Permanent effect/target rows and an explicit trigger × effect × target compatibility table covering
  Permanent, OnBattleStart, OnAttack, OnHit, OnDeath, OnKill and OnAllyDeath;
- distinguish OnAttack/OnHit current-hit damage modifiers, ordinary target-materialized effects and CRA-POL-002
  OnHit Counterpart BonusDamage retaliation exactly as the Phase 11 dispatcher executes them;
- require exact equipment identity, authored field/path and cause in deterministic validation errors;
- prove whole-candidate/bundle failure atomicity and no partial catalog publication;
- load the complete current production bundle and preserve all local weapon/profile/recipe/event metadata validation;
- replace or derive any ad hoc two-ID integration assertion from complete database-edge validation without reducing
  the existing runtime integration behavior proofs;
- update the nearest data/canonical contract only after runtime publication behavior and tests are current.

## Out Of Scope

- changing CRA-POL-002 retaliation runtime meaning, target resolution, occurrence ordering or Phase 11 dispatcher;
- implementing Phase 14 max-kills terminal delivery or Phase 15 projectile/basic publication compatibility;
- consolidating committed damage under P08-C05 or removing the disconnected DeathHandler under P08-C06;
- changing authored equipment/buff/skill IDs, balance values, effect enum/RON schema, public Rust construction shape,
  save/checkpoint, DTO/wire tags/nesting or server/Unity contract;
- removing runtime defensive lookups solely because publication now validates authored data;
- introducing permissive fallback, alias, default normalization, partial publication or compatibility dual paths;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. The complete current official bundle loads and produces the same typed data.
2. Valid Permanent, battle-start, attack/hit, OnDeath, OnKill and OnAllyDeath behavior remains exact.
3. CRA-POL-002 retaliation is accepted once, owner-attributed and non-recursive; invalid amplification-like rows do
   not bypass the executable matrix.
4. Publication is strict, deterministic and failure-atomic; no invalid candidate becomes partially visible.
5. Error provenance identifies the exact equipment and authored reference/matrix path plus cause.
6. Existing local weapon/profile/recipe/event validation and strict unknown-field behavior remain intact.
7. Runtime defensive missing-reference handling remains an integrity guard, not a second authoring authority.
8. Current RON, public Rust, save/checkpoint, DTO/wire/server/Unity shape remains exact under CRA-POL-006.
9. No future Phase behavior, compatibility shim, alias, fallback or placeholder contract is introduced.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture expanded
dirty status and prospective target status/diff/hash before inspecting current source/tests/data/canonical. Treat all
existing changes as user-owned and edit only the publication reference/matrix authority, exact tests and nearest
canonical.

Use test-first proof in this order:

1. Run nonzero equipment/local validation, full RON publication and Phase 11 dispatcher/live-item baselines.
2. Trace actual source parsing through atomic publication and every equipment reference/runtime consumer.
3. Add missing buff/skill and invalid Permanent negatives with exact object/path/cause; establish red results.
4. Derive the complete executable matrix from the current typed dispatcher and add exhaustive positive/negative
   table proofs, including CRA-POL-002 retaliation.
5. Implement the smallest typed publication validator at the cross-catalog authority; keep runtime integrity guards.
6. Re-run exact failures, valid official bundle, occurrence/live behavior, complete Core and affected server/static
   checks.

## Migration, Deployment And Compatibility

- No RON schema/data-ID, save/checkpoint, DTO/wire, API version or public Rust migration.
- Strict validation begins rejecting invalid authored combinations that runtime cannot execute; the current official
  bundle must need no content rewrite.
- No dual validator, fallback, alias or compatibility interval. The existing atomic publication boundary owns
  cutover.
- If the implemented Phase 11 matrix cannot be expressed without changing current valid content or public/RON shape,
  inventory exact rows and stop for direction rather than silently normalizing or migrating them.
- Server/Unity transport rollout is unnecessary because no serialized battle contract changes.

## Validation Plan

- Baseline: equipment validator, production bundle/RON, buff/skill reference loaders, Phase 11 dispatcher and live
  item integrations with nonzero counts.
- First proof: missing buff ID, missing activation skill ID, invalid Permanent effect/target and exhaustive occurrence
  rows with exact object/path/cause and atomic rejection.
- Focused final: every supported matrix row, representative unsupported row for every rejected class, current live
  IDs/behaviors, local metadata retention and official-source provenance.
- Broad: complete loader/publication families, trigger/skill/buff/live integrations, complete Core and affected
  game_server targets.
- Static: complete equipment edge inventory, runtime defensive-lookup disposition, obsolete ad hoc ID-list search,
  public/RON/save/wire shape audit, scoped status/diff/hashes, canonical comparison, formatting and
  `git diff --check`.
- Zero-test filters, parse-only success and test-fixture-only validation do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P13-C01 | complete parse/publication/reference/runtime graph identified | typed source→candidate→validation→publish→consumer inventory | complete — E01 |
| P13-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete — E01 |
| P13-C03 | current missing-reference/matrix gaps reproduced | exact programmatic/official red proofs | complete — E02 |
| P13-C04 | every equipment buff/skill edge resolves before publication | exhaustive reference traversal and errors | complete — E03 |
| P13-C05 | explicit executable occurrence matrix matches Phase 11/CRA-POL-002 | cross-product positive/negative table | complete — E03 |
| P13-C06 | strict failure atomicity and exact provenance hold | unchanged installed state plus object/path/cause | complete — E03 |
| P13-C07 | current bundle and valid occurrence behavior remain exact | live RON/dispatcher/item/local metadata proofs | complete — E03 |
| P13-C08 | focused and affected broad validation passes | nonzero counts and static checks | complete — E03 |
| P13-C09 | canonical/compatibility and later seams are current | contract and consumer audit | complete — E03 |
| P13-C10 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete — Complete / High |

## Stop Conditions

Stop and do not activate Phase 14 if current publication already validates all equipment edges/matrix; CRA-POL-002
conflicts with current runtime or official content; a valid live row cannot pass without an unapproved semantics/
RON/public migration; the error/atomicity boundary belongs to an independent source-of-truth not in this Candidate;
runtime defensive checks would have to be removed; dirty overlap is inseparable; a focused runner selects zero
tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
