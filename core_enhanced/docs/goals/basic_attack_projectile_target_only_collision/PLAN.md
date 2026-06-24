# Basic Attack Projectile Target-Only Collision Plan

## Objective

Change basic attack projectile impact from fixed-time automatic hit to target-only continuous collision, while preserving tile-based attack eligibility.

The intended long-term policy is:

- Basic attack start, target selection, and range eligibility remain tile-based.
- A projectile basic attack locks onto the target selected by tile eligibility at attack release.
- The launched projectile may only collide with that original target unit.
- The projectile must not collide with bystanders, blockers, or any other unit along the path.
- Damage is applied when the projectile sweep reaches the original target body.
- If the original target dies, disappears, becomes friendly, or fails non-range targetability before impact, the projectile misses.
- Projectile impact must not re-check tile range, target usefulness, or basic attack range at arrival.
- This is not a directional skillshot. Directional/path collision against arbitrary units remains a skill-only `DirectionalCollision` policy.

This goal is a refinement of `docs/goals/tile_based_attack_delivery_contract/PLAN.md`, not a reversal of it. The tile contract still owns whether an attack may be performed. This goal only changes how a projectile basic attack reaches the already chosen target.

## Source Of Truth Order

Use this order while implementing:

1. Runtime battle code:
   - `src/game/battle/core/commands.rs`
   - `src/game/battle/core/types.rs`
   - `src/game/battle/core/spatial.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/targeting.rs`
   - `src/game/battle/core/basic_attack.rs`
   - `src/game/battle/core/skill_runtime/projectile.rs`
2. Runtime timeline events and battle records:
   - `src/game/battle/timeline.rs`
   - `src/game/battle/enums.rs`
   - `debug_event_log_exports/`
   - `battle_records/`
3. Live RON/data:
   - weapon/equipment basic attack delivery data.
   - abnormality/corroded employee basic attack data.
4. Unity-facing DTO and contract docs:
   - `docs/skill_target_contract.md`
   - `docs/game_rulebook.md`
   - battle transport docs, if timeline meaning changes.

Do not trust older docs over runtime code and live data. If runtime code reveals a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` before applying it. If the better model changes gameplay policy, stop and ask the user.

## Current Evidence

Current code already has most required pieces:

- `BattleCore::spawn_basic_attack_projectile()` creates a `ProjectileRecord` with:
  - `attacker_instance_id`
  - `target_instance_id`
  - `start`
  - `current_position`
  - `aim`
  - `speed_units_per_ms`
  - `guidance`
  - `damage_type`
- `BattleCore::advance_basic_attack_projectile()` currently removes the projectile at the scheduled impact time and applies damage immediately if the original target is still valid.
- `sample_unit_body_at(unit_id, time_ms)` already samples a unit body at a battle time, including active movement segment interpolation.
- `moving_circle_sweep_hit_fraction()` already computes continuous sweep contact between projectile motion and target body motion.
- Skill homing projectile runtime already demonstrates target-only sweep against one target body.
- Timeline events already exist:
  - `BasicAttackProjectileLaunched`
  - `BasicAttackProjectileImpacted`
  - `ProjectileMiss`
- No Unity-facing DTO shape change appears necessary for the core gameplay change.

Current issue:

- A basic attack projectile records a launch and then auto-hits at `expected_impact_time_ms`.
- If the target moved between launch and impact, the gameplay hit is not based on projectile/body contact.
- This can make hit timing and impact position feel detached from the actual target position, even though target selection was correctly tile-based.

## Target Policy

### Eligibility

Do not change eligibility policy in this goal.

Basic attack eligibility remains:

- target selected by current tile-based targeting/range rules,
- target alive and hostile at attack release,
- target satisfies air/ground targetability,
- target is useful according to existing hostile target usefulness rules,
- target tile is inside the attacker's final basic attack tile range.

No continuous distance, hitbox radius, projectile path, or collision result may decide whether an attack can start.

### Projectile Launch

When a projectile basic attack is released:

- store the original `target_instance_id`,
- store `attacker_owner_at_launch` or an equivalent immutable launch-side snapshot,
- record `BasicAttackProjectileLaunched`,
- schedule projectile advancement from the launch time.

The launch event may keep `expected_impact_time_ms`, but that field becomes an initial presentation estimate rather than a guaranteed gameplay hit time. Gameplay impact is determined by target-only sweep.

### Projectile Advancement

Projectile advancement must:

- sample the projectile position over a time window,
- sample only the original target unit body over the same time window,
- sweep projectile motion against that target body,
- apply hit at the first contact time,
- schedule another advancement if no contact occurred and the projectile is still valid.

The advancement logic must not inspect or collide with any other unit.

### Impact Validity

At every advance window, before applying hit:

- hostility must be checked against `attacker_owner_at_launch`, not against a mutable or missing live attacker record,
- original target must exist,
- target must be alive,
- target must remain hostile to `attacker_owner_at_launch`,
- target must still satisfy non-range targetability such as air capability / untargetable state.

The projectile may continue after the attacker dies because the launch already happened. Damage calculation may still use existing live-attacker or graveyard attacker stats, but target hostility must not become unknowable just because the attacker unit left `units`.

Do not re-check:

- basic attack tile range,
- `range_units`,
- target usefulness,
- nearest/priority target selection,
- bystander collision.

### Miss / Expiration

The projectile misses when:

- original target is gone,
- original target is dead before contact,
- original target becomes friendly or otherwise invalid,
- projectile exceeds its maximum allowed travel/lifetime without contact.

The implementation must define the maximum travel/lifetime using existing launch data first. If current data cannot provide a safe maximum, stop and ask before inventing a new gameplay value.

Preferred implementation policy to validate from code before changing gameplay:

- first advance window is the launch-time estimated travel window,
- if the moving locked target was not contacted, continue homing in bounded reevaluation windows,
- expiration must be deterministic and finite,
- do not rely on "eventually reaches target" as an implicit lifetime.

### Event Cadence

The implementation must choose an internal reevaluation cadence that is accurate enough for target-only collision without exploding Unity-facing gameplay events.

Candidate policies to evaluate during implementation:

- reuse the previous basic projectile reevaluation tick if it is still recoverable from history/tests,
- reuse skill projectile reevaluation cadence if acceptable,
- introduce a basic projectile reevaluation constant with tests.

Record the chosen cadence and reason in `EXPERIMENTS.md`.

## Implementation Plan

1. Audit current basic projectile runtime.
   - Confirm all call sites for `ProjectileRecord`, `spawn_basic_attack_projectile()`, and `advance_basic_attack_projectile()`.
   - Confirm current tests that assume fixed-time auto-hit.
   - Confirm whether debug battle records need regeneration.
   - Record findings in `EXPERIMENTS.md`.

2. Define projectile runtime state.
   - Keep `target_instance_id` as the only collision target.
   - Ensure `ProjectileRecord` has enough state for repeated advancement.
   - Add `attacker_owner_at_launch` or an equivalent immutable owner snapshot so target hostility can be validated even if the attacker dies before impact.
   - Add `max_travel_ms`, `expires_at_ms`, or equivalent only if existing data is insufficient.
   - Prefer using launch distance / speed as the first expected impact window, then continue only when needed by target movement.

3. Implement target-only continuous sweep.
   - Reuse `sample_unit_body_at()` for target body interpolation.
   - Reuse `moving_circle_sweep_hit_fraction()` for the projectile-target sweep.
   - Reuse the skill projectile hit-fraction position helper or introduce a shared helper if duplication grows.
   - Do not enumerate all units.

4. Preserve target-locked validity.
   - Keep impact-time validity checks narrow:
     - target exists,
     - target alive,
     - target hostile to `attacker_owner_at_launch`,
     - target satisfies non-range targetability.
   - Do not re-check range or usefulness.

5. Update timeline behavior only as needed.
   - Preserve current event names unless a real DTO gap is found.
   - `BasicAttackProjectileImpacted.hit = true` should mean the projectile contacted the locked target.
   - `BasicAttackProjectileImpacted.hit = false` should mean the locked target was invalid or the projectile expired.
   - Document `BasicAttackProjectileLaunched.expected_impact_time_ms` as an initial presentation estimate, not the authoritative impact time.

6. Update tests.
   - Replace stale "auto-hit at expected impact time" tests with target-only collision tests.
   - Add a test where a bystander crosses the projectile path and is not hit.
   - Add a test where the original target moves and the projectile hits only when the target body is swept.
   - Add a test where the original target dies before contact and the projectile misses.
   - Add a test proving tile range is not rechecked at projectile arrival.
   - Keep existing tile eligibility tests from `tile_based_attack_delivery_contract` intact.

7. Update docs.
   - Update `docs/goals/tile_based_attack_delivery_contract/PLAN.md` so its completed projectile section points to this target-only continuous-collision refinement instead of saying target-locked arrival never sweeps the locked target.
   - Update `docs/skill_target_contract.md` if it currently says projectile basic attacks auto-hit at fixed arrival.
   - Update `docs/game_rulebook.md` if it describes projectile basic attack impact timing.
   - Add implementation notes to this goal's `EXPERIMENTS.md` and policy notes to `EXPERIMENT_NOTES.md`.

8. Validate incrementally.
   - Run focused tests after each small runtime change.
   - Run `cargo check -p game_core`.
   - Run `cargo test -p game_core --test skill_refactor_validation` if skill/basic projectile fixtures are touched.
   - Run the relevant `commands` / battle core unit tests.
   - Finish with `cargo test -p game_core`.

## Stop Conditions

Stop and ask the user if any of these appear:

- Implementing target-only collision requires changing basic attack tile eligibility.
- Runtime cannot provide a safe projectile expiration policy from existing data.
- Runtime cannot preserve original attacker side without changing a public DTO or inventing a compatibility path.
- Live data requires projectile basic attacks to hit bystanders or path targets.
- Unity-facing DTO shape must change rather than only event timing/semantics.
- Existing battle records or tests encode a conflicting design that seems intentional.
- The implementation would need a compatibility layer or dual projectile schema.
- A target-only collision projectile can loop forever without an obvious authored maximum.

## Completion Conditions

This goal is complete when:

- Basic attack target selection and attack release remain tile-based.
- Projectile basic attacks sweep only against the original target unit.
- Projectile basic attacks never hit bystanders.
- Projectile basic attacks hit when the projectile contacts the original target body.
- Projectile basic attacks miss when the original target becomes invalid before contact.
- Projectile basic attacks can validate hostility from immutable launch-side data even if the attacker dies before impact.
- Projectile basic attacks have a deterministic finite expiration policy.
- Projectile arrival does not re-check tile range, `range_units`, usefulness, or retargeting.
- Projectile timeline events remain coherent for Unity presentation.
- Relevant docs describe target-only continuous collision accurately.
- Goal `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` are updated with implementation decisions and validation results.
- Focused tests and final broad tests pass.

## Final Report Requirements

On completion, report:

- changed runtime files,
- changed docs,
- removed legacy behavior,
- newly fixed projectile impact contract,
- tests added or updated,
- remaining risks,
- exact validation commands run.
