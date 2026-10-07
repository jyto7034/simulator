# Phase 15 Goal — CRA-P09-C01 Close Projectile Publication And Basic-Delivery Compatibility

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P09-C01
Master Phase: 15 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 14's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make projectile scalar validity and delivery/consumer compatibility one strict atomic publication contract for RON
and programmatic construction. Projectile speed and every present optional count/time/limit must be positive. Skill
definitions retain valid TargetLocked and DirectionalCollision behavior, while basic attacks and weapon profiles
accept only target-locked delivery fields their runtime actually executes and reject skill-only policies rather than
silently ignoring them.

This Phase is complete only when the current RON parse→local validation→cross-catalog validation→official/programmatic
publication and skill/basic/weapon runtime graph is identified; zero speed/limits and ignored basic/weapon fields
first fail through otherwise-valid real entry paths with exact object/path/cause; one validation authority covers
RON and programmatic candidates before publication; no partial startup, normalization, implicit instant fallback or
permissive compatibility branch remains; all current live data and positive projectile ceiling behavior stay exact;
public/RON/save/wire shapes remain compatible; focused/broad validation passes; and the independent phase-exit audit
returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audit and reaudit evidence found that skill publication validates collision/policy combinations without
rejecting zero `speed_units_per_ms` or zero optional scalar policies constructed programmatically. RON serde rejects
some optional zeros, so RON and programmatic publication disagree. Basic definitions and weapon profiles reject
TileArea/DirectionalCollision in part but accept target-locked projectile fields—targetless, range/lifetime,
kill/pierce and nondefault collision policies—that basic runtime silently ignores. Zero speed currently reaches a
defensive zero-flight fallback rather than failing publication.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 15 registry row.
- Direct linear predecessor: Phase 14 CRA-P09-C02, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_14_p09_c02/COMPLETION_AUDIT.md`.
- Candidate entry dependency: Phase 01 CRA-P09-C08 repaired the false-positive negative fixtures so expanded strict
  rejection must assert the intended exact cause.
- Approved direct policy: CRA-POL-003, strict projectile/basic publication.
- Cross-cutting compatibility policy: CRA-POL-006.
- Downstream seam: Phase 29 CRA-P09-C05 may extract shared projectile motion only after P09-C01 publication,
  P09-C02 terminal delivery and P08-C05 committed-hit boundaries are frozen.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_09_battle_targeting_and_delivery/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_09_battle_targeting_delivery_and_projectiles/BOUNDARY_AUDIT.md`
- Approved decision and final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No candidate-specific decision is open: CRA-POL-003 explicitly approves rejecting formerly constructible invalid
programmatic/basic values. A public/schema migration or compatibility interval remains unapproved under CRA-POL-006.

## Priority And Current Disposition

- Priority axes: `2 / 3 / 2 / 3 / 3 / 2 / unknown`; known subtotal `15/18`.
- Final disposition at activation: `still_valid`, compatibility-significant correctness correction, test first.
- Current live-tree disposition: implemented and broadly validated. Exact RON/programmatic scalar and compatibility
  reds are green through the shared semantic authority; current live data, runtime, release and server validation
  pass. Independent completion review remains pending.

## Current And Target Authority

- Historical current authority: `DeliveryDef` serde, skill database validation, `BasicAttackDef`,
  `WeaponCombatProfile` and their consumers each enforce only part of projectile validity. Runtime retains defensive
  zero-flight and ignored-field behavior.
- Target authority: one typed publication validation contract accepts only positive projectile scalars and an
  explicit consumer/delivery compatibility matrix. RON and programmatic official candidates pass through the same
  semantic checks before publication; runtime guards remain integrity defense, not accepted-input fallback.
- Exact current type/function/file names, builder error behavior, validation order and live-data construction must be
  rediscovered after the Child read/dirty gate; historical names are evidence, not mutation instructions.

## In Scope

- inventory DeliveryDef parsing/defaults, positive scalar representation/validation, SkillDatabase construction,
  official live source parsing/cross-validation/publication and programmatic GameData builders;
- inventory BasicAttackDef and WeaponCombatProfile delivery validation plus every runtime-consumed versus ignored
  field for Instant, TargetLocked Projectile, DirectionalCollision and TileArea;
- capture expanded dirty status, scoped diffs and fingerprints for runtime/data/test/canonical targets;
- add otherwise-valid RON and programmatic zero-speed/zero-limit negative tables with exact object/path/cause;
- add exhaustive basic/weapon delivery compatibility tables: valid Instant/default TargetLocked positive projectile
  and invalid directional, area, targetless, range/lifetime, kill/pierce and nondefault collision policies;
- converge RON and programmatic semantic validation before candidate publication, preserving typed provenance and
  atomic whole-candidate failure;
- retain positive ceiling flight cases, valid skill TargetLocked/DirectionalCollision behavior, basic weapon
  override behavior and every current live bundle;
- update the nearest data-loading/projectile canonical only after the corrected publication contract is implemented.

## Out Of Scope

- changing authored positive projectile speed, range, lifetime, kill, pierce, collision or balance meaning;
- Phase 16 P09-C06 tile pattern/query consolidation, Phase 17/18 usefulness/preview corrections;
- Phase 28 committed-damage consolidation or Phase 29 shared projectile motion extraction;
- Phase 14 terminal delivery changes, collision geometry, damage/effects, target acquisition or released-work rules;
- public DeliveryDef/BasicAttackDef/WeaponCombatProfile/RON/save/checkpoint/DTO/wire shape migration;
- accepting invalid legacy values through normalization, instant fallback, alias, duplicate validator or temporary
  compatibility branch;
- unrelated loader refactor, formatting, data rewrite, cleanup or dirty-worktree normalization.

## Preserved Invariants

1. Every current official live RON source loads and publishes atomically.
2. Every valid positive projectile retains current ceiling flight time, target guidance, collision and effect order.
3. Valid skill TargetLocked and DirectionalCollision definitions retain exact behavior.
4. Valid basic and weapon Instant/default TargetLocked projectile definitions retain exact runtime behavior and
   override precedence.
5. RON and programmatic candidates receive the same semantic accept/reject decision and field-specific provenance.
6. Invalid candidate publication leaves no partial database/startup state and performs no silent normalization.
7. Runtime defensive guards do not become publication fallback or a second accepted contract.
8. Current public Rust, RON, V3 checkpoint, DTO/wire/server/Unity shapes remain exact under CRA-POL-006.
9. Phase 16/17/18/28/29 authority, Phase 14 terminal delivery and all unrelated content remain untouched.
10. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After all three Child documents exist and Master records sole activation, read them completely and reread the full
Phase 14 completion audit. Capture expanded status and prospective target diff/hash before inspecting production
source, tests or live data. Treat every existing hunk as user-owned until reconciled and edit only semantic
publication gates, direct negative/positive proofs and the nearest canonical.

Use test-first proof in this order:

1. Trace RON and programmatic parse/validation/publication plus skill/basic/weapon consumption fields.
2. Run nonzero current skill validation, basic/weapon delivery, live RON and positive flight baselines.
3. Add otherwise-valid zero scalar and compatibility matrices; prove the intended current acceptances red with exact
   error expectations rather than catch-any-panic.
4. Introduce the smallest shared semantic validation authority before publication, preserving existing public types.
5. Re-run exact reds and complete skill/basic/weapon/live publication suites.
6. Run full Core, affected server, release and static/dirty/compatibility checks.

## Migration, Deployment And Compatibility

- CRA-POL-003 approves a strict cutover for invalid values; no compatibility interval or normalization is allowed.
- No RON field/tag/default, public Rust type, V3 save/checkpoint, DTO/wire or API version migration.
- Current valid live content requires no rewrite. Invalid RON and programmatic candidates fail before publication with
  exact provenance; there is no partial startup or last-known-good concern in the startup-only loader.
- Core/game_server validate from one repository state. Unity build/runtime and live WebSocket probes are omitted
  unless current inspection finds a serialized/message-order change, which triggers a policy stop.

## Validation Plan

- Baseline: skill-data validation, delivery serde/defaults, basic/weapon validation, projectile flight boundaries,
  complete live RON and live skill/basic behavior with nonzero counts.
- First proof: RON/programmatic zero speed and every optional zero limit; full basic/weapon ignored-field table with
  exact typed object/path/cause; repaired otherwise-valid fixtures.
- Focused final: valid/invalid decision table, atomic whole-candidate failure, positive ceiling flight, valid skill
  directional/target-locked behavior and valid basic weapon override behavior.
- Broad: complete affected Core integration families, Core lib, game_server targets, live RON and release.
- Static: producer/caller/validator inventory, runtime fallback disposition, stale duplicate/ignored-field searches,
  public/serde/save/wire comparison, scoped status/diff/hashes, canonical comparison, formatter and diff check.
- Zero-test filters, helper-only predicates, catch-any-panic and compile-only success do not count as publication proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P15-C01 | complete RON/programmatic/publication/consumer graph identified | parse→semantic gate→atomic publication→runtime inventory | complete |
| P15-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P15-C03 | zero projectile scalars reject identically | exact otherwise-valid RON/programmatic negative matrix | complete |
| P15-C04 | basic/weapon compatibility matrix is strict and complete | valid defaults plus every ignored-field exact rejection | complete |
| P15-C05 | invalid candidates fail before atomic publication | whole-candidate no-partial-state/provenance proofs | complete |
| P15-C06 | valid skill/basic projectile behavior remains exact | positive flight/directional/target-locked/override matrix | complete |
| P15-C07 | focused and affected broad validation passes | nonzero Core/live/server/release/static counts | complete |
| P15-C08 | canonical/compatibility and Phase 29 seam are current | data/contract/consumer/legacy audit | complete |
| P15-C09 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete — Complete / High |

## Stop Conditions

Stop and do not activate Phase 16 if the current tree already enforces the complete approved matrix; an intended
negative fixture fails for an earlier unrelated cause; rejecting a live valid definition would require content or
balance migration; exact basic/weapon executed-field meaning is ambiguous; a public/RON/save/wire shape or
compatibility interval becomes necessary; atomic publication cannot be preserved in scope; dirty overlap is
inseparable; a required runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
