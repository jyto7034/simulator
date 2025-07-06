use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use crate::AppState;

// JWT Claims 구조체 (auth_server의 것과 동일해야 함)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String, // Subject (user's steam_id)
    pub iat: usize,
    pub exp: usize,
}

// 핸들러에서 인증된 사용자 정보를 담을 구조체
#[derive(Debug)]
pub struct AuthenticatedUser {
    pub steam_id: i64,
}

// Actix-Web 추출기(Extractor) 구현
impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let app_state = req.app_data::<web::Data<AppState>>().unwrap();

        // 1. Authorization 헤더에서 토큰 추출
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header.to_str().unwrap_or(""),
            None => {
                return ready(Err(actix_web::error::ErrorUnauthorized(
                    "Missing Authorization header",
                )))
            }
        };

        if !auth_header.starts_with("Bearer ") {
            return ready(Err(actix_web::error::ErrorUnauthorized(
                "Invalid token format",
            )));
        }

        let token = &auth_header["Bearer ".len()..];

        // 2. JWT 디코딩 및 검증
        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(app_state.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(data) => data,
            Err(e) => {
                // 로그에 에러 기록
                tracing::warn!("JWT validation failed: {}", e);
                return ready(Err(actix_web::error::ErrorUnauthorized("Invalid token")));
            }
        };

        // 3. Claims에서 steam_id 파싱
        let steam_id = match token_data.claims.sub.parse::<i64>() {
            Ok(id) => id,
            Err(_) => {
                return ready(Err(actix_web::error::ErrorBadRequest(
                    "Invalid steam_id in token",
                )))
            }
        };

        // 4. 성공 시 AuthenticatedUser 반환
        ready(Ok(AuthenticatedUser { steam_id }))
    }
}
