# "로딩 단계" 도입 리팩토링 계획

**목표:** 매칭 성공 후, 모든 플레이어가 리소스 로딩을 완료했을 때만 게임 서버(`dedicated_server`)를 할당하도록 시스템 흐름을 변경한다. 이를 통해 서버 리소스 낭비를 최소화하고, 모든 플레이어가 동시에 게임에 진입하는 경험을 보장한다.

---

## 상세 실행 계획

### 1단계: 프로토콜 및 상태 정의 변경

**목적:** "로딩" 상태를 관리하기 위한 새로운 메시지와 Redis 데이터 구조를 정의한다.

-   **1.1. `protocol.rs` 수정:**
    -   클라이언트 -> 서버 메시지에 `LoadingComplete { loading_session_id: Uuid }`를 추가한다.
    -   서버 -> 클라이언트 메시지에 `StartLoading { loading_session_id: Uuid }`를 추가한다.
-   **1.2. Redis 데이터 구조 설계 (신규):**
    -   로딩 중인 세션의 상태를 저장할 새로운 `Hash` 타입의 데이터 구조를 정의한다.
        -   **Key:** `loading:{loading_session_id}`
        -   **Fields:**
            -   `player_1_id`: "loading" 또는 "ready"
            -   `player_2_id`: "loading" 또는 "ready"
            -   `game_mode`: "Normal_1v1" (재입장 로직 등에 활용)

### 2단계: `Matchmaker` 로직 변경 (핵심)

**목적:** `Matchmaker`가 매칭 성공 시 서버를 바로 할당하는 대신, "로딩 세션"을 관리하도록 책임을 변경한다.

-   **2.1. `TryMatch` 핸들러 수정:**
    -   매칭이 성사되면, `DedicatedServerProvider`를 호출하는 대신 다음을 수행한다:
        1.  고유한 `loading_session_id`를 생성한다.
        2.  위에서 설계한 대로 Redis에 `loading:{loading_session_id}` 해시(Hash)를 생성하고, 매칭된 플레이어들의 상태를 모두 "loading"으로 초기화한다.
        3.  매칭된 모든 플레이어에게 `StartLoading { loading_session_id }` 메시지를 PUBLISH한다.
-   **2.2. `LoadingComplete` 메시지 핸들러 추가 (신규):**
    -   `Matchmaker`에 `HandleLoadingComplete { player_id: Uuid, loading_session_id: Uuid }` 라는 새로운 메시지와 핸들러를 추가한다.
    -   이 핸들러는 다음을 수행한다:
        1.  Redis에서 `loading:{loading_session_id}`의 해당 `player_id` 필드 값을 "ready"로 업데이트한다.
        2.  **(중요)** 해당 로딩 세션의 모든 플레이어 상태가 "ready"인지 확인한다.
        3.  **모두 "ready"일 경우에만,** `DedicatedServerProvider`에게 서버 할당을 요청하고, 최종 `MatchFound` 메시지를 플레���어들에게 PUBLISH하는 기존 로직을 실행한다.
        4.  모든 작업이 끝나면 `loading:{...}` 키를 Redis에서 삭제한다.

### 3단계: `ws_session.rs` 수정

**목적:** 클라이언트의 `LoadingComplete` 메시지를 받아 `Matchmaker`에게 전달하는 역할을 추가한다.

-   **3.1. `StreamHandler`의 `handle` 메소드 수정:**
    -   클라이언트로부터 `ClientMessage::LoadingComplete` 메시지를 수신하는 `match` 분기를 추가한다.
    -   해당 메시지를 받으면, `Matchmaker`에게 위에서 정의한 `HandleLoadingComplete` 메시지를 보낸다.

### 4단계: `test_client` 수정

**목적:** 새로운 로딩 흐름을 테스트할 수 있도록 클라이언트 로직을 수정한다.

-   **4.1. `main.rs` 로직 수정:**
    -   서버로부터 `StartLoading` 메시지를 받으면, `loading_session_id`를 저장하고 "로딩 중..." 메시지를 출력한다. (실제 클라이언트에서는 이 때 에셋 로딩 시작)
    -   임의의 시간(예: `tokio::time::sleep`) 동안 대기하여 로딩을 시뮬레이션한다.
    -   대기 시간이 끝나면, 저장해둔 `loading_session_id`와 함께 `LoadingComplete` 메시지를 서버로 전송한다.
    -   이후, 최종 `MatchFound` 메시지를 받을 때까지 계속 대기한다.

---
이 계획에 따라 리��토링을 진행하겠습니다.
