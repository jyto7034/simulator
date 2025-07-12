# Match Server 잠재적 버그 분석 보고서

## 개요
Simulator Match Server 코드베이스를 분석하여 발견된 잠재적 버그들을 심각도별로 분류하고 해결 방안을 제시합니다.

---

## 🔴 심각한 버그 (Critical)

### 1. WebSocket 세션 중복 큐잉 방지 로직 오류
**파일**: `simulator_match_server/src/ws_session.rs:118-137`

**문제점**:
```rust
if self.player_id.is_some() {
    warn!("Player {} tried to enqueue more than once.", player_id);
    return;
}
```
- 이미 `player_id`가 설정된 세션에서 재큐 요청을 무조건 차단
- 연결이 끊어진 후 재연결 시 정상적인 큐 진입이 불가능
- 클라이언트가 네트워크 문제로 재연결할 때 서비스 이용 불가

**영향도**: 높음 - 사용자 경험 저해, 서비스 가용성 문제

**해결 방안**:
- Redis에서 실제 큐 상태를 확인 후 결정
- 세션 상태를 더 세밀하게 관리 (queued, loading, matched 등)

### 2. 매치메이킹 원자성 위반
**파일**: `simulator_match_server/src/matchmaker/handlers.rs:159-172`

**문제점**:
```rust
let script = Script::new(ATOMIC_MATCH_SCRIPT);
let player_ids: Vec<String> = match script
    .key(&queue_key)
    .arg(required_players)
    .invoke_async(&mut redis)
    .await
```
- Lua 스크립트 실행과 후속 처리 사이의 원자성 보장 부족
- 동시성 환경에서 같은 플레이어가 여러 매치에 배정될 위험
- 매칭 후 Redis 상태 업데이트 실패 시 일관성 문제

**영향도**: 높음 - 데이터 무결성 위반, 게임 세션 충돌

**해결 방안**:
- 전체 매칭 프로세스를 하나의 Lua 스크립트로 통합
- 분산 락(Distributed Lock) 도입
- 매칭 상태를 Redis에서 원자적으로 관리

---

## 🟡 중간 위험 버그 (High)

### 3. Redis 연결 실패 시 무한 재시도
**파일**: `simulator_match_server/src/pubsub.rs:114-158`

**문제점**:
```rust
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

impl Handler<Connect> for RedisSubscriber {
    fn handle(&mut self, _msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        ctx.run_later(RECONNECT_DELAY, |act, ctx| {
            act.connect_and_subscribe(ctx);
        });
    }
}
```
- 고정된 5초 간격으로 무한 재시도
- 백오프(exponential backoff) 전략 없음
- Redis 장애 시 CPU 과부하 및 로그 스팸 발생

**영향도**: 중간 - 시스템 리소스 낭비, 운영 비용 증가

**해결 방안**:
- 지수 백오프 알고리즘 구현
- 최대 재시도 횟수 제한
- Circuit Breaker 패턴 도입

### 4. SubscriptionManager 메모리 누수
**파일**: `simulator_match_server/src/pubsub.rs:66-76`

**문제점**:
```rust
impl Handler<Register> for SubscriptionManager {
    fn handle(&mut self, msg: Register, _ctx: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.player_id, msg.addr);
    }
}
```
- 연결이 끊어진 세션의 자동 정리 메커니즘 부족
- `Deregister` 호출에만 의존하는 정리 로직
- 비정상 종료 시 HashMap에 좀비 세션 누적

**영향도**: 중간 - 장기 운영 시 메모리 사용량 증가

**해결 방안**:
- 주기적인 세션 생존 확인 메커니즘
- Weak reference 사용 검토
- TTL 기반 자동 정리

### 5. Loading Session 타임아웃 처리 불완전
**파일**: `simulator_match_server/src/matchmaker/handlers.rs:467-496`

**문제점**:
- 타임아웃된 세션의 플레이어들에게 알림 후 즉시 재큐잉
- 클라이언트가 아직 로딩 중일 가능성 무시
- 동일한 플레이어가 빠르게 재매칭될 수 있음

**영향도**: 중간 - 사용자 혼란, 중복 매칭 가능성

---

## 🟢 경미한 이슈 (Medium)

### 6. 에러 처리 불일치
**파일**: `simulator_match_server/src/matchmaker/handlers.rs` 전반

**문제점**:
- Redis 실패 시 일부는 로그만, 일부는 클라이언트 알림
- 에러 메시지 형식과 내용의 일관성 부족
- 에러 코드 체계 부재

**해결 방안**:
- 통일된 에러 처리 인터페이스 구현
- 에러 코드 enum 정의
- 클라이언트 알림 정책 표준화

### 7. 하드코딩된 설정값들
**파일**: `simulator_match_server/src/ws_session.rs:13-14`, `simulator_match_server/src/matchmaker/actor.rs:9`

**문제점**:
```rust
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const LOADING_SESSION_TIMEOUT_SECONDS: u64 = 60;
```
- 중요한 타이밍 값들이 하드코딩됨
- 환경별 설정 변경 불가
- 운영 중 조정 어려움

**해결 방안**:
- 설정 파일로 이동
- 환경 변수 지원
- 런타임 설정 변경 API 제공

### 8. 로그 레벨 부적절
**파일**: 전체 코드베이스

**문제점**:
- 정상 동작도 `info!` 레벨로 기록
- 디버깅 정보와 운영 정보 구분 부족
- 과도한 로그로 인한 성능 영향

**해결 방안**:
- 로그 레벨 재조정
- 구조화된 로깅 도입
- 성능 크리티컬 섹션에서 로그 최소화

---

## 🔥 즉시 수정 권장사항

### 우선순위 1: WebSocket 세션 중복 처리 로직
- 가장 사용자 경험에 직접적 영향
- 수정 복잡도 낮음
- 즉시 배포 가능

### 우선순위 2: 매치메이킹 원자성 보장
- 데이터 무결성 관련 심각한 이슈
- Lua 스크립트 수정으로 해결 가능
- 철저한 테스트 필요

### 우선순위 3: Redis 연결 안정성 개선
- 운영 안정성 향상
- 점진적 개선 가능
- 모니터링 강화 필요

---

## 테스트 시나리오

### 동시성 테스트
1. 동일 플레이어 다중 큐잉 시도
2. 매칭 중 연결 끊김
3. Redis 연결 불안정 상황

### 장애 복구 테스트
1. Redis 서버 재시작
2. 네트워크 단절 및 복구
3. 높은 부하 상황에서의 안정성

### 메모리 누수 테스트
1. 장시간 다중 사용자 접속
2. 비정상 클라이언트 종료 반복
3. 메모리 사용량 모니터링

---

## 결론

현재 Match Server는 기본적인 기능은 동작하지만, 프로덕션 환경에서 발생할 수 있는 여러 엣지 케이스와 동시성 문제에 대한 대비가 부족합니다. 특히 WebSocket 세션 관리와 Redis 원자성 보장 부분의 개선이 시급합니다.

**생성일**: 2025-07-10  
**분석 대상**: simulator_match_server  
**분석자**: Claude Code