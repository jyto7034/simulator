// src/auth/error.rs

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

// 애플리케이션의 모든 에러를 통합 관리하는 Enum
#[derive(Debug)]
pub enum AuthError {
    BadRequest(String),
    Unauthorized(String),
    GatewayTimeout(String),
    InternalServerError(anyhow::Error),
}

// 에러 메시지를 예쁘게 출력하기 위한 Display 구현
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::BadRequest(reason) => write!(f, "Bad Request: {}", reason),
            AuthError::Unauthorized(reason) => write!(f, "Unauthorized: {}", reason),
            AuthError::GatewayTimeout(reason) => write!(f, "Gateway Timeout: {}", reason),
            AuthError::InternalServerError(e) => write!(f, "Internal Server Error: {:?}", e),
        }
    }
}

// Actix-web이 에러를 HTTP 응답으로 변환할 수 있도록 ResponseError 구현
impl ResponseError for AuthError {
    // 각 에러 타입에 맞는 HTTP 상태 코드를 반환합니다.
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AuthError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AuthError::GatewayTimeout(_) => StatusCode::GATEWAY_TIMEOUT,
            AuthError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // 실제 HTTP 응답 본문을 생성합니다.
    fn error_response(&self) -> HttpResponse {
        // 모든 에러를 서버 로그에 기록합니다 (중요!)
        tracing::error!("{}", self);

        let status = self.status_code();
        let message = match self {
            // 5xx 서버 에러는 클라이언트에게 상세 내용을 노출하지 않는 것이 보안상 좋습니다.
            AuthError::InternalServerError(_) => "An internal server error occurred.".to_string(),
            // 4xx 클라이언트 에러는 원인을 알려주는 것이 좋습니다.
            _ => self.to_string(),
        };

        HttpResponse::build(status).json(json!({ "error": message }))
    }
}

// 다른 라이브러리의 에러를 우리 AuthError로 쉽게 변환하기 위한 `From` 구현
impl From<actix::MailboxError> for AuthError {
    fn from(e: actix::MailboxError) -> Self {
        AuthError::InternalServerError(anyhow::anyhow!(e))
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(e: anyhow::Error) -> Self {
        AuthError::InternalServerError(e)
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        AuthError::InternalServerError(anyhow::anyhow!(e))
    }
}
