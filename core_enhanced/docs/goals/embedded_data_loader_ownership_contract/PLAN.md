# Embedded Data Loader Ownership Contract

## Objective

Define and implement a clear ownership contract for embedded RON/builtin data loaders.

`GameDataBase::load_live_embedded()` is already the official embedded live RON loader for the main game-data bundle. However, the codebase still contains domain-level embedded loaders such as map templates, map generation policy, combat preview policy, run policy helpers, buff builtins, and test audit manifests.

This goal decides which embedded data belongs inside the `GameDataBase` live bundle and which data is allowed to remain in explicitly named domain-owned builtin loaders.

The goal is not hot reload and not mod support.

## Source Of Truth Order

1. Current runtime code and actual loader call sites.
2. Live RON/data layout under `game_resources/data`.
3. Production server startup path.
4. Live RON integration tests and probe/smoke tests.
5. Canonical docs describing data ownership and validation.

## Non-Goals

- Do not introduce filesystem loading, hot reload, data packs, or mod overlay behavior.
- Do not change live RON schema unless ownership cannot be made explicit otherwise.
- Do not merge unrelated catalogs into `GameDataBase` merely to remove every `include_str!`.
- Do not create compatibility loader paths or dual schemas.
- Do not change Unity-facing DTO shape.

## Current Suspect Area

Known embedded readers include, but may not be limited to:

- `GameDataBase::load_live_embedded()` in `src/game/data/mod.rs`.
- `src/game/map/types.rs`.
- `src/game/map/generator.rs`.
- `src/game/combat_preview/mod.rs`.
- `src/game/data/run_policy_data.rs`.
- `src/game/data/corroded_wave_data.rs`.
- `src/game/battle/buffs.rs`.
- test-only audit manifests such as `tests/live_skill_catalog_audit.rs`.

Re-run search before implementing:

```text
rg -n "include_str!|load_live_embedded|from_ron_str" src tests ../game_server -S
```

## Desired Design Direction

The final design should make loader ownership obvious:

```text
GameDataBase::load_live_embedded()
  owns production live game-data bundle domains used by runtime game rules.

DomainBuiltinLoader / explicit domain methods
  may own truly static engine/catalog policy data when keeping them outside GameDataBase reduces coupling.

Tests
  may load test/audit manifests directly if they are not runtime live game data.
```

If a domain loader remains outside `GameDataBase`, document why it is domain-owned and how it is validated.

Adding a new production live gameplay RON domain should have one obvious place to wire it.

## Plan

1. Inventory all embedded RON and builtin loader call sites.
2. Classify each loader as one of:
   - `GameDataBase live domain`;
   - `domain-owned builtin policy/catalog`;
   - `test-only audit fixture`;
   - obsolete/duplicate loader to remove.
3. Record the classification table in `EXPERIMENT_NOTES.md` before editing.
4. Decide whether any current domain-owned loader should move into `GameDataBase`.
5. If moving data ownership:
   - update the data structs/builders;
   - route production/server/tests through the chosen owner;
   - remove duplicate loader paths.
6. If keeping domain-owned loaders:
   - rename or document them so they cannot be mistaken for alternate live game-data loaders;
   - add or verify validation coverage.
7. Update canonical data-loading documentation if ownership policy changes.
8. Run focused live RON tests and server/core checks.

## Completion Conditions

- Every embedded RON/builtin loader is classified.
- Production live game-data domains have one official ownership path.
- Domain-owned builtin loaders, if retained, are explicit and justified.
- Test-only direct `include_str!` usage is clearly separated from runtime live loading.
- No hidden duplicate production live loader remains.
- Server startup and live RON tests still use the same production game-data loader path.
- Goal docs record the final ownership policy and validation results.

## Validation Commands

Start with inventory:

- `rg -n "include_str!|load_live_embedded|from_ron_str" src tests ../game_server -S`

Then run:

- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core --test ron_loading -- --test-threads=1`
- `cargo test -p game_core --test live_skill_catalog_audit -- --test-threads=1`

If map/combat-preview ownership changes, also run relevant map/combat preview tests discovered during inspection.

## Stop Conditions

Complete the goal and report questions before continuing if:

- It is unclear whether a data domain should be owned by `GameDataBase` or a separate domain catalog.
- Moving a loader would introduce a dependency cycle or heavy coupling.
- A live RON schema change, DTO change, or runtime data migration becomes necessary.
- Hot reload, mod overlay, or filesystem data-pack policy becomes part of the solution.
