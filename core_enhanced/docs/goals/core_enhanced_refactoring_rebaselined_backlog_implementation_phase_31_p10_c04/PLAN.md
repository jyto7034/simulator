# Phase 31 CRA-P10-C04 Typed Host Snapshot Overlay Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 31 of 44
Candidate / Work Type: CRA-P10-C04 / `behavior_preserving_refactor`
Direct Dependency: Phase 30 CRA-P10-C01 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Replace server-side mutation of an already serialized Core player snapshot with one typed host snapshot overlay and
typed capability merge. Core continues to own gameplay snapshot fields and gameplay capabilities; the server
continues to own active-run and encounter-compendium persistence status plus the four host recovery capabilities.

The final server snapshot must be serialized once from typed state while producing exactly the current flattened
root object. This is an ownership refactor only: action order, duplicate absence, fatal-compendium replacement,
field presence/nullability, persistence behavior and every command/result/wire shape remain exact.

## User/Contract-Visible Problem

Historical and post-Boss audits found that the server receives a typed Core snapshot, serializes it to
`serde_json::Value`, indexes root keys, pushes and clears capabilities, and appends host persistence objects. This
works today but creates a second untyped construction authority where a typo, duplicate action, reordered recovery
command or stale fatal-recovery branch can silently drift from the typed Core/server contract.

Clients consume the flattened combined root, so the duplication cannot be removed by nesting or deleting host
fields. The problem is the untyped merge mechanism and future change amplification, not the current visible shape.

## Audit Finding ID

CRA-P10-C04. Original Phase 10 and post-Boss Phase 10 audits both classify it `still_valid` and
`table_tests_then_implementation`.

## Current Runtime And Live Data Evidence

Historical/current predecessor evidence identifies these prospective surfaces:

- Core typed run snapshot and typed `action_capabilities` producer;
- server `PlayerStateSnapshotDto` construction and `build_state_snapshot` host mutation path;
- active-run load states: unavailable store, missing, loaded current/backup and corrupt/reset-required;
- encounter-compendium persistence states: clean, recovered/dirty and fatal/reset-required;
- NotStarted, ordinary non-battle and active-battle capability behavior;
- four typed host recovery identities/scopes from completed Phase 30;
- server actor/persistence snapshot tests and nearest Core/server/Unity canonicals.

No new Phase-31-directed source, test, data or canonical search/read occurred during initialization. Phase 30's host
lifecycle audit necessarily read overlapping server code, but that evidence is not substituted for this Phase's
post-activation current-tree inventory, prospective fingerprints or baseline.

## Source Of Truth And Ownership

### Current authority

- Core owns all gameplay snapshot fields, root gameplay capability order and state-dependent action availability.
- The server owns active-run persistence status, encounter-compendium persistence status and availability of the
  four host recovery operations.
- Clients consume one flattened root object containing both authorities.
- The current merge is operationally owned by server snapshot construction but implemented through untyped
  post-serialization `Value` indexing/mutation.

### Target authority

One typed server-owned host overlay/merge owns:

- typed active-run continuation/reset status;
- typed encounter-compendium persistence status;
- exact host capability additions and fatal-recovery replacement semantics;
- the flattened composition boundary before one final serialization.

Core's typed snapshot/capabilities remain input authority. Phase 30's typed route/scope metadata identifies host
commands but does not decide persistence availability. The server may merge and replace capabilities only from its
own typed persistence state; it must not infer or reconstruct gameplay capabilities.

### Producers and consumers

- Core producer: typed run snapshot and gameplay capability vector.
- Server producers: active-run store/load outcome, compendium persistence state and compressed result-log
  attachment, if still part of the current snapshot wrapper.
- Server composition consumer: player actor attach, command trailing snapshot and other current snapshot builders.
- Contract consumers: current server message serialization, accessible Unity canonical/source and unknown deployed
  Rust/Unity consumers constrained by CRA-POL-006.

Exact current types, call sites, wrapper ownership and consumers are discovery pending after sole activation.

## Invariants

- The final JSON remains one flattened root with every Core field preserved exactly.
- Host fields retain exact key names, object shapes, field order significance where observable, presence/absence,
  nullability and values.
- Core gameplay capabilities retain their exact order outside battle and remain absent/empty in the same contexts.
- Host recovery capabilities retain exact order, scope, enabled/reason representation and state predicates.
- No action appears twice after Core/host composition.
- Fatal compendium recovery retains exact replacement semantics: unrelated gameplay and active-run recovery actions
  are removed while the currently permitted compendium recovery decisions remain in exact order.
- Clean/dirty/recovered/fatal compendium meaning and missing/loaded/backup/corrupt active-run meaning remain exact.
- Snapshot construction remains read-only with respect to Core gameplay and persistence stores.
- Attach, command response, side-message and trailing snapshot ordering remain exact.
- No command/result/error DTO, RON, V3/save/checkpoint, public Boss, Unity UX or gameplay policy change.
- Phase 30 route/scope ownership remains stable; this Phase does not add or reinterpret commands.
- Existing user-owned dirty work and the protected Phase 27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C04.
- Current disposition: `still_valid`; Tier B; `table_tests_then_implementation`.
- Priority axes: ownership `3`, correctness/atomicity `2`, lifecycle `3`, change amplification `3`, test fragility
  `3`, failure blast radius `3`, change frequency `unknown`; known subtotal `17/18`.
- Consolidation scope: untyped post-serialization root mutation, ad hoc host capability push/clear logic and repeated
  host field construction that can become one typed merge.
- Retained distinct authorities: Core gameplay snapshot/capabilities, active-run and compendium stores/state
  machines, server actor lifecycle/publication and Phase 30 route metadata.
- First safety net: game-state × active-run × compendium overlay table with exact full JSON/capability sequence,
  duplicate absence and fatal replacement, passing on old untyped production before extraction.
- Behavior-preservation boundary: any difference in flattened shape, key presence/nullability/value, action
  membership/order/reason, fatal replacement, persistence access or transport ordering stops the refactor.

## Approved Policy And Evidence

- CRA-POL-006 freezes current command/result tags, payloads, nesting, V3/RON and public/API compatibility. Unknown
  deployed consumers prohibit shape inference from a negative workspace search.
- Original P10-C04 evidence requires a typed host snapshot overlay/capability merge before one serialization, while
  preserving flattened Core root keys, battle-root behavior, active-run metadata, compendium status, exact recovery
  order and no duplicates.
- Post-Boss P10 reaudit confirms the `Value` indexing/push/clear implementation remains and assigns an exact
  active-run × compendium × game-state table plus fatal replacement as the first proof.
- Final exact order places CRA-P10-C04 at item 31 after the Phase 30 route descriptor and before error/result/
  transport consolidation.

## Open Policy Decisions

None at initialization. Exact current output is frozen. A proposal to nest host fields, rename/change nullability,
change a persistence state meaning, deduplicate with a new precedence rule, reorder actions, move persistence into
Core or alter transport timing is a new policy/scope decision and triggers a stop.

## In Scope

- current-tree inventory of typed Core snapshot, server wrapper, all host field/capability mutations and callers;
- exhaustive characterization of the complete overlay state matrix on old production;
- one typed server-owned host status/overlay and typed capability composition authority;
- removal of in-scope `value["..."]` host mutation and ad hoc action push/clear paths;
- exact serde parity for flattened root, host objects and capability order/duplicates;
- current actor/persistence and Core/server broad debug/release validation;
- nearest canonical correction only if it is stale relative to the implemented current contract;
- separate completion audit and dirty-tree preservation proof.

## Out Of Scope

- changing Core gameplay snapshot authority or rebuilding its fields in the server;
- changing active-run/compendium persistence semantics, file formats, rollback or notification policy;
- adding/removing/renaming/reinterpreting any command, capability, result, error or host status field;
- CRA-P10-C05 error code authority, CRA-P10-C02 result projection, CRA-P10-C03 battle transport DTO reuse or any
  cleanup Phase;
- top-level setup/update or nested resync migration, RunComplete changes or first public Boss wire;
- RON/content/balance, V3/checkpoint, Unity UX or client store changes;
- compatibility aliases, dual schemas, fallback JSON construction or future-Phase placeholders;
- unrelated warning, format, dependency, target or dirty-work cleanup.

## Implementation Scope

After activation and current-tree proof:

1. inventory the exact Core/server producer, wrapper, serialized root and all snapshot call sites;
2. enumerate every relevant game-state × active-run × compendium combination and capture exact old JSON;
3. add the nonzero table/golden safety net before production extraction;
4. design the smallest typed overlay whose serde flattening/presence semantics reproduce the old root exactly;
5. derive host capabilities from typed persistence state and Phase 30 identities/scopes with one explicit merge/
   fatal-replacement authority;
6. switch all current snapshot builders, remove only the proven untyped mutation/push/clear paths and preserve actor
   ordering;
7. prove exact JSON parity, no duplicate action and no compatibility/fallback/next-Phase prebuild;
8. update a durable canonical only for a demonstrated stale current description.

No field representation, `Option` policy, flattening strategy or module location is fixed until the activated tree
and old-output table prove the exact serialization boundary.

## DTO, Data, Save And Migration

- DTO/wire migration: none. The typed server composition must serialize to the exact existing flattened JSON.
- Live RON/data migration: none. Host persistence status is runtime/store-derived, not authored content.
- Save/checkpoint migration: none. Active-run and Core V3 payload/envelope readers/writers remain exact.
- Public Rust migration: none. Prefer server-private typed overlay/status types; do not narrow existing public Core or
  server message symbols.
- Compatibility: no temporary alias, dual serializer, old/new field route or fallback `Value` builder is approved.

## Deployment And Compatibility

Core and game_server are validated atomically from the same repository state. Exact old/new JSON must prove that
existing server and client consumers require no rollout coordination. Accessible external Unity evidence is
required for the unchanged flattened contract; unknown deployed consumers remain a risk closed only by parity, not
by assuming absence. Any wire difference requires policy, version, rollout and rollback approval.

## Focused Validation

Commands and exact current counts are discovery pending. Required families are:

- a complete game-state × active-run × compendium overlay characterization table on old production;
- exact ordered capability vectors and duplicate-key/action absence for every table row;
- exact full flattened JSON or equivalent old/new serialized value parity, including field absence/nullability;
- fatal compendium replacement and recovery-order tests;
- active-run missing/loaded/backup/corrupt and compendium clean/recovered/dirty/fatal real actor/store paths;
- battle/root capability and Core-root preservation tests;
- attach/command trailing-snapshot and publication-order tests;
- static one-owner/caller/legacy/fallback/placeholder inventory with nonzero test-selection proof.

## Broad Validation

At minimum after affected-target discovery:

- affected server actor/message/persistence tests in debug and representative release;
- Core snapshot/capability suites when typed producer/consumer boundaries are touched;
- Core and server all-target tests/checks in debug/release for any cross-package type change;
- live RON integration as compatibility evidence even though no data change is expected;
- Core/server format, candidate-scoped diff/static audit and Clippy as a diagnostic baseline gate;
- final source/canonical/consumer hashes and expanded dirty-status preservation check.

Exact commands and runner-reported pass/fail/ignored counts must be recorded. Zero-test targets and compile-only
success do not count as behavior evidence.

## Omitted Validation

- Live WebSocket probing is provisionally omitted only if real typed actor tests prove the same snapshot values and
  response/side-message/trailing-snapshot order. Use the registered probe if transport order cannot otherwise close.
- Unity Edit/PlayMode is provisionally omitted because no Unity/wire change is authorized; reconsider on any
  serialized parity difference or external source edit.
- Performance benchmarking is omitted because snapshot composition is not a hot simulation algorithm; reconsider
  if activated evidence shows an unbounded or repeated serialization regression.
- Repository-wide unrelated warning/format cleanup is always omitted.

## Completion Conditions

- Every Core/server snapshot producer, overlay mutation and consumer is inventoried.
- The old untyped behavior is first characterized by a nonzero complete state table.
- One typed host overlay owns active-run/compendium host fields and capability composition before one serialization.
- Core remains sole gameplay snapshot/capability authority and persistence remains server-owned.
- Flattened JSON, host field presence/nullability/value and full Core root are exact.
- Capability membership/order/reasons are exact, contain no duplicates and fatal replacement is exact.
- All in-scope `Value` key mutation and ad hoc push/clear routes are removed with no fallback/dual serializer.
- DTO/RON/save/checkpoint/public/server/accessible-consumer compatibility remains exact under CRA-POL-006.
- Focused and broad debug/release/static validation is green with actual counts and classified failures.
- Existing dirty work and protected approval marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.
- No Phase 32 error-code work or other future placeholder is introduced.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P31-C01 | complete snapshot/overlay/caller graph identified | typed Core producer, server wrapper, persistence state and consumer inventory | complete — audit pass |
| P31-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P31-C03 | old overlay behavior characterized first | complete state matrix with exact serialized values and nonzero counts | complete — audit pass |
| P31-C04 | one typed host overlay/composition authority exists | source/static ownership and visibility audit | complete — audit pass |
| P31-C05 | full flattened root and host field shape are exact | old/new JSON parity, presence/nullability and Core-root preservation | complete — audit pass |
| P31-C06 | capability order, duplicate absence and state predicates are exact | complete ordered action table and duplicate checks | complete — audit pass |
| P31-C07 | fatal replacement and persistence lifecycles are exact | corrupt/fatal/retry/reset/backup actor/store tests | complete — audit pass |
| P31-C08 | DTO/data/save/server/Unity compatibility is exact | unchanged shapes/data/readers and accessible consumer/canonical evidence | complete — audit pass |
| P31-C09 | legacy JSON mutation/fallback/placeholder routes are absent | static key-index/push/clear/caller/placeholder audit | complete — audit pass |
| P31-C10 | focused/broad debug/release/static validation is green | actual nonzero counts and failure classification | complete — audit pass |
| P31-C11 | downstream boundary and independent audit are current | no Phase 32 prebuild and `Complete / High` review | complete — audit pass |

## Activation Protocol

Before Phase-31-directed production inspection:

1. Master records Phase 30 `complete — Complete / High`, Phase 31 as the sole `active` Child and Phases 32~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase 30 completion audit, original and
   post-Boss Phase 10 evidence, Final catalog/order/test/gate/policy inputs and Master activation contract.
4. Capture HEAD, expanded status count/normalized hash and prospective hashes/status for likely Core/server source,
   tests and canonicals using path discovery only, without reading new Phase-31 production content.
5. Record that evidence in the Child before detailed current-tree investigation or tests.

No Phase-31-directed source/test/data/canonical inspection or production mutation is authorized before all five
steps. Prior overlapping Phase 30 observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 32 if evidence requires changing any flattened key, presence/nullability/value,
capability membership/order/reason, fatal replacement, persistence lifecycle, message timing, DTO/RON/save/wire/
public shape or CRA-POL-006 compatibility; moving gameplay authority to server or persistence authority to Core;
adding a fallback/dual serializer/alias; absorbing P10-C02/C03/C05 or cleanup work; user-owned overlap cannot be
separated; required tests select zero; the candidate no longer exists; or the independent audit is not
`Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status: 527 rows.
- Normalized C-sort SHA-256:
  `50611a48e9742620b40f47951efb6bb213b77c69b0fe661133603f86ff025d21`.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved; the marker's approval remains limited to the
  historical Phase 27 one-path exception.
