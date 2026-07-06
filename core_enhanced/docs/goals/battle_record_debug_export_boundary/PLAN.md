# Battle Record Debug Export Boundary

## Objective

Resolve P-004, P-005, and P-006 from the stacked-code audit by making battle record identity and debug export ownership explicit.

Current policy says `battle_records` is a run-local abnormality codex/observation record: one representative record per `abnormality_uuid`. It is not a per-battle replay archive. At goal start, the code still used names like `battle_uuid` for abnormality-keyed export files, and `RunState::record_battle(...)` both mutated runtime state and wrote JSON debug files under the repository root.

## Source Of Truth Order

1. Runtime code and tests around `RunState`, combat result completion, and battle records.
2. Live generated/export behavior in `battle_records/`, `debug_event_log_exports/`, and test helpers.
3. Canonical docs, especially `docs/game_rulebook.md`.
4. Audit/goal notes.

Do not trust audit wording without re-reading current code. The current codex-style abnormality dedupe is intentional unless a newer canonical policy says otherwise.

## Scope

- Rename battle-record export fields and helper parameter names so abnormality-keyed records do not masquerade as per-battle UUID records.
- Split runtime state mutation from JSON debug export.
- Move test/debug generated files away from repository-root tracked output when feasible.
- Keep `abnormality_uuid` dedupe behavior.

## Non-Goals

- Do not introduce true per-battle replay archive semantics.
- Do not change combat result rewards, research, node completion, or battle outcome logic.
- Do not make battle record JSON a gameplay source of truth.
- Do not remove the ability to export debug records unless a replacement debug path exists.

## Plan

1. Re-read:
   - `src/game/world/state.rs`
   - combat result callers in `src/game/world/combat.rs`
   - tests that assert `battle_records` and exported JSON
   - helpers that write `debug_event_log_exports`
   - canonical `battle_records` wording in docs.
2. Rename identity fields and helpers:
   - Prefer names such as `abnormality_uuid`, `codex_record_uuid`, or `record_subject_uuid` over `battle_uuid` for abnormality-keyed records.
   - Keep true battle transport IDs named `battle_uuid`.
3. Separate responsibilities:
   - runtime mutation method stores the representative codex record;
   - export helper writes debug JSON and is clearly debug/export-only;
   - call order should be obvious and tested.
4. Quarantine generated outputs:
   - move default test/debug exports to `target/` or a temp directory;
   - keep assertions on data contents in tests instead of requiring repository-root generated files;
   - remove or stop modifying tracked generated artifacts if they are not fixtures.
5. Update docs that mention battle record purpose or export path.
6. Run focused tests after each unit of change.

## Completion Conditions

- No abnormality-keyed battle record export field is named `battle_uuid`.
- `RunState` code makes state mutation and debug export responsibilities distinct.
- Tests no longer depend on modifying repository-root generated output.
- Existing codex behavior remains: one representative record per `abnormality_uuid`.
- Per-battle replay/archive semantics are not introduced accidentally.
- Canonical docs distinguish codex/observation records from future battle-uuid replay archives.

## Implementation Summary

- `RunState::store_abnormality_battle_record(...)` now owns only the run-local representative codex/observation record mutation.
- `RunState::export_abnormality_battle_record_debug_json(...)` now owns JSON debug export.
- Debug export JSON uses `abnormality_uuid`, not `battle_uuid`, for the abnormality-keyed record subject.
- Default debug export paths now live under `target/battle_records/` and `target/debug_event_log_exports/`, not repository-root output directories.
- `docs/game_rulebook.md` and `docs/refactor_preparation_plan.md` now describe the abnormality-keyed codex record and debug export locations.

## Validation Commands

Adjust filters after reading exact test names:

- `cargo test -p game_core battle_record --lib -- --test-threads=1`
- `cargo test -p game_core combat_result --lib -- --test-threads=1`
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

Also run a repository cleanliness check for generated outputs after tests:

- `git status --short battle_records debug_event_log_exports logs`

## Stop Conditions

Complete the goal and report questions if:

- A gameplay feature requires true per-battle replay archive semantics.
- JSON export path must remain repository-root for an external tool.
- Existing tracked files under `battle_records/`, `debug_event_log_exports/`, or `logs/` are confirmed to be intentional golden fixtures.
- Changing export names would require a Unity-facing DTO or save-data migration.
