# Phase 11 Goal — CRA-P08-C01 Dispatch Every Item Trigger Occurrence Through One Typed Path

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P08-C01
Master Phase: 11 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 10's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make BattleCore own one typed runtime occurrence dispatcher for item OnBattleStart, OnAttack, OnHit, OnKill,
OnDeath and OnAllyDeath effects and ability activations. It must collect runtime items in deterministic order,
materialize ordinary effects before activations, preserve current-hit inputs explicitly, and carry complete
counterpart/dead-participant context without occurrence-specific omissions.

This Phase is complete only when every current occurrence collector/materializer/caller is identified; a live
two-ally `parasite_seed` test first proves the missing OnBattleStart +6% AllAllies effect; same-occurrence tests prove
effects-before-activations; attack/hit/kill/death/ally-death matrices protect targets, ordering, proc identity and
dead-caster rules; CRA-POL-002 tests distinguish one non-recursive OnHit Counterpart BonusDamage retaliation from
current-hit amplification; one common typed path replaces the split assembly without absorbing damage formula or
ability delivery; current live data and public/save/wire shapes remain compatible; focused/broad validation passes;
and the phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found a live omitted effect: `run_on_battle_start_hooks` collected only ability activations, so
the published `parasite_seed` OnBattleStart AllAllies Attack +6% never executed even though activation-only live
tests passed. Instant basic hits, projectile impacts and death finalization separately assembled overlapping effect/
activation occurrences, leaving target, ordering and dead-participant rules vulnerable to drift. The live
`thorned_lantern_guard` OnHit Counterpart BonusDamage was also interpreted as incoming-hit amplification despite the
approved target meaning.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 11 registry row.
- Direct predecessor: Phase 10 CRA-P08-C03, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_10_p08_c03/COMPLETION_AUDIT.md`.
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

CRA-POL-002 requires OnHit `CounterpartUnit + BonusDamage` to become one non-recursive retaliation against the
attacker, attributed to the effect owner, using the triggering committed-hit snapshot and existing typed damage.
CRA-POL-006 preserves all unrelated public/RON/save/checkpoint/wire contracts. This Phase establishes the occurrence
contract required by later P08-C04 publication validation, P08-C05 committed-damage consolidation and P08-C06 false-
death cleanup.

## Priority And Current Disposition

- Priority axes: `3 / 3 / 3 / 3 / 3 / 2 / unknown`; known subtotal `17/18`.
- Final disposition at activation: `still_valid`, correctness correction, live parasite test first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historical current authority: sorted runtime item lookup exists, but battle start, instant/projectile basic hits
  and death paths decide separately which effects/activations run and how commands are ordered.
- Target authority: one typed BattleCore occurrence dispatcher takes trigger, source/owner, counterpart, target/dead-
  participant and current-hit context as required, then emits one deterministic plan/command sequence. Current-hit
  modifiers remain explicit inputs; ordinary effect commands precede activation commands.
- Current names, occurrence carriers, call graph and exact committed-damage entrypoints must be rediscovered after the
  required document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify all runtime item lookup, trigger collection, proc roll/cooldown/count, effect targeting, activation and
  command materialization paths for the six occurrence classes;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical/live-data target;
- add the live two-ally parasite-seed exact StatChanged/final-Attack case with deterministic ally order and no enemy
  application;
- add a same-occurrence effect+activation proof that asserts event/command-visible effects-before-activations;
- add/strengthen a table over OnAttack, OnHit, OnKill, OnDeath and OnAllyDeath ordinary effects plus activations,
  counterpart/all-side targets, dead source/counterpart handling and sorted chained deaths;
- implement CRA-POL-002 retaliation once per authored effect, with effect-owner attribution, missing/dead participant
  handling and no OnAttack/OnHit recursive collection;
- introduce/reuse the smallest common typed dispatcher/plan while leaving formula and delivery owners intact;
- update nearest canonical only after runtime behavior and tests are current.

## Out Of Scope

- implementing Phase 12 MaxHealth-caused HP/death/provenance/kill-credit transitions;
- implementing Phase 13 equipment reference/compatibility publication validation, except consuming the approved
  runtime meaning already required here;
- consolidating instant/projectile/general committed damage under P08-C05;
- changing target selection, tile footprints, projectile collision/motion, ability delivery or skill usefulness;
- making ordinary skill/buff/area damage enter item OnAttack/OnHit occurrence collection;
- changing proc chance/cooldown/max-count formula, item order, RNG identity, damage formula or balance values;
- changing RON schema/data IDs, public types, save/checkpoint, DTO/wire tags/nesting or server/Unity contract;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. Runtime item and unit traversal order, proc roll identity, cooldown and max-trigger counts remain deterministic.
2. Permanent effects remain battle-entry-only and do not re-enter runtime occurrence dispatch.
3. OnAttack/OnHit committed-hit timing and explicit current-hit BonusDamage/ModifyDamage inputs remain exact except
   the approved Counterpart BonusDamage retaliation correction.
4. Every ordinary occurrence materializes effect commands before activation commands.
5. OnKill/OnDeath/OnAllyDeath target sets, sorted ally order and dead-caster activation permission remain exact.
6. Released effects/projectiles survive later control/death according to their existing leaf lifecycle.
7. Retaliation is attributed to the effect owner, applies at most once per authored effect and never recursively
   collects OnAttack/OnHit procs.
8. Damage calculation and application, ability delivery, HP/death finalization and event cause ownership stay with
   their current domains.
9. Current RON, public Rust, save/checkpoint, DTO/wire/server/Unity shape remains exact under CRA-POL-006.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded dirty status and prospective target status/diff/hash before inspecting current source/tests/data/canonical.
Treat all existing changes as user-owned and edit only the common occurrence authority, scoped callers, exact tests
and nearest canonical.

Use test-first proof in this order:

1. Run nonzero trigger collection, command/death, live item, live RON and directly affected Core baselines.
2. Trace every occurrence from trigger entry through sorted items, target resolution, proc state, commands and logs.
3. Add live parasite and same-occurrence ordering cases; establish the battle-start omission red result.
4. Add CRA-POL-002 retaliation/no-recursion and remaining occurrence/dead-participant decision tables.
5. Extract one typed occurrence plan/dispatcher and route all six occurrence classes through it.
6. Re-run exact failures, all occurrence/proc/death/live tests, complete Core and server/static checks.

## Migration, Deployment And Compatibility

- No schema, save/checkpoint, DTO/wire, API version or data-ID migration.
- Live parasite behavior begins matching its already-published definition. Live thorned-lantern Counterpart
  BonusDamage changes to the explicitly approved retaliation meaning; no dual behavior or fallback remains.
- Command/result/event carrier shapes stay unchanged, so server/Unity require no transport rollout.
- If a correct dispatcher requires new target/delivery policy, public breaking carrier or committed-damage redesign,
  stop for user direction rather than expanding this Phase.

## Validation Plan

- Baseline: trigger collection, committed basic instant/projectile commands, death runtime, proc limits, live item,
  live RON and selected world battle families with nonzero counts.
- First proof: live parasite two-ally exact final/event order; same-occurrence effect+activation order; retaliation
  versus amplification and non-recursion.
- Focused final: all six occurrences, target modes, missing/dead counterpart/source, multiple effects, sorted ally
  deaths, proc identity/count/cooldown, ordinary effects-before-activations and exact event causes.
- Broad: damage/death/buff/control/skill/live integrations, complete Core and game_server all targets.
- Static: complete caller/variant matrix, duplicate occurrence assembly and obsolete wrapper search, public/data shape
  audit, scoped status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P11-C01 | complete six-occurrence authority/caller graph identified | typed entry→plan→commands/log inventory | complete — E01/E03 |
| P11-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete — E01/E03 |
| P11-C03 | live battle-start omission and ordering reproduced | parasite and effect+activation red proofs | complete — E02/E03 |
| P11-C04 | CRA-POL-002 retaliation/no-recursion is exact | decision-table red/green proof | complete — E02/E03 |
| P11-C05 | one typed dispatcher owns all scoped occurrence assembly | caller/variant/removal inventory | complete — E03 |
| P11-C06 | target/dead/proc/order/lifecycle invariants remain exact | focused matrix | complete — E03 |
| P11-C07 | focused and affected broad validation passes | nonzero counts and static checks | complete — E03 |
| P11-C08 | canonical/compatibility and later seams are current | contract and consumer audit | complete — E03 |
| P11-C09 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete — `Complete / High` |

## Stop Conditions

Stop and do not activate Phase 12 if current code already dispatches all six classes through one complete typed path;
retaliation cannot be delivered without prematurely implementing P08-C05; dead-participant/target behavior conflicts
with CRA-POL-002 or existing canonical; a new gameplay/delivery policy or public/data/save shape is required; dirty
overlap is inseparable; a focused runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
