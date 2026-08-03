# Contextual Action Capability Projection Goal

Status: complete

Implementation complete: true

Created: 2026-07-13

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Master child: G06

Primary findings:

- DCA-M-13
- DCA-M-18
- DCA-M-19 shared preflight portion only

Accepted policy: PD-02

Predecessor: G01 `battle_projection_revision_contract` — Complete / High / no immediate correction

## Objective

Replace root `allowed_actions` and duplicate state-table/cached execution authority with one Core-owned typed context capability projection and a reusable pure preflight/validation seam. Migrate the common Rust DTO, server root transport, and non-battle Unity consumer without absorbing battle-specific or domain-specific payload policy owned by later children.

## User-Visible Problem

The current payload-less action list can claim an action is available while every concrete payload fails later, can expose mutually incompatible actions together, and can diverge from Unity-local rules. Users therefore see enabled buttons that fail after selection or different availability after reconnect. Query and recovery commands also appear indistinguishable from ordinary gameplay actions.

## Goal Working Method

At every start/resume, read this goal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` completely before code exploration, modification, or validation. Meaningful findings, experiments, failures, corrections, and scope decisions update all three documents.

This child follows the source-of-truth order:

1. Core command gate, domain validators, mutation entry points, and snapshot projector.
2. Live RON/data only where capability projection depends on authored content.
3. Rust typed root DTO and exact serialization.
4. game_server root snapshot mapping and ordering.
5. Unity root reader/store/presenter and command submission.
6. Current canonical gameplay/wire documents.
7. Master/child historical documents.

## Accepted PD-02 Contract

- Remove root `allowed_actions`; do not redefine it or keep a compatibility alias.
- Add typed `action_capabilities` entries containing stable action identity, scope (`gameplay`, `query`, `recovery`), enabled state, and a typed disabled reason when blocked.
- Omit context-irrelevant actions. Retain context-relevant blocked actions as disabled entries with reasons.
- A parameterized context capability is enabled only when at least one executable payload currently exists; this does not imply every payload is executable.
- Concrete eligibility belongs to typed domain DTOs or selected-payload preflight. Do not add a generic payload bag or unbounded Cartesian-product projection.
- Projection/preflight and execution share pure validators and typed reason vocabulary. Execution revalidates atomically.
- Remove `GameCoreState.action_validator` cached HashSet and state-only `ActionScheduler` as execution authorities.
- Capability snapshots are presentation projections, not authorization tokens.
- Query/recovery scope remains distinct from gameplay presentation.
- G01 revision semantics remain authoritative. G06 must not move battle cursor/commit/reconnect ownership.

## Source Of Truth And Ownership

- Core pure capability projector owns context relevance, scope, enabled state, and common reason codes.
- Domain pure validators own payload eligibility and command rejection reasons.
- Core command execution remains the final atomic authority and calls the same validators.
- Rust typed DTO owns exact serialized capability shape.
- game_server transports the typed root snapshot without reinterpreting capability.
- Unity renders Core capability and localizes typed reasons; it does not synthesize missing capability.
- G07 owns battle context capability, battle payload eligibility, expected-revision commands, reconnect/resync capability integration, and static battle whitelist removal.
- G03/G21/G24/G28 own Event, Safe Zone research, inventory/consumable, and Shop payload policy respectively.

## Invariants

- One command kind has one stable typed capability identity and one declared scope.
- Context relevance is not confused with payload executability.
- Enabled means at least one currently executable payload exists or the payload-free command passes the shared validator.
- Disabled projection and execution return the same stable typed reason against the same authoritative state.
- Every mutation command revalidates immediately before mutation.
- No execution gate depends on a cached HashSet, Unity whitelist, or duplicated state-only table after cutover.
- Root snapshots never emit both `allowed_actions` and `action_capabilities`.
- Invalid or unknown reason/action values fail typed deserialization; no stringly generic payload fallback is introduced.
- G01 battle revision, cursor, emission order, reconnect, and resync semantics remain unchanged.

## In Scope

- Inventory of current action identities, scopes, gates, validators, root projection, server serialization, Unity root consumers, and tests.
- Shared typed capability/action-scope/common-reason DTOs.
- Pure common projector/preflight seam and command-gate composition.
- Removal of root `allowed_actions`, `GameCoreState.action_validator` cache authority, and state-only `ActionScheduler` execution authority.
- Migration of common/root server and Unity consumers to `action_capabilities`.
- Context tests for omitted irrelevant actions, disabled relevant actions/reasons, and gameplay/query/recovery separation.
- At least one payload-free and one parameterized common parity proof where current domain evidence permits without absorbing later policy.
- Exact Rust serialization fixture consumed by Unity for the common root contract.
- Canonical common action/capability document synchronization.

## Explicitly Out Of Scope

- Battle context capability and payload eligibility, expected battle revision commands, battle reconnect capability, and Unity static battle whitelist removal: G07.
- Final Event choice/employee eligibility: G03.
- Final Safe Zone research delivery eligibility: G21.
- Inventory/consumable capacity/use preview and concrete eligibility: G24. G06 owns only the shared seam required by DCA-M-19.
- Shop purchase/sale/refresh eligibility: G28.
- Maintenance content/balance policy: deferred G22.
- Rewriting all domain commands or adding generic untyped payload eligibility.
- Compatibility aliases, dual schemas, or Unity fallback inference.

## Current Historical Evidence To Revalidate

- `ActionKind` is payload-free and the first gate compares only `PlayerBehavior.kind()`.
- `ActionScheduler` projects coarse state tables and may expose Event/support/recruitment/shop actions outside their concrete content.
- `GameCoreState.action_validator` caches a HashSet while `get_allowed_actions()` may recompute another view.
- Concrete UUID/resource/tile/target checks occur later in domain handlers.
- Pause/Resume and blocking Visual Novel advance/choice can be projected too coarsely, but final battle behavior belongs to G07.
- Unity has root `allowed_actions` consumers and a separate static battle whitelist; only the common/root consumer migrates here.
- `run_checkpoint.can_load` duplicates current command executability while Unity has no consumer for it. G06 removes that field and represents `LoadRunCheckpoint` as a typed recovery capability; inventory/consumable eligibility remains with G24.

These are hypotheses until current runtime, live data, server, and Unity paths are read.

## Current Implementation Checkpoint

Revalidated on 2026-07-13:

- `GameCore::gate_behavior_action`, `get_allowed_actions`, and root snapshot projection all recompute `ActionScheduler::get_allowed_actions_for_context`; the cached `GameCoreState.action_validator` is written during transitions/content refresh but is not read by the live gate or snapshot. It is dead duplicate authority/state.
- `ActionScheduler` remains a coarse context table: every `InNode` receives mutually unrelated node actions, every Shop receives purchase/sale/reroll/exit, every battle receives ten actions, and active VN receives both progression kinds.
- Payload checks remain distributed across domain dispatch and handlers. Only starter selection, map selection, equip item, and reward selection have explicit pre-dispatch validators; other domains validate internally.
- `RunSnapshotDto.allowed_actions` is preserved generically by game_server through the typed flattened root snapshot.
- Unity `GameSnapshotStore` parses root `allowed_actions` into a string HashSet. `GameCommandBus`, common screen composers, and live-flow tests call `HasAllowedAction`.
- When a battle runtime exists, Unity ignores the root set and substitutes a static `battleRuntimeActions` whitelist. Its removal remains G07, but G06 must provide the shared store/DTO seam it will consume.
- Unity mocks and many contract tests hand-author `allowed_actions`. The admin debug command/UI also exposes `dump_allowed_actions` / `Dump Allowed` and must be migrated or removed in the no-compatibility cutover.
- No live RON, production loader, run checkpoint, or saved state owns `allowed_actions`/`ActionValidator`; G06 requires no data or save migration.
- Focused baseline is green: legacy scheduler 14/14, kind-level transition gate 1/1, server root preservation 1/1, and Unity EditMode contract class 182/182. These prove the current coarse contract is stable, not that it meets PD-02.
- The independent broad trace reconciled with the local trace: `PlayerBehavior::kind()` remains only stable command-family identity; common execution authority will be a tri-state pure evaluation (`irrelevant`, `disabled(reason)`, `enabled`), while battle and deferred domain handlers remain their atomic authority until their owning child migrates them.
- Query ownership is explicit: `RequestMapData` is common G06; `RequestBattleState` and `RequestDeploymentRangePreview` are G07. Recovery ownership is explicit: `LoadRunCheckpoint` is common G06; `RecoverBattleSetupLoss` is G07. `RetreatBattle` remains gameplay.
- The first vertical parity trial is `LoadRunCheckpoint` plus `SelectMapNode`: both have bounded authoritative state, expose a current ambiguity/failure mode, and can share projection and execution validation without deciding another child's payload policy.
- Unity MCP is available against `ark@b49d32f6f2f70bbe` (Unity 6000.3.4f1). It is the required compiler/console/EditMode/PlayMode verification path for Unity changes in this goal.
- The next evidence gate is the first pure-validator trial and typed DTO serialization test, followed by the atomic root cutover.
- Typed `ActionScope`, `ActionDisabledReasonCode`, `ActionCapabilityDto`, and `GameError::ActionUnavailable` now exist as the first reversible implementation slice. Root serialization is not cut over yet.
- A first `RequestMapData` / `SelectMapNode` / `LoadRunCheckpoint` tri-state projector and atomic preflight slice is implemented and verified 1/1 after correcting the test-only imports.
- The Rust root DTO is now cut over atomically from `allowed_actions` to `action_capabilities`, and the redundant factual DTO field `run_checkpoint.can_load` is removed. `cargo check -p game_core` passes; server/Unity consumers and remaining action families are not yet migrated.
- Unity production store/command/presenter now has a strict typed capability implementation. The first MCP compile found 18 expected test-only references to the deleted `HasAllowedAction` API; production scripts introduced no reported compiler error after the new script was explicitly imported through Unity MCP.
- After test API renaming, Unity MCP compilation is clean. Full EditMode executes 182 tests: 160 pass and 22 fail, all currently traced to hand-authored `allowed_actions` fixtures producing absent capabilities or tests explicitly protecting the old contract. Fixture migration and semantic test replacement are in progress.
- The Core projector now includes common start/starter/map/node/support/HQ/shop/reward/non-battle-VN/combat-result families with current session predicates. The first focused rerun failed because the earlier trial expected an empty NotStarted capability list; `StartNewGame` is now correctly present and the test needs an explicit typed assertion.
- Legacy `ActionScheduler`, cached `ActionValidator`, transition refreshes, old root query APIs, and their module files are removed. Production `cargo check -p game_core` passes; `cargo test --no-run` now reports 33 compile errors exclusively in tests still calling the deleted API, which must be replaced with behavior/capability assertions by owner.
- All 33 stale Core test API call sites are migrated or removed by ownership. `cargo test -p game_core --lib --no-run` now succeeds; one resulting unused test import was removed after the compile.
- Focused state-transition and blocking-VN tests pass 1/1 each. The checkpoint-limit test executed 1 and failed on a remaining assertion for the intentionally removed `run_checkpoint.can_load` DTO field; runtime capability state was already correct.
- Unity fixture/admin migration removes all `allowed_actions`/old admin names from `Assets`. MCP compile is clean and EditMode improves from 160/182 to 179/182. The final three failures have been corrected as policy-aligned tests: VN snapshots switch from Advance to Choice, irrelevant map Confirm is omitted, and local placeholder commands remain outside Core capability omission.
- First broad Core run executes 628 tests: 621 pass and 7 fail. Two expose a real G07-boundary bug (G06 preflight intercepted battle VN); one exposes premature Shop execution ownership (context projection masked a specific G28 handler error); four are expected typed-error assertion migrations. Corrections are scoped and in progress.
- After correcting ownership and typed expectations, the broad Core library suite passes 628/628 with 0 ignored.
- Unity MCP full EditMode passed 182/182 before adding four new strict/provenance tests. The new Rust generator exactly matches the checked-in Unity fixture, Unity imports it as a TextAsset, and the four new fixture/strictness/localization tests pass 4/4. Current full expected EditMode count is 186 pending final rerun.
- Server focused root preservation and typed disabled error code tests each pass 1/1; error responses now preserve the exact disabled reason code rather than only a generic action-unavailable code.
- Full Unity PlayMode executes 53 tests: 45 pass and 8 fail, all in `ArkClientMockRenderTests`. Failures cluster around NodeMap utility subscenes/status/modal/admin/scroll state and require focused triage; reconnect tests completed. No completion claim is made.
- PlayMode triage proved the first two failures were obsolete pre-preview Confirm-button expectations; their early abort leaked UI roots and caused the other six failures. After policy-aligned omission assertions, full PlayMode passes 53/53.
- Final broad verification is green: Core all-targets 669 nonzero tests, game_server all-targets 26,
  Unity MCP EditMode 186/186, and Unity MCP PlayMode 53/53.
- Runnable Unity WebSocket probes and their closest contract documents now validate typed
  `action_capabilities`, mutual-exclusive VN progression, and factual checkpoint state without
  `can_load`. Python AST parsing, exact Rust fixture comparison, formatting, scoped diff checks,
  legacy searches, and Unity Console error audit all pass.
- `COMPLETION_REVIEW.md` records Complete / High / no immediate correction. Battle-only local
  capability substitution is explicitly handed to G07; domain payload eligibility remains with
  its named child owners.

## Implementation Plan

1. Record dirty-worktree overlap and current Core/server/Unity baseline after the mandatory document read.
2. Trace command entry through action gate, pure/domain validation, mutation, root projection, server mapping, and Unity rendering.
3. Build an action ownership matrix: action identity, scope, context relevance, payload owner, existing validator, reason source, and owning child.
4. Run small read-only/focused trials to determine the smallest pure common projector and validator interface.
5. Introduce typed common action scope/reason/capability DTOs and exact serialization tests.
6. Compose projection and execution through shared validators; preserve atomic revalidation.
7. Remove root `allowed_actions`, cached `ActionValidator` authority, and state-only `ActionScheduler` authority without compatibility.
8. Migrate game_server root serialization and common Unity reader/store/presenter.
9. Add an actual Rust-produced common capability fixture consumed by Unity.
10. Run focused parity/context/negative tests, then broad Core/server/Unity validation.
11. Audit old fields/tables/caches/local inference/ignored tests and synchronize canonical documents.
12. Apply the completion review guide before marking G06 complete.

## User-Visible Completion Conditions

- Unity common/root UI uses only Core `action_capabilities` for context actions and typed disabled reasons.
- Irrelevant actions are absent; relevant blocked actions remain visible only where intended and explain why they are disabled.
- Gameplay, query, and recovery scopes do not leak into one another's ordinary button presentation.
- A projected enabled common capability executes successfully against unchanged authoritative state.
- A projected disabled common command is rejected by execution with the same typed reason.
- State mutation between projection and command is atomically revalidated rather than trusting the old snapshot.
- Root/server/Unity exact JSON agrees through an actual Rust fixture.
- `allowed_actions`, cached HashSet authority, state-only scheduler execution authority, common Unity local substitution, aliases, and fallback paths are absent.
- G01 revision/reconnect tests remain green.
- Focused and broad Core/server/Unity tests pass with nonzero counts.
- Completion review verdict is Complete with High long-term fit and no immediate correction.

## DTO, Data, And Migration

- Unity-facing DTO shape: accepted replacement of `allowed_actions` with typed `action_capabilities`; no alias or dual field.
- Live RON schema/content: no change expected for the common seam. Stop if a domain data change is required.
- Save-data migration: none expected because root capability is derived projection, not saved authority.
- Compatibility: not authorized.

## Planned Verification Categories

- Pure projector relevance/scope/reason tests.
- Projection/execution parity and mutation-between-projection atomic revalidation tests.
- Exact root Rust JSON serialization and server preservation tests.
- Actual Rust capability fixture consumed by Unity EditMode tests.
- Unity common UI rendering/localization/no-local-substitution tests.
- Existing G01 focused revision/reconnect regression tests.
- Full Core all-targets, game_server all-targets, affected Unity EditMode/PlayMode, formatting, diff, and legacy searches.

Exact commands/counts will be recorded after current test entry points are inspected.

## Stop Conditions

Set `Status: policy_decision_required` and stop before implementing any newly discovered decision about:

- a different public capability DTO shape or reason semantics than accepted PD-02,
- which currently ambiguous action should be visible/hidden if the answer changes UX rather than follows accepted relevance rules,
- live RON schema/content, save migration, compatibility, balance, reward/failure timing, or content removal,
- battle-specific capability/expected-revision behavior owned by G07,
- concrete Event, inventory, Shop, Safe Zone, or Maintenance eligibility owned by another child.

Technical details required to realize the accepted common projector/validator seam are recorded and resolved in scope. Out-of-scope improvements are recorded as follow-up candidates and skipped.
