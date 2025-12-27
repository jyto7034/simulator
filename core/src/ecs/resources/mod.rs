use std::collections::HashMap;

use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::behavior::{GameError, PlayerBehavior};
use crate::game::data::bonus_data::BonusMetadata;
use crate::game::data::random_event_data::RandomEventMetadata;
use crate::game::data::shop_data::ShopMetadata;
use crate::game::enums::{GameOption, OrdealType, PhaseType, Side};

pub mod inventory;
pub mod item_slot;
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
    /// 보너스 진입
    InBonus { bonus_uuid: Uuid },
    /// 보너스 수령 완료 (Exit만 가능)
    InBonusClaimed { bonus_uuid: Uuid },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn manhattan(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnitPlacement {
    pub uuid: Uuid,
    pub side: Side,
}

#[derive(Resource, Debug, Clone)]
pub struct Field {
    pub width: u8,
    pub height: u8,
    pub placements: HashMap<Position, UnitPlacement>,
    pub unit_positions: HashMap<Uuid, Position>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new(3, 3)
    }
}

impl Field {
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            placements: HashMap::new(),
            unit_positions: HashMap::new(),
        }
    }

    pub fn place(&mut self, uuid: Uuid, side: Side, pos: Position) -> Result<(), GameError> {
        if pos.x < 0 || pos.x >= self.width as i32 || pos.y < 0 || pos.y >= self.height as i32 {
            return Err(GameError::OutOfBounds);
        }

        if self.placements.contains_key(&pos) {
            return Err(GameError::PositionOccupied);
        }

        if self.unit_positions.contains_key(&uuid) {
            return Err(GameError::UnitAlreadyPlaced);
        }

        self.placements.insert(pos, UnitPlacement { uuid, side });
        self.unit_positions.insert(uuid, pos);
        Ok(())
    }

    pub fn remove(&mut self, unit_uuid: Uuid) -> Option<Position> {
        if let Some(pos) = self.unit_positions.remove(&unit_uuid) {
            self.placements.remove(&pos);
            Some(pos)
        } else {
            None
        }
    }

    pub fn move_unit(&mut self, unit_uuid: Uuid, new_pos: Position) -> Result<(), GameError> {
        if new_pos.x < 0
            || new_pos.x >= self.width as i32
            || new_pos.y < 0
            || new_pos.y >= self.height as i32
        {
            return Err(GameError::OutOfBounds);
        }

        if self.placements.contains_key(&new_pos) {
            return Err(GameError::PositionOccupied);
        }

        let old_pos = self
            .unit_positions
            .get(&unit_uuid)
            .ok_or(GameError::UnitNotFound)?;

        let placement = self
            .placements
            .remove(old_pos)
            .ok_or(GameError::UnitNotFound)?;

        self.placements.insert(new_pos, placement);
        self.unit_positions.insert(unit_uuid, new_pos);
        Ok(())
    }

    pub fn get_position(&self, unit_uuid: Uuid) -> Option<Position> {
        self.unit_positions.get(&unit_uuid).copied()
    }

    pub fn get_unit_at(&self, pos: Position) -> Option<Uuid> {
        self.placements.get(&pos).map(|p| p.uuid)
    }

    pub fn get_placement_at(&self, pos: Position) -> Option<&UnitPlacement> {
        self.placements.get(&pos)
    }

    pub fn find_nearest_enemy(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.unit_positions.get(&from_uuid)?;
        self.find_nearest(from_pos, from_side, true)
    }

    pub fn find_nearest_ally(&self, from_uuid: Uuid, from_side: Side) -> Option<Uuid> {
        let from_pos = self.unit_positions.get(&from_uuid)?;
        self.find_nearest(from_pos, from_side, false)
    }

    fn find_nearest(&self, from_pos: &Position, from_side: Side, find_enemy: bool) -> Option<Uuid> {
        let mut nearest: Option<(Uuid, i32)> = None;

        for (pos, placement) in &self.placements {
            let is_enemy = placement.side != from_side;

            if is_enemy != find_enemy {
                continue;
            }

            let distance = from_pos.manhattan(pos);

            match nearest {
                None => nearest = Some((placement.uuid, distance)),
                Some((_best_uuid, best_dist)) if distance < best_dist => {
                    nearest = Some((placement.uuid, distance));
                }
                Some((best_uuid, best_dist))
                    if distance == best_dist
                        && placement.uuid.as_bytes() < best_uuid.as_bytes() =>
                {
                    // 동률이면 더 작은 UUID를 선택 (결정성)
                    nearest = Some((placement.uuid, distance));
                }
                _ => {}
            }
        }

        nearest.map(|(uuid, _)| uuid)
    }

    pub fn get_units_by_side(&self, side: Side) -> Vec<Uuid> {
        self.placements
            .values()
            .filter(|p| p.side == side)
            .map(|p| p.uuid)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }

    pub fn count_by_side(&self, side: Side) -> usize {
        self.placements.values().filter(|p| p.side == side).count()
    }

    pub fn get_positions_by_side(&self, side: Side) -> HashMap<Uuid, Position> {
        self.placements
            .iter()
            .filter(|(_, p)| p.side == side)
            .map(|(pos, p)| (p.uuid, *pos))
            .collect()
    }

    pub fn clear(&mut self) {
        self.placements.clear();
        self.unit_positions.clear();
    }

    pub fn clear_side(&mut self, side: Side) {
        let uuids_to_remove: Vec<Uuid> = self
            .placements
            .iter()
            .filter(|(_, p)| p.side == side)
            .map(|(_, p)| p.uuid)
            .collect();

        for uuid in uuids_to_remove {
            self.remove(uuid);
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QliphothLevel {
    Stable,   // 안정 (10-8) - 정상 운영
    Caution,  // 주의 (7-5) - 경보 발령
    Critical, // 위기 (4-2) - 비상 사태
    Meltdown, // 붕괴 (1-0) - 시설 붕괴
}

/// 클리포드
#[derive(Resource, Debug, Clone)]
pub struct Qliphoth {
    pub level: QliphothLevel,
    pub amount: u32,
}

impl Qliphoth {
    pub fn new() -> Qliphoth {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        Self {
            level: QliphothLevel::Stable,
            amount: thresholds.stable_min,
        }
    }

    pub fn level(&self) -> QliphothLevel {
        self.level
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    /// 클리포트 값 증가
    pub fn increase(&mut self, amount: u32) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.amount = (self.amount + amount).min(thresholds.stable_min);
        self.update_level();
    }

    /// 클리포트 값 감소
    pub fn decrease(&mut self, amount: u32) {
        self.amount = self.amount.saturating_sub(amount);
        self.update_level();
    }

    /// 클리포트 값 설정 (테스트용)
    pub fn set_amount(&mut self, amount: u32) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.amount = amount.min(thresholds.stable_min);
        self.update_level();
    }

    /// 현재 클리포트 양에 따라 레벨 업데이트
    fn update_level(&mut self) {
        use crate::config::balance;
        let thresholds = balance::qliphoth_thresholds();

        self.level = match self.amount {
            x if x >= thresholds.stable_max => QliphothLevel::Stable,
            x if x >= thresholds.caution_max => QliphothLevel::Caution,
            x if x >= thresholds.critical_max => QliphothLevel::Critical,
            _ => QliphothLevel::Meltdown,
        };
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
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_shop_mut(&mut self) -> Result<&mut ShopMetadata, GameError> {
        match &mut self.event {
            GameOption::Shop { shop } => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_bonus(&self) -> Result<&BonusMetadata, GameError> {
        match &self.event {
            GameOption::Bonus { bonus } => Ok(bonus),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_random_event(&self) -> Result<&RandomEventMetadata, GameError> {
        match &self.event {
            GameOption::Random { event } => Ok(event),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_suppression(&self) -> Result<(&str, Uuid), GameError> {
        match &self.event {
            GameOption::SuppressAbnormality {
                abnormality_id,
                uuid,
                ..
            } => Ok((abnormality_id.as_str(), *uuid)),
            _ => Err(GameError::EventTypeMismatch),
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
pub struct ActionValidator {
    /// 현재 허용된 행동 목록
    pub allowed_actions: Vec<PlayerBehavior>,
}

impl ActionValidator {
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
    use crate::game::enums::RiskLevel;

    use super::*;
    use {OrdealType, PhaseType};

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
        // Given: NotStarted → WaitingPhaseRequest
        let state = GameState::WaitingPhaseRequest;
        assert_eq!(state, GameState::WaitingPhaseRequest);

        // Given: WaitingPhaseRequest → SelectingEvent
        let state = GameState::SelectingEvent;
        assert_eq!(state, GameState::SelectingEvent);

        // Given: SelectingEvent → InShop
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
        let context = ActionValidator::new();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    #[test]
    fn test_current_game_context_default() {
        let context = ActionValidator::default();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    #[test]
    fn test_current_game_context_set_allowed_actions() {
        let mut context = ActionValidator::new();

        let actions = vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ];

        context.set_allowed_actions(actions.clone());
        assert_eq!(context.allowed_actions.len(), 2);
    }

    #[test]
    fn test_current_game_context_is_action_allowed() {
        let mut context = ActionValidator::new();

        let actions = vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ];

        context.set_allowed_actions(actions);

        // Then: 허용된 행동
        assert!(context.is_action_allowed(&PlayerBehavior::StartNewGame));
        assert!(context.is_action_allowed(&PlayerBehavior::RequestPhaseData));

        // Then: 허용되지 않은 행동
        assert!(!context.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: Uuid::new_v4()
        }));
    }

    #[test]
    fn test_current_game_context_variant_matching() {
        let mut context = ActionValidator::new();

        // Given: SelectEvent 템플릿을 허용 목록에 추가
        context.set_allowed_actions(vec![PlayerBehavior::SelectEvent {
            event_id: Uuid::nil(), // NOTE: 템플릿 (모든 UUID 허용)
        }]);

        // Then: 다른 UUID를 가진 SelectEvent도 허용되어야 함 (Variant만 비교)
        let different_uuid = Uuid::new_v4();
        assert!(context.is_action_allowed(&PlayerBehavior::SelectEvent {
            event_id: different_uuid
        }));

        // Then: 다른 Variant는 허용되지 않음
        assert!(!context.is_action_allowed(&PlayerBehavior::StartNewGame));
    }

    #[test]
    fn test_current_game_context_clear() {
        let mut context = ActionValidator::new();

        context.set_allowed_actions(vec![
            PlayerBehavior::StartNewGame,
            PlayerBehavior::RequestPhaseData,
        ]);

        assert_eq!(context.allowed_actions.len(), 2);

        context.clear();
        assert_eq!(context.allowed_actions.len(), 0);
    }

    #[test]
    fn test_field_move_unit_success() {
        use crate::game::enums::Side;

        let unit_uuid = Uuid::new_v4();
        let mut field = Field::new(3, 3);
        field
            .place(unit_uuid, Side::Player, Position::new(0, 0))
            .unwrap();

        field.move_unit(unit_uuid, Position::new(2, 1)).unwrap();

        assert_eq!(field.get_position(unit_uuid), Some(Position::new(2, 1)));
        assert_eq!(field.get_unit_at(Position::new(0, 0)), None);
        assert_eq!(field.get_unit_at(Position::new(2, 1)), Some(unit_uuid));
    }

    #[test]
    fn test_field_move_unit_rejects_occupied_destination() {
        use crate::game::enums::Side;

        let unit1 = Uuid::new_v4();
        let unit2 = Uuid::new_v4();
        let mut field = Field::new(3, 3);
        field
            .place(unit1, Side::Player, Position::new(0, 0))
            .unwrap();
        field
            .place(unit2, Side::Player, Position::new(1, 1))
            .unwrap();

        let err = field.move_unit(unit1, Position::new(1, 1)).unwrap_err();
        assert!(matches!(err, GameError::PositionOccupied));
    }

    #[test]
    fn test_field_move_unit_rejects_out_of_bounds() {
        use crate::game::enums::Side;

        let unit_uuid = Uuid::new_v4();
        let mut field = Field::new(3, 3);
        field
            .place(unit_uuid, Side::Player, Position::new(0, 0))
            .unwrap();

        let err = field.move_unit(unit_uuid, Position::new(99, 0)).unwrap_err();
        assert!(matches!(err, GameError::OutOfBounds));
    }
}
