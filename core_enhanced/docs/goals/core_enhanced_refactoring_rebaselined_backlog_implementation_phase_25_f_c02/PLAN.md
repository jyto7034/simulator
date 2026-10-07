# Phase 25 Goal — CRA-F-C02 Common Headquarters Transaction And Explicit Completion Context

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-F-C02
Source Candidate IDs: CRA-P03-C05, CRA-P04-C02
Master Phase: 25 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after sole-active registration and completion of all mandatory activation reads/fingerprints
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Replace Headquarters' private handwritten rollback implementation with the existing common non-battle world
transaction, and replace the hidden active-content `take()`/restore completion bypass with an explicit resolved-HQ
completion context. Preserve all recruit, emergency-supplies, open-shop and common-node-completion behavior exactly.

This Phase is complete only when the complete current transaction/completion caller graph is rediscovered; tests
first compare the full non-battle state on every successful HQ branch and injected late failures, including queue,
candidate, roster, UUID, currency, inventory/grants, Shop session handoff, node/map/progression, checkpoint/research,
errors and monotonic encounter-compendium exclusion; the common transaction owns all HQ rollback; completion accepts
an explicit narrow HQ context without hidden mutation; the duplicate ten-field rollback and bypass are removed; no
active-battle, profile, public/DTO/save/RON/wire or branch policy changes occur; Phase 26 is not prebuilt; focused and
broad debug/release validation passes; and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audits found no current observed rollback divergence, but Headquarters duplicates world transaction
ownership with a manual ten-field snapshot/restore list. New non-battle root fields can silently escape that list.
HQ completion also temporarily removes active content so the common completion planner does not reject an active HQ
session, making valid completion context implicit in a mutation trick.

The intended result is behavior-preserving: players must receive the same candidates, queue order, recruited
employee/UUID, emergency resources, Shop handoff, completion result and errors. This Phase changes private atomicity
and context ownership, not Headquarters rules.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 25 registry row.
- Direct linear predecessor: Phase 24 CRA-P04-C03, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_24_p04_c03/COMPLETION_AUDIT.md`.
- Required prerequisites: Phase 23 CRA-P03-C04 freezes grant-ledger caller behavior; Phase 24 freezes HQ Shop
  persisted-active commits.
- Downstream dependency: Phase 26 CRA-P04-C04 may start only after this Phase is independently `Complete / High`.
- Original source evidence:
  - `../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel/CANDIDATE_CATALOG.md`
- Original merge evidence:
  - `../core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_final_catalog_consolidation/CROSS_BOUNDARY_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/CANDIDATE_REAUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No gameplay policy ID is assigned because this is private transaction/context parity. CRA-POL-006 remains the
compatibility default for public/admin/serialized/consumer behavior.

## Priority And Current Disposition

- Merged source identity: CRA-P03-C05 + CRA-P04-C02, one root cause and one target authority.
- Priority axes: `2 / 2 / 2 / 2 / 2 / 2 / unknown`; known subtotal `12/18`.
- Final tier/readiness: Tier B, full-state characterization first.
- Re-audited disposition: `still_valid`; BossOmen is nested in run state and does not remove the field-list risk or
  hidden completion-context bypass.
- Current discovery confirms `GameCoreState::clone_non_battle_transaction_state` captures all 19 current roots,
  rejects an active battle before running the operation, and deliberately substitutes an empty encounter-compendium
  snapshot so `with_state_transaction` can max/union-merge live profile knowledge after rollback. The former HQ
  helper copied only game state, active content, node session, run, UUID manager, Enkephalin, inventory, roster,
  skill fragments and checkpoint.

## Current And Target Authority

- Historical current rollback authority: a Headquarters-specific helper manually clones/restores ten roots while a
  common world transaction already owns complete non-battle rollback and monotonic compendium handling.
- Historical current completion context: HQ temporarily hides active content before calling the common completion
  planner, then restores it.
- Target rollback authority: the existing common non-battle transaction is the sole rollback owner for recruit,
  emergency and open-shop HQ branches; a thin HQ wrapper may delegate if it adds domain naming without state copying.
- Target completion context: a narrow explicit resolved-HQ context authorizes the valid HQ completion route without
  mutating active content merely to pass validation.
- HQ retains branch validation/staging. Common node completion retains map/progression/cleanup ownership. Encounter
  compendium remains an excluded monotonic profile aggregate and active battle remains outside non-battle rollback.

## In Scope

- all Headquarters branch entry, validation, staging, commit and late-failure paths;
- the Headquarters-specific transaction helper and every manually captured/restored root;
- the common non-battle transaction boundary and exact rollback/exclusion semantics;
- HQ completion planning's active-content admission and resolved-context representation;
- recruit success/failure: candidate identity/order, recruitment queue/session, roster/equipment/UUID/grant roots;
- emergency success/failure: currency/inventory/grant/completion state;
- open-shop success/failure: recruitment return, persisted-active Shop pair, transition/result and rollback;
- common completion: node session, map/progression, pending research, checkpoint and active content cleanup;
- full non-battle state comparison and encounter-compendium monotonic exclusion;
- focused/broad Core/server debug/release, formatter, diff, static and canonical compatibility checks.

## Out Of Scope

- changing HQ options, costs/rewards, candidate count/order, recruitment ownership or completion meaning;
- rolling back or otherwise changing encounter-compendium monotonic profile knowledge;
- including active battle state in the non-battle transaction or weakening active-battle rejection;
- merging Headquarters, Event, Shop, Reward or other active session types;
- changing Phase 24 Event/Shop paired-commit semantics or helper modes;
- changing checkpoint/save version, public session/behavior/DTO/V3/RON/wire shape;
- adding retry, idempotence, recovery, fallback, compatibility or migration policy;
- Phase 26 VN assembly/queue-placement consolidation;
- unrelated root-state, warning-debt, data, UI/server/Unity or transaction refactors.

## Required Test-First Characterization Matrix

| Family | Success evidence | Injected late-failure evidence | Exclusion/order evidence |
| --- | --- | --- | --- |
| Recruit employee | result, queue/session, roster/loadout, UUID and node completion exact | complete non-battle state unchanged after failure following staged branch work | encounter compendium preserves monotonic union; result/error order exact |
| Emergency supplies | exact Enkephalin/grant roots and common completion | late completion/research failure rolls back every non-battle root | no roster/Shop drift; compendium exclusion exact |
| Open HQ Shop | queue return, complete persisted-active Shop equality, transition/result exact | transition or later injected failure restores complete pre-state | Phase 24 Shop mode/order preserved; compendium exclusion exact |
| Common HQ completion context | valid resolved HQ completes without hidden active-content mutation | invalid/wrong/stale context rejects without mutation | active battle remains rejected; non-HQ node policy unchanged |
| Checkpoint/research/map | checkpoint replacement, pending research, node/map progression exact | invalid pending research or equivalent late defect has zero partial commit | ordering and original error variant exact |

Full-state assertions must compare every current `GameCoreState` field or a complete serializable/equivalent state
projection, then separately assert any deliberately excluded monotonic profile field. Tests must observe behavior and
state, not require a helper name or exact private context type.

## Implementation Constraints

1. Capture expanded status, HEAD and prospective hashes before inspecting Phase 25 production detail.
2. Inventory common/HQ transaction state coverage, exclusion semantics and every HQ completion caller.
3. Run relevant nonzero baseline filters before adding any test.
4. Add or strengthen the full-state success/late-failure matrix on the old implementation first.
5. Prefer delegation to the existing common transaction; do not create a parallel snapshot aggregate.
6. Make HQ completion context explicit and narrow; do not add a boolean with unclear call-site meaning.
7. Preserve exact validation, staging, commit and result/error order unless current tests prove a stronger existing
   contract that the target must retain.
8. Remove only the proven manual rollback list and hidden `take()`/restore bypass.
9. Do not modify public/session/serialization/data/canonical surfaces for private helper placement.
10. Stop for independent audit before Phase 26 activation.

## DTO, Data, Save And Deployment

- DTO migration: none expected; discovery must confirm no public shape changes.
- Live RON/data migration: none expected; HQ authored content and loaders are behavior inputs only.
- Save/checkpoint migration: none expected; transaction rollback may restore checkpoint state but cannot change its
  schema/version or serialized meaning.
- Deployment: Core/server/Unity contract remains byte-identical; no dual read/write, fallback or compatibility
  window is authorized.

## Validation Strategy

- Baseline: current Headquarters/node-session, common completion/map flow, checkpoint/research, Shop and transaction
  filters with explicit nonzero counts.
- Test-first: full-state success and injected late failure for recruit, emergency, open-shop and explicit context.
- Focused implementation: rerun each HQ family plus Phase 24 Shop parity and common completion/checkpoint tests.
- Broad Core: complete debug/release unit and official integrations; debug/release all-target checks.
- Broad server: complete debug/release tests/checks because Core state/result paths are transitively consumed.
- Compatibility: public/serde/V3/RON/DTO/server/external Unity hashes and source inventory.
- Static: no manual HQ clone/restore list, no hidden active-content `take()` bypass, no new fallback/transaction owner,
  no Phase 26 symbols, scoped hashes/status, formatter and repository-wide diff check.
- Diagnostic: non-strict Clippy may inventory debt; strict `-D warnings` is not claimed unless globally green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P25-C01 | complete HQ/common transaction and completion graph identified | all branch callers, roots, exclusions and consumers | complete |
| P25-C02 | dirty targets and concurrent work preserved | pre/post expanded status, scoped diff and hashes | complete |
| P25-C03 | full-state success/late-failure behavior characterized first | nonzero matrix for recruit/emergency/open-shop/completion | complete |
| P25-C04 | common transaction is sole HQ rollback authority | delegate/source evidence; manual ten-root list absent | complete |
| P25-C05 | explicit resolved-HQ completion context replaces hidden bypass | narrow context and caller evidence; no `take()` trick | complete |
| P25-C06 | HQ results, state, errors and ordering remain exact | full-state and branch-specific tests | complete |
| P25-C07 | monotonic compendium and active-battle exclusions remain exact | rollback/exclusion and rejection evidence | complete |
| P25-C08 | public/save/data/DTO/server/Unity compatibility exact | schema/API/consumer/canonical evidence | complete |
| P25-C09 | focused and broad validation green | nonzero debug/release/Core/server/static counts | complete |
| P25-C10 | downstream boundary and independent audit current | no Phase 26 prebuild and `Complete / High` review | complete |

## Activation Protocol

Before production inspection:

1. Master records Phase 25 as the sole `active` Child and Phases 26–44 remain pending.
2. Change all three Child headers from `initialized` to `active` and authorize only this Phase.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` and final Phase 24 audit.
4. Capture HEAD, expanded status count/hash and prospective hashes for likely source/test/canonical targets without
   reading their content.
5. Record that evidence in the Child before examining production details.

No source/test/data/canonical inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 26 if current evidence requires changing any HQ branch outcome, candidate/queue/
roster/UUID/currency/Shop/completion/error order; rolling back compendium knowledge; admitting active battle;
changing public/serde/V3/RON/DTO/wire/save shape; adding compatibility/retry/migration policy; creating a second common
transaction; prebuilding Phase 26; user-owned dirty overlap cannot be separated; required tests select zero cases;
or the independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
