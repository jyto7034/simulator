use serde::{Deserialize, Serialize};

use crate::{
    card::cards::{CardVecExt, Cards},
    enums::UUID,
    exception::ServerError,
};

use super::types::ValidationPayload;

/// 공통 메시지 envelope를 정의합니다.
/// serde의 내부 태그 기능을 이용해, action에 따라 다른 payload를 선택합니다.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", content = "payload")]
pub enum MulliganMessage {
    #[serde(rename = "deal")]
    Deal(MulliganPayload),
    #[serde(rename = "reroll-request")]
    RerollRequest(MulliganPayload),
    #[serde(rename = "reroll-answer")]
    RerollAnswer(MulliganPayload),
    #[serde(rename = "complete")]
    Complete(MulliganPayload),
    #[serde(rename = "error")]
    Error(ErrorPayload),
}

/// 각 단계에서 공통으로 사용되는 payload 구조체입니다.
#[derive(Serialize, Deserialize, Debug)]
pub struct MulliganPayload {
    pub player: String,
    pub cards: Vec<UUID>,
}

impl ValidationPayload for MulliganPayload {
    fn validate(&self, player_cards: &Cards) -> Option<()> {
        // self.cards 가 빈 경우에는 무조건 true 를 반환함.
        if !self
            .cards
            .iter()
            .all(|uuid| player_cards.contains_uuid(uuid.clone()))
        {
            return None;
        }

        if !matches!(self.player.as_str(), "player1" | "player2") {
            return None;
        }
        Some(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorPayload {
    pub message: String,
}

/// 각 단계별 에러 JSON 직렬화 함수입니다.
/// 이 함수를 통해 endpoint 내에서 단계에 맞는 JSON 에러 문자열을 깔끔하게 생성할 수 있습니다.
#[macro_export]
macro_rules! serialize_error {
    ($error_msg:expr) => {{
        let message =
            $crate::server::jsons::MulliganMessage::Error($crate::server::jsons::ErrorPayload {
                message: $error_msg.to_string(),
            });
        serde_json::to_string(&message)
            .map_err(|_| $crate::exception::ServerError::InternalServerError)
    }};
}

pub fn serialize_deal_message<T: Into<String>>(
    player: T,
    cards: Vec<UUID>,
) -> Result<String, ServerError> {
    let message = MulliganMessage::Deal(MulliganPayload {
        player: player.into(),
        cards,
    });
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}

pub fn serialize_complete_message<T: Into<String>>(
    player: T,
    cards: Vec<UUID>,
) -> Result<String, ServerError> {
    let message = MulliganMessage::Complete(MulliganPayload {
        player: player.into(),
        cards,
    });
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}
