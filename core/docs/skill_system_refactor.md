# Skill System Refactor

## Goal

Replace the current single-target-group skill model with a step-based model that can express:

- different target groups inside one skill
- delayed multi-step execution
- mixed delivery styles inside one skill
- richer replay/presentation hints without hardcoding in Unity

This refactor intentionally drops legacy compatibility. Existing `.ron` data and tests should be migrated to the new schema instead of supporting both formats.

## Problems In Current Model

Current shape:

- `SkillDef { target, delivery, effects[] }`

Limitations:

- one skill can only resolve one target group
- every effect is applied to the same resolved target set
- no native concept of delayed sub-effects
- no natural way to express "ally heal + enemy damage" or "hit then explode"
- projectile skills and instant skills are modeled only at whole-skill granularity

## New Model

### SkillDef

- `id`
- `name`
- `focus_time_ms`
- `focus_permissions`
- `steps: Vec<SkillStepDef>`

### SkillStepDef

- `id`
- `delay_ms`
- `range_tiles`
- `target: SkillTarget`
- `delivery: DeliveryDef`
- `effects: Vec<SkillEffectDef>`
- `presentation: SkillPresentationDef`

### SkillPresentationDef

Runtime does not need to fully consume this yet, but the data shape should exist now.

- `cast_state: Option<String>`
- `projectile_vfx_id: Option<String>`
- `impact_vfx_id: Option<String>`
- `target_anchor: Option<String>`

## Execution Model

1. `AutoCastStart`
   - validate caster
   - select initial cast context from the first step
   - record `AutoCastStart`
2. `AutoCastEnd`
   - record `AbilityCast`
   - enqueue one runtime event per step at `cast_end_ms + step.delay_ms`
3. `SkillStep`
   - resolve targets for that step using stored cast context
   - if delivery is `Instant`, apply commands immediately
   - if delivery is `Projectile`, spawn projectile hit event with step payload

## Cast Context

`PendingSkillCast` should store:

- `skill_id`
- `cast_target`

For now, one shared cast target is enough. More advanced per-step contexts can be added later if needed.

## Target Resolution Rules

- single-target steps reuse the stored cast target when valid
- if a single-target step has no valid cast target, it resolves to no targets
- area steps can use tile or unit anchor from the stored cast target
- if no anchor exists, they fall back to caster position

This keeps step behavior deterministic and avoids retargeting to unrelated enemies during execution.

## Timeline Impact

Keep existing high-level events:

- `AutoCastStart`
- `AutoCastEnd`
- `AbilityCast`

Add:

- `AbilityStepTriggered { skill_id, step_id, caster_instance_id, target_instance_id? }`

This is enough for replay to distinguish multi-step skills without overhauling all consumers at once.

## Runtime Event Impact

Add:

- `BattleEvent::SkillStep`

Update projectile payload:

- `ProjectilePayload::SkillStep { skill_id, step_id, cast_target }`

## Data Migration

Old data is not preserved.

- rewrite `game_resources/data/skills/base.ron`
- update any tests constructing `SkillDef`
- keep abnormality metadata pointing at `skill_id`

## Validation / Test Expectations

Add or update tests for:

- single-step instant skill
- single-step projectile skill
- two-step mixed-target skill
- two-step mixed-delivery skill
- `.ron` deserialization using new schema

## Initial Scope

This refactor covers:

- `core/src/game/ability.rs`
- battle runtime events and payloads
- `sim.rs` skill execution flow
- timeline event additions
- `game_resources/data/skills/base.ron`
- tests and `.ron` loading helpers

Unity replay can consume the new `AbilityStepTriggered` timeline event in a follow-up change.
