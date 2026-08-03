use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    game::employee::{
        ActiveConsumableModifier, EmployeeCondition, EmployeeEligibility, MentalConditionState,
        PhysicalDamageState,
    },
    game::resources::{
        ConsumableItemDto, EquipItemResultDto, EquipmentItemDto, InventoryDiffDto, Position,
        RunFailureReason, UnequipItemResultDto,
    },
    game::{
        ability::{
            FocusPermissions, SkillActivationMode, SkillAreaAnchorSource, SkillAreaTickPolicy,
            SkillAreaTracking, SkillHitTargetFilter, SkillId, SkillKind, SkillPresentationDef,
            SkillTarget, SkillTargetReach, SkillTileAreaOrigin,
        },
        battle::{
            buffs::BuffId,
            cooldown::CooldownSource,
            core::movement::{
                types::{EventLogVec2, WorldVec2},
                MovementSegmentEndKind,
            },
            damage::{DamageBreakdown, DamageFeedbackTag, DamageSource, DamageType},
            event_log::{
                AttackDelivery, AttackKind, BattleEventCause, BattleEventLog, BattleEventLogEntry,
                BattleLogEvent, BattleProjectileGuidance, BattleSkillAreaShape, BuffExpireReason,
                HpChangeReason, MovementStopReason, SkillCastCancelReason, SkillCastTarget,
            },
            ids::UnitInstanceId,
            result_stats::BattleResultStatsDto,
            tile_range::{FacingDirection, TileRangePattern},
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
            run_policy_data::TraumaRecovery,
            shop_data::{ShopContext, ShopMerchandise},
            skill_fragment_data::{
                SkillFragmentCompatibilityFailureCode, SkillFragmentCompatibilityRequirements,
                SkillFragmentId,
            },
        },
        employee::{EmployeeDefinition, EmployeeProfile},
        enums::{RewardMode, RiskLevel, ShopEventOption, Side},
        map::{
            GameMode, HeadquartersContactOption, MapNodeCategory, MapNodeId, MapNodeKindId,
            MapNodePayload, MapViewDto, NodeSession,
        },
        reward::{
            RewardEffect, RewardOption, SkillFragmentGrantDiffDto, SkillFragmentResearchDiffDto,
        },
        skill_fragment::{SkillFragmentProgress, SkillFragmentResearchDelivery},
        stats::{StatModifier, UnitStats},
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
    pub physical_damage_before: u8,
    pub physical_damage_after: u8,
    pub trauma_before: u32,
    pub trauma_after: u32,
    pub experience_before: u32,
    pub experience_after: u32,
    pub was_alive: bool,
    pub is_alive: bool,
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

/// Stable command-family identity. This is not an authorization token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionKind {
    StartNewGame,
    ContinueRun,
    ResetActiveRun,
    RetryEncounterCompendium,
    ResetEncounterCompendium,
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
    RequestMapData,
    SelectMapNode,
    ConvertMapNodeWithMonolith,
    ConfirmEnterNode,
    CancelSelectedNode,
    CompleteNode,
    LoadRunCheckpoint,
    ConfirmRunFailure,
    AbandonRun,
    RecruitEmployee,
    RequestEmergencySupplies,
    OpenHeadquartersShop,
    SelectReward,
    PurchaseItem,
    SellItem,
    RefreshShop,
    ExitShop,
    ClaimReward,
    ExitReward,
    AdvanceEventScene,
    ResolveEventChoiceCheck,
    AdvanceVisualNovel,
    SelectVisualNovelChoice,
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

impl ActionKind {
    pub const fn scope(self) -> ActionScope {
        match self {
            Self::RequestMapData
            | Self::RequestBattleState
            | Self::RequestDeploymentRangePreview => ActionScope::Query,
            Self::ContinueRun
            | Self::ResetActiveRun
            | Self::RetryEncounterCompendium
            | Self::ResetEncounterCompendium
            | Self::LoadRunCheckpoint
            | Self::RecoverBattleSetupLoss => ActionScope::Recovery,
            _ => ActionScope::Gameplay,
        }
    }

    pub const fn requires_battle_authority(self) -> bool {
        matches!(
            self,
            Self::DeployUnit
                | Self::WithdrawUnit
                | Self::ActivateSkill
                | Self::RetreatBattle
                | Self::PauseBattle
                | Self::ResumeBattle
                | Self::SetBattleSpeed
                | Self::AdvanceVisualNovel
                | Self::SelectVisualNovelChoice
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionScope {
    Gameplay,
    Query,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionDisabledReasonCode {
    ContextIrrelevant,
    NoEligiblePayload,
    MissingEventChoiceKey,
    CheckpointUnavailable,
    ProgressionUnavailable,
    ContentUnavailable,
    CompletionUnavailable,
    AlreadyActive,
    InsufficientResources,
    RedeployCooldown,
    UnitUnavailable,
    EmployeeDead,
    CriticalPhysicalDamage,
    Breakdown,
    NoTrauma,
    SkillUnavailable,
    ActivationModeNotManual,
    HardCrowdControl,
    Silenced,
    ActionLocked,
    ResonanceNotFull,
    NoValidTarget,
    OfferUnavailable,
    RefreshUnavailable,
    RefreshUsesExhausted,
    PoolExhausted,
    SellingUnavailable,
    InvalidQuantity,
    InvalidQuote,
    ItemBound,
    ItemEquipped,
    TagNotAccepted,
}

impl ActionDisabledReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContextIrrelevant => "context_irrelevant",
            Self::NoEligiblePayload => "no_eligible_payload",
            Self::MissingEventChoiceKey => "missing_event_choice_key",
            Self::CheckpointUnavailable => "checkpoint_unavailable",
            Self::ProgressionUnavailable => "progression_unavailable",
            Self::ContentUnavailable => "content_unavailable",
            Self::CompletionUnavailable => "completion_unavailable",
            Self::AlreadyActive => "already_active",
            Self::InsufficientResources => "insufficient_resources",
            Self::RedeployCooldown => "redeploy_cooldown",
            Self::UnitUnavailable => "unit_unavailable",
            Self::EmployeeDead => "employee_dead",
            Self::CriticalPhysicalDamage => "critical_physical_damage",
            Self::Breakdown => "breakdown",
            Self::NoTrauma => "no_trauma",
            Self::SkillUnavailable => "skill_unavailable",
            Self::ActivationModeNotManual => "activation_mode_not_manual",
            Self::HardCrowdControl => "hard_crowd_control",
            Self::Silenced => "silenced",
            Self::ActionLocked => "action_locked",
            Self::ResonanceNotFull => "resonance_not_full",
            Self::NoValidTarget => "no_valid_target",
            Self::OfferUnavailable => "offer_unavailable",
            Self::RefreshUnavailable => "refresh_unavailable",
            Self::RefreshUsesExhausted => "refresh_uses_exhausted",
            Self::PoolExhausted => "pool_exhausted",
            Self::SellingUnavailable => "selling_unavailable",
            Self::InvalidQuantity => "invalid_quantity",
            Self::InvalidQuote => "invalid_quote",
            Self::ItemBound => "item_bound",
            Self::ItemEquipped => "item_equipped",
            Self::TagNotAccepted => "tag_not_accepted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattlePayloadEligibilityDto {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason_code: Option<ActionDisabledReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadEligibilityDto {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason_code: Option<ActionDisabledReasonCode>,
}

impl PayloadEligibilityDto {
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            disabled_reason_code: None,
        }
    }

    pub const fn disabled(reason: ActionDisabledReasonCode) -> Self {
        Self {
            enabled: false,
            disabled_reason_code: Some(reason),
        }
    }
}

impl BattlePayloadEligibilityDto {
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            disabled_reason_code: None,
        }
    }

    pub const fn disabled(reason: ActionDisabledReasonCode) -> Self {
        Self {
            enabled: false,
            disabled_reason_code: Some(reason),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionCapabilityDto {
    pub action: ActionKind,
    pub scope: ActionScope,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason_code: Option<ActionDisabledReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleStateAuthorityDto {
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
}

impl ActionCapabilityDto {
    pub const fn enabled(action: ActionKind, scope: ActionScope) -> Self {
        Self {
            action,
            scope,
            enabled: true,
            disabled_reason_code: None,
        }
    }

    pub const fn disabled(
        action: ActionKind,
        scope: ActionScope,
        reason: ActionDisabledReasonCode,
    ) -> Self {
        Self {
            action,
            scope,
            enabled: false,
            disabled_reason_code: Some(reason),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunProgressionSnapshotDto {
    pub run_seed: u64,
    pub game_mode: GameMode,
    pub floor_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_floors: Option<u8>,
    pub current_floor_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAbnormalityResearchSnapshotDto {
    pub entries: Vec<RunAbnormalityResearchEntrySnapshotDto>,
    pub all_response_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAbnormalityResearchEntrySnapshotDto {
    pub abnormality_id: String,
    pub research_points: u32,
    pub research_required: u32,
    pub response_complete: bool,
    pub suppression_wins: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_encountered_floor: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at_floor: Option<u32>,
    pub unique_fragment_granted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_complete_skill_fragment_id: Option<SkillFragmentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumSnapshotDto {
    pub abnormalities: Vec<EncounterCompendiumAbnormalitySnapshotDto>,
    pub corrosion_profiles: Vec<EncounterCompendiumCorrosionSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumAbnormalitySnapshotDto {
    pub abnormality_id: String,
    pub discovered: bool,
    pub defeated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumCorrosionSnapshotDto {
    pub profile_id: String,
    pub discovered: bool,
    pub defeated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapNavigationSnapshotDto {
    pub current_node_id: MapNodeId,
    pub selectable_node_ids: Vec<MapNodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithConversionOptionDto {
    pub abnormality_id: String,
    pub abnormality_uuid: Uuid,
    pub display_name: String,
    pub risk_level: RiskLevel,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    pub next_stage: u8,
    pub uses_before: u8,
    pub uses_after: u8,
    pub max_attempts: u8,
    pub reward_tier: MonolithRewardTierDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithConversionSnapshotDto {
    pub action_label: String,
    pub node_id: MapNodeId,
    pub options: Vec<MonolithConversionOptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithRemanifestationSnapshotDto {
    pub abnormality_id: String,
    pub abnormality_uuid: Uuid,
    pub display_name: String,
    pub risk_level: RiskLevel,
    pub stage: u8,
    pub max_attempts: u8,
    pub reward_tier: MonolithRewardTierDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonolithRewardTierDto {
    Enhanced,
    FinalEnhanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventSceneSnapshotDto {
    pub event_id: crate::game::data::event_data::EventId,
    pub current_scene_id: crate::game::data::event_data::EventSceneId,
    pub presentation: crate::game::data::event_data::EventScenePresentation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_check: Option<EventCommittedCheckDto>,
    pub interaction: EventInteractionDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventInteractionDto {
    Advance,
    EmployeeCheckChoices {
        choices: Vec<EventChoiceEvaluationDto>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoiceEvaluationDto {
    pub choice_id: crate::game::data::event_data::EventChoiceId,
    pub label_id: String,
    pub aptitude: crate::game::employee::EventAptitude,
    pub dc: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_choice_key: Option<crate::game::data::event_data::EventChoiceKeyId>,
    pub eligibility: PayloadEligibilityDto,
    pub employee_candidates: Vec<EventEmployeeEvaluationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventEmployeeEvaluationDto {
    pub employee_uuid: Uuid,
    pub name: String,
    pub eligibility: PayloadEligibilityDto,
    pub employee_modifier: u8,
    pub equipment_modifier: u8,
    pub modifier_total: u8,
    pub modifier_components: Vec<crate::game::resources::EventCheckModifierComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_key_evidence: Option<crate::game::resources::EventChoiceKeyEvidence>,
    pub success_chance_percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventCommittedCheckDto {
    pub source_scene_id: crate::game::data::event_data::EventSceneId,
    pub choice_id: crate::game::data::event_data::EventChoiceId,
    pub employee_uuid: Uuid,
    pub aptitude: crate::game::employee::EventAptitude,
    pub dc: u8,
    pub raw_roll: u8,
    pub modifier_components: Vec<crate::game::resources::EventCheckModifierComponent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice_key_evidence: Option<crate::game::resources::EventChoiceKeyEvidence>,
    pub modifier_total: u8,
    pub total_roll: u8,
    pub outcome: crate::game::data::event_data::EventCheckOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VisualNovelSourceDto {
    EventNode { node_id: MapNodeId },
    HqBriefing,
    Map,
    Loadout,
    Battle { battle_uuid: Uuid },
    Tutorial,
    Radio,
    Cutscene,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualNovelSpeakerDto {
    pub speaker_id: String,
    #[serde(default)]
    pub portrait_id: Option<String>,
    pub position: VisualNovelSpeakerPositionDto,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisualNovelSpeakerPositionDto {
    FarLeft,
    Left,
    Center,
    Right,
    FarRight,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualNovelChoicePreviewDto {
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub starts_combat: bool,
    #[serde(default)]
    pub reward_hint_id: Option<String>,
    #[serde(default)]
    pub penalty_hint_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualNovelChoiceSnapshotDto {
    pub choice_id: String,
    pub label_id: String,
    #[serde(default)]
    pub preview: VisualNovelChoicePreviewDto,
    pub selectable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VisualNovelPresentationModeDto {
    Cinematic,
    Radio,
    Compact,
    NonBlocking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VisualNovelSnapshotDto {
    pub source: VisualNovelSourceDto,
    pub session_id: String,
    pub vn_id: String,
    pub current_scene_id: String,
    #[serde(default)]
    pub background_id: Option<String>,
    pub script_id: String,
    pub speakers: Vec<VisualNovelSpeakerDto>,
    pub choices: Vec<VisualNovelChoiceSnapshotDto>,
    pub blocking: bool,
    pub presentation_mode: VisualNovelPresentationModeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResourcesSnapshotDto {
    pub enkephalin: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithProgressSnapshotDto {
    pub abnormality_id: String,
    pub display_name: String,
    pub risk_level: RiskLevel,
    pub natural_outcome: String,
    pub victories: u8,
    pub purchased: bool,
    pub uses_remaining: u8,
    pub active_node_id: Option<MapNodeId>,
    pub unique_fragment_granted: bool,
    pub residual_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithResidualSnapshotDto {
    pub sequence: u64,
    pub source_abnormality_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonolithRunSnapshotDto {
    pub entries: Vec<MonolithProgressSnapshotDto>,
    pub residuals: Vec<MonolithResidualSnapshotDto>,
    pub residual_exchange_threshold: usize,
    pub exchange_state: String,
    pub exchange_shop_guarantee_issued: bool,
    pub next_floor_shop_guarantee_pending: bool,
    pub corrupted_monolith_acquired: bool,
    pub elite_pool_exhausted: bool,
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
    pub reserved_slot_count: usize,
    pub occupied_slot_count: usize,
    pub equipments: Vec<InventoryEquipmentSnapshotDto>,
    pub equipment_materials: Vec<InventoryEquipmentMaterialSnapshotDto>,
    pub consumables: Vec<ConsumableItemDto>,
    pub skill_fragments: Vec<InventorySkillFragmentSnapshotDto>,
    pub skill_fragment_progress: Vec<InventorySkillFragmentProgressSnapshotDto>,
    pub pending_research_deliveries: Vec<PendingResearchDeliverySnapshotDto>,
    pub fragment_dust: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeConditionSnapshotDto {
    pub physical_damage: u8,
    pub physical_state: PhysicalDamageState,
    pub trauma: u32,
    pub mental_state: MentalConditionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeEligibilitySnapshotDto {
    pub combat: EmployeeEligibility,
    pub event: EmployeeEligibility,
    pub high_risk: EmployeeEligibility,
    pub consumable: EmployeeEligibility,
    pub loadout: EmployeeEligibility,
    pub physical_treatment: EmployeeEligibility,
    pub trauma_recovery: EmployeeEligibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeePsychologyMemorySnapshotDto {
    pub kind: String,
    pub subject: crate::game::employee_psychology::PsychologySubject,
    pub occurrence_count: u32,
    pub intensity: u8,
    pub first_source: crate::game::employee_psychology::PsychologyEventSource,
    pub last_source: crate::game::employee_psychology::PsychologyEventSource,
    pub remaining_nodes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeePsychologyRecentEventSnapshotDto {
    pub event: String,
    pub source: crate::game::employee_psychology::PsychologyEventSource,
    pub trust_delta: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeePsychologySnapshotDto {
    pub trust_score: i16,
    pub trust_band: String,
    pub memories: Vec<EmployeePsychologyMemorySnapshotDto>,
    pub recent_events: Vec<EmployeePsychologyRecentEventSnapshotDto>,
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
    pub employee_definition_id: String,
    pub name: String,
    pub profile: EmployeeProfile,
    pub level: u32,
    pub experience: u32,
    pub condition: EmployeeConditionSnapshotDto,
    pub eligibility: EmployeeEligibilitySnapshotDto,
    pub psychology: EmployeePsychologySnapshotDto,
    pub combat_profile: EmployeeCombatProfileSnapshotDto,
    pub skill_fragments: EmployeeSkillFragmentsSnapshotDto,
    pub active_consumable_modifier: Option<ActiveConsumableModifierSnapshotDto>,
    pub equipped_items: Vec<EmployeeEquippedItemSnapshotDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRosterSnapshotDto {
    pub employees: Vec<EmployeeSnapshotDto>,
    pub combat_eligible_employee_ids: Vec<Uuid>,
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
pub struct DisplayMonolithItemSnapshotDto {
    pub uuid: Uuid,
    pub abnormality_id: String,
    pub name: String,
    pub risk_level: RiskLevel,
    pub victories: u8,
    pub next_stage: u8,
    pub uses_granted: u8,
    pub kind: String,
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DisplayItemSnapshotDto {
    Equipment(DisplayEquipmentItemSnapshotDto),
    Consumable(DisplayConsumableItemSnapshotDto),
    Monolith(DisplayMonolithItemSnapshotDto),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraumaRecoveryEmployeePreviewDto {
    pub employee_uuid: Uuid,
    pub employee_name: String,
    pub eligibility: EmployeeEligibility,
    pub physical_damage: u8,
    pub trauma_before: u32,
    pub trauma_after: u32,
    pub mental_state_before: MentalConditionState,
    pub mental_state_after: MentalConditionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraumaRecoveryPreviewDto {
    pub recovery: TraumaRecovery,
    pub employees: Vec<TraumaRecoveryEmployeePreviewDto>,
    pub physical_damage_unchanged: bool,
    pub replaces_run_checkpoint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SelectedEventSnapshotDto {
    Shop {
        id: String,
        name: String,
        uuid: Uuid,
        context: ShopContext,
        offers: Vec<ShopOfferSnapshotDto>,
        refresh: ShopRefreshSnapshotDto,
        selling_enabled: bool,
        sale_candidates: Vec<ShopSaleCandidateSnapshotDto>,
    },
    Reward {
        stage_uuid: Uuid,
        mode: RewardMode,
        rewards: Vec<RewardOptionSnapshotDto>,
        selected_reward_uuid: Option<Uuid>,
        can_skip: bool,
    },
    SavePoint {
        node_id: MapNodeId,
        recovery_preview: TraumaRecoveryPreviewDto,
    },
    Maintenance {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
    },
    HeadquartersContact {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<EmployeeDefinition>,
        shop_pool_id: Option<String>,
    },
    Event {
        node_id: MapNodeId,
        event: EventSceneSnapshotDto,
    },
    CombatBattle {
        primary_abnormality_id: Option<String>,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        abnormality_uuid: Uuid,
        winner: BattleWinner,
        reward_mode: RewardMode,
        rewards: Vec<RewardOptionSnapshotDto>,
        result_stats: BattleResultStatsDto,
        bonus_objectives: Vec<crate::game::pve_bonus_objectives::PveBonusObjectiveOutcomeDto>,
        has_event_log: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopPayloadEligibilityDto {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_reason_code: Option<ActionDisabledReasonCode>,
}

impl ShopPayloadEligibilityDto {
    pub const fn enabled() -> Self {
        Self {
            enabled: true,
            disabled_reason_code: None,
        }
    }

    pub const fn disabled(reason: ActionDisabledReasonCode) -> Self {
        Self {
            enabled: false,
            disabled_reason_code: Some(reason),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopOfferSnapshotDto {
    pub offer_id: Uuid,
    pub merchandise: ShopMerchandise,
    pub quantity: u32,
    pub total_price: u32,
    pub item: DisplayItemSnapshotDto,
    pub eligibility: ShopPayloadEligibilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopRefreshSnapshotDto {
    pub available: bool,
    pub current_cost: Option<u32>,
    pub max_uses: u32,
    pub uses_consumed: u32,
    pub remaining_uses: u32,
    pub eligibility: ShopPayloadEligibilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSaleCandidateSnapshotDto {
    pub item_uuid: Uuid,
    pub merchandise: ShopMerchandise,
    pub available_quantity: u32,
    pub unit_quote: u32,
    pub eligibility: ShopPayloadEligibilityDto,
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
    PreRunBriefing {
        game_mode: GameMode,
    },
    SelectingStarterEmployees {
        required_count: usize,
        candidates: Vec<EmployeeDefinition>,
    },
    ViewingMap,
    NodeConfirm {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        combat_preview: Option<CombatPreview>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trauma_recovery_preview: Option<TraumaRecoveryPreviewDto>,
        abnormality_attempt: Option<AbnormalityAttemptDto>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        omen: Option<crate::game::boss_omen::NodeConfirmOmenDto>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monolith_conversion: Option<MonolithConversionSnapshotDto>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monolith_remanifestation: Option<MonolithRemanifestationSnapshotDto>,
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
        battle_state_revision: u64,
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
    AwaitingRunFailureDecision {
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshotDto<SelectedEvent = SelectedEventSnapshotDto> {
    pub game_state: String,
    pub game_state_context: GameStateContextDto,
    pub action_capabilities: Vec<ActionCapabilityDto>,
    pub run_checkpoint: RunCheckpointSnapshotDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combat_recovery_restriction: Option<CombatRecoveryRestrictionSnapshotDto>,
    pub run_progression: Option<RunProgressionSnapshotDto>,
    pub abnormality_research: Option<RunAbnormalityResearchSnapshotDto>,
    pub monolith: Option<MonolithRunSnapshotDto>,
    pub encounter_compendium: EncounterCompendiumSnapshotDto,
    pub map_navigation: Option<MapNavigationSnapshotDto>,
    pub map: Option<MapViewDto>,
    pub current_node_session: Option<NodeSession>,
    pub visual_novel: Option<VisualNovelSnapshotDto>,
    pub selected_event: Option<SelectedEvent>,
    pub skill_catalog: SkillCatalogDto,
    pub roster: EmployeeRosterSnapshotDto,
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
            action_capabilities,
            run_checkpoint,
            combat_recovery_restriction,
            run_progression,
            abnormality_research,
            monolith,
            encounter_compendium,
            map_navigation,
            map,
            current_node_session,
            visual_novel,
            selected_event,
            skill_catalog,
            roster,
            inventory,
            resources,
        } = self;

        RunSnapshotDto {
            game_state,
            game_state_context,
            action_capabilities,
            run_checkpoint,
            combat_recovery_restriction,
            run_progression,
            abnormality_research,
            monolith,
            encounter_compendium,
            map_navigation,
            map,
            current_node_session,
            visual_novel,
            selected_event: f(selected_event),
            skill_catalog,
            roster,
            inventory,
            resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatRecoveryRestrictionSnapshotDto {
    pub reason: RunFailureReason,
    pub allowed_node_ids: Vec<MapNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCheckpointSnapshotDto {
    pub exists: bool,
    pub loadable: bool,
    pub loads_used: u8,
    pub max_loads: u8,
    pub remaining_loads: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<RunCheckpointLocationDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCheckpointLocationDto {
    pub floor_index: u32,
    pub node_id: MapNodeId,
    pub slot_id: crate::game::map::MapSlotId,
    pub kind_id: MapNodeKindId,
    pub room_display_name: String,
}

/// GameServer에서 GameCore로 전달되는 플레이어 행동
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlayerBehavior {
    // ============================================================
    // 게임 관련 행동
    // ============================================================
    /// 새 게임 시작
    StartNewGame {
        game_mode: GameMode,
        replace_active_run: bool,
    },
    /// Host의 local-install active-run을 마지막 완료 SavePoint에서 이어감
    ContinueRun,
    /// 손상된 local-install active-run 원본을 명시적으로 초기화
    ResetActiveRun,
    /// Local-install encounter compendium bootstrap/save를 명시적으로 재시도
    RetryEncounterCompendium,
    /// 손상된 local-install encounter compendium 원본과 메모리 set을 명시적으로 초기화
    ResetEncounterCompendium,
    /// 시작 후보 직원 중 이번 런에 투입할 3명을 선택
    SelectStarterEmployees {
        employee_definition_ids: Vec<String>,
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
    /// 현재 런 맵 상태 요청
    RequestMapData,
    /// 사용 가능한 맵 노드 선택
    SelectMapNode {
        node_id: MapNodeId,
    },
    /// NodeConfirm의 일반 전투 노드를 선택한 모노리스 재현 노드로 원자적으로 치환
    ConvertMapNodeWithMonolith {
        node_id: MapNodeId,
        abnormality_id: String,
    },
    /// 선택한 맵 노드에 실제로 진입
    ConfirmEnterNode,
    /// 선택한 맵 노드 프리뷰를 취소하고 맵으로 복귀
    CancelSelectedNode,
    /// 현재 노드 처리 완료
    CompleteNode,
    /// Safezone에서 마지막 SavePoint 체크포인트를 불러옴
    LoadRunCheckpoint,
    /// 실패 사전 선택에서 체크포인트를 사용하지 않고 RunFailed를 확정
    ConfirmRunFailure,
    /// 현재 run을 자발적으로 포기
    AbandonRun,
    /// 본사 연락 노드에서 후보 직원 1명을 채용하고 노드를 완료
    RecruitEmployee {
        employee_definition_id: String,
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
        offer_id: Uuid,
    },
    /// 아이템 판매
    SellItem {
        item_uuid: Uuid,
        quantity: u32,
    },
    /// 상점 새로고침 (현재 페이지 전체 교체)
    RefreshShop,
    /// 상점 나가기
    ExitShop,
    // ============================================================
    // 보상 세션 관련 행동
    // ============================================================
    /// 보상 수령
    ClaimReward,
    /// 보상 화면 나가기
    ExitReward,
    /// 선택지가 없는 Event scene을 다음 scene으로 진행
    AdvanceEventScene {
        node_id: MapNodeId,
        event_id: crate::game::data::event_data::EventId,
        current_scene_id: crate::game::data::event_data::EventSceneId,
    },
    /// Event employee check 선택지와 수행자를 확정하고 d20을 한 번 판정
    ResolveEventChoiceCheck {
        node_id: MapNodeId,
        event_id: crate::game::data::event_data::EventId,
        current_scene_id: crate::game::data::event_data::EventSceneId,
        choice_id: crate::game::data::event_data::EventChoiceId,
        employee_uuid: Uuid,
    },
    /// 선택지가 없는 Visual Novel scene을 다음 scene으로 진행하거나 종료
    AdvanceVisualNovel {
        session_id: String,
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId,
    },
    /// Visual Novel scene 선택지를 확정
    SelectVisualNovelChoice {
        session_id: String,
        current_scene_id: crate::game::data::visual_novel_data::VisualNovelSceneId,
        choice_id: crate::game::data::visual_novel_data::VisualNovelChoiceId,
    },
    /// 전투 결과 확인 완료
    CompleteCombatResult,
    /// 실시간 전투 상태 요청
    RequestBattleState {
        #[serde(default)]
        since_seq: Option<u64>,
    },
    /// Unity scene/setup loss recovery. Reprojects immutable setup and current revisioned state.
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
            PlayerBehavior::StartNewGame { .. } => ActionKind::StartNewGame,
            PlayerBehavior::ContinueRun => ActionKind::ContinueRun,
            PlayerBehavior::ResetActiveRun => ActionKind::ResetActiveRun,
            PlayerBehavior::RetryEncounterCompendium => ActionKind::RetryEncounterCompendium,
            PlayerBehavior::ResetEncounterCompendium => ActionKind::ResetEncounterCompendium,
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
            PlayerBehavior::RequestMapData => ActionKind::RequestMapData,
            PlayerBehavior::SelectMapNode { .. } => ActionKind::SelectMapNode,
            PlayerBehavior::ConvertMapNodeWithMonolith { .. } => {
                ActionKind::ConvertMapNodeWithMonolith
            }
            PlayerBehavior::ConfirmEnterNode => ActionKind::ConfirmEnterNode,
            PlayerBehavior::CancelSelectedNode => ActionKind::CancelSelectedNode,
            PlayerBehavior::CompleteNode => ActionKind::CompleteNode,
            PlayerBehavior::LoadRunCheckpoint => ActionKind::LoadRunCheckpoint,
            PlayerBehavior::ConfirmRunFailure => ActionKind::ConfirmRunFailure,
            PlayerBehavior::AbandonRun => ActionKind::AbandonRun,
            PlayerBehavior::RecruitEmployee { .. } => ActionKind::RecruitEmployee,
            PlayerBehavior::RequestEmergencySupplies => ActionKind::RequestEmergencySupplies,
            PlayerBehavior::OpenHeadquartersShop => ActionKind::OpenHeadquartersShop,
            PlayerBehavior::SelectReward { .. } => ActionKind::SelectReward,
            PlayerBehavior::PurchaseItem { .. } => ActionKind::PurchaseItem,
            PlayerBehavior::SellItem { .. } => ActionKind::SellItem,
            PlayerBehavior::RefreshShop => ActionKind::RefreshShop,
            PlayerBehavior::ExitShop => ActionKind::ExitShop,
            PlayerBehavior::ClaimReward => ActionKind::ClaimReward,
            PlayerBehavior::ExitReward => ActionKind::ExitReward,
            PlayerBehavior::AdvanceEventScene { .. } => ActionKind::AdvanceEventScene,
            PlayerBehavior::ResolveEventChoiceCheck { .. } => ActionKind::ResolveEventChoiceCheck,
            PlayerBehavior::AdvanceVisualNovel { .. } => ActionKind::AdvanceVisualNovel,
            PlayerBehavior::SelectVisualNovelChoice { .. } => ActionKind::SelectVisualNovelChoice,
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
    pub eligibility: BattlePayloadEligibilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleDeployedUnitDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub position: Position,
    pub facing: FacingDirection,
    pub withdraw_eligibility: BattlePayloadEligibilityDto,
    pub skill_readiness: LiveBattleSkillReadinessDto,
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
    pub eligibility: BattlePayloadEligibilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattlePlaybackEligibilityDto {
    pub pause: BattlePayloadEligibilityDto,
    pub resume: BattlePayloadEligibilityDto,
    pub speeds: Vec<LiveBattleSpeedEligibilityDto>,
}

impl LiveBattlePlaybackEligibilityDto {
    pub fn for_state(playback: BattlePlaybackState) -> Self {
        let active = ActionDisabledReasonCode::AlreadyActive;
        Self {
            pause: if playback.paused {
                BattlePayloadEligibilityDto::disabled(active)
            } else {
                BattlePayloadEligibilityDto::enabled()
            },
            resume: if playback.paused {
                BattlePayloadEligibilityDto::enabled()
            } else {
                BattlePayloadEligibilityDto::disabled(active)
            },
            speeds: [
                BattlePlaybackSpeed::X0_5,
                BattlePlaybackSpeed::X1,
                BattlePlaybackSpeed::X2,
                BattlePlaybackSpeed::X3,
            ]
            .into_iter()
            .map(|speed| LiveBattleSpeedEligibilityDto {
                speed,
                eligibility: if speed == playback.speed {
                    BattlePayloadEligibilityDto::disabled(active)
                } else {
                    BattlePayloadEligibilityDto::enabled()
                },
            })
            .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSpeedEligibilityDto {
    pub speed: BattlePlaybackSpeed,
    pub eligibility: BattlePayloadEligibilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleUpdateDto {
    #[serde(rename = "type")]
    pub message_type: LiveBattleUpdateMessageType,
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
    pub action_capabilities: Vec<ActionCapabilityDto>,
    pub server_battle_time_ms: u64,
    pub events_delta: LiveBattleEventDeltaDto,
    pub checkpoint: LiveBattleStateCheckpointDto,
    pub visual_novel: Option<VisualNovelSnapshotDto>,
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
    pub events: Vec<LiveBattlePresentationEventDto>,
}

/// Unity-facing presentation event entry for live `battle_update.events_delta`.
///
/// This intentionally mirrors the stable JSON fields Unity consumes without
/// exposing the internal append-only `BattleEventLogEntry` type as the wire
/// boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattlePresentationEventDto {
    pub time_ms: u64,
    pub seq: u64,
    #[serde(default)]
    pub cause: BattleEventCause,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_command_id: Option<String>,
    pub event: LiveBattlePresentationEventKindDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LiveBattlePresentationEventKindDto {
    BattleStart {
        width: u8,
        height: u8,
    },
    ItemSpawned {
        item_instance_id: Uuid,
        owner: Side,
        owner_unit_instance_id: UnitInstanceId,
        base_uuid: Uuid,
    },
    UnitSpawned {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        #[serde(default)]
        role: BattleUnitRole,
        #[serde(default)]
        threat_class: BattleUnitThreatClass,
        #[serde(default)]
        mobility_kind: MobilityKind,
        base_uuid: Uuid,
        unit_source: BattleUnitSourceIdentity,
        world_position: EventLogVec2,
        stats: UnitStats,
    },
    UnitDeployed {
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        position: Position,
        facing: FacingDirection,
    },
    UnitWithdrawn {
        unit_instance_id: UnitInstanceId,
        world_position: EventLogVec2,
        position: Position,
    },
    MovementSegmentStarted {
        unit_instance_id: UnitInstanceId,
        start: EventLogVec2,
        target: EventLogVec2,
        started_at_ms: u64,
        ends_at_ms: u64,
        end_kind: MovementSegmentEndKind,
    },
    MovementStopped {
        unit_instance_id: UnitInstanceId,
        reason: MovementStopReason,
        world_position: EventLogVec2,
        stopped_at_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        until_ms: Option<u64>,
    },
    AttackStart {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackResolve {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    AttackMiss {
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        kind: Option<AttackKind>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery: Option<AttackDelivery>,
    },
    BasicAttackProjectileLaunched {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_impact_time_ms: u64,
    },
    BasicAttackProjectileImpacted {
        projectile_id: Uuid,
        attacker_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        impact_position: EventLogVec2,
        hit: bool,
    },
    AutoCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: Option<SkillId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    AutoCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    ManualCastStart {
        caster_instance_id: UnitInstanceId,
        skill_id: SkillId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
    },
    ManualCastEnd {
        caster_instance_id: UnitInstanceId,
    },
    SkillCastInterrupted {
        interrupter_instance_id: UnitInstanceId,
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
    },
    SkillCastCancelled {
        caster_instance_id: UnitInstanceId,
        interrupted_skill_id: SkillId,
        interrupted_cast_seq: u64,
        reason: SkillCastCancelReason,
    },
    TriggeredAbilityProc {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_instance_id: Option<UnitInstanceId>,
        activation_source: CooldownSource,
        binding_index: usize,
    },
    AbilityCast {
        skill_id: SkillId,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
    },
    AbilityStepTriggered {
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        target_instance_id: Option<UnitInstanceId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        presentation: Option<SkillPresentationDef>,
    },
    SkillAreaDeclared {
        area_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        shape: BattleSkillAreaShape,
        origin: EventLogVec2,
        center: EventLogVec2,
        direction_hint: EventLogVec2,
        start_time_ms: u64,
        duration_ms: u32,
        display_duration_ms: u32,
        #[serde(default)]
        warning_ms: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tick_interval_ms: Option<u32>,
        tick_policy: SkillAreaTickPolicy,
        #[serde(default)]
        tracking: SkillAreaTracking,
        hit_targets: SkillHitTargetFilter,
        include_caster: bool,
    },
    SkillProjectileLaunched {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target: Option<SkillCastTarget>,
        guidance: BattleProjectileGuidance,
        start: EventLogVec2,
        aim: EventLogVec2,
        fired_at_ms: u64,
        expected_end_time_ms: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projectile_vfx_id: Option<String>,
    },
    SkillProjectileImpacted {
        delivery_id: Uuid,
        skill_id: SkillId,
        step_id: String,
        caster_instance_id: UnitInstanceId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first_hit_unit_id: Option<UnitInstanceId>,
        impact_position: EventLogVec2,
        terminal: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        impact_vfx_id: Option<String>,
    },
    BuffApplied {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        duration_ms: u64,
    },
    BuffTick {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
    },
    BuffExpired {
        caster_instance_id: UnitInstanceId,
        target_instance_id: UnitInstanceId,
        buff_id: BuffId,
        reason: BuffExpireReason,
    },
    HpChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        delta: i32,
        hp_before: u32,
        hp_after: u32,
        reason: HpChangeReason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_source: Option<DamageSource>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_type: Option<DamageType>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        raw_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        final_damage: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        damage_breakdown: Option<Vec<DamageBreakdown>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        critical: Option<bool>,
        #[serde(default)]
        feedback_tags: Vec<DamageFeedbackTag>,
    },
    StatChanged {
        source_instance_id: Option<UnitInstanceId>,
        target_instance_id: UnitInstanceId,
        modifier: StatModifier,
        stats_before: UnitStats,
        stats_after: UnitStats,
    },
    ResonanceChanged {
        unit_instance_id: UnitInstanceId,
        before: u32,
        after: u32,
        max: u32,
    },
    UnitDied {
        unit_instance_id: UnitInstanceId,
        owner: Side,
        killer_instance_id: Option<UnitInstanceId>,
        world_position: EventLogVec2,
        position: Position,
    },
    BattleEnd {
        winner: BattleWinner,
    },
}

impl From<BattleLogEvent> for LiveBattlePresentationEventKindDto {
    fn from(value: BattleLogEvent) -> Self {
        match value {
            BattleLogEvent::BattleStart { width, height } => Self::BattleStart { width, height },
            BattleLogEvent::ItemSpawned {
                item_instance_id,
                owner,
                owner_unit_instance_id,
                base_uuid,
            } => Self::ItemSpawned {
                item_instance_id,
                owner,
                owner_unit_instance_id,
                base_uuid,
            },
            BattleLogEvent::UnitSpawned {
                unit_instance_id,
                owner,
                role,
                threat_class,
                mobility_kind,
                base_uuid,
                unit_source,
                world_position,
                stats,
            } => Self::UnitSpawned {
                unit_instance_id,
                owner,
                role,
                threat_class,
                mobility_kind,
                base_uuid,
                unit_source,
                world_position,
                stats,
            },
            BattleLogEvent::UnitDeployed {
                employee_uuid,
                unit_instance_id,
                position,
                facing,
            } => Self::UnitDeployed {
                employee_uuid,
                unit_instance_id,
                position,
                facing,
            },
            BattleLogEvent::UnitWithdrawn {
                unit_instance_id,
                world_position,
                position,
            } => Self::UnitWithdrawn {
                unit_instance_id,
                world_position,
                position,
            },
            BattleLogEvent::MovementSegmentStarted {
                unit_instance_id,
                start,
                target,
                started_at_ms,
                ends_at_ms,
                end_kind,
            } => Self::MovementSegmentStarted {
                unit_instance_id,
                start,
                target,
                started_at_ms,
                ends_at_ms,
                end_kind,
            },
            BattleLogEvent::MovementStopped {
                unit_instance_id,
                reason,
                world_position,
                stopped_at_ms,
                until_ms,
            } => Self::MovementStopped {
                unit_instance_id,
                reason,
                world_position,
                stopped_at_ms,
                until_ms,
            },
            BattleLogEvent::AttackStart {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            } => Self::AttackStart {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            },
            BattleLogEvent::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            } => Self::AttackResolve {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            },
            BattleLogEvent::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            } => Self::AttackMiss {
                attacker_instance_id,
                target_instance_id,
                kind,
                delivery,
            },
            BattleLogEvent::BasicAttackProjectileLaunched {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                start,
                aim,
                fired_at_ms,
                expected_impact_time_ms,
            } => Self::BasicAttackProjectileLaunched {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                start,
                aim,
                fired_at_ms,
                expected_impact_time_ms,
            },
            BattleLogEvent::BasicAttackProjectileImpacted {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                impact_position,
                hit,
            } => Self::BasicAttackProjectileImpacted {
                projectile_id,
                attacker_instance_id,
                target_instance_id,
                impact_position,
                hit,
            },
            BattleLogEvent::AutoCastStart {
                caster_instance_id,
                skill_id,
                target,
            } => Self::AutoCastStart {
                caster_instance_id,
                skill_id,
                target,
            },
            BattleLogEvent::AutoCastEnd { caster_instance_id } => {
                Self::AutoCastEnd { caster_instance_id }
            }
            BattleLogEvent::ManualCastStart {
                caster_instance_id,
                skill_id,
                target,
            } => Self::ManualCastStart {
                caster_instance_id,
                skill_id,
                target,
            },
            BattleLogEvent::ManualCastEnd { caster_instance_id } => {
                Self::ManualCastEnd { caster_instance_id }
            }
            BattleLogEvent::SkillCastInterrupted {
                interrupter_instance_id,
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
            } => Self::SkillCastInterrupted {
                interrupter_instance_id,
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
            },
            BattleLogEvent::SkillCastCancelled {
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
                reason,
            } => Self::SkillCastCancelled {
                caster_instance_id,
                interrupted_skill_id,
                interrupted_cast_seq,
                reason,
            },
            BattleLogEvent::TriggeredAbilityProc {
                skill_id,
                caster_instance_id,
                target_instance_id,
                activation_source,
                binding_index,
            } => Self::TriggeredAbilityProc {
                skill_id,
                caster_instance_id,
                target_instance_id,
                activation_source,
                binding_index,
            },
            BattleLogEvent::AbilityCast {
                skill_id,
                caster_instance_id,
                target_instance_id,
            } => Self::AbilityCast {
                skill_id,
                caster_instance_id,
                target_instance_id,
            },
            BattleLogEvent::AbilityStepTriggered {
                skill_id,
                step_id,
                caster_instance_id,
                target_instance_id,
                presentation,
            } => Self::AbilityStepTriggered {
                skill_id,
                step_id,
                caster_instance_id,
                target_instance_id,
                presentation,
            },
            BattleLogEvent::SkillAreaDeclared {
                area_id,
                skill_id,
                step_id,
                caster_instance_id,
                target,
                shape,
                origin,
                center,
                direction_hint,
                start_time_ms,
                duration_ms,
                display_duration_ms,
                warning_ms,
                tick_interval_ms,
                tick_policy,
                tracking,
                hit_targets,
                include_caster,
            } => Self::SkillAreaDeclared {
                area_id,
                skill_id,
                step_id,
                caster_instance_id,
                target,
                shape,
                origin,
                center,
                direction_hint,
                start_time_ms,
                duration_ms,
                display_duration_ms,
                warning_ms,
                tick_interval_ms,
                tick_policy,
                tracking,
                hit_targets,
                include_caster,
            },
            BattleLogEvent::SkillProjectileLaunched {
                delivery_id,
                skill_id,
                step_id,
                caster_instance_id,
                target,
                guidance,
                start,
                aim,
                fired_at_ms,
                expected_end_time_ms,
                projectile_vfx_id,
            } => Self::SkillProjectileLaunched {
                delivery_id,
                skill_id,
                step_id,
                caster_instance_id,
                target,
                guidance,
                start,
                aim,
                fired_at_ms,
                expected_end_time_ms,
                projectile_vfx_id,
            },
            BattleLogEvent::SkillProjectileImpacted {
                delivery_id,
                skill_id,
                step_id,
                caster_instance_id,
                first_hit_unit_id,
                impact_position,
                terminal,
                impact_vfx_id,
            } => Self::SkillProjectileImpacted {
                delivery_id,
                skill_id,
                step_id,
                caster_instance_id,
                first_hit_unit_id,
                impact_position,
                terminal,
                impact_vfx_id,
            },
            BattleLogEvent::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            } => Self::BuffApplied {
                caster_instance_id,
                target_instance_id,
                buff_id,
                duration_ms,
            },
            BattleLogEvent::BuffTick {
                caster_instance_id,
                target_instance_id,
                buff_id,
            } => Self::BuffTick {
                caster_instance_id,
                target_instance_id,
                buff_id,
            },
            BattleLogEvent::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason,
            } => Self::BuffExpired {
                caster_instance_id,
                target_instance_id,
                buff_id,
                reason,
            },
            BattleLogEvent::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
                damage_source,
                damage_type,
                raw_damage,
                final_damage,
                damage_breakdown,
                critical,
                feedback_tags,
            } => Self::HpChanged {
                source_instance_id,
                target_instance_id,
                delta,
                hp_before,
                hp_after,
                reason,
                damage_source,
                damage_type,
                raw_damage,
                final_damage,
                damage_breakdown,
                critical,
                feedback_tags,
            },
            BattleLogEvent::StatChanged {
                source_instance_id,
                target_instance_id,
                modifier,
                stats_before,
                stats_after,
            } => Self::StatChanged {
                source_instance_id,
                target_instance_id,
                modifier,
                stats_before,
                stats_after,
            },
            BattleLogEvent::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            } => Self::ResonanceChanged {
                unit_instance_id,
                before,
                after,
                max,
            },
            BattleLogEvent::UnitDied {
                unit_instance_id,
                owner,
                killer_instance_id,
                world_position,
                position,
            } => Self::UnitDied {
                unit_instance_id,
                owner,
                killer_instance_id,
                world_position,
                position,
            },
            BattleLogEvent::BattleEnd { winner } => Self::BattleEnd { winner },
        }
    }
}

impl From<BattleEventLogEntry> for LiveBattlePresentationEventDto {
    fn from(value: BattleEventLogEntry) -> Self {
        Self {
            time_ms: value.time_ms,
            seq: value.seq,
            cause: value.cause,
            source_command_id: value.source_command_id,
            event: value.event.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleStateCheckpointDto {
    pub at_seq: u64,
    pub battle_time_ms: u64,
    pub playback: BattlePlaybackState,
    pub playback_eligibility: LiveBattlePlaybackEligibilityDto,
    pub units: Vec<LiveBattleUnitCheckpointDto>,
    pub deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleUnitCheckpointDto {
    pub unit_instance_id: UnitInstanceId,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub hud: LiveBattleUnitHudDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_previews: Option<LiveBattleRangePreviewsDto>,
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
    pub deploy_eligibility: BattlePayloadEligibilityDto,
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
    pub reach: SkillTargetReach,
    pub target_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCatalogStepDto {
    pub step_id: String,
    pub delay_ms: u32,
    pub target_policy: SkillTarget,
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
    pub footprint: TileRangePattern,
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
    /// 새 런 시작 흐름 진입
    RunStartFlowStarted {
        game_mode: GameMode,
        visual_novel: Option<VisualNovelSnapshotDto>,
    },
    /// 시작 직원 선택 완료
    StarterEmployeesSelected {
        selected_employee_definition_ids: Vec<String>,
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
        trauma_recovery_preview: Option<TraumaRecoveryPreviewDto>,
    },
    /// 맵 노드 완료
    NodeCompleted {
        map: MapViewDto,
        outcome: Option<NodeOutcomeSummary>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    /// SavePoint 노드 상태
    SavePointState {
        node_id: MapNodeId,
        recovery_preview: TraumaRecoveryPreviewDto,
    },
    /// 정비 노드 상태
    MaintenanceState {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
    },
    /// 본사 연락 노드 상태
    HeadquartersContactState {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<EmployeeDefinition>,
        shop_pool_id: Option<String>,
    },
    /// Gate 노드 완료로 다음 Floor에 진입
    FloorAdvanced {
        game_mode: GameMode,
        floor_index: u32,
        map: MapViewDto,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
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
    /// checkpoint load 또는 명시적 실패 확정을 기다림
    RunFailureDecisionRequired {
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
        outcome: Option<NodeOutcomeSummary>,
    },
    /// 사용자가 현재 run을 자발적으로 종료함
    RunAbandoned,

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
        active_modifier: Option<ActiveConsumableModifier>,
        condition: EmployeeCondition,
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
    /// 상점 상태 업데이트 (예: 리롤 이후)
    ShopState {
        shop: ShopEventOption,
    },

    RefreshShop {
        enkephalin: u32,
        shop: ShopEventOption,
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
        employee_definition_id: String,
        employee_uuid: Uuid,
        completion: Box<BehaviorResult>,
    },

    /// 본사 연락 노드에서 긴급 보급 수령 완료
    EmergencySuppliesGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        completion: Box<BehaviorResult>,
    },

    /// Event node의 현재 scene 상태
    EventState {
        node_id: MapNodeId,
        event: EventSceneSnapshotDto,
    },

    /// Global Visual Novel overlay 상태
    VisualNovelState {
        visual_novel: Option<VisualNovelSnapshotDto>,
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
    VisualNovel(VisualNovelCommandResult),
    General(GeneralCommandResult),
}

#[derive(Debug, Clone)]
pub enum RunCommandResult {
    RunStartFlowStarted {
        game_mode: GameMode,
        visual_novel: Option<VisualNovelSnapshotDto>,
    },
    StarterEmployeesSelected {
        selected_employee_definition_ids: Vec<String>,
        employee_uuids: Vec<Uuid>,
        map: MapViewDto,
    },
    FloorAdvanced {
        game_mode: GameMode,
        floor_index: u32,
        map: MapViewDto,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    RunComplete {
        map: MapViewDto,
    },
    RunFailed {
        reason: RunFailureReason,
        outcome: Option<NodeOutcomeSummary>,
    },
    RunFailureDecisionRequired {
        reason: RunFailureReason,
        contributing_reason: Option<RunFailureReason>,
        outcome: Option<NodeOutcomeSummary>,
    },
    RunAbandoned,
}

#[derive(Debug, Clone)]
pub enum MapCommandResult {
    MapState { map: MapViewDto },
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
        trauma_recovery_preview: Option<TraumaRecoveryPreviewDto>,
    },
    NodeEntered {
        node_id: MapNodeId,
        kind_id: MapNodeKindId,
        category: MapNodeCategory,
        payload: MapNodePayload,
        session: NodeSession,
    },
    NodeCompleted {
        map: MapViewDto,
        outcome: Option<NodeOutcomeSummary>,
        research_deliveries: Vec<SkillFragmentResearchDelivery>,
    },
    EventState {
        node_id: MapNodeId,
        event: EventSceneSnapshotDto,
    },
}

#[derive(Debug, Clone)]
pub enum FacilityCommandResult {
    SavePointState {
        node_id: MapNodeId,
        recovery_preview: TraumaRecoveryPreviewDto,
    },
    MaintenanceState {
        node_id: MapNodeId,
        maintenance_options: MaintenanceOptionsDto,
    },
    HeadquartersContactState {
        node_id: MapNodeId,
        options: Vec<HeadquartersContactOption>,
        recruitment_candidates: Vec<EmployeeDefinition>,
        shop_pool_id: Option<String>,
    },
    EmployeeRecruited {
        employee_definition_id: String,
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
        active_modifier: Option<ActiveConsumableModifier>,
        condition: EmployeeCondition,
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
        employee_definition_id: String,
        employee_uuid: Uuid,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[derive(Debug, Clone)]
pub enum ShopCommandResult {
    ShopState {
        shop: ShopEventOption,
    },
    RefreshShop {
        enkephalin: u32,
        shop: ShopEventOption,
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
pub enum VisualNovelCommandResult {
    VisualNovelState {
        visual_novel: Option<VisualNovelSnapshotDto>,
    },
}

#[derive(Debug, Clone)]
pub enum GeneralCommandResult {
    Ok,
}

impl BehaviorResult {
    pub fn into_command_result_contract(self) -> BehaviorCommandResultContract {
        match self {
            BehaviorResult::RunStartFlowStarted {
                game_mode,
                visual_novel,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::RunStartFlowStarted {
                    game_mode,
                    visual_novel,
                },
            )),
            BehaviorResult::StarterEmployeesSelected {
                selected_employee_definition_ids,
                employee_uuids,
                map,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::StarterEmployeesSelected {
                    selected_employee_definition_ids,
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
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                NodeCommandResult::NodeEntered {
                    node_id,
                    kind_id,
                    category,
                    payload,
                    session,
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
                trauma_recovery_preview,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                NodeCommandResult::NodePreview {
                    node_id,
                    kind_id,
                    category,
                    payload,
                    session,
                    map,
                    combat_preview,
                    trauma_recovery_preview,
                },
            )),
            BehaviorResult::NodeCompleted {
                map,
                outcome,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                NodeCommandResult::NodeCompleted {
                    map,
                    outcome,
                    research_deliveries,
                },
            )),
            BehaviorResult::EventState { node_id, event } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                    NodeCommandResult::EventState { node_id, event },
                ))
            }
            BehaviorResult::VisualNovelState { visual_novel } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::VisualNovel(
                        VisualNovelCommandResult::VisualNovelState { visual_novel },
                    ),
                )
            }
            BehaviorResult::SavePointState {
                node_id,
                recovery_preview,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::SavePointState {
                    node_id,
                    recovery_preview,
                }),
            ),
            BehaviorResult::MaintenanceState {
                node_id,
                maintenance_options,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::MaintenanceState {
                    node_id,
                    maintenance_options,
                }),
            ),
            BehaviorResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(
                    FacilityCommandResult::HeadquartersContactState {
                        node_id,
                        options,
                        recruitment_candidates,
                        shop_pool_id,
                    },
                ),
            ),
            BehaviorResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
                research_deliveries,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::FloorAdvanced {
                    game_mode,
                    floor_index,
                    map,
                    research_deliveries,
                },
            )),
            BehaviorResult::RunComplete { map } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Run(RunCommandResult::RunComplete { map }),
            ),
            BehaviorResult::RunFailed { reason, outcome } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                    RunCommandResult::RunFailed { reason, outcome },
                ))
            }
            BehaviorResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::RunFailureDecisionRequired {
                    reason,
                    contributing_reason,
                    outcome,
                },
            )),
            BehaviorResult::RunAbandoned => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Run(RunCommandResult::RunAbandoned),
            ),
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
                active_modifier,
                condition,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::ConsumableItemUsed {
                        item_uuid,
                        target_employee_uuid,
                        replaced_modifier,
                        active_modifier,
                        condition,
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
            BehaviorResult::ShopState { shop } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Shop(ShopCommandResult::ShopState { shop }),
            ),
            BehaviorResult::RefreshShop { enkephalin, shop } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Shop(
                    ShopCommandResult::RefreshShop { enkephalin, shop },
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
                employee_definition_id,
                employee_uuid,
                completion,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Facility(FacilityCommandResult::EmployeeRecruited {
                    employee_definition_id,
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
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::RewardState {
                    mode,
                    rewards,
                    selected_reward_uuid,
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

    pub fn is_run_start_flow_started(&self) -> bool {
        matches!(self, BehaviorResult::RunStartFlowStarted { .. })
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

    /// RefreshShop → 갱신된 Shop projection 참조 반환
    pub fn as_refresh_shop(&self) -> Option<(u32, &ShopEventOption)> {
        match self {
            BehaviorResult::RefreshShop { enkephalin, shop } => Some((*enkephalin, shop)),
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
            CommandResultPayloadContract::VisualNovel(result) => result.result_type(),
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
            CommandResultPayloadContract::VisualNovel(result) => result.payload_value(),
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
            RunCommandResult::RunStartFlowStarted { .. } => "RunStartFlowStarted",
            RunCommandResult::StarterEmployeesSelected { .. } => "StarterEmployeesSelected",
            RunCommandResult::FloorAdvanced { .. } => "FloorAdvanced",
            RunCommandResult::RunComplete { .. } => "RunComplete",
            RunCommandResult::RunFailed { .. } => "RunFailed",
            RunCommandResult::RunFailureDecisionRequired { .. } => "RunFailureDecisionRequired",
            RunCommandResult::RunAbandoned => "RunAbandoned",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            RunCommandResult::RunStartFlowStarted {
                game_mode,
                visual_novel,
            } => Ok(json!({
                "game_mode": game_mode,
                "visual_novel": visual_novel,
            })),
            RunCommandResult::StarterEmployeesSelected {
                selected_employee_definition_ids,
                employee_uuids,
                map,
            } => Ok(json!({
                "selected_employee_definition_ids": selected_employee_definition_ids,
                "employee_uuids": employee_uuids,
                "map": map,
            })),
            RunCommandResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
                research_deliveries,
            } => Ok(json!({
                "game_mode": game_mode,
                "floor_index": floor_index,
                "map": map,
                "research_deliveries": research_deliveries,
            })),
            RunCommandResult::RunComplete { map } => serde_json::to_value(map),
            RunCommandResult::RunFailed { reason, outcome } => Ok(json!({
                "reason": reason,
                "outcome": outcome,
            })),
            RunCommandResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            } => Ok(json!({
                "reason": reason,
                "contributing_reason": contributing_reason,
                "outcome": outcome,
            })),
            RunCommandResult::RunAbandoned => Ok(json!({})),
        }
    }
}

impl MapCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            MapCommandResult::MapState { .. } => "MapState",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            MapCommandResult::MapState { map } => Ok(json!({ "map": map })),
        }
    }
}

impl NodeCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            NodeCommandResult::NodePreview { .. } => "NodePreview",
            NodeCommandResult::NodeEntered { .. } => "NodeEntered",
            NodeCommandResult::NodeCompleted { .. } => "NodeCompleted",
            NodeCommandResult::EventState { .. } => "EventState",
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
                trauma_recovery_preview,
            } => Ok(json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
                "map": map,
                "combat_preview": combat_preview,
                "trauma_recovery_preview": trauma_recovery_preview,
            })),
            NodeCommandResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
            } => Ok(json!({
                "node_id": node_id,
                "kind_id": kind_id,
                "category": category,
                "payload": payload,
                "session": session,
            })),
            NodeCommandResult::NodeCompleted {
                map,
                outcome,
                research_deliveries,
            } => Ok(json!({
                "map": map,
                "outcome": outcome,
                "research_deliveries": research_deliveries,
            })),
            NodeCommandResult::EventState { node_id, event } => Ok(json!({
                "node_id": node_id,
                "event": event,
            })),
        }
    }
}

impl FacilityCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            FacilityCommandResult::SavePointState { .. } => "SavePointState",
            FacilityCommandResult::MaintenanceState { .. } => "MaintenanceState",
            FacilityCommandResult::HeadquartersContactState { .. } => "HeadquartersContactState",
            FacilityCommandResult::EmployeeRecruited { .. } => "EmployeeRecruited",
            FacilityCommandResult::EmergencySuppliesGranted { .. } => "EmergencySuppliesGranted",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            FacilityCommandResult::SavePointState {
                node_id,
                recovery_preview,
            } => Ok(json!({
                "node_id": node_id,
                "recovery_preview": recovery_preview,
            })),
            FacilityCommandResult::MaintenanceState {
                node_id,
                maintenance_options,
            } => Ok(json!({
                "node_id": node_id,
                "maintenance_options": maintenance_options,
            })),
            FacilityCommandResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
            } => Ok(json!({
                "node_id": node_id,
                "options": options,
                "recruitment_candidates": recruitment_candidates,
                "shop_pool_id": shop_pool_id,
            })),
            FacilityCommandResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion,
            } => Ok(json!({
                "employee_definition_id": employee_definition_id,
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
                active_modifier,
                condition,
                inventory_diff,
            } => Ok(json!({
                "item_uuid": item_uuid,
                "target_employee_uuid": target_employee_uuid,
                "replaced_modifier": replaced_modifier,
                "active_modifier": active_modifier,
                "condition": condition,
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
                employee_definition_id,
                employee_uuid,
                completion,
            } => Ok(json!({
                "employee_definition_id": employee_definition_id,
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
            ShopCommandResult::RefreshShop { .. } => "RefreshShop",
            ShopCommandResult::SellItem { .. } => "SellItem",
            ShopCommandResult::PurchaseItem { .. } => "PurchaseItem",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            ShopCommandResult::ShopState { shop } => Ok(json!({
                "shop": shop,
            })),
            ShopCommandResult::RefreshShop { enkephalin, shop } => Ok(json!({
                "enkephalin": enkephalin,
                "shop": shop,
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
            } => Ok(json!({
                "mode": mode,
                "rewards": rewards,
                "selected_reward_uuid": selected_reward_uuid,
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

impl VisualNovelCommandResult {
    fn result_type(&self) -> &'static str {
        match self {
            VisualNovelCommandResult::VisualNovelState { .. } => "VisualNovelState",
        }
    }

    fn payload_value(&self) -> serde_json::Result<Value> {
        match self {
            VisualNovelCommandResult::VisualNovelState { visual_novel } => Ok(json!({
                "visual_novel": visual_novel,
            })),
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
    /// A context capability or selected payload failed the shared typed preflight.
    ActionUnavailable {
        action: ActionKind,
        reason: ActionDisabledReasonCode,
    },
    /// 클라이언트가 현재 전투에 존재하지 않는 미래 presentation seq로 resync를 요청했을 때
    InvalidBattleResyncSeq { requested: u64, latest: u64 },
    /// 전투 상태 revision을 더 이상 증가시킬 수 없을 때
    BattleStateRevisionOverflow,
    /// 활성 전투 mutation 명령이 필수 UUID/revision authority를 보내지 않았을 때
    MissingBattleStateAuthority { current: BattleStateAuthorityDto },
    /// 전투 mutation 명령이 현재 Core authority와 다른 battle UUID/revision을 제시했을 때
    StaleBattleStateAuthority {
        expected: BattleStateAuthorityDto,
        current: BattleStateAuthorityDto,
    },

    /// 상점 상태가 아니거나, 활성 노드 콘텐츠에 Shop 정보가 없을 때
    NotInShopState,
    /// 보상 상태가 아니거나, 활성 노드 콘텐츠에 Reward 정보가 없을 때
    NotInRewardState,
    /// 현재 Shop session에서 offer identity를 찾지 못했을 때
    ShopItemNotFound,

    /// 인벤토리가 가득 차서 아이템을 추가할 수 없을 때 (예: 아티팩트 슬롯)
    InventoryFull,
    /// 인벤토리에서 아이템을 찾을 수 없을 때
    InventoryItemNotFound,
    /// 인벤토리에 존재하지만 해당 경로로 제거할 수 없는 아이템일 때
    InventoryItemNotRemovable,
    /// 유일해야 하는 아티팩트를 이미 보유 중일 때

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
    use crate::game::battle::event_log::{BattleEventCause, BattleEventRootCause, BattleLogEvent};
    use crate::game::data::shop_data::ShopContext;
    use crate::game::resources::InventoryDiffDto;

    #[test]
    fn monolith_reward_wire_contract_is_qualitative_only() {
        let dto = MonolithConversionOptionDto {
            abnormality_id: "abno".to_string(),
            abnormality_uuid: Uuid::nil(),
            display_name: "Abnormality".to_string(),
            risk_level: RiskLevel::ZAYIN,
            enabled: true,
            disabled_reason: None,
            next_stage: 3,
            uses_before: 2,
            uses_after: 1,
            max_attempts: 3,
            reward_tier: MonolithRewardTierDto::FinalEnhanced,
        };

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["reward_tier"], "final_enhanced");
        assert!(value.get("reward_multiplier_percent").is_none());
    }

    #[test]
    fn resolve_event_choice_check_uses_one_exact_scene_qualified_wire_command() {
        let node_id = MapNodeId::new(Uuid::from_u128(0xE001));
        let employee_uuid = Uuid::from_u128(0xE002);
        let behavior = PlayerBehavior::ResolveEventChoiceCheck {
            node_id,
            event_id: crate::game::data::event_data::EventId::new("event_a"),
            current_scene_id: crate::game::data::event_data::EventSceneId::new("check"),
            choice_id: crate::game::data::event_data::EventChoiceId::new("choice_a"),
            employee_uuid,
        };
        let value = serde_json::to_value(&behavior).expect("event command serializes");
        assert_eq!(
            value,
            serde_json::json!({
                "type": "resolve_event_choice_check",
                "node_id": node_id.0.to_string(),
                "event_id": "event_a",
                "current_scene_id": "check",
                "choice_id": "choice_a",
                "employee_uuid": employee_uuid.to_string(),
            })
        );
        assert!(serde_json::from_value::<PlayerBehavior>(value).is_ok());
        for removed in [
            serde_json::json!({
                "type": "select_event_choice",
                "node_id": node_id.0.to_string(),
                "event_id": "event_a",
                "choice_id": "choice_a",
            }),
            serde_json::json!({
                "type": "resolve_event_choice_check",
                "node_id": node_id.0.to_string(),
                "event_id": "event_a",
                "choice_id": "choice_a",
                "employee_uuid": employee_uuid.to_string(),
            }),
            serde_json::json!({
                "type": "resolve_event_choice_check",
                "node_id": node_id.0.to_string(),
                "event_id": "event_a",
                "current_scene_id": "check",
                "choice_id": "choice_a",
                "employee_uuid": employee_uuid.to_string(),
                "modifier_total": 5,
            }),
        ] {
            assert!(serde_json::from_value::<PlayerBehavior>(removed).is_err());
        }
    }

    #[test]
    fn action_capability_serialization_is_typed_and_omits_enabled_reason() {
        let enabled = ActionCapabilityDto::enabled(ActionKind::RequestMapData, ActionScope::Query);
        assert_eq!(
            serde_json::to_value(enabled).unwrap(),
            serde_json::json!({
                "action": "RequestMapData",
                "scope": "query",
                "enabled": true
            })
        );

        let disabled = ActionCapabilityDto::disabled(
            ActionKind::LoadRunCheckpoint,
            ActionScope::Recovery,
            ActionDisabledReasonCode::CheckpointUnavailable,
        );
        assert_eq!(
            serde_json::to_value(disabled).unwrap(),
            serde_json::json!({
                "action": "LoadRunCheckpoint",
                "scope": "recovery",
                "enabled": false,
                "disabled_reason_code": "checkpoint_unavailable"
            })
        );
    }

    #[test]
    fn shop_commands_require_offer_identity_quantity_and_refresh_vocabulary() {
        let offer_id = Uuid::from_u128(0x5100);
        let item_uuid = Uuid::from_u128(0x5200);
        assert_eq!(
            serde_json::to_value(PlayerBehavior::PurchaseItem { offer_id }).unwrap(),
            serde_json::json!({
                "type": "purchase_item",
                "offer_id": offer_id,
            })
        );
        assert_eq!(
            serde_json::to_value(PlayerBehavior::SellItem {
                item_uuid,
                quantity: 3,
            })
            .unwrap(),
            serde_json::json!({
                "type": "sell_item",
                "item_uuid": item_uuid,
                "quantity": 3,
            })
        );
        assert_eq!(
            serde_json::to_value(PlayerBehavior::RefreshShop).unwrap(),
            serde_json::json!({ "type": "refresh_shop" })
        );
        for removed in [
            serde_json::json!({ "type": "reroll_shop" }),
            serde_json::json!({ "type": "purchase_item", "item_uuid": item_uuid }),
            serde_json::json!({ "type": "sell_item", "item_uuid": item_uuid }),
        ] {
            assert!(serde_json::from_value::<PlayerBehavior>(removed).is_err());
        }
    }

    #[test]
    fn behavior_result_helpers_match_variants() {
        assert!(BehaviorResult::RunStartFlowStarted {
            game_mode: GameMode::Standard,
            visual_novel: None,
        }
        .is_run_start_flow_started());
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
            context: ShopContext::UnknownExploration,
            offers: vec![],
            refresh: None,
            selling_enabled: false,
        };
        let shop_state = BehaviorResult::ShopState { shop: shop.clone() };
        assert_eq!(shop_state.as_shop_state(), Some(&shop));
    }

    #[test]
    fn research_delivery_is_present_only_on_authoritative_planning_return_payloads() {
        let map = MapViewDto {
            map_template_id: crate::game::map::MapTemplateId::new("delivery_contract"),
            facility_template_version: crate::game::map::FACILITY_TEMPLATE_VERSION,
            nodes: vec![],
        };
        let delivery = SkillFragmentResearchDelivery {
            fragment_id: SkillFragmentId::from("fragment_delivery_contract"),
            count: 1,
            total_count: 2,
        };
        let BehaviorCommandResultContract::CommandPayload(completed) =
            (BehaviorResult::NodeCompleted {
                map: map.clone(),
                outcome: None,
                research_deliveries: vec![delivery.clone()],
            })
            .into_command_result_contract()
        else {
            panic!("node completion must be a command payload");
        };
        assert_eq!(completed.result_type(), "NodeCompleted");
        let completed_json = completed.payload_value().unwrap();
        assert_eq!(
            completed_json["research_deliveries"][0]["fragment_id"],
            delivery.fragment_id.as_str()
        );
        assert_eq!(completed_json["research_deliveries"][0]["count"], 1);

        let BehaviorCommandResultContract::CommandPayload(floor_advanced) =
            (BehaviorResult::FloorAdvanced {
                game_mode: GameMode::Standard,
                floor_index: 1,
                map,
                research_deliveries: vec![delivery],
            })
            .into_command_result_contract()
        else {
            panic!("floor advance must be a command payload");
        };
        assert_eq!(floor_advanced.result_type(), "FloorAdvanced");
        assert!(floor_advanced.payload_value().unwrap()["research_deliveries"].is_array());

        let BehaviorCommandResultContract::CommandPayload(save_point) =
            (BehaviorResult::SavePointState {
                node_id: MapNodeId::new(Uuid::from_u128(0xD311)),
                recovery_preview: TraumaRecoveryPreviewDto {
                    recovery: TraumaRecovery::CurrentPercent { percent: 15 },
                    employees: vec![],
                    physical_damage_unchanged: true,
                    replaces_run_checkpoint: true,
                },
            })
            .into_command_result_contract()
        else {
            panic!("SavePoint state must be a command payload");
        };
        assert!(save_point
            .payload_value()
            .unwrap()
            .get("research_deliveries")
            .is_none());
        assert_eq!(
            save_point.payload_value().unwrap()["recovery_preview"]["recovery"]["CurrentPercent"]
                ["percent"],
            15
        );
    }

    #[test]
    fn live_battle_presentation_event_dto_keeps_stable_wire_shape() {
        let entry = BattleEventLogEntry {
            time_ms: 100,
            seq: 7,
            cause: BattleEventCause::Root {
                kind: BattleEventRootCause::Period,
            },
            source_command_id: Some("command-1".to_string()),
            event: BattleLogEvent::BattleStart {
                width: 9,
                height: 7,
            },
        };

        let dto = LiveBattlePresentationEventDto::from(entry);
        let json = serde_json::to_value(dto).expect("presentation event dto should serialize");

        assert_eq!(json["time_ms"], 100);
        assert_eq!(json["seq"], 7);
        assert_eq!(json["cause"]["cause_type"], "root");
        assert_eq!(json["cause"]["kind"], "period");
        assert_eq!(json["source_command_id"], "command-1");
        assert_eq!(json["event"]["type"], "BattleStart");
        assert_eq!(json["event"]["width"], 9);
        assert_eq!(json["event"]["height"], 7);
    }

    #[test]
    fn live_battle_presentation_event_kind_preserves_internal_event_json_shape() {
        let internal = BattleLogEvent::MovementStopped {
            unit_instance_id: UnitInstanceId::from(Uuid::from_u128(0xA)),
            reason: MovementStopReason::CastStarted,
            world_position: EventLogVec2 {
                x_milli: 1_250,
                y_milli: 2_500,
            },
            stopped_at_ms: 1_234,
            until_ms: Some(2_000),
        };
        let expected_json =
            serde_json::to_value(internal.clone()).expect("internal event should serialize");
        let wire_json = serde_json::to_value(LiveBattlePresentationEventKindDto::from(internal))
            .expect("wire event should serialize");

        assert_eq!(wire_json, expected_json);
        assert_eq!(wire_json["type"], "MovementStopped");
        assert_eq!(wire_json["stopped_at_ms"], 1_234);
    }
}
