use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    game::employee::ActiveConsumableModifier,
    game::resources::{
        ArtifactItemDto, ConsumableItemDto, EquipItemResultDto, EquipmentItemDto, InventoryDiffDto,
        Position, RunFailureReason, UnequipItemResultDto,
    },
    game::{
        ability::{
            FocusPermissions, SkillActivationMode, SkillAreaAnchorSource, SkillAreaTickPolicy,
            SkillAreaTracking, SkillHitTargetFilter, SkillId, SkillKind, SkillTarget,
            SkillTileAreaOrigin,
        },
        battle::{
            core::movement::types::WorldVec2,
            event_log::{BattleEventLog, BattleEventLogEntry, SkillCastTarget},
            ids::UnitInstanceId,
            result_stats::BattleResultStatsDto,
            tile_range::{FacingDirection, TileRangePattern, TileRangePolicy},
            types::{
                BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
                DeploymentAffinity, MobilityKind,
            },
        },
        combat_preview::{
            BattlefieldRoute, BattlefieldTile, CombatMissionVariant, CombatNodeType, CombatPreview,
            DeploymentZone, SpawnZone,
        },
        data::{
            abnormality_data::BasicAttackDef,
            consumable_data::{
                ConsumableDurationPolicy, ConsumableEffect, ConsumableTargetPolicy, ConsumableTier,
            },
            equipment_data::{EquipmentType, WeaponCombatProfile},
            reward_data::RewardGrantKind,
            shop_data::ShopType,
            skill_fragment_data::{
                SkillFragmentCompatibilityFailureCode, SkillFragmentCompatibilityRequirements,
                SkillFragmentId,
            },
        },
        employee::StarterEmployeeCandidate,
        enums::{RewardMode, RiskLevel, ShopEventOption, Side},
        map::{
            HeadquartersContactOption, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload,
            MapViewDto, NodeSession, SupportNodeMode, SupportNodeType,
        },
        reward::{
            RewardEffect, RewardOption, SkillFragmentGrantDiffDto, SkillFragmentResearchDiffDto,
        },
        skill_fragment::{SkillFragmentProgress, SkillFragmentResearchDelivery},
        stats::UnitStats,
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
    LoadRunCheckpoint,
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
    RecoverBattleSetupLoss,
    RequestDeploymentRangePreview,
    DeployUnit,
    WithdrawUnit,
    ActivateSkill,
    RetreatBattle,
    PauseBattle,
    ResumeBattle,
    SetBattleSpeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunProgressionSnapshotDto {
    pub run_seed: u64,
    pub act_index: u8,
    pub max_acts: u8,
    pub current_act_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProgressionSnapshotDto {
    pub current_node_id: Option<MapNodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResourcesSnapshotDto {
    pub enkephalin: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotErrorDto {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFragmentProgressSnapshotDto {
    pub research_progress: u32,
    pub research_completion_count: u32,
    pub upgrade_level: u8,
    pub awakening_progress: u32,
    pub awakening_available: bool,
    pub awakened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySkillFragmentSnapshotDto {
    pub id: SkillFragmentId,
    pub count: u32,
    pub name: Option<String>,
    pub effect: Option<String>,
    pub requirements: Option<SkillFragmentCompatibilityRequirements>,
    pub progress: SkillFragmentProgressSnapshotDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySkillFragmentProgressSnapshotDto {
    pub id: SkillFragmentId,
    pub owned_count: u32,
    pub name: Option<String>,
    pub progress: SkillFragmentProgressSnapshotDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEquipmentSnapshotDto {
    pub instance_uuid: Uuid,
    pub item: EquipmentItemDto,
    pub equipped_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryEquipmentMaterialSnapshotDto {
    pub material_id: String,
    pub amount: u32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub material_type: Option<String>,
    pub rarity: Option<RiskLevel>,
    pub equipment_type: Option<EquipmentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingResearchDeliverySnapshotDto {
    pub fragment_id: SkillFragmentId,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySnapshotDto {
    pub equipments: Vec<InventoryEquipmentSnapshotDto>,
    pub equipment_materials: Vec<InventoryEquipmentMaterialSnapshotDto>,
    pub artifacts: Vec<ArtifactItemDto>,
    pub consumables: Vec<ConsumableItemDto>,
    pub skill_fragments: Vec<InventorySkillFragmentSnapshotDto>,
    pub skill_fragment_progress: Vec<InventorySkillFragmentProgressSnapshotDto>,
    pub pending_research_deliveries: Vec<PendingResearchDeliverySnapshotDto>,
    pub fragment_dust: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterOrderSlotSnapshotDto {
    pub slot: usize,
    pub unit_uuid: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterOrderSnapshotDto {
    pub max_slots: usize,
    pub slots: Vec<RosterOrderSlotSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeHealthSnapshotDto {
    pub current_hp: u32,
    pub max_hp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeInjurySnapshotDto {
    pub id: String,
    pub severity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeTrustMemorySnapshotDto {
    pub kind: String,
    pub intensity: u8,
    pub remaining_nodes: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeTrustReactionSnapshotDto {
    pub event: String,
    pub trust_delta: i16,
    pub cue_count: usize,
    pub combat_modifier_count: usize,
    pub trauma_modifier_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeTrustSnapshotDto {
    pub score: i16,
    pub band: String,
    pub traits: Vec<String>,
    pub memories: Vec<EmployeeTrustMemorySnapshotDto>,
    pub recent_reactions: Vec<EmployeeTrustReactionSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeCombatProfileSnapshotDto {
    pub battle_tier: String,
    pub base_stats: UnitStats,
    pub basic_attack: BasicAttackDef,
    pub deployment_affinity: DeploymentAffinity,
    pub effective_stats: Option<UnitStats>,
    pub effective_basic_attack: Option<BasicAttackDef>,
    pub effective_weapon_profile: Option<WeaponCombatProfile>,
    pub effective_skill_id: Option<SkillId>,
    pub effective_deployment_affinity: Option<DeploymentAffinity>,
    pub effective_profile_error: Option<SnapshotErrorDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSkillFragmentBriefSnapshotDto {
    pub id: SkillFragmentId,
    pub name: Option<String>,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSkillFragmentCompatibilitySnapshotDto {
    pub id: SkillFragmentId,
    pub name: Option<String>,
    pub is_compatible: bool,
    pub failure_codes: Vec<SkillFragmentCompatibilityFailureCode>,
    pub requirements: Option<SkillFragmentCompatibilityRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSkillFragmentsSnapshotDto {
    pub equipped: Vec<EmployeeSkillFragmentBriefSnapshotDto>,
    pub baseline_ids: Vec<SkillFragmentId>,
    pub active_fragment_id: Option<SkillFragmentId>,
    pub equipped_ids: Vec<SkillFragmentId>,
    pub compatibility: Vec<EmployeeSkillFragmentCompatibilitySnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveConsumableModifierSnapshotDto {
    pub source_item_uuid: Uuid,
    pub definition_id: String,
    pub name: String,
    pub tier: ConsumableTier,
    pub duration_policy: ConsumableDurationPolicy,
    pub remaining_combat_nodes: u32,
    pub effect: ConsumableEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeEquippedItemSnapshotDto {
    pub instance_uuid: Uuid,
    pub base_uuid: Uuid,
    pub equipment_type: EquipmentType,
    pub definition_id: Option<String>,
    pub name: Option<String>,
    pub weapon_profile: Option<WeaponCombatProfile>,
    pub bound: bool,
    pub can_unequip: bool,
    pub cannot_unequip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSnapshotDto {
    pub uuid: Uuid,
    pub name: String,
    pub level: u32,
    pub experience: u32,
    pub life_state: String,
    pub availability: String,
    pub available_for_combat: bool,
    pub trauma: u32,
    pub health: EmployeeHealthSnapshotDto,
    pub injuries: Vec<EmployeeInjurySnapshotDto>,
    pub trust: EmployeeTrustSnapshotDto,
    pub combat_profile: EmployeeCombatProfileSnapshotDto,
    pub skill_fragments: EmployeeSkillFragmentsSnapshotDto,
    pub active_consumable_modifier: Option<ActiveConsumableModifierSnapshotDto>,
    pub equipped_items: Vec<EmployeeEquippedItemSnapshotDto>,
    pub roster_slot: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRosterSnapshotDto {
    pub employees: Vec<EmployeeSnapshotDto>,
    pub available_employee_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayEquipmentItemSnapshotDto {
    pub uuid: Uuid,
    pub kind: String,
    pub definition_id: String,
    pub id: String,
    pub name: String,
    pub rarity: RiskLevel,
    pub price: u32,
    #[serde(rename = "type")]
    pub item_type: String,
    pub equipment_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayArtifactItemSnapshotDto {
    pub uuid: Uuid,
    pub kind: String,
    pub definition_id: String,
    pub id: String,
    pub name: String,
    pub rarity: RiskLevel,
    pub price: u32,
    #[serde(rename = "type")]
    pub item_type: String,
    pub artifact_type: String,
    pub effect_id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConsumableItemSnapshotDto {
    pub uuid: Uuid,
    pub definition_id: String,
    pub name: String,
    pub description: String,
    pub tier: ConsumableTier,
    pub rarity: RiskLevel,
    pub price: u32,
    pub target_policy: ConsumableTargetPolicy,
    pub duration_policy: ConsumableDurationPolicy,
    pub effect: ConsumableEffect,
    pub kind: String,
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisplayItemSnapshotDto {
    Equipment(DisplayEquipmentItemSnapshotDto),
    Artifact(DisplayArtifactItemSnapshotDto),
    Consumable(DisplayConsumableItemSnapshotDto),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardOptionSnapshotDto {
    pub uuid: Uuid,
    pub kind: String,
    pub id: String,
    pub name: String,
    pub rarity: Option<RiskLevel>,
    pub price: u32,
    pub description: String,
    pub icon: String,
    pub grant_kinds: Vec<RewardGrantKind>,
    pub effects: Vec<RewardEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SelectedEventSnapshotDto {
    Shop {
        id: String,
        name: String,
        uuid: Uuid,
        shop_type: ShopType,
        can_reroll: bool,
        visible_items: Vec<DisplayItemSnapshotDto>,
        hidden_items: Vec<DisplayItemSnapshotDto>,
        visible_item_uuids: Vec<Uuid>,
        hidden_item_uuids: Vec<Uuid>,
    },
    Reward {
        stage_uuid: Uuid,
        mode: RewardMode,
        rewards: Vec<RewardOptionSnapshotDto>,
        selected_reward_uuid: Option<Uuid>,
        can_skip: bool,
    },
    Support {
        node_id: MapNodeId,
        support_mode: SupportNodeMode,
        support_type: Option<SupportNodeType>,
        choices: Vec<SupportNodeType>,
        selected_support_type: Option<SupportNodeType>,
    },
    Maintenance {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
    },
    HeadquartersContact {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<StarterEmployeeCandidate>,
        shop_pool_id: Option<String>,
    },
    CombatBattle {
        abnormality_id: String,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        abnormality_uuid: Uuid,
        winner: BattleWinner,
        reward_mode: RewardMode,
        rewards: Vec<RewardOptionSnapshotDto>,
        result_stats: BattleResultStatsDto,
        has_event_log: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResultEventLogAttachmentDto {
    pub winner: BattleWinner,
    pub event_log: BattleEventLog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameStateContextDto {
    NotStarted,
    SelectingStarterEmployees {
        required_count: usize,
        candidates: Vec<StarterEmployeeCandidate>,
    },
    ViewingMap,
    NodeConfirm {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        combat_preview: Option<CombatPreview>,
        abnormality_attempt: Option<AbnormalityAttemptDto>,
    },
    InNode {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
    },
    InShop {
        shop_uuid: Uuid,
    },
    InReward {
        reward_uuid: Uuid,
    },
    InRewardClaimed {
        reward_uuid: Uuid,
    },
    CombatResult {
        battle_uuid: Uuid,
    },
    InBattle {
        battle_uuid: Uuid,
        node_type: Option<CombatNodeType>,
        mission_variant: Option<CombatMissionVariant>,
        encounter_id: Option<String>,
        combat_preview: Option<CombatPreview>,
        last_pushed_event_log_seq: Option<u64>,
        deployment: Option<LiveBattleDeploymentDto>,
        playback: Option<BattlePlaybackState>,
        abnormality_attempt: Option<AbnormalityAttemptDto>,
        can_retreat: bool,
    },
    GameOver,
    RunComplete,
    RunFailed {
        reason: RunFailureReason,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshotDto<SelectedEvent = SelectedEventSnapshotDto> {
    pub game_state: String,
    pub game_state_context: GameStateContextDto,
    pub allowed_actions: Vec<ActionKind>,
    pub run_checkpoint: RunCheckpointSnapshotDto,
    pub run_progression: Option<RunProgressionSnapshotDto>,
    pub map_progression: Option<MapProgressionSnapshotDto>,
    pub map: Option<MapViewDto>,
    pub current_node_session: Option<NodeSession>,
    pub selected_event: Option<SelectedEvent>,
    pub skill_catalog: SkillCatalogDto,
    pub roster: EmployeeRosterSnapshotDto,
    pub roster_order: RosterOrderSnapshotDto,
    pub inventory: InventorySnapshotDto,
    pub resources: RunResourcesSnapshotDto,
}

impl<SelectedEvent> RunSnapshotDto<SelectedEvent> {
    pub fn map_selected_event<MappedSelectedEvent>(
        self,
        f: impl FnOnce(Option<SelectedEvent>) -> Option<MappedSelectedEvent>,
    ) -> RunSnapshotDto<MappedSelectedEvent> {
        let RunSnapshotDto {
            game_state,
            game_state_context,
            allowed_actions,
            run_checkpoint,
            run_progression,
            map_progression,
            map,
            current_node_session,
            selected_event,
            skill_catalog,
            roster,
            roster_order,
            inventory,
            resources,
        } = self;

        RunSnapshotDto {
            game_state,
            game_state_context,
            allowed_actions,
            run_checkpoint,
            run_progression,
            map_progression,
            map,
            current_node_session,
            selected_event: f(selected_event),
            skill_catalog,
            roster,
            roster_order,
            inventory,
            resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCheckpointSnapshotDto {
    pub exists: bool,
    pub loads_used: u8,
    pub max_loads: u8,
    pub remaining_loads: u8,
    pub can_load: bool,
}

/// GameServer에서 GameCore로 전달되는 플레이어 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
    /// Safezone에서 마지막 SavePoint 체크포인트를 불러옴
    LoadRunCheckpoint,
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
    /// Unity scene/setup loss recovery. Discards the active live battle and returns to node confirm.
    RecoverBattleSetupLoss,
    /// 실시간 DefenseRoute 전투에서 아직 배치되지 않은 직원의 방향별 범위 preview 요청
    RequestDeploymentRangePreview {
        employee_uuid: Uuid,
        position: Position,
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
            PlayerBehavior::LoadRunCheckpoint => ActionKind::LoadRunCheckpoint,
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
            PlayerBehavior::RecoverBattleSetupLoss => ActionKind::RecoverBattleSetupLoss,
            PlayerBehavior::RequestDeploymentRangePreview { .. } => {
                ActionKind::RequestDeploymentRangePreview
            }
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
    pub position: Position,
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
pub struct LiveBattleUpdateDto {
    #[serde(rename = "type")]
    pub message_type: LiveBattleUpdateMessageType,
    pub battle_uuid: Uuid,
    pub server_battle_time_ms: u64,
    pub events_delta: LiveBattleEventDeltaDto,
    pub checkpoint: LiveBattleStateCheckpointDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleUpdateMessageType {
    BattleUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleSetupSnapshotDto {
    #[serde(rename = "type")]
    pub message_type: LiveBattleSetupSnapshotMessageType,
    pub setup_version: u32,
    pub battle_uuid: Uuid,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub survive_timer_ms: Option<u64>,
    pub battlefield: LiveBattleSetupBattlefieldDto,
    pub routes: Vec<BattlefieldRoute>,
    pub deployment_zones: Vec<DeploymentZone>,
    pub spawn_zones: Vec<SpawnZone>,
    pub tactical_points: Vec<LiveBattleSetupTacticalPointDto>,
    pub static_objects: Vec<LiveBattleSetupStaticObjectDto>,
    pub initial_units: Vec<LiveBattleSetupInitialUnitDto>,
    pub catalog_refs: LiveBattleSetupCatalogRefsDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleSetupSnapshotMessageType {
    BattleSetupSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleSetupBattlefieldDto {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<BattlefieldTile>,
    pub valid_tiles: Vec<Position>,
    pub blocked_tiles: Vec<Position>,
    pub static_obstacles: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSetupTacticalPointDto {
    pub id: String,
    pub point_type: LiveBattleSetupTacticalPointType,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleSetupTacticalPointType {
    TacticalPoint,
    SurvivalAnchor,
    RouteEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSetupStaticObjectDto {
    pub id: String,
    pub object_type: LiveBattleSetupStaticObjectType,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleSetupStaticObjectType {
    DefenseObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSetupInitialUnitDto {
    pub unit_ref: String,
    pub owner: Side,
    pub base_uuid: Uuid,
    pub position: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiveBattleSetupCatalogRefsDto {
    pub battlefield_template_id: String,
    pub abnormality_ids: Vec<String>,
}

impl LiveBattleSetupSnapshotDto {
    pub const SETUP_VERSION: u32 = 1;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleEventDeltaDto {
    pub after_seq: u64,
    pub to_seq: u64,
    pub events: Vec<BattleEventLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleStateCheckpointDto {
    pub at_seq: u64,
    pub battle_time_ms: u64,
    pub playback: BattlePlaybackState,
    pub units: Vec<LiveBattleUnitCheckpointDto>,
    pub deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleUnitCheckpointDto {
    pub unit_instance_id: UnitInstanceId,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub hud: LiveBattleUnitHudDto,
    pub range_previews: LiveBattleRangePreviewsDto,
    pub mobility_kind: MobilityKind,
    pub unit_source: BattleUnitSourceIdentity,
    pub position: Position,
    pub world_position: WorldVec2,
    pub stats: UnitStats,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleHudBarMode {
    HpOnly,
    HpAndResonance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleUnitHudDto {
    pub threat_class: BattleUnitThreatClass,
    pub bar_mode: LiveBattleHudBarMode,
    pub resonance_current: Option<u32>,
    pub resonance_max: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleRangePreviewSource {
    FallbackBasicAttack,
    WeaponProfile,
    SkillDefinition,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleRangePreviewReason {
    NoSkill,
    MissingRangeData,
    RequiresRuntimeTarget,
    InvalidPlacement,
    NoValidCells,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleActiveSkillTargetingKind {
    SelfTarget,
    AutoUnit,
    ManualTile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleBasicAttackRangePreviewDto {
    pub available: bool,
    pub reason: Option<LiveBattleRangePreviewReason>,
    pub cells: Vec<Position>,
    pub source: LiveBattleRangePreviewSource,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleActiveSkillRangePreviewDto {
    pub available: bool,
    pub reason: Option<LiveBattleRangePreviewReason>,
    pub skill_id: Option<SkillId>,
    pub targeting_kind: Option<LiveBattleActiveSkillTargetingKind>,
    pub requires_manual_target: bool,
    pub cast_cells: Vec<Position>,
    pub effect_preview_cells: Vec<Position>,
    pub source: LiveBattleRangePreviewSource,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleRangePreviewsDto {
    pub basic_attack: LiveBattleBasicAttackRangePreviewDto,
    pub active_skill: LiveBattleActiveSkillRangePreviewDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentRangePreviewResultDto {
    pub employee_uuid: Uuid,
    pub position: Position,
    pub facings: DeploymentRangePreviewFacingsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentRangePreviewFacingsDto {
    pub up: DeploymentRangePreviewFacingDto,
    pub right: DeploymentRangePreviewFacingDto,
    pub down: DeploymentRangePreviewFacingDto,
    pub left: DeploymentRangePreviewFacingDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentRangePreviewFacingDto {
    pub range_previews: LiveBattleRangePreviewsDto,
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
    pub target_policy: SkillTarget,
    pub range_policy: TileRangePolicy,
    pub defense_tile_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogStepDto {
    pub step_id: String,
    pub delay_ms: u32,
    pub target_policy: SkillTarget,
    pub range_policy: TileRangePolicy,
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
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
    },

    /// 전투 리플레이 종료 후 자동 지급된 보상과 이어진 노드 진행 결과
    CombatRewardsGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        completion: Box<BehaviorResult>,
    },

    /// 실시간 전투 tick 결과
    BattleAdvanced {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        battle_setup_snapshot: Option<LiveBattleSetupSnapshotDto>,
        battle_update: LiveBattleUpdateDto,
        finished: bool,
    },

    /// 실시간 전투 상태 조회 결과
    BattleState {
        battle_uuid: Uuid,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        encounter_id: String,
        combat_preview: CombatPreview,
        battle_update: LiveBattleUpdateDto,
        finished: bool,
    },

    /// 아직 배치되지 않은 직원의 방향별 배치 범위 preview
    DeploymentRangePreview {
        result: DeploymentRangePreviewResultDto,
    },

    /// 실시간 전투 재생 상태가 변경됨
    BattlePlaybackChanged {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
    },

    /// 실시간 전투에서 직원 배치가 적용됨
    BattleUnitDeployed {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
    },

    /// 실시간 전투에서 직원 후퇴가 적용됨
    BattleUnitWithdrawn {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
    },

    /// 실시간 전투에서 직원 수동 스킬 발동 명령이 적용됨
    BattleSkillActivated {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        skill_id: crate::game::ability::SkillId,
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

#[derive(Debug, Clone)]
pub enum BehaviorCommandResultContract {
    CommandPayload(CommandResultPayloadContract),
    BattleUpdate(BattleUpdateCommandResultContract),
    ReadOnlyPreview(CommandResultPayloadContract),
}

#[derive(Debug, Clone)]
pub struct BattleUpdateCommandResultContract {
    pub battle_update: LiveBattleUpdateDto,
    pub battle_setup_snapshot: Option<LiveBattleSetupSnapshotDto>,
}

#[derive(Debug, Clone)]
pub enum CommandResultPayloadContract {
    Run(RunCommandResult),
    Map(MapCommandResult),
    Node(NodeCommandResult),
    Facility(FacilityCommandResult),
    Inventory(InventoryCommandResult),
    Roster(RosterCommandResult),
    Shop(ShopCommandResult),
    Reward(RewardCommandResult),
    Battle(BattleCommandResult),
    General(GeneralCommandResult),
}

#[derive(Debug, Clone)]
pub enum RunCommandResult {
    StartNewGame {
        candidates: Vec<StarterEmployeeCandidate>,
        required_count: usize,
    },
    StarterEmployeesSelected {
        selected_candidate_ids: Vec<String>,
        employee_uuids: Vec<Uuid>,
        map: MapViewDto,
    },
    ActComplete {
        act_index: u8,
        map: MapViewDto,
    },
    RunComplete {
        map: MapViewDto,
    },
    RunFailed {
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    },
}

#[derive(Debug, Clone)]
pub enum MapCommandResult {
    MapState { map: MapViewDto },
    MoveRosterUnit { roster_slots: Vec<RosterSlotDto> },
}

#[derive(Debug, Clone)]
pub enum NodeCommandResult {
    NodePreview {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        payload: MapNodePayload,
        session: NodeSession,
        map: MapViewDto,
        combat_preview: Option<CombatPreview>,
    },
    NodeEntered {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        payload: MapNodePayload,
        session: NodeSession,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    NodeCompleted {
        map: MapViewDto,
        outcome: Option<NodeOutcomeSummary>,
    },
}

#[derive(Debug, Clone)]
pub enum FacilityCommandResult {
    SupportState {
        node_id: MapNodeId,
        support_mode: SupportNodeMode,
        support_type: Option<SupportNodeType>,
        choices: Vec<SupportNodeType>,
        selected_support_type: Option<SupportNodeType>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    MaintenanceState {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    HeadquartersContactState {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<StarterEmployeeCandidate>,
        shop_pool_id: Option<String>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    EmployeeRecruited {
        candidate_id: String,
        employee_uuid: Uuid,
        completion: Box<CommandResultPayloadContract>,
    },
    EmergencySuppliesGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[derive(Debug, Clone)]
pub enum InventoryCommandResult {
    UnEquipItem {
        result: UnequipItemResultDto,
    },
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
}

#[derive(Debug, Clone)]
pub enum RosterCommandResult {
    EmployeeRecruited {
        candidate_id: String,
        employee_uuid: Uuid,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[derive(Debug, Clone)]
pub enum ShopCommandResult {
    ShopState {
        shop: ShopEventOption,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    RerollShop {
        new_items: Vec<Uuid>,
    },
    SellItem {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
    },
    PurchaseItem {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
    },
}

#[derive(Debug, Clone)]
pub enum RewardCommandResult {
    RewardState {
        mode: RewardMode,
        rewards: Vec<RewardOption>,
        selected_reward_uuid: Option<Uuid>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    RewardGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
    },
    CombatRewardsGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        skill_fragment_research_diffs: Vec<SkillFragmentResearchDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[derive(Debug, Clone)]
pub enum BattleCommandResult {
    DeploymentRangePreview {
        result: DeploymentRangePreviewResultDto,
    },
}

#[derive(Debug, Clone)]
pub enum GeneralCommandResult {
    Ok,
}

impl BehaviorResult {
    pub fn into_command_result_contract(self) -> BehaviorCommandResultContract {
        match self {
            BehaviorResult::StartNewGame {
                candidates,
                required_count,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::StartNewGame {
                    candidates,
                    required_count,
                },
            )),
            BehaviorResult::StarterEmployeesSelected {
                selected_candidate_ids,
                employee_uuids,
                map,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::StarterEmployeesSelected {
                    selected_candidate_ids,
                    employee_uuids,
                    map,
                },
            )),
            BehaviorResult::MapState { map } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Map(MapCommandResult::MapState { map }),
            ),
            BehaviorResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                NodeCommandResult::NodeEntered {
                    node_id,
                    kind_id,
                    category,
                    payload,
                    session,
                    research_deliveries,
                },
            )),
            BehaviorResult::NodePreview {
                node_id,
                kind_id,
                category,
                payload,
                session,
                map,
                combat_preview,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                NodeCommandResult::NodePreview {
                    node_id,
                    kind_id,
                    category,
                    payload,
                    session,
                    map,
                    combat_preview,
                },
            )),
            BehaviorResult::NodeCompleted { map, outcome } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                    NodeCommandResult::NodeCompleted { map, outcome },
                ))
            }
            BehaviorResult::SupportState {
                node_id,
                support_mode,
                support_type,
                choices,
                selected_support_type,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::SupportState {
                    node_id,
                    support_mode,
                    support_type,
                    choices,
                    selected_support_type,
                    research_deliveries,
                }),
            ),
            BehaviorResult::MaintenanceState {
                node_id,
                maintenance_options,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::MaintenanceState {
                    node_id,
                    maintenance_options,
                    research_deliveries,
                }),
            ),
            BehaviorResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(
                    FacilityCommandResult::HeadquartersContactState {
                        node_id,
                        options,
                        recruitment_candidates,
                        shop_pool_id,
                        research_deliveries,
                    },
                ),
            ),
            BehaviorResult::ActComplete { act_index, map } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                    RunCommandResult::ActComplete { act_index, map },
                ))
            }
            BehaviorResult::RunComplete { map } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Run(RunCommandResult::RunComplete { map }),
            ),
            BehaviorResult::RunFailed { reason, outcome } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                    RunCommandResult::RunFailed { reason, outcome },
                ))
            }
            BehaviorResult::UnEquipItem { result } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::Inventory(InventoryCommandResult::UnEquipItem {
                        result,
                    }),
                )
            }
            BehaviorResult::EquipItem { result } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(InventoryCommandResult::EquipItem {
                    result,
                }),
            ),
            BehaviorResult::ConsumableItemUsed {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                applied_modifier,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::ConsumableItemUsed {
                        item_uuid,
                        target_employee_uuid,
                        replaced_modifier,
                        applied_modifier,
                        inventory_diff,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                equipped_fragment_ids,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentLoadoutUpdated {
                        employee_uuid,
                        equipped_fragment_ids,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentUpgraded {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentUpgraded {
                        target_fragment_id,
                        dust_spent,
                        remaining_dust,
                        progress,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentAwakened {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentAwakened {
                        target_fragment_id,
                        dust_spent,
                        remaining_dust,
                        progress,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentDismantled {
                fragment_id,
                remaining_count,
                dust_gained,
                total_dust,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentDismantled {
                        fragment_id,
                        remaining_count,
                        dust_gained,
                        total_dust,
                    },
                ),
            ),
            BehaviorResult::EquipmentDismantled {
                item_uuid,
                equipment_id,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::EquipmentDismantled {
                        item_uuid,
                        equipment_id,
                        inventory_diff,
                    },
                ),
            ),
            BehaviorResult::EquipmentEnhanced {
                item_uuid,
                equipment_id,
                enhancement_level,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::EquipmentEnhanced {
                        item_uuid,
                        equipment_id,
                        enhancement_level,
                        inventory_diff,
                    },
                ),
            ),
            BehaviorResult::MoveRosterUnit { roster_slots } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Map(
                    MapCommandResult::MoveRosterUnit { roster_slots },
                ))
            }
            BehaviorResult::ShopState {
                shop,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Shop(
                ShopCommandResult::ShopState {
                    shop,
                    research_deliveries,
                },
            )),
            BehaviorResult::RerollShop { new_items } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Shop(
                    ShopCommandResult::RerollShop { new_items },
                ))
            }
            BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Shop(
                ShopCommandResult::SellItem {
                    enkephalin,
                    inventory_diff,
                },
            )),
            BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Shop(
                ShopCommandResult::PurchaseItem {
                    enkephalin,
                    inventory_diff,
                },
            )),
            BehaviorResult::EmployeeRecruited {
                candidate_id,
                employee_uuid,
                completion,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::EmployeeRecruited {
                    candidate_id,
                    employee_uuid,
                    completion: Box::new(completion.into_command_payload_contract()),
                }),
            ),
            BehaviorResult::EmergencySuppliesGranted {
                enkephalin,
                inventory_diff,
                completion,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(
                    FacilityCommandResult::EmergencySuppliesGranted {
                        enkephalin,
                        inventory_diff,
                        completion: Box::new(completion.into_command_payload_contract()),
                    },
                ),
            ),
            BehaviorResult::RewardGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::RewardGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
                    skill_fragment_research_diffs,
                    employee_experience_diffs,
                }),
            ),
            BehaviorResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
                outcome,
                completion,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::CombatRewardsGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
                    skill_fragment_research_diffs,
                    employee_experience_diffs,
                    outcome,
                    completion: Box::new(completion.into_command_payload_contract()),
                }),
            ),
            BehaviorResult::BattleAdvanced {
                battle_setup_snapshot,
                battle_update,
                ..
            } => BehaviorCommandResultContract::BattleUpdate(BattleUpdateCommandResultContract {
                battle_update,
                battle_setup_snapshot,
            }),
            BehaviorResult::BattleState { battle_update, .. }
            | BehaviorResult::BattlePlaybackChanged { battle_update, .. }
            | BehaviorResult::BattleUnitDeployed { battle_update, .. }
            | BehaviorResult::BattleUnitWithdrawn { battle_update, .. }
            | BehaviorResult::BattleSkillActivated { battle_update, .. } => {
                BehaviorCommandResultContract::BattleUpdate(BattleUpdateCommandResultContract {
                    battle_update,
                    battle_setup_snapshot: None,
                })
            }
            BehaviorResult::DeploymentRangePreview { result } => {
                BehaviorCommandResultContract::ReadOnlyPreview(
                    CommandResultPayloadContract::Battle(
                        BattleCommandResult::DeploymentRangePreview { result },
                    ),
                )
            }
            BehaviorResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::RewardState {
                    mode,
                    rewards,
                    selected_reward_uuid,
                    research_deliveries,
                }),
            ),
            BehaviorResult::Ok => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::General(GeneralCommandResult::Ok),
            ),
        }
    }

    fn into_command_payload_contract(self) -> CommandResultPayloadContract {
        match self.into_command_result_contract() {
            BehaviorCommandResultContract::CommandPayload(payload)
            | BehaviorCommandResultContract::ReadOnlyPreview(payload) => payload,
            BehaviorCommandResultContract::BattleUpdate(_) => {
                panic!("battle update results cannot be nested inside command result payloads")
            }
        }
    }

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
                ..
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

impl CommandResultPayloadContract {
    pub fn result_type(&self) -> &'static str {
        match self {
            CommandResultPayloadContract::Run(result) => result.result_type(),
            CommandResultPayloadContract::Map(result) => result.result_type(),
            CommandResultPayloadContract::Node(result) => result.result_type(),
            CommandResultPayloadContract::Facility(result) => result.result_type(),
            CommandResultPayloadContract::Inventory(result) => result.result_type(),
            CommandResultPayloadContract::Roster(result) => result.result_type(),
            CommandResultPayloadContract::Shop(result) => result.result_type(),
            CommandResultPayloadContract::Reward(result) => result.result_type(),
            CommandResultPayloadContract::Battle(result) => result.result_type(),
            CommandResultPayloadContract::General(result) => result.result_type(),
        }
    }

    pub fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            CommandResultPayloadContract::Run(result) => result.payload_value(),
            CommandResultPayloadContract::Map(result) => result.payload_value(),
            CommandResultPayloadContract::Node(result) => result.payload_value(),
            CommandResultPayloadContract::Facility(result) => result.payload_value(),
            CommandResultPayloadContract::Inventory(result) => result.payload_value(),
            CommandResultPayloadContract::Roster(result) => result.payload_value(),
            CommandResultPayloadContract::Shop(result) => result.payload_value(),
            CommandResultPayloadContract::Reward(result) => result.payload_value(),
            CommandResultPayloadContract::Battle(result) => result.payload_value(),
            CommandResultPayloadContract::General(result) => result.payload_value(),
        }
    }
}

fn nested_payload_value(completion: &CommandResultPayloadContract) -> serde_json::Result<Value> {
    Ok(json!({
        "result_type": completion.result_type(),
        "payload": completion.payload_value()?,
    }))
}

impl RunCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            RunCommandResult::StartNewGame { .. } => "StartNewGame",
            RunCommandResult::StarterEmployeesSelected { .. } => "StarterEmployeesSelected",
            RunCommandResult::ActComplete { .. } => "ActComplete",
            RunCommandResult::RunComplete { .. } => "RunComplete",
            RunCommandResult::RunFailed { .. } => "RunFailed",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            RunCommandResult::StartNewGame {
                candidates,
                required_count,
            } => Ok(json!({
                "candidates": candidates,
                "required_count": required_count,
            })),
            RunCommandResult::StarterEmployeesSelected {
                selected_candidate_ids,
                employee_uuids,
                map,
            } => Ok(json!({
                "selected_candidate_ids": selected_candidate_ids,
                "employee_uuids": employee_uuids,
                "map": map,
            })),
            RunCommandResult::ActComplete { act_index, map } => Ok(json!({
                "act_index": act_index,
                "map": map,
            })),
            RunCommandResult::RunComplete { map } => serde_json::to_value(map),
            RunCommandResult::RunFailed { reason, outcome } => Ok(json!({
                "reason": reason,
                "outcome": outcome,
            })),
        }
    }
}

impl MapCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            MapCommandResult::MapState { .. } => "MapState",
            MapCommandResult::MoveRosterUnit { .. } => "MoveRosterUnit",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            MapCommandResult::MapState { map } => Ok(json!({ "map": map })),
            MapCommandResult::MoveRosterUnit { roster_slots } => Ok(json!({
                "roster_slots": roster_slots,
            })),
        }
    }
}

impl NodeCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            NodeCommandResult::NodePreview { .. } => "NodePreview",
            NodeCommandResult::NodeEntered { .. } => "NodeEntered",
            NodeCommandResult::NodeCompleted { .. } => "NodeCompleted",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            NodeCommandResult::NodePreview {
                node_id,
                kind_id,
                category,
                payload,
                session,
                map,
                combat_preview,
            } => Ok(json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
                "map": map,
                "combat_preview": combat_preview,
            })),
            NodeCommandResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
                research_deliveries,
            } => Ok(json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
                "research_deliveries": research_deliveries,
            })),
            NodeCommandResult::NodeCompleted { map, outcome } => Ok(json!({
                "map": map,
                "outcome": outcome,
            })),
        }
    }
}

impl FacilityCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            FacilityCommandResult::SupportState { .. } => "SupportState",
            FacilityCommandResult::MaintenanceState { .. } => "MaintenanceState",
            FacilityCommandResult::HeadquartersContactState { .. } => "HeadquartersContactState",
            FacilityCommandResult::EmployeeRecruited { .. } => "EmployeeRecruited",
            FacilityCommandResult::EmergencySuppliesGranted { .. } => "EmergencySuppliesGranted",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            FacilityCommandResult::SupportState {
                node_id,
                support_mode,
                support_type,
                choices,
                selected_support_type,
                research_deliveries,
            } => Ok(json!({
                "node_id": node_id,
                "support_mode": support_mode,
                "support_type": support_type,
                "choices": choices,
                "selected_support_type": selected_support_type,
                "research_deliveries": research_deliveries,
            })),
            FacilityCommandResult::MaintenanceState {
                node_id,
                maintenance_options,
                research_deliveries,
            } => Ok(json!({
                "node_id": node_id,
                "maintenance_options": maintenance_options,
                "research_deliveries": research_deliveries,
            })),
            FacilityCommandResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
                research_deliveries,
            } => Ok(json!({
                "node_id": node_id,
                "options": options,
                "recruitment_candidates": recruitment_candidates,
                "shop_pool_id": shop_pool_id,
                "research_deliveries": research_deliveries,
            })),
            FacilityCommandResult::EmployeeRecruited {
                candidate_id,
                employee_uuid,
                completion,
            } => Ok(json!({
                "candidate_id": candidate_id,
                "employee_uuid": employee_uuid,
                "completion": nested_payload_value(completion)?,
            })),
            FacilityCommandResult::EmergencySuppliesGranted {
                enkephalin,
                inventory_diff,
                completion,
            } => Ok(json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
                "completion": nested_payload_value(completion)?,
            })),
        }
    }
}

impl InventoryCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            InventoryCommandResult::UnEquipItem { .. } => "UnEquipItem",
            InventoryCommandResult::EquipItem { .. } => "EquipItem",
            InventoryCommandResult::ConsumableItemUsed { .. } => "ConsumableItemUsed",
            InventoryCommandResult::SkillFragmentLoadoutUpdated { .. } => {
                "SkillFragmentLoadoutUpdated"
            }
            InventoryCommandResult::SkillFragmentUpgraded { .. } => "SkillFragmentUpgraded",
            InventoryCommandResult::SkillFragmentAwakened { .. } => "SkillFragmentAwakened",
            InventoryCommandResult::SkillFragmentDismantled { .. } => "SkillFragmentDismantled",
            InventoryCommandResult::EquipmentDismantled { .. } => "EquipmentDismantled",
            InventoryCommandResult::EquipmentEnhanced { .. } => "EquipmentEnhanced",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            InventoryCommandResult::UnEquipItem { result } => serde_json::to_value(result),
            InventoryCommandResult::EquipItem { result } => serde_json::to_value(result),
            InventoryCommandResult::ConsumableItemUsed {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                applied_modifier,
                inventory_diff,
            } => Ok(json!({
                "item_uuid": item_uuid,
                "target_employee_uuid": target_employee_uuid,
                "replaced_modifier": replaced_modifier,
                "applied_modifier": applied_modifier,
                "inventory_diff": inventory_diff,
            })),
            InventoryCommandResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                equipped_fragment_ids,
            } => Ok(json!({
                "employee_uuid": employee_uuid,
                "equipped_fragment_ids": equipped_fragment_ids,
            })),
            InventoryCommandResult::SkillFragmentUpgraded {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            }
            | InventoryCommandResult::SkillFragmentAwakened {
                target_fragment_id,
                dust_spent,
                remaining_dust,
                progress,
            } => Ok(json!({
                "target_fragment_id": target_fragment_id,
                "dust_spent": dust_spent,
                "remaining_dust": remaining_dust,
                "progress": progress,
            })),
            InventoryCommandResult::SkillFragmentDismantled {
                fragment_id,
                remaining_count,
                dust_gained,
                total_dust,
            } => Ok(json!({
                "fragment_id": fragment_id,
                "remaining_count": remaining_count,
                "dust_gained": dust_gained,
                "total_dust": total_dust,
            })),
            InventoryCommandResult::EquipmentDismantled {
                item_uuid,
                equipment_id,
                inventory_diff,
            } => Ok(json!({
                "item_uuid": item_uuid,
                "equipment_id": equipment_id,
                "inventory_diff": inventory_diff,
            })),
            InventoryCommandResult::EquipmentEnhanced {
                item_uuid,
                equipment_id,
                enhancement_level,
                inventory_diff,
            } => Ok(json!({
                "item_uuid": item_uuid,
                "equipment_id": equipment_id,
                "enhancement_level": enhancement_level,
                "inventory_diff": inventory_diff,
            })),
        }
    }
}

impl RosterCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            RosterCommandResult::EmployeeRecruited { .. } => "EmployeeRecruited",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            RosterCommandResult::EmployeeRecruited {
                candidate_id,
                employee_uuid,
                completion,
            } => Ok(json!({
                "candidate_id": candidate_id,
                "employee_uuid": employee_uuid,
                "completion": nested_payload_value(completion)?,
            })),
        }
    }
}

impl ShopCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            ShopCommandResult::ShopState { .. } => "ShopState",
            ShopCommandResult::RerollShop { .. } => "RerollShop",
            ShopCommandResult::SellItem { .. } => "SellItem",
            ShopCommandResult::PurchaseItem { .. } => "PurchaseItem",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            ShopCommandResult::ShopState {
                shop,
                research_deliveries,
            } => Ok(json!({
                "shop": shop,
                "research_deliveries": research_deliveries,
            })),
            ShopCommandResult::RerollShop { new_items } => Ok(json!({
                "new_items": new_items,
            })),
            ShopCommandResult::SellItem {
                enkephalin,
                inventory_diff,
            }
            | ShopCommandResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => Ok(json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
            })),
        }
    }
}

impl RewardCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            RewardCommandResult::RewardState { .. } => "RewardState",
            RewardCommandResult::RewardGranted { .. } => "RewardGranted",
            RewardCommandResult::CombatRewardsGranted { .. } => "CombatRewardsGranted",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            RewardCommandResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
                research_deliveries,
            } => Ok(json!({
                "mode": mode,
                "rewards": rewards,
                "selected_reward_uuid": selected_reward_uuid,
                "research_deliveries": research_deliveries,
            })),
            RewardCommandResult::RewardGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
            } => Ok(json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
                "skill_fragment_diffs": skill_fragment_diffs,
                "skill_fragment_research_diffs": skill_fragment_research_diffs,
                "employee_experience_diffs": employee_experience_diffs,
            })),
            RewardCommandResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                skill_fragment_research_diffs,
                employee_experience_diffs,
                outcome,
                completion,
            } => Ok(json!({
                "enkephalin": enkephalin,
                "inventory_diff": inventory_diff,
                "skill_fragment_diffs": skill_fragment_diffs,
                "skill_fragment_research_diffs": skill_fragment_research_diffs,
                "employee_experience_diffs": employee_experience_diffs,
                "outcome": outcome,
                "completion": nested_payload_value(completion)?,
            })),
        }
    }
}

impl BattleCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            BattleCommandResult::DeploymentRangePreview { .. } => "DeploymentRangePreview",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            BattleCommandResult::DeploymentRangePreview { result } => serde_json::to_value(result),
        }
    }
}

impl GeneralCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            GeneralCommandResult::Ok => "Ok",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            GeneralCommandResult::Ok => Ok(Value::Null),
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
    /// 클라이언트가 현재 전투에 존재하지 않는 미래 presentation seq로 resync를 요청했을 때
    InvalidBattleResyncSeq { requested: u64, latest: u64 },

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
    /// 인벤토리에 존재하지만 해당 경로로 제거할 수 없는 아이템일 때
    InventoryItemNotRemovable,
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
    /// 정적 장애물 때문에 해당 타일을 사용할 수 없을 때
    StaticObstacleBlocked,
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
