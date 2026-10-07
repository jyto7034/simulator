# Phase 26 Goal — CRA-P04-C04 Shared Visual-Novel Session Assembly And Explicit Queue Placement

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P04-C04
Master Phase: 26 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after sole-active registration and completion of all mandatory activation reads/fingerprints
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Replace the duplicate visual-novel session construction performed by trigger evaluation and Monolith exchange with
one private assembly authority, while keeping queue placement an explicit caller-owned choice: triggers append at
the tail and Monolith inserts at the front. Preserve definition lookup, presentation conversion, UUID allocation and
ordering, entry-scene selection, blocking/source metadata, choice initialization, active-session noninterruption,
trigger priority, checkpoint projection and every accept/decline result exactly.

This Phase is complete only when the current producer/consumer/serialization graph is rediscovered; characterization
tests first prove the full Trigger and Monolith matrix on the old implementation; a single private factory owns only
the duplicated session assembly; a narrow typed placement policy makes `Tail` versus `FrontOfQueue` explicit; no raw
duplicate constructor or ambiguous queue mutation remains at those two producers; public, save, RON, wire, server and
Unity contracts are unchanged; no Phase 27 work is prebuilt; focused and broad debug/release validation passes; and
an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical audits found that trigger evaluation and Monolith exchange independently perform the same definition
lookup, presentation conversion, UUID generation, entry-scene selection, blocking/source assignment and empty-choice
initialization. Their queue positions intentionally differ, so blind deduplication could silently invert narrative
priority or interrupt the active session.

The intended result is behavior-preserving. Players must see the same VN, UUID sequence, trigger order, active
session, pending order, blocking behavior and Monolith accept/decline consequences. This Phase changes private
construction ownership, not narrative content or queue policy.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 26 registry row.
- Direct linear predecessor: Phase 25 CRA-F-C02, final `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_25_f_c02/COMPLETION_AUDIT.md`.
- Required prerequisite: Phase 25 stabilizes interactive-node completion/transaction semantics before VN queue
  ownership is changed.
- Downstream dependency: Phase 27 CRA-P05-C03 may start only after this Phase is independently `Complete / High`.
- Original source evidence:
  - `../core_enhanced_refactoring_audit_phase_04_interactive_node_and_visual_novel/CANDIDATE_CATALOG.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/CANDIDATE_REAUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No gameplay policy ID is assigned because the target is private assembly and caller-explicit placement. CRA-POL-006
remains the compatibility default for public/admin/serialized/consumer behavior.

## Priority And Current Disposition

- Priority axes: `2 / 1 / 2 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Final tier/readiness: Tier C, characterization before consolidation.
- Re-audited disposition: `still_valid`; Boss-era trigger consumers expand the graph but do not centralize assembly.
- Historical Trigger behavior: construct from the resolved definition, allocate the next UUID, then enqueue at the
  tail without replacing or interrupting active content.
- Historical Monolith behavior: construct the same session shape, allocate through the same UUID authority, then
  insert at index zero/front so it precedes older pending sessions while leaving the current active session intact.

## Current And Target Authority

- Historical current assembly authority: duplicated constructor/conversion logic in Trigger and Monolith producers.
- Historical current placement authority: local raw queue append versus index-zero insertion.
- Target assembly authority: one private function/type owns definition-to-runtime-session conversion and UUID
  allocation exactly once.
- Target placement authority: callers explicitly choose a narrow `Tail` or `FrontOfQueue` policy; the factory does
  not infer placement from source kind.
- Trigger matching, priority and consumption remain owned by trigger evaluation. Monolith exchange validation,
  payment/reward and accept/decline behavior remain owned by Monolith. Runtime advancement and serialization retain
  their current owners.

## In Scope

- every current Trigger and Monolith producer of the duplicated VN runtime session shape;
- definition lookup and missing-definition behavior at those producer paths;
- definition-to-presentation/runtime conversion;
- exact UUID allocation point and order relative to validation, mutations and enqueue;
- entry-scene, source, blocking and uncommitted-choice initialization;
- tail and front-of-queue placement, including existing active and pending sessions;
- trigger priority/order/noninterruption and Monolith accept/decline behavior;
- checkpoint/save projection and restore compatibility checks;
- focused/broad Core/server debug/release, formatter, diff, static and canonical compatibility checks.

## Out Of Scope

- changing VN definitions, authored scenes, choices, triggers, trigger priority or source identity;
- changing Monolith costs, rewards, validation, accept/decline semantics or node completion;
- replacing the active VN, interrupting it, or collapsing active and pending ownership;
- changing UUID format, generator, persistence or allocation semantics;
- changing public session/behavior/DTO/V3/RON/wire/checkpoint shapes or enum tags;
- broad queue abstraction, scheduler, priority system, retry, deduplication or idempotence policy;
- consolidating direct test fixtures unless a shared fixture remains behavior-observing and is independently useful;
- CRA-P04-C01 or Phase 27 implementation;
- unrelated warning debt, data, server, Unity, node-session or transaction refactors.

## Required Test-First Characterization Matrix

| Family | Required success evidence | Order/noninterruption evidence | Failure/compatibility evidence |
| --- | --- | --- | --- |
| Trigger with empty queue | exact runtime session fields and result | new session is tail/only pending; UUID exact | missing/invalid definition preserves state and error |
| Trigger with active VN | active session byte/equality unchanged | triggered session queues behind active and prior pending in priority order | no implicit replacement or advancement |
| Multiple triggers | exact winning/eligible trigger behavior | trigger priority and UUID progression exact | ineligible/consumed triggers cause no drift |
| Monolith accept | exact exchange/result plus VN fields | new session is queue front, ahead of older pending, active unchanged | insufficient/invalid exchange preserves state/error |
| Monolith decline | exact decline result/state | no unintended queue/UUID mutation | public behavior unchanged |
| Projection/restore | session identity, source, blocking, scene and choice state exact | active/pending order round-trips | serialized/checkpoint schema unchanged |

Tests must observe externally meaningful state and results. They may use deterministic projections, but must not
require a particular private helper name, enum spelling or function placement. Existing direct constructors in test
fixtures are not production duplication unless they themselves define an independent behavior authority.

## Implementation Constraints

1. Capture expanded status, HEAD and prospective hashes before inspecting Phase 26 production detail.
2. Inventory every VN session producer, constructor field, definition converter, queue mutation and serialized
   consumer before choosing the factory boundary.
3. Run relevant nonzero baseline filters before adding any test.
4. Add or strengthen the full Trigger/Monolith/UUID/order/noninterruption matrix on the old implementation first.
5. Keep assembly and placement separate: one shared constructor plus explicit typed placement.
6. Preserve the exact allocation and validation order; do not allocate a UUID on paths that previously did not.
7. Preserve active-session noninterruption and exact pending order under both empty and populated queues.
8. Remove only proven duplicated production assembly/raw placement at the two candidate producers.
9. Do not modify public/session/serialization/data/canonical surfaces for a private ownership change.
10. Stop for independent audit before Phase 27 activation.

## DTO, Data, Save And Deployment

- DTO migration: none expected; discovery must prove all public representations remain exact.
- Live RON/data migration: none expected; authored VN and trigger content are behavior inputs and stay unchanged.
- Save/checkpoint migration: none expected; runtime session fields/order/meaning and schema version stay unchanged.
- Deployment: Core/server/Unity contract remains byte-identical; no dual read/write, fallback or compatibility window
  is authorized.

## Validation Strategy

- Baseline: current VN trigger, Monolith/world/facility, map flow, checkpoint/snapshot and runtime filters with explicit
  nonzero counts.
- Test-first: Trigger Tail, Monolith FrontOfQueue, UUID progression, active noninterruption, trigger priority,
  blocking, accept/decline and projection/restore.
- Focused implementation: rerun each producer family plus runtime queue/choice/advance and checkpoint tests.
- Broad Core: complete debug/release unit and official integrations; debug/release all-target checks.
- Broad server: complete debug/release tests/checks because Core state/result/checkpoint paths are consumed there.
- Compatibility: public/serde/V3/RON/DTO/server/external Unity hashes and source inventory.
- Static: one private assembly owner, explicit two-placement vocabulary, no duplicate producer constructor blocks, no
  ambiguous raw candidate queue insertion, no fallback or Phase 27 symbols, scoped hashes/status, formatter and diff.
- Diagnostic: non-strict Clippy may inventory established debt; strict `-D warnings` is not claimed unless globally
  green.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P26-C01 | complete VN producer/consumer graph identified | Trigger, Monolith, runtime, queue, UUID and projections | complete |
| P26-C02 | dirty targets and concurrent work preserved | pre/post expanded status, scoped diff and hashes | complete |
| P26-C03 | old behavior characterized first | nonzero Trigger/Monolith/order/UUID/noninterruption matrix | complete |
| P26-C04 | one private session-assembly authority | shared factory/source evidence; duplicate blocks absent | complete |
| P26-C05 | placement remains explicit and exact | typed Tail/FrontOfQueue calls and populated-queue tests | complete |
| P26-C06 | result, errors, UUID and trigger behavior exact | state/result/order tests | complete |
| P26-C07 | active, blocking, choice and projection semantics exact | runtime/checkpoint tests | complete |
| P26-C08 | public/save/data/DTO/server/Unity compatibility exact | schema/API/consumer/canonical evidence | complete |
| P26-C09 | focused and broad validation green | nonzero debug/release/Core/server/static counts | complete |
| P26-C10 | downstream boundary and independent audit current | no Phase 27 prebuild and `Complete / High` review | complete |

## Activation Protocol

Before production inspection:

1. Master records Phase 26 as the sole `active` Child and Phases 27–44 remain pending.
2. Change all three Child headers from `initialized` to `active` and authorize only this Phase.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` and final Phase 25 audit.
4. Capture HEAD, expanded status count/hash and prospective hashes for likely source/test/data/canonical targets using
   path discovery only, without reading their content.
5. Record that evidence in the Child before examining production details.

No source/test/data/canonical inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 27 if current evidence requires changing any narrative content, trigger eligibility or
priority, Monolith exchange result, UUID allocation, active/pending order, blocking/source/choice meaning, checkpoint
or public/serde/V3/RON/DTO/wire shape; adding retry/deduplication/migration/fallback policy; broadening into a queue
scheduler; prebuilding Phase 27; user-owned dirty overlap cannot be separated; required tests select zero cases; or
the independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
