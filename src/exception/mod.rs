use crate::card::types::PlayerType;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
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
    InvalidCards,
    JsonParseFailed,
    InvalidPlayer,
    DecodeError,
    DeckParseError,
    ReadFileFailed,
    NoCardsLeft,
    CardError,
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
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerInitializeFailed => write!(f, "PLAYER_INITIALIZE_FAILED"),
            Self::PlayerDataNotIntegrity => write!(f, "PLAYER_DATA_NOT_INTEGRITY"),
            Self::GenerateUUIDFaild => write!(f, "GENERATE_UUID_FAILED"),
            Self::JsonParseFailed => write!(f, "JSON_PARSE_FAILED"),
            Self::DeckParseError => write!(f, "DECK_PARSE_ERROR"),
            Self::PathNotExist => write!(f, "PATH_NOT_EXIST"),
            Self::CardError => write!(f, "CARD_ERROR"),
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
            _ => write!(f, ""),
        }
    }
}

impl ResponseError for GameError {
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
            Self::NoCardsLeft => HttpResponse::build(StatusCode::BAD_REQUEST).body("No cards left"),
            _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .body("An unknown error occurred"),
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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub enum MessageProcessResult<T> {
    Success(T),                  // 성공적으로 메시지 처리
    NeedRetry,                   // 에러가 발생했지만 재시도 가능
    TerminateSession(GameError), // 세션 종료 필요
}
