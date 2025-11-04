use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, info, warn};

/// Circuit Breaker Pattern 구현
///
/// Redis 연속 실패 시 일정 시간동안 요청을 차단하여
/// 불필요한 재시도를 방지하고 시스템 부하를 줄입니다.
#[derive(Debug)]
pub struct CircuitBreaker {
    /// 연속 실패 횟수
    consecutive_failures: AtomicU64,
    /// Circuit을 열기 위한 임계값 (예: 5번 연속 실패)
    threshold: u64,
    /// Circuit이 열린 시각 (Unix timestamp)
    open_until: AtomicU64,
    /// Circuit이 열린 후 대기 시간 (초)
    cooldown_seconds: u64,
}

impl CircuitBreaker {
    /// 새 Circuit Breaker 생성
    ///
    /// # Arguments
    /// * `threshold` - Circuit을 열기 위한 연속 실패 횟수 (권장: 5)
    /// * `cooldown_seconds` - Circuit이 열린 후 대기 시간 (권장: 60초)
    pub fn new(threshold: u64, cooldown_seconds: u64) -> Self {
        Self {
            consecutive_failures: AtomicU64::new(0),
            threshold,
            open_until: AtomicU64::new(0),
            cooldown_seconds,
        }
    }

    /// Circuit이 열려있는지 확인
    ///
    /// # Returns
    /// * `Ok(())` - Circuit이 닫혀있음 (정상 작동 가능)
    /// * `Err(String)` - Circuit이 열려있음 (차단 중)
    pub fn check(&self) -> Result<(), String> {
        let now = Utc::now().timestamp() as u64;
        let open_until = self.open_until.load(Ordering::Relaxed);

        if open_until > now {
            let remaining = open_until - now;
            return Err(format!("Circuit open for {}s", remaining));
        }

        Ok(())
    }

    /// 성공 기록 - Circuit 닫기 및 카운터 리셋
    pub fn record_success(&self) {
        let previous = self.consecutive_failures.swap(0, Ordering::Relaxed);
        let was_open = self.open_until.swap(0, Ordering::Relaxed);

        if was_open > 0 {
            info!(
                "Circuit breaker CLOSED (recovered after {} failures)",
                previous
            );
        }
    }

    /// 실패 기록 - 카운터 증가, threshold 도달 시 Circuit 열기
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

        if failures >= self.threshold {
            let now = Utc::now().timestamp() as u64;
            let open_until = now + self.cooldown_seconds;
            self.open_until.store(open_until, Ordering::Relaxed);

            error!(
                "Circuit breaker OPEN! {} consecutive failures. \
                 Blocking operations for {}s",
                failures, self.cooldown_seconds
            );

            // Record circuit breaker open metric
            metrics::CIRCUIT_BREAKER_OPEN_TOTAL.inc();
        } else if failures % 2 == 0 {
            warn!(
                "Circuit breaker failure count: {}/{}",
                failures, self.threshold
            );
        }
    }

    /// 현재 연속 실패 횟수 조회 (디버깅/모니터링 용도)
    pub fn get_failure_count(&self) -> u64 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// Circuit이 현재 열려있는지 확인
    pub fn is_open(&self) -> bool {
        let now = Utc::now().timestamp() as u64;
        let open_until = self.open_until.load(Ordering::Relaxed);
        open_until > now
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, 2);

        // 3번 실패해야 열림
        cb.record_failure();
        assert!(!cb.is_open());

        cb.record_failure();
        assert!(!cb.is_open());

        cb.record_failure();
        assert!(cb.is_open()); // 3번째에 열림
    }

    #[test]
    fn test_circuit_breaker_closes_after_cooldown() {
        let cb = CircuitBreaker::new(2, 1); // 1초 cooldown

        cb.record_failure();
        cb.record_failure();
        assert!(cb.is_open());

        sleep(Duration::from_secs(2));
        assert!(!cb.is_open()); // cooldown 지나면 자동으로 닫힘
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new(3, 60);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_failure_count(), 2);

        cb.record_success();
        assert_eq!(cb.get_failure_count(), 0);
        assert!(!cb.is_open());
    }
}
