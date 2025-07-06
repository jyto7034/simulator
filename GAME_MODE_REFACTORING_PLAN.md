# 게임 모드 시스템 리팩토링 계획

**목표:** "일반 모드"를 기본으로 구현하되, 향후 "랭크 모드" 등 다양한 게임 모드를 쉽게 추가할 수 있는 확장성 있는 구조를 설계하고 적용한다.

---

## 상세 실행 계획

### 1단계: 게임 모드 설정 재정의

**목적:** 단순한 문자열 목록이었던 게임 모드 설정을, 각 모드의 속성(필요 인원, MMR 적용 여부 등)을 포함하는 구조적인 데이터로 변경한다.

-   **1.1. `config/development.toml` 수정:**
    -   `matchmaking.game_modes`를 단순 배열에서 **테이블 배열(array of tables)**로 변경한다.
    -   기존 `"1v1_ranked"`를 `"Normal_1v1"`로 변경하고, `required_players`와 `use_mmr_matching` 속성을 추가한다.
    -   향후 추가될 랭크 모드의 예시를 주석으로 남겨 확장성을 명확히 보여준다.

-   **1.2. `src/env.rs` 수정:**
    -   `MatchmakingSettings` 내부의 `game_modes` 필드 타입을 `Vec<String>`에서 `Vec<GameModeSettings>`로 변경한다.
    -   TOML의 테이블 구조에 맞춰 `id`, `required_players`, `use_mmr_matching` 필드를 가지는 `GameModeSettings` 구조체를 새로 정의한다.

### 2단계: `Matchmaker` 로직 수정

**목적:** `Matchmaker`가 새로운 게임 모드 설정 구조를 이해하고, 각 모드의 속성에 따라 다르게 동작할 수 있는 기반을 마련한다.

-   **2.1. `matchmaker/actor.rs`의 `TryMatch` 메시지 수정:**
    -   `TryMatch` 메시지가 `game_mode: String` 대신, 모든 속성 정보가 담긴 `game_mode: GameModeSettings`를 갖도록 변경한다.

-   **2.2. `Matchmaker`의 `started` 핸들러 수정:**
    -   서버 시작 시, `settings.game_modes` (`Vec<GameModeSettings>`)를 순회하며 각 `GameModeSettings` 객체를 `TryMatch` 메시지에 담아 보내도록 수정한다.

-   **2.3. `TryMatch` 핸들러 로직 수정:**
    -   메시지로 받은 `GameModeSettings` 객체에서 `id`를 가져와 Redis 키를 생성하고, `required_players`를 가져와 Lua 스크립트의 인자로 사용하도록 변경한다.
    -   **(핵심 확장성 설계)** `use_mmr_matching` 값에 따라 분기하는 `if`문을 추가한다. 지금 당장은 두 분기 모두 동일한 로직(단순 매칭)을 수행하지만, 이는 향후 랭크 모드의 MMR 기반 매칭 로직이 추가될 자리를 명확하게 보여준다.

### 3단계: 클라이언트 요청 수정

**목적:** `test_client`가 새로운 기본 게임 모드인 "Normal_1v1"로 매칭을 요청하도��� 수정한다.

-   **3.1. `test_client/src/main.rs` 수정:**
    -   `Enqueue` 메시지를 보낼 때, `game_mode`의 값을 `"1v1_ranked"`에서 `"Normal_1v1"`로 변경한다.

---
이 계획에 따라 리팩토링을 진행하겠습니다.
