// In: simulator_core/src/exception/mod.rs

use crate::card::types::PlayerKind;
use actix::MailboxError;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;
use uuid::Uuid;

// ===================================================================
// 1. 세분화된 에러 타입을 정의합니다.
// ===================================================================

/// 시스템 레벨의 에러 (네트워크, I/O, 내부 로직 등)
#[derive(Debug)]
pub enum SystemError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Mailbox(MailboxError),
    LockFailed(String),
    TaskFailed(String),
    Internal(String), // 예기치 못한 내부 로직 에러
}

/// 게임 상태와 관련된 에러
#[derive(Debug, PartialEq, Clone)]
pub enum StateError {
    InvalidPhaseTransition,
    InvalidActionForPhase {
        current_phase: String,
        action: String,
    },
    GameAlreadyOver,
    GameAborted,
    PlayerNotReady(PlayerKind),
}

/// 클라이언트 연결 및 인증과 관련된 에러
#[derive(Debug, PartialEq, Clone)]
pub enum ConnectionError {
    AuthenticationFailed(String),
    SessionExists(Uuid), // 이미 세션이 존재하는 플레이어 ID
    InvalidPayload(String),
}

/// 게임 플레이 규칙과 관련된 에러
#[derive(Debug, PartialEq, Clone)]
pub enum GameplayError {
    ResourceNotFound {
        kind: &'static str,
        id: String,
    },
    DeckError(DeckError),
    InvalidTarget {
        reason: String,
    },
    InvalidAction {
        reason: String,
    },
    ChainError {
        reason: String,
    },
    NotEnoughResources {
        resource: &'static str,
        needed: i32,
        available: i32,
    },
}

/// 덱 구성 및 처리 관련 에러
#[derive(Debug, PartialEq, Clone)]
pub enum DeckError {
    ParseFailed(String),
    CodeMissingFor(PlayerKind),
    ExceededCardLimit(String),
    NoCardsLeftToDraw,
}

// ===================================================================
// 2. 최상위 GameError Enum을 새롭게 정의합니다.
// ===================================================================

#[derive(Debug)]
pub enum GameError {
    System(SystemError),
    Connection(ConnectionError),
    State(StateError),
    Gameplay(GameplayError),
}

// ===================================================================
// 3. Display 트레이트를 구현하여 명확한 로그 메시지를 생성합니다.
// ===================================================================

// 각 하위 에러 타입에 대한 Display 구현
impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::Io(e) => write!(f, "System I/O error: {}", e),
            SystemError::Json(e) => write!(f, "System JSON processing error: {}", e),
            SystemError::Mailbox(e) => write!(f, "Actor mailbox error: {}", e),
            SystemError::LockFailed(name) => write!(f, "Failed to acquire lock on '{}'", name),
            SystemError::TaskFailed(name) => write!(f, "Async task '{}' failed to complete", name),
            SystemError::Internal(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::InvalidPhaseTransition => {
                write!(f, "Invalid game phase transition attempted")
            }
            StateError::InvalidActionForPhase {
                current_phase,
                action,
            } => write!(
                f,
                "Action '{}' is not allowed during phase '{}'",
                action, current_phase
            ),
            StateError::GameAlreadyOver => {
                write!(f, "Action attempted but the game is already over")
            }
            StateError::GameAborted => write!(f, "Action attempted but the game was aborted"),
            StateError::PlayerNotReady(pk) => {
                write!(f, "Player {:?} is not ready for the action", pk)
            }
        }
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::AuthenticationFailed(reason) => {
                write!(f, "Authentication failed: {}", reason)
            }
            ConnectionError::SessionExists(player_id) => write!(
                f,
                "An active session already exists for player {}",
                player_id
            ),
            ConnectionError::InvalidPayload(reason) => {
                write!(f, "Received invalid payload from client: {}", reason)
            }
        }
    }
}

impl fmt::Display for GameplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameplayError::ResourceNotFound { kind, id } => {
                write!(f, "Resource not found: kind={}, id={}", kind, id)
            }
            GameplayError::DeckError(e) => write!(f, "Deck error: {}", e),
            GameplayError::InvalidTarget { reason } => write!(f, "Invalid target: {}", reason),
            GameplayError::InvalidAction { reason } => write!(f, "Invalid action: {}", reason),
            GameplayError::ChainError { reason } => write!(f, "Chain error: {}", reason),
            GameplayError::NotEnoughResources {
                resource,
                needed,
                available,
            } => write!(
                f,
                "Not enough {}: needed {}, available {}",
                resource, needed, available
            ),
        }
    }
}

impl fmt::Display for DeckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeckError::ParseFailed(s) => write!(f, "Failed to parse deck: {}", s),
            DeckError::CodeMissingFor(pk) => write!(f, "Deck code is missing for player {:?}", pk),
            DeckError::ExceededCardLimit(s) => write!(f, "Deck limit exceeded: {}", s),
            DeckError::NoCardsLeftToDraw => write!(f, "No cards left in the deck to draw"),
        }
    }
}

// 최상위 GameError에 대한 Display 구현
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::System(e) => e.fmt(f),
            GameError::Connection(e) => e.fmt(f),
            GameError::State(e) => e.fmt(f),
            GameError::Gameplay(e) => e.fmt(f),
        }
    }
}

// ===================================================================
// 4. ResponseError를 구현하여 HTTP 응답으로 변환합니다.
// ===================================================================

impl ResponseError for GameError {
    fn status_code(&self) -> StatusCode {
        match self {
            // 시스템 에러는 모두 500번대
            GameError::System(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // 상태 에러는 주로 동시성 문제나 순서 문제이므로 409 Conflict
            GameError::State(StateError::InvalidPhaseTransition) => StatusCode::CONFLICT,
            GameError::State(StateError::InvalidActionForPhase { .. }) => StatusCode::CONFLICT,
            GameError::State(_) => StatusCode::CONFLICT,
            // 연결 에러는 클라이언트의 잘못일 가능성이 높으므로 400번대
            GameError::Connection(ConnectionError::AuthenticationFailed(_)) => {
                StatusCode::UNAUTHORIZED
            }
            GameError::Connection(ConnectionError::SessionExists(_)) => StatusCode::CONFLICT,
            GameError::Connection(ConnectionError::InvalidPayload(_)) => StatusCode::BAD_REQUEST,
            // 게임플레이 에러는 규칙 위반이므로 400 Bad Request
            GameError::Gameplay(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_message = self.to_string(); // Display 구현을 사용

        // 프로덕션에서는 내부 에러를 클라이언트에 노출하지 않는 것이 좋습니다.
        let client_message = if status.is_server_error() {
            "An internal server error occurred.".to_string()
        } else {
            error_message.clone()
        };

        // 서버 로그에는 상세한 에러를 남깁니다.
        tracing::error!("Request failed: {}", error_message);

        HttpResponse::build(status).json(serde_json::json!({ "error": client_message }))
    }
}

// ===================================================================
// 5. From 트레이트를 구현하여 에러 변환을 쉽게 합니다.
// ===================================================================

impl From<MailboxError> for GameError {
    fn from(e: MailboxError) -> Self {
        GameError::System(SystemError::Mailbox(e))
    }
}

impl From<serde_json::Error> for GameError {
    fn from(e: serde_json::Error) -> Self {
        GameError::System(SystemError::Json(e))
    }
}

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> Self {
        GameError::System(SystemError::Io(e))
    }
}

// 기존에 정의된 GameError enum과 const 문자열들은 모두 삭제하시면 됩니다.
