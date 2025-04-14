use std::{future::Future, pin::Pin};

use actix::{Actor, AsyncContext, Context};
use actix_web::{get, web, FromRequest, HttpRequest, HttpResponse};
use actix_ws::handle;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    card::types::PlayerType,
    exception::GameError,
    server::{types::ServerState, ws_actor::heartbeat::HeartbeatActor},
};

#[derive(Debug, Clone, Copy)]
pub struct AuthPlayer {
    ptype: PlayerType,
    session_id: Uuid,
}

impl AuthPlayer {
    fn new(ptype: PlayerType, session_id: Uuid) -> Self {
        Self { ptype, session_id }
    }
}

impl AuthPlayer {
    fn reverse(&self) -> PlayerType {
        match self.ptype {
            PlayerType::Player1 => PlayerType::Player2,
            PlayerType::Player2 => PlayerType::Player1,
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

            let Some(player_name) = req.cookie("user_id") else {
                error!("쿠키 누락: 'user_id' 쿠키를 찾을 수 없음");
                return Err(GameError::CookieNotFound);
            };
            let Some(game_step) = req.cookie("game_step") else {
                error!("쿠키 누락: 'game_step' 쿠키를 찾을 수 없음");
                return Err(GameError::CookieNotFound);
            };

            let player_name = player_name.to_string().replace("user_id=", "");
            let game_step = game_step.to_string().replace("game_step=", "");
            debug!(
                "쿠키 파싱 완료: player_name={}, game_step={}",
                player_name, game_step
            );

            if let Some(state) = req.app_data::<web::Data<ServerState>>() {
                let player_name_str = player_name.to_string();
                let p1_key = state.player_cookie.get();
                let p2_key = state.opponent_cookie.get();

                let player_type = match player_name_str.as_str() {
                    key if key == p1_key => {
                        debug!("플레이어1로 인증됨");
                        PlayerType::Player1
                    }
                    key if key == p2_key => {
                        debug!("플레이어2로 인증됨");
                        PlayerType::Player2
                    }
                    _ => {
                        error!("잘못된 플레이어 키: {}", player_name_str);
                        return Err(GameError::InternalServerError);
                    }
                };

                debug!(
                    "세션 등록 시작: player_type={:?}, game_step={}",
                    player_type, game_step
                );
                let session_id = state
                    .session_manager
                    .register_session(player_type, game_step.clone().into())
                    .await?;
                debug!("세션 등록 완료: session_id={}", session_id);

                info!(
                    "인증 성공: player_type={:?}, session_id={}",
                    player_type, session_id
                );
                Ok(AuthPlayer::new(player_type, session_id))
            } else {
                error!("서버 상태 객체를 찾을 수 없음");
                Err(GameError::ServerStateNotFound)
            }
        })
    }
}

impl From<AuthPlayer> for PlayerType {
    fn from(value: AuthPlayer) -> Self {
        value.ptype
    }
}

impl From<AuthPlayer> for String {
    fn from(value: AuthPlayer) -> Self {
        value.ptype.into()
    }
}

/// Game 의 전반적인 기능을 책임지는 end point
#[get("/game")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype, session_id = ?player.session_id))]
pub async fn game(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    todo!()
}
/*
각 Phase 에 해당하는 end point 를 만들어서 게임 진행에 따라 접속해야하는 end point 가 달라져.
예를 들어 처음에는 mulligan 이니까 클라이언트는 mulligan  엔드포인트에 접속해서 ws 객체를 얻은 후 mulligan 단계를 진행 해,
진행이 끝나면 기존 ws 객체는 close 되고. 그 다음 단계인 DrawPhase 엔드포인트에 접속해서 또 다시 ws 객체를 얻어. 단계 진행 후 close. 이 방법이 다른 phase 에서도 똑같이 반복돼.
이 구조에 문제가 있을까?

*/
#[get("/heartbeat")]
#[instrument(skip(state, req, payload), fields(player_type = ?player.ptype, session_id = ?player.session_id))]
pub async fn heartbeat(
    player: AuthPlayer,
    state: web::Data<ServerState>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, GameError> {
    let player_type = player.ptype;
    let session_id = player.session_id;
    let session_manager = state.session_manager.clone();

    info!(
        "Attempting to upgrade heartbeat connection for player {:?} (session_id: {})",
        player_type, session_id
    );

    // WebSocket 핸드셰이크 및 스트림/세션 분리
    match handle(&req, payload) {
        Ok((response, session, stream)) => {
            info!("WebSocket handshake successful for player {:?}. Starting actor and adding stream...", player_type);

            // --- Actor::create를 사용하여 액터 생성 및 스트림 추가 ---
            let _actor_addr = HeartbeatActor::create(|ctx: &mut Context<HeartbeatActor>| {
                // 액터 인스턴스 생성
                let actor = HeartbeatActor::new(
                    player_type,
                    session_id,
                    session_manager, // 클론된 session_manager 전달
                    session,         // WebSocket 세션 전달
                );

                ctx.add_stream(stream);
                info!(
                    "WebSocket stream added to HeartbeatActor context for player {:?}.",
                    player_type
                );

                actor
            });
            // Actor::create는 내부적으로 actor.start()와 유사하게 액터를 시작시킵니다.
            // 반환값은 Addr<HeartbeatActor> 이지만 여기서는 사용하지 않으므로 _actor_addr로 받습니다.

            info!(
                "HeartbeatActor created and started with stream for player {:?} (session_id: {})",
                player_type, session_id
            );

            Ok(response)
        }
        Err(e) => {
            error!(
                "WebSocket handling/upgrade failed for player {:?}: {:?}",
                player_type, e
            );
            Err(GameError::HandleFailed)
        }
    }
}
