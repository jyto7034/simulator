# Phase 37 CRA-P06-C04 Disconnected Event-Log Validator Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 37 of 44
Candidate / Work Type: CRA-P06-C04 / `cleanup`
Direct Linear Dependency: Phase 36 CRA-P05-C05 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Replace the disconnected event-log validator's false contract authority with assertions over logs actually produced
by the current `BattleCore`. First establish which version, sequence, parent/cause, spawn/reference, buff, death and
cadence predicates remain useful on representative produced logs; move equal or stronger positive and malformed-log
checks into durable producer-owned tests; only then remove or depublish the parallel validator, ghost configuration
and its six self-referential tests when CRA-POL-006 consumer evidence authorizes that public-surface change.

## User/Contract-Visible Problem

The Phase-6 audits found a public 14-file validation subsystem whose workspace references and six tests were
self-contained. Its default contiguous-sequence rule historically expected the first log entry at sequence zero,
while runtime logs begin at sequence one, and a movement-path consistency option was declared but unread. Because no
runtime, result, publication or integration consumer ran the validator against an actual generated log, its green
synthetic tests described an incompatible alternate event contract rather than protecting the production authority.

Deleting it immediately would also be wrong: several predicates encode potentially valuable version, relationship,
reference, buff/death and cadence assertions, and the module/public types may have unknown deployed Rust consumers.
Assertion transfer therefore precedes cleanup, and public narrowing requires more than a negative workspace search.

## Audit Finding ID

CRA-P06-C04 with test-cleanup row CRA-TC-05. The rebaselined catalog classifies it `still_valid`, `cleanup`, Tier D,
known subtotal `10/18`, and `transfer checks/external inventory` ready. Exact linear item 37 requires Phase 36
`Complete / High`, CRA-POL-006, a public consumer inventory and useful malformed-log predicates executing against
real produced logs before the false authority and tests are removed.

## Historical Evidence And Current-Proof Gate

Historical original/post-Boss evidence prospectively locates the candidate beneath
`core_enhanced/src/game/battle/validation/**`, with runtime production around the battle event-log/core seams and six
validator unit tests. It records:

- all workspace references to `EventLogValidator` and its related public types were inside the 14-file subsystem and
  its tests;
- runtime event logs began at sequence one while default validator contiguity began at zero;
- `require_movement_path_consistent` was never read;
- no real produced log passed through the validator;
- the six green tests used synthetic logs and configuration exceptions;
- the runtime event-log types, serialization, result/debug consumers and actual producer behavior were explicitly
  outside removal scope.

Those are locating hypotheses, not current deletion authority. After sole activation this Phase must refresh exact
file count, names, visibility, exports, callers, downstream/tool/deployment evidence, configuration reads, test
names and current producer behavior against the activation dirty tree before any production edit or removal claim.

## Source Of Truth And Ownership

### Current prospective authority

- `BattleCore` event recording owns production sequence allocation, time ordering, cause/parent correlation and
  emitted runtime log shape.
- Battle finalization/result aggregation, live deltas and debug tracing consume the committed production log without
  replaying gameplay rules.
- The public validation module historically owns a separate configurable acceptance contract but has no proved live
  producer/consumer connection.
- Public exports, Cargo/package metadata, repository/deployment/tool documentation and actual accessible consumers
  jointly determine whether removal or depublication is compatible.

### Target authority

Focused tests built from representative current `BattleCore` executions own event-log invariants. A valid generated
log must satisfy each retained predicate, and malformed cases must begin with that generated log and change one
relationship at a time so the test proves production relevance rather than a synthetic alternate schema. No
parallel public validator/config/self-test authority remains only when CRA-POL-006 evidence permits its removal or
depublication; otherwise the Phase records the external block and does not claim completion or activate Phase 38.

## Invariants

- Runtime event-log version, serialized shape, sequence allocation and nondecreasing production order remain exact.
- Cause/parent and command-correlation meaning remain exact; tests may assert the current contract but must not
  invent causal ordering from correlation-only fields.
- Spawned/reference identities, buff application/removal, cast/attack/death behavior and cadence remain gameplay-
  equivalent and deterministic.
- Battle results, live delta/resync, debug trace/export and compression/serialization consumers remain unchanged.
- Runtime event-log types and production record/finalization paths are never deleted as part of validator cleanup.
- A malformed-log assertion is transferred before the validator test/body that carried it is removed.
- Public Rust symbols, types, fields, methods, module exports and package targets are not narrowed from negative
  workspace search alone.
- No new shim, alias, deprecated duplicate, compatibility fallback or second validator is introduced merely to make
  deletion appear complete.
- No RON/content, DTO/wire, V3 save/checkpoint, server/Unity behavior or canonical contract changes.
- Phase 41 owns `ScenarioEvent.once`; Phase 42 owns event-log file-I/O/debug-boundary cleanup; neither is pulled
  forward.
- Existing user/campaign dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P06-C04.
- Work type: `cleanup`; `still_valid`; Tier D; assertion-transfer/external-inventory ready.
- Priority axes: ownership `2`, correctness/atomicity `1`, lifecycle `1`, change amplification `2`, test fragility
  `3`, failure blast radius `1`, change frequency `unknown`; known subtotal `10/18`.
- Prospective removal scope: the current equivalent of the historical 14-file validator harness, ghost/unread config
  and its six self-tests, only after current enumeration, assertion transfer and public-consumer authorization.
- Retained scope: runtime log types/serialization, `BattleCore` producer, result/live/debug consumers, production
  cadence and focused tests that exercise actual generated logs.
- First safety net: execute representative current battles, inventory each validator predicate against their real
  logs, then add positive generated-log and one-edge malformed-log assertions before removing any carrier.
- Behavior-preservation boundary: any runtime log/output/order/error change, loss of a useful assertion, API/package
  narrowing without migration evidence, or adjacent P06-C06/P06-C07 cleanup stops the implementation.

## Approved Policy And Evidence

- Original Phase-6 C04: transfer useful parent/buff/death/cadence and related invariants to real producer tests before
  removing or depublishing the disconnected harness.
- Original test audit: six synthetic self-tests are the only Phase-6 remove family and cannot survive as proof merely
  because they are green.
- Post-Boss C04: the disconnected public validator, ghost config and self-tests remained `still_valid`; the original
  external-consumer gate remained.
- CRA-TC-05: run useful sequence/reference/buff/death/cadence checks over real produced logs and preserve malformed-
  log assertions there before six self-tests and the harness disappear.
- Exact linear item 37: useful malformed-log predicates must execute against real produced logs before false
  authority/tests are removed.
- Phase 36: direct predecessor is independently `Complete / High` and created no Phase-37 seam or placeholder.
- CRA-POL-006: public Rust/package surfaces require a consumer inventory plus deprecation/versioned migration or new
  explicit breaking approval before narrowing; unknown deployed consumers remain an evidence gate.

## Open Policy Decisions

None. Master PLAN Phase 37 row 234 now records the user's explicit decision: external consumers do not exist and the
breaking cleanup should proceed as proposed. This resolves the CRA-POL-006 removal gate for
`game_core::game::battle::validation`, its public reexports/types, 14-file implementation and six self-tests. The
approval does not broaden scope to the runtime event log, the unused Cargo feature owned by Phase 44, adjacent
ScenarioEvent/file-I/O cleanup, or any DTO/data/save/server/Unity/canonical change.

## Current Runtime And Live Data Evidence

- Pre-cleanup discovery found exactly 14 Rust files and 2,926 lines. Four files (`buffs.rs`, `deaths.rs`,
  `spawns.rs`, `validator.rs`) were already modified before Phase 37; the other ten were clean. Explicit approval
  now authorizes the whole surface, so all 14 files and their six self-tests are deleted together.
- `battle/mod.rs` no longer exports `validation`; the former public reexports `SkillFocusTimeProvider`,
  `EventLogExpectedCounts`, `EventLogValidatorConfig`, `EventLogViolation`, `EventLogViolationKind` and
  `EventLogValidator` have no current source path or reference. The removal is the approved breaking change.
- The unread `require_movement_path_consistent` field disappeared with the harness. The unused manifest feature
  `event_log_checks` remains exact because package/feature cleanup belongs to Phase 44 and is not pulled forward.
- `game_core` is a library package, `publish` is not disabled, `game_server` consumes it by path, and the repository
  has a public GitHub origin. Workspace evidence alone could not exclude standalone/deployed Rust consumers, but
  Master PLAN line 234 now records the user's authoritative inventory decision that no external consumer exists and
  explicitly authorizes proceeding. The CRA-POL-006 removal gate is resolved by that breaking approval.
- A real delayed-wave `BattleCore` execution produced 17 entries from `BattleStart` through `BattleEnd`, numbered
  1 through 17. The default public validator rejected that otherwise-valid log only with 17
  `NonContiguousSeq` violations because it compares each sequence to a zero-based array index.
- Useful version, start/end, one-based sequence, nondecreasing time, prior-parent, spawn/reference, HP-delta,
  unique-death/dead-unit, attack-kind and auto-attack-cadence assertions now execute over that actual log. Each
  retained malformed case is derived by changing one edge of that generated control.
- Current producer-owned buff tests cover unknown-ID no-op behavior, `RefreshDuration` reapplication and exact
  natural expiry, with malformed unknown-ID/expiry negatives. Existing real hard-CC overlap coverage remains.
  Current producer-owned cast tests now prove valid and malformed manual-interrupt and active-cancel references.
- The stale zero-based sequence expectation and unread movement option were not transferred. A generic cause-parent
  requirement was not invented for manual interrupts: current production records a root/system cause while the
  dedicated `interrupted_cast_seq` field carries the correlation.

## In Scope

- current file/export/type/config/test/caller inventory for the full validator subsystem;
- current package/publication/deployment/tool/consumer evidence sufficient to classify public removal feasibility;
- representative actual `BattleCore` logs covering battle start/end and useful spawn/reference, attack/cast,
  buff/death and cadence relationships present in current behavior;
- predicate-by-predicate execution of the current validator against those logs to distinguish useful rules, stale
  incompatibilities and unread configuration;
- test-first positive assertions on generated logs plus single-mutation malformed-log negatives for retained rules;
- removal or depublication only after assertion parity and CRA-POL-006 authorization;
- exact test-discovery delta, legacy/ignored/placeholder searches, focused and broad validation;
- separate completion audit or explicit external-block disposition.

## Out Of Scope

- changing runtime event version, event variants/fields, sequence origin, ordering or cause/parent semantics;
- changing combat mechanics, cadence, buff/death handling, result aggregation, live delta/resync or debug trace;
- retaining or introducing a general validator as a new runtime/publication admission path;
- ScenarioEvent `once` cleanup (Phase 41) or battle-log file-I/O/helper cleanup (Phase 42);
- RON/data, DTO/wire, save/checkpoint, server/Unity, canonical gameplay or package-target migration;
- broad event-log refactoring unrelated to the disconnected validation authority;
- unrelated formatting, lint or dirty-worktree cleanup.

## Implementation Scope

After sole activation and prospective fingerprints:

1. refresh the exact validation directory/module/export/type/config/test graph and all accessible consumers;
2. establish old/current nonzero baselines for the six historical self-tests and current producer-owned battle tests;
3. generate representative logs only through current production battle setup/execution helpers;
4. evaluate every validator predicate against those generated logs and record pass/fail/stale/unread disposition;
5. add durable positive checks and one-edge malformed copies for each retained useful invariant, and prove them on
   old/current production before removing any test or validator carrier;
6. classify each prospective symbol/file as private-removable, public-removable with affirmative consumer/migration
   evidence, or externally blocked under CRA-POL-006;
7. apply the smallest authorized cleanup with no runtime/log/serialization change and rerun exact discovery;
8. run focused/broad debug/release/static validation, verify prospective hashes and protected dirty state;
9. perform an independent completion audit; transition linearly only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none; event-log transport and resync shape remain byte/structure equivalent.
- Live RON/data migration: none; representative battles use current accepted data/builders without authored changes.
- Save/checkpoint migration: none.
- Runtime serialization migration: none; current event-log version/tags/fields/sequence origin remain exact.
- Public Rust migration: none preapproved beyond a removal proven compatible by affirmative current evidence. If a
  deprecation/versioned/breaking path is required, stop for separate approval.
- Unity/external migration: none.

## Deployment And Compatibility

Core and accessible in-workspace consumers are validated from one repository state. Package metadata, public docs,
fixtures/tools and accessible deployment records must be inventoried affirmatively; workspace symbol absence alone
does not prove external absence. No coordinated rollout or external repository edit is authorized. If public
validator removal is essential to the candidate but its consumers cannot be resolved, record `externally_blocked`,
leave Phase 38 pending and request the missing consumer/migration decision through the Master rather than adding a
shim or silently weakening the goal.

## Focused Validation

Required after current discovery:

- exact module/file/export/config/method/test and caller/consumer inventories;
- the current six-test validator baseline with actual nonzero counts;
- representative generated-log controls covering the production event families needed by retained predicates;
- positive version/sequence/order/parent-cause/spawn-reference/buff/death/cadence assertions over those logs;
- one-edge malformed-log negatives derived from generated controls, with exact failure meaning;
- proof that stale/default-incompatible or unread checks do not become a new production authority;
- exact pre/post test-name and file/reference deltas for any authorized removal.

Exact commands, selected tests, pass/fail/ignored counts and old/current results are recorded beginning at E04.
Zero-selected filters, compile-only success and purely synthetic fixture tests do not count as transfer proof.

## Broad Validation

At minimum after implementation:

- complete affected Core battle/event-log test families in debug and representative release;
- `cargo test -p game_core --all-targets` in debug and release with exact per-target counts;
- workspace all-target checks in debug/release;
- `cargo fmt --all -- --check`, full `git diff --check` and normal Core Clippy diagnostics;
- live-RON integration if the chosen representative producer path consumes authored battle data;
- final expanded status/path hashes, prospective non-target hashes, public/legacy/placeholder searches and protected-
  marker verification.

## Omitted Validation

- game_server and Unity runtime/build are provisionally omitted because no DTO/wire/runtime result change is
  approved; reconsider if current consumer discovery finds compilation or fixture coupling.
- WebSocket/live server probes are omitted unless a real transport path is required to prove unchanged log delivery.
- Save/checkpoint and performance validation are omitted unless current discovery finds a changed carrier/hot path.
- Canonical edits are omitted if and only if runtime/public contract remains exact and only false test authority is
  removed. A public API migration would require policy first and would reassess canonical/deployment evidence.
- Repository-wide final Stage-D validation remains a later Master boundary.

## Completion Conditions

- The current validator's exact files, exports, public symbols, config reads, tests and consumers are identified.
- Representative actual `BattleCore` logs expose the current production sequence origin and all retained useful
  relationships.
- Every useful invariant has equal or stronger producer-owned positive and malformed-log coverage before its old
  carrier or self-test is removed.
- The disconnected validator/config/self-test authority is removed or depublished only with affirmative
  CRA-POL-006 consumer/migration evidence; otherwise the Phase is explicitly externally blocked and not complete.
- Runtime log types/version/serialization, sequence/order/correlation, gameplay, result/live/debug consumers and
  errors remain exact.
- No adjacent ScenarioEvent/file-I/O cleanup, compatibility shim, ignored test, fallback or placeholder is added.
- No RON/data, DTO/wire, save/checkpoint, server/Unity, canonical or package-target contract changes.
- Focused/broad debug/release/static validation is green with actual nonzero counts.
- Dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding before Phase 38 becomes ready.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P37-C01 | current validator/export/config/test graph is exact | file, symbol, visibility, read and test inventory | complete — E04 |
| P37-C02 | current consumers and removal gate are exact | workspace/package/tool/deployment inventory plus CRA-POL-006 disposition | complete — explicit user approval recorded in Master row 234; E07 |
| P37-C03 | dirty targets and concurrent work are preserved | pre/post status, scoped diffs, hashes and protected marker | complete — E08 |
| P37-C04 | representative real produced logs are characterized | nonzero current `BattleCore` controls and event-family inventory | complete — E04 |
| P37-C05 | useful predicates are classified on produced logs | per-predicate pass/fail/stale/unread matrix | complete — E04 |
| P37-C06 | retained positive assertions move first | named producer-owned tests over generated logs | complete through current authorized scope — E05 |
| P37-C07 | retained malformed assertions move first | one-edge mutations of generated logs and exact failures | complete through current authorized scope — E05 |
| P37-C08 | only authorized false authority is removed/depublished | exact file/symbol/test delta and no parallel validator | complete — one export + 14 files + six tests removed; E08 |
| P37-C09 | runtime event-log/gameplay/result behavior is exact | version/seq/order/correlation/serialization and consumer parity | complete — producer 108 and Core debug/release all-target green; E08 |
| P37-C10 | DTO/data/save/public/external contracts are unchanged or explicitly gated | prospective hashes and compatibility evidence | complete — approved public break only; runtime/DTO/data/save/manifests exact; E08 |
| P37-C11 | focused/broad/static validation is green | actual counts, profiles and failure classification | complete — 1,068 debug/release, workspace checks, format/diff/Clippy green; E08 |
| P37-C12 | independent audit and next boundary are current | `Complete / High` review; Phase 38 absent until exit | complete — 240-line audit at `6cc9a92d...` |

## Activation Protocol

Before Phase-37-directed production content, caller, validator-predicate, test or consumer inspection:

1. Master records Phase 36 `complete — Complete / High`, Phase 37 as the sole `ready` Child and Phases 38~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 37 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-36 completion audit, original/post-Boss
   P06-C04 evidence, Final catalog/order/test/gate/policy inputs and the full Master contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and filename-only/path-known prospective hashes/status
   for the historical validation directory, producer/test/event-log consumers, manifests, data/canonicals and
   dependency inputs without reading their current content.
5. Record activation evidence before current-tree content, caller, test or external-consumer investigation.

Historical audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 38 if a useful predicate lacks a producer-owned replacement; a malformed case is
synthetic-only rather than derived from a real generated control; runtime log behavior/version/shape/order changes;
consumer evidence is insufficient for required public narrowing; a deprecation/versioned/breaking migration is
needed without approval; dirty overlap cannot be separated; an adjacent P06-C06/P06-C07, data, DTO, save, server,
Unity, canonical or package migration is required; any required test fails or selects zero; the protected marker
changes; or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 553 rows.
- Canonical C-sort SHA-256:
  `729b1a8b113f67de2fc0662ce54481fedbf39b5316c3b2f0460e53a9d918e086`.
- Phase 37 and Phase 38 paths are absent; no implementation Phase is active.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- These three new Child files are the only expected status-set additions before activation evidence.
- All existing rows, including completed Phase-36 source/test/doc changes, are user/campaign-owned and preserved.
