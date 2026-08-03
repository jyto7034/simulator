# Unity PlayMode Pointer Isolation Correction Experiments

Goal: G31

Updated: 2026-07-14

## Entry evidence

- R05 full PlayMode failed 8/56 after focused hover tests.
- The first failure expected Ada but displayed Cyra; its exact rerun failed identically.
- The existing method-local `WarpCursorPosition(Vector2.zero)` plus one frame is therefore not a
  reliable Input System reset. Seven later failures are consistent with aborted method-local cleanup.

## Dirty baseline

- Simulator status fingerprint:
  `5978cca65e31b5bd8f008165bad1af1b30cbcf7d56a100240574cae0f0c0abb0`.
- Unity status fingerprint:
  `d4cbf7fe310ed5a6d2eb3337c344bc75b4963519f0caa944b70410bd9908daee`.
- Preserve all prior changes; only the PlayMode test isolation seam and G31/R05/master evidence are
  writable for this correction.

## Design

- Use class-level `UnitySetUp`/`UnityTearDown`, which NUnit runs around coroutine tests including
  assertion failures.
- Capture root GameObject instance ids across loaded scenes before each test and destroy only roots
  created after that baseline during teardown.
- Set `Mouse.current.position` immediately through Input System `InputState.Change` to an off-screen
  coordinate before and after a frame, and clear EventSystem selection.
- Reflection against the loaded Input System confirmed the generic `InputState.Change(InputControl,
  TState, ...)` API in the current package.

## Implementation

- Added class-level `UnitySetUp` that captures all loaded-scene root ids, immediately moves the
  current mouse control off-screen, waits one frame, repeats the state change, and clears selected UI.
- Added `UnityTearDown` that destroys only roots created after the per-test baseline, waits for
  destruction, neutralizes pointer state, and clears the baseline even after assertion failure.
- Removed the method-local warp/yield patch. Production starter hover code and assertions are
  unchanged.

## Validation

- Exact previously failing starter test: 1/1.
- Previously contaminating focused HUD/pending-preview PlayMode sequence: 2/2.
- Full PlayMode immediately after that sequence: 52 passed, 0 failed, four explicit external-server
  skips.
- Second full PlayMode without manual cleanup: 52 passed, 0 failed, four explicit skips.
- Full EditMode: 215/215. Final cleared console: zero errors.
- Scoped Unity whitespace and retry/order/ignore/warp/expected-name-relaxation audits passed.
