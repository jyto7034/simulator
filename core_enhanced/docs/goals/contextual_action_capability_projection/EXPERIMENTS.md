# Contextual Action Capability Projection Experiments

Status: active

Implementation complete: false

Created: 2026-07-13

## 2026-07-13 — Goal startup

Hypothesis:

Accepted PD-02 and completed G01 provide enough policy and dependency stability to begin G06 current-state discovery without reopening DTO meaning or battle authority.

Minimal experiment:

- Confirmed R00 passed and G01 completion review is Complete / High / no immediate correction.
- Read the Master G06 catalog, DCA-M-13/M-18/M-19 rows, Phase 1 ordering, and complete accepted PD-02 contract.
- Created the three mandatory G06 documents before any G06 code search or validation.

Changed files:

- G06 documentation only.
- Master status already marks G06 ready after G01 completion.

Commands and actual test counts:

- No G06 tests executed yet; actual count 0 and not validation success.
- Predecessor evidence is not relabeled as G06 validation: G01 closed with Core 682, server 25, Unity EditMode 182, Unity PlayMode 49.

Observed result:

- No unresolved G06 policy gate is known at startup.
- Battle-specific capability, expected-revision, reconnect, and Unity whitelist work remains explicitly assigned to G07.
- Concrete Event/Safe Zone/inventory/Shop eligibility remains assigned to G03/G21/G24/G28.

Failure cause:

None.

Correction:

None.

Decision and next step:

Re-read all three G06 documents, then trace the current common gate/projector/server/Unity paths and build the action ownership matrix before choosing an implementation seam.

## 2026-07-13 — Current common action authority trace

Hypothesis:

The defect is a cross-layer authority split rather than only a root field rename, and the cached `ActionValidator` may no longer be a live gate after earlier execution refactors.

Minimal experiment:

- Read the current `ActionKind`, `PlayerBehavior::kind`, `ActionScheduler`, `ActionValidator`, `GameCoreState`, transition/refresh helpers, command execution pipeline, root snapshot projection, server root preservation, Unity store/command/composer consumers, mocks, tests, admin surface, and prior gate audits.
- Recorded dirty worktrees across Core/server/Unity without modifying user-owned files.
- Started an independent read-only Warp Grep trace for reconciliation.

Changed files:

- G06 goal documents only during this experiment.

Commands and actual test counts:

- Read-only `git status`, `rg`, `sed`, and `nl` traces; 0 tests executed and no validation success claimed.

Observed result:

- Live execution gate and root snapshot both call the on-demand coarse scheduler; `GameCoreState.action_validator` is only written, never read by production paths.
- Coarse tables expose unrelated InNode actions, all Shop actions, all ten battle actions, and both VN progression kinds.
- Payload validation is domain-local and cannot be represented by `Vec<ActionKind>`.
- game_server forwards the typed root snapshot rather than inventing action policy.
- Unity stores string action names, common UI gates through `HasAllowedAction`, and active battle substitutes a static whitelist.
- Mocks/tests/admin debug surfaces still hand-author or name `allowed_actions`.
- No live RON/save/checkpoint migration is implicated.

Failure cause:

The current contract has one payload-less table for context affordance, scattered payload validation, a dead cached copy, and a Unity-local battle substitute.

Correction:

Not implemented yet. The common seam must be an on-demand typed projector plus shared pure validation, not a renamed state table or retained cache.

Decision and next step:

Run focused current baselines, reconcile the Warp Grep report, and complete the action ownership matrix. Then choose which common payload-free/finite-payload validators G06 can migrate without absorbing G07/G03/G21/G24/G28.

## 2026-07-13 — Focused coarse-contract baseline passes

Hypothesis:

The current coarse action contract is reproducibly green before replacement, and its tests can be classified as legacy shape evidence rather than PD-02 parity evidence.

Minimal experiment:

Ran the scheduler module, the existing kind-level state-transition gate test, the server typed-root preservation test, and the complete Unity EditMode contract class in the open Editor.

Changed files:

- G06 goal documents only during this experiment.

Commands and actual test counts:

- `cargo test -p game_core action_scheduler --lib -- --test-threads=1`: 14 passed, 0 failed, 627 filtered out.
- `cargo test -p game_core game_core_rejects_disallowed_actions_and_updates_allowed_actions_on_state_transition --lib -- --test-threads=1`: 1 passed, 0 failed, 640 filtered out.
- `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1`: 1 passed, 0 failed, 24 filtered out; zero-test binary target excluded.
- Unity MCP EditMode `ArkClient.Tests.EditMode.ArkClientContractTests`: 182 passed, 0 failed, 0 skipped.

Observed result:

- Baseline is green across Core, server root transport, and Unity.
- The 14 scheduler tests explicitly protect coarse cardinality/presence such as all four Shop actions, all ten battle actions, and both VN progression kinds. They must be replaced with relevance/enabled/reason/scope behavior tests rather than mechanically renamed.
- The one Core transition test proves only kind-level state gating.

Failure cause:

None.

Correction:

None.

Decision and next step:

Use these counts as pre-cutover baseline, not completion evidence. Finish the action matrix and pure common preflight design, then replace legacy expectations with PD-02 contract tests.

## Experiment Template

## 2026-07-13 — Independent trace reconciliation and checkpoint affordance audit

Hypothesis:

The broad semantic trace will either expose a second live authority missed locally or confirm that G06 can begin with a bounded map/recovery vertical slice. `run_checkpoint.can_load` may be removable if no external consumer depends on it.

Minimal experiment:

- Reconciled the Warp Grep command/gate/projector/server/Unity report with direct `rg` and source reads.
- Searched Core, server, documents, and the live Unity project for `run_checkpoint`/`can_load` consumers.
- Read the open Unity MCP instance list, custom-tool inventory, and canonical editor readiness state.

Changed files:

- G06 goal documents only during this experiment.

Commands and actual test counts:

- Read-only `rg`/`sed` and Unity MCP resource reads; 0 tests executed and no validation success claimed.

Observed result:

- No hidden execution reader of `GameCoreState.action_validator` exists; it is write-only dead derived state.
- The server is transparent for the typed root. Unity consumes `allowed_actions` centrally but does not read `run_checkpoint.can_load` anywhere.
- `RunCheckpointState::can_load()` remains the correct Core domain predicate. Only the duplicated DTO boolean is ambiguous and removable.
- Action ownership is cleanly split: common map/recovery actions can migrate in G06; battle query/recovery/gameplay remains G07; Event/G24/G28/deferred Maintenance retain their child owners.
- Unity MCP reports one ready instance, `ark@b49d32f6f2f70bbe`, Unity 6000.3.4f1, idle compilation, and no running tests.

Failure cause:

None.

Correction:

The earlier note that final `can_load` meaning belonged to a later owner was too broad. G06 will remove only the redundant root boolean and project `LoadRunCheckpoint` as recovery capability; it will not change checkpoint load limits or rollback semantics.

Decision and next step:

Implement a tri-state common evaluator and the typed capability DTO, proving `LoadRunCheckpoint` and `SelectMapNode` projection/execution parity first. Deferred domain families are not admitted through a replacement coarse table.

## 2026-07-13 — Typed capability wire vocabulary trial

Hypothesis:

The accepted PD-02 shape can be represented without a generic payload field or compatibility alias, while enabled entries omit a reason and disabled entries serialize a stable snake-case reason.

Minimal experiment:

- Added typed action scope, disabled reason, and capability DTO constructors.
- Added typed `ActionUnavailable` Core error and server/snapshot error mappings.
- Added one exact JSON serialization test covering enabled query and disabled recovery entries.

Changed files:

- `src/game/behavior.rs`
- `src/game/world/snapshot.rs`
- `../game_server/src/game/player_game_actor/state.rs`
- G06 goal documents

Commands and actual test counts:

- `cargo fmt --all -- --check`: passed.
- `cargo test -p game_core action_capability_serialization_is_typed_and_omits_enabled_reason --lib -- --exact --test-threads=1`: 0 matched, 0 executed; invalid as success evidence.
- `cargo check -p game_server`: started behind shared Cargo locks; the captured invocation did not return a completed result and is not success evidence.

Observed result:

- The DTO compiles far enough for rustfmt and uses the required exact field names/casing.
- The first test filter incorrectly assumed the unqualified name was exact.

Failure cause:

Rust test harness exact names include the module path, so `--exact` with only the function name matched zero tests. Concurrent Cargo commands also contended for the shared target lock.

Correction:

Re-ran sequentially:

- `cargo test -p game_core action_capability_serialization_is_typed_and_omits_enabled_reason --lib -- --test-threads=1`: 1 passed, 0 failed, 641 filtered out.
- `cargo check -p game_server`: passed.

Decision and next step:

The small type slice is verified with nonzero execution. Add the tri-state common projector next.

## 2026-07-13 — First common projector/preflight compile

Hypothesis:

Map query, map selection, and checkpoint load can share a tri-state projector and execution preflight without changing domain policy.

Minimal experiment:

- Added an on-demand common projector for the three actions.
- Routed those behaviors through the same evaluation before the legacy gate.
- Removed duplicate map-select/checkpoint conditions from mutation handlers.
- Added context, scope, disabled parity, selected-payload rejection, and stale-projection revalidation assertions.

Changed files:

- `src/game/world/action_capability.rs`
- `src/game/world.rs`
- `src/game/world/node_flow.rs`
- `src/game/world/tests/map_flow.rs`
- `src/game/behavior.rs`
- G06 goal documents

Commands and actual test counts:

- `cargo fmt --all -- --check`: failed with formatting differences; no tests.
- `cargo fmt --all`: applied the mechanical formatting.
- `cargo test -p game_core common_map_and_checkpoint_capabilities_are_typed_and_revalidated --lib -- --test-threads=1`: compile failed, 0 tests executed.

Observed result:

- The production slice reached test compilation.
- The focused test file did not inherit the newly added behavior DTO types through `super::*` and lacked explicit imports.

Failure cause:

Test-module import omission for `ActionCapabilityDto`, `ActionDisabledReasonCode`, and `ActionScope`.

Correction:

Added the explicit test-only imports. The rerun executed 1 test: 1 passed, 0 failed, 642 filtered out.

Decision and next step:

The initial vertical slice is verified with nonzero execution. Expand the projector action-by-action before the root schema cutover.

## 2026-07-13 — Rust root capability schema cutover

Hypothesis:

The typed Rust root can replace `allowed_actions` and remove the redundant checkpoint affordance without affecting saved data or requiring a dual schema.

Minimal experiment:

- Replaced `RunSnapshotDto.allowed_actions` with `action_capabilities`.
- Updated selected-event generic mapping and the root snapshot producer.
- Removed `RunCheckpointSnapshotDto.can_load`; retained the Core predicate and factual count fields.

Changed files:

- `src/game/behavior.rs`
- `src/game/world/snapshot.rs`
- `src/game/world/state.rs`
- G06 goal documents

Commands and actual test counts:

- `cargo check -p game_core`: passed; compile-only, 0 tests and not runtime completion evidence.
- `cargo test -p game_server player_state_snapshot_preserves_core_snapshot_root_fields -- --test-threads=1`: library target 1 passed, 0 failed, 24 filtered out; binary target executed 0 and is excluded from evidence.

Observed result:

- The Core library compiles with one root schema only.
- No save/checkpoint payload migration was needed because capabilities remain derived projection.
- The server's generic flattened typed wrapper preserved every new Core root field/value without maintaining a second field list.

Failure cause:

None.

Correction:

None.

Decision and next step:

Migrate server exact-root tests and Unity typed consumption, while expanding the projector before broad behavior validation.

## 2026-07-13 — Unity typed reader/store/presenter compile trial

Hypothesis:

Unity can consume strict typed capabilities, omit irrelevant commands, localize disabled reasons, and retain the explicitly deferred G07 battle compatibility path without a root `allowed_actions` fallback.

Minimal experiment:

- Added the strict `ActionCapability` model and reason localizer.
- Replaced the production HashSet/API with typed dictionary lookup and enabled checks.
- Updated common presenters and command submission.
- Imported the new script and requested compilation through the open Unity MCP instance.

Changed files:

- Unity `ActionCapability.cs`, `GameActionNames.cs`, `GameSnapshotStore.cs`, `GameCommandBus.cs`
- Unity common presentation composers
- G06 goal documents

Commands and actual test counts:

- Unity MCP script refresh/compile: completed compilation, 0 tests.
- Unity MCP Console error read: 18 compiler errors, all in EditMode/PlayMode tests calling the removed test-facing `HasAllowedAction`; no production error remained after explicit asset import.

Observed result:

- External creation of the new C# file was not discovered by the first scripts-only refresh; Unity initially reported three missing-type production errors.
- Explicit Unity MCP asset import assigned a GUID and compiled the new type.
- The next compile reduced errors to stale test API calls only.

Failure cause:

- New external file needed explicit AssetDatabase import in the open Editor.
- Tests still named the deleted legacy API.

Correction:

Migrate test assertions to `IsActionEnabled`, then replace all hand-authored `allowed_actions` fixtures with typed entries. Recompile through Unity MCP before running tests.

Decision and next step:

Do not add a compatibility method. Complete the test/fixture migration and preserve the initial import failure as MCP workflow evidence.

## 2026-07-13 — First full Unity EditMode run after typed cutover

Hypothesis:

Once stale method calls compile, the full test run will identify the exact fixture/presentation blast radius of removing `allowed_actions`.

Minimal experiment:

- Replaced all test calls to `HasAllowedAction` with `IsActionEnabled` without adding a compatibility method.
- Recompiled through Unity MCP and read the error Console.
- Ran the complete `ArkClientContractTests` EditMode class.

Changed files:

- Unity EditMode/PlayMode test API calls
- G06 goal documents

Commands and actual test counts:

- Unity MCP compile/Console: 0 compiler errors.
- Unity MCP EditMode `ArkClient.Tests.EditMode.ArkClientContractTests`: 182 executed, 160 passed, 22 failed, 0 skipped.

Observed result:

- Failures cover command buses and presenters whose input JSON still hand-authors `allowed_actions`; typed store correctly treats the absent `action_capabilities` field as no capabilities.
- The old test named `SnapshotStore_UsesContextTypeAndAllowedActions` explicitly protects the removed contract and requires replacement, not expectation-only editing.
- Battle-facing failures remain tied to old fixture input even though the G07 static runtime path is still present after a battle runtime is established.

Failure cause:

Test and mock fixture schema has not yet been migrated. Production intentionally has no fallback parser.

Correction:

Introduce a test/mock typed capability fixture builder and replace old JSON roots atomically. Add strict negative tests for malformed/unknown capability entries and disabled reason rendering.

Decision and next step:

Keep all 22 failures as expected cutover evidence until fixture migration; do not restore legacy parsing.

## 2026-07-13 — Common family projector expansion

Hypothesis:

Common and finite-session families can replace the coarse state table using active domain content and existing finite predicates, while battle/Event/inventory/Maintenance details remain with their owners.

Minimal experiment:

- Added start/starter/node completion/support/HQ/shop/reward/non-battle VN/combat-result evaluations.
- Made VN next-scene lookup reusable so Advance and Choice are mutually exclusive.
- Routed these common actions through typed preflight.
- Re-ran the initial focused map/checkpoint test.

Changed files:

- `src/game/world/action_capability.rs`
- `src/game/world/visual_novel_flow.rs`
- G06 goal documents

Commands and actual test counts:

- `cargo fmt --all -- --check`: failed only on rustfmt layout; no tests.
- `cargo fmt --all`: applied formatting.
- Focused map/checkpoint test: 1 executed, 0 passed, 1 failed.

Observed result:

- Compilation succeeded.
- The failure was the old assertion that NotStarted had no projected common entries. The expanded projector correctly emits enabled `StartNewGame` gameplay capability.

Failure cause:

The reversible three-action trial test encoded its intentionally incomplete initial action set.

Correction:

Replaced the empty-list assertion with the durable NotStarted contract. Focused rerun: 1 passed, 0 failed, 642 filtered out.

Decision and next step:

The expanded projector preserves the focused vertical slice. Add family-specific projector tests next.

## 2026-07-13 — Legacy scheduler/cache authority removal

Hypothesis:

The live gate can remove the coarse scheduler and dead cache without weakening atomic domain validation; stale tests will reveal every remaining dependency on the old query API.

Minimal experiment:

- Removed scheduler/cache modules, fields, transition refreshes, and root query methods.
- Migrated admin Core output identity to `action_capabilities`.
- Left deferred-domain commands to their existing handler preflights rather than admitting them through a replacement coarse table.
- Compiled production and then test targets without running tests.

Changed files:

- Core world/state/helpers/resources/managers/admin modules
- Deleted `action_scheduler.rs` and `resources/action.rs`
- G06 goal documents

Commands and actual test counts:

- `cargo check -p game_core`: passed with two unused-import warnings, then imports were removed.
- `cargo test -p game_core --lib --no-run`: compile failed; 0 tests executed; 33 errors, all stale calls to `get_allowed_actions`/`is_action_allowed` in test modules.

Observed result:

- Production no longer reads or writes a scheduler/cache authority.
- The failures enumerate legacy contract tests across admin/start/map/support/equipment/combat.

Failure cause:

Tests explicitly coupled to the removed payload-less list API.

Correction:

Replaced common-family assertions with typed capability scope/enabled/reason behavior. Removed battle/inventory global-list assertions while retaining their actual handler/gameplay assertions. `cargo test -p game_core --lib --no-run` then succeeded; one unused admin test import was removed.

Decision and next step:

Test-target compilation is restored with no old API call sites. Run focused behavioral tests, then the broad suite after remaining schema work.

## 2026-07-13 — Focused post-legacy behavioral rerun

Hypothesis:

Typed common state transitions and blocking VN behavior remain correct after scheduler deletion; checkpoint tests may expose leftover factual DTO expectations.

Minimal experiment:

Ran the renamed state-transition test, blocking VN test, and three-load checkpoint limit test.

Changed files:

- G06 goal documents before correction.

Commands and actual test counts:

- State-transition capability test: 1 passed, 0 failed, 627 filtered out.
- Blocking VN capability test: 1 passed, 0 failed, 627 filtered out.
- Checkpoint three-load limit test: 1 executed, 0 passed, 1 failed, 627 filtered out.

Observed result:

- Blocking VN exposes only the executable progression kind and atomically rejects background map query.
- Checkpoint capability correctly became disabled with `checkpoint_unavailable`; failure was only `snapshot["run_checkpoint"]["can_load"] == false`, which is now null by design.

Failure cause:

Three support tests still referenced the removed ambiguous checkpoint DTO boolean; the focused test reached one of them.

Correction:

Asserted that the old field is absent and used typed recovery capability for executability. Updated exhausted-load rejection to matching `ActionUnavailable`. `cargo test -p game_core load_run_checkpoint --lib -- --test-threads=1`: 2 passed, 0 failed, 626 filtered out.

Decision and next step:

Checkpoint projection/execution parity is verified. Continue Unity fixture and admin cutover.

## 2026-07-13 — Unity fixture/admin migration and three-failure triage

Hypothesis:

Exact typed test/mock fixtures will restore most Unity behavior without a parser fallback; remaining failures will reveal genuine semantics requiring test replacement or presenter correction.

Minimal experiment:

- Migrated every Unity test/mock root action array to typed capability objects.
- Renamed admin dump end-to-end to action capabilities.
- Verified the legacy search is empty under `Assets`.
- Recompiled and ran all 182 EditMode tests, then one remaining test individually.

Changed files:

- Unity mocks, EditMode/PlayMode fixtures, admin bus/gateway/UI, common composition context
- G06 goal documents

Commands and actual test counts:

- `rg` legacy Unity terms: 0 matches.
- Unity MCP compile/Console: 0 compiler errors.
- Unity MCP EditMode class: 182 executed, 179 passed, 3 failed, 0 skipped.
- Individual typed-payload presenter test: 1 executed, 0 passed, 1 failed.

Observed result:

- VN command test reused an Advance-only snapshot for Choice.
- Node-map test expected an irrelevant Confirm button to remain disabled.
- The typed-payload test expected a local Settings placeholder; `AddPlaceholderCommand` had been over-constrained to Core capability even though it explicitly represents local/unimplemented commands.

Failure cause:

Two tests protected old coarse/dual-action semantics. One presenter change conflated Core commands with local placeholders.

Correction:

- Apply a Choice-only snapshot before sending Choice.
- Assert Confirm is absent until the server projects NodeConfirm context.
- Keep strict omission in `AddCommand`. In `AddPlaceholderCommand`, omit absent known Core actions but retain explicitly local placeholders.

Focused rerun of the three failures: 3 executed, 2 passed, 1 failed because unconditional placeholder restoration also restored absent Core Confirm. After separating known Core from local action identity, the remaining presenter test reran 1/1 passed.

Decision and next step:

All three original failures are corrected. Run the full EditMode class next.

## 2026-07-13 — First broad Core run after scheduler removal

Hypothesis:

Broad runtime tests will distinguish expected typed rejection changes from domain actions accidentally intercepted by the common preflight.

Minimal experiment:

Ran the entire Core library suite serially after legacy API migration.

Changed files:

- G06 goal documents before correction.

Commands and actual test counts:

- `cargo test -p game_core --lib -- --test-threads=1`: 628 executed, 621 passed, 7 failed, 0 ignored.

Observed result:

- Two battle VN tests failed because non-battle VN evaluation returned context-irrelevant and common preflight intercepted the commands before G07 battle transport.
- Artifact sale failed because the Shop context capability had no sellable payload and G06 preflight masked the selected artifact handler's more specific `InventoryItemNotRemovable`.
- Two map-selection tests, one blocked starter-selection test, and one support completion test still expected generic `InvalidAction` instead of the new common typed reasons.

Failure cause:

- Common execution ownership matching was broader than G06 ownership for battle VN and Shop selected payloads.
- Four assertions retained generic legacy errors.

Correction:

- Bypass common VN preflight in battle; leave it to G07/domain.
- Keep Shop context projection but do not make G06 its execution owner; G28/handler preserves selected-payload errors.
- Update common map/blocking/support assertions to exact `ActionUnavailable` action/reason.

Broad rerun after correction: 628 passed, 0 failed, 0 ignored.

Decision and next step:

All seven are resolved and the full Core suite is green. Continue exact Rust-to-Unity fixture and strict negative tests.

## 2026-07-13 — Rust-to-Unity capability fixture and strict consumer tests

Hypothesis:

An actual typed Rust root snapshot can be checked into Unity and consumed without reconstruction, while strict negative and localization tests close the reader/presenter contract.

Minimal experiment:

- Added a typed game_server example generator for a NotStarted root snapshot.
- Checked its JSON into Unity and verified value equality in generator compare mode.
- Imported the fixture through Unity MCP.
- Added Unity tests for actual fixture consumption, absent/disabled/scope distinction, malformed/unknown/duplicate rejection, and disabled reason localization.
- Preserved reason code through server error mapping and added a focused server test.

Changed files:

- `../game_server/examples/generate_action_capability_fixture.rs`
- Unity `action_capability_contract.json` plus generated metadata and EditMode tests
- Core/server reason-code mapping
- G06 goal documents

Commands and actual test counts:

- `cargo run -q -p game_server --example generate_action_capability_fixture`: generator output succeeded; 0 tests.
- Generator compare against checked-in Unity fixture: passed exact JSON value equality; 0 tests.
- Unity MCP fixture import: succeeded as TextAsset.
- Unity MCP new capability tests: 4 passed, 0 failed, 0 skipped.
- Server typed reason test: library 1 passed, 0 failed, 25 filtered out; binary 0 excluded.
- Server root preservation test: library 1 passed, 0 failed, 25 filtered out; binary 0 excluded.
- `cargo fmt --all -- --check`: initially found mechanical layout differences; `cargo fmt --all` applied them.

Observed result:

- The fixture contains only `action_capabilities`, no checkpoint `can_load`, and enabled StartNewGame gameplay scope from the real Core snapshot.
- Unity rejects structurally invalid entries and localizes typed disabled reasons.
- Server error `code` equals the stable reason code such as `checkpoint_unavailable`.

Failure cause:

Only rustfmt layout differences; no semantic test failure.

Correction:

Applied rustfmt and retained exact fixture comparison.

Decision and next step:

Run full Unity EditMode at the new 186 count, PlayMode, full server, formatting, and legacy/diff audits.

## 2026-07-13 — First full Unity PlayMode run after typed fixture cutover

Hypothesis:

Typed mocks and common presenter changes preserve rendered utility flows and reconnect behavior.

Minimal experiment:

Ran every PlayMode test through the open Unity MCP Editor after EditMode 186/186.

Changed files:

- G06 goal documents before correction.

Commands and actual test counts:

- Unity MCP PlayMode all: 53 executed, 45 passed, 8 failed, 0 skipped.

Observed result:

- All eight failures are in `ArkClientMockRenderTests`; reconnect tests reached completion.
- Failures include missing NodeMap item/loadout utility objects, stale status text/panel, missing admin catalog control, modal hit ordering, and inventory scroll reset.

Failure cause:

Focused first-test rerun reproduced 0/1 and source inspection identified an obsolete assertion that ViewingMap renders a disabled Confirm button. The second failing test had the same expectation and asserted object identity across preview. Their assertion aborts skipped root cleanup, cascading shared UI state into six later tests.

Correction:

- Assert Confirm is absent before NodeConfirm capability exists.
- After the preview mock arrives, assert the newly relevant Confirm button is present/enabled; do not preserve identity for an object that did not previously exist.
- Full Unity MCP PlayMode rerun: 53 passed, 0 failed, 0 skipped.

Decision and next step:

PlayMode is green. Proceed to final server/format/legacy/diff/canonical audits.

## 2026-07-13 — Final broad, probe, legacy, and completion-review audit

Hypothesis:

The atomic root cutover is complete only if every broad suite is green, runnable probe contracts
consume the same typed shape, old authority is absent, and the completion guide finds no immediate
correction.

Minimal experiment:

- Ran full Core/server/Unity suites and exact Rust-to-Unity fixture comparison.
- Migrated the two runnable Python WebSocket probes plus their closest contract documents from the
  deleted root list and checkpoint derived boolean.
- Parsed both probes with Python AST, searched production/canonical surfaces for legacy names,
  checked formatting/diffs, and audited the live Unity Console.
- Applied `docs/goal_completion_review_guide.md` and wrote `COMPLETION_REVIEW.md`.

Changed files:

- G06 Core/server/Unity implementation and tests listed in prior experiments.
- Unity `docs/probe/unity_ws_smoke_probe.py`,
  `docs/probe/unity_savepoint_checkpoint_probe.py`,
  `docs/probe/unity_ws_smoke_test_contract.md`, and
  `docs/probe/unity_noncombat_node_ws_contract.md`.
- G06 goal documents and completion review.

Commands and actual test counts:

- Core all-targets: 669 nonzero tests passed (628 + 3 + 3 + 18 + 13 + 4); zero-test binary target
  excluded from evidence.
- game_server all-targets: 26 nonzero tests passed; zero-test binary/example targets excluded.
- Unity MCP EditMode all: 186 passed, 0 failed, 0 skipped.
- Unity MCP PlayMode all: 53 passed, 0 failed, 0 skipped.
- Exact Rust fixture comparison: passed; 0 tests, recorded only as contract comparison.
- Python AST parse of both runnable probes: passed; 0 tests.
- `cargo fmt --all -- --check`: passed.
- Scoped Core and Unity `git diff --check`: passed.
- Unity MCP Console error query: 0 entries.
- Production/canonical legacy searches: no common/root legacy occurrence. The Unity battle-only
  static set remains as the explicit G07 boundary; two negative test assertions prove deleted fields
  remain absent.

Observed result:

All required suites and contract audits pass. Rust, server, Unity, canonical docs, and runnable
probes agree on the typed root contract.

Failure cause:

None in the final audit. Earlier PlayMode and formatting failures are recorded with their
corrections above.

Correction:

No additional runtime correction was required. Probe contracts were synchronized to the already
implemented runtime shape.

Decision and next step:

G06 is Complete / High / no immediate correction. Activate G07 and preserve its battle-specific
boundary.

### YYYY-MM-DD — Title

Hypothesis:

Minimal experiment:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Correction:

Decision and next step:
