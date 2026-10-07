# Phase 09 Goal — CRA-P07-C02 Preflight And Commit One Atomic Tactical World Mutation

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P07-C02
Master Phase: 09 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 08's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make deploy, withdraw, stance change, tactical relocation and tactical swap each execute through one atomic world
mutation boundary. Every request must validate battle authority, revision capacity, event cursor and command-specific
fallible inputs before authoritative commit, or stage all affected state until fallible work and response projection
succeed. A returned error must leave the complete world state exactly unchanged.

This Phase is complete only when all five command entry/validation/Core/world/reconciliation/revision/DTO paths are
identified; otherwise-valid future-cursor tests reproduce post-error mutation for every command; rare Core-deploy or
employee-reconciliation failure is injected at the actual late seam; one shared transaction/preflight boundary
protects complete Core and adjacent world state; success retains exact command reasons, one cost/revision/event and
Pause semantics; current DTO/wire shape is unchanged; focused/broad validation passes; and phase-exit audit returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found five world handlers that mutate live BattleCore and adjacent deployment/resource/employee
state, commit revision, and only then construct a battle update that can reject a future event cursor. Deploy and
withdraw also had fallible employee reconciliation after Core mutation. A caller can therefore receive an error
even though the next checkpoint exposes committed gameplay state.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 09 registry row.
- Direct predecessor: Phase 08 CRA-P07-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_08_p07_c01/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Phase 08 stabilizes the complete tactical-effect transition that this transaction must protect. No
candidate-specific policy ID applies. CRA-POL-006 requires exact command/result/nesting and checkpoint/DTO/wire/
public compatibility; failure atomicity is an existing canonical contract, not a new gameplay decision.

## Priority And Current Disposition

- Priority axes: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`.
- Final disposition at activation: `still_valid`, correctness correction, staged command tests first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historical current authority: five handlers repeat validate→mutate Core/world→commit revision→build DTO, while
  cursor validation and some employee reconciliation remain fallible after authoritative mutation.
- Target authority: one common tactical world mutation boundary preflights all shared fallible inputs, stages any
  remaining fallible Core/adjacent mutation, and publishes Core/resource/deployment/employee/revision/DTO exactly once
  only after complete success.
- Current function/type names, existing transaction helpers and exact late failures must be rediscovered after the
  required document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify deploy, withdraw, stance, relocation and swap world entrypoints through authority validation, Core
  command, tactical resource, deployment maps/salt/locks/facing, employee attempt reconciliation, signals, revision
  and final update DTO;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical target;
- add otherwise-valid future-cursor tests for all five commands and compare complete affected world state;
- inject or construct real Core-deploy and employee-reconciliation failure after the earliest shared preflight and
  prove exact rollback/no publication;
- introduce or reuse the smallest common preflight/staging boundary without batching separate client commands;
- preserve stale UUID/revision, overflow, command-specific disabled reasons, cost, Pause, cooldown, source command
  correlation, Phase 08 effects and success response ordering;
- update the nearest canonical only after runtime behavior is real.

## Out Of Scope

- changing Phase 08 tactical-effect meaning or field-work assignment policy;
- merging clock advance, manual skill targeting/cast or unrelated player commands into one global transaction;
- batching multiple requests or rolling back an earlier successful serialized request because a later request fails;
- changing tactical costs, cooldown, deploy salt/redeploy HP/lock, incapacitation or resource policy;
- movement backend/Boss cleanup and server actor ownership changes;
- changing RON, DTO/wire/public Rust, checkpoint/save version, command/result tags or nesting;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. Every failure leaves Core units/scenario/log/signals/queue, deployment maps/salt/locks/facing, tactical resource,
   employee attempt/condition, focus, revision and response-visible state exactly unchanged.
2. Every success commits exactly one command-specific Core mutation, resource charge where applicable, event/log
   set, world reconciliation and battle revision increment.
3. Command-specific validation priority and typed disabled/error reason remain exact.
4. Battle UUID/revision authority, revision overflow and future cursor reject before publish.
5. Pause freezes simulation time but continues to allow the same command-time mutations.
6. Phase 08 final tactical effects and field-work final occupancy are preserved inside the transaction.
7. Source-command correlation and final battle update/checkpoint/event ordering remain exact.
8. CRA-POL-006 preserves current RON, DTO/wire/public Rust, server and checkpoint/save shape.
9. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded dirty status and prospective target status/diff/hash before inspecting current source/tests/canonical.
Treat all existing changes as user-owned and edit only the shared tactical transaction/preflight, five call sites,
exact failure/success tests and nearest canonical.

Use test-first proof in this order:

1. Run nonzero live deploy/withdraw/stance/relocation/swap, authority/revision and transaction baselines.
2. Trace the complete state footprint and enumerate every `?`/fallible call before and after first mutation.
3. For each command, submit future `after_seq` after otherwise-valid preflight and snapshot exact whole-world state.
4. Add the real late Core-deploy/employee-reconciliation failure case without weakening production validation.
5. Preflight cursor/shared authority and stage remaining fallible state through one narrow transaction boundary.
6. Re-run exact failures, successful one-cost/one-revision/one-event behavior, Pause, Phase 08 and owning suites.

## Migration, Deployment And Compatibility

- No data, schema, save/checkpoint, DTO/wire or version migration.
- Errors that previously leaked committed state become genuinely no-mutation; successful serialized command behavior
  and response shape remain identical.
- Server/Unity require no rollout because command/result tags, payload nesting and ordering are unchanged.
- If completion requires a public breaking transaction API, broader command batching or new failure policy, stop for
  user direction rather than expanding this Phase.

## Validation Plan

- Baseline: five command world families plus Core tactical/field-work and authority/revision tests with nonzero
  counts.
- First proof: five future-cursor whole-state equality cases and a real late reconciliation/Core-deploy failure.
- Focused final: exact error priority, complete rollback, one success cost/revision/event, Pause, deployment salt/
  lock/facing, employee condition, resource, signals, Phase 08 effects and update DTO.
- Broad: affected world combat/tactical families, active battle projection/server compile, live RON and complete Core.
- Static: complete five-handler fallible-call/mutation inventory, post-commit `?` search, transaction caller scope,
  policy/public shape audit, scoped status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P09-C01 | five complete command/state/failure paths identified | entry→mutation→DTO inventory | completed |
| P09-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | completed |
| P09-C03 | future cursor reproduces post-error mutation for every command | five exact whole-state red proofs | completed |
| P09-C04 | rare late reconciliation/Core failure is atomic | injected/constructed exact rollback proof | completed |
| P09-C05 | one shared boundary owns tactical mutation publish | preflight/staging and caller inventory | completed |
| P09-C06 | success, Pause, effects, cost/revision/event invariants remain exact | focused matrix | completed |
| P09-C07 | focused and affected broad validation passes | nonzero counts and static checks | completed |
| P09-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | completed — Complete / High |

## Stop Conditions

Stop and do not activate Phase 10 if current code already rejects every named failure before mutation or stages all
affected state; a complete transaction requires changing separate-command semantics; exact failure rollback cannot
preserve current success ordering; public/data/save shape must change; dirty overlap is inseparable; a focused runner
selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.

## Completion Result

- Phase exit audit: `Complete / High`.
- Improvement class: `none`.
- Deploy, withdraw, stance, tactical relocation and tactical swap now publish one complete staged active-battle
  transaction only after final transport projection succeeds.
- Complete Core validation: 955 passed, 0 failed; live integration and game_server all-target compile pass; no data,
  DTO/wire/public or persistence migration.
- Master may mark Phase 09 complete and make Phase 10 ready.
