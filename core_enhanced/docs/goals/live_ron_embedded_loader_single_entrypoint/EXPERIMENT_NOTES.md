# Live RON Embedded Loader Single Entrypoint Notes

## Policy Decision

Keep embedded RON loading for now.

This goal should not implement hot reload, data packs, or mod overlays. RON edits require recompilation.

## Initial Judgment

Loader drift has already caused real failures around newly added data domains such as event data and boss omen chains. A single embedded loader is the long-term direction.

The official loader should live on `GameDataBase` so both `game_core` integration tests and `game_server` can call the same embedded RON path without adding filesystem or hot-reload semantics.

Validation remains part of the official loader because the previous production server path already validated generated combat previews. Moving tests to the same loader intentionally exposes live data contract failures that test-only loaders previously skipped.

## Guardrails

- Do not merge the RON files themselves.
- Do not weaken validation to make loader delegation easier.
- Do not hide custom test fixtures behind the live loader name.

## Follow-Up Candidates

- Future `load_from_files(path)` if modding/hot reload becomes a real product goal.
- Data versioning and overlay policy for future mod support.
