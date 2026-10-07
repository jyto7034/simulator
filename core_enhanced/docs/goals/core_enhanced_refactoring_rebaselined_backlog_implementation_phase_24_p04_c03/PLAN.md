# Phase 24 Goal — CRA-P04-C03 Centralize Event/Shop Persisted-Active Session Commits

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P04-C03
Master Phase: 24 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after sole-active registration and completion of all mandatory activation reads/fingerprints
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Centralize the repeated paired commit of Event and Shop sessions to their run-persisted cache and active-node mirror.
Create one private explicit Event commit path and one private explicit Shop commit path, including named required-run
versus active-only semantics where the current behavior differs. Route every legitimate production Event/Shop/HQ
entry or update through the appropriate domain boundary without changing any session value or lifecycle.

This Phase is complete only when the current persisted/active owner and consumer graph is rediscovered; tests first
freeze exact paired equality for every Event entry/update and Shop map/HQ entry/update, explicit no-run behavior,
identity/history/page/purchase/check state, checkpoint/re-entry/retreat behavior, result/error order and public/admin
fixture compatibility; the smallest private domain commit boundaries own all legitimate paired writes; superseded
direct production insert-plus-active assignments are removed; no persistence cache, public/session/DTO/V3/RON/wire
shape or facility model changes; Phase 25/26 work is not prebuilt; focused and broad debug/release validation passes;
and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audits found that Event and Shop session progress has two required roles: `RunState` retains durable
progress for checkpoint/re-entry, while active-node content owns current admission and projection. Domain commit
helpers update both roles, but initial Event/Shop map entry and the Headquarters-to-Shop handoff repeat equivalent
paired writes directly.

The helper behaviors also differ implicitly when no run exists: Event requires a run, while Shop historically keeps
an active-only fixture/runtime behavior. This Phase names and preserves that difference; it does not invent one
generic rule for both domains or reinterpret missing-run behavior as a correctness defect.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 24 registry row.
- Direct linear predecessor: Phase 23 CRA-P03-C04, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_23_p03_c04/COMPLETION_AUDIT.md`.
- Downstream dependency: Phase 25 CRA-F-C02 may start only after this Phase is independently `Complete / High`.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel/TEST_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No policy ID is assigned because this is a private ownership consolidation. CRA-POL-006 remains the compatibility
default: public/admin fixture behavior and every serialized/wire/consumer surface stay exact.

## Priority And Current Disposition

- Priority axes: `2 / 1 / 2 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Final tier/readiness: Tier B, `paired-state characterization`.
- Re-audited disposition: `still_valid`; behavior refactor, paired-state characterization first.
- Historical live-tree disposition: map Event/Shop and HQ Shop activation repeat paired writes outside the helpers;
  BossOmen/Monolith add consumers but no paired-commit owner.
- Current disposition, exact callers, visibility and behavior must be confirmed only after activation.

## Current And Target Authority

- Persisted authority: the current run's Event and Shop caches retain session progress across checkpoint/re-entry.
- Active authority: active node content owns current command admission, projection and result construction.
- Historical shared ownership: domain commit helpers plus initial map/HQ entry sites directly write both copies.
- Target Event authority: one private Event path owns required-run persisted-plus-active commit semantics.
- Target Shop authority: one private Shop path owns explicit persisted-plus-active or preserved active-only mode.
- Callers retain definition lookup, session construction, merchant selection, HQ branch planning, node admission,
  common completion and command/result ownership.
- Exact current types, clone semantics, equality relation, error timing, test fixtures and consumers are activation
  evidence slots rather than assumptions.

## In Scope

- every production Event session entry/update that writes a persisted cache and/or active mirror;
- every production Shop map entry, mutation/update and Headquarters-to-Shop handoff with those writes;
- current private Event/Shop commit helpers, their signatures and missing-run behavior;
- equality and independence of persisted and active session values after each successful transition;
- current session identity, Event check/scene history, Shop page/offer/purchase/refresh history;
- checkpoint capture/restore, re-entry, retreat and projection consumers relevant to the paired copies;
- public/admin fixture construction as compatibility inventory without forced migration;
- exact result/error order and no partial mutation for relevant failures;
- focused and broad Core/server debug/release, formatter, diff, static and canonical checks.

## Out Of Scope

- deleting either persisted cache or deriving one copy from the other;
- merging Event and Shop into one generic facility/session type;
- changing Event graph/check/retry/grant/battle behavior or Shop economy/merchandise behavior;
- changing no-run behavior unless current evidence proves the historical statement obsolete;
- changing session identity, cache key, clone depth, map/common-completion order or checkpoint schema;
- modifying public session constructors, admin/test fixtures merely to make the private boundary uniform;
- adding serialization, migration, repair, fallback, retry or idempotency policy;
- Phase 25 common Headquarters transaction/context replacement;
- Phase 26 VN assembly/queue-placement consolidation;
- unrelated cleanup, data tuning, UI/server/Unity refactors or warning-debt cleanup.

## Required Test-First Characterization Matrix

| Family | Success equality | History/identity evidence | Failure/no-run evidence |
| --- | --- | --- | --- |
| Event map entry | persisted session equals active Event session | node/session/event identity and initial scene exact | required-run failure and full state unchanged |
| Event update/check/advance | both copies equal after every committed mutation | committed check, scene and at-most-once grant/battle state exact | invalid/stale/late failure preserves both copies and order |
| Event checkpoint/re-entry/retreat | restored/re-entered active view agrees with persisted progress | no reroll/regrant; same identity/history | invalid restore/re-entry behavior unchanged |
| Shop map entry | persisted session equals active Shop session | shop/profile/page/offer identity exact | required current missing-run semantics exact |
| Shop purchase/sale/refresh updates | both copies equal after each success | page, purchases, offers, ledger and remaining-use history exact | failure leaves both copies and economy exact |
| HQ-to-Shop handoff | persisted Shop and active Shop values exact | HQ branch result/order and merchant selection exact | late branch failure/current transaction rollback exact |
| Shop checkpoint/re-entry/exit | durable history and active projection remain exact | no page/purchase regeneration | result/common completion/error order exact |
| Fixture/admin surface | existing construction/admission stays compatible | no public signature/shape change | active-only Shop behavior explicitly retained if current |

Characterization must compare the complete relevant session values, not merely discriminants or selected fields.
Tests must observe public world behavior or authoritative state and must not require a particular helper name.

## Implementation Constraints

1. Capture expanded status, HEAD and hashes before inspecting Phase 24 production detail.
2. Inventory every read/write/copy of Event/Shop persisted and active sessions and classify fixtures separately.
3. Run current relevant filters and record nonzero selected counts before adding tests.
4. Add or strengthen the complete paired-state matrix on the old implementation before production edits.
5. Prefer two narrow domain-specific paths or one boundary with an explicit typed mode; do not encode missing-run
   semantics through a boolean whose meaning is invisible at call sites.
6. Preserve exact clone/move behavior and existing successful commit order unless tests prove value equivalence is
   insufficient for a consumer.
7. Keep session construction, domain validation, common completion and HQ transaction ownership outside.
8. Remove only proven superseded production paired-write boilerplate.
9. Rerun focused tests after each caller family, then broad debug/release/static gates.
10. Stop for independent audit before Phase 25 activation.

## Validation Strategy

- Baseline: current Event/Shop/HQ node-session and map-flow filters, checkpoint/re-entry/retreat and public serde/
  snapshot tests with explicit nonzero counts.
- Test-first: each required matrix row must fail or demonstrate the missing structural safety net before refactor;
  record any fixture defect separately from production behavior.
- Focused implementation: rerun every touched family plus Event/Shop pure/helper tests.
- Broad Core: complete debug and release unit plus official integrations; debug/release all-target checks.
- Broad server: complete debug and release tests/checks if Core public or dependency inputs are in scope, otherwise
  retain only evidence whose inputs provably stayed byte-identical and explain why.
- Compatibility: public API, serde/V3/RON/DTO/server/external Unity inventories and canonical hashes/diffs.
- Static: direct persisted-cache/active assignment search, helper/caller inventory, fallback/legacy search, no Phase
  25/26 prebuild, scoped hashes/status, formatter and repository-wide diff check.
- Diagnostic: non-strict Clippy may inventory existing debt; strict `-D warnings` is not claimed unless globally green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P24-C01 | complete paired-session owner/consumer graph identified | Event/Shop/HQ reads, writes, entry/update and fixture inventory | complete |
| P24-C02 | dirty targets and concurrent work preserved | pre/post expanded status, scoped diff and hashes | complete |
| P24-C03 | exact paired/no-run behavior characterized first | complete matrix with nonzero before/after evidence | complete |
| P24-C04 | explicit private Event and Shop commit authority exists | source/type/visibility/mode evidence | complete |
| P24-C05 | all legitimate production entry/update callers delegate | map Event/Shop, updates and HQ handoff | complete |
| P24-C06 | superseded direct paired writes removed | negative assignment/fallback audit | complete |
| P24-C07 | identity/history/checkpoint/error behavior exact | full session equality and lifecycle tests | complete |
| P24-C08 | public/V3/RON/DTO/server/Unity compatibility exact | schema/API/consumer/canonical evidence | complete |
| P24-C09 | focused and broad validation green | nonzero debug/release/Core/server/static counts | complete |
| P24-C10 | downstream boundary and independent audit current | no Phase 25/26 prebuild and `Complete / High` review | complete |

## Activation Protocol

Before production inspection:

1. Master records Phase 24 as the sole `active` Child and Phases 25–44 remain pending.
2. Change all three Child document headers from `initialized` to `active` and authorize only this Phase.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` and the final Phase 23 completion audit.
4. Capture HEAD, expanded status count/hash and prospective hashes for likely source/test/canonical targets without
   reading their content.
5. Record that evidence in the Child before examining production details.

No source/test/data/canonical inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 25 if current evidence requires deleting a persisted cache, changing required-run or
active-only semantics, changing clone identity/history, error/result ordering, Event/Shop gameplay, checkpoint/public/
DTO/wire shape, admin fixture behavior or a deployed consumer; a new migration/fallback/retry policy is required;
user-owned dirty overlap cannot be separated; required tests select zero cases; a Phase 25/26 owner must be prebuilt;
or the independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
