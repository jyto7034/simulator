# Boss Runtime Boundary Audit

Goal: `boss_runtime_implementation_readiness_audit`  
Audit date: 2026-08-08  
Verdict: `conditionally_ready`

## Executive finding

현재 Core에는 Boss Runtime, Boss 전용 RON catalog, encounter/actor behavior binding이 존재하지 않는다.
그러나 기존 BattleCore의 battle-time event queue, authored route, blocking, movement, attack/skill/effect/projectile,
append-only event log는 Boss Runtime이 leaf executor로 재사용할 수 있다.

바로 구현을 시작하기 전에 반드시 닫아야 할 correctness gate는 다음 세 가지다.

1. `BattleEvent`의 동일 timestamp 순서를 완전한 total order로 만든다.
2. world battle advance가 projection/후처리 실패 때 일부 Core 상태나 revision을 남기지 않도록 한다.
3. max-time/terminal outcome을 모든 advance API에서 한 번만, 같은 의미로 확정한다.

Boss 기반과 함께 만들어야 할 경계는 별도 Boss catalog, strict validation, typed setup binding, activation/task
identity, deterministic target/path query와 movement-intent seam이다. 일반 route enemy의 현재 planner fallback은
Boss opt-in이 없을 때 그대로 유지한다.

Unity는 Core보다 몇 차례 개발이 뒤처진 stale consumer다. 따라서 Unity가 아직 읽지 못하는 현재 Core 필드를
Core 계약의 누락으로 역추론하지 않는다. Unity 차이는 첫 공개 Boss presentation 계약을 도입할 때 수행할
downstream synchronization debt로 분류한다. 다만 현재 Core checkpoint에도 facing이 없고 기본 공격 취소의
명시적 public 의미 사건이 없다는 점은 Unity의 신선도와 무관한 producer-side 의미 정보 gap이다.

## Evidence scope

다음 current working-tree source를 직접 읽었다.

- Canonical: `game_rulebook.md` 전체 1~961줄, `core_runtime_contract.md`,
  `data_loading_contract.md`, `skill_target_contract.md`
- Target decision: `docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md`의 BP-01~BP-34
- Publication/data: `src/game/data/live_ron.rs`, `pve_data.rs`, live `game_resources/data/**`
- Setup/scenario: `src/game/combat_setup/**`, `src/game/battle/scenario.rs`
- Runtime: `src/game/battle/core/**`, `src/game/battle/{types,enums,event_log}.rs`
- World projection: `src/game/world/combat.rs`, `src/game/behavior.rs`
- Server: `game_server/src/game/player_game_actor/{messages,handlers,state,session}.rs`
- Unity stale baseline: `BattleRuntimeState`, `GameSnapshotStore`, battlefield actor/model/render and damage feedback
  sources 및 관련 EditMode tests under `/mnt/f/unity projects/ark`
- Prior refactor evidence: final candidate catalog와 P01/P05/P06/P07/P08/P09/P10 detailed findings
- Final-boss stress input: `docs/skills/abnormality_skill_design_notes.ko.md`

Source inspection으로 판정 가능한 조사 Goal이므로 runner는 실행하지 않았다. Boss target behavior는 아직
구현되어 있지 않아 green test가 readiness를 증명하지 못하고, current dirty tree의 broad runner는 unrelated
failure를 섞을 수 있다.

## Current end-to-end trace

| Boundary | Current producer and state | Current consumer | Finding |
| --- | --- | --- | --- |
| Live publication | `data/live_ron.rs`의 `LIVE_GAMEPLAY_RON_SOURCE_COUNT`, `GameplayRonSourceSet`, `ParsedGameplayRon`, `GameDataBaseParts`, `load_live_embedded` | `Arc<GameDataBase>`를 고정한 world/battle setup | 22개 source가 수동 inventory다. parse→cross-validation→DB construction은 startup에서 fail-closed하고 한 번에 publish되지만 Boss source/last-known-good hot-reload publisher는 없다. |
| Encounter opt-in | `data/pve_data.rs::PveEncounter` (`deny_unknown_fields`)와 live `encounters.ron` | combat preview/setup | Boss runtime reference가 없다. FinalBoss entries도 ordinary encounter이며 현재는 placeholder content다. |
| Preview/feasibility | `combat_preview/**`, `live_ron.rs::validate_generated_combat_preview_contracts` | setup selection/UI preview | baseline 난이도와 threat-warning 중심이다. 모든 manifest difficulty와 실제 spawn/field/tactical/setup feasibility를 publish 전에 보장하지 않는다. |
| Typed setup | `combat_setup/**`가 encounter, map, route, spawn을 반복 해석 | `battle/scenario.rs::BattleScenario`와 spawn groups | `ScenarioUnitSpawn`에는 behavior binding이 없고 `ScenarioSpawnGroup.enemy_movement_plan`만 optional이다. Boss actor reference를 start 전에 immutable typed plan으로 resolve할 seam이 필요하다. |
| Authored route | `combat_setup/enemy_spawns.rs`가 route ID를 cell path로 해석하고 `EnemyMovementPlan::PathAlongCells`를 설정 | movement planner | ordinary DefenseRoute enemy의 현재 authority이며 유지 대상이다. |
| Movement planning | `battle/core/movement/planner.rs` | movement engine/steering | 매 tick block/attack target/route를 조합한다. Boss intent owner가 없고 route fallback보다 앞서 적용할 explicit seam이 없다. |
| Route progress | `battle/core/movement/path.rs`의 nearest projection | planner | 거리 동률이면 더 큰 progress를 택하지만 actor가 monotonic cursor를 소유하지 않는다. route 이탈 후 뒤쪽 지점으로 복귀할 수 있다. |
| Spatial execution | `movement/{engine,steering,blocking,rapier_backend}.rs` | `BattleCore` unit state | static obstacle/void에서 direct point steering이 멈출 수 있으며 grid pathfinder는 없다. `MovementStopped` reason이 상위 typed task result로 전달되지 않는다. |
| Basic target | `battle/core/targeting.rs`와 `RuntimeUnit.current_target` | basic attack/movement planner | skill/basic 용도 결정론 primitive는 재사용 가능하지만 Boss Pattern lock이나 typed lexicographic query owner가 아니다. |
| Attack/skill leaf | `battle/core/mod.rs`, attack/skill/effect/projectile modules | scheduled `BattleEvent` | hard CC는 unreleased windup/cast를 epoch로 무효화하고 released projectile/delivery는 계속된다. BP-09의 기반은 있으나 state-owned action lease가 없다. |
| Same-time scheduling | `battle/enums.rs::BattleEvent`의 `priority`/`Ord`, `battle/core/mod.rs` drain loop | all battle subsystems | 같은 timestamp에 새 event를 다시 drain하는 microstep 기반은 있다. 그러나 서로 다른 variant가 `_ => Equal`이 될 수 있어 total order가 아니며 explicit microstep limit도 없다. |
| World advance | `world/combat.rs::advance_active_battle_by_internal`, next-event path, `active_battle_advanced_result` | Core execution, VN triggers, DTO projection, revision commit | mutation 뒤 fallible trigger/projection/cursor 단계가 있어 error가 partial mutation을 남길 수 있다. stage→validate→commit 전제에 부적합하다. |
| Terminal | BattleCore stop/max-time와 world finalization | battle result/update | API별 max-time/finished 판정이 갈리고 elapsed가 Running으로 남을 수 있다. Boss terminal/transition 우선순위 전에 통일해야 한다. |
| Event log | `battle/event_log.rs`, `behavior.rs::LiveBattleUpdate` | server and Unity | seq/time 기반 append-only delta, checkpoint와 revision은 유효한 기반이다. Boss 내부 graph를 wire로 보내지 않아도 된다. |
| Checkpoint | `behavior.rs` unit/public battle checkpoint | reconnect/resync consumer | 위치/HP/생존/전술/field state는 reconcile 가능하다. facing과 Boss public phase/objective/lifecycle 상태는 없다. Core-only active runtime은 in-memory session 안에 유지된다. |
| Server transport | `player_game_actor/messages.rs`, `handlers.rs` | WebSocket client | event/checkpoint는 대체로 Core DTO를 운반하지만 setup/update 일부 field를 server가 다시 조립한다. reconnect는 같은 battle UUID의 setup와 seq 0 이후 update를 다시 보낼 수 있다. |
| Unity replay | stale `BattleRuntimeState`/`GameSnapshotStore`와 presentation consumers | actors/VFX | seq 연속성, revision, checkpoint reconcile의 기반은 있으나 current Core 누적 변경과 동일 battle setup-loss reset 의미가 아직 모두 이식되지 않았다. downstream sync gate다. |

Publication 세부 근거:

- `live_ron.rs::GameplayRonSourceSet`은 manifest 외 22개 field를 수동 열거한다. Data canonical의 “manifest 외
  21개”는 current code와 어긋나는 문서 drift다.
- `GameDataBase::load_live_embedded`는 final generated-preview 검증 전 candidate를 호출자에게 반환하지 않는다.
  이 atomic startup seam은 재사용하되 current loader에는 hot reload generation/LKG owner가 없다.
- `validate_combat_preview_threat_warning_contract`는 encounter마다 floor 0, category Combat, 고정 5 seed의
  baseline preview/warning만 확인한다. 모든 live difficulty manifestation이나 resolved Boss binding을 검증하지
  않는다.

Setup 세부 근거:

- `CombatExecutor::build_battle_scenario_from_preview`가 encounter와 preview identity를 다시 검증하고 mission,
  defense object, wave group/event를 조립한다.
- `enemy_spawn_groups_from_preview`는 `SpawnWave.route_id`를 exact route cells로 resolve한다. 그 plan은 group
  단위이고 `spawn_scenario_group`에서 각 `RuntimeUnit.enemy_movement_plan`으로 clone된다.
- `BattleScenario`/`ScenarioUnitSpawn`에는 encounter/actor Boss binding이 없다. Publication-resolved immutable
  encounter definition, actor definition과 typed argument frame을 start 전에 보유할 typed seam이 필요하다.
- Current setup test는 preview→scenario route cells/objective/timed wave 변환을 실제 builder로 확인하므로
  non-opt-in parity safety net으로 유지한다.

Movement/target 세부 근거:

- Planner는 per-unit plan이 없으면 scenario route로 fallback한다. 이 exact branch 앞에 Boss opt-in actor intent
  admission을 두되 non-opt-in flow를 변경하지 않는다.
- Route progress는 current position의 nearest-segment projection이고 tie에서만 later progress를 고른다.
  BP-02에는 actor-owned monotonic forward cursor가 별도로 필요하다.
- Continuous movement engine의 `MovementStopReasonContinuous`는 apply 과정에서 Task result로 승격되지 않는다.
  Presentation segment cleanup과 authored Task result는 별도 책임으로 연결해야 한다.
- `RuntimeUnit.current_target`는 basic/block target authority다. Boss activation lock은 별도 state로 소유하고
  leaf attack admission 시 explicit target/lease로 전달해야 한다.

## Current and target authority

| Concern | Current authority | Target Boss authority | Required seam |
| --- | --- | --- | --- |
| Ordinary enemy route | Scenario movement plan + planner | unchanged | Boss opt-in이 없으면 기존 분기 100% 유지 |
| Boss phase/objective | 없음 | `BossEncounterRuntime` | encounter binding과 immutable runtime definition |
| Boss actor control | route/block/basic target가 분산 | `BossActorBehaviorRuntime` | planner 앞의 explicit Boss movement/action intent |
| Target selection | basic/skill subsystem별 query | activation-owned typed query/lock | snapshot query + stable ID final tie-break |
| Route return | nearest route projection | monotonic cursor 기반 `RejoinForwardRoute` | deterministic reachable path query와 closed result |
| Attack/skill correctness | 기존 typed leaf executor | 계속 기존 executor | action owner/lease와 activation epoch만 연결 |
| Transition | 없음 | graph arbitration + atomic commit | total event order, stage/validate/commit, stale no-op |
| Presentation | leaf event log/checkpoint | 같은 log + 필요한 public lifecycle만 | 내부 phase graph 노출 금지, stable presentation instance ID |

### Competing authority to avoid

- Boss task가 movement engine을 복제하거나 `RuntimeUnit.current_target`를 Boss graph의 영구 blackboard로 재사용하면
  basic attack retarget과 Pattern target lock이 서로 덮어쓴다.
- Boss route 복귀가 현재 nearest projection만 호출하면 승인된 forward-only Rejoin을 보장하지 못한다.
- Boss transition이 world command transaction과 하나의 거대한 transaction lifetime을 공유하면 서로 독립인
  tick 실행과 player command 실패 범위를 불필요하게 결합한다.
- Unity가 HP 변화나 위치를 보고 phase/target/interrupt를 추론하면 Core authority를 침범한다.

## Rulebook drift audit

`game_rulebook.md`를 처음부터 EOF까지 읽은 뒤 관련 runtime/live data/test와 대조했다.

| Rulebook contract | Current implementation | Boss target relation | Disposition |
| --- | --- | --- | --- |
| DefenseRoute가 전투 공간/보호 대상/route를 제공 | setup/scenario와 authored path가 구현 | Boss도 같은 battlefield에서 opt-in | 일치, ordinary route 보존 |
| 침식체는 authored route를 따라 전진하고 block 시 교전 | planner/blocking/basic attack가 구현 | Boss만 명시적으로 route 이탈 가능 | target extension, canonical 변경은 후속 구현 시 필요 |
| Tactical movement는 player command | 별도 world tactical command | Boss AI intent와 owner가 다름 | 충돌 없음, transaction primitive만 공유 가능 |
| hard CC는 미발사 attack/cast/movement를 취소하고 released delivery는 존속 | epoch/cast interrupt/projectile tests가 보호 | state exit도 같은 의미가 필요 | 기존 primitive 재사용 + action ownership 보완 |
| Pause 동안 simulation/battle time 정지 | battle-time advance가 권위 | deadline/cooldown도 battle time | 일치 |
| 일반 wave는 시간축 기반 | PVE wave/setup가 time based | Boss phase를 ordinary wave 조건으로 끼워 넣지 않음 | 분리 유지 |
| 수동 skill target은 Core 판정 | typed skill target contract/command | Boss query는 별도 AI policy | leaf validation 재사용 |
| active battle은 Core authority, Unity는 event replay/checkpoint reconcile | in-memory active session + delta/checkpoint | Boss 내부 runtime도 Core-only | 일치, public lifecycle만 projection |
| final boss는 특수 mechanic을 가진다 | live data는 ordinary encounter placeholder | concept notes로 kernel stress | current content gap이나 v1 blocker로 판정하지 않음 |
| Rulebook이 외부 Unity contract를 canonical로 링크 | 실제 Unity project/code/docs는 current Core/game_server보다 stale | Boss wire는 current Core semantic DTO를 기준으로 downstream sync | external consumer documentation freshness drift; Core 계약을 Unity에 맞춰 역추론하지 않음 |

Rulebook gameplay 의미와 실제 코드의 material conflict는 발견하지 않았다. 다만 외부 Unity 문서를 canonical로
분류한 링크는 consumer 신선도와 어긋나는 documentation drift다. Boss target model은 아직 canonical current
behavior가 아닌 승인된 future policy이므로, 구현 Goal에서 durable canonical을 함께 갱신해야 한다.

### Adjacent canonical versus target/code

| Canonical claim | Current/target comparison | Disposition |
| --- | --- | --- |
| Runtime mutation failure는 state/resource/event/revision을 바꾸지 않음 | command-time preflight 일부는 지키지만 live advance는 step/reconcile/revision commit 뒤 cursor/trigger/projection error가 날 수 있음 | code `not_satisfied`; staged advance/rollback이 Boss 전 prerequisite |
| Event seq와 checkpoint가 presentation/reconcile을 분리 | current Core primitive가 존재하고 BP-30도 같은 stream을 재사용 | reusable foundation |
| Embedded live bundle은 parse/cross-validation 뒤 한 번 publish | startup fail-closed primitive는 존재; hot reload와 failed update의 last-known-good retention은 없음 | Boss source를 같은 candidate bundle에 coimplement, hot reload 자체는 v1 비범위로 분리 |
| Basic/skill targeting은 deterministic이며 captured target/delivery lifecycle을 가짐 | Boss activation lock, typed lexicographic query와 path-cost/reachability owner는 없음 | leaf validation 재사용 + Boss query foundation 필요 |

위 표의 live advance 항목은 source 재검증으로 확정됐다. `advance_active_battle_*_after_seq`는 Core step 뒤 VN
trigger와 resource/reconcile/revision/DTO를 차례로 수행하며, future `after_seq` 검사는 revision commit 뒤에 있다.
따라서 `InvalidBattleResyncSeq`가 이미 변경된 battle time/event/resource/trigger/revision과 함께 반환될 수 있다.
Current tests의 future-seq rejection은 read-only state request만 다루고 advance rollback을 보장하지 않는다.
이 행의 status는 문서상 목표 불변식과 대비한 code `not_satisfied`이며 CRA-P06-C03을
`prerequisite_before_boss`로 유지한다.

## Movement and targeting readiness

### Reusable

- stable unit IDs, sorted iteration과 current basic/skill target validation
- continuous movement tick, collision/blocking, movement epoch
- hard-CC cancellation 및 released delivery continuation
- append-only movement/attack/skill/damage/death events

### Missing

- activation-owned target lock과 explicit re-evaluation decision point
- filter/order-key가 닫힌 typed query (`Min(PathCost)` 포함) 및 canonical final ID tie-break
- deterministic battlefield path-cost/path result
- actor-owned monotonic route cursor와 `RejoinForwardRoute`
- `Reached`, `TargetInvalidated`, `PathUnavailable`, `TimedOut`, `Interrupted` closed result 전달
- state-owned unreleased action을 정확히 취소하는 owner token/lease

현재 Rapier backend나 P09-C06 tile reach는 위 navigation 문제의 대체물이 아니다. 첫 slice는 existing static
cell/void/block constraints 위의 결정론적 cardinal grid query로 제한하고 dynamic nav, LOS, roaming target은
도입하지 않는다. `RepathToProtectedObjective`는 schema에서 선택되면 validation error로 거절한다.

## Scheduler and terminal prerequisite detail

- `BattleEvent::Ord`는 timestamp와 priority 뒤 일부 variant만 field 비교하고 나머지는 `Equal`로 처리한다.
  특히 같은 priority의 서로 다른 cast-end kind와 별도 manual cast-end event가 equal일 수 있으므로, container
  insertion order와 무관한 total order를 증명하지 못한다.
- Same-time 처리 중 enqueue된 같은 timestamp event를 다음 bucket에서 계속 drain하는 구조는 BP-34 microstep의
  좋은 기반이다. 다만 current source에는 timestamp별 microstep budget이나 zero-time cycle/livelock 종료가 없다.
- Direct step은 next event가 max time을 넘을 때 Draw로 finalize하지만 `step_battle_execution_until`은 같은
  경계에서 Running을 반환할 수 있다. Boss terminal/death 우선순위와 phase transition을 얹기 전에 모든
  advance entry point를 단일 terminal helper로 수렴해야 한다.
- Current `finalized` guard와 attack-resolve-before-end test는 유지한다. Acceptance는 distinct event total
  ordering, insertion permutation parity, microstep overflow no-partial-mutation, max-time API parity까지 넓힌다.

## Event, checkpoint and replay readiness

첫 추적형 slice는 기존 movement/attack/skill/damage/death event만으로 표현 가능하다. 따라서 내부
Phase/Pattern/State/Task나 activation ID를 public Wire에 노출할 필요가 없다.

향후 mechanic이 duration presentation을 요구할 때는 Core가 다음을 명시해야 한다.

- stable semantic unit/catalog ID
- presentation lifecycle의 `presentation_instance_id`
- start/update/end 또는 cancel/interrupt cause
- public phase/objective/telegraph 상태 중 reconnect 뒤에도 지속되는 값
- authoritative facing 또는 direction
- animation 우선순위를 바꾸는 unreleased attack cancel/skill interrupt/hard CC/forced movement/death 의미

Snapshot/checkpoint에는 최신 위치, 방향, 생존, HP, 방어/제어 상태, 지속 중인 public phase/objective와 활성
duration presentation만 둔다. 공격/피격/사망처럼 이미 종료한 일회성 cue는 checkpoint 복구 때 재생하지 않는다.
Movement는 checkpoint의 현재 위치/방향/이동 상태와 start/stop/interrupt 사건을 조합한다.

여기서 “checkpoint”는 두 층을 구분한다.

- Core authoritative graph는 same process의 `ActiveBattleSession`/`BattleCore`에만 남는다. Run checkpoint/save와
  retreat용 `BattleEntrySnapshot`은 active battle을 저장하지 않으므로 current scope의 save migration은 없다.
- `LiveBattleStateCheckpointDto`는 그 active session에서 만드는 public reconcile projection이다. Current DTO는
  active unit position/world position과 전술/제어 상태는 가지지만 facing 및 지속 중인 Boss public lifecycle은
  없다. 첫 Wire-changing slice가 필요로 하는 public phase/objective/duration/facing만 추가하며 내부 graph,
  target lock, epoch와 task queue는 추가하지 않는다.

seq는 중복/누락 방지, battle time은 ordering/playback, revision은 authoritative projection 세대를 구분한다.
같은 battle reconnect/setup-loss에서는 client가 actor와 pending presentation queue를 버리고 setup→checkpoint→
delta로 재구축해야 한다. 현재 stale Unity가 이 최신 계약을 완전히 구현하지 않은 것은 Core blocker가 아니라
첫 public Boss Wire cutover의 필수 consumer sync 항목이다.

`game_server` 경계의 증폭 규칙은 다음과 같다.

- `BattleResync`는 Core `LiveBattleUpdateDto`를 typed nested value로 보존한다.
- Normal `BattleUpdate`와 `BattleSetupSnapshot`은 top-level fields를 server enum으로 수동 재열거한다. Event delta,
  checkpoint와 setup catalog refs는 nested typed value 그대로지만 새 top-level field는 mapper 동시 수정 없이는
  누락될 수 있다.
- Existing tests는 start/setup/update/final ordering과 selected fields/cursors를 보호하지만 Core DTO 대 server
  envelope의 exact JSON parity는 증명하지 않는다.
- 따라서 첫 public Boss event/checkpoint/setup 변경은 normal update, resync와 setup의 동일 generation을 함께
  검사하고 exact normalized JSON parity test를 추가한다. Server는 Boss graph나 우선순위를 재계산하지 않는다.

Stale Unity consumer의 actual readiness는 transport 수신보다 좁다.

- Revision/seq gap/resync 및 `at_seq` 뒤 checkpoint reconcile, buffered clock과 movement end/stop timing은 source와
  representative tests가 있다.
- 실제 battlefield presentation은 spawn identity, movement interpolation과 damage feedback까지다. 감사 도중
  병행 작성된 `ArkClient.SpinePresentation`은 caller-driven semantic action/facing/interrupt/death playback과
  lifecycle primitive를 제공하지만 assembly가 `ArkClient.Runtime`/JSON/Wire를 의도적으로 참조하지 않고
  sandbox local input만 사용한다. 기존 battlefield/runtime assembly에서도 이를 참조하지 않는다. 따라서
  Attack/skill/projectile/area/death Spine action은 여전히 battle event에 연결되어 있지 않다. 일부 projectile/
  area 이름은 checkpoint delay marker일 뿐 VFX consumer가 아니다.
- Unknown Boss event는 raw envelope로 보존되지만 즉시 presented 처리된다. Duration telegraph/mechanic에는 stable
  instance ID, start/end/cancel/interrupt consumer와 active lifecycle checkpoint가 새로 필요하다.
- 같은 UUID의 setup snapshot은 runtime reset을 하지 않아 setup-loss 문서의 “local actor/queue 폐기 후 rebuild”를
  구현하지 않는다. 첫 reconnect-safe public Boss presentation 전에 existing consumer debt로 닫는다.
- Active-unit checkpoint에서 빠진 actor는 destroy되며 death presentation 완료 owner가 없다. Current Core가
  facing을 checkpoint에 보내지 않아 Unity optional parser는 right default를 사용한다.
- `ArkClient.SpinePresentation`은 future G3b adapter가 재사용할 수 있는 consumer-side primitive지만,
  `presentation_instance_id`, seq idempotence, checkpoint lifecycle rebuild와 battle-event semantic mapping을
  자체로 제공하지 않으므로 public Boss readiness gate를 닫지 않는다.

## Final-boss concept stress boundary

세 concept은 current 계약이나 v1 content requirement가 아니다. 오직 foundation의 확장성만 검사한다.

| Stress input | Foundation으로 수용할 부분 | Future separate owner / v1 제외 |
| --- | --- | --- |
| WhiteNight | encounter phase/objective, multi-actor binding, threshold event, mechanic adapter | run-local confession policy, apostle third-side control, terminal VN interception, resurrection/partial reset, exact zones/skills |
| Silent Orchestra | battle-time phase deadline, typed immunity/target condition, wave and phase-end event, encounter resource hook | exact damage axes, wave/pulse numbers, reward fragment behavior and public presentation |
| Apocalypse Bird | persistent objectives, irreversible mechanic disable, scheduled hazard cancellation, actor/encounter split | fog/visibility, large-map navigation, TacticalDeployable, camera/minimap, exact egg/power/pattern content |

세 항목 모두를 boss-specific enum으로 넣지 않고 finite Phase/Pattern/State/Task와 explicit typed mechanic adapter로
분해할 수 있다. 따라서 universal script VM은 필요하지 않다. 반대로 위 future owner를 v1이 이미 지원한다고
주장해서도 안 된다.

## Existing refactor correlation

| Candidate | Classification | Evidence and boundary decision |
| --- | --- | --- |
| CRA-P01-C01 | `coimplement_with_boss_foundation` | 수동 source count/inventory에 Boss catalog를 추가할 때 단일 source authority와 exact inventory test를 함께 만든다. |
| CRA-P01-C02 | `prerequisite_before_boss` | live Boss opt-in publication 전에 모든 manifested difficulty preview validation이 필요하다. isolated kernel 작성은 막지 않는다. |
| CRA-P05-C02 | `prerequisite_before_boss` | live Boss encounter publish 전에 spawn/setup/field/tactical/Boss ref feasibility를 완전 검증한다. |
| CRA-P05-C04 | `coimplement_with_boss_foundation` | 반복 lookup 대신 immutable typed setup plan에서 Boss encounter/actor binding을 resolve한다. |
| CRA-P06-C01 | `prerequisite_before_boss` | terminal/max-time semantics를 모든 advance API에서 통일한다. |
| CRA-P06-C02 | `prerequisite_before_boss` | Boss same-time transition보다 먼저 `BattleEvent` total order를 보장한다. |
| CRA-P06-C03 | `prerequisite_before_boss` | Boss stage/commit 전에 world advance partial mutation을 제거한다. |
| CRA-P07-C01 | `independent_existing_wave` | field/tactical final-state authority 수정은 Boss actor runtime owner와 별개다. |
| CRA-P07-C02 | `independent_existing_wave` | atomic mutation primitive/interface는 조율하되 player command와 Boss tick transaction을 합치지 않는다. |
| CRA-P07-C03 | `defer_until_after_vertical_slice` | movement integration cleanup은 새 Boss intent seam과 production parity가 정해진 뒤 수행한다. |
| CRA-P07-C04 | `defer_until_after_vertical_slice` | ghost movement field/output 제거가 typed TaskResult seam을 없애지 않도록 slice 뒤 재평가한다. |
| CRA-P08-C01 | `independent_existing_wave` | trigger occurrence bug는 별도 correctness wave이며 Boss kernel 구조의 전제는 아니다. |
| CRA-P08-C05 | `independent_existing_wave` | committed damage 중복 경로는 leaf subsystem 개선이며 Boss는 정식 leaf API만 호출한다. |
| CRA-P09-C03 | `independent_existing_wave` | multi-step delivery usefulness 정책은 Boss target query와 다른 gameplay 정책이므로 병합하거나 Boss gate로 삼지 않는다. |
| CRA-P09-C05 | `defer_until_after_vertical_slice` | projectile kernel 정리는 기존 released-delivery 계약을 유지한 채 후속 수행한다. |
| CRA-P09-C06 | `independent_existing_wave` | skill tile/footprint reach는 공유 geometry를 조율할 수 있지만 navigation path-cost 대체물이 아니다. |
| CRA-P10-C02 | `independent_existing_wave` | command-result projection 중복은 Boss runtime과 별도다. |
| CRA-P10-C03 | `defer_until_after_vertical_slice` | existing Wire만 쓰는 internal slice는 막지 않는다. 최초 public Boss setup/event/checkpoint 변경 시 exact JSON parity와 함께 수행한다. |
| CRA-P10-C04 | `independent_existing_wave` | host snapshot overlay typing은 Boss active runtime과 별도다. |

전체 51개 refactor catalog는 선행 조건이 아니다. 실제 필수 순서는 P06-C01/C02/C03 correctness, Boss
foundation, minimal slice이며 P01-C02/P05-C02는 live data opt-in 전 publication gate다.

## Test evidence disposition

| Existing test family | Disposition | Protected invariant / gap |
| --- | --- | --- |
| strict RON parse and PVE cross-reference tests | `retain` + `strengthen` | unknown/dangling reject는 유지; Boss source exact inventory, cross-catalog refs, unimplemented enum reject 추가 |
| authored route/block/movement tests | `retain` + `strengthen` | ordinary non-opt-in regression 유지; monotonic cursor/path tie/Rejoin result 추가 |
| hard CC, stale cast/attack epoch, released projectile tests | `retain` + `strengthen` | cancellation primitive 유지; state-owned action lease와 stale Boss activation 추가 |
| same-timestamp attack-before-end regression | `retain` + `strengthen` | 단일 사례 유지; 모든 variant total-order/permutation/microstep limit로 확대 |
| world setup loss/pause/revision overflow/tactical atomic tests | `retain` + `strengthen` | failure no-mutation을 Core state/event/revision/cooldown까지 확대 |
| server setup/reconnect/resync tests | `retain` + `strengthen` | same UUID/revision/seq는 유지; public Boss lifecycle 도입 시 exact transport/catch-up 추가 |
| stale Unity seq/revision/checkpoint/movement/damage tests | `unverified` for current Core parity | consumer sync 뒤 same-battle reset, lifecycle idempotence와 cue non-replay로 강화 |
| P07 movement parity/body tests | `merge` after slice | 새 movement intent invariant 이전 뒤 중복 fixture만 정리 |
| private/ghost state shape tests | `remove` only after invariant transfer | 현재 즉시 제거하지 않음 |

## Boundary verdict

- Current runtime preservation: 가능
- Boss kernel isolated implementation: P06 correctness 뒤 가능
- Live RON Boss opt-in: P01/P05 publication/setup validation 뒤 가능
- First internal tracking slice using existing events: 가능
- Public Boss mechanic lifecycle: Core semantic DTO + server parity + stale Unity synchronization을 하나의 cutover로
  수행한 뒤 가능
- Final-boss content: kernel extension points 검증 뒤 별도 Goals

필수 consumer에는 모두 접근했다. 외부 배포 Rust consumer나 실제 Unity PlayMode visual asset mapping은 이번
investigation의 구현 준비도 판정에 필요하지 않아 `unverified_external` blocker가 아니다.
