# Match Server 리팩토링 계획 (3차)

이 문서는 `new_issues.md`에서 제기된 치명적인 버그들을 해결하기 위한 최종 실행 계획을 기술한다.

**목표:** 경쟁 상태(Race Condition)와 단일 실패점(SPOF) 문제를 해결하여, `match_server`를 프로덕션 수준의 안정성을 갖춘 애플리케이션으로 완성한다.

---

## 1단계: 경쟁 상태(Race Condition) 문제 해결

-   **문제 ID:** #1 (new_issues.md)
-   **전략:** 로딩 세션 키를 즉시 삭제하는 대신, 상태 필드(`status`)를 추가하여 여러 핸들러가 세션의 현재 상태를 안전하게 공유하도록 한다.

-   **1.1. `CancelLoadingSession` 핸들러 수정**
    -   **파일:** `matchmaker/handlers.rs`
    -   **수정 사항:**
        -   `redis.del()` 호출을 제거한다.
        -   대신, `redis.hset(loading_key, "status", "cancelled")`를 호출하여 세션이 취소되었음을 명시적으로 표시한다.
        -   나머지 플레이어들을 재입장시키는 로직은 그대로 유지한다.

-   **1.2. `ATOMIC_LOADING_COMPLETE_SCRIPT` 수정**
    -   **파일:** `matchmaker/scripts.rs`
    -   **수정 사항:**
        -   스크립트 상단에서, `HGET loading_key "status"`를 통해 `status` 필드를 먼저 확인한다.
        -   만약 `status`가 `"cancelled"` 또는 다른 비정상 상태라면, 아무 작업도 수행하지 않고 즉시 빈 값을 반환한다.
        -   `status`가 정상일 경우에만 기존 로직(플레이어 상태를 "ready"로 변경하고, 모두 준비되었는지 확인)을 수행한다.

-   **1.3. `CheckStaleLoadingSessions` 핸들러 수정 (일관성 확보)**
    -   **파일:** `matchmaker/handlers.rs`
    -   **수정 사항:**
        -   타임아웃된 세션을 발견했을 때, `redis.del()`을 호출하는 대신 `redis.hset(loading_key, "status", "timed_out")`을 호출하도록 변경하는 것을 고려한다. 이는 로직의 일관성을 높이고, 다른 핸들러들이 타임아웃 상태를 인지할 수 있게 한다. (선택적이지만 권장됨)

---

## 2단계: 단일 실패점(SPOF) 문제 해결

-   **문제 ID:** #5 (new_issues.md)
-   **전략:** `RedisSubscriber` 액터에 자동 재연결 및 재시도 로직을 구현한다.

-   **2.1. `RedisSubscriber` 액터 로직 수정**
    -   **파일:** `pubsub.rs`
    -   **수정 사항:**
        -   `started` 훅에서는 `self.connect_and_subscribe(ctx)`를 호출만 하도록 변경한다.
        -   `connect_and_subscribe`라는 새로운 메소드를 구현한다.
        -   이 메소드 안에서 Redis 연결 및 `psubscribe`를 시도한다.
        -   **연결 실패 시:**
            -   에러를 로그에 남긴다.
            -   `ctx.run_later(delay, |act, ctx| act.connect_and_subscribe(ctx))`를 사용하여, 지연 시간(예: 5초) 후에 재시도하도록 스케줄링한다. (지수 백오프 적용 권장)
        -   **스트림 종료 시:**
            -   `while let Some(...)` 루프가 끝난 후 (이는 연결이 비정상적으로 끊겼음을 의미), "Connection closed, attempting to reconnect..." 로그를 남기고 다시 `ctx.run_later`를 통해 재연결을 시도한다.

---

이 계획에 따라 리팩토링을 진행하고, 각 단계가 끝날 때마다 Git에 커밋하여 작업 내역을 관리한다.
