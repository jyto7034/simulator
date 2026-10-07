# Core Enhanced Refactoring Audit Phase 10 — Candidate Catalog

Catalog Date: 2026-08-07
Status: complete; accepted by phase-exit audit `Complete / High`

Axes are ownership duplication, correctness/atomicity, lifecycle complexity, change amplification, test fragility,
failure blast radius and change frequency. Current dirty work makes frequency `unknown`; known subtotals do not
automatically determine disposition.

## Confirmed Candidates

### CRA-P10-C01 — Consolidate Typed Command Route And Owner Metadata

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: `ActionKind` ownership/scope/authority is repeated across enum methods, `PlayerBehavior::kind`,
  projected arrays, common/battle membership matches, Core dispatch and four server host intercepts. Arrays can omit
  variants without an exhaustive-match error.
- Current Authority And Consumers: Core domain handlers own gameplay; common/battle evaluators own shared preflight;
  the server owns four recovery operations but consumes Core carrier/metadata.
- Target Authority: one typed command descriptor/route classifies Core-common, Core-battle and host-recovery owner,
  scope and battle-authority need. Availability still comes from the current domain evaluator.
- Preserved Invariants: every serde wire name, current action list by state, typed disabled reason, payload validator,
  revision preflight and all four host persistence/recovery behaviors remain exact.
- Removal Or Consolidation Scope: remove duplicate owner/authority membership lists and server knowledge that can be
  obtained from the typed route. Do not route host recovery through Core domain dispatch.
- Priority Axes And Total: `3 / 1 / 2 / 3 / 2 / 2 / unknown`; known subtotal `13/18`.
- Validation Safety Net: exhaustive all-variant descriptor table; projected-versus-executable parity by state;
  battle-authority and disabled-reason tests; exact host recovery request/capability tests.
- Policy/External Contract Decision: none if wire names and owner behavior are frozen. Public descriptor shape itself
  need not become a wire contract.
- Readiness: `tests_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P10-C02 — Make One Typed Command-Result Projection Authority

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: result identity and fields are authored in `BehaviorResult`, copied into eleven domain result
  enums, then matched again for `result_type` and JSON. `RosterCommandResult` is never constructed. Public result
  helpers are workspace-test-only. `RunComplete` has an uncovered direct-map payload shape.
- Current Authority And Consumers: Core owns semantic results and the derived command contract; the server consumes
  only that contract. External direct consumers of public Core result APIs are unverified.
- Target Authority: one typed wire-result projection emits transport class, stable result type and payload from one
  representation; nested completion and battle-update classification share that authority.
- Preserved Invariants: every current result type string, payload key/omission, nested completion, read-only preview,
  battle accepted/update split and the current direct-map `RunComplete` payload remain byte-shape equivalent.
- Removal Or Consolidation Scope: remove the eleven parallel domain enums/manual JSON builders where they add no
  invariant, the unconstructed roster branch and unused helpers after external API inventory. Remove/merge
  `behavior_result_helpers_match_variants` only with the public helpers.
- Priority Axes And Total: `3 / 2 / 3 / 3 / 3 / 3 / unknown`; known subtotal `17/18`.
- Validation Safety Net: construct every result variant including nested completions; golden `result_type` and exact
  JSON; explicit `RunComplete` direct-map assertion; battle/read-only transport tests; external API build check.
- Policy/External Contract Decision: behavior-preserving work freezes current `RunComplete`. Wrapping it in `map` is
  a separate migration requiring consumer approval.
- Readiness: `all_variant_goldens_and_external_api_inventory_first`.
- Disposition: `priority_candidate`.

### CRA-P10-C03 — Reuse Core Battle Transport DTOs Without Field Reauthoring

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: Core setup/update DTOs already own tagged serialized content, while server message variants
  repeat and constructors move each field. Every Core field change amplifies into the adapter and its fixtures.
- Current Authority And Consumers: Core owns battle setup/update contents; server owns top-level message selection,
  resync wrapping and publication order; Unity is an unverified external consumer.
- Target Authority: server transport composes/serializes the complete Core DTO once and adds only the genuinely
  server-owned resync envelope.
- Preserved Invariants: exact `type`, top-level rather than nested update/setup fields, revision/time/event/checkpoint
  fields, resync nesting, response-before-side-message and reconnect ordering.
- Removal Or Consolidation Scope: remove duplicate server field declarations/destructuring and the extra
  `battle_update_from_behavior_result` variant list once C02 exposes transport classification.
- Priority Axes And Total: `2 / 1 / 2 / 3 / 3 / 3 / unknown`; known subtotal `14/18`.
- Validation Safety Net: serialize old/new setup, update and resync values and compare exact JSON; actor order tests;
  regenerate/compare external fixtures only in an authorized consumer-aware Goal.
- Policy/External Contract Decision: none for exact parity. Any nesting/tag change is excluded migration work.
- Readiness: `exact_serde_parity_tests_first`.
- Disposition: `candidate`.

### CRA-P10-C04 — Replace JSON Snapshot Mutation With A Typed Host Overlay

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: Core returns a typed snapshot, but server serializes to `Value`, indexes and mutates
  `action_capabilities`, appends active-run/compendium fields and clears/rebuilds capabilities in fatal recovery.
- Current Authority And Consumers: Core owns gameplay snapshot/capabilities; server owns persistence-derived status
  and four host recovery capabilities; clients consume the flattened root object.
- Target Authority: a typed host snapshot overlay/capability merge combines Core and host-owned fields before one
  serialization, with explicit fatal-recovery replacement semantics.
- Preserved Invariants: flattened Core root keys, root capabilities outside battle, empty root battle actions,
  recovery action enable/reason/order, active-run metadata, compendium status and no duplicate actions.
- Removal Or Consolidation Scope: remove `value["..."]` mutation and ad hoc capability push/clear paths. Do not move
  persistence state into Core or let the host infer gameplay actions.
- Priority Axes And Total: `3 / 2 / 3 / 3 / 3 / 3 / unknown`; known subtotal `17/18`.
- Validation Safety Net: exact snapshots for missing/loaded/backup/corrupt active run and clean/dirty/fatal
  compendium; battle root behavior; Core-root preservation; duplicate/order assertions.
- Policy/External Contract Decision: none if flattened JSON and current host fields are frozen.
- Readiness: `table_tests_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P10-C05 — Centralize Stable Game Error Codes

- Work Type: `behavior_preserving_refactor`
- Problem Evidence: all `GameError` variants are independently matched for code in two Core snapshot helpers and the
  server adapter. Existing tests cover only a small subset.
- Current Authority And Consumers: Core error enum owns identity; Core snapshot and server own contextual messages;
  stable codes are shared protocol vocabulary.
- Target Authority: `GameError::code()` (or an equivalent typed code enum) owns stable code identity; adapters retain
  their context-specific messages.
- Preserved Invariants: every current snake-case code, `ActionUnavailable` reason-as-code behavior, detailed authority
  fields and all human messages.
- Removal Or Consolidation Scope: remove duplicate code literals/exhaustive code matches, not message formatting.
- Priority Axes And Total: `3 / 1 / 1 / 2 / 2 / 2 / unknown`; known subtotal `11/18`.
- Validation Safety Net: exhaustive variant/code table including parameterized variants and every disabled reason;
  retain server message assertions.
- Policy/External Contract Decision: none while codes remain exact.
- Readiness: `full_code_table_then_implementation`.
- Disposition: `candidate`.

### CRA-P10-C06 — Normalize Core Package Targets And Direct Dependencies

- Work Type: `behavior_preserving_cleanup`
- Problem Evidence: no build script exists but four build dependencies are resolved; a large direct dependency set
  has no source/test use; `src/main.rs` is an empty auto binary; the ctor hook is empty; fixture bins are test-enabled
  zero-test targets; Core logging has no workspace consumer; `tokio_unstable` is enabled without Core tokio use.
- Current Authority And Consumers: Cargo manifest/metadata own workspace build targets. Some fixture binaries and
  standalone Core API consumers may exist outside the workspace and remain unverified.
- Target Authority: a minimal manifest with explicit runnable fixture targets, no empty default binary/build graph,
  and dependencies justified by compiled Core sources. Fixture generators remain runnable and are marked non-test.
- Preserved Invariants: library API and behavior, fixture bin names/output, server path dependency, feature behavior
  and supported standalone commands confirmed by consumer inventory.
- Removal Or Consolidation Scope: remove confirmed-unused build/runtime dependencies, empty main/ctor hook and
  zero-test target participation; remove logger support only after external API inventory; reassess member config.
- Priority Axes And Total: `2 / 1 / 2 / 3 / 2 / 2 / unknown`; known subtotal `12/18`.
- Validation Safety Net: `cargo metadata`, `cargo check/test` for Core/server/all targets, feature check, every retained
  fixture invocation to `/tmp`, dependency graph diff and documented standalone consumer check.
- Policy/External Contract Decision: external package/API/tool consumers must be inventoried first. Fixture output
  regeneration against Unity requires separate authorization.
- Readiness: `external_inventory_then_mechanical_cleanup`.
- Disposition: `candidate_with_external_gate`.

### CRA-P10-C07 — Remove The Empty Disconnected Test Module

- Work Type: `behavior_preserving_cleanup`
- Problem Evidence: `core_enhanced/tests/test_actor/mod.rs` is empty, Cargo metadata does not list it as a test target,
  and no module edge includes it.
- Current Authority And Consumers: none.
- Target Authority: Cargo's five actual integration-test roots and inline test modules remain the complete test graph.
- Preserved Invariants: test discovery/count, production behavior and fixture coverage remain unchanged.
- Removal Or Consolidation Scope: delete only the empty file/directory; do not remove real integration tests.
- Priority Axes And Total: `1 / 0 / 0 / 1 / 1 / 0 / unknown`; known subtotal `3/18`.
- Validation Safety Net: metadata/test list before and after plus Core test runner.
- Policy/External Contract Decision: none.
- Readiness: `implementation_ready`.
- Disposition: `low_priority_confirmed_cleanup`.

## Deferred, Rejected And Unverified Signals

| ID | Signal | Work type | Disposition and gate |
| --- | --- | --- | --- |
| CRA-P10-S01 | Change `RunComplete` payload to `{ "map": ... }` | `compatibility_migration` | deferred; current direct-map shape must first be frozen/tested and external consumer migration approved |
| CRA-P10-S02 | Put admin commands into the normal player transaction/publication path | `policy_or_feature_migration` | deferred; admin durability, rollback, revision and battle publication policy is absent |
| CRA-P10-S03 | Delete Core/server fixture generators because runtime has no references | `cleanup` | rejected on present evidence; historical exact external fixture provenance exists and current Unity is unverified |
| CRA-P10-S04 | Delete Phase 9 ghost range/public surfaces | `cleanup` | direct server absence confirmed, external/public consumers unverified; return to P09-C07 gate |
| CRA-P10-S05 | Choose per-step or union effect-preview cells | `policy_or_feature_migration` | remains P09-C04 policy gate; server only forwards DTO |
| CRA-P10-S06 | Treat nested Core workflow as active repository CI | `false_authority` | rejected; repository root is parent and official GitHub discovery is root `.github/workflows`; standalone copy remains unverified |

## Priority Summary

1. C02 and C04 remove the highest-drift wire/snapshot seams, but both are test-first.
2. C01 makes owner and authority metadata explicit before further command additions.
3. C03 follows C02 because result transport classification can remove its remaining duplicate battle match.
4. C05 is a contained code-authority consolidation.
5. C06 has useful build amplification reduction but must wait for external inventory.
6. C07 is safe and ready but deliberately low priority.

No signal requiring a policy or external compatibility decision is represented as an implementation-ready refactor.
