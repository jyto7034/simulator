# 멀티플레이어 게임 백엔드 향후 개선 계획

이 문서는 현재 백엔드 아키텍처를 기반으로, 상용 수준의 멀티플레이어 게임 서비스를 목표로 추가할 수 있는 기능 및 개선 사항을 요약합니다.

## 1. 매치메이킹 시스템 고도화

현재의 매치메이킹은 단순한 선입선출(FIFO) 방식입니다. 더 나은 플레이어 경험을 제공하기 위해 다음 기능들이 중요합니다.

### 1.1. 실력 기반 매치메이킹 (MMR/ELO)

-   **목표:** 비슷한 실력 수준의 플레이어들을 매칭시켜 공정하고 재미있는 게임을 제공합니다.
-   **구현 단계:**
    1.  **데이터베이스:** `auth_server`의 플레이어 정보에 `mmr` (또는 `elo`) 컬럼을 추가합니다.
    2.  **결과 처리:** 게임 결과를 처리하는 별도의 서비스를 만들거나 `dedicated_server`에 로직을 추가합니다. 게임 종료 시, 승패와 양 팀의 평균 MMR을 기반으로 MMR 변동을 계산하고 DB를 업데이트합니다.
    3.  **대기열 관리:** Redis의 Set을 **Sorted Set**으로 변경합니다.
        -   `ZADD` 명령어를 사용하여 플레이어의 MMR을 점수(score)로 하여 대기열에 추가합니다.
        -   `ZRANGEBYSCORE` 또는 `ZPOPMAX` 같은 명령어를 사용하여 비슷한 MMR 점수대의 플레이어들을 효율적으로 찾습니다.

### 1.2. MMR 범위 확장

-   **목표:** 플레이어가 너무 오래 기다리는 것을 방지하기 위해, 매칭 가능한 MMR 범위를 점진적으로 넓힙니다.
-   **구현 단계:**
    1.  **타임스탬프 저장:** 플레이어가 대기열에 들어갈 때, ID 및 MMR과 함께 입장 시간을 Redis에 저장합니다.
    2.  **동적 범위 계산:** `Matchmaker`의 `TryMatch` 로직은 다음을 수행해야 합니다.
        -   대기열에 있는 플레이어들의 대기 시간을 확인합니다.
        -   대기 시간에 따라 MMR 검색 범위를 (`mmr ± 50` -> `mmr ± 100` 등으로) 동적으로 확장합니다.

### 1.3. 파티 / 그룹 매치메이킹

-   **목표:** 플레이어들이 친구와 그룹을 맺어 함께 매칭을 신청할 수 있도록 합니다.
-   **구현 단계:**
    1.  **클라이언트:** 파티 생성, 초대, 관리 기능을 위한 UI와 로직을 구현합니다.
    2.  **매치 서버:**
        -   `match_server`는 파티 단위의 매칭 요청을 처리해야 합니다.
        -   `Matchmaker`는 파티를 단일 개체로 취급하고, 파티의 평균 MMR을 사용하여 매칭을 진행합니다.
        -   가급적 비슷한 규모와 평균 MMR을 가진 다른 파티와 매칭시키는 것을 우선적으로 고려할 수 있습니다.

## 2. 확장성 및 서버 관리

### 2.1. Dedicated Server 플릿(Fleet) 관리

-   **목표:** 수요에 따라 `dedicated_server` 인스턴스 풀을 동적으로 관리하고 확장/축소합니다.
-   **구현:**
    -   **플릿 매니저(Fleet Manager)** 서비스를 도입합니다. 이 서비스는 클라우드 제공업체(예: AWS EC2, GCP Compute Engine)나 컨테이너 오케스트레이터(Kubernetes) 환경에서 새로운 서버 인스턴스를 생성하고 관리하는 책임을 집니다.
    -   `dedicated_server` 인스턴스는 시작 시 플릿 매니저에 자신을 등록하고 상태(유휴, 사용 중)를 보고합니다.
    -   `match_server`는 더 이상 Redis를 직접 조회하지 않고, 플릿 매니저의 API를 통해 "사용 가능한 서버 인스턴스를 달라"고 요청합니다.
    -   **추천 도구:** AWS GameLift, Agones (Kubernetes 기반), 또는 직접 구현.

### 2.2. 지역(Region) 기반 매치메이킹

-   **목표:** 플레이어를 지리적으로 가까운 서버에 매칭시켜 지연 시간(ping)을 최소화합니다.
-   **구현 단계:**
    1.  **지역 감지:** 클라이언트가 자신의 지역을 확인합니다. (여러 엔드포인트에 핑 테스���를 하거나 유저가 직접 선택)
    2.  **지역별 대기열:** `match_server`는 각 지역별로 독립된 매칭 대기열을 운영합니다. (예: `queue:ap-northeast-2:1v1_ranked`)
    3.  **지역별 플릿:** 플릿 매니저는 각 클라우드 리전(예: 서울, 도쿄, 버지니아)별로 `dedicated_server` 인스턴스 풀을 관리합니다.

## 3. 안정성 및 사용자 경험

### 3.1. 매칭 수락/거절 기능

-   **목표:** 매칭이 성사되었을 때, 모든 플레이어가 준비되었는지 확인한 후 게임을 시작합니다.
-   **구현 단계:**
    1.  **새로운 상태:** 매칭이 감지되면, `Matchmaker`는 즉시 세션을 생성하는 대신 모든 플레이어에게 `ReadyCheck` 메시지를 보냅니다.
    2.  **플레이어 응답:** 플레이어들은 제한 시간(예: 10-15초) 내에 `Accepted` 응답을 보내야 합니다.
    3.  **확정:** 모든 플레이어가 수락하면, `Matchmaker`는 `dedicated_server`에 세션 생성을 요청합니다.
    4.  **실패/거절:** 한 명이라도 거절하거나 시간 내에 응답하지 않으면 매칭은 취소됩니다. 거절한 플레이어에게는 약간의 페널티를 부여하고, 나머지 플레이어들은 대기열의 가장 앞으로 다시 배치합니다.

### 3.2. 재접속 기능

-   **목표:** 게임 도중 의도치 않게 연결이 끊긴 플레이어가 진행 중이던 게임에 다시 참여할 수 있도록 합니다.
-   **구현 단계:**
    1.  **플레이어 상태 추적:** 중앙 서비스(또는 `auth_server`)가 플레이어의 현재 상태(`online`, `in-game` 등)와 참여 중인 게임의 `session_id`, `server_address`를 저장해야 합니다.
    2.  **재접속 로직:** 플레이어가 다시 로그인하면, 시스템은 해당 플레이어에게 진행 중인 게임 세션이 있는지 확인합니다.
    3.  **리다이렉트:** 진행 중인 세션이 있다면, 서버는 클라이언트에게 저장된 `dedicated_server`의 주소와 세션 정보를 보내 즉시 재접속하도록 유도합니다.
    4.  **Dedicated Server:** `dedicated_server`는 재접속한 플레이어를 다시 인증하고, 현재 게임 상태와 동기화시켜줄 수 있어야 합니다.