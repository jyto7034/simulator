# Phase 02 Goal — CRA-F-C01 Complete Checkpoint Invariant Graph

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-F-C01
Master Phase: 02 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 request to read and perform the Master PLAN activates
this Phase after Phase 01's `Complete / High` exit; authorization is limited to this Phase until its own
`Complete / High` review
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Make the current V3 checkpoint a complete fail-closed invariant graph before any restore mutation. One validation
authority must compose the serialized run/UUID seed, graph-derived progression, roster, equipment/reverse bindings,
fragments and economy roots so a one-edge inconsistency is rejected while the current Core state, host load count
and re-export remain unchanged. Valid current V3 round trips must retain exact behavior.

This Phase is complete only when the live current writer, validator, load entry points, mutation boundary and tests
have been identified; every affected dirty path is preserved; one-edge-at-a-time current-V3 negatives cover all
named graph domains before mutation for Continue and explicit load as applicable; valid round-trip compatibility is
unchanged; focused and affected broad validation passes; and an independent completion review returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical current-tree evidence found that checkpoint validation rejects several local reference errors but does
not compose the document's seed/progression and asset/economy roots into one pre-restore predicate. Restore then
assigns those roots. A syntactically valid but internally inconsistent current V3 document can therefore cross the
admission boundary and produce state that no normal runtime transition would create.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 02 registry row.
- Direct predecessor: Phase 01 CRA-P09-C08, independently reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_01_p09_c08/COMPLETION_AUDIT.md`.
- Merged historical identities: CRA-P02-C02 seed/progression contribution plus CRA-P03-C01
  roster/inventory/economy contribution; neither remains a separate implementation row.
- Downstream dependencies: CRA-P02-C03 checkpoint mapping and CRA-P03-C03 economy mapping must build on this
  admission invariant.
- Historical evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/TEST_REAUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Historical documents identify the candidate boundary but do not substitute for current code, data, consumer or
test inspection.

## Priority And Current Disposition

- Priority axes: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`.
- Final disposition at activation: `still_valid`, `correctness_correction`, test first.
- Current live-tree disposition: implemented and independently reviewed `Complete / High`; completion evidence is
  in `COMPLETION_AUDIT.md`.

## Current And Target Authority

- Historically identified current authority: Core V3 checkpoint capture/validation/restore, including
  `validate_run_checkpoint_payload`; server bytes are the serialized producer and server Continue/explicit-load
  flows are host consumers.
- Host authority remains limited to envelope checksum/version, current/backup selection, tombstone and monotonic
  metadata; it does not define Core payload meaning.
- `MapProgression`/`RunMap` remain graph-derived progression/selectability authorities.
- Exact live source files, functions and owning Cargo targets must be rediscovered from the activation tree before
  mutation and then recorded in `EXPERIMENTS.md`.
- Target authority: one current-version validation graph that executes before all restore mutation and is shared by
  every applicable load route rather than duplicated in a host caller.

## In Scope

- discover the current V3 writer, validator, restore mutation boundary, Continue and explicit-load consumers;
- inventory the serialized seed/progression, roster, inventory/equipment, fragment and economy relationships and
  distinguish document invariants from graph-derived runtime invariants;
- capture status, diff and fingerprints for every affected path before editing;
- create otherwise-valid current V3 fixtures and mutate exactly one relationship per negative;
- reject inconsistent run/UUID seed and graph-derived progression/selectability edges;
- reject inconsistent roster ownership, equipment slot/reverse binding, inventory reserve, material stack,
  fragment ownership/loadout/progress and other evidence-confirmed economy edges;
- prove rejection occurs before state/load-count/host metadata mutation and leaves a subsequent export unchanged;
- retain valid current V3 round-trip behavior and current public error/compatibility boundaries;
- update the nearest canonical only if the implemented current contract is not already described accurately.

## Out Of Scope

- automatic checkpoint repair, rebinding, normalization, partial salvage or old-version migration;
- changing the current V3 serialized field set, tags, public Rust/API surface or host envelope contract;
- implementing CRA-P02-C03/CRA-P03-C03 mapping refactors or any later candidate;
- persisting active tactical/Boss runtime, which remains deliberately private and excluded from V3;
- cross-actor lease/conflict policy, admin durability or a new load protocol;
- unrelated cleanup or dirty-worktree normalization.

## Preserved Invariants And Policy

1. CRA-POL-006 applies directly: current V3 is a strict current-version document and inconsistent graphs fail before
   mutation; repair/rebind and old-version migration are not introduced.
2. A valid V3 produced by the current writer retains exact round-trip behavior and public shape.
3. Invalid input cannot partially change Core state, host `loads_used`/monotonic metadata, selected file generation
   or the bytes/meaning of a subsequent export.
4. Each negative differs from an otherwise-valid control by one named graph edge.
5. Graph-derived progression/selectability remains owned by the map/progression authorities, not a parallel rule
   embedded in persistence.
6. Active `BattleCore`, Boss runtime and tactical queues remain excluded and cleared according to the existing V3
   lifetime contract.
7. Pre-existing and concurrent user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct policy: CRA-POL-006. Open policy decisions: none at activation. If implementation requires choosing a repair,
migration, new public error/wire shape or a different ownership rule, stop instead of guessing.

## Implementation And Dirty-Worktree Strategy

After these three Child documents exist and Master records sole activation, establish the full status baseline and
the exact status/diff/content fingerprint of each discovered source, test, canonical and fixture target. Treat all
pre-existing changes as user-owned. Restrict edits to identifiable validation/test seams; stop if overlapping intent
cannot be separated.

Use test-first proof in this order:

1. Establish nonzero valid-round-trip and current negative baselines for the owning targets.
2. Build or reuse a valid checkpoint from the current writer and prove it round trips unchanged.
3. Add one-edge seed/progression negatives and prove pre-mutation rejection.
4. Add one-edge roster/equipment/fragment/economy negatives and prove the same atomic boundary.
5. Implement the smallest shared validation-graph correction using existing domain authorities.
6. Re-run focused, boundary and owning-package validation; inspect public shape, canonical alignment and dirty
   preservation before completion review.

## Migration, Deployment And Compatibility

- Checkpoint/save: preserve exact current V3 field set and version; only internally inconsistent current documents
  become reliably rejected before mutation.
- DTO/wire: preserve command/result tags, payload nesting and byte transport behavior.
- Public Rust/API: preserve current symbols and signatures unless an already-internal helper is introduced.
- Live RON/data: no authored-data schema or publication change.
- Boss runtime: remains private/non-serde and outside checkpoint payload.
- Deployment: no automatic repair or rollout migration; external current-V3 producers must already satisfy the
  writer's graph invariants.
- Temporary compatibility: none permitted or required.

## Validation Plan

- Baseline: current valid checkpoint round trip plus the smallest existing checkpoint-negative filters, each with a
  nonzero executed count.
- First proof: one-edge current-V3 seed/progression/asset/economy negatives reject before mutation; valid round trip
  remains unchanged.
- Atomicity: snapshot/equality and re-export checks around both Continue and explicit load where both reach the
  admission authority; assert load count and host metadata do not advance on failure.
- Focused regression: complete checkpoint validation/round-trip test module with actual pass/fail/skip counts.
- Boundary regression: owning Core package plus server persistence/continue consumers selected from current
  metadata and call graph.
- Static checks: exact V3 shape/serde diff, scoped validator/restore call inventory, canonical comparison,
  target-file status/diff/hash and `git diff --check`.
- Broader checks are chosen after current Cargo topology is confirmed; zero-test filters are not counted as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P02-C01 | live writer/validator/restore/load authorities identified | current call/consumer inventory and E01 | pass |
| P02-C02 | dirty targets and concurrent work preserved | pre/post status, diff and content fingerprints | pass |
| P02-C03 | valid current V3 round trip remains exact | nonzero control test and shape/export evidence | pass |
| P02-C04 | seed/progression graph is fail-closed | one-edge negatives with exact pre-mutation assertions | pass |
| P02-C05 | roster/equipment/fragment/economy graph is fail-closed | one-edge negatives with exact pre-mutation assertions | pass |
| P02-C06 | all load routes share the admission boundary | Continue/explicit-load consumer evidence and atomicity tests | pass |
| P02-C07 | no unauthorized compatibility or ownership drift | CRA-POL-006, V3/wire/API/Boss and canonical review | pass |
| P02-C08 | affected focused and broad validation passes | runner-reported nonzero counts and static checks | pass |
| P02-C09 | independent review is Complete / High | `COMPLETION_AUDIT.md` | pass |

## Stop Conditions

Stop this Phase and do not activate Phase 03 if the candidate no longer reproduces; the current writer itself
produces a graph that contradicts the proposed invariant; a relationship cannot be decided from runtime/domain
authority; rejection cannot be placed before mutation without changing public protocol; repair/rebind/migration or a
new policy is required; target edits overlap inseparably with user work; a focused runner executes zero tests; or
the completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
