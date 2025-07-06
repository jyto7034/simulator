# 알려진 문제점 및 해결 방안

이 문서는 현재 시스템에 남아있는 주요 문제점과 이를 해결하기 위한 계획을 기술합니다.

## 1. 고아 로딩 세션 (Orphaned Loading Sessions)

### 문제 상황

1.  두 플레이어가 매칭되어 로딩 세션(예: `loading:1234`)이 Redis에 생성됩니다.
2.  한 플레이어(Player B)가 로딩 중 클라이언트를 강제 종료하거나 네트워크 연결이 끊깁니다.
3.  Player B의 `MatchmakingSession`은 `stopping` 훅에서 `DequeuePlayer` 메시지를 보내지만, Player B는 이미 "대기열"이 아닌 "로딩 세션"에 있으므로 아무런 효과가 없습니다.
4.  **결과:** 다른 플레이어(Player A)는 영원히 Player B의 로딩 완료를 기다리게 되며, `loading:1234` 세션은 Redis에 "고아"처럼 남게 됩니다.

### 해결 방안: 로딩 취소 로직 도입

-   **`MatchmakingSession` 수정:** `stopping` 훅에서, 자신이 로딩 세션에 참여 중이었다면 `DequeuePlayer` 대신 `CancelLoadingSession { player_id, loading_session_id }`와 같은 새로운 메시지를 `Matchmaker`에게 보냅니다.
-   **`Matchmaker` 수정:** `CancelLoadingSession` 메시지를 처리하는 새로운 핸들러를 추가합니다.
    1.  Redis에서 해당 로딩 세션 정보를 가져옵니다.
    2.  연결이 끊긴 플레이어를 제외한 나머지 모든 플레이어에게 "매칭이 취소되었습니다. 대기열로 돌아갑니다." 라는 알림을 보냅니다.
    3.  나머지 플레이어들을 다시 대기열에 넣습니다 (`requeue_players` 로직 재사용).
    4.  Redis에서 해당 로딩 세션 키를 삭제하여 정리합니다.

## 2. 로딩 타임아웃 부재

### 문제 상황

-   플레이어가 `StartLoading` 메시지를 받았지만, 클라이언트의 버그나 매우 느린 네트워크 환경으로 인해 `LoadingComplete` 메시지를 합리적인 시간 내에 보내지 못합니다.
-   위 1번 문제와 마찬가지로, 함께 매칭된 다른 플레이어는 무한정 기다리게 됩니다.

### 해결 방안: 로딩 세션에 타임아웃 적용

-   **로딩 세션 생성 시 타임스탬프 기록:** `Matchmaker`가 로딩 세션을 생성할 때, Redis Hash에 `created_at: <unix_timestamp>` 필드를 함께 저장합니다.
-   **오래된 세션 정리 로직 추가:**
    1.  `Matchmaker`에 주기적으로 실행되는 `CheckStaleLoadingSessions` 라는 새로운 내부 메시지와 핸들러를 추가합니다. (기존 `TryMatch`��� 유사한 방식)
    2.  이 핸들러는 `SCAN` 명령어를 사용하여 `loading:*` 패턴의 키들을 조회합니다.
    3.  각 로딩 세션의 `created_at` 값을 현재 시간과 비교하여, 설정된 타임아웃(예: 60초)을 초과한 세션을 "오래된(stale)" 세션으로 간주합니다.
    4.  오래된 세션을 발견하면, 위 1번의 해결 방안과 동일하게 관련된 모든 플레이어를 대기열로 돌려보내고 로딩 세션 키를 삭제합니다.

---
이 계획에 따라 문제 해결을 진행하겠습니다.
