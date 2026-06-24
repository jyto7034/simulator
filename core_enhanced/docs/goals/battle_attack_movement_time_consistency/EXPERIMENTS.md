# Battle Attack Movement Time Consistency Experiments

## 2026-06-20 - Initial Trace Of Reported Battle Record

Command/inspection:

- Read `battle_records/run_563fc7221a903487/d72f2099-12fa-42e8-b689-c859e5ec9db3.json`.
- Compared early `UnitDeployed`, `MovementSegmentStarted`, `AttackStart`, projectile, and `HpChanged` events.
- Read live equipment data for:
  - `placeholder_apocalypse_bird_weapon`
  - `placeholder_big_bad_wolf_weapon`
- Read target selection and movement tick code:
  - `src/game/battle/core/sim.rs`
  - `src/game/battle/core/movement/engine.rs`
  - `src/game/battle/core/basic_attack.rs`
  - `src/game/battle/core/targeting.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/tile_range.rs`

Result:

- The visible `12 Magic` damage is sourced from unit `ffc87fc3-8ee6-463c-bcbb-d54eec512753`.
- That unit is employee `52771015-d180-459a-9591-833e68bed44a` / Choi Doyoon.
- Wire snapshot after equipment changes shows Choi Doyoon equipped:
  - `placeholder_apocalypse_bird_weapon`
  - `placeholder_apocalypse_bird_armor`
- Apocalypse Bird weapon is magic, projectile, `interval_ms: 1800`, `windup_ms: 280`, with a multi-tile forward pattern.
- Han Yujin has `placeholder_big_bad_wolf_weapon`, which is physical, instant, one-tile forward.

Conclusion:

- The first `12 Magic` is not caused by the one-tile Big Bad Wolf weapon.
- The damage source/equipment mapping is internally consistent.
- The remaining bug is timing consistency: `AttackStart` can be recorded at the start of a movement segment using a target position that only becomes presentable at the segment end.

## 2026-06-20 - Movement Tick Ordering Audit

Inspection:

- `BattleEvent::ContinuousMovementTick` runs movement first and `try_start_pending_basic_attacks(time_ms)` second.
- Movement output application immediately writes the tick-end body position to runtime units.
- The Unity-facing movement segment is recorded as `time_ms..time_ms + dt_ms`.
- Attack target selection then sees the tick-end body position while recording `AttackStart` at `time_ms`.

Result:

```text
5150ms movement segment:
  start  (0.500, 0.890) -> projected tile (0,0)
  target (0.500, 1.020) -> projected tile (0,1)

5150ms AttackStart:
  uses a target that is now in range after the movement output is applied.
```

Conclusion:

- The issue is not primarily movement tick length.
- The issue is a mismatch between sampled position time and recorded attack event time.

## Pending Experiments

- Add a focused failing unit test for a target crossing into a valid tile during one movement segment.
- Try attack selection before movement output for the same tick.
- Try battle-time-sampled target selection helpers.
- Audit skill auto-cast target selection for the same mismatch.
- Run relevant focused tests and record failures/fixes here.
