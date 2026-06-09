use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    game::employee::ActiveConsumableModifier,
    game::resources::{
        EquipItemResultDto, InventoryDiffDto, Position, RunFailureReason, UnequipItemResultDto,
    },
    game::{
        ability::{
            FocusPermissions, SkillActivationMode, SkillAreaAnchorSource, SkillAreaTickPolicy,
            SkillAreaTracking, SkillHitTargetFilter, SkillId, SkillKind, SkillTarget,
            SkillTileAreaOrigin,
        },
        battle::{
            ids::UnitInstanceId,
            tile_range::{FacingDirection, TileRangePattern},
            timeline::{SkillCastTarget, TimelineEntry},
            types::BattleWinner,
        },
        combat_preview::{CombatMissionVariant, CombatNodeType, CombatPreview},
        data::equipment_data::EquipmentType,
        data::skill_fragment_data::{SkillFragmentCompatibilityFailureCode, SkillFragmentId},
        employee::StarterEmployeeCandidate,
        enums::{RewardMode, ShopEventOption},
        map::{
            HeadquartersContactOption, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload,
            MapViewDto, MedicalTreatmentKind, NodeSession, SupportNodeMode, SupportNodeType,
        },
        reward::RewardOption,
        skill_fragment::{SkillFragmentProgress, SkillFragmentResearchDelivery},
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutcomeSummary {
    pub node_id: MapNodeId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub mission_success: bool,
    pub combat: Option<CombatOutcomeSummary>,
    pub employee_changes: Vec<NodeOutcomeEmployeeChange>,
    pub inventory_diff: InventoryDiffDto,
    pub research_deliveries: Vec<SkillFragmentResearchDelivery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatOutcomeSummary {
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub winner: BattleWinner,
    pub retreated: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbnormalityAttemptDto {
    pub node_id: MapNodeId,
    pub max_attempts: u8,
    pub attempts_started: u8,
    pub remaining_attempts: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutcomeEmployeeChange {
    pub employee_uuid: Uuid,
    pub survived: bool,
    pub became_incapacitated: bool,
    pub run_hp_before: u32,
    pub run_hp_after: u32,
    pub trauma_before: u32,
    pub trauma_after: u32,
    pub experience_before: u32,
    pub experience_after: u32,
    pub was_alive: bool,
    pub is_alive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterSlotDto {
    pub slot: usize,
    pub unit_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceOptionsDto {
    pub items: Vec<MaintenanceItemPreviewDto>,
    pub materials: Vec<MaintenanceMaterialAmountDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceItemPreviewDto {
    pub target_id: String,
    pub target_kind: MaintenanceTargetKind,
    pub equipment_type: Option<EquipmentType>,
    pub display_name: String,
    pub source: MaintenanceSourceDto,
    pub operations: MaintenanceOperationsDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MaintenanceTargetKind {
    SkillFragment,
    Equipment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSourceDto {
    #[serde(rename = "type")]
    pub source_type: MaintenanceSourceKind,
    pub employee_uuid: Option<Uuid>,
    pub slot_kind: Option<MaintenanceSlotKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceSourceKind {
    Bag,
    Equipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceSlotKind {
    SkillFragment,
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceOperationsDto {
    pub dismantle: MaintenanceOperationPreviewDto,
    pub enhance: MaintenanceOperationPreviewDto,
    pub awaken: MaintenanceOperationPreviewDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceOperationPreviewDto {
    pub can_execute: bool,
    pub disabled_reason: Option<String>,
    pub costs: Vec<MaintenanceMaterialAmountDto>,
    pub gains: Vec<MaintenanceMaterialAmountDto>,
    pub before: MaintenanceTargetStatePreviewDto,
    pub after: Option<MaintenanceTargetStatePreviewDto>,
    pub requires_confirm: bool,
    pub will_unequip: bool,
    pub warnings: Vec<MaintenanceWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceMaterialAmountDto {
    pub material_id: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaintenanceTargetStatePreviewDto {
    pub stack_count: Option<u32>,
    pub enhancement_level: Option<u8>,
    pub upgrade_level: Option<u8>,
    pub awakening_progress: Option<u32>,
    pub awakening_available: Option<bool>,
    pub awakened: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceWarning {
    EquippedItemWillBeUnequipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattlePlaybackSpeed {
    X0_5,
    X1,
    X2,
    X3,
}

impl BattlePlaybackSpeed {
    pub fn ratio(self) -> (u64, u64) {
        match self {
            Self::X0_5 => (1, 2),
            Self::X1 => (1, 1),
            Self::X2 => (2, 1),
            Self::X3 => (3, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePlaybackState {
    pub paused: bool,
    pub speed: BattlePlaybackSpeed,
}

impl Default for BattlePlaybackState {
    fn default() -> Self {
        Self {
            paused: false,
            speed: BattlePlaybackSpeed::X1,
        }
    }
}

/// 상태 게이트에서 사용하는 payload-less 액션 capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionKind {
    StartNewGame,
    SelectStarterEmployees,
    UnEquipItem,
    EquipItem,
    UseConsumableItem,
    EquipSkillFragment,
    UnequipSkillFragment,
    UpgradeSkillFragment,
    AwakenSkillFragment,
    DismantleSkillFragment,
    DismantleEquipment,
    EnhanceEquipment,
    MoveRosterUnit,
    RequestMapData,
    SelectMapNode,
    ConfirmEnterNode,
    CancelSelectedNode,
    CompleteNode,
    ChooseSupport,
    SelectSupportTarget,
    SelectMedicalTreatment,
    RecruitEmployee,
    RequestEmergencySupplies,
    OpenHeadquartersShop,
    SelectReward,
    PurchaseItem,
    SellItem,
    RerollShop,
    ExitShop,
    ClaimReward,
    ExitReward,
    CompleteCombatResult,
    RequestBattleState,
    DeployUnit,
    WithdrawUnit,
    ActivateSkill,
    RetreatBattle,
    PauseBattle,
    ResumeBattle,
    SetBattleSpeed,
}

/// GameServer에서 GameCore로 전달되는 플레이어 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerBehavior {
    // ============================================================
    // 게임 관련 행동
    // ============================================================
    /// 새 게임 시작
    StartNewGame,
    /// 시작 후보 직원 중 이번 런에 투입할 3명을 선택
    SelectStarterEmployees {
        candidate_ids: Vec<String>,
    },
    // 아이템 장착 해제
    UnEquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    /// 아이템 장착
    EquipItem {
        item_uuid: Uuid,
        target_unit: Uuid,
    },
    /// Safezone Item Use 장면에서 직원에게 섭취 아이템 사용
    UseConsumableItem {
        item_uuid: Uuid,
        target_employee_uuid: Uuid,
    },
    /// 직원에게 런 소유 스킬 파편 장착
    EquipSkillFragment {
        employee_uuid: Uuid,
        fragment_id: SkillFragmentId,
    },
    /// 직원에게 장착된 스킬 파편 해제
    UnequipSkillFragment {
        employee_uuid: Uuid,
        fragment_id: SkillFragmentId,
    },
    /// 파편 가루를 소모해 대상 파편의 강화/개화 진행도를 올림
    UpgradeSkillFragment {
        target_fragment_id: SkillFragmentId,
    },
    /// 개화 가능하거나 조기 개화 파편 가루 비용을 지불할 수 있는 대상 파편을 개화
    AwakenSkillFragment {
        target_fragment_id: SkillFragmentId,
    },
    /// Maintenance에서 스킬 파편을 분쇄해 파편 가루로 전환
    DismantleSkillFragment {
        fragment_id: SkillFragmentId,
    },
    /// Maintenance에서 완성 장비를 분해해 장비 재료로 전환
    DismantleEquipment {
        item_uuid: Uuid,
    },
    /// Maintenance에서 장비 재료를 소비해 소유 장비 인스턴스를 강화
    EnhanceEquipment {
        item_uuid: Uuid,
    },
    /// 런 중 직원 명단 표시 순서 이동
    MoveRosterUnit {
        target_unit_uuid: Uuid,
        dest_slot: usize,
        swap_with_unit_uuid: Option<Uuid>,
    },
    /// 현재 런 맵 상태 요청
    RequestMapData,
    /// 사용 가능한 맵 노드 선택
    SelectMapNode {
        node_id: MapNodeId,
    },
    /// 선택한 맵 노드에 실제로 진입
    ConfirmEnterNode,
    /// 선택한 맵 노드 프리뷰를 취소하고 맵으로 복귀
    CancelSelectedNode,
    /// 현재 노드 처리 완료
    CompleteNode,
    /// 선택형 지원 노드에서 지원 효과 선택
    ChooseSupport {
        support_type: SupportNodeType,
    },
    /// 대상 선택형 지원 노드에서 지원 대상 직원 선택
    SelectSupportTarget {
        employee_uuid: Uuid,
    },
    /// Medical 지원 노드에서 의료 처치 방식 선택
    SelectMedicalTreatment {
        treatment: MedicalTreatmentKind,
    },
    /// 본사 연락 노드에서 후보 직원 1명을 채용하고 노드를 완료
    RecruitEmployee {
        candidate_id: String,
    },
    /// 본사 연락 노드에서 긴급 보급을 요청하고 노드를 완료
    RequestEmergencySupplies,
    /// 본사 연락 노드에서 본사 보급 상점을 열어 해당 노드의 행동권을 소비
    OpenHeadquartersShop,
    /// 선택형 보상 목록에서 보상 선택
    SelectReward {
        reward_id: Uuid,
    },
    // ============================================================
    // 상점 관련 행동
    // ============================================================
    /// 아이템 구매
    PurchaseItem {
        item_uuid: Uuid,
    },
    /// 아이템 판매
    SellItem {
        item_uuid: Uuid,
    },
    /// 상점 리롤 (새로운 아이템으로 교체)
    RerollShop,
    /// 상점 나가기
    ExitShop,
    // ============================================================
    // 보상 세션 관련 행동
    // ============================================================
    /// 보상 수령
    ClaimReward,
    /// 보상 화면 나가기
    ExitReward,
    /// 전투 결과 확인 완료
    CompleteCombatResult,
    /// 실시간 전투 상태 요청
    RequestBattleState {
        #[serde(default)]
        since_seq: Option<u64>,
    },
    /// 실시간 DefenseRoute 전투에서 직원을 지정 위치에 배치
    DeployUnit {
        employee_uuid: Uuid,
        position: Position,
        facing: FacingDirection,
    },
    /// 실시간 DefenseRoute 전투에서 배치된 직원을 후퇴시키고 재배치 대기 상태로 전환
    WithdrawUnit {
        employee_uuid: Uuid,
    },
    /// 실시간 DefenseRoute 전투에서 배치된 직원의 수동 스킬을 발동
    ActivateSkill {
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        #[serde(default)]
        target: Option<SkillCastTarget>,
    },
    /// 실시간 비보스 전투에서 후퇴하고 현재 노드를 실패 처리
    RetreatBattle,
    /// 실시간 DefenseRoute 전투 시뮬레이션 시간을 정지
    PauseBattle,
    /// 실시간 DefenseRoute 전투 시뮬레이션 시간을 재생
    ResumeBattle,
    /// 실시간 DefenseRoute 전투 시뮬레이션 배속 변경
    SetBattleSpeed {
        speed: BattlePlaybackSpeed,
    },
}

impl PlayerBehavior {
    pub fn kind(&self) -> ActionKind {
        match self {
            PlayerBehavior::StartNewGame => ActionKind::StartNewGame,
            PlayerBehavior::SelectStarterEmployees { .. } => ActionKind::SelectStarterEmployees,
            PlayerBehavior::UnEquipItem { .. } => ActionKind::UnEquipItem,
            PlayerBehavior::EquipItem { .. } => ActionKind::EquipItem,
            PlayerBehavior::UseConsumableItem { .. } => ActionKind::UseConsumableItem,
            PlayerBehavior::EquipSkillFragment { .. } => ActionKind::EquipSkillFragment,
            PlayerBehavior::UnequipSkillFragment { .. } => ActionKind::UnequipSkillFragment,
            PlayerBehavior::UpgradeSkillFragment { .. } => ActionKind::UpgradeSkillFragment,
            PlayerBehavior::AwakenSkillFragment { .. } => ActionKind::AwakenSkillFragment,
            PlayerBehavior::DismantleSkillFragment { .. } => ActionKind::DismantleSkillFragment,
            PlayerBehavior::DismantleEquipment { .. } => ActionKind::DismantleEquipment,
            PlayerBehavior::EnhanceEquipment { .. } => ActionKind::EnhanceEquipment,
            PlayerBehavior::MoveRosterUnit { .. } => ActionKind::MoveRosterUnit,
            PlayerBehavior::RequestMapData => ActionKind::RequestMapData,
            PlayerBehavior::SelectMapNode { .. } => ActionKind::SelectMapNode,
            PlayerBehavior::ConfirmEnterNode => ActionKind::ConfirmEnterNode,
            PlayerBehavior::CancelSelectedNode => ActionKind::CancelSelectedNode,
            PlayerBehavior::CompleteNode => ActionKind::CompleteNode,
            PlayerBehavior::ChooseSupport { .. } => ActionKind::ChooseSupport,
            PlayerBehavior::SelectSupportTarget { .. } => ActionKind::SelectSupportTarget,
            PlayerBehavior::SelectMedicalTreatment { .. } => ActionKind::SelectMedicalTreatment,
            PlayerBehavior::RecruitEmployee { .. } => ActionKind::RecruitEmployee,
            PlayerBehavior::RequestEmergencySupplies => ActionKind::RequestEmergencySupplies,
            PlayerBehavior::OpenHeadquartersShop => ActionKind::OpenHeadquartersShop,
            PlayerBehavior::SelectReward { .. } => ActionKind::SelectReward,
            PlayerBehavior::PurchaseItem { .. } => ActionKind::PurchaseItem,
            PlayerBehavior::SellItem { .. } => ActionKind::SellItem,
            PlayerBehavior::RerollShop => ActionKind::RerollShop,
            PlayerBehavior::ExitShop => ActionKind::ExitShop,
            PlayerBehavior::ClaimReward => ActionKind::ClaimReward,
            PlayerBehavior::ExitReward => ActionKind::ExitReward,
            PlayerBehavior::CompleteCombatResult => ActionKind::CompleteCombatResult,
            PlayerBehavior::RequestBattleState { .. } => ActionKind::RequestBattleState,
            PlayerBehavior::DeployUnit { .. } => ActionKind::DeployUnit,
            PlayerBehavior::WithdrawUnit { .. } => ActionKind::WithdrawUnit,
            PlayerBehavior::ActivateSkill { .. } => ActionKind::ActivateSkill,
            PlayerBehavior::RetreatBattle => ActionKind::RetreatBattle,
            PlayerBehavior::PauseBattle => ActionKind::PauseBattle,
            PlayerBehavior::ResumeBattle => ActionKind::ResumeBattle,
            PlayerBehavior::SetBattleSpeed { .. } => ActionKind::SetBattleSpeed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleDeploymentDto {
    pub battle_time_ms: u64,
    /// Player-facing "공간 안정화치" value. Rust member name stays cost during the
    /// first Unity/server contract migration.
    pub current_cost: u32,
    /// Player-facing maximum "공간 안정화치".
    pub max_cost: u32,
    /// Stabilization required for a first deployment.
    pub base_deploy_cost: u32,
    /// Automatic stabilization recovery rate.
    pub cost_per_second: u32,
    pub unit_deploy_costs: Vec<LiveBattleUnitDeployCostDto>,
    pub deployed_units: Vec<LiveBattleDeployedUnitDto>,
    pub redeploying_units: Vec<LiveBattleRedeployUnitDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleUnitDeployCostDto {
    pub employee_uuid: Uuid,
    pub base_deploy_cost: u32,
    pub effective_deploy_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleDeployedUnitDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub facing: FacingDirection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_readiness: Option<LiveBattleSkillReadinessDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleRedeployUnitDto {
    pub employee_uuid: Uuid,
    pub ready_at_ms: u64,
    pub deploy_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSkillReadinessDto {
    pub skill_id: Option<SkillId>,
    pub activation_mode: SkillActivationMode,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub manual_activation_allowed: bool,
    #[serde(default)]
    pub target_required: bool,
    #[serde(default)]
    pub target_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_activate_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_block_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogDto {
    pub version: u32,
    pub range_source_of_truth: String,
    pub skills: Vec<SkillCatalogSkillDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogSkillDto {
    pub skill_id: SkillId,
    pub display_name: String,
    pub kind: SkillKind,
    pub focus_time_ms: u32,
    pub focus_permissions: FocusPermissions,
    pub cast_target: Option<SkillCatalogCastTargetDto>,
    pub steps: Vec<SkillCatalogStepDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogCastTargetDto {
    pub range_units: f32,
    pub target_policy: SkillTarget,
    pub defense_tile_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogStepDto {
    pub step_id: String,
    pub delay_ms: u32,
    pub range_units: f32,
    pub target_policy: SkillTarget,
    pub defense_tile_range: Option<TileRangePattern>,
    pub air_capable: bool,
    pub delivery: SkillCatalogDeliveryKind,
    pub tile_area: Option<SkillCatalogTileAreaDto>,
    pub effects_count: usize,
    pub presentation: crate::game::ability::SkillPresentationDef,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillCatalogDeliveryKind {
    Instant,
    Projectile,
    TileArea,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCatalogTileAreaDto {
    pub anchor: SkillAreaAnchorSource,
    pub tile_origin: SkillTileAreaOrigin,
    pub tracking: SkillAreaTracking,
    pub hit_targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub tick_policy: SkillAreaTickPolicy,
    pub duration_ms: u32,
    pub tick_interval_ms: Option<u32>,
}

/// BehaviorResult 는 변경된 모든 값을 넘길 의무가 있음
/// 예를 들어 PurchaseItem 의 경우
/// 1. 구매된 아이템
/// 2. 아이템은 어디에 저장되는지
/// 3. 남은 자원은 얼마인지
/// 4. 해당 아이템이 어디서 제거되는지,
///
/// 등. 클라이언트는 해당 값들을 반영만 하게끔 해야함.
#[derive(Debug, Serialize, Deserialize)]
pub enum BehaviorResult {
    /// 새 게임 시작
    StartNewGame {
        candidates: Vec<StarterEmployeeCandidate>,
        required_count: usize,
    },
    /// 시작 직원 선택 완료
    StarterEmployeesSelected {
        selected_candidate_ids: Vec<String>,
        employee_uuids: Vec<Uuid>,
        map: MapViewDto,
    },
    /// 현재 런 맵 상태
    MapState {
        map: MapViewDto,
    },
    /// 맵 노드 진입
    NodeEntered {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        payload: MapNodePayload,
        session: NodeSession,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    /// 맵 노드 진입 전 확인/준비 상태
    NodePreview {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        payload: MapNodePayload,
        session: NodeSession,
        map: MapViewDto,
        combat_preview: Option<CombatPreview>,
    },
    /// 맵 노드 완료
    NodeCompleted {
        map: MapViewDto,
        outcome: Option<NodeOutcomeSummary>,
    },
    /// 지원 노드 상태
    SupportState {
        node_id: MapNodeId,
        support_mode: SupportNodeMode,
        support_type: Option<SupportNodeType>,
        choices: Vec<SupportNodeType>,
        selected_support_type: Option<SupportNodeType>,
        target_candidates: Vec<Uuid>,
        selected_employee_uuid: Option<Uuid>,
        selected_medical_treatment: Option<MedicalTreatmentKind>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    /// 정비 노드 상태
    MaintenanceState {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    /// 본사 연락 노드 상태
    HeadquartersContactState {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<StarterEmployeeCandidate>,
        shop_pool_id: Option<String>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    /// 보스 노드 완료로 다음 Act에 진입
    ActComplete {
        act_index: u8,
        map: MapViewDto,
    },
    /// 보스 노드 완료로 런 클리어
    RunComplete {
        map: MapViewDto,
    },
    /// 더 이상 런을 진행할 수 없음
    RunFailed {
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    },

    // 아이템 장착 해제
    UnEquipItem {
        result: UnequipItemResultDto,
    },
    /// 아이템 장착/조합
    EquipItem {
        result: EquipItemResultDto,
    },
    ConsumableItemUsed {
        item_uuid: Uuid,
        target_employee_uuid: Uuid,
        replaced_modifier: Option<ActiveConsumableModifier>,
        applied_modifier: ActiveConsumableModifier,
        inventory_diff: InventoryDiffDto,
    },
    SkillFragmentLoadoutUpdated {
        employee_uuid: Uuid,
        equipped_fragment_ids: Vec<SkillFragmentId>,
    },
    SkillFragmentUpgraded {
        target_fragment_id: SkillFragmentId,
        dust_spent: u32,
        remaining_dust: u32,
        progress: SkillFragmentProgress,
    },
    SkillFragmentAwakened {
        target_fragment_id: SkillFragmentId,
        dust_spent: u32,
        remaining_dust: u32,
        progress: SkillFragmentProgress,
    },
    SkillFragmentDismantled {
        fragment_id: SkillFragmentId,
        remaining_count: u32,
        dust_gained: u32,
        total_dust: u32,
    },
    EquipmentDismantled {
        item_uuid: Uuid,
        equipment_id: String,
        inventory_diff: InventoryDiffDto,
    },
    EquipmentEnhanced {
        item_uuid: Uuid,
        equipment_id: String,
        enhancement_level: u8,
        inventory_diff: InventoryDiffDto,
    },
    /// 런 중 직원 명단 표시 순서 이동
    MoveRosterUnit {
        roster_slots: Vec<RosterSlotDto>,
    },

    /// 상점 상태 업데이트 (예: 리롤 이후)
    ShopState {
        shop: ShopEventOption,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
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

    /// 본사 연락 노드에서 직원 채용 완료
    EmployeeRecruited {
        candidate_id: String,
        employee_uuid: Uuid,
        completion: Box<BehaviorResult>,
    },

    /// 본사 연락 노드에서 긴급 보급 수령 완료
    EmergencySuppliesGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        completion: Box<BehaviorResult>,
    },

    /// 보상 수령 결과 (자원 및 인벤토리 변경)
    RewardGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
    },

    /// 전투 리플레이 종료 후 자동 지급된 보상과 이어진 노드 진행 결과
    CombatRewardsGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        outcome: NodeOutcomeSummary,
        completion: Box<BehaviorResult>,
    },

    /// 실시간 전투 tick 결과
    BattleAdvanced {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        playback: BattlePlaybackState,
        battle_time_ms: u64,
        timeline_delta: Vec<TimelineEntry>,
        last_timeline_seq: u64,
        finished: bool,
        deployment: Option<LiveBattleDeploymentDto>,
    },

    /// 실시간 전투 상태 조회 결과
    BattleState {
        battle_uuid: Uuid,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        encounter_id: String,
        combat_preview: CombatPreview,
        playback: BattlePlaybackState,
        battle_time_ms: u64,
        timeline_delta: Vec<TimelineEntry>,
        last_timeline_seq: u64,
        finished: bool,
        deployment: Option<LiveBattleDeploymentDto>,
    },

    /// 실시간 전투 재생 상태가 변경됨
    BattlePlaybackChanged {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        playback: BattlePlaybackState,
        battle_time_ms: u64,
        last_timeline_seq: u64,
        deployment: Option<LiveBattleDeploymentDto>,
    },

    /// 실시간 전투에서 직원 배치가 적용됨
    BattleUnitDeployed {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        playback: BattlePlaybackState,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        timeline_delta: Vec<TimelineEntry>,
        last_timeline_seq: u64,
        deployment: LiveBattleDeploymentDto,
    },

    /// 실시간 전투에서 직원 후퇴가 적용됨
    BattleUnitWithdrawn {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        playback: BattlePlaybackState,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        timeline_delta: Vec<TimelineEntry>,
        last_timeline_seq: u64,
        deployment: LiveBattleDeploymentDto,
    },

    /// 실시간 전투에서 직원 수동 스킬 발동 명령이 적용됨
    BattleSkillActivated {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        playback: BattlePlaybackState,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        skill_id: crate::game::ability::SkillId,
        timeline_delta: Vec<TimelineEntry>,
        last_timeline_seq: u64,
        deployment: LiveBattleDeploymentDto,
    },

    /// 보상 선택/수령 단계 상태
    RewardState {
        mode: RewardMode,
        rewards: Vec<RewardOption>,
        selected_reward_uuid: Option<Uuid>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },

    Ok,
}

impl BehaviorResult {
    // ============================================================
    // 타입 체크 헬퍼 (데이터가 없는 variant용)
    // ============================================================

    pub fn is_start_new_game(&self) -> bool {
        matches!(self, BehaviorResult::StartNewGame { .. })
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

    /// ShopState → ShopEventOption 참조 반환
    pub fn as_shop_state(&self) -> Option<&ShopEventOption> {
        match self {
            BehaviorResult::ShopState { shop, .. } => Some(shop),
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

    /// RewardGranted → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    pub fn as_reward_granted(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::RewardGranted {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }

    /// RewardState → (모드, 보상 목록, 현재 선택 보상 UUID) 반환
    pub fn as_reward_state(&self) -> Option<(RewardMode, &[RewardOption], Option<Uuid>)> {
        match self {
            BehaviorResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
                ..
            } => Some((*mode, rewards.as_slice(), *selected_reward_uuid)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameError {
    /// 선택한 이벤트/보상이 현재 세션 옵션에 존재하지 않을 때
    EventNotFound,
    /// 이벤트는 존재하지만 기대한 타입(Shop/Reward/Random 등)이 아닐 때
    EventTypeMismatch,

    /// 현재 GameState/Context에서 허용되지 않은 행동 (치팅 시도 포함)
    InvalidAction,

    /// 상점 상태가 아니거나, 활성 노드 콘텐츠에 Shop 정보가 없을 때
    NotInShopState,
    /// 보상 상태가 아니거나, 활성 노드 콘텐츠에 Reward 정보가 없을 때
    NotInRewardState,
    /// 상점이 리롤을 지원하지 않을 때
    ShopRerollNotAllowed,
    /// 상점의 visible_items / uuid_lookup_table에서 아이템을 찾지 못했을 때
    ShopItemNotFound,

    /// 인벤토리가 가득 차서 아이템을 추가할 수 없을 때 (예: 아티팩트 슬롯)
    InventoryFull,
    /// 인벤토리에서 아이템을 찾을 수 없을 때
    InventoryItemNotFound,
    /// 유일해야 하는 아티팩트를 이미 보유 중일 때
    AlreadyOwnedArtifact,

    /// 구매/행동에 필요한 자원이 부족할 때
    InsufficientResources,

    /// 필수 리소스(Enkephalin, Inventory 등)가 World에 없을 때
    MissingResource(&'static str),

    /// 기물의 전투 스탯이 정의되지 않았거나 잘못된 경우
    InvalidUnitStats(&'static str),
    /// RON 등 정적 데이터가 현재 엔진 계약을 위반할 때
    InvalidStaticData(String),
    /// 아직 구현되지 않은 핵심 게임 루프/콘텐츠를 호출했을 때
    NotImplemented(&'static str),

    /// 스킬 파편이 현재 직원/무기 전투 프로필과 호환되지 않을 때
    SkillFragmentIncompatible {
        fragment_id: SkillFragmentId,
        failure_codes: Vec<SkillFragmentCompatibilityFailureCode>,
    },

    /// 필드 위치가 범위를 벗어났을 때
    OutOfBounds,
    /// 해당 위치에 이미 기물이 배치되어 있을 때
    PositionOccupied,
    /// 해당 기물이 이미 필드에 배치되어 있을 때
    UnitAlreadyPlaced,
    /// 필드에서 기물을 찾을 수 없을 때
    UnitNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::shop_data::ShopType;
    use crate::game::resources::InventoryDiffDto;

    #[test]
    fn behavior_result_helpers_match_variants() {
        assert!(BehaviorResult::StartNewGame {
            candidates: Vec::new(),
            required_count: 3,
        }
        .is_start_new_game());
        assert!(BehaviorResult::Ok.is_ok());

        let sell = BehaviorResult::SellItem {
            enkephalin: 123,
            inventory_diff: InventoryDiffDto::default(),
        };
        assert!(sell.is_sell_item());
        let (remaining, diff) = sell.as_sell_item().unwrap();
        assert_eq!(remaining, 123);
        assert!(diff.added.is_empty());

        let purchase = BehaviorResult::PurchaseItem {
            enkephalin: 7,
            inventory_diff: InventoryDiffDto::default(),
        };
        let (remaining, diff) = purchase.as_purchase_item().unwrap();
        assert_eq!(remaining, 7);
        assert!(diff.removed.is_empty());

        let shop = ShopEventOption {
            id: "shop".to_string(),
            name: "Shop".to_string(),
            uuid: Uuid::nil(),
            shop_type: ShopType::Shop,
            can_reroll: true,
            visible_items: vec![Uuid::nil()],
        };
        let shop_state = BehaviorResult::ShopState {
            shop: shop.clone(),
            research_deliveries: vec![],
        };
        assert_eq!(shop_state.as_shop_state(), Some(&shop));
    }
}
