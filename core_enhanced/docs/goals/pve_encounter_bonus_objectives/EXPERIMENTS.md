# Pve Encounter Bonus Objectives Experiments

## Experiment Log

| Date | Experiment | Result | Next Action |
| --- | --- | --- | --- |
| 2026-06-26 | Goal creation | Split encounter bonus objectives from run-local abnormality research and repeat encounter weighting. | Start by auditing `PveEncounter`, damage result events, and combat result resolution. |
| 2026-06-27 | Runtime/source audit | `PveEncounter` has no bonus objective schema yet. `BattleLogEvent::HpChanged` carries `DamageSource` and `final_damage`; `UnitSpawned` carries `BattleUnitSourceIdentity` and `UnitStats.max_health`, so `DecisiveDamage` can be evaluated from battle event log without using summed result stats. Live PVE encounters currently use `encounter.abnormality_id` for research identity and do not spawn abnormality units unless explicitly authored. | Implement schema/evaluator and validate bonus-objective encounters require a spawned primary abnormality target. |
| 2026-06-27 | Focused bonus objective evaluator tests | `cargo test pve_bonus_objectives --lib` passed after changing DecisiveDamage to integer percent comparison. | Add integration coverage for combat result reward application. |
| 2026-06-27 | Bonus research reward integration test | `cargo test endless_combat_victory_adds_satisfied_bonus_objective_research --lib` passed. | Add validation/result DTO coverage and run live RON checks. |
| 2026-06-27 | Bonus objective validation bundle | `cargo test bonus_objective --lib` passed, covering RON deserialization, duplicate id rejection, zero research rejection, primary target spawn validation, evaluator behavior, defeat exclusion, and reward integration. | Verify result DTO and live RON loading. |
| 2026-06-27 | Combat result DTO surface check | `cargo test combat_result_snapshot_exposes_typed_result_stats --lib` passed after adding `selected_event.bonus_objectives` coverage. | Run live RON loading and broad checks. |
| 2026-06-27 | Live RON loading | `cargo test --test ron_loading` passed after updating integration PVE fixtures with `suppression_research: None`. Existing live encounters remain valid with no bonus objectives authored yet. | Run formatting and broad compile/test checks. |
| 2026-06-27 | Compile checks | `cargo fmt`, `cargo check --lib`, and `cargo check -p game_server` passed. | Run final broad tests. |
| 2026-06-27 | Broad lib test | `cargo test --lib` passed: 557 tests. | Phase 3 implementation is verified; keep master goal active for Phase 4. |
| 2026-06-27 | DecisiveDamage integer percent contract | Replaced float ratio authoring with `minimum_damage_percent_of_max_hp: u32`; `cargo fmt`, `cargo test pve_bonus_objectives --lib`, `cargo test bonus_objective --lib`, `cargo check --lib`, and `cargo test --test ron_loading` passed. | Keep future RON objective examples on integer percent thresholds. |

## Verification Commands

Record every focused and broad check here while implementing.

Initial planned checks:

```bash
cargo test pve --lib
cargo test bonus_objective --lib
cargo test decisive_damage --lib
cargo test combat_result --lib
cargo test ron_loading --test ron_loading
cargo check --lib
```

Update names after focused tests are added.

Executed checks:

```bash
cargo test pve_bonus_objectives --lib
cargo test endless_combat_victory_adds_satisfied_bonus_objective_research --lib
cargo test bonus_objective --lib
cargo test combat_result_snapshot_exposes_typed_result_stats --lib
cargo test --test ron_loading
cargo fmt
cargo check --lib
cargo check -p game_server
cargo test --lib
```

## Failed Attempts

### 2026-06-27: First focused bonus objective test compile exposed fixture updates

- Command: `cargo test pve_bonus_objectives --lib`
- Failure summary: compile failed because Rust test fixtures that construct `PveEncounter` and `CombatBattleState` literals need the new `suppression_research` and `bonus_objectives` fields. The new unit test also used stale `EventLogVec2 { x, y }` field names.
- Fix plan: add explicit `suppression_research: None` to existing PVE encounter fixtures, `bonus_objectives: vec![]` to existing combat result fixtures, and use current `EventLogVec2 { x_milli, y_milli }`.

### 2026-06-27: DecisiveDamage boundary failed with f32 ratio precision

- Command: `cargo test pve_bonus_objectives --lib`
- Failure summary: the exact 10% decisive hit case failed because `minimum_damage_ratio_of_max_hp: 0.10` was stored as `f32`, making `100 >= 1000 * 0.10` fail at the floating-point boundary.
- Fix: after an intermediate `f64` fix, the final contract was changed to integer percent authoring (`minimum_damage_percent_of_max_hp`) and integer comparison (`damage * 100 >= max_hp * percent`).

### 2026-06-27: ron_loading compile exposed integration fixture updates

- Command: `cargo test --test ron_loading`
- Failure summary: compile failed because `tests/common/mod.rs` constructs `PveEncounter` literals and needs explicit `suppression_research: None`.
- Fix plan: add the new optional field to the integration fixture encounters and rerun live RON loading.
