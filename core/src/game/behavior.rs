use serde::{Deserialize, Serialize};

use crate::{
    ecs::components::Player,
    game::events::event_selection::{bonus::BonusType, EventSelectionOptions},
};

/// GameServer에서 GameCore로 전달되는 플레이어 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerBehavior {
    /// 새 게임 시작
    StartNewGame,

    /// 이벤트 선택 (상점/보너스/랜덤)
    SelectEvent { event_id: String },

    /// 이상현상 진압
    SuppressAbnormality { abnormality_id: String },

    /// 시련(Ordeal) 전투
    Ordeal { opponent_data: Player },

    /// Phase 진행
    AdvancePhase,

    /// 아이템 구매
    PurchaseItem { shop_id: String, item_id: String },

    /// 보너스 선택
    SelectBonus { bonus_type: BonusType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorResult {
    /// 새 게임 시작 → 이벤트 선택지 반환
    StartNewGame { result: EventSelectionOptions },

    /// 이벤트 선택 → 선택된 이벤트 상세 정보
    SelectEvent {
        // TODO: 이벤트 타입에 따라 다른 결과 (Shop, Bonus, Random)
        event_result: String,
    },

    /// 진압 작업 → 진압 결과
    SuppressAbnormality {
        // TODO: 진압 성공/실패, 보상 등
        suppress_result: String,
    },

    /// 시련 전투 → 전투 결과
    Ordeal {
        // TODO: 승패, 보상, 전투 로그 등
        battle_result: String,
    },

    /// Phase 진행 → 다음 Phase 이벤트
    AdvancePhase {
        // TODO: 다음 Phase의 이벤트 정보
        next_phase_event: String,
    },

    /// 아이템 구매 → 구매 확인
    PurchaseItem {
        // TODO: 구매한 아이템, 남은 골드 등
        purchase_result: String,
    },

    /// 보너스 선택 → 보너스 적용 결과
    SelectBonus {
        // TODO: 적용된 보너스 정보
        bonus_result: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    InvalidEvent,
    InsufficientResources,
    PhaseNotReady,
}
