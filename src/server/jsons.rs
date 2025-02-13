use serde::{Deserialize, Serialize};

use crate::{enums::UUID, exception::ServerError};

/// 공통 메시지 envelope를 정의합니다.
/// serde의 내부 태그 기능을 이용해, action에 따라 다른 payload를 선택합니다.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MulliganMessage {
    Deal { payload: MulliganPayload },
    Reroll { payload: MulliganPayload },
    Complete,
}

/// 각 단계에서 공통으로 사용되는 payload 구조체입니다.
#[derive(Serialize, Deserialize)]
pub struct MulliganPayload {
    pub player: String,
    pub cards: Vec<UUID>,
}

/// 각 단계별 JSON 직렬화 함수입니다.
/// 이 함수를 통해 endpoint 내에서 단계에 맞는 JSON 문자열을 깔끔하게 생성할 수 있습니다.

pub fn serialize_deal_message<T: Into<String>>(
    player: T,
    cards: Vec<UUID>,
) -> Result<String, ServerError> {
    let message = MulliganMessage::Deal {
        payload: MulliganPayload {
            player: player.into(),
            cards,
        },
    };
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}

pub fn serialize_reroll_message<T: Into<String>>(
    player: T,
    cards: Vec<UUID>,
) -> Result<String, ServerError> {
    let message = MulliganMessage::Reroll {
        payload: MulliganPayload {
            player: player.into(),
            cards,
        },
    };
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}

pub fn serialize_complete_message() -> Result<String, ServerError> {
    let message = MulliganMessage::Complete {};
    serde_json::to_string(&message).map_err(|_| ServerError::InternalServerError)
}
