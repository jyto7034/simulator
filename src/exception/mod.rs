use crate::card::types::PlayerType;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

// 에러 메시지를 상수로 정의
pub const PLAYER_INITIALIZE_FAILED: &str = "PLAYER_INITIALIZE_FAILED";
pub const PLAYER_DATA_NOT_INTEGRITY: &str = "PLAYER_DATA_NOT_INTEGRITY";
pub const GENERATE_UUID_FAILED: &str = "GENERATE_UUID_FAILED";
pub const JSON_PARSE_FAILED: &str = "JSON_PARSE_FAILED";
pub const DECK_PARSE: &str = "DECK_PARSE";
pub const PATH_NOT_EXIST: &str = "PATH_NOT_EXIST";
pub const CARD_ERORR: &str = "CARD_ERORR";
pub const UNKNOWN: &str = "UNKNOWN";
pub const WRONG_PHASE: &str = "WRONG_PHASE";
pub const NOT_FOUND: &str = "NOT_FOUND";
pub const HANDLE_FAILED: &str = "HANDLE_FAILED";
pub const INTERNAL_SERVER: &str = "INTERNAL_SERVER";
pub const COOKIE_NOT_FOUND: &str = "COOKIE_NOT_FOUND";
pub const SERVER_STATE_NOT_FOUND: &str = "SERVER_STATE_NOT_FOUND";
pub const INVALID_PAYLOAD: &str = "INVALID_PAYLOAD";
pub const ACTIVE_SESSION_EXISTS: &str = "ACTIVE_SESSION_EXISTS";
pub const UNEXPECTED_MESSAGE: &str = "UNEXPECTED_MESSAGE";
pub const INVALID_APPROACH: &str = "INVALID_APPROACH";
pub const INVALID_CARDS: &str = "INVALID_CARDS";
pub const PARSE: &str = "PARSE";
pub const INVALID_PLAYER: &str = "INVALID_PLAYER";
pub const NOT_ALLOWED_RE_ENTRY: &str = "NOT_ALLOWED_RE_ENTRY";
pub const ALREADY_READY: &str = "ALREADY_READY";
pub const INVALID_OPERATION: &str = "INVALID_OPERATION";
pub const NO_CARDS_LEFT: &str = "NO_CARDS_LEFT";
pub const UNKNOWN_OCCURRED: &str = "AN_UNKNOWN_OCCURRED";
pub const INTERNAL_SERVER_MSG: &str = "INTERNAL_SERVER_MSG";
pub const PARSE_MSG: &str = "PARSE_MSG";
pub const EXCEEDED_CARD_LIMIT: &str = "EXCCED_CARD_LIMIT";

#[derive(Debug, PartialEq, Clone)]
pub enum GameError {
    CardCannotActivate,
    InputNotExpected,
    InvalidTarget,
    InvalidChainState,
    MissingInput,
    EffectNotFound,
    AlreadySelected,
    InvalidSelection,
    TooManySelections,
    SelectionClosed,
    InvalidEffectType,
    InvalidRequestId,

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
    WrongPhase,
    AlreadyReady,
    HandleFailed,
    NotAllowedReEntry,
    InternalServerError,
    UnexpectedMessage,
    CookieNotFound,
    ServerStateNotFound,
    ActiveSessionExists,
    ParseError,
    InvalidPayload,
    InvalidApproach,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerInitializeFailed => write!(f, "{}", PLAYER_INITIALIZE_FAILED),
            Self::PlayerDataNotIntegrity => write!(f, "{}", PLAYER_DATA_NOT_INTEGRITY),
            Self::GenerateUUIDFaild => write!(f, "{}", GENERATE_UUID_FAILED),
            Self::JsonParseFailed => write!(f, "{}", JSON_PARSE_FAILED),
            Self::DeckParseError => write!(f, "{}", DECK_PARSE),
            Self::PathNotExist => write!(f, "{}", PATH_NOT_EXIST),
            Self::CardError => write!(f, "{}", CARD_ERORR),
            Self::Unknown => write!(f, "{}", UNKNOWN),
            Self::WrongPhase => write!(f, "{}", WRONG_PHASE),
            Self::HandleFailed => write!(f, "{}", HANDLE_FAILED),
            Self::InternalServerError => write!(f, "{}", INTERNAL_SERVER),
            Self::CookieNotFound => write!(f, "{}", COOKIE_NOT_FOUND),
            Self::ServerStateNotFound => write!(f, "{}", SERVER_STATE_NOT_FOUND),
            Self::InvalidPayload => write!(f, "{}", INVALID_PAYLOAD),
            Self::ActiveSessionExists => write!(f, "{}", ACTIVE_SESSION_EXISTS),
            Self::UnexpectedMessage => write!(f, "{}", UNEXPECTED_MESSAGE),
            Self::InvalidApproach => write!(f, "{}", INVALID_APPROACH),
            Self::InvalidCards => write!(f, "{}", INVALID_CARDS),
            Self::ParseError => write!(f, "{}", PARSE),
            Self::InvalidPlayer => write!(f, "{}", INVALID_PLAYER),
            Self::NotAllowedReEntry => write!(f, "{}", NOT_ALLOWED_RE_ENTRY),
            Self::AlreadyReady => write!(f, "{}", ALREADY_READY),
            Self::InvalidOperation => write!(f, "{}", INVALID_OPERATION),
            Self::NoCardsLeft => write!(f, "{}", NO_CARDS_LEFT),
            Self::ExceededCardLimit => write!(f, "{}", EXCEEDED_CARD_LIMIT),
            _ => write!(f, ""),
        }
    }
}

impl ResponseError for GameError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Unknown => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(UNKNOWN_OCCURRED)
            }
            Self::WrongPhase => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(WRONG_PHASE)
            }
            Self::HandleFailed => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(UNKNOWN_OCCURRED)
            }
            Self::InternalServerError => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(INTERNAL_SERVER_MSG)
            }
            Self::CookieNotFound => {
                HttpResponse::build(StatusCode::NOT_FOUND).body(COOKIE_NOT_FOUND)
            }
            Self::ServerStateNotFound => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(SERVER_STATE_NOT_FOUND)
            }
            Self::InvalidPayload => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(INVALID_PAYLOAD)
            }
            Self::ActiveSessionExists => {
                HttpResponse::build(StatusCode::CONFLICT).body(ACTIVE_SESSION_EXISTS)
            }
            Self::ParseError => HttpResponse::build(StatusCode::BAD_REQUEST).body(PARSE_MSG),
            Self::UnexpectedMessage => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(UNEXPECTED_MESSAGE)
            }
            Self::InvalidCards => HttpResponse::build(StatusCode::BAD_REQUEST).body(INVALID_CARDS),
            Self::InvalidPlayer => {
                HttpResponse::build(StatusCode::UNAUTHORIZED).body(INVALID_PLAYER)
            }
            Self::InvalidApproach => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(INVALID_APPROACH)
            }
            Self::NotAllowedReEntry => {
                HttpResponse::build(StatusCode::CONFLICT).body(NOT_ALLOWED_RE_ENTRY)
            }
            Self::AlreadyReady => HttpResponse::build(StatusCode::CONFLICT).body(ALREADY_READY),
            Self::InvalidOperation => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(INVALID_OPERATION)
            }
            Self::NoCardsLeft => HttpResponse::build(StatusCode::BAD_REQUEST).body(NO_CARDS_LEFT),
            // TODO: draw_no_cards_left 테스트 작성중이었음.
            Self::ExceededCardLimit => {
                HttpResponse::build(StatusCode::BAD_REQUEST).body(EXCEEDED_CARD_LIMIT)
            }
            _ => HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body(UNKNOWN_OCCURRED),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WrongPhase => StatusCode::INTERNAL_SERVER_ERROR,
            Self::HandleFailed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CookieNotFound => StatusCode::NOT_FOUND,
            Self::ServerStateNotFound => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidPayload => StatusCode::BAD_REQUEST,
            Self::ActiveSessionExists => StatusCode::CONFLICT,
            Self::ParseError => StatusCode::BAD_REQUEST,
            Self::UnexpectedMessage => StatusCode::BAD_REQUEST,
            Self::InvalidCards => StatusCode::BAD_REQUEST,
            Self::InvalidPlayer => StatusCode::UNAUTHORIZED,
            Self::InvalidApproach => StatusCode::BAD_REQUEST,
            Self::NotAllowedReEntry => StatusCode::CONFLICT,
            Self::AlreadyReady => StatusCode::CONFLICT,
            Self::InvalidOperation => StatusCode::BAD_REQUEST,
            Self::NoCardsLeft => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub enum MessageProcessResult<T> {
    Success(T),                  // 성공적으로 메시지 처리
    SystemHandled,               // 게임 시스템에서 처리하는 메시지
    NeedRetry,                   // 에러가 발생했지만 재시도 가능
    TerminateSession(GameError), // 세션 종료 필요
}
