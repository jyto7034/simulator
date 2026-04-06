use std::collections::{HashMap, HashSet};

use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::data::shop_data::{ShopMetadata, ShopType};
use crate::game::enums::{
    BonusEventOption, GameOption, OrdealType, PhaseType, RewardMode, ShopEventOption, Side,
    SuppressionOption,
};
use crate::game::{
    battle::{timeline::Timeline, types::BattleWinner},
    behavior::{ActionKind, GameError, PlayerBehavior},
};

pub mod inventory;
pub mod item_slot;
pub use inventory::*;

/// 게임의 명시적인 상태
///
/// 현재 게임이 어떤 단계에 있는지 명확하게 표현
/// ActionScheduler가 이 상태를 보고 allowed_actions를 결정함
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub enum GameState {
    /// 게임 시작 전
    #[default]
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
    /// 진압 전투 리플레이 진행 중
    InSuppressionReplay { abnormality_uuid: Uuid },
    /// 시련 전투 진행 중
    InBattle { battle_uuid: Uuid },
    /// 게임 종료
    GameOver,
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

    pub fn chebyshev(&self, other: &Position) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
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

            let distance = from_pos.chebyshev(pos);

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

impl Default for Qliphoth {
    fn default() -> Self {
        Self::new()
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

#[derive(Debug, Clone)]
pub struct ShopSessionState {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
    pub shop_type: ShopType,
    pub can_reroll: bool,
    pub visible_items: Vec<Uuid>,
    pub hidden_items: Vec<Uuid>,
}

impl ShopSessionState {
    pub fn remove_visible_item(&mut self, uuid: Uuid) -> Result<(), GameError> {
        let pos = self
            .visible_items
            .iter()
            .position(|item| *item == uuid)
            .ok_or(GameError::ShopItemNotFound)?;
        self.visible_items.remove(pos);
        Ok(())
    }

    pub fn reroll_items(&mut self) {
        std::mem::swap(&mut self.hidden_items, &mut self.visible_items);
    }
}

impl From<&ShopMetadata> for ShopSessionState {
    fn from(value: &ShopMetadata) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            shop_type: value.shop_type,
            can_reroll: value.can_reroll,
            visible_items: value.visible_items.clone(),
            hidden_items: value.hidden_items.clone(),
        }
    }
}

impl From<ShopMetadata> for ShopSessionState {
    fn from(value: ShopMetadata) -> Self {
        Self::from(&value)
    }
}

impl From<&ShopEventOption> for ShopSessionState {
    fn from(value: &ShopEventOption) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            uuid: value.uuid,
            shop_type: value.shop_type,
            can_reroll: value.can_reroll,
            visible_items: value.visible_items.clone(),
            hidden_items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RewardSessionState {
    pub stage_uuid: Uuid,
    pub mode: RewardMode,
    pub rewards: Vec<BonusEventOption>,
    pub selected_reward_uuid: Option<Uuid>,
}

impl RewardSessionState {
    pub fn get_selected_reward(&self) -> Option<&BonusEventOption> {
        self.selected_reward_uuid
            .and_then(|uuid| self.rewards.iter().find(|reward| reward.uuid == uuid))
    }
}

#[derive(Debug, Clone)]
pub struct SuppressionBattleState {
    pub abnormality_id: String,
    pub encounter_id: String,
    pub abnormality_uuid: Uuid,
    pub winner: BattleWinner,
    pub timeline: Timeline,
    pub reward_mode: RewardMode,
    pub rewards: Vec<BonusEventOption>,
}

#[derive(Debug, Clone)]
pub enum SelectedEventState {
    Shop(ShopSessionState),
    Reward(RewardSessionState),
    Suppression(SuppressionOption),
    SuppressionBattle(SuppressionBattleState),
}

#[derive(Resource)]
pub struct SelectedEvent {
    pub event: SelectedEventState,
}

impl SelectedEvent {
    pub fn new(event: SelectedEventState) -> Self {
        Self { event }
    }

    pub fn as_shop(&self) -> Result<&ShopSessionState, GameError> {
        match &self.event {
            SelectedEventState::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_shop_mut(&mut self) -> Result<&mut ShopSessionState, GameError> {
        match &mut self.event {
            SelectedEventState::Shop(shop) => Ok(shop),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward(&self) -> Result<&RewardSessionState, GameError> {
        match &self.event {
            SelectedEventState::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_reward_mut(&mut self) -> Result<&mut RewardSessionState, GameError> {
        match &mut self.event {
            SelectedEventState::Reward(reward) => Ok(reward),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_suppression(&self) -> Result<(&str, &str, Uuid), GameError> {
        match &self.event {
            SelectedEventState::Suppression(option) => Ok((
                option.abnormality_id.as_str(),
                option.encounter_id.as_str(),
                option.uuid,
            )),
            _ => Err(GameError::EventTypeMismatch),
        }
    }

    pub fn as_suppression_battle(&self) -> Result<&SuppressionBattleState, GameError> {
        match &self.event {
            SelectedEventState::SuppressionBattle(battle) => Ok(battle),
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
/// 현재 상태에서 허용되는 액션 capability를 저장
/// 실제 payload 유효성은 각 액션 validator가 별도로 검사한다.
#[derive(Resource, Default)]
pub struct ActionValidator {
    /// 현재 허용된 액션 종류
    allowed_actions: HashSet<ActionKind>,
}

impl ActionValidator {
    pub fn new() -> Self {
        Self {
            allowed_actions: HashSet::new(),
        }
    }

    /// 허용된 행동 설정
    pub fn set_allowed_actions(&mut self, actions: Vec<ActionKind>) {
        self.allowed_actions = actions.into_iter().collect();
    }

    /// 특정 액션 종류가 허용되는지 확인
    pub fn is_kind_allowed(&self, action: ActionKind) -> bool {
        self.allowed_actions.contains(&action)
    }

    /// 특정 행동이 허용되는지 확인
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.is_kind_allowed(action.kind())
    }

    /// 현재 허용된 액션 종류를 정렬된 형태로 반환
    pub fn allowed_actions(&self) -> Vec<ActionKind> {
        let mut actions = self.allowed_actions.iter().copied().collect::<Vec<_>>();
        actions.sort();
        actions
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

    // ============================================================
    // CurrentPhaseEvents Tests
    // ============================================================

    #[test]
    fn test_current_phase_events_add_and_get() {
        let mut events = CurrentPhaseEvents::new();
        let uuid = Uuid::new_v4();

        let option = GameOption::SuppressAbnormality {
            abnormality_id: "F-01-02".to_string(),
            encounter_id: "encounter_1".to_string(),
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
            encounter_id: "encounter_1".to_string(),
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
                encounter_id: format!("encounter_{i}"),
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
    fn test_current_game_context_set_allowed_actions() {
        let mut context = ActionValidator::new();

        let actions = vec![ActionKind::StartNewGame, ActionKind::RequestPhaseData];

        context.set_allowed_actions(actions.clone());
        assert_eq!(context.allowed_actions().len(), 2);
    }

    #[test]
    fn test_current_game_context_is_action_allowed() {
        let mut context = ActionValidator::new();

        let actions = vec![ActionKind::StartNewGame, ActionKind::RequestPhaseData];

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

        // Given: SelectEvent capability를 허용 목록에 추가
        context.set_allowed_actions(vec![ActionKind::SelectEvent]);

        // Then: 다른 UUID를 가진 SelectEvent도 허용되어야 함
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

        context.set_allowed_actions(vec![ActionKind::StartNewGame, ActionKind::RequestPhaseData]);

        assert_eq!(context.allowed_actions().len(), 2);

        context.clear();
        assert_eq!(context.allowed_actions().len(), 0);
    }

    // ============================================================
    // Field Tests
    // ============================================================

    #[test]
    fn field_place_rejects_duplicate_position_and_duplicate_unit() {
        let mut field = Field::new(3, 3);
        let unit_a = Uuid::from_u128(1);
        let unit_b = Uuid::from_u128(2);

        field
            .place(unit_a, Side::Player, Position::new(0, 0))
            .unwrap();

        let err = field
            .place(unit_b, Side::Player, Position::new(0, 0))
            .unwrap_err();
        assert!(matches!(err, GameError::PositionOccupied));

        let err = field
            .place(unit_a, Side::Player, Position::new(1, 1))
            .unwrap_err();
        assert!(matches!(err, GameError::UnitAlreadyPlaced));
    }

    #[test]
    fn field_remove_clears_both_indices() {
        let mut field = Field::new(3, 3);
        let unit = Uuid::from_u128(1);
        field
            .place(unit, Side::Player, Position::new(1, 1))
            .unwrap();

        assert_eq!(field.get_position(unit), Some(Position::new(1, 1)));
        assert_eq!(field.get_unit_at(Position::new(1, 1)), Some(unit));

        let removed = field.remove(unit);
        assert_eq!(removed, Some(Position::new(1, 1)));
        assert_eq!(field.get_position(unit), None);
        assert_eq!(field.get_unit_at(Position::new(1, 1)), None);
    }

    #[test]
    fn field_find_nearest_enemy_tie_breaks_by_uuid() {
        let mut field = Field::new(5, 5);
        let from = Uuid::from_u128(10);
        let enemy_small = Uuid::from_u128(1);
        let enemy_large = Uuid::from_u128(2);
        assert!(enemy_small.as_bytes() < enemy_large.as_bytes());

        field
            .place(from, Side::Player, Position::new(2, 2))
            .unwrap();
        // Both enemies are at chebyshev distance 1.
        field
            .place(enemy_small, Side::Opponent, Position::new(1, 2))
            .unwrap();
        field
            .place(enemy_large, Side::Opponent, Position::new(2, 1))
            .unwrap();

        let nearest = field.find_nearest_enemy(from, Side::Player).unwrap();
        assert_eq!(nearest, enemy_small);
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

        let err = field
            .move_unit(unit_uuid, Position::new(99, 0))
            .unwrap_err();
        assert!(matches!(err, GameError::OutOfBounds));
    }
}
