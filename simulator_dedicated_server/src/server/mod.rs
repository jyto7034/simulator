use actix_web::{get, post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use actix_ws::handle;
use redis::AsyncCommands;
use serde::Deserialize;
use simulator_core::{
    card::types::PlayerKind,
    exception::{ConnectionError, GameError},
};
use simulator_metrics::ACTIVE_SESSIONS;
use std::{future::Future, pin::Pin};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::types::{CreateSessionRequest, CreateSessionResponse, ServerState};

// --- WebSocket Endpoint ---

#[derive(Deserialize)]
struct GameWsQuery {
    session_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthPlayer {
    ptype: PlayerKind,
    id: Uuid,
}

impl AuthPlayer {
    fn new(ptype: PlayerKind, id: Uuid) -> Self {
        Self { ptype, id }
    }
}

impl FromRequest for AuthPlayer {
    type Error = GameError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            debug!("AuthPlayer::from_request 시작: session_id 기반 인증 처리 중...");

            let query = match web::Query::<GameWsQuery>::from_query(req.query_string()) {
                Ok(q) => q,
                Err(e) => {
                    error!(
                        "쿼리 파라미터 파싱 실패: 'session_id'를 찾을 수 없거나 형식이 잘못됨. {}",
                        e
                    );
                    return Err(GameError::Connection(ConnectionError::InvalidPayload(
                        "Missing or invalid 'session_id' query parameter".to_string(),
                    )));
                }
            };

            let session_id = query.session_id;
            info!("세션 ID 확인: {}", session_id);

            let player_id = Uuid::new_v4();
            let player_type = PlayerKind::Player1;

            debug!(
                "Request Guard 통과: player_type={:?}, player_id={}",
                player_type, player_id
            );
            Ok(AuthPlayer::new(player_type, player_id))
        })
    }
}

#[get("/game")]
#[instrument(skip(req, payload), fields(player_type = ?player.ptype))]
pub async fn game(
    player: AuthPlayer,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    let player_type = player.ptype;
    let player_id = player.id;
    debug!("플레이어 타입 설정: {:?}", player_type);

    debug!("WebSocket 연결 업그레이드 시작");
    let (_response, _session, _message_stream) = match handle(&req, payload) {
        Ok(result) => {
            info!(
                "WebSocket handshake successful for player_id: {}",
                player_id
            );
            result
        }
        Err(e) => {
            error!(
                "WebSocket handshake failed for player_id: {}: {:?}",
                player_id, e
            );
            return Ok(
                HttpResponse::InternalServerError().body(format!("WS Handshake Error: {}", e))
            );
        }
    };

    todo!("GameActor를 생성하고 ConnectionActor와 연결해야 합니다.");
}

// --- HTTP Endpoint ---

#[post("/session/create")]
pub async fn create_session(
    req: web::Json<CreateSessionRequest>,
    state: web::Data<ServerState>,
) -> impl Responder {
    let session_id = Uuid::new_v4();
    tracing::info!(
        "Received request to create session for players: {:?}",
        req.players
    );

    let mut redis_conn = state.redis_conn.clone();
    let server_key = &state.server_id;

    let bind_address = "127.0.0.1:8088";
    let new_server_info = serde_json::to_string(&serde_json::json!({
        "address": bind_address,
        "status": "busy"
    }))
    .unwrap();

    match redis_conn
        .set::<_, _, ()>(server_key, new_server_info)
        .await
    {
        Ok(_) => tracing::info!("Server {} status updated to busy.", server_key),
        Err(e) => {
            tracing::error!(
                "Failed to update server {} status to busy: {}",
                server_key,
                e
            );
            return HttpResponse::InternalServerError().body("Failed to update server status.");
        }
    }

    ACTIVE_SESSIONS.inc();

    let response = CreateSessionResponse {
        server_address: format!("ws://{}/game?session_id={}", bind_address, session_id),
        session_id,
    };

    HttpResponse::Ok().json(response)
}
