# Basic Attack Tile-Based Movement Goal Plan

## Objective

Remove the remaining basic-attack movement/stop dependency on continuous `range_units`.

The long-term combat contract is:

- Basic attack target selection is tile-based.
- Basic attack eligibility is tile-based.
- Basic-attack-driven movement/stop decisions are tile-based.
- `range_units` is not a basic attack targeting, eligibility, or stop-distance source.
- Continuous distance remains valid only for explicitly continuous mechanics such as projectile collision, contact, aura, contagion, and other future purpose-specific systems.

This goal must not add compatibility fallbacks or preserve the old stop-at-continuous-distance behavior behind a second path.

## Source Of Truth Order

Use sources in this order:

1. Runtime combat code:
   - `src/game/battle/core/movement/planner.rs`
   - `src/game/battle/core/movement/steering.rs`
   - `src/game/battle/core/movement/engine.rs`
   - `src/game/battle/core/basic_attack.rs`
   - `src/game/battle/core/targeting.rs`
   - `src/game/battle/core/movement/blocking.rs`
2. Live RON/data:
   - weapon/equipment basic attack RON
   - abnormality basic attack RON
   - corroded employee profile/range preset RON
3. Canonical docs:
   - `docs/game_rulebook.md`
   - `docs/skill_target_contract.md`
4. Unity-facing transport docs only if DTO/event shape changes.

Do not trust any older goal document over live runtime code and current canonical docs.

## Initial Audit Findings

The final basic attack eligibility path was already tile-based before this goal:

- `choose_attack_target_in_range()` and `is_basic_attack_target_in_range()` use the unit's basic attack range policy and tile range.
- Tests already cover that large `range_units` does not make a target attackable when the target is outside tile range.

The mismatch identified at goal start was movement planning:

- `movement/planner.rs` created `MovementGoal::AttackUnit` with `desired_range: unit.basic_attack.range_units.max(0.0)`.
- `closest_enemy_in_attack_range()` found candidates with `unit.body.can_reach(candidate.body, desired_range)`.
- `movement/steering.rs` and `movement/engine.rs` interpreted `desired_range` as a continuous stop/approach distance.

This meant an actor could still decide to stop or approach based on continuous basic attack distance, even though the actual attack eligibility was tile-based.

## Implementation Status

The repair keeps `range_units` available for explicit continuous mechanics, but removes it from basic-attack movement/stop semantics:

- `MovementGoal::AttackUnit` now carries only `target_id`.
- Basic-attack movement acquisition uses tile-range target APIs.
- `closest_enemy_in_attack_range()` is no longer a basic-attack movement source.
- `MovementUnitInput.attack_range_units` was removed.
- Route-less boss/special-unit behavior was not generalized in this goal.

## Final Policy

### Basic Attack Range

Basic attack range is a set of logical tiles.

For basic attacks, runtime must not use `range_units` to decide:

- whether a unit can attack a target;
- which target is in range;
- whether a unit should stop moving to attack;
- where a unit should stand for basic attack purposes.

### Blocked Enemies

Blocked enemies use blocker-first behavior.

If an enemy is `blocked` by a unit, that enemy's basic attack target is the blocking unit before normal target selection runs.

This does not mean continuous range gets priority over tile range. It means blocked combat is represented as same-logical-tile engagement. Once the blocker dies, withdraws, or block state is released, the enemy returns to normal tile range targeting and route movement.

### Melee Route Enemies

Melee route enemies do not stop just because a continuous distance threshold is satisfied.

Expected behavior:

- If blocked, attack the blocker.
- If not blocked, follow the route.
- If a target is not in tile attack range, do not create a basic-attack stop goal for it.

### Ranged Route Enemies

Ranged route enemies follow the route until their tile attack range contains a valid target.

Expected behavior:

- If blocked, attack the blocker.
- If currently in the configured ranged reposition window and not blocked, continue route movement.
- Otherwise, if a valid target is in tile attack range, stop and attack.
- If no valid target is in tile attack range, continue route movement.
- `ranged_reposition_ms` remains data-driven and continues to control the short post-attack movement window.

### Player Fixed-Defense Units

Player units in fixed-defense tactical plans are normally placed and stationary. Their target selection and attack eligibility remain tile-based.

Do not introduce a continuous-distance approach rule for player basic attacks.

### Airborne Units

Airborne units may ignore blocking for movement where the existing movement policy allows it, but their basic attack target selection and attack eligibility remain tile-based.

Airborne status does not permit continuous-distance basic attack range.

### Route-less Bosses And Special Units

Route-less units are explicitly out of scope for this goal.

This includes bosses, stationary boss actors, special corroded employees, sniper-like special enemies, summoned special actors, and any unit whose behavior is driven by boss/special mechanics rather than `PathAlongCells`.

Do not invent a shared route-less basic attack acquisition fallback in this goal. Bosses and special route-less units may need unique phase, skill, whole-field, summon, aura, or scripted behavior. Those policies must be decided in boss/special-unit-specific work.

If implementation discovers that an existing route-less unit currently depends on continuous-distance basic attack acquisition, stop and report it instead of preserving the old behavior silently.

### `range_units`

Do not delete `range_units` globally in this goal.

Allowed future uses:

- projectile collision and sweep mechanics;
- contact effects;
- aura effects;
- contagion mechanics;
- explicitly continuous skills or environment effects.

Disallowed in this goal's final state:

- basic attack target selection;
- basic attack eligibility;
- basic attack movement stop distance;
- generic "closest enemy in range" acquisition for basic attacks.

## Non-Goals

- Do not redesign tile range authoring.
- Do not delete weapon or abnormality `range_units` fields globally.
- Do not change Unity-facing DTO shape unless runtime inspection proves it is necessary.
- Do not change projectile impact/collision policy.
- Do not change skill targeting except if a shared helper accidentally couples skill targeting to basic attack movement and the issue cannot be isolated.
- Do not define route-less boss/special-unit basic attack behavior in this goal.
- Do not change enemy wave, encounter, boss omen, or map generation policies.
- Do not add a compatibility layer that silently falls back from tile range to continuous distance.

## Implementation Plan

### Phase 1: Runtime Inventory

- Inventory every `MovementGoal::AttackUnit` construction.
- Inventory every read of `unit.basic_attack.range_units` in movement code.
- Separate:
  - basic attack movement/stop behavior;
  - physical collision/steering internals;
  - test fixtures;
  - future continuous-mechanic helpers.
- Record findings in `EXPERIMENTS.md`.

### Phase 2: Planner Contract Repair

- Replace basic-attack movement acquisition with tile-range-aware target acquisition.
- Remove `closest_enemy_in_attack_range()` as a basic-attack movement source, or rename/scope it so it is not used as basic attack range logic.
- Ensure fixed-defense enemy route units use:
  - blocked target first;
  - `choose_attack_target_in_range()` for tile range target acquisition;
  - route movement if no tile-range target exists.
- Ensure non-fixed-defense route-following paths do not accidentally preserve continuous basic attack acquisition.
- If a route-less combat archetype appears in the affected path, stop and ask the user instead of inventing or preserving a fallback.

### Phase 3: Movement Goal Semantics

- Adjust `MovementGoal::AttackUnit` or its call sites so basic-attack stop behavior no longer depends on `desired_range`.
- Prefer a domain-specific representation if needed, such as:
  - `AttackUnit { target_id }` for same-tile/blocker or tile-range attack intent;
  - `MoveToPoint` for route movement;
  - a clearly named non-basic-attack continuous goal only for future/explicit continuous mechanics.
- Do not leave `desired_range` populated from `basic_attack.range_units` for basic attack goals.
- If steering still needs a physical separation value to avoid overlapping presentation/physics bodies, use a collision/physics constant or body radius policy, not basic attack range.

### Phase 4: Tests

Add focused behavior tests before or with the repair:

- A unit with huge `range_units` and a target outside tile range must continue route movement instead of creating an attack stop goal.
- A ranged route enemy with a target inside tile range creates an attack goal.
- A ranged route enemy with no target inside tile range continues route movement.
- A melee route enemy not blocked continues route movement even if continuous distance is small.
- A blocked enemy attacks its blocker first.
- A blocker released/dead/withdrawn causes the enemy to resume normal tile-range targeting or route movement.
- An airborne unit still needs tile range to basic attack.
- Existing tests proving basic attack eligibility ignores `range_units` continue to pass.

Prefer user-visible behavior assertions:

- generated movement goal kind;
- target id chosen;
- absence of attack goal when outside tile range;
- event/timeline behavior if the unit is advanced far enough to attack.

### Phase 5: Docs And Verification

- Update `docs/game_rulebook.md` and `docs/skill_target_contract.md` if implementation clarifies wording around basic-attack movement/stop behavior.
- Record whether Unity-facing DTO shape changed. Expected answer: no DTO shape change.
- Run focused tests after each meaningful edit.
- Run final broad validation:
  - focused movement/targeting tests;
  - focused basic attack tests;
  - RON loading tests if any data validation changed;
  - `cargo check -p game_core` or broader repository command as appropriate.

## Completion Criteria

- No basic-attack movement/stop decision reads `basic_attack.range_units`.
- Basic attack target acquisition used by movement planning is tile-based.
- `closest_enemy_in_attack_range()` is removed from basic-attack movement flow or renamed/scoped away from basic attack semantics.
- Route-less boss/special-unit basic attack behavior is not changed or generalized by this goal.
- Blocked enemies attack their blocker first.
- Non-blocked melee route enemies continue route movement unless tile-based policy says they can attack.
- Ranged route enemies stop only when a valid target is inside tile attack range.
- `ranged_reposition_ms` remains data-driven and still works.
- Airborne units do not gain continuous-distance basic attack range.
- Focused tests fail on the old `range_units` stop behavior and pass after the repair.
- Canonical docs match runtime behavior.
- No Unity-facing DTO shape changes are introduced unless explicitly justified.
- Final report includes:
  - changed movement/attack behavior;
  - removed legacy/basic-attack `range_units` usages;
  - tests added/updated;
  - docs updated;
  - remaining risks;
  - exact verification commands.

## Stop Conditions

Stop and ask the user if any of these appear:

- A route-less boss/special-unit archetype is affected by the basic-attack movement repair.
- Removing `desired_range` from basic attack goals requires a broad physics/steering redesign.
- A basic attack movement repair would require Unity-facing DTO/event shape changes.
- Live RON relies on `range_units` as the only expression of a unit's basic attack tile range.
- A required policy conflicts with blocker-first behavior.
- The repair would change projectile collision, aura, contact, or contagion mechanics outside this goal.
