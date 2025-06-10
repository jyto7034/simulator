use serde::{Deserialize, Serialize};
use simulator_core::exception::{ConnectionError, GameError, StateError};
use types::PlayerInputResponse;
use uuid::Uuid;

pub mod connection;
pub mod messages;
pub mod types;

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum UserAction {
    #[serde(rename = "rerollRequestMulliganCard")]
    RerollRequestMulliganCard { card_id: Vec<Uuid> },
    #[serde(rename = "completeMulligan")]
    CompleteMulligan,
    #[serde(rename = "playCard")]
    PlayCard {
        card_id: Uuid,
        target_id: Option<Uuid>,
    },
    #[serde(rename = "attack")]
    Attack {
        attacker_id: Uuid,
        defender_id: Uuid,
    },
    #[serde(rename = "endTurn")]
    EndTurn,
    #[serde(rename = "submitInput")]
    SubmitInput {
        request_id: Uuid,
        #[serde(flatten)]
        response_data: PlayerInputResponse,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "heartbeat_connected")]
    HeartbeatConnected { player: String, session_id: Uuid },
    #[serde(rename = "mulligan_deal")]
    MulliganDealCards { player: String, cards: Vec<Uuid> },

    #[serde(rename = "error")]
    Error(ErrorMessagePayload),
}

impl ServerMessage {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| "{\"error\":\"json serialization failed\"}".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // JSON에서는 "ACTIVE_SESSION_EXISTS"와 같이 변환
pub enum ServerErrorCode {
    ActiveSessionExists,
    GameAborted,
    InvalidAction,
    InternalServerError,
}

/// 클라이언트에게 전송될 에러 메시지의 실제 내용입니다.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessagePayload {
    pub code: ServerErrorCode,
    pub message: String,
}

impl From<GameError> for ServerMessage {
    fn from(err: GameError) -> Self {
        // 서버 로그에는 변환 전의 상세한 에러를 기록합니다.
        tracing::warn!("Converting GameError to a client-facing message: {}", err);

        let (code, message) = match &err {
            // 시스템 에러는 내부 구현을 노출하지 않습니다.
            GameError::System(_) => (
                ServerErrorCode::InternalServerError,
                "An internal server error occurred.".to_string(),
            ),

            // 클라이언트가 명확히 처리할 수 있는 특정 에러들
            GameError::Connection(ConnectionError::SessionExists(_)) => {
                (ServerErrorCode::ActiveSessionExists, err.to_string())
            }
            GameError::State(StateError::GameAborted) => {
                (ServerErrorCode::GameAborted, err.to_string())
            }

            // 그 외 모든 에러는 클라이언트의 유효하지 않은 액션으로 간주합니다.
            // Display 트레이트 구현이 클라이언트에게 유용한 정보를 제공합니다.
            GameError::Connection(_) | GameError::State(_) | GameError::Gameplay(_) => {
                (ServerErrorCode::InvalidAction, err.to_string())
            }
        };

        ServerMessage::Error(ErrorMessagePayload { code, message })
    }
}
