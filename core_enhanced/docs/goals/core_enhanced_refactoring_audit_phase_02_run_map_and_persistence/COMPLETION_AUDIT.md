# Core Enhanced Refactoring Audit Phase 02 — Completion Audit

Audit Mode: phase_exit
Audit Target: `core_enhanced_refactoring_audit_phase_02_run_map_and_persistence`
Current Status At Audit Start: active
Implementation Complete Claim: not_applicable
Policy Source: Parent Master CRA-01~CRA-13; frozen `engineering_workflow.md#리팩토링-판단`
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Primary Artifacts: `BOUNDARY_AUDIT.md`, `CANDIDATE_CATALOG.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`, start commit
`f318414fb0156a67e04c5479000c2a2683fc0c09`, current dirty working tree

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy evidence | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P2-01 | exact boundary coverage | CRA-01, CRA-03, CRA-08, Phase scope | root world/map/RNG/checkpoint and direct server store/actor/canonical/test inventory reconciled | seven map files, focused world/store files and consumers recounted | pass |
| P2-02 | run root authority trace | CRA-03, CRA-05 | session seed → Core → briefing/starter selection → `RunProgression`/fixed Floor map; Core/server ownership separated | start/snapshot and map-flow tests | pass |
| P2-03 | map/Floor/node progression trace | CRA-03, CRA-05, CRA-09 | authored template topology, fixed `RunMap`, derived admission, staged node/Floor transitions and carry-forward traced | 25 map + 40 map-flow tests | pass |
| P2-04 | atomicity/rollback lifecycle | CRA-05, CRA-07 | world transaction, battle-entry snapshot and host rollback lifetimes distinguished; profile/peak exclusions traced | transaction filter 4 pass plus focused atomic failure tests | pass |
| P2-05 | checkpoint/version authority | CRA-03, CRA-05, CRA-07 | SavePoint capture/load, V3 header/body, max loads, UUID/VN behavior and strict rejection traced; gaps C02/C03 explicit | facility/node/start/UUID focused tests | pass |
| P2-06 | server persistence seam | CRA-03, CRA-08 | lazy Continue bootstrap, V3 envelope/checksum/current/backup/temp/tombstone/lock/fsync and actor rollback traced | 13 active-run server tests | pass |
| P2-07 | canonical/test safety net | CRA-02, CRA-03 | durable runtime/game/data claims compared with source; C01 mismatch and missing negative coverage explicit | 136 pass executions, 0 failed/ignored | pass |
| P2-08 | candidate/disposition quality | CRA-04~CRA-08, CRA-12 | C01~C04 and S01~S12 all carry work type/boundary or explicit rejection/defer/unknown reason | required-field and row recount | pass |
| P2-09 | dirty preservation/no mutation | CRA-10, CRA-11 | only Goal artifacts changed during Phase; production dirty set preserved | seven initial/final content/status hashes identical | pass |
| P2-10 | completion audit verdict | Phase completion gate | this independent audit rechecked current artifacts/source/tests/canonical | all matrix rows pass | pass |

## Runtime And State Evidence

- Core does not create the active run at `StartNewGame`; it opens the mode-specific briefing and installs
  `RunState` only after starter selection.
- `RunProgression` owns run seed/mode/Floor progression. Published run policy and map catalogs feed deterministic
  facility generation, which stores the complete map and facility-pool state.
- `RunMap` plus current authored template is navigation authority. Node selection is preview-only; confirmed entry
  and staged completion update progression, Floor state and root state through explicit transaction seams.
- Last-completed-SavePoint payload is separate from run-scoped load count and run peak. Explicit load restores the
  payload and increments once; Continue injects host/player state and does not increment.
- Non-battle world rollback and durable host rollback preserve their explicit monotonic aggregates; battle entry
  restoration remains a separate later-domain lifetime.

## Data And Map Evidence

- Phase 1 publication is accepted only as typed input. Phase 2 directly traced `RunPolicyData`, facility
  schedule/pool/template and manifested map consumption.
- Facility topology comes from semantic slots and directed authored connections; stored node IDs/content and pool
  cycle state prevent reconnect/load regeneration.
- `MapProgression` and node states currently maintain related availability representations. Valid flows pass, but
  mutation is distributed and import validation is reference-only in part; C02/C04 record the exact boundaries.
- Phase 1 C02 caveat remains intact: these map traces do not claim every manifested difficulty was startup
  preview-generated.

## DTO/Transport/Persistence Evidence

- Core V3 and host envelope V3 are separate strict version layers. V2, missing version, unknown/missing current
  tactical fields, malformed JSON/Base64 and checksum mismatch are rejected; no migration is implied.
- Host persists opaque Core bytes plus only `loads_used` and run peak. Model/anchor stay inside the Core payload.
- Current/backup recovery is explicit and tested; two invalid copies require reset rather than empty-run fallback.
- Server compares checkpoint exports around successful commands, invalidates terminal/replacement state, publishes
  changes and rolls Core back on host persistence failure.
- C01 proves ordinary current publish does not independently max-merge a lower incoming host high-water, despite
  correct backup merge.

## Consumer Evidence

- confirmed: official server startup/settings, `LoadBalanceActor`, player actor, `ActiveRunStore`, local active-run
  documents and public Core active-run calls used by the server.
- confirmed but policy-deferred: development admin mutation bypasses the normal persistence transaction wrapper.
- confirmed structural risk but policy-deferred: different player actors may share one local-install store without
  a logical lease/conflict rule.
- unknown: other public Rust/save tooling consumers.
- explicitly excluded/unverified: Unity persistence/restore behavior.

No excluded consumer is represented as verified.

## Legacy, Fallback, And Migration Audit

- Strict current V3/no-migration behavior is aligned across source, canonical and tests.
- Automatic backup recovery is validated, observable recovery behavior and was rejected as fallback debt.
- Generator failure has no synthetic fallback map; validated content is required.
- Removing serialized progression fields, accepting V2 or repairing inconsistent saves is deferred migration/policy
  work rather than labeled behavior-preserving refactor.
- Matching numeric version values do not merge Core and host schema authority.

## Scoped Diff And Dirty Preservation

The following initial/final aggregate SHA-256 hashes are identical:

| Category | Initial | Final |
| --- | --- | --- |
| Core world/map/resources | `4a734882937f92eb7e564b25943aac5f00e442d542cd1aedaa77a684697a0da4` | same |
| shared `world.rs`/`behavior.rs`/run policy | `415b3c0c0853421f8283970852cbb1d78f8f0e0dab96c40b62c09af715818bea` | same |
| Core integration tests | `08ed65a65446a8564e59af80a69e2d16150f0a182e25b1638c5ca6a1cee8e892` | same |
| relevant non-Goal canonical | `11f848dcd4f1a8d911fe70a5af3506cfc00de6fa06a0c97a5dff702262870ade` | same |
| server root/persistence | `2c57784db98c82909032849c90e08bd7302528bbb7b9c44a70752c41f9624c42` | same |
| direct active-run actor/store seam | `3a7c535fabb9e963a0ab42d1ca486ad1f757c2ace47f175e020199647bace97d` | same |
| direct Phase production status | `ddec869d42b79d8ea5badc65b08b68bcdc9c998022cd48ec3390395c6b38c28a` | same |

Only audit-owned Master/Phase Goal artifacts changed. Cargo used ignored build artifacts under repository `target`;
production files and existing user dirty changes were not restored, formatted, moved, deleted or modified.

## Validation Results

- `game::map::`: 25 passed, 0 failed/ignored.
- `game::world::tests::facility`: 11 passed, 0 failed/ignored.
- `game::world::tests::map_flow`: 40 passed, 0 failed/ignored.
- `game::world::tests::node_sessions`: 18 passed, 0 failed/ignored.
- `game::world::tests::snapshots_and_start`: 23 passed, 0 failed/ignored.
- UUID manager: 2 passed, 0 failed/ignored.
- server `active_run` filter: 13 passed, 0 failed/ignored; main binary selected 0.
- Core `transaction` filter: 4 passed, 0 failed/ignored.

Total runner executions: 136 passed, 0 failed, 0 ignored/skipped; 135 distinct test names because one checkpoint
transaction test matched two filters. Filtered counts are nonmatching tests, not skips.

## Baseline Failures

- No test baseline failure was observed.
- Parallel Core runners waited for Cargo package/build locks and all completed normally.
- One source-reading `sed` used a repository-root server path from the `core_enhanced` workdir and returned
  not-found. The file had already been read fully from repository root and was read again there; no evidence or
  validation remained missing.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Validation evidence | Audit verdict |
| --- | --- | --- | --- | --- | --- |
| CRA-P02-C01 | correctness correction | caller-trusted current metadata → store-enforced high-water | payload generation/V3/backup/atomic write preserved | lower-incoming real-file tests required first | pass |
| CRA-P02-C02 | correctness correction | partial reference validation → context-aware identity/progression validation | valid V3 restore and fail-closed behavior preserved | corrupted current-V3 negative tests required first | pass |
| CRA-P02-C03 | behavior-preserving refactor | manual bidirectional fields → one private typed conversion | exact V3 shape/exclusions preserved | golden field parity required | pass |
| CRA-P02-C04 | behavior-preserving refactor | distributed public mutations → progression-owned transitions | navigation and serialized fields preserved | transition characterization required | pass |

- Fit: High
- Improvement class: immediate test-first correction recommended for C01/C02; follow-up refactor for C03/C04
- Reason: the audit connects actual gameplay and host authority, separates strict recovery from fallback, does not
  choose missing cross-player/save-migration policy, records missing safety nets and preserves dirty production.

## Primary Verdict

Complete

## Status Transition

- from: active
- to: complete
- Phase verdict: `Complete / High`
- next Phase activation: allowed after Parent Master registry update

## Remaining Risk And Required Action

- C01 and C02 should be implemented test-first under separate authorized correction Goals.
- C03 must preserve exact V3 JSON and explicit root/run exclusions.
- C04 may centralize mutation but may not remove the serialized availability field without migration approval.
- Cross-player singleton-run ownership and admin durability remain policy-deferred.
- Unity and other external consumers remain unverified/out-of-scope.

