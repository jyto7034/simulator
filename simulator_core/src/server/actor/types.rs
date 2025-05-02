use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateSnapshot {
    pub current_phase: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInputRequest {
    pub request_id: Uuid,
    pub input_type: PlayerInputType,
    pub options: Vec<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerInputType {
    SelectCardFromHand,
    SelectTargetOnField,
    ChooseEffect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PlayerInputResponseData {
    CardSelection(Vec<Uuid>),
    TargetSelection(Uuid),
    EffectChoice(String),
}
