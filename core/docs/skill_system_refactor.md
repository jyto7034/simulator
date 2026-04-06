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
- `targeting: StepTargetingMode`
- `when: SkillStepCondition`
- `repeat: SkillStepRepeat`
- `delivery: DeliveryDef`
- `effects: Vec<SkillEffectDef>`
- `presentation: SkillPresentationDef`

### StepTargetingMode

- `ReuseCastTarget`
  - use the cast context selected at `AutoCastStart`
  - keeps delayed steps/projectiles tied to the original target or anchor
- `RetargetOnStep`
  - resolve a fresh target/anchor when the step actually executes
  - allows combinations like `SelfUnit -> EnemySingle`

### SkillStepCondition

- `Always`
- `IfPreviousStepDealtDamage`
- `IfCasterHasBuff { buff_id, min_stacks }`

### SkillStepRepeat

- `Once`
- `Times { count }`
- `ByBuffStacks { unit, buff_id, max? }`

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
   - resolve the step target context
   - evaluate `when`
   - evaluate `repeat`
   - resolve targets for each iteration using either the stored cast context or step-time retargeting
   - if delivery is `Instant`, apply commands immediately
   - if delivery is `Projectile`, spawn projectile hit event with step payload

## Cast Context

`PendingSkillCast` should store:

- `skill_id`
- `cast_target`

The cast stores one shared initial cast target, but each step can now choose whether to:

- reuse that cast target
- retarget at step execution time

This keeps delayed/projectile steps deterministic by default, while allowing step chains that need a fresh single-target resolution.

## Target Resolution Rules

- single-target steps can either reuse the stored cast target or retarget on execution
- if a single-target step has no valid cast target, it resolves to no targets
- area steps can use tile or unit anchor from the stored cast target
- if no anchor exists, they fall back to caster position

This keeps default behavior deterministic, while letting explicitly opted-in steps reacquire a fresh target when needed.

## Execution State

Each cast keeps runtime execution state so later steps can reference the previous resolved step result.

Current tracked result signals:

- whether the previous step hit
- whether the previous step dealt damage
- how many targets/effects were resolved

Projectile-delivered steps update this state at impact time, not at fire time.

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

- `ProjectilePayload::SkillStep { cast_seq, step_index, skill_id, step_id, step_target }`

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
- hit-gated follow-up step
- buff-stack-based repeated step
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
