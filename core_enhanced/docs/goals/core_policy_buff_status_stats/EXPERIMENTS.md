# Experiments

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Goal setup | Defined buff/status/stat implementation scope. | Not started. | Read active effect state and action gate code first. |
| 2026-06-22 | Hard CC source of truth | Added active hard-CC expiry helpers and routed movement, basic attack, manual skill, autocast, and resonance gain gates through active `Stun`/`Freeze` buffs instead of copying hard CC duration into `ActionLocks`. | Success. `cargo test hard_cc --lib` passed. | Continue buff lifecycle cleanup and timeline reason coverage. |
| 2026-06-22 | Buff expiration reasons | Added `BuffExpireReason` to `TimelineEvent::BuffExpired`. Hard CC replacement now emits `BuffExpired(reason=Replaced)` before the new `BuffApplied`, and death cleanup emits `TargetDied`/`CasterDied`. | Success. Focused replacement/death tests passed. | Continue validation and broad tests. |
| 2026-06-22 | Move speed stat source | Spawned body speed now uses final `UnitStats.move_speed_units_per_ms`; runtime `ApplyModifier(MoveSpeedUnitsPerMs)` updates `UnitBody.move_speed` for the next movement tick without rewriting active segments. | Success. Focused move-speed tests passed. | Run broad validation. |
| 2026-06-22 | Control status validation | Enforced `max_stacks == 1` for `Stun`, `Freeze`, and `Silence` metadata. | Success after updating an old `refreshing_silence` validator fixture that used `max_stacks: 3` despite not testing stacks. | Run live RON and broad validation. |
| 2026-06-22 | Final subgoal validation | Ran live RON loading, server compile, full core lib tests, and diff hygiene checks. | Success. Subgoal completion conditions are met. | Mark subgoal complete and continue master sequence. |

## Failed Approaches

| Date | Scope | Failed Attempt | Reason | Corrective Action |
| --- | --- | --- | --- | --- |
| 2026-06-22 | Control status validation | `cargo test buff --lib` initially failed on `buff_validator_respects_refresh_duration_reapply_policy`. | The fixture authored `BuffKind::Silence` with `max_stacks: 3`, which conflicts with the confirmed non-periodic control status invariant. | Changed the fixture to `max_stacks: 1` because the test covers `RefreshDuration` expiry replacement, not stack count. Re-ran `cargo test buff --lib` successfully. |

## Validation Commands

- `cargo check --lib` - passed after initial hard CC/stat patches.
- `cargo test hard_cc_replacement_records_explicit_buff_expired_reason --lib` - passed.
- `cargo test unit_death_clears_target_and_caster_active_buffs_with_reasons --lib` - passed.
- `cargo test move_speed_modifier_updates_runtime_body_speed_for_next_movement_tick --lib` - passed.
- `cargo test control_status_metadata_rejects_stackable_hard_cc --lib` - passed.
- `cargo test hard_cc --lib` - passed, 6 tests.
- `cargo test buff --lib` - failed once on legacy stackable silence fixture, then passed after fixture update, 16 tests.
- `cargo test move_speed --lib` - passed, 2 tests.
- `cargo test death_clears --lib` - passed, 1 test.
- `rg -n "is_hard_cc|Stun\\s*\\|\\s*crate::game::battle::buffs::BuffKind::Freeze|lock_movement_until\\(lock_until\\)|lock_basic_attack_until\\(lock_until\\)|lock_resonance_gain_until\\(lock_until\\)|next_action_time = .*lock_until" src/game/battle/core -S` - no hard CC lock-copy writes remained.
- `cargo test --test ron_loading` - passed, 18 tests.
- `cargo check -p game_server` - passed.
- `cargo test --lib -- --test-threads=1` - passed, 507 tests.
- `git diff --check` - passed.
