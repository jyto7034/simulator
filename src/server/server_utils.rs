use std::str::from_utf8;

use actix_web::web::Bytes;
use serde_json::{json, Value};

use crate::{enums::UUID, exception::ServerError};

use super::jsons::MulliganMessage;

pub fn parse_to_mulligan_msg(json_raw: String) -> MulliganMessage {
    serde_json::from_str(&json_raw).unwrap()
}

// TODO: 에러 처리 확실히.
pub fn serialize_cards_to_mulligan_json(cards: Vec<UUID>) -> Result<String, ServerError> {
    let value: Value = json!(cards);
    let bytes = serde_json::to_string(&value)
        .map(Bytes::from)
        .map_err(|_| return ServerError::InternalServerError)?;
    let result = from_utf8(&bytes).map_err(|_| return ServerError::InternalServerError)?;
    Ok(result.to_string())
}
