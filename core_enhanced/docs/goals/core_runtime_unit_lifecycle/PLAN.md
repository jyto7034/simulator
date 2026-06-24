# Core Runtime Unit Lifecycle

## Objective

Add `RuntimeUnitLifecycle { Active, Withdrawn, Dead }` and make lifecycle the canonical source for active gameplay participation.

Confirmed policy source: `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`, decisions 3, 4, 5, 6, 11, 12, 13, 14, and 15.

## Required Startup Protocol

At the beginning of this goal and every resumed run, read:

- `docs/goals/core_runtime_unit_lifecycle/PLAN.md`
- `docs/goals/core_runtime_unit_lifecycle/EXPERIMENTS.md`
- `docs/goals/core_runtime_unit_lifecycle/EXPERIMENT_NOTES.md`
- `docs/goals/core_followup_policy_implementation_master/PLAN.md`
- `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`
- `docs/refactor_preparation_plan.md`

Do not start code search, edits, or validation before reading those documents.

## Scope

### In Scope

- Add `RuntimeUnitLifecycle` to `RuntimeUnit`.
- Define `BattleCore.units` as materialized runtime entity registry, not active-only list.
- Make active gameplay participation depend on `lifecycle == Active`.
- Make death set `lifecycle = Dead`, `stats.current_health = 0`, and `ActionState::Dead`.
- Make `unit.is_dead()` lifecycle-based.
- Add withdrawn semantics without adding `ActionState::Withdrawn`.
- Ensure movement, targeting, skill casting, new damage candidates, and checkpoint `units` filter to Active units.
- Clear active gameplay state on withdrawal.
- Remove battle runtime buffs/debuffs from withdrawn units.
- Reset skill/action runtime state for future redeploy instances.
- Preserve withdrawn unit final `body.position`.

### Out Of Scope

- Redeploy HP and new `UnitInstanceId` creation.
- Projectile/delayed-effect resolution for inactive units.
- Event DTO position additions.
- Debug/admin inactive unit query design.

## Implementation Plan

1. Inventory `is_dead`, `ActionState::Dead`, unit filters, checkpoint unit filters, and withdrawal/death paths.
2. Add lifecycle type and initialize new runtime units as Active.
3. Convert death handling to set lifecycle plus existing invariants.
4. Convert active candidate filters to lifecycle gates.
5. Implement withdrawn lifecycle transition and active-state cleanup.
6. Remove withdrawn units from active gameplay loops without deleting them from `BattleCore.units`.
7. Update tests around death, targeting, movement, checkpoint visibility, and withdrawal.
8. Run focused battle runtime tests and `cargo check`.

## Policy Decision Completion Condition

If implementation exposes a new user-visible behavior not already covered by lifecycle decisions, such as ambiguous buff cleanup timing, action lock semantics, or checkpoint/debug visibility, complete the goal with `사용자와 정책 논의 필요` and code evidence.

## Completion Conditions

- `RuntimeUnitLifecycle` exists and is the canonical active/dead/withdrawn gate.
- Dead and withdrawn units remain in `BattleCore.units` but are excluded from active gameplay loops.
- `ActionState` and HP are invariants or behavior state, not lifecycle source of truth.
- Official checkpoint `units` contains Active units only.
- Focused and broad validation commands are recorded.

## Current Status

Status: implementation and completion review complete.

Report: `docs/goals/core_runtime_unit_lifecycle/POLICY_DECISION_REPORT.md`

Confirmed decision:

- Add `BuffExpireReason::TargetWithdrawn`.
- Add `BuffExpireReason::CasterWithdrawn`.
- Add `BattleLogEvent::SkillCastCancelled { caster_instance_id, interrupted_skill_id, interrupted_cast_seq, reason: SkillCastCancelReason::Withdrawn }`.
- Keep `SkillCastInterrupted` for external interruption only; do not overload it for self-withdrawal cleanup.
- Cover withdrawal cleanup events in validator and Unity-facing DTO tests.

Implementation notes from this pass:

- `RuntimeUnitLifecycle` is added to `RuntimeUnit`.
- Withdrawal keeps the runtime unit in `BattleCore.units` as `Withdrawn`.
- Death sets lifecycle `Dead`, HP 0, and `ActionState::Dead`.
- Official live checkpoint unit lists filter to Active units only.
- Active gameplay candidate paths now use lifecycle active checks.
- Withdrawal emits `UnitWithdrawn`, withdrawal-specific `BuffExpired` reasons, and `SkillCastCancelled(reason=Withdrawn)` for pending/active casts.
- `SkillCastCancelled.interrupted_cast_seq` references `AutoCastStart`/`ManualCastStart` for pending casts and `AbilityCast` for active casts; validator accepts those start events.
- Completion review: `docs/goals/core_runtime_unit_lifecycle/REVIEW.md`.
