# Live RON Embedded Loader Single Entrypoint

## Objective

Create one official embedded live RON loader path for `GameDataBase` and make production/server and live-data tests delegate to it.

The goal is not hot reload. Current policy is to keep `include_str!` embedded RON loading.

## Source Of Truth Order

1. Runtime loader code in core and game_server.
2. Live RON/data files.
3. Live RON tests.
4. Current canonical docs and audit note.

## Plan

1. [x] Re-read current loader copies:
   - `../game_server/src/main.rs`
   - `tests/common/mod.rs`
   - `src/game/world/tests/mod.rs`
   - `src/game/data/mod.rs`
2. [x] Design a core-side official loader, likely in `src/game/data/mod.rs`, with an explicit embedded name such as:
   - `GameDataBase::load_live_embedded()`
   - or `load_live_embedded_game_data()`
3. [x] Move embedded RON assembly into the official loader.
4. [x] Make `game_server` delegate to that loader.
5. [x] Make live RON integration tests delegate to that loader.
6. [x] Keep custom test fixture builders separate and explicit.
7. [x] Do not introduce filesystem loading or hot reload.
8. [x] Run server/core checks and live RON tests.

## Completion Notes

- `GameDataBase::load_live_embedded()` is now the official embedded live RON loader.
- `game_server` and live RON integration tests delegate to the same loader.
- Custom fixture builders still use `GameDataBuilder::empty()` and remain separate from live RON loading.
- Loader convergence exposed a live PVE spawn-zone mismatch; the authored wave now lets preview generation resolve a valid spawn zone from the selected battlefield template.

## Completion Conditions

- Live RON assembly exists in one official core-side embedded loader path.
- Production game server and live RON tests use the same loader path.
- Embedded RON policy is visible in naming or documentation.
- Adding a new live RON domain should require changing one loader path.
- Existing custom tests can still build minimal `GameDataBase` fixtures without pretending to be live RON.

## Validation Commands

- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core --test ron_loading -- --test-threads=1`
- `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1`

Use `CARGO_TARGET_DIR=/tmp/core_enhanced_target` if the workspace target directory is unavailable in the sandbox.

## Stop Conditions

Complete the goal and report questions if:

- The loader move requires changing public crate boundaries.
- Server cannot call the core loader cleanly without a dependency cycle.
- A live data domain should be optionally excluded in tests.
- Runtime hot reload, mod overlay, or filesystem data packs become necessary.
