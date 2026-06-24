# Battle Attack Movement Time Consistency Plan

## Objective

Make live battle attack timing consistent with movement presentation time.

At battle time `t`, core attack eligibility must be based on the same unit positions that Unity can present at battle time `t`. A unit must not start an attack at the start of a movement segment using the target position from the end of that segment.

This goal keeps the movement tick cadence at its current policy unless later evidence proves it must change. The primary fix is time/position consistency, not smaller ticks.

## Source Of Truth Order

Use this order while implementing:

1. Runtime battle code and exported battle records.
2. Live RON/data that creates employee equipment, range patterns, enemies, routes, and waves.
3. Unity-facing battle setup/update DTOs and battle record JSON.
4. Current policy documents.

Do not trust older documentation over runtime behavior. If runtime code reveals a better long-term design, record the evidence in `EXPERIMENT_NOTES.md` before changing the plan. Stop and report if the better design requires a user policy decision.

## Current Evidence

The user reported a visible early attack in:

- `battle_records/run_563fc7221a903487/d72f2099-12fa-42e8-b689-c859e5ec9db3.json`

Relevant timeline:

```text
5100ms  UnitDeployed
        employee 52771015-d180-459a-9591-833e68bed44a
        unit ffc87fc3-8ee6-463c-bcbb-d54eec512753
        position (0,4), facing up

5150ms  MovementSegmentStarted
        enemy a1fbb727-5065-44b3-b248-2cd9e88a484a
        (0.500,0.890) -> (0.500,1.020)
        started_at_ms 5150, ends_at_ms 5200

5150ms  AttackStart
        ffc87fc3-8ee6-463c-bcbb-d54eec512753 -> enemy
        delivery projectile

5430ms  BasicAttackProjectileLaunched
        start (0.500,4.500), aim (0.500,1.670)

5696ms  HpChanged
        source ffc87fc3-8ee6-463c-bcbb-d54eec512753
        target enemy
        -12 Magic
```

The `12 Magic` attack is not from the one-tile Big Bad Wolf weapon. It is from Choi Doyoon with Apocalypse Bird weapon:

- `placeholder_apocalypse_bird_weapon`
- `damage_type: Magic`
- `delivery: Projectile`
- `interval_ms: 1800`
- `windup_ms: 280`
- `defense_tile_range.rows: ["..X..", ".XXX.", "..X..", "..@..", "....."]`

That weapon can legally hit tile `(0,1)` from `(0,4)` while facing up. The problem is that the attack starts at `5150ms`, while Unity is still expected to present the movement segment from `(0.500,0.890)` toward `(0.500,1.020)` over `5150..5200ms`.

## Current Code Findings

Relevant code:

- `src/game/battle/core/sim.rs`
- `src/game/battle/core/movement/engine.rs`
- `src/game/battle/core/basic_attack.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/tile_range.rs`

Current behavior:

- `BattleEvent::ContinuousMovementTick` runs:
  1. `run_continuous_attack_movement_tick(time_ms, DEFAULT_MOVEMENT_TICK_MS)`
  2. `try_start_pending_basic_attacks(time_ms)`
  3. schedules the next movement tick.
- `run_continuous_movement_tick()` applies `MovementOutput::BodyMoved` immediately to the runtime unit body.
- `record_continuous_movement_segment(time_ms, dt_ms, ...)` records a Unity-facing segment with:
  - `started_at_ms = time_ms`
  - `ends_at_ms = time_ms + dt_ms`
  - `start = from`
  - `target = to`
- `try_start_pending_basic_attacks(time_ms)` then uses the already-updated runtime body positions for target selection.
- `is_basic_attack_target_in_range()` correctly uses tile range for fixed-defense player units. The bug is not a fallback to `range_units`.
- `sample_unit_world_position_at(unit_id, time_ms)` and `sample_unit_body_at(unit_id, time_ms)` exist and can sample an active movement segment at a specific battle time.
- Projectile advance/impact logic already uses time-aware sampling, but basic projectile launch and pre-launch range checks still need to be audited because some paths read current runtime bodies directly.

This means the attack decision can use tick-end positions while `AttackStart.time_ms` is still the tick-start time.

## Target Policy

At any battle time `t`:

- Attack target eligibility must use positions sampled at `t`.
- `AttackStart.time_ms` must correspond to the position sample used for the eligibility decision.
- Basic attack resolve/release checks and projectile launch aim/origin must also use positions sampled at their own event time, not a later tick-end body position.
- Automatic skill target eligibility must follow the same time/position policy as basic attacks.
- Movement segment presentation remains authoritative for Unity interpolation.
- Unity must not be required to infer that an `AttackStart` at `t` actually used the movement segment's `ends_at_ms` position.
- DefenseRoute player basic attack and skill target eligibility remain tile-based, not continuous `range_units` based.
- `range_units` remains data for non-fixed-defense movement/target policies where applicable, not the DefenseRoute player attack source.
- Do not solve this by reducing `DEFAULT_MOVEMENT_TICK_MS` unless tests prove time-consistent sampling is insufficient.

## Recommended Design

Prefer sampling positions at the event time used by the attack decision.

For `ContinuousMovementTick { time_ms }`, the most direct long-term fix is:

1. Preserve a view of positions at `time_ms` before movement outputs are applied.
2. Use that time-consistent view for attack target selection at `time_ms`.
3. Apply movement outputs and record `MovementSegmentStarted` for `time_ms..time_ms + dt_ms`.
4. If the target only becomes eligible at the end of the segment, the attack should start on the next eligible battle time, not at the segment start.

Possible implementation approaches:

- Add time-aware target/range helpers that sample bodies through `sample_unit_body_at(unit_id, time_ms)`.
- Or adjust movement tick ordering so pending attacks for `time_ms` are evaluated before movement outputs for `time_ms..time_ms + dt_ms`.
- Or model the movement output as becoming authoritative only at `ends_at_ms` for subsequent attack decisions.

After code review, prefer explicit time-aware helpers unless implementation discovers a cleaner equivalent. Current `AttackStart` handling can retarget when the event is processed, skill target selection has separate paths, and projectile launch has its own aim/origin reads. A helper-based design is more likely to keep all of those paths consistent than a local reorder in `ContinuousMovementTick` alone.

The implementation should choose the smallest coherent domain design after reading the code. Avoid one-off checks only around the reported Apocalypse Bird case.

## Implementation Plan

1. Reproduce the reported timing mismatch with a focused test.
   - Build a fixed-defense battle core fixture.
   - Player unit at `(0,4)`, facing up.
   - Use a tile range that includes `(0,1)` but not `(0,0)`.
   - Enemy movement segment crosses from tile `(0,0)` to tile `(0,1)` during one movement tick.
   - Assert no `AttackStart` is emitted at the segment start if the target was not in range at that battle time.
   - Assert `AttackStart` can occur once the battle time reaches the range-valid position.

2. Add a regression around the real weapon shape.
   - Use a profile equivalent to `placeholder_apocalypse_bird_weapon`.
   - Assert `(0,4)` facing up can hit `(0,1)`.
   - Assert it cannot start at `5150ms` when the target position sampled at `5150ms` is still `(0,0)`.

3. Audit current attack scheduling.
   - `try_start_pending_basic_attacks()`
   - `handle_basic_attack_start_event()`
   - `select_basic_attack_target()`
   - `persisted_target_in_range()`
   - `choose_attack_target_in_range()`
   - `resolve_basic_attack()`
   - Basic projectile launch origin/aim creation.
   - Skill auto-cast target selection and manual/explicit target validation, because the same time/position mismatch exists anywhere target eligibility reads current runtime body positions.

4. Refactor target range checks to support battle-time sampling.
   - Add explicit helper names if useful, for example:
     - `is_basic_attack_target_in_range_at(attacker, target, time_ms)`
     - `is_target_in_defense_tile_range_at(attacker, target, time_ms, pattern)`
     - `select_basic_attack_target_at(attacker, current_target, hinted_target, time_ms)`
     - `choose_skill_target_by_rule_at(..., time_ms)`
   - Use sampled bodies at `time_ms` for both attacker and target.
   - Use the same sampled bodies for range checks, distance tie-breaks, splash-cluster scoring, and nearest-target sorting.
   - Keep current non-time-aware helpers only if they remain valid for non-presentation contexts; otherwise replace call sites.

5. Keep timeline semantics clear.
   - `AttackStart` remains windup start.
   - `AttackResolve` remains windup completion.
   - Projectile launch/impact remain their own timed presentation events.
   - Do not add `AttackWindUp` unless a separate policy decision is made; the current `AttackStart -> AttackResolve` interval already represents windup.

6. Validate command and live behavior.
   - Confirm fixed-defense player basic attack still uses final tile range cells rather than `range_units`.
   - Confirm non-fixed-defense attackers still use their intended policy.
   - Confirm basic projectile launch aim/origin and projectile impact continue sampling moving targets at their event times.
   - Confirm automatic skill target acquisition and retarget-on-step paths do not use tick-end positions for earlier timestamps.
   - Confirm battle timeline seq ordering stays monotonic.

7. Update documentation if behavior or contract wording changes.
   - `docs/game_rulebook.md`: attack timing/range source policy if not already clear.
   - `docs/skill_target_contract.md`: time-consistent DefenseRoute target eligibility if needed.
   - External Unity battle transport docs only if the Unity-facing meaning of `AttackStart`, `MovementSegmentStarted`, or checkpoint reconcile changes.

8. Verify.
   - Run focused battle core tests after each meaningful change.
   - Run the existing movement, targeting, and battle record tests.
   - Finish with `cargo check -p game_core` or the current package's equivalent, plus a wider relevant test set.

## Test Plan

Focused tests:

- `fixed_defense_attack_does_not_use_tick_end_position_at_segment_start`
  - Enemy crosses into range during a movement segment.
  - No `AttackStart` at segment start.

- `fixed_defense_attack_starts_when_sampled_time_is_in_tile_range`
  - Same setup.
  - Attack starts no earlier than the first battle tick whose sampled battle time projects the enemy into the valid tile.

- `fixed_defense_player_basic_attack_still_uses_tile_range_not_range_units`
  - Preserve existing policy and strengthen with a moving target case.

- `apocalypse_bird_weapon_range_timing_regression`
  - Use the real range pattern from live data or a matching fixture.
  - Prove the reported early-start case is impossible after the fix.

- `projectile_aim_samples_target_at_launch_time`
  - Ensure basic projectile launch uses attacker/target positions sampled at launch time.
  - Ensure existing projectile advance/impact behavior remains time-aware.

- `skill_cast_target_selection_matches_sampled_time`
  - Auto-cast target selection and retarget-on-step target selection must not use tick-end positions for earlier event timestamps.

- `manual_skill_target_validation_matches_sampled_time`
  - Manual or explicit unit target validation should use the command/event battle time if it checks moving target range.

Live/behavior tests:

- A DefenseRoute scenario with a deployed ranged player unit and a route enemy crossing into range.
- Assert the first `AttackStart.time_ms` is not earlier than the target's range-entry presentation time.
- Assert the exported battle record can be replayed without the attack appearing to start before range entry.

## Non-Goals

- Do not reduce movement tick duration as the primary fix.
- Do not tune Apocalypse Bird or Big Bad Wolf weapon ranges to hide the issue.
- Do not replace tile-based DefenseRoute range with continuous distance.
- Do not add Unity-side delay hacks for attack VFX.
- Do not add compatibility paths or dual semantics for old attack timing.
- Do not redesign all movement physics or route generation.
- Do not introduce a new `AttackWindUp` event unless the user explicitly approves a timeline event contract change.

## Completion Conditions

The goal is complete when:

- Runtime attack eligibility at battle time `t` uses positions sampled at battle time `t`.
- Basic attack start, resolve/release validation, and projectile launch reads are time-consistent with their own event timestamps.
- The reported early `AttackStart` pattern is covered by a failing-then-passing test.
- DefenseRoute player attack range remains tile-based.
- Skill target selection and explicit skill target validation are either fixed under the same policy or explicitly proven unaffected in `EXPERIMENTS.md`.
- Exported battle records remain valid and monotonically sequenced.
- Relevant docs are updated if contract wording changed.
- Focused tests and a wider relevant `cargo` validation pass.

## Stop Conditions

Stop and ask the user before proceeding if implementation discovers:

- A clean fix requires changing Unity-facing event names or adding a new attack windup event.
- Attack timing must intentionally use tick-end positions for design reasons.
- Skill target timing needs a separate user-facing policy decision.
- Reducing movement tick duration is still necessary after time-consistent sampling.
- A compatibility layer is required for existing Unity consumers.
- Existing live RON data depends on continuous `range_units` for DefenseRoute player attacks.
