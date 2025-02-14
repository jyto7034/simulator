use crate::{card::types::PlayerType, enums::phase::Phase};
use actix_web::{
    get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder, ResponseError,
};
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
    CookieNotFound,
    ServerStateNotFound,
}

impl From<GameError> for ServerError {
    fn from(value: GameError) -> Self {
        match value {
            _ => ServerError::InternalServerError,
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "An unknown error occurred"),
            Self::WrongPhase(_, _) => write!(f, "Wrong Phase!"),
            Self::NotFound => todo!(),
            Self::HandleFailed => todo!(),
            Self::InternalServerError => todo!(),
            Self::CookieNotFound => todo!(),
            Self::ServerStateNotFound => todo!(),
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
            Self::InternalServerError => todo!(),
            Self::CookieNotFound => todo!(),
            Self::ServerStateNotFound => todo!(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WrongPhase(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::HandleFailed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServerError => todo!(),
            Self::CookieNotFound => todo!(),
            Self::ServerStateNotFound => todo!(),
        }
    }
}
