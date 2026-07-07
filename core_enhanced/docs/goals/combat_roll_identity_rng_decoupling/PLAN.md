# Combat Roll Identity RNG Decoupling

## Objective

Remove battle gameplay RNG dependence on `event_log_seq`.

Critical hit rolls and future combat rolls must be derived from stable combat-domain roll identity, not from event log order, event count, or logging implementation details.

Current problem:

```text
event log = observation/output
combat roll = gameplay judgment
```

Those two are currently coupled for damage critical rolls. Adding, deleting, or reordering non-gameplay log events can change later critical outcomes, which makes logging improvements behave like balance changes.

## Source Of Truth Order

1. Current runtime code.
2. Current live RON/data that drives attacks, skills, buffs, and encounters.
3. Unity-facing battle event/checkpoint contracts.
4. Canonical combat/damage policy docs.
5. Historical goal docs.

Do not trust older docs if runtime code disagrees. Record the disagreement in `EXPERIMENT_NOTES.md` and prefer a long-term runtime design unless a user-facing policy decision is needed.

## Current Runtime Findings

Relevant files:

- `src/game/battle/core/damage_runtime.rs`
  - `damage_roll_percent_with_event_log_seq(...)` mixes `self.event_log_seq` into the seed.
  - `damage_source_snapshot_for_unit(...)` freezes `crit_roll_percent` using that event-log-coupled seed.
  - `materialize_damage_source_snapshot_for_target(...)` also uses `crit_roll_event_log_seq` when a delayed snapshot needs a target-specific roll.
- `src/game/battle/damage.rs`
  - `DamageSourceSnapshot` stores `crit_roll_event_log_seq`.
- `src/game/battle/recording.rs`
  - `record_event_log(...)` owns event log sequence assignment and increments `event_log_seq`.

Existing policy context:

- `docs/goals/core_damage_snapshot_contract/PLAN.md` already says crit roll is frozen from source snapshot inputs.
- The missing piece is that those source snapshot inputs still include event log sequence.

## Confirmed Direction

Use explicit combat roll identity.

```text
RNG seed = battle seed + CombatRollIdentity
```

Do not use:

- `event_log_seq`
- event count
- event insertion position
- RNG call order / draw index

The roll identity should be an explicit domain value, not an ad hoc tuple assembled differently at each call site.

Recommended shape:

```rust
pub struct CombatRollIdentity {
    pub roll_kind: CombatRollKind,
    pub source_unit_id: Option<UnitInstanceId>,
    pub target_unit_id: Option<UnitInstanceId>,
    pub source_instance: CombatSourceInstanceId,
    pub hit_index: u32,
}

pub enum CombatRollKind {
    BasicAttackCrit,
    SkillCrit,
    ProjectileHitCrit,
    StatusProc,
    RandomTargetSelection,
}
```

The final code does not need to use these exact names, but it must preserve the same responsibilities:

- `roll_kind` separates different random judgments.
- `source_instance` distinguishes separate attacks, casts, projectile deliveries, status ticks, or environment pulses.
- `source_unit_id` and `target_unit_id` bind unit-specific rolls.
- `hit_index` distinguishes multi-hit, piercing, area, or repeated target hits from the same source instance.

`committed_at_ms` may remain useful for debugging and event presentation, but it must not be the primary uniqueness key for RNG.

## Design Requirements

1. Gameplay RNG must be independent of event logging.
2. Identical combat roll identity must produce the same roll for the same battle seed.
3. Adding a harmless event log entry before damage resolution must not change critical results.
4. Delayed damage paths must keep stable source identity from commit/launch time to impact time.
5. Multi-hit and projectile paths must distinguish hit instances without relying on log sequence.
6. The new identity type must be visible enough that future random judgments can reuse the pattern.
7. Existing damage source snapshot policy remains:
   - source-side context is frozen at commit/launch/step time;
   - target defensive context remains live at apply/impact time unless another confirmed policy changes it.

## Non-Goals

- Do not change crit chance, crit damage, damage formula, mitigation, or feedback tags.
- Do not change Unity-facing event/checkpoint DTO shape unless runtime code proves it is unavoidable.
- Do not change event log sequence semantics.
- Do not introduce global mutable RNG or call-order dependent draw indices.
- Do not change projectile hit/miss policy.
- Do not change source snapshot commit points except where required to give each roll a stable identity.
- Do not implement unrelated random systems such as map, reward, shop, or boss selection in this goal.

## Implementation Plan

1. Re-read all current uses of `event_log_seq`, `crit_roll_event_log_seq`, `crit_roll_percent`, and combat RNG helpers.
2. Classify current combat roll sites:
   - basic attack instant crit;
   - basic attack projectile crit;
   - skill instant damage crit;
   - skill projectile crit;
   - buff tick or environment damage if they can crit;
   - proc rolls in `sim.rs` as a follow-up candidate if they do not currently use event log sequence.
3. Define explicit roll identity types close to battle damage/runtime code.
4. Replace `crit_roll_event_log_seq` in `DamageSourceSnapshot` with a stable roll identity or source-instance identity sufficient to materialize per-target crit rolls.
5. Replace `damage_roll_percent_with_event_log_seq(...)` with a roll helper that accepts `CombatRollIdentity`.
6. Make delayed source snapshots carry the needed identity from commit/launch time to impact/materialization.
7. Add focused tests:
   - adding a non-damage event before a damage roll does not change the crit result;
   - identical identity produces identical roll;
   - different `hit_index` or different source instance can produce independent rolls;
   - projectile or delayed damage preserves its launch/commit roll identity.
8. Run focused combat/damage tests after each small change.
9. Run broad validation at the end.
10. Keep `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` updated with failed approaches, decisions, and verification results.

## Test Strategy

Prefer behavior and contract tests over private helper tests.

Required focused coverage:

- A regression test that records an extra harmless event before damage materialization and proves the critical result is unchanged.
- A delayed projectile or delayed damage path test proving identity is carried across time.
- A multi-hit or repeated-hit style test if the current code has an easy fixture; otherwise record the missing fixture as follow-up.

Useful focused commands:

- `cargo test -p game_core damage_roll --lib -- --test-threads=1`
- `cargo test -p game_core crit --lib -- --test-threads=1`
- `cargo test -p game_core projectile --lib -- --test-threads=1`
- `cargo test -p game_core game::battle --lib -- --test-threads=1`
- `cargo check -p game_core`
- `cargo fmt --check`

Adjust names to actual test filters created during implementation.

## Completion Conditions

- No gameplay RNG seed for damage critical rolls depends on `event_log_seq`.
- `DamageSourceSnapshot` no longer stores `crit_roll_event_log_seq`.
- Critical roll identity is explicit in the battle/damage domain.
- Event log insertion/removal cannot change existing critical outcomes.
- Delayed damage paths still produce deterministic target-specific rolls.
- Tests lock the event-log-decoupling behavior.
- No Unity-facing DTO, live RON schema, or damage balance policy changes are introduced.
- Final report lists changed contracts, removed coupling, new tests, remaining risks, and validation commands.

## Implementation Result (2026-07-06)

Status: complete.

Implemented the combat roll identity split for damage critical rolls:

- Added `CombatRollKind` and `CombatRollIdentity` to the battle damage domain.
- Replaced `DamageSourceSnapshot.crit_roll_event_log_seq` with `DamageSourceSnapshot.crit_roll_identity`.
- Replaced `damage_roll_percent_with_event_log_seq(...)` with a roll helper that derives from `battle seed + CombatRollIdentity`.
- Added `BattleCore.damage_source_seq` as a battle-domain source instance id counter. This is used to distinguish committed damage source instances and is independent of event log sequence, event count, and RNG draw order.
- Added `materialize_damage_source_snapshot_for_target_hit(...)` so delayed or multi-target materialization can bind a target and `hit_index` without event log sequence.
- Updated skill step damage materialization to pass per-target hit indexes when materializing from a source template.
- Added regression tests proving:
  - inserting an extra event log entry before damage source creation does not alter the crit roll identity or crit roll percent;
  - delayed source templates materialize target-specific identities and distinct hit indexes without using event log sequence.

No Unity-facing DTO, live RON schema, damage math, crit chance, crit damage, mitigation, projectile hit/miss, or event log sequence policy was changed.

Known follow-up: non-damage proc rolls in `src/game/battle/core/sim.rs` still use their existing trigger-count based helper and were intentionally left as a follow-up candidate because this goal was scoped to damage critical rolls and event-log coupling.

## Validation Run (2026-07-06)

- `cargo test -p game_core damage_crit_roll_is_independent_from_event_log_sequence --lib -- --test-threads=1` passed.
- `cargo test -p game_core delayed_damage_materialization_uses_hit_index_identity_not_event_log_sequence --lib -- --test-threads=1` passed.
- `cargo test -p game_core crit --lib -- --test-threads=1` passed.
- `cargo test -p game_core projectile --lib -- --test-threads=1` passed.
- `cargo test -p game_core game::battle --lib -- --test-threads=1` passed.
- `cargo check -p game_core` passed.
- `cargo fmt --check` passed.
- `rg -n "crit_roll_event_log_seq|damage_roll_percent_with_event_log_seq" src tests -g '*.rs'` returned no matches.
- `rg -n "event_log_seq" src/game/battle/core/damage_runtime.rs src/game/battle/damage.rs -g '*.rs'` returned no matches.

Observed unrelated warnings:

- `auth_server/Cargo.toml` has an unused manifest key warning for `env`.
- test builds still warn about an unused `buffs::BuffDatabase` import in `src/game/world/tests/mod.rs`.

## Stop Conditions

Complete this goal as a policy-decision report and stop if:

- Stable roll identity requires a new Unity-facing DTO field.
- Stable roll identity requires live RON schema changes.
- Existing damage paths cannot distinguish source instances without changing skill/basic attack semantics.
- A choice is needed about whether a specific future roll, such as random target selection or status proc, should share the same identity system in this goal.
- Removing `event_log_seq` changes known player-visible crit outcomes in a way that requires balance approval.

## Follow-Up Candidates

- Move non-damage proc rolls in `src/game/battle/core/sim.rs` onto the same `CombatRollIdentity` pattern if they are currently call-order or trigger-count sensitive.
- Add a replay-verification test that compares full battle result stability across event-log-only changes.
- Document the combat RNG identity contract in a canonical battle-system document after implementation is complete.
