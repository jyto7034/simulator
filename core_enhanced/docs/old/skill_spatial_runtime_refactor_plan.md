# Skill Spatial Runtime Refactor Plan

## Goal

Replace the current "step directly schedules projectile hit against one target"
runtime with a proper spatial delivery layer that can express:

- targeted homing skill projectile
- untargeted fixed / skillshot projectile
- instant area delivery
- persistent area delivery
- chained step effects based on projectile/area impact context

This refactor does **not** change the basic attack rule:

- basic attacks remain homing
- if fired at a living target, they hit

The new spatial runtime applies only to **skill** delivery.

## Related Focused Doc

If the next step is to replace launch-time projectile prediction with a
reactive projectile runtime, use:

- `docs/active_projectile_runtime_plan.md`

That document is the focused source of truth for:

- active projectile runtime
- reevaluation triggers
- determinism rules
- migration phases

## Current Progress

- orchestrator / movement continuous layer는 별도 phase에서 완료됨
- `untargeted fixed projectile -> first hit / miss -> SkillProjectileImpact`
  vertical slice는 구현 완료
- `instant area`
  - `Circle`
  - `Line`
  - `Box`
  - `Rectangle`
  - `Cone`
  - `AreaAnchorSource`
  - `include_caster`
  기반 overlap slice는 구현 완료
- `persistent area`
  - immediate first tick
  - repeated `SkillAreaTick`
  - `SkillAreaExpire`
  기반 lifecycle slice는 구현 완료
- `persistent area tick policy`
  - `EveryTick`
  - `OncePerArea`
  - `OnEnter`
  behavioral slice도 구현 완료
- `targeted homing skill projectile`
  - `SkillProjectileImpact`
  - `SkillImpactContext`
  기준 runtime path 통일도 구현 완료
- legacy `ProjectilePayload::SkillStep` 기반 skill projectile path 제거도 완료
- `piercing / max_hits`
  - explicit `piercing` schema
  - multi-impact fixed projectile runtime
  - `max_hits` cap
  behavioral slice도 구현 완료
- `despawn_on_hit`
  - compatibility-only optional field로 축소
  - runtime은 `piercing`을 authoritative collision contract로 사용
- live `.ron` migration
  - `queen_of_hatred_magical_beam` -> `Area(Line, CastTarget)`
  - `melting_love_slime_infection.slime_spread` -> `Area(Box, ImpactContext)`
  - `plague_mass_heal.mass_heal` -> `Area(Box, CastTarget, include_caster=true)`
  - `fragment_universe_nova.nova` -> `Area(Box, CastTarget)`
  - `fairy_festival_blessing.fairy_bless` -> `Area(Box, CastTarget, include_caster=true)`
  - `big_bird_dark_lamp.(lamp_gaze, lamp_burst)` -> `Area(Box, CastTarget)`
  - `mountain_mass_consumption.consume_burst` -> `Area(Box, CastTarget)`
  - `white_night_pale_benediction.(ally_salvation, ally_blessing)` -> `Area(Box, CastTarget, include_caster=true)`
  - `white_night_pale_benediction.enemy_judgement` -> `Area(Box, CastTarget)`
  representative migration slice는 이미 여러 live skills로 확대됨
- current active migration target is:
  - migrate more live `.ron` skills onto `Projectile/Area` spatial delivery where it improves clarity
  - optional line/pivot orientation expressiveness refinement beyond `Rectangle/Cone`
- remaining `Instant` steps are now mostly intentional
  single-target direct hit / self buff / extra attack steps,
  so the recommended next step is content audit + selective migration,
  not blanket conversion of every `Instant` step
- structural cleanup also started:
  - `sim.rs` no longer needs to own all skill target / area helper bodies
  - `skill_runtime/cast.rs`, `skill_runtime/area.rs`, `skill_runtime/projectile.rs`
    now host the split helpers
  - next structural split candidate is the step effect application bridge
- active projectile runtime 2차 계획도 시작됨:
  - `ActiveProjectileRuntime` state added
  - `BattleEvent::SkillProjectileAdvance` added
  - `untargeted fixed projectile` active runtime migration 완료
  - same-time settled board state ordering 완료
  - regression:
    blocker / original target death / post-launch hard CC /
    pierce / max_hits
    기준으로 보강 완료
  - `targeted homing skill projectile`는
    현재 launch-time scheduled targeted delivery로 유지
    (active runtime 전환은 보류)

## Current Status Summary

현재 이 문서를 한 줄로 읽으면:

- projectile / area spatial runtime의 주된 behavioral migration은 이미 끝났다
- 남은 본선은
  더 많은 live `.ron` skill migration,
  필요 시 area shape/orientation refinement,
  그리고 step effect bridge 구조 정리다

즉 다음 AI가 다시 해야 할 일은
projectile 모델을 더 확장하는 것이 아니라,
이미 만들어 둔 spatial runtime 위에 실제 skill 데이터를 더 올리는 것이다.

## Why The Current Runtime Is Not Enough

Current runtime shape:

- `BattleEvent::SkillStep`
  - resolves one `step_target`
  - if `delivery == Projectile`, schedules one `ProjectileHit`
- `ProjectilePayload::SkillStep`
  - carries `cast_seq`, `step_index`, `skill_id`, `step_id`, `step_target`
- `apply_projectile_hit()`
  - re-resolves the step and applies effects at impact

Current limitations:

1. Delivery and effect are conflated
- the projectile both decides contact and implicitly decides the effect target set

2. Untargeted projectile is under-modeled
- current payload still centers around one `target_instance_id`
- a true skillshot should collide with the first valid unit on its path,
  not only a preselected unit target

3. Area delivery is not a first-class runtime concept
- current area logic is just "resolve targets now from tile area"
- no explicit instant blast runtime or persistent ground zone runtime exists

4. Impact context cannot be chained between steps
- the current cast state only tracks step result summary
- it does not preserve:
  - impact position
  - first-hit unit
  - area instance id

These are acceptable for the step-based refactor v1, but not for the
continuous spatial layer.

## Long-Term Runtime Model

The runtime should split into two layers:

1. **Step execution layer**
- decides when a step fires
- decides which delivery to spawn
- consumes delivery impact context to apply effects

2. **Spatial delivery layer**
- owns projectile flight / collision
- owns instant area overlap
- owns persistent area lifetime and ticking

In short:

- `step decides intent`
- `delivery decides contact`
- `effect application consumes impact context`

## Proposed Runtime Types

### 1. `SkillDeliveryRuntime`

Represents an in-flight or active skill delivery object.

Planned variants:

- `Projectile`
- `Area`

Suggested common fields:

- `delivery_id`
- `cast_seq`
- `step_index`
- `skill_id`
- `step_id`
- `caster_instance_id`
- `fired_at_ms`
- `hit_targets`

### 2. `ProjectileRuntime`

Represents a skill projectile in flight.

Suggested fields:

- `start: ContinuousPosition`
- `aim: ContinuousPosition`
- `speed_units_per_ms`
- `guidance`
  - `Homing`
  - `Fixed`
- `collision_radius_units`
- `piercing`
- `max_hits`
- `hit_unit_ids`

Important rule:

- basic attacks do not use this collision/miss model
- only skill projectile uses it

### 3. `AreaRuntime`

Represents an active area effect.

Suggested fields:

- `anchor`
  - point position
  - or path/orientation snapshot if needed later
- `shape`
  - `Circle`
  - `Rectangle`
- `spawned_at_ms`
- `expires_at_ms`
- `tick_interval_ms`
- `hit_targets`
- `already_hit_this_tick` or `already_hit_units`

This supports:

- instant blast
- persistent ground zone

### 4. `SkillImpactContext`

This is the critical bridge between delivery and effect.

Required fields:

- `delivery_id`
- `impact_time_ms`
- `impact_position`
- `first_hit_unit_id`
- `hit_unit_ids`
- `spawned_area_id`

This context should be saved into active cast state so later steps can use it.

## Active Cast State Expansion

Current `ActiveSkillCast` keeps:

- `caster_owner`
- `anchor_position`
- `allow_dead_caster`
- `last_resolved_step`

This should be extended with spatial context.

Suggested additions:

- `last_impact_context: Option<SkillImpactContext>`
- `active_area_ids: Vec<AreaInstanceId>`

This enables step chains like:

1. projectile direct hit
2. explosion at impact point
3. persistent area spawned at impact point

## Event Model Changes

## Current Problem

`BattleEvent::ProjectileHit { projectile_id, attacker_instance_id, target_instance_id, payload }`
is too tied to one target instance.

That works for:

- basic attack
- targeted homing projectile

It does not fit well for:

- untargeted skillshot projectile
- projectile pierce
- projectile that hits one unit and explodes
- persistent area spawned at impact

## Target Event Model

Keep basic attack events simple.

For skills, add dedicated delivery events:

- `BattleEvent::SkillProjectileImpact`
- `BattleEvent::SkillAreaTick`
- `BattleEvent::SkillAreaExpire`

Suggested fields for `SkillProjectileImpact`:

- `time_ms`
- `delivery_id`
- `cast_seq`
- `step_index`
- `skill_id`
- `step_id`
- `impact_position`
- `first_hit_unit_id`

Suggested fields for `SkillAreaTick`:

- `time_ms`
- `area_id`
- `cast_seq`
- `step_index`
- `skill_id`
- `step_id`
- `center`

This lets the effect system rebuild target sets from impact context
instead of pretending the projectile already knew the final target at fire time.

## Data Model Evolution

### Existing `DeliveryDef`

Current:

- `Instant`
- `Projectile { speed_units_per_ms }`

### Target Direction

Add explicit runtime-oriented delivery definitions for skill steps.

Recommended future shape:

- `Instant`
- `Projectile`
  - `speed_units_per_ms`
  - `collision: SkillProjectileCollisionDef`
- `Area`
  - `shape: SkillAreaShapeDef`
  - `anchor: SkillAreaAnchorSource`
  - `hit_targets: SkillHitTargetFilter`
  - `duration_ms`
  - `tick_interval_ms`

Important:

- this does **not** require every instant skill to become area delivery
- `Instant` remains useful for self buffs, direct targeted effects, and simple
  non-spatial step application

## First Runtime Slice

The first vertical slice should be:

1. targeted skill projectile remains homing
2. untargeted skill projectile uses first-hit continuous collision
3. effect application still executes the original step's effects
4. no persistent area yet

Why:

- smallest slice that proves the new runtime shape
- avoids mixing projectile and area lifecycles too early

## Second Runtime Slice

Add instant area delivery:

- `Circle`
- `Rectangle`
- explicit anchor source
  - `CastTarget`
  - `ImpactContext`
  - `Caster`

This should support both:

- direct area step
- impact-follow-up area step using previous `SkillImpactContext`

## Third Runtime Slice

Add persistent area runtime:

- area instance spawn
- periodic `SkillAreaTick`
- expiration event
- optional per-tick hit memory

## Code Entry Points

### Current code that will change

- `core/src/game/ability.rs`
  - delivery data schema
- `core/src/game/battle/enums.rs`
  - new battle events
- `core/src/game/battle/core/types.rs`
  - runtime delivery / impact context / area instance state
- `core/src/game/battle/core/sim.rs`
  - `execute_skill_step()`
  - skill delivery spawn path
- `core/src/game/battle/core/commands.rs`
  - projectile resolution path
  - future area tick handlers

### Current code that should stay stable

- basic attack timing and hit rule
- movement/orchestrator decision layer
- continuous movement segment contract

## Migration Strategy

### Phase 1

- add new runtime types and event variants
- keep old `ProjectilePayload::SkillStep` behavior temporarily
- no behavior change yet

### Phase 2

- untargeted skill projectile uses new first-hit runtime path
- targeted skill projectile continues to emit `SkillProjectileImpact`,
  but remains launch-time scheduled unless a later requirement justifies
  active runtime migration
- basic attack path remains separate

### Phase 3

- add instant area runtime

### Phase 4

- add persistent area runtime

### Phase 5

- remove or heavily reduce legacy `ProjectilePayload::SkillStep` assumptions
- completed:
  - skill projectile no longer schedules legacy `ProjectileHit + ProjectilePayload::SkillStep`
  - `ProjectileHit` is back to basic-attack-only delivery
  - skill projectile delivery is unified on `SkillProjectileImpact`
  - structural cleanup:
    - `skill_runtime/cast.rs`
    - `skill_runtime/area.rs`
    - `skill_runtime/projectile.rs`
      now own the main skill runtime helpers that previously lived in `sim.rs` / `commands.rs`

## Testing Strategy

Add tests in this order:

1. schema tests
- projectile collision def
- area delivery def

2. spatial utility tests
- segment-circle collision
- rectangle overlap

3. delivery runtime tests
- untargeted projectile misses moving target
- untargeted projectile hits first valid unit only
- targeted projectile still homes and hits
- post-launch hard CC can change fixed projectile hit result

4. area runtime tests
- instant circle only hits units inside radius
- rectangle respects width/length
- persistent area ticks repeatedly and expires

5. battle-level scenario tests
- skillshot misses because target sidesteps
- projectile direct-hit then explosion
- persistent zone damages units over time

## Current Verdict

The right long-term move is **not** to keep stretching `ProjectilePayload::SkillStep`.

The right move is:

- formalize spatial delivery runtime
- treat projectile/area as delivery objects
- preserve impact context in the cast state
- let steps consume that context deterministically

## Remaining Work

현재 남은 직접 작업은 아래 순서가 맞다.

1. live `.ron` skill migration 확대
   - 아직 `Instant`로 남아 있지만 실제 spatial delivery가 더 명확한 step만 선별
2. area shape / orientation expressiveness refinement
   - 실제 데이터가 `Rectangle/Cone`을 넘어서는 표현력을 요구할 때만
3. structural cleanup
   - step effect application bridge 분리
4. projectile 추가 확장
   - 현재는 보류
   - 특히 `targeted homing skill projectile`를 fixed와 같은 active runtime으로
     올릴지는 실제 요구가 생길 때만 다시 연다
