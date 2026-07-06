# Pve Encounter Bonus Objectives Notes

## Initial Notes

- This goal should not invent global fallback bonus objectives.
- `ClearWithin` must be omitted for late-spawn boss encounters unless a future explicit spawn-relative objective is designed.
- `DecisiveDamage` intentionally replaced final-hit overkill logic.
- Abnormality part acquisition is presentation fiction only.
- Bonus objective research is granted only on victory.

## Risks To Check

- Damage events may not currently carry enough classification to distinguish periodic/DoT from direct impacts.
- Existing combat result stats may be presentation-only and not appropriate as the objective source of truth.
- PvE encounter data may need new validation for primary abnormality target.
- Result DTO changes may need Unity sync.

## Implementation Notes

- Final RON layout keeps base victory research in live RON `run/policy.ron`; Phase 3 only adds `PveEncounter.suppression_research.bonus_objectives`.
- `ClearWithin` is evaluated from `BattleResultStatsDto.battle.duration_ms`, which is derived from `BattleEnd` event time and uses battle start as origin.
- `DecisiveDamage` is evaluated from completed `BattleEventLog` entries, not accumulated result stats.
- `DecisiveDamage` counts only one `HpChanged` event with `damage_source` `BasicAttack` or `Ability`, against a spawned primary abnormality target matching `PveEncounter.abnormality_id`.
- `BuffTick` and `Environment` do not count for `DecisiveDamage`.
- Bonus objective outcomes are stored on `CombatBattleState` and exported through both battle record JSON and `selected_event.bonus_objectives`.
- Existing live PVE encounters currently do not author bonus objectives. This is intentional for now: many current suppression encounters use corroded employee waves and do not spawn the primary abnormality unit directly.
- A PVE encounter that authors bonus objectives must spawn its primary abnormality target directly; multi-primary abnormality objective policy remains out of scope and fails authoring validation.
- Unity may display `bonus_objectives`, but research reward mutation remains core-owned and is reflected in the post-completion `abnormality_research` snapshot.

## Out-Of-Scope Follow-Up Candidates

- Spawn-relative clear time objective.
- Flat decisive damage threshold.
- Multi-target boss objective target groups.
- Bonus objective UI.
