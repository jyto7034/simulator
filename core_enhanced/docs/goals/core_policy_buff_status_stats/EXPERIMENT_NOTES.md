# Experiment Notes

## Policy Notes

- Stun timing is gameplay-critical; prefer exact tick/event tests over implementation-shape tests.
- If event naming or DTO shape requires a new Unity-visible contract decision, mark `사용자와 정책 논의 필요`.
- Resolved implementation rule: `ActionLocks` remains for sequence locks such as cast/focus/windup/recovery, but hard CC duration is not copied into `ActionLocks` or `next_action_time`.
- Resolved implementation rule: active `Stun`/`Freeze` buff expiry is the source for movement, basic attack, cast completion/start, and resonance-gain gates. Runtime reschedules action events to `expires_at_ms + 1` to avoid same-timestamp ordering before `BuffExpire`.
- Resolved implementation rule: `BuffExpired.reason` is required. Normal scheduled expiry uses `Natural`; hard CC replacement uses `Replaced`; death cleanup uses `TargetDied` or `CasterDied`.
- Resolved implementation rule: `MoveSpeedUnitsPerMs` is synchronized into `UnitBody.move_speed` when final spawn stats or runtime stat modifiers change. Existing movement segments are not rewritten; the next movement tick uses the updated body speed.
- Resolved implementation rule: `Stun`, `Freeze`, and `Silence` metadata must declare `max_stacks == 1`; tests preserving stackable silence were updated rather than retained.

## Follow-Up Candidates

- None yet.
