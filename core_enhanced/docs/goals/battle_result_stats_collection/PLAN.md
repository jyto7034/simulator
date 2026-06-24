# Battle Result Stats Collection

## Objective

Implement core-owned battle result statistics for the Unity battle result screen.

The battle result UI needs MVP and per-employee metrics such as accumulated damage, kills, damage taken, skill use, deployment time, survival/incapacitation, XP, trauma delta, and reward summary. These metrics must be derived by core from authoritative battle data and exposed as typed DTOs. Unity should render and animate the result, not reinterpret raw battle events as gameplay rules.

The current battle result wireframe is a mock direction, not a requirement to implement every visible metric now. This goal's first implementation must target only statistics that are practically derivable from the current core event log and final result DTOs. Metrics that require new battle events, new mission result fields, balance policy, or unclear attribution must be deferred instead of guessed.

## Confirmed Direction

- `BattleEventLog` is the canonical source for battle-time facts.
- Core computes battle result statistics at battle end by reading the completed event log and final battle result state.
- Runtime does not maintain a second live mutable stats table as source of truth.
- Any future live HUD stats must be a non-canonical derived cache, not the battle result SoT.
- Unity consumes typed result-stat DTOs and performs presentation-only sorting, layout, animation, and icon selection.
- `compressed_event_log` remains replay/debug/detail material. It is not the primary Unity result-summary contract.

## Confirmed First-Slice Policies

Confirmed by the user on 2026-06-23:

- `damage_dealt` and `damage_taken` use actual HP delta, not theoretical damage. If a target has 10 HP and receives a 100-damage hit, the stat records 10.
- Basic attack count uses `AttackResolve`, not `AttackStart`, so only attacks that reached the resolution step are counted.
- Skill use count uses `AbilityCast`, not `ManualCastStart` / `AutoCastStart`, so cancelled or interrupted casts before ability execution are not counted as successful uses.
- MVP scoring should be data-driven through RON/static data rather than hard-coded in battle result aggregation.
- First-slice attribution counts only damage where `source_instance_id` maps directly to an employee unit. Summon/minion, DOT/buff ownership, reflected damage, defense-object damage, and environment attribution are deferred.
- Deployment time is the sum of intervals from `UnitDeployed` to the next `UnitWithdrawn`, `UnitDied`, or `BattleEnd`. Redeployment creates another interval and intervals are summed.
- Incapacitated employees can still be MVP candidates.

## Source Of Truth Order

1. Runtime battle code and event emission:
   - `src/game/battle/event_log.rs`
   - `src/game/battle/core/commands.rs`
   - `src/game/battle/core/sim.rs`
   - `src/game/battle/types.rs`
2. Combat result and active-node state:
   - `src/game/resources/selection.rs`
   - `src/game/world/combat.rs`
   - `src/game/world/state.rs`
3. Unity-facing snapshot/command/result DTO contracts:
   - `src/game/behavior.rs`
   - `src/game/world/snapshot.rs`
   - game server snapshot wrapping/compression code
4. Live RON/data and reward/growth policies.
5. Current policy documents and this goal.

Do not trust documentation alone. Re-read runtime event payloads and the current Unity-facing snapshot shape before editing.

## Current Runtime Facts To Verify Before Editing

- `BattleEventLog` is append-only and stores `BattleEventLogEntry { time_ms, seq, cause, source_command_id, event }`.
- `BattleLogEvent::HpChanged` currently carries:
  - `source_instance_id`
  - `target_instance_id`
  - `delta`
  - `hp_before`
  - `hp_after`
  - `reason`
  - optional damage metadata such as `damage_source`, `damage_type`, `raw_damage`, `final_damage`, `damage_breakdown`, `critical`, and `feedback_tags`.
- `BattleLogEvent::UnitDied` currently carries:
  - `unit_instance_id`
  - `owner`
  - `killer_instance_id`
  - `world_position`
  - projected tile `position`.
- `BattleLogEvent::BattleEnd` carries the winner.
- `ParticipantBattleResult` currently carries final participant state: `unit_instance_id`, `owned_uuid`, `side`, `survived`, `final_hp`, `max_hp`, and `became_incapacitated`.
- `CombatBattleState` currently stores combat identity, `winner`, `event_log`, `reward_mode`, rewards, and `participant_results`.
- `NodeOutcomeSummary` / `CombatOutcomeSummary` currently summarize combat completion but do not yet expose detailed MVP/per-employee battle stats.

If these facts are stale, update this document and base the implementation on the current code.

## Proposed DTO Contract

Introduce a typed core DTO for battle result statistics. Exact names can be adjusted to match local style, but the meaning should stay explicit.

```rust
pub struct BattleResultStatsDto {
    pub battle: BattleResultBattleStatsDto,
    pub employees: Vec<BattleResultEmployeeStatsDto>,
    pub mvp: Option<BattleResultMvpDto>,
}

pub struct BattleResultBattleStatsDto {
    pub duration_ms: u64,
    pub waves_cleared: Option<u32>,
    pub total_waves: Option<u32>,
    pub killed_enemy_count: u32,
    pub facility_damage: Option<u32>,
    pub incapacitated_employee_count: u32,
}

pub struct BattleResultEmployeeStatsDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub survived: bool,
    pub became_incapacitated: bool,
    pub final_hp: u32,
    pub max_hp: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub kill_count: u32,
    pub skill_use_count: u32,
    pub deployed_time_ms: u64,
    pub gained_xp: Option<u32>,
    pub trauma_delta: Option<i32>,
}

pub struct BattleResultMvpDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub score: i64,
    pub title: String,
    pub highlighted_metrics: Vec<BattleResultMetricHighlightDto>,
}
```

Initial implementation may omit fields that cannot be computed from existing canonical data, but omitted fields must be consciously documented. Do not fill missing values with guessed zeroes if zero is semantically different from unknown.

## Initial Collection Scope

Implement only the practical first slice supported by current runtime data. Do not attempt to recreate every field shown in the mock battle result screen.

Implement the first useful result-screen set:

- Battle-wide:
  - `duration_ms`
  - `winner`
  - `killed_enemy_count`
- Per employee:
  - `damage_dealt`
  - `damage_taken`
  - `kill_count`
  - `basic_attack_count`
  - `skill_cast_count`
  - `deployed_count`
  - `withdrawn_count`
  - `survived`
  - `became_incapacitated`
  - `final_hp`
  - `max_hp`
  - `deployed_time_ms` if deploy/withdraw/death/battle-end events are sufficient after code inspection.
- MVP:
  - deterministic MVP selection from implemented employee metrics.
  - title/highlight based on the winning metric.
  - scoring weights/rules loaded from RON/static data.

Defer, unless already available with clear SoT:

- healing done
- shield/prevented damage
- support contribution
- leak prevention percentage
- facility damage percentage
- wave timeline achievements
- reward list rendering data
- gained XP and trauma delta if current result completion emits them only after `CompleteCombatResult`.

Explicitly out of first-slice scope:

- every decorative/stat block shown in the mock wireframe.
- wave progress UI unless a clean wave-start/wave-cleared source exists.
- facility damage UI unless the defense-object/objective attribution and percentage rule are confirmed.
- advanced medals that require policy beyond highest damage, most kills, or simple survival contribution.
- reward/growth details inside the battle-stat DTO unless the implementation intentionally composes them from `CombatRewardsGranted` / `NodeOutcomeSummary`.

## Aggregation Rules

Create a core aggregation module, for example:

- `src/game/battle/result_stats.rs`, or
- another nearby module if existing battle result/resource organization suggests a better home.

The collector should consume immutable inputs:

- completed `BattleEventLog`
- `participant_results`
- final combat identity/context needed for wave/mission stats
- optional post-completion outcome data only when those fields are actually part of the final DTO.

Rules:

- Damage dealt:
  - sum positive damage to opponent targets from `HpChanged` where `source_instance_id` resolves to a player employee unit.
  - use applied damage from `final_damage` or `hp_before - hp_after`; prefer the existing rulebook/current code policy after inspection.
- Damage taken:
  - sum damage received by player employee units from `HpChanged`.
- Kills:
  - prefer `UnitDied.killer_instance_id` when it resolves to a player employee unit.
  - cross-check against lethal `HpChanged` where possible in tests.
- Battle duration:
  - use `BattleEnd.time_ms`.
- Incapacitation/final HP:
  - use `ParticipantBattleResult`.
- Deployment time:
  - derive from deploy/withdraw/death/end events only if event payloads provide a clean lifecycle contract.
  - If lifecycle events are insufficient, complete this goal with a policy/contract report instead of guessing.
- MVP:
  - use deterministic score from implemented metrics only.
  - load score weights/rules from RON/static data instead of hard-coding them in the collector.
  - keep incapacitated employees eligible for MVP.
  - if the MVP RON/data contract itself needs design beyond simple metric weights and tie-breakers, complete the goal with a policy report.

## Plan

1. Re-read battle event definitions and all current emitters for `HpChanged`, `UnitDied`, deployment, withdrawal, skill cast, and battle end.
2. Re-read combat result snapshot/result construction and identify the exact DTO boundary where stats should be exposed.
3. Write down in `EXPERIMENT_NOTES.md` which desired stats are currently computable from canonical data and which need new event/DTO policy.
4. Add focused tests for an event-log-only stats collector using synthetic or small battle event logs:
   - damage dealt attribution
   - damage taken attribution
   - kill attribution
   - final participant state merge
   - deterministic MVP selection
5. Implement the stats DTOs and collector in core.
6. Attach stats to the authoritative combat result state/snapshot boundary.
   - Prefer storing or deriving from `CombatBattleState` in one clear place.
   - Avoid server-only JSON mutation for gameplay meanings.
7. Add behavior/snapshot tests proving Unity-facing combat result snapshots expose the typed stats.
8. Add a focused live or integration test proving a real combat result can produce non-empty stats for at least one employee.
9. Update internal and Unity-facing contract documents after the runtime contract is implemented.
10. Run focused checks after small changes, then broader validation.
11. Review the implementation against `docs/goal_completion_review_guide.md` before marking complete.

## Completion Conditions

- Core exposes a typed battle result stats DTO for combat result UI.
- The DTO is derived from `BattleEventLog` plus final result state, not from a second live mutable stats SoT.
- Unity does not need to parse raw `compressed_event_log` to display MVP/per-employee result statistics.
- Damage dealt, damage taken, kill count, basic attack count, skill cast count, deploy/withdraw count, final participant state, and MVP selection are covered by focused tests.
- Unknown/unimplemented metrics are absent or explicit `Option` fields, not misleading zeroes.
- Mock-only metrics that are not backed by current runtime data are documented as deferred rather than implemented with placeholder values.
- No compatibility layer, fallback path, dual schema, or ignored legacy test is introduced.
- Any Unity-facing DTO shape change is documented in the relevant core/Unity contract docs.
- Any new policy decision discovered during implementation is handled by completing the goal with a policy question/report instead of guessing.
- Focused tests and a final broad validation command pass.

## Policy Completion Rule

If implementation reveals a policy decision not already covered, complete this goal and report the question list before continuing. This includes:

- MVP scoring RON schema/tiebreakers beyond simple metric weights.
- Whether summon/minion/facility damage is credited to an employee, source unit, or separate bucket.
- Whether damage over time, buffs, reflected damage, or environment damage counts toward employee damage dealt.
- Whether overheal/overkill/shield/prevented damage should be counted.
- Whether post-combat XP/trauma deltas belong in battle result stats or only in reward/growth outcome DTOs.
- Any serialized `BattleEventLog` version change.
- Any Unity-facing DTO shape or field naming change not already accepted by the user.

## Validation Commands

Use focused commands first, then broaden:

- `cargo test -p game_core battle::result_stats -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo check -p game_server`
- `cargo test -p game_core`

If server snapshot wrapping is touched, run a local server/probe flow and verify the combat result snapshot contains the typed stats.

## Completion Report Template

When complete, report:

- changed DTOs and files
- stats implemented
- stats intentionally deferred
- removed legacy/duplicate paths, if any
- new tests and contracts
- remaining risks
- validation commands run

## Completion Report

Status: implemented and verified on 2026-06-23.

Changed DTOs and files:

- Added `src/game/battle/result_stats.rs`.
- Exposed `battle::result_stats` from `src/game/battle/mod.rs`.
- Added `CombatBattleState.result_stats`.
- Added `SelectedEventSnapshotDto::CombatBattle.result_stats`.
- Added `BattleRecordExport.result_stats`.
- Added `battle_result_stats.mvp` policy to `RunPolicyData` and `../game_resources/data/run/policy.ron`.
- Updated server combat-result snapshot test to require `selected_event.result_stats`.
- Updated core and Unity-facing contract docs for `selected_event.result_stats`.

Stats implemented:

- battle `duration_ms`
- battle `winner`
- battle `killed_enemy_count`
- per-employee `damage_dealt` using actual HP delta
- per-employee `damage_taken` using actual HP delta
- per-employee `kill_count`
- per-employee `basic_attack_count` from `AttackResolve`
- per-employee `skill_cast_count` from `AbilityCast`
- per-employee `deployed_count`
- per-employee `withdrawn_count`
- per-employee `deployed_time_ms` from `UnitDeployed` to `UnitWithdrawn` / `UnitDied` / `BattleEnd`
- per-employee `survived`, `became_incapacitated`, `final_hp`, `max_hp` from `ParticipantBattleResult`
- MVP candidate and highlight from RON/static metric weights and tie-breakers

Stats intentionally deferred:

- wave clear timeline
- leak prevention
- facility damage percentage
- healing done
- shield/prevented damage
- advanced medals
- summon/minion/DOT/buff/reflection/environment attribution
- reward/growth details inside the battle-stat DTO

Removed legacy/duplicate paths:

- No new live mutable stats table was introduced.
- Unity no longer needs to parse raw/compressed event log for first-slice result stats.

New tests and contracts:

- `battle::result_stats` focused collector tests cover actual HP delta, damage taken, participant merge, deploy/withdraw counts, deployed time, kill count, basic attack count, skill cast count, and incapacitated MVP eligibility.
- `collector_excludes_player_side_defense_objects_from_employee_stats` covers the live-probe-discovered filtering rule that player-side defense objects are not employee result rows.
- `combat_result_snapshot_exposes_typed_result_stats` covers the Unity-facing selected-event DTO.
- Server combat result snapshot test now requires `selected_event.result_stats` alongside `compressed_event_log`.
- `docs/game_rulebook.md`, `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`, and `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md` document the new contract.

Remaining risks:

- MVP metric weights are initial data values and may need balance tuning after UI/gameplay review.
- Deferred attribution for summon/minion/DOT/buff/reflection/environment damage remains explicit follow-up work.
- Reward/growth data still comes from `CombatRewardsGranted` / `NodeOutcomeSummary`, not `result_stats`.

Validation commands run:

- `cargo fmt`
- `cargo test -p game_core battle::result_stats -- --nocapture`
- `cargo test -p game_core combat_result_snapshot_exposes_typed_result_stats -- --nocapture`
- `cargo test -p game_core game::data::run_policy_data -- --nocapture`
- `cargo check -p game_core`
- `cargo check -p game_server`
- `cargo test -p game_core --test ron_loading -- --nocapture`
- `cargo test -p game_core game::world::tests::combat -- --nocapture`
- `cargo test -p game_core game::world::tests::snapshots_and_start -- --nocapture`
- `cargo test -p game_server finished_live_battle_tick_pushes_combat_result_snapshot_after_final_update -- --nocapture`
- `cargo test -p game_core`
- `PYTHONPYCACHEPREFIX=/tmp python3 -m py_compile '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
- `APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run` from `/mnt/f/work/simulator/game_server`
- `PYTHONPYCACHEPREFIX=/tmp WS_PORT=18083 WS_CHECK_COMBAT_RESULT=1 WS_BATTLE_END_TIMEOUT=180 python3 '/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py'`
- Re-ran the same live probe after the defense-object filtering fix and confirmed concrete stats: `result_stats_employee_count: 2`, `damage_dealt: 180`, `kill_count: 2`, `basic_attack_count: 18`, MVP title `"Most Kills"`.
