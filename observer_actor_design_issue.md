# ObserverActor 설계 문제 분석

## 문제 개요

현재 ObserverActor의 per-player event tracking 구조에서 **점진적 ExpectEvent 추가**와 **완료 조건 판정** 사이의 근본적인 모순이 발견되었습니다.

## 현재 구조

```rust
pub struct ObserverActor {
    pub player_expectations: HashMap<Uuid, Vec<ExpectEvent>>, // 플레이어별 기대 이벤트
    pub player_steps: HashMap<Uuid, usize>, // 플레이어별 현재 step
    // ...
}
```

## 문제의 핵심

### 1. 점진적 ExpectEvent 추가 방식

```
ObserverActor 생성
└─ player_expectations = {} (빈 HashMap)

Player1 receives EnQueued
└─ on_enqueued() 실행
└─ "start_loading" ExpectEvent 생성 및 추가
└─ player_expectations[player1] = [ExpectEvent1] (길이: 1)

Player1 receives StartLoading  
└─ on_loading_start() 실행
└─ "match_found" ExpectEvent 생성 및 추가
└─ player_expectations[player1] = [ExpectEvent1, ExpectEvent2] (길이: 2)
```

### 2. 조기 완료 판정 문제

```rust
fn check_all_players_completed(&self, ctx: &mut actix::Context<Self>) {
    for (player_id, expectations) in &self.player_expectations {
        let current_step = *self.player_steps.get(player_id).unwrap_or(&0);
        if current_step < expectations.len() {  // ← 문제 지점
            all_completed = false;
            break;
        }
    }
}
```

## 구체적인 문제 시나리오

### 시간순 흐름

| 시점 | 이벤트 | player_expectations[player1].len() | current_step | 완료 판정 |
|------|--------|-----------------------------------|--------------|-----------|
| T1 | ObserverActor 생성 | 0 | 0 | ❌ (빈 HashMap) |
| T2 | EnQueued 받음 → on_enqueued() 실행 | 1 | 0 | ❌ (0 < 1) |
| T3 | start_loading 이벤트 매칭 | 1 | 1 | ✅ **잘못된 완료!** |
| T4 | StartLoading 받음 → on_loading_start() | 2 | 1 | ❌ (1 < 2) |
| T5 | match_found 이벤트 매칭 | 2 | 2 | ✅ 올바른 완료 |

### 문제점

**T3 시점에서 조기 완료 판정**:
- `current_step = 1`, `expectations.len() = 1`
- `1 < 1` = false → `all_completed = true`
- ObserverActor 종료
- 하지만 아직 `on_loading_start()`에서 추가될 ExpectEvent가 남아있음

## 근본적인 설계 모순

### 모순의 본질
```
❓ "아직 추가되지도 않은 ExpectEvent의 완료를 어떻게 기다릴 수 있는가?"
```

- **ExpectEvent 추가**: behavior 메서드 실행 시점에 **런타임에 동적으로** 발생
- **완료 조건 판정**: **현재까지 추가된 ExpectEvent만** 기준으로 판단
- **미래 ExpectEvent**: 아직 실행되지 않은 behavior에서 나올 ExpectEvent는 예측 불가

### 예시: Normal Player의 전체 흐름

```rust
// 예상되는 전체 ExpectEvent 시퀀스 (하지만 ObserverActor는 이를 모름)
[
    ExpectEvent("start_loading"),     // on_enqueued()에서 추가
    ExpectEvent("match_found"),       // on_loading_start()에서 추가  
    ExpectEvent("game_complete"),     // on_match_found()에서 추가 (가정)
]
```

**문제**: ObserverActor는 첫 번째 ExpectEvent만 보고 완료 판정을 내림

## 영향

### 1. 테스트 신뢰성 저하
- 시나리오가 중간에 조기 종료됨
- 중요한 검증 단계 (loading_complete → match_found) 건너뜀
- 버그 탐지 실패

### 2. 예측 불가능한 동작
- 플레이어의 behavior 순서에 따라 완료 시점이 달라짐
- 디버깅 어려움

### 3. 확장성 문제
- 새로운 behavior 추가 시 예상치 못한 조기 완료 발생
- 다단계 검증 시나리오 구현 불가

## 다음 단계

이 문제를 해결하기 위한 설계 대안 검토가 필요합니다:

1. **사전 등록 방식**: 시나리오 시작 시 모든 ExpectEvent 미리 정의
2. **단계별 완료 방식**: behavior 단계마다 완료 조건 설정
3. **명시적 종료 신호**: 특별한 "완료" ExpectEvent로 종료 제어
4. **타임아웃 기반**: 새 ExpectEvent 추가 중단 시 완료 판정
5. **상태 머신 방식**: 플레이어별 상태 전이 기반 완료 판정

각 방식의 장단점 분석과 구체적인 구현 방안이 필요합니다.