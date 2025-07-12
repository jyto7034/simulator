# `simulator_match_server` 잠재적 버그 및 개선점 분석

## 1. `HandleLoadingComplete` 핸들러의 경쟁 상태 (Race Condition)

-   **문제점:** `ATOMIC_LOADING_COMPLETE_SCRIPT` 실행 후 분산 락(Distributed Lock)을 해제하고, 그 다음에 데디케이티드 서버를 찾는 로직(`provider_addr.send(FindAvailableServer)`)이 실행됩니다. 락이 해제된 시점과 서버를 찾는 시점 사이에 미세한 시간 틈이 존재하여, 다른 이벤트(예: `CheckStaleLoadingSessions`의 타임아웃 처리)가 개입할 여지가 있습니다.
-   **영향:** 정상적으로 매칭이 완료되었음에도 불구하고, 데디케이티드 서버 할당 전에 `loading` 세션 정보가 다른 프로세스에 의해 정리(clean up)될 수 있습니다. 이 경우, 매치는 실패하고 플레이어들은 다시 큐로 돌아가게 됩니다.
-   **해결 제안:** 데디케이티드 서버를 찾고, 세션을 생성하고, 플레이어들에게 `MatchFound` 메시지를 보내는 전체 과정을 분산 락 임계 영역(critical section) 안에서 수행하도록 로직을 변경해야 합니다.
    -   **주의:** 이 경우 락 점유 시간이 길어지므로, 외부 서비스(데디케이티드 서버)의 응답 지연 가능성을 고려하여 락의 타임아웃(`LOCK_DURATION_MS`)을 충분히 길게 설정해야 합니다.

## 2. `CancelLoadingSession` 핸들러의 경쟁 상태

-   **문제점:** `CancelLoadingSession` 핸들러와 `HandleLoadingComplete` 핸들러가 거의 동시에 실행될 경우, 예측하기 어려운 상호작용이 발생할 수 있습니다. 예를 들어, `CancelLoadingSession`이 락을 잡고 세션 키를 삭제한 직후, `HandleLoadingComplete`가 락을 획득하면 이미 키가 사라진 상태이므로 정상적으로 매치를 성사시키지 못하고 종료됩니다.
-   **영향:** 로직이 여러 핸들러에 걸쳐 복잡하게 얽혀 있어 동작을 예측하기 어렵고, 의도치 않은 실패 케이스를 유발할 수 있습니다.
-   **해결 제안:** 세션을 취소하고 플레이어들을 다시 큐에 넣는 로직을 하나의 원자적인 Redis Lua 스크립트로 통합하는 것을 고려할 수 있습니다. 이 스크립트는 다음과 같은 작업을 수행합니다.
    1.  `loading` 세션 키의 존재 여부를 확인합니다.
    2.  키가 존재하면 삭제합니다.
    3.  세션에 포함되었던 다른 플레이어들의 ID 목록을 반환하여, 애플리케이션 레벨에서는 이들에게 알림을 보내고 큐에 다시 넣도록 처리합니다.

## 3. `CheckStaleLoadingSessions`의 비효율성 및 경쟁 상태

-   **문제점:**
    1.  **비효율성:** 주기적으로 Redis의 모든 `loading:*` 키를 `SCAN` 명령으로 가져와 애플리케이션 레벨에서 루프를 도는 방식은, 동시 `loading` 세션의 수가 많아질 경우 서버에 부하를 줄 수 있습니다.
    2.  **경쟁 상태:** 각 키를 처리하기 위해 개별적으로 락을 획득하는 방식은 다른 핸들러와의 경쟁 상태를 완벽하게 방지하지 못합니다. `CheckStaleLoadingSessions`가 특정 세션을 "오래됨(stale)"으로 판단하고 삭제하려는 순간, `HandleLoadingComplete` 핸들러가 해당 세션의 매칭을 성공시킬 수 있습니다.
-   **영향:** 정상적으로 진행 중인 세션이 타임아웃으로 오인되어 정리되거나, 반대로 정리되어야 할 세션이 다른 핸들러의 개입으로 인해 방치될 수 있습니다.
-   **해결 제안:**
    -   **Redis TTL 활용:** `loading:*` 세션 키를 생성할 때 `EXPIRE` 명령어를 사용하여 키의 TTL(Time-To-Live)을 `LOADING_SESSION_TIMEOUT_SECONDS`와 유사한 값으로 설정합니다. 이렇게 하면 Redis가 만료된 키를 자동으로 삭제해주므로, `CheckStaleLoadingSessions` 액터의 필요성이 크게 줄어들거나 없어질 수 있습니다.
    -   **정리 로직이 필요한 경우:** 만약 키 만료 시 플레이어를 큐에 다시 넣는 등의 복잡한 처리가 반드시 필요하다면, `CheckStaleLoadingSessions` 대신 Redis의 [키스페이스 알림(Keyspace Notifications)](https://redis.io/docs/manual/keyspace-notifications/) 기능을 사용하여 만료 이벤트를 구독하고 처리하는 방식을 고려해볼 수 있습니다. (단, 키스페이스 알림은 at-least-once 전송을 보장하지 않으므로, 100% 신뢰성이 필요한 시스템에는 부적합할 수 있습니다.)

## 4. 코드 안정성: `unwrap()`의 무분별한 사용

-   **문제점:** 코드 곳곳에 `unwrap()`과 `expect()`가 사용되고 있습니다. (`serde_json::to_string(...).unwrap()`, `Uuid::parse_str(...).unwrap()` 등). 이러한 코드는 JSON 직렬화/역직렬화 실패나 문자열 파싱 실패 시 현재 스레드에 `panic`을 일으킵니다. `actix` 액터 모델에서는 한 액터의 패닉이 다른 액터나 전체 시스템에 영향을 줄 수 있습니다.
-   **영향:** 특정 플레이어가 비정상적인 형식의 ID를 전송하거나, 서버 내부 데이터 구조의 변경으로 인해 직렬화에 실패하는 경우, 해당 액터가 중단되거나 최악의 경우 전체 `match_server` 프로세스가 다운될 수 있습니다.
-   **해결 제안:** 모든 `unwrap()`과 `expect()` 호출을 `if let Ok(...)` 또는 `match` 구문으로 변경하여, 오류 발생 시 패닉 대신 적절한 에러 처리(예: 경고 로그 기록, 클라이언트에게 에러 메시지 전송, 해당 요청 무시)를 수행하도록 리팩토링해야 합니다.

