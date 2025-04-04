use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    card::types::PlayerType, enums::ZoneType, exception::GameError, server::jsons::game_features,
};

// ChoiceType 은 카드 선택의 종류를 나타냄
// 클라이언트 단으로 전달할 때, 게임 진행을 위한 정보가 여럿 포함되어 있는데.
// 이러한 정보들은 effect 로부터 얻어와야함.
// 대표적으로 DigEffect 의 경우, src, dest 정보가 필요함.
// src 의 경우 selector 에서 얻어오고
// dst 의 경우 insert 에서 얻어오면 될 듯?
// 근데 take, insert 는 카드를 어디서 가져오는지 정보를 가지고 있지 않음.
// 그러한 정보는 외부에서 저장되어있음.
// 그래서 Effect 자체에서 가지고 있는게 나을듯?
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

impl ChoiceType {
    pub fn to_string(&self) -> String {
        match self {
            ChoiceType::Dig => "Dig".to_string(),
            ChoiceType::Discard => "Discard".to_string(),
            ChoiceType::SelectTarget => "SelectTarget".to_string(),
            ChoiceType::Sacrifice => "Sacrifice".to_string(),
            ChoiceType::Rearrange => "Rearrange".to_string(),
            ChoiceType::RevealChoice => "RevealChoice".to_string(),
            ChoiceType::MultiZone => "MultiZone".to_string(),
        }
    }
}

// 사용자
#[derive(Debug, Clone)]
pub struct ChoiceState {
    // 기본 정보
    player: PlayerType,
    choice_type: ChoiceType,

    // 소스 및 대상 정보
    source_card_id: Option<Uuid>,   // 선택 효과를 발동한 카드
    source_effect_id: Option<Uuid>, // 선택을 요청한 효과

    // 선택 제한 설정
    min_selections: usize, // 최소 선택 개수
    max_selections: usize, // 최대 선택 개수
    destination: ZoneType, // 선택 후 카드 목적지

    // 상태 관리
    is_open: bool,      // 선택이 활성화되어 있는지
    is_mandatory: bool, // 필수 선택 여부 (취소 불가)

    is_hidden_from_opponent: bool, // 상대방에게 숨김 여부
}

impl Default for ChoiceState {
    fn default() -> Self {
        Self {
            player: PlayerType::Player1,
            choice_type: ChoiceType::Dig,
            source_card_id: None,
            source_effect_id: None,
            min_selections: 1,
            max_selections: 1,
            destination: ZoneType::Hand,
            is_open: true,
            is_mandatory: true,
            is_hidden_from_opponent: false,
        }
    }
}

impl ChoiceState {
    pub fn builder(player: PlayerType, choice_type: ChoiceType) -> ChoiceStateBuilder {
        ChoiceStateBuilder::new(player, choice_type)
    }

    pub fn new(
        player: PlayerType,
        choice_type: ChoiceType,
        source_card_id: Option<Uuid>,
        source_effect_id: Option<Uuid>,
        min_selections: usize,
        max_selections: usize,
        destination: ZoneType,
        is_open: bool,
        is_mandatory: bool,
        is_hidden_from_opponent: bool,
    ) -> Self {
        Self {
            player,
            choice_type,
            source_card_id,
            source_effect_id,
            min_selections,
            max_selections,
            destination,
            is_open,
            is_mandatory,
            is_hidden_from_opponent,
        }
    }

    pub fn serialize_message(&self) -> Result<String, GameError> {
        // ChoiceState의 정보를 ChoiceCardPayload로 변환
        let message = game_features::ChoiceCardPayload {
            player: self.player.to_string(), // PlayerType을 문자열로 변환
            choice_type: self.choice_type.to_string(), // ChoiceType을 문자열로 변환
            source_card_id: todo!(),
            min_selections: self.min_selections,
            max_selections: self.max_selections,
            destination: todo!(),
            is_open: self.is_open,
            is_hidden_from_opponent: self.is_hidden_from_opponent,
        };

        // JSON 문자열로 직렬화
        serde_json::to_string(&message).map_err(|_| GameError::InternalServerError)
    }
}

pub struct ChoiceStateBuilder {
    player: PlayerType,
    choice_type: ChoiceType,
    source_card_id: Option<Uuid>,
    source_effect_id: Option<Uuid>,
    min_selections: usize,
    max_selections: usize,
    destination: ZoneType,
    is_open: bool,
    is_mandatory: bool,
    is_hidden_from_opponent: bool,
}

impl ChoiceStateBuilder {
    pub fn new(player: PlayerType, choice_type: ChoiceType) -> Self {
        Self {
            player,
            choice_type,
            source_card_id: None,
            source_effect_id: None,
            min_selections: 1,
            max_selections: 1,
            destination: ZoneType::Hand,
            is_open: false,
            is_mandatory: false,
            is_hidden_from_opponent: false,
        }
    }

    pub fn source_card(mut self, card_id: impl Into<Option<Uuid>>) -> Self {
        self.source_card_id = card_id.into();
        self
    }

    pub fn source_effect(mut self, effect_id: impl Into<Option<Uuid>>) -> Self {
        self.source_effect_id = effect_id.into();
        self
    }

    pub fn selections(mut self, min: usize, max: usize) -> Self {
        self.min_selections = min;
        self.max_selections = max;
        self
    }

    pub fn destination(mut self, destination: ZoneType) -> Self {
        self.destination = destination;
        self
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn mandatory(mut self, is_mandatory: bool) -> Self {
        self.is_mandatory = is_mandatory;
        self
    }

    pub fn hidden_from_opponent(mut self, is_hidden: bool) -> Self {
        self.is_hidden_from_opponent = is_hidden;
        self
    }

    pub fn build(self) -> ChoiceState {
        ChoiceState::new(
            self.player,
            self.choice_type,
            self.source_card_id,
            self.source_effect_id,
            self.min_selections,
            self.max_selections,
            self.destination,
            self.is_open,
            self.is_mandatory,
            self.is_hidden_from_opponent,
        )
    }
}
