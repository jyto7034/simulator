# Active Projectile Runtime Plan

## Purpose

이 문서는 현재 `launch-time scheduled projectile` 모델을
`active projectile runtime` 모델로 확장할지 판단하고,
실제로 구현 가능한 형태로 좁혀 놓기 위한 **집중 설계 문서**다.

기존 문서들은 범위가 너무 넓다.
이 문서는 아래 질문 하나에만 답한다.

- `스킬 투사체가 발사 후 전장 변화(CC, death, movement change)에 더 잘 반응하도록 만들 수 있는가?`

짧은 결론:

- **구현 가능하다**
- 다만 단순한 “계산량 증가”가 아니라
  **projectile runtime 모델 자체를 바꾸는 2차 리팩토링**이다
- 가장 안전한 방향은
  `launch-time prediction`을 완전히 버리는 것이 아니라,
  `active projectile runtime`을 `skill projectile`에만 도입하는 것이다

## Current Status

현재 상태를 짧게 요약하면:

- `untargeted fixed skill projectile`의 active runtime 전환은 완료됐다
- `same time_ms`에서 projectile reevaluation이 settled board state를 읽도록
  simulation bucket ordering도 조정됐다
- fixed projectile regression은
  blocker / target death / post-launch hard CC / pierce / max_hits까지
  battle-level test로 보강됐다
- 현재 남은 직접 구현 대상은
  `targeted homing skill projectile`를 더 리액티브한 runtime으로 올리는 것이 아니라,
  필요 시에만 그 전환을 다시 검토하는 것이다

즉 이 문서는 더 이상
“fixed projectile를 active runtime으로 옮길 수 있는가?”에 대한 가설 문서가 아니라,
“그 작업은 끝났고, 다음에 무엇을 보류/진행할 것인가?”를 적는 상태로 읽는 편이 맞다.

## Current Model

현재 모델은 projectile 종류에 따라 다르다.

1. `untargeted fixed skill projectile`
   - 발사 시 `ActiveProjectileRuntime`를 spawn한다
   - `BattleEvent::SkillProjectileAdvance`에서 1ms tick reevaluation을 수행한다
   - 실제 hit가 확정됐을 때만 `SkillProjectileImpact`를 발행한다
2. `targeted homing skill projectile`
   - 여전히 launch-time scheduled delivery다
   - impact 시점에 대상 생존 여부와 impact position은 다시 샘플링하지만,
     비행 중 active reevaluation은 하지 않는다

즉 현재 codebase는

- fixed projectile는 “전장 안에 살아 있는 runtime object”
- homing projectile는 “지정 대상을 향한 예약 delivery”

가 공존하는 하이브리드 상태다.

## Current Strengths

- 구현이 단순하다
- 결정론 유지가 쉽다
- 현재 event queue 모델과 잘 맞는다
- replay/timeline과 일관성이 높다
- fixed projectile, homing projectile, piercing까지 이미 붙어 있다

## Current Weaknesses

남은 핵심 약점은 **homing projectile가 발사 이후 전장 변화에 덜 반응적**이라는 점이다.

예:

- 발사 후 대상이 CC에 걸려 이동 경로가 바뀜
- 발사 후 대상이 죽음
- 발사 후 blocker가 새로 경로에 들어옴
- 발사 후 same-tick movement reorder가 발생함

현재 fixed projectile는 이 변화를 충분히 반영한다.
반면 homing projectile는 여전히 launch-time scheduled contract에 머무른다.

정리하면:

- movement는 계속 변한다
- homing projectile는 여전히 발사 시점 예약 비중이 크다

그래서 앞으로 이 문서가 다시 열릴 일은
거의 homing projectile contract를 더 리액티브하게 바꿔야 할 때다.

## Feasibility Assessment

### Conclusion

**실제로 구현 가능하다.**

이유:

1. movement는 이미 continuous segment 기반이다
   - `MovementSegmentStarted`
   - `sample_unit_position_at(...)`
2. event queue 기반 전투 루프가 이미 존재한다
3. skill projectile / area runtime skeleton도 이미 존재한다
4. projectile impact는 이미 `SkillProjectileImpact` 이벤트로 분리돼 있다

즉 필요한 것은 새 엔진이 아니라,
**projectile를 “예약 이벤트”에서 “active runtime object”로 승격하는 것**이다.

### What Makes It Hard

어려운 점은 계산량보다도 **결정론 규칙 설계**다.

active projectile runtime은 매번 전장 상태를 다시 읽기 때문에,
아래를 명확히 못 박아야 한다.

- 언제 재계산하는가
- 어떤 phase 뒤에 재계산하는가
- 둘 이상 동시에 맞으면 누구를 먼저 맞는가
- 좌표 계산 반올림 규칙은 무엇인가

이 규칙이 없으면 같은 seed에서도 결과가 달라질 수 있다.

## Recommended Runtime Model

### 1. Add `ActiveProjectileRuntime`

새 runtime state를 추가한다.

필수 필드:

- `delivery_id`
- `cast_seq`
- `step_index`
- `skill_id`
- `step_id`
- `caster_instance_id`
- `caster_owner`
- `spawned_at_ms`
- `start`
- `current_position`
- `aim`
- `speed_units_per_ms`
- `guidance`
- `collision`
- `hit_unit_ids`
- `next_reevaluation_ms`
- `max_travel_ms`

중요:

- 위치는 누적 갱신보다
  `spawned_at_ms + elapsed` 기준 재계산이 더 안전하다
- 누적 오차가 적고 replay/debug에 유리하다

### 2. New Event Model

현재는 발사 시 `SkillProjectileImpact`를 바로 예약한다.

새 모델은 이렇게 간다.

- `BattleEvent::SkillProjectileAdvance`
  - projectile reevaluation/tick 이벤트
- `BattleEvent::SkillProjectileImpact`
  - 실제 충돌이 확정됐을 때만 발행
- 선택적으로 `SkillProjectileExpire`
  - 경로 끝 도달 / no-hit 종료를 명시하고 싶다면 추가

### 3. Reevaluation Triggers

가장 현실적인 모델은 **hybrid**다.

즉:

- 일정 주기 tick
- 중요한 전장 변화 이벤트

둘 다 쓴다.

권장 트리거:

- `projectile_tick_ms`
  - 예: 16ms 또는 33ms
- `MovementSegmentStarted`
- `MovementStopped`
- `UnitDied`
- hard CC 상태 변화

이때 “트리거가 왔으니 즉시 계산”이 아니라,
**같은 time bucket 안에서 phase를 고정한 뒤 projectile reevaluation**을 해야 한다.

## Determinism Contract

active projectile runtime으로 가려면 아래 규칙을 명시해야 한다.

### 1. Reevaluation Phase Order

같은 `time_ms`에서는 다음 순서를 강제한다.

1. movement/battle state updates 적용
   - death
   - movement commit
   - segment start/stop
   - CC state update
2. active projectile reevaluation
3. `SkillProjectileImpact` 생성
4. impact resolution / damage / step effect
5. 추가 파생 이벤트 enqueue

즉 projectile는 **settled board state**를 읽어야 한다.

### 2. Tie-Break Rules

같은 reevaluation에서 둘 이상 충돌 후보가 있으면:

1. projectile path 상 더 이른 거리
2. 동일 거리면 더 작은 거리 제곱
3. 동일 거리면 `UnitInstanceId` byte order

를 쓴다.

### 3. Integer / Fixed Rules

부동소수점 대신 지금처럼 integer unit 기반을 유지한다.

- `ContinuousPosition` 그대로 사용
- 속도는 `units_per_ms`
- 거리/충돌은 integer / distance_sq 기반

### 4. Same-Target Hit Rule

projectile 하나가 같은 유닛을 여러 번 맞히지 않는 규칙은 유지한다.

- `hit_unit_ids` authoritative
- `piercing + max_hits`와 조합

## Geometry Model

충돌 판정은 현재처럼 “projectile 위치 + unit hitbox” 방식으로 간다.

즉:

- projectile path는 line/segment
- 유닛은 원(circle) hitbox
- 충돌은 segment/circle 또는 point/circle 판정

다만 2차 구현으로 갈수록,
지금의 `1ms sampling`은 analytic collision으로 바꿀 여지가 있다.

권장 순서:

1. 먼저 active runtime + reevaluation 구조를 붙인다
2. 그 뒤 필요하면 `segment-circle` analytic collision으로 바꾼다

즉 **모델 변경과 수학 최적화를 한 번에 하지 않는다.**

## Implementation Plan

### Phase A: Runtime Skeleton

목표:

- launch-time scheduled impact를 대체할 active projectile state 도입

작업:

- `types.rs` 또는 새 module에 `ActiveProjectileRuntime` 추가
- `BattleCore`에 `active_projectiles: HashMap<Uuid, ActiveProjectileRuntime>` 추가
- `BattleEvent::SkillProjectileAdvance` 추가
- launch 시 immediate impact scheduling 대신 active projectile spawn

현재 상태:

- 완료
- `ActiveProjectileRuntime`
- `BattleEvent::SkillProjectileAdvance`
- `BattleCore.active_projectiles`
  까지 모두 도입됨

### Phase B: Fixed Projectile Reevaluation

목표:

- `untargeted fixed projectile`만 먼저 active runtime으로 전환

작업:

- reevaluation마다 current projectile position 계산
- 그 시각의 unit position을 `sample_unit_position_at(...)`로 조회
- first hit / piercing / miss 결정
- hit이면 `SkillProjectileImpact`
- no-hit면 다음 advance 예약 또는 expire

현재 상태:

- 완료
- fixed skillshot은 발사 후 movement / blocker / death / hard CC 변화를
  reevaluation에서 다시 읽는다
- same-time phase ordering도 projectile reevaluation이 settled board state를 읽도록 맞춰졌다
- regression:
  - blocker enters path
  - original target dies, later unit hit
  - post-launch hard CC changes hit result
  - piercing / max_hits
    까지 보강됨

### Phase C: Homing Projectile Reevaluation

목표:

- `targeted homing skill projectile`도 active runtime으로 통일

작업:

- target alive 여부 재평가
- target current position 재샘플
- target lost/dead 시 impact cancel 또는 expire
- impact position도 launch snapshot이 아니라 reevaluated target position 사용

완료 기준:

- targeted projectile가 실제로 더 “유도체”처럼 동작

현재 상태:

- 아직 미구현
- 다만 현재 우선순위에서는 **보류**가 맞다

보류 이유:

- `targeted homing skill projectile`는 현재 게임 계약상
  “지정 대상을 향한 delivery” 성격이 강하다
- fixed projectile처럼 경로 중간 충돌 / blocker / re-hit case가 핵심 문제는 아니다
- 따라서 실제 요구가 생기기 전까지는
  fixed projectile와 같은 수준의 active runtime으로 올리지 않는 편이 리스크가 낮다

### Phase D: Cleanup

목표:

- old launch-time impact prediction helper 제거

작업:

- `compute_fixed_skill_projectile_impacts(...)` 제거
- `compute_homing_skill_projectile_impacts(...)` 제거
- `schedule_skill_projectile_impact_event(...)`는 launch helper가 아니라
  resolved impact enqueue helper로 축소

현재 상태:

- fixed helper cleanup은 사실상 완료에 가깝다
- 남은 직접 cleanup 대상은 homing launch-time helper를 언제 정리할지다
- homing을 유지하기로 하면 helper를 급하게 제거할 이유는 없다

## Why This Is The Right Scope

이 계획은 projectile만 바꾼다.

바꾸지 않는 것:

- orchestrator
- movement segment contract
- basic attack homing rule
- area runtime

즉 리스크를 통제하면서도,
현재 projectile 모델의 가장 큰 약점만 직접 해결한다.

## Non-Goals

이번 설계의 목표가 아닌 것:

- full physics simulation
- rigidbody-like continuous collision world
- obstacle / wall bounce / reflection
- general-purpose navmesh projectile

지금 필요한 건 LoL/TFT식 believable projectile 반응성이지,
물리 엔진이 아니다.

## Remaining Work

지금 시점에서 이 문서 기준의 남은 직접 작업은 아래뿐이다.

1. `targeted homing skill projectile`를 active runtime으로 올릴지 여부를
   실제 요구가 생길 때 다시 판단
2. fixed projectile의 1ms tick reevaluation을
   hybrid trigger로 확장할 실익이 있는지 검토
3. 필요할 경우만 homing launch-time helper cleanup 진행

권장 우선순위:

- 지금은 projectile를 더 확장하지 말고
- live skill migration / area expressiveness / structural cleanup 쪽으로 돌아간다

## Final Recommendation

최종 판단:

- **이 설계는 실제로 구현 가능하다**
- 현재 codebase 구조와도 맞다
- 다만 launch-time prediction 위에 땜질하지 말고
  `active projectile runtime`을 별도 vertical slice로 도입해야 한다

권장 판단:

- fixed projectile active runtime 단계는 완료로 본다
- `targeted homing skill projectile`는 현재 계약을 유지한다
- basic attack과 area runtime은 그대로 둔다
- 다음 직접 작업은 projectile 자체보다
  broader skill spatial runtime migration과 선택적 refinement다
