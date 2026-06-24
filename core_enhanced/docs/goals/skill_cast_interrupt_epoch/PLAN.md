# Skill Cast Interrupt Epoch Plan

## Objective

Implement a long-term skill cast interruption contract for battle skills.

The goal is to allow a skill effect to cancel another unit's currently pending skill focus/cast without deleting scheduled queue events. Scheduled `ManualCastEnd` and `AutoCastEnd` events must become safely stale when their original cast was interrupted.

This is a general battle runtime feature, not a The Silent Orchestra special case.

## Policy To Implement

1. `Silence` continues to mean "cannot start a new skill cast" and must not be repurposed as "cancel an already-started cast".
2. A new skill effect represents cast interruption explicitly.
3. Cast start timeline `seq` is the cast epoch/token.
4. `PendingSkillCast` stores the start `seq` that created it.
5. `ManualCastEnd` and `AutoCastEnd` only invoke the skill when their `cause: Parent { seq }` matches the current `PendingSkillCast.start_seq`.
6. If the pending cast was interrupted or replaced, the old cast-end event is stale and must not invoke the skill.
7. Successful interruption removes the target's pending cast and emits a Unity-facing timeline event.
8. Interruption applies only to the pending focus/cast before ability invocation.
9. Interruption does not cancel already emitted projectiles, active areas, scheduled skill steps, damage ticks, or triggered effects from a skill that has already invoked.
10. Cast start costs and already-spent resonance/focus resources are not refunded unless a future policy explicitly changes that.
11. Interrupt effects are hostile target-applied effects for automatic targeting only when they can attempt to interrupt a unit with a pending skill cast.
12. If no valid pending cast exists on the target, the interrupt effect is a no-op and must not fabricate a cast-cancel event.

## Scope

In scope:

- Add a first-class skill effect for interrupting a pending cast.
- Add cast epoch/token storage to pending manual and automatic skill casts.
- Make cast-end processing token-aware and stale-safe.
- Emit a presentation timeline event for successful interruption.
- Update validators so interrupted casts are accepted as valid terminal cast outcomes.
- Update target usefulness so interrupt-only hostile skills do not cast into targets that are not currently casting.
- Add focused runtime tests for manual cast interruption, auto cast interruption, stale cast-end handling, silence behavior, and Unity-facing event shape.
- Update core docs and Unity contract docs if the new timeline event becomes visible to Unity.
- Verify live RON loading still works, even if no live skill uses the new effect yet.

Out of scope:

- The Silent Orchestra boss chain, node rewards, balance, fragment acquisition, or conductor-specific scripting.
- Queue deletion or arbitrary event cancellation.
- Interrupting already invoked skill runtime objects such as projectiles, tile areas, persistent areas, or queued steps.
- Refund policy changes.
- New manual enemy target selection UX.
- Compatibility layers for old cast completion behavior.
- Long-lived dual schema for old and new timeline event names.
- Adding live content that uses the new effect unless the implementation needs a tiny test fixture.

## Source Of Truth

Verify in this order:

1. Runtime code:
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/core/types.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/battle/core/target_usefulness.rs`
   - `src/game/battle/timeline.rs`
   - `src/game/battle/enums.rs`
   - `src/game/battle/buffs.rs`
   - `src/game/ability.rs`
   - `src/game/data/skill_data.rs`
   - timeline validators under `src/game/battle/validation/`
2. Live RON/data:
   - `../game_resources/data/skills/base.ron`
   - `../game_resources/data/buffs/base.ron`
   - abnormality or fragment data discovered from runtime references
3. Unity-facing battle contract:
   - `docs/skill_target_contract.md`
   - `docs/game_rulebook.md`
   - external canonical `F:\unity projects\ark\docs\unity_core_contract.md`
4. Latest policy notes in goal docs and active design docs.

Do not trust the documents blindly. If runtime code shows a better long-term implementation than this plan, record the evidence in `EXPERIMENT_NOTES.md`. If the difference is a gameplay or Unity-facing policy decision, stop and ask the user.

## Plan

1. [x] Inventory current cast lifecycle.
   - Manual cast start/end.
   - Automatic cast start/end.
   - Pending cast storage.
   - Silence start blocking.
   - Action locks and resonance spending.
   - Timeline causes and validators.
2. [x] Confirm the cast epoch shape.
   - Prefer `CastStart` timeline `seq` as the epoch/token.
   - Avoid adding a second independent counter unless runtime evidence proves `seq` is insufficient.
3. [x] Add the data/model contract.
   - Add a new `SkillEffectDef` variant for cast interruption.
   - Add corresponding raw RON parsing/validation support.
   - Add `PendingSkillCast.start_seq`.
   - Add a `TimelineEvent::SkillCastInterrupted` event with enough data for Unity to tie it to the exact cast start.
4. [x] Implement cast start token storage.
   - Manual cast records `ManualCastStart`, then stores a pending cast with that start `seq`.
   - Auto cast records `AutoCastStart`, then stores a pending cast with that start `seq`.
   - Preserve existing focus/action-lock behavior.
5. [x] Implement token-aware cast end.
   - Extract expected start `seq` from the cast-end `cause`.
   - Invoke only if current pending cast exists and `pending.start_seq == expected_start_seq`.
   - Ignore stale cast-end events without recording misleading cast-end timeline events.
6. [x] Implement interrupt execution.
   - Convert the new skill effect into a battle command or equivalent runtime operation.
   - On a valid target with pending cast, remove pending cast and emit `SkillCastInterrupted`.
   - Ensure action locks and next action timing settle into a sane post-interrupt state without granting refunds.
7. [x] Update automatic target usefulness.
   - Interrupt-only hostile effects are useful only for units that currently have a pending skill cast.
   - Preserve normal target ordering after usefulness filtering.
8. [x] Update validators and DTO/event references.
   - Interrupted auto/manual casts are valid terminal outcomes.
   - `SkillCastInterrupted.interrupted_cast_seq` must reference a Unity-facing `ManualCastStart` or `AutoCastStart`.
   - Update unit-reference, spawn/death/battlefield, parent-cause, and version validation as needed.
9. [x] Add focused behavior tests.
   - Manual pending cast can be interrupted before completion.
   - Auto pending cast can be interrupted before completion.
   - Stale cast-end does not invoke an interrupted cast.
   - Stale cast-end does not invoke a newer cast that reused the same caster.
   - Silence still blocks starts but does not cancel already pending casts.
   - Interrupt-only automatic targeting waits when no enemy is casting.
   - Interrupt-only automatic targeting can select a currently casting enemy.
10. [x] Run focused checks after each implementation slice and record failures in `EXPERIMENTS.md`.
11. [x] Run broad checks before completion.
12. [x] Update canonical docs and remove any legacy wording that conflicts with the new contract.

## Completion Conditions

- Cast interruption is represented as a first-class skill effect, not as a `Silence` overload.
- Pending casts are guarded by a cast epoch/token derived from the cast-start timeline `seq`.
- Stale scheduled cast-end events cannot invoke interrupted or superseded casts.
- Successful interruption emits a Unity-facing timeline event that identifies the interrupted cast start.
- Interrupted casts do not emit misleading `ManualCastEnd` or `AutoCastEnd` completion events.
- Existing cast lifecycle behavior remains intact for uninterrupted casts.
- Automatic targeting treats interrupt effects as useful only when a target currently has an interruptible pending cast.
- No queue deletion, compatibility layer, fallback path, or boss-specific special case is introduced.
- Tests cover user-visible behavior, Unity-facing event shape, validation, live RON loading, and actual gameplay flow.
- Focused and broad verification commands pass.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Stop Conditions

Stop and ask the user if any of these are discovered:

- Interrupting should refund resonance, focus, cooldown, or other resources.
- Interruption should also cancel already invoked projectiles, areas, scheduled steps, or triggered effects.
- Interruption should apply to movement, basic attacks, deployment, boss phases, or non-skill actions.
- The new effect needs live RON schema choices that affect content authoring beyond a simple effect variant.
- Unity needs a different event name or payload than the runtime can naturally provide.
- Existing validation or transport contracts require preserving old misleading cast-end events after interruption.
- The implementation requires special rules for a named abnormality, named skill, or boss chain.
- The implementation would change command result shape, battle update envelope shape, or setup snapshot shape.

## Expected Verification Commands

Run focused checks while implementing:

- `cargo check -p game_core --tests`
- `cargo test -p game_core cast -- --nocapture`
- `cargo test -p game_core interrupt -- --nocapture`
- `cargo test -p game_core autocast -- --nocapture`
- `cargo test -p game_core timeline -- --nocapture`
- `cargo test -p game_core --test skill_refactor_validation -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`

Run broad checks before completion:

- `cargo fmt`
- `cargo test -p game_core`
- `cargo check -p game_server --tests`

## Implementation Summary

- Added first-class `SkillEffectDef::InterruptCast` and `BattleCommand::InterruptCast`.
- Added `PendingSkillCast.start_seq` and use cast-start timeline `seq` as the cast token.
- Added `TimelineEvent::SkillCastInterrupted` and bumped `TIMELINE_VERSION` to 24.
- Manual and auto CastEnd processing now ignores stale end events when their parent `seq` does not match the current pending cast.
- Successful interrupt removes the pending cast and emits `SkillCastInterrupted`.
- `ManualCastEnd` and `AutoCastEnd` now represent successful cast completion only; interrupted casts do not emit them.
- Automatic target usefulness treats interrupt-only hostile effects as useful only against targets with a pending cast.
- Timeline validation now accepts interrupted auto casts as terminal outcomes and verifies that `interrupted_cast_seq` references a real `ManualCastStart` or `AutoCastStart`.
- Core, local docs, README, goal docs, and external Unity contract docs were updated.

## Removed Legacy Behavior

- Removed implicit invocation of stale `ManualCastEnd` / `AutoCastEnd` events after a pending cast has been interrupted or replaced.
- Removed the old validator assumption that every `AutoCastStart` must terminate only with `AutoCastEnd`.
- Avoided overloading `Silence` as a cast cancel effect.

## Verification Results

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

## Remaining Risk

- No live RON skill currently uses `InterruptCast`; behavior is pinned by runtime tests and schema loading, but live content balancing remains a future content task.
- Interruption keeps existing action locks until their natural expiry. This avoids refunds/immediate recovery, but future gameplay tuning may decide to shorten locks explicitly.
