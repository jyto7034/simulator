use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::{Enkephalin, InventoryMetadata},
    game::{
        data::{
            random_event_data::RandomEventMetadata,
            shop_data::ShopMetadata,
            ItemReference,
        },
        enums::{Category, PhaseEvent},
    },
};

/// GameServer에서 GameCore로 전달되는 플레이어 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerBehavior {
    /// 새 게임 시작
    StartNewGame,

    /// 현재 페이즈 데이터 요청
    RequestPhaseData,

    /// 이벤트 선택 (상점/보너스/랜덤)
    SelectEvent { event_id: Uuid },

    // ============================================================
    // 상점 관련 행동
    // ============================================================
    /// 아이템 구매
    PurchaseItem {
        item_uuid: Uuid,
        item_category: Category,
    },

    /// 상점 리롤 (새로운 아이템으로 교체)
    RerollShop,

    /// 상점 나가기
    ExitShop,

    // ============================================================
    // 랜덤 이벤트 관련 행동
    // ============================================================
    /// 랜덤 이벤트 선택지 선택
    SelectEventChoice { choice_id: String },

    /// 랜덤 이벤트 종료
    ExitRandomEvent,
    // ============================================================
    // 진압 관련 행동 (TODO)
    // ============================================================
    // SelectWorkType { work_type: WorkType },
    // ExitSuppression,

    // ============================================================
    // 전투 관련 행동 (TODO)
    // ============================================================
    // UseCard { card_uuid: Uuid },
    // EndTurn,
}

/// BehaviorResult 는 변경된 모든 값을 넘길 의무가 있음
/// 예를 들어 PurchaseItem 의 경우
/// 1. 구매된 아이템
/// 2. 아이템은 어디에 저장되는지
/// 3. 남은 자원은 얼마인지
/// 4. 해당 아이템이 어디서 제거되는지,
/// 등. 클라이언트는 해당 값들을 반영만 하게끔 해야함.  
#[derive(Debug, Serialize, Deserialize)]
pub enum BehaviorResult {
    /// 새 게임 시작
    StartNewGame,

    /// 페이즈 데이터 요청 → PhaseEvent 반환 (3개의 GameOption 포함)
    RequestPhaseData(PhaseEvent),

    /// 이벤트 선택 완료 (추가 메타데이터 없음)
    EventSelected,

    /// 상점 상태 업데이트 (예: 리롤 이후)
    ShopState {
        shop: ShopMetadata,
    },

    RerollShop {
        new_items: Vec<ItemReference>,
    },

    SellItem,

    /// 아이템 구매 → 구매 확인
    PurchaseItem {
        enkephalin: u32,
        inventory_metadata: InventoryMetadata,
        shop_metadata: ShopMetadata,
    },

    /// 랜덤 이벤트 상태/결과 업데이트
    RandomEventState {
        event: RandomEventMetadata,
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

    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    InvalidEvent,
    InvalidAction, // 허용되지 않은 행동 (치팅 시도)
    InsufficientResources,
    PhaseNotReady,
}
