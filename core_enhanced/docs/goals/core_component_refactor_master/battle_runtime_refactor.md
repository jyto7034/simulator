# 전투 런타임 시스템 Refactor Audit

## Scope

- 기준 문서: `docs/refactor_preparation_plan.md`
- Runtime code:
  - `src/game/battle/core/mod.rs`
  - `src/game/battle/core/sim.rs`
  - `src/game/battle/core/build.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/core/types.rs`
  - `src/game/battle/timeline.rs`
  - `src/game/battle/damage.rs`
  - `src/game/world/combat.rs`
  - `src/game/world/state.rs`
  - `src/game/behavior.rs`
- Live RON/data touchpoints:
  - `../game_resources/data/pve/encounters.ron`
  - `../game_resources/data/map/battlefield_templates.ron`
  - `../game_resources/data/map/battlefield_archetypes.ron`
  - `../game_resources/data/skills/base.ron`
  - `../game_resources/data/buffs/base.ron`
  - `../game_resources/data/abnormalities/base.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
- Unity/server-facing contract:
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
  - `../game_server/src/game/player_game_actor/state.rs`
- Tests:
  - `src/game/world/tests/combat.rs`
  - `src/game/battle/core/*` module tests
  - `tests/skill_refactor_validation.rs`
  - `tests/live_item_skill_activation.rs`
  - `tests/ron_loading.rs`

## Current Structure

- `src/game/battle/core/mod.rs:43`의 `BattleCore`가 battle event queue, scenario runtime, runtime units/items/artifacts/buffs/projectiles/areas/movement state, battlefield, live signals, timeline, deterministic ids/seed를 소유한다.
- `src/game/battle/core/sim.rs:53`의 `BattleExecutionState`가 현재 execution cursor, start hook 실행 여부, finished/finalized 여부를 가진다.
- `src/game/battle/core/sim.rs:92`의 `BattleLiveCommand`는 live battle 중 외부 command가 core runtime에 적용되는 내부 command 표면이다.
- `src/game/battle/core/sim.rs:1344`의 `apply_live_command`는 deploy/withdraw/manual skill activation을 `BattleCore`에 적용하고 `BattleLiveCommandOutcome`을 반환한다.
- `src/game/battle/core/sim.rs:1376`의 `apply_live_command_with_source_command_id`는 command id를 timeline entry의 `source_command_id`로 기록하기 위해 command 적용을 감싼다.
- `src/game/battle/core/sim.rs:1445` 이후의 step loop가 `event_queue`에서 같은 time bucket을 꺼내 `process_event`를 실행하고, winner를 계산해 finish signal을 만든다.
- `src/game/battle/timeline.rs:121`은 `Timeline`을 append-only battle event log라고 설명한다. 역사적 이름은 Timeline이지만 현재 DefenseRoute live battle에서는 offline replay driver가 아니라 live `events_delta`, battle result record, debug export의 event log다.
- `src/game/world/state.rs:118`의 `ActiveBattleSession`이 `BattleCore`, `BattleExecutionState`, Unity delivery cursor인 `last_pushed_timeline_seq`, deployment economy state, playback state를 묶는다.
- `src/game/world/combat.rs:621`의 `advance_active_battle_by`와 `:668`의 `advance_active_battle_for_server_tick`이 server tick/playback을 core execution delta로 바꾼 뒤 `BattleAdvanced` result를 만든다.
- `src/game/world/state.rs:441`의 `drain_battle_update_dto`가 `last_pushed_timeline_seq` 이후 timeline entries를 `events_delta`로 만들고 cursor를 전진시킨다.
- `src/game/world/state.rs:461`의 `battle_update_dto_after`는 특정 seq 이후 catch-up update를 만들지만 `last_pushed_timeline_seq`를 전진시키지 않는다.
- `src/game/behavior.rs:535`의 `LiveBattleUpdateDto`는 Unity-facing battle update DTO이며, `:1010` 이후의 `BehaviorResult` battle variants는 core 내부 command 결과에 battle update를 담는다.

Unity/server 계약은 이 구조 위에 한 단계 더 있다. `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:31-37`은 event/checkpoint/command_result의 책임을 분리하고, `:287-297`은 `events_delta`가 연출 source이고 checkpoint가 현재 상태 source이며 초기 정책은 `checkpoint.at_seq == events_delta.to_seq`라고 고정한다. `../game_server/src/game/player_game_actor/state.rs:224-236`은 core의 battle `BehaviorResult`를 Unity-facing `CommandAccepted` response와 side `battle_update`/`battle_setup_snapshot` message로 변환한다.

## Source Of Truth Judgment

- Battle runtime simulation의 source of truth는 `BattleCore`의 runtime state와 `BattleExecutionState`다.
- Live presentation event source of truth는 `BattleCore.timeline` append-only entries다.
- Unity current-state source of truth는 `LiveBattleUpdateDto.checkpoint`다. checkpoint는 timeline events를 재생한 뒤 reconcile할 권위 상태다.
- Live battle transport source of truth는 `core_unity_battle_transport_contract.md`와 game_server mapping이다. Core 내부 `BehaviorResult::BattleUnitDeployed` 같은 상세 variant는 Unity에 그대로 나가지 않는다.
- Deployment economy source of truth는 `ActiveBattleSession.live_deployment`이다. 단, deployed unit의 생사/위치 source는 `BattleCore.units`와 `BattleCore.battlefield`다.
- Playback source of truth는 `ActiveBattleSession.playback`과 `playback_delta_remainder`다. `BattleCore` 자체는 pause/speed를 알지 않고 scaled delta만 받는다.
- Live RON/data는 직접 tick loop가 읽는 source가 아니라 `GameDataBase`, combat preview, scenario, skill/buff/profile lookup을 통해 runtime으로 들어온다.

## Refactor Candidates

### 1. `LiveBattleDeploymentState`와 `BattleCore` unit state의 sync boundary 강화

근거:

- `ActiveBattleSession.live_deployment.deployed_units`는 employee uuid에서 `BattleCore`의 `UnitInstanceId`로 가는 wrapper state다.
- `src/game/world/state.rs:258`의 `reconcile_live_deployment_with_battle_state`는 battle unit이 사라지거나 죽은 경우 deployed map에서 제거하고 redeploy lock을 만든다.
- `src/game/world/combat.rs:934`, `:1112`, `:1162`, `src/game/world/state.rs:441` 등 여러 path가 battle update 생성 전에 reconcile 또는 deployment signal 적용을 호출해야 한다.
- `src/game/world/combat.rs:1076-1094`의 withdraw path는 `deployed_units.remove(&employee_uuid)`를 먼저 수행한 뒤 `BattleCore::apply_live_command_with_source_command_id`를 호출한다. core command가 실패하면 wrapper deployment map은 이미 변할 수 있다.

판단:

- `live_deployment`는 순수 중복은 아니다. 배치 코스트, 재배치 쿨다운, employee_uuid mapping은 BattleCore보다 world/session wrapper에 가까운 상태다.
- 그러나 deployed unit membership은 BattleCore unit 존재와 함께 유지되어야 하므로 현재 구조는 drift risk가 있다.
- 우선순위 높은 리팩토링은 deploy/withdraw/death reconciliation을 `ActiveBattleSession`의 작은 transaction helper로 모으는 것이다. 예: `withdraw_live_unit(employee_uuid, source_command_id)`가 core success 이후에만 deployment map을 변경하고, 실패 시 wrapper state를 보존한다.
- 추가로 `live_deployment_dto`의 panic path는 runtime invariant를 강하게 가정한다. public DTO 생성 전 invariant check나 `GameError::InvalidAction`/`InvalidStaticData` 변환을 검토할 수 있다.

필요 검증:

- 실패하는 withdraw/deploy core command가 wrapper deployment state를 변형하지 않는 focused test.
- 죽은 deployed unit이 한 번만 redeploy lock으로 이동하고, checkpoint와 `live_deployment`가 같은 membership을 보여주는 flow test.

### 2. Withdraw event source-of-truth 정리

근거:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:577-584`는 event candidates에 `UnitWithdrawn`을 포함한다.
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:804-806`은 철수한 직원은 즉시 사라지고, 실제 철수 사건과 현재 redeploy state는 뒤따르는 `battle_update.events_delta`와 `battle_update.checkpoint.deployment`에서 읽는다고 설명한다.
- `src/game/battle/core/build.rs:289`의 `withdraw_unit`은 unit 제거, battlefield 제거, items 제거, movement segment 제거, target clear, block state reset을 수행하지만 timeline event를 기록하지 않는다.
- `src/game/world/tests/combat.rs:1922` 이후의 withdraw test는 `BattleUnitWithdrawn` result와 checkpoint deployment state를 검증하지만 `events_delta`의 withdraw event를 검증하지 않는다.

판단:

- 현재 Unity가 checkpoint만으로 철수 후 현재 상태를 알 수는 있다.
- 하지만 계약 문구가 "철수 사건"도 events_delta에서 읽는다고 되어 있으므로, event stream timing source와 checkpoint current-state source 사이가 약간 어긋나 있다.
- `TimelineEvent::UnitWithdrawn`을 추가하는 것은 Unity-facing event DTO shape 변경이다. 단순 내부 리팩토링이 아니라 계약 변경이므로 `사용자와 정책 논의 필요`.
- 정책을 바꾸지 않는다면 문서 쪽을 "withdraw는 command ack + checkpoint deployment state가 source, 별도 event 없음"으로 고쳐야 한다. 이것도 계약 의미 변경이므로 `사용자와 정책 논의 필요`.

### 3. Core battle `BehaviorResult`와 server transport mapping의 dual surface 정리

근거:

- Core는 `BehaviorResult::BattleAdvanced`, `BattlePlaybackChanged`, `BattleUnitDeployed`, `BattleUnitWithdrawn`, `BattleSkillActivated`에 `battle_update`를 담아 반환한다.
- `../game_server/src/game/player_game_actor/state.rs:254-264`는 이 결과들에서 `battle_update`를 추출한다.
- `../game_server/src/game/player_game_actor/state.rs:224-236`은 Unity-facing response를 `CommandAccepted`로 바꾸고 side message로 `battle_update`를 보낸다.
- 같은 파일 `:640-649`는 battle behavior result를 일반 command payload로 보내려 하면 `battle_update_transport_required` 오류를 반환한다.
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:470-504`는 battle command result가 상태 변경 source가 아니라고 고정한다.

판단:

- 현재 server adapter가 Unity 계약을 보호하고 있으므로 runtime 동작은 명확하다.
- 다만 core 내부 result 이름이 "BattleUnitDeployed command result payload"처럼 보이고 world tests도 그 variant를 직접 assert한다. 장기적으로는 core battle command 결과를 `BattleTransportOutcome { battle_update, optional_setup, finished, accepted_at }` 같은 내부 타입으로 좁혀, Unity-facing `BehaviorResult`와 transport side message mapping을 덜 헷갈리게 할 수 있다.
- 이 변경은 command result DTO와 server mapping을 건드리므로 15번 Unity/server-facing DTO/계약 표면과 함께 다뤄야 한다.

정책 영향:

- Unity-facing result/side-message shape를 바꾸는 변경은 `사용자와 정책 논의 필요`.

### 4. `last_pushed_timeline_seq` delivery cursor invariant 고정

근거:

- `src/game/world/state.rs:441`의 `drain_battle_update_dto`는 cursor 이후 event를 가져오고, events가 비어 있지 않을 때만 `last_pushed_timeline_seq`를 갱신한다.
- `src/game/world/state.rs:461`의 `battle_update_dto_after`는 catch-up용 update를 만들지만 cursor를 갱신하지 않는다.
- `src/game/world/tests/combat.rs:782`의 `battle_setup_snapshot_live_defense_state_request_returns_battle_update_without_advancing_cursor`는 state request가 cursor를 전진시키지 않는다는 계약을 검증한다.
- Unity 계약은 `events_delta.after_seq`, `to_seq`, `checkpoint.at_seq`의 관계를 강하게 고정한다.

판단:

- 이 구조는 source-of-truth 중복이라기보다 "push cursor"와 "resync query"라는 서로 다른 책임이다.
- 그래도 event delivery는 Unity-visible 핵심 계약이므로 `drain`/`after` 두 path의 차이를 타입 이름으로 더 선명하게 만들 수 있다. 예: `drain_next_push_update`와 `build_resync_update_after`.
- `checkpoint.at_seq == events_delta.to_seq` invariant는 tests가 일부 고정하지만, DTO 생성 helper 단위에서 더 직접적인 invariant test를 두는 편이 좋다.

### 5. Playback 정책은 `ActiveBattleSession`에 있고 simulation은 scaled delta만 받는 구조 유지

근거:

- `src/game/world/state.rs:194`의 `simulation_delta_for_tick`이 pause/speed/remainder를 처리한다.
- `src/game/world/combat.rs:668`의 server tick path는 이 helper가 `None`을 반환하면 update를 보내지 않는다.
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md:910-912`도 pause/speed 성공 후 current playback state는 checkpoint에서 읽고, 정지 중에는 서버가 빈 battle_update를 계속 보내지 않는다고 명시한다.

판단:

- 이 구조는 BattleCore에 transport/playback 정책을 섞지 않는 좋은 분리다.
- 리팩토링 대상은 아니다. 단, `pause/resume/set speed`가 timeline event를 만들지 않는다는 정책은 command_result ack + checkpoint playback으로 충분한지 계속 계약에 고정되어야 한다.

### 6. Hard-coded battle runtime constants/data ownership 점검

근거:

- `src/game/battle/core/sim.rs:51`의 `MAX_BATTLE_TIME_MS`는 60초로 hard-coded 되어 있다.
- `src/game/battle/core/movement/types.rs`에는 movement tick/data scale 같은 runtime constants가 있다.
- Deployment economy policy는 `RUN_SYSTEM_POLICY.live_deployment`에서 들어와 `LiveBattleDeploymentState`로 복사된다.

판단:

- `MAX_BATTLE_TIME_MS`가 실제 design/balance policy라면 data/policy source로 이동할 후보가 될 수 있다.
- 단, 현재 battle termination guard일 수도 있으므로 즉시 RON 이동을 제안하지 않는다. 밸런스/UX 의미가 있어 `사용자와 정책 논의 필요`.

## Existing Composition Simplification Candidates

- Battle command success 이후 Unity 상태 변경은 이미 `CommandAccepted` + `battle_update.events_delta/checkpoint` 조합으로 표현된다. Core 내부에서도 새 command result payload를 늘리기보다 이 조합을 유지하는 편이 맞다.
- Deploy command는 `UnitSpawned`와 `UnitDeployed`를 둘 다 기록한다. 계약상 `UnitSpawned`는 actor 생성, `UnitDeployed`는 command correlation/source_command_id용이므로 현재 조합을 유지한다.
- Playback command는 별도 timeline event 없이 checkpoint playback으로 표현된다. 이건 기존 checkpoint source-of-truth를 잘 쓰는 단순화라 유지한다.
- Request battle state/resync는 push cursor를 전진시키지 않는 query로 유지한다. 이 방식은 이미 `battle_update_dto_after`로 표현된다.

## Legacy, Fallback, Compatibility Removal Candidates

- `Timeline`이라는 타입 이름은 역사적이지만 `src/game/battle/timeline.rs:1-6`이 현재 의미를 battle event log로 명시하고 있다. 즉시 rename하면 serialized/client-facing 이름과 docs를 크게 건드리므로 보류한다.
- `BehaviorResult::BattleUnitDeployed`류 상세 core result는 server에서 `CommandAccepted`로 매핑되는 내부 surface다. 완전 제거는 15번 계약 표면과 함께 검토한다.
- `RecoverBattleSetupLoss`는 active battle을 폐기하고 node-confirm으로 돌아가는 현재 계약의 live recovery command다. legacy fallback으로 제거할 대상이 아니다.
- `battle_records` JSON export는 component 1에서 이미 별도 source-of-truth 후보로 기록했다. 전투 runtime 감사에서는 live gameplay source가 아니라 debug/archive artifact로 본다.

## Deferred Or Not Doing

- BattleCore와 ActiveBattleSession을 즉시 합치지 않는다. BattleCore는 deterministic simulation, ActiveBattleSession은 world/session/transport/playback/deployment wrapper로 책임이 다르다.
- Movement/backend 세부 리팩토링은 6번 이동/저지 시스템에서 다룬다.
- Skill runtime/projectile/area 세부 리팩토링은 7번 스킬/타겟팅/범위와 8번 기본 공격/투사체/판정에서 더 깊게 다룬다.
- Damage/stat/buff 세부 정책은 9번, 10번 컴포넌트에서 다룬다.

## Tests And Verification To Add When Refactoring

문서 감사 단계에서는 코드를 바꾸지 않았으므로 테스트를 실행하지 않았다. 실제 리팩토링 시 우선순위 검증은 아래와 같다.

- `cargo test -p game_core world::tests::combat`
- `cargo test -p game_core battle::core`
- `cargo test -p game_core --test skill_refactor_validation`
- `cargo test -p game_core --test live_item_skill_activation`
- `cargo test -p game_core --test ron_loading`
- `cargo test -p game_server game::player_game_actor`

추가하면 좋은 focused test:

- Withdraw core command 실패 시 `LiveBattleDeploymentState.deployed_units`가 rollback 없이 먼저 사라지지 않는지 검증.
- Withdraw 성공 후 계약 방향을 확정한다면 `UnitWithdrawn` event 또는 "event 없음, checkpoint만 source" 중 하나를 고정하는 test.
- `drain_battle_update_dto`와 `battle_update_dto_after`가 항상 `checkpoint.at_seq == events_delta.to_seq`를 유지하는 helper-level test.
- `CommandAccepted` mapping이 battle `BehaviorResult`를 일반 payload로 노출하지 않는 server test는 이미 있으므로, deploy/withdraw/skill/pause/speed 각각의 side `battle_update` 존재를 더 세분화해도 좋다.
- Paused tick이 `None`을 반환하고 빈 update를 만들지 않는 server-tick test.

## Policy Discussion Required

- `UnitWithdrawn` timeline event를 추가하거나, 문서에서 withdraw event requirement를 제거하는 결정: `사용자와 정책 논의 필요`.
- Core battle `BehaviorResult` shape와 game_server `CommandAccepted` mapping을 바꾸는 결정: `사용자와 정책 논의 필요`.
- `MAX_BATTLE_TIME_MS`, deployment economy, playback speed 같은 밸런스/UX 수치를 RON/policy data로 옮기는 결정: `사용자와 정책 논의 필요`.
- `Timeline` 타입/JSON 이름을 event log 명칭으로 rename하는 결정: `사용자와 정책 논의 필요`.
