### 최종 아키텍처 요약 (Steam 연동 카드 게임 서버)

#### 1. 주요 구성 요소 (서버 컴포넌트)

| 컴포넌트                              | 역할                                                                                                        | 주요 기술/패턴                                           |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| **Auth Server (인증 서버)**           | - 스팀 인증 티켓 검증<br>- JWT(세션 토큰) 발급 및 갱신<br>- 신규 플레이어 DB 등록                           | Rust (Actix/Axum 등), Steamworks Web API, JWT 라이브러리 |
| **Matchmaking Server (매칭 서버)**    | - JWT로 플레이어 인가<br>- MMR 기반 플레이어 매칭<br>- 게임 서버 인스턴스 할당 및 접속 정보 전달            | Rust, **Redis (Sorted Set)**                             |
| **Game Dedicated Server (게임 서버)** | - 실제 게임 로직 처리 (1v1 카드 배틀)<br>- 게임 결과 처리 및 DB 기록 요청                                   | Rust                                                     |
| **PostgreSQL DB**                     | - **영구 데이터 저장소**<br>- 플레이어 정보, 프로필(MMR), 카드/덱 정보, 매치 기록 등 저장                   | PostgreSQL                                               |
| **Redis**                             | - **휘발성/상태 데이터 저장소**<br>- 매치메이킹 대기열(Queue) 관리<br>- (선택) 빠른 조회를 위한 데이터 캐싱 | Redis                                                    |
| **Load Balancer**                     | - Auth Server, Matchmaking Server 앞단에 위치<br>- 트래픽 분산 및 고가용성 확보                             | Nginx, AWS ELB, Cloudflare 등                            |

#### 2. 데이터베이스 스키마 (PostgreSQL)

- **`players`**: `SteamID64` (BIGINT, PK)를 키로 사용. 자체 인증 정보 없이 스팀 계정과 게임 데이터를 매핑.
- **`player_profiles`**: `player_id` (BIGINT, FK)를 통해 `players`와 연결. `MMR`, `레벨`, `경험치` 등 게임 고유의 성장 데이터만 저장.
- **`cards`, `player_card_collection`, `player_decks`, `deck_cards`**: 카드 및 덱 관련 정보 저장. `player_id`는 모두 `BIGINT` 타입.
- **`match_history`, `match_participants`**: 게임 결과 기록. `player_id`는 `BIGINT` 타입.

#### 3. 핵심 데이터 흐름 (End-to-End Flow)

**A. 최초 로그인 (인증 및 세션 생성)**

1.  **(Client)** 게임 실행 → 로컬 스팀 클라이언트로부터 **인증 티켓** 발급.
2.  **(Client → Auth Server)** `/login` API에 **인증 티켓**을 담아 요청.
3.  **(Auth Server ↔ Steam Auth Server)** 티켓 유효성 검증.
4.  **(Auth Server → DB)** 검증 성공 시, 신뢰할 수 있는 `SteamID64`로 DB(`players` 테이블) 조회 또는 신규 생성.
5.  **(Auth Server → Client)** 서버의 비밀 키로 서명된 **JWT(세션 토큰)** 발급하여 클라이언트에게 전달.

---

**B. 매치메이킹 시작**

1.  **(Client → Matchmaking Server)** `/queue/start` API 요청. `Authorization` 헤더에 **JWT** 포함.
2.  **(Matchmaking Server)**
    - JWT 서명 검증 (Auth Server와 동일한 비밀 키 사용).
    - 검증 성공 시, JWT에서 `SteamID64` 추출하여 사용자 식별.
3.  **(Matchmaking Server → DB/Redis Cache)** `SteamID64`를 키로 사용하여 `player_profiles`에서 **MMR** 등 매칭 정보 조회.
4.  **(Matchmaking Server → Redis)** 조회된 MMR과 `SteamID64`를 **Redis의 Sorted Set (대기열)**에 `ZADD` 명령어로 추가.

---

**C. 매칭 성공 및 게임 시작**

1.  **(Matchmaking Server)** Redis의 Sorted Set을 주기적으로 스캔하여 조건에 맞는 플레이어 탐색 (`ZRANGEBYSCORE`).
2.  **(Matchmaking Server)** 매칭 성사 시, Redis 대기열에서 해당 플레이어들 제거 (`ZREM`).
3.  **(Matchmaking Server)**
    - 사용 가능한 **Game Dedicated Server** 인스턴스 확보.
    - 해당 게임 세션 전용 **임시 접속 토큰** 생성.
4.  **(Matchmaking Server → Clients)** 매칭된 모든 클라이언트에게 **Game Dedicated Server의 IP/Port**와 **임시 접속 토큰**을 전송.
5.  **(Clients → Game Dedicated Server)** 전달받은 정보로 게임 서버에 접속 시도 (임시 토큰 포함).
6.  **(Game Dedicated Server)** 토큰을 검증하고 모든 플레이어 입장이 확인되면 게임 시작.

---

**D. 게임 종료 및 결과 처리**

1.  **(Game Dedicated Server)** 게임 종료 시, 최종 결과(승패, 스탯 등)를 집계.
2.  **(Game Dedicated Server → DB)** 집계된 데이터를 `match_history` 및 `match_participants` 테이블에 저장.
3.  **(Game Dedicated Server → DB)** Glicko-2 알고리즘 등으로 계산된 새로운 MMR, 랭크 포인트, 경험치 등을 `player_profiles` 테이블에 업데이트.
4.  **(Game Dedicated Server → Clients)** 클라이언트에게 결과 화면을 보여주기 위한 최종 데이터 전송 후, 연결 종료 및 인스턴스 반환.

---

### 아키텍처 다이어그램 (개념도)

```
[ Game Client ] <--(HTTPS/WSS)--> [ Load Balancer ]
      |                                    /         \
      |                            [ Auth Server ]  [ Matchmaking Server ]
      | (스팀 클라이언트 API)               |         /         \
      |                                    | (JWT Secret)  | (Redis)
[ Steam Client ] <-------------------- [ PostgreSQL DB ] -- [ Redis ]
      |
      | (인증)
      |
[ Steam Auth Server ]


[ Game Client ] <--(UDP/TCP)--> [ Game Dedicated Server ]
                                      |
                                      | (DB 쓰기)
                                      V
                                [ PostgreSQL DB ]
```
