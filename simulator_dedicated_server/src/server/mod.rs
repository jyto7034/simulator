use std::{future::Future, pin::Pin};
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, get};
use actix_ws::handle;
use serde::Deserialize;
use simulator_core::{
    card::types::PlayerKind,
    exception::{ConnectionError, GameError},
};
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

// test 모듈 의존성 제거
// use crate::{connection::connection::ConnectionActor, test::ServerState}; 
use crate::connection::connection::ConnectionActor;


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
                    error!("쿼리 파라미터 파싱 실패: 'session_id'를 찾을 수 없거나 형식이 잘못됨. {}", e);
                    return Err(GameError::Connection(ConnectionError::InvalidPayload(
                        "Missing or invalid 'session_id' query parameter".to_string(),
                    )));
                }
            };
            
            let session_id = query.session_id;
            info!("세션 ID 확인: {}", session_id);

            // TODO: Redis 또는 내부 상태에서 이 session_id가 유효한지 검증하는 로직 필요
            let player_id = Uuid::new_v4(); 
            let player_type = PlayerKind::Player1;

            debug!("Request Guard 통과: player_type={:?}, player_id={}", player_type, player_id);
            Ok(AuthPlayer::new(player_type, player_id))
        })
    }
}

impl From<AuthPlayer> for PlayerKind {
    fn from(value: AuthPlayer) -> Self {
        value.ptype
    }
}

impl From<AuthPlayer> for String {
    fn from(value: AuthPlayer) -> Self {
        value.ptype.to_string()
    }
}

#[get("/create_room")]
pub async fn create_room(
    _player: AuthPlayer,
    _req: HttpRequest,
    _payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    todo!()
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

    // TODO: main.rs에서 GameActor를 생성하고 ServerState를 통해 전달받아 ConnectionActor와 연결해야 함
    todo!("GameActor를 생성하고 ConnectionActor와 연결해야 합니다.");
}