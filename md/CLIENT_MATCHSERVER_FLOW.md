# 클라이언트 <-> 매치 서버 통신 흐름 분석

이 문서는 `test_client`가 `match_server`에 접속하여 매칭이 완료되기까지의 전체적인 코드 흐름과 메시지 교환 과정을 상세히 기술한다.

## 주요 관련 컴포넌트

-   **`test_client`**: 매칭을 요청하는 테스트용 클라이언트.
-   **`match_server`**:
    -   `main.rs`: 웹소켓 엔드포인트(`/ws/`)를 설정하고 서버를 실행.
    -   `ws_session.rs`: 각 클라이언트의 웹소켓 연결을 담당하는 액터(Actor). 클라이언트와의 직접적인 통신을 처리.
    -   `protocol.rs`: 클라이언트와 서버가 교환하는 메시지(`ClientMessage`, `ServerMessage`)의 규격(Protocol)을 정의.
    -   `matchmaker/actor.rs`: 실제 매치메이킹 로직을 처리하는 핵심 액터. 대기열 관리, 매칭 시도, 로딩 상태 관리 등을 담당.
    -   **Redis**: `Matchmaker`가 대기열과 로딩 세션 상태를 저장하고, `ws_session`이 서버로부터의 알림을 수신하기 위한 Pub/Sub 채널로 사용.

---

## 통신 시퀀스 다이어그램 (단계별 흐름)

### 1단계: 웹소켓 연결 수립

1.  **Client (`test_client/main.rs`)**:
    -   `tokio_tungstenite::connect_async("ws://127.0.0.1:8080/ws/")`를 호출하여 서버에 웹소켓 연결을 시도한다.

2.  **Server (`match_server/main.rs`)**:
    -   `/ws/` 경로의 `GET` 요청을 수신한다.
    -   `matchmaking_ws_route` 핸들러가 실행된다.
    -   이 핸들러는 새로운 `MatchmakingSession` 액터를 생성하고, `actix_web_actors::ws::start`를 통해 클라이언트와 연결된 웹소켓 스트림을 이 액터에게 위임한다.

### 2단계: 매칭 요청 (Enqueue)

1.  **Client (`test_client/main.rs`)**:
    -   연결이 성공하면, `ClientMessage::Enqueue` 타입의 JSON 메시지를 생성하여 서버로 전송한다.
    -   `{"type":"enqueue","player_id":"...","game_mode":"Normal_1v1"}`

2.  **Server (`ws_session.rs`)**:
    -   `StreamHandler` 구현부가 클라이언트로부터 온 텍스트 메시지를 수신한다.
    -   메시지를 `ClientMessage` enum으로 역직렬화(deserialize)한다.
    -   `ClientMessage::Enqueue` 분기:
        -   자신의 상태(state)에 `player_id`와 `game_mode`를 저장한다.
        -   **(중요)** 별도의 비동기 작업(`ctx.spawn`)을 생성하여, 이 클라이언트만을 위한 Redis Pub/Sub 채널(`notifications:{player_id}`)을 구독(subscribe)한다. 이 작업은 앞으로 서버가 이 플레이어에게 보내는 모든 알림을 수신 대기한다.
        -   채널 구독이 성공하면, `Matchmaker` 액터에게 `EnqueuePlayer` 메시지를 보낸다.

3.  **Server (`matchmaker/actor.rs`)**:
    -   `EnqueuePlayer` 메시지를 수신한다.
    -   Redis의 `SADD` 명령어를 사용하여 `queue:Normal_1v1` Set에 해당 `player_id`를 추가한다.
    -   작업이 성공했음을 알리기 위해, Redis의 `PUBLISH` 명령어로 `notifications:{player_id}` 채널에 `ServerMessage::Queued` 메시지를 발행(publish)한다.

### 3단계: 매칭 요청 결과 수신

1.  **Server (`ws_session.rs`)**:
    -   2단계에서 생성했던 비동기 구독 작업이 `notifications:{player_id}` 채널로부터 `Queued` 메시지를 수신한다.
    -   이 메시지를 다시 자신의 액터 주소(`ctx.address()`)로 보낸다.
    -   `Handler<ServerMessage>` 구현부가 `Queued` 메시지를 처리한다.
    -   메시지를 JSON으로 직렬화(serialize)하여 웹소켓을 통해 클라이언트에게 전송한다.

2.  **Client (`test_client/main.rs`)**:
    -   서버로부터 `{"type":"queued"}` 메시지를 수신하고, 이를 콘솔에 출력한다.

### 4단계: 매칭 성공 및 로딩 시작

1.  **Server (`matchmaker/actor.rs`)**:
    -   주기적으로 실행되는 `TryMatch` 타이머가 동작한다.
    -   `queue:Normal_1v1` 대기열의 인원수가 2명 이상임을 확인하고, `SPOP` 명령어로 2명의 `player_id`를 꺼내온다.
    -   고유한 `loading_session_id`를 생성하고, Redis에 `loading:{id}` 해시(Hash)를 만들어 두 플레이어의 상태를 "loading"으로 기록한다.
    -   두 플레이어 각각의 `notifications:{player_id}` 채널에 `ServerMessage::StartLoading` 메시지를 발행한다.

2.  **Client (양쪽 모두)**:
    -   `ws_session`을 통해 `{"type":"start_loading", "loading_session_id":"..."}` 메시지를 수신한다.
    -   2초간의 로딩을 시뮬레이션(`tokio::time::sleep`)한다.
    -   로딩이 끝나면, `ClientMessage::LoadingComplete` 메시지를 서버로 전송한다.

### 5단계: 로딩 완료 및 게임 세션 생성

1.  **Server (`ws_session.rs`)**:
    -   클라이언트로부터 `LoadingComplete` 메시지를 수신하고, `Matchmaker`에게 `HandleLoadingComplete` 메시지를 보낸다.

2.  **Server (`matchmaker/actor.rs`)**:
    -   `HandleLoadingComplete` 메시지를 수신한다.
    -   **(핵심 로직)** `ATOMIC_LOADING_COMPLETE_SCRIPT` Lua 스크립트를 실행하여, 해당 플레이어의 상태를 "ready"로 바꾸고 모든 플레이어가 준비되었는지 **원자적으로 확인**한다.
    -   첫 번째 클라이언트의 요청 시에는 스크립트가 빈 값을 반환하므로 아무것도 하지 않는다.
    -   두 번째 클라이언트의 요청 시에는 스크립트가 "모두 준비됨"을 확인하고, 로딩 세션 키를 삭제한 뒤, 매칭된 플레이어 ID 목록을 반환한다.
    -   `Matchmaker`는 이 목록을 받고, `DedicatedServerProvider`를 통해 유휴 게임 서버를 찾은 후, HTTP 요청으로 `dedicated_server`에 게임 세션 생성을 요청한다.
    -   `dedicated_server`로부터 성공 응답을 받으면, `ServerMessage::MatchFound` 메시지를 생성한다.
    -   두 플레이어 각각의 `notifications:{player_id}` 채널에 `MatchFound` 메시지를 발행한다.

### 6단계: 최종 매칭 완료 및 종료

1.  **Client (양쪽 모두)**:
    -   `ws_session`을 통해 `{"type":"match_found", "session_id":"...", "server_address":"..."}` 메시지를 수신한다.
    -   "Match found!" 로그를 출력하고, `while` 루프를 `break`하여 프로그램을 정상적으로 종료한다.

2.  **Server (`ws_session.rs`)**:
    -   클라이언트의 연결이 끊어지면, `stopping()` 라이프사이클 훅이 호출된다.
    -   이때 `loading_session_id`가 여전히 남아있다면, `Matchmaker`에게 `CancelLoadingSession` 메시지를 보내 뒷정리를 시도한다. (��공 흐름에서는 이미 Lua 스크립트에 의해 키가 삭제되었으므로, `Matchmaker`는 "세션을 찾을 수 없음" 경고를 로그에 남기고 정상 종료된다.)
