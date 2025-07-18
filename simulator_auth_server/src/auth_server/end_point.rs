use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::info;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::auth_server::{
    db_operation,
    errors::AuthError,
    types::{AppState, SteamApiResponse, Claims},
};

// --- HTTP 요청 본문 구조체 ---
#[derive(Deserialize)]
struct SteamAuthRequest {
    ticket: String,
}

#[derive(Deserialize, Serialize)]
struct AuthSuccessResponse {
    message: String,
    steam_id: String,
    token: String,
}

#[derive(Serialize)]
struct GenericSuccessResponse {
    message: String,
}

#[derive(Deserialize)]
pub struct TestAuthRequest {
    pub steam_id: i64,
    pub username: Option<String>,
}

/// [TEST ONLY] /auth/test/login
/// 테스트 목적으로 Steam 인증 없이 가짜 플레이어에 대한 JWT를 발급합니다.
/// 이 엔드포인트는 `test-endpoints` 피처가 활성화된 경우에만 컴파일됩니다.
#[cfg(feature = "test-endpoints")]
#[actix_web::post("/test/login")]
pub async fn test_authentication_handler(
    state: web::Data<AppState>,
    req_body: web::Json<TestAuthRequest>,
) -> Result<HttpResponse, AuthError> {
    let steam_id_i64 = req_body.steam_id;
    let username = req_body.username.clone().unwrap_or(format!("test_user_{}", steam_id_i64));

    // 1. 실제 로그인과 동일한 DB 작업을 수행합니다.
    db_operation::upsert_player_on_login(&state.db_pool, steam_id_i64, &username).await?;
    tracing::info!("[TEST] Upserted test player: {}", steam_id_i64);

    // 2. JWT를 생성합니다.
    let now = Utc::now();
    let claims = Claims {
        sub: steam_id_i64.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(1)).timestamp() as usize, // 테스트용 토큰은 하루 동안 유효
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_ref()),
    )
    .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

    tracing::info!("[TEST] Generated JWT for test player: {}", steam_id_i64);

    // 3. 성공 응답에 토큰을 포함하여 반환합니다.
    Ok(HttpResponse::Ok().json(AuthSuccessResponse {
        message: "Test authentication successful.".to_string(),
        steam_id: steam_id_i64.to_string(),
        token,
    }))
}


// --- 엔드포인트 핸들러 ---
/// POST /auth/steam
/// 클라이언트로부터 스팀 티켓을 받아 인증을 처리합니다.
#[actix_web::post("/steam")]
pub async fn steam_authentication_handler(
    state: web::Data<AppState>,
    req_body: web::Json<SteamAuthRequest>,
) -> Result<HttpResponse, AuthError> {
    info!("Received Steam authentication request with ticket",);
    let api_url = "https://api.steampowered.com/ISteamUserAuth/AuthenticateUserTicket/v1/";

    // 1. 스팀 웹 API에 GET 요청을 보냅니다.
    let res = state
        .http_client
        .get(api_url)
        .query(&[
            ("key", &state.steam_web_api_key),
            ("appid", &state.app_id.to_string()),
            ("ticket", &req_body.ticket),
            ("identity", &state.expected_identity),
        ])
        .send()
        .await
        .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

    // 2. 응답 상태 코드 확인
    if !res.status().is_success() {
        return Err(AuthError::GatewayTimeout(format!(
            "Steam API returned non-success status: {}",
            res.status()
        )));
    }

    // 3. JSON 응답 파싱
    let steam_response = res
        .json::<SteamApiResponse>()
        .await
        .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

    // 4. 스팀 응답의 유효성 검사
    if let Some(params) = steam_response.response.params {
        if params.result == "OK" {
            // 성공!
            let steam_id_u64 = params.steamid.parse::<u64>().map_err(|_| {
                AuthError::InternalServerError(anyhow::anyhow!("Steam returned invalid SteamID"))
            })?;

            info!(
                "Steam Web API authentication successful for SteamID: {}",
                steam_id_u64
            );

            // 5. DB 작업 수행
            let temp_username = format!("user_{}", steam_id_u64);
            db_operation::upsert_player_on_login(
                &state.db_pool,
                steam_id_u64 as i64,
                &temp_username,
            )
            .await?;
            
            // JWT 생성
            let now = Utc::now();
            let claims = Claims {
                sub: steam_id_u64.to_string(),
                iat: now.timestamp() as usize,
                exp: (now + Duration::days(1)).timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_ref()),
            )
            .map_err(|e| AuthError::InternalServerError(anyhow::anyhow!(e)))?;

            Ok(HttpResponse::Ok().json(AuthSuccessResponse {
                message: "Steam Web API authentication successful.".to_string(),
                steam_id: steam_id_u64.to_string(),
                token,
            }))
        } else {
            // 결과가 "OK"가 아닌 경우
            Err(AuthError::Unauthorized(format!(
                "Steam validation failed with result: {}",
                params.result
            )))
        }
    } else if let Some(error) = steam_response.response.error {
        // 스팀 API가 에러를 반환한 경우
        Err(AuthError::Unauthorized(format!(
            "Steam API Error {}: {}",
            error.errorcode, error.errordesc
        )))
    } else {
        Err(AuthError::InternalServerError(anyhow::anyhow!(
            "Invalid response structure from Steam API"
        )))
    }
}

/// DELETE /test/player/{steam_id}
/// 테스트용으로 생성된 플���이어 계정과 관련 데이터를 삭제합니다.
#[actix_web::delete("/player/{steam_id}")]
pub async fn delete_player_handler(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse, AuthError> {
    let steam_id = path.into_inner();

    info!("Attempting to delete player data for SteamID: {}", steam_id);

    db_operation::delete_player_by_id(&state.db_pool, steam_id).await?;

    info!("Successfully deleted player data for SteamID: {}", steam_id);

    Ok(HttpResponse::Ok().json(GenericSuccessResponse {
        message: format!(
            "Player {} and all related data have been deleted.",
            steam_id
        ),
    }))
}
