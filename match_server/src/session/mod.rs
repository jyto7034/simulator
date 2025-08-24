pub mod ws_session;

#[derive(Clone, Copy, Debug, PartialEq)]
enum SessionState {
    Idle,          // 초기 상태. 아무런 행동을 지니지 않음.
    Enqueuing,     // Enqueue 요청을 받고 대기열에 등록 중.
    InQueue,       // 큐에 성공적으로 등록됨.
    InLoading,     // TryMatch 이후 StartLoading 메시지를 받고 에셋 로딩 중.
    Completed,     // 로딩 성공 후 정상적으로 매칭이 성사됨.
    Disconnecting, // 정상적으로 종료됨.
    Error,         // 오류 발생으로 인한 세션 종료 상태.
}
