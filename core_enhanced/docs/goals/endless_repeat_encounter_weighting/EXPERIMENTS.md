# Endless Repeat Encounter Weighting Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Goal creation | Split repeat encounter weighting from research state and boss omen chains. | Start by auditing `map_encounters.rs` and the final research state API. |
| 2026-06-27 | Runtime selection audit | `map_encounters.rs` currently builds difficulty/node-type candidates, then silently falls back to all non-boss encounters and finally all encounters if candidates are empty. Live PVE encounter difficulties currently span `1..=7`, while Endless difficulty uses `floor_index * 2`, so high Endless floors can exceed authored pools even before recent-Floor exclusion. This hits the goal stop condition for empty candidate/fallback policy. | Stop before implementation and ask for explicit fallback/difficulty cap policy. |
| 2026-06-27 | Empty-pool fallback policy resolved | User clarified that if the pool is empty after recent/repeat conditions, those soft conditions should be reset. Hard constraints such as encounter class, node kind/category, game mode, and final boss policy must remain intact. | Update the plan and implement hard-constrained selection plus soft repeat-weight reset. |
| 2026-06-27 | Repeat weighting implementation | Added live-RON `encounter_repeat_weighting`, passed run-local abnormality research into map encounter assignment, applied Endless-only soft recent/response-complete weighting to primary-abnormality candidates, and preserved hard constraints when soft fallback resets. | Run broad validation and update master phase status. |
| 2026-06-27 | Final validation | Focused checks, `cargo check`, full `cargo test --lib`, and live RON loading test passed. | Phase 4B completion criteria satisfied. |
| 2026-06-27 | Appearance history correction | User clarified recent exclusion should use actual appearance/entry history, not victory-only research history. Added separate run-local abnormality encounter history and kept research state as response-complete source. | Focused repeat/map/checkpoint tests and broad validation passed. |

## Verification Commands

Record every focused and broad check here while implementing.

Focused checks run:

```bash
cargo test encounter_repeat --lib
cargo test repeat_weight --lib
cargo test empty_repeat_candidate --lib
cargo test standard_selection_does_not_apply --lib
cargo test map_flow --lib
cargo test run_policy --lib
cargo test recent_appearance --lib
cargo test abnormality_run_state --lib
cargo check
cargo check --lib
```

Broad checks run:

```bash
cargo test --lib
cargo test --test ron_loading
```

## Failed Attempts

### 2026-06-27: Stopped before fallback policy was decided

Runtime audit stopped before edits because the current empty-candidate behavior required a user policy decision.

Resolution:

- User approved resetting only soft repeat-weighting constraints when they empty the pool.
- Hard constraints must remain intact.
- Implementation may proceed.
