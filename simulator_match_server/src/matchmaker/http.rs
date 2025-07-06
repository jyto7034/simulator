use actix_web::{post, web, HttpResponse, Responder};
use tracing::{info, warn};

use crate::{
    auth::AuthenticatedUser,
    matchmaker::actor::{EnqueuePlayer, EnqueueResult}, // EnqueueResult 임포트
    AppState,
};

#[derive(serde::Deserialize)]
pub struct MatchmakingRequest {
    pub game_mode: String,
}

/// POST /matchmaking/queue
/// 인증된 사용자를 매칭 대기열에 추가하라는 요청을 Matchmaker 액터에게 보냅니다.
#[post("/queue")]
pub async fn enqueue_player(
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    req_body: web::Json<MatchmakingRequest>,
) -> impl Responder {
    info!(
        "Player {} requests matchmaking for game mode: {}",
        user.steam_id, req_body.game_mode
    );

    // Matchmaker 액터에게 메시지를 보내고 응답을 기다립니다.
    match state
        .matchmaker_addr
        .send(EnqueuePlayer {
            steam_id: user.steam_id,
            game_mode: req_body.game_mode.clone(),
        })
        .await
    {
        // 액터로부터 응답을 성공적으로 받은 경우
        Ok(result) => match result {
            EnqueueResult::Success => HttpResponse::Ok().json(serde_json::json!({
                "message": "Successfully joined the matchmaking queue.",
                "steam_id": user.steam_id,
                "status": "pending"
            })),
            EnqueueResult::AlreadyInQueue => HttpResponse::Conflict().json(serde_json::json!({
                "message": "You are already in the matchmaking queue.",
                "steam_id": user.steam_id,
                "status": "already_in_queue"
            })),
            EnqueueResult::InternalError => {
                HttpResponse::InternalServerError().json("Failed to add player to queue")
            }
        },
        // 액터에게 메시지를 보내는 데 실패한 경우 (예: 액터가 다운됨)
        Err(e) => {
            warn!("Actor send error: {}", e);
            HttpResponse::InternalServerError().json("Internal server error")
        }
    }
}
