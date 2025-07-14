# 리팩토링 계획: Panic 및 경쟁 상태(Race Condition) 해결

이 문서는 `simulator_match_server`에서 발견된 심각한 안정성 문제인 **Panic 유발 가능성**과 **경쟁 상태**를 해결하기 위한 리팩토링 계획을 정의합니다.

---

## 1. Panic 위험성 해결

**문제점:** `unwrap()`, `expect()`, 경계 검사 없는 인덱싱 등은 코드 실행 중 `panic`을 유발할 수 있습니다. `panic`은 우리가 구현한 우아한 종료(Graceful Shutdown) 메커니즘을 우회하고 프로세스를 즉시 중단시켜, Redis에 오래된(stale) 데이터를 남기는 등 심각한 상태 불일치를 야기합니다.

**목표:** 모든 `panic` 유발 가능성을 제거하고, 오류 발생 시 적절한 로깅과 함께 작업을 안전하게 중단하거나 실패 처리하도록 코드를 수정합니다.

**대상 파일:** `simulator_match_server/src/matchmaker/handlers.rs`

### 수정 계획

#### 가. `TryMatch` 핸들러

-   **`SystemTime` 계산:**
    -   **현재 코드:** `.expect("...")`
    -   **수정:** `match` 또는 `if let` 구문을 사용하여 `Result`를 안전하게 처리합니다. 실패 시, 에러를 로깅하고 분산 락을 해제한 뒤 함수를 조기 반환합니다.
-   **Redis 스크립트 결과 처리:**
    -   **현재 코드:** `Uuid::parse_str(&script_result[1]).unwrap()` 및 `script_result[0].clone()`
    -   **수정:**
        -   `.get(index)` 메서드를 사용하여 `Option`을 안전하게 반환받습니다.
        -   결과가 `None`일 경우(스크립트 반환값이 예상과 다를 경우)를 처리하는 로직을 추가합니다. (예: 에러 로깅, 작업 중단)
        -   UUID 파싱은 `and_then(|s| s.parse().ok())` 와 같이 체이닝하여 `Option`으로 처리하고, 실패 시 대체 값을 사용하거나 작업을 중단합니다.

#### 나. `HandleLoadingComplete`, `CancelLoadingSession`, `CheckStaleLoadingSessions` 핸들러

-   **`script_result.remove(0)`:**
    -   **현재 코드:** 벡터가 비어있을 경우 `panic`을 유발합니다.
    -   **수정:** `remove(0)`을 호출하기 전에 `if !script_result.is_empty()` 또는 `if let Some(game_mode) = script_result.get(0).cloned()` 와 같은 가드(guard)를 추가하여 벡터가 비어있지 않음을 보장합니다. (실제 구현 시에는 `get`과 `remove`를 안전하게 조합하거나, `if-let`과 `remove`를 분리하여 사용)

-   **`CheckStaleLoadingSessions`의 `SystemTime` 계산:**
    -   `TryMatch` 핸들러와 동일한 방식으로 안전하게 수정합니다.

---

## 2. WebSocket 세션 경쟁 상태(Race Condition) 해결

**문제점:** 클라이언트가 짧은 시간 안에 `Enqueue` 메시지를 두 번 보낼 경우, 서버가 첫 번째 요청을 처리하여 세션 상태를 업데이트하기 전에 두 번째 요청이 도착할 수 있습니다. 이로 인해 `player_id` 중복 체크 로직이 우회되고, 동일한 플레이어가 매치메이킹 큐에 중복으로 등록되는 문제가 발생할 수 있습니다.

**목표:** `Enqueue` 요청 처리 로직을 원자적(atomic)으로 만들어 중복 요청을 확실하게 방지합니다.

**대상 파일:** `simulator_match_server/src/ws_session.rs`

### 수정 계획

-   **`MatchmakingSession` 상태 관리 강화:**
    -   현재 `Option<Uuid>`으로 관리되는 `player_id` 대신, 세션의 상태를 더 명확하게 표현하는 `enum`을 도입합니다. (예: `enum SessionState { Idle, Enqueuing, Enqueued, InLoading }`)
    -   `MatchmakingSession` 구조체에 `state: SessionState` 필드를 추가합니다.

-   **`Enqueue` 메시지 처리 로직 수정:**
    -   `ClientMessage::Enqueue` 메시지를 수신했을 때, 가장 먼저 현재 세션의 `state`를 확인합니다.
    -   만약 `state`가 `Idle`이 아니면, 이미 다른 요청이 처리 중이거나 큐에 등록된 상태이므로, 경고를 로깅하고 해당 요청을 무시합니다.
    -   `state`가 `Idle`일 경우, **다른 액터에게 메시지를 보내기 전에 즉시** `state`를 `SessionState::Enqueuing`으로 변경합니다.
    -   이후 `player_id`와 `game_mode`를 세션에 저장하고, `Matchmaker`와 `SubscriptionManager`에 메시지를 보내는 기존 로직을 수행합니다.

이 계획을 통해 시스템의 안정성과 데이터 정합성을 크게 향상시킬 수 있습니다.
