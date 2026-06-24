# Skill Cast Interrupt Epoch Experiments

This file records implementation attempts, failures, fixes, and verification results for the skill cast interrupt epoch goal.

## 2026-06-16 Goal Document Setup

Goal:

- Create the working goal documents for implementing skill cast interruption with cast epoch/token protection.
- Keep the feature general and long-term, not tied to a single boss or test fixture.

Files read:

- `docs/goals/automatic_hostile_target_usefulness/PLAN.md`
- `docs/goals/automatic_hostile_target_usefulness/EXPERIMENTS.md`
- `docs/goals/automatic_hostile_target_usefulness/EXPERIMENT_NOTES.md`
- `docs/skill_target_contract.md`
- `docs/game_rulebook.md`
- `src/game/battle/core/types.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/buffs.rs`
- `src/game/ability.rs`
- `src/game/data/skill_data.rs`
- `src/game/battle/core/target_usefulness.rs`
- `src/game/battle/validation/autocast.rs`

Files changed:

- `docs/goals/skill_cast_interrupt_epoch/PLAN.md`
- `docs/goals/skill_cast_interrupt_epoch/EXPERIMENTS.md`
- `docs/goals/skill_cast_interrupt_epoch/EXPERIMENT_NOTES.md`

Result:

- Created the required goal tracking documents.
- Fixed the initial policy that `Silence` blocks new starts, while cast interruption cancels pending casts.
- Fixed the initial long-term design direction: use the cast-start timeline `seq` as the cast epoch/token.
- Defined stop conditions for refund policy, already-invoked skill cancellation, Unity-facing event payload, live RON schema, and boss-specific behavior.

Failure cause, if any:

- None. This was a documentation-only setup step.

Fix or next action:

- Start implementation by reading the runtime cast lifecycle and validators before editing behavior.
- Record runtime findings in `EXPERIMENT_NOTES.md`.
- Add tests around stale cast-end events before broad refactors.

Verification:

- Not run. Documentation-only goal setup.

## 2026-06-16 Runtime Implementation

Goal:

- Implement `InterruptCast` as a first-class skill effect.
- Use cast-start timeline `seq` as the pending cast token.
- Make stale `ManualCastEnd` and `AutoCastEnd` harmless without deleting queued events.

Files read:

- `src/game/ability.rs`
- `src/game/battle/damage.rs`
- `src/game/battle/core/types.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/core/target_usefulness.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/validation/autocast.rs`
- `src/game/battle/validation/deaths.rs`
- `src/game/battle/validation/spawns.rs`
- `src/game/battle/validation/parent.rs`
- `src/game/battle/validation/validator.rs`
- `src/game/data/skill_data.rs`

Files changed:

- `src/game/ability.rs`
- `src/game/battle/damage.rs`
- `src/game/battle/core/types.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/core/target_usefulness.rs`
- `src/game/battle/timeline.rs`
- `src/game/battle/validation/autocast.rs`
- `src/game/battle/validation/auto_attack.rs`
- `src/game/battle/validation/deaths.rs`
- `src/game/battle/validation/spawns.rs`
- `src/game/battle/validation/parent.rs`
- `src/game/battle/validation/validator.rs`
- `src/game/data/skill_data.rs`

Result:

- Added `SkillEffectDef::InterruptCast`.
- Added `BattleCommand::InterruptCast`.
- Added `PendingSkillCast.start_seq`.
- Added `TimelineEvent::SkillCastInterrupted` and bumped `TIMELINE_VERSION` to 24.
- Manual and auto cast starts now store the cast-start `seq` in the pending cast.
- Manual and auto cast ends now invoke only when their parent `seq` matches the current pending cast token.
- A stale cast-end with no matching pending cast is ignored and does not emit `ManualCastEnd` or `AutoCastEnd`.
- Successful interrupt removes the pending cast and records `SkillCastInterrupted`.
- Interrupt-only automatic targeting is useful only against targets that currently have a pending cast.

Failure cause, if any:

- First focused test compile failed because `start_manual_skill_cast` was private to `sim.rs`; core module tests could not call it directly.

Fix or next action:

- Changed `start_manual_skill_cast` to `pub(super)` so battle-core internal tests can exercise the real manual cast lifecycle without creating a new public gameplay API.
- Continue with focused tests and broad verification.

Verification:

- `cargo check -p game_core --tests`
- `cargo test -p game_core interrupt -- --nocapture`
- `cargo test -p game_core cast -- --nocapture`
- `cargo test -p game_core autocast -- --nocapture`
- `cargo test -p game_core timeline -- --nocapture`

## 2026-06-16 Focused Tests And Docs

Goal:

- Pin user-visible and Unity-facing behavior.
- Keep policy docs synchronized with the new runtime contract.

Files changed:

- `src/game/battle/core/mod.rs`
- `src/game/battle/validation/validator.rs`
- `docs/skill_target_contract.md`
- `docs/game_rulebook.md`
- `F:\unity projects\ark\docs\unity_core_contract.md`

Result:

- Added focused tests for manual pending cast interruption, auto pending cast interruption, stale cast-end behavior, silence not canceling pending casts, interrupt-only target usefulness, interrupt effect command mapping, and validator rejection of invalid `interrupted_cast_seq`.
- Documented that `Silence` blocks starts while `InterruptCast` cancels pending casts.
- Documented that `SkillCastInterrupted.interrupted_cast_seq` points at the canceled `ManualCastStart` or `AutoCastStart`.
- Documented that interrupted casts do not produce `ManualCastEnd` or `AutoCastEnd`.

Failure cause, if any:

- None after the `pub(super)` test-access fix.

Fix or next action:

- Run formatting, live RON loading, skill validation, full `game_core`, and server check.

Verification:

- `cargo test -p game_core interrupt -- --nocapture`
- `cargo test -p game_core cast -- --nocapture`
- `cargo test -p game_core autocast -- --nocapture`
- `cargo test -p game_core timeline -- --nocapture`

## 2026-06-16 Final Verification

Goal:

- Verify the implementation across focused runtime tests, live RON loading, skill validation, full `game_core`, and server-facing compile checks.

Files changed:

- `docs/goals/skill_cast_interrupt_epoch/PLAN.md`
- `docs/goals/skill_cast_interrupt_epoch/EXPERIMENTS.md`
- `docs/goals/skill_cast_interrupt_epoch/EXPERIMENT_NOTES.md`
- `docs/README.md`

Result:

- Focused checks and broad checks passed.
- No live RON content needed to be changed for this goal.
- No command result, battle update envelope, or setup snapshot shape was changed.
- No compatibility layer, fallback path, queue deletion, or boss-specific special case was introduced.

Failure cause, if any:

- None in final verification.

Fix or next action:

- Future content work can add The Silent Orchestra skill fragments that use `InterruptCast`.
- Future policy can revisit post-interrupt action lock shortening if gameplay needs immediate recovery.

Verification:

- `cargo fmt`
- `cargo check -p game_core --tests`
- `cargo test -p game_core interrupt -- --nocapture`
- `cargo test -p game_core cast -- --nocapture`
- `cargo test -p game_core autocast -- --nocapture`
- `cargo test -p game_core timeline -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation -- --nocapture`
- `cargo test -p game_core`
- `cargo check -p game_server --tests`

## Experiment Log Template

Use this format for each implementation attempt:

```text
## YYYY-MM-DD <short attempt name>

Goal:

Files read:

Files changed:

Result:

Failure cause, if any:

Fix or next action:

Verification:
```
