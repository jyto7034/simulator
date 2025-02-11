use std::str::from_utf8;

use actix_web::web::Bytes;
use serde_json::{json, Value};
use serde::Deserialize;

use crate::{card::types::PlayerType, enums::UUID, exception::ServerError};

use super::types::ServerGameStep;

pub fn serialize_mulligan_complete_json() -> Result<String, ServerError> {
    let value: Value = json!("");
    let bytes = serde_json::to_string(&value)
        .map(Bytes::from)
        .map_err(|_| return ServerError::InternalServerError)?;
    let result = from_utf8(&bytes).map_err(|_| return ServerError::InternalServerError)?;
    Ok(result.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Reroll,
    Complete,
}

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub player: PlayerType,
    pub cards: Vec<UUID>,
}

#[derive(Debug, Deserialize)]
pub struct MulliganMessage {
    pub step: ServerGameStep,
    pub action: Action,
    pub payload: Payload,
}