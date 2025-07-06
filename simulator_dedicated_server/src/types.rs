use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// match_server로부터 세션 생성을 요청받을 때 사용하는 구조체입니다.
#[derive(Deserialize, Debug)]
pub struct CreateSessionRequest {
    pub players: Vec<Uuid>,
}

/// match_server에게 세션 생성 결과를 응답할 때 사용하는 구조체입니다.
#[derive(Serialize, Debug)]
pub struct CreateSessionResponse {
    pub server_address: String,
    pub session_id: Uuid,
}

/// Actix-web AppState로 사용될 서버의 전역 상태 구조체입니다.
pub struct ServerState {
    pub redis_conn: redis::aio::ConnectionManager,
    pub server_id: String,
}
