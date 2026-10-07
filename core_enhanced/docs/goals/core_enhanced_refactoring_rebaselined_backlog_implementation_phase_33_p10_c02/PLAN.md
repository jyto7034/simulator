# Phase 33 CRA-P10-C02 Command-Result Projection Authority Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 33 of 44
Candidate / Work Type: CRA-P10-C02 / `behavior_preserving_refactor`
Direct Dependency: Phase 32 CRA-P10-C05 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Give each semantic Core `BehaviorResult` exactly one typed command-result projection authority for stable result
type, payload and transport classification, then make the current Core/server command-result path consume it.

This is an ownership refactor only. Every current result string, payload key/value/omission, direct-map versus
wrapped shape, nested completion, read-only preview behavior, accepted/update split, trailing snapshot decision,
request correlation and publication order must remain exact.

## User/Contract-Visible Problem

Original and post-Boss Phase-10 audits found that one semantic `BehaviorResult` expands through parallel domain
result representations and is then matched again to derive `result_type` and JSON payload. This repeats semantic
identity and fields inside Core, while the server consumes the derived contract. A valid change to one match can
silently drift from another.

Historical evidence also found an unconstructed roster result branch, workspace-test-only public helpers and an
uncovered `RunComplete` payload that serializes as the map object directly rather than `{ "map": ... }`. Those facts
are investigation inputs, not permission to remove public APIs or change the odd but frozen wire shape.

## Audit Finding ID

CRA-P10-C02. Original and post-Boss P10 audits classify it `still_valid`, priority Tier B, known subtotal `17/18`
and `all_variant_goldens_and_external_api_inventory_first`.

## Current Runtime And Live Data Evidence

Historical and final evidence identifies these prospective surfaces:

- public semantic `BehaviorResult` variants and any result convenience methods;
- `BehaviorCommandResultContract` and its current domain-specific result representations;
- stable `result_type` and JSON payload projection helpers;
- nested completion, run completion, research delivery, roster, preview and live-battle result families;
- the game_server conversion into command responses, optional side messages and trailing snapshots;
- server message serialization, actor response/side-message/snapshot ordering and reconnect behavior;
- workspace/public Rust consumers and accessible external wire/canonical consumers.

No new Phase-33-directed production source, test, data or canonical read/search occurred during initialization.
Historical P10 evidence and completed Phase-30~32 overlap are inputs only. Exact current types, variants, helper
visibility, construction paths, callers, serialized values and dirty attribution remain discovery pending after
sole activation and prospective fingerprinting.

## Source Of Truth And Ownership

### Current authority

- Core domain handlers own semantic command outcomes and return `BehaviorResult`.
- Historical evidence shows Core converts those outcomes through multiple result representations and separately
  derives type strings and JSON payloads.
- The server consumes the Core command-result contract and owns the transport envelope, request correlation,
  response/side-message/snapshot publication order and host persistence lifecycle.
- Core battle DTOs own battle setup/update contents; their server field de-duplication is Phase 34, not this Phase.

### Target authority

One typed Core projection derived from the semantic result emits the current stable result type, exact payload and
transport classification. Nested completion and battle/read-only classification consume the same authority rather
than reauthoring a parallel variant list.

The target may reuse, simplify or replace current internal result representations only after exhaustive old-output
goldens and public API inventory prove the smallest safe boundary. It must not move server envelope/order/persistence
ownership into Core or prebuild Phase-34 DTO composition.

### Producers and consumers

- Producer: current Core domain handlers and `GameCore` dispatch returning semantic results.
- Projection consumer: current Core command-result contract and its type/payload/transport accessors.
- Server consumers: command response mapping, battle side-message selection, trailing snapshot decision and actor
  publication.
- Public consumers: workspace Rust callers plus unknown standalone/deployed consumers covered by CRA-POL-006.
- External consumers: accessible Unity/server canonical readers of the current result envelope and payload shapes.

Exact symbols, counts, visibility and caller graph are discovery pending after activation.

## Invariants

- Every current semantic result retains its exact stable `result_type` string.
- Every payload retains exact keys, values, nullability, omission, nesting, collection order and enum casing.
- `RunComplete` remains the current direct-map payload; it is not wrapped in a new `map` key.
- Nested common/run/event/reward/research completion results retain their exact projection semantics.
- Read-only previews retain their exact response type/payload and trailing-snapshot/revision behavior.
- Live-battle success retains the current accepted-result versus side-message update/resync split.
- Server response, side-message and optional root-snapshot order remains exact.
- Request IDs, command IDs, accepted battle time/revision and persistence decisions remain exact.
- No result is collapsed, renamed, reclassified, converted to an error or silently omitted.
- No public Rust helper/type is removed or narrowed from negative workspace search alone.
- No command/result/battle DTO, RON, live data, V3/save/checkpoint, Boss-private boundary or Unity shape changes.
- Phase 30 route metadata, Phase 31 host overlay and Phase 32 error-code authority remain untouched and exact.
- Existing user-owned dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C02.
- Current disposition: `still_valid`; Tier B; `all_variant_goldens_and_external_api_inventory_first`.
- Priority axes: ownership `3`, correctness/atomicity `2`, lifecycle `3`, change amplification `3`, test fragility
  `3`, failure blast radius `3`, change frequency `unknown`; known subtotal `17/18`.
- Consolidation scope: parallel domain result representations, repeated result-identity matches and manual payload
  builders that add no independent invariant.
- Conditional removal scope: historically unconstructed roster branch and workspace-test-only public helpers, but
  only after current construction/caller/serialized/public/deployed consumer inventory. Negative search is not
  removal authorization.
- Retained distinct authorities: semantic domain outcomes, server envelopes/order/persistence, Core battle DTO
  content and external presentation.
- First safety net: every current semantic result variant, including nested completions and direct-map
  `RunComplete`, characterized through exact type/payload/transport/nesting goldens before production changes.
- Behavior-preservation boundary: any string, payload, shape, transport class, snapshot decision, timing/order,
  failure or public API difference stops the refactor.

## Approved Policy And Evidence

- CRA-POL-006 freezes current command/result tags, payloads and nesting, including direct-map `RunComplete`,
  top-level setup/update and nested resync.
- Public Rust symbols are not removed or narrowed without consumer inventory plus deprecation/versioned migration,
  or a new explicit breaking approval.
- Original P10-C02 evidence assigns result projection content to Core and leaves server envelope/publication
  ownership distinct. It requires all-variant goldens and explicit `RunComplete` coverage first.
- Post-Boss evidence confirms the multiple Core representations and missing parity remain. Private Boss scheduling
  creates no new public result carrier and must remain non-serde.
- Final item 33 follows Phase 32 only by the strict campaign gate and is the technical predecessor of Phase 34
  transport DTO de-duplication. It must expose a stable classification seam without implementing Phase 34 early.

## Open Policy Decisions

None at initialization. Current result strings, payloads, direct-map `RunComplete`, nesting, transport classes and
public APIs are frozen. Removing/narrowing a public symbol, wrapping `RunComplete`, changing a result tag/payload,
altering success/side-message/snapshot behavior or adding a Boss result is a new policy/migration decision and
triggers a stop.

## In Scope

- current-tree inventory of every semantic result, construction path, projection representation and direct caller;
- workspace/public/deployed consumer classification for result types and helpers;
- exhaustive old-production characterization of every result's exact type, payload, transport class and nesting;
- one typed Core command-result projection authority and removal/consolidation of only proven duplicate internals;
- exact server mapping, side-message, snapshot-decision, actor-order and serialized parity;
- direct-map `RunComplete`, nested completion, roster/research/preview/live-battle special-case coverage;
- affected Core/server debug/release tests, live-RON compatibility and static one-owner/legacy audit;
- nearest canonical correction only if current-tree evidence proves a statement stale;
- independent completion audit and exact dirty-tree preservation proof.

## Out Of Scope

- changing any result type string, payload field, nesting, nullability, omission or transport class;
- wrapping `RunComplete` or otherwise correcting/normalizing its current shape;
- changing command execution, domain mutation, error mapping, persistence or actor publication order;
- CRA-P10-C03 server battle DTO field de-duplication or any later cleanup Phase;
- adding a first public Boss result/event/checkpoint/server/Unity carrier;
- removing/narrowing public Rust API without the CRA-POL-006 inventory/migration gate;
- RON/content/balance, V3/checkpoint, persistence schema or Unity source changes;
- compatibility aliases, dual projections, fallback serializers, unknown/default types or future placeholders;
- unrelated warning, formatting, dependency, target or dirty-work cleanup.

## Implementation Scope

After activation and current-tree proof:

1. inventory every current `BehaviorResult`, construction site, helper, intermediate result type and projector;
2. trace result contract production through server response/side-message/snapshot publication and accessible
   consumers;
3. classify public helper/type consumers across workspace, standalone/deployed unknown and serialized boundaries;
4. add a nonzero exhaustive old-production golden table for every semantic result, nested completion and special
   transport path, with explicit direct-map `RunComplete` assertion;
5. prove current type/payload/transport/nesting parity or stop and classify any existing inconsistency;
6. introduce the smallest typed projection authority and switch only proven duplicate projection consumers to it;
7. remove only duplicate internal representations/helpers whose construction/API/consumer evidence permits it;
8. prove exact result, serde, side-message, trailing-snapshot, actor-order and no-behavior-change parity;
9. audit public visibility, legacy representations, fallbacks, aliases and Phase-34 placeholders;
10. update durable canonical text only for a demonstrated current-contract drift.

No target type name, module, visibility or retained representation is final until the active tree and consumer
inventory prove it.

## DTO, Data, Save And Migration

- DTO/wire migration: none. All result types, payloads, envelopes, nesting and ordering remain exact.
- Live RON/data migration: none. Result projection neither authors nor migrates content.
- Save/checkpoint migration: none. No V3, active-run or persistence representation changes.
- Public Rust migration: none approved. Private consolidation may proceed; public removal/narrowing stops unless a
  complete consumer inventory and approved deprecation/versioned migration already exist.
- Compatibility: no alias, legacy result table, dual projection, fallback serializer or version interval approved.

## Deployment And Compatibility

Core and game_server are validated atomically from the same repository state. Exact old/new serialized and actor
behavior proves current consumers require no coordinated rollout. Accessible external consumers/canonicals are read
when relevant. Unknown standalone/deployed Rust and Unity revisions remain an explicit risk controlled by exact
parity and additive/private consolidation, never by assuming absence.

## Focused Validation

Commands and counts are discovery pending. Required families are:

- one exhaustive all-variant semantic result table with a nonzero count and compile-visible inventory;
- exact `result_type`, payload `Value`, transport class, nesting and snapshot-decision assertion for every row;
- explicit direct-map `RunComplete`, nested completion, roster/research, preview and live-battle rows;
- server conversion and exact top-level command response/side-message outputs using real Core contracts;
- actor response → side-message → optional snapshot ordering and representative reconnect/persistence paths;
- public helper/API construction and consumer inventory with no zero-test or negative-search removal claim;
- static one-owner/duplicate-representation/legacy/fallback/placeholder inventory.

## Broad Validation

At minimum after affected-target discovery:

- affected Core behavior/result projection tests in debug and representative release;
- affected server state/message/handler/order tests in debug and representative release;
- Core and server all-target tests and workspace checks in debug/release for the cross-crate result seam;
- live RON integration as unchanged-contract evidence;
- format, candidate-scoped diff/static audit and normal Clippy diagnostic baselines;
- final source/canonical/consumer hashes and expanded dirty-status preservation check.

Exact runner-reported pass/fail/ignored counts must be recorded. Zero-test targets and compile-only success do not
count as behavior evidence.

## Omitted Validation

- Live WebSocket probing is provisionally omitted if exact real Core/server serde and actor-order tests close the
  same result values and ordering. Use a registered probe if transport behavior cannot otherwise close.
- Unity EditMode/PlayMode is provisionally omitted because no Unity/wire change is authorized; reconsider on any
  serialized difference or external source edit.
- Performance benchmarking is omitted because projection simplification is not a simulation hot-path algorithm.
- Repository-wide unrelated warning/format cleanup is always omitted.

## Completion Conditions

- Every semantic result, construction path, projection representation, helper and direct consumer is inventoried.
- Public Rust helper/type consumers are classified before any removal/narrowing decision.
- Old duplicated behavior is first characterized by a nonzero all-variant exact golden table.
- One typed Core authority owns result type, payload and transport classification.
- Proven duplicate result representations/manual projection matches are removed or reduced to distinct invariants.
- Every current result string, payload, omission, direct-map/nested shape and serialized envelope remains exact.
- Server side-message, trailing snapshot, actor publication, reconnect and persistence behavior remains exact.
- No public API removal, fallback/default/alias/dual projection, Boss wire or Phase-34 placeholder exists.
- DTO/RON/save/checkpoint/public/server/accessible-consumer compatibility remains exact under CRA-POL-006.
- Focused and broad debug/release/static validation is green with actual nonzero counts and classified failures.
- Existing dirty work and the protected approval marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P33-C01 | complete semantic-result/projection/caller graph identified | variants, constructors, helpers, intermediate types and server consumers | complete — audit pass |
| P33-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P33-C03 | public helper/type consumer gate closed | workspace/external/API inventory and removal decisions | complete — audit pass |
| P33-C04 | old duplicate behavior characterized first | all-variant exact golden table with nonzero count | complete — audit pass |
| P33-C05 | one typed result projection authority exists | source/static ownership and exhaustive-match proof | complete — audit pass |
| P33-C06 | every stable result type and payload is exact | old/new all-row `Value` parity and direct-map RunComplete | complete — audit pass |
| P33-C07 | transport/nesting/snapshot decisions remain exact | battle/read-only/nested classification assertions | complete — audit pass |
| P33-C08 | server envelope and actor order remain exact | mapping/serde/order/reconnect/persistence evidence | complete — audit pass |
| P33-C09 | DTO/data/save/public/consumer compatibility is exact | unchanged shapes/data/readers and CRA-POL-006 evidence | complete — audit pass |
| P33-C10 | no fallback/alias/dual projection/future placeholder remains | static legacy and Phase-34 inventory | complete — audit pass |
| P33-C11 | focused/broad debug/release/static validation is green | actual counts and failure classification | complete — audit pass |
| P33-C12 | independent completion audit is current | `Complete / High` review and downstream boundary | complete — audit pass |

## Activation Protocol

Before Phase-33-directed production inspection:

1. Master records Phase 32 `complete — Complete / High`, Phase 33 as the sole `active` Child and Phases 34~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-32 completion audit, original and
   post-Boss P10 evidence, Final catalog/order/test/gate/policy inputs and Master activation contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and prospective hashes/status for likely Core/server
   source, tests, public API and canonicals using path discovery only, without reading new Phase-33 production
   contents.
5. Record that evidence in the Child before detailed current-tree investigation or tests.

No Phase-33-directed source/test/data/canonical inspection or production mutation is authorized before all five
steps. Prior overlapping audit or Phase-32 source observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 34 if evidence requires changing any result type, payload, direct-map/nested shape,
transport class, request correlation, snapshot decision, publication order, persistence behavior or public API;
consumer inventory cannot support a proposed removal; a compatibility/deprecation interval is required without
approval; the target would require fallback/default/alias/dual projection; Phase 34 DTO work or a Boss wire is
absorbed; user-owned overlap cannot be safely separated; required tests select zero; the candidate no longer exists;
or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 535 rows.
- Canonical C-sort SHA-256:
  `4f8c208d7f99ca7c1001fc0113910de5b80df2ec85e68e083e88cc2266a380f9`.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved; the marker's approval remains limited to the
  historical Phase-27 one-path exception.
