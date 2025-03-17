use std::any::Any;

use crate::{
    card::cards::{CardVecExt, Cards},
    exception::GameError,
};

use serde::{Deserialize, Serialize};

/// 모든 메시지 페이로드가 구현해야 하는 기본 트레이트
pub trait MessagePayload: Serialize + for<'de> Deserialize<'de> + Clone + std::fmt::Debug {}

/// 모든 검증 가능한 페이로드가 구현해야 하는 트레이트
pub trait ValidationPayload {
    fn validate(&self, context: &dyn Any) -> Option<()>;
}

/// 모든 메시지가 구현해야 하는 마커 트레이트
// 서버와 클라이언트의 json 구조체가 분리된 시점에서 불필요할 수 도 있음.
pub trait Message: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug {}

/// 공용 에러 메세지 페이로드
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorPayload {
    pub message: String,
}

/// 서버에서 클라이언트로 전송되는 에러 메시지
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", content = "payload")]
pub enum ErrorMessage {
    #[serde(rename = "error")]
    Error(ErrorPayload),
}

//------------------------------------------------------------------------------
// Mulligan 관련 메시지 정의
//------------------------------------------------------------------------------
pub mod mulligan {
    use uuid::Uuid;

    use super::*;

    /// 멀리건 관련 페이로드
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MulliganPayload {
        pub player: String,
        pub cards: Vec<String>,
    }

    impl MulliganPayload {
        fn new(player: String, cards: Vec<Uuid>) -> Self {
            MulliganPayload {
                player,
                cards: cards.iter().map(|uuid| uuid.to_string()).collect(),
            }
        }
    }

    impl MessagePayload for MulliganPayload {}

    impl ValidationPayload for MulliganPayload {
        fn validate(&self, context: &dyn Any) -> Option<()> {
            if let Some(player_cards) = context.downcast_ref::<Cards>() {
                // self.cards가 비어 있으면 무조건 유효함
                if self.cards.is_empty() {
                    return Some(());
                }

                // 카드 UUID 유효성 검사
                // TODO: Unwrap 대신 match를 사용하여 안전하게 처리할 수 있도록 수정
                if !self.cards.iter().all(|uuid| {
                    player_cards.contains_uuid(Uuid::parse_str(uuid).unwrap_or_else(|e| {
                        // TODO: Log 함수 사용
                        Uuid::nil()
                    }))
                }) {
                    return None;
                }

                // 플레이어 ID 유효성 검사
                if !matches!(self.player.as_str(), "player1" | "player2") {
                    return None;
                }

                Some(())
            } else {
                None // 잘못된 컨텍스트 타입
            }
        }
    }

    impl MessagePayload for ErrorPayload {}

    /// 클라이언트에서 서버로 전송되는 멀리건 메시지
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", content = "payload")]
    pub enum ClientMessage {
        #[serde(rename = "reroll-request")]
        RerollRequest(MulliganPayload),
        #[serde(rename = "complete")]
        Complete(MulliganPayload),
    }

    impl Message for ClientMessage {}

    /// 서버에서 클라이언트로 전송되는 멀리건 메시지
    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", content = "payload")]
    pub enum ServerMessage {
        #[serde(rename = "deal")]
        Deal(MulliganPayload),
        #[serde(rename = "reroll-answer")]
        RerollAnswer(MulliganPayload),
    }

    impl Message for ServerMessage {}

    /// 서버에서 클라이언트로 특정 카드들을 제공하는 메시지를 직렬화합니다.
    pub fn serialize_deal_message<T: Into<String>>(
        player: T,
        cards: Vec<Uuid>,
    ) -> Result<String, GameError> {
        let message = ServerMessage::Deal(MulliganPayload::new(player.into(), cards));
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }

    /// 리리롤 응답 메시지를 직렬화합니다.
    pub fn serialize_reroll_answer<T: Into<String>>(
        player: T,
        cards: Vec<Uuid>,
    ) -> Result<String, GameError> {
        let message = ServerMessage::RerollAnswer(MulliganPayload::new(player.into(), cards));
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }

    /// 완료 응답 메시지를 직렬화합니다.
    pub fn serialize_complete_message<T: Into<String>>(
        player: T,
        cards: Vec<Uuid>,
    ) -> Result<String, GameError> {
        let message = ClientMessage::Complete(MulliganPayload::new(player.into(), cards));
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }
}

//------------------------------------------------------------------------------
// Draw 관련 메시지 정의
//------------------------------------------------------------------------------
pub mod draw {
    use uuid::Uuid;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DrawPayload {
        pub player: String,
        pub cards: String,
    }

    impl MessagePayload for DrawPayload {}

    impl ValidationPayload for DrawPayload {
        fn validate(&self, context: &dyn Any) -> Option<()> {
            if let Some(player_cards) = context.downcast_ref::<Cards>() {
                // 카드 UUID 유효성 검사
                if player_cards.contains_uuid(Uuid::parse_str(&self.cards).ok()?) {
                    return None;
                }

                // 플레이어 ID 유효성 검사
                if !matches!(self.player.as_str(), "player1" | "player2") {
                    return None;
                }

                Some(())
            } else {
                None // 잘못된 컨텍스트 타입
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct ErrorPayload {
        pub message: String,
    }

    impl MessagePayload for ErrorPayload {}

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", content = "payload")]
    pub enum ClientMessage {
        #[serde(rename = "draw-request")]
        DrawRequest(DrawPayload),
    }

    impl Message for ClientMessage {}

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "action", content = "payload")]
    pub enum ServerMessage {
        #[serde(rename = "draw-answer")]
        DrawAnswer(DrawPayload),
    }

    impl Message for ServerMessage {}

    /// 클라이언트로 전송할 Draw 카드의 정보가 담긴 메세지를 직렬화합니다.
    pub fn serialize_draw_answer_message<T: Into<String>>(
        player: T,
        cards: Uuid,
    ) -> Result<String, GameError> {
        let message = ServerMessage::DrawAnswer(DrawPayload {
            player: player.into(),
            cards: cards.to_string(),
        });
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }

    /// 클라이언트로 전송할 Draw 카드의 정보가 담긴 메세지를 직렬화합니다.
    pub fn serialize_draw_request_message<T: Into<String>>(
        player: T,
        cards: Uuid,
    ) -> Result<String, GameError> {
        let message = ClientMessage::DrawRequest(DrawPayload {
            player: player.into(),
            cards: cards.to_string(),
        });
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }
}

//------------------------------------------------------------------------------
// 메시지 매크로 및 유틸리티 함수
//------------------------------------------------------------------------------

/// 에러 메시지 직렬화를 위한 매크로
#[macro_export]
macro_rules! serialize_error {
    ($error_msg:expr) => {{
        let message =
            $crate::server::jsons::ErrorMessage::Error($crate::server::jsons::ErrorPayload {
                message: $error_msg.to_string(),
            });
        serde_json::to_string(&message)
            .map_err(|_| $crate::exception::GameError::InternalServerError)
    }};
    ($module:ident, $error_msg:expr) => {{
        let message = $crate::server::jsons::$module::ServerMessage::Error {
            message: $error_msg.to_string(),
        };
        serde_json::to_string(&message)
            .map_err(|_| $crate::exception::GameError::InternalServerError)
    }};
}
