# Battle Record Removal Experiments

Goal: G16

Updated: 2026-07-14

## 2026-07-14 — Dirty baseline

- Core: branch `main`, HEAD `c2c5dc5ec54bd26530e46f0d7ee7909de3ed7a1f`; scoped fingerprint
  `599daece65b6d245421d92235ba295f674351a1dcec4bdde887ed37f52c6c73e`.
- game_server: branch `main`, same HEAD; full diff fingerprint
  `693e1ff42b3a8604e6839fd777c128a9ec7b9db9b1ee63cfe7b18edef150090b`.
- Unity ArkClient: branch `master`, HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`; full diff
  fingerprint `ac0fff23bf4bd07ad8693c63af3c2a32955b091f89a364cc63e5f1f2177a4809`.
- Relevant Core files overlap completed predecessor work and must be edited hunk-by-hunk. Eleven
  tracked repository-root battle-record JSON files and existing debug export JSON files were already
  deleted in the baseline; those deletions align with PD-12 and must not be restored.

## Initial trace

- `RunState.battle_records` retains full `CombatBattleState`, including event log/result facts,
  across floors and run completion.
- Checkpoint load deliberately preserves current records instead of restoring the checkpoint copy.
- `GameCore::battle_records()` exposes the retained state publicly.
- Finished live battle transition first inserts an abnormality-keyed representative record, then
  writes `target/battle_records/run_<seed>/<abnormality_uuid>.json`.
- Export failure currently propagates through `?` and can reject the authoritative transition to
  CombatResult. This violates accepted PD-12 even though the output directory is under `target`.
- Active CombatResult already owns the current `CombatBattleState` and supplies its event-log
  attachment without reading `RunState.battle_records`; it can remain independent.

## Implementation

- Removed `RunState.battle_records`, all floor/run-complete/checkpoint copy or preserve paths, the
  public `GameCore::battle_records()` query, the abnormality representative-record store/export
  methods, and record-specific test expectations.
- Added the diagnostic-only `world::combat_trace` module. It writes
  `target/combat_traces/run_<seed>/<battle_uuid>.json`, carries both the actual battle UUID and the
  combat node UUID, and is not referenced by run/save/checkpoint/profile state.
- The live finish seam attempts trace export before publishing CombatResult, logs any error, and
  continues the same authoritative transition. A forced directory-creation failure test proves the
  error cannot reject or change the result transition.
- Live defense tests now inspect the battle-keyed trace directly and independently assert active
  CombatResult/result-event continuity. Floor/run-complete tests now cover only the run-scoped
  research state that is actually retained; retreat snapshot equality remains the no-mutation
  proof.
- Canonical runtime/rulebook and engineering workflow text now states that active result facts end
  with their lifecycle, player battle history does not exist, story documents are independent, and
  developer traces are non-authoritative and best-effort.

## Focused validation

- Initial focused compile failed with one `GameError` Display formatting error, two owned UUID
  dereference errors, and unused imports left by record removal. Corrected to debug formatting,
  owned UUID matching, and exact imports.
- Two first state-test invocations used `--exact` without the module-qualified name and therefore
  ran 0 tests. They are excluded from evidence and were rerun with nonzero filters.
- `next_floor_run_state_preserves_research_and_resets_node_scoped_state`: 1 passed.
- `run_complete_state_preserves_run_scoped_research`: 1 passed.
- `defense_combat_node_smoke_writes_debug_event_log_export`: 1 passed.
- `live_ron_defense_route_playable_path_runs_to_combat_result`: 1 passed.
- The first forced-failure test compile found `Option::and_then`/`Result` mismatch in a test-only
  assertion. Corrected to unwrap the known active combat content explicitly.
- `combat_trace_export_failure_does_not_block_result_transition`: 1 passed.

## Final validation and audit

- Retreat-focused Core tests: 5 passed; exact pre-entry snapshot/RNG continuation, repeated retreat,
  final boss retreat, Event-started combat retreat, and battle-local warning rollback all passed.
- Checkpoint visible-state restore: 1 passed.
- `cargo check --all-targets`: passed without warnings.
- `cargo test --all-targets`: 738 passed, 0 failed, 0 ignored across nonzero targets: 696 lib,
  3 live item/skill activation, 3 live skill catalog, 19 live RON, 13 skill runtime contract, and
  4 skill suite tests. Zero-test binary/example targets are excluded from the count.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- Production Core, game_server, Unity Assets, canonical runtime/rulebook/loading, and engineering
  workflow searches found no `battle_records`, `battle_record`, or `target/battle_records` use.
  Historical goal/audit/tmp records retain old names only as historical evidence.
- No relevant ignored test, serde alias, compatibility field, replacement history, or diagnostic
  consumer exists. The only current diagnostic path is battle-keyed `target/combat_traces`.
- No live RON, serialized DTO, server transport, or Unity reader changed, so server and Unity suites
  were not rerun.
- Final scoped tracked-diff fingerprint:
  `fecff4eb3828f1bbfcd50d8ec4b47c473fe8ca844e1d564765d72c8e683330bb`; new diagnostic module
  fingerprint: `e7a3382180b51905faf2cdc634f4d5d7a44183606daaf9ca15424ae48b003e01`.
