use crate::{
    matchmaking::session::Ctx,
    shared::protocol::{ErrorCode, ServerMessage},
};

pub enum TransitionViolation {
    Minor,
    Major,
    Critical,
}

/// WebSocket을 통해 에러 메시지를 전송합니다.
///
/// # Arguments
/// * `ctx` - WebSocket context
/// * `code` - 에러 코드
/// * `message` - 에러 메시지
///
/// # Note
/// - 메트릭 자동 증가 (`MATCHMAKING_ERRORS_TOTAL`)
/// - JSON 직렬화 실패 시 조용히 무시
pub fn send_err(ctx: &mut Ctx, code: ErrorCode, message: &str) {
    // Metrics: 에러 메시지 카운트
    metrics::MATCHMAKING_ERRORS_TOTAL.inc();

    if let Ok(text) = serde_json::to_string(&ServerMessage::Error {
        code,
        message: message.to_string(),
    }) {
        ctx.text(text);
    }
}

pub fn classify_violation(from: SessionState, to: SessionState) -> TransitionViolation {
    use SessionState::*;
    match (from, to) {
        // 클라이언트 타이밍 이슈 (경미함)
        (InQueue, Enqueuing) => TransitionViolation::Minor,
        (InQueue, Dequeuing) if from == to => TransitionViolation::Minor,

        // 논리적 모순 (심각함)
        (Completed, Enqueuing) => TransitionViolation::Major,
        (Completed, Dequeuing) => TransitionViolation::Major,

        // 명백한 프로토콜 위반 (치명적)
        (Error, InQueue) => TransitionViolation::Critical,
        (Error, Enqueuing) => TransitionViolation::Critical,

        _ => TransitionViolation::Major,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionState {
    Idle,          // 초기 상태. 세션 생성 직후.
    Enqueuing,     // Enqueue 요청을 받고 대기열에 등록 중.
    InQueue,       // 큐에 성공적으로 등록됨.
    Dequeuing,     // Dequeue 요청을 받고 대기열에서 제거 중.
    Dequeued,      // 큐에서 성공적으로 제거됨.
    Completed,     // 정상적으로 매칭이 성사됨.
    Disconnecting, // 정상적으로 종료됨.
    Error,         // 오류 발생으로 인한 세션 종료 상태.
}

impl SessionState {
    pub fn can_transition_to(&self, new_state: SessionState) -> bool {
        use SessionState::*;
        match (self, new_state) {
            // From Idle
            (Idle, Enqueuing) => true,
            (Idle, Error) => true,

            // From Enqueuing
            (Enqueuing, InQueue) => true,
            (Enqueuing, Error) => true,
            (Enqueuing, Disconnecting) => true,

            // From InQueue
            (InQueue, Dequeuing) => true,
            (InQueue, Completed) => true,
            (InQueue, Error) => true,
            (InQueue, Disconnecting) => true,

            // From Dequeuing
            (Dequeuing, Dequeued) => true,
            (Dequeuing, Error) => true,
            (Dequeuing, Disconnecting) => true,

            // From Dequeued
            (Dequeued, Enqueuing) => true,
            (Dequeued, Error) => true,
            (Dequeued, Disconnecting) => true,

            // From Completed
            (Completed, Disconnecting) => true,
            (Completed, Error) => true,

            // Disconnecting, Error 에서 State 전환 불가.
            _ => false,
        }
    }

    /// Get human-readable description of the state
    pub fn description(&self) -> &'static str {
        match self {
            SessionState::Idle => "Waiting for player input",
            SessionState::Enqueuing => "Processing enqueue request",
            SessionState::InQueue => "Waiting for match",
            SessionState::Dequeuing => "Processing dequeue request",
            SessionState::Dequeued => "Dequeued from queue",
            SessionState::Completed => "Match completed successfully",
            SessionState::Disconnecting => "Cleaning up connection",
            SessionState::Error => "Error occurred, cleaning up",
        }
    }
}
