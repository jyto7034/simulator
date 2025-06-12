use serde::Serialize;
use uuid::Uuid;

use crate::{card::types::PlayerKind, enums::ZoneType};

/// 상태 변경의 최소 단위를 나타내는 enum (델타)
/// Serialize를 통해 JSON으로 변환 가능해야 합니다.
#[derive(Serialize, Clone, Debug)]
pub enum StateChange {
    CardMoved {
        card_uuid: Uuid,
        from: ZoneType,
        to: ZoneType,
    },
    StatChanged {
        card_uuid: Uuid,
        new_attack: Option<i32>,
        new_health: Option<i32>,
    },
    ResourceUpdated {
        player: PlayerKind,
        new_mana: i32,
        new_cost: i32,
    },
    TurnChanged {
        new_turn_player: PlayerKind,
        turn_count: usize,
    },
    PhaseChanged {
        new_phase: String, // Phase enum을 문자열로 변환하여 전송
    },
}

/// 클라이언트에게 실제로 전송될 이벤트 묶음
/// 이 구조체가 하나의 동기화 단위를 형성합니다.
#[derive(Serialize, Clone, Debug)]
pub struct StateUpdatePayload {
    pub seq: u64,
    pub changes: Vec<StateChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_hash: Option<String>,
}
