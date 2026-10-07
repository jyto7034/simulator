# Boss Runtime v1 P00 — Current Tree Drift Audit

Status: complete

Last Updated: 2026-08-09

## Method

2026-08-08 readiness 결론을 현재 source/test body와 다시 대조한다. 동일 commit이라는 사실만으로 working-tree
동일성을 가정하지 않으며, 각 subsystem을 확인한 즉시 `confirmed`, `material_drift`, `superseded` 또는
`pending`으로 판정한다.

## P01 G0 lifecycle correctness

| Candidate | Current source evidence | Readiness comparison | Disposition |
| --- | --- | --- | --- |
| CRA-P06-C01 terminal/max-time | `battle/core/sim.rs::step_battle_execution_with_post_spawn_setup`는 next event가 cap보다 뒤면 `Finished(Draw)`를 반환한다. 반면 `step_battle_execution_until_with_post_spawn_setup`는 target을 cap으로 clamp한 뒤 next event가 target보다 뒤고 winner가 없으면 `Running`을 반환한다. `BattleExecutionState`는 `finished/finalized` boolean만 소유하고 finalization은 caller signal을 받는다. | readiness counterexample과 동일 | `confirmed_prerequisite`; P01 첫 milestone 유지 |
| CRA-P06-C02 total event order | `battle/enums.rs::BattleEvent::cmp`는 time/priority와 일부 payload만 비교하고 마지막 `_ => Ordering::Equal`을 사용한다. Priority 8의 `AutoCastEnd`/`ManualCastEnd`, 서로 다른 `ManualCastEnd`, cause 또는 비교에서 누락된 payload가 다른 event도 `Equal`이 가능하다. Current test는 선택 variant heap 순서 한 건뿐이다. | readiness 결론과 동일; `FieldWorkCompletion`도 자체 key는 있으나 전체 totality를 만들지는 않음 | `confirmed_prerequisite`; current priority table을 보존하며 모든 variant semantic key 필요 |
| CRA-P06-C03 atomic world advance | `world/combat.rs::advance_active_battle_by_internal`과 `advance_active_battle_to_next_event_bucket`은 Core step/finalize 후 VN trigger, reconcile, revision commit, DTO cursor validation과 result transition을 수행한다. `active_battle_advanced_result`는 revision을 commit한 뒤 `battle_update_dto_after`를 호출하며 future `after_seq`가 여기서 실패할 수 있다. | readiness failure-after-mutation 경로와 동일 | `confirmed_prerequisite`; 두 entry point를 하나의 rollback-safe pipeline으로 수렴 |

## P01 dependency and implementation constraints

- 세 candidate 모두 현재 코드에서 이미 해결되지 않았으며 Boss scheduler 이전 prerequisite라는 순서는 유지된다.
- C01과 C02는 `sim.rs`/`enums.rs`의 Core boundary, C03은 `world/combat.rs`와 `world/state.rs`의 active-session
  boundary다. Boss type/catalog/runtime을 이 Phase에 섞지 않는다.
- `BattleCore`와 `ActiveBattleSession`은 현재 full `Clone` transaction seam을 제공하지 않는다. C03은 단순히
  existing clone을 호출하는 handoff로 작성할 수 없으며, preflight 가능한 failure를 앞당기고 남은 fallible
  state를 명시적으로 stage/rollback하는 설계가 필요하다.
- Existing dirty changes가 세 owner 모두에 있으므로 P01은 current body를 baseline으로 test-first, symbol-level
  patch를 사용한다.

## Remaining subsystem audit

| Surface | Status |
| --- | --- |
| data source inventory, publication and typed setup | confirmed below |
| Boss opt-in and runtime storage seams | confirmed below |
| movement/target/attack/skill preserved leaves | confirmed below |
| server DTO/public checkpoint and Unity consumer | confirmed below |
| active-battle deployment/migration lifetime | confirmed below |

## Data publication and typed setup

- `GameplayRonSourceSet::embedded()` remains the source authority: one manifest plus 22 non-manifest files.
  `parse_gameplay_ron_sources` parses the typed set, `ParsedGameplayRon::validate_domain_catalogs` performs strict
  domain/cross-reference validation, and `GameDataBase::load_live_embedded` publishes one `Arc<GameDataBase>` only
  after generated-preview validation. There is no filesystem hot reload, candidate generation or LKG publisher.
- No Boss Runtime catalog/source/type is present. Repository-wide current-source searches found no
  `BossEncounterRuntime`, `BossActorBehaviorRuntime`, `TrackAndRejoin`, `RejoinForwardRoute` or Boss-runtime RON.
- `PveEncounter` already distinguishes `NormalBoss`/`FinalBoss` content class, but has no optional Boss behavior
  definition reference or typed actor binding. Existing “boss” classification is not Boss Runtime opt-in.
- Current setup repeatedly resolves the encounter across preview/reward/start-plan/scenario assembly. World then
  manually constructs `ActiveBattleSession`; `CombatExecutor::build_battle_scenario_from_preview` builds groups/events
  and `BattleCore`, but no single resolved setup object carries an immutable Boss definition/frame.
- Publication validates generated previews, while setup capacity and final scenario feasibility can still fail in
  `enemy_spawn_groups_from_preview`/scenario construction at battle entry. CRA-P01-C02 and CRA-P05-C02 therefore
  remain official-live P05 gates.

Disposition: P02/P03 separation remains valid. P02 owns the separate strict catalog, compiled immutable definition,
optional encounter/actor binding and one typed setup result while coimplementing CRA-P01-C01/P05-C04. P03 consumes
that result and owns live runtime; it must not perform tick-time raw catalog lookup. P05 broadens the already-pure
setup builder over every manifested difficulty/seed and publishes the first official opt-in.

## Movement, targeting and leaf authority

- Ordinary opponent setup resolves authored routes into per-unit `EnemyMovementPlan::PathAlongCells`; planner falls
  back to the scenario plan only when a unit has no plan. This is the non-opt-in behavior to preserve.
- `build_continuous_attack_goals_from_current_block_state` owns ordinary attack/route goals. The movement engine
  already accepts an explicit goal map and emits typed `MovementOutput`, including terminal stop reasons, but no
  Boss intent owner consumes exactly one closed Task result.
- `movement/path.rs` provides forward route progress/next-waypoint utilities, not a static-grid path query. No A*/BFS
  or Boss path-cost owner exists. Route progress is recomputed from current position and does not maintain a
  monotonic Boss rejoin cursor.
- Existing `targeting.rs` and skill targeting own leaf attack/skill eligibility and deterministic fallback keys.
  There is no closed Boss selector/query schema, activation-owned target lock or immediate invalidation owner.

Disposition: readiness movement/target seam remains current. P03 adds an explicit Boss intent/query/result adapter
before ordinary planner fallback while preserving existing movement/blocking/attack/skill/effect/projectile/damage/
death authorities; P04 proves the `TrackAndRejoin` slice and non-opt-in trace parity.

## Public producer, server and Unity consumer

- Core normal update, setup snapshot and resync projections remain typed DTOs. `game_server` passes resync as the
  nested Core update but manually re-enumerates the top-level fields for normal `BattleUpdate` and
  `BattleSetupSnapshot`; CRA-P10-C03 exact normalized parity is still a first-public-change gate.
- Current checkpoint has no Boss public phase/objective/duration lifecycle. `LiveBattleUnitCheckpointDto` has
  position/world position but no Core-authored facing field.
- Current Unity consumes raw JSON setup/update/resync, validates revision/seq/checkpoint continuity and defers known
  duration events. Events without a known duration marker are considered immediately presentation-ready; there is
  no stable Boss presentation-instance lifecycle registry, cancel/end-once owner or reconnect rebuild for such a
  lifecycle. Battlefield code reads optional `facing` and can fall back to `right`, which is not a Core-facing
  authority.
- Both simulator server files and relevant Unity consumer/test files are already dirty user-owned paths.

Disposition: P04's existing movement/damage/death event slice can remain Wire-neutral. P06 remains conditional and
must activate only if the approved live reference needs a semantic lifecycle/facing/telegraph not representable by
the existing stream; if activated, Core/server/Unity are one synchronized cutover.

## Deployment and migration lifetime

- `game_server` loads live data once at process startup and stores one shared `Arc<GameDataBase>` in `AppState` for
  player actors.
- Reconnect reads the same in-memory active session. `active_battle` is excluded from run checkpoint/save and the
  non-battle transaction clone explicitly refuses an active battle.
- No runtime live-data reload or in-place active-session generation swap exists.

Disposition: the Master's default “new binary/data generation starts new battles; existing active battles are not
converted” assumption matches current topology. No save reader or in-place Boss runtime migration is needed for
P01~P05. Operational session draining across process deployment is a rollout concern, not a gameplay schema choice;
raise a policy decision only if future deployment requirements demand preserving active battles across replacement.
