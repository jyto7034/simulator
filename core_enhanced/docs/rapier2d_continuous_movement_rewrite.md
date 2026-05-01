# Rapier2D Continuous Movement Rewrite Plan

## Purpose

이 문서는 기존 타일 기반 이동 시스템을 유지하지 않고, Rapier2D 도입을 전제로 전투 이동 시스템을 연속 좌표 기반으로 재작성하기 위한 최종 계획서다.

기존 호환은 목표가 아니다. 현재 movement stack은 대담하게 제거하고, 전투 코어가 앞으로 의존해야 할 새 이동 모델을 먼저 고정한다.

대상 독자:

- 이동 시스템을 이어서 구현할 AI/개발자
- 전투 코어와 타임라인을 함께 수정해야 하는 개발자
- Rapier2D 도입 범위와 비범위를 판단해야 하는 개발자

## Read This First

이 문서가 현재 movement rewrite의 source of truth다.

`docs/old/movement_orchestrator_plan.md`와 `docs/old/movement_orchestrator_execution.md`는 이전 방향의 문서다. 그 문서들은 "타일 점유/예약 authority를 유지하면서 movement orchestrator를 개선한다"는 전제를 가진다. 현재 결정은 그 반대다. 새 방향은 `Battlefield` 타일 점유/예약을 전투 중 movement authority에서 제거하고, continuous world body를 canonical 위치로 삼는다.

따라서 문서 간 충돌이 있으면 이 문서를 우선한다.

현재 코드 상태 요약:

- workspace는 상위 `/mnt/f/work/simulator/Cargo.toml`에서 `core` 대신 `core_enhanced`를 `game_core` member로 사용한다.
- `game_server/Cargo.toml`도 `game_core = { path = "../core_enhanced" }`를 참조한다.
- `src/game/battle/core/movement/types.rs`에 `WorldVec2`, `TimelineVec2`, `UnitBody`, `MovementMode`, `MovementGoal`이 추가되어 있다.
- `src/game/battle/placement.rs`에 `PlacementBoard`, `PlacementSlot`, `PlacementSlotId`가 추가되어 있다. `PlayerDeckInfo.positions`의 `Position`은 현재 하위 호환 입력 타입이지만, 전투 시작 시 TFT-like flat-top hex placement slot id로 해석해 hex center `WorldVec2`로 변환한다.
- `src/game/battle/core/movement/engine.rs`에 `MovementEngine`, `DirectContinuousMovement`, `BattleCore` adapter가 추가되어 있다.
- `BattleCore`는 `ContinuousMovementBackend`를 상태로 소유한다. 현재 variant는 `DirectContinuousMovement`와 `RapierMovementWorld`이며, 기본값은 `RapierMovementWorld`다. `DirectContinuousMovement`는 명시적으로 선택하는 fallback/test backend로만 남긴다.
- `src/game/battle/core/targeting.rs`가 basic attack range/target selection을 소유한다.
- `run_battle`은 전투 시작 시 legacy `MovementIntent`가 아니라 `BattleEvent::ContinuousMovementTick`을 스케줄한다.
- `ContinuousMovementTick`은 `run_continuous_attack_movement_tick()`을 실행하고 다음 fixed tick을 다시 스케줄한다.
- movement timeline은 `TimelineEvent::MovementSegmentStarted`를 continuous movement segment로 사용한다.
- 같은 quantized velocity로 이어지는 continuous movement tick은 기존 `MovementSegmentStarted`의 `ends_at_ms`/target 좌표를 연장해 timeline noise를 줄인다.
- timeline 외부 좌표 출력은 legacy `*_units` 필드가 아니라 `TimelineVec2 { x_milli, y_milli }` fixed-point 좌표를 사용한다. `TIMELINE_VERSION`은 19이다.
- `TimelineEvent::ContinuousUnitMoved`는 제거되어 있다.
- legacy tile movement test modules와 `movement_intent_*` unit tests는 continuous rewrite 중 보수하지 않도록 제거되어 있다.

아직 남아 있는 legacy:

- `BattleEvent::MovementIntent`와 `BattleEvent::MoveStep` 타입은 제거되어 있다.
- `movement/orchestrator.rs`, `movement/execute.rs`는 삭제되어 있다.
- `movement/plan.rs`는 삭제되어 있다. BFS/tile chase planner는 제거됐고, basic attack range/target helper는 `core/targeting.rs`로 이동했다.
- `RuntimeUnit`은 `UnitBody`를 직접 필드로 가진다. legacy `pos_x_units`/`pos_y_units` 필드와 `set_legacy_position_units()` helper는 제거되어 있고, 위치 변경은 `set_world_position(WorldVec2)`로만 들어간다.
- 기본 공격 range 판정과 `select_basic_attack_target`은 `UnitBody` continuous distance + radius 기준으로 전환되어 있다.
- skill target selection도 `UnitBody` continuous distance + radius 기준으로 전환되어 있다.
- skill area target collision은 `ContinuousPosition` integer geometry가 아니라 `WorldVec2` 기반 spatial 함수로 판정한다.
- fixed/homing skill projectile runtime은 launch/current/aim/impact collision 계산을 `WorldVec2`로 수행한다.
- `BattleCore`는 skill area/projectile collision 판정용 `SpatialQueryBackend`를 소유한다. 기본값은 `SpatialQueryBackend::Rapier`다.
- `SpatialQueryBackend::Rapier`는 Rapier/Parry shape intersection으로 circle/box/line/rectangle/cone area와 projectile circle overlap을 처리한다. fixed/homing projectile은 window 단위 circle shape cast로 최초 hit fraction을 계산한다.
- line area는 Parry `Capsule` intersection으로 처리한다. cone area는 원형 sector가 아니라 Parry `Triangle` convex wedge가 공식 semantics다. `length_units`는 중앙선 도달 거리이며 edge vertex는 `length / cos(half_angle)`로 배치해 중앙선 reach를 보존한다. 기존 `DirectSpatialQuery`는 fallback이 아니라 parity 비교/test backend로만 남긴다.
- 기존 `WorldVec2` 수학 판정은 `DirectSpatialQuery`로 격리되어 parity backend와 비교 테스트에 사용한다.
- fixed projectile의 event time은 이벤트 큐 실행을 위해 ms 단위로 올림 quantize하지만, `impact_position`은 quantized event time의 projectile sample이 아니라 Rapier hit fraction 기반 연속 좌표로 기록한다.
- targeted homing skill projectile은 더 이상 launch-time impact 예약으로 처리하지 않는다. `SkillProjectileAdvance` active runtime이 매 tick target body 현재 위치로 aim을 갱신하고, Rapier/Parry sweep으로 최초 접촉을 계산한다.
- skill projectile timeline은 `SkillProjectileLaunched`와 `SkillProjectileImpacted`를 내보낸다. 클라이언트는 launch의 `start`/`aim`/`expected_end_time_ms`와 impact의 `impact_position`/VFX id로 projectile flight/despawn/hit effect를 직접 보간할 수 있다.
- basic attack projectile은 더 이상 target-lock `ProjectileHit` 예약 이벤트로 판정하지 않는다. `BasicAttackProjectileAdvance`가 active projectile을 전진시키고, 매 window마다 Rapier/Parry sweep으로 target body와 projectile radius의 최초 접촉을 계산한다. timeline은 `BasicAttackProjectileLaunched`와 실제 collision 기반 `BasicAttackProjectileImpacted`를 내보낸다.
- `BattleEvent::SkillProjectileImpact`와 `SkillImpactContext`는 impact 위치를 `WorldVec2`로 보관한다. event queue와 skill follow-up context의 projectile 위치는 더 이상 `ContinuousPosition` bridge를 거치지 않는다.
- skill area runtime geometry와 `BattleEvent::SkillAreaTick.center`는 `WorldVec2`를 보관한다. area 외부 timeline 출력은 `TimelineVec2`로 quantize한다.
- `ContinuousPosition` 타입과 `tile_center_units(Position)` helper는 제거되어 있다. 기본 공격 projectile record/replay spatial sampling도 `WorldVec2`를 사용한다.
- homing projectile lifetime/expected end time은 legacy tile distance가 아니라 caster/target continuous aim 위치의 Euclidean distance를 기준으로 계산한다. 실제 hit는 active runtime의 sweep collision 시점에 발생한다.
- skill/basic attack data schema는 `range_units: f32`로 전환되어 있다. `range_tiles`는 `src/old/movement.rs`의 보관용 old code에만 남아 있다.
- `rapier2d = "0.32.0"` dependency가 추가되어 있다.
- `movement/rapier_backend.rs`에 Rapier handle 소유권을 `BattleCore`/combat rule에서 분리하기 위한 `RapierMovementWorld`가 추가되어 있다.
- `RapierMovementWorld::sync_units()`는 alive unit의 kinematic body와 ball collider를 생성/갱신하고, dead/absent unit을 Rapier world에서 제거한다.
- `ContinuousMovementBackend::Rapier(RapierMovementWorld)` variant가 추가되어 있고, 현재 `BattleCore`의 기본 movement backend다. `BattleCore::use_direct_continuous_movement_backend()`은 Direct fallback/test backend를 명시적으로 선택할 때만 사용한다.
- Rapier backend는 `KinematicCharacterController::move_shape()`로 desired translation을 collision-aware corrected translation으로 바꾸는 tick path를 가진다.
- Rapier backend는 board bounds를 four-wall static collider로 동기화한다.
- Rapier backend의 unit-unit 강제 depenetration은 꺼져 있다. 겹침은 planner/steering으로 줄이고, Rapier KCC는 최종 movement correction을 담당한다.
- `BattleCore`는 read-only diagnostics로 live body snapshot과 active projectile/area runtime count를 노출한다. Rapier long-battle smoke는 60초 전투(기본 movement tick 1,200회 이상), static obstacle, 다수 유닛, 최종 body finite/bounds/severe-overlap, active runtime 누수를 검증한다.
- Unity timeline contract용 7v7 Rapier showcase export가 추가되어 있다. `tests/unity_timeline_contract_exports.rs`는 movement segment/stop, basic attack projectile launch/impact, skill projectile launch/impact, skill area declaration, damage event가 모두 포함된 `timeline_exports/unity_contract_7v7_rapier_showcase.json`을 산출한다.
- Unity timeline contract export는 movement 품질 metric도 검증한다. 6v6 melee/7v7 mixed showcase는 spawned unit participation, movement segment count, max segment speed, very short segment count를 assertion으로 보호한다.
- `Battlefield`는 전투 중 이동 권위가 아니라 static obstacle metadata를 보관할 수 있고, `BattleCore`는 이를 `MovementStaticObstacle`로 movement input에 투영한다.
- Rapier backend는 `MovementStaticObstacle`을 fixed rigid body + cuboid collider로 동기화한다.
- `PveEncounter.static_obstacles` schema가 추가되어 suppression encounter가 고정 장애물을 선언할 수 있다. encounter obstacle 좌표는 opponent half local coordinate로 해석하고 battle field 생성 시 opponent row offset을 더한다.

현재 검증 명령:

```bash
cargo check -p game_core
cargo test -p game_core --lib
cargo test -p game_core --test battle_rapier_movement -- --nocapture
cargo test -p game_core --test unity_timeline_contract_exports -- --nocapture
cargo test -p game_core --test movement_timeline_exports -- --nocapture
cargo test -p game_core --test skill_refactor_validation
cargo test -p game_core --test battle_ranged_attack -- --nocapture
```

현재 직접 이어서 할 일:

1. 실제 `game_resources/data/pve/encounters.ron`에 obstacle 선언이 필요한 encounter를 설계한다.
2. projectile launch/impact timeline contract를 클라이언트/Unity mock에서 소비하는 쪽까지 연결한다.

## Summary

현재 이동 시스템은 `Position` 타일, `Battlefield` 점유/예약, `MovementIntent -> MoveStep` 이벤트, `Vec<Position>` path를 중심으로 구성되어 있다.

이 모델은 `1 tile = 1 occupied slot` 규칙이 너무 강해서 TFT 같은 자연스러운 자동전투 연출과 충돌한다. Rapier2D를 이 구조 위에 얹으면 물리/연속 좌표 시스템이 아니라 타일 레거시를 보정하는 얇은 레이어가 될 가능성이 높다.

따라서 새 방향은 아래와 같다.

- 전투 중 canonical 위치는 타일이 아니라 continuous world position이다.
- 유닛은 원형 body를 가진다.
- 이동, 충돌, 사거리, 근접 engage는 모두 거리와 radius 기준으로 판정한다.
- `Position`은 deck 입력/placement slot id와 내부 coarse/debug 용도로만 남긴다.
- Unity가 소비하는 timeline 계약에는 tile `position`, `from`, `to`를 노출하지 않는다.
- `Battlefield`는 전투 중 점유 권위를 잃고, spawn/bounds/coarse map 역할로 축소한다.
- Rapier2D는 dynamic full physics가 아니라 kinematic/query movement backend로 사용한다.
- 기존 이동 타임라인 이벤트는 새 continuous movement timeline으로 대체한다.

## Non-Goals

이번 재작성의 비목표:

- 기존 movement timeline과의 하위 호환 유지
- 기존 `UnitMoved { from: Position, to: Position }` semantics 유지
- 기존 `MoveStep` 기반 타일 이동 타이밍 유지
- 기존 BFS 목적지 선택 정책 유지
- 모든 전투 오브젝트를 dynamic rigid body로 전환
- 물리엔진에 전투 판정을 위임

전투 규칙은 계속 `BattleCore`가 소유한다. Rapier2D는 이동/충돌/공간 query backend다.

## Current Legacy To Remove

삭제 또는 폐기할 핵심 개념:

- `BattleEvent::MovementIntent`
- `BattleEvent::MoveStep`
- `MovementState.path: Vec<Position>`
- `MovementState.path_cursor`
- `MovementState.reserved_destination`
- `MovementState.planned_continuation`
- `MovementState.step_from`
- `MovementState.step_to`
- `MovementState.step_start_x_units`
- `MovementState.step_start_y_units`
- `MovementState.target_x_units`
- `MovementState.target_y_units`
- `MovementState.step_started_at_ms`
- `MovementState.step_ends_at_ms`
- `MovementSegmentEndKind::Boundary`
- `MovementSegmentEndKind::RangeEnter` as a tile-step concept
- `Battlefield::reserve`
- `Battlefield::cancel_reservation`
- `Battlefield::reservation_at`
- `Battlefield::reservation_blocks_for`
- `Battlefield::reserved_destination_of`
- `Battlefield::reserved_unit_at`
- `Battlefield::move_unit` as normal movement authority
- BFS-based "empty tile around enemy" destination selection

일시적으로 남길 수는 있지만, 새 movement path에서 호출되면 안 된다.

## Concepts To Keep In Reduced Form

축소해서 유지할 개념:

- `Position`
  - 하위 호환 배치 slot id input
  - board/debug projection
  - coarse map cell
- `PlacementBoard`
  - TFT-like pre-combat deployment slots
  - slot id를 flat-top hex center `WorldVec2` spawn position으로 변환
- `Battlefield`
  - board width/height
  - spawn validation
  - static obstacle/coarse blocker metadata
  - debug projection helper
- deterministic seed
  - tie-break, goal selection, jitter가 필요하면 계속 사용
- `Side`
  - initial formation, target filtering, deterministic tie-break에 사용
- existing combat, skill, damage systems
  - 위치 조회와 사거리 판정을 continuous model로 바꾸면서 유지

## New Runtime Model

### Units

전투 중 유닛의 위치 상태는 `RuntimeUnit` 내부 또는 별도 `MovementWorld`에 저장한다.

예상 타입:

```rust
pub struct WorldVec2 {
    pub x: f32,
    pub y: f32,
}

pub struct UnitBody {
    pub position: WorldVec2,
    pub previous_position: WorldVec2,
    pub velocity: WorldVec2,
    pub radius: f32,
    pub move_speed: f32,
    pub body_handle: Option<UnitPhysicsHandle>,
}

pub enum MovementMode {
    Idle,
    Moving,
    MovementLocked { until_ms: u64 },
    Dead,
}

pub enum MovementGoal {
    AttackUnit {
        target_id: UnitInstanceId,
        desired_range: f32,
    },
    MoveToPoint {
        point: WorldVec2,
        stop_radius: f32,
    },
}
```

`ActionState`는 combat action state와 movement state가 섞여 있으므로 분리한다.

권장 분리:

- `ActionState`: 공격/시전/CC/dead 등 combat-oriented 상태
- `MovementMode`: idle/moving/locked/dead 등 movement-oriented 상태
- `MovementGoal`: 왜 움직이는지

### Coordinate Scale

새 좌표계는 단순해야 한다.

- `1.0 world unit = old 1 tile width`
- 전투 중 canonical 위치는 `WorldVec2`다.
- `PlayerDeckInfo.positions`의 `Position { x, y }`는 현재 하위 호환 입력 타입이며, 전투 시작 시 placement slot id로 해석한다.
- 초기 spawn은 `PlacementBoard::world_center(PlacementSlotId)`가 산출한 TFT-like flat-top hex center 좌표를 사용한다.

현재 placement board 정책:

- hex orientation: flat-top
- offset coordinate: odd-r row offset
- hex radius: `0.45`
- hex width: `sqrt(3) * radius`
- row vertical spacing: `1.5 * radius`
- odd row x offset: `hex_width / 2`
- origin: `(0.5, 0.5)`
- 예: slot `(0, 1)`은 world center `(0.5 + hex_width / 2, 0.5 + 1.5 * radius)`다.

주의:

- `WorldVec2::from_tile_center(Position)`은 debug/coarse tile conversion과 old compatibility에만 남긴다.
- battle start spawn path는 `from_tile_center`가 아니라 `PlacementBoard`를 사용해야 한다.
- 기존 `tile_center_units(Position)` 같은 integer unit 변환은 제거한다.
- timeline은 `f32` 또는 quantized integer milliworld units 중 하나를 선택한다.

결정론과 JSON 안정성을 우선하면 timeline에는 quantized integer를 기록한다.

```rust
pub struct TimelinePoint {
    pub x_milli: i32,
    pub y_milli: i32,
}
```

내부 simulation은 `f32`, 기록은 `round(x * 1000.0)` 방식으로 시작한다.

## Movement Loop

기존 이벤트 기반 `MovementIntent -> MoveStep`를 제거하고 고정 tick movement loop로 바꾼다.

권장 tick:

- combat events는 기존처럼 event queue time 기준으로 처리한다.
- movement는 fixed timestep으로 처리한다.
- 1차 구현은 `MOVEMENT_TICK_MS = 50`으로 시작한다.
- 더 자연스러운 replay가 필요하면 33ms 또는 16ms로 낮춘다.

새 loop 개념:

```text
current_time_ms
  process combat events due at current_time_ms
  movement_engine.tick(current_time_ms, dt_ms)
  acquire targets / start pending attacks
  emit timeline movement segments
  check winner
```

중요:

- 이동은 매 tick 모든 alive unit을 전역적으로 평가한다.
- 유닛별 타일 예약은 없다.
- 같은 target으로 몰리는 유닛은 ring/orbit goal과 collision avoidance로 분산한다.
- target acquisition은 movement tick 이후 한 번 더 실행한다.

## Targeting And Range

모든 사거리 판정은 distance 기반으로 바꾼다.

```text
distance(attacker.position, target.position)
  <= attack_range + attacker.radius + target.radius
```

`range_units`는 world-space 단위의 floating point range로 해석한다.

```text
range_units = 1.0 -> attack_range = 1.0
range_units = 3.0 -> attack_range = 3.0
```

이전 `range_tiles` schema는 제거됐다. 새 데이터는 타일 개수가 아니라 연속 좌표계의 거리값을 선언해야 한다.

근접 유닛의 목표는 "적 주변 빈 타일"이 아니다.

근접 목표:

```text
target.position - normalized(target.position - unit.position) * desired_range
```

다수 근접 유닛:

- 같은 target을 공격하는 유닛들을 target 주변 ring slot에 배치한다.
- ring slot은 고정 각도 후보를 deterministic하게 정렬한다.
- slot은 hard reservation이 아니라 preferred point다.
- 실제 충돌은 Rapier/steering이 보정한다.

현재 구현 상태:

- `BattleCore`는 `melee_slot_reservations`를 movement runtime state로 가진다.
- `MovementGoal::AttackUnit`은 `approach_point: Option<WorldVec2>`를 포함한다.
- `movement/planner.rs`가 sticky/nearest target 선택과 근접 engagement point 생성을 담당한다.
- 근접 사거리(`range_units <= 1.0`) 유닛은 타겟 중심이 아니라 선호 engagement point를 movement goal로 받는다.
- engagement point는 hard reservation이 아니라 planner가 고른 soft preferred position이다.
- 같은 target의 같은 `slot_index`는 한 유닛만 선호점으로 사용할 수 있다.
- 다른 target의 선호점도 world-space radius 기준으로 겹치지 않게 1차 분산한다.
- 이미 `can_reach`인 근접 유닛은 engagement point 재정렬을 위해 이동하지 않는다.
- 이전 engagement point는 target이 작은 폭으로 움직인 경우 유지한다.
- `movement/steering.rs`가 goal target position, desired displacement, separation/side-bias velocity를 담당한다.
- steering은 목표 근처에서 arrival slowdown을 적용하지 않는다. tick별 displacement 감속은 forward congestion에만 적용한다.
- steering side-bias는 가까운 forward blocker가 있을 때만 강하게 적용한다. 멀리서 출발하는 초반 이동은 약한 lane pressure로 좌우 흔들림이 생기지 않도록 early straightening을 적용한다.
- Direct/Rapier backend는 steering이 만든 desired displacement를 받아 backend별 collision correction을 적용한다.
- 유닛 간 강제 depenetration은 꺼져 있다. 겹침을 물리적으로 밀어내지 않고, planner/steering으로 줄이는 방향이다.

후속 문제와 우선순위:

1. 목적지는 달라도 접근 경로 corridor가 겹칠 수 있다.
   - corridor hard reservation을 바로 추가하지 않는다.
   - steering layer의 separation/side-bias/congestion slow를 우선 적용한다.
2. target 이동/사망/변경 시 engagement point가 크게 흔들릴 수 있다.
   - 작은 target drift는 유지하지만, 큰 target drift/target swap/death의 재예약 품질은 계속 검증해야 한다.
3. 보드 가장자리에서는 유효 slot 후보가 부족하다.
   - invalid slot 제외 후 fallback 후보를 더 많이 만들거나 valid arc 기반으로 재배치해야 한다.
4. 원거리 유닛도 장기적으로 선호 거리/후방 slot/kiting 위치 선정이 필요할 수 있다.

원거리 유닛:

- 사거리 안이면 이동하지 않는다.
- 사거리 밖이면 target을 향해 이동한다.
- 카이팅은 별도 정책이 생기기 전까지 하지 않는다.

## Rapier2D Integration Strategy

### Why Not Dynamic Bodies

Dynamic body는 물리엔진이 위치와 속도를 소유한다. 힘, 질량, 반발, 마찰, 충돌 반응이 위치를 결정한다.

자동전투 이동에서는 아래 문제가 생긴다.

- 결정론 디버깅이 어려워진다.
- 공격 사거리 진입 시점을 전투 규칙이 통제하기 어렵다.
- 유닛끼리 밀고 밀리는 결과가 전투 정책보다 물리 파라미터에 좌우된다.
- tiny numerical difference가 timeline 차이로 커질 수 있다.

따라서 유닛은 dynamic body로 시작하지 않는다.

### Kinematic/Query First

Rapier2D는 아래 용도로 사용한다.

- circle collider overlap
- shape cast
- kinematic character movement correction
- obstacle/body proximity query

유닛 body:

- `RigidBodyBuilder::kinematic_position_based()`
- `ColliderBuilder::ball(radius)`
- gravity disabled
- sensor는 상황별로만 사용

매 tick:

```text
for each unit:
  choose target
  compute desired velocity
  compute desired displacement = velocity * dt
  ask Rapier/controller/query for corrected displacement
  apply corrected position
```

전투 판정:

- Rapier contact가 아니라 `BattleCore` distance helper가 판단한다.
- Rapier는 이동 가능성/충돌 보정만 제공한다.

공식 Rapier character controller 문서 기준:

- `KinematicCharacterController`는 collider/body handle이나 position을 소유하지 않는 behavior object다.
- 같은 controller instance를 여러 unit collider에 재사용할 수 있다.
- `move_shape`는 원하는 이동량을 입력받고, 장애물과 controller option을 고려한 실제 적용 가능한 이동량을 반환한다.
- position-based kinematic body는 corrected movement를 현재 위치에 더한 다음 kinematic position으로 반영하는 방식이 권장된다.

## New Movement Engine Modules

권장 파일 구조:

```text
src/game/battle/core/movement/
  mod.rs
  types.rs
  engine.rs
  planner.rs
  steering.rs
  rapier_backend.rs
  timeline.rs
```

역할:

- `types.rs`
  - `WorldVec2`, `UnitBody`, `MovementMode`, `MovementGoal`
- `engine.rs`
  - movement tick orchestration
  - output generation
  - backend dispatch boundary
- `planner.rs`
  - nearest target
  - sticky target
  - melee engagement point
  - ranged hold/approach
  - planner-level preferred position conflict scoring
- `steering.rs`
  - goal target position
  - desired velocity
  - separation
  - side-bias local avoidance
  - desired displacement generation shared by Direct/Rapier backend
- `rapier_backend.rs`
  - Rapier sets and handles
  - collider sync
  - query/controller movement
- `timeline.rs`
  - movement timeline segment emission policy

기존 `orchestrator.rs`, `execute.rs`, `plan.rs`는 삭제한다.

## New Timeline Model

기존 `UnitMoved`는 제거한다.

새 movement timeline은 segment 중심으로 간다. 매 tick position update를 모두 기록하면 timeline이 너무 커진다.

권장 이벤트:

```rust
MovementSegmentStarted {
    unit_instance_id: UnitInstanceId,
    start: TimelineVec2,
    target: TimelineVec2,
    started_at_ms: u64,
    ends_at_ms: u64,
    end_kind: MovementSegmentEndKind,
}

MovementStopped {
    unit_instance_id: UnitInstanceId,
    world_position: TimelineVec2,
    reason: MovementStopReason,
}
```

`MovementSegmentChanged`는 아래 상황에서만 기록한다.

- target 변경
- collision correction이 유의미하게 발생
- movement lock/stun
- attack range 도달
- death
- large direction change

클라이언트는 segment를 보간한다. 서버는 매 tick 좌표를 전부 기록하지 않는다.

## Migration Phases

상태 표기:

- Done: 현재 코드에 반영됨
- Partial: 일부 반영됐지만 legacy bridge 또는 legacy code가 남아 있음
- Pending: 아직 시작 전

### Phase 1: Delete Tile Movement Authority

Status: Done

목표:

- 기존 movement event path를 제거한다.
- 컴파일 에러를 통해 의존성을 드러낸다.
- 전투 루프에서 movement tick slot을 만든다.

작업:

- `BattleEvent::MovementIntent` 제거: Done
- `BattleEvent::MoveStep` 제거: Done
- `split_bucket`에서 movement event bucket 제거: Done
- `schedule_movement_intent` 제거: Done
- `schedule_move_step_at` 제거: Done
- `movement/orchestrator.rs` 삭제: Done
- `movement/execute.rs` 삭제: Done
- `movement/plan.rs` 삭제: Done
- `ActionState::Moving(MovementState)` 제거: Done
- legacy movement state variants 제거: Done
- `Battlefield` reservation API 제거
- `BattleEvent::ContinuousMovementTick` 추가: Done
- `run_battle` 시작 tick을 `ContinuousMovementTick`으로 전환: Done
- no-target attack path에서 legacy `MovementIntent` 재스케줄 제거: Done

완료 기준:

- movement 관련 컴파일 에러 목록이 새 continuous model 작업으로 분류된다.
- 전투 루프에 fixed movement tick hook이 존재한다.

### Phase 2: Add Continuous Unit Body

Status: Partial

목표:

- `RuntimeUnit`이 continuous body를 가진다.
- spawn 시 `Position`이 `WorldVec2`로 변환된다.
- 위치 조회 helper가 continuous position 기준으로 바뀐다.

작업:

- `WorldVec2` 추가: Done
- `TimelineVec2` 추가: Done
- `UnitBody` 추가: Done
- `RuntimeUnit.body` 추가: Done
- old `pos_x_units`, `pos_y_units` 제거: Done
- `RuntimeUnit::set_legacy_position_units()` 제거: Done
- `BattleCore::unit_world_position(unit_id) -> Option<WorldVec2>` 추가: Done
- `BattleCore::unit_body_view(unit_id) -> Option<UnitBody>` 추가: Done
- `BattleCore::apply_unit_body_position(unit_id, body)` 추가: Done
- `BattleCore::project_unit_tile(unit_id) -> Option<Position>` 추가: Pending
- build runtime field에서 body position 초기화: Done

완료 기준:

- 유닛 spawn timeline은 projected tile과 continuous position을 함께 기록하거나 continuous-only로 바뀐다.
- combat code가 타일 position 대신 helper를 통해 위치를 읽기 시작한다.

### Phase 3: Distance-Based Combat Range

Status: Done

목표:

- 기본 공격 target acquisition이 continuous distance 기반이 된다.
- `range_units` 기반 continuous distance 비교를 combat runtime의 기본 range contract로 만든다.

작업:

- `attack_range_units(base_uuid) -> f32` 추가: Done
- `is_basic_attack_target_in_range`를 distance 기반으로 재작성: Done
- nearest enemy selection을 continuous distance 기반으로 재작성: Done
  - movement goal selection은 continuous distance를 쓴다.
  - combat attack target selection도 continuous distance를 쓴다.
- skill target range helper도 continuous distance 기반으로 전환: Done
- homing projectile travel time을 continuous aim distance 기반으로 전환: Done
- skill/basic attack data schema를 `range_units: f32`로 전환: Done
- existing Chebyshev range helper는 projection/debug 전용으로 격리: Done

완료 기준:

- 근접 1v1에서 유닛이 radius/attack range 기준으로 공격을 시작한다: Done
- 원거리 유닛이 range 안에서 이동하지 않는다: Done

### Phase 4: Direct Continuous Movement Without Rapier

Status: Done

목표:

- Rapier 없이도 전투가 움직인다.
- 단순 steering으로 movement tick을 검증한다.

작업:

- `MovementEngine::tick` 구현: Done
- nearest/sticky target goal 계산을 `movement/planner.rs`로 분리: Done
- 근접 engagement point 생성을 planner 계층으로 분리: Done
- desired velocity/displacement 계산을 `movement/steering.rs`로 분리: Done
- steering separation/side-bias velocity 추가: Done
- steering arrival slowdown 제거: Done
- steering congestion slowdown 추가: Done
- steering side-bias activation threshold/early straightening 추가: Done
- 이미 사거리 안인 근접 유닛의 engagement point 재정렬 방지: Done
- 작은 target drift에서 이전 engagement point 유지: Done
- Unity timeline movement 품질 metric 추가: Done
- TFT-like placement board 추가: Done
- deck `Position` 입력을 placement slot id로 해석하고 flat-top hex center로 변환: Done
- deterministic separation/overlap correction 추가: Done
- board bounds clamp 추가: Done
- `MovementSegmentStarted` 기록: Done
- same-velocity segment extension으로 timeline 압축: Done
- movement/area timeline 좌표를 `TimelineVec2 { x_milli, y_milli }`로 전환: Done
- fixed/homing skill projectile runtime position/collision을 `WorldVec2`로 전환: Done
- movement lock/stun/death interrupt 처리: Done

완료 기준:

- 1v1 melee approach test 통과: Done
- ranged hold test 통과: Done
- 3v3 melee blob이 완전히 한 점으로 겹치지 않음: Done
- deterministic seed 기반 timeline 재현 가능: Done

### Phase 5: Add Rapier2D Backend

Status: Partial

목표:

- 직접 steering collision을 Rapier query/kinematic correction으로 교체한다.

작업:

- `BattleCore`가 continuous movement backend state를 소유하도록 전환: Done
- `rapier2d` dependency 추가: Done
- `RapierMovementWorld` 추가: Done
- unit body/collider handle 관리: Partial
- spawn/death sync: Done
- Rapier backend가 planner goal과 steering displacement를 입력으로 받고 KCC correction만 적용하도록 1차 정리: Done
- kinematic body position sync: Done
- ball collider radius sync: Done
- `ContinuousMovementBackend::Rapier` variant 추가: Done
- Rapier backend를 `BattleCore` default runtime backend로 승격: Done
- Direct backend를 명시적 fallback/test backend로 분리: Done
- query/controller based corrected displacement 적용: Partial
- initial unit overlap depenetration 적용: Done
- real battle smoke/reproducibility semantic test 추가: Done
- dense/mixed-team battle semantic test 추가: Done
- static obstacle + dense/mixed-team battle semantic test 추가: Done
- static bounds collider 추가: Done
- optional static obstacles 지원: Partial
  - `Battlefield` static obstacle metadata: Done
  - `MovementStaticObstacle` input projection: Done
  - Rapier fixed cuboid collider sync: Done
  - `PveEncounter.static_obstacles` schema: Done
  - 실제 resource data 적용: Pending

완료 기준:

- 유닛끼리 collider radius 이상으로 심하게 겹치지 않는다: Partial
- blocker scenario에서 collision correction이 발생한다: Done at backend unit and dense battle level
- 같은 seed에서 결과가 재현된다: Done at full-battle smoke and dense/mixed-team level
- movement tick 비용이 허용 범위에 있다.

### Phase 6: Timeline And Tests Rewrite

Status: Partial

목표:

- 기존 tile movement tests를 제거하고 의미 기반 continuous tests로 대체한다.

작업:

- `movement_timeline_exports.rs` 재작성: Partial
- legacy tile movement unit tests 제거: Done
- legacy moving-target projectile tests 제거: Done
- `skill_refactor_validation`을 continuous/body 기준 기대값으로 정리: Done
- `battle_ranged_attack.rs`의 tile-step assertion 제거: Partial
  - legacy chebyshev/tile-step/result-fixed tests 제거: Done
  - continuous body range semantic replacement test 추가: Done
  - ignored tests 최종 삭제/대체: Done
- replay validation이 continuous movement event를 이해하도록 수정: Done
- death/spawn validation에서 `UnitMoved` 의존 제거: Done
- timeline exporter가 continuous event를 저장하도록 갱신: Done
  - 현재는 same-velocity continuous movement tick을 하나의 `MovementSegmentStarted`로 연장한다.
  - spawn, movement segment/stop, projectile, skill area 좌표는 `TimelineVec2` fixed-point 좌표로 저장한다.
  - timeline version 20부터 `UnitSpawned.position`, `MovementSegmentStarted.from/to`, `MovementStopped.position`, `UnitMoved`는 제거되었다.

완료 기준:

- movement tests가 tile count/time이 아니라 behavior를 검증한다.
- replay가 movement segment interpolation을 통해 상태를 복원할 수 있다.

## Initial Test Scenarios

새 테스트는 아래를 먼저 만든다.

1. `melee_unit_stops_when_continuous_attack_range_reached`
2. `ranged_unit_holds_position_when_target_in_range`
3. `melee_units_spread_around_single_target`
4. `movement_lock_interrupts_continuous_motion`
5. `dead_unit_is_removed_from_movement_world`
6. `continuous_targeting_prefers_nearest_enemy_by_distance`
7. `rapier_backend_prevents_severe_unit_overlap`: Done at backend/core tick level
8. `same_seed_replays_same_continuous_movement_timeline`: Done at Rapier full-battle smoke level

기존 `UnitMoved` 시간 assertion은 새 시스템에서 가치가 낮으므로 삭제한다.

## Implementation Order

권장 실제 작업 순서:

1. 이 문서를 기준으로 기존 movement files의 public API 사용처를 전부 검색한다.
2. `WorldVec2`, `UnitBody`, `MovementMode`, `MovementGoal` 타입을 추가한다.
3. `RuntimeUnit`에 `UnitBody`를 추가하고 old coordinate fields 제거 준비를 한다.
4. `BattleEvent::MovementIntent`와 `MoveStep`를 제거한다.
5. `sim.rs`에 fixed movement tick hook을 추가한다.
6. 기존 `movement/orchestrator.rs`, `movement/execute.rs`, `movement/plan.rs`를 삭제한다.
7. 깨진 combat range helper를 `core/targeting.rs`의 continuous distance helper로 복구한다.
8. Rapier 없는 direct movement engine을 먼저 통과시킨다.
9. timeline event schema를 continuous segment로 바꾼다.
10. Rapier2D를 추가하고 backend를 교체한다.

## Risks

주요 위험:

- 기존 tests 대부분이 깨진다.
- replay/validation layer가 movement timeline schema에 강하게 의존한다.
- skill projectile/area runtime이 `Position` projection에 의존하는 부분이 남아 있을 수 있다.
- deterministic behavior가 Rapier floating-point 결과에 영향을 받을 수 있다.
- fixed movement tick 도입으로 combat event ordering이 달라질 수 있다.

대응:

- 초반에는 Rapier 없이 직접 continuous movement를 먼저 구현한다.
- Rapier backend는 query/kinematic 용도로 제한한다.
- timeline에는 quantized coordinates를 기록한다.
- behavior tests는 exact tile-step 대신 semantic assertions로 재작성한다.
- movement tick ordering을 문서화하고 고정한다.

## Decision Record

확정 결정:

- 기존 tile movement compatibility는 유지하지 않는다.
- `UnitMoved` 중심 timeline은 폐기한다.
- 전투 중 위치의 canonical source는 continuous body다.
- `Battlefield`는 movement authority가 아니다.
- Rapier2D는 dynamic physics가 아니라 kinematic/query backend로 시작한다.
- 거리 기반 사거리 판정으로 전환한다.
- BFS empty attack tile planner는 제거한다.

보류 결정:

- timeline coordinate를 `f32`로 직렬화할지 `milli-units i32`로 직렬화할지
- fixed movement tick을 50ms, 33ms, 16ms 중 어디로 둘지
- melee ring slot을 movement engine 내부 정책으로 둘지 target acquisition layer로 둘지
- Rapier character controller를 바로 쓸지 shape cast 기반 custom correction을 먼저 쓸지
