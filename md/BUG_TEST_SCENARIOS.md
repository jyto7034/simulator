# 매치메이킹 서버 버그 시나리오 테스트 계획

이 문서는 `simulator_match_server`에서 발생할 수 있는 주요 버그 및 예외 상황에 대한 테스트 시나리오를 정의합니다.
`test_client` 또는 수동 테스트를 통해 아래 시나리오들을 검증하여 서버의 안정성을 확보하는 것을 목표로 합니다.

---

## 1. 실패 및 취소 시나리오 (Failure and Cancellation Scenarios)

### 시나리오 1: 로딩 중 플레이어 연결 종료

- **개요:** 매치가 성사되어 로딩 상태에 진입했으나, 한 명의 플레이어가 로딩을 완료하기 전에 연결을 종료하는 상황을 테스트합니다.
- **트리거:** `CancelLoadingSession` 핸들러
- **재현 방법:**
  1. 2명 이상의 플레이어가 매치메이킹을 시작하여 매치에 성공하고 `StartLoading` 메시지를 수신합니다.
  2. 플레이어 중 한 명(Player A)이 `LoadingComplete` 메시지를 보내기 전에 WebSocket 연결을 강제로 종료합니다.
  3. 나머지 플레이어(Player B)는 정상적으로 `LoadingComplete` 메시지를 전송하거나 대기합니다.
- **예상 결과:**
  - Player B는 "A player disconnected during loading. You have been returned to the queue." 와 같은 내용의 `Error` 메시지를 수신해야 합니다.
  - Player B는 자동으로 다시 매치메이킹 큐에 등록되어야 합니다.

### 시나리오 2: 로딩 타임아웃

- **개요:** 매치가 성사되었으나, 한 명 또는 그 이상의 플레이어가 의도적으로 또는 네트워크 문제로 인해 제한 시간(60초) 내에 로딩을 완료하지 못하는 상황을 테스트합니다.
- **트리거:** `CheckStaleLoadingSessions` 핸들러
- **재현 방법:**
  1. 2명 이상의 플레이어가 매치에 성공하여 `StartLoading` 메시지를 수신합니다.
  2. 플레이어 중 한 명 이상이 `LoadingComplete` 메시지를 60초 이상 보내지 않고 대기합니다.
- **예상 결과:**
  - 해당 로딩 세션에 있던 **모든** 플레이어는 "Matchmaking timed out. You have been returned to the queue." 와 같은 내용의 `Error` 메시지를 수신해야 합니다.
  - 모든 플레이어는 자동으로 다시 매치메이킹 큐에 등록되어야 합니다.

### 시나리오 3: 사용 가능한 Dedicated Server 없음

- **개요:** 모든 플레이어가 성공적으로 로딩을 완료했지만, 현재 시스템에 가용한(idle 상태) Dedicated Server가 없는 상황을 테스트합니다.
- **트리거:** `HandleLoadingComplete` 핸들러 내부의 `FindAvailableServer` 요청 실패
- **재현 방법:**
  1. 테스트 환경에서 모든 Dedicated Server를 "busy" 상태로 만듭니다. (예: Redis에서 `dedicated_server:*` 키의 상태 값을 수동으로 변경)
  2. 플레이어들이 매치에 성공하고 모두 `LoadingComplete` 메시지를 전송합니다.
- **예상 결과:**
  - 모든 플레이어는 "All dedicated servers are busy." 와 같은 내용의 `Error` 메시지를 수신해야 합니다.
  - 모든 플레이어는 자동으로 다시 매치메이킹 큐에 등록되어야 합니다.

### 시나리오 4: Dedicated Server 세션 생성 실패

- **개요:** 가용한 Dedicated Server를 찾았으나, 해당 서버의 API 호출(세션 생성)이 실패하는 상황을 테스트합니다.
- **트리거:** `HandleLoadingComplete` 핸들러 내부의 `http_client` 요청 실패
- **재현 방법:**
  1. 테스트용 Dedicated Server를 실행하되, `/session/create` 엔드포인트가 500 Internal Server Error를 반환하도록 수정합니다.
  2. 플레이어들이 매치에 성공하고 모두 `LoadingComplete` 메시지를 전송합니다.
- **예상 결과:**
  - 모든 플레이어는 "Dedicated server returned error..." 와 같은 내용의 `Error` 메시지를 수신해야 합니다.
  - 모든 플레이어는 자동으로 다시 매치메이킹 큐에 등록되어야 합니다.

---

## 2. 레이스 컨디션 검증 시나리오 (Race Condition Scenarios)

### 시나리오 5: "유령 플레이어" 버그 검증

- **개요:** `new_issues.md`에서 언급된 가장 치명적인 버그입니다. 플레이어의 로딩 완료와 다른 플레이어의 연결 종료가 거의 동시에 발생할 때, 남은 플레이어가 응답을 받지 못하고 무한 대기 상태에 빠지는지를 검증합니다. (코드는 수정되었으나, 반드시 재현 테스트가 필요합니다.)
- **트리거:** `CancelLoadingSession`과 `HandleLoadingComplete` 핸들러의 동시 실행
- **재현 방법:**
  1. 플레이어 A와 B가 매치되어 `StartLoading` 메시지를 수신합니다.
  2. **(타이밍이 중요)** 플레이어 A는 즉시 WebSocket 연결을 종료합니다.
  3. 거의 동시에, 플레이어 B는 로딩을 완료하고 `LoadingComplete` 메시지를 서버로 전송합니다.
- **예상 결과 (버그가 해결되었다면):**
  - 플레이어 B는 아무런 응답을 받지 못하고 멈추는 것이 아니라, "A player disconnected..." 와 같은 `Error` 메시지를 정상적으로 수신하고 다시 큐로 돌아가야 합니다.
- **실패 케이스 (버그가 존재한다면):**
  - 플레이어 B는 서버로부터 아무런 응답도 받지 못하고 클라이언트 상에서 영원히 대기하게 됩니다.
