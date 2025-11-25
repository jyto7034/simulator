use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::behavior::{GameError, PlayerBehavior};
use crate::game::data::shop_data::ShopMetadata;
use crate::game::enums::{GameOption, OrdealType, PhaseType};

pub mod inventory;
pub use inventory::*;

/// 게임의 명시적인 상태
///
/// 현재 게임이 어떤 단계에 있는지 명확하게 표현
/// ActionScheduler가 이 상태를 보고 allowed_actions를 결정함
#[derive(Resource, Debug, Clone, PartialEq)]
pub enum GameState {
    /// 게임 시작 전
    NotStarted,
    /// 게임 시작 후, Phase 데이터 요청 대기 중
    WaitingPhaseRequest,
    /// Phase 데이터 받음, 이벤트 선택 대기 중
    SelectingEvent,
    /// 상점 진입
    InShop { shop_uuid: Uuid },
    /// 랜덤 이벤트 진입
    InRandomEvent { event_uuid: Uuid },
    /// 진압 작업 진행 중
    InSuppression { abnormality_uuid: Uuid },
    /// 시련 전투 진행 중
    InBattle { battle_uuid: Uuid },
    /// 게임 종료
    GameOver,
}

impl Default for GameState {
    fn default() -> Self {
        Self::NotStarted
    }
}
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Enkephalin {
    pub amount: u32,
}

impl Enkephalin {
    pub fn new(initial_amount: u32) -> Self {
        Self {
            amount: initial_amount,
        }
    }
}

#[derive(Resource)]
pub struct Level {
    pub level: u32,
}

impl Level {
    pub fn new(initial_level: u32) -> Self {
        Self {
            level: initial_level,
        }
    }
}

/// 게임 진행 상황 (Ordeal, Phase) - 순수 데이터만
#[derive(Resource, Debug, Clone)]
pub struct GameProgression {
    pub current_ordeal: OrdealType,
    pub current_phase: PhaseType,
}

impl GameProgression {
    pub fn new() -> Self {
        Self {
            current_ordeal: OrdealType::Dawn,
            current_phase: PhaseType::I,
        }
    }
}

impl Default for GameProgression {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource)]
pub struct WinCount {
    pub count: u32,
}

impl WinCount {
    pub fn new(initial_count: u32) -> Self {
        Self {
            count: initial_count,
        }
    }
}

#[derive(Resource)]
pub struct SelectedEvent {
    pub event: GameOption,
}

impl SelectedEvent {
    pub fn new(event: GameOption) -> Self {
        Self { event }
    }

    pub fn as_shop(&self) -> Result<&ShopMetadata, GameError> {
        match &self.event {
            GameOption::Shop { shop } => Ok(shop),
            _ => Err(GameError::InvalidEvent),
        }
    }

    pub fn as_shop_mut(&mut self) -> Result<&mut ShopMetadata, GameError> {
        match &mut self.event {
            GameOption::Shop { shop } => Ok(shop),
            _ => Err(GameError::InvalidEvent),
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentPhaseEvents {
    pub events: HashMap<Uuid, GameOption>,
}

impl CurrentPhaseEvents {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// 이벤트 추가 (GameOption 내부의 uuid를 key로 사용)
    pub fn add_event(&mut self, event: GameOption) {
        let uuid = event.uuid();
        self.events.insert(uuid, event);
    }

    /// UUID로 이벤트 조회
    pub fn get_event(&self, uuid: Uuid) -> Option<&GameOption> {
        self.events.get(&uuid)
    }

    /// 이벤트 제거 (실행 후)
    pub fn remove_event(&mut self, uuid: Uuid) -> Option<GameOption> {
        self.events.remove(&uuid)
    }

    /// 모든 이벤트 클리어 (Phase 전환 시)
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// 현재 저장된 이벤트 개수
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// 현재 게임 컨텍스트 (치팅 방지용)
///
/// 현재 플레이어가 수행할 수 있는 행동 목록을 저장
/// 플레이어의 행동이 이 목록에 없으면 거부됨
#[derive(Resource, Default)]
pub struct CurrentGameContext {
    /// 현재 허용된 행동 목록
    pub allowed_actions: Vec<PlayerBehavior>,
}

impl CurrentGameContext {
    pub fn new() -> Self {
        Self {
            allowed_actions: vec![],
        }
    }

    /// 허용된 행동 설정
    pub fn set_allowed_actions(&mut self, actions: Vec<PlayerBehavior>) {
        self.allowed_actions = actions;
    }

    /// 특정 행동이 허용되는지 확인 (Variant만 비교)
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.allowed_actions
            .iter()
            .any(|allowed| Self::same_variant(allowed, action))
    }

    /// 두 PlayerBehavior가 같은 Variant인지 확인
    fn same_variant(a: &PlayerBehavior, b: &PlayerBehavior) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }

    /// 모든 허용 행동 클리어
    pub fn clear(&mut self) {
        self.allowed_actions.clear();
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::enums::{OrdealType, PhaseType};

    // ============================================================
    // GameState Tests
    // ============================================================

    #[test]
    fn test_game_state_default() {
        let state = GameState::default();
        assert_eq!(state, GameState::NotStarted);
    }

    #[test]
    fn test_game_state_transitions() {
        // NotStarted → WaitingPhaseRequest
        let state = GameState::WaitingPhaseRequest;
        assert_eq!(state, GameState::WaitingPhaseRequest);

        // WaitingPhaseRequest → SelectingEvent
        let state = GameState::SelectingEvent;
        assert_eq!(state, GameState::SelectingEvent);

        // SelectingEvent → InShop
        let shop_uuid = Uuid::new_v4();
        let state = GameState::InShop { shop_uuid };
        if let GameState::InShop { shop_uuid: uuid } = state {
            assert_eq!(uuid, shop_uuid);
        } else {
            panic!("Expected InShop state");
        }
    }

    // ============================================================
    // Enkephalin Tests
    // ============================================================

    #[test]
    fn test_enkephalin_new() {
        let enkephalin = Enkephalin::new(100);
        assert_eq!(enkephalin.amount, 100);
    }

    // ============================================================
    // Level Tests
    // ============================================================

    #[test]
    fn test_level_new() {
        let level = Level::new(1);
        assert_eq!(level.level, 1);
    }

    // ============================================================
    // GameProgression Tests
    // ============================================================

    #[test]
    fn test_game_progression_new() {
        let progression = GameProgression::new();
        assert_eq!(progression.current_ordeal, OrdealType::Dawn);
        assert_eq!(progression.current_phase, PhaseType::I);
    }

    #[test]
    fn test_game_progression_default() {
        let progression = GameProgression::default();
        assert_eq!(progression.current_ordeal, OrdealType::Dawn);
        assert_eq!(progression.current_phase, PhaseType::I);
    }

    // ============================================================
    // WinCount Tests
    // ============================================================

    #[test]
    fn test_win_count_new() {
        let win_count = WinCount::new(0);
        assert_eq!(win_count.count, 0);

        let win_count = WinCount::new(5);
        assert_eq!(win_count.count, 5);
    }

    // ============================================================
    // CurrentPhaseEvents Tests
    // ============================================================

    #[test]
    fn test_current_phase_events_new() {
        let events = CurrentPhaseEvents::new();
        assert_eq!(events.len(), 0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_current_phase_events_default() {
        let events = CurrentPhaseEvents::default();
        assert!(events.is_empty());
    }

    #[test]
    fn test_current_phase_events_add_and_get() {
        use crate::game::enums::{GameOption, RiskLevel};

        let mut events = CurrentPhaseEvents::new();
        let uuid = Uuid::new_v4();

        let option = GameOption::SuppressAbnormality {
            abnormality_id: "F-01-02".to_string(),
            risk_level: RiskLevel::HE,
            uuid,
        };

        events.add_event(option.clone());

        assert_eq!(events.len(), 1);
        assert!(!events.is_empty());

        let retrieved = events.get_event(uuid);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_current_phase_events_remove() {
        use crate::game::enums::{GameOption, RiskLevel};

        let mut events = CurrentPhaseEvents::new();
        let uuid = Uuid::new_v4();

        let option = GameOption::SuppressAbnormality {
            abnormality_id: "F-01-02".to_string(),
            risk_level: RiskLevel::HE,
            uuid,
        };

        events.add_event(option.clone());
        assert_eq!(events.len(), 1);

        let removed = events.remove_event(uuid);
        assert!(removed.is_some());
        assert_eq!(events.len(), 0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_current_phase_events_clear() {
        use crate::game::enums::{GameOption, RiskLevel};

        let mut events = CurrentPhaseEvents::new();

        for i in 0..3 {
            let option = GameOption::SuppressAbnormality {
                abnormality_id: format!("F-01-0{}", i),
                risk_level: RiskLevel::ZAYIN,
                uuid: Uuid::new_v4(),
            };
            events.add_event(option);
        }

        assert_eq!(events.len(), 3);

        events.clear();
        assert_eq!(events.len(), 0);
        assert!(events.is_empty());
    }

    // ============================================================
    // CurrentGameContext Tests
    // ============================================================

    #[test]
    fn test_current_game_context_new() {
        let context = CurrentGameContext::new();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    #[test]
    fn test_current_game_context_default() {
        let context = CurrentGameContext::default();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    #[test]
    fn test_current_game_context_set_allowed_actions() {
        let mut context = CurrentGameContext::new();

        let actions = vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ];

        context.set_allowed_actions(actions.clone());
        assert_eq!(context.allowed_actions.len(), 2);
    }

    #[test]
    fn test_current_game_context_is_action_allowed() {
        let mut context = CurrentGameContext::new();

        let actions = vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ];

        context.set_allowed_actions(actions);

        // 허용된 행동
        assert!(context.is_action_allowed(&PlayerBehavior::StartNewGame));
        assert!(context.is_action_allowed(&PlayerBehavior::RequestPhaseData));

        // 허용되지 않은 행동
        assert!(!context.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: Uuid::new_v4()
        }));
    }

    #[test]
    fn test_current_game_context_variant_matching() {
        let mut context = CurrentGameContext::new();

        // SelectEvent 템플릿을 허용 목록에 추가
        context.set_allowed_actions(vec![PlayerBehavior::SelectEvent {
            event_id: Uuid::nil(), // 템플릿 (모든 UUID 허용)
        }]);

        // 다른 UUID를 가진 SelectEvent도 허용되어야 함 (Variant만 비교)
        let different_uuid = Uuid::new_v4();
        assert!(context.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: different_uuid
        }));

        // 다른 Variant는 허용되지 않음
        assert!(!context.is_action_allowed(&PlayerBehavior::StartNewGame));
    }

    #[test]
    fn test_current_game_context_clear() {
        let mut context = CurrentGameContext::new();

        context.set_allowed_actions(vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ]);

        assert_eq!(context.allowed_actions.len(), 2);

        context.clear();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    // #[test]
    // fn test_current_game_context_shop_actions() {
    //     let mut context = CurrentGameContext::new();

    //     context.set_allowed_actions(vec![
    //         PlayerBehavior::PurchaseItem {
    //             item_uuid: Uuid::nil(),
    //         },
    //         PlayerBehavior::RerollShop,
    //         PlayerBehavior::ExitShop,
    //     ]);

    //     // 모든 상점 행동 허용됨
    //     assert!(context.is_action_allowed(&PlayerBehavior::PurchaseItem {
    //         item_uuid: Uuid::new_v4()
    //     }));
    //     assert!(context.is_action_allowed(&PlayerBehavior::RerollShop));
    //     assert!(context.is_action_allowed(&PlayerBehavior::ExitShop));

    //     // 상점 밖 행동은 허용되지 않음
    //     assert!(!context.is_action_allowed(&PlayerBehavior::RequestPhaseData));
    // }
}
