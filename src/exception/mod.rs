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
    NoCardLeft,
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
    HandleFailed,
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
}

impl From<GameError> for ServerError {
    fn from(value: GameError) -> Self {
        match value {
            GameError::InvalidTargetCount => todo!(),
            GameError::NoValidTargets => todo!(),
            GameError::CannotActivate => todo!(),
            GameError::DeckCodeIsMissing(player_type) => todo!(),
            GameError::PlayerInitializeFailed => todo!(),
            GameError::PlayerDataNotIntegrity => todo!(),
            GameError::PathNotExist => todo!(),
            GameError::CardsNotFound => todo!(),
            GameError::GameInitializeFailed => todo!(),
            GameError::DifferentCardTypes => todo!(),
            GameError::GenerateUUIDFaild => todo!(),
            GameError::CardNotFound => todo!(),
            GameError::ExceededCardLimit => todo!(),
            GameError::FailedToDrawCard => todo!(),
            GameError::NothingToRemove => todo!(),
            GameError::InvalidCardData => todo!(),
            GameError::NotAuthenticated => todo!(),
            GameError::InvalidCardType => todo!(),
            GameError::InvalidPlayerType => todo!(),
            GameError::InvalidOperation => todo!(),
            GameError::JsonParseFailed => todo!(),
            GameError::DecodeError => todo!(),
            GameError::DeckParseError => todo!(),
            GameError::ReadFileFailed => todo!(),
            GameError::NoCardsLeft => todo!(),
            GameError::NoCardLeft => todo!(),
            GameError::CardError => todo!(),
            GameError::Ok => todo!(),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::WrongPhase(_, _) => write!(f, "WRONG_PHASE"),
            Self::NotFound => todo!(),
            Self::HandleFailed => todo!(),
            Self::InternalServerError => todo!(),
            Self::CookieNotFound => todo!(),
            Self::ServerStateNotFound => todo!(),
            Self::InvalidPayload => todo!(),
            Self::ActiveSessionExists(_) => todo!(),
            Self::UnexpectedMessage => todo!(),
            Self::InvalidApproach => write!(f, "INVALID_APPROACH"),
            Self::InvalidCards => write!(f, "INVALID_CARDS"),
            Self::ParseError(_) => write!(f, "PARSE_ERROR"),
            Self::InvalidPlayer => write!(f, "INVALID_PLAYER"),
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
            Self::ActiveSessionExists(_) => todo!(),
            Self::ParseError(_) => todo!(),
            Self::UnexpectedMessage => todo!(),
            Self::InvalidCards => todo!(),
            Self::InvalidPlayer => todo!(),
            Self::InvalidApproach => todo!(),
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
            Self::ActiveSessionExists(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ParseError(_) => todo!(),
            Self::UnexpectedMessage => todo!(),
            Self::InvalidCards => todo!(),
            Self::InvalidPlayer => todo!(),
            Self::InvalidApproach => todo!(),
        }
    }
}

pub enum MessageProcessResult<T> {
    Success(T),                    // 성공적으로 메시지 처리
    NeedRetry,                     // 에러가 발생했지만 재시도 가능
    TerminateSession(ServerError), // 세션 종료 필요
}
