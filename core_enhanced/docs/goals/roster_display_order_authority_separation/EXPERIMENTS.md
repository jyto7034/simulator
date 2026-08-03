# Roster Display Order Authority Separation Experiments

Status: complete

Created: 2026-07-15

Last updated: 2026-07-15

## 2026-07-15 — Pre-implementation static contract discovery

Hypothesis:

`RosterOrder` is presentation-only authority and can be removed without changing employee gameplay or battle deployment rules.

Experiment:

- searched Core, game_server, and Unity for the order state, command, fields, capability, result, mocks, and tests,
- read the Core mutation/snapshot/rollback path, typed DTO serialization, server wrapper, Unity reader/command/presenter path, and relevant tests,
- inspected the current mock employee UI and illustrated Rest target.

Changed:

- no production code, data, fixture, mock, test, or canonical document,
- the earlier Master/Child Goal documents were later replaced by this single Goal document set at the user's request.

Commands executed included:

```bash
rg -n --hidden -S "RosterOrder|MoveRosterUnit|roster_order|roster_slot|move_roster" .
rg -n -S "MoveRosterUnit|move_roster_unit|roster_order|roster_slot|roster_slots" ../game_server
rg -n -S "MoveRosterUnit|move_roster_unit|roster_order|roster_slot|roster_slots" \
  "/mnt/f/unity projects/ark/Assets/Scripts" "/mnt/f/unity projects/ark/Assets/Tests"
```

Tests executed: 0.

Pass/fail/skip: not applicable; this was read-only discovery and is not recorded as passing validation.

Result:

- Core uses were confined to display-order storage/sync, command/capability/result, snapshot projection, checkpoint/battle-entry copying, and their tests/docs.
- Battle deployment and employee gameplay use independent UUID-based Core state.
- No live RON use or independent game_server roster-order state was found.
- Unity has old-contract consumers but no local committed order.
- The intended formation interaction is not present in the inspected runtime scene.

Failure cause:

- None for the discovery hypothesis; mandatory baseline and runtime tests have not run.

Reverted or corrected:

- Pair swapping and adjacent-arrow UX interpretations were corrected during user discussion to sequential click assignment with reclick removal and number compaction.
- The over-decomposed five-Goal document structure was removed and replaced with this single contract-boundary Goal.

Judgment:

The authority migration is a single tightly coupled producer-forwarder-consumer contract boundary. Implementation remains stopped until local persistence lifetime and identity policy are approved.

## 2026-07-15 — Resume-time persistence gate confirmation

Hypothesis:

The current Unity runtime does not already define a durable player-scoped UI-settings identity, so persistence lifetime cannot be treated as an internal implementation detail.

Experiment:

- re-read all three Goal documents from start to finish,
- searched only the relevant Unity runtime/tests for persistence and player-identity seams,
- inspected `GameWsClient`, `GameSnapshotStore`, and `GameUiSelectionState`.

Changed:

- Goal execution records only; no production file.

Command executed:

```bash
rg -n -S "PlayerPrefs|persistentDataPath|playerId|player_id|Configure\(" \
  Assets/Scripts/ArkClient/Runtime Assets/Tests -g '*.cs'
```

Tests executed: 0.

Pass/fail/skip: not applicable; this was policy-independent read-only discovery.

Result:

- No general local settings or `PlayerPrefs` usage exists.
- `GameWsClient.playerId` is private, configured before connection, and used for auth only.
- `Application.persistentDataPath` is used for the wire log, not UI preferences.
- `GameSnapshotStore` replaces authoritative snapshots and owns no local settings.
- `GameUiSelectionState` is transient and cleared, so it is not a committed-order owner.

Failure cause:

- None. The evidence confirms the existing policy gate.

Reverted or corrected:

- None.

Judgment:

Implementation remains stopped until the user chooses session-local order or restart-persistent player-scoped order and, for persistence, approves the player identity source.

## 2026-07-15 — Persistence policy decision

Hypothesis:

Device-local, restart-persistent, player-scoped storage best matches a user-managed UI preference without returning authority to Core or game_server.

Experiment or validation:

- no runtime experiment; the user reviewed the alternatives and explicitly approved the long-term recommendation.

Changed:

- Goal policy and migration plan only.

Commands executed: none.

Tests executed: 0.

Pass/fail/skip: not applicable; this is an explicit user policy decision, not test evidence.

Result:

- use the stable `/game` player/account UUID as an explicitly injected settings scope,
- persist committed order in versioned device-local JSON,
- keep formation drafts transient and tests storage-injected,
- do not sync across devices or store order in Core/game_server.

Failure cause: none.

Reverted or corrected: none.

Judgment:

The policy gate is resolved. Status returns to `active`; runtime revalidation, dirty-diff inspection, and baseline are now authorized.

## 2026-07-15 — Core legacy-contract baseline

Hypothesis:

Before removing display-order authority, the current dirty worktree still passes the focused roster-order, command/capability, retreat, and checkpoint contracts.

Experiment or validation:

- ran four focused Core unit-test filters before changing any production file,
- used a single test thread so stateful fixtures could not obscure the baseline.

Changed: no production file.

Commands and actual results:

```text
cargo test -p game_core roster_order --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 714 filtered out

cargo test -p game_core move_roster_unit_ --lib -- --test-threads=1
3 passed; 0 failed; 0 ignored; 712 filtered out

cargo test -p game_core retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 714 filtered out

cargo test -p game_core load_run_checkpoint_restores_visible_run_state_but_not_load_count --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 714 filtered out
```

Tests executed: 6 total across four filtered commands.

Pass/fail/skip: 6 passed; 0 failed; 0 ignored.

Result:

- the legacy roster-order contracts are green before removal,
- the broader retreat and checkpoint flows are also green before display-order fields are removed.

Failure cause: none.

Reverted or corrected: none.

Judgment:

The focused Core baseline is usable for post-change regression comparison. Server and Unity baselines remain to be recorded before their production files are changed.

## 2026-07-15 — Core authority removal, focused verification

Hypothesis:

Removing display-order-only state and transport shapes from Core will leave employee gameplay, retreat, and checkpoint behavior intact.

Experiment or validation:

- removed `RosterOrder`, its state accessors/synchronization, checkpoint and battle-entry copies,
- removed `MoveRosterUnit` action/behavior/result/capability and map-result serialization,
- removed root `roster_order` and employee `roster_slot` projection,
- deleted the three legacy movement tests and replaced snapshot expectations with explicit field-absence assertions,
- removed roster-order synchronization calls from combat resolution, recruitment, and admin execution,
- updated the Core runtime contract and gameplay rulebook boundary.

Commands and actual results:

```text
cargo test -p game_core start_new_game_creates_starter_employee_roster_and_equipment --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 710 filtered out

cargo test -p game_core employee_roster_snapshot_exposes_gameplay_status_and_loadout_without_display_slot --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 710 filtered out

cargo test -p game_core retreat_battle_restores_exact_pre_entry_snapshot_and_rng_continuation --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 710 filtered out

cargo test -p game_core load_run_checkpoint_restores_visible_run_state_but_not_load_count --lib -- --test-threads=1
1 passed; 0 failed; 0 ignored; 710 filtered out
```

Tests executed: 4.

Pass/fail/skip: 4 passed; 0 failed; 0 ignored.

Actual result:

- Core compiles without the display-order contract,
- employee gameplay snapshot fields remain,
- root and employee display-order fields are explicitly absent,
- retreat and checkpoint focused flows remain green.

Failure cause:

- the first verification build reported that `available_player_unit_ids` and `validate_owned_unit_exists` became dead code after removal.

Reverted or corrected:

- removed those two now-unreferenced helpers; they only served the deleted display-order path.

Judgment:

The Core slice is functionally removed at focused-test scope. Broad Core validation and producer-consumer fixture verification remain pending.

## 2026-07-15 — Server cut and Unity legacy baseline

Hypothesis:

game_server only needs its obsolete deserialization assertion removed, while the existing Unity tests establish a green baseline for the legacy command, reader, presenter, and Rest route before their replacement.

Experiment or validation:

- attempted the focused server test immediately after the coordinated Core contract cut,
- removed the sole `MoveRosterUnit` server deserialization assertion,
- updated the checked-in typed Rust capability fixture by removing root `roster_order`,
- compared the fixture against the real Rust generator,
- ran five focused Unity EditMode legacy/fixture tests and one Rest PlayMode test before changing Unity runtime code.

Commands and actual results:

```text
cargo test -p game_server deserializes_snake_case_node_flow_requests -- --test-threads=1
first attempt: compile failed because the unchanged server test still referenced removed PlayerBehavior::MoveRosterUnit
after removal: lib 1 passed; 0 failed; 0 ignored; 29 filtered out; main 0 tests

cargo run -p game_server --example generate_action_capability_fixture -- '/mnt/f/unity projects/ark/Assets/Tests/Fixtures/action_capability_contract.json'
exit 0; checked-in JSON exactly matched typed Rust serialization

Unity EditMode focused job 82191301114d4007aba4a39833bec660
5 passed; 0 failed; 0 skipped

Unity PlayMode focused job 58d70ab7214f400c8cf25680bb356c31
1 passed; 0 failed; 0 skipped
```

Unity EditMode tests executed:

- `RustGeneratedActionCapabilityFixture_IsConsumedWithoutUnityInference`
- `GameCommandBus_RecoveryAndRosterCommandsUseExistingTypedWireShapes`
- `MapNodeActionHandler_RosterMoveUsesCapabilityAndTypedPayload`
- `RosterModel_ReadsCurrentRosterContract`
- `Presenter_RecoveryAndRosterControlsFollowProjectedCapabilities`

Unity PlayMode test executed:

- `Bootstrap_RendersRestNodeWithoutLoadoutEditingControls`

Pass/fail/skip across successful commands: 8 passed; 0 failed; 0 skipped. The first server attempt was a compile failure and executed 0 tests.

Failure cause:

- a pre-Core server baseline was not run before the producer variant was removed; the coordinated cut exposed the expected stale server test as a compile error.

Reverted or corrected:

- removed only that stale assertion and reran the same server filter successfully.

Judgment:

The server is aligned with the new producer, the Rust fixture proves root-field removal, and the Unity legacy behavior has a recorded green baseline. Unity runtime replacement is now authorized.

## 2026-07-15 — Unity local authority and formation interaction

Hypothesis:

A player-scoped local state can preserve display order across authoritative snapshot replacement while an empty transient Rest formation draft provides the approved click-in-order behavior without any Core command.

Experiment or validation:

- added injectable memory and version-1 JSON roster-display-order storage,
- scoped production JSON by the explicitly exposed `/game` player UUID under `Application.persistentDataPath/ui-settings/<player-uuid>/roster-display-order.json`,
- reconciled new, removed, unavailable, and newly available employees on snapshot apply,
- added an explicit display-ordered roster reader and migrated Item Use, Loadout, Maintenance, Battle, and Rest roster presenters,
- removed Unity command bus, gateway, action handler, reader, mock, Node Preview arrows, payload, and legacy tests,
- added the empty formation draft, append/reclick compaction, all-eligible completion gate, cancel, local commit, disabled dead/unavailable units, red/blue outlines, and top-left badges,
- compiled Unity and ran focused EditMode and PlayMode evidence.

Commands and actual results:

```text
Unity script refresh/compile
0 compile errors

Unity EditMode job b06dfe8abc844356ae73c1891dfb9f98
7 passed; 0 failed; 0 skipped

Unity PlayMode job b5f74774680d4838b1587751cf58044e
1 passed; 0 failed; 0 skipped
```

EditMode tests executed:

- `RustGeneratedActionCapabilityFixture_IsConsumedWithoutUnityInference`
- `GameCommandBus_RecoveryCommandsUseExistingTypedWireShapes`
- `RosterDisplayOrder_ReconcilesAvailabilityAndIgnoresAuthoritativeArrayOrder`
- `JsonRosterDisplayOrder_IsVersionedRestartPersistentAndPlayerScoped`
- `RosterFormationDraft_AppendsRemovesAndCompactsWithoutCommitting`
- `Presenter_RecoveryControlDoesNotCreateNodePreviewRosterControls`
- `RosterModel_ReadsCurrentRosterContract`

PlayMode test executed:

- `Bootstrap_RendersRestNodeWithoutLoadoutEditingControls`

Actual result:

- restart persistence and player isolation passed against real temporary JSON files,
- snapshot array reorder did not replace a committed order,
- unavailable removal and later append, deletion pruning, and new employee append passed,
- sequential selection, reclick removal, number compaction, unavailable rejection, cancel, and completion passed,
- real UGUI clicks rendered red/blue outlines and a visible top-left `1` badge,
- Node Preview contains neither roster rows nor move actions.

Failure cause:

- the first Unity compile after runtime removal correctly exposed stale PlayMode/EditMode test references,
- a later compile exposed that internal color tokens were inaccessible to the PlayMode test assembly.

Reverted or corrected:

- replaced the legacy tests instead of preserving removed contracts,
- kept production color tokens internal and asserted their exact rendered RGBA values in PlayMode.

Judgment:

The Unity local-order and Rest interaction slices now satisfy focused behavior evidence. Canonical docs, broader validation, exact legacy searches, and dirty-diff review remain pending.

## 2026-07-15 — Broad validation and canonical synchronization

Hypothesis:

The reduced Core/server contract and Unity-local replacement preserve all unrelated gameplay and client behavior across the full available test suites.

Experiment or validation:

- updated Core, Unity wire/UI, smoke-probe, and Unity implementation canonical documents,
- added a presenter contract proving Item Use, Loadout, Maintenance, Battle, and Rest consume one committed local order,
- ran Rust formatting checks, full Core tests including live RON integration, full game_server tests, full Unity EditMode, and full Unity PlayMode,
- ran scoped whitespace checks and exact legacy-symbol searches.

Commands and actual results:

```text
cargo fmt --check -p game_core -p game_server
first attempt: failed with formatting diffs in world.rs and world/snapshot.rs
after targeted rustfmt: exit 0

cargo test -p game_core --lib -- --test-threads=1
711 passed; 0 failed; 0 ignored

cargo test -p game_core -- --test-threads=1
lib 711 passed
live_item_skill_activation 3 passed
live_skill_catalog_audit 4 passed
ron_loading 20 passed
skill_runtime_contract 13 passed
skill_test_suite 4 passed
total actual tests: 755 passed; 0 failed; 0 ignored
zero-test binary/doc targets were reported separately and are not counted as passing tests

cargo test -p game_server -- --test-threads=1
lib 30 passed; 0 failed; 0 ignored
main/doc targets reported 0 tests and are not counted

Unity EditMode full job 67b2efc7a2ac4b4eaddce7251facdbfb
224 completed; 1 failure reported; runner did not return pass/skip counts for this failed job

Unity EditMode focused correction job 8b9fe697ae734b90a7912df7f33decb4
1 passed; 0 failed; 0 skipped

Unity EditMode full rerun 0a396f7c594b430d90d4fb72c87ac863
224 passed; 0 failed; 0 skipped

Unity PlayMode full job 39e3e88284464ef387136f0e46baf1a0
56 total; 52 passed; 0 failed; 4 skipped Explicit live-server tests

Unity validate_script on RosterDisplayOrder.cs, GameSnapshotStore.cs,
SupportScreenComposer.cs, BodyPanelActionHandler.cs, UguiGameScreenViewAdapter.cs
all 5: 0 errors; 0 warnings

git diff --check -- <Goal-scoped Core/server paths>
exit 0

git diff --check -- <Goal-scoped Unity paths>
exit 0
```

Failure cause:

- formatting changed around removed imports and an emptied method boundary,
- the first full EditMode suite found one stale Rest presenter assertion that still required the roster panel to have zero actions.

Reverted or corrected:

- ran `rustfmt` only on the two reported Rust files and rechecked formatting,
- replaced the stale assertion with the new employee-action and formation-entry contract, verified it focused, then reran all 224 EditMode tests.

Actual result:

- all available non-explicit Rust, server, EditMode, and PlayMode tests pass,
- the four skipped PlayMode tests are pre-existing Explicit tests requiring a live game_server/admin environment,
- live RON loading/validation remains green and no roster-order RON dependency exists,
- all five in-scope Unity roster surfaces use one committed local order,
- canonical docs describe the reduced wire contract and local Rest formation policy.

Judgment:

Broad regression evidence is green. Final completion audit must still connect every PLAN condition to evidence, review scoped diffs/status, and update Goal status.

## 2026-07-15 — Invalid local-settings recovery and final Unity rerun

Hypothesis:

Invalid duplicate or blank local employee identities can be rejected and quarantined with a visible diagnostic without weakening gameplay or silently accepting corrupt order data.

Experiment or validation:

- wrapped the entire JSON load/shape validation path in explicit invalid-file quarantine,
- added duplicate and blank employee-identity fixtures for two player scopes,
- verified the invalid original is moved aside, the active session starts with an empty order, and a warning names the quarantine,
- reran the full Unity suites after the final runtime change.

Commands and actual results:

```text
Unity focused EditMode job cbd74ba0a1e947c2808b66a7863fba19
1 completed; 0 failures reported; result summary was not returned after a Unity plugin domain-reload disconnect, so this job alone is not counted as pass evidence

Unity EditMode full job f737f238213c4b4298195c27bb14687b
225 passed; 0 failed; 0 skipped

Unity PlayMode full job 1e46898c264f4b0ea9df48eda21f1344
56 total; 52 passed; 0 failed; 4 skipped Explicit live-server tests
```

Failure cause:

- the first focused result poll crossed a Unity domain reload and the plugin session disconnected; polling after reconnect showed completion with no failure but no summary.

Reverted or corrected:

- used the subsequent full 225-test EditMode run as the count-bearing evidence instead of inferring a pass from the incomplete focused report.

Judgment:

Invalid local data recovery now matches the approved diagnosable quarantine policy, and the final Unity runtime state is covered by count-bearing full-suite results.

## 2026-07-15 — Live Rust server retreat and reconnect boundary

Hypothesis:

A committed Unity-local display order remains unchanged through a real Rust-server battle retreat and a WebSocket disconnect/reconnect that delivers a fresh authoritative snapshot.

Experiment or validation:

- started the development game_server on `0.0.0.0:8081` with embedded live RON,
- extended the existing explicit live combat flow to commit a reversed local order before battle,
- asserted the reduced Rust snapshot has neither root `roster_order` nor employee `roster_slot`,
- retreated from a real battle, reconnected the same player session, and compared the active display order after each fresh snapshot,
- shut the server down gracefully after the test.

Commands and actual results:

```text
cd /mnt/f/work/simulator/game_server && cargo run
server loaded embedded live data and listened on 0.0.0.0:8081

Unity PlayMode explicit job 9acf0e55799c403187f01d6993361372
1 completed; failed in the pre-existing screenshot visibility assertion before the new retreat/reconnect assertions

Unity PlayMode explicit rerun 64490ee0670146d4a607dbfb8dedbae2
1 passed; 0 failed; 0 skipped

Ctrl+C
server reported graceful shutdown and exited 0
```

Failure cause:

- the first run captured a zero-visible-pixel battlefield screenshot, an existing environment-sensitive assertion unrelated to roster order.

Reverted or corrected:

- no production or test expectation was weakened; reran the same unchanged live test and it passed through retreat and reconnect.

Actual result:

- the real Rust snapshot exposes the reduced contract,
- the same committed Unity order survived real `RetreatBattle`, a new `node_confirm` snapshot, disconnect, re-authentication, and the reconnect snapshot,
- the live server was not left running.

Judgment:

The cross-layer retreat/reconnect boundary now has real server evidence in addition to Core checkpoint/retreat tests and Unity snapshot-replacement tests.

## 2026-07-15 — Dirty-output completion audit

Hypothesis:

All Goal tests and local server runs preserved pre-existing user-owned dirty files exactly.

Experiment or validation:

- compared final scoped status with the recorded initial dirty status,
- inspected screenshot test implementations and validation artifact timestamps,
- searched the project, parent Unity path, `Library`, `Temp`, and `Logs` for alternate copies,
- inspected the current game_server log boundary.

Commands included:

```text
rg -n -C 8 'battlefield_live_validation|battlefield_overlay_inclusive_validation|CaptureLiveBattlefieldScreenshot' Assets/Tests -g '*.cs'
find ... -name 'battlefield_live_validation*.png' -o -name 'battlefield_overlay_inclusive_validation*.png'
rg -n '2026-07-15T09:14:46|2026-07-15T09:16:46' logs/app.log.2026-07-15
```

Tests executed: 0. This was a preservation audit, not a test run.

Actual result:

- both PNGs were dirty before the Goal,
- the existing tests explicitly `File.Delete` and `ScreenCapture.CaptureScreenshot` to the same paths,
- their current timestamps correspond to this Goal's test runs,
- no prior duplicate copy was found,
- the untracked current-day server log contains appended entries from the two required live-server runs.

Failure cause:

- the test-output side effects were not identified and backed up before broad/live validation.

Reverted or corrected:

- no destructive attempt was made to guess or reconstruct the missing user-owned binary bytes,
- the local server was stopped gracefully and no further screenshot-producing tests will run without explicit artifact handling.

Judgment:

Status is `externally_blocked` for completion reporting only. The roster-order implementation is functionally complete, but the Goal cannot certify dirty-worktree preservation until the user accepts the regenerated validation artifacts/log append or supplies originals to restore.

## 2026-07-15 — Generated-artifact acceptance and Goal completion

Hypothesis:

Explicit user acceptance of the regenerated validation PNGs and appended generated server log resolves the only remaining completion-audit exception without further workspace mutation.

Experiment or validation:

- re-read all three Goal documents from start to finish,
- presented the preservation tradeoff and recommended retaining the current generated artifacts,
- received explicit user confirmation that the previous generated files were not important and that the recommendation should be applied.

Changed:

- Goal status and audit records only; no production source, fixture, gameplay data, screenshot, or server log was modified in this completion step.

Commands executed: document reads and scoped document diff checks only.

Tests executed: 0. Existing count-bearing validation remains authoritative; screenshot-producing tests were intentionally not rerun.

Pass/fail/skip: not applicable; this is an explicit user acceptance decision and completion audit.

Actual result:

- the current `battlefield_live_validation.png` and `battlefield_overlay_inclusive_validation.png` are accepted as the retained validation artifacts,
- the appended entries in `game_server/logs/app.log.2026-07-15` are accepted as generated validation history,
- all functional, contract, persistence, interaction, formatting, cross-layer, and live retreat/reconnect completion conditions remain satisfied.

Failure cause: none.

Reverted or corrected: none.

Judgment:

The sole external completion condition is resolved. This Goal is complete.

## 2026-07-15 — Post-completion formation placement correction

Hypothesis:

The existing local formation interaction can move from the support Rest screen to the SafeZone Loadout roster panel without changing persistence, eligibility, or Core/server authority.

Experiment or validation:

- removed all formation actions and mode-specific command suppression from Rest,
- added formation start, employee assignment, complete, and cancel to the SafeZone Loadout roster panel,
- kept normal Loadout employee targeting outside formation mode and changed only employee-card click ownership while formation is active,
- canceled transient formation state when presentation leaves the Loadout route,
- migrated EditMode and real UGUI PlayMode assertions to the new surface,
- updated current Unity canonical documents.

Commands and actual results:

```text
Unity validate_script: LoadoutScreenComposer.cs, SupportScreenComposer.cs,
GameScreenPresenter.cs
all 3: 0 errors; 0 warnings

Unity EditMode focused job c8813400322d47a6b180bf5eaa775ba6
3 passed; 0 failed; 0 skipped

Unity PlayMode focused job 6cc3c3ed2ccf4b278a2819acf54e2c28
2 passed; 0 failed; 0 skipped

git diff --check -- <placement-scoped Unity runtime/tests/docs>
exit 0
```

Failure cause:

- the first two focused EditMode attempts encoded the extra wounded/dead SafeZone fixture employees in insertion order, while the production local-order view correctly places inactive employees afterward in deterministic UUID order.
- one final result poll crossed a Unity domain reload and disconnected the plugin session before returning a result.

Reverted or corrected:

- corrected the test to assert the actual committed-active-first, inactive-UUID ordering; no production behavior was changed for the failed expectation.
- reconnected to the same Unity editor and reran the unchanged three-test filter to obtain the count-bearing passing result above.

Actual result:

- SafeZone Loadout is the only formation-editing surface,
- the real UGUI test covers red/blue outline, top-left number badge, reclick removal, cancel, completion, and disabled unavailable employee behavior there,
- Rest has no formation start or completion controls,
- no Core command or wire contract changed.

Judgment:

The user-directed placement correction is complete and the Goal remains complete.

## 2026-07-15 — Final SafeZone Item placement correction

Hypothesis:

The formation interaction can share the SafeZone Item employee panel while preserving normal item targeting and Loadout long-press outside formation mode.

Experiment or validation:

- moved formation start, assignment, complete, and cancel from `LoadoutRoster` to `ItemUseEmployees`,
- switched Item employee cards from item/long-press actions to formation-only actions while the draft is active,
- restored Loadout to equipment-target selection only,
- canceled the transient draft whenever presentation leaves the Item route,
- updated current canonical and Goal records,
- reran focused presenter and real UGUI interaction evidence.

Commands and actual results:

```text
Unity validate_script: ItemUseScreenComposer.cs, LoadoutScreenComposer.cs,
SupportScreenComposer.cs, GameScreenPresenter.cs
all 4: 0 errors; 0 warnings

Unity EditMode focused job f0aac9286e7a4cdd87ab9f09ebbb491d
4 passed; 0 failed; 0 skipped

Unity PlayMode focused job c998db7dfc1e490f90f59fd8c24e03f6
2 passed; 0 failed; 0 skipped
```

Failure cause:

- the first EditMode launch was rejected before executing tests because the Unity editor remained in Play Mode after the previous runner job.

Reverted or corrected:

- explicitly stopped Play Mode and reran the unchanged four-test EditMode filter successfully.

Actual result:

- SafeZone Item is the only formation-editing surface,
- normal Item cards still support item targeting and Loadout long-press outside formation mode,
- formation mode owns those clicks and renders the existing outline/badge interaction,
- Rest and Loadout have no formation controls,
- leaving Item cancels only the draft and preserves the committed order.

Judgment:

The final user-directed surface placement is complete. Renaming the visible SafeZone `Item` label remains a separate pending UX wording decision.
