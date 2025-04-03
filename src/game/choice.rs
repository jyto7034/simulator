use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{card::types::PlayerType, exception::GameError, server::jsons::game_features};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChoiceType {
    Dig,          // 덱에서 카드 탐색
    Discard,      // 핸드에서 버릴 카드 선택
    SelectTarget, // 대상 선택 (유닛, 플레이어 등)
    Sacrifice,    // 희생할 카드 선택
    Rearrange,    // 카드 재배치/순서 변경
    RevealChoice, // 공개된 카드 중 선택
    MultiZone,    // 여러 영역에서 선택
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChoiceDestination {
    Hand,               // 손으로
    Field,              // 필드로
    Graveyard,          // 버림 더미로
    TopOfDeck,          // 덱 상단으로
    BottomOfDeck,       // 덱 하단으로
    Exile,              // 추방 영역으로
    Shuffle,            // 덱에 섞기
    CustomZone(String), // 특수 영역
}

#[derive(Debug, Clone)]
pub struct ChoiceState {
    // 기본 정보
    player: PlayerType,
    choice_type: ChoiceType,

    // 소스 및 대상 정보
    source_card_id: Option<Uuid>,   // 선택 효과를 발동한 카드
    source_effect_id: Option<Uuid>, // 선택을 요청한 효과
    selectable_cards: Vec<Uuid>,    // 선택 가능한 카드 목록
    selected_cards: Vec<Uuid>,      // 현재 선택된 카드 목록

    // 선택 제한 설정
    min_selections: usize,          // 최소 선택 개수
    max_selections: usize,          // 최대 선택 개수
    destination: ChoiceDestination, // 선택 후 카드 목적지

    // 상태 관리
    is_open: bool,             // 선택이 활성화되어 있는지
    is_mandatory: bool,        // 필수 선택 여부 (취소 불가)
    created_at: Instant,       // 선택 요청 생성 시간
    timeout: Option<Duration>, // 제한 시간

    is_hidden_from_opponent: bool, // 상대방에게 숨김 여부
}

impl ChoiceState {
    pub fn new(player: PlayerType, choice_type: ChoiceType) -> Self {
        Self {
            player,
            choice_type,
            source_card_id: None,
            source_effect_id: None,
            selectable_cards: Vec::new(),
            selected_cards: Vec::new(),
            min_selections: 1,
            max_selections: 1,
            destination: ChoiceDestination::Hand,
            is_open: true,
            is_mandatory: true,
            created_at: Instant::now(),
            timeout: None,
            is_hidden_from_opponent: false,
        }
    }

    pub fn serialize_message(&self) -> Result<String, GameError> {
        let message = game_features::ChoiceCardPayload {
            player: todo!(),
            choice_type: todo!(),
            min_selections: todo!(),
            max_selections: todo!(),
            is_open: todo!(),
            is_hidden_from_opponent: todo!(),
        };
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }
}
