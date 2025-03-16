use crate::card::types::PlayerType;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GameError {
    InvalidTargetCount,
    NoValidTargets,
    CannotActivate,
    DeckCodeIsMissing(PlayerType),
    PlayerInitializeFailed,
    PlayerDataNotIntegrity,
    PathNotExist,
    CardsNotFound,
    GameInitializeFailed,
    DifferentCardTypes,
    GenerateUUIDFaild,
    CardNotFound,
    ExceededCardLimit,
    FailedToDrawCard,
    NothingToRemove,
    InvalidCardData,
    NotAuthenticated,
    InvalidCardType,
    InvalidPlayerType,
    InvalidOperation,
    JsonParseFailed,
    DecodeError,
    DeckParseError,
    ReadFileFailed,
    NoCardsLeft,
    CardError,
    Ok,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerInitializeFailed => write!(f, "PlayerInitializeFailed"),
            Self::PlayerDataNotIntegrity => write!(f, "PlayerDataNotIntegrity"),
            Self::GenerateUUIDFaild => write!(f, "GenerateUUIDFaild"),
            Self::JsonParseFailed => write!(f, "Json Parse Failed"),
            Self::DeckParseError => write!(f, "Deck Parse Error"),
            Self::PathNotExist => write!(f, "Path Not Exist"),
            Self::CardError => write!(f, "Card Error"),
            Self::Ok => write!(f, "Ok"),
            _ => write!(f, ""),
        }
    }
}

/// 사용자 정의 에러 타입
#[derive(Debug)]
pub enum ServerError {
    Unknown,
    NotFound,
    WrongPhase(String, String),
    AlreadyReady,
    HandleFailed,
    NotAllowedReEntry,
    InternalServerError,
    UnexpectedMessage,
    CookieNotFound,
    ServerStateNotFound,
    ActiveSessionExists(String),
    ParseError(String),
    InvalidPayload,
    InvalidApproach,
    InvalidCards,
    InvalidPlayer,
    InvalidOperation,
}

impl From<GameError> for ServerError {
    fn from(value: GameError) -> Self {
        match value {
            GameError::InvalidTargetCount => ServerError::InvalidCards,
            GameError::NoValidTargets => ServerError::InvalidCards,
            GameError::CannotActivate => ServerError::InvalidOperation,
            GameError::DeckCodeIsMissing(_) => ServerError::InvalidPayload,
            GameError::PlayerInitializeFailed => ServerError::InternalServerError,
            GameError::PlayerDataNotIntegrity => ServerError::InternalServerError,
            GameError::PathNotExist => ServerError::NotFound,
            GameError::CardsNotFound => ServerError::NotFound,
            GameError::GameInitializeFailed => ServerError::InternalServerError,
            GameError::DifferentCardTypes => ServerError::InvalidCards,
            GameError::GenerateUUIDFaild => ServerError::InternalServerError,
            GameError::CardNotFound => ServerError::NotFound,
            GameError::ExceededCardLimit => ServerError::InvalidOperation,
            GameError::FailedToDrawCard => ServerError::InvalidOperation,
            GameError::NothingToRemove => ServerError::InvalidOperation,
            GameError::InvalidCardData => ServerError::InvalidCards,
            GameError::NotAuthenticated => ServerError::InvalidPlayer,
            GameError::InvalidCardType => ServerError::InvalidCards,
            GameError::InvalidPlayerType => ServerError::InvalidPlayer,
            GameError::InvalidOperation => ServerError::InvalidOperation,
            GameError::JsonParseFailed => ServerError::ParseError("Json Parse Failed".to_string()),
            GameError::DecodeError => ServerError::ParseError("Decode Error".to_string()),
            GameError::DeckParseError => ServerError::ParseError("Deck Parse Error".to_string()),
            GameError::ReadFileFailed => ServerError::InternalServerError,
            GameError::NoCardsLeft => ServerError::InvalidOperation,
            GameError::CardError => ServerError::InvalidCards,
            GameError::Ok => ServerError::Unknown,
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::WrongPhase(_, _) => write!(f, "WRONG_PHASE"),
            Self::NotFound => write!(f, "NOT_FOUND"),
            Self::HandleFailed => write!(f, "HANDLE_FAILED"),
            Self::InternalServerError => write!(f, "INTERNAL_SERVER_ERROR"),
            Self::CookieNotFound => write!(f, "COOKIE_NOT_FOUND"),
            Self::ServerStateNotFound => write!(f, "SERVER_STATE_NOT_FOUND"),
            Self::InvalidPayload => write!(f, "INVALID_PAYLOAD"),
            Self::ActiveSessionExists(_) => write!(f, "ACTIVE_SESSION_EXISTS"),
            Self::UnexpectedMessage => write!(f, "UNEXPECTED_MESSAGE"),
            Self::InvalidApproach => write!(f, "INVALID_APPROACH"),
            Self::InvalidCards => write!(f, "INVALID_CARDS"),
            Self::ParseError(_) => write!(f, "PARSE_ERROR"),
            Self::InvalidPlayer => write!(f, "INVALID_PLAYER"),
            Self::NotAllowedReEntry => write!(f, "NOT_ALLOWED_RE_ENTRY"),
            Self::AlreadyReady => write!(f, "ALREADY_READY"),
            Self::InvalidOperation => write!(f, "INVALID_OPERATION"),
        }
    }
}

/// MyCustomError 에 대해 ResponseError 트레이트 구현으로 HTTP 응답을 커스터마이징
impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Unknown => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("An unknown error occurred"),
            Self::WrongPhase(value, _value) => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Wrong Phase! expected: {}, Got: {}", value, _value))
            }
            Self::NotFound => HttpResponse::build(StatusCode::NOT_FOUND).body("Not Found"),
            Self::HandleFailed => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("An unknown error occurred"),
            Self::InternalServerError => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("Internal Server Error")
            }
            Self::CookieNotFound => {
                HttpResponse::build(StatusCode::NOT_FOUND).body("Cookie Not Found")
            }
            Self::ServerStateNotFound => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Server State Not Found"),
            Self::InvalidPayload => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Invalid Payload")
            }
            Self::ActiveSessionExists(msg) => HttpResponse::build(StatusCode::CONFLICT)
                .body(format!("Active session exists: {}", msg)),
            Self::ParseError(msg) => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(format!("Parse error: {}", msg))
            }
            Self::UnexpectedMessage => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Unexpected message")
            }
            Self::InvalidCards => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Invalid cards")
            }
            Self::InvalidPlayer => {
                HttpResponse::build(StatusCode::UNAUTHORIZED).body("Invalid player")
            }
            Self::InvalidApproach => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Invalid approach")
            }
            Self::NotAllowedReEntry => {
                HttpResponse::build(StatusCode::CONFLICT).body("Not allowed re-entry")
            }
            Self::AlreadyReady => HttpResponse::build(StatusCode::CONFLICT).body("Already ready"),
            Self::InvalidOperation => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body("Invalid operation")
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WrongPhase(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::HandleFailed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CookieNotFound => StatusCode::NOT_FOUND,
            Self::ServerStateNotFound => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidPayload => StatusCode::BAD_REQUEST,
            Self::ActiveSessionExists(_) => StatusCode::CONFLICT,
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedMessage => StatusCode::BAD_REQUEST,
            Self::InvalidCards => StatusCode::BAD_REQUEST,
            Self::InvalidPlayer => StatusCode::UNAUTHORIZED,
            Self::InvalidApproach => StatusCode::BAD_REQUEST,
            Self::NotAllowedReEntry => StatusCode::CONFLICT,
            Self::AlreadyReady => StatusCode::CONFLICT,
            Self::InvalidOperation => StatusCode::BAD_REQUEST,
        }
    }
}

pub enum MessageProcessResult<T> {
    Success(T),                    // 성공적으로 메시지 처리
    NeedRetry,                     // 에러가 발생했지만 재시도 가능
    TerminateSession(ServerError), // 세션 종료 필요
}
