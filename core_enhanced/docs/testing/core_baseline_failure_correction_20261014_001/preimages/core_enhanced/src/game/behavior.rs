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
            SkillAreaTracking, SkillCastSelection, SkillHitTargetFilter, SkillId, SkillKind,
            SkillPresentationDef, SkillTarget, SkillTargetReach, SkillTileAreaOrigin,
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
                AttackDelivery, AttackKind, BattleEventCause, BattleEventLogEntry, BattleLogEvent,
                BattleProjectileGuidance, BattleSkillAreaShape, BuffExpireReason, HpChangeReason,
                MovementStopReason, SkillCastCancelReason, SkillCastTarget,
            },
            ids::UnitInstanceId,
            result_stats::BattleResultStatsDto,
            tile_range::{FacingDirection, TileRangePattern},
            types::{
                BattleUnitRole, BattleUnitSourceIdentity, BattleUnitThreatClass, BattleWinner,
                DeploymentAffinity, EngagementStance, MobilityKind, TacticalTileKind,
                UnitExclusiveActivity,
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
        reward::{RewardEffect, RewardOption, SkillFragmentGrantDiffDto},
        skill_fragment::OwnedFragment,
        stats::{StatModifier, UnitStats},
    },
};

#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
use crate::game::battle::event_log::BattleEventLog;

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
    pub outcome_id: crate::game::battle::types::BattleOutcomeId,
    pub verdict: crate::game::battle::types::BattleVerdict,
    pub winner: BattleWinner,
    pub retreated: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbnormalityAttemptDto {
    pub node_id: MapNodeId,
    pub max_attempts: u8,
    pub attempts_started: u8,
    pub remaining_attempts: u8,
    pub retreats_used: u8,
    pub max_retreats: u8,
    pub remaining_retreats: u8,
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
    #[serde(default)]
    pub fragment_forms: Vec<FragmentFormSnapshotDto>,
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
    pub fragment_copy_uuid: Option<Uuid>,
    pub selected_form: Option<String>,
    pub unlocked_forms: Vec<String>,
    pub owner: Option<Uuid>,
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

macro_rules! define_action_kinds {
    ($($variant:ident),+ $(,)?) => {
        /// Stable command-family identity. This is not an authorization token.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        pub enum ActionKind {
            $($variant),+
        }

        impl ActionKind {
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

define_action_kinds!(
    StartNewGame,
    SkipTutorial,
    AcknowledgeFirstReturnStory,
    ChooseExpeditionContinuation,
    ContinueRun,
    ResetActiveRun,
    RetryPlayerProfile,
    ResetPlayerProfile,
    SelectStarterEmployees,
    UnEquipItem,
    EquipItem,
    UseConsumableItem,
    EquipSkillFragment,
    UnequipSkillFragment,
    SelectSkillFragmentForm,
    TransferSkillFragment,
    AwakenSkillFragment,
    DismantleSkillFragment,
    DismantleEquipment,
    EnhanceEquipment,
    EquipRelic,
    UnequipRelic,
    RequestRelicState,
    RequestRelicNumericPlan,
    RequestRelicEffectivePlan,
    PreviewRelicCorruption,
    CorruptRelic,
    PreviewRelicDismantle,
    DismantleRelic,
    RequestMapData,
    MoveToFacilityRoom,
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
    ChooseRelicCorruption,
    AdvanceVisualNovel,
    SelectVisualNovelChoice,
    SweepMasteredElite,
    CompleteMasteredEliteSweep,
    CompleteCombatResult,
    RequestBattleState,
    RecoverBattleSetupLoss,
    RequestDeploymentRangePreview,
    DeployUnit,
    WithdrawUnit,
    SetEngagementStance,
    TacticalRelocateUnit,
    TacticalSwapUnits,
    FocusBattleUnit,
    ClearBattleFocus,
    BeginDeploymentSelection,
    CancelDeploymentSelection,
    OpenBattleSettings,
    CloseBattleSettings,
    BeginSkillTargeting,
    CancelSkillTargeting,
    PreviewSkillTarget,
    PreviewTacticalCommand,
    ActivateSkill,
    RetreatBattle,
    PauseBattle,
    ResumeBattle,
    SetBattleSpeed,
);

/// Typed command routing fact. Availability and payload validation remain domain-owned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionRoute {
    /// Shared root-capability preflight owns admission.
    CoreCommon,
    /// Root capability is projected, but the concrete domain owns admission.
    CoreDomain,
    /// Revisioned battle capability preflight owns admission.
    CoreBattle,
    /// Visual Novel admission is common outside battle and battle-owned inside battle.
    CoreContextual,
    /// The server host owns persistence/recovery execution.
    HostRecovery(HostRecoveryAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostRecoveryAction {
    ContinueRun,
    ResetActiveRun,
    RetryPlayerProfile,
    ResetPlayerProfile,
}

impl ActionRoute {
    pub(crate) const fn projects_common_capability(self) -> bool {
        matches!(
            self,
            Self::CoreCommon | Self::CoreDomain | Self::CoreContextual
        )
    }

    pub(crate) const fn uses_common_preflight(self) -> bool {
        matches!(self, Self::CoreCommon | Self::CoreContextual)
    }

    pub(crate) const fn uses_battle_preflight(self) -> bool {
        matches!(self, Self::CoreBattle | Self::CoreContextual)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActionDescriptor {
    route: ActionRoute,
    scope: ActionScope,
    requires_battle_authority: bool,
}

impl ActionDescriptor {
    const fn new(route: ActionRoute, scope: ActionScope, requires_battle_authority: bool) -> Self {
        Self {
            route,
            scope,
            requires_battle_authority,
        }
    }
}

impl ActionKind {
    const fn descriptor(self) -> ActionDescriptor {
        use ActionRoute::{CoreBattle, CoreCommon, CoreContextual, CoreDomain, HostRecovery};
        use ActionScope::{Gameplay, Query, Recovery};
        use HostRecoveryAction::{
            ContinueRun as HostContinueRun, ResetActiveRun as HostResetActiveRun,
            ResetPlayerProfile as HostResetPlayerProfile,
            RetryPlayerProfile as HostRetryPlayerProfile,
        };

        match self {
            Self::ContinueRun => {
                ActionDescriptor::new(HostRecovery(HostContinueRun), Recovery, false)
            }
            Self::ResetActiveRun => {
                ActionDescriptor::new(HostRecovery(HostResetActiveRun), Recovery, false)
            }
            Self::RetryPlayerProfile => {
                ActionDescriptor::new(HostRecovery(HostRetryPlayerProfile), Recovery, false)
            }
            Self::ResetPlayerProfile => {
                ActionDescriptor::new(HostRecovery(HostResetPlayerProfile), Recovery, false)
            }

            Self::UnEquipItem
            | Self::EquipItem
            | Self::UseConsumableItem
            | Self::EquipSkillFragment
            | Self::UnequipSkillFragment
            | Self::TransferSkillFragment
            | Self::EquipRelic
            | Self::UnequipRelic
            | Self::CorruptRelic
            | Self::DismantleRelic
            | Self::ConvertMapNodeWithMonolith
            | Self::PurchaseItem
            | Self::SellItem
            | Self::RefreshShop
            | Self::ExitShop => ActionDescriptor::new(CoreDomain, Gameplay, false),

            Self::AdvanceVisualNovel | Self::SelectVisualNovelChoice => {
                ActionDescriptor::new(CoreContextual, Gameplay, true)
            }

            Self::RequestBattleState
            | Self::RequestDeploymentRangePreview
            | Self::PreviewSkillTarget
            | Self::PreviewTacticalCommand => ActionDescriptor::new(CoreBattle, Query, false),
            Self::RecoverBattleSetupLoss => ActionDescriptor::new(CoreBattle, Recovery, false),
            Self::DeployUnit
            | Self::WithdrawUnit
            | Self::SetEngagementStance
            | Self::TacticalRelocateUnit
            | Self::TacticalSwapUnits
            | Self::FocusBattleUnit
            | Self::ClearBattleFocus
            | Self::BeginDeploymentSelection
            | Self::CancelDeploymentSelection
            | Self::OpenBattleSettings
            | Self::CloseBattleSettings
            | Self::BeginSkillTargeting
            | Self::CancelSkillTargeting
            | Self::ActivateSkill
            | Self::RetreatBattle
            | Self::PauseBattle
            | Self::ResumeBattle
            | Self::SetBattleSpeed => ActionDescriptor::new(CoreBattle, Gameplay, true),

            Self::RequestMapData
            | Self::RequestRelicState
            | Self::RequestRelicNumericPlan
            | Self::RequestRelicEffectivePlan
            | Self::PreviewRelicCorruption
            | Self::PreviewRelicDismantle => ActionDescriptor::new(CoreCommon, Query, false),
            Self::LoadRunCheckpoint => ActionDescriptor::new(CoreCommon, Recovery, false),
            Self::StartNewGame
            | Self::SkipTutorial
            | Self::AcknowledgeFirstReturnStory
            | Self::ChooseExpeditionContinuation
            | Self::SelectStarterEmployees
            | Self::SelectSkillFragmentForm
            | Self::AwakenSkillFragment
            | Self::DismantleSkillFragment
            | Self::DismantleEquipment
            | Self::EnhanceEquipment
            | Self::MoveToFacilityRoom
            | Self::SelectMapNode
            | Self::ConfirmEnterNode
            | Self::CancelSelectedNode
            | Self::CompleteNode
            | Self::ConfirmRunFailure
            | Self::AbandonRun
            | Self::RecruitEmployee
            | Self::RequestEmergencySupplies
            | Self::OpenHeadquartersShop
            | Self::SelectReward
            | Self::ClaimReward
            | Self::ExitReward
            | Self::AdvanceEventScene
            | Self::ResolveEventChoiceCheck
            | Self::ChooseRelicCorruption
            | Self::SweepMasteredElite
            | Self::CompleteMasteredEliteSweep
            | Self::CompleteCombatResult => ActionDescriptor::new(CoreCommon, Gameplay, false),
        }
    }

    pub const fn route(self) -> ActionRoute {
        self.descriptor().route
    }

    pub const fn scope(self) -> ActionScope {
        self.descriptor().scope
    }

    pub const fn requires_battle_authority(self) -> bool {
        self.descriptor().requires_battle_authority
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
    CorrosionForcedPause,
    SettingsOpen,
    SelectionActive,
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
    StanceCooldown,
    ForcedDefense,
    DeploymentLimit,
    RelocationCooldown,
    UnitUnavailable,
    WeaponRequired,
    EmployeeDead,
    CriticalPhysicalDamage,
    Breakdown,
    NoTrauma,
    SkillUnavailable,
    ActivationModeNotManual,
    HardCrowdControl,
    Silenced,
    DefenseStance,
    DefenseRequired,
    ActionInProgress,
    ActionLocked,
    FieldWorking,
    ResonanceNotFull,
    SkillTargetRequired,
    SkillTargetTypeMismatch,
    NoValidTarget,
    OfferUnavailable,
    RefreshUnavailable,
    RefreshUsesExhausted,
    PoolExhausted,
    SellingUnavailable,
    InvalidQuantity,
    InvalidQuote,
    ItemBound,
    RelicNotSellable,
    ItemEquipped,
    TagNotAccepted,
}

impl ActionDisabledReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CorrosionForcedPause => "corrosion_forced_pause",
            Self::SettingsOpen => "settings_open",
            Self::SelectionActive => "selection_active",
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
            Self::StanceCooldown => "stance_cooldown",
            Self::ForcedDefense => "forced_defense",
            Self::DeploymentLimit => "deployment_limit",
            Self::RelocationCooldown => "relocation_cooldown",
            Self::UnitUnavailable => "unit_unavailable",
            Self::WeaponRequired => "weapon_required",
            Self::EmployeeDead => "employee_dead",
            Self::CriticalPhysicalDamage => "critical_physical_damage",
            Self::Breakdown => "breakdown",
            Self::NoTrauma => "no_trauma",
            Self::SkillUnavailable => "skill_unavailable",
            Self::ActivationModeNotManual => "activation_mode_not_manual",
            Self::HardCrowdControl => "hard_crowd_control",
            Self::Silenced => "silenced",
            Self::DefenseStance => "defense_stance",
            Self::DefenseRequired => "defense_required",
            Self::ActionInProgress => "action_in_progress",
            Self::ActionLocked => "action_locked",
            Self::FieldWorking => "field_working",
            Self::ResonanceNotFull => "resonance_not_full",
            Self::SkillTargetRequired => "skill_target_required",
            Self::SkillTargetTypeMismatch => "skill_target_type_mismatch",
            Self::NoValidTarget => "no_valid_target",
            Self::OfferUnavailable => "offer_unavailable",
            Self::RefreshUnavailable => "refresh_unavailable",
            Self::RefreshUsesExhausted => "refresh_uses_exhausted",
            Self::PoolExhausted => "pool_exhausted",
            Self::SellingUnavailable => "selling_unavailable",
            Self::InvalidQuantity => "invalid_quantity",
            Self::InvalidQuote => "invalid_quote",
            Self::ItemBound => "item_bound",
            Self::RelicNotSellable => "relic_not_sellable",
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
    pub instance_namespace: Uuid,
    pub expedition_instance: Option<crate::game::expedition_lifecycle::ExpeditionInstance>,
    pub deterrence: u32,
    pub pending_continuation: Option<crate::game::expedition_lifecycle::PendingExpeditionContinuation>,
    pub game_mode: GameMode,
    pub floor_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_floors: Option<u8>,
    pub current_floor_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumSnapshotDto {
    pub abnormalities: Vec<EncounterCompendiumAbnormalitySnapshotDto>,
    pub corrosion_profiles: Vec<EncounterCompendiumCorrosionSnapshotDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfileSnapshotDto {
    pub lifecycle: crate::game::expedition_lifecycle::ExpeditionLifecycle,
    pub highest_floor_reached: u32,
    pub longest_deathless_floor_streak: u32,
    pub all_live_final_bosses_defeated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumAbnormalitySnapshotDto {
    pub abnormality_id: String,
    pub registered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_level: Option<RiskLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterCompendiumCorrosionSnapshotDto {
    pub profile_id: String,
    pub registered: bool,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relic_corruption_selections:
        Option<Vec<crate::game::resources::CommittedRelicCorruptionSelection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum EventInteractionDto {
    Advance,
    RelicCorruption {
        candidates: Vec<RelicCorruptionCandidateDto>,
        can_decline: bool,
        catalyst_cost: u32,
    },
    EmployeeCheckChoices {
        choices: Vec<EventChoiceEvaluationDto>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelicCorruptionCandidateDto {
    pub copy: crate::game::relic::OwnedRelic,
    pub eligibility: PayloadEligibilityDto,
    pub effects: Vec<crate::game::data::relic_data::RelicDamageEffect>,
    pub penalties: Vec<crate::game::data::relic_data::RelicPenalty>,
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
    pub remanifestation_failed: bool,
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
pub struct FragmentFormSnapshotDto {
    pub form_id: String,
    pub skill_id: crate::game::ability::SkillId,
    pub resonance_max: u32,
    pub unlocked: bool,
    pub selected: bool,
    pub can_unlock: bool,
    pub can_select: bool,
    pub unlock_unavailable_reason: Option<String>,
    pub select_unavailable_reason: Option<String>,
    pub recipe: Option<crate::game::data::skill_fragment_data::SkillFragmentRecipe>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySkillFragmentSnapshotDto {
    pub copy: OwnedFragment,
    pub definition_count: u32,
    pub name: Option<String>,
    pub effect: Option<String>,
    pub forms: Vec<FragmentFormSnapshotDto>,
    pub dismantle_recipe: Option<crate::game::data::skill_fragment_data::SkillFragmentRecipe>,
    pub unavailable_reason: Option<String>,
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
pub struct InventorySnapshotDto {
    pub reserved_slot_count: usize,
    pub occupied_slot_count: usize,
    pub equipments: Vec<InventoryEquipmentSnapshotDto>,
    pub equipment_materials: Vec<InventoryEquipmentMaterialSnapshotDto>,
    pub consumables: Vec<ConsumableItemDto>,
    pub skill_fragments: Vec<InventorySkillFragmentSnapshotDto>,
    pub fragment_dust_usage: String,
    pub fragment_dust: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeConditionSnapshotDto {
    pub breakdown_remaining_cost: Option<u32>,
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
pub struct EmployeeCombatProfileSnapshotDto {
    pub battle_tier: String,
    pub base_stats: UnitStats,
    pub basic_attack: BasicAttackDef,
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
    pub copy_uuid: Uuid,
    pub definition_id: SkillFragmentId,
    pub selected_form: String,
    pub name: Option<String>,
    pub is_compatible: bool,
    pub failure_codes: Vec<SkillFragmentCompatibilityFailureCode>,
    pub requirements: Option<SkillFragmentCompatibilityRequirements>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSkillFragmentsSnapshotDto {
    pub equipped: Vec<EmployeeSkillFragmentBriefSnapshotDto>,
    pub baseline_ids: Vec<SkillFragmentId>,
    pub active_copy_uuid: Option<Uuid>,
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
    pub role: crate::game::employee::EmployeeRole,
    pub uuid: Uuid,
    pub employee_definition_id: String,
    pub name: String,
    pub profile: EmployeeProfile,
    pub level: u32,
    pub experience: u32,
    pub condition: EmployeeConditionSnapshotDto,
    pub eligibility: EmployeeEligibilitySnapshotDto,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteredEliteSweepRewardBranchDto {
    ReacquireFragment,
    RepeatReward,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MasteredEliteResolutionDto {
    pub direct_battle: ActionCapabilityDto,
    pub sweep: ActionCapabilityDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteredEliteSweepResultDto {
    pub sweep_uuid: Uuid,
    pub node_id: MapNodeId,
    pub encounter_id: String,
    pub reward_source_id: String,
    pub fragment_id: SkillFragmentId,
    pub reward_branch: MasteredEliteSweepRewardBranchDto,
    pub base_rewards: Vec<RewardOption>,
    pub exclusive_effects: Vec<RewardEffect>,
    pub enkephalin_after: u32,
    pub inventory_diff: InventoryDiffDto,
    pub skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
    pub employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
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
        outcome_id: crate::game::battle::types::BattleOutcomeId,
        verdict: crate::game::battle::types::BattleVerdict,
        winner: BattleWinner,
        reward_mode: RewardMode,
        rewards: Vec<RewardOptionSnapshotDto>,
        result_stats: BattleResultStatsDto,
        /// A valid ChooseOne outcome produced no candidates and needs no claim.
        no_reward_candidates: bool,
        has_event_log: bool,
    },
    MasteredEliteSweepResult {
        result: MasteredEliteSweepResultDto,
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
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
pub struct CombatResultEventLogAttachmentDto {
    pub outcome_id: crate::game::battle::types::BattleOutcomeId,
    pub verdict: crate::game::battle::types::BattleVerdict,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mastered_elite_resolution: Option<MasteredEliteResolutionDto>,
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
    MasteredEliteSweepResult {
        sweep_uuid: Uuid,
    },
    InBattle {
        battle_uuid: Uuid,
        node_type: Option<CombatNodeType>,
        mission_variant: Option<CombatMissionVariant>,
        encounter_id: Option<String>,
        combat_preview: Option<CombatPreview>,
        battle_state_revision: u64,
        tactical_resource: LiveBattleTacticalResourceDto,
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
    pub monolith: Option<MonolithRunSnapshotDto>,
    pub player_profile: PlayerProfileSnapshotDto,
    pub encounter_compendium: EncounterCompendiumSnapshotDto,
    pub map_navigation: Option<MapNavigationSnapshotDto>,
    pub map: Option<MapViewDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facility: Option<crate::game::map::FacilityViewDto>,
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
            monolith,
            player_profile,
            encounter_compendium,
            map_navigation,
            map,
            facility,
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
            monolith,
            player_profile,
            encounter_compendium,
            map_navigation,
            map,
            facility,
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
pub(crate) enum PlayerBehavior {
    // ============================================================
    // 게임 관련 행동
    // ============================================================
    /// 새 게임 시작
    StartNewGame {
        game_mode: GameMode,
        replace_active_run: bool,
    },
    /// Leave the tutorial briefing without changing the permanent profile.
    SkipTutorial,
    /// Consume the pending first successful-return story exactly once.
    AcknowledgeFirstReturnStory,
    ChooseExpeditionContinuation { choice: crate::game::expedition_lifecycle::ExpeditionContinuationChoice },
    /// Host의 local-install active-run을 마지막 완료 SavePoint에서 이어감
    ContinueRun,
    /// 손상된 local-install active-run 원본을 명시적으로 초기화
    ResetActiveRun,
    /// Local-install player profile bootstrap/save를 명시적으로 재시도
    RetryPlayerProfile,
    /// 손상된 local-install player profile 원본과 메모리 상태 전체를 명시적으로 초기화
    ResetPlayerProfile,
    /// 시작 후보 직원 중 이번 런에 투입할 3명을 선택
    SelectStarterEmployees {
        employee_definition_ids: Vec<String>,
        loan_copy_ids: Vec<Uuid>,
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
    /// Equip one exact owned copy in the employee active slot.
    EquipSkillFragment {
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    },
    UnequipSkillFragment {
        employee_uuid: Uuid,
        copy_uuid: Uuid,
    },
    TransferSkillFragment {
        copy_uuid: Uuid,
        target_employee_uuid: Uuid,
    },
    AwakenSkillFragment {
        copy_uuid: Uuid,
        form_id: String,
    },
    SelectSkillFragmentForm {
        copy_uuid: Uuid,
        form_id: String,
    },
    DismantleSkillFragment {
        copy_uuid: Uuid,
        confirmed: bool,
    },
    /// Maintenance에서 완성 장비를 분해해 장비 재료로 전환
    DismantleEquipment {
        item_uuid: Uuid,
    },
    /// Maintenance에서 장비 재료를 소비해 소유 장비 인스턴스를 강화
    EnhanceEquipment {
        item_uuid: Uuid,
    },
    EquipRelic {
        copy_uuid: Uuid,
        employee_uuid: Uuid,
        slot: u8,
    },
    UnequipRelic {
        copy_uuid: Uuid,
    },
    RequestRelicState,
    RequestRelicNumericPlan {
        employee_uuid: Uuid,
    },
    /// Read-only prospective structural plan; no execution credentials.
    RequestRelicEffectivePlan {
        employee_uuid: Uuid,
    },
    PreviewRelicCorruption {
        copy_uuid: Uuid,
    },
    CorruptRelic {
        copy_uuid: Uuid,
    },
    PreviewRelicDismantle {
        copy_uuid: Uuid,
    },
    DismantleRelic {
        copy_uuid: Uuid,
        confirmed: bool,
    },
    /// 현재 런 맵 상태 요청
    RequestMapData,
    /// Move through a reached Gate along an authored room connection.
    MoveToFacilityRoom { room_id: String },
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
    /// 현재 Event 선택 scene의 무료 변환 또는 거절을 원자적으로 확정.
    /// historical retry에서 current_scene_id는 원래 요청 source scene이다.
    ChooseRelicCorruption {
        node_id: MapNodeId,
        event_id: crate::game::data::event_data::EventId,
        current_scene_id: crate::game::data::event_data::EventSceneId,
        copy_uuid: Option<Uuid>,
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
    /// 현재 NodeConfirm의 숙달 Elite를 전투 없이 해결할 결과를 staging
    SweepMasteredElite {},
    /// 이미 staging된 숙달 Elite 소탕 결과를 원자적으로 확정
    CompleteMasteredEliteSweep {},
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
    /// 실시간 DefenseRoute 전투에서 배치된 직원의 Attack/Defense 태세를 변경
    SetEngagementStance {
        employee_uuid: Uuid,
        stance: EngagementStance,
    },
    /// 실시간 DefenseRoute 전투에서 직원의 위치/방향을 즉시 변경
    TacticalRelocateUnit {
        employee_uuid: Uuid,
        destination: Position,
        facing: FacingDirection,
    },
    /// 두 배치 직원의 위치를 하나의 원자적 명령으로 교환
    TacticalSwapUnits {
        first_employee_uuid: Uuid,
        second_employee_uuid: Uuid,
        first_facing: FacingDirection,
        second_facing: FacingDirection,
    },
    /// Open or switch the general ally/enemy information focus.
    FocusBattleUnit {
        unit_instance_id: UnitInstanceId,
    },
    ClearBattleFocus,
    /// Begin a roster deployment/redeployment selection without placing a unit.
    BeginDeploymentSelection {
        employee_uuid: Uuid,
    },
    CancelDeploymentSelection,
    OpenBattleSettings,
    CloseBattleSettings,
    /// PlayerUnit/PlayerTile 수동 스킬 target focus 시작
    BeginSkillTargeting {
        employee_uuid: Uuid,
        skill_id: SkillId,
    },
    /// 현재 authoritative skill target focus 취소
    CancelSkillTargeting,
    /// Read-only validation of the final relocation or swap payload.
    PreviewTacticalCommand {
        command: TacticalCommandPreviewInput,
    },
    /// 현재 revision에서 explicit skill target candidate를 read-only 검증
    PreviewSkillTarget {
        employee_uuid: Uuid,
        skill_id: SkillId,
        target: SkillCastTarget,
    },
    /// 실시간 DefenseRoute 전투에서 배치된 직원의 수동 스킬을 발동
    ActivateSkill {
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        #[serde(default)]
        target: Option<SkillCastTarget>,
    },
    /// 현재 source activation을 정확히 소비하는 지속 스킬 재입력.
    ReinputSustainedSkill {
        employee_uuid: Uuid,
        skill_id: crate::game::ability::SkillId,
        activation: u64,
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
            PlayerBehavior::ChooseExpeditionContinuation { .. } => ActionKind::ChooseExpeditionContinuation,
            PlayerBehavior::SkipTutorial => ActionKind::SkipTutorial,
            PlayerBehavior::AcknowledgeFirstReturnStory => ActionKind::AcknowledgeFirstReturnStory,
            PlayerBehavior::ContinueRun => ActionKind::ContinueRun,
            PlayerBehavior::ResetActiveRun => ActionKind::ResetActiveRun,
            PlayerBehavior::RetryPlayerProfile => ActionKind::RetryPlayerProfile,
            PlayerBehavior::ResetPlayerProfile => ActionKind::ResetPlayerProfile,
            PlayerBehavior::SelectStarterEmployees { .. } => ActionKind::SelectStarterEmployees,
            PlayerBehavior::UnEquipItem { .. } => ActionKind::UnEquipItem,
            PlayerBehavior::EquipItem { .. } => ActionKind::EquipItem,
            PlayerBehavior::UseConsumableItem { .. } => ActionKind::UseConsumableItem,
            PlayerBehavior::EquipSkillFragment { .. } => ActionKind::EquipSkillFragment,
            PlayerBehavior::UnequipSkillFragment { .. } => ActionKind::UnequipSkillFragment,
            PlayerBehavior::SelectSkillFragmentForm { .. } => ActionKind::SelectSkillFragmentForm,
            PlayerBehavior::TransferSkillFragment { .. } => ActionKind::TransferSkillFragment,
            PlayerBehavior::AwakenSkillFragment { .. } => ActionKind::AwakenSkillFragment,
            PlayerBehavior::DismantleSkillFragment { .. } => ActionKind::DismantleSkillFragment,
            PlayerBehavior::DismantleEquipment { .. } => ActionKind::DismantleEquipment,
            PlayerBehavior::EnhanceEquipment { .. } => ActionKind::EnhanceEquipment,
            PlayerBehavior::EquipRelic { .. } => ActionKind::EquipRelic,
            PlayerBehavior::UnequipRelic { .. } => ActionKind::UnequipRelic,
            PlayerBehavior::RequestRelicState => ActionKind::RequestRelicState,
            PlayerBehavior::RequestRelicNumericPlan { .. } => ActionKind::RequestRelicNumericPlan,
            PlayerBehavior::RequestRelicEffectivePlan { .. } => {
                ActionKind::RequestRelicEffectivePlan
            }
            PlayerBehavior::PreviewRelicCorruption { .. } => ActionKind::PreviewRelicCorruption,
            PlayerBehavior::CorruptRelic { .. } => ActionKind::CorruptRelic,
            PlayerBehavior::PreviewRelicDismantle { .. } => ActionKind::PreviewRelicDismantle,
            PlayerBehavior::DismantleRelic { .. } => ActionKind::DismantleRelic,
            PlayerBehavior::RequestMapData => ActionKind::RequestMapData,
            PlayerBehavior::MoveToFacilityRoom { .. } => ActionKind::MoveToFacilityRoom,
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
            PlayerBehavior::ChooseRelicCorruption { .. } => ActionKind::ChooseRelicCorruption,
            PlayerBehavior::AdvanceVisualNovel { .. } => ActionKind::AdvanceVisualNovel,
            PlayerBehavior::SelectVisualNovelChoice { .. } => ActionKind::SelectVisualNovelChoice,
            PlayerBehavior::SweepMasteredElite { .. } => ActionKind::SweepMasteredElite,
            PlayerBehavior::CompleteMasteredEliteSweep { .. } => {
                ActionKind::CompleteMasteredEliteSweep
            }
            PlayerBehavior::CompleteCombatResult => ActionKind::CompleteCombatResult,
            PlayerBehavior::RequestBattleState { .. } => ActionKind::RequestBattleState,
            PlayerBehavior::RecoverBattleSetupLoss => ActionKind::RecoverBattleSetupLoss,
            PlayerBehavior::RequestDeploymentRangePreview { .. } => {
                ActionKind::RequestDeploymentRangePreview
            }
            PlayerBehavior::DeployUnit { .. } => ActionKind::DeployUnit,
            PlayerBehavior::WithdrawUnit { .. } => ActionKind::WithdrawUnit,
            PlayerBehavior::SetEngagementStance { .. } => ActionKind::SetEngagementStance,
            PlayerBehavior::TacticalRelocateUnit { .. } => ActionKind::TacticalRelocateUnit,
            PlayerBehavior::TacticalSwapUnits { .. } => ActionKind::TacticalSwapUnits,
            PlayerBehavior::FocusBattleUnit { .. } => ActionKind::FocusBattleUnit,
            PlayerBehavior::ClearBattleFocus => ActionKind::ClearBattleFocus,
            PlayerBehavior::BeginDeploymentSelection { .. } => ActionKind::BeginDeploymentSelection,
            PlayerBehavior::CancelDeploymentSelection => ActionKind::CancelDeploymentSelection,
            PlayerBehavior::OpenBattleSettings => ActionKind::OpenBattleSettings,
            PlayerBehavior::CloseBattleSettings => ActionKind::CloseBattleSettings,
            PlayerBehavior::BeginSkillTargeting { .. } => ActionKind::BeginSkillTargeting,
            PlayerBehavior::CancelSkillTargeting => ActionKind::CancelSkillTargeting,
            PlayerBehavior::PreviewSkillTarget { .. } => ActionKind::PreviewSkillTarget,
            PlayerBehavior::PreviewTacticalCommand { .. } => ActionKind::PreviewTacticalCommand,
            PlayerBehavior::ActivateSkill { .. } | PlayerBehavior::ReinputSustainedSkill { .. } => {
                ActionKind::ActivateSkill
            }
            PlayerBehavior::RetreatBattle => ActionKind::RetreatBattle,
            PlayerBehavior::PauseBattle => ActionKind::PauseBattle,
            PlayerBehavior::ResumeBattle => ActionKind::ResumeBattle,
            PlayerBehavior::SetBattleSpeed { .. } => ActionKind::SetBattleSpeed,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LiveBattleTacticalResourceDto {
    pub current: u32,
    pub maximum: u32,
    pub recovery_per_second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LiveBattleDeploymentDto {
    pub battle_time_ms: u64,
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
    pub tactical_state: LiveBattleUnitTacticalStateDto,
    pub skill_readiness: LiveBattleSkillReadinessDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleTileEffectDto {
    None,
    DefenseFocus,
    Rest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleStanceEligibilityDto {
    pub attack: BattlePayloadEligibilityDto,
    pub defense: BattlePayloadEligibilityDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleUnitTacticalStateDto {
    pub stance: EngagementStance,
    pub tile_kind: TacticalTileKind,
    pub tile_effect: LiveBattleTileEffectDto,
    pub stance_cooldown_until_ms: u64,
    pub relocation_cooldown_until_ms: u64,
    pub forced_defense_until_ms: Option<u64>,
    pub forced_defense_remaining_ms: u64,
    pub relocation_eligibility: BattlePayloadEligibilityDto,
    pub stance_eligibility: LiveBattleStanceEligibilityDto,
    pub base_block_capacity: u32,
    pub effective_block_capacity: u32,
    pub defense_focus_damage_reduction_percent: u32,
    pub next_rest_tick_ms: Option<u64>,
    pub activity: UnitExclusiveActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleRedeployUnitDto {
    pub employee_uuid: Uuid,
    pub ready_at_ms: u64,
    pub deploy_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResonanceUsePhaseDto {
    Accepted,
    Casting,
    Acting,
    Effects,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResonanceGainBlockReasonDto {
    HardCrowdControl,
    SkillUse,
    AutonomousRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResonanceUseDto {
    pub action_ordinal: u64,
    pub skill_id: SkillId,
    pub consumed: u32,
    pub phase: ResonanceUsePhaseDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSkillReadinessDto {
    pub resonance_use: Option<ResonanceUseDto>,
    pub resonance_gain_block_reasons: Vec<ResonanceGainBlockReasonDto>,
    pub corrosion_ready: bool,
    pub skill_id: Option<SkillId>,
    pub activation_mode: SkillActivationMode,
    pub selection: Option<SkillCastSelection>,
    pub resonance_current: u32,
    pub resonance_max: u32,
    pub manual_activation_allowed: bool,
    #[serde(default)]
    pub target_required: bool,
    #[serde(default)]
    pub target_available: bool,
    pub eligibility: BattlePayloadEligibilityDto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleTacticalFocusPurposeDto {
    PlayerUnitSkill,
    PlayerTileSkill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleTacticalFocusDto {
    pub purpose: LiveBattleTacticalFocusPurposeDto,
    pub caster_unit_instance_id: UnitInstanceId,
    pub skill_id: SkillId,
    pub restore_playback: BattlePlaybackState,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleInteractionDto {
    pub focused_unit_instance_id: Option<UnitInstanceId>,
    pub deployment_employee_uuid: Option<Uuid>,
    pub restore_playback: Option<BattlePlaybackState>,
    pub settings_restore_playback: Option<BattlePlaybackState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattlePlaybackEligibilityDto {
    pub forced_pause_remaining_ms: Option<u64>,
    pub pause: BattlePayloadEligibilityDto,
    pub resume: BattlePayloadEligibilityDto,
    pub speeds: Vec<LiveBattleSpeedEligibilityDto>,
}

impl LiveBattlePlaybackEligibilityDto {
    pub fn for_state(playback: BattlePlaybackState) -> Self {
        let active = ActionDisabledReasonCode::AlreadyActive;
        Self {
            forced_pause_remaining_ms: None,
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
    SnapshotResyncRequired,
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
    pub field_work_objects: Vec<LiveBattleSetupFieldWorkObjectDto>,
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
pub struct LiveBattleSetupFieldWorkObjectDto {
    pub id: String,
    pub presentation_kind: LiveBattleFieldWorkPresentationKindDto,
    pub position: Position,
    pub work_cells: Vec<LiveBattleSetupFieldWorkCellDto>,
    pub required_work_ms: u64,
    pub completion_effect: LiveBattleFieldWorkCompletionEffectDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleSetupFieldWorkCellDto {
    pub id: String,
    pub position: Position,
    pub tile_kind: TacticalTileKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleFieldWorkPresentationKindDto {
    EmergencyRepairTerminal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveBattleFieldWorkCompletionEffectDto {
    RepairProtectedUnit { unit_ref: String, flat_hp: u32 },
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
    pub const SETUP_VERSION: u32 = 2;
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
    CorrosionStarted {
        actor: UnitInstanceId,
        duration_ms: u64,
    },
    CorrosionEnded {
        actor: UnitInstanceId,
    },
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
    StanceChanged {
        unit_instance_id: UnitInstanceId,
        before: crate::game::battle::types::EngagementStance,
        after: crate::game::battle::types::EngagementStance,
        cooldown_until_ms: u64,
    },
    FieldWorkAssignmentStarted {
        object_id: String,
        work_cell_id: String,
        worker_unit_instance_id: UnitInstanceId,
    },
    FieldWorkAssignmentStopped {
        object_id: String,
        work_cell_id: String,
        worker_unit_instance_id: UnitInstanceId,
    },
    FieldWorkCompleted {
        object_id: String,
        repaired_unit_instance_id: Option<UnitInstanceId>,
        requested_flat_hp: u32,
        actual_repaired_hp: u32,
    },
    UnitDisplaced {
        source_instance_id: UnitInstanceId,
        unit_instance_id: UnitInstanceId,
        from_position: Position,
        to_position: Position,
        from_world_position: EventLogVec2,
        to_world_position: EventLogVec2,
    },
    UnitTacticallyRelocated {
        unit_instance_id: UnitInstanceId,
        from_position: Position,
        to_position: Position,
        from_world_position: EventLogVec2,
        to_world_position: EventLogVec2,
        before_facing: FacingDirection,
        after_facing: FacingDirection,
        cooldown_until_ms: u64,
    },
    UnitsTacticallySwapped {
        first_unit_instance_id: UnitInstanceId,
        second_unit_instance_id: UnitInstanceId,
        first_from_position: Position,
        first_to_position: Position,
        second_from_position: Position,
        second_to_position: Position,
        first_from_world_position: EventLogVec2,
        first_to_world_position: EventLogVec2,
        second_from_world_position: EventLogVec2,
        second_to_world_position: EventLogVec2,
        first_before_facing: FacingDirection,
        first_after_facing: FacingDirection,
        second_before_facing: FacingDirection,
        second_after_facing: FacingDirection,
        cooldown_until_ms: u64,
    },
    IndependentObjectRemoved {
        receipt: crate::game::battle::core::ObjectRemovalReceipt,
    },
    IndependentObjectEvolved {
        receipt: crate::game::battle::core::ObjectEvolutionReceipt,
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
            BattleLogEvent::CorrosionStarted { actor, duration_ms } => {
                Self::CorrosionStarted { actor, duration_ms }
            }
            BattleLogEvent::CorrosionEnded { actor } => Self::CorrosionEnded { actor },
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
            BattleLogEvent::StanceChanged {
                unit_instance_id,
                before,
                after,
                cooldown_until_ms,
            } => Self::StanceChanged {
                unit_instance_id,
                before,
                after,
                cooldown_until_ms,
            },
            BattleLogEvent::FieldWorkAssignmentStarted {
                object_id,
                work_cell_id,
                worker_unit_instance_id,
            } => Self::FieldWorkAssignmentStarted {
                object_id,
                work_cell_id,
                worker_unit_instance_id,
            },
            BattleLogEvent::FieldWorkAssignmentStopped {
                object_id,
                work_cell_id,
                worker_unit_instance_id,
            } => Self::FieldWorkAssignmentStopped {
                object_id,
                work_cell_id,
                worker_unit_instance_id,
            },
            BattleLogEvent::FieldWorkCompleted {
                object_id,
                repaired_unit_instance_id,
                requested_flat_hp,
                actual_repaired_hp,
            } => Self::FieldWorkCompleted {
                object_id,
                repaired_unit_instance_id,
                requested_flat_hp,
                actual_repaired_hp,
            },
            BattleLogEvent::UnitDisplaced {
                source_instance_id,
                unit_instance_id,
                from_position,
                to_position,
                from_world_position,
                to_world_position,
            } => Self::UnitDisplaced {
                source_instance_id,
                unit_instance_id,
                from_position,
                to_position,
                from_world_position,
                to_world_position,
            },
            BattleLogEvent::UnitTacticallyRelocated {
                unit_instance_id,
                from_position,
                to_position,
                from_world_position,
                to_world_position,
                before_facing,
                after_facing,
                cooldown_until_ms,
            } => Self::UnitTacticallyRelocated {
                unit_instance_id,
                from_position,
                to_position,
                from_world_position,
                to_world_position,
                before_facing,
                after_facing,
                cooldown_until_ms,
            },
            BattleLogEvent::UnitsTacticallySwapped {
                first_unit_instance_id,
                second_unit_instance_id,
                first_from_position,
                first_to_position,
                second_from_position,
                second_to_position,
                first_from_world_position,
                first_to_world_position,
                second_from_world_position,
                second_to_world_position,
                first_before_facing,
                first_after_facing,
                second_before_facing,
                second_after_facing,
                cooldown_until_ms,
            } => Self::UnitsTacticallySwapped {
                first_unit_instance_id,
                second_unit_instance_id,
                first_from_position,
                first_to_position,
                second_from_position,
                second_to_position,
                first_from_world_position,
                first_to_world_position,
                second_from_world_position,
                second_to_world_position,
                first_before_facing,
                first_after_facing,
                second_before_facing,
                second_after_facing,
                cooldown_until_ms,
            },
            BattleLogEvent::IndependentObjectRemoved { receipt } => {
                Self::IndependentObjectRemoved { receipt }
            }
            BattleLogEvent::IndependentObjectEvolved { receipt } => {
                Self::IndependentObjectEvolved { receipt }
            }
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
    #[serde(default)]
    pub sustained_effects: crate::game::ability::sustained::SustainedEffectsSnapshot,
    #[serde(default)]
    pub independent_objects: crate::game::battle::core::IndependentObjectsSnapshot,
    pub at_seq: u64,
    pub battle_time_ms: u64,
    pub playback: BattlePlaybackState,
    pub playback_eligibility: LiveBattlePlaybackEligibilityDto,
    pub tactical_focus: Option<LiveBattleTacticalFocusDto>,
    pub interaction: LiveBattleInteractionDto,
    pub field_work_objects: Vec<LiveBattleFieldWorkObjectStateDto>,
    pub units: Vec<LiveBattleUnitCheckpointDto>,
    pub tactical_resource: LiveBattleTacticalResourceDto,
    pub deployment: Option<LiveBattleDeploymentDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleFieldWorkObjectStateDto {
    pub id: String,
    pub status: LiveBattleFieldWorkObjectStatusDto,
    pub accumulated_work_ms: u64,
    pub required_work_ms: u64,
    pub work_cells: Vec<LiveBattleFieldWorkCellStateDto>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleFieldWorkObjectStatusDto {
    Active,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleFieldWorkCellStateDto {
    pub id: String,
    pub position: Position,
    pub tile_kind: TacticalTileKind,
    pub worker_unit_instance_id: Option<UnitInstanceId>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleUnitControlStateDto {
    pub hard_cc_active: bool,
    pub hard_cc_release_time_ms: Option<u64>,
    pub silenced: bool,
    pub skill_start_disabled_reason: Option<ActionDisabledReasonCode>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LiveBattleBasicActionKind {
    #[default]
    Damage,
    Heal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveBattleBasicActionDto {
    pub kind: LiveBattleBasicActionKind,
    pub blocked_by: Option<UnitInstanceId>,
    pub blocking: Vec<UnitInstanceId>,
    pub preview_available: bool,
    pub selected_target_id: Option<UnitInstanceId>,
    pub eligible_target_ids: Vec<UnitInstanceId>,
    pub block_exception_target_ids: Vec<UnitInstanceId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveBattleUnitCheckpointDto {
    pub basic_action: LiveBattleBasicActionDto,
    pub unit_instance_id: UnitInstanceId,
    pub owner: Side,
    pub role: BattleUnitRole,
    pub hud: LiveBattleUnitHudDto,
    pub control: LiveBattleUnitControlStateDto,
    pub tactical_state: Option<LiveBattleUnitTacticalStateDto>,
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
    ManualUnit,
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
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TacticalCommandPreviewInput {
    Relocate {
        employee_uuid: Uuid,
        destination: Position,
        facing: FacingDirection,
    },
    Swap {
        first_employee_uuid: Uuid,
        second_employee_uuid: Uuid,
        first_facing: FacingDirection,
        second_facing: FacingDirection,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TacticalCommandPreviewUnitDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub destination: Position,
    pub facing: FacingDirection,
    pub forced_defense_ms: u64,
    pub range_previews: LiveBattleRangePreviewsDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TacticalCommandPreviewResultDto {
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
    pub command: TacticalCommandPreviewInput,
    pub cost: u32,
    pub relocation_cooldown_ms: u64,
    pub forced_defense_ms: u64,
    pub eligibility: BattlePayloadEligibilityDto,
    pub units: Vec<TacticalCommandPreviewUnitDto>,
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
    pub selection: SkillCastSelection,
    pub target_policy: SkillTarget,
    pub reach: SkillTargetReach,
    pub target_range: Option<TileRangePattern>,
    pub air_capable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillTargetPreviewResultDto {
    pub employee_uuid: Uuid,
    pub unit_instance_id: UnitInstanceId,
    pub skill_id: SkillId,
    pub target: SkillCastTarget,
    pub battle_state_revision: u64,
    pub eligibility: BattlePayloadEligibilityDto,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_placements: Vec<crate::game::battle::core::ObjectPlacementPreview>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum BehaviorResult {
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
    RelicState {
        result: crate::game::relic::RelicStateDto,
    },
    #[serde(skip_deserializing)]
    RelicNumericPlan {
        result: crate::game::relic::plan::RelicNumericPlanDto,
    },
    #[serde(skip_deserializing)]
    RelicEffectivePlan {
        result: Option<crate::game::relic::structural::dto::RelicEffectivePlanDto>,
    },
    RelicPreview {
        result: crate::game::relic::RelicOperationPreviewDto,
    },
    SkillFragmentLoadoutUpdated {
        employee_uuid: Uuid,
        active_copy_uuid: Option<Uuid>,
    },
    SkillFragmentTransferred {
        copy: OwnedFragment,
        previous_owner: Uuid,
        changed: bool,
    },
    SkillFragmentAwakened {
        copy: OwnedFragment,
        changed: bool,
        enkephalin_spent: u32,
        inventory_diff: InventoryDiffDto,
    },
    SkillFragmentFormSelected {
        copy: OwnedFragment,
        changed: bool,
    },
    SkillFragmentDismantled {
        copy_uuid: Uuid,
        definition_id: SkillFragmentId,
        enkephalin_gained: u32,
        inventory_diff: InventoryDiffDto,
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
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
    },

    /// 전투 리플레이 종료 후 자동 지급된 보상과 이어진 노드 진행 결과
    CombatRewardsGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        completion: Box<BehaviorResult>,
    },

    /// 전투 독립 숙달 Elite 소탕 결과가 확정 전 상태로 staging됨
    MasteredEliteSweepResultStaged {
        result: MasteredEliteSweepResultDto,
    },

    /// staging된 숙달 Elite 소탕 결과를 확인해 모든 run-local 효과를 함께 commit함
    MasteredEliteSweepCompleted {
        sweep_uuid: Uuid,
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
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

    /// Explicit skill target candidate read-only validation result
    SkillTargetPreview {
        result: SkillTargetPreviewResultDto,
    },
    TacticalCommandPreview {
        result: TacticalCommandPreviewResultDto,
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

    /// 실시간 전투에서 직원 태세 변경이 적용됨
    BattleEngagementStanceChanged {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        stance: EngagementStance,
    },

    /// 실시간 전투에서 직원 전술 재배치가 적용됨
    BattleUnitTacticallyRelocated {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        employee_uuid: Uuid,
        unit_instance_id: UnitInstanceId,
        destination: Position,
        facing: FacingDirection,
    },

    /// 실시간 전투에서 두 직원의 전술 위치 교환이 적용됨
    BattleUnitsTacticallySwapped {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
        first_employee_uuid: Uuid,
        second_employee_uuid: Uuid,
        first_unit_instance_id: UnitInstanceId,
        second_unit_instance_id: UnitInstanceId,
        first_destination: Position,
        second_destination: Position,
        first_facing: FacingDirection,
        second_facing: FacingDirection,
    },

    /// Authoritative manual-skill targeting focus changed
    BattleSkillTargetingFocusChanged {
        battle_uuid: Uuid,
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
        battle_update: LiveBattleUpdateDto,
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum BehaviorCommandResultContract {
    CommandPayload(CommandResultPayloadContract),
    BattleUpdate(BattleUpdateCommandResultContract),
    ReadOnlyPreview(CommandResultPayloadContract),
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) struct BattleUpdateCommandResultContract {
    pub battle_update: LiveBattleUpdateDto,
    pub battle_setup_snapshot: Option<LiveBattleSetupSnapshotDto>,
}

#[derive(Debug, Clone)]
pub(crate) enum CoreResultProjection {
    CommandPayload(CoreResultPayload),
    BattleUpdate,
    ReadOnlyPreview(CoreResultPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CoreResultPayload {
    pub result_type: &'static str,
    pub payload: Value,
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum CommandResultPayloadContract {
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum RunCommandResult {
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum MapCommandResult {
    MapState { map: MapViewDto },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum NodeCommandResult {
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
    },
    EventState {
        node_id: MapNodeId,
        event: EventSceneSnapshotDto,
    },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum FacilityCommandResult {
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum InventoryCommandResult {
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
    RelicState {
        result: crate::game::relic::RelicStateDto,
    },
    RelicNumericPlan {
        result: crate::game::relic::plan::RelicNumericPlanDto,
    },
    RelicEffectivePlan {
        result: Option<crate::game::relic::structural::dto::RelicEffectivePlanDto>,
    },
    RelicPreview {
        result: crate::game::relic::RelicOperationPreviewDto,
    },
    SkillFragmentLoadoutUpdated {
        employee_uuid: Uuid,
        active_copy_uuid: Option<Uuid>,
    },
    SkillFragmentTransferred {
        copy: OwnedFragment,
        previous_owner: Uuid,
        changed: bool,
    },
    SkillFragmentAwakened {
        copy: OwnedFragment,
        changed: bool,
        enkephalin_spent: u32,
        inventory_diff: InventoryDiffDto,
    },
    SkillFragmentFormSelected {
        copy: OwnedFragment,
        changed: bool,
    },
    SkillFragmentDismantled {
        copy_uuid: Uuid,
        definition_id: SkillFragmentId,
        enkephalin_gained: u32,
        inventory_diff: InventoryDiffDto,
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum RosterCommandResult {
    EmployeeRecruited {
        employee_definition_id: String,
        employee_uuid: Uuid,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum ShopCommandResult {
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

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum RewardCommandResult {
    RewardState {
        mode: RewardMode,
        rewards: Vec<RewardOption>,
        selected_reward_uuid: Option<Uuid>,
    },
    RewardGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
    },
    CombatRewardsGranted {
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        outcome: NodeOutcomeSummary,
        completion: Box<CommandResultPayloadContract>,
    },
    MasteredEliteSweepResultStaged {
        result: MasteredEliteSweepResultDto,
    },
    MasteredEliteSweepCompleted {
        sweep_uuid: Uuid,
        enkephalin: u32,
        inventory_diff: InventoryDiffDto,
        skill_fragment_diffs: Vec<SkillFragmentGrantDiffDto>,
        employee_experience_diffs: Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        completion: Box<CommandResultPayloadContract>,
    },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum BattleCommandResult {
    DeploymentRangePreview {
        result: DeploymentRangePreviewResultDto,
    },
    SkillTargetPreview {
        result: SkillTargetPreviewResultDto,
    },
    TacticalCommandPreview {
        result: TacticalCommandPreviewResultDto,
    },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum VisualNovelCommandResult {
    VisualNovelState {
        visual_novel: Option<VisualNovelSnapshotDto>,
    },
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub(crate) enum GeneralCommandResult {
    Ok,
}

fn nested_command_result_projection(completion: &BehaviorResult) -> serde_json::Result<Value> {
    let projection = match completion.application_result_projection()? {
        CoreResultProjection::CommandPayload(projection)
        | CoreResultProjection::ReadOnlyPreview(projection) => projection,
        CoreResultProjection::BattleUpdate => {
            panic!("battle update results cannot be nested inside command result payloads")
        }
    };
    crate::authoring::projection::object!({
        "result_type": projection.result_type,
        "payload": projection.payload,
    })
}

impl BehaviorResult {
    pub(crate) fn application_result_projection(&self) -> serde_json::Result<CoreResultProjection> {
        // Borrow the actual payload fields before any owned JSON allocation.
        macro_rules! json {
            ({ $($key:literal : $value:expr),* $(,)? }) => {
                crate::authoring::projection::object!({ $($key : $value),* })?
            };
        }
        macro_rules! command_payload {
            ($result_type:literal, $payload:expr) => {
                Ok(CoreResultProjection::CommandPayload(CoreResultPayload {
                    result_type: $result_type,
                    payload: $payload,
                }))
            };
        }
        macro_rules! read_only_preview {
            ($result_type:literal, $payload:expr) => {
                Ok(CoreResultProjection::ReadOnlyPreview(CoreResultPayload {
                    result_type: $result_type,
                    payload: $payload,
                }))
            };
        }

        match self {
            BehaviorResult::RunStartFlowStarted {
                game_mode,
                visual_novel,
            } => command_payload!(
                "RunStartFlowStarted",
                json!({
                    "game_mode": game_mode,
                    "visual_novel": visual_novel,
                })
            ),
            BehaviorResult::StarterEmployeesSelected {
                selected_employee_definition_ids,
                employee_uuids,
                map,
            } => command_payload!(
                "StarterEmployeesSelected",
                json!({
                    "selected_employee_definition_ids": selected_employee_definition_ids,
                    "employee_uuids": employee_uuids,
                    "map": map,
                })
            ),
            BehaviorResult::MapState { map } => {
                command_payload!("MapState", json!({ "map": map }))
            }
            BehaviorResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
            } => command_payload!(
                "NodeEntered",
                json!({
                    "node_id": node_id,
                    "kind_id": kind_id,
                    "category": category,
                    "payload": payload,
                    "session": session,
                })
            ),
            BehaviorResult::NodePreview {
                node_id,
                kind_id,
                category,
                payload,
                session,
                map,
                combat_preview,
                trauma_recovery_preview,
            } => command_payload!(
                "NodePreview",
                json!({
                    "node_id": node_id,
                    "kind_id": kind_id,
                    "category": category,
                    "payload": payload,
                    "session": session,
                    "map": map,
                    "combat_preview": combat_preview,
                    "trauma_recovery_preview": trauma_recovery_preview,
                })
            ),
            BehaviorResult::NodeCompleted { map, outcome } => command_payload!(
                "NodeCompleted",
                json!({
                    "map": map,
                    "outcome": outcome,
                })
            ),
            BehaviorResult::SavePointState {
                node_id,
                recovery_preview,
            } => command_payload!(
                "SavePointState",
                json!({
                    "node_id": node_id,
                    "recovery_preview": recovery_preview,
                })
            ),
            BehaviorResult::MaintenanceState {
                node_id,
                maintenance_options,
            } => command_payload!(
                "MaintenanceState",
                json!({
                    "node_id": node_id,
                    "maintenance_options": maintenance_options,
                })
            ),
            BehaviorResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
            } => command_payload!(
                "HeadquartersContactState",
                json!({
                    "node_id": node_id,
                    "options": options,
                    "recruitment_candidates": recruitment_candidates,
                    "shop_pool_id": shop_pool_id,
                })
            ),
            BehaviorResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
            } => command_payload!(
                "FloorAdvanced",
                json!({
                    "game_mode": game_mode,
                    "floor_index": floor_index,
                    "map": map,
                })
            ),
            BehaviorResult::RunComplete { map } => {
                command_payload!("RunComplete", crate::authoring::projection::to_value(map)?)
            }
            BehaviorResult::RunFailed { reason, outcome } => command_payload!(
                "RunFailed",
                json!({
                    "reason": reason,
                    "outcome": outcome,
                })
            ),
            BehaviorResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            } => command_payload!(
                "RunFailureDecisionRequired",
                json!({
                    "reason": reason,
                    "contributing_reason": contributing_reason,
                    "outcome": outcome,
                })
            ),
            BehaviorResult::RunAbandoned => command_payload!("RunAbandoned", json!({})),
            BehaviorResult::UnEquipItem { result } => {
                command_payload!(
                    "UnEquipItem",
                    crate::authoring::projection::to_value(result)?
                )
            }
            BehaviorResult::EquipItem { result } => {
                command_payload!("EquipItem", crate::authoring::projection::to_value(result)?)
            }
            BehaviorResult::ConsumableItemUsed {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                active_modifier,
                condition,
                inventory_diff,
            } => command_payload!(
                "ConsumableItemUsed",
                json!({
                    "item_uuid": item_uuid,
                    "target_employee_uuid": target_employee_uuid,
                    "replaced_modifier": replaced_modifier,
                    "active_modifier": active_modifier,
                    "condition": condition,
                    "inventory_diff": inventory_diff,
                })
            ),
            BehaviorResult::RelicState { result } => command_payload!(
                "RelicState",
                crate::authoring::projection::to_value(result)?
            ),
            BehaviorResult::RelicNumericPlan { result } => command_payload!(
                "RelicNumericPlan",
                crate::authoring::projection::to_value(result)?
            ),
            BehaviorResult::RelicEffectivePlan { result } => command_payload!(
                "RelicEffectivePlan",
                crate::authoring::projection::to_value(result)?
            ),
            BehaviorResult::RelicPreview { result } => command_payload!(
                "RelicPreview",
                crate::authoring::projection::to_value(result)?
            ),
            BehaviorResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                active_copy_uuid,
            } => command_payload!(
                "SkillFragmentLoadoutUpdated",
                json!({"employee_uuid": employee_uuid, "active_copy_uuid": active_copy_uuid})
            ),
            BehaviorResult::SkillFragmentTransferred {
                copy,
                previous_owner,
                changed,
            } => command_payload!(
                "SkillFragmentTransferred",
                json!({"copy": copy, "previous_owner": previous_owner, "changed": changed})
            ),
            BehaviorResult::SkillFragmentAwakened {
                copy,
                changed,
                enkephalin_spent,
                inventory_diff,
            } => command_payload!(
                "SkillFragmentAwakened",
                json!({"copy": copy, "changed": changed, "enkephalin_spent": enkephalin_spent, "inventory_diff": inventory_diff})
            ),
            BehaviorResult::SkillFragmentFormSelected { copy, changed } => command_payload!(
                "SkillFragmentFormSelected",
                json!({"copy": copy, "changed": changed})
            ),
            BehaviorResult::SkillFragmentDismantled {
                copy_uuid,
                definition_id,
                enkephalin_gained,
                inventory_diff,
            } => command_payload!(
                "SkillFragmentDismantled",
                json!({"copy_uuid": copy_uuid, "definition_id": definition_id, "enkephalin_gained": enkephalin_gained, "inventory_diff": inventory_diff})
            ),
            BehaviorResult::EquipmentDismantled {
                item_uuid,
                equipment_id,
                inventory_diff,
            } => command_payload!(
                "EquipmentDismantled",
                json!({
                    "item_uuid": item_uuid,
                    "equipment_id": equipment_id,
                    "inventory_diff": inventory_diff,
                })
            ),
            BehaviorResult::EquipmentEnhanced {
                item_uuid,
                equipment_id,
                enhancement_level,
                inventory_diff,
            } => command_payload!(
                "EquipmentEnhanced",
                json!({
                    "item_uuid": item_uuid,
                    "equipment_id": equipment_id,
                    "enhancement_level": enhancement_level,
                    "inventory_diff": inventory_diff,
                })
            ),
            BehaviorResult::ShopState { shop } => {
                command_payload!("ShopState", json!({ "shop": shop }))
            }
            BehaviorResult::RefreshShop { enkephalin, shop } => command_payload!(
                "RefreshShop",
                json!({
                    "enkephalin": enkephalin,
                    "shop": shop,
                })
            ),
            BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            } => command_payload!(
                "SellItem",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                })
            ),
            BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => command_payload!(
                "PurchaseItem",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                })
            ),
            BehaviorResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion,
            } => command_payload!(
                "EmployeeRecruited",
                json!({
                    "employee_definition_id": employee_definition_id,
                    "employee_uuid": employee_uuid,
                    "completion": nested_command_result_projection(completion)?,
                })
            ),
            BehaviorResult::EmergencySuppliesGranted {
                enkephalin,
                inventory_diff,
                completion,
            } => command_payload!(
                "EmergencySuppliesGranted",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "completion": nested_command_result_projection(completion)?,
                })
            ),
            BehaviorResult::EventState { node_id, event } => command_payload!(
                "EventState",
                json!({
                    "node_id": node_id,
                    "event": event,
                })
            ),
            BehaviorResult::VisualNovelState { visual_novel } => command_payload!(
                "VisualNovelState",
                json!({
                    "visual_novel": visual_novel,
                })
            ),
            BehaviorResult::RewardGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
            } => command_payload!(
                "RewardGranted",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "skill_fragment_diffs": skill_fragment_diffs,
                    "employee_experience_diffs": employee_experience_diffs,
                })
            ),
            BehaviorResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                outcome,
                completion,
            } => command_payload!(
                "CombatRewardsGranted",
                json!({
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "skill_fragment_diffs": skill_fragment_diffs,
                    "employee_experience_diffs": employee_experience_diffs,
                    "outcome": outcome,
                    "completion": nested_command_result_projection(completion)?,
                })
            ),
            BehaviorResult::MasteredEliteSweepResultStaged { result } => command_payload!(
                "MasteredEliteSweepResultStaged",
                json!({ "result": result })
            ),
            BehaviorResult::MasteredEliteSweepCompleted {
                sweep_uuid,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                completion,
            } => command_payload!(
                "MasteredEliteSweepCompleted",
                json!({
                    "sweep_uuid": sweep_uuid,
                    "enkephalin": enkephalin,
                    "inventory_diff": inventory_diff,
                    "skill_fragment_diffs": skill_fragment_diffs,
                    "employee_experience_diffs": employee_experience_diffs,
                    "completion": nested_command_result_projection(completion)?,
                })
            ),
            BehaviorResult::BattleAdvanced {
                battle_setup_snapshot,
                battle_update,
                ..
            } => {
                let _ = (battle_setup_snapshot, battle_update);
                Ok(CoreResultProjection::BattleUpdate)
            }
            BehaviorResult::BattleState { battle_update, .. }
            | BehaviorResult::BattlePlaybackChanged { battle_update, .. }
            | BehaviorResult::BattleUnitDeployed { battle_update, .. }
            | BehaviorResult::BattleUnitWithdrawn { battle_update, .. }
            | BehaviorResult::BattleEngagementStanceChanged { battle_update, .. }
            | BehaviorResult::BattleUnitTacticallyRelocated { battle_update, .. }
            | BehaviorResult::BattleUnitsTacticallySwapped { battle_update, .. }
            | BehaviorResult::BattleSkillTargetingFocusChanged { battle_update, .. }
            | BehaviorResult::BattleSkillActivated { battle_update, .. } => {
                let _ = battle_update;
                Ok(CoreResultProjection::BattleUpdate)
            }
            BehaviorResult::DeploymentRangePreview { result } => {
                read_only_preview!(
                    "DeploymentRangePreview",
                    crate::authoring::projection::to_value(result)?
                )
            }
            BehaviorResult::TacticalCommandPreview { result } => {
                read_only_preview!(
                    "TacticalCommandPreview",
                    crate::authoring::projection::to_value(result)?
                )
            }
            BehaviorResult::SkillTargetPreview { result } => {
                read_only_preview!(
                    "SkillTargetPreview",
                    crate::authoring::projection::to_value(result)?
                )
            }
            BehaviorResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
            } => command_payload!(
                "RewardState",
                json!({
                    "mode": mode,
                    "rewards": rewards,
                    "selected_reward_uuid": selected_reward_uuid,
                })
            ),
            BehaviorResult::Ok => {
                command_payload!("Ok", crate::authoring::projection::to_value(())?)
            }
        }
    }

    #[cfg(test)]
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
            BehaviorResult::NodeCompleted { map, outcome } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Node(
                    NodeCommandResult::NodeCompleted { map, outcome },
                ))
            }
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
            BehaviorResult::MasteredEliteSweepResultStaged { result } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Reward(
                    RewardCommandResult::MasteredEliteSweepResultStaged { result },
                ))
            }
            BehaviorResult::MasteredEliteSweepCompleted {
                sweep_uuid,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                completion,
            } => {
                BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Reward(
                    RewardCommandResult::MasteredEliteSweepCompleted {
                        sweep_uuid,
                        enkephalin,
                        inventory_diff,
                        skill_fragment_diffs,
                        employee_experience_diffs,
                        completion: Box::new(completion.into_command_payload_contract()),
                    },
                ))
            }
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
            } => BehaviorCommandResultContract::CommandPayload(CommandResultPayloadContract::Run(
                RunCommandResult::FloorAdvanced {
                    game_mode,
                    floor_index,
                    map,
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
            BehaviorResult::RelicState { result } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(InventoryCommandResult::RelicState {
                    result,
                }),
            ),
            BehaviorResult::RelicNumericPlan { result } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::Inventory(
                        InventoryCommandResult::RelicNumericPlan { result },
                    ),
                )
            }
            BehaviorResult::RelicEffectivePlan { result } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::Inventory(
                        InventoryCommandResult::RelicEffectivePlan { result },
                    ),
                )
            }
            BehaviorResult::RelicPreview { result } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::Inventory(InventoryCommandResult::RelicPreview {
                        result,
                    }),
                )
            }
            BehaviorResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                active_copy_uuid,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentLoadoutUpdated {
                        employee_uuid,
                        active_copy_uuid,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentTransferred {
                copy,
                previous_owner,
                changed,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentTransferred {
                        copy,
                        previous_owner,
                        changed,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentAwakened {
                copy,
                changed,
                enkephalin_spent,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentAwakened {
                        copy,
                        changed,
                        enkephalin_spent,
                        inventory_diff,
                    },
                ),
            ),
            BehaviorResult::SkillFragmentFormSelected { copy, changed } => {
                BehaviorCommandResultContract::CommandPayload(
                    CommandResultPayloadContract::Inventory(
                        InventoryCommandResult::SkillFragmentFormSelected { copy, changed },
                    ),
                )
            }
            BehaviorResult::SkillFragmentDismantled {
                copy_uuid,
                definition_id,
                enkephalin_gained,
                inventory_diff,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Inventory(
                    InventoryCommandResult::SkillFragmentDismantled {
                        copy_uuid,
                        definition_id,
                        enkephalin_gained,
                        inventory_diff,
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
                employee_experience_diffs,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::RewardGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
                    employee_experience_diffs,
                }),
            ),
            BehaviorResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                outcome,
                completion,
            } => BehaviorCommandResultContract::CommandPayload(
                CommandResultPayloadContract::Reward(RewardCommandResult::CombatRewardsGranted {
                    enkephalin,
                    inventory_diff,
                    skill_fragment_diffs,
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
            | BehaviorResult::BattleEngagementStanceChanged { battle_update, .. }
            | BehaviorResult::BattleUnitTacticallyRelocated { battle_update, .. }
            | BehaviorResult::BattleUnitsTacticallySwapped { battle_update, .. }
            | BehaviorResult::BattleSkillTargetingFocusChanged { battle_update, .. }
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
            BehaviorResult::TacticalCommandPreview { result } => {
                BehaviorCommandResultContract::ReadOnlyPreview(
                    CommandResultPayloadContract::Battle(
                        BattleCommandResult::TacticalCommandPreview { result },
                    ),
                )
            }
            BehaviorResult::SkillTargetPreview { result } => {
                BehaviorCommandResultContract::ReadOnlyPreview(
                    CommandResultPayloadContract::Battle(BattleCommandResult::SkillTargetPreview {
                        result,
                    }),
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

    #[cfg(test)]
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

    #[cfg(test)]
    pub fn is_run_start_flow_started(&self) -> bool {
        matches!(self, BehaviorResult::RunStartFlowStarted { .. })
    }

    #[cfg(test)]
    pub fn is_sell_item(&self) -> bool {
        matches!(self, BehaviorResult::SellItem { .. })
    }

    /// SellItem → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    #[cfg(test)]
    pub fn as_sell_item(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn is_ok(&self) -> bool {
        matches!(self, BehaviorResult::Ok)
    }

    // ============================================================
    // 데이터 추출 헬퍼 (참조 반환)
    // ============================================================

    /// ShopState → ShopEventOption 참조 반환
    #[cfg(test)]
    pub fn as_shop_state(&self) -> Option<&ShopEventOption> {
        match self {
            BehaviorResult::ShopState { shop, .. } => Some(shop),
            _ => None,
        }
    }

    /// PurchaseItem → (남은 엔케팔린, 인벤토리 변경 사항) 반환
    #[cfg(test)]
    pub fn as_purchase_item(&self) -> Option<(u32, &InventoryDiffDto)> {
        match self {
            BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => Some((*enkephalin, inventory_diff)),
            _ => None,
        }
    }
}

#[cfg(test)]
impl CommandResultPayloadContract {
    pub fn result_type(&self) -> &'static str {
        self.command_result_projection()
            .expect("typed command result payload must serialize")
            .result_type
    }

    pub fn payload_value(&self) -> serde_json::Result<Value> {
        Ok(self.command_result_projection()?.payload)
    }

    fn command_result_projection(&self) -> serde_json::Result<CoreResultPayload> {
        let result = self.clone().into_behavior_result();
        match result.application_result_projection()? {
            CoreResultProjection::CommandPayload(projection)
            | CoreResultProjection::ReadOnlyPreview(projection) => Ok(projection),
            CoreResultProjection::BattleUpdate => {
                unreachable!("payload compatibility types cannot contain a battle update")
            }
        }
    }

    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            CommandResultPayloadContract::Run(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Map(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Node(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Facility(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Inventory(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Roster(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Shop(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Reward(result) => result.into_behavior_result(),
            CommandResultPayloadContract::Battle(result) => result.into_behavior_result(),
            CommandResultPayloadContract::VisualNovel(result) => result.into_behavior_result(),
            CommandResultPayloadContract::General(result) => result.into_behavior_result(),
        }
    }
}

#[cfg(test)]
impl RunCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            RunCommandResult::RunStartFlowStarted {
                game_mode,
                visual_novel,
            } => BehaviorResult::RunStartFlowStarted {
                game_mode,
                visual_novel,
            },
            RunCommandResult::StarterEmployeesSelected {
                selected_employee_definition_ids,
                employee_uuids,
                map,
            } => BehaviorResult::StarterEmployeesSelected {
                selected_employee_definition_ids,
                employee_uuids,
                map,
            },
            RunCommandResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
            } => BehaviorResult::FloorAdvanced {
                game_mode,
                floor_index,
                map,
            },
            RunCommandResult::RunComplete { map } => BehaviorResult::RunComplete { map },
            RunCommandResult::RunFailed { reason, outcome } => {
                BehaviorResult::RunFailed { reason, outcome }
            }
            RunCommandResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            } => BehaviorResult::RunFailureDecisionRequired {
                reason,
                contributing_reason,
                outcome,
            },
            RunCommandResult::RunAbandoned => BehaviorResult::RunAbandoned,
        }
    }
}

#[cfg(test)]
impl MapCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            MapCommandResult::MapState { map } => BehaviorResult::MapState { map },
        }
    }
}

#[cfg(test)]
impl NodeCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
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
            } => BehaviorResult::NodePreview {
                node_id,
                kind_id,
                category,
                payload,
                session,
                map,
                combat_preview,
                trauma_recovery_preview,
            },
            NodeCommandResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
            } => BehaviorResult::NodeEntered {
                node_id,
                kind_id,
                category,
                payload,
                session,
            },
            NodeCommandResult::NodeCompleted { map, outcome } => {
                BehaviorResult::NodeCompleted { map, outcome }
            }
            NodeCommandResult::EventState { node_id, event } => {
                BehaviorResult::EventState { node_id, event }
            }
        }
    }
}

#[cfg(test)]
impl FacilityCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            FacilityCommandResult::SavePointState {
                node_id,
                recovery_preview,
            } => BehaviorResult::SavePointState {
                node_id,
                recovery_preview,
            },
            FacilityCommandResult::MaintenanceState {
                node_id,
                maintenance_options,
            } => BehaviorResult::MaintenanceState {
                node_id,
                maintenance_options,
            },
            FacilityCommandResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
            } => BehaviorResult::HeadquartersContactState {
                node_id,
                options,
                recruitment_candidates,
                shop_pool_id,
            },
            FacilityCommandResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion,
            } => BehaviorResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion: Box::new(completion.into_behavior_result()),
            },
            FacilityCommandResult::EmergencySuppliesGranted {
                enkephalin,
                inventory_diff,
                completion,
            } => BehaviorResult::EmergencySuppliesGranted {
                enkephalin,
                inventory_diff,
                completion: Box::new(completion.into_behavior_result()),
            },
        }
    }
}

#[cfg(test)]
impl InventoryCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            InventoryCommandResult::RelicState { result } => BehaviorResult::RelicState { result },
            InventoryCommandResult::RelicNumericPlan { result } => {
                BehaviorResult::RelicNumericPlan { result }
            }
            InventoryCommandResult::RelicEffectivePlan { result } => {
                BehaviorResult::RelicEffectivePlan { result }
            }
            InventoryCommandResult::RelicPreview { result } => {
                BehaviorResult::RelicPreview { result }
            }
            InventoryCommandResult::UnEquipItem { result } => {
                BehaviorResult::UnEquipItem { result }
            }
            InventoryCommandResult::EquipItem { result } => BehaviorResult::EquipItem { result },
            InventoryCommandResult::ConsumableItemUsed {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                active_modifier,
                condition,
                inventory_diff,
            } => BehaviorResult::ConsumableItemUsed {
                item_uuid,
                target_employee_uuid,
                replaced_modifier,
                active_modifier,
                condition,
                inventory_diff,
            },
            InventoryCommandResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                active_copy_uuid,
            } => BehaviorResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                active_copy_uuid,
            },
            InventoryCommandResult::SkillFragmentTransferred {
                copy,
                previous_owner,
                changed,
            } => BehaviorResult::SkillFragmentTransferred {
                copy,
                previous_owner,
                changed,
            },
            InventoryCommandResult::SkillFragmentAwakened {
                copy,
                changed,
                enkephalin_spent,
                inventory_diff,
            } => BehaviorResult::SkillFragmentAwakened {
                copy,
                changed,
                enkephalin_spent,
                inventory_diff,
            },
            InventoryCommandResult::SkillFragmentFormSelected { copy, changed } => {
                BehaviorResult::SkillFragmentFormSelected { copy, changed }
            }
            InventoryCommandResult::SkillFragmentDismantled {
                copy_uuid,
                definition_id,
                enkephalin_gained,
                inventory_diff,
            } => BehaviorResult::SkillFragmentDismantled {
                copy_uuid,
                definition_id,
                enkephalin_gained,
                inventory_diff,
            },
            InventoryCommandResult::EquipmentDismantled {
                item_uuid,
                equipment_id,
                inventory_diff,
            } => BehaviorResult::EquipmentDismantled {
                item_uuid,
                equipment_id,
                inventory_diff,
            },
            InventoryCommandResult::EquipmentEnhanced {
                item_uuid,
                equipment_id,
                enhancement_level,
                inventory_diff,
            } => BehaviorResult::EquipmentEnhanced {
                item_uuid,
                equipment_id,
                enhancement_level,
                inventory_diff,
            },
        }
    }
}

#[cfg(test)]
impl RosterCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            RosterCommandResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion,
            } => BehaviorResult::EmployeeRecruited {
                employee_definition_id,
                employee_uuid,
                completion: Box::new(completion.into_behavior_result()),
            },
        }
    }
}

#[cfg(test)]
impl ShopCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            ShopCommandResult::ShopState { shop } => BehaviorResult::ShopState { shop },
            ShopCommandResult::RefreshShop { enkephalin, shop } => {
                BehaviorResult::RefreshShop { enkephalin, shop }
            }
            ShopCommandResult::SellItem {
                enkephalin,
                inventory_diff,
            } => BehaviorResult::SellItem {
                enkephalin,
                inventory_diff,
            },
            ShopCommandResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            } => BehaviorResult::PurchaseItem {
                enkephalin,
                inventory_diff,
            },
        }
    }
}

#[cfg(test)]
impl RewardCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            RewardCommandResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
            } => BehaviorResult::RewardState {
                mode,
                rewards,
                selected_reward_uuid,
            },
            RewardCommandResult::RewardGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
            } => BehaviorResult::RewardGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
            },
            RewardCommandResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                outcome,
                completion,
            } => BehaviorResult::CombatRewardsGranted {
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                outcome,
                completion: Box::new(completion.into_behavior_result()),
            },
            RewardCommandResult::MasteredEliteSweepResultStaged { result } => {
                BehaviorResult::MasteredEliteSweepResultStaged { result }
            }
            RewardCommandResult::MasteredEliteSweepCompleted {
                sweep_uuid,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                completion,
            } => BehaviorResult::MasteredEliteSweepCompleted {
                sweep_uuid,
                enkephalin,
                inventory_diff,
                skill_fragment_diffs,
                employee_experience_diffs,
                completion: Box::new(completion.into_behavior_result()),
            },
        }
    }
}

#[cfg(test)]
impl BattleCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            BattleCommandResult::DeploymentRangePreview { result } => {
                BehaviorResult::DeploymentRangePreview { result }
            }
            BattleCommandResult::TacticalCommandPreview { result } => {
                BehaviorResult::TacticalCommandPreview { result }
            }
            BattleCommandResult::SkillTargetPreview { result } => {
                BehaviorResult::SkillTargetPreview { result }
            }
        }
    }
}

#[cfg(test)]
impl VisualNovelCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            VisualNovelCommandResult::VisualNovelState { visual_novel } => {
                BehaviorResult::VisualNovelState { visual_novel }
            }
        }
    }
}

#[cfg(test)]
impl GeneralCommandResult {
    fn into_behavior_result(self) -> BehaviorResult {
        match self {
            GeneralCommandResult::Ok => BehaviorResult::Ok,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameError {
    /// Native battle admission/operation failure, with allocation-free details.
    BattleRuntimeFault(crate::game::battle::runtime_control::BattleRuntimeFault),
    /// A compiled value expression failed while consuming a battle occurrence.
    ValueExpressionFault(crate::game::value_expr::ValueExprFault),
    /// 선택한 이벤트/보상이 현재 세션 옵션에 존재하지 않을 때
    EventNotFound,
    /// 이벤트는 존재하지만 기대한 타입(Shop/Reward/Random 등)이 아닐 때
    EventTypeMismatch,

    /// 현재 GameState/Context에서 허용되지 않은 행동 (치팅 시도 포함)
    InvalidAction,
    /// Endless admission requires completion of the fixed Standard tutorial.
    TutorialCompletionRequired { game_mode: GameMode },
    /// A context capability or selected payload failed the shared typed preflight.
    ActionUnavailable {
        action: ActionKind,
        reason: ActionDisabledReasonCode,
    },
    /// A battle-derived employee profile was requested without an equipped weapon.
    WeaponRequired,
    /// 클라이언트가 현재 전투에 존재하지 않는 미래 presentation seq로 resync를 요청했을 때
    InvalidBattleResyncSeq { requested: u64, latest: u64 },
    /// 요청 cursor가 bounded active history floor보다 오래되어 snapshot resync가 필요할 때
    BattleSnapshotResyncRequired {
        requested: u64,
        oldest_available_after_seq: u64,
        latest: u64,
    },
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
    /// Caller-selected data was not found; retains the historical public error
    /// envelope while distinguishing input refusal from trusted runtime damage.
    InvalidCommandData(String),
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

impl GameError {
    /// Stable protocol identity shared by Core projections and server adapters.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BattleRuntimeFault(fault) => fault.code.as_str(),
            Self::ValueExpressionFault(_) => "value_expression_fault",
            Self::EventNotFound => "event_not_found",
            Self::EventTypeMismatch => "event_type_mismatch",
            Self::InvalidAction => "invalid_action",
            Self::TutorialCompletionRequired { .. } => "standard_tutorial_completion_required",
            Self::ActionUnavailable { reason, .. } => reason.as_str(),
            Self::WeaponRequired => "weapon_required",
            Self::InvalidBattleResyncSeq { .. } => "invalid_battle_resync_seq",
            Self::BattleSnapshotResyncRequired { .. } => "battle_snapshot_resync_required",
            Self::BattleStateRevisionOverflow => "battle_state_revision_overflow",
            Self::MissingBattleStateAuthority { .. } => "missing_battle_state_authority",
            Self::StaleBattleStateAuthority { .. } => "stale_battle_state_authority",
            Self::NotInShopState => "not_in_shop_state",
            Self::NotInRewardState => "not_in_reward_state",
            Self::ShopItemNotFound => "shop_item_not_found",
            Self::InventoryFull => "inventory_full",
            Self::InventoryItemNotFound => "inventory_item_not_found",
            Self::InventoryItemNotRemovable => "inventory_item_not_removable",
            Self::InsufficientResources => "insufficient_resources",
            Self::MissingResource(_) => "missing_resource",
            Self::InvalidUnitStats(_) => "invalid_unit_stats",
            Self::InvalidStaticData(_) | Self::InvalidCommandData(_) => "invalid_static_data",
            Self::NotImplemented(_) => "not_implemented",
            Self::SkillFragmentIncompatible { .. } => "skill_fragment_incompatible",
            Self::OutOfBounds => "out_of_bounds",
            Self::PositionOccupied => "position_occupied",
            Self::StaticObstacleBlocked => "static_obstacle_blocked",
            Self::UnitAlreadyPlaced => "unit_already_placed",
            Self::UnitNotFound => "unit_not_found",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::battle::event_log::{BattleEventCause, BattleEventRootCause, BattleLogEvent};
    use crate::game::data::shop_data::ShopContext;
    use crate::game::resources::InventoryDiffDto;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum CharacterizedActionRoute {
        CoreCommon,
        CoreDomain,
        CoreBattle,
        CoreContextual,
        HostRecovery,
    }

    #[test]
    fn every_action_kind_has_one_characterized_route_scope_and_authority_contract() {
        use ActionKind::*;
        use ActionScope::{Gameplay, Query, Recovery};
        use CharacterizedActionRoute::{
            CoreBattle, CoreCommon, CoreContextual, CoreDomain, HostRecovery,
        };

        let cases = [
            (StartNewGame, CoreCommon, Gameplay, false),
            (SkipTutorial, CoreCommon, Gameplay, false),
            (AcknowledgeFirstReturnStory, CoreCommon, Gameplay, false),
            (ChooseExpeditionContinuation, CoreCommon, Gameplay, false),
            (ContinueRun, HostRecovery, Recovery, false),
            (ResetActiveRun, HostRecovery, Recovery, false),
            (RetryPlayerProfile, HostRecovery, Recovery, false),
            (ResetPlayerProfile, HostRecovery, Recovery, false),
            (SelectStarterEmployees, CoreCommon, Gameplay, false),
            (UnEquipItem, CoreDomain, Gameplay, false),
            (EquipItem, CoreDomain, Gameplay, false),
            (UseConsumableItem, CoreDomain, Gameplay, false),
            (EquipSkillFragment, CoreDomain, Gameplay, false),
            (UnequipSkillFragment, CoreDomain, Gameplay, false),
            (SelectSkillFragmentForm, CoreCommon, Gameplay, false),
            (TransferSkillFragment, CoreDomain, Gameplay, false),
            (AwakenSkillFragment, CoreCommon, Gameplay, false),
            (DismantleSkillFragment, CoreCommon, Gameplay, false),
            (DismantleEquipment, CoreCommon, Gameplay, false),
            (EnhanceEquipment, CoreCommon, Gameplay, false),
            (EquipRelic, CoreDomain, Gameplay, false),
            (UnequipRelic, CoreDomain, Gameplay, false),
            (RequestRelicState, CoreCommon, Query, false),
            (RequestRelicNumericPlan, CoreCommon, Query, false),
            (RequestRelicEffectivePlan, CoreCommon, Query, false),
            (PreviewRelicCorruption, CoreCommon, Query, false),
            (CorruptRelic, CoreDomain, Gameplay, false),
            (PreviewRelicDismantle, CoreCommon, Query, false),
            (DismantleRelic, CoreDomain, Gameplay, false),
            (RequestMapData, CoreCommon, Query, false),
            (MoveToFacilityRoom, CoreCommon, Gameplay, false),
            (SelectMapNode, CoreCommon, Gameplay, false),
            (ConvertMapNodeWithMonolith, CoreDomain, Gameplay, false),
            (ConfirmEnterNode, CoreCommon, Gameplay, false),
            (CancelSelectedNode, CoreCommon, Gameplay, false),
            (CompleteNode, CoreCommon, Gameplay, false),
            (LoadRunCheckpoint, CoreCommon, Recovery, false),
            (ConfirmRunFailure, CoreCommon, Gameplay, false),
            (AbandonRun, CoreCommon, Gameplay, false),
            (RecruitEmployee, CoreCommon, Gameplay, false),
            (RequestEmergencySupplies, CoreCommon, Gameplay, false),
            (OpenHeadquartersShop, CoreCommon, Gameplay, false),
            (SelectReward, CoreCommon, Gameplay, false),
            (PurchaseItem, CoreDomain, Gameplay, false),
            (SellItem, CoreDomain, Gameplay, false),
            (RefreshShop, CoreDomain, Gameplay, false),
            (ExitShop, CoreDomain, Gameplay, false),
            (ClaimReward, CoreCommon, Gameplay, false),
            (ExitReward, CoreCommon, Gameplay, false),
            (AdvanceEventScene, CoreCommon, Gameplay, false),
            (ResolveEventChoiceCheck, CoreCommon, Gameplay, false),
            (ChooseRelicCorruption, CoreCommon, Gameplay, false),
            (AdvanceVisualNovel, CoreContextual, Gameplay, true),
            (SelectVisualNovelChoice, CoreContextual, Gameplay, true),
            (SweepMasteredElite, CoreCommon, Gameplay, false),
            (CompleteMasteredEliteSweep, CoreCommon, Gameplay, false),
            (CompleteCombatResult, CoreCommon, Gameplay, false),
            (RequestBattleState, CoreBattle, Query, false),
            (RecoverBattleSetupLoss, CoreBattle, Recovery, false),
            (RequestDeploymentRangePreview, CoreBattle, Query, false),
            (DeployUnit, CoreBattle, Gameplay, true),
            (WithdrawUnit, CoreBattle, Gameplay, true),
            (SetEngagementStance, CoreBattle, Gameplay, true),
            (TacticalRelocateUnit, CoreBattle, Gameplay, true),
            (TacticalSwapUnits, CoreBattle, Gameplay, true),
            (FocusBattleUnit, CoreBattle, Gameplay, true),
            (ClearBattleFocus, CoreBattle, Gameplay, true),
            (BeginDeploymentSelection, CoreBattle, Gameplay, true),
            (CancelDeploymentSelection, CoreBattle, Gameplay, true),
            (OpenBattleSettings, CoreBattle, Gameplay, true),
            (CloseBattleSettings, CoreBattle, Gameplay, true),
            (BeginSkillTargeting, CoreBattle, Gameplay, true),
            (CancelSkillTargeting, CoreBattle, Gameplay, true),
            (PreviewSkillTarget, CoreBattle, Query, false),
            (PreviewTacticalCommand, CoreBattle, Query, false),
            (ActivateSkill, CoreBattle, Gameplay, true),
            (RetreatBattle, CoreBattle, Gameplay, true),
            (PauseBattle, CoreBattle, Gameplay, true),
            (ResumeBattle, CoreBattle, Gameplay, true),
            (SetBattleSpeed, CoreBattle, Gameplay, true),
        ];

        assert_eq!(
            cases.len(),
            79,
            "the current command vocabulary is exhaustive"
        );
        let characterized_actions = cases
            .iter()
            .map(|(action, ..)| *action)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(
            characterized_actions.len(),
            cases.len(),
            "each command must have exactly one descriptor row"
        );
        assert_eq!(
            characterized_actions,
            ActionKind::ALL.iter().copied().collect(),
            "the descriptor table must cover the macro-owned command vocabulary"
        );
        assert_eq!(
            cases
                .iter()
                .filter(|(_, route, ..)| *route == HostRecovery)
                .count(),
            4,
            "the host owns exactly the four established recovery commands"
        );

        for (action, route, scope, requires_battle_authority) in cases {
            let actual_route = match action.route() {
                ActionRoute::CoreCommon => CoreCommon,
                ActionRoute::CoreDomain => CoreDomain,
                ActionRoute::CoreBattle => CoreBattle,
                ActionRoute::CoreContextual => CoreContextual,
                ActionRoute::HostRecovery(operation) => {
                    assert_eq!(
                        action,
                        match operation {
                            HostRecoveryAction::ContinueRun => ContinueRun,
                            HostRecoveryAction::ResetActiveRun => ResetActiveRun,
                            HostRecoveryAction::RetryPlayerProfile => {
                                RetryPlayerProfile
                            }
                            HostRecoveryAction::ResetPlayerProfile => {
                                ResetPlayerProfile
                            }
                        },
                        "host operation identity drift for {action:?}"
                    );
                    HostRecovery
                }
            };
            assert_eq!(actual_route, route, "route drift for {action:?}");
            assert_eq!(action.scope(), scope, "scope drift for {action:?}");
            assert_eq!(
                action.requires_battle_authority(),
                requires_battle_authority,
                "battle-authority drift for {action:?}"
            );
            assert_eq!(
                serde_json::to_value(action).unwrap(),
                serde_json::Value::String(format!("{action:?}")),
                "ActionKind wire name drift for {action:?}"
            );
        }
    }

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
    fn choose_relic_corruption_wire_preserves_source_scene_and_optional_copy() {
        let node_id = MapNodeId::new(Uuid::from_u128(0xE081));
        for copy_uuid in [Some(Uuid::from_u128(0xE082)), None] {
            let behavior = PlayerBehavior::ChooseRelicCorruption {
                node_id,
                event_id: crate::game::data::event_data::EventId::new("event_relic"),
                current_scene_id: crate::game::data::event_data::EventSceneId::new("source"),
                copy_uuid,
            };
            let value = serde_json::to_value(&behavior).unwrap();
            assert_eq!(
                value,
                serde_json::json!({
                    "type": "choose_relic_corruption",
                    "node_id": node_id.0,
                    "event_id": "event_relic",
                    "current_scene_id": "source",
                    "copy_uuid": copy_uuid,
                })
            );
            let decoded: PlayerBehavior = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(decoded.kind(), ActionKind::ChooseRelicCorruption);
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
            for required in ["node_id", "event_id", "current_scene_id"] {
                let mut missing = value.clone();
                missing.as_object_mut().unwrap().remove(required);
                assert!(serde_json::from_value::<PlayerBehavior>(missing).is_err());
            }
            let mut forged = value;
            forged["catalyst_cost"] = serde_json::json!(0);
            assert!(serde_json::from_value::<PlayerBehavior>(forged).is_err());
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
            game_mode: GameMode::Expedition,
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum CharacterizedResultTransport {
        CommandPayload,
        BattleUpdate,
        ReadOnlyPreview,
    }

    fn characterized_map() -> MapViewDto {
        MapViewDto {
            map_template_id: crate::game::map::MapTemplateId::new("result_contract"),
            facility_template_version: crate::game::map::FACILITY_TEMPLATE_VERSION,
            nodes: Vec::new(),
        }
    }

    fn characterized_session() -> NodeSession {
        NodeSession {
            node_id: MapNodeId::new(Uuid::from_u128(0x3301)),
            kind_id: MapNodeKindId::new("save_point"),
            category: MapNodeCategory::SavePoint,
            payload: MapNodePayload::SavePoint,
        }
    }

    fn characterized_outcome() -> NodeOutcomeSummary {
        NodeOutcomeSummary {
            node_id: MapNodeId::new(Uuid::from_u128(0x3302)),
            kind_id: MapNodeKindId::new("combat"),
            category: MapNodeCategory::Combat,
            mission_success: true,
            combat: None,
            employee_changes: Vec::new(),
            inventory_diff: InventoryDiffDto::default(),
        }
    }

    fn characterized_shop() -> ShopEventOption {
        ShopEventOption {
            id: "result_shop".to_string(),
            name: "Result Shop".to_string(),
            uuid: Uuid::from_u128(0x3303),
            context: ShopContext::UnknownExploration,
            offers: Vec::new(),
            refresh: None,
            selling_enabled: false,
        }
    }

    fn characterized_combat_preview() -> CombatPreview {
        CombatPreview {
            deterrence: Default::default(),
            node_id: MapNodeId::new(Uuid::from_u128(0x3304)),
            encounter_id: Some("result_encounter".to_string()),
            battlefield_template_id: "result_battlefield".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            mission_risk: crate::game::combat_preview::CombatMissionRisk::Controlled,
            archetype: crate::game::combat_preview::BattlefieldArchetype::OpenHall,
            size_class: crate::game::combat_preview::BattlefieldSizeClass::Small,
            width: 1,
            height: 1,
            tiles: Vec::new(),
            valid_tiles: Vec::new(),
            deployment_zones: Vec::new(),
            spawn_zones: Vec::new(),
            routes: Vec::new(),
            spawn_waves: Vec::new(),
            obstacles: Vec::new(),
            enemy_briefing: Vec::new(),
            threat_warnings: Vec::new(),
            enemy_stat_scale: Default::default(),
        }
    }

    fn characterized_battle_update(battle_uuid: Uuid) -> LiveBattleUpdateDto {
        LiveBattleUpdateDto {
            message_type: LiveBattleUpdateMessageType::BattleUpdate,
            battle_uuid,
            battle_state_revision: 33,
            action_capabilities: Vec::new(),
            server_battle_time_ms: 3_300,
            events_delta: LiveBattleEventDeltaDto {
                after_seq: 4,
                to_seq: 5,
                events: Vec::new(),
            },
            checkpoint: LiveBattleStateCheckpointDto {
                sustained_effects: Default::default(),
                independent_objects: Default::default(),
                at_seq: 5,
                battle_time_ms: 3_300,
                playback: BattlePlaybackState::default(),
                playback_eligibility: LiveBattlePlaybackEligibilityDto::for_state(
                    BattlePlaybackState::default(),
                ),
                tactical_focus: None,
                interaction: Default::default(),
                field_work_objects: Vec::new(),
                units: Vec::new(),
                tactical_resource: LiveBattleTacticalResourceDto {
                    current: 7,
                    maximum: 10,
                    recovery_per_second: 1,
                },
                deployment: None,
            },
            visual_novel: None,
        }
    }

    fn characterized_battle_setup(battle_uuid: Uuid) -> LiveBattleSetupSnapshotDto {
        LiveBattleSetupSnapshotDto {
            message_type: LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
            setup_version: LiveBattleSetupSnapshotDto::SETUP_VERSION,
            battle_uuid,
            encounter_id: "result_encounter".to_string(),
            node_type: CombatNodeType::Defense,
            mission_variant: CombatMissionVariant::Defense,
            survive_timer_ms: None,
            battlefield: LiveBattleSetupBattlefieldDto {
                width: 1,
                height: 1,
                tiles: Vec::new(),
                valid_tiles: Vec::new(),
                blocked_tiles: Vec::new(),
                static_obstacles: Vec::new(),
            },
            routes: Vec::new(),
            deployment_zones: Vec::new(),
            spawn_zones: Vec::new(),
            tactical_points: Vec::new(),
            static_objects: Vec::new(),
            field_work_objects: Vec::new(),
            initial_units: Vec::new(),
            catalog_refs: LiveBattleSetupCatalogRefsDto::default(),
        }
    }

    fn characterized_range_previews() -> LiveBattleRangePreviewsDto {
        LiveBattleRangePreviewsDto {
            basic_attack: LiveBattleBasicAttackRangePreviewDto {
                available: true,
                reason: None,
                cells: vec![Position::new(1, 1)],
                source: LiveBattleRangePreviewSource::FallbackBasicAttack,
                source_id: None,
            },
            active_skill: LiveBattleActiveSkillRangePreviewDto {
                available: false,
                reason: Some(LiveBattleRangePreviewReason::NoSkill),
                skill_id: None,
                targeting_kind: None,
                requires_manual_target: false,
                cast_cells: Vec::new(),
                effect_preview_cells: Vec::new(),
                source: LiveBattleRangePreviewSource::Unavailable,
                source_id: None,
            },
        }
    }

    fn characterized_deployment_preview() -> DeploymentRangePreviewResultDto {
        let facing = DeploymentRangePreviewFacingDto {
            range_previews: characterized_range_previews(),
        };
        DeploymentRangePreviewResultDto {
            employee_uuid: Uuid::from_u128(0x3305),
            position: Position::new(1, 2),
            deploy_eligibility: BattlePayloadEligibilityDto::enabled(),
            facings: DeploymentRangePreviewFacingsDto {
                up: facing.clone(),
                right: facing.clone(),
                down: facing.clone(),
                left: facing,
            },
        }
    }

    fn characterized_result_transport(result: &BehaviorResult) -> CharacterizedResultTransport {
        match result {
            BehaviorResult::BattleAdvanced { .. }
            | BehaviorResult::BattleState { .. }
            | BehaviorResult::BattlePlaybackChanged { .. }
            | BehaviorResult::BattleUnitDeployed { .. }
            | BehaviorResult::BattleUnitWithdrawn { .. }
            | BehaviorResult::BattleEngagementStanceChanged { .. }
            | BehaviorResult::BattleUnitTacticallyRelocated { .. }
            | BehaviorResult::BattleUnitsTacticallySwapped { .. }
            | BehaviorResult::BattleSkillTargetingFocusChanged { .. }
            | BehaviorResult::BattleSkillActivated { .. } => {
                CharacterizedResultTransport::BattleUpdate
            }
            BehaviorResult::DeploymentRangePreview { .. }
            | BehaviorResult::SkillTargetPreview { .. }
            | BehaviorResult::TacticalCommandPreview { .. } => {
                CharacterizedResultTransport::ReadOnlyPreview
            }
            BehaviorResult::RunStartFlowStarted { .. }
            | BehaviorResult::StarterEmployeesSelected { .. }
            | BehaviorResult::MapState { .. }
            | BehaviorResult::NodeEntered { .. }
            | BehaviorResult::NodePreview { .. }
            | BehaviorResult::NodeCompleted { .. }
            | BehaviorResult::SavePointState { .. }
            | BehaviorResult::MaintenanceState { .. }
            | BehaviorResult::HeadquartersContactState { .. }
            | BehaviorResult::FloorAdvanced { .. }
            | BehaviorResult::RunComplete { .. }
            | BehaviorResult::RunFailed { .. }
            | BehaviorResult::RunFailureDecisionRequired { .. }
            | BehaviorResult::RunAbandoned
            | BehaviorResult::UnEquipItem { .. }
            | BehaviorResult::EquipItem { .. }
            | BehaviorResult::ConsumableItemUsed { .. }
            | BehaviorResult::RelicState { .. }
            | BehaviorResult::RelicNumericPlan { .. }
            | BehaviorResult::RelicEffectivePlan { .. }
            | BehaviorResult::RelicPreview { .. }
            | BehaviorResult::SkillFragmentLoadoutUpdated { .. }
            | BehaviorResult::SkillFragmentFormSelected { .. }
            | BehaviorResult::SkillFragmentTransferred { .. }
            | BehaviorResult::SkillFragmentAwakened { .. }
            | BehaviorResult::SkillFragmentDismantled { .. }
            | BehaviorResult::EquipmentDismantled { .. }
            | BehaviorResult::EquipmentEnhanced { .. }
            | BehaviorResult::ShopState { .. }
            | BehaviorResult::RefreshShop { .. }
            | BehaviorResult::SellItem { .. }
            | BehaviorResult::PurchaseItem { .. }
            | BehaviorResult::EmployeeRecruited { .. }
            | BehaviorResult::EmergencySuppliesGranted { .. }
            | BehaviorResult::EventState { .. }
            | BehaviorResult::VisualNovelState { .. }
            | BehaviorResult::RewardGranted { .. }
            | BehaviorResult::CombatRewardsGranted { .. }
            | BehaviorResult::MasteredEliteSweepResultStaged { .. }
            | BehaviorResult::MasteredEliteSweepCompleted { .. }
            | BehaviorResult::RewardState { .. }
            | BehaviorResult::Ok => CharacterizedResultTransport::CommandPayload,
        }
    }

    fn characterized_serialized_variant(value: Value) -> (String, Value) {
        match value {
            Value::String(result_type) => (result_type, json!({})),
            Value::Object(object) => {
                assert_eq!(
                    object.len(),
                    1,
                    "externally tagged result must have one key"
                );
                object.into_iter().next().unwrap()
            }
            other => panic!("unexpected BehaviorResult serde shape: {other:?}"),
        }
    }

    fn characterized_expected_payload(result_type: &str, fields: Value) -> Value {
        match result_type {
            "RunComplete" => fields["map"].clone(),
            "UnEquipItem"
            | "EquipItem"
            | "RelicState"
            | "RelicNumericPlan"
            | "RelicEffectivePlan"
            | "RelicPreview"
            | "DeploymentRangePreview"
            | "SkillTargetPreview"
            | "TacticalCommandPreview" => fields["result"].clone(),
            "RunAbandoned" => json!({}),
            "Ok" => Value::Null,
            "EmployeeRecruited"
            | "EmergencySuppliesGranted"
            | "CombatRewardsGranted"
            | "MasteredEliteSweepCompleted" => {
                let mut fields = fields.as_object().unwrap().clone();
                let (nested_type, nested_fields) =
                    characterized_serialized_variant(fields.remove("completion").unwrap());
                fields.insert(
                    "completion".to_string(),
                    json!({
                        "result_type": nested_type,
                        "payload": characterized_expected_payload(&nested_type, nested_fields),
                    }),
                );
                Value::Object(fields)
            }
            _ => fields,
        }
    }

    fn characterized_results() -> Vec<BehaviorResult> {
        let battle_uuid = Uuid::from_u128(0x33B0);
        let employee_uuid = Uuid::from_u128(0x33E0);
        let other_employee_uuid = Uuid::from_u128(0x33E1);
        let unit_instance_id = UnitInstanceId::from(Uuid::from_u128(0x33A0));
        let other_unit_instance_id = UnitInstanceId::from(Uuid::from_u128(0x33A1));
        let session = characterized_session();

        vec![
            BehaviorResult::RunStartFlowStarted {
                game_mode: GameMode::Expedition,
                visual_novel: None,
            },
            BehaviorResult::StarterEmployeesSelected {
                selected_employee_definition_ids: vec!["starter".to_string()],
                employee_uuids: vec![employee_uuid],
                map: characterized_map(),
            },
            BehaviorResult::MapState {
                map: characterized_map(),
            },
            BehaviorResult::NodeEntered {
                node_id: session.node_id,
                kind_id: session.kind_id.clone(),
                category: session.category,
                payload: session.payload.clone(),
                session: session.clone(),
            },
            BehaviorResult::NodePreview {
                node_id: session.node_id,
                kind_id: session.kind_id.clone(),
                category: session.category,
                payload: session.payload.clone(),
                session,
                map: characterized_map(),
                combat_preview: None,
                trauma_recovery_preview: None,
            },
            BehaviorResult::NodeCompleted {
                map: characterized_map(),
                outcome: Some(characterized_outcome()),
            },
            BehaviorResult::SavePointState {
                node_id: MapNodeId::new(Uuid::from_u128(0x3310)),
                recovery_preview: TraumaRecoveryPreviewDto {
                    recovery: TraumaRecovery::CurrentPercent { percent: 15 },
                    employees: Vec::new(),
                    physical_damage_unchanged: true,
                    replaces_run_checkpoint: true,
                },
            },
            BehaviorResult::MaintenanceState {
                node_id: MapNodeId::new(Uuid::from_u128(0x3311)),
                maintenance_options: MaintenanceOptionsDto {
                    items: Vec::new(),
                    materials: Vec::new(),
                },
            },
            BehaviorResult::HeadquartersContactState {
                node_id: MapNodeId::new(Uuid::from_u128(0x3312)),
                options: vec![HeadquartersContactOption::RecruitEmployee],
                recruitment_candidates: Vec::new(),
                shop_pool_id: Some("hq_shop".to_string()),
            },
            BehaviorResult::FloorAdvanced {
                game_mode: GameMode::Expedition,
                floor_index: 2,
                map: characterized_map(),
            },
            BehaviorResult::RunComplete {
                map: characterized_map(),
            },
            BehaviorResult::RunFailed {
                reason: RunFailureReason::NoLivingEmployees,
                outcome: None,
            },
            BehaviorResult::RunFailureDecisionRequired {
                reason: RunFailureReason::NoDeployableEmployees,
                contributing_reason: Some(RunFailureReason::BossDefeated),
                outcome: Some(characterized_outcome()),
            },
            BehaviorResult::RunAbandoned,
            BehaviorResult::UnEquipItem {
                result: UnequipItemResultDto {
                    item_uuid: Uuid::from_u128(0x3320),
                    target_unit: employee_uuid,
                    equipped_items: Vec::new(),
                    inventory_diff: InventoryDiffDto::default(),
                },
            },
            BehaviorResult::EquipItem {
                result: EquipItemResultDto {
                    requested_item_uuid: Uuid::from_u128(0x3321),
                    target_unit: employee_uuid,
                    outcome: crate::game::resources::EquipItemOutcomeDto::Equipped {
                        item_uuid: Uuid::from_u128(0x3321),
                    },
                    equipped_items: Vec::new(),
                    inventory_diff: InventoryDiffDto::default(),
                },
            },
            BehaviorResult::ConsumableItemUsed {
                item_uuid: Uuid::from_u128(0x3322),
                target_employee_uuid: employee_uuid,
                replaced_modifier: None,
                active_modifier: None,
                condition: EmployeeCondition::default(),
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::RelicState {
                result: crate::game::relic::RelicStateDto {
                    copies: vec![],
                    loadouts: vec![],
                    not_sellable_copy_uuids: vec![],
                    resources: Default::default(),
                },
            },
            BehaviorResult::RelicNumericPlan {
                result: crate::game::relic::plan::RelicNumericPlanDto {
                    employee_uuid,
                    generation: crate::game::data::content_pack::ContentIdentity {
                        contract: crate::game::data::content_pack::ContentContract::current(),
                        semantic_sha256: "0".repeat(64),
                    },
                    scope: crate::game::relic::plan::RelicNumericPlanScope::ProspectiveLoadout,
                    base_battle_stats: None,
                    effective_battle_stats: None,
                    stats_unavailable_reason: Some(crate::game::relic::plan::RelicNumericPlanUnresolvedReason::ProviderUnavailable),
                    penalties: vec![],
                    targets: vec![],
                },
            },
            BehaviorResult::RelicEffectivePlan { result: None },
            BehaviorResult::RelicPreview {
                result: crate::game::relic::RelicOperationPreviewDto {
                    copy: crate::game::relic::OwnedRelic {
                        copy_uuid: employee_uuid,
                        definition_id: "fixture".into(),
                        variant: Default::default(),
                    },
                    operation: crate::game::relic::RelicOperationKind::Corrupt,
                    effects: vec![],
                    penalties: vec![],
                    catalyst_cost: 1,
                    equipment_dust_gained: 0,
                    irreversible: true,
                    equipped_to: None,
                },
            },
            BehaviorResult::SkillFragmentLoadoutUpdated {
                employee_uuid,
                active_copy_uuid: Some(Uuid::from_u128(99)),
            },
            BehaviorResult::SkillFragmentFormSelected {
                copy: OwnedFragment {
                    copy_uuid: Uuid::from_u128(99),
                    definition_id: SkillFragmentId::from("fragment"),
                    unlocked_forms: std::collections::BTreeSet::from(["base".into()]),
                    selected_form: "base".into(),
                    equipped_to: None,
                },
                changed: true,
            },
            BehaviorResult::SkillFragmentTransferred {
                copy: OwnedFragment {
                    copy_uuid: Uuid::from_u128(99),
                    definition_id: SkillFragmentId::from("fragment"),
                    unlocked_forms: std::collections::BTreeSet::from(["base".into()]),
                    selected_form: "base".into(),
                    equipped_to: None,
                },
                previous_owner: employee_uuid,
                changed: true,
            },
            BehaviorResult::SkillFragmentAwakened {
                copy: OwnedFragment {
                    copy_uuid: Uuid::from_u128(99),
                    definition_id: SkillFragmentId::from("fragment"),
                    unlocked_forms: std::collections::BTreeSet::from(["base".into()]),
                    selected_form: "base".into(),
                    equipped_to: None,
                },
                changed: true,
                enkephalin_spent: 3,
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::SkillFragmentDismantled {
                copy_uuid: Uuid::from_u128(99),
                definition_id: SkillFragmentId::from("fragment"),
                enkephalin_gained: 2,
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::EquipmentDismantled {
                item_uuid: Uuid::from_u128(0x3323),
                equipment_id: "equipment_a".to_string(),
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::EquipmentEnhanced {
                item_uuid: Uuid::from_u128(0x3324),
                equipment_id: "equipment_b".to_string(),
                enhancement_level: 2,
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::ShopState {
                shop: characterized_shop(),
            },
            BehaviorResult::RefreshShop {
                enkephalin: 10,
                shop: characterized_shop(),
            },
            BehaviorResult::SellItem {
                enkephalin: 11,
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::PurchaseItem {
                enkephalin: 12,
                inventory_diff: InventoryDiffDto::default(),
            },
            BehaviorResult::EmployeeRecruited {
                employee_definition_id: "recruit".to_string(),
                employee_uuid,
                completion: Box::new(BehaviorResult::NodeCompleted {
                    map: characterized_map(),
                    outcome: None,
                }),
            },
            BehaviorResult::EmergencySuppliesGranted {
                enkephalin: 13,
                inventory_diff: InventoryDiffDto::default(),
                completion: Box::new(BehaviorResult::RunComplete {
                    map: characterized_map(),
                }),
            },
            BehaviorResult::EventState {
                node_id: MapNodeId::new(Uuid::from_u128(0x3330)),
                event: EventSceneSnapshotDto {
                    event_id: crate::game::data::event_data::EventId::new("event"),
                    current_scene_id: crate::game::data::event_data::EventSceneId::new("scene"),
                    presentation: crate::game::data::event_data::EventScenePresentation {
                        panels: Vec::new(),
                    },
                    committed_check: None,
                    relic_corruption_selections: None,
                    interaction: EventInteractionDto::Advance,
                },
            },
            BehaviorResult::VisualNovelState { visual_novel: None },
            BehaviorResult::RewardGranted {
                enkephalin: 14,
                inventory_diff: InventoryDiffDto::default(),
                skill_fragment_diffs: Vec::new(),
                employee_experience_diffs: Vec::new(),
            },
            BehaviorResult::CombatRewardsGranted {
                enkephalin: 15,
                inventory_diff: InventoryDiffDto::default(),
                skill_fragment_diffs: Vec::new(),
                employee_experience_diffs: Vec::new(),
                outcome: characterized_outcome(),
                completion: Box::new(BehaviorResult::FloorAdvanced {
                    game_mode: GameMode::Expedition,
                    floor_index: 3,
                    map: characterized_map(),
                }),
            },
            BehaviorResult::MasteredEliteSweepResultStaged {
                result: MasteredEliteSweepResultDto {
                    sweep_uuid: Uuid::from_u128(0x3340),
                    node_id: MapNodeId::new(Uuid::from_u128(0x3341)),
                    encounter_id: "mastered_elite_encounter".to_string(),
                    reward_source_id: "mastered_elite_reward".to_string(),
                    fragment_id: SkillFragmentId::from("mastered_elite_fragment"),
                    reward_branch: MasteredEliteSweepRewardBranchDto::RepeatReward,
                    base_rewards: Vec::new(),
                    exclusive_effects: Vec::new(),
                    enkephalin_after: 16,
                    inventory_diff: InventoryDiffDto::default(),
                    skill_fragment_diffs: Vec::new(),
                    employee_experience_diffs: Vec::new(),
                },
            },
            BehaviorResult::MasteredEliteSweepCompleted {
                sweep_uuid: Uuid::from_u128(0x3340),
                enkephalin: 16,
                inventory_diff: InventoryDiffDto::default(),
                skill_fragment_diffs: Vec::new(),
                employee_experience_diffs: Vec::new(),
                completion: Box::new(BehaviorResult::RunComplete {
                    map: characterized_map(),
                }),
            },
            BehaviorResult::BattleAdvanced {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_setup_snapshot: Some(characterized_battle_setup(battle_uuid)),
                battle_update: characterized_battle_update(battle_uuid),
                finished: false,
            },
            BehaviorResult::BattleState {
                battle_uuid,
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                encounter_id: "result_encounter".to_string(),
                combat_preview: characterized_combat_preview(),
                battle_update: characterized_battle_update(battle_uuid),
                finished: false,
            },
            BehaviorResult::DeploymentRangePreview {
                result: characterized_deployment_preview(),
            },
            BehaviorResult::TacticalCommandPreview {
                result: TacticalCommandPreviewResultDto {
                    battle_uuid: Uuid::from_u128(0xC006),
                    battle_state_revision: 1,
                    command: TacticalCommandPreviewInput::Relocate {
                        employee_uuid: Uuid::from_u128(0x3305),
                        destination: Position::new(1, 2),
                        facing: FacingDirection::Right,
                    },
                    cost: 5,
                    relocation_cooldown_ms: 10_000,
                    forced_defense_ms: 1500,
                    eligibility: BattlePayloadEligibilityDto::enabled(),
                    units: Vec::new(),
                },
            },
            BehaviorResult::SkillTargetPreview {
                result: SkillTargetPreviewResultDto {
                    object_placements: Vec::new(),
                    employee_uuid,
                    unit_instance_id,
                    skill_id: SkillId::from("skill_a"),
                    target: SkillCastTarget::Tile {
                        position: Position::new(2, 3),
                    },
                    battle_state_revision: 33,
                    eligibility: BattlePayloadEligibilityDto::enabled(),
                },
            },
            BehaviorResult::BattlePlaybackChanged {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
            },
            BehaviorResult::BattleUnitDeployed {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                employee_uuid,
                unit_instance_id,
            },
            BehaviorResult::BattleUnitWithdrawn {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                employee_uuid,
                unit_instance_id,
            },
            BehaviorResult::BattleEngagementStanceChanged {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                employee_uuid,
                unit_instance_id,
                stance: EngagementStance::Defense,
            },
            BehaviorResult::BattleUnitTacticallyRelocated {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                employee_uuid,
                unit_instance_id,
                destination: Position::new(3, 4),
                facing: FacingDirection::Right,
            },
            BehaviorResult::BattleUnitsTacticallySwapped {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                first_employee_uuid: employee_uuid,
                second_employee_uuid: other_employee_uuid,
                first_unit_instance_id: unit_instance_id,
                second_unit_instance_id: other_unit_instance_id,
                first_destination: Position::new(4, 5),
                second_destination: Position::new(5, 4),
                first_facing: FacingDirection::Left,
                second_facing: FacingDirection::Right,
            },
            BehaviorResult::BattleSkillTargetingFocusChanged {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
            },
            BehaviorResult::BattleSkillActivated {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_update: characterized_battle_update(battle_uuid),
                employee_uuid,
                unit_instance_id,
                skill_id: SkillId::from("skill_b"),
            },
            BehaviorResult::RewardState {
                mode: RewardMode::ChooseOne,
                rewards: Vec::new(),
                selected_reward_uuid: None,
            },
            BehaviorResult::Ok,
        ]
    }

    #[test]
    fn every_behavior_result_has_exact_old_command_projection_contract() {
        const EXPECTED_RESULT_TYPES: [&str; 55] = [
            "RunStartFlowStarted",
            "StarterEmployeesSelected",
            "MapState",
            "NodeEntered",
            "NodePreview",
            "NodeCompleted",
            "SavePointState",
            "MaintenanceState",
            "HeadquartersContactState",
            "FloorAdvanced",
            "RunComplete",
            "RunFailed",
            "RunFailureDecisionRequired",
            "RunAbandoned",
            "UnEquipItem",
            "EquipItem",
            "ConsumableItemUsed",
            "RelicState",
            "RelicNumericPlan",
            "RelicEffectivePlan",
            "RelicPreview",
            "SkillFragmentLoadoutUpdated",
            "SkillFragmentFormSelected",
            "SkillFragmentTransferred",
            "SkillFragmentAwakened",
            "SkillFragmentDismantled",
            "EquipmentDismantled",
            "EquipmentEnhanced",
            "ShopState",
            "RefreshShop",
            "SellItem",
            "PurchaseItem",
            "EmployeeRecruited",
            "EmergencySuppliesGranted",
            "EventState",
            "VisualNovelState",
            "RewardGranted",
            "CombatRewardsGranted",
            "MasteredEliteSweepResultStaged",
            "MasteredEliteSweepCompleted",
            "BattleAdvanced",
            "BattleState",
            "DeploymentRangePreview",
            "TacticalCommandPreview",
            "SkillTargetPreview",
            "BattlePlaybackChanged",
            "BattleUnitDeployed",
            "BattleUnitWithdrawn",
            "BattleEngagementStanceChanged",
            "BattleUnitTacticallyRelocated",
            "BattleUnitsTacticallySwapped",
            "BattleSkillTargetingFocusChanged",
            "BattleSkillActivated",
            "RewardState",
            "Ok",
        ];
        let cases = characterized_results();
        assert_eq!(cases.len(), EXPECTED_RESULT_TYPES.len());
        assert_eq!(
            crate::application::ApplicationResultKind::PLAYER_RESULTS.len(),
            EXPECTED_RESULT_TYPES.len()
        );

        for (result, expected_result_type) in cases.into_iter().zip(EXPECTED_RESULT_TYPES) {
            let expected_transport = characterized_result_transport(&result);
            let (serialized_result_type, fields) =
                characterized_serialized_variant(serde_json::to_value(&result).unwrap());
            assert_eq!(serialized_result_type, expected_result_type);
            let application_result =
                crate::application::ApplicationResult::from_behavior(&result).unwrap();
            let (bounded, _) = crate::authoring::control::run(
                &crate::authoring::ExecutionLimits::default(),
                &crate::authoring::CancellationToken::default(),
                crate::authoring::ExecutionPhase::TrialExecution,
                || crate::application::ApplicationResult::from_behavior(&result),
            )
            .unwrap();
            assert_eq!(
                bounded.unwrap(),
                application_result,
                "{expected_result_type}"
            );
            assert_eq!(application_result.meaning(), expected_result_type);
            assert_ne!(
                application_result.disposition(),
                crate::application::ApplicationResultDisposition::AcceptedRecoveryPending
            );
            let expected_application_payload = match expected_transport {
                CharacterizedResultTransport::BattleUpdate => fields.clone(),
                CharacterizedResultTransport::CommandPayload
                | CharacterizedResultTransport::ReadOnlyPreview => {
                    characterized_expected_payload(expected_result_type, fields.clone())
                }
            };
            assert_eq!(
                application_result.payload().as_value(),
                &expected_application_payload,
                "application projection: {expected_result_type}"
            );

            match (
                expected_transport,
                result.application_result_projection().unwrap(),
            ) {
                (
                    CharacterizedResultTransport::CommandPayload,
                    CoreResultProjection::CommandPayload(projection),
                )
                | (
                    CharacterizedResultTransport::ReadOnlyPreview,
                    CoreResultProjection::ReadOnlyPreview(projection),
                ) => {
                    assert_eq!(projection.result_type, expected_result_type);
                    assert_eq!(
                        projection.payload,
                        characterized_expected_payload(expected_result_type, fields.clone()),
                        "central projection: {expected_result_type}"
                    );
                }
                (
                    CharacterizedResultTransport::BattleUpdate,
                    CoreResultProjection::BattleUpdate,
                ) => {
                    assert!(fields.get("battle_update").is_some());
                }
                (expected, actual) => {
                    panic!(
                        "central transport drift for {expected_result_type}: expected {expected:?}, got {actual:?}"
                    )
                }
            }

            match (expected_transport, result.into_command_result_contract()) {
                (
                    CharacterizedResultTransport::CommandPayload,
                    BehaviorCommandResultContract::CommandPayload(payload),
                )
                | (
                    CharacterizedResultTransport::ReadOnlyPreview,
                    BehaviorCommandResultContract::ReadOnlyPreview(payload),
                ) => {
                    let expected_payload =
                        characterized_expected_payload(expected_result_type, fields);
                    assert_eq!(payload.result_type(), expected_result_type);
                    assert_eq!(
                        payload.payload_value().unwrap(),
                        expected_payload,
                        "{expected_result_type}"
                    );
                    if expected_result_type == "RunComplete" {
                        assert_eq!(
                            expected_payload,
                            serde_json::to_value(characterized_map()).unwrap()
                        );
                        assert!(expected_payload.get("map").is_none());
                    }
                }
                (
                    CharacterizedResultTransport::BattleUpdate,
                    BehaviorCommandResultContract::BattleUpdate(update),
                ) => {
                    assert_eq!(
                        serde_json::to_value(update.battle_update).unwrap(),
                        fields["battle_update"],
                        "{expected_result_type}"
                    );
                    let expected_setup = if expected_result_type == "BattleAdvanced" {
                        fields["battle_setup_snapshot"].clone()
                    } else {
                        Value::Null
                    };
                    assert_eq!(
                        serde_json::to_value(update.battle_setup_snapshot).unwrap(),
                        expected_setup,
                        "{expected_result_type}"
                    );
                }
                (expected, actual) => {
                    panic!(
                        "transport drift for {expected_result_type}: expected {expected:?}, got {actual:?}"
                    )
                }
            }
        }
    }

    #[test]
    fn retained_roster_result_api_delegates_to_the_central_projection() {
        let map = characterized_map();
        let expected_map = serde_json::to_value(&map).unwrap();
        let contract =
            CommandResultPayloadContract::Roster(RosterCommandResult::EmployeeRecruited {
                employee_definition_id: "public_recruit".to_string(),
                employee_uuid: Uuid::from_u128(0x33F0),
                completion: Box::new(CommandResultPayloadContract::Run(
                    RunCommandResult::RunComplete { map },
                )),
            });

        assert_eq!(contract.result_type(), "EmployeeRecruited");
        let payload = contract.payload_value().unwrap();
        assert_eq!(payload["completion"]["result_type"], "RunComplete");
        assert_eq!(payload["completion"]["payload"], expected_map);
        assert!(payload["completion"]["payload"].get("map").is_none());
    }

    #[test]
    #[should_panic(
        expected = "battle update results cannot be nested inside command result payloads"
    )]
    fn central_projection_rejects_nested_battle_updates() {
        let battle_uuid = Uuid::from_u128(0x33F1);
        let result = BehaviorResult::EmployeeRecruited {
            employee_definition_id: "invalid_nested_battle".to_string(),
            employee_uuid: Uuid::from_u128(0x33F2),
            completion: Box::new(BehaviorResult::BattleAdvanced {
                battle_uuid,
                encounter_id: "result_encounter".to_string(),
                node_type: CombatNodeType::Defense,
                mission_variant: CombatMissionVariant::Defense,
                battle_setup_snapshot: None,
                battle_update: characterized_battle_update(battle_uuid),
                finished: false,
            }),
        };

        let _ = result.application_result_projection();
    }

    #[test]
    fn live_battle_presentation_event_dto_keeps_stable_wire_shape() {
        let entry = BattleEventLogEntry {
            skill_effect_source: None,
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
