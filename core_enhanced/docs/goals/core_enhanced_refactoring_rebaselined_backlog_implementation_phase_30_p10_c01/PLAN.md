# Phase 30 CRA-P10-C01 Typed Command Route And Owner Metadata Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 30 of 44
Candidate / Work Type: CRA-P10-C01 / `behavior_preserving_refactor`
Direct Dependency: Phase 29 CRA-P09-C05 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Consolidate the command route facts currently repeated across `ActionKind`, `PlayerBehavior`, common/battle
capability projection, Core dispatch and server host-recovery interception into one typed command descriptor. The
descriptor classifies route owner, command scope and battle-authority requirement; the existing domain evaluator
continues to decide whether a command is currently executable.

This is an ownership refactor only. It must make omissions and route drift mechanically harder without changing any
wire name, action availability, disabled reason, payload validation, revision preflight, mutation, persistence,
rollback or publication behavior.

## User/Contract-Visible Problem

The same command identity is independently classified by enum methods, behavior-to-kind conversion, projected
arrays and membership matches, Core dispatch, and four server recovery intercepts. Exhaustive matches protect some
sites, but separately maintained arrays or sets may omit a new `ActionKind` without an exhaustive-match failure.
The observable behavior is currently valid; the problem is duplicated authority and future change amplification.

## Audit Finding ID

CRA-P10-C01. Original Phase 10 and post-Boss Phase 10 audits both classify it `still_valid`.

## Current Runtime And Live Data Evidence

Initialization respected the no-read gate. After sole activation, prospective fingerprints and dirty ownership were
captured before the current tree was opened. The activated graph proves:

- `ActionKind` has 56 variants and `PlayerBehavior::kind()` remains the exhaustive producer mapping;
- the old tree separately repeated four query scopes, six recovery scopes, fourteen battle-authority mutations, a
  36-action ordered common projection, an 18-action battle evaluator and four server recovery intercepts;
- ten common-projected inventory/Monolith/Shop actions deliberately retain domain preflight, and the two Visual
  Novel actions remain contextual between common and battle preflight;
- the four host operations remain `ContinueRun`, `ResetActiveRun`, `RetryEncounterCompendium` and
  `ResetEncounterCompendium`;
- Core domain dispatch remains an exhaustive payload/mutation router and is not duplicate metadata;
- no RON, live content, save/checkpoint, result/error DTO, Unity source or Boss-runtime surface participates in the
  refactor.

The exact prospective and final fingerprints, status attribution and baseline counts are recorded in
`EXPERIMENTS.md` E03 through E06.

## Source Of Truth And Ownership

### Pre-change authority

- Domain handlers own gameplay mutations.
- Common and battle capability evaluators own the corresponding shared preflight and availability decisions.
- Core owns ordinary player-command validation and dispatch.
- The server host owns the four persistence/recovery commands even though they use the Core command carrier.
- Repeated owner/scope/authority lists were derived classifications, but no single typed descriptor owned all three
  facts before this Phase.

### Target authority

One typed descriptor keyed exhaustively by every `ActionKind` owns only:

- route owner: Core-common, Core-domain, Core-battle, Core-contextual or host-recovery, using the current exact
  boundary;
- command scope;
- whether battle UUID/revision authority is required.

Existing common/battle evaluators continue to own state-dependent availability and typed disabled reasons. Domain
handlers continue to own mutations. The server continues to execute and persist its four host-recovery routes; they
must not be redirected through Core domain dispatch.

### Producers and consumers

- Producers: `PlayerBehavior` decoding/kind selection and the exhaustive `ActionKind` definition.
- Core consumers: scope/authority checks, common/battle capability projection, execution preflight and dispatch.
- Server consumers: host-recovery interception/capability construction and ordinary Core command forwarding.
- Contract consumers: current serde command names, result/error/snapshot DTOs, server actors and accessible client
  canonicals. Unknown deployed Rust/Unity consumers remain compatibility constraints.

The final caller, evaluator, dispatcher and consumer inventory is recorded in `EXPERIMENTS.md` E04 through E06 and
is rechecked by the separate completion audit.

## Invariants

- Every `PlayerBehavior` variant maps to the same `ActionKind` and preserves the same serde wire name.
- Every command retains its current Core-common, Core-domain, Core-battle, Core-contextual or host-recovery route.
- `ActionKind::scope` and battle-authority requirements remain exact.
- State-dependent projected action lists and execution acceptance/rejection remain identical for every applicable
  game/battle/VN/revision state.
- Capability absence, disabled capability and payload-specific validation remain distinct with the same typed
  reasons and errors.
- Stale/missing battle UUID or revision rejects before mutation exactly as today.
- Root capabilities remain absent during battle where revisioned battle capabilities are authoritative.
- The four host recovery commands retain their current server persistence, rollback/replacement, ordering and
  capability behavior.
- Ordinary command dispatch, active-run persistence, compendium handling and response/side-message ordering remain
  exact.
- No DTO, RON, save/checkpoint, wire, existing public Rust behavior or Boss-runtime contract changes. The direct
  Core/server compile boundary may expose only the minimal additive, non-serialized route metadata type.
- Existing user-owned dirty work and the protected Phase 27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C01.
- Current disposition: `still_valid`; Tier B; `tests_first_then_implementation`.
- Priority axes: ownership `3`, correctness/atomicity `1`, lifecycle `2`, change amplification `3`, test fragility
  `2`, failure blast radius `2`, change frequency `unknown`; known subtotal `13/18`.
- Consolidation scope: repeated route owner, scope and battle-authority membership facts that can be derived from
  one typed descriptor.
- Retained distinct authorities: state-dependent availability evaluators, payload validators, domain mutations,
  Core ordinary dispatch, server host-recovery execution/persistence and transport publication.
- First safety net: an exhaustive all-variant owner/scope/authority descriptor table plus projected-versus-
  executable parity by representative authoritative states.
- Behavior-preservation boundary: any difference in route, action list, reason/error, revision/no-mutation,
  persistence, ordering, serialization or an existing public shape stops this refactor.

## Approved Policy And Evidence

- CRA-POL-006 freezes current command/result tags, payloads, nesting and public/API compatibility. A negative
  workspace search cannot authorize public narrowing.
- Original Phase 10 catalog: one typed descriptor may classify ownership while availability remains with current
  domain evaluators; four server recovery operations remain host-owned.
- Post-Boss Phase 10 reaudit: the candidate remains current and policy-neutral after its named characterization
  tests; no public Boss wire exists or is to be invented.
- Final exact order: item 30, after Phase 29 and before typed host overlay CRA-P10-C04.

## Open Policy Decisions

None at initialization. Exact behavior is frozen. A needed wire/public API change, a route-owner change, moving host
recovery into Core, or changing capability semantics is a new policy/scope decision and triggers a stop.

## In Scope

- current-tree inventory of every `ActionKind` classification and `PlayerBehavior` mapping;
- characterization tests for exhaustive descriptor facts and projected/executable parity;
- one private or minimally visible typed descriptor at the narrowest shared Core/server boundary;
- replacement of duplicated owner/scope/authority lists only where exact parity is proven;
- directly affected Core and server focused tests;
- nearest canonical correction only if it is stale relative to unchanged runtime behavior;
- candidate-scoped debug/release and broad validation plus independent completion audit.

## Out Of Scope

- adding, deleting, renaming or reinterpreting commands;
- changing gameplay availability, payload validation, disabled reasons, battle revision or VN rules;
- routing host recovery through Core domain dispatch or moving host persistence into Core;
- CRA-P10-C02 result projection, CRA-P10-C03 battle DTO reuse, CRA-P10-C04 host overlay, CRA-P10-C05 error-code
  authority or any cleanup Phase;
- admin command lifecycle/publication consolidation;
- `RunComplete` shape, result/error schema, snapshot overlay, setup/update/resync topology or first Boss wire;
- RON/content/balance, V3/checkpoint or Unity UX changes;
- compatibility shims, aliases, fallback routes, dual descriptors or future-Phase placeholders;
- unrelated formatting, warning cleanup or dirty-work cleanup.

## Implementation Scope

After activation and current-tree proof:

1. inventory every `ActionKind`, its `PlayerBehavior` producer and all route/scope/authority consumers;
2. capture a nonzero baseline and add the exhaustive characterization table before production consolidation;
3. introduce the smallest typed descriptor that represents only existing facts;
4. switch Core and server consumers only where the descriptor is the correct fact owner;
5. remove obsolete duplicate lists/matches while retaining domain dispatch and host-recovery adapters;
6. prove static single ownership, exact behavior/serialization compatibility and no next-Phase prebuild;
7. update durable canonical only for a demonstrated stale description, never to describe a future contract.

No implementation approach is fixed until the activated current tree proves the exact owner and visibility boundary.

## DTO, Data, Save And Migration

- DTO/wire migration: none. Command enum variants, serde names, capability/result/error/snapshot shapes and nesting
  remain exact.
- Live RON/data migration: none. No data schema or production bundle change is expected.
- Save/checkpoint migration: none. V3 and active-battle/checkpoint content remain exact.
- Public Rust migration: none. The separate server crate requires a minimally public additive `ActionRoute` and
  `HostRecoveryAction`; neither derives serde nor narrows or changes an existing public symbol.
- Compatibility: no temporary compatibility path is approved or needed.

## Deployment And Compatibility

Core and game_server are validated from the same repository state. Exact current server behavior and accessible
external canonical evidence are required where the descriptor crosses their compile boundary. Unknown deployed
Rust/Unity consumers remain explicit risk, closed for this Phase only by preserving all observable/public shapes.
No coordinated rollout or version cutover is authorized.

## Focused Validation

The required audited validation families are:

- exhaustive `ActionKind` → owner/scope/battle-authority descriptor characterization;
- exhaustive `PlayerBehavior` → `ActionKind` parity;
- projected-versus-executable parity across common, battle, VN-blocked, stale/missing authority and revision states;
- exact host recovery request, capability, persistence/replacement and ordering tests for all four commands;
- behavior/serde tests proving wire names and carrier shapes unchanged;
- static caller/duplicate/route inventory and nonzero test-selection proof.

The characterization must pass against the duplicated old production before consolidation.

## Broad Validation

At minimum, after affected-target discovery:

- Core affected library/integration suites in debug and representative release;
- server affected mapping/actor/persistence suites in debug and representative release;
- Core and server all-target tests and checks in debug/release when the cross-package route boundary changes;
- repository Rust format check and candidate-scoped diff/static audit;
- Clippy as a diagnostic gate with baseline/new-warning distinction, not an unrecorded strict-warning claim.

Exact commands and runner-reported pass/fail/ignored counts must be recorded. Compile-only or zero-test targets do
not count as behavior evidence.

## Implementation Result

- `behavior.rs` now macro-owns the 56-value `ActionKind` vocabulary and one exhaustive private
  `ActionDescriptor { route, scope, requires_battle_authority }` table. Existing `scope()` and
  `requires_battle_authority()` delegate to it; additive `route()` exposes the minimal server seam.
- `ActionRoute` represents the five proven current boundaries: `CoreCommon`, `CoreDomain`, `CoreBattle`,
  `CoreContextual` and typed `HostRecovery`. It does not decide state-dependent availability or payload validity.
- common capability membership and preflight, battle preflight and server host interception consume the descriptor.
  The existing 36-action common projection order remains an explicit observable ordering view, with bidirectional
  debug parity against descriptor membership.
- battle evaluator order and all domain payload/mutation matches remain unchanged. Bidirectional debug parity makes
  a battle-routed descriptor omission visible without treating the evaluator as metadata duplication.
- server actor ordering remains exact: compendium management still returns before sync, reset-active-run remains
  after sync, and continue-run follows the existing rollback/checkpoint/persistence flow.
- the nearest Core runtime and Unity bridge canonicals now describe the unchanged ownership boundary. Game rules,
  server bridge, external Unity canonical, data and save contracts remain untouched.

## Validation Result Before Independent Audit

- Test-first characterization passed on duplicated old production: 1 passed, 0 failed, 0 ignored, 1,021 filtered.
- Final focused Core behavior: 9 passed; four exact common/battle/VN/revision selections: 1 passed each; server
  library: 59 passed. All selected tests were nonzero with no failure or ignored case.
- Core full library: 1,022 passed. Core all-target debug and release: 1,073 passed each. Server all-target debug and
  release: 59 passed each. All had zero failed/ignored tests; zero-test binaries were identified and excluded from
  behavioral counts.
- Core and server all-target `cargo check` and `cargo clippy` passed in debug and release. Clippy reports only the
  repository's existing warning baseline; format and scoped diff checks exit 0.
- Final implementation status before an audit artifact is 526 rows at normalized SHA-256
  `882d5f6bfdfe9c4e53fab15eaf99a478573cdba88fb622a8d6f44765db73f5d2`. The protected marker remains 57 bytes at
  SHA-256 `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with exact timestamps.
- Phase 31 remains pending and absent. These results authorize the separate completion audit, not completion by
  themselves.

## Omitted Validation

- Live WebSocket probing is provisionally omitted if typed Core/server mapping and actor-order tests close the same
  unchanged contract; reconsider if a real transport sequence cannot otherwise be proven.
- Unity Edit/PlayMode is provisionally omitted because no wire or Unity source change is authorized; exact
  accessible canonical/source fingerprints remain required.
- Performance benchmarking is omitted because route metadata lookup is not an approved algorithmic behavior
  change; reconsider if current-tree implementation evidence shows a hot-path regression risk.
- Repository-wide unrelated lint/format cleanup is always omitted.

## Completion Conditions

- Every current command and producer/consumer route is inventoried with no unclassified variant.
- The old duplicated behavior is characterized first with a nonzero exhaustive descriptor/parity safety net.
- One typed descriptor owns route owner, scope and battle-authority facts.
- State-dependent capability/execution authorities remain distinct and exact.
- All four host recovery operations remain server-owned and behavior-identical.
- Obsolete duplicated owner/scope/authority lists are removed; no dual/fallback/placeholder route remains.
- Wire/public DTO/RON/save/checkpoint/server/accessible-consumer contracts remain exact under CRA-POL-006.
- Focused and broad debug/release/static validation is green with actual counts and classified failures.
- Existing dirty work and the protected approval marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.
- No Phase 31 implementation or placeholder is introduced.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P30-C01 | complete command producer/consumer/route graph identified | exhaustive variant, mapper, evaluator, dispatcher and host-intercept inventory | complete — audit pass |
| P30-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P30-C03 | old route behavior characterized first | exhaustive descriptor and projected-versus-executable parity with nonzero counts | complete — audit pass |
| P30-C04 | one typed route metadata authority exists | source/static ownership and visibility audit | complete — audit pass |
| P30-C05 | every command owner/scope/authority fact is exact | all-variant table and old/new parity | complete — audit pass |
| P30-C06 | capability and execution semantics remain exact | common/battle/VN/revision/no-mutation tests | complete — audit pass |
| P30-C07 | four host recovery lifecycles remain exact | server request/capability/persistence/order tests | complete — audit pass |
| P30-C08 | public/data/save/server/Unity compatibility is exact | unchanged shapes, data and accessible canonical/consumer evidence | complete — audit pass |
| P30-C09 | duplicate/legacy/fallback routes are absent | static caller/list/placeholder audit | complete — audit pass |
| P30-C10 | focused/broad debug/release/static validation is green | actual nonzero counts and failure classification | complete — audit pass |
| P30-C11 | downstream boundary and independent audit are current | no Phase 31 prebuild and `Complete / High` review | complete — audit pass |

## Activation Protocol

Before production inspection:

1. Master records Phase 29 `complete — Complete / High`, Phase 30 as the sole `active` Child and Phases 31~44
   pending.
2. Change all three Child headers from `initialized` to `active` and authorize only Phase 30.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase 29 completion audit, original and
   post-Boss Phase 10 evidence, Final catalog/order/test/gate/policy inputs.
4. Capture HEAD, expanded status count/normalized hash and prospective hashes for likely source/test/canonical
   targets using path discovery only, without reading their content.
5. Record that evidence in the Child before examining production details.

No Phase 30 source/test/data/canonical content inspection or mutation is authorized before all five steps.

## Stop Conditions

Stop and do not activate Phase 31 if evidence requires changing any command name/route, availability, disabled
reason, payload validation, revision/no-mutation rule, host persistence/recovery, result/snapshot/wire/public shape,
admin lifecycle, Boss public boundary or CRA-POL-006 compatibility; adding fallback/dual descriptor/alias; absorbing
P10-C02~C05 or cleanup work; user-owned overlap cannot be separated; required tests select zero; the candidate no
longer exists; or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status: 522 rows.
- Normalized C-sort SHA-256:
  `3ec521d59411b6a660dcd28f24510332f9725677ca4ddbebb2780657d6f4ef7c`.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved; the marker's approval remains limited to the
  historical Phase 27 one-path exception.
