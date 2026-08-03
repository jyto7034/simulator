# Illustrated Employee-Check Event Flow Experiments

Goal: G03

Updated: 2026-07-14

## 2026-07-14 — Goal initialized

Question:

What is the smallest coordinated cutover from the current provisional Event scene/effect model to
the accepted illustrated employee-check contract?

Evidence available at initialization:

- G02 established one shared root/result Event inner DTO seam and corrected generic wire identities.
- Phase 1 capability/strict-loader and Phase 2 battle transaction/recovery reviews are Complete /
  High.
- Current code still contains provisional Event scene choices, Event-to-VN trigger behavior, and no
  Unity Event route.

Claim status:

No G03 implementation success is claimed. Inventory current runtime/data/Unity seams, design the
single migration, then create red tests before production edits.

## 2026-07-14 — Current Event seam inventory

Question:

Which provisional owners and duplicate paths must the coordinated cutover replace?

Evidence:

- Live `white_night_confession_01` uses flat `background_id + script_ids`, preview hints, two
  no-effect narrative choices, and no employee/check/result state.
- `EventSessionState` persists scene, committed choice, completed scenes, staged rewards, and
  started combat, but no selected employee, roll, outcome, or once-only success marker.
- `event_node.rs` immediately commits `SelectEventChoice`, supports separate immediate/staged
  effects and Event-specific combat reward timing, and invokes `EventSceneEntered` VN triggers.
- Missing combat references still panic inside runtime planning even though G12 supplies a strict
  shared bundle-validation seam.
- Event actions are absent from the current common capability projector/preflight; dispatch routes
  them directly by behavior variant.
- Unity has no Event route/composer/typed nested reader or command methods. `SelectedEventModel.Raw`
  remains the only generic access and `in_node + type:event` resolves to Unknown.
- G02 already guarantees the root/result envelope is one inner DTO, so the migration need only
  replace that inner type once.

Result:

The accepted flow requires one coordinated data/session/DTO/capability/handler/Unity cutover. An
incremental alias window would preserve all current ownership defects and is rejected.

Decision and next step:

Design the exact new authoring/session/action state and classify the current live no-effect choices
before adding red serialization, roll, idempotence, validation, and Unity route tests.

## 2026-07-14 — Dirty-worktree baseline

- Core branch/HEAD: `main` / `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Unity branch/HEAD: `master` / `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Core scoped Event fingerprint:
  `3a97d8b6f12bd8a5b869e1d6da5d3ac79327c68b70d3e82eb4578a9ee426a48c`.
- Unity scoped Event fingerprint:
  `e06342087a9d83c46488689cfb6a42e350f279d73159e336c0246898dee13d03`.
- Both repositories are heavily dirty. Target Event files overlap completed predecessor work
  (strict loader, capability, VN, Phase 2, and G02) and are the intended continuation surface.
  Unrelated screenshots, IDE files, deleted historical docs/assets, and other UI work remain
  user-owned and will not be modified, restored, stashed, or cleaned.

## 2026-07-14 — Core Event contract cutover and first focused validation

Implemented contract:

- Replaced flat `background_id/script_ids` with ordered `presentation.panels[]`.
- Replaced preview/selectable/immediate/staged choice authoring with one
  `EmployeeCheckChoices { required_roll, choices }` state and one optional success action.
- Replaced `SelectEventChoice` with exact scene-qualified `ResolveEventChoiceCheck`, including the
  selected employee UUID.
- Added living-only employee candidate eligibility, one deterministic d20 draw, persisted
  performer/roll/threshold/outcome, critical 1/20 classification, exact duplicate idempotence, and
  normal failure result panels.
- Moved Event actions into the common capability projector/preflight through shared Event
  evaluator/payload validators.
- Removed Event-to-VN `EventSceneEntered`, Event-specific staged combat reward policy, and runtime
  panic lookup paths. Combat success actions use the normal live battle transaction/retry path.
- Migrated live White Night story content to typed advance-only panels without inventing balance,
  combat, or reward semantics.

Validation evidence:

- `cargo check --lib`: passed after production loader/runtime cutover.
- First `cargo check --all-targets`: passed; test fixture migration compiled.
- `cargo test --lib event`: 54 passed, 0 failed, 0 ignored, 630 filtered before the final shared
  evaluator refactor.
- After the shared evaluator refactor,
  `cargo test --lib 'game::world::tests::map_flow::event_'`: 7 passed, 0 failed, 0 ignored, 679
  filtered.
- `cargo test --lib resolve_event_choice_check_uses_one_exact_scene_qualified_wire_command`: 1
  passed, 0 failed, 0 ignored, 685 filtered.
- Focused coverage pins typed serialization, root/result DTO identity, capability projection,
  stale identity rejection, living-only eligibility, one-time grant, exact duplicate result reuse,
  failure state/final advance, combat start, and retreat preservation.

Environment note:

- The resumed sandbox made the shared parent Cargo target read-only, so validation moved to
  `CARGO_TARGET_DIR=/tmp/core_enhanced_target`. One focused test process was interrupted by the
  automatic continuation boundary before producing a result; the identical command was rerun and
  passed 7/7.

## 2026-07-14 — Live admin Event fixture focused validation

- Added `AdminEnterEvent { event_id }` as an explicit debug-only path into the production live Event
  projection. It creates the same typed `EventSessionState`; it does not add a server mapping or an
  Event-specific transport path.
- The first focused compile failed because the new test referenced a nonexistent `started_core`
  helper. The test was corrected to use `GameDataBase::load_live_embedded()` plus the existing
  `core_with_run` setup.
- Identical rerun:
  `cargo test --lib admin_enter_event_uses_live_typed_event_projection -- --nocapture`: 1 passed,
  0 failed, 0 ignored, 686 filtered.
- The assertion pins the live `white_night_confession_01` typed Event route, exact Event id,
  advance interaction, and enabled `AdvanceEventScene` capability.

## 2026-07-14 — Actual Rust server to Unity Event round trip

- The first server launch from the workspace root built successfully but exited before binding:
  its relative `config/development` path was not present in that working directory. Restarting the
  identical binary from `game_server/` loaded the production live bundle and bound port 8081.
- Ran only
  `ArkClient.Tests.PlayMode.ArkClientLiveFlowTests.LiveAdmin_EventUsesTypedRouteAndExactAdvanceCommand`
  against that server with admin commands enabled: 1 passed, 0 failed, 0 skipped (Unity job
  `1ce4702781174540b5737e133e063e07`).
- The flow initialized a real run, entered `white_night_confession_01` through `AdminEnterEvent`,
  observed the first-class typed Event route, and sent the exact scene-qualified
  `AdvanceEventScene` command. The server log shows the transition into the admin Event node; the
  server then shut down gracefully via Ctrl+C.

## 2026-07-14 — Unity typed Event implementation and fixture validation

Implemented:

- Added one strict `EventSceneModel` over the G02 shared inner DTO; it has no Event Raw fallback.
- Added `GameScreenRoute.Event`, dedicated Event body layout/panels, `EventScreenComposer`, local
  panel/text cursor and local uncommitted choice/employee selection.
- Added exact `AdvanceEventScene` and `ResolveEventChoiceCheck` command senders/handlers and removed
  the old Event select action name.
- Added an illustration resource hook to the UGUI projection, a current Event mock, dedicated
  layout anchors, and committed d20 result display.
- Generated `Assets/Tests/Fixtures/event_employee_check_contract.json` from the Rust fixture binary
  and consumed it in Unity rather than hand-maintaining a competing DTO example.

Focused validation:

- Initial Event EditMode suite: 6/6 passed after the first five tests and the committed-result /
  advance test were combined.
- The first Event PlayMode render assertion expected the exact choice label, while the UGUI selected
  state intentionally prefixes a marker. The assertion was corrected to match rendered content;
  rerun passed 1/1.
- Actual live Rust-server PlayMode flow passed 1/1.

## 2026-07-14 — Completion-review correction: fail-closed Unity scalar types

- Review found that the first reader used `ToString()` for non-string scalar fields and filtered
  malformed `text_ids` entries. That could accept a numerically typed identity or a mixed array
  even though the wire contract is exact.
- Corrected the reader to require JSON strings for string fields, require every `text_ids` entry,
  require boolean eligibility objects for choices/employees, and require integer roll fields.
- Added `Reader_RejectsNonStringIdentityMalformedTextAndMissingEligibility`.
- Final focused Event EditMode: 7 passed, 0 failed, 0 skipped (job
  `80dc25ffc32a4ac3a65930b79cffad10`).
- Final Event PlayMode render: 1 passed (job `ab266a4f858c4c14829037cc91b2fd40`).
- Final live Rust-server Event round trip on the strict reader: 1 passed (job
  `67e311a44da54a9c988741824558bb74`).

## 2026-07-14 — Final broad, formatting, canonical, and legacy validation

- Core `cargo test --all-targets`: 728 passed total (687 lib + 41 integration), 0 failed, 0
  ignored. Zero-test binary targets were accounted for and are not used as evidence.
- Core `cargo check --all-targets`: passed.
- game_server `cargo test --all-targets`: 28 passed, 0 failed, 0 ignored; zero-test binary/example
  targets were accounted for.
- First `cargo fmt --all -- --check` reported standard formatting diffs in the G03 Rust edits.
  `cargo fmt --all` was applied and the identical check then passed.
- Unity final EditMode: 201/201 passed (job `5a30f623197a495cae0b3f1ede20bd0f`).
- Unity ordinary PlayMode classes (mock render, reconnect, Event render): 52/52 passed (job
  `2591db8e8a48482cae4d29e13924ac83`). Explicit live Event: 1/1 passed; console was cleared after
  the test runner's result-file log and then contained 0 errors.
- Canonical Core runtime/rulebook/loading documents and Unity core/screen documents now describe
  panels, tagged interaction, exact resolve identity, persisted d20, and Event/VN separation.
- Production legacy search found no `SelectEventChoice`, `EventSceneEntered`, immediate/staged
  Event effects, combat reward commit, Event Raw reader, or old flat live schema. Lowercase
  `select_event_choice` and `background_id/script_ids` hits are intentional strict rejection tests
  only. Relevant Event tests contain no ignore marker.
