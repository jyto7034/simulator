use crate::card::types::PlayerKind;
use actix::Message;
use serde::Serialize;
use uuid::Uuid; // Phase enum import

// ===================================================================
// 스냅샷을 위한 데이터 구조체들
// ===================================================================

/// 카드의 공개 정보를 담는 스냅샷 구조체
#[derive(Serialize, Clone, Debug)]
pub struct CardSnapshot {
    pub uuid: Uuid,
    pub id: String, // "HM_001"과 같은 카드 DB ID
    pub attack: i32,
    pub health: i32,
    pub cost: i32,
    pub is_tapped: bool, // 행동 완료(공격 등) 여부
                         // ... 클라이언트 UI에 필요한 카드의 모든 공개 정보
}

/// 내 손패에 있는 카드처럼 모든 정보가 공개되는 스냅샷 구조체
#[derive(Serialize, Clone, Debug)]
pub struct PrivateCardSnapshot {
    pub uuid: Uuid,
    pub id: String,
    // ... CardSnapshot의 모든 필드 포함 가능
}

/// 게임 전체의 상태를 담는 최상위 스냅샷 구조체
#[derive(Serialize, Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct GameStateSnapshot {
    // --- 글로벌 게임 정보 ---
    pub seq: u64, // 이 스냅샷이 유효한 시점의 시퀀스 번호
    pub state_hash: Option<String>,
    pub current_phase: String, // Phase enum을 문자열로 변환
    pub turn_player: PlayerKind,
    pub turn_count: usize,

    // --- '나'의 관점에서의 정보 ---
    pub my_info: PlayerStateSnapshot,

    // --- '상대'의 관점에서의 정보 ---
    pub opponent_info: OpponentStateSnapshot,
}

/// 플레이어 한 명의 전체 상태를 담는 스냅샷
#[derive(Serialize, Clone, Debug)]
pub struct PlayerStateSnapshot {
    pub player_kind: PlayerKind,
    pub health: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub deck_count: usize,
    pub hand: Vec<PrivateCardSnapshot>, // 내 손패는 모든 정보가 보임
    pub field: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
}

/// 상대방의 공개 정보만 담는 스냅샷
#[derive(Serialize, Clone, Debug)]
pub struct OpponentStateSnapshot {
    pub player_kind: PlayerKind,
    pub health: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub deck_count: usize,
    pub hand_count: usize, // 상대 손패는 개수만 보임
    pub field: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
}
