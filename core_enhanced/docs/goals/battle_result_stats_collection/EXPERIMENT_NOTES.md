# Battle Result Stats Collection Notes

## Initial Policy Baseline

- Battle-time result metrics are core-owned gameplay interpretation.
- `BattleEventLog` is the canonical battle-time source.
- Battle result stats are derived at battle end from the completed event log and final result state.
- Unity should render typed stats and should not implement gameplay attribution rules from raw event logs.
- A live mutable stats cache is out of scope for the first implementation and must not become a second source of truth.
- The battle result wireframe is a mock direction. The first implementation should not try to implement every visible mock metric.
- First-slice scope is limited to metrics that current core code can compute from `BattleEventLog` and `ParticipantBattleResult` without new policy.

## Confirmed First-Slice Policies

Confirmed by the user on 2026-06-23:

- Damage stats record actual HP delta. Overkill is not counted.
- Basic attack count uses `AttackResolve`.
- Skill use count uses `AbilityCast`.
- MVP scoring is data-driven through RON/static data.
- First slice attributes only directly employee-sourced damage. Summon/minion, DOT/buff owner credit, reflection, defense-object, and environment attribution are deferred.
- Deployment time is summed from `UnitDeployed` to `UnitWithdrawn` / `UnitDied` / `BattleEnd`, including multiple deploy intervals.
- Incapacitated employees remain eligible for MVP.

## Initial Computability Notes

Likely computable from current code, subject to re-reading emitters before implementation:

- battle duration from `BattleEnd` entry time.
- damage dealt/taken from `HpChanged`.
- kill count from `UnitDied.killer_instance_id`.
- basic attack count from `AttackResolve`.
- skill cast count from `AbilityCast`.
- deploy/withdraw counts from `UnitDeployed` and `UnitWithdrawn`.
- survival/final HP/incapacitation from `ParticipantBattleResult`.
- enemy kill count from `UnitDied.owner`.

Needs code inspection before committing:

- deployment time: depends on current deploy/withdraw/death event coverage and whether all relevant lifecycle transitions are recorded.
- skill use count: depends on current skill cast/step event shape and whether retries/interruption semantics are unambiguous.
- healing done: `HpChanged.delta > 0` may be enough only if source attribution is reliable for healing.
- facility damage/leak prevention/wave progress: may require scenario-specific event fields or existing mission state.
- XP and trauma delta: may belong to combat reward/growth outcome rather than pure battle event stats.

## Implementation Notes

- `BattleResultStatsDto` is stored on `CombatBattleState`, so battle records and selected-event snapshots share the same core-owned derived result.
- `SelectedEventSnapshotDto::CombatBattle.result_stats` is the Unity result-summary source.
- `/game` server transport still adds `selected_event.compressed_event_log` for event-log details/debugging, but it does not own the result-stat meaning.
- MVP scoring uses `RunPolicyData.battle_result_stats.mvp.metric_weights` and `tiebreakers` from RON/static data.
- The collector deliberately ignores overkill by using `hp_before - hp_after`.
- The collector deliberately excludes non-first-slice damage sources such as buff tick and environment damage.
- Ability-sourced HP changes are identified through `DamageSource::Ability`; `HpChangeReason` remains the broader log reason (`Command` for command/ability-driven HP changes).
- Incapacitated employees remain present in `employees` and eligible for MVP.

Defer for now rather than filling placeholders:

- mock-only MVP panel fields that are not backed by current core data.
- wave clear timeline.
- leak prevention.
- facility damage percentage.
- shield/prevented damage.
- advanced medal taxonomy.

## Policy Questions To Stop For

If any of these becomes necessary during implementation, complete the goal with a question/report instead of guessing:

- summon/minion contribution ownership.
- DOT/buff/reflection/environment damage credit.
- MVP RON/static data schema if simple metric weights and tiebreakers are not enough.
- advanced medal rules.
- whether overkill, overheal, shield, or prevented damage count.
- whether reward/growth deltas belong in the same DTO.
- any event log schema version bump.
- any Unity-facing DTO shape not already accepted.

## Out Of Scope Follow-Ups

- Real-time in-battle leaderboard/HUD stats.
- Replay UI built from raw event log.
- Detailed medal achievement taxonomy.
- Balance tuning for MVP score weights.
- Historical battle record migration.

## Completion Review

Reviewed against `docs/goal_completion_review_guide.md` on 2026-06-23.

- Runtime source of truth: `BattleResultStatsDto` is derived from the completed `BattleEventLog` plus `ParticipantBattleResult`; no live mutable stats table or server-only JSON gameplay mutation was introduced.
- Data source: `RunPolicyData.battle_result_stats.mvp` and `../game_resources/data/run/policy.ron` own MVP weights/tie-breakers only, not per-battle measured facts.
- Unity/server contract: `SelectedEventSnapshotDto::CombatBattle.result_stats` is the typed Unity-facing source. `compressed_event_log` remains a transport/debug/detail attachment.
- Live probe evidence: `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py` now asserts the current JSON contract and passed against a real local server with `combat_result_game_state: "combat_result"`, `selected_event_type: "combat_battle"`, `has_event_log: true`, `result_stats_employee_count: 3`, and `result_stats_has_mvp: true`.
- Test evidence: focused collector tests cover actual HP delta, damage taken, basic attack count, skill cast count, deploy/withdraw count, deployment interval, participant merge, and MVP selection/eligibility; snapshot/server tests cover DTO exposure.
- Legacy/fallback audit: no compatibility schema, ignored legacy test, or alternate Unity stat-calculation path was added.
- Long-term fit: high. The implementation keeps gameplay interpretation in core, keeps policy data-driven, and avoids making Unity parse battle logs for summary stats.
