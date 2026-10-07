# Phase 28 Goal — CRA-P08-C05 One Committed-Damage Authority

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P08-C05
Master Phase: 28 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — limited to Phase 28 after the Master sole-active transition, complete
required-document reread and prospective fingerprint capture; no Phase 29 work is authorized
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate the overlapping instant-basic, projectile-impact and general `ApplyDamage` committed-damage orchestration
into one private typed authority around the retained pure `calculate_damage` formula. The shared authority must
materialize the correct source/target snapshot and explicit trigger policy, calculate once, apply/log HP and normal
death once, update actual-delta resonance where applicable and return the existing execution outcome without
changing observable behavior.

Preserve launch-source versus impact-time live-target semantics, deterministic per-target/hit roll identity,
basic-only OnAttack/OnHit collection, effect-before-activation command order, CRA-POL-002 non-recursive retaliation,
CRA-POL-001 provenance/death/kill credit, same-time death behavior, event lineage and all current public/serialized
contracts. Inventory every request/context/result/command carrier before removing a field or variant. Under
CRA-POL-006, a negative workspace search cannot authorize public Rust narrowing; private consolidation may remove
only authority proven duplicate or semantically unread.

This Phase is complete only when the current damage producer/formula/application/death/resonance/trigger/caller graph
is rediscovered; old behavior is characterized first across instant, projectile and general delivery with stat,
source, target, crit, retaliation, kill, lineage and ordering drift; one typed committed-damage authority replaces
the duplicate orchestration without absorbing targeting, projectile motion/collision, skill-area lifecycle or the
disconnected public DeathHandler cleanup; every retained/removable carrier has a consumer disposition; focused and
broad Core/server debug/release validation passes; no Phase 29 code is prebuilt; and an independent review returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical audits found one pure damage formula but three overlapping execution authorities around it. Instant basic
damage partially uses a helper, projectile impact reconstructs occurrence effects/activations, formula inputs,
resonance and command ordering, and general `ApplyDamage` reconstructs another context/application path. Formula
output also calculates a `BattleCommand::UnitDied` signal that production executors ignore while the real HP/death
authority finalizes from committed state.

The user-visible result must not change. Identical authoritative inputs must produce identical damage, HP/death,
resonance, trigger, retaliation, event-cause and result behavior regardless of delivery carrier. Consolidation must
remove synchronization risk without turning preview into mutation, applying basic triggers to general skills,
recomputing launch snapshots at impact or creating a second death authority.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 28 registry row.
- Direct linear predecessor: Phase 27 CRA-P05-C03, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_27_p05_c03/COMPLETION_AUDIT.md`.
- Required semantic predecessor: Phase 11 CRA-P08-C01, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_11_p08_c01/COMPLETION_AUDIT.md`; one typed
  occurrence dispatcher and CRA-POL-002 retaliation handoff are current authority.
- Required HP/death predecessor: Phase 12 CRA-P08-C02, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_12_p08_c02/COMPLETION_AUDIT.md`; provenance,
  HP logging, normal death finalization and CRA-POL-001 kill credit are current authority.
- Required publication predecessor: Phase 13 CRA-P08-C04, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_13_p08_c04/COMPLETION_AUDIT.md`; the
  executable occurrence/effect matrix is fixed before this consumer consolidation.
- Stabilized numeric prerequisite: Phase 10 CRA-P08-C03 is complete; range/cap/rounding behavior is not reopened.
- Downstream seams: Phase 29 CRA-P09-C05 requires this committed-hit boundary before motion consolidation; Phase 38
  CRA-P08-C06 requires final production damage/death assertions before false DeathHandler removal.
- Historical/final evidence:
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/{CANDIDATE_CATALOG,TEST_AUDIT,BOUNDARY_AUDIT}.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/{CANDIDATE_REAUDIT,TEST_REAUDIT,BOUNDARY_AUDIT}.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/{REBASELINED_CANDIDATE_CATALOG,EXACT_LINEAR_IMPLEMENTATION_ORDER,TEST_REBASELINE_CATALOG,IMPLEMENTATION_WAVES_AND_GATES,APPROVED_POLICY_DECISIONS}.md`

## Approved Policy And Current Disposition

- CRA-POL-001: any lethal HP transition, including causal MaxHealth clamp, uses normal death finalization; causal
  source receives ordinary kill credit and source-less/system causes do not.
- CRA-POL-002: OnHit Counterpart BonusDamage is one owner-attributed, non-recursive retaliation against the attacker;
  it is not incoming-hit amplification and does not re-enter OnAttack/OnHit collection.
- CRA-POL-006: current public Rust, RON, save, DTO, wire and package compatibility is preserved. Unknown deployed
  Rust consumers prohibit public carrier/field/variant removal on workspace-only evidence.
- Priority axes: `3 / 1 / 3 / 3 / 2 / 2 / unknown`; known subtotal `14/18`.
- Final readiness: Tier B after P08-C01/C02 characterization and P08-C04 matrix stabilization.
- Reaudited disposition: `still_valid`; Boss runtime leaves released projectile/damage/effect/death work under these
  ordinary leaf authorities and introduces no competing Boss damage formula.
- Open policy decisions: none. A newly required trigger, death-credit, public-removal, delivery or wire meaning stops
  this Phase; incidental consolidation cannot supersede CRA-POL-001/002/006.

## Historical Current And Target Authority

- Historical formula authority: `calculate_damage` owns pure typed arithmetic, modifier/crit/mitigation/minimum and
  breakdown calculation and is retained.
- Historical orchestration: instant basic, projectile impact and general `ApplyDamage` separately assemble portions
  of source/target context, occurrence inputs, committed HP/death and result/command handling.
- Historical death signal: formula-derived `BattleCommand::UnitDied` is ignored while committed HP reaching zero
  drives the real death authority.
- Target private authority: one typed committed-damage input/plan/execution summary explicitly carries delivery and
  trigger policy, surrounds the retained formula and commits through the existing HP/death authority exactly once.
- Preserved owners: occurrence classification/dispatch stays Phase 11/13-owned; HP/death stays normal Core authority;
  projectile motion/collision remains P09-owned; targeting/area lifecycle remains its existing delivery owner.

These are historical pointers only. Current production after activation is authoritative and may refine exact names,
visibility and whether a reported carrier remains ghost state.

## In Scope

- every production caller that calculates or commits damage: instant basic, basic projectile, skill instant/
  projectile/spatial/persistent-area, buff/trigger/retaliation and any other current `ApplyDamage` consumer;
- source snapshot lifetime, live target defense/body/lifecycle lookup and deterministic roll identity;
- explicit basic-versus-general occurrence policy and released command ordering;
- pure formula input/output, committed HP delta, normal death, kill summaries, graveyard/cleanup and exact event cause;
- actual-delta basic resonance gain and all conditions under which it is absent;
- CRA-POL-002 retaliation application and proof of no reflection recursion;
- `BattleCommand::UnitDied`, damage request/context/result and execution-summary field read/write/caller inventory;
- public Rust, serde, save/checkpoint, server and accessible/canonical external consumer classification;
- characterization-first focused tests, debug/release broad Core/server validation and static authority/removal audit.

## Out Of Scope

- changing damage formula arithmetic, balance values, crit/mitigation/minimum/rounding or typed breakdown meaning;
- changing occurrence/effect publication compatibility, target semantics or retaliation policy;
- changing MaxHealth stat-transition policy, HP/death/kill-credit semantics or death trigger order;
- merging targeting/acquisition, projectile movement/collision/terminal lifecycle or skill footprint/area scheduling;
- deleting/depublishing the disconnected `battle::death` model owned by Phase 38 P08-C06;
- public Rust carrier/field/variant narrowing without CRA-POL-006 consumer/migration evidence;
- changing DTO/event tags, serialization, V3/checkpoint, live RON, server/Unity shape or message order;
- adding fallback, dual damage paths, compatibility aliases, silent normalization or test-only production branches;
- Phase 29 implementation or future motion-kernel placeholders.

## Required Test-First Characterization Matrix

| Case | Delivery parity | Required drift/ordering proof | Preserved exclusions |
| --- | --- | --- | --- |
| instant vs projectile basic | same authoritative hit inputs produce equal breakdown, HP delta, resonance and logs | source stat changes/death after launch; target defense changes before impact; stable per-hit roll/crit identity | projectile timing/collision and launch snapshot remain distinct |
| general `ApplyDamage` | same component/formula/application result as equivalent committed leaf | source/target missing or dead, exact source lineage, HP/death/kill summary and command order | no implicit basic OnAttack/OnHit collection or resonance |
| current-hit effects | ModifyDamage/BonusDamage and effect-before-activation order are exact | instant/projectile parity and stable item/source ordering | Phase 11/13 classifier remains authority |
| retaliation | one owner-attributed component damages attacker after triggering hit | missing/dead/fatal participants, multiple authored effects, exact log/death/kill order | no reflection recursion or incoming amplification |
| death boundary | lethal/nonlethal/same-time/nested damage finalizes once | one HpChanged, UnitDied, cleanup, triggers, graveyard, lineage and result count | no formula-derived second death command authority |
| pure preview/formula | read-only formula and usefulness callers remain no-mutation and exact | formula goldens and current public request/result behavior | committed state helper is not used as preview authority |
| carriers/consumers | every removable/retained field and variant has a reader/consumer disposition | workspace, server, test, canonical external and unknown deployment inventory | public narrowing requires separate approval/migration |

Tests protect observable state/events/results and delivery lifetime, not a future private helper name. Existing tests
count only after their setup/assertions/current reachability are independently checked. New parity tests must pass on
old production before consolidation unless they directly reproduce an approved correctness defect outside this
behavior-preserving Phase, in which case the Phase stops for scope review.

## Implementation Constraints

1. Capture current expanded status, HEAD and prospective source/test/data/canonical hashes before production-detail
   inspection.
2. Inventory formula, context, request, command, commit, resonance, death and result callers before choosing a seam.
3. Classify each delivery path by source snapshot lifetime, target lookup time, trigger policy and roll identity.
4. Run nonzero old-production focused baselines and add the complete parity matrix before production refactoring.
5. Retain `calculate_damage` as pure formula authority and existing HP/death finalization as mutation authority.
6. Use an explicit typed policy/input rather than booleans or inference from source kind when paths differ.
7. Preserve current command/event order and released-work semantics; do not recompute launch state at impact.
8. Remove only proven duplicate private orchestration and semantically unread private fields.
9. Retain public carriers when deployed consumers remain unknown under CRA-POL-006; do not add replacement aliases.
10. Stop for independent audit before Phase 29 activation.

## DTO, Data, Save And Deployment

- DTO/serde migration: none authorized; exact battle log/result/setup/update/resync tags and fields remain.
- Live RON migration: none; Equipment/Buff/Skill/encounter definitions and values remain unchanged.
- Save/checkpoint migration: none; committed battle runtime remains under existing private/current persistence rules.
- Public Rust deployment: inventory all damage carriers, but retain public surfaces absent consumer/migration proof.
- Server/Unity deployment: no wire change, version cutover, dual read/write or fallback is authorized.
- Boss deployment: private Boss runtime continues to release ordinary leaf work; no Boss-specific public damage path.

## Validation Strategy

- Baseline: damage formula, Core damage runtime/commands/death/triggers/resonance, basic instant/projectile, skill
  instant/projectile/spatial/area, live item/skill and Boss released-projectile families with nonzero counts.
- Test-first: execute the complete matrix above on the pre-refactor production implementation.
- Focused implementation: rerun every formula/producer/consumer/death/trigger/resonance/delivery family.
- Broad Core: complete debug/release unit and official integration tests plus debug/release all-target checks.
- Broad server: complete debug/release tests/checks because public event/result carriers compile through server.
- Static: one committed-damage authority, retained one pure formula/one death owner, no duplicate application,
  ignored `UnitDied` or unread-private carrier left within approved scope, no public narrowing/fallback/Phase 29 seam.
- Compatibility: live RON and public/serde/V3/server/Unity canonical hashes/source reads.
- Diagnostic: non-strict Clippy may inventory established debt; strict warning denial is not claimed unless globally
  green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P28-C01 | complete formula/commit/trigger/resonance/death/caller graph identified | all delivery classes, inputs, outputs and consumers | verified — E03/E06 |
| P28-C02 | dirty targets and concurrent work preserved | pre/post normalized status, scoped diffs, hashes and marker preservation | verified — E02/E05/E07 |
| P28-C03 | old behavior characterized first | nonzero instant/projectile/general drift/crit/retaliation/kill/order matrix | verified — E04/E05 |
| P28-C04 | one private committed-damage authority surrounds retained formula | source/static ownership and negative duplicate search | verified — E06 |
| P28-C05 | source/target/roll/delivery lifetime parity exact | launch/live drift and Boss released-work tests | verified — E05/E07 |
| P28-C06 | occurrence, retaliation and command order exact | CRA-POL-002/current-hit tests and no-recursion proof | verified — E03/E05/E07 |
| P28-C07 | HP/death/resonance/lineage exact | CRA-POL-001, actual-delta, same-time/nested death and result tests | verified — E05/E07 |
| P28-C08 | carrier removal/retention and public compatibility closed | complete reader/consumer inventory; no unauthorized narrowing | verified — E03/E06 |
| P28-C09 | DTO/RON/save/server/Unity compatibility exact | type/schema/data/consumer/canonical evidence | verified — E02/E03/E07 |
| P28-C10 | focused and broad debug/release/static validation green | actual nonzero counts and baseline/new failure distinction | verified — E04/E07 |
| P28-C11 | downstream boundary and independent audit current | no Phase 29 prebuild and `Complete / High` review | pass — 320-line `COMPLETION_AUDIT.md` |

## Activation Protocol

Before production inspection:

1. Master records Phase 27 `complete — Complete / High`, Phase 28 as the sole `active` Child and Phases 29~44 pending.
2. Change all three Child headers from `initialized` to `active` and authorize only Phase 28.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, final Phase 27 audit and Phase 11/12/13 dependency
   audits.
4. Capture HEAD, expanded status count/normalized hash and prospective hashes for likely source/test/data/canonical
   targets using path discovery only, without reading their content.
5. Record that evidence in the Child before examining production details.

No Phase 28 source/test/data/canonical content inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 29 if evidence requires changing damage arithmetic, crit/mitigation/minimum meaning,
source snapshot or target lookup lifetime, OnAttack/OnHit applicability, retaliation/death/kill-credit policy,
resonance semantics, target/projectile/area lifecycle, public Rust symbols/fields/variants, serialized DTO/V3/RON/
wire shape, Boss released-work behavior or deployment assumptions; adding fallback/dual paths/aliases; absorbing
P09-C05 or P08-C06; prebuilding Phase 29; user-owned dirty overlap cannot be separated; required tests select zero;
or independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
