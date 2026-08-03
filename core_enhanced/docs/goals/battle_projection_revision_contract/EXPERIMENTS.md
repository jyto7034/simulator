# Battle Projection Revision Contract Experiments

Status: complete

Implementation complete: true

Created: 2026-07-12

## 2026-07-12 — Goal startup

Hypothesis:

Accepted PD-01 is sufficiently detailed to begin current-state discovery without another gameplay or wire policy decision.

Action:

- Phase 0 and R00 passed.
- Created the three G01 goal documents before any G01 code search or validation.
- Copied the binding PD-01 invariants, scope, completion conditions, and stop conditions into the child boundary.

Changed files:

- G01 documentation only.

Commands and actual test counts:

- No G01 tests executed yet; actual count 0 and not validation success.
- Phase 0 baseline available as prior evidence: Core 681/681, server 25/25.

Observed result:

- No policy gate is currently unresolved.
- Current runtime/server/Unity hypotheses still require direct revalidation before design or code changes.

Decision and next step:

Read all three G01 documents, then trace the complete current mutation/projection/emission/application paths before choosing a commit seam.

## 2026-07-12 — Current projection and ordering trace

Hypothesis:

The defect is not only a missing DTO number: Core cursor mutation, projection timing, server multi-sender emission, and Unity time-based dedupe form one cross-layer authority split.

Minimal experiment:

Read current Core update construction and mutation handlers, server result/tick emission, Unity apply/resync logic, relevant tests, and dirty diffs without modifying code or running G01 tests.

Changed files:

- G01 documents only.

Commands and actual test counts:

- Read-only `rg`/`sed` traces across Core, game_server, Unity runtime/tests, and current diffs.
- No G01 tests executed; actual count 0 and not validation success.

Observed result:

- `ActiveBattleSession` owns `last_pushed_event_log_seq`; `drain_battle_update_dto()` both projects and mutates that cursor.
- Root `GameStateContextDto::InBattle` serializes `last_pushed_event_log_seq`, exposing connection delivery state as gameplay snapshot data.
- `battle_update_dto_after()` is read-only and already accepts an explicit event cursor, but mutation handlers use the draining path.
- Active battle start initializes the Core cursor to `None`.
- Pause/resume/speed mutate directly and drain immediately; repeated pause/resume or selecting the active speed has no no-op guard.
- Deploy, withdraw, skill activation, simulation tick, and initial start all build update DTOs inside their domain handlers.
- Simulation update DTO construction occurs before BattleStarted/BattleTimeReached VN trigger evaluation; the VN projection is attached afterward. This happens to refresh VN but makes the projection transaction boundary fragmented.
- Battle VN commands create an update by querying the latest event seq after changing the VN state.
- `RequestBattleState` calls deployment reconciliation before its read-only projection, so current query semantics can mutate derived battle state.
- Server command results are returned to the session actor, which sends response/setup/update. Tick pushes are sent directly by the player actor to the socket. Two senders can enqueue dependent battle messages.
- Command acknowledgement contains only `accepted_at_battle_time_ms`; no revision.
- game_server owns no per-connection event cursor; Core's drain supplies it.
- Unity ordinary freshness uses event seq and server battle time. Same-seq/same-time updates are discarded.
- Unity `ApplyResync()` delegates to ordinary `ApplyUpdate()`, so there is no explicit same-revision authoritative reapply path.
- Unity's existing same-seq test only accepts the update when server battle time increases and intentionally rejects older time.
- Active battle state is not part of run checkpoint persistence, so no current save-data migration was found in this trace.
- `battle_uuid` is currently copied from `node_id.0`. Re-entering the same node therefore reuses both battle UUID and the event sequence starting at 1, contrary to PD-01's new-battle identity rule.
- Current setup-loss recovery and server attach destroy the active battle and return to NodeConfirm. Accepted PD-01 instead requires reconnect/resync to preserve the battle and current revision.
- Deploy increments `next_instance_salt` before the BattleCore command succeeds; withdraw removes the deployed employee before the BattleCore command succeeds. A revision-only patch would leave failed-command partial mutation unaddressed.
- The external canonical documents still prescribe discard-to-NodeConfirm recovery and are implementation-update targets, not current PD-01 authority.
- Current Unity tests construct revisionless `JObject` fixtures by hand. No actual Rust JSON consumed by Unity tests was found.

Failure cause:

Projection authority and delivery authority are mixed: Core consumes a connection cursor, DTOs lack a state revision, server emission has multiple senders, and Unity infers projection freshness from unrelated event/time fields.

Correction:

Not implemented yet. The likely long-term boundary is a Core-owned revision committed after the full authoritative transaction, pure explicit-cursor projection, a player-actor-owned ordered outbox/cursor, and Unity revision-first application with a distinct authoritative resync path.

Decision and next step:

Complete the mutation/transaction table and reconcile it with the read-only Warp Grep report before editing code. Then implement from Core outward in small validated steps.

Warp Grep reconciliation:

- The independent read-only trace reached the same commit seam and added three required corrections: fresh battle UUID per battle instance, failed deploy/withdraw atomicity, and reconnect preservation rather than setup-loss discard.
- G01 implementation must therefore start by separating battle identity/node identity and pure projection/cursor ownership; merely adding a DTO field is explicitly rejected.

## 2026-07-12 — Resumed partial Core cutover audit

Hypothesis:

The interrupted Core-first edit can be continued from its current state without reverting user work, provided the remaining legacy projection calls are treated as an incomplete cutover rather than supported compatibility.

Minimal experiment:

- Re-read the complete G01 documents, the relevant Master/Phase/G04 review material, every Master startup document, and both audit inputs before code inspection.
- Inspected the current dirty worktree and the exact G01 diff/call sites without running compilation or tests.

Changed files:

- Goal documents only in this experiment.
- Existing partial runtime edits were inspected, not newly validated.

Commands and actual test counts:

- Read-only `git status`, `rg`, `sed`, and scoped `git diff`.
- No G01 tests executed; actual count 0 and not validation success.

Observed result:

- `LiveBattleUpdateDto` and root battle context now expose `battle_state_revision`; checked overflow and a fresh battle-session UUID allocator exist.
- `ActiveBattleSession` now separates `node_id` from `battle_uuid`, owns a revision, and exposes an explicit-cursor read-only projection.
- The old Core delivery cursor and draining helper are absent, but `combat.rs`, battle VN conversion, and several tests still call the removed APIs or pass `Option<u64>`.
- `world.rs` already routes an explicit server cursor to battle mutation handlers, while those handler signatures have not yet been migrated.
- The current state is therefore a normal incomplete atomic cutover and is expected not to compile until all Core callers are updated.

Failure cause:

The prior turn ended between introducing the new authority types and migrating their callers.

Correction:

Complete the Core transaction boundary in one direction only: successful projection mutations finalize state, commit once, then project with explicit `after_seq`; query/resync/resend and actual no-ops remain read-only.

Decision and next step:

Finish Core mutation/projection callers and atomic deploy/withdraw behavior before the first focused compile check. Do not reintroduce a Core cursor or optional compatibility signature to obtain an intermediate green build.

## 2026-07-12 — Core caller cutover compiles

Hypothesis:

Every current Core battle projection mutation can use the accepted revision seam without restoring connection cursor ownership or introducing a compatibility API.

Minimal experiment:

- Added mutation-before-commit overflow preflight.
- Migrated simulation start/tick/next-event, pause/resume/speed, deploy, withdraw, skill, battle VN, and battle-state query projection to explicit `after_seq`.
- Finalized BattleStarted/BattleTimeReached Visual Novel triggers before committing/building the simulation update.
- Treated same-value playback commands as non-mutating and made `RequestBattleState` projection read-only.
- Delayed deploy salt consumption and withdrawal deployment removal until after the BattleCore command succeeds.

Changed files:

- `src/game/world/state.rs`
- `src/game/world.rs`
- `src/game/world/combat.rs`
- `src/game/world/visual_novel_flow.rs`
- `src/game/world/tests/combat.rs` (signature-only explicit cursor migration so the suite compiles)

Commands and actual test counts:

- `cargo check -p game_core`: passed.
- Compile check executes 0 tests and is not behavior validation.

Observed result:

- The Core package compiles with no `drain_battle_update_dto`, optional cursor adapter, or Core delivery cursor restored.
- Initial live-battle activation can commit revision 0 to wire revision 1 after trigger finalization.
- Every migrated mutation result projects against a caller-provided event cursor.
- Behavior-level invariants still require focused tests before the Core portion is considered complete.

Failure cause:

No compile failure occurred.

Correction:

None for compilation. Add focused tests rather than relying on type-checking.

Decision and next step:

Add and run the Core revision/no-op/query/overflow/atomic-failure matrix, then fix any semantic failures before beginning the server authority cutover.

## 2026-07-12 — First Core combat focused run failed during test compilation

Hypothesis:

The updated combat suite will execute the revision contract matrix without breaking existing combat behavior.

Minimal experiment:

Ran the complete `game::world::tests::combat` lib-test filter after adding revision assertions to existing gameplay flows and one overflow test.

Changed files:

- `src/game/world/tests/combat.rs`
- Setup-loss recovery runtime had also been changed to return immutable setup plus current revisioned state without discarding the battle.

Commands and actual test counts:

- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`: failed during test compilation; 0 tests executed.

Observed result:

- Rust error E0614 at the new recovery-state assertion: the matched `battle_uuid` is a copied `Uuid`, but the assertion attempted `*battle_uuid`.
- No runtime assertion ran, so this command proves no behavior.

Failure cause:

Test-only pattern binding type mistake introduced with the new recovery contract assertion.

Correction:

Compare the `Uuid` value directly without dereferencing it.

Decision and next step:

Apply the one-line test correction and rerun the exact same combat filter. Preserve this failed 0-test invocation in the experiment history.

## 2026-07-12 — Core combat suite exposes legacy query-side reconciliation expectation

Hypothesis:

After the test-only compile fix, the combat suite will validate the new pure-query boundary.

Minimal experiment:

Corrected the `Uuid` comparison and reran the identical combat filter.

Changed files:

- One test assertion correction before the run.

Commands and actual test counts:

- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`: 45 executed, 44 passed, 1 failed.

Observed result:

- All new initial revision, setup recovery, pause/resume/no-op, VN same-seq/time, speed remainder, overflow, fresh battle UUID, and failed deploy/withdraw assertions passed.
- `live_deployment_reconciles_defeated_player_unit_before_state_dto` failed because it directly mutates a BattleCore unit into a defeated state and expects a subsequent `RequestBattleState` query to remove the deployment entry.
- G01 deliberately removed that query-side reconciliation because read-only projection cannot mutate authoritative state or revision.

Failure cause:

The legacy test fixture bypasses the authoritative simulation transaction and protects the superseded query-mutation behavior.

Correction:

Rewrite the test so the defeat is produced/finalized by an authoritative battle mutation path, or explicitly exercise the transaction finalization seam before issuing a read-only query. Assert that the mutation consumes one revision and the later query consumes none.

Decision and next step:

Inspect the exact fixture and migrate the test to the transaction boundary. Do not restore reconciliation inside `RequestBattleState` to make the old test green.

## 2026-07-12 — Core revision transaction focused suite passes

Hypothesis:

Moving deployment reconciliation to authoritative transaction finalization will preserve existing gameplay while making projection/query read-only.

Minimal experiment:

- Replaced the direct-mutation/query-repair expectation with direct defeat injection followed by `advance_active_battle_by(0)` transaction finalization.
- Asserted that finalization consumes exactly one revision, removes the defeated deployment, creates its redeploy lock, and that the following `RequestBattleState` returns the same revision without mutation.
- Reran the complete combat module suite.

Changed files:

- `src/game/world/tests/combat.rs`

Commands and actual test counts:

- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`: 45 passed, 0 failed, 0 ignored, 596 filtered out.

Observed result:

- Initial battle projection is revision 1 and root/query projections agree.
- Deploy, pause, resume, speed, simulation tick, and battle VN progression each advance revision exactly once when state changes.
- Same-value Pause/Resume/Speed, blocked tick, `RequestBattleState`, and setup-loss recovery preserve revision.
- Same-seq/same-time Pause and both VN advance/close changes carry higher revisions.
- Setup-loss recovery returns immutable setup plus current state without destroying the active battle.
- New attempts on the same node use fresh battle UUIDs and restart at revision 1.
- Overflow rejects before playback mutation; insufficient deploy and repeated withdraw preserve revision and deployment bookkeeping.
- Existing 45 combat flows remain green.

Failure cause:

The previous 44/45 run failed only because the old test required query-side mutation after bypassing the transaction boundary.

Correction:

The test now drives the accepted authoritative finalization seam and separately proves read-only query behavior.

Decision and next step:

Treat the Core focused boundary as green. Begin the server connection-cursor/ordered-emission/acknowledgement migration, then return for broad Core validation after all shared DTO consumers compile.

## 2026-07-12 — Server authority cutover implementation started

Hypothesis:

One `PlayerGameActor` emission boundary can order command acknowledgement, setup/update/resync, tick deltas, and terminal snapshots while owning the connection event cursor.

Minimal experiment:

- Added per-connection `(battle_uuid, last_sent_seq)` state to `PlayerGameActor`.
- Passed its explicit cursor to Core command mutations and server ticks.
- Made successful player-command batches emit from `PlayerGameActor` in response → side messages → snapshot order; removed duplicate success sending from `PlayerGameSession`.
- Replaced destructive active-battle attach recovery with an attach result containing root snapshot plus immutable setup/current update.
- Added required revision to top-level `battle_update` and command acknowledgement mapping.

Changed files:

- `src/game/world.rs`
- `../game_server/src/game/player_game_actor/mod.rs`
- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`
- `../game_server/src/game/player_game_actor/session.rs`

Commands and actual test counts:

- No server check/tests executed after these edits yet; actual count 0 and not validation evidence.

Observed result:

- The intended single actor/outbox and attach-batch structure is present in source.
- Manual server fixtures/pattern matches still need compiler-guided migration for required revision and `AttachSessionResult`.

Failure cause:

Not evaluated yet.

Correction:

Run the smallest server compile check, record every failure, then migrate exact typed callers without aliases or optional revision fallback.

Decision and next step:

Run `cargo check -p game_server` before adding behavioral assertions.

## 2026-07-12 — Server production targets compile

Hypothesis:

The single-actor emission and cursor ownership changes are type-coherent in production server code.

Minimal experiment:

Ran the game_server package check after the first authority cutover.

Changed files:

- No additional files changed during this experiment.

Commands and actual test counts:

- `cargo check -p game_server`: passed.
- Tests executed: 0; not behavior evidence.

Observed result:

- Core and production game_server targets compile together with required revision fields and the new attach result.
- Test-only constructors and legacy behavioral expectations are not covered by `cargo check` and remain unproven.

Failure cause:

None for production compilation.

Correction:

None. Compile and execute the focused server actor/message/state tests next.

Decision and next step:

Run a nonzero player-game-actor focused suite, use its compile/runtime failures to migrate all fixtures and legacy ordering/reconnect expectations, and do not count this check as contract completion.

## 2026-07-12 — First server actor focused run fails on test-only migration

Hypothesis:

The focused actor suite will enumerate the remaining required-field and attach-contract test callers.

Minimal experiment:

Ran the `player_game_actor` server filter after production `cargo check` passed.

Changed files:

- None during the command.

Commands and actual test counts:

- `cargo test -p game_server player_game_actor -- --test-threads=1`: failed during test compilation; 0 tests executed.

Observed result:

- Eight compiler errors identify only test migration work:
  - three `AttachSessionResult` values still indexed as raw JSON,
  - one exhaustive `BattleUpdate` pattern omits `battle_state_revision`,
  - one top-level `BattleUpdate` and two `LiveBattleUpdateDto` constructors omit the required revision.
- The old active-battle attach test still expects destructive NodeConfirm recovery and must be rewritten to inspect the new InBattle snapshot plus setup/current-update batch.

Failure cause:

`cargo check` does not compile test-only fixtures, so their old shape survived the production cutover until this nonzero-intended test command.

Correction:

Migrate every fixture/pattern directly to the required revision and `AttachSessionResult`; rewrite reconnect assertions to preserve UUID/revision and ordered recovery messages. Do not add defaults, serde aliases, or optional fields.

Decision and next step:

Apply the compiler-owned test migration and rerun the exact same focused command. Preserve this 0-test failure as invalid success evidence.

## 2026-07-12 — Server actor suite exposes legacy multi-sender expectations

Hypothesis:

After required-field/attach migration, existing actor tests will distinguish command batches from later tick pushes.

Minimal experiment:

- Migrated test constructors and patterns to required revision.
- Rewrote active-battle attach assertions to expect InBattle root state plus setup/current-update recovery.
- Reran the exact focused actor suite.

Changed files:

- `../game_server/src/game/player_game_actor/messages.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- `../game_server/src/game/player_game_actor/handlers.rs`

Commands and actual test counts:

- `cargo test -p game_server player_game_actor -- --test-threads=1`: 24 executed, 22 passed, 2 failed, 1 filtered out.

Observed result:

- Required top-level update, nested resync, acknowledgement mapping, attach result, reconnect preservation, and 18 other actor/message/state tests passed.
- `live_battle_tick_pushes_update_without_state_snapshot_after_confirm_enter` and `paused_live_battle_tick_does_not_push_delta_until_resumed` failed because their probe assertions assumed command-side updates were returned only to `PlayerGameSession` and therefore absent from the actor socket.
- The new actor-owned sender correctly emits those command batches, so the tests must isolate command delivery from subsequent tick delivery.

Failure cause:

Legacy test observation was tied to the old two-sender implementation rather than the durable ordered message contract.

Correction:

Assert actor-socket order for command response → setup/update, clear the probe, then assert tick delta continuity or paused tick silence. Add revision/cursor assertions while making that separation.

Decision and next step:

Rewrite the two tests around observable ordered batches and rerun all 24 focused tests. Do not suppress actor command emission to preserve the old test shape.

## 2026-07-12 — Server cursor and ordered actor emission focused suite passes

Hypothesis:

Partitioning command and tick observation windows will prove one actor-owned outbox and cursor continuity without relying on the old absence-of-command-message behavior.

Minimal experiment:

- Asserted the actor socket receives battle start as command acknowledgement → setup → update, with acknowledgement/update revision 1.
- Cleared the observed command batch before scheduling tick/resync, then asserted the tick delta begins at the previously delivered sequence cursor.
- Asserted resync replays from requested sequence zero at the same current revision without regressing the actor cursor.
- Asserted pause acknowledgement precedes its update, cleared the batch, proved paused tick silence, then proved resume/tick uses a higher revision and non-regressing cursor.
- Renamed the reconnect test to describe preserved battle authority rather than destructive NodeConfirm recovery.

Changed files:

- `../game_server/src/game/player_game_actor/handlers.rs`

Commands and actual test counts:

- `cargo test -p game_server player_game_actor -- --test-threads=1`: 24 passed, 0 failed, 0 ignored, 1 filtered out in the lib target.
- The main binary target matched 0 tests and is not counted as validation evidence.

Observed result:

- Top-level update, nested resync, and command acknowledgement carry the same revision.
- The server connection cursor begins at zero, advances on emitted updates, does not regress after a lower-since-seq resync, and feeds the next tick delta.
- Successful player command and tick messages share the `PlayerGameActor` emission owner.
- Attach preserves the active Core battle and returns InBattle snapshot plus immutable setup/current revision update.
- Pause produces no automatic tick updates until resume.

Failure cause:

The previous failures were legacy observation assumptions about the former two-sender architecture.

Correction:

Tests now protect observable order, revision parity, and cursor continuity instead of sender-specific absence.

Decision and next step:

Treat the server focused boundary as green. Migrate Unity DTO/application/resync semantics and create an actual Rust-produced fixture consumed by Unity before broad server validation.

## 2026-07-12 — Unity revision-first implementation and Rust fixture generation

Hypothesis:

Unity can separate projection freshness from event continuity and presentation time without changing simulation presentation behavior.

Minimal experiment:

- Added `LastAppliedRevision`/`LastAppliedBattleRevision` and required positive `battle_state_revision` validation.
- Replaced seq/time freshness branches with ordinary revision dedupe followed by event-cursor continuity validation.
- Implemented a distinct authoritative resync path that permits equal current revision, rejects older revision, and does not destroy runtime on legacy nested setup.
- Updated the central Unity battle-update test factory, raw fixtures, mock runtime update, and PlayMode JSON to include revision.
- Added explicit higher/equal/lower revision, newer-revision gap, missing-revision, same-revision resync, and UUID-reset assertions.
- Added a typed Rust fixture generator and a Unity EditMode test that reads its checked-in output directly.

Changed files:

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs`
- `/mnt/f/unity projects/ark/Assets/Tests/EditMode/ArkClientContractTests.cs`
- `/mnt/f/unity projects/ark/Assets/Tests/PlayMode/ArkClientMockRenderTests.cs`
- `/mnt/f/unity projects/ark/Assets/Tests/Fixtures/battle_revision_contract.json` and Unity metadata
- `../game_server/examples/generate_battle_revision_fixture.rs`

Commands and actual test counts:

- `cargo run -q -p game_server --example generate_battle_revision_fixture -- '<Unity fixture path>'`: passed exact JSON value comparison.
- This generator check executes 0 tests; it is serialization provenance evidence, not Unity behavior evidence.
- Unity tests not run yet.

Observed result:

- Checked-in fixture exactly equals typed Rust serialization for top-level setup/update and nested resync envelopes.
- The fixture contains revision 1 unpaused state, same-seq/same-time revision 2 paused state, and same-revision resync repairing it to unpaused.
- No production Unity revisionless fallback or time-based staleness path remains in the edited runtime.

Failure cause:

No generator/fixture mismatch occurred.

Correction:

None yet. Compile and run Unity EditMode tests to catch C# or behavioral issues.

Decision and next step:

Run the smallest Unity EditMode contract suite using the installed project Editor, record all actual counts/failures, then correct without weakening required revision semantics.

## 2026-07-13 — Open-Editor Unity focused contract suite passes

Hypothesis:

The revision-first Unity runtime and checked-in Rust fixture compile and satisfy the complete existing EditMode contract suite in the already-open project Editor.

Minimal experiment:

- Connected Unity MCP to the single open `ark@b49d32f6f2f70bbe` instance running Unity 6000.3.4f1.
- Forced an all-asset refresh with script compilation and waited for the Editor readiness state after domain reload.
- Queried Console errors, then ran the full `ArkClient.Tests.EditMode.ArkClientContractTests` class through the Unity Test Framework.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- Unity MCP `refresh_unity(scope=all, mode=force, compile=request)`: completed; Editor returned idle/ready and Console error query returned 0 entries.
- Unity MCP EditMode filter `ArkClient.Tests.EditMode.ArkClientContractTests`: 182 executed, 182 passed, 0 failed, 0 skipped in 13.95 seconds.

Observed result:

- Unity compiled the current external project rather than a copied or second Editor instance.
- The actual-Rust JSON fixture test executed as part of the 182-test class and passed.
- Higher revision with unchanged event sequence/time applies; equal/lower ordinary revisions remain stale; gaps request resync; same-revision resync repairs authoritative state; UUID scope reset behavior remains green.
- No C# correction or contract weakening was required.

Failure cause:

None.

Correction:

None.

Decision and next step:

Treat the Unity focused boundary as green. Run the affected PlayMode suite, then perform broad Core/server/Unity validation, legacy/fallback searches, canonical document synchronization, and the required G01 completion review.

## 2026-07-13 — Affected Unity PlayMode suite passes

Hypothesis:

Required battle revision fields in mock/raw payloads and the revised runtime application seam preserve the existing visible mock-render gameplay flows.

Minimal experiment:

Ran the complete `ArkClient.Tests.PlayMode.ArkClientMockRenderTests` class in the same open Editor through Unity MCP.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- Unity MCP PlayMode filter `ArkClient.Tests.PlayMode.ArkClientMockRenderTests`: 49 executed, 49 passed, 0 failed, 0 skipped in 4.72 seconds.

Observed result:

- The updated raw/mock battle JSON composes with actual PlayMode rendering and interaction paths.
- No visible-flow regression or test-runner compilation error was reported.

Failure cause:

None.

Correction:

None.

Decision and next step:

Proceed to broad Core/server validation and the final legacy, serialization, canonical-document, and completion-review audit.

## 2026-07-13 — Broad Core all-targets suite passes

Hypothesis:

The completed Core transaction/revision cutover preserves the complete Core behavior and live-data loading baseline outside the focused combat module.

Minimal experiment:

Ran the complete game_core package across all targets with deterministic single-threaded test execution.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- `cargo test -p game_core --all-targets -- --test-threads=1`: 682 executed, 682 passed, 0 failed, 0 ignored.
- Count composition: 641 lib tests plus integration targets containing 3 + 3 + 18 + 13 + 4 tests. The binary target executed 0 tests and is not counted.

Observed result:

- The revision transaction, projection, live RON loading, skill runtime, and all existing Core gameplay flows remain green together.
- The prior Phase 0 Core count of 681 increases by one because G01 adds one net broad test; no baseline test disappeared.

Failure cause:

None.

Correction:

None.

Decision and next step:

Run game_server all-targets broad validation, then formatting/diff and legacy/canonical audits.

## 2026-07-13 — Broad game_server all-targets suite passes

Hypothesis:

The actor-owned cursor/emission and reconnect cutover preserves all server behavior while compiling the typed Rust fixture generator as an all-target.

Minimal experiment:

Ran the complete game_server package across all targets with deterministic single-threaded test execution.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- `cargo test -p game_server --all-targets -- --test-threads=1`: 25 executed, 25 passed, 0 failed, 0 ignored.
- The main binary and `generate_battle_revision_fixture` example targets each executed 0 tests and are not counted. Their compilation succeeded; the generator's behavior is separately covered by the earlier exact fixture comparison command.

Observed result:

- All Phase 0 server baseline tests and the new reconnect/revision/ordering coverage pass together.
- No server test was removed relative to the 25-test Phase 0 baseline.

Failure cause:

None.

Correction:

None.

Decision and next step:

Run formatting and scoped diff checks, search for superseded cursor/time/revisionless authority paths, synchronize canonical documents, and apply the completion review guide.

## 2026-07-13 — First formatting check and whole-Unity diff check fail

Hypothesis:

The completed cross-repository cutover has standard Rust formatting and no whitespace errors in changed files.

Minimal experiment:

Ran workspace rustfmt check and repository diff checks for Core, game_server, and Unity without modifying files.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- `cargo fmt --all -- --check`: failed; 0 tests executed.
- Core `git diff --check`: passed; 0 tests executed.
- game_server `git diff --check`: passed; 0 tests executed.
- whole Unity repository `git diff --check`: failed; 0 tests executed.

Observed result:

- rustfmt reports only mechanical layout differences in G01-touched Core/server Rust files, including `snapshot.rs`, `state.rs`, `world.rs`, combat tests, the fixture generator, and server actor tests.
- The Unity failure is dominated by unrelated pre-existing CRLF additions in `.gitignore` and `.idea`; it does not identify a G01 runtime/test/fixture path. Whole-repository Unity diff output is therefore not usable as G01 evidence.

Failure cause:

- Rust files have not yet had the standard workspace formatter applied after the final edits.
- The dirty Unity worktree contains unrelated CRLF files outside G01 scope.

Correction:

- Run `cargo fmt --all`, then rerun `cargo fmt --all -- --check` and Rust diff checks.
- Run Unity `git diff --check` restricted to the exact G01 runtime/test/fixture paths, preserving unrelated user-owned CRLF files.

Decision and next step:

Apply the mechanical Rust formatter and rerun the corrected scoped checks. Do not normalize or edit unrelated Unity `.gitignore`/`.idea` files.

## 2026-07-13 — Formatting and scoped whitespace checks pass

Hypothesis:

Applying standard rustfmt and restricting Unity diff checking to the exact G01 paths resolves the prior evidence failures without touching unrelated user files.

Minimal experiment:

- Applied the workspace Rust formatter.
- Reran rustfmt check and both Rust repository diff checks.
- Reran Unity diff checking only for G01 runtime, mock, EditMode, PlayMode, and fixture paths.

Changed files:

- Mechanical rustfmt changes in the previously identified G01-touched Rust files.
- G01 goal documents.
- No unrelated Unity CRLF file was changed.

Commands and actual test counts:

- `cargo fmt --all`: passed; 0 tests executed.
- `cargo fmt --all -- --check`: passed; 0 tests executed.
- Core `git diff --check`: passed; 0 tests executed.
- game_server `git diff --check`: passed; 0 tests executed.
- Unity `git diff --check -- <exact G01 affected paths>`: passed; 0 tests executed.

Observed result:

- Rust source now matches workspace formatting.
- All G01-scoped changed paths are free of whitespace errors.
- Unrelated Unity `.gitignore`/`.idea` CRLF state remains preserved.

Failure cause:

None on the corrected rerun.

Correction:

The mechanical formatter and exact-path scope described in the previous failed experiment.

Decision and next step:

Audit for legacy Core cursors, time/sequence freshness inference, revisionless producers, duplicate server emission authority, stale canonical wording, and compatibility paths.

## 2026-07-13 — Legacy and canonical audit finds one root/mock fallback seam

Hypothesis:

All old cursor/freshness/recovery authorities and revisionless producers were removed by the first cross-layer cutover.

Minimal experiment:

Searched Core/server runtime and DTOs, Unity runtime/mocks/tests, and the two external canonical Unity contract documents for old cursor names, revision fields, time/sequence freshness, setup-loss recovery, and active-battle discard wording.

Changed files:

- G01 goal documents only during this experiment.

Commands and actual test counts:

- Read-only `rg`/`sed` authority and canonical searches; 0 tests executed.

Observed result:

- Core no longer contains `last_pushed_event_log_seq` or `drain_battle_update_dto`; server owns the explicit connection cursor.
- Unity ordinary application compares revision first and uses event seq only for continuity; battle time is retained only for presentation.
- Top-level and nested live updates have required `u64` revision fields.
- One remaining fallback seam exists: root `GameStateContextDto::InBattle.battle_state_revision` is `Option<u64>`, and `ArkMockSnapshots` substitutes revision 1 if that root field is missing.
- `core_unity_battle_transport_contract.md` and `unity_core_contract.md` still prescribe active-battle discard and `node_confirm` recovery on reconnect/setup loss; the latter still shows `last_pushed_event_log_seq` and revisionless command acknowledgements.

Failure cause:

The first cutover made the live update strict but did not finish the root snapshot/mock producer or durable canonical wording.

Correction:

- Make the root InBattle revision a required positive `u64` sourced from the active battle invariant.
- Make the Unity mock require the root revision rather than fabricating 1.
- Replace stale canonical recovery/cursor/freshness/acknowledgement examples with the implemented PD-01 contract.

Decision and next step:

Apply the bounded accepted-contract corrections, then rerun Core focused/broad serialization, Unity compilation/EditMode/PlayMode, formatting/diff, and legacy searches as affected.

## 2026-07-13 — First Unity compile after mock strictness fails

Hypothesis:

The strict mock revision read compiles after replacing the fallback and adding revision 1 to the authored mock state.

Minimal experiment:

- Ran Core combat and server actor focused suites after the root DTO change.
- Refreshed/recompiled the already-open Unity Editor and queried Console errors.

Changed files:

- `src/game/behavior.rs`
- `src/game/world/snapshot.rs`
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs`
- G01 goal documents.

Commands and actual test counts:

- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`: 45 passed, 0 failed, 596 filtered out.
- `cargo test -p game_server player_game_actor -- --test-threads=1`: 24 passed, 0 failed, 1 filtered out; zero-test binary target excluded.
- Unity MCP refresh/compile: failed with one C# compiler diagnostic; 0 Unity tests executed.

Observed result:

- Rust root DTO strictness and server serialization consumers are focused-green.
- Unity CS0103 at `ArkMockSnapshots.cs:168`: `battleStateRevision` is read in `BattleMessagesForState`, but `MockBattleUpdate` is a separate method and did not receive it.

Failure cause:

Variable-scope/parameter omission in the mock helper, not a contract or data failure.

Correction:

Pass the already validated positive `ulong battleStateRevision` explicitly into `MockBattleUpdate` and recompile the same Editor.

Decision and next step:

Apply the parameter correction, refresh/recompile, and rerun both affected Unity suites. Preserve this zero-test compile failure as failed evidence.

## 2026-07-13 — Root/mock strictness correction passes cross-layer focused validation

Hypothesis:

Explicitly passing the validated positive revision into the mock update builder removes the last revision fallback without affecting runtime/visible behavior.

Minimal experiment:

- Passed `battleStateRevision` as a required `ulong` helper parameter.
- Recompiled the same open Unity Editor and queried Console errors.
- Reran the complete affected EditMode and PlayMode classes.

Changed files:

- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Mock/ArkMockSnapshots.cs`
- G01 goal documents.

Commands and actual test counts:

- Unity MCP refresh/compile: completed; Console errors 0.
- Unity MCP EditMode `ArkClient.Tests.EditMode.ArkClientContractTests`: 182 passed, 0 failed, 0 skipped.
- Unity MCP PlayMode `ArkClient.Tests.PlayMode.ArkClientMockRenderTests`: 49 passed, 0 failed, 0 skipped.
- Preceding affected Rust reruns in the failed experiment remain valid: Core combat 45/45 and server actor 24/24.

Observed result:

- Root InBattle JSON now requires a numeric revision by Rust type, and the mock requires/forwards that exact positive value.
- No mock fallback or revision fabrication remains.
- Actual Rust fixture consumption and visible mock-render flows remain green.

Failure cause:

None on the corrected rerun.

Correction:

Explicit helper parameter flow from the already validated root context value.

Decision and next step:

Update stale runtime comment and canonical external documents to the proven same-battle reconnect/setup recovery, revision ordering, acknowledgement, and cursor ownership contract. Then rerun broad affected validation and final legacy searches.

## 2026-07-13 — Canonical synchronization and final post-correction validation pass

Hypothesis:

After root/mock strictness and canonical synchronization, every G01 layer shares one revision/ordering/recovery contract with no old authority or fallback remaining.

Minimal experiment:

- Updated the stale Core behavior comment and the two external canonical Unity documents.
- Reran Core and server all-targets suites after the final DTO correction.
- Rechecked the actual Rust fixture, formatting, repository-scoped whitespace, and old authority/recovery/freshness patterns.
- Reused the immediately preceding post-correction Unity compilation and complete affected suite results.

Changed files:

- `src/game/behavior.rs`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- G01 goal documents.

Commands and actual test counts:

- `cargo test -p game_core --all-targets -- --test-threads=1`: 682 passed, 0 failed, 0 ignored; zero-test binary excluded.
- `cargo test -p game_server --all-targets -- --test-threads=1`: 25 passed, 0 failed, 0 ignored; zero-test binary/example excluded.
- Unity MCP EditMode `ArkClientContractTests`: 182 passed, 0 failed, 0 skipped.
- Unity MCP PlayMode `ArkClientMockRenderTests`: 49 passed, 0 failed, 0 skipped.
- `cargo run -q -p game_server --example generate_battle_revision_fixture -- '<Unity fixture path>'`: exact JSON comparison passed; 0 tests, provenance evidence only.
- `cargo fmt --all -- --check`: passed; 0 tests.
- Core and game_server `git diff --check`: passed; 0 tests.
- exact G01 Unity path `git diff --check`, including canonical docs: passed; 0 tests.
- Final read-only legacy/canonical `rg` searches: completed; 0 tests.

Observed result:

- Core/server broad counts remain 682 and 25 after making root revision required.
- The Rust fixture still exactly matches typed serialization and the Unity consumer suite remains green.
- No `last_pushed_event_log_seq`, `drain_battle_update_dto`, optional root revision, destructive setup-loss comment, mock `?? 1`, or Unity time/seq freshness comparison remains.
- Canonical docs now specify initial revision 1, one commit per authoritative projection mutation, ack/update parity, stale/gap rules, same-revision resync, fresh UUID per battle, and preserved active-battle reconnect/setup recovery.
- The only search matches resembling old phrases are explicit negation (“do not discard active battle”) and a negative presentation-clock initialization guard; neither is freshness authority.

Failure cause:

None on final validation.

Correction:

The bounded root/mock strictness and canonical synchronization recorded in the preceding experiments.

Decision and next step:

Read and apply `docs/goal_completion_review_guide.md` against current runtime/data/DTO/server/Unity/tests/legacy evidence. Do not mark G01 complete before the review verdict.

## 2026-07-13 — G01 completion review passes

Hypothesis:

Current runtime/data/DTO/server/Unity/canonical evidence satisfies every G01 completion condition with no immediate correction.

Minimal experiment:

Read the complete completion review guide and applied its per-item, long-term direction, cross-component, validation, and legacy/fallback checks to DCA-C-01, CPR-C-01, and PD-01.

Changed files:

- `docs/goals/battle_projection_revision_contract/COMPLETION_REVIEW.md`
- G01 and Master goal status/progress documents.

Commands and actual test counts:

- Review consumed the final previously recorded nonzero evidence: Core 682, server 25, Unity EditMode 182, Unity PlayMode 49; no extra tests were run solely to rewrite the same evidence.

Observed result:

- Primary verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- No immediate correction, unresolved policy, compatibility layer, stale canonical contract, or missing required runner remains.

Failure cause:

None.

Correction:

None.

Decision and next step:

Mark G01 complete, update the Master registry, and begin G06 according to the Phase 1 dependency graph. R01 retains an independent cross-goal review obligation after G06/G07/G12.

## Experiment Template

### YYYY-MM-DD — Title

Hypothesis:

Minimal experiment:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Correction:

Decision and next step:
