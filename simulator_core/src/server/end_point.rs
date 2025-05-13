use std::{future::Future, pin::Pin};

use actix::{Actor, AsyncContext, Context};
use actix_web::{get, web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::handle;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerKind,
    exception::GameError,
    server::{actor::connection::ConnectionActor, types::ServerState},
};

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

impl AuthPlayer {
    fn reverse(&self) -> PlayerKind {
        match self.ptype {
            PlayerKind::Player1 => PlayerKind::Player2,
            PlayerKind::Player2 => PlayerKind::Player1,
        }
    }
}

impl FromRequest for AuthPlayer {
    type Error = GameError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            debug!("AuthPlayer::from_request 시작: 인증 처리 중...");

            let Some(player_id_cookie) = req.cookie("user_id") else {
                error!("쿠키 누락: 'user_id' 쿠키를 찾을 수 없음");
                return Err(GameError::CookieNotFound);
            };

            let player_id_string = player_id_cookie.to_string().replace("user_id=", "");
            debug!("쿠키 파싱 완료: player_name={}", player_id_string);

            if let Some(state) = req.app_data::<web::Data<ServerState>>() {
                let player_id = match Uuid::parse_str(&player_id_string) {
                    Ok(id) => id,
                    Err(e) => {
                        warn!(
                            "Failed to parse player_id from cookie: '{}'. Error: {}",
                            player_id_string, e
                        );
                        return Err(GameError::InvalidPayload);
                    }
                };

                // 서버 상태에서 플레이어 ID 가져오기 (state: &web::Data<ServerState>)
                let p1_key = state.player1_id;
                let p2_key = state.player2_id;

                // if-else if-else 로 PlayerType 결정
                let player_type = if player_id == p1_key {
                    debug!("Player authenticated as Player1 (ID: {})", player_id);
                    PlayerKind::Player1
                } else if player_id == p2_key {
                    debug!("Player authenticated as Player2 (ID: {})", player_id);
                    PlayerKind::Player2
                } else {
                    // 알 수 없는 ID 처리 (명확한 오류 반환)
                    error!(
                        "Authentication failed: Unknown player ID '{}' from cookie. Expected {} or {}.",
                        player_id, p1_key, p2_key
                    );
                    // 인증 실패 또는 잘못된 플레이어 오류 반환
                    return Err(GameError::InvalidPlayer);
                };

                debug!("Request Guard 통과: player_type={:?}", player_type);

                Ok(AuthPlayer::new(player_type, player_id))
            } else {
                error!("서버 상태 객체를 찾을 수 없음");
                Err(GameError::ServerStateNotFound)
            }
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

/// Game 의 전반적인 기능을 책임지는 end point
#[get("/game")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype))]
pub async fn game(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    info!("멀리건 단계 핸들러 시작: player={:?}", player.ptype);

    let player_type = player.ptype;
    let player_id = player.id;
    debug!("플레이어 타입 설정: {:?}", player_type);

    // Http 업그레이드: 이때 session과 stream이 반환됩니다.
    debug!("WebSocket 연결 업그레이드 시작");
    let (response, session, message_stream) = match handle(&req, payload) {
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

    ConnectionActor::create(|ctx: &mut Context<ConnectionActor>| {
        let new_actor = ConnectionActor::new(session, state.game.clone(), player_id, player_type);
        ctx.add_stream(message_stream);
        new_actor
    });

    Ok(response)
}
