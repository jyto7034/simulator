# Phase 21 Goal — CRA-P02-C04 Encapsulate MapProgression Mutation And Derived Availability

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P02-C04
Master Phase: 21 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 20's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate direct production mutation of the public/serialized `MapProgression` IDs and paired `RunMap` node state
behind typed normal, forced-terminal/BossOmen and admin-fixture transition authorities. Graph-derived selectability
remains admission authority, and the exact current public/V3/map DTO fields and behavior remain unchanged.

This Phase is complete only when every current normal/BossOmen/admin/progression restore mutation and consumer is
inventoried; tests first freeze exact before/after map+progression state for valid and invalid transitions; restored
V3 progression satisfies the same post-restore invariant; one typed progression boundary owns paired ID/node-state
updates for each legitimate lifecycle; direct production mutations outside that boundary are removed; sorting,
deduplication, directed edges, concealed/obscured behavior, BossOmen exit reveal, admin fixture results and map DTOs
remain exact; F-C01, Phase 20 conversion and public/save/wire compatibility are preserved; focused and broad
debug/release validation passes; and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

`MapProgression` already has typed normal transitions, but historical and Boss-era evidence found public stored IDs
and `RunMap` node states also changed directly by BossOmen and admin fixture paths. This creates several mutation
seams for one graph invariant even though current valid flows are green.

This is change-amplification and invariant-ownership risk, not authorization to alter map navigation. The target is
private mutation encapsulation with zero public/V3/DTO/gameplay change.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 21 registry row.
- Direct linear predecessor: Phase 20 CRA-P02-C03, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_20_p02_c03/COMPLETION_AUDIT.md`.
- Correctness dependency: Phase 02 CRA-F-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01/COMPLETION_AUDIT.md`.
- Compatibility policy: CRA-POL-006 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_02_run_map_and_persistence/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

F-C01 now validates restored stored IDs/node states against graph-derived progression before mutation. Phase 20 now
restores the exact validated V3 state through one conversion boundary. CRA-POL-006 permits private mutation
encapsulation but no field removal, navigation change, compatibility shim or public narrowing.

## Priority And Current Disposition

- Priority axes: `3 / 2 / 2 / 2 / 2 / 2 / unknown`; known subtotal `13/18`.
- Re-audited disposition: `still_valid`, `behavior refactor; transition characterization`.
- Current live-tree disposition: implemented and broadly verified; P21-C01–C09 are complete and P21-C10 awaits the
  independent phase-exit audit.

## Current And Target Authority

- Historical current authority: authored `FacilityTemplate` owns topology; `RunMap` node states and public
  `MapProgression` stored IDs jointly own progress; graph-derived `selectable_node_ids` owns admission.
- Historical direct seams: normal typed methods, BossOmen forced-exit mutation and admin fixture assignment/clearing.
- Target authority: typed progression transitions update paired stored IDs and node states together for normal,
  forced-terminal and fixture lifecycles, with a single current invariant checker.
- Graph-derived selectability remains the admission authority. Stored `available_node_ids` remains serialized and
  behaviorally exact; this Phase does not make it an authorization source or delete it.
- Exact current source types, methods, callers and direct-write counts must be rediscovered only after the mandatory
  Child/predecessor read and dirty-baseline gate.

## In Scope

- inventory `MapProgression`, `RunMap`, facility topology, selection/completion/unlock, BossOmen forced capstone/exit,
  admin map fixtures, checkpoint validation/restore and map snapshot/navigation consumers;
- enumerate each stored ID and node-state mutation with precondition, paired effects, sort/dedup behavior and owner;
- capture expanded dirty status, scoped diffs and target fingerprints before production inspection/edits;
- add test-first normal/BossOmen/admin transition tables with exact full map+progression parity and invalid
  no-mutation cases;
- add/retain post-V3-restore invariant proof using the Phase 20 writer/reader and F-C01 admission;
- introduce the smallest typed transition/invariant authority and transfer direct production callers;
- remove only proven direct production mutations superseded by that authority;
- update the nearest map/runtime canonical only if durable ownership wording materially changes.

## Out Of Scope

- removing, privatizing, renaming or changing serialized/public `current_node_id`, `available_node_ids` or
  `completed_node_ids`;
- changing graph-derived selectability, connection direction, sorting/dedup, map generation or node-state meaning;
- changing concealed/obscured rules, BossOmen generation/content policy, forced Boss lifecycle or exit timing;
- changing admin authorization, durability/publication policy or fixture-visible results;
- changing checkpoint V3, F-C01 validation semantics/errors, Phase 20 conversion, load/Continue or host persistence;
- changing map DTO/wire fields, Unity navigation/UX, public Rust consumers or adding compatibility readers;
- Phase 22 equipment/loadout work or any later transaction/cleanup prebuild;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. Public Rust and strict V3 `MapProgression` fields/names/types/values remain exact.
2. Graph-derived selectable nodes remain the command/snapshot admission authority.
3. Normal select/complete/unlock behavior, node state and deterministic stored-ID order remain exact.
4. Directed topology, concealed/obscured nodes and completion transit rules remain exact.
5. BossOmen forced capstone completion reveals exactly the precommitted exit and connection at the current timing.
6. Admin fixtures create the same map/progression/node states and remain separate from player commands.
7. Invalid transitions and invalid restored documents fail without partial map/progression mutation.
8. Phase 20 capture/restore and F-C01 admission remain exact and ordered before live restore mutation.
9. Map snapshot/DTO/server/Unity behavior and authored data remain unchanged.
10. Existing user changes are preserved without reset, restore, clean, stash or broad unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all Child documents exist and Master records sole activation, read them completely and reread the full Phase
20 completion audit. Capture expanded status plus prospective source/test/canonical hashes before production detail.
Treat every existing hunk as user/campaign-owned until reconciled.

Use characterization-first proof in this order:

1. Trace current map generation, stored-ID/node-state mutation, selectability, BossOmen, admin and restore paths.
2. Build an exact transition table and caller/direct-write inventory.
3. Run current map/progression/map-flow/Boss/admin/checkpoint baselines with nonzero counts.
4. Add full before/after valid and invalid transition characterization before structural changes.
5. Introduce one typed paired mutation/invariant boundary without changing public/serde fields.
6. Transfer legitimate direct callers and remove only obsolete direct writes.
7. Re-run focused, complete Core debug/release, server debug/release and static/compatibility gates.

This is behavior-preserving, so intended behavioral red is not required. A structural characterization may fail only
when it directly proves an outside direct-write seam; compile-only or private method-count evidence is insufficient.

## Migration, Deployment And Compatibility

- No migration or coordinated rollout is approved or required for private mutation encapsulation.
- Public fields remain because current V3/public consumers may construct/read them. Negative search cannot narrow
  them.
- Server and Unity continue consuming the same Core map/checkpoint DTOs and never learn the private helper shape.
- If a safe transition requires field/schema/navigation/error behavior change, stop as `policy_decision_required`.

## Validation Plan

- Baseline: normal progression, map flow, BossOmen capstone/exit, admin fixtures, checkpoint validation/round trip.
- Test-first table: normal select/complete/unlock; forced capstone/exit; admin assignment/reset; invalid duplicate,
  missing, wrong-state and unavailable transitions with exact no-mutation.
- Restore: writer-produced V3 restores to the same stored-ID/node-state invariant and selectable projection.
- Behavior: deterministic sort/dedup, directed/concealed/obscured semantics and exact map DTO stay unchanged.
- Broad: complete Core debug/release, official integrations and complete server debug/release.
- Static/compatibility: public/serde field inventory, direct-write/caller search, scoped hashes/status/diff, formatter,
  canonical comparison and no Phase 22 prebuild.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P21-C01 | complete mutation/consumer graph and transition table identified | current caller/direct-write/lifetime inventory | complete |
| P21-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P21-C03 | current normal/Boss/admin behavior characterized first | exact before/after and invalid no-mutation tables | complete |
| P21-C04 | one typed paired progression mutation authority exists | source/caller inventory | complete |
| P21-C05 | obsolete direct production mutation is removed | direct-write and fallback audit | complete |
| P21-C06 | stored IDs/node states/selectability remain exact | invariant, sort/dedup and DTO parity | complete |
| P21-C07 | checkpoint restore and F-C01 remain exact | strict negative and post-restore invariant | complete |
| P21-C08 | public/save/server/Unity/admin/Boss boundaries remain exact | consumer and compatibility evidence | complete |
| P21-C09 | focused and broad validation is green | nonzero debug/release/Core/server/static counts | complete |
| P21-C10 | canonical/downstream boundary and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 22 if public/serialized fields must change; graph-derived navigation or Boss/admin
behavior must change; F-C01/Phase 20 ordering would be bypassed; a direct-write consumer cannot be migrated without
external evidence; user-owned dirty overlap cannot be separated; required tests select zero cases; or the
independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
