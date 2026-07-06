# Enemy Targeting and Wave Preset Cleanup Experiments

| Date | Experiment | Result | Next Step |
| --- | --- | --- | --- |
| 2026-07-02 | Goal setup from doc/code audit | Found two remaining contract mismatches worth implementing together: enemy ranged target selection ignores `targeting_profile` for non-player attackers, and `CorrodedWavePreset` still uses obsolete `difficulty` terminology after encounter difficulty removal. | Start with runtime/data audit before editing code. |
| 2026-07-02 | Follow-up audit of `CorrodedWavePreset.difficulty` impact | Confirmed `difficulty` is not read by generated wave resolution. Runtime wave strength uses `budget`, `count_range`, `role_mix`, optional `budget_override`, and Floor scaling multiplier. | Narrow Phase 3 from "rename or remove" to direct removal with no replacement field. |
| 2026-07-02 | Phase 1 targeting audit | `select_basic_attack_target` preserves fixed defense route blocker priority, airborne special handling, ranged persisted-target validation, then calls `choose_attack_target_in_range` for new target selection. `choose_attack_target_in_range` applies tile/usefulness filtering but non-player attackers still sort by nearest world distance. `closest_enemy_in_attack_range` is a separate movement-goal helper used outside the fixed defense route ranged target path. | Repair `choose_attack_target_in_range`; record `closest_enemy_in_attack_range` as a follow-up risk unless focused tests prove it belongs in this goal. |
| 2026-07-02 | Phase 2 enemy targeting repair | Changed `choose_attack_target_in_range` so all non-airborne attackers sort filtered candidates through `targeting_profile`; fixed defense route blocker and persisted-target behavior remain in `select_basic_attack_target`. Added enemy-focused tests for low-defense, low-magic-resist, blocker priority, persisted target priority, and out-of-range exclusion. | Continue to wave preset schema cleanup. |
| 2026-07-02 | Phase 3 wave preset schema cleanup | Removed `CorrodedWavePreset.difficulty` from runtime schema, Rust fixtures, and live `corroded_wave_presets.ron`. Added `deny_unknown_fields` on the wave preset RON structs and focused tests for live parse success plus stale `difficulty` rejection. | Run broader validation and finish docs. |
| 2026-07-02 | Phase 4 contract sync | Canonical docs already described ranged basic attacks as `targeting_profile` driven, but broad wording still mentioned distance as a generic sorting criterion. Updated `game_rulebook.md` and `skill_target_contract.md` to describe profile-specific fallback/tie-breaker behavior instead. No Unity-facing DTO shape changed. | Final verification. |

## Verification Log

Record every focused test, failed attempt, and broad validation command here during implementation.

Use this format:

```text
YYYY-MM-DD
Command:
Result:
Failure cause:
Fix:
Re-run result:
```

```text
2026-07-02
Command: cargo test -p core_enhanced game::battle::core::targeting::tests:: -- --nocapture
Result: Failed before running tests.
Failure cause: Cargo package is named game_core, not core_enhanced.
Fix: Re-run with -p game_core.
Re-run result: cargo test -p game_core game::battle::core::targeting::tests:: -- --nocapture failed during integration-test compilation because unrelated test fixtures are missing AbnormalityMetadata.omen_chain_id.
```

```text
2026-07-02
Command: cargo test -p game_core --lib game::battle::core::targeting::tests:: -- --nocapture
Result: Failed 1 of 11 targeting tests.
Failure cause: The new out-of-range test used a custom one-row TileRangePattern that did not match the existing facing/pattern coordinate assumptions.
Fix: Use the existing broad fixture pattern and place the excluded target outside that pattern.
Re-run result: Passed. 11 passed; 0 failed.
```

```text
2026-07-02
Command: cargo test -p game_core --lib game::data::corroded_wave_data::tests:: -- --nocapture
Result: Passed. 2 passed; 0 failed.
Failure cause: n/a
Fix: n/a
Re-run result: n/a
```

```text
2026-07-02
Command: cargo test -p game_core --lib game::wave_resolution::tests:: -- --nocapture
Result: Passed. 2 passed; 0 failed.
Failure cause: n/a
Fix: n/a
Re-run result: n/a
```

```text
2026-07-02
Command: cargo test -p game_core --lib generated_corroded_wave_source_resolves_during_preview_generation -- --nocapture
Result: Passed. 1 passed; 0 failed.
Failure cause: n/a
Fix: n/a
Re-run result: n/a
```

```text
2026-07-02
Command: cargo check -p game_core
Result: Passed.
Failure cause: n/a
Fix: n/a
Re-run result: n/a
```

```text
2026-07-02
Command: cargo test -p game_core --lib
Result: Passed. 589 passed; 0 failed.
Failure cause: n/a
Fix: n/a
Re-run result: n/a
```
