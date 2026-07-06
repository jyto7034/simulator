# Experiment Notes

## Initial Audit Basis At Goal Start

- `RunState.battle_records` is documented as abnormality codex/observation state keyed by `abnormality_uuid`.
- `BattleRecordExport.battle_uuid` received `battle.abnormality_uuid`, which was a naming mismatch.
- `RunState::record_battle(...)` both wrote a JSON file and pushed the record into runtime state.
- Repository-root generated files under `battle_records/`, `debug_event_log_exports/`, and `logs/` were tracked or modified by test/debug runs.

## Policy Notes

- Keep the current abnormality-level dedupe unless the user explicitly chooses true per-battle archive semantics.
- Treat JSON output as debug/export artifact, not gameplay source of truth.
- If a generated JSON file is actually a golden fixture, document that explicitly before moving it.

## Implementation Notes

- The current runtime still keeps the public in-memory field name `battle_records` because canonical docs use it as the codex/observation record collection name. The ambiguous parts were the JSON field/helper names and mixed mutation/export responsibility.
- `store_abnormality_battle_record` returns whether a new representative record was inserted. The combat result flow exports debug JSON only for newly inserted representative records, preserving the existing one-record-per-`abnormality_uuid` behavior.
- `target/battle_records/` and `target/debug_event_log_exports/` are intentionally debug output locations. They are not gameplay source of truth and are not golden fixtures.
- Repo-root tracked generated files were not removed in this goal. The implementation stops newly run tests from depending on or rewriting those paths.
- The failed `skill_test_suite` validation is likely from a separate static-data/test-fixture drift: the dummy abnormality used by the skill tests does not satisfy the current `response_complete_skill_fragment_id` validation.

## Follow-Up Candidates Outside Scope

- Introduce a real battle-attempt UUID separate from map node UUID.
- Build a dedicated replay archive contract keyed by battle instance UUID.
