# Match Server 리팩토링 계획 (2차)

이 문서는 `POTENTIAL_ISSUES.md`에서 제기된 문제점 중, GameLift Fleet 기능으로 해결되지 않는 나머지 문제들을 해결하기 위한 구체적인 실행 계획을 기술한다.

**목표:** 로직의 정확성을 높이고, Redis 커넥션 관리 아키텍처를 개선하여 `match_server`의 안정성과 확장성을 확보한다.

---

## 1단계: 유효성 검사 및 로직 오류 수정

이 단계에서는 비교적 간단한 수정으로 코드의 정확성을 높인다.

-   **1.1. (문제 #1 해결) `EnqueuePlayer` 핸들러에 게임 모드 검증 추가**
    -   **파일:** `simulator_match_server/src/matchmaker/actor.rs`
    -   **수정 사항:**
        -   `EnqueuePlayer` 메시지를 처리할 때, 요청에 포함된 `game_mode` 문자열이 `self.settings.game_modes`에 정의된 유효한 `id`인지 확인하는 로직을 추가한다.
        -   만약 유효하지 않은 게임 모드일 경우, 대기열에 추가하는 대신 `ServerMessage::Error`를 클라이언트에게 전송하여 즉시 피드백을 준다.

-   **1.2. (문제 #2 해결) 서버 할당 실패 시 올바른 대기열로 복귀하도록 수정**
    -   **파일:** `simulator_match_server/src/matchmaker/actor.rs`
    -   **수정 사항:**
        -   `ATOMIC_LOADING_COMPLETE_SCRIPT` Lua 스크립트가 플레이어 ID 목록을 반환할 때, `game_mode`도 함께 반환하도록 수정한다. (예: `return {game_mode, player_id_1, player_id_2, ...}`)
        -   `HandleLoadingComplete` 핸들러는 스크립트 결과값에서 `game_mode`를 추출하여 변수에 저장해 둔다.
        -   이후 `dedicated_server` 호출 실패 등으로 `requeue_players`를 호출해야 할 때, 하드코딩된 `"Normal_1v1"` 대신 이 변수에 저장된 정확한 `game_mode`를 사용한다.

---

## 2단계: Redis Pub/Sub 아키텍처 리팩토링

이 단계에서는 중앙 집중형 액터 모델을 도입하여 Redis 연결 관리 문제를 근본적으로 해결한다.

-   **2.1. 신규 액터 파일 생성**
    -   **파일:** `simulator_match_server/src/pubsub.rs` (신규 생성)
    -   **내용:**
        -   **`RedisSubscriber` 액터:** Redis Pub/Sub과의 모든 상호작용을 전담한다.
            -   시작 시, 단 하나의 전용 연결을 생성하고 `PSUBSCRIBE "notifications:*"`를 실행한다.
            -   수신한 모든 메시지를 `SubscriptionManager`에게 전달한다.
        -   **`SubscriptionManager` 액터:** `player_id`와 `ws_session` 액터 주소의 매핑을 관리한다.
            -   내부에 `HashMap<Uuid, Addr<MatchmakingSession>>`을 상태로 가진다.
            -   `Register`, `Deregister`, `ForwardMessage` 등의 메시지를 처리한다.

-   **2.2. `lib.rs` 및 `main.rs` 수정**
    -   **파일:** `simulator_match_server/src/lib.rs`
    -   **수정 사항:** 새로 만든 `pubsub` 모듈을 `pub mod pubsub;`으로 선언한다.
    -   **파일:** `simulator_match_server/src/main.rs`
    -   **수정 사항:**
        -   서버 시작 시, `RedisSubscriber`와 `SubscriptionManager` 액터를 생성하고 시작한다.
        -   `AppState` 구조체를 수정하여, 기존 `redis_client` 대신 두 액터의 주소(`Addr`)를 저장하도록 변경한다.

-   **2.3. `ws_session.rs` 로직 전면 수정**
    -   **파일:** `simulator_match_server/src/ws_session.rs`
    -   **수정 사항:**
        -   더 이상 `redis_client`를 상태로 갖지 않는다. 대신 `Addr<SubscriptionManager>`를 갖는다.
        -   `Enqueue` 메시지 수신 시, Redis에 직접 연결하는 대신 `SubscriptionManager`에게 `Register` 메시지를 보내 자신을 등록한다.
        -   액터 중지 시(`stopping` 훅), `SubscriptionManager`에게 `Deregister` 메시지를 보내 등록을 해제한다.
        -   서버로부터 오는 메시지(`ServerMessage`)는 이제 `SubscriptionManager`를 통해 직접 전달받는다.

---

이 계획에 따라 단계적으로 리팩토링을 진행하고, 각 주요 단계가 끝날 때마다 Git에 커밋하여 작업 내역을 관리한다.
