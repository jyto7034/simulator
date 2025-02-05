use std::fmt;

use rocket::{http::Status, response::Responder};
use paste::paste;
use serde::Serialize;

use crate::{card::types::PlayerType, enums::phase::Phase};

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


#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    error_type: String,
}

macro_rules! define_server_errors {
    (
        struct {
            $(
                $(#[$sattr:meta])*
                $svariant:ident { $($sfield:ident : $stype:ty),* $(,)? } => ($sstatus:expr, $serror:expr, $smessage:expr)
            ),* $(,)?
        }
        tuple {
            $(
                $(#[$tattr:meta])*
                $tvariant:ident ( $ttype:ty ) => ($tstatus:expr, $terror:expr, $tmessage:expr)
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, PartialEq, Clone)]
        pub enum ServerError {
            $(
                $(#[$sattr])*
                $svariant { $($sfield: $stype),* },
            )*
            $(
                $(#[$tattr])*
                $tvariant($ttype),
            )*
        }

        impl<'r> Responder<'r, 'static> for ServerError {
            fn respond_to(self, _req: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
                let (status, msg, err_type) = match self {
                    $(
                        ServerError::$svariant { $($sfield),* } => {
                            (
                                $sstatus,
                                format!($smessage $(, $sfield)*),
                                $serror
                            )
                        },
                    )*
                    $(
                        ServerError::$tvariant(val) => {
                            (
                                $tstatus,
                                format!($tmessage, val),
                                $terror
                            )
                        },
                    )*
                };

            // JSON 응답 생성
            let error_response = ErrorResponse {
                code: status.code,
                message: msg,
                error_type: err_type.to_string(),
            };

            rocket::Response::build()
                .status(status)
                .header(rocket::http::ContentType::JSON)
                .sized_body(None, std::io::Cursor::new(
                    serde_json::to_string(&error_response).unwrap()
                ))
                .ok()
            }
        }
        paste! {
            impl ServerError {
                $(
                    /// 편의 생성자: 인자 없이 기본 메시지로 생성합니다.
                    pub fn [<$tvariant:snake _default>]() -> Self {
                        // $tmessage는 예를 들어 "Player not found: {}"와 같이 정의되어 있으므로,
                        // 인자로 빈 문자열을 넘겨 기본 메시지("Player not found: ")를 생성합니다.
                        Self::$tvariant(format!($tmessage, "").trim().to_string())
                    }
                )*
            }
        }
    };
}
// 매크로 사용 예시
define_server_errors! {
    struct {
        WrongPhase { current: Phase, expected: Phase } 
            => (Status::BadRequest, "WrongPhase", "Wrong game phase. Current: {:?}, Expected: {:?}"),
        WrongTurn { current: PlayerType, expected: PlayerType } 
            => (Status::BadRequest, "WorngTurn", "Wrong game turn. Current: {:?}, Expected: {:?}")
    }
    tuple {
        PlayerNotFound(String) 
            => (Status::NotFound, "PlayerNotFound", "Player not found: {}"),
        SystemError(String)
            => (Status::InternalServerError, "SystemError", "System error: {}"),
        NotAuthenticated(String)
            => (Status::Unauthorized, "NotAuthenticated", "Not authenticated: {}"),
        BadRequest(String)
            => (Status::BadRequest, "BadRequest", "Bad request {}")
    }
}