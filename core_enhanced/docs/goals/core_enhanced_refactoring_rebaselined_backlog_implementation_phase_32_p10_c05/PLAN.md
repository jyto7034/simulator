# Phase 32 CRA-P10-C05 Stable GameError Code Authority Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 32 of 44
Candidate / Work Type: CRA-P10-C05 / `behavior_preserving_refactor`
Direct Dependency: Phase 31 CRA-P10-C04 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Give every Core `GameError` identity exactly one stable error-code authority and make both Core snapshot projection
and the game_server adapter consume it. Context-specific Core/server human messages, disabled reasons, structured
authority details and all current serialized behavior remain owned by their existing adapters.

This is an ownership refactor only. Every current snake-case code, parameterized-variant result, message string,
field presence/value, failure/no-mutation behavior and transport order must remain exact.

## User/Contract-Visible Problem

Original and post-Boss Phase-10 audits found that the full `GameError` enum is independently matched for stable code
identity by two Core snapshot helpers and again by the server error adapter. Exhaustive Rust matches prevent an
unhandled variant but do not prevent three valid arms from assigning different protocol strings to the same error.
Representative tests cover only a small subset, so stable error vocabulary can drift without a complete equality
proof.

Messages legitimately differ by presentation context. The problem is duplicated stable code identity, not the
message wording or the existence of distinct Core/server adapters.

## Audit Finding ID

CRA-P10-C05. Original and post-Boss P10 audits classify it `still_valid`, Tier C, and
`full_code_table_then_implementation`.

## Current Runtime And Live Data Evidence

Historical and final evidence identifies these prospective surfaces:

- the Core `GameError` enum, including parameterized variants;
- two Core snapshot error code/message projection helpers;
- the game_server conversion from `GameError` to its actor/transport error;
- `ActionUnavailable` disabled-reason projection whose reason is also a stable code value;
- structured authority/revision fields and context-specific Core/server messages;
- focused Core behavior/snapshot/capability tests and server error/mapping/actor tests;
- current Core/server protocol canonicals and accessible Unity error consumers, if present.

No Phase-32-directed production source, test, data or canonical read/search occurred during initialization. Earlier
Phase-10 and completed Phase-31 evidence is input only; current types, variants, call sites, serialized forms and
dirty attribution are discovery pending after sole activation and prospective fingerprinting.

## Source Of Truth And Ownership

### Current authority

- Core `GameError` owns semantic error identity and parameterized detail.
- Stable protocol code literals are duplicated by two Core snapshot projections and the server adapter.
- Each adapter separately owns its contextual human message and any structured output fields.
- Domain handlers and preflight evaluators own when an error occurs and whether mutation is rejected.

### Target authority

One exhaustive typed Core authority—preferably `GameError::code()` unless current-tree evidence proves a narrower
typed code representation—maps every `GameError` variant to its exact current stable code. Core snapshot projection
and the server adapter call that authority instead of reauthoring literals.

Adapters continue to own message formatting and structured context. The new code authority must not decide
availability, mutation, disabled reason selection, transport envelope or presentation wording.

### Producers and consumers

- Producer: the authoritative Core `GameError` value emitted by command/domain/preflight paths.
- Code consumers: both current Core snapshot projections and the game_server error adapter.
- Message/detail consumers: existing Core snapshot/client-facing capability readers and server command error
  serialization.
- External consumers: accessible Unity/source/canonical readers and unknown deployed Rust/Unity clients constrained
  by CRA-POL-006.

Exact current symbols, visibility needs, producer paths and consumer inventory are discovery pending after sole
activation.

## Invariants

- Every `GameError` variant retains its exact current snake-case code.
- Parameter values never change code identity unless the current mapping already does so; exhaustive characterization
  freezes the actual behavior first.
- `ActionUnavailable` retains its exact reason-as-code behavior and omission/value rules.
- Core snapshot and server mappings remain exactly equal for the same semantic error.
- Every Core and server human message remains byte-for-byte/context-for-context exact.
- Structured details such as action, battle UUID, expected/actual revision or other authority fields remain exact.
- Error occurrence, precedence, validation timing, rollback/no-mutation and publication order remain unchanged.
- No error is collapsed, renamed, reclassified, converted to success or silently recovered.
- No command/result/snapshot/battle DTO, RON, live data, V3/save/checkpoint or public Boss shape changes.
- Phase 30 route metadata and Phase 31 typed host overlay remain untouched and behavior-identical.
- Existing user-owned dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C05.
- Current disposition: `still_valid`; Tier C; `full_code_table_then_implementation`.
- Priority axes: ownership `3`, correctness/atomicity `1`, lifecycle `1`, change amplification `2`, test fragility
  `2`, failure blast radius `2`, change frequency `unknown`; known subtotal `11/18`.
- Consolidation scope: duplicate stable code literals and duplicate exhaustive code-only matches in the two Core
  snapshot paths and server adapter.
- Retained distinct authorities: `GameError` construction/semantic details, domain/preflight failure decisions,
  adapter-specific messages, structured fields, serialization envelopes and actor publication.
- First safety net: exhaustive parameterized `GameError` and disabled-reason code table proving exact equality across
  both Core snapshot paths and server on old duplicated production.
- Behavior-preservation boundary: any code, message, detail, serde shape, failure timing, mutation or transport-order
  difference stops the refactor.

## Approved Policy And Evidence

- CRA-POL-006 freezes current command/result tags, payloads, nesting, stable error vocabulary and public/API
  compatibility. Unknown deployed consumers prohibit changing or removing codes based on workspace search.
- Original P10-C05 evidence assigns stable identity to `GameError::code()` or an equivalent typed code authority,
  while explicitly retaining contextual messages in adapters.
- Post-Boss P10 evidence confirms all three mappings remain independently authored and requires exhaustive
  parameterized error/disabled-reason equality before implementation.
- Final item 32 has no technical predecessor beyond the strict Phase-31 campaign gate and precedes Phase-33 result
  projection. It must not prebuild result or battle-transport work.

## Open Policy Decisions

None at initialization. Exact current codes, messages and fields are frozen. Renaming a code, standardizing message
text, changing fields/nullability, exposing a new wire enum, altering error precedence or adding compatibility aliases
is a new policy/scope decision and triggers a stop.

## In Scope

- current-tree inventory of `GameError`, both Core code projections, server conversion and all direct tests/consumers;
- exhaustive old-production characterization for every variant, parameter category and disabled reason;
- one typed Core stable-code authority with the minimum visibility required by current consumers;
- replacement of only duplicate code literals/matches by calls to that authority;
- exact code/message/detail/serde parity and error/no-mutation behavior validation;
- affected Core/server debug/release tests, live-RON compatibility and static one-owner audit;
- nearest canonical correction only if current-tree inspection proves it stale;
- independent completion audit and exact dirty-tree preservation proof.

## Out Of Scope

- changing which `GameError` a command/domain/preflight path emits or when it emits it;
- normalizing, translating or otherwise consolidating contextual human messages;
- changing disabled reason selection, structured authority/revision detail or retry/recovery meaning;
- CRA-P10-C02 result projection, CRA-P10-C03 battle DTO transport or any cleanup Phase;
- adding new errors, public wire tags, result fields, first public Boss wire or client UX;
- RON/content/balance, V3/checkpoint, persistence or Unity source changes;
- compatibility aliases, dual code maps, fallback matches, unknown/default codes or future-Phase placeholders;
- unrelated warning, formatting, dependency, target or dirty-work cleanup.

## Implementation Scope

After activation and current-tree proof:

1. inventory the exact `GameError` variants, parameters, constructors and all three stable-code projections;
2. inventory message/detail construction separately so it cannot be accidentally centralized or altered;
3. add a nonzero exhaustive old-production table covering every variant and disabled reason across both Core paths
   and the server adapter;
4. prove exact current code equality or stop and classify any existing divergence before choosing an authority;
5. add the smallest exhaustive typed stable-code method/representation on the Core semantic error;
6. switch only code consumers to it and remove only the proven duplicate literals/code-only matches;
7. prove exact code, message, structured detail, serialized output, error behavior and no-mutation parity;
8. audit public visibility, legacy mappings, fallbacks, aliases and Phase-33/34 placeholders;
9. update durable canonical text only if a demonstrated stale statement describes the current contract incorrectly.

No method name, return type, visibility or module placement is final until the activated tree and old behavior table
prove the smallest authority boundary.

## DTO, Data, Save And Migration

- DTO/wire migration: none. All codes, messages, fields, tags, nesting and omission rules remain exact.
- Live RON/data migration: none. Error projection does not author or migrate content.
- Save/checkpoint migration: none. No V3 or persistence representation changes.
- Public Rust migration: none. An additive code accessor may be needed for the server dependency, but no existing
  public symbol is narrowed or removed; exact visibility is discovery pending.
- Compatibility: no alias, legacy code table, dual mapping, unknown/default fallback or version interval is approved.

## Deployment And Compatibility

Core and game_server are validated atomically from the same repository state. Exact old/new output proves that
existing consumers require no coordinated rollout or compatibility layer. Accessible external consumer/canonical
evidence is inspected read-only when relevant. Unknown deployed consumers remain explicit risk closed by exact
parity, never by assuming absence.

## Focused Validation

Commands and current counts are discovery pending. Required families are:

- one exhaustive table containing every current `GameError` variant and representative values for parameterized
  variants, with a row-count assertion;
- equality across both Core snapshot projections and the server adapter for every table row;
- every current disabled reason and `ActionUnavailable` reason-as-code mapping;
- exact Core/server message and structured-detail assertions that prove those adapter responsibilities did not move;
- representative command/preflight/authority failures with complete no-mutation or state fingerprints;
- exact serialized `Value`/bytes where error or disabled-reason DTOs cross the wire;
- static one-owner/duplicate-literal/legacy/fallback/placeholder inventory with nonzero test selection.

## Broad Validation

At minimum after affected-target discovery:

- affected Core behavior/snapshot/capability tests in debug and representative release;
- affected server mapping/message/actor tests in debug and representative release;
- Core and server all-target tests and checks in debug/release because a cross-crate error API changes internally;
- live RON integration as unchanged-contract evidence;
- format, candidate-scoped diff/static audit and Clippy as a diagnostic baseline gate;
- final source/canonical/consumer hashes and expanded dirty-status preservation check.

Exact commands and runner-reported pass/fail/ignored counts must be recorded. Zero-test targets and compile-only
success do not count as behavior evidence.

## Omitted Validation

- Live WebSocket probing is provisionally omitted if exact producer/adapter serde tests and real actor command-error
  tests close the same values and order. Use a registered probe if transport behavior cannot otherwise close.
- Unity EditMode/PlayMode is provisionally omitted because no Unity/wire change is authorized; reconsider on any
  serialized difference or external source edit.
- Performance benchmarking is omitted because a constant error-code match is not a hot simulation algorithm.
- Repository-wide unrelated warning/format cleanup is always omitted.

## Completion Conditions

- Every `GameError` variant/parameter category and all three current code projections are inventoried.
- The old duplicated behavior is first characterized by a nonzero exhaustive Core/server equality table.
- One exhaustive typed Core authority owns stable error code identity.
- Both Core snapshot paths and the server consume that authority; duplicate code literals/code-only matches are gone.
- Every current code, disabled reason, message, structured field and serialized shape remains exact.
- Error occurrence, precedence, rollback/no-mutation and publication ordering remain unchanged.
- No fallback/default/alias/dual mapping, new error, public wire migration or Phase-33/34 placeholder exists.
- DTO/RON/save/checkpoint/public/server/accessible-consumer compatibility remains exact under CRA-POL-006.
- Focused and broad debug/release/static validation is green with actual nonzero counts and classified failures.
- Existing dirty work and the protected approval marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P32-C01 | complete GameError/code/message/consumer graph identified | enum/parameter/constructor/Core/server/consumer inventory | complete — audit pass |
| P32-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P32-C03 | old duplicate mappings characterized first | exhaustive parameterized code table with nonzero row count | complete — audit pass |
| P32-C04 | one typed stable-code authority exists | source/static ownership and exhaustive-match proof | complete — audit pass |
| P32-C05 | Core snapshot paths and server consume one code authority | caller and duplicate-literal/match audit | complete — audit pass |
| P32-C06 | every stable code and disabled reason is exact | complete old/new equality table and serialized parity | complete — audit pass |
| P32-C07 | messages and structured details remain context-exact | adapter-specific golden/detail assertions | complete — audit pass |
| P32-C08 | failure/no-mutation/ordering behavior remains exact | focused command/preflight/actor tests | complete — audit pass |
| P32-C09 | DTO/data/save/public/consumer compatibility is exact | unchanged shapes/data/readers and CRA-POL-006 evidence | complete — audit pass |
| P32-C10 | no fallback/alias/dual map/future placeholder remains | static legacy and Phase-33/34 inventory | complete — audit pass |
| P32-C11 | focused/broad debug/release/static validation is green | actual counts and failure classification | complete — audit pass |
| P32-C12 | independent completion audit is current | `Complete / High` review and downstream boundary | complete — audit pass |

## Activation Protocol

Before Phase-32-directed production inspection:

1. Master records Phase 31 `complete — Complete / High`, Phase 32 as the sole `active` Child and Phases 33~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-31 completion audit, original and
   post-Boss P10 evidence, Final catalog/order/test/gate/policy inputs and Master activation contract.
4. Capture HEAD, expanded status count/normalized hash and prospective hashes/status for likely Core/server source,
   tests and canonicals using path discovery only, without reading new Phase-32 production content.
5. Record that evidence in the Child before detailed current-tree investigation or tests.

No Phase-32-directed source/test/data/canonical inspection or production mutation is authorized before all five
steps. Prior overlapping audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 33 if evidence requires changing any stable code, disabled reason, contextual message,
structured field, serde shape, failure precedence/timing, no-mutation behavior or publication ordering; existing
Core/server codes diverge and no already-authoritative value can be selected without policy; the method would require
fallback/default/alias/dual mapping; public API or deployed consumer migration is required; P10-C02/C03 or cleanup is
absorbed; user-owned overlap cannot be safely separated; required tests select zero; the candidate no longer exists;
or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 531 rows.
- Normalized C-sort SHA-256:
  `5b566e36b4107b89ea83a1ed84b7ff10a68ccad0017e930af073fd5ee0cf836d`.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved; the marker's approval remains limited to the
  historical Phase-27 one-path exception.
