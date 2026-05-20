# 2D 연속좌표 시스템 감사

작성 목적: 이동, 피격, 사거리, 투사체, 범위 스킬처럼 `WorldVec2` 기반으로 동작하는 전투 코어를 최신 코드 기준으로 읽고 잠재 버그와 개선안을 정리한다. 이 문서는 즉시 리팩토링 지시서가 아니라, 현재 게임 흐름을 유지하면서 다음 작업자가 놓치기 쉬운 좌표계 계약을 남기는 기술 부채 목록이다.

## 감사 범위

읽은 주요 코드:

- `src/game/battle/core/movement/types.rs`
- `src/game/battle/core/movement/engine.rs`
- `src/game/battle/core/movement/rapier_backend.rs`
- `src/game/battle/core/movement/planner.rs`
- `src/game/battle/core/movement/steering.rs`
- `src/game/battle/core/movement/lifecycle.rs`
- `src/game/battle/core/spatial.rs`
- `src/game/battle/core/targeting.rs`
- `src/game/battle/core/commands.rs`
- `src/game/battle/core/sim.rs`
- `src/game/battle/core/skill_runtime/area.rs`
- `src/game/battle/core/skill_runtime/projectile.rs`
- `src/game/battle/battlefield/field.rs`
- `src/game/battle/battlefield/bfs.rs`
- `src/game/battle/scenario.rs`
- `tests/battle_rapier_movement.rs`
- `tests/battle_ranged_attack.rs`
- `tests/movement_timeline_exports.rs`
- `tests/skill_refactor_validation.rs`

제외 범위:

- 보상, 상점, 신뢰도, 장비 정책.
- 밸런스 수치 조정.
- Unity 클라이언트 표현/애니메이션.
- Rapier 내부 알고리즘 자체 검증.

## 현재 구조 요약

전투 코어는 타일 기반 authoring과 연속좌표 기반 runtime을 함께 사용한다.

## 2026-05-19 후속 처리 상태

이번 감사에서 P1로 분류했던 세 항목 중 다음 항목은 코드로 처리했다.

- tile/world drift: death snapshot과 cast target fallback에서 마지막 `WorldVec2`를 보존/사용하도록 했다.
- unit radius 고정값: `MovementDef.radius_units`를 추가하고 spawn body radius가 authored radius를 사용하도록 했다.
- persistent area geometry 고정: `SkillAreaTracking`을 추가해 `GroundFixed`, `FollowCaster`, `FollowTarget`을 데이터 계약으로 분리했고, follow 계열은 tick마다 geometry를 다시 계산한다.

```text
Battlefield tile
  - Position { x, y }
  - valid_tiles / static_obstacles / deployment / spawn
  - 시나리오 배치, 맵 구조, 정적 장애물, 일부 생존/무덤 스냅샷

World space
  - WorldVec2 { x, y }
  - 1 tile = 1 world unit
  - 이동, 사거리, 투사체, area 판정의 runtime source of truth

Data units
  - 1 world unit = 1_000_000 data units
  - RON 스킬 사거리, projectile 속도, area 크기

Timeline
  - TimelineVec2
  - world position을 milli 단위로 양자화해서 기록
```

핵심 계약:

- `WorldVec2::from_tile_center(Position)`은 `(x + 0.5, y + 0.5)`를 사용한다.
- `UnitBody.position`과 `UnitBody.radius`가 runtime 피격/사거리 판정의 기준이다.
- `UnitBody::can_reach`는 중심 거리 `<= range + self.radius + other.radius`로 판정한다.
- 전투 중 continuous movement는 `Battlefield.unit_pos`를 계속 갱신하지 않는다. 즉, tile position은 spawn/authoring 쪽 상태이고, runtime 판정은 가능한 한 `UnitBody`를 사용해야 한다.
- 비정형 맵의 invalid tile은 `void_tile` static obstacle로 continuous movement에 투영된다.

## 이동 흐름

1. `spawn_scenario_group`에서 `UnitBody`를 tile center에 만들고, authored `speed_units_per_ms`를 world units/sec로 변환한다.
2. `ContinuousMovementTick`은 기본 50ms 간격으로 돈다.
3. `build_continuous_attack_goals`가 전술 계획과 현재 목표를 보고 `MovementGoal`을 만든다.
4. `build_continuous_movement_input_with_goals`가 live unit body, 이동 가능 여부, board bounds, static/void obstacles를 movement backend 입력으로 변환한다.
5. 기본 backend는 `RapierMovementWorld`다. Direct backend는 테스트/폴백 성격이다.
6. backend 출력 `BodyMoved`는 `RuntimeUnit.body`를 갱신하고 `MovementSegmentStarted` timeline을 기록하거나 기존 segment를 연장한다.

주의할 점:

- Rapier backend도 완전한 동시 step이 아니다. 입력 유닛을 canonical unit id 순서로 정렬한 뒤 순차적으로 `KinematicCharacterController::move_shape`를 적용한다.
- `ContinuousMovementTick`은 이벤트 우선순위가 낮다. 같은 timestamp의 projectile/area/attack resolve가 먼저 처리되고, movement tick은 마지막에 처리된다. 대신 projectile/area 샘플링은 active movement segment를 보므로 이전 movement segment 안의 보간 위치를 읽을 수 있다.
- `MovementStopped` timeline은 주로 `interrupt_movement` 경로에서 기록된다. continuous engine이 `TargetReached/NoGoal`을 반환하는 경우에는 active segment만 제거하고 별도 stopped event를 남기지 않는다.

## 피격/사거리 흐름

### 기본 공격

- target selection은 `choose_attack_target_in_range`와 `is_basic_attack_target_in_range`를 통해 `UnitBody::can_reach`를 사용한다.
- instant basic attack은 resolve 시점에 다시 range를 검증한 뒤 즉시 damage를 적용한다.
- projectile basic attack은 발사 시점에 attacker/target의 world position을 aim으로 기록하고, 이후 1ms reevaluation으로 target body의 이동 segment를 샘플링한다.
- basic projectile collision은 `moving_circle_sweep_hit_fraction`으로 projectile과 target이 모두 움직이는 상황을 처리한다.

### 스킬 투사체

- targeted enemy single projectile은 homing, 그 외는 fixed projectile로 dispatch된다.
- fixed projectile은 window `[from_elapsed_ms, to_elapsed_ms]` 안에서 모든 허용 대상의 이동 segment를 샘플링하고 hit time, 거리, unit id 순으로 정렬한다.
- homing projectile은 target이 죽었거나 더 이상 valid target이 아니면 현재 위치에서 terminal impact/miss 성격의 event를 낸다.
- projectile collision radius는 authored radius + target body radius를 사용한다.

### Area 스킬

- `resolve_area_geometry`가 origin, center, direction_hint를 만든다.
- target unit마다 `sample_unit_body_at(time_ms)`로 body를 샘플링하고, `body.radius`를 `AreaQueryShape.hitbox_expansion`으로 넣는다.
- circle/box/rectangle/line/cone은 `SpatialQueryBackend`를 통해 판정한다.
- persistent area는 등록 시 geometry를 고정하고, tick policy에 따라 `EveryTick`, `OncePerArea`, `OnEnter`를 적용한다.

## 검토 결과

### P1. tile/world dual state drift는 명시적 계약으로 관리해야 한다

상태: 핵심 fallback 경로 처리 완료. 여전히 `Battlefield.position_of`는 authoring/tile state라는 계약을 유지한다.

`Battlefield.position_of`는 continuous movement 중 갱신되지 않는다. 현재 사거리, projectile, area, target selection 대부분은 `UnitBody`를 사용하므로 핵심 전투 판정은 world space에 있다. 다만 다음 경로는 tile state를 아직 참조한다.

- `resolve_cast_origin_context`는 live caster의 tile 존재 여부를 확인한다.
- `dispatch_skill_projectile_delivery`는 unit target의 `battlefield.position_of`를 먼저 확인한 뒤 world aim을 계산한다.
- death snapshot은 `battlefield.remove`가 반환한 tile position을 graveyard에 저장한다.
- `unit_world_position_or_tile_center`는 live unit이면 body position을 우선하지만, dead unit이면 graveyard tile center로 fallback한다.

처리 내용:

- `UnitSnapshot`에 `world_position`을 추가해 사망/무덤 상태에서도 마지막 continuous 위치를 보존한다.
- `unit_world_position_or_tile_center`의 graveyard fallback이 tile center 대신 보존된 world position을 사용한다.
- delayed area/cast target anchor가 unit tile이 아니라 cast 시점의 world position을 저장해 재사용한다.
- `apply_hp_delta_records_died_stop_with_latest_continuous_position`와 delayed target snapshot 테스트로 계약을 고정했다.

남은 개선안:

- runtime 전투 판정에서 tile이 필요한 이유를 함수명/주석으로 분리한다. 예: `live_unit_exists_on_battlefield`와 `runtime_unit_world_position`을 분리.
- future rule에서 "현재 타일", "점령", "장판 위 사망 위치"가 필요해지면 `projected_tile`을 명시적으로 갱신할지, 별도 occupancy layer를 둘지 먼저 결정한다.

### P1. 기본 공격 projectile impact time은 hit fraction을 쓰지 않는다

상태: 낮은 빈도의 정확도 리스크.

basic projectile은 sweep hit fraction으로 `impact_position`은 정확히 계산하지만, damage와 timeline event time은 현재 reevaluation timestamp를 그대로 쓴다. reevaluation 간격이 1ms라 현재 체감 오차는 작다. 반면 skill projectile은 hit fraction으로 `impact_time_ms`를 계산한다.

개선안:

- basic projectile도 skill projectile처럼 hit fraction 기반 `impact_time_ms`를 계산해서 `apply_basic_attack_projectile_hit_at`에 넘긴다.
- 변경 시 기존 ranged attack timeline test를 보강해 1ms 안에서 교차한 projectile의 event time/position 계약을 고정한다.

### P1. unit radius가 전 유닛 고정값이다

상태: 처리 완료. 콘텐츠가 unit body radius를 authoring할 수 있다.

현재 spawn path는 모든 combat unit에 `DEFAULT_UNIT_RADIUS = 0.35`를 넣는다. 사거리, 충돌, area expansion, projectile collision이 모두 이 radius를 사용하므로, 보스/소형 유닛/방어 오브젝트가 늘어나면 실제 크기와 피격감이 어긋날 수 있다.

처리 내용:

- `MovementDef.radius_units`를 추가했고 RON 누락 시 기존 기본값 `350_000` data units를 사용한다.
- scenario spawn path가 `DEFAULT_UNIT_RADIUS` 대신 `combat_profile.movement.radius_units`에서 `UnitBody.radius`를 만든다.
- 테스트/헬퍼의 직접 `MovementDef` 리터럴은 기존 기본 반경 `350_000`을 명시한다.
- `spawned_unit_uses_authored_body_radius`가 authored radius가 runtime body로 들어가는지 검증한다.

남은 개선안:

- radius가 달라지면 target preference tie-break도 중심 거리 대신 edge distance 또는 surface gap 기준을 검토한다.
- 큰 radius 유닛이 좁은 void/static obstacle corridor에서 spawn될 때 clamp/depenetration 결과가 유효한지 테스트한다.

### P2. canonical unit id 순차 이동은 deterministic하지만 완전 동시 이동은 아니다

상태: 문서화된 계약. 대규모 본대 이동에서 체감될 수 있다.

Direct/Rapier backend 모두 입력을 canonical unit id 순서로 정렬한다. Rapier backend는 앞에서 처리된 유닛의 translation을 바로 physics world에 반영하므로, 뒤 유닛은 앞 유닛의 새 위치를 기준으로 controller collision을 받는다.

개선안:

- 지금은 replay determinism을 위해 유지한다.
- 문제가 보이면 random shuffle이 아니라 two-phase movement를 설계한다. 예: 모든 desired displacement 산출, collision reservation, commit 순서 분리.
- player/opponent 또는 UUID 생성 방식이 이동 우선권으로 체감되는지 dense battle replay에서 관찰한다.

### P2. Direct backend와 Rapier backend는 같은 gameplay feel을 보장하지 않는다

상태: 문서화된 계약.

Direct backend는 expanded AABB sweep/depenetration과 간단한 separation solver를 사용한다. Rapier backend는 character controller, ball collider, cuboid walls/static obstacles를 사용한다. 둘 다 board bounds/static/void obstacle을 반영하지만, 좁은 통로와 다중 유닛 접촉에서 결과가 다를 수 있다.

개선안:

- production 감각 판단은 기본값인 Rapier 기준으로 한다.
- Direct backend는 deterministic fallback과 최소 계약 테스트로만 취급한다.
- 테스트 이름/문서에서 spatial query backend 동등성 테스트와 movement backend 동등성을 혼동하지 않게 한다.

### P2. projectile/area는 obstacle line-of-sight를 보지 않는다

상태: 게임 디자인 확인 필요.

movement는 static obstacle과 void tile을 충돌체로 반영한다. 하지만 projectile sweep과 area shape 판정은 unit body와 shape geometry만 보고, 벽/void tile에 막히는지 확인하지 않는다. 현재 의도라면 "장애물은 이동만 막고 공격/스킬은 막지 않는다"는 계약이다.

개선안:

- 장애물이 projectile/line/rectangle/cone을 막아야 하는 게임이면 spatial query에 obstacle cast를 추가한다.
- 장애물이 피격을 막지 않는 게임이면 이 계약을 스킬 문서와 전투 규칙서에 명시한다.

### P2. persistent area geometry는 생성 시 고정된다

상태: 처리 완료. 기본값은 기존과 같은 `GroundFixed`이고, follow 동작은 명시적으로 opt-in한다.

persistent area는 `register_persistent_area` 시점의 origin/center/direction_hint를 runtime에 저장하고, 이후 tick에서 같은 geometry를 쓴다. caster나 target을 따라가는 장판이 필요한 경우 현재 모델로는 표현되지 않는다.

처리 내용:

- `SkillAreaTracking::{GroundFixed, FollowCaster, FollowTarget}`을 추가했다.
- persistent area runtime이 anchor/tracking/duration/step target을 보존한다.
- `FollowCaster`와 `FollowTarget`은 tick마다 현재 caster/target 위치로 geometry를 다시 계산한다.
- runtime contract validation은 `FollowCaster`가 `Caster` anchor를, `FollowTarget`이 `CastTarget`/`CastTargetStart` anchor를 요구한다.
- timeline `SkillAreaDeclared`에 tracking 값을 기록한다.
- `persistent_area_follow_caster_updates_geometry_each_tick`, `persistent_area_follow_target_updates_geometry_each_tick`로 추적 장판 계약을 고정했다.

### P3. cone은 원형 부채꼴이 아니라 확장된 triangle wedge다

상태: 문서화된 계약.

cone 판정은 정확한 circular sector의 Minkowski sum이 아니라, centerline reach를 보존하기 위해 edge reach를 늘린 triangle wedge다. Direct와 Rapier spatial query가 같은 대표점을 검증한다.

개선안:

- 대형 유닛/보스 스킬에서 cone 경계 체감이 중요해지면 circular sector 기반 판정으로 바꿀지 별도 검토한다.
- 바꾸기 전까지는 스킬 설명과 VFX가 triangle wedge 계약에 맞아야 한다.

### P3. projectile speed 0은 instant처럼 처리된다

상태: 방어적 구현. authoring 오류를 숨길 수 있다.

basic/skill projectile flight time 계산은 speed 0을 panic 대신 0ms로 처리한다. 테스트도 이 동작을 기대한다. 안정성 측면에서는 좋지만, 데이터 실수로 projectile이 즉발화될 수 있다.

개선안:

- RON validation 또는 catalog audit에서 projectile speed 0을 warning/error로 잡는다.
- 정말 즉발 projectile이 필요하면 `DeliveryDef::Instant` 또는 별도 explicit flag로 표현한다.

## 테스트 커버리지 메모

확인한 관련 테스트:

- Rapier movement full battle/dense/reproducibility: `tests/battle_rapier_movement.rs`
- movement timeline export와 static blocker detour: `tests/movement_timeline_exports.rs`
- ranged projectile hit/miss/zero speed/continuous range: `tests/battle_ranged_attack.rs`
- skill projectile conditional follow-up, untargeted projectile, piercing/max hits: `tests/skill_refactor_validation.rs`
- unit-level spatial tests: `src/game/battle/core/spatial.rs`
- movement backend/order/static obstacle/void tile tests: `src/game/battle/core/movement/engine.rs`, `src/game/battle/core/movement/rapier_backend.rs`
- area anchor/shape/persistent tick tests: `src/game/battle/core/mod.rs`

테스트 보강 후보:

- basic projectile impact time이 hit fraction 기반으로 기록되는지.
- moving unit 사망 후 graveyard fallback이 delayed area/projectile follow-up에서 기대한 위치를 쓰는지.
- per-unit radius 도입 전후 target preference와 corridor movement.
- obstacle line-of-sight를 막기로 결정할 경우 projectile/area obstacle cast.

## 권장 작업 순서

1. 좌표계 계약을 유지한다면 `Battlefield.position_of`가 runtime world position이 아니라는 점을 주석/문서/함수명으로 더 강하게 드러낸다.
2. basic projectile impact time을 skill projectile과 맞출지 결정한다. 작고 독립적인 개선으로 보인다.
3. `MovementDef`에 radius authoring을 넣을지 결정한다. 콘텐츠 확장 전에 처리하는 편이 좋다.
4. 장애물이 공격/스킬을 막는지 게임 디자인 결정을 내리고, 결정에 맞춰 spatial query 또는 문서를 보강한다.
5. 대규모 본대 이동에서 순차 처리 체감이 보이면 two-phase movement를 별도 목표로 잡는다.

## 이번 감사에서 사용한 확인 명령

```bash
rg --files
rg -n "Vec2|WorldPos|Position|position|coord|coordinate|Movement|movement|velocity|Velocity|collision|Collision|hit|Hit|damage|Damage|Projectile|projectile|radius|range" src tests
rg -n "fn .*movement|fn .*projectile|fn .*area|fn .*range|fn .*continuous|fn .*rapier|fn .*cone|fn .*radius" src/game/battle/core src/game/battle/core/movement tests/battle_rapier_movement.rs tests/skill_refactor_validation.rs tests/battle_ranged_attack.rs tests/movement_timeline_exports.rs
sed -n '1,260p' src/game/battle/core/movement/types.rs
sed -n '1,760p' src/game/battle/core/movement/engine.rs
sed -n '1,900p' src/game/battle/core/movement/rapier_backend.rs
sed -n '1,360p' src/game/battle/core/movement/planner.rs
sed -n '1,320p' src/game/battle/core/movement/steering.rs
sed -n '1,760p' src/game/battle/core/spatial.rs
sed -n '1,840p' src/game/battle/core/skill_runtime/area.rs
sed -n '1,960p' src/game/battle/core/skill_runtime/projectile.rs
sed -n '1,1235p' src/game/battle/core/commands.rs
sed -n '230,370p' src/game/battle/core/sim.rs
sed -n '1040,1185p' src/game/battle/core/sim.rs
sed -n '1560,1885p' src/game/battle/core/sim.rs
sed -n '2415,2455p' src/game/battle/core/sim.rs
sed -n '1,320p' src/game/battle/battlefield/field.rs
sed -n '1,280p' src/game/battle/battlefield/bfs.rs
```

2026-05-19 구현 검증:

```bash
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo check
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core persistent_area -- --nocapture
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core authored_body_radius -- --nocapture
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core apply_hp_delta_records_died_stop_with_latest_continuous_position -- --nocapture
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core area_reuse_cast_target_preserves_unit_identity_for_anchor_sampling -- --nocapture
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core diagonal_basic_attack_uses_continuous_body_range_not_chebyshev_tiles --test battle_ranged_attack -- --nocapture
env CARGO_TARGET_DIR=/tmp/core_enhanced_target cargo test -p game_core
```
