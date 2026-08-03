# Battle Capability And Reconnect Authority Experiments

## 2026-07-13 — Goal initialization

Hypothesis:

Completed G01 revision authority and G06 typed capability authority provide the two bounded seams
needed to implement G07 without changing later gameplay policy.

Minimal experiment:

- Read the startup instruction, Master G07 findings/PD-01/PD-02/Phase 1 sections, R00 review, G01
  completion review, and G06 completion review.
- Created all three G07 documents before G07 code search, modification, or validation.

Changed files:

- G07 goal documents only.

Commands and actual test counts:

- Documentation initialization only; 0 tests executed and no implementation success claimed.

Observed result:

- G07 is unblocked and its battle-specific boundary is explicit.
- Exact current DTO placement, validators, and Unity/server consumers still require source-order
  tracing.

Failure cause:

- None.

Correction:

- None.

Decision and next step:

Read all three G07 documents at the next resume/start boundary, then trace runtime through Unity
before editing implementation.

## Experiment Template

## 2026-07-13 — Initial semantic search rate-limited

Hypothesis:

Repository-level semantic search can quickly locate the complete Core/server/Unity battle authority
flow before bounded source reads.

Minimal experiment:

Ran one natural-language codebase search per repository for the battle validation/revision,
transport/reconnect, and Unity presentation/submission flows.

Changed files:

- G07 goal documents only after the failure.

Commands and actual test counts:

- Three semantic search calls: all returned HTTP 429; 0 tests.

Observed result:

No search result was produced and no source claim can be made from this attempt.

Failure cause:

The semantic search service rate-limited all three concurrent requests.

Correction:

Use the explicitly assigned read-only Warp Grep subagent and bounded `rg`/source reads. Do not retry
concurrently or reinterpret the failure as evidence.

Decision and next step:

Proceed with source-order tracing using available local tools and the open Unity MCP.

## 2026-07-13 — First bounded cross-layer authority trace

Hypothesis:

Current battle DTOs already contain useful bounded domain state, but static Unity action synthesis
and missing expected authority leave capability and stale-command gaps.

Minimal experiment:

- Inspected Core action types, update/checkpoint/deployment DTOs, execute routing, playback/query/
  recovery/deploy/withdraw/skill handlers, and revision commit helpers.
- Inspected server client/server envelopes, attach/cursor/order mapping, result side messages, and
  current bridge documents.
- Inspected Unity store/runtime, command bus/gateway/sender, battle composer/action handler, local
  deployment validation, and the live MCP test resource. Queried the live Unity Console earlier in
  the same Editor session; it remains the validation path after implementation.
- Recorded scoped dirty-worktree evidence for all three repositories.

Changed files:

- G07 goal documents only.

Commands and actual test counts:

- Read-only `rg`, `sed`, `git status`, Unity MCP resource inspection: 0 tests.

Observed result:

- Complete battle projection has no capability field.
- Mutation handlers validate real payloads and commit revisions, but no expected UUID/revision
  arrives with commands.
- Server ordering remains G01-compliant and transparent.
- Unity replaces capability with a static set and independently derives deployment, cost, playback,
  and partial skill affordances.
- Existing deployment/readiness DTOs are the likely bounded foundation; exact typed eligibility and
  selected preflight composition still require a validator/action matrix.

Failure cause:

- No source/test failure. The earlier semantic-search 429 is recorded separately.

Correction:

- None yet.

Decision and next step:

Finish the validator and payload ownership matrix, identify the smallest expected-authority execute
context, then run focused baseline tests before implementation.

## 2026-07-13 — Focused battle authority baseline

Hypothesis:

Current G01 revision/reconnect behavior and the existing Unity battle interaction tests are green
before G07 changes, while their old capability expectations can be classified explicitly.

Minimal experiment:

- Ran current live-defense Core tests, live-battle server tests, and six targeted Unity EditMode
  tests through the open MCP Editor.

Changed files:

- G07 goal documents only after results.

Commands and actual test counts:

- First parallel Core/server attempt became inconclusive after both waited on shared Cargo locks;
  no test count or success is claimed from that attempt.
- `cargo test -p game_core --lib live_defense_ -- --test-threads=1`: 9 passed, 0 failed,
  619 filtered out.
- `cargo test -p game_server --lib live_battle -- --test-threads=1`: 5 passed, 0 failed,
  21 filtered out.
- Unity MCP six named EditMode tests: 6 passed, 0 failed, 0 skipped.

Observed result:

- G01 pause/speed/recovery/revision and server tick/resync/final ordering baselines are green.
- Unity static runtime capability, presenter, query envelope, gap/resync, and deployment preview
  baselines are green.
- `BattleRuntimeActionsUseBattleTransportReadinessInsteadOfStaleSnapshotAllowedActions` and
  `BattlePresenterEnablesBattleCommandsFromBattleRuntimeReadiness` protect the superseded local/
  static contract and must be replaced with Core capability behavior tests.

Failure cause:

- The first concurrent Cargo run contended on the shared target directory and returned without a
  usable final result to the orchestrator.

Correction:

- Reran Core then server sequentially and recorded nonzero results.

Decision and next step:

Use these counts as the G07 baseline. Complete the ownership matrix and begin the smallest reversible
Core projection/expected-authority trial.

## 2026-07-13 — First expected-authority Core trial

Hypothesis:

One typed `BattleStateAuthorityDto` passed through a Core transport execution context can reject a
stale command before playback mutation without changing existing callers during the reversible
trial.

Minimal experiment:

- Added the authority DTO and stale error.
- Added `execute_with_transport_context` while the existing execute seam supplies no authority during
  the trial.
- Compared supplied authority with the active battle before the normal gate/domain path.
- Added one stale/current Pause test checking zero stale side effects and one successful commit.

Changed files:

- `src/game/behavior.rs`
- `src/game/world.rs`
- `src/game/world/tests/combat.rs`
- G07 goal documents

Commands and actual test counts:

- `cargo fmt --all -- --check`: passed; 0 tests.
- Focused stale-authority test: compile failed before test execution; 0 tests.

Observed result:

Two exhaustive functions in `src/game/world/snapshot.rs` did not cover
`StaleBattleStateAuthority`.

Failure cause:

The trial introduced a new `GameError` variant but had not yet added its canonical code/message to
snapshot-side error formatting.

Correction:

Add exact `stale_battle_state_authority` code/message arms, then rerun the one focused test. Do not
weaken the exhaustive matches.

The first rerun after that correction compiled but executed 0 tests because `--exact` was paired
with an unqualified function name. This is explicitly invalid evidence. Rerun without `--exact` or
with the complete module-qualified name.

Corrected command and actual count:

- `cargo test -p game_core --lib supplied_stale_battle_authority_rejects_before_playback_mutation -- --test-threads=1`: 1 passed, 0 failed, 628 filtered out.

Observed correction result:

- Supplied stale revision rejects before playback or revision changes.
- Supplying the exact current UUID/revision permits Pause and commits exactly one new revision.

Next step:

Thread the optional authority through typed server transport, then atomically make it mandatory for
active battle mutations only after all Core/server/Unity callers and tests are migrated.

Server transport trial result:

- Added optional typed `battle_authority` to the client command envelope, internal transport command,
  session mapping, actor call, and stale error mapping.
- `cargo check -p game_server`: passed; 0 tests.
- `cargo test -p game_server --lib --no-run`: failed before tests because 10 handler test fixtures
  construct `ExecutePlayerBehavior` without the new field.

Failure cause and correction:

- Production mapping is complete for the optional trial, but typed test literals correctly require
  explicit migration. Add `battle_authority: None` to non-mutation/query/setup fixtures during the
  trial; later mutation fixtures will use exact current authority when missing authority becomes
  mandatory. Do not hide the field behind a compatibility constructor.

Revalidation:

- First fixture migration reduced the compile inventory from 10 missing fields to 2 duplicate
  enter-battle fixtures; those two were then made explicit.
- `cargo test -p game_server --lib --no-run`: passed; 0 tests, compile evidence only.
- `cargo test -p game_server --lib deserializes_live_battle_requests -- --test-threads=1`: 1 passed,
  0 failed, 25 filtered out.

The optional cross-layer Rust transport trial is viable. Add revisioned battle capability/payload
projection before the mandatory no-missing-authority cutover so Unity sends authority only from an
actually applied update.

## 2026-07-13 — First revisioned battle capability slice

Hypothesis:

The existing final battle VN attachment seam can also attach Core-projected typed context
capabilities to every revisioned update without a separate cache/message.

Minimal experiment:

- Added `action_capabilities` to `LiveBattleUpdateDto`.
- Low-level session projection initializes it empty; the GameCore final projection seam replaces it
  after current VN state is known.
- Added current query/recovery, blocking-VN, playback, speed, deployment-family, withdrawal, skill,
  retreat, and mutually exclusive VN family projection from current authoritative state.

Changed files:

- `src/game/behavior.rs`
- `src/game/world.rs`
- `src/game/world/state.rs`
- `src/game/world/visual_novel_flow.rs`
- `src/game/world/battle_capability.rs`
- G07 goal documents

Commands and actual test counts:

- `cargo fmt --all -- --check`: failed on mechanical layout only; 0 tests.
- Playback focused test then executed 1 and failed before battle entry: the pre-run start helper
  received `InvalidAction`.

Observed result:

The failure occurred before the new battle update projector ran. It may be transient/shared-state or
an unintended execute-path interaction; current evidence is insufficient.

Failure cause:

Not yet established.

Correction:

Apply rustfmt, rerun the exact focused test in isolation, then trace only if it reproduces. Do not
change the new projection or unrelated pre-run policy without evidence.

Isolated rerun reproduced the same 0/1 failure. Source trace found the exact cause: capability
projection was evaluated before the `BehaviorResult` match, while the attachment function is called
for every command. Therefore RunStart called a battle-only projector and received `InvalidAction`.

Correction: evaluate battle capability only inside the battle-result match arm, alongside assignment
to `LiveBattleUpdateDto`. This preserves the shared finalizer without making non-battle commands
depend on active battle state.

Revalidation:

- `cargo fmt --all -- --check`: passed.
- Playback capability/revision focused test: 1 passed, 0 failed, 628 filtered out.
- Blocking battle VN capability/recovery focused test: 1 passed, 0 failed, 628 filtered out.

Observed correction result:

- Unpaused update exposes Pause but not Resume; paused update exposes Resume but not Pause.
- Query and recovery scopes remain enabled.
- Blocking battle VN exposes exactly Advance, RequestBattleState, and RecoverBattleSetupLoss while
  the tick remains blocked.

Decision and next step:

Preserve this finalizer seam, transport the field through game_server, then add typed bounded payload
eligibility before Unity cutover.

Server field transport compile:

- Added `action_capabilities` to top-level `PlayerGameServerMessage::BattleUpdate` and its typed
  mapper. Nested `BattleResync` already preserves the complete typed update.
- `cargo test -p game_server --lib --no-run` failed with four test-only migrations: one exhaustive
  BattleUpdate pattern, one top-level message fixture, and two `LiveBattleUpdateDto` fixtures.
- Next correction is to populate and assert the field in serialization/real-flow tests, using empty
  vectors only for isolated dummy mapping fixtures where capability content is not under test.

Revalidation:

- Server library `--no-run`: passed after explicit fixture/pattern migration; 0 tests.
- Formatting initially reported one import-layout diff; `cargo fmt --all` corrected it.
- `cargo test -p game_server --lib serializes_battle_ -- --test-threads=1`: 3 passed, 0 failed,
  23 filtered out.

Decision and next step:

The revisioned capability field now crosses Core/server update and nested resync. Implement typed
payload eligibility and exact projector/preflight parity before migrating Unity away from its static
set.

## 2026-07-13 — Unity revisioned capability consumer cutover

Hypothesis:

`BattleRuntimeState` can atomically parse/store the update-owned capability vector and
`GameSnapshotStore` can remove its static battle substitution without affecting root capability.

Minimal experiment:

- Added strict battle capability parsing, duplicate rejection, reset, and same-revision resync apply
  to `BattleRuntimeState`.
- Routed battle `TryGetActionCapability` to that revisioned store.
- Deleted `battleRuntimeActions` and `ActionCapability.BattleCompatibility`.
- Refreshed/compiled through the open Unity MCP and ran four focused tests.

Changed files:

- Unity `GameSnapshotStore.cs`, `BattleRuntimeState.cs`, `ActionCapability.cs`
- Unity EditMode fixtures/tests
- G07 goal documents

Commands and actual test counts:

- Unity MCP script refresh/compile: Console reports 0 errors; 0 tests.
- First focused EditMode run: 4 executed, 0 passed, 4 failed.

Observed result:

All four old fixtures were rejected because their battle updates lacked `action_capabilities`; route,
revision, and static-action assertions consequently observed no applied runtime.

Failure cause:

Hand-built `BattleUpdateMessage` encoded the pre-G07 shape. Two tests also explicitly expected every
static action/VN action enabled.

Correction:

- Make the shared typed test builder emit explicit revisioned capability objects.
- Replace the static-whitelist test with Core-update ownership expectations: Pause but not Resume,
  no VN action without a VN projection, recovery/query scopes present, and root Confirm ignored.
- Rerun the same four tests through MCP.

Decision and next step:

Complete fixture migration without adding an optional parser/default. Then expand strict negative
and actual Rust fixture tests before payload eligibility cutover.

Revalidation:

- Unity MCP compile after fixture/test migration: 0 Console errors.
- Same four focused EditMode tests: 4 passed, 0 failed, 0 skipped.
- The replacement test proves update-owned Pause/Deploy/etc capability, Resume/VN absence, recovery
  presence, and root Confirm non-leakage.

Unity expected-authority send slice:

- Added battle-mutation identity classification, typed envelope emission from the exact applied
  battle UUID/revision, and no authority for RequestBattleState query.
- Unity MCP compile: 0 Console errors.
- First result poll disconnected transiently while the Editor job continued; retrying the same job
  ID returned success.
- Mutation-envelope and query-envelope focused EditMode tests: 2 passed, 0 failed, 0 skipped.

Decision and next step:

Core/server/Unity now agree on optional authority transport. Add ordered stale error/resync behavior
and migrate all battle mutation fixtures before making missing authority a hard Core rejection.

Stale authority ordered recovery slice:

- game_server now catches the typed stale Core error inside the actor, obtains a read-only current
  projection, and emits request-scoped Error followed by BattleResync through the existing ordered
  delivery boundary.
- `cargo test -p game_server --lib stale_battle_authority_emits_error_then_current_resync_without_mutation -- --test-threads=1`: 1 passed, 0 failed, 26 filtered out.
- The test proves current revision remains 1, playback remains unpaused, and probe delivery order is
  exactly Error then BattleResync.
- Unity store now raises its existing resync request event on `stale_battle_state_authority` using
  `LastAppliedSeq`.
- Unity MCP compile: 0 Console errors.
- Unity stale-authority resync EditMode test: 1 passed, 0 failed, 0 skipped.

Decision and next step:

The stale path is end-to-end at the transport seam. Add mandatory missing-authority rejection and
then bounded payload eligibility before broad fixture migration/testing.

Mandatory authority cutover slice:

- Added exhaustive battle-mutation identity and `MissingBattleStateAuthority`.
- Core validates only active battle mutations; query/recovery remain authority-free.
- Added a test-only helper that reads current authority explicitly for mutation tests; no production
  auto-fill/fallback was added.
- Core focused playback: 1 passed; blocking VN: 1 passed; missing/stale/current authority: 1 passed.
- The first combined format/test command reported mechanical test layout diffs while the test passed;
  `cargo fmt --all` corrected them.
- Updated server pause/resume actor fixture to pass revision 1 then the committed paused revision.
- Server paused tick test: 1 passed, 0 failed, 26 filtered out.
- That compile emitted two unused-variable warnings caused by an over-broad fixture edit in unrelated
  enter tests; both bindings were reverted immediately.

Decision and next step:

Migrate remaining Core/server battle mutation tests to explicit authority during broad runs, then
finish typed payload eligibility and Unity local-inference removal.

First broad Core result after mandatory authority:

- `cargo test -p game_core --lib -- --test-threads=1`: 629 executed, 611 passed, 18 failed,
  0 ignored.
- Seventeen failures are combat tests directly calling Deploy/Withdraw/Skill/Pause/Speed/VN/Retreat
  without the new authority; one is the Event-combat retreat map-flow test doing the same.
- The revision-overflow test now stops at missing authority, so it must supply the exact current
  max-revision authority to continue proving overflow atomicity.

Failure cause:

Expected test-call migration from removing the production missing-authority fallback. No failing
trace reached a domain mutation with valid authority.

Correction:

Replace direct battle mutation calls with the explicit current-authority test helper. Do not alter
query/recovery calls or add automatic authority to production `execute`.

Decision and next step:

Migrate the 18 tests and rerun the full library suite before proceeding to payload eligibility.

Decision and next step:

Reproduce first. If green, add direct capability assertions; if red, trace execute/pre-run state
before further G07 implementation.

Decision and next step:

Complete the bounded error mapping correction and revalidate before server/Unity transport work.

### YYYY-MM-DD — Title

Hypothesis:

Minimal experiment:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Correction:

Decision and next step:

### 2026-07-13 — Mandatory-authority broad migration rerun

Hypothesis:

After migrating the first broad failure inventory to the explicit test-only current-authority
helper, any remaining failures should be bounded fixture/call-site omissions rather than runtime
behavior regressions.

Minimal experiment:

Run the entire Core library suite serially with quiet output so every remaining failure is visible.

Changed files:

- None before the rerun.

Commands and actual test counts:

- `cargo test -q -p game_core --lib -- --test-threads=1`: 629 executed, 626 passed, 3 failed,
  0 ignored.

Observed result:

All three failures returned `MissingBattleStateAuthority` from direct `DeployUnit` calls in the live
RON defense route, defeated-unit redeployment, and deployment-cost-reduction tests.

Failure cause:

The three test call sites had not yet been migrated to the explicit authority-bearing helper. No
domain mutation ran and no different gameplay assertion failed.

Correction:

Replace only those direct mutation calls with `execute_current_battle_mutation`; keep production
missing-authority rejection unchanged.

Decision and next step:

Rerun all 629 Core library tests, then proceed to exact typed payload eligibility only when the
mandatory-authority migration is green.

Follow-up result:

- `cargo fmt --all -- --check`: failed on one indentation-only rustfmt diff; no tests ran.
- `cargo fmt --all`: succeeded.
- `cargo test -q -p game_core --lib -- --test-threads=1`: 629 executed, 628 passed, 1 failed,
  0 ignored.

The remaining failure was a second, conditional third-employee deploy inside the already identified
live RON defense test. It is the same missing-authority migration class; migrate that call and rerun.

Final correction result:

- `cargo fmt --all -- --check`: succeeded.
- `cargo test -q -p game_core --lib -- --test-threads=1`: 629 executed, 629 passed, 0 failed,
  0 ignored.

Mandatory battle-authority migration is green without a production fallback. Proceed to typed
payload eligibility and shared preflight parity.

### 2026-07-13 — Domain-owned typed payload eligibility trial

Hypothesis:

Bounded payload eligibility can live beside its owning battle DTO instead of creating a Cartesian
action list: deployment candidates own cost/cooldown eligibility, deployed units own withdraw/skill
eligibility, and playback owns Pause/Resume/speed-option eligibility. Context capability can then be
derived from those same DTO facts.

Minimal experiment:

Introduce one stable typed payload eligibility shape and reason enum, project it in the existing
revisioned checkpoint, reject already-active playback payloads, and replace the target-missing skill
expectation with the accepted disabled-payload contract.

Changed files:

- `src/game/behavior.rs`
- `src/game/world/state.rs`
- `src/game/world/battle_capability.rs`
- `src/game/world/combat.rs`
- `src/game/battle/core/sim.rs`
- `src/game/world/tests/combat.rs`

Commands and actual test counts:

- `cargo check -p game_core`: succeeded.
- `cargo test -p game_core --lib --no-run`: compiled the 629-test library target.
- `cargo test -q -p game_core --lib playback -- --test-threads=1`: initially 4 executed,
  3 passed, 1 failed because the old test expected repeated Pause to succeed as a no-op.
- Same playback command after replacing the obsolete expectation: 4 executed, 4 passed.
- `cargo test -q -p game_core --lib manual_fragment_readiness_disables_payload_when_target_is_missing -- --exact`:
  0 executed; invalid as evidence because the full module path was omitted.
- Same filter without `--exact`: 1 executed, 1 passed.

Observed result:

The DTO shape compiles and the focused playback/target-missing behavior matches the accepted policy.
The old repeated Pause/Resume test was the expected legacy coarse behavior. The initial skill command
matched zero tests and was explicitly rerun correctly.

Failure cause:

Playback failure: stale no-op contract. Zero-test skill command: incorrect exact test-name filter.

Correction:

Repeated current playback payloads now return typed `already_active` without mutation. Skill
readiness is being refactored to call the same manual-cast validator used by execution, with stable
typed reasons instead of free-form strings.

Decision and next step:

Complete shared deployment and skill preflight parity, add exact DTO/handler reason tests, then
compile the server transport to enumerate every required constructor/fixture migration.

Follow-up parity and transport results:

- `cargo fmt --all -- --check`: failed on rustfmt-only layout after the skill validator refactor;
  no tests ran. `cargo fmt --all` corrected it.
- Target-missing skill projection/handler parity: 1 executed, 1 passed.
- Redeploy cooldown projection/handler parity: 1 executed, 1 passed.
- Insufficient stabilization projection/handler parity: 1 executed, 1 passed.
- Current-speed projection/handler parity plus fractional tick preservation: 1 executed, 1 passed.
- `cargo check -p game_server`: succeeded.
- First `cargo test -p game_server --no-run`: failed to compile. Four checkpoint fixtures lacked
  `playback_eligibility`; the actual Rust→Unity fixture generator also still lacked G06
  `action_capabilities`. No tests executed.
- After migrating those explicit constructors and the generator,
  `cargo test -p game_server --no-run`: succeeded; both server test executables compiled.

The compile failure was useful schema inventory, not runtime evidence. No serde default or fallback
was added; every producer/fixture now has to construct the current DTO explicitly.

### 2026-07-13 — Unity typed eligibility cutover through MCP

Hypothesis:

Unity can consume only revisioned Core payload eligibility for battle controls while retaining local
tile geometry solely as a visual highlight hint. Missing/malformed payload eligibility should reject
the entire update and request resync, matching the strict action-capability behavior.

Minimal experiment:

Use the live `ark@b49d32f6f2f70bbe` Unity MCP instance to add strict checkpoint validation, typed
readers, payload-aware command/body affordances, and selected deployment preview eligibility; remove
local cost execution gating and local selected-tile validation as command authority.

Changed Unity files:

- `BattleRuntimeState.cs`
- `DeploymentModel.cs`
- `ActionCapability.cs`
- `GameScreenCompositionContext.cs`
- `GameScreenViewModel.cs`
- `BattleScreenComposer.cs`
- `BattleActionHandler.cs`
- `BattlefieldDeploymentModel.cs`
- `BattlefieldPendingDeploymentRangePreview.cs`
- `ArkClientContractTests.cs`

Commands and actual test counts:

- First MCP compile: 5 errors. Three were the new body-action disabled-reason field not yet present;
  two were nullable-bool expressions. No tests ran.
- After correction, MCP compile: 4 errors, all stale tests still calling removed local
  `CanAffordDeployCost`/`TryValidateDeployment`. No tests ran.
- After current-contract test migration, MCP compile: 0 errors.
- First focused test request used the wrong namespace and executed 0 tests; invalid evidence.
- Second focused request also omitted `.EditMode` and executed 0 tests; invalid evidence.
- Correct focused request: 5 executed, 3 passed, 2 failed. Failures were missing stable reason
  localization and an obsolete root fixture without eligibility.
- After migration: 5 executed, 5 passed.
- The deployment reader test was accidentally no longer attributed `[Test]` by a whole-method MCP
  replacement and therefore was absent from that five-test run. Attribute restored; exact rerun:
  1 executed, 1 passed.

Observed result:

Unity compiles with zero errors and six focused contract behaviors pass. Current speed is disabled,
insufficient-cost roster actions are disabled from Core eligibility, latest authoritative eligibility
drives drag affordance, and selected preview parses Core deploy eligibility.

Failure cause:

Expected stale test/mocks plus one MCP structured-edit boundary overlap: multiple anchors targeting
the same class region briefly duplicated/corrupted the UnitDeployCostView boundary despite the tool's
structural validation reporting success. Direct file reread caught it before compilation.

Correction:

Replaced the affected class boundary atomically, reread it, compiled in Unity, migrated stale tests,
and restored the lost test attribute. No compatibility parser or default-enabled path was added;
missing eligibility is disabled and revisioned updates are rejected/resynced.

Decision and next step:

Run the full Unity EditMode suite to obtain the complete stale fixture inventory, migrate only current
wire fixtures, then validate an actual Rust-generated JSON fixture before PlayMode/broad Rust suites.

Full EditMode inventory:

- Unity MCP EditMode: 187 executed, 167 passed, 20 failed, 0 skipped.
- Nine BattleActionHandler failures and five presenter failures report
  `missing_core_eligibility` from stale root/mock deployment fixtures.
- Two battle update continuity tests did not apply their projection because hand-built deployment
  collections are objects instead of current arrays.
- One presenter speed test still expects all four speed payloads enabled.
- Two presenter selection tests fail downstream because stale mock payload eligibility suppressed the
  expected controls.
- The actual Rust-generated revision fixture test applies zero updates because the checked-in fixture
  has not yet been regenerated from the now-current generator.

No failure indicates that a well-formed current eligibility payload was accepted with the wrong
meaning. Migrate the shared mocks/builders and regenerate actual Rust JSON; do not localize or enable
`missing_core_eligibility` as a fallback.

Second full EditMode result after shared builder/mock and Rust fixture migration:

- Unity MCP EditMode: 187 executed, 177 passed, 10 failed, 0 skipped.
- The Rust-generated fixture regression is fixed and no longer fails.
- Three deployment-handler tests still assert the removed local affinity/occupancy rejection instead
  of sending a selected Core preview request.
- One contiguous-update test still injects a malformed deployed-unit fixture.
- Six presenter tests still consume stale mock/direct deployment entries without typed eligibility,
  including one minimal Pause fixture missing playback eligibility.

Continue migration; do not reintroduce local selected-payload validation to satisfy the three obsolete
tests.

Final EditMode migration sequence:

- Third full run: 187 executed, 181 passed, 6 failed. The three obsolete local-validator tests and
  malformed update were fixed; remaining failures were presenter handling of deployed roster
  membership plus one stale skill fixture.
- Deployed employees are now recognized as absent-by-design from deploy candidate eligibility rather
  than treated as malformed missing candidates.
- The minimal battle presenter fixture was moved from root battle capability to an actual revisioned
  BattleSetup/BattleUpdate, matching the G06/G07 authority boundary.
- The selected-unit HP fixture gained current withdraw and skill eligibility.
- Final Unity MCP EditMode: 187 executed, 187 passed, 0 failed, 0 skipped.
- Unity editor compile before the final run: 0 errors.
- `cargo run -q -p game_server --example generate_battle_revision_fixture`: generated current typed
  Rust JSON.
- The generated JSON was checked with the example's exact equality mode against the staged fixture:
  succeeded, then copied to Unity's checked-in fixture.

Obsolete local occupied-tile, affinity-mismatch, and missing-affinity tests were replaced with tests
that require a Core deployment-preview request and consume its rejection. Local highlight geometry
was not restored as execution authority.

Broad Rust and PlayMode results:

- `cargo fmt --all -- --check`: succeeded.
- `cargo test -q -p game_core --lib -- --test-threads=1`: 629 executed, 629 passed.
- `cargo test -q -p game_server -- --test-threads=1`: 27 executed, 27 passed; two additional
  zero-test binary/doc groups were reported and are not counted as evidence.
- First full Unity PlayMode: 53 executed, 43 passed, 10 failed. The first failure was a stale raw
  battle fixture missing capabilities/eligibility; the remaining nine were downstream UI scene
  contamination after the failed test did not clean up.
- After migrating that fixture, its exact focused run initially failed on stale deployment collection
  shape, then on `deployment: null` being incompatible with the test's battlefield reader. A complete
  empty current deployment DTO fixed it; exact run 1/1 passed.
- Next full PlayMode: 53 executed, 44 passed, 9 failed, confirming the first contract failure was
  fixed but the battle mock render test itself still failed later on an unrelated stale selected-card
  assertion, again contaminating following tests.
- The stale selected-card assertion was removed from this battle-action/result CTA test; selection is
  already covered by dedicated EditMode contracts. Exact test: 1/1 passed.
- Final Unity PlayMode: 53 executed, 51 passed, 0 failed, 2 explicitly skipped live-server tests.
- Unity editor compilation immediately before final PlayMode: 0 errors.

Decision and next step:

Run Core/server all-targets, audit diffs/legacy terms and current wire docs, then perform the mandated
G07 completion review before changing child-goal status.

All-targets results:

- `cargo test -q -p game_core --all-targets -- --test-threads=1`: nonzero groups
  629 + 3 + 3 + 18 + 13 + 4 = 670 passed, 0 failed; one zero-test group not counted.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`: 27 passed, 0 failed; three
  zero-test groups not counted.

Proceed to diff/legacy/canonical audit and completion review.

### 2026-07-13 — Scoped legacy and canonical contract audit

Hypothesis:

The runtime cutover removed the old battle authority paths, but the canonical server bridge document
may still describe the pre-G01/G06/G07 snapshot and two-message model.

Minimal experiment and commands:

- Scoped `git status --short` and `git diff --stat` over G07/Core paths.
- `rg -n` for `battleRuntimeActions`, `CanAffordDeployCost`, `TryValidateDeployment`,
  `can_activate_reason`, `target_block_reason`, `allowed_actions`, `ActionValidator`, current typed
  eligibility fields, and `battle_authority` across Core, game_server, Unity scripts/tests, and
  non-goal docs.
- Read `game_server/docs/unity_bridge_protocol.md` and
  `docs/goal_completion_review_guide.md` directly.

Result:

- No removed static whitelist, local affordability/deploy validator, or free-form skill readiness
  field remains in active Core/server/Unity runtime paths.
- The only Unity `allowed_actions` occurrence is a negative contract assertion proving the removed
  root field is absent.
- Historical audit/tmp documents still describe the original findings and are intentionally not
  rewritten as runtime contracts.
- The canonical bridge document was stale: it called root snapshot actions authoritative during
  battle and described only a generic two-message response flow.

Correction:

Updated the canonical bridge contract to document the typed command-envelope battle authority,
revisioned battle side-message projection, payload eligibility, atomic Unity consumption, and
ordered typed `error -> battle_resync` stale recovery. No runtime or compatibility behavior changed.

Next judgment:

Add focused Unity negative parsing tests for malformed payload eligibility, then run their nonzero
MCP result before the completion review.

Follow-up source-of-truth correction:

The MCP-read `BattleRuntimeState` accepted a deployed unit without `skill_readiness`, while the Core
producer always returns a readiness DTO for every valid deployed player unit (using disabled typed
eligibility when no skill exists). The Rust field was still `Option` from the older projection. This
is a hidden schema fallback and must be removed before the negative test pass: require the field in
Rust, require it in Unity validation, and prove missing/mismatched payloads reject the update and
request resync.

### 2026-07-13 — Final strict deployed-skill and payload validation

Hypothesis:

The actual Core producer can make deployed-unit skill readiness required without changing gameplay,
and Unity can reject every malformed eligibility projection before applying its revision.

Changed files:

- Core deployed-unit DTO, projection, and capability read.
- Unity `BattleRuntimeState.cs` and `ArkClientContractTests.cs` through the pinned MCP instance.

Commands and actual test counts:

- Core focused target-missing readiness parity: 1 executed, 1 passed.
- Unity MCP new strict negative test: 1 executed, 1 passed. The single test exercises six malformed
  projections: missing playback eligibility, duplicate speed, playback-state disagreement, missing
  candidate eligibility, missing deployed skill readiness, and skill boolean/eligibility mismatch.
- First Unity MCP full EditMode after requiring skill readiness: 188 executed, 185 passed, 3 failed.
  Two shared `PlayerDeployment` fixtures and one direct deployed-unit fixture omitted the now-required
  field, so their whole updates were correctly rejected before the old assertions.
- One MCP `insert_method(after)` attempt could not locate the nested-object helper boundary and made
  no edit. Retried with the stable `before PlayerDeployment` method anchor.
- After explicit current-shape fixture migration, the three failed tests passed 3/3.
- Final Unity MCP EditMode: 188 executed, 188 passed, 0 failed, 0 skipped.
- Unity compile and Console audit: 0 errors.
- `cargo test -q -p game_core --all-targets -- --test-threads=1`: 670 nonzero passed, 0 failed.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`: 27 nonzero passed, 0 failed.

Observed result:

`LiveBattleDeployedUnitDto.skill_readiness` is required and always contains typed eligibility,
including `skill_unavailable` when appropriate. Unity rejects malformed payload projections and
requests resync without applying their revision. No optional field, default, alias, or hide-button
fallback remains.

Decision and next step:

Rerun PlayMode because its live-like fixtures also cross the strict consumer, then audit the two
external Unity canonical documents and runnable probes before the completion review.

PlayMode and canonical/probe audit result:

- Unity MCP PlayMode: 53 executed, 51 passed, 0 failed, 2 explicit live-server skips.
- Exact typed Rust battle fixture equality: passed.
- External `unity_core_contract.md` and `core_unity_battle_transport_contract.md` still contained
  the pre-G07 local affordability, target-missing skill, optional readiness, missing capability,
  and authority-free mutation examples. They were migrated to the current contract.
- The runnable `unity_ws_smoke_probe.py` parsed successfully after adding authority/eligibility
  checks, but its first live run failed at deploy with `no battle_update` timeout.

Live probe failure cause:

The probe retained the revision from an earlier resync while Core continued automatic battle ticks.
Its deploy command was therefore correctly stale. Server emitted the new typed stale recovery path,
but the old probe receive loop waited only for `command_result + battle_update` and did not understand
`error + battle_resync`.

Correction:

Teach the probe's mutation helper to treat stale rejection as a non-mutating recovery outcome,
consume the current resync projection, and retry the same selected payload under a new request ID.
This is required live evidence for the accepted race contract; do not suppress ticks, fabricate a
revision, or relax Core validation merely to make the probe deterministic.

Live correction and revalidation:

- The first server launch from the workspace root failed before listening because the development
  config is resolved relative to `game_server/`. This ran 0 probe assertions and is not success
  evidence. Relaunching from the server package directory loaded live data and listened on 18083.
- The corrected probe uses a dedicated battle-mutation helper. A stale attempt waits for correlated
  `stale_battle_state_authority`, then the following current `battle_resync`, validates its complete
  authority/eligibility projection, and retries under a new request ID.
- First corrected live probe: exit 0. It observed one deploy race at revision 4, retried as
  `req-005-deploy-unit-retry-1`, and committed revision 5; extra deploy and speed then committed
  revisions 6 and 7.
- After removing the redundant authority-optional entry helper and retaining separate entry/query/
  mutation functions, Python AST parsing passed and a second live probe exited 0 on a fresh run.
- Both live runs validated action capabilities, canonical playback eligibility, deploy candidate/
  withdraw/required skill readiness eligibility, selected preview `deploy_eligibility`, ack/update
  revision equality, and actual stale `error -> battle_resync` recovery.
- Both temporary server processes were stopped after the probes.

Decision and next step:

The live probe now exercises rather than avoids mutation races. Finish scoped diff/legacy/ignored-
test audits and write the mandatory completion review.

### 2026-07-13 — Post-format Unity MCP and final hygiene revalidation

Hypothesis:

The final indentation-only Unity edits did not alter strict payload rejection, and no scoped
whitespace or ignored-test defect remains before completion review.

Commands and actual results:

- Unity MCP exact EditMode
  `ArkClient.Tests.EditMode.ArkClientContractTests.BattleRuntime_RejectsMalformedPayloadEligibilityAndRequestsResync`:
  1 executed, 1 passed, 0 failed, 0 skipped.
- Unity MCP editor state: idle, not compiling, no domain reload pending.
- Unity MCP Console error audit: 0 entries.
- Core scoped `git diff --check`: passed.
- game_server scoped `git diff --check`: passed.
- Unity scoped `git diff --check`: passed.
- Relevant ignored-test search across Core, game_server, and Unity battle/capability/revision tests:
  no matches.

Decision and next step:

All planned implementation, external live-probe, broad-suite, strict-consumer, and hygiene evidence
is available. Apply `docs/goal_completion_review_guide.md`; do not mark G07 complete until that
review reaches Complete / High / no immediate correction.

### 2026-07-13 — Completion review

Procedure:

- Applied `docs/goal_completion_review_guide.md` item by item to DCA-H-11, DCA-H-12, DCA-M-12,
  and the G07 battle portions of PD-01/PD-02.
- Rechecked runtime ownership, unchanged live-data boundary, Rust/server transport, Unity consumer,
  actual fixture, live probe, canonical documents, legacy/fallback search, ignored tests, and broad
  validation evidence.

Result:

- `COMPLETION_REVIEW.md` primary verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Immediate correction candidates: none.
- Policy questions: none.

Decision:

Mark G07 complete. Do not begin later-phase implementation; Phase 1 proceeds to G12 and then R01.
