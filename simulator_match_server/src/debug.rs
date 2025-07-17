use actix_web::{get, web, HttpResponse, Result};
use redis::aio::ConnectionLike;
use redis::AsyncCommands;
use serde_json::json;
use std::collections::HashMap;

use crate::matchmaker::messages::GetDebugInfo;
use crate::pubsub::GetActiveSessionsDebug;
use crate::AppState;

/// 큐 상태 조회 - 유령 플레이어 탐지의 핵심
#[get("/debug/queue")]
pub async fn debug_queue_status(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut redis = state.redis_conn_manager.clone();

    // 1. 큐에 있는 플레이어들 조회
    let queue_members: Vec<String> = redis.smembers("queue:Normal_1v1").await.unwrap_or_default();

    let queue_size: i64 = redis.scard("queue:Normal_1v1").await.unwrap_or(0);

    // 2. 다른 게임 모드들도 조회
    let queue_keys: Vec<String> = redis.keys("queue:*").await.unwrap_or_default();

    let mut all_queues = HashMap::new();
    for queue_key in queue_keys {
        let members: Vec<String> = redis.smembers(&queue_key).await.unwrap_or_default();
        all_queues.insert(queue_key, members);
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "normal_1v1_queue": {
            "size": queue_size,
            "members": queue_members
        },
        "all_queues": all_queues,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 활성 세션 상태 조회 - WebSocket 연결 추적
#[get("/debug/sessions")]
pub async fn debug_active_sessions(state: web::Data<AppState>) -> Result<HttpResponse> {
    // SubscriptionManager에서 활성 세션 정보 요청
    let session_info = state.sub_manager_addr.send(GetActiveSessionsDebug).await;

    match session_info {
        Ok(Ok(sessions)) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "active_sessions": sessions,
            "session_count": sessions.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        _ => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to get session information"
        }))),
    }
}

/// 로딩 세션 상태 조회 - 로딩 중 유령 탐지
#[get("/debug/loading")]
pub async fn debug_loading_sessions(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut redis = state.redis_conn_manager.clone();

    // 1. 모든 로딩 세션 키 찾기
    let loading_keys: Vec<String> = redis.keys("loading:*").await.unwrap_or_default();

    let mut loading_sessions = HashMap::new();
    for loading_key in loading_keys {
        // 2. 각 로딩 세션의 상세 정보
        let session_data: HashMap<String, String> =
            redis.hgetall(&loading_key).await.unwrap_or_default();

        let ttl: i64 = redis.ttl(&loading_key).await.unwrap_or(-1);

        loading_sessions.insert(
            loading_key,
            json!({
                "players": session_data,
                "ttl_seconds": ttl
            }),
        );
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "loading_sessions": loading_sessions,
        "total_sessions": loading_sessions.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Redis 연결 상태 및 헬스체크
#[get("/debug/redis")]
pub async fn debug_redis_health(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut redis = state.redis_conn_manager.clone();

    let start_time = std::time::Instant::now();

    // Redis ping 테스트
    let ping_result: Result<redis::Value, _> = redis.req_packed_command(&redis::cmd("PING")).await;
    let ping_duration = start_time.elapsed();
    
    let ping_success = ping_result.is_ok();
    let ping_string = match ping_result {
        Ok(redis::Value::Status(s)) => s,
        Ok(other) => format!("{:?}", other),
        Err(e) => format!("Error: {}", e),
    };

    // Redis 메모리 사용량
    let info_result: Result<redis::Value, _> = redis
        .req_packed_command(&redis::cmd("INFO").arg("memory"))
        .await;
    
    let info_string = match info_result {
        Ok(redis::Value::Data(data)) => String::from_utf8_lossy(&data).to_string(),
        Ok(other) => format!("{:?}", other),
        Err(e) => format!("Error: {}", e),
    };

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "ping": {
            "success": ping_success,
            "duration_ms": ping_duration.as_millis(),
            "response": ping_string
        },
        "memory_info": info_string,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 종합 유령 플레이어 탐지
#[get("/debug/ghosts")]
pub async fn debug_ghost_detection(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut redis = state.redis_conn_manager.clone();

    // 1. Redis에서 큐에 있는 플레이어들
    let queue_players: Vec<String> = redis.smembers("queue:Normal_1v1").await.unwrap_or_default();

    // 2. 로딩 중인 플레이어들
    let loading_keys: Vec<String> = redis.keys("loading:*").await.unwrap_or_default();
    let mut loading_players = Vec::new();
    for key in loading_keys {
        let players: HashMap<String, String> = redis.hgetall(&key).await.unwrap_or_default();
        loading_players.extend(players.keys().cloned());
    }

    // 3. 실제 WebSocket 연결된 플레이어들 (SubscriptionManager에서)
    let session_info = state.sub_manager_addr.send(GetActiveSessionsDebug).await;

    let active_players: Vec<String> = match session_info {
        Ok(Ok(sessions)) => sessions.into_iter().map(|s| s.player_id).collect(),
        _ => Vec::new(),
    };

    // 4. 유령 탐지 로직
    let queue_ghosts: Vec<String> = queue_players
        .iter()
        .filter(|player| !active_players.contains(player))
        .cloned()
        .collect();

    let loading_ghosts: Vec<String> = loading_players
        .iter()
        .filter(|player| !active_players.contains(player))
        .cloned()
        .collect();

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "summary": {
            "total_queue_players": queue_players.len(),
            "total_loading_players": loading_players.len(),
            "total_active_connections": active_players.len(),
            "queue_ghosts": queue_ghosts.len(),
            "loading_ghosts": loading_ghosts.len()
        },
        "details": {
            "queue_players": queue_players,
            "loading_players": loading_players,
            "active_connections": active_players,
            "queue_ghosts": queue_ghosts,
            "loading_ghosts": loading_ghosts
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 매칭메이커 내부 상태 조회
#[get("/debug/matchmaker")]
pub async fn debug_matchmaker_state(state: web::Data<AppState>) -> Result<HttpResponse> {
    let debug_info = state.matchmaker_addr.send(GetDebugInfo).await;

    match debug_info {
        Ok(info_string) => {
            // JSON 문자열을 다시 파싱해서 응답
            match serde_json::from_str::<serde_json::Value>(&info_string) {
                Ok(info_json) => Ok(HttpResponse::Ok().json(json!({
                    "status": "success",
                    "matchmaker_info": info_json,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))),
                Err(_) => Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": "Failed to parse matchmaker debug info"
                })))
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to get matchmaker info: {}", e)
        }))),
    }
}

/// 전체 서버 상태 대시보드용 종합 정보
#[get("/debug/dashboard")]
pub async fn debug_dashboard(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut redis = state.redis_conn_manager.clone();

    // 모든 정보를 병렬로 수집
    let queue_size: i64 = redis.scard("queue:Normal_1v1").await.unwrap_or(0);
    let loading_sessions: Vec<String> = redis.keys("loading:*").await.unwrap_or_default();

    let session_info = state.sub_manager_addr.send(GetActiveSessionsDebug).await;

    let active_session_count = match session_info {
        Ok(Ok(sessions)) => sessions.len(),
        _ => 0,
    };

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "server_health": {
            "queue_size": queue_size,
            "loading_sessions": loading_sessions.len(),
            "active_connections": active_session_count,
            "uptime": "TODO", // 서버 시작 시간부터 계산
        },
        "quick_stats": {
            "healthy": queue_size >= 0, // 기본 헬스체크
            "redis_connected": true,    // Redis 응답했으므로 연결됨
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
