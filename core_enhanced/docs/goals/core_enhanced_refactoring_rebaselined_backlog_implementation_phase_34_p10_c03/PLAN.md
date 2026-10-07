# Phase 34 CRA-P10-C03 Core Battle DTO Transport Reuse Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 34 of 44
Candidate / Work Type: CRA-P10-C03 / `behavior_preserving_refactor`
Direct Dependencies: Phase 33 CRA-P10-C02 and Phase 18 CRA-P09-C04, both `Complete / High`
Direct Policies: CRA-POL-005, CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Make game_server transport reuse the complete authoritative Core battle setup/update DTOs without re-declaring or
destructuring each Core field, while preserving the exact current top-level setup/update JSON, nested resync JSON,
message selection and actor/reconnect publication order.

This is an ownership refactor only. It must not nest an existing top-level setup/update body under a new field,
change any discriminator or field, alter preview union content, expose private Boss runtime state or move server
envelope/order authority into Core.

## User/Contract-Visible Problem

Original and post-Boss Phase-10 audits found that Core DTOs already own complete tagged battle setup and update
content, but server message variants repeat their fields and constructors destructure the Core values field by
field. Every Core field addition therefore requires a synchronized server declaration, constructor and fixture
update even when the server adds no invariant.

The current transport topology is deliberate: setup and update fields are top-level, resync wraps the current update,
and actor batches have fixed order. Reusing the Core DTO must remove change amplification without changing that
wire shape. Phase 33 now supplies the required typed result-transport classification seam; Phase 18 already fixed
the public effect preview to the approved deterministic union.

## Audit Finding ID

CRA-P10-C03. Original and post-Boss P10 audits classify it `still_valid`, `behavior_preserving_refactor`, priority
Tier B, known subtotal `14/18` and `exact_serde_parity_tests_first`. The ordinary transport refactor is current. Any
first public Boss-wire extension remains deferred/conditional and is not part of this Phase.

## Current Runtime And Live Data Evidence

Historical and completed-dependency evidence identifies these prospective surfaces:

- authoritative Core `LiveBattleSetupSnapshotDto` and `LiveBattleUpdateDto` definitions and serde behavior;
- server `PlayerGameServerMessage` setup/update/resync variants and constructors;
- the Phase-33 typed battle result projection and server command-result mapper;
- actor command, tick, reconnect, resync and final-snapshot publication paths;
- exact server serde fixtures/tests and any generated accessible Unity fixture comparison;
- local and external battle-transport canonicals and Unity readers;
- public preview cells already governed by CRA-POL-005.

No new Phase-34-directed production source, test, data, canonical or external-consumer read/search occurred during
initialization. Historical P10 evidence, completed Phase-18/33 evidence and observations made for earlier Phases are
inputs only. Exact current field sets, serde attributes, wrapper layout, constructors, consumers, fixtures and dirty
attribution remain discovery pending after sole activation and prospective fingerprinting.

## Source Of Truth And Ownership

### Current authority

- Core battle setup/update DTOs own battle UUID, timing/revision, events, checkpoint, capabilities, eligibility,
  visual novel, range previews and other public battle content.
- Historical evidence shows server message variants repeat those Core fields and constructors move them one by one.
- Server owns which top-level message is selected, command acknowledgement, setup/update/resync wrapping,
  request correlation and response/side-message/snapshot ordering.
- Unity consumes the public JSON and applies setup/update/resync through distinct transport paths.

### Target authority

Core DTOs remain the sole complete content/schema authority. Server transport holds the complete typed Core DTO once
and composes only the genuinely server-owned top-level envelope or resync wrapper needed to retain exact JSON.

The target Rust representation is not predetermined. Flattening, transparent/newtype or a small serialization
adapter is acceptable only if old/new exact `serde_json::Value` proof shows the complete current object remains
top-level and no second field list exists. A custom serializer that re-enumerates fields is not consolidation.

### Producers and consumers

- Producers: Core live-battle setup/update builders and Phase-33 `BehaviorCommandResultProjection::BattleUpdate`.
- Server consumers: message constructors, command mapper, tick, authority recovery/resync, reconnect and session.
- External consumers: accessible Unity message parser/store/battle runtime, server/Core fixture outputs and durable
  canonicals; deployed revisions remain unknown.
- Distinct owners: server envelope/order/persistence and private Core Boss scheduling remain outside content DTO
  ownership.

Exact symbols, visibility, fields, tests, fixture locations and consumer graph are discovery pending after activation.

## Invariants

- `battle_setup_snapshot`, `battle_update` and `battle_resync` top-level discriminators remain exact.
- Setup and update fields remain top-level at their current paths; no new `setup`, `update` or `payload` nesting is
  introduced except the already-current resync `update` wrapper.
- Every current field, value, nullability, omission rule, enum casing, collection order and numeric representation
  remains exact.
- Battle UUID, setup identity, server time, battle revision, event range/checkpoint, visual novel, capabilities,
  eligibility, preview union cells and all nested DTO values remain exact.
- CRA-POL-005's deterministic one-collection effect-preview union remains exact; no per-step representation appears.
- `CommandAccepted` response contents and response-before-setup/update or response-before-resync order remain exact.
- Command battle mutation sends no trailing root snapshot; final tick and reconnect orders remain exact.
- Stale/missing authority rejection still sends error then current resync with no mutation.
- Server remains the setup/update/resync selector and publication-order owner.
- Private Boss runtime/wake/session state remains non-serde and absent from public setup/update/checkpoint/event data.
- No RON, save/checkpoint V3, persistence, semantic battle mutation or Unity source shape changes.
- Existing user-owned dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C03.
- Current disposition: `still_valid`; Tier B; `exact_serde_parity_tests_first`; ordinary refactor current and
  Boss-specific expansion deferred/conditional.
- Priority axes: ownership `2`, correctness/atomicity `1`, lifecycle `2`, change amplification `3`, test fragility
  `3`, failure blast radius `3`, change frequency `unknown`; known subtotal `14/18`.
- Consolidation scope: duplicate server setup/update field declarations, field-by-field destructuring and only
  adapters that add no server-owned invariant.
- Retained distinct authorities: Core DTO construction, server envelope/message selection, resync wrapping,
  request correlation, actor ordering, reconnect lifecycle and Unity presentation.
- First safety net: exact old-production `serde_json::Value` for representative complete setup, update and nested
  resync values, including rich optional/nested fields, before production changes.
- Behavior-preservation boundary: any JSON, order, selection, lifecycle, public field, Boss visibility or consumer
  difference stops the refactor.

## Approved Policy And Evidence

- CRA-POL-005 defines public effect preview as the deterministic union of step footprints using the existing single
  collection DTO. Phase 18 implemented and audited that shape; Phase 34 may forward it but not redesign it.
- CRA-POL-006 freezes current save/RON/wire/public API behavior, including top-level setup/update and nested resync.
- Original P10 evidence assigns complete battle content to Core and transport selection/order to server. It requires
  old/new setup/update/resync JSON plus actor-order tests before consolidation.
- Post-Boss evidence confirms the ordinary duplication remains. `BossRuntimeWake` and Boss scheduler state are
  private/non-serde, with no server/C# public lifecycle carrier; no Boss DTO row may be invented.
- Final order makes Phase 33 and Phase 18 direct dependencies. Both are now `Complete / High`.

## Open Policy Decisions

None at initialization. Exact parity is policy-neutral. A discriminator, top-level/nested shape, field, omission,
preview representation, resync topology, actor order or public Boss lifecycle change is a new migration/product
decision and triggers a stop.

## In Scope

- current-tree inventory of authoritative Core setup/update DTOs, serde rules, builders and public fields;
- server message variants, constructors, wrappers, selectors and all direct transport consumers;
- exact old-production setup/update/resync JSON characterization before mutation;
- actor command, tick, resync, reconnect and final-snapshot ordering evidence;
- consumer-aware fixture/source comparison against accessible Unity and current canonicals;
- smallest typed composition that stores/reuses complete Core DTOs once with exact flattened/top-level JSON;
- removal of duplicate server field declarations/destructuring and obsolete Phase-33 transitional extraction only if
  current evidence proves it redundant;
- Core/server debug/release, live-RON, static one-owner and dirty-preservation validation;
- nearest canonical correction only if current runtime evidence proves a statement stale;
- independent completion audit.

## Out Of Scope

- any wire discriminator, field name/value/type/default/omission/nesting/order change;
- per-step preview DTOs or any change to CRA-POL-005's union collection;
- new Boss event/checkpoint/setup/update/session/lifecycle data or serialization of private scheduler state;
- changing `CommandAccepted`, error/resync behavior, actor ordering, snapshot cadence or reconnect semantics;
- moving server message selection, envelope, request correlation, persistence or socket publication into Core;
- changing battle simulation, setup generation, update generation, event log, checkpoint or revision logic;
- RON/content/balance, V3/save/checkpoint schema or Unity source changes;
- compatibility aliases, dual schemas, fallback serializers, unknown/default fields or Phase-35 cleanup;
- unrelated warning, formatting, dependency, target or dirty-work cleanup.

## Implementation Scope

After activation and current-tree proof:

1. inventory every Core setup/update field, serde attribute, constructor and server duplicate declaration;
2. trace command, tick, setup recovery, resync, reconnect and Unity consumption paths;
3. locate existing serde/fixture/order coverage and build a complete exact old-production parity table first;
4. prove setup, update and nested resync old values, including rich optional and preview-union fields;
5. choose the smallest typed server representation that preserves top-level shape without field enumeration;
6. switch all server constructors/consumers atomically and remove only the duplicate field lists/destructuring;
7. compare old/new exact JSON and fixture outputs and prove every actor/reconnect order unchanged;
8. audit for custom field enumeration, fallback, alias, dual shape, Boss placeholder and stale adapter remnants;
9. run affected Core/server debug/release, live-data, static, format, Clippy and dirty-preservation gates;
10. update durable canonical text only for a demonstrated current-contract drift.

No target enum representation, serde strategy, helper name or file list is final until current-tree evidence proves it.

## DTO, Data, Save And Migration

- DTO/wire migration: none. The refactor reuses current DTO values under byte/structure-equivalent JSON.
- Live RON/data migration: none. No content is added, removed or normalized.
- Save/checkpoint migration: none. Public battle checkpoint content is forwarded unchanged; V3/private active battle
  serialization remains excluded.
- Public Rust migration: none approved. Core DTO/public server constructor visibility must be inventoried before any
  narrowing; additive/private composition is preferred.
- Unity migration: none. Accessible sources and fixtures are comparison consumers, not mutation targets.
- Compatibility: no alias, dual reader/writer, legacy message, fallback serializer or version interval approved.

## Deployment And Compatibility

Core and game_server are validated atomically from the same repository tree. Exact old/new setup/update/resync
serialization and actor order prove no coordinated rollout is required. Accessible Unity source/fixtures/canonicals
are inspected and compared read-only. Unknown deployed consumers remain an explicit risk controlled by exact wire
and public-API parity, never by assuming absence.

## Focused Validation

Commands and counts are discovery pending. Required families are:

- exact old/new `serde_json::Value` table for complete setup, update and nested resync;
- field-path/key-set equality that proves setup/update remain top-level and resync nesting remains singular;
- rich values covering optional/null/omitted setup data, events, checkpoint, capabilities, eligibility, visual novel,
  preview union cells, revision and times;
- message constructor/serde tests using real Core DTOs;
- command accepted → setup/update, update-only, resync, stale error → resync, reconnect and final tick ordering;
- accessible Unity fixture/source comparison and exact current canonical alignment;
- static proof of one Core field-schema owner and absence of server field re-enumeration/custom fallback;
- no-Boss-public-carrier inventory.

## Broad Validation

At minimum after affected-target discovery:

- affected Core setup/update and preview tests in debug and representative release;
- affected server message/state/handler/session tests in debug and representative release;
- Core and server all-target tests plus workspace checks in debug/release;
- live RON integration as unchanged-content evidence;
- exact fixture generator comparison when an existing authorized reproducible path is available;
- format, candidate-scoped diff/static audit and normal Clippy diagnostic baselines;
- final source/canonical/consumer hashes and expanded dirty-status preservation check.

Actual runner-reported pass/fail/ignored counts must be recorded. Zero-test targets and compile-only success are not
behavior evidence.

## Omitted Validation

- Live WebSocket probing is provisionally omitted if exact real serde plus actor/session order tests cover the same
  values and sequence. Use a registered probe if any transport-only path remains unclosed.
- Unity EditMode/PlayMode is provisionally omitted only if no Unity source/fixture change occurs and exact server
  output plus accessible parser/store inspection proves compatibility. Reconsider on any fixture mismatch.
- Performance benchmarking is omitted because field composition is not a simulation hot-path algorithm.
- Repository-wide unrelated warning/format cleanup is always omitted.

## Completion Conditions

- Every authoritative Core setup/update field, server duplicate, constructor, wrapper and direct consumer is
  inventoried.
- Old setup/update/resync JSON is characterized first with complete nonzero exact values.
- Server transport stores/composes the complete Core DTO once and owns no duplicate content field schema.
- Exact top-level setup/update and nested resync JSON remains byte/structure equivalent.
- Command, tick, authority recovery, reconnect and final-snapshot order remains exact.
- CRA-POL-005 preview union content, revision/time/events/checkpoint/capability semantics remain exact.
- No Boss public wire, fallback, alias, dual schema, compatibility carrier or custom re-enumerating serializer exists.
- DTO/RON/save/public/API/Unity consumer compatibility remains exact under CRA-POL-006.
- Focused and broad debug/release/static validation is green with actual nonzero counts and classified failures.
- Existing dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P34-C01 | complete Core DTO/server transport/consumer graph identified | fields, serde, constructors, wrappers, actor and Unity consumers | complete — audit pass |
| P34-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P34-C03 | old setup/update/resync behavior characterized first | exact complete JSON table with nonzero rows | complete — audit pass |
| P34-C04 | one Core content schema authority exists | server representation and static duplicate-field proof | complete — audit pass |
| P34-C05 | setup/update top-level JSON remains exact | old/new type/key/value/omission equality | complete — audit pass |
| P34-C06 | nested resync JSON remains exact | single current wrapper and complete inner update parity | complete — audit pass |
| P34-C07 | preview/revision/event/checkpoint content remains exact | CRA-POL-005/006 rich-value assertions | complete — audit pass |
| P34-C08 | command/tick/reconnect/publication order remains exact | actor/session/order and no-extra-snapshot evidence | complete — audit pass |
| P34-C09 | DTO/data/save/public/Unity compatibility is exact | unchanged shapes/data/readers and fixture comparison | complete — audit pass |
| P34-C10 | no fallback/alias/dual schema/Boss placeholder remains | static legacy/custom-serde/private-boundary audit | complete — audit pass |
| P34-C11 | focused/broad debug/release/static validation is green | actual counts and failure classification | complete — audit pass |
| P34-C12 | independent completion audit is current | `Complete / High` review and downstream boundary | complete — audit pass |

## Activation Protocol

Before Phase-34-directed production inspection:

1. Master records Phase 33 `complete — Complete / High`, Phase 34 as the sole `ready` Child and Phases 35~44 pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 34 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-33 completion audit, original/post-Boss
   P10-C03 evidence, Final catalog/order/test/gate/policy inputs and Master activation contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and prospective hashes/status for likely Core/server
   source, tests, fixtures and canonicals using path discovery only, without reading new Phase-34 production content.
5. Record that evidence in the Child before detailed current-tree investigation or tests.

No Phase-34-directed source/test/data/canonical/external-consumer inspection or production mutation is authorized
before all five steps. Prior overlapping Phase-18/33 observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 35 if evidence requires changing a discriminator, field, omission, top-level/nested
shape, preview union, result/side-message/snapshot order, persistence, public API or Unity consumer; exact parity
requires custom field re-enumeration or a dual/fallback schema; a Boss public carrier is needed; an accessible fixture
mismatch cannot be explained without consumer migration; user-owned overlap cannot be safely separated; required
tests select zero; the candidate no longer exists; or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 539 rows.
- Canonical C-sort SHA-256:
  `f17ab3657507e3cc39e91ac7136b8851d9db346cfeb421cdfbf5b79239b971cc`.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved; the marker's approval remains limited to the
  historical Phase-27 one-path exception.
