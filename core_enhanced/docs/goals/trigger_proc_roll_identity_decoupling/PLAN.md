# Trigger Proc Roll Identity Decoupling

## Objective

Move non-damage triggered ability proc rolls from state-count based RNG identity to explicit proc occurrence identity.

Current damage critical rolls already use explicit `CombatRollIdentity`. Triggered ability proc rolls should follow the same principle:

```text
event log / trigger state = observation and proc state
proc roll identity = gameplay judgment identity
```

The current proc roll code is not directly coupled to `event_log_seq`, but it still uses `trigger_count` as RNG seed material. `trigger_count` is closer to "successful proc count" than "which proc candidate is being judged". That makes simultaneous or repeated proc candidates hard to reason about, especially when multiple targets, hit indexes, death triggers, or projectile impacts create multiple candidates at the same battle time.

## Source Of Truth Order

1. Current runtime code.
2. Current live RON/data that defines ability activations, triggered effects, attacks, skills, items, and artifacts.
3. Unity-facing battle event/checkpoint contracts.
4. Canonical battle/combat policy docs.
5. Historical goal docs.

Do not trust older docs if runtime code disagrees. Record disagreements in `EXPERIMENT_NOTES.md` and prefer a long-term runtime design unless a user-facing policy decision is needed.

## Current Runtime Findings

Relevant files:

- `src/game/battle/core/sim.rs`
  - `proc_roll_percent(...)` uses `source`, `ability_id`, `binding_index`, `trigger_count`, and `time_ms`.
  - `should_fire_triggered_ability(...)` uses `AbilityProcState.trigger_count` both as successful proc state and RNG seed material.
- `src/game/battle/damage.rs`
  - `BattleCommand::TriggerAbility` carries activation source and binding information, but no explicit proc occurrence identity.
- `src/game/battle/core/commands.rs`
  - `activation_commands_from_bindings(...)` converts `SourcedAbilityActivation` into `BattleCommand::TriggerAbility`.
  - The conversion currently loses richer trigger occurrence context.
- `src/game/battle/core/basic_attack_projectile.rs`
  - Basic attack projectile impact creates OnAttack/OnHit trigger ability commands.
- `src/game/battle/core/death_runtime.rs`
  - OnDeath, OnKill, and OnAllyDeath trigger ability commands are created here.
- `src/game/battle/core/types.rs`
  - `AbilityProcKey` and `AbilityProcState` track cooldown/max-trigger state.

Important distinction:

```text
AbilityProcState.trigger_count = proc state for max triggers, cooldown/debug, and successful activation tracking.
ProcRollIdentity = RNG identity for one proc candidate judgment.
```

Do not remove `trigger_count` just to decouple RNG. It still has a useful state role.

## Confirmed Direction

Introduce explicit proc roll identity.

Recommended shape:

```rust
pub struct ProcRollIdentity {
    pub trigger_type: TriggerType,
    pub activation_source: CooldownSource,
    pub ability_id: SkillId,
    pub binding_index: usize,
    pub caster_id: UnitInstanceId,
    pub trigger_unit_id: UnitInstanceId,
    pub counterpart_unit_id: Option<UnitInstanceId>,
    pub target_id: Option<UnitInstanceId>,
    pub occurrence_id: uuid::Uuid,
    pub occurrence_index: u32,
}
```

The final code does not need to use these exact names, but it must preserve the responsibilities:

- `trigger_type` distinguishes OnAttack, OnHit, OnDeath, OnKill, OnAllyDeath, OnBattleStart, and future triggers.
- `activation_source`, `ability_id`, and `binding_index` keep the proc binding identity.
- `caster_id`, `trigger_unit_id`, `counterpart_unit_id`, and `target_id` keep unit context explicit.
- `occurrence_id` distinguishes a stable trigger occurrence such as an attack resolution, projectile impact, death event, battle start hook, skill step, or future cast step.
- `occurrence_index` distinguishes multiple proc candidates from the same occurrence, such as several targets or several bindings.

Do not use:

- `event_log_seq`
- event count
- event insertion position
- global RNG draw order
- successful `trigger_count` as the primary roll identity

`current_time_ms` may remain useful context, but it should not be the only thing that distinguishes simultaneous proc candidates.

## Design Requirements

1. Triggered ability proc RNG must be based on explicit proc occurrence identity.
2. `AbilityProcState.trigger_count` must remain available for max trigger count, internal cooldown, and debug/state tracking.
3. Failed proc candidates must not cause later candidates at the same time/source/binding to accidentally share the same roll because successful count did not advance.
4. Multiple proc candidates from the same trigger occurrence must be independently identifiable.
5. Proc identity must be created before `BattleCommand::TriggerAbility` loses trigger occurrence context.
6. Existing proc chance, internal cooldown, max triggers, trigger timing, command ordering, and skill invocation semantics must not change.
7. No Unity-facing DTO or live RON schema changes are expected for this goal.

## Non-Goals

- Do not change proc chance values.
- Do not change `internal_cooldown_ms` semantics.
- Do not change `max_triggers_per_battle` semantics.
- Do not remove `AbilityProcState.trigger_count`.
- Do not change triggered effect application rules.
- Do not change skill cast, projectile, death, or on-battle-start trigger ordering.
- Do not change damage critical roll identity work already completed in `docs/goals/combat_roll_identity_rng_decoupling`.
- Do not introduce a global mutable RNG or draw-order dependent RNG.
- Do not change Unity-facing battle event/checkpoint DTOs unless runtime code proves it is unavoidable.

## Implementation Plan

1. Re-read current proc roll and trigger activation paths:
   - `proc_roll_percent(...)`
   - `should_fire_triggered_ability(...)`
   - `BattleCommand::TriggerAbility`
   - `activation_commands_from_bindings(...)`
   - OnAttack/OnHit immediate basic attack paths
   - basic attack projectile impact paths
   - OnDeath/OnKill/OnAllyDeath paths
   - OnBattleStart paths
2. Define a battle-domain proc roll identity type close to trigger/proc runtime code.
3. Extend the trigger activation conversion boundary so each `BattleCommand::TriggerAbility` carries explicit proc roll identity.
4. Add trigger occurrence context to `activation_commands_from_bindings(...)` or replace it with a more explicit helper.
5. Update all call sites to provide stable occurrence context:
   - basic attack resolution;
   - basic attack projectile impact;
   - target OnHit;
   - attacker OnAttack;
   - death/killer/ally-death triggers;
   - battle start triggers.
6. Replace `proc_roll_percent(...)` seed inputs so it accepts `ProcRollIdentity`.
7. Keep `AbilityProcKey` / `AbilityProcState` for cooldown and max-trigger state.
8. Add focused tests that lock behavior:
   - multiple failed proc candidates at the same time do not share a roll solely because `trigger_count` did not advance;
   - OnAttack and OnHit proc identities differ for the same battle time;
   - projectile impact proc identity is stable and independent from unrelated event log insertion;
   - trigger count still enforces `max_triggers_per_battle`;
   - internal cooldown still blocks later proc attempts.
9. Run focused tests after each small change.
10. Run broad validation at the end.
11. Keep `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` updated with failed approaches, decisions, and verification results.

## Test Strategy

Prefer behavior and contract tests over private helper tests.

Required focused coverage:

- A regression test showing proc roll identity does not use `event_log_seq`.
- A test showing two proc candidates from the same source/ability/binding/time can receive distinct occurrence identities even if the first fails and `trigger_count` remains unchanged.
- A test proving `trigger_count` still increments only on successful proc and still gates `max_triggers_per_battle`.
- A test proving internal cooldown still applies after a successful proc.
- A projectile or death-trigger path test if fixture cost is reasonable; otherwise record the gap and reason in `EXPERIMENT_NOTES.md`.

Useful focused commands:

- `cargo test -p game_core proc_roll --lib -- --test-threads=1`
- `cargo test -p game_core trigger --lib -- --test-threads=1`
- `cargo test -p game_core item_activation_proc --lib -- --test-threads=1`
- `cargo test -p game_core projectile --lib -- --test-threads=1`
- `cargo test -p game_core death --lib -- --test-threads=1`
- `cargo test -p game_core game::battle --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

Adjust filters to actual test names created during implementation.

## Completion Conditions

- Triggered ability proc RNG no longer uses successful `trigger_count` as primary seed identity.
- `BattleCommand::TriggerAbility` or its equivalent carries explicit proc roll identity.
- Proc state and proc roll identity have separate responsibilities in code.
- Multiple proc candidates are identifiable by trigger occurrence, not by event log order or successful proc count.
- Existing proc chance/cooldown/max-trigger behavior is preserved.
- Tests lock the new identity contract and existing state semantics.
- No Unity-facing DTO, live RON schema, or balance value changes are introduced.
- Final report lists changed contracts, preserved semantics, new tests, remaining risks, and validation commands.

## Implementation Result (2026-07-06)

Status: complete.

Implemented explicit proc occurrence identity for triggered ability proc rolls:

- Added `ProcRollIdentity` to the battle damage/command domain.
- Extended `BattleCommand::TriggerAbility` to carry `proc_roll_identity`.
- Added `TriggerAbilityContext` so `activation_commands_from_bindings(...)` receives trigger occurrence context before the command conversion loses it.
- Replaced `proc_roll_percent(...)` inputs with `ProcRollIdentity`.
- Kept `AbilityProcState.trigger_count` for successful proc count, max-trigger gating, internal cooldown state, and debug/state semantics.
- Built proc state keys from the carried proc identity fields so state and roll identity cannot silently diverge inside `should_fire_triggered_ability(...)`.
- Updated trigger activation call sites:
  - instant basic attack OnAttack/OnHit use the committed damage source instance id as occurrence id;
  - basic attack projectile impact OnAttack/OnHit use the projectile id as occurrence id;
  - OnDeath/OnKill/OnAllyDeath use a deterministic death occurrence id;
  - OnBattleStart uses a deterministic battle-start occurrence id per unit.
- Added focused tests for:
  - proc roll independence from event log sequence;
  - proc occurrence identity affecting rolls independently from successful count;
  - `TriggerAbility` carrying proc roll identity through the conversion boundary;
  - `max_triggers_per_battle` still gating successful activations.

No Unity-facing DTO, live RON schema, proc chance, internal cooldown, max-trigger, trigger order, skill invocation, or damage critical roll policy was changed.

## Validation Run (2026-07-06)

- `cargo test -p game_core proc_roll --lib -- --test-threads=1` passed.
- `cargo test -p game_core item_activation_proc --lib -- --test-threads=1` passed.
- `cargo test -p game_core trigger --lib -- --test-threads=1` passed.
- `cargo test -p game_core projectile --lib -- --test-threads=1` passed.
- `cargo test -p game_core death --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::battle --lib -- --test-threads=1` passed.
- `cargo check -p game_core` passed.
- `cargo fmt --check` passed.
- `rg -n "trigger_count" src/game/battle/core/sim.rs src/game/battle/core/commands.rs src/game/battle/damage.rs -g '*.rs'` shows `trigger_count` only in proc state read/increment paths.

Observed unrelated warnings:

- `auth_server/Cargo.toml` has an unused manifest key warning for `env`.
- test builds still warn about an unused `buffs::BuffDatabase` import in `src/game/world/tests/mod.rs`.

## Stop Conditions

Complete this goal as a policy-decision report and stop if:

- Stable proc occurrence identity requires a Unity-facing DTO change.
- Stable proc occurrence identity requires live RON schema changes.
- Existing trigger call sites cannot provide enough occurrence context without changing trigger semantics.
- A policy decision is needed about whether a trigger should roll per target, per source, per hit, per binding, or per event.
- The change would alter known player-visible proc outcomes in a way that requires balance approval.

## Follow-Up Candidates

- Unify naming between `CombatRollIdentity` and proc roll identity if both patterns stabilize.
- Document the general combat RNG identity contract in a canonical battle-system document:
  - event log is observation;
  - combat roll identity is gameplay;
  - proc state is not RNG identity.
- Add replay-level verification that full battle outcomes remain stable across event-log-only changes.
