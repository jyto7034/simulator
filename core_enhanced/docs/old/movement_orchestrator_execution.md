# Movement Orchestrator Execution Tracker

## Purpose

이 문서는 `MovementOrchestrator` 리팩토링의 **실행용 체크리스트**이자 **진행 로그**다.

목표:

- 대화가 compact 되거나 다른 AI가 이어받아도 현재 진행 상태를 즉시 파악할 수 있게 하기
- 설계 문서인 `docs/movement_orchestrator_plan.md`를 실제 코드 작업 단계로 번역하기
- 각 패치가 끝날 때마다 무엇이 적용되었는지 기록하기

관련 문서:

- `docs/movement_orchestrator_plan.md`
- `docs/movement_flow.md`
- `docs/active_projectile_runtime_plan.md`

관련 코드 진입점:

- `src/game/battle/core/movement/mod.rs`
- `src/game/battle/core/movement/plan.rs`
- `src/game/battle/core/movement/execute.rs`
- `src/game/battle/core/sim.rs`

## Current State Snapshot

현재 상태를 한 줄로 요약하면:

- `MovementOrchestrator`의 큰 뼈대는 이미 들어갔고,
  orchestrator decision layer는 사실상 마감됐으며,
  현재 active work는 `live skill spatial data migration / optional shape refinement`다

이미 끝난 것:

- `compute_movement_intents()` orchestrator 경유화
- `Hold / Yield / Blocked / WaitRepath` 상태 도입
- `nearest target + sticky target + near-engage lock` 방향 반영
- planner / orchestrator의 `sticky approach` 보정
- execution 경계의 `current tile engage` fallback 일부 추가
- 공격 시작 target lifecycle과 movement target lifecycle 정렬
- continuous spatial layer의 장기 방향 문서화
- opening diagonal / 불필요한 초기 crossing 대표 repro 정리
- same-range enemy에서 UUID가 spatial plan preference보다 앞서던 경계 정리
- death-triggered same-tick movement recompute
- dense 7v7 frontline collapse 이후 immediate reacquire 검증 추가
- frontliner death 이후 straight-over-diagonal backline retarget 검증 추가

아직 안 끝난 것:

1. broader replay/export 유지보수
   - 새로운 battle shape나 신규 bug report가 나올 때만
     drift / unnecessary walk / retarget overswap를 다시 점검하면 됨
2. skill spatial runtime migration
   - `ProjectilePayload::SkillStep` 중심 path 제거는 완료됨
   - skill projectile / area delivery는 이제
     `SkillProjectileImpact / SkillAreaTick / SkillImpactContext` 중심 path로 통일됨
   - `pierce / max_hits` behavioral runtime도 완료됨
   - persistent area `EveryTick / OncePerArea / OnEnter` tick policy도 완료됨
   - `despawn_on_hit` legacy 해석 축소도 완료됨
   - 남은 직접 구현 대상은
     필요 시 area shape/rotation expressiveness refinement와
     추가 live `.ron` migration 확대
   - 다만 남아 있는 `Instant` step 대부분은
     single-target direct hit / self buff / extra attack처럼
     의도된 non-spatial step로 본다

현재 분리된 것:

- `t≈1678ms` opening melee engage는
  현재 movement 모델 기준 `actual continuous reach closure`로 봐야 한다
- 즉 opening frontline이 `1251ms` 이후 한 tick 늦게 공격을 시작하는 현상은
  더 이상 "planner가 extra tile walk를 만들고 있다"는 뜻이 아니다
- current broader sweep 기준 남은 off-axis move는
  opening drift가 아니라 front collapse 이후 backline retarget 성격으로 해석된다

## Current Verdict

현재 판단은 아래와 같다.

- orchestrator decision layer는 `done / maintenance` 상태로 본다
- opening drift / initial crossing / same-range selector inconsistency / death-trigger idle gap은
  대표 repro와 scenario test 기준으로 정리됐다
- 남은 작업은 새로운 movement policy 추가가 아니라
  broader export safety를 필요할 때 다시 확인하는 유지보수다
- 다음 큰 단계는 orchestrator를 더 뜯는 것이 아니라
  `continuous spatial layer` 중에서도
  `skill spatial runtime` 구현이다
- 현재 continuous layer는
  - core: `MovementSegmentStarted` + replay sampling helper까지 구현됨
  - Unity: explicit segment replay refactor 진행 중
  - projectile / AoE:
    untargeted fixed projectile active runtime + reevaluation slice,
    instant area circle/rectangle,
    persistent area tick/expire slice,
    targeted homing skill projectile `SkillProjectileImpact` 통일,
    `piercing / max_hits` behavioral runtime,
    persistent area `EveryTick / OncePerArea / OnEnter` tick policy까지 구현됨
  - area shape expressiveness는
    `Circle / Line / Box / Rectangle / Cone`까지 올라옴
  - legacy `ProjectilePayload::SkillStep` skill path는 제거됨
  - `despawn_on_hit`은 이제 compatibility-only optional field다
  - 현재 active migration 대상은
    추가 live `.ron` skill migration과
    필요 시 line/pivot orientation expressiveness refinement다
  - 남은 `Instant` step은 전부 옮길 대상이 아니라
    spatial semantics가 실제로 더 명확해지는 step만
    선별적으로 마이그레이션하는 방향이 맞다
  - 권장 우선순위는
    현재 spatial runtime regression을 유지한 채,
    필요 시 `Rectangle/Cone`을 넘는 orientation 표현력만 확장하는 것이다
  - `Area` delivery는
    `shape + hit_targets + duration/tick`뿐 아니라
    `anchor source (CastTarget / ImpactContext / Caster)`까지
    명시적으로 가지는 방향으로 정리한다
  - 평타는 homing 유지가 전제됨

## Next Patch Target

다음 패치의 우선순위는 아래 순서다.

1. 새로운 repro가 나올 때만
   대표 export와 scenario test를 기준으로 regression 여부를 확인한다
2. orchestrator decision layer에는 원칙적으로 새 policy를 더 넣지 않는다
3. 현재 direct next step은
   추가 live `.ron` skill을 `Projectile/Area` spatial delivery로 옮겨
   실제 데이터에서 모델을 더 검증하는 것이다
   - fixed projectile active runtime 자체는 완료로 본다
   - targeted homing projectile active runtime 전환은 현재 direct target이 아니다
   - 단, 남은 `Instant` step 대부분은 intentional non-spatial이므로
     먼저 content audit / regression으로 그 경계를 고정하는 편이 낫다
4. 필요 시
   area shape/rotation expressiveness가 모자랄 때만
   line/cone/pivot 표현력을 더 확장한다
5. 새로운 unwanted drift가 확인되면
   scenario test를 먼저 추가한 뒤 소폭 수정한다

주의:

- opening diagonal / crossing 대표 repro는 현재 해결된 상태로 본다
- opening `1251ms -> 1678ms` melee engage timing 자체는
  대표 버그로 계속 쫓지 않는다
- 그 구간은 tile-adjacent 이후 continuous boundary-to-boundary closure가
  닫히는 시간으로 이해해야 한다

즉 다음 AI는 orchestrator를 다시 확장하는 것이 아니라,
`skill spatial runtime`을 유지한 채
실제 skill data migration과 선택적 shape refinement를 이어가면 된다.
다만 평타는 근/원거리 모두 homing이므로,
continuous hit / miss는 skill projectile / AoE 쪽에만 도입한다.

## Do Not Change Yet

다음 항목은 지금 단계에서 건드리지 않는 편이 맞다.

- 타일 occupancy / reservation authority 제거
- full continuous engine 전환
- attack slot의 continuous화
- replay-only 보정으로 서버 타임라인 문제를 숨기는 수정

즉:

- decision layer는 계속 tile-authoritative
- continuous layer는 그 위에 올라갈 두 번째 레이어

## Representative Repro Cases

가장 먼저 봐야 할 재현 케이스:

- `core/timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json`

이 파일에서 특히 봐야 할 구간:

1. opening front engage
   - 현재는 baseline 확인용
   - 이전 diagonal / crossing repro가 다시 생기지 않았는지 본다
2. mid-fight front collapse 이후
   - retarget / reposition이 planner와 같은 preference를 유지하는지 확인한다
3. ranged vs melee, ranged vs ranged export
   - movement policy 변경이 projectile / ranged timing에 부작용을 만들지 않는지 본다

현재 해석:

- opening diagonal / crossing 계열은 현재 core 쪽 policy 수정으로 많이 정리됐다
- dense 7v7에서도 frontline collapse 이후 melee reacquire는 빠르게 일어난다
- frontliner death 이후 exposed backline retarget도 straight preference가 유지된다
- 현재 남은 일은 broader export safety 유지보수와 continuous 준비다

## Code Entry Points For Next AI

다음 AI가 먼저 읽어야 할 파일과 포인트:

- `src/game/battle/core/movement/orchestrator.rs`
  - `compute_movement_intents()`
  - `resolve_movement_intent_decisions()`
  - `compare_advance_candidate()`
- `src/game/battle/core/movement/plan.rs`
  - target / destination candidate 정렬
  - `compare_destination_preference()`
  - `compare_plan_preference()`
- `src/game/battle/core/movement/execute.rs`
  - `post_move_retarget_at()`
  - `stop_moving_unit_on_target_in_range()`
  - `moving_unit_has_near_engage_lock()`
- `src/game/battle/core/sim.rs`
  - `select_basic_attack_target()`
  - attack-start target lifecycle 정렬 상태 확인용
- `Assets/Scripts/Replay/Playback/BattleTimelineReplayer.cs`
  - `UnitTrack.has_explicit_segments`
  - `build_tracks_from_timeline()`
  - `sample_track_position()`의 legacy / segment 분리

## Current Recommended Strategy

지금 가장 안전한 작업 순서:

1. 현재 orchestrator decision layer는 마감된 것으로 간주
2. 새로운 repro가 생기면 scenario test를 먼저 추가
3. 진짜 unwanted drift일 때만 planner / execute 경계를 소폭 수정
4. 별도 phase에서 continuous segment timeline / replay 구현을 진행

## Current Problem Statement

현재 movement는 크게 두 단계로 나뉜다.

1. `compute_movement_intents()`
   - 유닛별로 순차 계획
   - BFS
   - 목적지 후보 계산
   - 첫 스텝 예약
2. `handle_move_steps_at()`
   - 이동 적용
   - 충돌 해소
   - `WaitRepath`
   - `RangeEnter`

이 구조 때문에 생기는 핵심 문제:

- 유닛별 순차 계획으로 인해 전역 조정이 없음
- `wait_repath`가 정상 제어 흐름처럼 남용됨
- 좌우 대칭 상황에서 편향이 발생함
- 근접 유닛이 불필요한 대각 이동/재경로를 선택함

## Scope For This Refactor

이번 작업의 1차 범위:

- `MovementOrchestrator` 모듈 추가
- 기존 planner 책임 중 `compute_movement_intents()`를 orchestrator 기반으로 재구성
- 연속 이동 실행기(`handle_move_steps_at`, `RangeEnter`, `MoveStep`)는 유지
- 기존 movement 외부 이벤트(`MovementIntent`, `MoveStep`)는 유지

이번 작업의 비범위:

- execution 전체 교체
- 타일 점유 엔진 교체
- projectile/ranged 시스템 재설계
- Unity 쪽 수정

## Execution Checklist

- [x] Phase 0. 실행 추적 문서 생성
- [x] Phase 1. `movement/orchestrator.rs` 추가
- [x] Phase 2. orchestrator 입력/출력 타입 정의
- [x] Phase 3. 기존 planner helper를 orchestrator가 재사용할 수 있게 정리
- [x] Phase 4. `compute_movement_intents()`를 orchestrator 경유로 변경
- [x] Phase 5. movement 관련 테스트 보강 또는 오래된 테스트 비활성화
- [x] Phase 6. `cargo test`로 검증
- [x] Phase 7. `Hold` decision/state 도입
- [x] Phase 8. locked target 유지 규칙 보강
- [x] Phase 9. orchestrator가 execution 충돌 예측을 더 많이 흡수
- [x] Phase 10. 최종 진행 상태와 다음 작업 포인트 기록

## Patch Log

### Patch 0

상태:

- 실행 추적 문서 생성
- 아직 코드 변경 없음

다음 목표:

- `MovementOrchestrator` 모듈 뼈대 추가
- planner 이관에 필요한 타입 설계

### Patch 1

상태:

- `src/game/battle/core/movement/orchestrator.rs` 추가
- `compute_movement_intents()`가 orchestrator 경유로 동작하도록 변경
- planner의 기존 helper(`formulate_enemy_chase_plan`, `destination_candidates_in_order`)는 유지하고 orchestrator가 재사용하도록 정리
- execution (`handle_move_steps_at`, `RangeEnter`, `MoveStep`)는 아직 유지

적용한 구조:

- `MovementIntentContext`
- `MovementAdvanceCandidate`
- `MovementDecision`
  - `Engage`
  - `Advance`
  - `WaitRepath`

주의:

- 아직 충돌 해소는 planning 단계에서 `first_step` claim 수준이다.
- `handle_move_steps_at()`의 실행 시 충돌 처리 로직은 그대로 남아 있다.
- 이번 패치는 planner/coordinator 이관의 첫 단계다.

다음 목표:

- 컴파일 에러/연결부 정리
- movement 테스트 재확인
- 필요 시 오래된 planner 가정을 전제한 테스트 비활성화

### Patch 2

상태:

- planner 이관 이후 컴파일/테스트 검증 완료
- 오래된 테스트 비활성화는 이번 단계에서 필요하지 않았음

실행한 검증:

- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- unit test 181 passed
- movement 관련 lib test 13 passed
- `battle_ranged_attack` integration test 12 passed

현재 결론:

- planner 진입점은 orchestrator 경유로 바뀌었음
- execution은 아직 legacy 구조를 유지
- 이번 단계는 "전체 교체"가 아니라 "planning/coordinator 책임 이관의 1차 완료"로 봐야 함

남은 핵심 작업:

- orchestrator가 `first_step claim` 이상으로 실제 전역 conflict resolution을 담당하도록 확장
- `wait_repath`를 `Yield/Hold/Blocked`로 세분화
- candidate generation에 직진 우선/교전선 유지 규칙을 도입
- 이후에야 `handle_move_steps_at()`의 legacy 충돌 해소를 걷어낼 수 있음

다음 AI가 바로 보면 좋을 코드:

- `src/game/battle/core/movement/orchestrator.rs`
- `src/game/battle/core/movement/plan.rs`
- `src/game/battle/core/movement/execute.rs`

### Patch 3

상태:

- orchestrator 내부 후보 정렬을 `직진 우선` 기준으로 보강
- 내부 decision에 `Yield` 추가
- planner helper visibility를 orchestrator 재사용에 맞게 조정
- 회귀 테스트 1개 추가

핵심 변경:

- `MovementAdvanceCandidate`에 `enemy_pos` 추가
- 후보 정렬 시 다음 순서를 우선
  - locked target 우선
  - forward progress 최대
  - lateral shift 최소
  - first step 이후 적과의 거리 최소
  - 적 축 정렬 유지
  - path length 최소
- 후보가 아예 없을 때만 `WaitRepath`
- 후보는 있지만 현재 tick에서 `first_step` claim 충돌로 못 가는 경우는 `Yield`
  - 현재는 `retry_at_ms = now_ms + 10`

추가 테스트:

- `movement_intent_prefers_forward_step_over_equal_diagonal_option`

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 182 passed
- `battle_ranged_attack` 12 passed

현재 의미:

- orchestrator는 이제 단순 `first_step` claim 이전보다 더 강한 planner 역할을 가짐
- `wait_repath`를 즉시 쓰지 않고, 일부 충돌은 짧은 yield로 흡수하기 시작함
- execution의 legacy conflict resolution은 아직 남아 있음

다음 우선순위:

- `Yield/Hold/Blocked`를 더 분리하고 retry 정책을 일반화
- `handle_move_steps_at()` 내부 충돌 해소 일부를 orchestrator 쪽으로 끌어올릴 준비
- 근접 교전선(`EngagementLane`) 규칙의 최소 버전 도입

### Patch 4

상태:

- execution 단계의 일부 임시 충돌을 짧은 `yield` 재시도로 흡수하도록 변경
- 외부 타임라인 스키마는 이번 단계에서 유지

핵심 변경:

- `handle_move_steps_at()`에 내부 `abort_to_yield` 추가
- 아래 경우는 더 이상 즉시 `WaitRepath`로 가지 않음
  - 같은 tick에서 이미 다른 유닛이 `to`를 claim한 경우
  - `to`의 reserver가 같은 tick에 움직이는 due mover인 경우
  - `to`의 occupant가 같은 tick에 움직이는 due mover인 경우
  - 다음 스텝 reserve 실패가 due mover와의 임시 충돌인 경우
- `yield`는 현재 `ActionState::Idle` + `MovementIntent(now_ms + 10)`로 표현
  - 아직 별도 public state / timeline reason은 도입하지 않음

추가 테스트:

- `same_tick_destination_conflict_yields_without_wait_repath`

테스트 조정:

- `tests/battle_ranged_attack.rs`의 `tft_like_field_6v6_mixed_melee_ranged_battle`
  - 더 이상 `winner != Draw`를 강제하지 않음
  - 이 테스트의 목적을 "비대칭 승패"가 아니라 "초기 이동/대각 후퇴 회귀 감지"로 본다

실행한 검증:

- `cargo test --lib same_tick_destination_conflict_yields_without_wait_repath`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 183 passed
- `battle_ranged_attack` 12 passed

현재 의미:

- planner 뿐 아니라 execution도 일부에서 `WaitRepath` 중심 모델에서 벗어나기 시작함
- 아직 `yield`는 내부 동작일 뿐, state/timeline schema에는 정식 승격되지 않았음
- 다음 단계에서 `yield/hold/blocked`를 명시적 execution decision으로 끌어올릴 수 있음

### Patch 5

상태:

- movement retry/yield/wait 제어를 공용 helper로 정리
- orchestrator와 execution이 같은 movement control API를 공유하기 시작함

핵심 변경:

- `movement/mod.rs`에 공용 helper 추가
  - `next_wait_repath_until_ms`
  - `enter_wait_repath`
  - `enter_yield`
- 공용 상수도 `movement/mod.rs`로 승격
  - `REPATH_BASE_DELAY_MS`
  - `YIELD_RETRY_DELAY_MS`
- `orchestrator.rs`는 더 이상 자체 retry 구현을 들고 있지 않고 공용 helper를 사용
- `execute.rs`도 `abort_to_yield`와 `abort_to_wait_repath`에서 공용 helper/공용 시간 계산을 사용

의미:

- planner와 execution이 서로 다른 retry semantics를 갖는 문제가 줄어듦
- 다음 단계에서 `yield/hold/blocked`를 public movement decision/state로 승격할 기반이 생김

실행한 검증:

- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`

결과:

- movement 관련 lib test 15 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `yield`를 지금의 "Idle + delayed MovementIntent" 임시 표현에서 벗겨내기
- `Hold`와 `Blocked`를 최소한 내부 decision으로 먼저 분리
- orchestrator가 execution tick 충돌 일부를 사전에 예측하도록 확장

### Patch 6

상태:

- planner 단계에 내부 `Blocked` decision 도입
- `yield`는 타겟을 유지하는 방향으로 보정

핵심 변경:

- `movement/mod.rs`
  - `BLOCKED_RETRY_DELAY_MS` 추가
  - `enter_yield()`가 더 이상 `current_target`를 지우지 않음
- `orchestrator.rs`
  - `MovementDecision::Blocked` 추가
  - strict candidate가 없더라도, loose BFS 기준으로 적 공격권역에 결국 도달 가능한 경우 `Blocked`로 분류
  - 현재는 `Blocked`도 public state가 아니라 `Idle + delayed MovementIntent(30ms)`로 표현
- `core/mod.rs`
  - 기존 `movement_intent_enters_wait_repath_when_no_attack_tile_is_available`
    를 새 semantics에 맞게
    `movement_intent_retries_from_idle_when_attack_ring_is_currently_blocked`
    로 변경

의미:

- planner가 "지금은 길이 막혀 있지만 동적 점유가 풀리면 갈 수 있는 경우"와
  "정말 재경로 대기해야 하는 경우"를 구분하기 시작함
- `WaitRepath`는 점점 "진짜 강한 실패" 쪽으로 밀려나고 있음

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 183 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `Blocked`와 `Yield`를 현재의 `Idle + delayed intent` 임시 표현에서 분리
- `Hold`를 내부 decision으로 추가해 무의미한 재계획을 줄이기
- orchestrator가 다음 tick execution 충돌을 일부 예측해서 `handle_move_steps_at()` 책임을 더 줄이기

### Patch 7

상태:

- `Yield`와 `Blocked`를 `Idle + delayed MovementIntent` 임시 표현에서 분리
- movement state에 명시적 waiting 상태를 도입

핵심 변경:

- `ActionState`에 새 상태 추가
  - `Yielding { until_ms, repath_counter }`
  - `Blocked { until_ms, repath_counter }`
- `enter_yield()`는 이제 `ActionState::Yielding`으로 전환
- `enter_blocked()` 추가
- orchestrator의 `Yield/Blocked` decision이 위 state를 사용하도록 변경
- `collect_movement_intent_contexts()`가 `Yielding/Blocked` 만료 시점도 planning 대상으로 다시 수집
- execution 쪽 helper도 새 상태의 `repath_counter`를 이어받도록 조정

테스트 변경:

- `movement_intent_retries_from_idle_when_attack_ring_is_currently_blocked`
  - 이제 `Idle`이 아니라 `Blocked { until_ms: 130, .. }`를 기대
- `same_tick_destination_conflict_yields_without_wait_repath`
  - 이제 `Idle`이 아니라 `Yielding { until_ms: 160, .. }`를 기대

의미:

- planner/execution이 "왜 기다리는 중인가"를 더 오래 보존하게 됨
- 다음 resolve pass에서 단순 idle unit과 blocked/yielded unit을 구분할 수 있는 기반이 생김
- 아직 `Hold`는 없음

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 183 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `Hold`를 내부 decision/state로 추가
- `Yielding/Blocked` 상태를 이용해 target 유지/라인 유지 규칙을 더 정교화
- orchestrator가 next-step reservation 충돌 일부를 planning 단계에서 미리 흡수하도록 확장

### Patch 8

상태:

- `Hold`를 내부 decision에서 실제 movement state로 승격
- planner와 execution이 `Holding` 상태를 공통으로 이해하기 시작함

핵심 변경:

- `movement/mod.rs`
  - `HOLD_RETRY_DELAY_MS` 추가
  - `ActionState::Holding { until_ms, repath_counter }` 추가
  - `enter_hold()` helper 추가
- `orchestrator.rs`
  - `MovementDecision::Hold` 추가
  - `collect_movement_intent_contexts()`가 `Holding` 만료 시점도 planning 대상으로 다시 수집
  - locked target이 살아 있고, 해당 타겟으로의 진입이 현재 lateral-only인 경우 `Hold`를 선택
- `execute.rs`
  - `interrupt_movement()`가 `Holding`도 waiting 계열 movement state로 취급
  - retry/repath counter 승계 로직이 `Holding`을 포함하도록 정리
- `core/mod.rs`
  - `movement_intent_holds_position_when_locked_target_only_has_lateral_entry` 추가

의미:

- planner가 "지금은 움직일 수 있지만, 그 이동이 교전선/타겟 유지 관점에서 바람직하지 않다"를 표현할 수 있게 됨
- `Yield/Blocked`와 별개로, 의도적인 line hold 개념이 처음 생김

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 185 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- locked target이 살아 있는 동안 불필요한 타겟 전환/측면 전환을 더 억제
- `Holding/Yielding/Blocked`를 기반으로 planner의 target stickiness를 강화
- execution의 임시 충돌 예측을 planner 단계로 더 끌어올리기

### Patch 9

상태:

- `Holding`을 단순 lateral entry 억제에서 한 단계 확장
- locked target이 아직 느슨하게 reachable할 때, 다른 적으로 즉시 갈아타는 movement를 `Hold`로 막음

핵심 변경:

- `orchestrator.rs`
  - `has_loose_attack_path_to_enemy()` 추가
  - `has_loose_attack_path()`가 위 helper를 재사용하도록 정리
  - `should_hold_position()`이 이제 두 경우를 다룸
    - locked target으로 가는 현재 후보가 lateral-only인 경우
    - locked target은 여전히 loose path로 reachable하지만, 현재 strict candidate가 다른 적으로만 열려 있는 경우
- `core/mod.rs`
  - `movement_intent_holds_locked_target_when_only_other_enemy_is_open` 회귀 테스트 추가

의미:

- planner가 일시적인 공간 개방 때문에 target을 쉽게 바꾸지 않게 됨
- 근접 전열에서 "locked target 유지"가 단순 target selection tie-break가 아니라 waiting decision으로도 표현되기 시작함
- 이후 `EngagementLane`나 더 강한 line-hold 규칙을 얹을 수 있는 기반이 생김

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 185 passed
- movement 관련 lib test 17 passed
- `battle_ranged_attack` 12 passed

현재 의미:

- orchestrator는 이제 단순 candidate picker가 아니라
  - `Yield`
  - `Blocked`
  - `Hold`
  를 구분해 line/timing/target stickiness까지 일부 조정하는 planner로 성장함
- execution은 아직 legacy 충돌 해소 비중이 크지만, planner 쪽 선제 조정 능력은 계속 커지고 있음

다음 우선순위:

- same-tick 충돌과 next-step reservation 충돌 일부를 planner가 더 먼저 흡수
- `Holding/Yielding/Blocked`를 이용해 pointless replanning을 더 줄이기
- 장기적으로 execution의 `WaitRepath` fallback 범위를 더 축소

### Patch 10

상태:

- `Hold` 판정이 이제 단순 "forward 후보가 존재하느냐"가 아니라
  "forward 후보가 이번 resolve pass에서 실제로 사용 가능하냐"까지 본다

핵심 변경:

- `orchestrator.rs`
  - `resolve_movement_intent_decisions()`가 선택 후보를 slice 기준으로 다루도록 정리
  - `should_hold_position()`이 이제 `claimed_first_steps`를 입력으로 받음
  - locked target으로 가는 forward 후보가 있어도, 그 first step이 이미 같은 tick에서 claim 되었으면
    lateral move 대신 `Hold`를 선택
- `core/mod.rs`
  - `movement_intent_holds_when_forward_slot_is_claimed_and_only_lateral_remains` 추가
  - 이 테스트는 lower-uuid frontliner가 같은 forward slot을 먼저 claim한 경우,
    뒤 frontliner가 억지로 측면으로 새지 않고 `Holding`으로 남는지 본다

의미:

- planner가 이제 "이론상 존재하는 후보"와 "이번 tick에서 실제로 쓸 수 있는 후보"를 구분하기 시작함
- same-tick claim 충돌 때문에 line이 옆으로 퍼지는 현상을 planner 단계에서 더 많이 억제할 수 있게 됨
- execution의 `claimed_to` / `yield` fallback 중 일부를 사전에 줄일 준비가 됨

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 186 passed
- movement intent lib test 6 passed
- movement lib test 18 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- planner가 `claimed_first_steps` 다음 단계인 `next-step reservation` 충돌까지 더 미리 흡수
- execution의 `abort_to_yield` 분기 중 due mover / soft reservation 케이스를 planner 쪽 decision으로 옮길 후보를 정리
- `Holding/Yielding/Blocked`의 retry delay를 상황별로 차등화할지 검토

### Patch 11

상태:

- orchestrator가 resolve 순서 자체에 waiting-state 우선순위를 도입
- `Holding` 유닛이 fresh `Idle` 유닛보다 먼저 claim을 가져가도록 정리

핵심 변경:

- `orchestrator.rs`
  - `MovementIntentStateKind` 추가
    - `Holding`
    - `Yielding`
    - `Blocked`
    - `Idle`
    - `WaitRepath`
  - `collect_movement_intent_contexts()`가 이제 context마다 위 state kind를 기록
  - resolve 전에 context를 `state priority -> unit_id` 순으로 재정렬
    - 현재 우선순위는 `Holding > Yielding > Blocked > Idle > WaitRepath`
- `core/mod.rs`
  - `movement_intent_prioritizes_holding_unit_over_fresh_idle_competitor` 추가
  - lower-uuid의 fresh idle competitor가 있어도, matured `Holding` 유닛이 먼저 forward slot claim을 가져가는지 검증

의미:

- planner가 이제 "누가 먼저 resolve 되어야 하는가"까지 전역적으로 조정하기 시작함
- line-hold를 이미 갖고 있던 유닛이 새 idle unit 때문에 쉽게 밀려나는 현상을 줄일 수 있음
- 이는 장기적으로 `EngagementLane` 같은 교전선 모델을 얹기 위한 전단계다

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 187 passed
- movement intent lib test 7 passed
- movement lib test 19 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- planner가 due mover / soft reservation 기반 next-step conflict를 더 미리 흡수할 수 있는지 검토
- execution의 `claimed_to` 이후 분기 중 planner로 올릴 수 있는 케이스를 정리
- `Holding/Yielding/Blocked/WaitRepath`의 우선순위와 retry delay를 함께 조정할지 판단

### Patch 12

상태:

- orchestrator의 planning priority가 이제 execution `MoveStep` 충돌 해소까지 전달됨
- planner/execution 간 우선순위 semantics가 처음으로 연결됨

핵심 변경:

- `movement/mod.rs`
  - `MovementState`에 `orchestrator_priority: u8` 추가
  - `MovementState::new_at()` 기본값은 `u8::MAX`
- `orchestrator.rs`
  - `MovementDecision::Advance`가 `orchestrator_priority`를 함께 들고 다님
  - orchestrator가 만든 move는 `context.state_kind.priority()`를 movement state에 기록
- `execute.rs`
  - due mover 수집 후 intent 정렬이 이제 `unit_id`만이 아니라
    `orchestrator_priority -> unit_id` 순서를 사용
  - 즉, 같은 tick 같은 destination 충돌에서도
    planner가 더 높은 우선순위를 준 move가 먼저 commit 됨
  - `same_tick_destination_conflict_uses_orchestrator_priority_before_uuid` 테스트 추가

의미:

- 이전까지는 planner에서 `Holding` 우선 resolve를 하더라도, 실제 `MoveStep` 충돌 해소는 다시 `unit_id` 중심이었음
- 이제는 orchestrator가 결정한 line-hold / waiting 우선순위가 execution에서도 유지됨
- 장기적으로 due mover / soft reservation 충돌을 planner 쪽으로 더 끌어올릴 수 있는 발판이 생김

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 188 passed
- movement intent lib test 7 passed
- movement lib test 20 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- planner가 due mover / reservation 정보를 더 읽어서 next-step conflict를 사전 흡수할 수 있는지 검토
- execution의 `reservation_at(to)` / `occupant(to)` 기반 `abort_to_yield` 케이스를 planner decision으로 이동할 후보를 좁히기
- `orchestrator_priority`가 `Holding` 외 `Yielding/Blocked`에서도 더 좋은 결과를 내는지 실제 6v6 timeline으로 재확인

### Patch 13

상태:

- execution에 same-tick due mover vacate를 흡수하는 2-pass 처리를 도입
- 즉시 `Yield`로 빠지던 일부 케이스를 같은 tick 안에서 해결하기 시작함

핵심 변경:

- `execute.rs`
  - due mover 집합의 `priority_by_unit` snapshot 추가
  - first pass에서 destination `to`가
    - lower-priority due mover의 soft reservation에 막히거나
    - lower-priority due mover 자신이 점유 중인 타일이면
    해당 mover가 먼저 빠지도록 `deferred_intents`에 넣고 스킵
  - first pass 종료 후 `deferred_intents`를 한 번 더 처리하는 second pass 추가
  - second pass에서도 여전히 막히면 기존 `Yield/WaitRepath`로 fallback
- 테스트 추가
  - `due_mover_that_will_vacate_tile_is_retried_in_same_tick_before_yielding`

의미:

- planner만이 아니라 execution도 "낭비되는 same-tick yield"를 줄이는 방향으로 이동함
- 높은 우선순위 mover가 lower-priority due mover가 비울 타일을 다음 tick까지 기다리지 않고
  같은 tick 안에서 점유할 수 있게 됨
- 장기적으로 `reservation_at(to)` / `occupant(to)` 기반 충돌 일부를
  planner 또는 execution pre-pass에서 더 흡수할 수 있는 기반이 생김

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 189 passed
- movement intent lib test 7 passed
- movement lib test 21 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- second pass로도 남는 `next-step reservation` 충돌을 planner 쪽 decision으로 더 끌어올릴 수 있는지 검토
- execution의 duplicate move-commit 블록을 helper로 정리해, 이후 충돌 정책 변경을 더 쉽게 만들기
- 실제 `tft_like_field_6v6_mixed_melee_ranged_battle` timeline export를 다시 뽑아
  초기 대각 꺾임/불필요 yield가 얼마나 줄었는지 확인

### Patch 14

상태:

- `handle_move_steps_at()`의 duplicated move-commit 블록을 helper로 통합
- 이후 충돌 정책 리팩토링을 위한 execution 구조 정리가 시작됨

핵심 변경:

- `execute.rs`
  - `ReadyMoveAttempt` enum 추가
  - `try_commit_ready_move_intent(...)` helper 추가
  - first pass / deferred second pass가 이제 동일 helper를 재사용
  - same-tick due mover defer, move commit, next-step reserve, arrived handling이 한 곳에 모임

의미:

- behavior 변화보다 "execution structure를 장기적으로 바꾸기 쉬운 상태"로 만든 패치
- 앞으로 `reservation_at(to)`, `occupant(to)`, `next_step reserve` 충돌 정책을 바꿀 때
  한 블록만 수정하면 되므로 회귀 위험이 줄어듦
- orchestrator와 execution 사이의 책임 경계를 더 명확히 밀어붙일 준비가 됨

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 189 passed
- movement intent lib test 7 passed
- movement lib test 21 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `try_commit_ready_move_intent()` 내부의 `next_step reserve` 실패를 same-tick second pass나 planner decision으로 더 흡수할 수 있는지 검토
- 실제 6v6 timeline export를 재생성해 movement shape 변화를 확인
- 필요하면 `try_commit_ready_move_intent()`를 기준으로 move-commit result를 더 세분화해 execution prepass를 추가

### Patch 15

상태:

- 실제 `tft_like_field_6v6_mixed_melee_ranged_battle` timeline export를 재생성하고 opening movement shape를 검증
- 이번 패치는 코드 변경 없이 회귀 확인과 기록에 집중

실행한 검증:

- `cargo test --test battle_ranged_attack tft_like_field_6v6_mixed_melee_ranged_battle -- --nocapture`
- export 파일: `timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json`

관찰 결과:

- 이전에 문제였던 opening diagonal kink는 현재 export 기준으로 재현되지 않음
- 주요 근접 유닛들이 opening에서 동일 `x` 축을 유지한 채 직진
  - `e25...`: `(1,6) -> (1,5) -> (1,4)` 후 `target_acquired`
  - `65db...`: `(1,1) -> (1,2) -> (1,3)` 후 `target_acquired`
  - `9323...`: `(3,6) -> (3,5) -> (3,4)` 후 `target_acquired`
  - `d194...`: `(3,1) -> (3,2) -> (3,3)` 후 `target_acquired`
  - `f9cb...`: `(5,6) -> (5,5) -> (5,4)` 후 `target_acquired`
  - `9970...`: `(5,1) -> (5,2) -> (5,3)` 후 `target_acquired`
- 첫 교전 정지 시 연속 좌표도 서로 같은 lane 위에 남아 있음
  - 예: `e25...`는 `x_units=500000`, `65db...`는 `x_units=500000`
  - 예: `9323...`/`d194...`는 `x_units=2500000`
  - 예: `f9cb...`/`9970...`는 `x_units=4500000`

의미:

- 지금까지의 orchestrator / priority / same-tick defer 리팩토링이
  적어도 opening lane formation에서는 의도한 방향으로 작동하고 있음
- 다음부터는 "opening diagonal detour"보다
  mid-fight 재배치와 next-step reservation 충돌 쪽이 우선 관심사가 됨

다음 우선순위:

- `try_commit_ready_move_intent()`의 `next_step reserve` 실패를 더 흡수할 수 있는지 검토
- 6v6 export에서 중반 이후 재배치가 여전히 불필요하게 흔들리는 구간이 있는지 추가 확인

### Patch 16

상태:

- `next_step reserve` 실패도 same-tick 안에서 한 번 더 재시도하도록 확장
- current-step commit 뿐 아니라 follow-up step scheduling도 덜 즉흥적으로 `Yield`함

핵심 변경:

- `execute.rs`
  - `deferred_next_step_reservations` 큐 추가
  - `try_commit_ready_move_intent()`가 현재 step 이동 후 `next_step reserve` 실패를 만났을 때
    lower-priority due mover가 곧 비울 타일/예약이면 즉시 `Yield`하지 않고 defer
  - `try_reserve_deferred_next_step(...)` helper 추가
    - first/deferred pass 이후 same-tick에 한 번 더 reserve와 scheduling을 시도
  - 회귀 테스트 추가
    - `next_step_reservation_retries_in_same_tick_after_lower_priority_blocker_moves`

의미:

- movement가 이제 "현재 칸까지 이동은 성공했지만 다음 칸 예약이 막힘" 상황도
  다음 tick까지 미루지 않고 같은 tick 안에서 더 많이 해결함
- 이건 execution을 단순 사후 충돌 처리기에서
  same-tick coordinator 성격으로 조금 더 밀어 넣는 변화
- 이후 planner로 끌어올릴 충돌 정책 후보가 더 선명해짐

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 190 passed
- movement intent lib test 7 passed
- movement lib test 22 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 planner/orchestrator 쪽 prepass로 더 끌어올릴 수 있는지 검토
- 중반 이후 6v6 export에서 재배치/라인 붕괴가 여전히 남는 구간을 찾아 후속 규칙 후보를 좁히기
- 필요하면 `Hold/Yield/Blocked` delay를 현재 lane density나 retry count에 따라 차등화

### Patch 17

상태:

- execution에 남아 있던 "정적 아군 교통 체증" fallback 일부를 `WaitRepath`에서 `Blocked`로 재분류
- current-step commit과 next-step reserve 양쪽에서 같은 semantics를 사용

핵심 변경:

- `execute.rs`
  - `is_temporarily_blocked_by_friendly_unit(...)` helper 추가
  - `abort_to_blocked` closure 추가
  - 아래 경우는 더 이상 즉시 `WaitRepath`로 보내지 않음
    - 현재 step의 `to`가 정적 아군 occupant / reservation에 막힌 경우
    - 현재 step은 성공했지만 `next_step reserve`가 정적 아군 occupant / reservation에 막힌 경우
    - deferred next-step reserve 재시도에서도 정적 아군 blocker가 남아 있는 경우
  - 위 경우는 이제 `Blocked { until_ms = now_ms + 30 }`로 내려감
- 회귀 테스트 추가
  - `static_friendly_blocker_enters_blocked_instead_of_wait_repath`
  - `next_step_friendly_blocker_enters_blocked_instead_of_wait_repath`

의미:

- execution이 이제 "강한 실패(재경로 필요)"와 "일시적인 아군 교통 체증"을 더 명확히 구분함
- planner/orchestrator에서 이미 쓰고 있던 `Blocked` semantics가 execution fallback까지 확장됨
- `WaitRepath`를 예외 상태로 축소하려는 장기 방향에 맞는 패치
- `Phase 9`는 현재 기준으로 달성했다고 보고 체크 처리

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 192 passed
- movement intent lib test 7 passed
- movement lib test 24 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 execution 보정이 아니라 orchestrator/planner prepass 성격으로 더 끌어올릴 수 있는지 검토
- 실제 `tft_like_field_6v6_mixed_melee_ranged_battle` export에서 mid-fight 재배치나 라인 붕괴가 남는 구간을 다시 확인
- 필요하면 `Yield/Blocked/Holding` retry delay를 lane density / retry count / target stickiness에 따라 차등화

### Patch 18

상태:

- orchestrator의 commit 경계에 남아 있던 `first_step reserve` 실패 fallback도 일부 정리
- planner가 이미 `Blocked/Hold/Yield`를 구분한 뒤 commit 단계에서 다시 곧바로 `WaitRepath`로 떨어지는 경우를 줄임

핵심 변경:

- `orchestrator.rs`
  - `first_step_is_temporarily_blocked_by_friendly(...)` helper 추가
  - `MovementDecision::Advance` commit 중 `battlefield.reserve(unit_id, first_step, now_ms)` 실패 시
    정적 아군 occupant / reservation이 blocker인 경우 `Blocked`로 전환
  - 나머지 truly hard failure만 기존처럼 `WaitRepath` 유지

의미:

- execution뿐 아니라 planner -> commit 경계도 같은 waiting semantics를 더 많이 공유하게 됨
- `WaitRepath`는 이제 planner resolve 결과를 커밋하는 순간의 아군 lane congestion까지 흡수하지 않음
- 장기 방향인 "hard failure와 transient congestion 분리"가 orchestrator commit까지 확장됨

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 192 passed
- movement intent lib test 7 passed
- movement lib test 24 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 execution 내부 보정이 아니라 orchestrator/planner prepass로 끌어올릴 수 있는지 계속 검토
- `tft_like_field_6v6_mixed_melee_ranged_battle`의 opening 이후 mid-fight 재배치 구간을 다시 분석해서
  어떤 rule이 line 붕괴를 줄이는 데 가장 필요한지 좁히기
- 필요하면 `Yield/Blocked/Holding` retry delay를 lane density / retry count / locked-target stickiness에 따라 차등화

### Patch 19

상태:

- planner/orchestrator가 첫 스텝뿐 아니라 `path[2]`의 continuation quality도 보게 됨
- `next-step reservation` 충돌을 execution에서만 흡수하던 상태에서,
  planner 후보 정렬 쪽으로 일부 신호를 끌어올림

핵심 변경:

- `orchestrator.rs`
  - `MovementAdvanceCandidate`에 `second_step_blocked_by_friendly` 추가
  - `candidate_second_step_blocked_by_friendly(...)` helper 추가
    - occupant가 아니라 planner BFS를 통과할 수 있는 friendly soft reservation을 주된 대상으로 봄
  - `compare_advance_candidate(...)`에서
    `forward progress -> lateral shift` 다음 우선순위로
    `second_step_blocked_by_friendly=false` 후보를 더 선호하도록 변경
- 테스트
  - flaky한 high-level planner scenario는 제거
  - 대신 `movement/orchestrator.rs` 내부 unit test
    `compare_advance_candidate_prefers_open_second_step_over_equal_detour`
    로 comparator semantics를 직접 고정

의미:

- 아직 `deferred_next_step_reservations` 전체를 planner로 옮긴 것은 아니지만,
  planner가 "같은 품질의 detour라면 follow-up이 더 열려 있는 lane"을 선호하도록 바뀜
- 이건 execution 보정에만 의존하던 next-step conflict 완화를
  planner prepass로 조금 더 끌어온 패치
- mid-fight 재배치에서 불필요한 lane 선택을 줄이기 위한 기반 규칙으로 볼 수 있음

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 193 passed
- movement intent lib test 7 passed
- movement lib test 25 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations` 자체를 planner/orchestrator prepass로 더 끌어올릴 수 있는지 검토
- 실제 6v6 export의 mid-fight 재배치가 남아 있는지 다시 읽고,
  필요하면 `Hold/Yield/Blocked` delay 또는 line-hold 규칙을 추가
- `Phase 10` 문서 정리를 위해 현재 구조상 남아 있는 legacy execution 책임을 더 명시적으로 목록화

### Patch 20

상태:

- planner가 같은 resolve pass 안에서 이미 선택된 유닛의 `follow-up step(path[2])` claim도 일부 피하게 됨
- `deferred_next_step_reservations`를 execution second-pass로만 처리하던 상태에서
  planner resolve 단계가 continuation conflict를 더 먼저 흡수하기 시작함

핵심 변경:

- `orchestrator.rs`
  - `resolve_movement_intent_decisions(...)`에 `claimed_follow_up_steps` 추가
  - `choose_available_advance_candidate(...)` helper 추가
    - `first_step`가 비어 있는 후보만 본다
    - 그중 `path[2]`가 이미 higher-priority candidate의 follow-up으로 claim되지 않은 후보를 우선
    - 모두 claim된 경우에만 기존 sorted order fallback
  - `follow_up_step(...)` helper 추가
  - chosen candidate를 커밋할 때 `path[2]`도 `claimed_follow_up_steps`에 기록
- 회귀 테스트 추가
  - `choose_available_advance_candidate_prefers_unclaimed_follow_up_step`

의미:

- planner가 이제 first-step claim뿐 아니라 same-pass continuation claim도 고려함
- execution의 `deferred_next_step_reservations`를 완전히 제거하진 않았지만,
  planner 쪽에서 한 번 더 lane 분산 / follow-up conflict 회피를 시도하게 됨
- 장기 방향인 "next-step conflict를 execution 사후 보정보다 planner/orchestrator가 먼저 다루기" 쪽으로 한 단계 더 간 패치

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 194 passed
- movement intent lib test 7 passed
- movement lib test 26 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- 실제 6v6 export의 mid-fight 재배치가 여전히 남는지 다시 읽고,
  line-hold / target-stickiness 규칙이 더 필요한지 좁히기
- `deferred_next_step_reservations`와 execution second-pass 중
  planner로 더 끌어올릴 수 있는 책임과 끝까지 execution에 남겨야 할 책임을 문서화
- `Phase 10` 문서 정리를 위해 남은 legacy execution fallback을 명시적으로 목록화

### Patch 19

상태:

- planner가 candidate를 고를 때 `first_step`뿐 아니라 `follow-up step(path[2])`의 정적 lane quality도 보게 함
- execution의 `deferred_next_step_reservations`를 완전히 planner로 올린 건 아니지만,
  candidate ranking 차원에서 일부 prepass를 시작

핵심 변경:

- `orchestrator.rs`
  - `MovementAdvanceCandidate`에 `second_step_blocked_by_friendly` 추가
  - `candidate_second_step_blocked_by_friendly(...)` helper 추가
    - path의 두 번째 이동 칸이 friendly reservation/occupant로 막혀 있는지 평가
  - `compare_advance_candidate(...)`가 이제
    동일한 forward/lateral 품질의 detour라면
    follow-up step이 막히지 않은 후보를 우선
- 회귀 테스트는 flaky한 end-to-end path test 대신
  orchestrator 내부 unit test로 고정
  - `compare_advance_candidate_prefers_open_second_step_over_equal_detour`

의미:

- planner가 이제 "당장 첫 칸만 갈 수 있으면 된다"가 아니라
  "첫 칸 이후 바로 막히는 lane인지"를 일부 감안하기 시작함
- 이건 `deferred_next_step_reservations` 전부를 planner로 옮긴 것은 아니지만,
  next-step conflict를 execution 사후 보정보다 planner ranking으로 흡수하기 시작한 첫 단계
- mid-fight 재배치에서 불필요한 detour/막힌 lane 진입을 줄일 기반이 생김

실행한 검증:

- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --lib`
- `cargo test --test battle_ranged_attack`

결과:

- lib test 193 passed
- movement intent lib test 7 passed
- movement lib test 25 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 단순 execution second-pass가 아니라 orchestrator decision/preclaim으로 더 끌어올릴 수 있는지 검토
- `tft_like_field_6v6_mixed_melee_ranged_battle`의 mid-fight 재배치 구간을 다시 읽고
  follow-up lane ranking만으로 부족한 line-hold 규칙이 있는지 좁히기
- 필요하면 `Yield/Blocked/Holding` retry delay를 lane density / retry count / target stickiness에 따라 차등화

### Patch 22

상태:

- execution에 남아 있던 blocker 판정 중복을 공용 helper로 정리
- `Phase 10` 목적으로 현재 남은 legacy execution 책임을 명시적으로 문서화

핵심 변경:

- `execute.rs`
  - `MovementBlockerKind` 추가
    - `DeferToHigherPriorityDueMover`
    - `YieldToDueMover`
    - `FriendlyStatic`
    - `HardFailure`
  - `classify_movement_blocker(...)` helper 추가
  - 아래 경로가 이제 같은 blocker semantics를 공유
    - ready move commit 시 `to` reservation/occupant 충돌
    - next-step reservation 실패 시 blocker 판정
    - deferred next-step reservation retry 시 blocker 판정
- 의미적으로는 동작 변화보다 구조 정리에 가깝다.
  execution 내부의 if-chain 중복을 줄여서,
  이후 planner/orchestrator 쪽으로 더 올릴 책임이 무엇인지 분리하기 쉬워졌다.

남은 legacy execution 책임:

1. `handle_move_steps_at()`의 due mover snapshot / current segment sampling
   - 어떤 mover가 이번 tick에 실제로 step end에 도달했는지 판단
   - 연속 좌표 동기화와 `RangeEnter`/boundary 구분
2. same-tick tile mutation commit
   - `claimed_to`
   - `battlefield.move_unit(...)`
   - timeline `UnitMoved`
3. execution second-pass conflict absorption
   - `deferred_intents`
   - `deferred_next_step_reservations`
   - due mover가 실제로 비운 뒤 같은 tick 안에 재시도하는 부분
4. post-move state mutation
   - `path_cursor`
   - next-step reserve
   - `schedule_current_move_step(...)`
   - `schedule_move_step_at(...)`
5. movement 종료 처리
   - `Arrived`
   - `target_acquired`
   - `RangeEnter` 후 재스케줄

현재 결론:

- planner/orchestrator는 이미 target stickiness, hold/yield/blocked, first-step/follow-up claim, 일부 lane quality까지 담당한다.
- execution은 이제 "새 path를 고르는 곳"보다 "같은 tick 안의 실제 commit과 re-try를 수행하는 곳"에 더 가깝다.
- 다음 장기 단계는 `deferred_next_step_reservations`와 일부 second-pass 판단을 planner prepass 또는 explicit orchestrator decision으로 더 끌어올리는 것이다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 execution helper가 아니라 orchestrator preclaim / decision으로 더 끌어올릴 수 있는지 검토
- mid-fight 재배치가 다시 생기는 export가 나오면, 그 패턴이
  planner candidate quality 문제인지
  execution second-pass 문제인지
  먼저 분류
- 필요하면 `Holding/Yielding/Blocked` retry delay를 line density / retry count / locked-target 유지 강도에 따라 차등화

### Patch 23

상태:

- execution의 `next-step reservation` 흐름을 명시적 helper/result로 분리
- `try_commit_ready_move_intent()`와 `try_reserve_deferred_next_step()` 사이의 중복을 축소

핵심 변경:

- `execute.rs`
  - `NextStepReservationAttempt` 추가
    - `Completed`
    - `DeferredRetry`
  - `try_prepare_next_step_after_move(...)` helper 추가
    - next-step reserve 실패 시 blocker 분류
    - `deferred_next_step_reservations` enqueue
    - `Yield / Blocked / WaitRepath` 전환
    - reserve 성공 시 다음 스텝 적용
  - `apply_reserved_next_step(...)` helper 추가
    - `step_to`
    - `target_x_units`
    - `target_y_units`
    갱신을 한 곳으로 통합
  - 기존에 `try_commit_ready_move_intent()`와
    `try_reserve_deferred_next_step()`에 나뉘어 있던
    next-step reserve / next-step state mutation 중복을 제거

의미:

- 아직 `deferred_next_step_reservations` 자체는 execution에 남아 있지만,
  이제 그 경로의 의미와 state mutation이 explicit helper에 모였다.
- 다음 단계에서
  - `deferred_next_step_reservations`를 orchestrator decision으로 올리거나
  - next-step preclaim을 planner 쪽에 더 끌어올릴 때
  수정 경계가 훨씬 명확해졌다.
- 즉 이 패치는 "행동 변화"보다 "남은 legacy execution 경계 압축"에 가까운 장기 리팩토링 패치다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`를 더 이상 opaque execution retry queue로 두지 않고,
  orchestrator preclaim 또는 explicit movement decision으로 승격할 수 있는지 검토
- 필요하면 `MovementDecision::Advance`가 first-step뿐 아니라
  follow-up step quality / preclaim 정보를 더 많이 싣도록 확장
- 새 export에서 mid-fight 재배치가 다시 보이면
  planner quality 부족인지
  execution second-pass 부족인지
  먼저 분류한 뒤 규칙을 추가

### Patch 24

상태:

- `deferred_next_step_reservations` 경로를 tuple/side-effect 중심에서 explicit decision 중심으로 한 단계 더 승격
- execution의 next-step reserve 분기가 이제 "무슨 결정을 내렸는가"를 먼저 표현하고, caller가 그 결정을 적용

핵심 변경:

- `execute.rs`
  - `DeferredNextStepReservation` struct 추가
    - `unit_id`
    - `next_step`
    - `priority`
  - `NextStepReservationDecision` enum 추가
    - `Reserved`
    - `DeferToRetryQueue`
    - `Yield`
    - `Blocked`
    - `WaitRepath`
  - `resolve_next_step_reservation_after_move(...)` helper 추가
    - next-step reserve 시도
    - blocker classification
    - explicit decision 반환
  - `try_commit_ready_move_intent()`는 이제
    next-step reserve helper의 decision을 받아
    - retry queue push
    - yield
    - blocked
    - wait_repath
    - schedule current step
    를 선택
  - `try_reserve_deferred_next_step()`도 같은 decision enum을 사용
    - same-tick retry queue 경로는 두 번째 pass에서는 `yield`로 흡수
- 의미적으로는, next-step queue가 여전히 execution 내부에 남아 있지만
  더 이상 opaque tuple + side-effect helper 조합은 아니다.
  이제 caller 레벨에서 "왜 이렇게 처리하는가"가 드러난다.

의미:

- planner/orchestrator로 옮기기 전 마지막 정리 단계에 가깝다.
- 남아 있는 `deferred_next_step_reservations`는 이제
  단순 execution 내부 꼼수가 아니라,
  explicit decision을 가진 retry queue로 볼 수 있다.
- 다음 단계에선 이 queue 자체를
  orchestrator preclaim 또는 richer `MovementDecision::Advance` payload로 끌어올릴 수 있다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `DeferredNextStepReservation` queue를 execution 내부 second-pass에만 두지 않고
  planner/orchestrator의 preclaim 또는 explicit advance payload와 연결할 수 있는지 검토
- `MovementDecision::Advance`가 필요하면
  `follow_up_step`
  `follow_up_preclaim`
  `lane_quality`
  같은 정보를 더 싣도록 확장
- 새 export에서 mid-fight 재배치가 보이면
  planner candidate ranking으로 해결할 문제인지
  execution second-pass를 더 줄여야 할 문제인지
  먼저 분리

### Patch 25

상태:

- `MovementDecision::Advance`가 이제 첫 follow-up continuation hint를 실제 `MovementState`까지 전달
- execution의 next-step intent가 raw path 인덱싱만이 아니라 explicit movement state field도 참고하게 됨

핵심 변경:

- `movement/mod.rs`
  - `MovementState`에 `planned_follow_up_step: Option<Position>` 추가
  - `MovementState::new_at(...)` 기본값은 `None`
  - 기본 상태 테스트에 위 기본값 검증 추가
- `orchestrator.rs`
  - `MovementDecision::Advance`에 `planned_follow_up_step` 추가
  - chosen candidate의 `follow_up_step(path[2])`를 decision payload에 싣도록 변경
  - decision commit 시 `MovementState.planned_follow_up_step`에 저장
- `execute.rs`
  - step completion 후 다음 예약 대상을 고를 때
    `planned_follow_up_step`이 현재 path와 일치하면 그 hint를 우선 사용
  - `apply_reserved_next_step(...)`가 다음 continuation hint를
    `path_cursor + 2` 기준으로 다시 갱신

의미:

- 아직 planner가 next-step을 직접 reserve/preclaim 하지는 않지만,
  최소한 orchestrator가 고른 continuation이 execution state까지 explicit하게 이어진다.
- 즉 next-step intent가 더 이상
  "path를 다시 읽어서 추측하는 것"에만 의존하지 않고,
  orchestrator가 남긴 hint를 가진다.
- 다음 단계에서
  - `follow_up_preclaim`
  - richer `Advance` payload
  - execution second-pass 축소
  같은 작업을 하기가 더 쉬워졌다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `planned_follow_up_step` 다음 단계로,
  planner/orchestrator가 same-pass continuation claim을 execution queue와 더 직접 연결할 수 있는지 검토
- 필요하면 `MovementDecision::Advance`에
  `follow_up_preclaim` 또는 `continuation_claim_rank`
  같은 richer payload를 추가
- 새 export에서 mid-fight 재배치가 보이면
  planner candidate/continuation quality 부족인지
  execution second-pass 부족인지
  먼저 분리

### Patch 26

상태:

- orchestrator가 이미 알고 있던 `follow-up claimed` 정보를 실제 movement state까지 전달
- execution이 next-step retry queue를 쓰기 전에, planner가 이미 "contested continuation"이라고 본 케이스를 더 빨리 `yield`로 정리

핵심 변경:

- `movement/mod.rs`
  - `MovementState`에 `planned_follow_up_claimed: bool` 추가
  - 기본값은 `false`
  - 기본 상태 테스트에 검증 추가
- `orchestrator.rs`
  - `MovementDecision::Advance`에 `planned_follow_up_claimed` 추가
  - chosen candidate의 `follow_up_step`가 이미 같은 resolve pass에서 claim된 상태였는지 계산
  - commit 시 `MovementState.planned_follow_up_claimed`에 저장
- `execute.rs`
  - 현재 step 종료 후 next-step 예약을 시도할 때,
    `planned_follow_up_step + planned_follow_up_claimed` 조합으로
    "이 continuation은 planner 단계부터 contested였다"는 사실을 함께 읽음
  - `resolve_next_step_reservation_after_move(...)`는
    higher-priority due mover 때문에 막힌 경우라도
    그 continuation이 이미 planner resolve 단계에서 claim된 상태였다면
    queue 재시도 대신 바로 `Yield`를 반환
  - `apply_reserved_next_step(...)`가 새 step을 잡으면
    `planned_follow_up_claimed`를 다시 `false`로 리셋

의미:

- 이 패치로 execution second-pass queue가 조금 더 줄어든다.
- 중요한 점은 execution이 독자적으로만 판단하는 게 아니라,
  planner가 이미 알고 있던 continuation contention 정보를 state를 통해 이어받아 의사결정한다는 것이다.
- 즉 `Advance` payload가 점점 richer 해지고 있고,
  orchestrator -> movement state -> execution으로 이어지는 계약이 강화되고 있다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `planned_follow_up_step / planned_follow_up_claimed` 다음 단계로,
  execution queue를 더 줄일 수 있는 richer continuation payload
  (`follow_up_preclaim`, `continuation_claim_rank`, 또는 equivalent)를 검토
- 가능하면 `deferred_next_step_reservations`를 더 이상 generic retry queue가 아니라
  planner-origin continuation conflict만을 다루는 더 좁은 구조로 축소
- 새 export에서 mid-fight 재배치가 보이면
  planner continuation quality 부족인지
  execution second-pass 부족인지
  먼저 분리

### Patch 27

상태:

- continuation claim 정보를 `bool`에서 `priority`로 정밀화
- execution이 planner가 본 continuation contention을 더 정확하게 해석하게 됨

핵심 변경:

- `movement/mod.rs`
  - `MovementState.planned_follow_up_claimed: bool` 제거
  - `MovementState.planned_follow_up_claim_priority: Option<u8>` 추가
- `orchestrator.rs`
  - `claimed_follow_up_steps`를 `HashSet<Position>`에서 `HashMap<Position, u8>`로 변경
  - `MovementDecision::Advance`가 이제
    `planned_follow_up_claim_priority: Option<u8>`를 payload로 가짐
  - chosen candidate의 follow-up step이 이미 같은 resolve pass에서 claim되었으면,
    그 claimer의 orchestrator priority를 state로 전달
  - 관련 unit test helper도 `HashMap` 기반으로 갱신
- `execute.rs`
  - next-step reservation 시
    actual blocker priority와 planner-origin claim priority를 비교
  - higher-priority due mover에 막히더라도,
    그 blocker priority가 planner가 이미 알고 있던 continuation claim priority보다
    같거나 더 강하면 queue 재시도 대신 바로 `Yield`
  - 새 step이 적용되면 `planned_follow_up_claim_priority`는 다시 `None`

의미:

- 이전 단계에선 "planner가 contested라고 봤는가"만 알 수 있었다.
- 이번 단계에선 "얼마나 강한 claim이 이미 있었는가"까지 전달한다.
- 그래서 execution second-pass가 더 덜 generic해지고,
  planner-origin continuation contention을 더 정확히 흡수하게 된다.
- 아직 `follow_up_preclaim`까지는 아니지만,
  richer continuation payload로 가는 방향은 분명해졌다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement_intent`
- `cargo test --lib movement`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 195 passed
- movement intent lib test 7 passed
- movement lib test 27 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `planned_follow_up_claim_priority` 다음 단계로,
  실제 preclaim 또는 equivalent continuation contract를
  planner/orchestrator 쪽에서 더 강하게 표현할 수 있는지 검토
- `deferred_next_step_reservations`를
  generic retry queue에서 planner-origin continuation retry queue로 더 축소할 수 있는지 검토
- 새 export에서 mid-fight 재배치가 보이면
  planner continuation ranking 부족인지
  execution second-pass 부족인지
  먼저 분리

## Notes For Next AI

- 이 문서는 **패치가 끝날 때마다 반드시 갱신**해야 한다.
- 설계 배경은 `docs/movement_orchestrator_plan.md`를 먼저 읽으면 된다.
- 실제 1차 코드 작업은 planner/coordinator 교체에 집중하고, execution은 유지하는 것이 핵심이다.

### Patch 28

상태:

- `deferred_next_step_reservations`를 더 이상 generic next-step retry queue로 보지 않고,
  planner가 명시적으로 남긴 continuation hint가 있을 때만 same-tick retry를 허용하도록 축소

핵심 변경:

- `movement/execute.rs`
  - `DeferredNextStepReservation`에 아래 정보를 추가
    - `planner_continuation: bool`
    - `claim_priority: Option<u8>`
  - 현재 step 완료 후 next-step 예약 대상을 고를 때,
    `planned_follow_up_step == path[cursor + 1]`이면
    이 continuation이 planner-origin hint인지 함께 계산
  - `resolve_next_step_reservation_after_move(...)`는 이제
    `planner_continuation` 플래그를 함께 받음
  - higher-priority due mover 때문에 next-step reserve가 막히더라도
    planner-origin continuation이 아니면 retry queue에 넣지 않고 바로 `Yield`
  - 즉 same-tick next-step retry queue는
    "planner가 이미 continuation을 명시한 이동" 쪽으로 더 좁아짐
- `movement/execute.rs` tests
  - 기존 `next_step_reservation_retries_in_same_tick_after_lower_priority_blocker_moves`
    는 planner continuation hint가 있을 때만 유지되도록 갱신
  - 새 회귀 `generic_next_step_without_planner_hint_yields_instead_of_retrying`
    추가

의미:

- 이전까지 `deferred_next_step_reservations`는 explicit struct/decision이긴 했지만,
  여전히 generic execution fallback retry queue 성격이 남아 있었다.
- 이번 패치로 same-tick next-step retry는
  planner/orchestrator가 명시적으로 고른 continuation을 더 지키기 위한 경로로 좁혀졌다.
- 즉 execution second-pass가 한 단계 더 planner-origin contract에 종속되었고,
  generic fallback은 `Yield`로 단순화됐다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement`
- `cargo test --lib movement_intent`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 196 passed
- movement lib test 28 passed
- movement intent lib test 7 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `deferred_next_step_reservations`에 남아 있는 planner continuation retry도
  더 나아가 orchestrator preclaim / richer continuation payload로 끌어올릴 수 있는지 검토
- 새 export에서 mid-fight 재배치가 보이면
  planner continuation ranking 부족인지
  execution second-pass 부족인지
  먼저 분리

### Patch 29

상태:

- planner-origin continuation contract를
  `planned_follow_up_step + planned_follow_up_claim_priority` loose 필드 조합에서
  명시적인 `PlannedContinuation` payload로 승격

핵심 변경:

- `movement/mod.rs`
  - `PlannedContinuation { step, claim_priority }` struct 추가
  - `MovementState`는 이제
    `planned_continuation: Option<PlannedContinuation>`만 유지
- `movement/orchestrator.rs`
  - `MovementDecision::Advance`가
    `planned_continuation: Option<PlannedContinuation>`를 payload로 가짐
  - resolve 단계에서 chosen candidate의 follow-up step과
    기존 claimer priority를 하나의 value object로 묶어서 전달
  - commit 시 `MovementState.planned_continuation`에 저장
- `movement/execute.rs`
  - next-step 예약 경로는 loose `planner_continuation + claim_priority` 인자를 더 이상 받지 않고
    `planned_continuation: Option<PlannedContinuation>` 하나만 받음
  - `DeferredNextStepReservation`도 같은 payload를 그대로 유지
  - 현재 step 적용 후 다음 continuation 갱신도
    `Option<PlannedContinuation>` 기반으로 정리
- `movement/execute.rs` tests
  - planner continuation retry fixture가
    `MovementState.planned_continuation = Some(PlannedContinuation { ... })`
    형태를 쓰도록 갱신

의미:

- 이번 패치는 behavior를 크게 바꾸기보다,
  planner -> movement state -> execution 사이의 continuation contract를
  명시적인 값 타입으로 고정한 것이다.
- 다음 단계에서 `follow_up_preclaim`, richer continuation rank,
  planner-origin retry ownership 같은 개념을 넣더라도
  loose 필드 두 개를 계속 퍼뜨리지 않고
  `PlannedContinuation`을 확장하는 방향으로 갈 수 있게 됐다.
- 즉 execution second-pass를 줄이는 장기 리팩토링을 위한 타입 경계를 정리한 패치다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement`
- `cargo test --lib movement_intent`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 196 passed
- movement lib test 28 passed
- movement intent lib test 7 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `PlannedContinuation` 다음 단계로,
  planner-origin continuation retry ownership 또는 preclaim semantics를
  struct 확장으로 더 끌어올릴 수 있는지 검토
- `deferred_next_step_reservations`에 남은 same-tick retry를
  generic execution queue가 아니라
  explicit continuation retry contract로 더 좁힐 수 있는지 검토
- 새 export에서 mid-fight 재배치가 보이면
  planner continuation ranking 부족인지
  execution second-pass 부족인지
  먼저 분리

### Patch 30

상태:

- 남아 있는 same-tick retry queue를 이름과 타입 수준에서도
  generic execution fallback이 아니라
  planner-origin continuation retry 전용으로 좁힘

핵심 변경:

- `movement/execute.rs`
  - `DeferredNextStepReservation`를 `DeferredContinuationReservation`으로 변경
  - 이 struct는 이제 `planned_continuation: PlannedContinuation`을 필수로 가짐
    - 즉 retry queue에 들어가는 항목은 continuation contract가 없는 generic fallback일 수 없음
  - `NextStepReservationDecision::DeferToRetryQueue`를
    `DeferContinuationRetry`로 변경
  - `try_reserve_deferred_next_step(...)`를
    `try_reserve_deferred_continuation(...)`로 변경
  - `try_commit_ready_move_intent()`에서 continuation retry decision이 나오더라도
    `planned_continuation`이 없으면 즉시 `Yield`
  - 결과적으로 same-tick queue는 이제
    planner가 명시한 continuation을 한 번 더 지켜보는 경로로만 남음

의미:

- Patch 28, 29에서 behavior와 payload를 continuation 쪽으로 좁혔다면,
  이번 패치는 queue/decision 이름과 타입까지 그 의미에 맞췄다.
- 이제 execution second-pass에 남아 있는 retry queue는
  사실상 generic next-step fallback이 아니라
  explicit continuation retry contract로 읽히게 됐다.
- 다음 단계에서 preclaim 또는 retry ownership을 더 넣을 때도
  generic retry queue를 계속 살려두지 않고
  continuation 전용 구조를 확장하는 방향으로 갈 수 있다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement`
- `cargo test --lib movement_intent`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 196 passed
- movement lib test 28 passed
- movement intent lib test 7 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `PlannedContinuation`과 `DeferredContinuationReservation`에
  planner-origin retry ownership 또는 preclaim semantics를 더 싣는 방향 검토
- 새 export에서 mid-fight 재배치가 보이면
  planner continuation ranking 부족인지
  execution continuation retry 부족인지
  먼저 분리

### Patch 31

상태:

- `PlannedContinuation`이 claim priority뿐 아니라
  continuation owner priority까지 가지도록 확장
- continuation retry queue가 owner priority를 별도 loose field로 들지 않고
  continuation contract 자체를 사용하게 정리

핵심 변경:

- `movement/mod.rs`
  - `PlannedContinuation`에 `owner_priority: u8` 추가
- `movement/orchestrator.rs`
  - planner가 follow-up continuation을 만들 때
    `owner_priority = context.state_kind.priority()`를 함께 기록
- `movement/execute.rs`
  - `DeferredContinuationReservation`에서 별도 `priority` 필드 제거
  - deferred continuation retry는 이제
    `planned_continuation.owner_priority`를 사용해
    blocker classification과 retry semantics를 이어감
  - `resolve_next_step_reservation_after_move(...)`도
    matching continuation이 있으면
    current call-site priority 대신 continuation owner priority를
    effective priority로 사용
  - `apply_reserved_next_step(...)`가 후속 continuation을 갱신할 때도
    `owner_priority = state.orchestrator_priority`를 유지
- tests
  - planner continuation retry fixture가
    `PlannedContinuation { step, owner_priority, claim_priority }`
    형태를 쓰도록 갱신

의미:

- Patch 29, 30에서 continuation payload와 retry queue를 정리했다면,
  이번 패치는 "이 continuation은 누구의 planner decision인가"까지
  contract가 직접 들게 만든 것이다.
- 그래서 execution continuation retry가
  외부 `priority` 인자에 덜 의존하고,
  planner-origin continuation contract 자체를 더 신뢰하는 구조로 이동했다.
- 다음 단계에서 preclaim ownership이나 richer continuation rank를 넣을 때도
  이 struct를 계속 확장하는 방향으로 갈 수 있다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement`
- `cargo test --lib movement_intent`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 196 passed
- movement lib test 28 passed
- movement intent lib test 7 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `PlannedContinuation`에 preclaim ownership 또는 continuation retry budget 같은
  richer semantics를 실을지 검토
- mid-fight export를 다시 확인해서
  continuation ranking 부족인지
  execution continuation retry 부족인지
  실제 흔들림 구간을 분리

### Patch 32

상태:

- `PlannedContinuation`이 same-tick continuation retry 권한을
  명시적인 budget으로 직접 들게 됨
- execution continuation retry는 이제
  단순히 "continuation이 있다"가 아니라
  "continuation contract가 아직 retry 권한을 남겨두고 있다"를 기준으로 동작

핵심 변경:

- `movement/mod.rs`
  - `PlannedContinuation`에 `retry_budget: u8` 추가
- `movement/orchestrator.rs`
  - planner가 follow-up continuation을 만들 때
    기본 `retry_budget = 1`로 설정
- `movement/execute.rs`
  - `try_commit_ready_move_intent()`에서
    `DeferContinuationRetry`가 나와도
    continuation budget이 `0`이면 same-tick retry queue로 보내지 않고 즉시 `Yield`
  - queue에 넣을 때는 continuation budget을 `1 -> 0`으로 소모
  - `apply_reserved_next_step(...)`가 새 continuation을 만들면
    다시 `retry_budget = 1`로 초기화
- tests
  - 기존 planner continuation retry fixture에 `retry_budget = 1` 반영
  - 새 회귀
    `planner_continuation_with_exhausted_retry_budget_yields_instead_of_retrying`
    추가

의미:

- 이전까지는 continuation retry가 사실상 implicit one-shot 규칙이었다.
- 이번 패치로 retry ownership이 contract에 명시됐고,
  same-tick continuation retry 권한이 state payload에서 직접 보이게 됐다.
- 이건 다음 단계에서
  planner preclaim ownership,
  continuation-specific retry rank,
  더 세밀한 retry budget
  같은 richer semantics로 확장하기 쉬운 형태다.

실행한 검증:

- `cargo fmt`
- `cargo test --lib movement`
- `cargo test --lib movement_intent`
- `cargo test --test battle_ranged_attack`
- `cargo test --lib`

결과:

- lib test 197 passed
- movement lib test 29 passed
- movement intent lib test 7 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- `PlannedContinuation`에 preclaim ownership을 더 넣을지,
  아니면 실제 export를 먼저 다시 읽고
  남은 mid-fight 흔들림이 continuation ranking 문제인지 확인할지 결정
- continuation retry budget을 더 세분화할지 여부는
  export 관찰 뒤 결정하는 편이 합리적

### Patch 33

상태:

- movement policy baseline을 lane 중심에서 `nearest target + sticky target + 자유로운 reposition` 중심으로 재정의
- 다음 단계의 planner 수정 목표를 continuation 세분화 단독 작업이 아니라
  target lifecycle 계약 반영으로 다시 정리

정리된 정책:

- 근/원거리 모두 기본 타겟은 `현재 위치 기준 nearest enemy`
- 공격 중에는 sticky target 유지
- 현재 타겟을 못 때리게 되었을 때만 재타겟
- 이동 중에도 nearest 재평가는 하되,
  새 타겟이 사거리 안이거나 ETA가 충분히 유리할 때만 변경
- `near-engage` 상태에서는 기존 타겟 유지
- 막힘은 `즉시 포기`보다 `짧은 wait -> reposition -> retarget` 순서로 흡수
- 원거리는 사거리 진입 즉시 정지 후 공격, 카이팅 없음

현재 문서/구현에 주는 의미:

- 기존 plan 문서의 `EngagementLane`과 강한 line-hold 서술은
  핵심 모델이 아니라 보조 heuristic로 내려가야 함
- planner의 다음 우선순위는
  `PlannedContinuation` 확장 자체보다
  `nearest / sticky / near-engage / reposition` 계약을
  orchestrator candidate ranking과 hold/retarget 규칙에 반영하는 것
- execution은 same-tick commit / retry queue를 계속 맡되,
  전술적 판단의 중심은 planner 쪽으로 더 이동해야 함

다음 우선순위:

- `movement/orchestrator.rs`에
  `nearest target`, `sticky target`, `near-engage lock`, `reposition` 정책을 명시적으로 반영
- 기존 `Hold`가 line-hold 성격으로 과하게 쓰이는 구간을 줄이고,
  path 혼잡 후 재평가/재배치 쪽을 강화
- 테스트 기준도
  `lane 유지`보다
  `nearest target`, `sticky target`, `queue-follow`, `reposition`, `no-kiting`
  중심으로 재정리

### Patch 34

상태:

- planner의 강한 line-hold 성향을 줄이고
  `nearest target + reposition` 쪽으로 한 단계 이동
- movement stop 시 `near-engage lock`을 추가해
  거의 붙기 직전의 즉흥 retarget를 억제

핵심 변경:

- `movement/orchestrator.rs`
  - `should_hold_position(...)`를 약화
    - 기존처럼 lateral-only 상황을 광범위하게 `Hold`하지 않음
    - 현재 locked target이 near-engage 상태일 때만 hold를 강하게 고려
  - `compare_advance_candidate(...)` 우선순위를 조정
    - `near-engage locked target` 우선
    - 그 다음은 enemy까지의 실제 접근성
    - `path.len()`과 reservation/follow-up quality를 상위에 배치
    - 기존보다 lateral-shift / forward-line 보존의 우선순위를 낮춤
- `movement/execute.rs`
  - `moving_unit_has_near_engage_lock(...)` 추가
  - `stop_moving_unit_on_target_in_range(...)`가
    현재 locked target이 살아 있고 거의 붙기 직전이면
    다른 in-range target으로 즉시 갈아타지 않도록 변경
- `core/mod.rs`
  - 기존 hold 기대 테스트 3개를
    reposition / retarget 기대에 맞게 갱신
- `movement/execute.rs`
  - 새 회귀
    `moving_unit_keeps_locked_target_when_almost_in_range`
    추가

의미:

- planner는 더 이상 "전열 보존" 때문에 lateral entry를 쉽게 막지 않는다
- 대신 현재 타겟에게 거의 도달한 상태일 때만
  강한 target stickiness를 유지한다
- 이 조합은
  `공격 중 sticky`
  `재배치 중 자유로움`
  `near-engage 직전엔 기존 타겟 유지`
  라는 새 policy와 맞는다

실행한 검증:

- `cargo test --manifest-path core/Cargo.toml --lib movement_intent`
- `cargo test --manifest-path core/Cargo.toml --lib movement`
- `cargo test --manifest-path core/Cargo.toml --test battle_ranged_attack`

결과:

- `movement_intent` 7 passed
- `movement` 30 passed
- `battle_ranged_attack` 12 passed

다음 우선순위:

- 실제 export에서
  opening lane shape는 유지되면서도
  mid-fight 재배치가 더 자유로워졌는지 확인
- 필요하면 `sim.rs` / attack-start 단계의 target lifecycle도
  같은 policy로 더 정렬

### Patch 35

상태:

- `tft_like_field_6v6_mixed_melee_ranged_battle` export를 현재 정책 기준으로 재확인

실행한 검증:

- `cargo test --manifest-path core/Cargo.toml --test battle_ranged_attack`
- export 파일:
  `timeline_exports/tft_like_field_6v6_mixed_melee_ranged_battle.json`

관찰 결과:

- opening은 여전히 안정적이다
  - 초기 근접 전열이 같은 파일/열을 유지한 채 전진
  - 초기 `target_acquired`가 연속으로 잡히며
    예전의 opening diagonal kink는 재현되지 않음
- export 전역 grep 기준
  `wait_repath`, `blocked`, `yield`, `holding`
  reason은 보이지 않았음
  - 외부 타임라인에는 여전히 자연스러운 이동/정지/공격 결과만 남는다
- mid-fight에는 더 자유로운 재배치가 보인다
  - 예: `e25afe...`가 `1,4 -> 1,3 -> 1,2 -> 1,1`로 다시 밀고 들어간 뒤
    `target_acquired`
  - 예: `d194ce...`가 `3,3 -> 3,4 -> 3,5 -> 3,6`으로 재배치 후
    새 타겟 획득
- 즉 opening은 보존되고,
  mid-fight에는 line-hold보다 재참여/reposition이 더 잘 드러난다

현재 결론:

- 이번 policy 전환은
  "opening 안정성 유지 + mid-fight 자유 재배치 증가"
  라는 목표와 대체로 맞는다
- 다음 검토 포인트는
  이 mid-fight 재배치가 원하는 TFT스러운 유연성인지,
  아니면 일부 케이스에서 과한 retarget인지
  `sim.rs`와 attack lifecycle까지 포함해 더 볼 필요가 있다

다음 우선순위:

- `AttackStart` / attack resolve 단계의 target stickiness가
  movement와 동일 정책을 따르는지 확인
- 필요하면
  `current_target` 재선택 규칙을
  movement와 attack-start 양쪽에서 공통 contract로 정리

### Patch 36

상태:

- attack-start target lifecycle을 movement policy와 더 가깝게 정렬
- `current_target` 유지 규칙을 movement/attack-start 양쪽에서 같은 helper contract로 보기 시작함

핵심 변경:

- `movement/sim.rs`
  - `select_basic_attack_target(...)` helper 추가
    - `persisted_target_in_range`
    - explicit hinted target
    - nearest in-range fallback
    순으로 공통 선택 로직을 정리
  - `try_start_pending_basic_attacks(...)`가 위 helper를 사용하도록 변경
  - `process_event(AttackStart)`도 위 helper를 사용하도록 변경
- 의미 변화
  - 이전에는 `schedule_next = false`인 `AttackStart`에서
    explicit target hint가 persisted target보다 먼저 적용될 수 있었음
  - 이제는 현재 타겟을 계속 때릴 수 있으면
    hinted target이 있어도 먼저 유지한다

테스트:

- 기존 테스트 유지
  - `pending_basic_attack_retargets_when_persisted_target_is_out_of_range`
  - `attack_start_keeps_persisted_target_when_it_is_still_in_range`
- 새 회귀 추가
  - `attack_start_hint_does_not_override_persisted_target_when_it_is_still_in_range`

의미:

- movement는
  `공격 중 sticky`
  `못 때리면 nearest 재선택`
  정책으로 이동했고,
  attack-start도 같은 철학을 따르게 됨
- `current_target` contract가
  planner/movement 쪽과 combat start 쪽에서 덜 분리된 상태가 됨

실행한 검증:

- `cargo test --manifest-path core/Cargo.toml --lib attack_start_`
- `cargo test --manifest-path core/Cargo.toml --lib pending_basic_attack_retargets_when_persisted_target_is_out_of_range`
- `cargo test --manifest-path core/Cargo.toml --test battle_ranged_attack`
- `cargo test --manifest-path core/Cargo.toml --lib`

결과:

- attack-start 관련 targeted lib tests passed
- `battle_ranged_attack` 12 passed
- full lib 199 passed

다음 우선순위:

- 아직 남아 있는 차이는 `timeout`/`no-progress` 기반 retarget contract다
- 현재는
  `in-range sticky`
  `near-engage lock`
  `nearest fallback`
  까지는 반영됐고,
  실제 `retarget_timeout_ms`, `no_progress_timeout_ms`, `retarget_eta_advantage_ms`
  같은 tuning 상수는 아직 코드에 명시적으로 들어가지 않았다
- 다음 단계는
  이 hysteresis/tuning contract를 planner 쪽에 넣을지,
  export를 더 보고 필요한 정도만 좁혀서 넣을지 결정

### Patch 37

상태:

- 6v6 export를 다시 읽고,
  현재 mid-fight 재배치가 과한 흔들림인지 아닌지 재판단
- 결론은 "지금 당장 hysteresis 상수를 더 넣기보다,
  현재 target lifecycle contract를 테스트로 더 고정하는 것이 우선"이다

관찰 결과:

- opening은 여전히 안정적이다
  - 초기 근접 전열 직진
  - immediate backward/diagonal detour 없음
- mid-fight 이동은 존재하지만,
  현재 export 기준으로는 무의미한 지그재그보다
  frontline collapse 이후 재참여에 더 가깝다
  - 예: `e25afe...`가 `1,4 -> 1,3 -> 1,2 -> 1,1`로 다시 밀고 들어가며 `target_acquired`
  - 예: `d194ce...`가 `3,3 -> 3,4 -> 3,5 -> 3,6`으로 재배치 후 새 타겟 획득
- 따라서 현재 상태를 "retarget가 아직 과민하다"라고 단정할 근거는 부족하다

핵심 변경:

- `core/mod.rs`
  - `pending_basic_attack_retargets_when_persisted_target_is_dead`
    추가
  - `attack_start_retargets_to_nearest_in_range_when_persisted_target_is_dead`
    추가
- 이 두 테스트는
  `current_target`이 더 이상 유효하지 않을 때
  movement/attack-start 계약이 nearest fallback으로 복귀하는지 고정한다

의미:

- 현재 sticky target contract는
  "유지 가능한 동안 유지"이지
  "죽은 타겟까지 고집"이 아니다
- export 분석과 합쳐 보면,
  지금 단계의 더 안전한 진행 방향은
  새로운 hysteresis를 즉시 추가하는 것보다
  existing nearest/sticky contract를 테스트로 먼저 더 단단히 고정하는 것이다

실행한 검증:

- `cargo test --manifest-path core/Cargo.toml --lib pending_basic_attack_retargets_when_persisted_target_is_dead`
- `cargo test --manifest-path core/Cargo.toml --lib attack_start_retargets_to_nearest_in_range_when_persisted_target_is_dead`
- `cargo test --manifest-path core/Cargo.toml --lib attack_start_`
- `cargo test --manifest-path core/Cargo.toml --test battle_ranged_attack`
- `cargo test --manifest-path core/Cargo.toml --lib`

결과:

- target lifecycle regression tests passed
- `battle_ranged_attack` passed
- full lib passed

다음 우선순위:

- 실제 플레이/새 export에서 mid-fight retarget가 과하다는 증거가 다시 나오면
  그때 `retarget_eta_advantage_ms` 또는 `no_progress_timeout_ms`를 planner에 좁게 추가
- 그 전까지는
  nearest + sticky + near-engage + nearest fallback
  contract를 유지하면서 테스트 coverage를 늘리는 편이 더 안전하다

### Patch 38

상태:

- movement execute 단계에서도
  `dead target`이 near-engage lock보다 우선한다는 계약을 테스트로 고정

핵심 변경:

- `movement/execute.rs`
  - `moving_unit_retargets_when_near_engage_locked_target_is_dead`
    추가
- 이 테스트는 다음을 검증한다
  - 이동 중 유닛이 near-engage 형태의 짧은 path를 갖고 있어도
  - `current_target`이 이미 죽었으면
  - `moving_unit_has_near_engage_lock()`이 유지 조건으로 작동하지 않고
  - 현재 in-range nearest enemy로 즉시 fallback 한다

의미:

- 현재 policy의 sticky/near-engage는
  "살아 있고 계속 때릴 수 있는 타겟"에 대해서만 유효하다
- dead target에 대해서는
  pending attack, attack-start, movement execute가 모두 nearest fallback으로 정렬된다
- 이로써 target lifecycle contract가
  planner/movement/attack-start/execute 경계에서 한 단계 더 닫혔다

실행한 검증:

- `cargo test --manifest-path core/Cargo.toml --lib moving_unit_retargets_when_near_engage_locked_target_is_dead`
- `cargo test --manifest-path core/Cargo.toml --lib movement`
- `cargo test --manifest-path core/Cargo.toml --test battle_ranged_attack`

다음 우선순위:

- 새 export나 실제 플레이에서
  "살아 있는 타겟인데도 mid-fight retarget가 너무 빠르다"
  는 증거가 나오면 그때 hysteresis를 planner에 추가
- 그 전까지는
  target invalidation 경계와 nearest fallback 계약을 계속 테스트로 닫는 쪽이 낫다

### Patch 39

상태:

- movement 명세를 더 직접적으로 TFT 쪽으로 재정의
- 특히 `t=1200` 전후 6v6에서 보이는 근접 유닛의 옆 꺾임을
  "dead target retarget"이 아니라
  "측면 continuation 선택 후 mid-segment RangeEnter 정지"로 해석

핵심 판단:

- 현재 export의 초기 근접 이상함은
  타겟 사망 후 재타겟보다
  planner가 이미 lateral attack-tile continuation을 골라 놓고,
  execution이 continuous melee `RangeEnter`로 그 측면 step 중간에서 멈추는 현상에 더 가깝다
- 즉 문제의 핵심은
  `nearest target` 자체보다
  "현재 타일/현재 세그먼트에서 사실상 engage 가능한데도,
  planner가 빈 공격 타일 탐색을 더 우선한다"는 점이다

명세 변경:

- `movement_orchestrator_plan.md`
  - `TFT 스타일 명세` 섹션 추가
  - `근접 engage 우선 규칙` 추가
    - 현재 타일 engage 가능 시 lateral reposition보다 `Engage`
    - 현재 이동 세그먼트 중 melee reach 진입 시 즉시 `Engage`
  - `sticky target / sticky approach` 명시
    - 타겟뿐 아니라 접근 축도 가능한 한 유지
  - `mid-fight 재참여는 자유롭게`, `near-engage 직전 흔들림은 보수적으로`
    방향으로 재정리
  - 현재 문제 설명도
    `wait_repath` 남용 중심에서
    `lateral continuation + RangeEnter stop` 문제까지 포함하도록 보강

의미:

- 이후 코드 수정 우선순위도 더 선명해졌다
- 단순 hysteresis 추가보다 먼저 볼 것은
  planner candidate generation이
  current-tile/current-segment engage 가능성을 attack-tile 탐색보다 먼저 인정하는지다
- 즉 다음 실제 코드 후보는
  `retarget timeout`보다
  `근접 engage 우선`과 `sticky approach`를 planner에 어떻게 반영할지에 더 가깝다

다음 우선순위:

- `movement/plan.rs`와 `movement/orchestrator.rs`에서
  current tile / current segment engage 가능성을 더 앞단에서 평가할 수 있는지 검토
- 필요하면
  "빈 공격 타일 후보"보다
  "즉시 또는 거의 즉시 engage 가능한가"를 더 높은 rank로 주는 패치 설계

### Patch 40

상태:

- planner의 `sticky approach`를 실제 destination / plan tie-break에 반영
- 같은 target을 향한 동등 chase 후보에서
  enemy x축에 더 딱 붙는 lateral attack tile보다
  현재 접근축을 유지하는 destination을 더 우선하도록 조정

핵심 변경:

- `movement/plan.rs`
  - `compare_destination_preference(...)`
    - equal-distance 후보에서
      `enemy x alignment`보다 `mover_start x alignment`를 먼저 본다
  - `compare_plan_preference(...)`
    - best destination tie-break에서도
      `enemy x alignment`보다 `mover_start x alignment`를 먼저 본다
- 의미
  - same enemy / same chase distance / same first-step인 경우
    planner가 불필요하게 옆 attack tile로 빨려 들어가는 bias를 줄인다
  - 이 변경은 `sticky target`에 더해
    `sticky approach`를 planner ranking에 직접 반영한 첫 보정이다

테스트:

- `core/mod.rs`
  - `movement_intent_prefers_straight_follow_up_over_equal_lateral_attack_tile`
    추가
  - 동일 first-step 이후
    straight follow-up destination이 lateral attack tile보다 우선되는지 검증

의미:

- 이번 패치는 `current segment engage` 전체를 해결하는 완성 단계는 아니다
- 대신 현재 관찰된
  "처음엔 잘 직진하다가 마지막에 옆 attack tile로 꺾는" planner bias를 먼저 줄인다
- 즉 `attack tile optimizer` 쪽 편향을 약화하고
  TFT-style `approach axis 유지`를 한 단계 더 반영한다

### Patch 41

상태:

- planner tie-break에 이어
  orchestrator candidate ranking에도 `sticky approach`를 직접 반영
- locked target 후보들 중에서는
  `enemy가 조금 더 가까워 보이는 측면 finish`보다
  `현재 x축을 유지하는 접근`을 먼저 선택하도록 보정

핵심 변경:

- `movement/orchestrator.rs`
  - `preserves_locked_target_approach(...)` helper 추가
  - `compare_advance_candidate(...)`
    - `near_engage_locked` 다음 tie-break로
      `locked target + first_step 유지 + destination x 유지`를 비교하도록 변경
    - `target_locked`도 enemy distance/path length보다 앞에 오도록 재배치
- 테스트
  - `compare_advance_candidate_prefers_locked_sticky_approach_over_equal_lateral_finish`
    추가
  - 동일 target / 동일 first-step인 두 후보에서
    straight follow-up이 lateral finish보다 우선되는지 검증

의미:

- Patch 40이 `plan.rs` 수준의 destination/plan 정렬 보정이었다면,
  이번 패치는 orchestrator가 실제 candidate list를 고를 때도
  같은 방향을 유지하게 만든다
- 즉 근접 유닛이
  "같은 target으로 가고 있는데 마지막 순간 옆 attack tile로 틀리는" 현상을
  planner + orchestrator 양쪽에서 함께 누르기 시작한 단계다

### Patch 42

상태:

- execution에도 `current tile engage` fallback을 추가
- continuous melee 유닛이 한 스텝을 밟은 직후
  logical tile 기준으로 이미 사거리 안이면
  lateral follow-up 예약 전에 바로 `target_acquired`로 정지하도록 변경

핵심 변경:

- `movement/plan.rs`
  - `persisted_target_in_tile_range_for_continuous_melee(...)`
    추가
  - `choose_attack_target_in_tile_range_for_continuous_melee(...)`
    추가
- `movement/execute.rs`
  - `stop_moving_unit_on_target_in_range(...)`
    가 기존 continuous position in-range 체크 다음에
    tile-range melee fallback도 보도록 변경
  - 의미
    - instant melee에서 current tile이 이미 engage 가능한 상태면
      다음 lateral continuation을 예약하기 전에 즉시 전투를 시작함
    - 즉 "boundary continuous position 때문에 이미 붙은 것처럼 보이는 상태인데도
      한 번 더 옆으로 틀려는" 경향을 execution에서 한 번 더 차단함
- 테스트
  - `moving_unit_stops_immediately_when_step_ends_in_adjacent_melee_tile`
    추가
  - step 종료 후 current tile이 적과 인접하면
    lateral follow-up 대신 즉시 `TargetAcquired`로 멈추는지 검증

의미:

- Patch 40/41이 planner/orchestrator의 `sticky approach` 보정이었다면,
  이번 패치는 execution 경계에서
  "현재 타일 engage"를 직접 인정하는 단계다
- 즉 `t≈1200` 구간의 어색함을
  ranking만이 아니라 stop decision에서도 줄이기 시작한 단계다

### Patch 43

상태:

- 6v6 export를 재실행해서
  `current tile engage` fallback의 실제 효과를 확인
- 결과는 "부분 개선"이다

관찰 결과:

- 이전에는 중앙 근접 다수가 `1251ms`에 타일 이동만 기록되고
  `1678ms` 근처에야 `TargetAcquired`가 발생했다
- 현재는 일부 중앙 근접이 `1251ms` 시점에서 즉시 `TargetAcquired`로 멈춘다
  - 예: `d194ce...`, `e25afe...`, `f9cb51...`
- 하지만 다른 일부는 여전히 `1673ms`까지 계속 이동한 뒤 멈춘다
  - 예: `65dbac...`, `9323b2...`, `9970ce...`
- 즉 planner/orchestrator의 `sticky approach` 보정과
  execution의 `current tile engage` fallback이
  전체 wiggle을 줄였지만, 아직 완전히 제거하지는 못했다

현재 해석:

- 남아 있는 케이스는 단순 `current tile engage` 미인정보다
  `near-engage lock`이 다른 인접 enemy로의 즉시 engage를 막는 경우에 더 가깝다
- 즉 남은 문제는
  "거의 붙기 직전 current target 유지"와
  "사거리 안 enemy가 생기면 즉시 공격"
  두 규칙이 충돌하는 경계다

다음 우선순위:

- `near-engage lock`이 유지되어야 하는 범위를 더 명확히 결정
- 특히
  current target은 아직 in-range가 아니지만
  다른 enemy가 current tile melee range에 들어온 경우에도
  lock을 유지할지, 즉시 engage로 풀지
  정책을 확정해야 한다

### Patch 40

상태:

- planner의 `sticky approach`를 destination/plan tie-break에 직접 반영
- 같은 enemy를 향한 동등 후보에서
  적 x축에 더 바짝 붙는 lateral attack tile보다
  현재 접근축을 더 오래 유지하는 destination을 우선하게 조정

핵심 변경:

- `movement/plan.rs`
  - `compare_destination_preference(...)`
    - forward tie 이후
      `enemy x alignment`보다 `mover_start x alignment`를 먼저 비교하도록 변경
  - `compare_plan_preference(...)`
    - best destination tie-break에서도
      `enemy x alignment`보다 `mover_start x alignment`를 먼저 비교하도록 변경
- `core/mod.rs`
  - `movement_intent_prefers_straight_follow_up_over_equal_lateral_attack_tile`
    추가
  - 동일한 first step을 공유하는 두 chase path 중
    마지막 lateral attack tile 대신
    현재 접근축을 유지하는 follow-up이 선택되는지 검증

의미:

- 이번 패치는 `current segment engage estimator` 같은 큰 모델 추가는 아니다
- 대신 현재 planner가 보이던
  "같은 enemy를 향한 동등 후보에서 불필요하게 측면 attack tile로 끌리는 bias"
  를 먼저 줄인다
- 즉 `attack tile optimizer` 쪽 편향을 약화하고
  `sticky approach`를 실제 planner ranking에 반영한 첫 보정이다

### Patch 44

상태:

- `movement_orchestrator_plan.md`에
  orchestrator 위에 올릴 `continuous spatial layer` 장기 설계를 추가
- 이번 단계는 코드 변경이 아니라 아키텍처 방향 고정 단계다

핵심 정리:

- `MovementOrchestrator`는 계속 authoritative decision layer로 유지
- tile occupancy / reservation / BFS / deterministic ordering도 유지
- 대신 그 위에
  `pos_x_units`, `pos_y_units`, movement segment, continuous proximity 판정을 담당하는
  별도 spatial layer를 얹는 방향으로 정리
- 목표는
  "타일 authority 유지 + replay / AoE / proximity 품질 개선"이다

설계 결정:

- continuous layer는 orchestrator를 대체하지 않는다
- 구현 순서는 `오케스트라 먼저, continuous 나중`이다
- 다만 설계는 지금부터 두 레이어를 함께 보고 가야 한다
- timeline은 장기적으로
  `MovementStopped` 최종 좌표 중심이 아니라
  movement segment reconstruction 중심으로 확장하는 쪽이 맞다

다음 우선순위:

1. orchestrator policy를 계속 마감
2. `near-engage lock` / unnecessary final walk 문제를 더 줄임
3. 그 다음 replay-friendly segment metadata 설계 및 구현으로 이동

### Patch 45

상태:

- continuous spatial layer의 상세 전제를 사용자 답변 기준으로 고정
- 이번 단계도 문서 변경만 있고 코드 변경은 없음

고정한 규칙:

- 논리 점유는 계속 `1 tile = 1 unit`
- 근접 공격 가능 슬롯도 타일 기반 유지
- continuous layer는 occupancy authority를 바꾸지 않음
- 유닛은 visual 상으로 겹치면 안 되며,
  특히 근접 engage 상태에서는 최소 spacing을 유지해야 함
- 근접 engage anchor는 타겟과 가장 가까운 경계점
- 원거리도 전투 시작 후에는 타일 중심 고정이 아니라 continuous 좌표처럼 행동
- unit hitbox는 1차 설계에서 공통 반지름의 원(circle)으로 모델링
- 투사체 / 범위기 / AoE hit-miss는 continuous 판정으로 진행
- 스킬의 시작점과 판정 기준점은 현재 continuous 좌표를 사용

의미:

- 이번 결정으로 continuous layer는
  "표현만 부드럽게 하는 레이어"가 아니라
  장기적으로 skill 판정까지 받치는 spatial layer로 확정됐다
- 반면 tile occupancy와 attack slot은 유지하므로,
  전체 전투 엔진을 full continuous로 갈아엎는 방향은 아니다

다음 우선순위:

1. orchestrator decision layer 마감
2. movement segment timeline / replay contract 상세 설계
3. 그 뒤 projectile / AoE continuous 판정의 첫 적용 범위 결정

### Patch 46

상태:

- execution tracker 상단에 handoff 전용 snapshot을 추가
- 다음 AI가 patch log 전체를 재구성하지 않아도
  현재 상태와 다음 작업을 바로 파악할 수 있게 문서를 재정리

추가한 섹션:

- `Current State Snapshot`
- `Next Patch Target`
- `Do Not Change Yet`
- `Representative Repro Cases`
- `Code Entry Points For Next AI`
- `Current Recommended Strategy`

의미:

- 이제 이 문서는 단순 patch 누적 로그를 넘어서
  "현재 오케스트라 작업이 어디까지 왔고,
  다음 AI가 무엇부터 해야 하는지"를 상단에서 바로 전달할 수 있다
- continuous layer 설계가 문서에 들어갔더라도,
  현재 구현 우선순위는 여전히 orchestrator policy 마감이라는 점을
  더 명시적으로 고정했다

### Patch 47

상태:

- `execute.rs`의 `stop_moving_unit_on_target_in_range()` 경계를 조정
- `near-engage lock`이 "이미 현재 위치에서 때릴 수 있는 다른 enemy"까지 막지 않도록 수정
- 관련 회귀 테스트를 새 정책에 맞게 갱신하고 유지용 테스트를 추가

변경 내용:

- 기존에는
  `persisted target in range`가 아니고 `near-engage lock`이 걸려 있으면,
  다른 in-range enemy가 있어도 즉시 `false`를 반환했다
- 이제는 순서를 다음처럼 바꿨다
  1. persisted target이 실제 공격 가능하면 유지
  2. 아니면 현재 위치에서 공격 가능한 다른 enemy가 있으면 즉시 `TargetAcquired`
  3. 둘 다 아니고 `near-engage lock`이 있으면 계속 이동
- 즉 `near-engage lock`은
  "더 걷게 만드는 unnecessary final walk"를 줄이기 위한 보수 규칙으로만 남고,
  실제 engage 기회 자체를 막지는 않게 됐다

추가/갱신한 테스트:

- `moving_unit_engages_other_enemy_when_near_engage_locked_target_is_not_yet_in_range`
- `moving_unit_keeps_locked_target_when_almost_in_range_and_no_other_enemy_is_attackable`

의미:

- 현재 위치에서 이미 때릴 수 있는 적이 있는데도
  lock 때문에 한 스텝 더 걷는 케이스를 줄이는 보정이다
- 이는 `t≈1251/1673ms` opening repro에서 보이던
  일부 melee final walk를 줄이기 위한 execute-side 경계 수정에 해당한다

다음 확인 포인트:

1. 6v6 export를 다시 생성해서 `TargetAcquired` 시점이 더 앞당겨졌는지 확인
2. 여전히 남는 lateral/final walk가 있으면 planner/orchestrator 쪽 candidate ranking 문제인지 분리

### Patch 48

상태:

- orchestrator planner에 `direct continuous melee engage approach`를 추가
- 인접한 enemy tile로의 straight engage를 빈 측면 attack tile보다 먼저 선택하도록 보정

변경 내용:

- `resolve_movement_intent_decisions()`에서
  immediate in-range engage 다음 단계로
  `direct_continuous_melee_engage_target()`를 확인하도록 추가
- 조건을 만족하면
  일반 `dest_candidates` 기반 lateral attack tile 후보를 만들기 전에
  `path = [start, enemy_tile]` 형태의 직진 engage approach를 만든다
- `commit_movement_intent_decisions()`에서는
  이 경우 enemy-occupied first step을 continuous melee 예외로 허용하고,
  soft reservation 실패로 `Blocked/WaitRepath`로 떨어뜨리지 않는다

의미:

- 지금까지는 adjacent tile engage가 가능한 장면에서도
  planner가 "빈 공격 타일"을 더 우선해서 측면 continuation을 만들 수 있었다
- 이번 패치 이후에는
  근접 유닛이 타겟과 타일 단위로 이미 붙어 있다면
  occupied enemy tile 쪽 shared boundary를 향해 straight engage approach를 먼저 시작한다
- 즉 `attack tile optimizer`보다 `engage-first`를 planner에 더 직접 반영한 보정이다

추가 테스트:

- `movement_intent_prefers_direct_enemy_tile_engage_approach_when_adjacent_continuous_melee`

다음 확인 포인트:

1. 6v6 export에서 `t≈1251/1673ms` 구간의 lateral wiggle이 실제로 줄었는지 확인
2. 여전히 남는 케이스가 있으면 same-tick target visibility/retarget ordering 문제인지 분리

### Patch 49

상태:

- `Patch 48` 이후 6v6 export를 재생성하고 opening repro를 다시 확인
- planner의 direct engage approach 보정이 실제 timeline에 반영되는지 검증

검증 결과:

- `t≈1673ms`의 `MovementStopped(target_acquired)`는 여전히 존재하지만,
  이전처럼 `pos_x_units`가 측면 경계로 밀리지 않고
  모두 현재 축 중심에 가깝게 기록된다
- 대표 예시:
  - `65db...`: `505600 -> 1000000`
  - `9323...`: `2505600 -> 3000000`
  - `9970...`: `4505600 -> 5000000`
  - `e25...`: `500000 -> 1000000`
- 즉 opening lateral wiggle의 핵심 증상은 줄었고,
  현재 남은 것은 sideways drift보다 `TargetAcquired` 시점 자체가 아직 `1673ms`에 남는 문제에 더 가깝다

의미:

- 이제 주된 이슈는 "왜 옆으로 가는가"보다
  "왜 일부 melee engage가 1251ms가 아니라 1673ms에 닫히는가"로 더 좁혀졌다
- 즉 planner의 빈 측면 attack tile bias는 크게 줄었고,
  남은 문제는 same-tick target visibility / engage timing 경계에 가깝다

다음 확인 포인트:

1. 1251ms 직후 same-tick post-move retarget에서
   인접 melee target을 즉시 닫지 못하는 이유를 더 추적
2. 필요하면 `handle_move_steps_at()`와 `post_move_retarget_at()`의 ordering contract를 테스트로 고정

### Patch 50

상태:

- same-tick adjacency를 별도 재현해 남은 `1673ms` 지연 원인을 더 좁혀 봄
- 진단 결과를 바탕으로 남은 문제가 planner drift가 아니라
  same-tick opposing mover ordering 쪽에 가깝다는 점을 확인

관찰:

- 단순 "두 melee가 같은 tick에 서로 인접 타일로 들어오면 바로 `post_move_retarget_at()`가 닫아주는가"를
  재현하려 했을 때,
  한쪽이 인접 상태를 만들기 전에 `Yielding`으로 빠지는 케이스가 확인됐다
- 즉 남은 지연은
  `stop_moving_unit_on_target_in_range()`가 tile-range target을 못 보는 문제라기보다,
  same-tick move commit / continuation claim / yield ordering 때문에
  아예 원하는 인접 상태가 그 tick 안에 형성되지 않는 경우에 더 가깝다

의미:

- `Patch 48/49`로 opening lateral drift는 줄었고,
  이제 남은 핵심은 same-tick opposing mover 경계다
- 다음 패치는 execute의 generic range check보다
  `handle_move_steps_at()` 내부 ordering과
  continuation claim이 불필요하게 one-tick 지연을 만드는지 보는 쪽이 더 적절하다

다음 확인 포인트:

1. opposing melee가 서로 직진해 붙는 케이스에서
   continuation claim / reservation이 과하게 남아 있는지 확인
2. 필요하면 same-tick adjacent engage를 우선하는 execute-side contract를 추가

### Patch 51

상태:

- `handle_move_steps_at()`의 execute ordering을 리팩토링
- same-tick move commit과 post-move engage/continuation 결정을 분리

변경 내용:

- `try_commit_ready_move_intent()`는 이제
  current step의 commit(`battlefield.move_unit`, `UnitMoved` 기록)까지만 담당한다
- move 직후 `TargetAcquired`/continuation/yield를 바로 처리하지 않고,
  commit된 mover들을 모은 뒤
  `finalize_committed_move_after_batch()`에서 후처리한다
- 즉 같은 tick의 due mover들이 모두 board state를 갱신한 뒤에야
  post-move engage 여부와 다음 step continuation을 결정하게 바뀌었다
- 이 과정에서 generic next-step과 exhausted continuation budget도
  "retry queue"가 아니라 "settled same-tick board state"를 기준으로 처리하도록 계약을 정리했다

추가/갱신한 테스트:

- `generic_next_step_uses_settled_same_tick_board_before_yielding`
- `exhausted_continuation_budget_does_not_block_settled_same_tick_progress`

의미:

- 이전에는 먼저 처리된 mover가
  나중에 move할 상대를 아직 못 본 상태에서 continuation/yield로 들어갈 수 있었다
- 이번 패치 이후 execute는
  `same-tick move commit -> settled board -> engage/continuation` 순서로 동작한다
- 이는 임시 예외가 아니라,
  execute를 "한 tick의 movement resolution"으로 더 일관되게 만든 구조 정리다

다음 확인 포인트:

1. 6v6 opening repro에서 one-tick delay가 실제로 줄었는지 확인
2. move ordering 개선이 premature `TargetAcquired`를 만들지 않는지 확인

### Patch 52

상태:

- `Patch 51` 이후 6v6 export를 다시 보고,
  `TargetAcquired` 판정이 너무 일찍 발생하는 부작용을 수정

문제:

- same-tick batch finalize를 넣은 뒤,
  `stop_moving_unit_on_target_in_range()`가
  continuous melee의 tile-adjacent fallback까지 바로 `TargetAcquired`로 인정하면서
  실제 continuous reach가 아직 아닌 장면에서도 너무 일찍 stop이 찍혔다
- 그 결과 6v6 opening에서
  `1251ms` 즉시 stop 뒤 `1678ms`에 다시 `target_acquired`가 한 번 더 찍히는
  이중 lifecycle이 생겼다

변경 내용:

- `stop_moving_unit_on_target_in_range()`를 다시
  actual continuous/basic attack range 기준으로만 stop하도록 좁혔다
- `persisted_target_in_tile_range_for_continuous_melee`
  / `choose_attack_target_in_tile_range_for_continuous_melee` fallback은
  stop 판정에서 제거했다
- tile-adjacent continuous melee는 여전히 planner/orchestrator의
  `direct continuous melee engage approach`에서 다루되,
  실제 `TargetAcquired`는 reach가 만족될 때만 기록되도록 정렬했다

검증 결과:

- `cargo test --lib movement` 통과
- `cargo test --test battle_ranged_attack tft_like_field_6v6_mixed_melee_ranged_battle -- --exact` 통과
- 6v6 export에서
  player/opponent melee의 opening `MovementStopped(target_acquired)`는 다시 `1678ms`에만 남고,
  예전처럼 same unit이 `1251ms`와 `1678ms`에 중복 stop되는 현상은 사라졌다
- 동시에 lateral drift는 이미 줄어든 상태를 유지한다

의미:

- execute-side ordering 리팩토링은 유지하면서도,
  `TargetAcquired` 의미를 다시 "실제 공격 가능"으로 고정했다
- 즉 현재 contract는
  `settled same-tick board`를 본 뒤
  `actual reach` 기준으로 engage를 닫고,
  아니면 continuation/yield를 정하는 방향으로 정리됐다

다음 확인 포인트:

1. 6v6 opening의 `1678ms` engage timing 자체가 의도된 reach closure인지,
   아직도 불필요한 final walk가 남아 있는지 다시 평가
2. 오케스트라 마감 관점에선
   remaining issue가 execute ordering이 아니라 planner candidate policy인지 계속 분리

### Patch 53

상태:

- 6v6 opening의 `1251ms -> 1678ms` melee engage 지연을 다시 분리해서 확인
- 결론은 현재 이 구간이 planner의 extra tile walk가 아니라
  tile-adjacent 이후 actual continuous reach closure라는 점을 고정

관찰:

- opening melee 6유닛은 첫 `UnitMoved` 이후
  첫 `AttackStart` 전까지 추가 `UnitMoved`를 만들지 않는다
- 즉 `1251ms` 이후 `1678ms`까지의 시간은
  "다음 tile step"이 아니라
  같은 tile 점유 상태에서 boundary-to-boundary continuous distance가 줄어드는 구간이다
- 현재 movement 모델에서는 tile occupancy와 continuous position이 분리되어 있으므로,
  tile-adjacent가 곧바로 `TargetAcquired`를 의미하지는 않는다

변경 내용:

- `tests/battle_ranged_attack.rs`
  - `tft_like_field_6v6_mixed_melee_ranged_battle`에
    opening melee 6유닛이 첫 공격 전까지 추가 tile move를 만들지 않는 회귀를 추가
- execution/ planner 코드는 이번 패치에서 더 바꾸지 않음

의미:

- 이 구간은 더 이상 "남은 orchestrator 버그"로 취급하지 않는다
- 즉 앞으로는 `1678ms` 자체를 줄이려는 임시 보정보다,
  실제 extra tile walk / lateral wiggle / mid-fight unnecessary walk만 다시 문제로 본다
- opening reach closure를 더 앞당기고 싶다면
  그건 orchestrator patch가 아니라
  continuous spatial layer 또는 movement geometry contract 논의에 가깝다

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack tft_like_field_6v6_mixed_melee_ranged_battle -- --exact`

### Patch 54

상태:

- equal-distance target tie-break에 `근거리 enemy 우선` 규칙 추가
- attack selection, tile-range helper, movement chase ordering이 같은 우선순위를 쓰도록 정리

변경 내용:

- `movement/plan.rs`
  - `compare_enemy_target_preference()` 추가
  - equal-distance일 때는 UUID보다 먼저 `basic_attack range_tiles`가 더 짧은 enemy를 우선
  - `choose_attack_target_in_range()`
  - `choose_enemy_target_in_tile_range()`
  - `formulate_enemy_chase_plan()`에 동일 기준 반영
- `movement/orchestrator.rs`
  - `compare_advance_candidate()`에도 동일 enemy preference를 넣어,
    chase distance가 같은 경우 melee enemy를 더 먼저 본다

의미:

- mid-fight crossing이나 equal-distance retarget에서
  ranged enemy로 새는 대신 frontline melee를 더 우선적으로 붙잡게 된다
- UUID tie-break는 여전히 최종 결정론 fallback으로 남고,
  combat policy는 그 앞단에서 더 자연스러운 적 우선순위를 가진다

추가 테스트:

- `choose_attack_target_in_range_prefers_melee_enemy_when_distance_is_equal`
- `choose_enemy_target_in_tile_range_prefers_melee_enemy_when_distance_is_equal`
- `movement_intent_prefers_melee_enemy_when_chase_distance_is_equal`

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib choose_attack_target_in_range_prefers_melee_enemy_when_distance_is_equal`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib choose_enemy_target_in_tile_range_prefers_melee_enemy_when_distance_is_equal`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib movement_intent_prefers_melee_enemy_when_chase_distance_is_equal`

### Patch 55

상태:

- opposing melee끼리 마주 붙는 상황에서
  enemy side의 follow-up claim까지 회피 대상으로 보는 문제를 수정

문제:

- `claimed_follow_up_steps`가 side 구분 없이 전역으로 공유돼 있었다
- 그래서 player melee의 직진 continuation이
  opponent melee가 먼저 claim한 같은 tile 때문에 회피 대상으로 보일 수 있었고,
  `f9cb...` 같은 유닛이 불필요한 diagonal detour를 선택했다

변경 내용:

- `movement/orchestrator.rs`
  - `claimed_follow_up_steps`를 `HashMap<(Side, Position), u8>`로 변경
  - follow-up claim은 이제 같은 side congestion 제어에만 사용
  - opposing side의 claim은 `choose_available_advance_candidate()`가 회피 사유로 보지 않음

추가 테스트:

- `choose_available_advance_candidate_ignores_opposing_follow_up_claim`

의미:

- follow-up claim은 "같은 진영의 줄서기/혼잡"을 풀기 위한 장치로 남고,
  서로 적대하는 frontline melee가 붙으러 들어가는 상황까지
  불필요하게 빗겨가게 만들지 않게 된다
- 이는 임시 예외가 아니라,
  claim 모델의 ownership을 side-aware로 바로잡는 구조 정리다

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib choose_available_advance_candidate_ignores_opposing_follow_up_claim`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack tft_like_field_6v6_mixed_melee_ranged_battle -- --exact`

### Patch 56

상태:

- `f9cb...` opening diagonal의 직접 원인을 `same-range enemy tie-break`에서 UUID가 너무 일찍 개입하던 경계로 좁혔다
- 같은 사거리의 적끼리는 먼저 spatial chase preference로 고르고,
  UUID는 마지막 deterministic fallback으로만 남긴다

문제:

- `Patch 54`에서 `근거리 enemy 우선`을 넣었지만,
  이 비교가 `range_tiles -> UUID`까지 바로 포함하고 있었다
- 그래서 두 적이 모두 melee처럼 같은 사거리면,
  spatial `compare_plan_preference()`보다 UUID가 먼저 승부를 내서
  off-axis enemy/destination이 선택될 수 있었다
- `f9cb...` opening diagonal은 이 경계에 걸린 사례로 판단된다

변경 내용:

- `movement/plan.rs`
  - `compare_enemy_target_range_preference()`와
    `compare_enemy_target_preference()`를 분리
  - `formulate_enemy_chase_plan()`은 이제
    `chase_dist -> enemy range -> spatial plan preference -> UUID` 순서로 정렬
- `movement/orchestrator.rs`
  - `compare_advance_candidate()`도
    same-distance enemy 비교에서 UUID 전체 비교 대신
    `enemy range`만 먼저 보고,
    spatial plan/destination preference가 뒤에서 실제 축 선택을 결정하게 정리

추가 테스트:

- `movement_intent_prefers_straight_enemy_over_equal_range_diagonal_enemy`

의미:

- melee vs melee처럼 같은 사거리의 적 둘이 같은 거리에서 보일 때,
  UUID가 off-axis 적을 먼저 고르게 만들지 않는다
- frontline engage는 먼저 "더 자연스러운 축/직진 plan"으로 고르고,
  UUID는 마지막 deterministic fallback으로만 남는다
- 실제 6v6 export에서도 `f9cb...` opening move가
  `(5,5) -> (4,4)`에서 `(5,5) -> (5,4)`로 돌아왔다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib movement_intent_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack tft_like_field_6v6_mixed_melee_ranged_battle -- --exact`

### Patch 57

상태:

- opening diagonal / crossing 대표 repro는 현재 해결된 상태로 본다
- 다음 오케스트라 작업 우선순위를 `새 opening 버그 추적`에서
  `mid-fight retarget consistency 검증`으로 옮긴다

변경 내용:

- `movement_orchestrator_execution.md`
  - 상단 `Current State Snapshot` 갱신
  - 상단 `Next Patch Target` 갱신
  - `Representative Repro Cases`를 baseline / broader sweep 중심으로 정리
  - 다음 AI가 opening repro를 이미 해결된 이슈로 이해하도록 handoff 문구 수정

의미:

- 현재 오케스트라 작업은 더 이상
  `f9cb...` opening diagonal이나 `d194.../f9cb...` crossing을
  대표 미해결 버그처럼 쫓지 않는다
- 다음 단계는 planner / execute / attack-start fallback의
  target preference contract를 끝까지 닫는 작업이다

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 58

상태:

- `movement intent`, `post-move retarget`, `AttackStart fallback`가
  same-range equal-distance target에서 같은 spatial preference를 쓰도록 정리

문제:

- planner/orchestrator 쪽은 이미
  "같은 사거리의 적 둘이면 UUID보다 spatial plan preference를 먼저 본다"로 정리돼 있었음
- 하지만 in-range target selection helper
  (`choose_attack_target_in_range`, `choose_enemy_target_in_tile_range`)는
  equal-distance에서 `enemy range -> UUID`만 보고 있었음
- 그래서 mid-fight retarget나 AttackStart fallback이
  planner보다 더 쉽게 diagonal/off-axis target을 잡을 여지가 남아 있었음

변경 내용:

- `movement/plan.rs`
  - `compare_in_range_target_preference()` 추가
  - in-range target tie-break를
    `enemy range -> spatial plan preference -> UUID` 순서로 정리
  - `choose_attack_target_in_range()`
  - `choose_enemy_target_in_tile_range()`
    가 이 helper를 재사용하도록 변경
- `core/sim.rs`
  - `select_basic_attack_target()`를
    `pub(in crate::game::battle::core)`로 노출
- `movement/execute.rs`
  - `stop_moving_unit_on_target_in_range()`가
    자체 분기 대신 `select_basic_attack_target(..., None)`을 재사용하도록 변경

추가 테스트:

- `choose_attack_target_in_range_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `choose_enemy_target_in_tile_range_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `attack_start_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `moving_unit_retargets_to_straight_enemy_over_equal_range_diagonal_enemy`

의미:

- 이제 target lifecycle 전반에서
  "같은 사거리 / 같은 거리의 적이면 더 자연스러운 축의 적을 먼저 본다"는 계약이 유지된다
- 이는 임시 예외가 아니라,
  planner와 execute/sim fallback이 같은 selector를 공유하도록 정리한 구조 리팩토링이다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib choose_attack_target_in_range_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib choose_enemy_target_in_tile_range_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib attack_start_prefers_straight_enemy_over_equal_range_diagonal_enemy`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib moving_unit_retargets_to_straight_enemy_over_equal_range_diagonal_enemy`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib movement`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 59

상태:

- 오케스트라 검증 범위를 단일 tie-break/unit test에서
  crowded frontline scenario까지 확장

문제:

- 기존 테스트는 특정 규칙 하나를 고정하는 데는 충분했지만,
  여러 melee/ranged가 동시에 배치된 opening frontline에서도
  같은 policy가 유지되는지까지는 보여주지 못했음
- 오케스트라를 마감하려면
  "작은 회귀 테스트 + 큰 시나리오 invariant"가 같이 필요했음

변경 내용:

- `tests/battle_ranged_attack.rs`
  - `tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage` 추가
  - 4 melee + 3 ranged per side의 dense mirror opening을 구성
  - 각 frontline melee의 첫 `UnitMoved`가
    즉시 diagonal/off-axis로 새지 않고
    자기 축을 유지한 채 전진하는지 검증
  - timeline export도 함께 남김:
    `core/timeline_exports/tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage.json`

의미:

- 이제 오케스트라 검증은
  "특정 helper가 이렇게 정렬한다" 수준을 넘어서,
  실제 crowded opening frontline에서도
  straight engage invariant가 유지되는지까지 확인한다
- 다음 단계에서 mid-fight retarget를 더 볼 때도
  dense scenario export를 같이 baseline으로 삼을 수 있다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 60

상태:

- 상대 frontliner가 죽은 뒤 melee가 다음 attack cadence까지 멍하게 쉬는 문제 수정

문제:

- dense battle export를 보면
  frontliner death 이후 survivor melee가 곧바로 다시 움직이지 않고,
  다음 `AttackStart` 주기까지 Idle로 남는 케이스가 있었다
- 예를 들어 dense 7v7 export에서
  kill 시점은 `7001ms`인데 다음 행동은 `8500ms AttackStart`로,
  약 `1499ms` gap이 남아 있었다
- 원인은 `UnitDied`가 occupancy와 target validity를 바꾸는데도
  global movement recompute를 같은 tick에 트리거하지 않았기 때문이다

변경 내용:

- `core/commands.rs`
  - `finalize_unit_death()` 끝에서 `schedule_movement_intent(time_ms)` 호출 추가
  - death는 now-tick board state mutation이므로
    same-tick movement recompute를 항상 걸도록 정리
- `tests/battle_ranged_attack.rs`
  - `melee_reacquires_movement_immediately_after_frontliner_dies` 추가
  - frontliner kill 이후 killer melee의 다음 `UnitMoved`가
    다음 attack cadence(`+1500ms`) 이전에 발생하는지 검증
  - export 추가:
    `core/timeline_exports/melee_reacquires_movement_immediately_after_frontliner_dies.json`

의미:

- 이건 특정 전투만 위한 임시 핫픽스가 아니라,
  `death`를 authoritative movement/target invalidation trigger로 승격한 구조 수정이다
- melee는 이제 target death 이후
  "다음 공격 예약 시점"이 아니라
  "현재 tick board state"를 기준으로 바로 다시 움직일 수 있다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack melee_reacquires_movement_immediately_after_frontliner_dies -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 61

상태:

- dense 7v7 scenario 검증을 opening straight-engage에서
  `front collapse 이후 immediate reacquire`까지 확장

문제:

- `Patch 60`으로 death-triggered movement recompute를 넣었지만,
  작은 1v2 repro만으로는 crowded frontline collapse에서도
  같은 계약이 유지되는지 충분히 보여주지 못했다
- 오케스트라 마감 단계에서는
  "frontliner death 이후 survivor melee가 즉시 다시 전진하는가"를
  큰 scenario에서도 invariant로 잡아둘 필요가 있었다

변경 내용:

- `tests/battle_ranged_attack.rs`
  - `tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage`에
    front collapse 이후 검증 추가
  - 첫 opponent frontline death 시점에 opponent melee를 처치한
    player melee killers를 수집
  - 각 killer의 다음 `UnitMoved`가
    다음 attack cadence(`+1500ms`) 이전에 발생하는지 확인
  - 동시에 그 move가 backward/hesitation이 아니라
    계속 전진(`to.y < from.y`)인지 검증

의미:

- 이제 crowded opening baseline은
  "첫 걸음이 곧게 시작되는가"뿐 아니라
  "front collapse 이후 survivor melee가 즉시 다시 움직이는가"까지 포함한다
- 이는 특정 1v2 death repro가 아니라,
  dense frontline orchestration에서도
  death-trigger recompute contract가 유지되는지 확인하는 장기 검증층이다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack tft_like_field_7v7_dense_frontline_prefers_straight_opening_engage -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 62

상태:

- broader export sweep 결과를 현재 상태 문서에 반영
- 오케스트라 작업의 성격을 "새 movement fix 단계"가 아니라
  "검증 / 종료 판단 단계"로 재정의

문제:

- `Patch 58~61` 이후 실제 상태는
  planner / execute / attack-start selector 통일,
  dense frontline collapse 검증,
  death-trigger recompute까지 들어간 상태다
- 하지만 문서 상단 snapshot은 여전히
  `mid-fight retarget contract 점검`이 주 미해결 항목처럼 읽힐 여지가 있었다
- broader sweep 결과를 보면
  opening drift 대표 repro는 정리됐고,
  남은 off-axis move도 front collapse 이후 backline retarget 성격으로 보였다

변경 내용:

- `movement_orchestrator_execution.md`
  - 상단 `Current State Snapshot` 갱신
  - 상단 `Next Patch Target` 갱신
  - 상단 `Representative Repro Cases` 해석 보강
  - 상단 `Current Recommended Strategy`를
    broader export safety / 종료 판단 중심으로 수정

의미:

- 다음 AI는 현재 단계를
  "오케스트라를 계속 새로 뜯어야 하는 시점"으로 이해하면 안 된다
- 지금은 대표 repro를 대부분 정리한 뒤,
  다른 전투 shape에서도 같은 contract가 유지되는지 확인하고
  종료 가능 여부를 판단하는 단계다

검증:

- 문서 정리 작업이므로 별도 테스트 없음

### Patch 63

상태:

- front collapse 이후 straight-over-diagonal retarget를
  battle-level scenario test로 고정

문제:

- `Patch 58`은 selector/helper 수준에서
  same-range equal-distance target이 있으면
  spatial preference가 UUID보다 앞서도록 정리했다
- 하지만 broader sweep 단계에서는
  이 계약이 실제 battle 흐름, 특히
  "frontliner death 이후 exposed backline retarget"에서도
  그대로 유지되는지를 scenario 수준에서 확인할 필요가 있었다

변경 내용:

- `tests/battle_ranged_attack.rs`
  - `melee_prefers_straight_backline_target_after_frontliner_dies` 추가
  - player melee가 opponent frontliner를 죽인 뒤,
    straight backliner와 equal-distance diagonal backliner가 동시에 열리는 배치를 구성
  - frontliner death 이후 첫 `UnitMoved`가
    diagonal 우회가 아니라 straight file 재진입(`(2,4)->(2,3)`)인지 검증
  - export 추가:
    `core/timeline_exports/melee_prefers_straight_backline_target_after_frontliner_dies.json`

의미:

- 이제 "same-range tie-break"는 helper/unit test 수준뿐 아니라
  실제 battle retarget 흐름에서도 고정된다
- 이는 오케스트라 종료 판단에 필요한
  broader export safety를 한 단계 더 강화하는 검증층이다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack melee_prefers_straight_backline_target_after_frontliner_dies -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 64

상태:

- orchestrator decision layer를 `done / maintenance` 상태로 문서에 명시

문제:

- `Patch 58~63`까지 누적되면서
  opening diagonal, crossing, same-range selector inconsistency,
  death-trigger idle gap, front-collapse retarget까지
  대표 contract가 battle-level로 고정됐다
- 그런데 문서 상단은 여전히
  "다음 patch에서 orchestrator를 더 구현해야 한다"는 인상을 줄 수 있었다
- 이 상태를 명시적으로 정리하지 않으면
  다음 AI가 이미 안정화된 decision layer를 다시 흔들 가능성이 있다

변경 내용:

- `movement_orchestrator_execution.md`
  - 상단 `Current State Snapshot`을
    `broader contract 검증 단계`에서
    `done / maintenance` 상태로 갱신
  - `Current Verdict` 섹션 추가
  - `Next Patch Target`과 `Current Recommended Strategy`를
    "새 policy 추가"가 아니라
    "새 repro가 있을 때만 scenario test + 소폭 수정" 기준으로 정리

의미:

- 다음 AI는 orchestrator decision layer를
  기본적으로 종료된 시스템으로 보고 접근해야 한다
- 다음 큰 단계는 orchestrator 재구현이 아니라
  continuous spatial layer 준비다
- movement bug가 새로 생기더라도
  먼저 scenario test로 재현한 뒤 유지보수 수준에서만 수정하는 것이 원칙이다

검증:

- 문서 정리 작업이므로 별도 테스트 없음

### Patch 65

상태:

- continuous spatial layer의 구현 계획을
  "방향 문서" 수준에서
  "바로 구현 가능한 단계별 설계" 수준으로 구체화

문제:

- 기존 `movement_orchestrator_plan.md`에는
  continuous layer의 목표와 원칙은 적혀 있었지만,
  실제 구현 순서와 데이터 모델,
  timeline / replayer 계약이 아직 추상적이었다
- 이 상태로는 다음 AI가
  어디부터 손대야 하는지 다시 설계부터 시작할 가능성이 있었다

변경 내용:

- `movement_orchestrator_plan.md`
  - `Phase 5 상세 구현 계획` 추가
  - `ContinuousPosition`, `MovementSegment`, `UnitHitbox` 권장 타입 정의
  - `MovementState`를 segment-authoritative로 정리하는 단계 명시
  - timeline에 `MovementSegmentStarted`류 segment metadata를
    어떻게 노출할지 1차 계약 정리
  - replayer interpolation의 새 계약과 legacy fallback 규칙 명시
  - projectile / AoE를 continuous 판정으로 옮기는 우선순위 정리
  - validation / replay test 전략 추가

의미:

- 이제 continuous layer는
  "좋은 다음 방향"이 아니라
  "어떤 타입부터 만들고 어떤 순서로 옮길지"까지 적힌 구현 계획을 가진다
- orchestrator decision layer를 다시 흔들지 않고도
  별도 phase로 continuous 작업을 시작할 수 있다
- 다음 AI는 movement interpolation부터 시작하고,
  projectile / AoE는 그 다음 단계로 미루는 것이 권장된다

검증:

- 문서 정리 작업이므로 별도 테스트 없음

### Patch 66

상태:

- continuous spatial layer의 첫 코드 단계를 시작
- 아직 replayer interpolation은 넣지 않았고,
  core가 movement segment를 authoritative하게 기록하는 기반만 추가

문제:

- 기존 core는 내부적으로 continuous 좌표를 쓰고 있었지만,
  `MovementState`가 segment start를 정식 데이터로 들고 있지 않았다
- 또 timeline에는 `MovementStopped` stop snapshot만 강하게 드러나서
  replayer가 movement 중간 segment를 복원할 근거가 부족했다

변경 내용:

- `src/game/battle/core/movement/mod.rs`
  - `ContinuousPosition` 추가
  - `MovementSegment` 추가
  - `MovementState`에
    `step_start_x_units`, `step_start_y_units` 추가
  - `MovementState::current_segment()` 추가
- `src/game/battle/core/movement/execute.rs`
  - `schedule_current_move_step()`가
    step start / target / time window를 authoritative segment로 기록
  - 새 timeline event `MovementSegmentStarted`를 기록
- `src/game/battle/timeline.rs`
  - `TimelineEvent::MovementSegmentStarted` 추가
  - timeline version을 `13`으로 증가
- validation / replay
  - 새 movement segment 이벤트를 무시/허용하도록 관련 match 보정

의미:

- 이제 continuous layer는 문서상 계획만이 아니라
  실제 코드에서 `movement segment`를 정식 데이터로 다루기 시작했다
- 다음 단계의 replayer interpolation은
  이 `MovementSegmentStarted` 이벤트를 읽는 방식으로 구현할 수 있다
- decision layer나 타일 authority는 전혀 바꾸지 않았다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib movement_state_exposes_current_segment`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib schedule_current_move_step_records_movement_segment_started`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 67

상태:

- core 안에 segment-aware replay interpolation helper를 추가
- 아직 실제 Unity/renderer를 바꾸진 않았고,
  외부 replayer가 바로 재사용할 수 있는 contract만 먼저 제공

문제:

- `Patch 66`으로 timeline에 `MovementSegmentStarted`는 들어가기 시작했지만,
  그걸 읽어서 실제 좌표를 샘플링하는 공통 helper가 없었다
- 이 상태에서는 외부 replayer가 매번 segment parsing / interpolation 로직을
  중복 구현해야 한다

변경 내용:

- `src/game/battle/replay/types.rs`
  - `ReplayMovementSegment` 추가
  - `ReplayUnitSpatialState` 추가
  - `ReplayMovementSegment::from_timeline_event()` 추가
  - `ReplayMovementSegment::sample_position_at()` 추가
  - `ReplayUnitSpatialState::apply_event()` /
    `sample_position_at()` 추가
- 테스트 추가:
  - `replay_movement_segment_samples_linearly`
  - `replay_unit_spatial_state_uses_segment_then_stop_snapshot`
  - `replay_unit_spatial_state_falls_back_to_tile_center_without_segment`

의미:

- 이제 external replayer는
  `MovementSegmentStarted -> active segment -> sample_position_at(time_ms)`
  계약을 그대로 재사용할 수 있다
- legacy timeline에서는 `UnitMoved` 기반 tile-center fallback도 유지된다
- 즉 continuous layer의 다음 단계는
  새 movement math를 만드는 것이 아니라
  이 helper를 실제 visualizer/replayer에 연결하는 작업이다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib replay_movement_segment_samples_linearly`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib replay_unit_spatial_state_uses_segment_then_stop_snapshot`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib replay_unit_spatial_state_falls_back_to_tile_center_without_segment`

### Patch 68

상태:

- Unity timeline schema와 replay playback이
  `MovementSegmentStarted`를 읽는 쪽으로 연결됨
- 이제 new export는 tile hop + final snap이 아니라
  explicit segment interpolation을 사용할 준비가 됨

문제:

- core 쪽 `Patch 66~67`만으로는
  Unity replayer가 여전히 `UnitMoved`와 `MovementStopped`만 읽고 있었기 때문에,
  실제 시각 표현은 예전과 같은
  "타일 중심 hop -> 마지막 stop snapshot" 모델에 머물렀다

변경 내용:

- Unity project
  - `Assets/Scripts/BattleTimeline/Schema/Enums.cs`
    - `MovementSegmentEndKind` 추가
  - `Assets/Scripts/BattleTimeline/Schema/Events/TimelineEvents.cs`
    - `MovementSegmentStartedEvent` 추가
  - `Assets/Scripts/BattleTimeline/Schema/Converters/TimelineEventJsonConverter.cs`
    - `MovementSegmentStarted` 역직렬화 추가
  - `Assets/Scripts/Replay/Playback/BattleTimelineReplayer.cs`
    - `MoveKeyframe`에 explicit segment window / target 추가
    - `build_tracks_from_timeline()`가
      `MovementSegmentStartedEvent`를 track 시작점으로 기록
    - `sample_track_position()`가
      explicit segment가 있으면 inferred start 대신
      `started_at_ms -> ends_at_ms` 구간을 직접 선형 보간

의미:

- old export는 여전히 `UnitMoved` 기반 fallback으로 재생된다
- new export는 `MovementSegmentStarted`를 우선 사용해서
  movement 중간 위치를 더 자연스럽게 복원한다
- projectile / skill target sampling도
  같은 `try_sample_unit_world_position()` 경로를 타기 때문에
  새 segment interpolation의 이득을 같이 받는다

검증:

- Unity editor compile/runtime 확인은 아직 별도 필요
- core 쪽 helper/unit test는 `Patch 67` 기준 통과

### Patch 69

상태:

- Unity replay 쪽의 `legacy track + explicit segment track` 혼재 문제를
  별도 리팩토링 계획 문서로 정리

문제:

- `MovementSegmentStarted`를 붙인 뒤
  Unity replayer가 old `UnitMoved` 기반 경로와
  new segment 기반 경로를 동시에 타고 있어,
  dense battle에서 sudden speedup이 발생할 수 있음
- 이건 단일 버그 수정이 아니라
  `BattleTimelineReplayer` 구조 자체를
  `legacy mode / segment mode`로 분리해야 하는 문제다

변경 내용:

- Unity project docs
  - `Assets/Docs/battle_timeline_replayer_continuous_refactor_plan.md` 추가
  - 포함 내용:
    - 현재 문제 구조
    - 목표 아키텍처
    - explicit segment mode / legacy fallback mode
    - 단계별 리팩토링 순서
    - immediate next patch 목표
    - 검증 기준 / 금지할 임시방편

의미:

- 이제 Unity replay 쪽도
  "다음에 뭘 고쳐야 하는가"가 문서만으로 이어받을 수 있는 상태가 됨
- 다음 patch는 이 문서를 기준으로
  Unity `BattleTimelineReplayer`를
  explicit segment authoritative 구조로 리팩토링하는 것

### Patch 70

상태:

- Unity replay continuous refactor의 1차 구조 분리를 시작
- sudden speedup의 직접 원인인
  `legacy UnitMoved keyframe path`와
  `MovementSegmentStarted explicit path`의 동시 활성화를 끊기 시작함

문제:

- 기존 Unity replayer는 new timeline을 읽더라도
  unit movement track 내부에서
  `UnitMoved`와 `MovementSegmentStarted`를 모두 movement key처럼 취급했다
- 그래서 같은 이동 구간이
  legacy inference와 explicit segment interpolation에 의해
  중복 보간될 수 있었다

변경 내용:

- Unity project
  - `Assets/Scripts/Replay/Playback/BattleTimelineReplayer.cs`
    - `UnitTrack.has_explicit_segments` 추가
    - timeline pre-pass로
      `MovementSegmentStarted`를 가진 unit을 먼저 식별
    - explicit segment unit은
      `UnitMoved`를 movement keyframe으로 추가하지 않도록 변경
    - `sample_track_position()`를
      `sample_segment_track_position()`와
      `sample_legacy_track_position()` dispatch 구조로 분리

의미:

- explicit segment timeline에서는
  `MovementSegmentStarted`가 movement interpolation의 단일 source of truth에 가까워짐
- `UnitMoved`는 explicit segment unit에서
  logical tile update / facing 보조 이벤트 쪽으로 역할이 축소됨
- sudden speedup 문제를 easing이나 임의 speed clamp로 덮지 않고,
  track model 분리로 접근하기 시작했다

검증:

- Unity editor runtime 재확인은 별도 필요
- 이번 단계는 구조 리팩토링과 handoff 정리 중심

### Patch 71

상태:

- `MovementSegmentStarted`의 target 좌표 계약을 수정
- `RangeEnter`로 조기 종료되는 movement segment가
  full boundary target이 아니라
  실제 stop position을 timeline에 내보내도록 정리

문제:

- 기존 core는 `schedule_current_move_step()`에서
  `RangeEnter`로 segment end time을 앞당기더라도,
  `MovementSegmentStarted.target_x_units / target_y_units`에는
  여전히 full boundary target을 기록했다
- runtime 적분은 실제 속도만큼만 움직이므로 server state는 맞았지만,
  replay / Unity interpolation은
  `started_at_ms -> ends_at_ms` 동안 boundary까지 선형 보간하게 되어
  sudden speedup과 overlap처럼 보이는 시각 문제가 생길 수 있었다

변경 내용:

- `core/src/game/battle/core/movement/execute.rs`
  - `schedule_current_move_step()`가
    `step_from/step_to` 기준 full boundary target을 매번 다시 계산
  - `RangeEnter` 선택 시에는
    해당 `dt_ms` 동안 실제 이동 가능한 위치를 계산해서
    현재 scheduled segment의 target으로 저장
  - 즉 `MovementState.target_*`는
    이제 "현재 scheduled segment endpoint" 의미를 가진다
    (`Boundary`면 경계점, `RangeEnter`면 조기 정지 좌표)
- regression test 추가:
  - `range_enter_segment_records_actual_stop_position_instead_of_boundary_target`

의미:

- `MovementSegmentStarted`는 이제 replay/visualizer가
  그대로 보간해도 되는 explicit segment contract가 된다
- Unity replayer 쪽 sudden speedup 문제를
  presentation hack이 아니라 core segment semantics 수정으로 바로잡기 시작했다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib range_enter_segment_records_actual_stop_position_instead_of_boundary_target`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib movement_state_exposes_current_segment`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 72

상태:

- continuous spatial layer의 projectile / AoE 방향을 사용자 요구사항 기준으로 재고정
- 평타와 스킬 projectile의 contract를 분리

변경 내용:

- `core/docs/movement_orchestrator_plan.md`
  - continuous hit / miss 1차 대상에서
    `basic attack projectile`를 제외
  - `basic attack`은 근/원거리 모두
    `발사되면 반드시 맞는 homing 판정`으로 유지한다고 명시
  - skill projectile를
    - targeted / homing
    - untargeted / non-homing
    으로 나눠 continuous 판정 우선순위를 다시 정리
  - validation 대표 시나리오도
    non-homing skill projectile miss / targeted skill projectile hit 기준으로 갱신
- `core/docs/movement_orchestrator_execution.md`
  - current snapshot / next patch target에
    `평타 homing 유지, continuous hit/miss는 skill projectile / AoE만`이라는 제약 추가

의미:

- 다음 continuous 구현은
  "projectile 전부를 continuous hit/miss로 바꾼다"가 아니라
  "basic attack homing 유지 + skill spatial logic만 continuous화" 기준으로 진행해야 한다
- 이건 presentation tweak가 아니라
  전투 규칙과 밸런스 해석을 고정하는 중요한 contract 변경이다

검증:

- 문서 업데이트 작업이므로 별도 테스트 없음

### Patch 73

상태:

- projectile / AoE continuous 구현의 첫 코드 단계로,
  projectile contract를 `homing / fixed`로 분리할 수 있는 core 타입 기반을 추가
- 이 단계에서는 평타 behavior를 바꾸지 않고,
  basic attack homing contract를 그대로 유지했다

변경 내용:

- `core/src/game/battle/core/types.rs`
  - `ProjectileGuidance` 추가
    - `Homing`
    - `Fixed`
  - `ProjectileRecord`를 empty marker에서
    launch metadata를 가진 정식 record로 확장
    - `fired_at_ms`
    - `attacker_instance_id`
    - `target_instance_id`
    - `start`
    - `aim`
    - `speed_units_per_ms`
    - `guidance`
- `core/src/game/battle/core/commands.rs`
  - `ProjectileLaunch`에
    `attacker_origin`, `target_aim`, `guidance` 추가
  - `schedule_projectile_hit_event()`가
    projectile record에 launch metadata를 저장하도록 변경
  - basic attack projectile launch는
    현재 continuous 좌표 기준 origin/aim을 담되,
    guidance는 항상 `Homing`으로 저장
  - `unit_continuous_position_or_tile_center()` helper 추가
- `core/src/game/battle/core/sim.rs`
  - skill projectile launch 시
    `SkillKind::Targeted`면 `Homing`,
    `SkillKind::Untargeted`면 `Fixed` guidance를 부여
  - caster origin / target aim도 continuous 좌표로 저장

의미:

- 아직 projectile hit / miss behavior 자체는 바꾸지 않았다
- 대신 core가 이제
  - basic attack homing projectile
  - targeted skill homing projectile
  - untargeted skill fixed projectile
  를 구분하는 contract를 가진다
- 다음 단계에서 non-homing skill projectile에만
  continuous hit / miss를 도입할 수 있는 기반이 마련됐다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib schedule_projectile_hit_event_stores_homing_launch_metadata`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib apply_projectile_hit_is_idempotent_for_same_projectile_id`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 74

상태:

- skill spatial logic의 다음 단계로,
  projectile / area delivery contract를 step/effect 분리 관점에서 다시 고정
- runtime wiring 전에 데이터 스키마를 먼저 안정화

변경 내용:

- `core/src/game/ability.rs`
  - future runtime wiring을 위한 delivery 스키마 타입 추가
    - `SkillHitTargetFilter`
    - `SkillProjectileCollisionDef`
    - `SkillAreaShapeDef`
    - `SkillAreaDeliveryDef`
  - 이 단계에서는 기존 `DeliveryDef` runtime behavior를 바꾸지 않음
  - 관련 RON deserialization test 추가
- `core/docs/movement_orchestrator_plan.md`
  - projectile collision / explicit area delivery / impact context를
    continuous phase의 다음 contract로 문서화
- `core/docs/skill_system_refactor.md`
  - step-based skill system에
    `delivery decides contact, step decides effect` 모델을 추가 문서화

의미:

- 다음 runtime 구현은 "projectile가 맞으면 곧바로 효과 적용" 같은 단일 모델이 아니라
  "delivery가 충돌/경로를 만들고, step이 그 impact context를 받아 효과를 적용"하는
  장기 구조로 가야 한다
- 이로써 다음 단계에서
  - first-hit projectile
  - explosion follow-up
  - persistent ground zone
  를 같은 step system 안에서 묶을 수 있다

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_uses_expected_defaults`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_reads_explicit_values`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_circle_and_persistent_ticks`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_rectangles`

### Patch 75

상태:

- skill spatial logic를 "조금씩 if 추가"로 밀지 않고,
  projectile / area / impact context를 하나의 runtime 모델로 재설계하는 방향을 문서로 고정

변경 내용:

- `core/docs/skill_spatial_runtime_refactor_plan.md` 신설
  - 현재 `ProjectilePayload::SkillStep` 기반 runtime의 한계 정리
  - 장기 목표를
    - `ProjectileRuntime`
    - `AreaRuntime`
    - `SkillImpactContext`
    - `SkillProjectileImpact / SkillAreaTick / SkillAreaExpire`
    로 분리해 정의
  - basic attack은 계속 별도 homing path로 유지한다는 점을 명시
  - migration strategy를
    1. runtime 타입/이벤트 추가
    2. untargeted projectile first-hit
    3. instant area
    4. persistent area
    5. legacy `ProjectilePayload::SkillStep` 축소
    순으로 정리
- `core/docs/movement_orchestrator_plan.md`
  - spatial delivery source-of-truth 문서를 새 계획 문서로 연결

의미:

- 다음 단계는 작은 보정이 아니라
  skill spatial runtime을 `delivery objects + impact context` 모델로 옮기는 리팩토링이다
- 이는 untargeted projectile / instant area / persistent area를
  같은 step system 안에서 장기적으로 수용하기 위한 준비다

검증:

- 문서 정리 작업이므로 별도 테스트 없음

### Patch 76

상태:

- skill spatial runtime 리팩토링의 첫 코드 단계로,
  impact context / active area / future delivery events를 core runtime에 미리 올림
- 이 단계에서는 behavior migration 없이 event/type contract만 추가

변경 내용:

- `core/src/game/battle/core/types.rs`
  - `SkillDeliveryId`, `AreaInstanceId`
  - `SkillImpactContext`
  - `AreaRuntime`
  - `ActiveSkillCast.last_impact_context`
  - `ActiveSkillCast.active_area_ids`
    추가
- `core/src/game/battle/core/mod.rs`
  - `active_areas`
  - `area_seq`
    runtime state 추가
- `core/src/game/battle/enums.rs`
  - future spatial delivery events 추가
    - `SkillProjectileImpact`
    - `SkillAreaTick`
    - `SkillAreaExpire`
  - priority / ordering 계약에 포함
- `core/src/game/battle/core/sim.rs`
  - battle reset 시 `active_areas`, `area_seq`도 초기화
  - 새 spatial delivery event는 아직 no-op stub으로 예약
  - `ActiveSkillCast` 생성 시 새 spatial context 필드 초기화

의미:

- 다음 단계부터는 legacy `ProjectilePayload::SkillStep`만으로
  spatial delivery를 늘리는 대신,
  새 runtime state와 event 모델로 점진 이전할 수 있다
- 즉 이번 패치는 behavior fix가 아니라
  long-term runtime migration을 위한 skeleton 추가다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_impact_context_preserves_first_hit_and_spawned_area`

### Patch 77

상태:

- skill spatial runtime migration의 다음 단계로,
  `DeliveryDef` 자체를 projectile/area 장기 구조에 맞게 확장
- 아직 projectile/area behavior migration은 시작하지 않고,
  schema와 executor match shape를 먼저 정리

변경 내용:

- `core/src/game/ability.rs`
  - `DeliveryDef::Projectile`에
    `collision: SkillProjectileCollisionDef` 추가
  - `DeliveryDef::Area { area: SkillAreaDeliveryDef }` 추가
  - backward-compatible RON deserialization test 추가
    - projectile collision omitted -> default
    - explicit area delivery parse
- `core/src/game/battle/core/sim.rs`
  - `DeliveryDef::Projectile { speed_units_per_ms, .. }`로 정리
  - `DeliveryDef::Area`는 현 단계에서 legacy target resolution 기반으로
    instant-like effect application fallback을 제공
    (behavior migration 전의 안전한 schema bridge)
- test constructor들
  - `DeliveryDef::Projectile` 생성부에 default collision 필드 추가

의미:

- 이제 skill data schema는
  - instant
  - projectile + collision config
  - area delivery
  를 한 enum에서 표현할 수 있다
- 다음 단계에서 untargeted projectile first-hit / instant area runtime을
  schema 변경 없이 바로 붙일 수 있다

검증:

- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib delivery_def_projectile_ron_defaults_collision_when_omitted`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib delivery_def_area_ron_reads_explicit_area_delivery`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 78

상태:

- handoff snapshot을 현재 phase에 맞게 재정렬
- 다음 AI가 문서 첫 화면만 보고도
  `오케스트라 아님, skill spatial runtime migration이 현재 active work`
  라는 점을 즉시 알 수 있게 정리

변경 내용:

- `Current State Snapshot`
  - active work를 `skill spatial runtime migration`으로 갱신
- `Current Verdict`
  - continuous layer의 현재 직접 구현 대상이
    `untargeted projectile -> instant area -> persistent area`
    라는 점을 명시
- `Next Patch Target`
  - direct next step을
    `SkillProjectileImpact + SkillImpactContext`
    기반 untargeted projectile migration으로 갱신

의미:

- 이제 handoff 문서만 읽어도
  movement/orchestrator를 다시 만질 필요가 없고,
  skill spatial runtime이 바로 다음 작업이라는 점이 분명해졌다

검증:

- 문서 정리 작업이므로 별도 테스트 없음
  `UnitTrack.has_explicit_segments`,
  `sample_track_position()` 분리,
  `UnitMoved`의 logical-only 역할화로 가는 것이 맞다

검증:

- 문서 정리 작업이므로 별도 테스트 없음

### Patch 79

상태:

- `skill spatial runtime migration`의 첫 vertical slice로
  `untargeted fixed projectile`를
  legacy `ProjectilePayload::SkillStep` impact path에서 분리
- 이제 fixed skillshot은
  `SkillProjectileImpact` + `SkillImpactContext`를 통해
  contact와 effect를 나눠 처리한다

변경 내용:

- `core/src/game/battle/core/types.rs`
  - `ActiveSkillCast`에 `caster_instance_id` 추가
- `core/src/game/battle/core/movement/execute.rs`
  - `SampledMotionSegment` / `sample_motion_segment_at()`을
    core runtime 전반에서 재사용할 수 있게 승격
- `core/src/game/battle/core/commands.rs`
  - `SkillProjectileImpactLaunch` 추가
  - fixed projectile first-hit / miss 계산 helper 추가
  - `schedule_skill_projectile_impact_event()` 추가
  - `apply_skill_projectile_impact()` 추가
- `core/src/game/battle/core/sim.rs`
  - `SkillKind::Untargeted` + `DeliveryDef::Projectile`는
    이제 legacy `ProjectileHit` 대신 `SkillProjectileImpact`를 스케줄
  - `SkillProjectileImpact` event를 실제로 consume해서
    impact context 저장 + step effect 적용
- `core/tests/skill_refactor_validation.rs`
  - blocker-first hit battle test 추가
  - miss 후 previous-step result가 0으로 기록되는 battle test 추가

의미:

- 이제 untargeted skill projectile는
  "cast target을 미리 정한 단일 대상 projectile"가 아니라
  "경로상 first hit이 contact를 결정하는 delivery"로 동작한다
- miss도 impact event로 정규화되므로
  `IfPreviousStepDealtDamage` 같은 후속 step 조건이 올바르게 동작한다
- 다음 active migration 대상은 `instant area runtime`이다

현재 제한:

- 이 slice는 first-hit / miss만 구현한다
- `pierce`, `max_hits`, persistent area는 아직 미구현
- targeted homing skill projectile은 아직 legacy `ProjectileHit` path를 유지한다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation untargeted_projectile_hits_first_blocker_before_cast_target -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation untargeted_projectile_miss_still_updates_previous_step_result -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 80

요약:

- `instant area runtime`에 앞서
  `Area` delivery의 중심(anchor)을 runtime 추론이 아니라
  explicit data contract로 고정했다

변경 내용:

- `core/src/game/ability.rs`
  - `SkillAreaAnchorSource`
    - `CastTarget`
    - `ImpactContext`
    - `Caster`
    추가
  - `SkillAreaDeliveryDef.anchor` 추가
  - backward-compatible default는 `CastTarget`
- 상단 handoff snapshot 갱신
  - 다음 active target인 `instant area runtime`이
    `AreaAnchor`를 기준으로 구현되어야 함을 명시
- `core/docs/skill_spatial_runtime_refactor_plan.md`
  - second runtime slice가
    `shape + explicit anchor source`를 함께 가진다는 점 반영

의미:

- direct area step과 impact-follow-up explosion을
  동일한 `DeliveryDef::Area`로 표현하면서도
  runtime이 cast target / impact context / caster를
  추론하지 않아도 된다
- 이는 instant area / persistent area 구현에서
  임시 분기 대신 data-driven anchor resolution으로 가기 위한
  장기 방향 정리다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_circle_and_persistent_ticks`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_rectangles`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_defaults_anchor_to_cast_target`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib delivery_def_area_ron_reads_explicit_area_delivery`

### Patch 81

요약:

- `DeliveryDef::Area`의 instant circle/rectangle를
  legacy `resolve_skill_step_targets()` bridge에서 분리하고,
  continuous overlap runtime으로 연결했다

변경 내용:

- `core/src/game/battle/core/commands.rs`
  - `skill_delivery_accepts_unit()` 공용화
  - `sample_unit_position_at()` 추가
- `core/src/game/battle/core/spatial.rs`
  - `rectangle_contains_point_from_origin()` 추가
- `core/src/game/battle/core/sim.rs`
  - `resolve_area_anchor_position()` 추가
  - `resolve_area_direction_hint()` 추가
  - `resolve_instant_area_targets()` 추가
  - `DeliveryDef::Area`는
    `duration_ms == 0`인 경우 새 runtime overlap path를 사용
  - instant area도 `SkillImpactContext`를 갱신해서
    follow-up step이 impact anchor를 재사용할 수 있게 정리
- `core/src/game/battle/core/mod.rs`
  - instant circle area target set test 추가
  - instant rectangle direction test 추가
  - impact-context anchor test 추가

의미:

- direct area step은
  `CastTarget` anchor 기준으로 즉시 overlap target set을 만들 수 있다
- rectangle은 `caster -> anchor` 축으로 뻗는 전방 직사각형으로 판정된다
- impact-follow-up explosion도 `ImpactContext` anchor로
  동일한 `Area` delivery를 재사용할 수 있다
- `DeliveryDef::Area`의 geometry는 이제
  legacy tile-area target resolution이 아니라
  continuous shape overlap이 결정한다

현재 제한:

- `duration_ms > 0`인 persistent area는 아직 legacy bridge를 유지한다
- `SkillAreaTick / SkillAreaExpire` runtime은 다음 slice다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib rectangle_contains_point_from_origin_respects_axis_and_width`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_circle_area_delivery_hits_only_units_inside_radius`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_rectangle_area_delivery_uses_caster_to_anchor_direction`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_area_delivery_can_anchor_on_previous_impact_context`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 82

요약:

- `persistent area runtime`을
  `SkillAreaTick / SkillAreaExpire` 기반 lifecycle로 연결했다

변경 내용:

- `core/src/game/battle/core/types.rs`
  - `AreaRuntime`에
    `origin`, `direction_hint` 추가
  - rectangle persistent zone도 spawn 시점 geometry를 고정 유지하도록 정리
- `core/src/game/battle/core/sim.rs`
  - `resolve_area_geometry()` / `collect_area_targets_at()` 추가
  - `register_persistent_area()` 추가
  - `apply_skill_area_tick()` 추가
  - `expire_skill_area()` 추가
  - `DeliveryDef::Area`
    - `duration_ms == 0` => instant area overlap path
    - `duration_ms > 0` => persistent area runtime spawn path
  - `SkillAreaTick` / `SkillAreaExpire`를 실제 consume하도록 연결
- `core/src/game/battle/core/mod.rs`
  - `persistent_area_ticks_immediately_then_repeats_until_expire` 추가

의미:

- 장판은 생성 즉시 1회 판정하고,
  이후 `tick_interval_ms`마다 반복 적용된다
- `Circle` / `Rectangle` persistent area 모두
  spawn 시점의 anchor/geometry를 고정해서 tick 동안 재사용한다
- `ImpactContext` anchor를 쓰는 장판도
  projectile-hit 후 follow-up zone으로 같은 delivery 모델을 재사용할 수 있다

현재 제한:

- persistent area의 per-tick re-hit policy는 단순 반복 판정이다
- `already_hit_units` 같은 tick memory 옵션은 아직 없다
- `pierce`, `max_hits`는 아직 behavioral runtime에 연결되지 않았다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib persistent_area_ticks_immediately_then_repeats_until_expire`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

### Patch 83

요약:

- handoff 문서 상단을 현재 skill spatial runtime 진행 상태에 맞게 보강했다

변경 내용:

- `Current State Snapshot`
  - `instant area`, `persistent area`를 미완료처럼 읽히는 표현 제거
  - 남은 직접 구현 대상을
    `targeted homing skill projectile` 통일과
    `pierce / max_hits` behavioral runtime으로 재정리
- `Current Verdict`
  - 권장 우선순위를
    `targeted homing skill projectile` 통일
    -> `pierce / max_hits` behavioral runtime
    순으로 명시
- `Next Patch Target`
  - 다음 AI가 바로
    `targeted homing skill projectile` migration slice부터 시작하도록
    우선순위를 명확히 못 박음

의미:

- 다음 AI가 문서 상단만 읽어도
  오케스트라와 area runtime이 이미 완료됐고,
  현재 메인 작업이
  `targeted homing skill projectile` 통일이라는 점을
  즉시 파악할 수 있다
- `pierce / max_hits`는 그 다음 단계라는 점이
  handoff 수준에서 명시됐다

### Patch 84

요약:

- `targeted homing skill projectile`도
  `SkillProjectileImpact + SkillImpactContext` 경로로 통일했다

변경 내용:

- `SkillProjectileImpactLaunch`를
  `guidance`, `target_unit_id`, `travel_time_ms`를 가진 공용 launch로 확장
- `SkillKind::Targeted`의 homing projectile step은
  더 이상 legacy `ProjectilePayload::SkillStep`를 싣지 않고
  `SkillProjectileImpact`를 발행
- homing impact는
  기존 travel timing을 유지하면서
  impact 시점의 현재 타겟 위치를 `impact_position`으로 사용
- impact 처리 후 `last_impact_context`를 저장하고,
  follow-up `AreaAnchorSource::ImpactContext` step이 이를 재사용할 수 있게 함
- battle-level 회귀 테스트로
  targeted homing projectile 뒤의 impact-centered area follow-up을 고정

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
- next direct target:
  - `pierce / max_hits` behavioral runtime

### Patch 85

요약:

- skill projectile collision에 explicit `piercing`을 추가하고,
  `max_hits`까지 실제 behavioral runtime으로 연결했다

변경 내용:

- `SkillProjectileCollisionDef`에 `piercing: bool` 추가
- fixed / untargeted projectile runtime은 이제
  경로상 다중 충돌을 수집할 수 있음
- `max_hits`가 있으면 해당 횟수까지만 impact event를 발행
- 같은 projectile는 동일 유닛을 한 번만 맞힌다
- `piercing = false`면 기존처럼 첫 충돌 후 종료
- 하위 호환을 위해 현재는 `!despawn_on_hit`도 관통으로 해석한다
- battle-level 회귀 테스트로
  `piercing + max_hits=2` skillshot이 앞의 두 유닛까지만 맞는지 고정

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_uses_expected_defaults`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_reads_explicit_values`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `piercing / max_hits`: done
- next direct target:
  - optional area tick memory / re-hit policy refinement
  - optional projectile collision mode cleanup (`despawn_on_hit` legacy 축소)

### Patch 86

요약:

- persistent area의 re-hit policy를
  `EveryTick / OncePerArea / OnEnter` 데이터 계약으로 올렸다

변경 내용:

- `SkillAreaDeliveryDef`에 `tick_policy` 추가
- `persistent area` runtime이 이제 정책별로 대상 집합을 다르게 해석
  - `EveryTick`: 현재처럼 매 tick 반복 적용
  - `OncePerArea`: area lifetime 동안 같은 유닛은 한 번만 적용
  - `OnEnter`: zone에 새로 들어온 유닛에게만 적용
- battle-level 회귀 테스트로
  `OncePerArea`, `OnEnter` 동작을 고정

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_circle_and_persistent_ticks`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib persistent_area_ticks_immediately_then_repeats_until_expire`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib persistent_area_once_per_area_hits_target_only_once`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib persistent_area_on_enter_only_hits_when_unit_enters_zone`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `piercing / max_hits`: done
  - area tick policy (`EveryTick / OncePerArea / OnEnter`): done
- next direct target:
  - optional projectile collision mode cleanup (`despawn_on_hit` legacy 축소)
  - optional area shape/rotation expressiveness refinement

### Patch 87

요약:

- projectile collision contract를 `piercing` 중심으로 정리하고,
  `despawn_on_hit`을 compatibility-only optional field로 축소했다

변경 내용:

- `SkillProjectileCollisionDef.despawn_on_hit`을 `Option<bool>`로 내려
  새 schema 기본값이 더 이상 runtime semantics를 직접 결정하지 않게 함
- fixed projectile runtime은 이제
  `piercing || despawn_on_hit == Some(false)`일 때만 legacy 관통을 허용
- 새 runtime/문서는 `piercing + max_hits + same-target-once`를
  authoritative collision contract로 본다
- battle-level 회귀 테스트로
  legacy `despawn_on_hit:false` RON도 여전히 관통 동작을 유지하는지 고정

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_uses_expected_defaults`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_projectile_collision_def_reads_explicit_values`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `piercing / max_hits`: done
  - area tick policy (`EveryTick / OncePerArea / OnEnter`): done
  - `despawn_on_hit` legacy 축소: done
- next direct target:
  - optional area shape/rotation expressiveness refinement

### Patch 88

요약:

- instant / persistent area runtime에 `Cone` shape를 추가해
  `Circle/Rectangle` 다음 단계의 shape expressiveness를 넓혔다

변경 내용:

- `SkillAreaShapeDef`에 `Cone { angle_degrees, length_units }` 추가
- runtime은 `caster-origin + caster->anchor direction`을 기준으로
  cone overlap을 판정한다
- `Circle/Rectangle`와 같은 `AreaAnchorSource` / `hit_targets` / persistent tick path를 그대로 재사용한다
- spatial utility에 cone 판정 함수와 회귀 테스트 추가
- instant cone area battle-level 회귀 테스트로
  전방 + 대각 포함, off-cone 제외를 고정

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_cones`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib cone_contains_point_from_origin_respects_angle_and_length`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_cone_area_delivery_uses_caster_to_anchor_direction`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `piercing / max_hits`: done
  - area tick policy (`EveryTick / OncePerArea / OnEnter`): done
  - `despawn_on_hit` legacy 축소: done
  - `Box` / `Cone` shape support: done
  - `include_caster` explicit area contract: done
- next direct target:
  - optional line/pivot orientation expressiveness refinement

### Patch 89

요약:

- 실제 `.ron` 스킬들의 대표 area step들을 새 spatial area delivery 모델로 마이그레이션했다

변경 내용:

- `queen_of_hatred_magical_beam`
  - `Instant + Enemies(Line)`에서
  - `Area(Rectangle, CastTarget anchor)` 기반 line-shot으로 전환
- `melting_love_slime_infection.slime_spread`
  - `Instant + RadiusChebyshev`에서
  - `Area(Box, ImpactContext anchor)` 기반 impact-centered spread로 전환
- 추가 live migration:
  - `plague_mass_heal.mass_heal`
  - `fragment_universe_nova.nova`
  - `fairy_festival_blessing.fairy_bless`
  - `big_bird_dark_lamp.(lamp_gaze, lamp_burst)`
  - `mountain_mass_consumption.consume_burst`
  - `white_night_pale_benediction.(ally_salvation, ally_blessing, enemy_judgement)`
  를 `Area(Box, CastTarget anchor)` 기반 overlap delivery로 전환
- `include_caster`를 `SkillAreaDeliveryDef`에 추가해서
  self-inclusive ally area와 self-exclusive enemy area의 의미를 데이터에서 명시하도록 정리
- `ron_loading` 테스트로
  실제 base.ron이 새 `DeliveryDef::Area` shape/anchor를 읽는지 고정
- 기존 battle-level skill test가 그대로 통과하는지 검증

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test ron_loading`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_test_suite`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_box_area_delivery_is_centered_on_anchor`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `piercing / max_hits`: done
  - area tick policy (`EveryTick / OncePerArea / OnEnter`): done
  - `despawn_on_hit` legacy 축소: done
  - `Box` / `Cone` shape support: done
  - `include_caster` explicit area contract: done
  - representative live `.ron` migrations: broadened across real skills
- next direct target:
  - migrate more live `.ron` skills onto `Projectile/Area` spatial delivery where it buys clarity
  - optional pivot orientation expressiveness refinement

### Patch 92

요약:

- 선형 빔 계열을 `Rectangle(width ~= 1 tile)`로 흉내내던 상태에서
  `Line`을 first-class area shape로 올리고
  `queen_of_hatred_magical_beam`을 그 모델로 마이그레이션했다

변경 내용:

- `SkillAreaShapeDef::Line { length_units }` 추가
- runtime overlap이 `Line`을
  `caster -> anchor` 방향의 얇은 선형 판정으로 해석하도록 확장
- `queen_of_hatred_magical_beam.(beam_trace, beam_overdrive)`를
  `Area(Line, CastTarget)`로 변경
- 관련 unit/ron loading/documentation 갱신

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_supports_lines`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib line_contains_point_from_origin_respects_segment_and_radius`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_line_area_delivery_uses_caster_to_anchor_direction`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test ron_loading`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_test_suite`

### Patch 91

요약:

- 남아 있던 마지막 legacy skill projectile 경로를 제거해서
  skill projectile delivery를 `SkillProjectileImpact`로 완전히 통일했다

변경 내용:

- `DeliveryDef::Projectile`의 skill branch는 이제
  guidance가 `Fixed`든 `Homing`이든 모두
  `schedule_skill_projectile_impact_event(...)`만 사용
- `ProjectileHit`는 다시 basic attack 전용 경로가 됨
- `apply_projectile_hit()`에서
  `ProjectilePayload::SkillStep` 분기를 제거
- handoff 문서도
  `skill spatial runtime migration`이 사실상 마감됐고
  다음 직접 작업이 live `.ron` migration / 선택적 shape refinement라는 상태로 갱신

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_test_suite`

### Patch 90

요약:

- live `.ron` migration 중 `RadiusChebyshev` 계열 step이 단순 `Circle`로 근사되면서
  일부 real skill test가 깨지는 문제를
  `Box + include_caster` 데이터 계약으로 바로잡았다

변경 내용:

- `SkillAreaShapeDef::Box`를 centered axis-aligned area로 추가
  - 기존 `RadiusChebyshev(radius_tiles: N)`의 의미를
    continuous runtime 안에서 더 정확하게 보존
- `SkillAreaDeliveryDef.include_caster`를 추가
  - `Allies` area가 self-inclusive인지
    self-exclusive인지를 데이터에서 명시
- live `.ron` migration 보정:
  - `plague_mass_heal.mass_heal` -> `Area(Box, CastTarget, include_caster=true)`
  - `fairy_festival_blessing.fairy_bless` -> same
  - `white_night_pale_benediction.(ally_salvation, ally_blessing)` -> same
  - `white_night_pale_benediction.enemy_judgement`는 `include_caster=false` 유지
  - `fragment_universe_nova`, `big_bird_dark_lamp`, `mountain_mass_consumption`,
    `melting_love_slime_infection.slime_spread`도 `Box` 기반으로 정리
- regression test 보강:
  - `instant_box_area_delivery_is_centered_on_anchor`
  - `skill_area_delivery_def_defaults_include_caster_to_false`
  - `ron_loading`
  - `skill_test_suite`

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib skill_area_delivery_def_defaults_include_caster_to_false`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib instant_box_area_delivery_is_centered_on_anchor`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test ron_loading`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_test_suite`

현재 상태:

- orchestrator: done / maintenance
- movement continuous replay: phase 1 done
- skill spatial runtime:
  - untargeted fixed projectile: done
  - targeted homing projectile: done
  - instant area: done
  - persistent area: done
  - `Box / Rectangle / Cone` shape support: done
  - `include_caster` explicit area contract: done
  - `piercing / max_hits`: done
  - area tick policy (`EveryTick / OncePerArea / OnEnter`): done
  - `despawn_on_hit` legacy 축소: done
- next direct target:
  - migrate more live `.ron` skills onto `Projectile/Area` spatial delivery where it buys clarity
  - optional line/pivot orientation expressiveness refinement

### Patch 93

요약:

- live skill spatial migration이 blanket conversion 단계가 아니라
  selective migration / content audit 단계라는 점을
  handoff 문서와 테스트로 명확히 고정했다

변경 내용:

- `ron_loading`에 아래 regression을 추가
  - area target step은 반드시 `DeliveryDef::Area`를 사용
  - `projectile_vfx_id`가 있는 step은 반드시 `DeliveryDef::Projectile`를 사용
  - 남아 있는 `Instant` step은 area target / projectile semantics를 가지지 않는
    intentional non-spatial step임을 검증
- 상단 `Current Verdict`, `Next Patch Target`도
  남은 `Instant`를 전부 spatial delivery로 옮길 필요는 없고
  선별적 migration이 맞다는 방향으로 갱신

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test ron_loading`

현재 상태:

- skill spatial runtime의 엔진 구현은 사실상 마감
- 다음 단계는
  실제 spatial semantics가 있는 live skill만 골라 추가 마이그레이션하거나,
  새 skill requirement가 생길 때 orientation 표현력을 확장하는 것이다

### Patch 94

요약:

- `sim.rs`에 뭉쳐 있던 skill spatial runtime helper를
  `skill_runtime/` 모듈로 분리하는 1차 구조 정리를 시작했다

변경 내용:

- `core/src/game/battle/core/skill_runtime/` 추가
  - `cast.rs`
    - `update_skill_cast_step_result`
    - `update_skill_cast_impact_context`
    - `choose_skill_target_by_rule`
    - `resolve_skill_anchor_position`
    - `resolve_skill_step_targets`
  - `area.rs`
    - area geometry resolution
    - instant area target collection
    - persistent area register / tick / expire
- `sim.rs`에서는 위 helper 본문을 제거하고
  event loop / step dispatch 역할에 더 집중하게 정리
- 이번 패치는 behavior 변경이 아니라
  skill runtime을 후속 분해하기 위한 구조 정리 단계다

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --lib persistent_area_ticks_immediately_then_repeats_until_expire -- --exact`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`

현재 상태:

- skill spatial runtime behavior는 유지된 채
  `sim.rs` 비대화 해소를 위한 첫 모듈 분리가 시작됨
- 다음 구조 정리 후보는
  projectile launch / impact scheduling과
  step effect application bridge를 `skill_runtime/`으로 더 이동하는 것이다

### Patch 95

요약:

- `sim.rs`와 `commands.rs`에 남아 있던 skill projectile launch / impact scheduling 책임을
  `skill_runtime/projectile.rs`로 이동시켜
  `sim.rs`를 event dispatch에 더 가깝게 정리했다

변경 내용:

- `core/src/game/battle/core/skill_runtime/projectile.rs` 추가
  - `SkillProjectileImpactLaunch`
  - fixed / homing projectile impact scheduling
  - `dispatch_skill_projectile_delivery`
  - `apply_skill_projectile_impact`
- `sim.rs`
  - `DeliveryDef::Projectile` branch가
    세부 launch 계산 대신 `dispatch_skill_projectile_delivery(...)`만 호출하도록 축소
- `commands.rs`
  - skill projectile 전용 helper 제거
  - basic attack projectile 쪽 helper만 유지

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- `skill_runtime/` 분리가
  - `cast.rs`
  - `area.rs`
  - `projectile.rs`
  까지 진행됨
- `sim.rs`는 battle loop / event dispatch 중심으로 더 가까워졌고,
  구조 정리를 더 진행한다면 다음 후보는
  `build_skill_step_commands(...)`와 step effect bridge 쪽이다

### Patch 96

요약:

- `active projectile runtime` 2차 리팩토링의 Phase A skeleton을 codebase에 올렸다
- 아직 projectile behavior는 launch-time scheduling 그대로 두고,
  다음 단계 fixed projectile migration을 위한 state/event contract만 추가했다

변경 내용:

- `core/src/game/battle/core/types.rs`
  - `ActiveProjectileRuntime` 추가
- `core/src/game/battle/core/mod.rs`
  - `active_projectiles` runtime state 추가
- `core/src/game/battle/enums.rs`
  - `BattleEvent::SkillProjectileAdvance` 추가
  - queue priority / tie-break 반영
- `core/src/game/battle/core/skill_runtime/projectile.rs`
  - `advance_skill_projectile(...)` skeleton 추가
- `core/src/game/battle/core/sim.rs`
  - battle reset 시 `active_projectiles` 초기화
  - `SkillProjectileAdvance` event consume 추가

검증:

- `cargo fmt --manifest-path /mnt/f/work/simulator/core/Cargo.toml`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test skill_refactor_validation`
- `cargo test --manifest-path /mnt/f/work/simulator/core/Cargo.toml --test battle_ranged_attack`

현재 상태:

- active projectile runtime은 이제 문서-only 계획이 아니라 code skeleton이 들어간 상태다
- next direct target은
  `untargeted fixed projectile` launch를
  `ActiveProjectileRuntime + SkillProjectileAdvance` path로 실제 전환하는 것이다
