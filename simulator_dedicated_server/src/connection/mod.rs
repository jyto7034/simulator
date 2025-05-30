use serde::{Deserialize, Serialize};
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
}

impl ServerMessage {
    pub fn to_json(&self) -> String {
        match self {
            ServerMessage::HeartbeatConnected { player, session_id } => serde_json::json!({
                "type": "heartbeat_connected",
                "player": player,
                "session_id": session_id.to_string()
            })
            .to_string(),
            ServerMessage::MulliganDealCards { player, cards } => serde_json::json!({
                "type": "mulligan_deal",
                "player": player,
                "cards": cards.iter().map(|id| id.to_string()).collect::<Vec<_>>()
            })
            .to_string(),
        }
    }
}
