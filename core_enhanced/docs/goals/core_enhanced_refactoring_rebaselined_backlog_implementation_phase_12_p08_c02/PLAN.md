# Phase 12 Goal — CRA-P08-C02 Make Runtime Stat Mutation HP- And Death-Aware

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P08-C02
Master Phase: 12 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 11's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make runtime stat mutation preserve causal provenance and bridge MaxHealth-caused current-HP changes into the one
authoritative HP/death lifecycle. A nonlethal clamp emits one causally attributed HP change; a clamp to zero enters
normal idempotent death finalization with the approved killer/kill-credit rule; no active zero-HP terminal state is
possible.

This Phase is complete only when every runtime modifier producer, command carrier, HP recorder, death/kill consumer
and prebattle stat path is identified; flat/percent, sourced/unsourced and nonlethal/lethal decision tables first
prove the current gaps; one typed runtime transition owns before/after stats, dependent body updates, StatChanged,
conditional HpChanged and exactly-once death handoff; observable order is StatChanged then HpChanged then existing
death sequence; ordinary Attack/movement/positive-MaxHealth behavior and Phase 10 arithmetic remain exact; Phase 11
OnDeath/OnKill/OnAllyDeath dispatch receives the causal killer without duplicate/post-death mutation; public/RON/
save/wire shapes remain compatible; focused/broad validation passes; and the phase-exit audit returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical and re-audit evidence found that a runtime negative MaxHealth modifier can clamp current HP inside
`UnitStats` while the runtime command records only a source-less StatChanged event. The HP authority is not notified,
so no HpChanged/death cleanup/kill trigger runs and an Active zero-HP unit can remain. The command conversion also
discards the modifier's causal unit, making the approved kill-credit rule impossible to apply.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 12 registry row.
- Direct predecessor: Phase 11 CRA-P08-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_11_p08_c01/COMPLETION_AUDIT.md`.
- Numeric prerequisite: Phase 10 CRA-P08-C03 fixed signed/percentage/cap arithmetic before HP-aware transitions.
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

CRA-POL-001 requires a lethal runtime MaxHealth clamp to use normal death finalization, preserve causal
`source_instance_id`, assign an existing causal unit as killer, leave system/unsourced death without killer, and
commit observable order as StatChanged→HpChanged→death. CRA-POL-006 preserves unrelated public/RON/save/wire shape.
This Phase supplies final HP/death/provenance semantics required by later P08-C05 committed-damage consolidation.

## Priority And Current Disposition

- Priority axes: `2 / 3 / 3 / 2 / 3 / 3 / unknown`; known subtotal `16/18`.
- Final disposition at activation: `still_valid`, correctness correction, approved decision-table tests first.
- Current live-tree disposition: discovery pending until Child required-document read and scoped dirty inspection.

## Current And Target Authority

- Historical current authority: runtime modifier application mutates stats directly, separately updates movement and
  writes source-less StatChanged; damage/HP code alone owns HpChanged and death finalization.
- Target authority: one typed runtime stat-transition path carries optional causal source, computes exact before/after
  once, commits dependent body state, emits StatChanged and only an actual HP delta, then hands lethal zero to the
  existing death authority exactly once.
- Persistent battle-entry composition stays event-free and separate. Existing HP/death authority is reused rather
  than duplicated. Current names, public visibility, producer inventory and exact event carrier must be rediscovered
  after the required document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify every runtime modifier producer/converter/caller, causal source already available at each entry, public or
  private command consumer, UnitStats clamp, body-speed dependent update, HP recorder and death/kill trigger path;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical/live-data target;
- add flat and percent MaxHealth reductions above and at lethal boundary, each with sourced and unsourced causes;
- assert exact before/after max/current HP, StatChanged/HpChanged/source/order, active/dead lifecycle, graveyard,
  movement/action cleanup, buff cleanup, UnitDied and kill/ally/death occurrence counts;
- add a sourced lethal case proving ordinary OnKill credit and an unsourced lethal case proving no killer/OnKill;
- add nested death-trigger damage/heal protection against duplicate death or post-death target mutation;
- preserve non-MaxHealth modifiers, positive MaxHealth no-auto-heal, no-HP-delta event suppression, movement speed
  update, Phase 10 saturation/cap/rounding and persistent prebattle composition;
- introduce or reuse the smallest internal typed transition/provenance carrier that satisfies CRA-POL-001 without
  changing frozen public/save/wire contracts;
- update the nearest canonical only after runtime behavior and tests are current.

## Out Of Scope

- implementing Phase 13 equipment reference/matrix publication validation;
- consolidating instant/projectile/general committed damage under P08-C05;
- removing the disconnected public DeathHandler under P08-C06;
- changing UnitStats arithmetic, caps, percent rounding or battle-entry scaling fixed by Phase 10;
- changing damage formula, retaliation semantics, trigger occurrence order, ability delivery or skill targeting;
- changing RON schema/data IDs, public Rust construction/match shape, save/checkpoint, DTO/wire tags/nesting or
  server/Unity contract without a new explicit compatibility decision;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. UnitStats modifier order, saturation, MaxHealth cap and flat/percent rounding remain Phase 10 exact.
2. Positive MaxHealth changes do not automatically heal; current HP never exceeds max HP.
3. Non-MaxHealth modifiers do not emit HpChanged; movement speed updates immediately when affected.
4. One runtime mutation emits exactly one StatChanged and at most one HpChanged only when current HP changes.
5. Lethal order is StatChanged→HpChanged→existing death sequence; nonlethal order stops after the applicable events.
6. An existing causal source unit is killer; system/unsourced mutation has no killer and grants no kill credit.
7. Death finalization, cleanup and Phase 11 OnDeath/OnKill/OnAllyDeath occurrences execute at most once.
8. Persistent prebattle stat composition remains event-free and does not enter runtime death semantics.
9. Current RON, public Rust, save/checkpoint, DTO/wire/server/Unity shape remains exact under CRA-POL-006.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture expanded
dirty status and prospective target status/diff/hash before inspecting current source/tests/data/canonical. Treat all
existing changes as user-owned and edit only the runtime transition/provenance seam, exact tests and nearest
canonical.

Use test-first proof in this order:

1. Run nonzero runtime stat command, UnitStats, death cleanup/trigger and directly affected Core baselines.
2. Trace every modifier producer through command materialization and every HP/death/provenance consumer.
3. Add sourced/unsourced flat/percent nonlethal/lethal tables and establish exact red results.
4. Add kill-credit and nested-death no-duplicate proofs against the production Phase 11 occurrence path.
5. Implement one typed transition/handoff while preserving public compatibility and prebattle separation.
6. Re-run exact failures, ordinary modifier/death/trigger/live tests, complete Core and server/static checks.

## Migration, Deployment And Compatibility

- No RON, save/checkpoint, DTO/wire, API version or data-ID migration.
- Runtime behavior begins emitting the already-approved HP/death/provenance sequence; no dual behavior or fallback.
- If causal provenance cannot be carried without changing a public Rust construction/match contract, inventory all
  consumers and stop for direction rather than silently breaking CRA-POL-006 or introducing a hidden side table.
- Event/result carrier shapes remain unchanged. Server/Unity require no transport rollout.

## Validation Plan

- Baseline: runtime stat commands, UnitStats, production death cleanup, Phase 11 death occurrence, live item/RON and
  selected skill/effect families with nonzero counts.
- First proof: flat/percent sourced/unsourced MaxHealth decision table; lethal no-active-zero; exact event/source/order;
  killer versus no-killer.
- Focused final: ordinary Attack/movement/positive MaxHealth, nonlethal/lethal cleanup, nested death effects,
  exactly-once OnDeath/OnKill/OnAllyDeath, graveyard/buff/action state and source provenance.
- Broad: stat/damage/death/buff/control/skill/live integrations, complete Core and game_server all targets.
- Static: complete producer/consumer matrix, obsolete source-less runtime conversion search, public/data shape audit,
  scoped status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P12-C01 | complete runtime modifier/HP/death/provenance graph identified | typed producer→transition→events→death/kill inventory | complete — E01 |
| P12-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete — E01 |
| P12-C03 | current nonlethal/lethal gaps reproduced | sourced/unsourced flat/percent red proofs | complete — E02 |
| P12-C04 | CRA-POL-001 order and attribution are exact | decision-table red/green proof | complete — E02/E03 |
| P12-C05 | one runtime stat transition owns HP/death handoff | caller/removal/authority inventory | complete — E03 |
| P12-C06 | arithmetic/body/prebattle/death invariants remain exact | focused preservation matrix | complete — E03 |
| P12-C07 | focused and affected broad validation passes | nonzero counts and static checks | complete — E03 |
| P12-C08 | canonical/compatibility and later seams are current | contract and consumer audit | complete — E03 |
| P12-C09 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete — Complete / High |

## Stop Conditions

Stop and do not activate Phase 13 if current code already routes all runtime MaxHealth HP deltas through one complete
causal transition; CRA-POL-001 conflicts with current canonical; correct provenance requires an unapproved public/
save/wire break; normal death finalization cannot be reused without prematurely implementing P08-C05/C06; persistent
prebattle composition would be forced through runtime events; dirty overlap is inseparable; a focused runner selects
zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
