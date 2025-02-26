use serde::{Deserialize, Serialize};

use crate::{enums::UUID, exception::ServerError};

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
    #[serde(rename = "invalid-approach")]
    InvalidApproach(ErrorPayload),
}

/// 각 단계에서 공통으로 사용되는 payload 구조체입니다.
#[derive(Serialize, Deserialize, Debug)]
pub struct MulliganPayload {
    pub player: String,
    pub cards: Vec<UUID>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorPayload {
    pub message: String,
}

/// 각 단계별 JSON 직렬화 함수입니다.
/// 이 함수를 통해 endpoint 내에서 단계에 맞는 JSON 문자열을 깔끔하게 생성할 수 있습니다.

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

pub fn serialize_invalid_approach() -> Result<String, ServerError> {
    let message = MulliganMessage::InvalidApproach(ErrorPayload {
        message: "Invalid approach".to_string(),
    });
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}
