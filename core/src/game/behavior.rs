use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    ecs::resources::InventoryDiffDto,
    game::{
        data::{random_event_data::RandomEventMetadata, shop_data::ShopMetadata},
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

    /// 아이템 판매
    SellItem { item_uuid: Uuid },

    /// 상점 리롤 (새로운 아이템으로 교체)
    RerollShop,

    /// 상점 나가기
    ExitShop,
    // ============================================================
    // 보너스 관련 행동
    // ============================================================
    /// 보너스 수령
    ClaimBonus,
    /// 보너스 화면 나가기
    ExitBonus,
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
        new_items: Vec<Uuid>,
    },

    /// 아이템 판매 → 판매 확인
    SellItem {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
    },

    /// 아이템 구매 → 구매 확인
    PurchaseItem {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
    },

    /// 랜덤 이벤트 상태/결과 업데이트
    RandomEventState {
        event: RandomEventMetadata,
    },
    /// 보너스 결과 (자원 및 인벤토리 변경)
    BonusReward {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
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

impl BehaviorResult {
    // ============================================================
    // 타입 체크 헬퍼 (데이터가 없는 variant용)
    // ============================================================

    pub fn is_start_new_game(&self) -> bool {
        matches!(self, BehaviorResult::StartNewGame)
    }

    pub fn is_event_selected(&self) -> bool {
        matches!(self, BehaviorResult::EventSelected)
    }

    pub fn is_sell_item(&self) -> bool {
        matches!(self, BehaviorResult::SellItem { .. })
    }

    /// SellItem → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    pub fn as_sell_item(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, BehaviorResult::Ok)
    }

    // ============================================================
    // 데이터 추출 헬퍼 (참조 반환)
    // ============================================================

    /// RequestPhaseData → PhaseEvent 참조 반환
    pub fn as_request_phase_data(&self) -> Option<&PhaseEvent> {
        match self {
            BehaviorResult::RequestPhaseData(event) => Some(event),
            _ => None,
        }
    }

    /// ShopState → ShopMetadata 참조 반환
    pub fn as_shop_state(&self) -> Option<&ShopMetadata> {
        match self {
            BehaviorResult::ShopState { shop } => Some(shop),
            _ => None,
        }
    }

    /// RerollShop → 새로운 아이템 UUID 리스트 참조 반환
    pub fn as_reroll_shop(&self) -> Option<&Vec<Uuid>> {
        match self {
            BehaviorResult::RerollShop { new_items } => Some(new_items),
            _ => None,
        }
    }

    /// PurchaseItem → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    pub fn as_purchase_item(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }

    /// RandomEventState → RandomEventMetadata 참조 반환
    pub fn as_random_event_state(&self) -> Option<&RandomEventMetadata> {
        match self {
            BehaviorResult::RandomEventState { event } => Some(event),
            _ => None,
        }
    }

    /// BonusReward → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    pub fn as_bonus_reward(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::BonusReward {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }

    /// SuppressAbnormality → 진압 결과 문자열 참조 반환
    pub fn as_suppress_abnormality(&self) -> Option<&str> {
        match self {
            BehaviorResult::SuppressAbnormality { suppress_result } => Some(suppress_result),
            _ => None,
        }
    }

    /// Ordeal → 전투 결과 문자열 참조 반환
    pub fn as_ordeal(&self) -> Option<&str> {
        match self {
            BehaviorResult::Ordeal { battle_result } => Some(battle_result),
            _ => None,
        }
    }

    /// AdvancePhase → 다음 Phase 이벤트 문자열 참조 반환
    pub fn as_advance_phase(&self) -> Option<&str> {
        match self {
            BehaviorResult::AdvancePhase { next_phase_event } => Some(next_phase_event),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    /// 선택한 이벤트 ID가 현재 Phase의 옵션에 존재하지 않을 때
    EventNotFound,
    /// 이벤트는 존재하지만 기대한 타입(Shop/Bonus/Random 등)이 아닐 때
    EventTypeMismatch,

    /// 현재 GameState/Context에서 허용되지 않은 행동 (치팅 시도 포함)
    InvalidAction,

    /// 상점 상태가 아니거나, SelectedEvent에 Shop 정보가 없을 때
    NotInShopState,
    /// 보너스 상태가 아니거나, SelectedEvent에 Bonus 정보가 없을 때
    NotInBonusState,
    /// 상점이 리롤을 지원하지 않을 때
    ShopRerollNotAllowed,
    /// 상점의 visible_items / uuid_lookup_table에서 아이템을 찾지 못했을 때
    ShopItemNotFound,

    /// 인벤토리가 가득 차서 아이템을 추가할 수 없을 때 (예: 아티팩트 슬롯)
    InventoryFull,
    /// 인벤토리에서 아이템을 찾을 수 없을 때
    InventoryItemNotFound,

    /// 구매/행동에 필요한 자원이 부족할 때
    InsufficientResources,

    /// 아직 Phase 진행이 준비되지 않았을 때
    PhaseNotReady,

    /// 필수 리소스(Enkephalin, Inventory 등)가 World에 없을 때
    MissingResource(&'static str),

    /// 기물의 전투 스탯이 정의되지 않았거나 잘못된 경우
    InvalidUnitStats(&'static str),
}
