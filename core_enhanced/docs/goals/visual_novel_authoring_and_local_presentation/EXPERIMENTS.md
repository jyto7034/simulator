# Visual Novel Authoring And Local Presentation Experiments

Goal: G19

Updated: 2026-07-14

## 2026-07-14 — Goal initialized

Question:

What is the smallest complete cleanup that removes duplicate VN authoring authority and closes the
known Endless local-presentation gap without changing VN gameplay or content policy?

Fixed evidence:

- G03 is Complete / High and Event has no VN dependency/fallback.
- DCA-M-11 identifies unused `VisualNovelDefinition.default_blocking`; runtime sessions use trigger
  `blocking`.
- DCA-L-01 identifies a live Endless VN/script id absent from Unity's local catalog, where the
  current missing-script path may display `"..."`.

Claim status:

No G19 implementation success is claimed. Inventory exact live/runtime/Unity ids and fallbacks,
record the dirty baseline, then add negative/coverage tests before production edits.

## 2026-07-14 — Dirty-worktree baseline

- Core/game_server branch and HEAD: `main` /
  `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`.
- Unity branch and HEAD: `master` / `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- Scoped source hashes before G19:
  - Core VN definition `2be48afc...`, trigger definition `f622f5c0...`, runtime activation
    `52a24ae6...`.
  - live VN RON `eea80460...`, live trigger RON `4216706a...`.
  - server handler fixture `217cad59...`.
  - Unity local catalog `3c080e5a...`, presenter `954018c4...`, bootstrap `63959c44...`, EditMode
    contract test `c4833e28...`, live PlayMode test `f07e4f8e...`.
- Core, game_server, and Unity remain heavily dirty with completed predecessor and unrelated
  user-owned changes. G19 targets only the VN schema fixtures, exact local catalog, related tests,
  and canonical docs; it will not stash, reset, restore, clean, or reformat unrelated files.

## 2026-07-14 — Red contract reproductions

- Added `visual_novel_definition_rejects_removed_default_blocking_authority`. The first compile used
  the unavailable `ron::from_str` path; corrected it to `ron::de::from_str`. The intended rerun then
  failed because the current schema still accepted and populated `default_blocking`.
- Added
  `VisualNovelLocalCatalog_CoversEndlessBriefingAndRejectsMissingScripts`. It executed 1 test and
  failed because `hq_briefing_endless_intro` resolved through the current fallback cue containing
  exactly `"..."`.
- These failures isolate DCA-M-11 and DCA-L-01 without changing gameplay or trigger policy.

## 2026-07-14 — Trigger-only blocking and exact Unity-local presentation cutover

Implemented:

- Removed `VisualNovelDefinition.default_blocking` from the strict Core schema, every Core/server
  fixture, and all three production VN definitions. `VisualNovelTrigger.blocking` remains unchanged
  and is the only value copied into `VisualNovelSession`.
- Added a Core regression that sets the Standard trigger to nonblocking and proves both result DTO
  and active runtime session use that exact trigger value.
- Added exact Unity-local `hq_briefing_endless_intro` cues parallel to the existing Standard briefing.
- Replaced `ScriptFor`'s generated `Narration("...")` fallback with `TryGetScript` plus an explicit
  exception for direct required lookups. The presenter now renders a diagnostic overlay and disables
  local/server advance when a script is missing; bootstrap local advance also fails closed.
- Added an explicit actual-server Endless run-start test covering exact VN/script identity,
  trigger-owned blocking, local cue resolution, rendered line, and transition to starter selection.
- Updated Core and Unity canonical documents with trigger-only blocking, exact Standard/Endless ids,
  missing-script fail-closed behavior, and Event separation.

Focused validation:

- Removed-authority RON rejection: 1/1 passed.
- Trigger-only runtime session blocking: 1/1 passed.
- Core `visual_novel` focused group: 22/22 passed.
- Unity local catalog/missing-script regression: 1/1 passed (job
  `fb66712ee6504542838ea7edcf8deb2a`).
- Actual Rust-server Endless briefing: 1/1 passed (job
  `ee2c0b4b444c4607b972e214692fd473`). The server log shows
  `NotStarted -> PreRunBriefing { Endless } -> SelectingStarterEmployees` and graceful shutdown.

Broad validation:

- Core `cargo check --all-targets`: passed.
- Core `cargo test --all-targets`: 730 passed total (689 lib + 41 integration), 0 failed, 0
  ignored. Zero-test binary targets were accounted for.
- game_server `cargo test --all-targets`: 28 passed, 0 failed, 0 ignored; zero-test binary/example
  targets were accounted for.
- `cargo fmt --all -- --check`: passed without correction.
- Unity EditMode: 202/202 passed (job `82fa9120acba41dba4ca9a92caf812d3`).
- Unity ordinary PlayMode: 52/52 passed (job `942cfafb1dbb450ba99ce10b9ba814b3`).
- Unity console after clearing the test-runner result-file entry: 0 errors.

Legacy/fallback audit:

- Production Core, live RON, and server contain no `default_blocking`; remaining hits are the strict
  rejection test and canonical statement that the removed field is invalid.
- Unity runtime contains no generated `Narration("...")` or missing-script `VisualNovelLocalScript`
  fallback. Presenter/bootstrap production lookups use `TryGetScript` and fail closed.
- Exact Standard/Endless VN and script ids agree across live definition RON, trigger RON, Unity
  catalog, canonical docs, tests, and actual server output.
- VN trigger/runtime sources contain no `EventSceneEntered`; relevant Core/Unity VN tests contain no
  ignore marker.
