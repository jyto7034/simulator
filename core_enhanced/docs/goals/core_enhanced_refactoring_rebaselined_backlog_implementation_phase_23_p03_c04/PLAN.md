# Phase 23 Goal — CRA-P03-C04 Consolidate Five-Ledger Grant Transaction State

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P03-C04
Master Phase: 23 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after sole-active registration and completion of all mandatory activation reads/fingerprints
Repository Root: `/mnt/f/work/simulator`
Initialization Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate the repeated production capture, pass and commit mapping for the five grant-owned roots — inventory,
skill-fragment inventory, employee roster, UUID manager and Enkephalin — behind the smallest private typed grant
state/transaction boundary. Preserve every existing grant effect and caller outcome exactly.

This Phase is complete only when every current production `GrantExecutor` caller and all public/unknown-consumer
surfaces are inventoried; tests first freeze complete five-ledger before/after state, result/diff order, effect order,
seed derivation, UUID continuation, checked arithmetic, target context and late-failure no-commit behavior for reward,
preview, shop, event, combat and admin families; one private typed boundary owns the shared five-root capture/apply/
commit mapping; callers retain domain-specific offer/session/combat/admin outer staging and exact transaction lifetime;
proven duplicate field-family boilerplate is removed; the public `GrantExecutor` API and all public/RON/V3/DTO/wire
contracts remain compatible; no Phase 24/25 work is prebuilt; focused and broad debug/release validation passes; and
an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

`GrantExecutor` owns ordered grant effect semantics but accepts five mutable roots separately. Historical and
Boss-era audits found that reward apply/preview, shop purchase, event grant, combat result and admin grant callers
independently clone, pass and commit the same inventory/fragments/roster/UUID/Enkephalin field family.

Every confirmed current production caller already stages those roots or operates inside a broader staged state, so
no reachable partial-commit correctness defect was established. This is an ownership and change-amplification risk:
adding a new grant-owned root or changing commit mapping requires synchronized edits at every caller, and an omitted
field can compile while breaking atomicity or parity.

## Dependency And Evidence Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 23 registry row.
- Direct linear predecessor: Phase 22 CRA-P03-C03, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_22_p03_c03/COMPLETION_AUDIT.md`.
- Downstream dependency: Phase 25 CRA-F-C02 may start only after this Phase and Phase 24 are `Complete / High`.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/BOUNDARY_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No policy ID is assigned because private caller aggregation is policy-neutral. Historical evidence nevertheless
requires retaining the public `GrantExecutor` signature: unknown external Rust consumers make removal or narrowing
consumer-gated work rather than an implicit part of this private refactor.

## Priority And Current Disposition

- Priority axes: `3 / 1 / 2 / 3 / 2 / 3 / unknown`; known subtotal `14/18`.
- Final tier/readiness: Tier B, `caller parity first`.
- Re-audited disposition: `still_valid`; behavior refactor, caller parity first.
- Current live-tree disposition: confirmed `still_valid`; eight production executor call expressions still repeat the
  same five-root family across reward, shop, event, combat and admin, with no current partial-commit defect found.

## Current And Target Authority

- Historical current authority: each of the five roots owns its asset state; public `GrantExecutor` owns effect
  validation/application order; each production caller independently owns clone/pass/commit boilerplate and its
  wider domain transaction.
- Historical consumers: generic reward application and preview, shop purchase, event grant, combat result, admin
  grant and any helper reached by starter or maintenance reward effects.
- Target authority: the smallest private typed state/transaction boundary captures or borrows exactly the five
  roots, invokes the existing executor with the same inputs and owns their exact successful commit mapping.
- Domain-specific outer state remains with reward claim, shop offer/session, event planning/session, combat result,
  admin authorization and run initialization owners.
- Exact current types, visibility, caller count, staging topology, tests, public APIs and canonical consumers must be
  rediscovered only after the mandatory activation protocol.

## In Scope

- all current Core production callers that clone, pass, stage or commit the five grant roots;
- public `GrantExecutor` and grant effect/result/diff types as compatibility inventory, without narrowing them;
- effect application and preview paths, including any common helper introduced or changed since the audits;
- reward, shop, event, combat, admin, starter and maintenance handoff classification where they invoke grants;
- exact root capture source, commit destination, outer staged state, validation/error order and transaction lifetime;
- expanded dirty status, scoped diffs and prospective source/test/canonical hashes before production detail;
- test-first complete five-ledger state parity across caller families and representative effect kinds;
- a private typed capture/apply/commit boundary and migration of legitimate internal callers;
- removal only of proven repeated five-field capture/pass/commit boilerplate;
- nearest durable canonical update only if ownership wording materially changes.

## Out Of Scope

- changing, removing, privatizing or deprecating the public `GrantExecutor` signature or public grant effect/result
  types without a separate complete consumer inventory and explicit authority;
- changing effect order, per-index seed derivation, random generation, UUID allocation or continuation;
- changing checked quantities, currency behavior, fragment progress/research, employee experience targeting or
  equipment identity/capacity behavior;
- changing any error variant/order, result/diff order, DTO/wire/RON/V3 shape, public Rust compatibility or canonical
  gameplay contract;
- making the public executor independently rollback-capable or claiming it is a transaction for arbitrary callers;
- merging shop/event/combat/admin sessions, offer removal, event persistence, combat result state or admin policy;
- widening an outer transaction lifetime or changing whether/when domain state commits;
- Phase 24 Event/Shop persisted-active consolidation, Phase 25 common HQ transaction or any later candidate;
- schema migration, fallback, alias, repair, broad formatting or unrelated dirty-tree cleanup.

## Preserved Invariants

1. Grant effects execute in the same declared order and stop at the same first error.
2. Every effect receives the same seed derivation from caller seed and effect index.
3. UUID allocation and the next UUID continuation are byte/semantically identical on success and failure.
4. Inventory items/materials, fragment ownership/progress/dust, roster state, UUID manager and Enkephalin produce the
   exact same complete state and ordering.
5. Existing checked arithmetic, capacity/duplicate UUID failures and no-partial-caller-commit behavior remain exact.
6. `GrantExecutionContext`, selected employee targeting and policy/database lookups retain the same meaning and error
   order.
7. Preview remains read-only and returns the same exact diff/error without advancing any live root.
8. Reward, shop, event, combat, admin, starter and maintenance callers retain their existing outer staging, session,
   offer, authorization and commit lifetime.
9. Public Rust/RON/V3/DTO/wire shapes and public `GrantExecutor` compatibility remain unchanged.
10. Existing user changes are preserved without reset, restore, clean, stash or unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all Child documents exist and the Master records Phase 23 as the sole active row, read the Child completely and
reread the final 299-line Phase 22 audit. Capture expanded status plus prospective source/test/data/canonical hashes
before production detail. Treat every existing hunk as user/campaign-owned until reconciled.

Use characterization-first proof in this order:

1. Inventory the public executor signature, all five root types, every production caller and each outer stage/commit.
2. Build a caller matrix with seed/context/effects, clone source, commit destination, outer domain state, result/diff
   builder and failure behavior.
3. Run current reward/shop/event/combat/admin/starter/maintenance/grant baselines with nonzero counts.
4. Add exact serialized or semantic five-root success and late-failure characterization before structural edits.
5. Introduce the smallest private typed boundary without changing public executor or domain transaction lifetime.
6. Transfer legitimate internal callers and remove only exact superseded five-field boilerplate.
7. Re-run focused, complete Core debug/release, server debug/release and static/compatibility gates.

Because this is behavior-preserving, intended gameplay Red is not required. A structural Red may prove an absent
private boundary only if full behavior parity already exists; compile-only evidence never replaces the caller state
and outcome matrix.

## Required Caller And Effect Matrix

| Family | Required success proof | Required failure/no-mutation proof | Preserved outer owner |
| --- | --- | --- | --- |
| generic reward/apply | exact five roots and diff order over multiple effects | late effect fails after earlier staged success | reward claim/application |
| preview | exact predicted diff and errors | all live roots and UUID continuation unchanged | preview/read path |
| shop purchase | exact price deduction, grant state, offer/result order | insufficient/late grant failure leaves roots and offer/session exact | shop purchase/session |
| event grant | exact planned/applied roots, seed and result | planned late failure leaves event/session state exact | event planning/commit |
| combat result | exact rewards, XP target, UUID/diff order | existing late failure leaves live combat/run roots exact | staged combat result |
| admin grant | exact authorization-independent grant state/result | grant failure installs none of five roots | admin command owner |
| starter/maintenance handoff | classify and retain exact use of common grant path | no new outer transaction claim | run initialization/maintenance |

Representative effects must cover equipment UUID allocation and duplicate/capacity failure, material/currency checked
quantities, fragment ownership and research progress, employee experience targeting, Enkephalin and multi-effect
ordering. Current live vocabulary and actual callers determine the final matrix after activation.

## Migration, Deployment And Compatibility

- No migration, fallback, dual path, repair or coordinated rollout is approved or required.
- Public executor compatibility remains even if workspace callers migrate to a private aggregate.
- The private boundary must be invisible to server and Unity and must not change serialized or wire payloads.
- If safe consolidation requires public signature removal, observable timing/order change or a wider outer transaction,
  stop as `policy_decision_required` rather than assuming permission.

## Validation Plan

- Baseline: grant executor unit tests and all reward, shop, event/map-flow, combat-result, admin, starter/snapshot,
  maintenance/facility and live RON families selected by the caller inventory.
- Test-first caller parity: complete five-root before/after state and exact result/diff/seed/UUID evidence per family.
- Failure: multi-effect late failure, duplicate equipment UUID/capacity or overflow, checked resource failure and
  invalid selected-employee target preserve exact existing no-commit boundaries.
- Preview: exact result parity and zero mutation/UUID advancement.
- Public/compatibility: executor signature, effect/result types, V3/RON/DTO/wire/server/Unity and canonical comparison.
- Broad: complete Core debug/release, official integrations and complete server debug/release.
- Static: caller/direct-field/legacy/fallback search, scoped hashes/status/diff, formatter and no Phase 24/25 prebuild.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P23-C01 | complete executor/caller/five-root graph identified | signature, caller, stage/commit and consumer inventory | complete |
| P23-C02 | dirty targets and concurrent work preserved | pre/post expanded status, scoped diff and hashes | complete |
| P23-C03 | exact caller behavior characterized first | five-root success/failure/result matrix | complete |
| P23-C04 | one private typed grant state boundary exists | source/type/visibility and ownership evidence | complete |
| P23-C05 | legitimate internal callers use the boundary | reward/shop/event/combat/admin plus handoff evidence | complete |
| P23-C06 | superseded five-field boilerplate removed | negative direct-capture/commit and fallback audit | complete |
| P23-C07 | effect/diff/seed/UUID/context/atomicity exact | focused parity and late-failure evidence | complete |
| P23-C08 | public/V3/RON/DTO/server/Unity compatibility exact | signature/schema/consumer/canonical evidence | complete |
| P23-C09 | focused and broad validation green | nonzero debug/release/Core/server/static counts | complete |
| P23-C10 | downstream boundary and independent audit current | Phase 24/25 no-prebuild and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 24 if the public executor/API or serialized/wire shape must be narrowed; effect,
diff, seed, UUID, arithmetic, targeting, error or transaction timing must change; an existing caller cannot retain its
outer session/offer/combat/admin semantics; unknown external consumers require migration; a new ledger changes the
candidate identity; user-owned dirty overlap cannot be separated; required tests select zero cases; a future Phase
must be prebuilt; or the independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
