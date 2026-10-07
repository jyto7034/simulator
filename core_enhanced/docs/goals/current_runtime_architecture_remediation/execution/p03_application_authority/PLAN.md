# P03 Application Authority Execution Plan

상태: `GEX-10 exit conditions met / audit target pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY)`
- Phase / atomic node: `PHASE-CRAR-C2-03-APPLICATION-AUTHORITY` /
  `AG-CRAR-C2-01-APPLICATION-AUTHORITY` (`WP-CRAR-C2-02`, `WP-CRAR-C2-03`)
- Current pass: `GEX-10 exit conditions met`; the two work packages were implemented together and remain one audit
  target.
- Goal Quality Contract: `2`
- Authorization: Core-only selected-Goal mutation and test execution. `game_server`, Unity, external transports and
  consumers are excluded.

## Exact Basis and Direct Dependencies

- Freeze / Plan / Receipt / Goal Set: `FPF-CRAR-P2G30-002` / `PP-CRAR-P2G40-002` /
  `PVR-CRAR-P2G40-002` / `GS-CRAR-P2G50-002`.
- P01: `Complete / High`, latest read-only currentness `ATR-CRAR-C2-P01-GEX20-002`.
- P02: `Complete / High` on replacement target `ATR-CRAR-C2-P02-GEX10-002`, current/unchanged at P02 GEX-20 exit.
- Applicable seams: `IVA-CRAR-C2-02` rejection equality/retry and `IVA-CRAR-C2-03` lifecycle owner/no dual write.

## Objective and Boundary

- Replace convention-owned root mutation with one Core application authority:
  `validate -> stage -> prepare fallible output -> commit -> infallible publish`.
- Replace the freely written root phase field with a private closed `LifecycleContext`; authoritative publication must
  reject incoherent phase/run/node/battle context before commit.
- Register every Core mutating entry in one exhaustive route-owner registry and make the application commit port the
  only root state/run-seed writer.
- Preserve all gameplay algorithms, behavior/result meanings, ordering and deterministic output.
- Out of scope: battle-kernel privacy/finite types, history cap/result lifetime, checkpoint format/version,
  transport-neutral public projection, performance PASS/optimization, external compatibility.

## GEX-00 Current Inventory

- `GameCore::execute_with_transport_context` is the broad behavior router. Domain handlers mutate `GameCoreState`
  directly and then may perform fallible projection.
- `GameCoreState` stores a closed `GameState` phase beside parallel `run`, `node_session`, `active_node_content`,
  `active_battle`, pending-mode and pending-sweep optionals. The state module is private, but child modules can write
  these roots independently and no root commit gate checks their coherence.
- CA-003 actual six-command family is pause, resume, speed, begin/cancel skill targeting and activate skill. Their
  response cursor projection can reject after battle revision/state mutation.
- CA-004 `StartNewGame` clears/initializes root state before a fallible RunStart trigger and transition.
- Existing correct seams (`with_state_transaction`, tactical staging, battle-advance staging, economy plans) prove the
  gameplay algorithms can run on a draft, but they do not establish one root authority.
- Public/root mutation routes found at intake:
  behavior execution; elapsed/server/bucket battle advance; disconnect auto-pause; admin command; active-run rollback
  restore and durable continue; player-profile merge/reset. `set_enkephalin` is a documented test helper and must be
  production-inaccessible rather than a production route.
- Read-only projections/export/capture routes remain queries and do not gain mutation authority.

## Approved GEX-10 Design

1. Add an exhaustive typed `ApplicationRoute` / owner registry covering all `ActionKind` values and every non-behavior
   root mutation family. Registry completeness is compiler/test enforced; no catch-all owner exists.
2. Make `GameCoreState` cloneable and stage a complete Core authority draft, including active battle and `run_seed`.
   Run all validators, gameplay handlers and fallible result projection on the draft.
3. Produce a private typed prepared-commit token only after lifecycle coherence validation. The single infallible
   commit port replaces the authoritative root once and returns a typed accepted publication outcome.
4. Move public mutating entries through that application executor and split public wrappers from draft-only handlers to
   prevent recursion/bypass. Existing local staging may remain only as algorithm-local preparation inside the root
   draft and may not write the live root.
5. Replace direct `game_state` storage/writes with private `LifecycleContext` access and transitions. Validate exact
   battle UUID, node/session identity and phase-required context at the publish boundary.
6. Add actual CA-003 six-command future-cursor equality/retry tests, CA-004 missing-trigger equality/retry tests,
   invalid-phase equality, owner registry completeness, lifecycle incoherence rejection and post-commit accepted/
   recovery-pending typing tests.
7. Align `docs/core_runtime_contract.md` only after the actual implementation is verified.

## Completion Contract

- all mutating routes have exactly one registered owner and use one root commit port; executor bypass/dual root writer 0
- every returned `Err`, including invalid phase and CA-003/004 late failures, preserves exact authoritative
  state/event/revision/checkpoint/durable intent and `run_seed`
- retry begins from the same basis and is deterministic; successful mutating commands commit exactly once
- no fallible result projection occurs after commit; a post-commit publication fault is a typed accepted/recovery-pending
  outcome, never `Err`
- closed lifecycle context cannot be written through a parallel phase field; incoherent context cannot be published
- full Core preservation oracle and focused success/invalid/rollback/retry/ordering/boundary/restart/producer-consumer/
  legacy-absence matrix pass
- canonical runtime contract matches current Core-only authority; external claims remain 0

## Stop Conditions

- an actual mutating route cannot be staged or represented by the closed lifecycle without changing gameplay meaning
- an authoritative post-commit failure must remain a returned `Err`
- policy meaning, Goal/Phase/node mapping, dependency graph or checkpoint/history/public-projection ownership must change
- a required completion claim depends on `game_server`, Unity or another external consumer
- predecessor evidence becomes stale without an approved, direct non-impact/currentness re-audit path
- dirty overlap cannot be isolated safely

## Pre-edit Identities

- `src/game/world.rs` `0da87a4432dad2cd4ad2fe69cb4a480814a2c7202449b5071392ddc8d2ab6752`
- `src/game/world/state.rs` `b82abeb2abeadd8e5b1ae12d76aa342efffc9174d3531a1119022f303bdff9b6`
- `src/game/world/combat.rs` `ad63b1c148d15bcbf86cd038b84448b1021ca3d564b04c3709259f3c4deb3cb5`
- `src/game/world/active_run.rs` `63c0ccaa79a61a600a5161b122f76e9ad75c55f86761a3a8220488650662421a`
- `src/game/world/admin.rs` `8c0c4bcac2bf7672267967e7750034ad8fb383f6c20564d9a4b00fe0b13b9be9`
- `src/game/world/helpers.rs` `e48b02d9bec756d5885802aa3daad5b94af4ecebcdd09d263051c51bf2d19da1`
- `src/game/behavior.rs` `66f037715fd574277b04e8842f33cbb89d9ce3cd62f1c34be59f1d9915c2bd40`
- `docs/core_runtime_contract.md` `73bc18d42f11b54a69f6d861c675cda7d35f6e2090f6b34cb23e539e3d8364f6`

All are pre-existing dirty/mixed paths. P03 owns only exact application-authority/lifecycle/test/document hunks; all
other current bytes remain user-owned.

## Implemented Current Boundary

- `src/game/world/application.rs` now owns exhaustive `ApplicationRoute -> ApplicationOwner`, the complete authority
  draft, typed prepared token, sole live-root commit port and infallible accepted publication disposition.
- Behavior dispatch consumes the owner registry. The superseded broad root variant router and battle-only root staging
  helper are absent.
- Every public Core mutation family enters `execute_application`; complete `GameCoreState`, battle/event/revision,
  durable checkpoint intent and `run_seed` are discarded together on `Err`.
- Private `LifecycleContext` owns `GameState`; direct parallel `game_state` storage/writes are absent. Commit validation
  binds exact node/session, shop/reward, active battle and result contexts.
- CA-003's six real commands now share the root draft and have current future-cursor equality plus identical retry
  evidence. CA-004 missing RunStart trigger has full nonbattle/profile/run-seed equality and identical retry error.
- Default release cannot access `set_enkephalin`, `GameCore.state` or the private state module; valid release probe runs
  actual live publication -> `GameCore` -> StartNewGame application route.
- Canonical runtime contract is aligned to the actual Core-only boundary. P01/P02 successor currentness is restored by
  `ATR-CRAR-C2-P01-GEX20-003` and `ATR-CRAR-C2-P02-GEX20-003`.
