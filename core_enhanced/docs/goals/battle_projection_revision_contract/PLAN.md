# Battle Projection Revision Contract Goal

Status: complete

Implementation complete: true

Created: 2026-07-12

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Master child: G01

Audit findings:

- DCA-C-01
- CPR-C-01

Accepted policy: PD-01

## Objective

Introduce one authoritative monotonic `battle_state_revision` commit contract, scoped by `battle_uuid`, so Core, game_server, and Unity apply every committed battle projection change exactly in order even when event sequence and battle time do not change.

## User-Visible Problem

Pause, resume, speed changes, blocking Visual Novel changes, capability changes, checkpoint/setup mutations, and other side-band battle state can currently share the same event sequence and battle time. Unity can treat a valid newer state as a duplicate and discard it, leaving playback or overlays stale and potentially blocking the battle flow.

## Goal Working Method

At every start/resume, read this goal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` completely before code exploration, modification, or validation. Meaningful findings, trials, failures, and decisions update all three documents.

This child follows the source-of-truth order:

1. Core runtime transaction and projection code.
2. Live battle data only where it exercises the path.
3. Rust typed battle DTO serialization.
4. game_server mapping, connection cursor ownership, and message ordering.
5. Unity battle reader/store/apply/resync/command code.
6. Current canonical transport/runtime documents.
7. Master and historical audit documents.

## Accepted PD-01 Contract

- The wire field is `battle_state_revision`, scoped by `battle_uuid`.
- It orders authoritative battle-state commits, not packets, serialization attempts, event-log entries, or simulation time.
- The initial committed battle projection starts at revision 1.
- Increment exactly once after every successful transaction that changes any client-authoritative battle projection, including simulation/checkpoint state, playback, deployment, skill readiness, battle Visual Novel state, and exact battle capabilities.
- Finalize nested triggers, Visual Novel projection, and capability changes before stamping and building the update. One revision must never combine state from two commit points.
- Failed commands, rejected/no-op mutations, `RequestBattleState`, ordinary serialization, duplicate resend, and resync do not increment.
- Overflow is checked and returns an error; never saturate or reuse a revision.
- Every top-level `battle_update`, nested `battle_resync.update`, and battle command acknowledgement carries the same committed revision.
- A resync may authoritatively reapply the same current revision. An ordinary update may not.
- Unity orders ordinary state updates by `(battle_uuid, battle_state_revision)`.
- Event sequence validates event-delta continuity only. Battle time drives simulation/presentation only. Neither decides projection staleness.
- A higher revision with equal event sequence and equal battle time is valid and must apply.
- Equal/lower ordinary revision is duplicate/stale and ignored.
- A newer revision with an event-delta gap requests resync instead of guessing.
- The revision covers the complete battle projection. G06/G07 will add exact battle capabilities under this revision; G01 must provide the stable seam without inventing their final eligibility policy.
- Battle mutation commands carry the expected revision through the accepted PD-02/G07 contract. G01 must establish the revisioned acknowledgement/state foundation but must not pre-implement unrelated domain capability rules.
- Transport remains ordered. Command-result side updates and actor tick pushes use one ordered per-session battle outbox or an equivalent proven sequencing mechanism so a later event delta cannot overtake the delta it depends on.
- `last_pushed_event_log_seq` is connection/session cursor state, not Core gameplay/root snapshot state.
- Core owns the immutable event log and current revision. Update/resync projection is read-only and accepts explicit `after_seq` input.
- Battle reset creates a new `battle_uuid` and revision sequence.
- Reconnect/resync preserves the active battle's current revision.
- Setup-loss recovery carries immutable setup plus current revisioned state rather than relying on time.

## Current Historical Evidence To Revalidate

- Core pause/resume/speed and battle VN can create updates without advancing event sequence or battle time.
- Unity `BattleRuntimeState.ApplyUpdate` historically ignored equal-seq/equal-time updates before applying their checkpoint/VN projection.
- Core stores `last_pushed_event_log_seq` inside `ActiveBattleSession` and/or root gameplay state, mixing connection delivery with gameplay authority.
- game_server command replies and actor tick pushes may use distinct emission paths.
- Existing tests cover increasing time/sequence and isolated Core behavior but not the full same-seq/same-time Core → server → Unity sequence.

These are audit hypotheses until current code is read.

## Source Of Truth And Ownership

- Core battle transaction boundary owns `battle_state_revision` increments and the complete committed projection.
- Core event log owns immutable event sequence and delta generation.
- game_server connection/session state owns the last-sent event cursor and ordered emission.
- Rust typed DTO owns exact JSON field shape and casing.
- Unity battle runtime store owns last-applied `(battle_uuid, revision)` and requests resync on continuity gaps; it does not infer freshness from time.
- Canonical runtime and external Unity battle transport documents describe the implemented contract after completion.

## Invariants

- One successful authoritative battle transaction produces at most one new revision.
- Every authoritative projection change produces exactly one new revision.
- One revision identifies one coherent complete projection.
- Read-only projection and transport resend cannot mutate Core state.
- Event sequence, battle time, and battle state revision remain separate concepts.
- Ordinary update dedupe cannot discard a higher revision.
- Stale or duplicate messages never roll Unity state backward.
- Event gaps are never silently accepted.
- Reconnect and resync are idempotent for the current revision.
- No temporary timestamp bump, fabricated event, dual revision field, compatibility alias, or Unity fallback is permitted.

## In Scope

- Core battle session revision state and checked commit semantics.
- Revision inclusion in setup/update/resync and battle command acknowledgement typed DTOs as required by the accepted contract.
- Explicit read-only projection cursor input and removal of connection cursor state from Core/root snapshots.
- game_server per-session ordered battle emission and cursor ownership.
- Unity DTO/read/apply/dedupe/resync ordering by battle UUID and revision.
- Exact Rust JSON shape and real Rust-to-Unity fixture/reader tests.
- Same-seq/same-time pause, resume, speed, VN transition/close, empty-delta, reconnect, resend, resync, stale, gap, and reset test matrix.
- Canonical Core runtime and external Unity battle transport documentation updates.
- Removal of old time/sequence staleness inference and competing cursor ownership.

## Explicitly Out Of Scope

- Final context/payload capability semantics owned by G06/G07.
- Event feature flow, BossOmen policy, employee result aggregation, and other later phases.
- Changing battle simulation timing or event-log meaning.
- Supporting old revisionless DTOs through aliases, dual fields, or fallback inference.
- Save-data migration unless current evidence proves battle state is persisted across process versions; if so, stop for policy.

## Implementation Plan

1. Record dirty-worktree overlap and current Core/server/Unity baseline.
2. Trace every battle mutation/projection path from command or tick entry through final DTO and Unity application.
3. Inventory every current event cursor, battle timestamp, setup/update/resync field, command acknowledgement, and emission path.
4. Write a transaction/revision state table and run small read-only or focused trials to identify the correct single commit seam.
5. Implement Core revision state, checked increments, read-only projections, and exact typed DTOs.
6. Move connection cursor ownership and unify/prove ordered server emission.
7. Migrate Unity reader/store/apply/resync and remove time-based freshness logic.
8. Add exact Rust serialization and actual Rust JSON consumed by Unity tests.
9. Run focused tests after each boundary, then Core/server/Unity broad validation.
10. Audit old fields, fallbacks, stale mocks, ignored tests, and canonical documents.
11. Apply the completion review guide before marking G01 complete.

## Current Implementation Checkpoint

Revalidated on 2026-07-12 after a resumed turn:

- Core DTO/session foundations are partially applied: `battle_state_revision`, checked overflow, distinct battle-session UUID allocation, and distinct `node_id` exist.
- The Core-owned `last_pushed_event_log_seq` field and its draining projection helper have been removed from `ActiveBattleSession` and the root battle context.
- The earlier interrupted cutover was not compile-ready because battle mutation and Visual Novel paths still called removed APIs; those callers have now been migrated in one direction without compatibility overloads.
- The Core caller cutover is now compile-ready: battle mutation and Visual Novel paths use explicit cursors; failed/no-op/query paths do not consume revision; deploy/withdraw pre-command state mutation was removed.
- `cargo check -p game_core` passes after the coherent caller migration. This is compilation evidence only, not the required behavior proof.
- The first combat focused invocation compiled production code but stopped while compiling a new test assertion (`Uuid` was incorrectly dereferenced); 0 tests executed and remains failed historical evidence.
- The corrected rerun executed 45 tests: 44 passed and 1 failed because a legacy test directly killed a BattleCore unit and expected read-only `RequestBattleState` projection to reconcile/mutate deployment.
- The legacy test was replaced with an authoritative advance/finalization assertion; the exact combat suite now passes 45/45 with 596 filtered out.
- Core focused evidence now covers initial/increment/no-op/query/recovery/overflow/fresh-identity/atomic-failure semantics. Broad Core validation remains pending until the cross-layer cutover is complete.
- The next unit is game_server cursor ownership, acknowledgement revision, ordered command/tick/reconnect emission, and exact serialization tests.

Server implementation checkpoint:

- `PlayerGameActor` now has the in-progress per-connection event cursor and routes mutation/tick calls through explicit Core cursor APIs.
- Successful player command response/side-message/snapshot batches are emitted by `PlayerGameActor`; the WebSocket session no longer duplicates that success emission.
- Attach now returns a root snapshot plus optional ordered setup/current-update recovery batch and no longer destroys an active battle.
- `cargo check -p game_server` passes for production targets. It executes 0 tests and does not compile/prove all test fixtures or ordering behavior.
- Server tests still require migration and execution for the new required revision field, attach-result type, command emission owner, reconnect batch, and cursor continuity.
- First focused server invocation failed during test compilation with 8 one-way migration errors and executed 0 tests: two attach-result indexing sites, three required-revision constructors, one exhaustive pattern, and the legacy destructive reconnect expectation.
- After fixture migration, the focused server suite executed 24 tests: 22 passed and 2 failed solely because legacy tests expected command-side battle updates not to be emitted through the actor-owned socket before tick assertions.
- The two ordering tests were rewritten around actor-owned command batches and isolated tick windows; the focused server suite now passes 24/24 with 1 filtered out (plus a zero-test main binary target that is not counted).
- Broad server validation remains pending after Unity and fixture cutover.

Unity implementation checkpoint:

- `BattleRuntimeState` now requires positive revision, orders ordinary updates by revision, treats seq only as delta continuity, and uses a distinct same-revision-capable resync path.
- Time-based same-seq freshness inference is removed; battle time remains presentation clock input only.
- Unity test/mocks now produce required revision fields, including explicit higher/equal/lower same-seq cases.
- A typed Rust generator produces setup/revision-1/revision-2/same-revision-resync JSON; its read-only check matches the checked-in Unity fixture exactly.
- The open `ark` Unity 6000.3.4f1 Editor was refreshed and completed its domain reload with zero Console compilation errors.
- Unity MCP executed `ArkClient.Tests.EditMode.ArkClientContractTests`: 182 passed, 0 failed, 0 skipped. This includes the checked-in actual-Rust fixture consumer and the revision/stale/gap/resync contract matrix.
- Unity MCP executed `ArkClient.Tests.PlayMode.ArkClientMockRenderTests`: 49 passed, 0 failed, 0 skipped.
- The Unity focused and affected PlayMode boundaries are green. Broad cross-layer validation remains pending.

Broad validation checkpoint:

- `cargo test -p game_core --all-targets -- --test-threads=1` passes 682/682 across the lib and integration targets; the zero-test binary target is excluded from the count.
- `cargo test -p game_server --all-targets -- --test-threads=1` passes 25/25; zero-test binary/example targets are excluded from the count.
- First `cargo fmt --all -- --check` failed on standard rustfmt layout differences in G01-touched Core/server files; formatter application and recheck are required.
- Core and game_server `git diff --check` passed. Whole-repository Unity `git diff --check` is unusable as scoped evidence because unrelated pre-existing CRLF additions in `.gitignore`/`.idea` fail it; a G01-path-scoped Unity check is required.
- `cargo fmt --all` was applied; the rustfmt recheck, both Rust repository diff checks, and the exact G01-path Unity diff check now pass.
- Legacy audit found one remaining revision fallback seam: root `game_state_context.in_battle.battle_state_revision` is still optional and `ArkMockSnapshots` fabricates revision 1 when it is absent. G01 must make the root field required/positive and make the mock require it, without a fallback.
- Canonical synchronization found stale active-battle discard/`node_confirm` recovery and `last_pushed_event_log_seq` wording in the two external Unity contract documents.
- The first Unity refresh after root/mock strictness found one C# compile error: the validated mock revision was not passed into the separate update-builder helper. This must be corrected and recompiled; no Unity tests executed in that failed attempt.
- The helper now receives the validated revision explicitly; Unity recompiles with zero errors, EditMode passes 182/182, and affected PlayMode passes 49/49. Core combat remains 45/45 and server actor remains 24/24 after root DTO strictness.
- Runtime comment and both external canonical documents now describe required revision ordering, same-revision resync, same-battle setup/reconnect recovery, actor-owned cursor, and revision-bearing acknowledgements.
- Final post-correction validation passes Core 682/682, server 25/25, Unity EditMode 182/182, Unity PlayMode 49/49, exact Rust fixture comparison, rustfmt, and all scoped diff checks.
- Final searches find no Core delivery cursor/drain helper, optional root revision, mock revision fallback, Unity time/seq freshness comparison, or stale destructive recovery wording.
- `COMPLETION_REVIEW.md` verdict is `Complete`, long-term fit `High`, improvement class `none`; G01 is closed and G06 is unblocked.

## User-Visible Completion Conditions

- Pause, resume, speed, VN scene advance, and VN close all apply in Unity when revision increases even if event seq/time are unchanged.
- Failed/no-op/read-only/resend/resync operations do not consume a revision.
- Every mutation acknowledgement and resulting update expose the same revision.
- A stale ordinary update is ignored; a newer update with an event gap requests resync; resync can reapply the current revision.
- Reconnect and setup recovery restore the current authoritative revisioned battle projection.
- Core projection is read-only and server connection owns delivery cursor state.
- Ordered server emission prevents a later delta from overtaking its dependency.
- New battle UUID resets the revision sequence to initial revision 1.
- Exact Rust JSON and Unity reader/application tests use the same contract.
- No old time-based freshness inference, revisionless mock, compatibility field, or duplicate authority remains.
- Focused and broad Core/server/Unity tests pass with nonzero counts.
- Completion review verdict is Complete with High long-term fit and no immediate correction.

## DTO, Data, And Migration

- Unity-facing DTO shape: accepted change; add required `battle_state_revision` and remove superseded freshness/cursor assumptions without aliases.
- Live RON schema/content: no change expected.
- Save-data migration: none expected; active battle persistence/reconnect representation must be verified.
- Compatibility: not authorized.

## Planned Verification Categories

- Core revision transaction unit tests.
- Core same-seq/same-time battle update tests.
- Core projection read-only/cursor ownership tests.
- Rust exact JSON serialization for setup/update/resync/acknowledgement.
- game_server ordered command/tick/reconnect/resync tests.
- Unity EditMode reader/store/dedupe/gap/resync tests using actual Rust JSON.
- Unity PlayMode only where visible playback/VN application requires it.
- Full Core all-targets, full game_server all-targets, formatting/checks, and affected Unity suites.

Exact commands and counts will be recorded after current test entry points are inspected.

## Stop Conditions

Set `Status: policy_decision_required` and stop before implementing any newly discovered decision about:

- a different public DTO shape than accepted PD-01,
- save migration or backward compatibility,
- a change to gameplay timing, pause meaning, or event-log meaning,
- capability semantics beyond the accepted revision seam,
- a Unity UX change not implied by applying authoritative state.

Technical implementation details inside the accepted revision/ordering contract are recorded and resolved in scope. Unrelated improvements are recorded as follow-up candidates and skipped.
