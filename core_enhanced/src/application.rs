//! Transport-neutral public application boundary for the Core runtime.
//!
//! This module owns commands, results, errors and read models. The domain root
//! stays private and cannot be borrowed or converted out of [`CoreApplication`].

mod fault;
pub use fault::{ApplicationBattleFault, BattleFaultCause, BattleFaultOperation};

use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::game::{
    behavior::{
        ActionKind, BattleStateAuthorityDto, BehaviorResult, CoreResultProjection, GameError,
        PlayerBehavior,
    },
    data::ValidatedGameData,
    encounter_compendium::{EncounterCompendiumProfileState, PlayerProfileState},
    world::{AdminCommand, GameCore},
};

// The feature-off expansion keeps the original operation body and has no
// observer storage, clock reads, TLS hooks or dependency on a collector.
macro_rules! observed_operation {
    ($this:ident, $operation:expr, $body:block) => {{
        #[cfg(feature = "runtime-observation")]
        let observation = $this.observer.as_mut().and_then(|observer| {
            observer.begin_with_identity(
                $operation,
                std::ptr::from_ref(&$this.core) as usize,
                $this.core.observation_identity(),
            )
        });
        let result = (|| $body)();
        #[cfg(feature = "runtime-observation")]
        if let Some(observation) = observation {
            observation.finish(&result);
        }
        result
    }};
}

/// Immutable semantic document owned by the Core application boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApplicationDocument(Value);

impl ApplicationDocument {
    pub fn new(value: Value) -> Self {
        Self(value)
    }

    pub fn null() -> Self {
        Self(Value::Null)
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

impl From<Value> for ApplicationDocument {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

macro_rules! define_application_command_kinds {
    ($($variant:ident),+ $(,)?) => {
        /// Exhaustive public identity of every player command owned by Core.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum ApplicationCommandKind {
            $($variant),+
        }

        impl ApplicationCommandKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            fn from_domain(kind: ActionKind) -> Self {
                match kind {
                    $(ActionKind::$variant => Self::$variant),+
                }
            }

            #[cfg(test)]
            fn into_domain(self) -> ActionKind {
                match self {
                    $(Self::$variant => ActionKind::$variant),+
                }
            }
        }
    };
}

define_application_command_kinds!(
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
    PreviewSkillFragmentBloom,
    PreviewRelicDismantle,
    DismantleRelic,
    RequestMapData,
    MoveToFacilityRoom,
    SelectMapNode,
    ConvertMapNodeWithMonolith,
    HatchEncounter,
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
    InvestRelicReward,
    SelectRelicFusionInputs,
    ConfirmRelicFusion,
    DealerAct,
    StressCombatAct,
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
    SelectSkillRelation,
    RetreatBattle,
    PauseBattle,
    ResumeBattle,
    SetBattleSpeed,
);

/// Validated application command. Its private domain command cannot escape.
#[derive(Debug, Clone)]
pub struct ApplicationCommand {
    kind: ApplicationCommandKind,
    document: ApplicationDocument,
    behavior: PlayerBehavior,
}

impl ApplicationCommand {
    /// Closed typed authoring queries move their domain inputs after the first
    /// admitted JSON projection; they need no clone-and-decode of a Value tree.
    pub(crate) fn for_authoring_preview(
        request: crate::authoring::TrialPreview,
        context: &ApplicationExecutionContext,
    ) -> Result<Self, ApplicationError> {
        request
            .admit(context)
            .map_err(ApplicationError::from_domain)?;
        let document = crate::authoring::projection::to_value(&request).map_err(|error| {
            ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidCommand,
                "invalid_application_command",
                &error,
            )
        })?;
        use crate::authoring::TrialPreview;
        let behavior = match request {
            TrialPreview::RequestDeploymentRangePreview {
                employee_uuid,
                position,
            } => PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position,
            },
            TrialPreview::PreviewSkillTarget {
                employee_uuid,
                skill_id,
                target,
            } => PlayerBehavior::PreviewSkillTarget {
                employee_uuid,
                skill_id,
                target,
            },
        };
        Ok(Self {
            kind: ApplicationCommandKind::from_domain(behavior.kind()),
            document: ApplicationDocument(document),
            behavior,
        })
    }

    /// Decode the complete command document, including its `type` member.
    pub fn from_document(document: ApplicationDocument) -> Result<Self, ApplicationError> {
        let behavior: PlayerBehavior =
            serde_json::from_value(document.0.clone()).map_err(|error| {
                ApplicationError::invalid_input(
                    ApplicationErrorKind::InvalidCommand,
                    "invalid_application_command",
                    &error,
                )
            })?;
        let kind = ApplicationCommandKind::from_domain(behavior.kind());
        Ok(Self {
            kind,
            document,
            behavior,
        })
    }

    /// Build a command from an exhaustive kind and an object containing only
    /// that command's arguments.
    pub fn try_new(
        kind: ApplicationCommandKind,
        arguments: ApplicationDocument,
    ) -> Result<Self, ApplicationError> {
        let Value::Object(mut object) = arguments.0 else {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidCommand,
                "invalid_application_command",
                "application command arguments must be an object",
            ));
        };
        let serialized_kind =
            serde_json::to_value(kind).expect("ApplicationCommandKind serialization is infallible");
        object.insert("type".to_string(), serialized_kind);
        let command = Self::from_document(ApplicationDocument(Value::Object(object)))?;
        if command.kind != kind {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidCommand,
                "invalid_application_command",
                "application command kind/payload mismatch",
            ));
        }
        Ok(command)
    }

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, ApplicationError> {
        let value = serde_json::from_slice(bytes).map_err(|error| {
            ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidCommand,
                "invalid_application_command",
                &error,
            )
        })?;
        Self::from_document(ApplicationDocument(value))
    }

    pub fn kind(&self) -> ApplicationCommandKind {
        self.kind
    }

    pub fn document(&self) -> &ApplicationDocument {
        &self.document
    }

    fn into_domain(self) -> PlayerBehavior {
        self.behavior
    }
}

/// Optional command metadata that has Core semantics but no transport owner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplicationExecutionContext {
    pub source_command_id: Option<String>,
    pub battle_event_after_seq: u64,
    pub expected_battle_authority: Option<ApplicationBattleAuthority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationBattleAuthority {
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
}

impl From<ApplicationBattleAuthority> for BattleStateAuthorityDto {
    fn from(authority: ApplicationBattleAuthority) -> Self {
        Self {
            battle_uuid: authority.battle_uuid,
            battle_state_revision: authority.battle_state_revision,
        }
    }
}

macro_rules! define_application_result_kinds {
    ($($variant:ident),+ $(,)?) => {
        /// Exhaustive public identity of every player-command result meaning.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(rename_all = "snake_case")]
        pub enum ApplicationResultKind {
            $($variant),+,
            /// Specialized Core admin application operation.
            AdminOperation,
        }

        impl ApplicationResultKind {
            pub const PLAYER_RESULTS: &'static [Self] = &[$(Self::$variant),+];

            pub const fn meaning(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),+,
                    Self::AdminOperation => "AdminOperation",
                }
            }
        }
    };
}

define_application_result_kinds!(
    RunStartFlowStarted,
    StarterEmployeesSelected,
    MapState,
    NodeEntered,
    NodePreview,
    NodeCompleted,
    SavePointState,
    MaintenanceState,
    HeadquartersContactState,
    FloorAdvanced,
    RunComplete,
    RunFailed,
    RunFailureDecisionRequired,
    RunAbandoned,
    UnEquipItem,
    EquipItem,
    ConsumableItemUsed,
    RelicState,
    RelicNumericPlan,
    RelicEffectivePlan,
    RelicPreview,
    SkillFragmentLoadoutUpdated,
    SkillFragmentFormSelected,
    SkillFragmentTransferred,
    SkillFragmentAwakened,
    SkillFragmentDismantled,
    EquipmentDismantled,
    EquipmentEnhanced,
    ShopState,
    RefreshShop,
    SellItem,
    PurchaseItem,
    EmployeeRecruited,
    EmergencySuppliesGranted,
    EventState,
    VisualNovelState,
    RewardGranted,
    CombatRewardsGranted,
    MasteredEliteSweepResultStaged,
    MasteredEliteSweepCompleted,
    BattleAdvanced,
    BattleState,
    DeploymentRangePreview,
    SkillTargetPreview,
    TacticalCommandPreview,
    BattlePlaybackChanged,
    BattleUnitDeployed,
    BattleUnitWithdrawn,
    BattleEngagementStanceChanged,
    BattleUnitTacticallyRelocated,
    BattleUnitsTacticallySwapped,
    BattleSkillTargetingFocusChanged,
    BattleSkillActivated,
    BattleRelationSelected,
    RewardState,
    DealerState,
    Ok,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationResultDisposition {
    Accepted,
    ReadOnly,
    AcceptedRecoveryPending,
}

/// Public application result containing no domain result or mutation handle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationResult {
    kind: ApplicationResultKind,
    meaning: String,
    disposition: ApplicationResultDisposition,
    payload: ApplicationDocument,
}

impl ApplicationResult {
    pub fn kind(&self) -> ApplicationResultKind {
        self.kind
    }

    pub fn meaning(&self) -> &str {
        &self.meaning
    }

    pub fn disposition(&self) -> ApplicationResultDisposition {
        self.disposition
    }

    pub fn payload(&self) -> &ApplicationDocument {
        &self.payload
    }

    pub(crate) fn from_behavior(result: &BehaviorResult) -> Result<Self, ApplicationError> {
        ApplicationError::check_projection_control()?;
        let kind = application_result_kind(result);
        match result.application_result_projection() {
            Ok(CoreResultProjection::CommandPayload(projection)) => Ok(Self {
                kind,
                meaning: kind.meaning().to_string(),
                disposition: ApplicationResultDisposition::Accepted,
                payload: ApplicationDocument(projection.payload),
            }),
            Ok(CoreResultProjection::ReadOnlyPreview(projection)) => Ok(Self {
                kind,
                meaning: kind.meaning().to_string(),
                disposition: ApplicationResultDisposition::ReadOnly,
                payload: ApplicationDocument(projection.payload),
            }),
            Ok(CoreResultProjection::BattleUpdate) => {
                match behavior_variant_payload(result, kind.meaning()) {
                    Ok(payload) => Ok(Self {
                        kind,
                        meaning: kind.meaning().to_string(),
                        disposition: if kind == ApplicationResultKind::BattleState {
                            ApplicationResultDisposition::ReadOnly
                        } else {
                            ApplicationResultDisposition::Accepted
                        },
                        payload: ApplicationDocument(payload),
                    }),
                    Err(error) => Self::projection_recovery_pending(kind, error),
                }
            }
            Err(error) => Self::projection_recovery_pending(kind, error),
        }
    }

    fn from_read_only_behavior(result: &BehaviorResult) -> Result<Self, ApplicationError> {
        let mut projection = Self::from_behavior(result)?;
        if projection.disposition != ApplicationResultDisposition::AcceptedRecoveryPending {
            projection.disposition = ApplicationResultDisposition::ReadOnly;
        }
        Ok(projection)
    }

    fn projection_recovery_pending(
        kind: ApplicationResultKind,
        error: impl fmt::Display,
    ) -> Result<Self, ApplicationError> {
        ApplicationError::check_projection_control()?;
        let message = crate::authoring::projection::display(error)
            .map_err(ApplicationError::diagnostic_interruption)?;
        let payload = crate::authoring::projection::object!({"projection_error": message})
            .map_err(ApplicationError::diagnostic_interruption)?;
        Ok(Self {
            kind,
            meaning: kind.meaning().to_string(),
            disposition: ApplicationResultDisposition::AcceptedRecoveryPending,
            payload: ApplicationDocument(payload),
        })
    }

    fn admin(meaning: &'static str, payload: Value) -> Self {
        Self {
            kind: ApplicationResultKind::AdminOperation,
            meaning: meaning.to_string(),
            disposition: ApplicationResultDisposition::Accepted,
            payload: ApplicationDocument(payload),
        }
    }
}

fn behavior_variant_payload(
    result: &BehaviorResult,
    expected_meaning: &str,
) -> serde_json::Result<Value> {
    let value = crate::authoring::projection::to_value(result)?;
    match value {
        Value::Object(mut object) if object.len() == 1 => object
            .remove(expected_meaning)
            .ok_or_else(|| serde_json::Error::io(std::io::Error::other("result kind mismatch"))),
        Value::String(unit) if unit == expected_meaning => Ok(Value::Null),
        _ => Err(serde_json::Error::io(std::io::Error::other(
            "result must have one exhaustive variant",
        ))),
    }
}

fn application_result_kind(result: &BehaviorResult) -> ApplicationResultKind {
    use ApplicationResultKind as K;
    match result {
        BehaviorResult::RunStartFlowStarted { .. } => K::RunStartFlowStarted,
        BehaviorResult::StarterEmployeesSelected { .. } => K::StarterEmployeesSelected,
        BehaviorResult::MapState { .. } => K::MapState,
        BehaviorResult::NodeEntered { .. } => K::NodeEntered,
        BehaviorResult::NodePreview { .. } => K::NodePreview,
        BehaviorResult::NodeCompleted { .. } => K::NodeCompleted,
        BehaviorResult::SavePointState { .. } => K::SavePointState,
        BehaviorResult::MaintenanceState { .. } => K::MaintenanceState,
        BehaviorResult::HeadquartersContactState { .. } => K::HeadquartersContactState,
        BehaviorResult::FloorAdvanced { .. } => K::FloorAdvanced,
        BehaviorResult::RunComplete { .. } => K::RunComplete,
        BehaviorResult::RunFailed { .. } => K::RunFailed,
        BehaviorResult::RunFailureDecisionRequired { .. } => K::RunFailureDecisionRequired,
        BehaviorResult::RunAbandoned => K::RunAbandoned,
        BehaviorResult::UnEquipItem { .. } => K::UnEquipItem,
        BehaviorResult::EquipItem { .. } => K::EquipItem,
        BehaviorResult::ConsumableItemUsed { .. } => K::ConsumableItemUsed,
        BehaviorResult::RelicState { .. } => K::RelicState,
        BehaviorResult::RelicNumericPlan { .. } => K::RelicNumericPlan,
        BehaviorResult::RelicEffectivePlan { .. } => K::RelicEffectivePlan,
        BehaviorResult::RelicPreview { .. } => K::RelicPreview,
        BehaviorResult::SkillFragmentLoadoutUpdated { .. } => K::SkillFragmentLoadoutUpdated,
        BehaviorResult::SkillFragmentFormSelected { .. } => K::SkillFragmentFormSelected,
        BehaviorResult::SkillFragmentTransferred { .. } => K::SkillFragmentTransferred,
        BehaviorResult::SkillFragmentAwakened { .. } => K::SkillFragmentAwakened,
        BehaviorResult::SkillFragmentDismantled { .. } => K::SkillFragmentDismantled,
        BehaviorResult::EquipmentDismantled { .. } => K::EquipmentDismantled,
        BehaviorResult::EquipmentEnhanced { .. } => K::EquipmentEnhanced,
        BehaviorResult::ShopState { .. } => K::ShopState,
        BehaviorResult::RefreshShop { .. } => K::RefreshShop,
        BehaviorResult::SellItem { .. } => K::SellItem,
        BehaviorResult::PurchaseItem { .. } => K::PurchaseItem,
        BehaviorResult::EmployeeRecruited { .. } => K::EmployeeRecruited,
        BehaviorResult::EmergencySuppliesGranted { .. } => K::EmergencySuppliesGranted,
        BehaviorResult::EventState { .. } => K::EventState,
        BehaviorResult::VisualNovelState { .. } => K::VisualNovelState,
        BehaviorResult::RewardGranted { .. } => K::RewardGranted,
        BehaviorResult::CombatRewardsGranted { .. } => K::CombatRewardsGranted,
        BehaviorResult::MasteredEliteSweepResultStaged { .. } => K::MasteredEliteSweepResultStaged,
        BehaviorResult::MasteredEliteSweepCompleted { .. } => K::MasteredEliteSweepCompleted,
        BehaviorResult::BattleAdvanced { .. } => K::BattleAdvanced,
        BehaviorResult::BattleState { .. } => K::BattleState,
        BehaviorResult::DeploymentRangePreview { .. } => K::DeploymentRangePreview,
        BehaviorResult::SkillTargetPreview { .. } => K::SkillTargetPreview,
        BehaviorResult::TacticalCommandPreview { .. } => K::TacticalCommandPreview,
        BehaviorResult::BattlePlaybackChanged { .. } => K::BattlePlaybackChanged,
        BehaviorResult::BattleUnitDeployed { .. } => K::BattleUnitDeployed,
        BehaviorResult::BattleUnitWithdrawn { .. } => K::BattleUnitWithdrawn,
        BehaviorResult::BattleEngagementStanceChanged { .. } => K::BattleEngagementStanceChanged,
        BehaviorResult::BattleUnitTacticallyRelocated { .. } => K::BattleUnitTacticallyRelocated,
        BehaviorResult::BattleUnitsTacticallySwapped { .. } => K::BattleUnitsTacticallySwapped,
        BehaviorResult::BattleSkillTargetingFocusChanged { .. } => {
            K::BattleSkillTargetingFocusChanged
        }
        BehaviorResult::BattleSkillActivated { .. } => K::BattleSkillActivated,
        BehaviorResult::BattleRelationSelected { .. } => K::BattleRelationSelected,
        BehaviorResult::RewardState { .. } => K::RewardState,
        BehaviorResult::DealerState { .. } => K::DealerState,
        BehaviorResult::Ok => K::Ok,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationErrorKind {
    BattleRuntimeFault,
    InvalidCommand,
    InvalidAdminCommand,
    InvalidProfile,
    ProjectionFailure,
    EventNotFound,
    EventTypeMismatch,
    InvalidAction,
    TutorialCompletionRequired,
    ActionUnavailable,
    WeaponRequired,
    InvalidBattleResyncSeq,
    BattleSnapshotResyncRequired,
    BattleStateRevisionOverflow,
    MissingBattleStateAuthority,
    StaleBattleStateAuthority,
    NotInShopState,
    NotInRewardState,
    ShopItemNotFound,
    InventoryFull,
    InventoryItemNotFound,
    InventoryItemNotRemovable,
    InsufficientResources,
    MissingResource,
    InvalidUnitStats,
    InvalidStaticData,
    NotImplemented,
    SkillFragmentIncompatible,
    OutOfBounds,
    PositionOccupied,
    StaticObstacleBlocked,
    UnitAlreadyPlaced,
    UnitNotFound,
}

impl ApplicationErrorKind {
    pub const DOMAIN_ERRORS: &'static [Self] = &[
        Self::BattleRuntimeFault,
        Self::EventNotFound,
        Self::EventTypeMismatch,
        Self::InvalidAction,
        Self::TutorialCompletionRequired,
        Self::ActionUnavailable,
        Self::WeaponRequired,
        Self::InvalidBattleResyncSeq,
        Self::BattleSnapshotResyncRequired,
        Self::BattleStateRevisionOverflow,
        Self::MissingBattleStateAuthority,
        Self::StaleBattleStateAuthority,
        Self::NotInShopState,
        Self::NotInRewardState,
        Self::ShopItemNotFound,
        Self::InventoryFull,
        Self::InventoryItemNotFound,
        Self::InventoryItemNotRemovable,
        Self::InsufficientResources,
        Self::MissingResource,
        Self::InvalidUnitStats,
        Self::InvalidStaticData,
        Self::NotImplemented,
        Self::SkillFragmentIncompatible,
        Self::OutOfBounds,
        Self::PositionOccupied,
        Self::StaticObstacleBlocked,
        Self::UnitAlreadyPlaced,
        Self::UnitNotFound,
    ];
}

/// Stable transport-neutral rejection returned by every public application port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationError {
    kind: ApplicationErrorKind,
    code: String,
    details: ApplicationDocument,
}

impl ApplicationError {
    pub fn kind(&self) -> ApplicationErrorKind {
        self.kind
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn details(&self) -> &ApplicationDocument {
        &self.details
    }

    // This empty envelope is exclusively an Err from a private active scope.
    // It is discarded when that scope returns its typed first interruption.
    fn check_projection_control() -> Result<(), Self> {
        if crate::authoring::control::interruption("application_diagnostic_entry").is_some()
            || crate::game::battle::runtime_control::poll().is_err()
        {
            Err(Self {
                kind: ApplicationErrorKind::ProjectionFailure,
                code: String::new(),
                details: ApplicationDocument(Value::Null),
            })
        } else {
            Ok(())
        }
    }

    fn diagnostic_interruption(error: serde_json::Error) -> Self {
        match Self::check_projection_control() {
            Err(interruption) => interruption,
            // These private fields are literal keys, supported native values
            // and a Display string; json! previously required the same invariant.
            Ok(()) => panic!("infallible application diagnostic serialization: {error}"),
        }
    }

    fn invalid_input(
        kind: ApplicationErrorKind,
        code: &'static str,
        message: impl fmt::Display,
    ) -> Self {
        if let Err(interruption) = Self::check_projection_control() {
            return interruption;
        }
        let message = match crate::authoring::projection::display(message) {
            Ok(message) => message,
            Err(error) => return Self::diagnostic_interruption(error),
        };
        let details = match crate::authoring::projection::object!({"message": message}) {
            Ok(details) => details,
            Err(error) => return Self::diagnostic_interruption(error),
        };
        Self {
            kind,
            code: code.to_string(),
            details: ApplicationDocument(details),
        }
    }

    pub(crate) fn from_domain(error: GameError) -> Self {
        if let Err(interruption) = Self::check_projection_control() {
            return interruption;
        }
        macro_rules! json {
            ($($tokens:tt)*) => {
                match crate::authoring::projection::object!($($tokens)*) {
                    Ok(details) => details,
                    Err(error) => return Self::diagnostic_interruption(error),
                }
            };
        }
        use ApplicationErrorKind as K;
        let code = error.code();
        let (kind, details) = match error {
            GameError::EventNotFound => (K::EventNotFound, Value::Null),
            GameError::EventTypeMismatch => (K::EventTypeMismatch, Value::Null),
            GameError::InvalidAction => (K::InvalidAction, Value::Null),
            GameError::TutorialCompletionRequired { game_mode } => (
                K::TutorialCompletionRequired,
                json!({ "game_mode": game_mode }),
            ),
            GameError::ActionUnavailable { action, reason } => (
                K::ActionUnavailable,
                json!({ "action": action, "reason": reason }),
            ),
            GameError::WeaponRequired => (K::WeaponRequired, Value::Null),
            GameError::InvalidBattleResyncSeq { requested, latest } => (
                K::InvalidBattleResyncSeq,
                json!({ "requested": requested, "latest": latest }),
            ),
            GameError::BattleSnapshotResyncRequired {
                requested,
                oldest_available_after_seq,
                latest,
            } => (
                K::BattleSnapshotResyncRequired,
                json!({
                    "requested": requested,
                    "oldest_available_after_seq": oldest_available_after_seq,
                    "latest": latest,
                }),
            ),
            GameError::BattleStateRevisionOverflow => (K::BattleStateRevisionOverflow, Value::Null),
            GameError::BattleRuntimeFault(fault) => (
                K::BattleRuntimeFault,
                json!({
                    "code": fault.code,
                    "site": fault.site,
                    "limit": fault.limit,
                    "requested": fault.requested,
                }),
            ),
            GameError::ValueExpressionFault(fault) => {
                (K::InvalidStaticData, json!({ "fault": fault }))
            }
            GameError::MissingBattleStateAuthority { current } => (
                K::MissingBattleStateAuthority,
                json!({ "current": current }),
            ),
            GameError::StaleBattleStateAuthority { expected, current } => (
                K::StaleBattleStateAuthority,
                json!({ "expected": expected, "current": current }),
            ),
            GameError::NotInShopState => (K::NotInShopState, Value::Null),
            GameError::NotInRewardState => (K::NotInRewardState, Value::Null),
            GameError::ShopItemNotFound => (K::ShopItemNotFound, Value::Null),
            GameError::InventoryFull => (K::InventoryFull, Value::Null),
            GameError::InventoryItemNotFound => (K::InventoryItemNotFound, Value::Null),
            GameError::InventoryItemNotRemovable => (K::InventoryItemNotRemovable, Value::Null),
            GameError::InsufficientResources => (K::InsufficientResources, Value::Null),
            GameError::MissingResource(resource) => {
                (K::MissingResource, json!({ "resource": resource }))
            }
            GameError::InvalidUnitStats(reason) => {
                (K::InvalidUnitStats, json!({ "reason": reason }))
            }
            GameError::InvalidStaticData(reason) | GameError::InvalidCommandData(reason) => {
                (K::InvalidStaticData, json!({ "reason": reason }))
            }
            GameError::NotImplemented(feature) => {
                (K::NotImplemented, json!({ "feature": feature }))
            }
            GameError::SkillFragmentIncompatible {
                fragment_id,
                failure_codes,
            } => (
                K::SkillFragmentIncompatible,
                json!({
                    "fragment_id": fragment_id,
                    "failure_codes": failure_codes,
                }),
            ),
            GameError::OutOfBounds => (K::OutOfBounds, Value::Null),
            GameError::PositionOccupied => (K::PositionOccupied, Value::Null),
            GameError::StaticObstacleBlocked => (K::StaticObstacleBlocked, Value::Null),
            GameError::UnitAlreadyPlaced => (K::UnitAlreadyPlaced, Value::Null),
            GameError::UnitNotFound => (K::UnitNotFound, Value::Null),
        };
        Self {
            kind,
            code: code.to_string(),
            details: ApplicationDocument(details),
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({:?})", self.code, self.kind)
    }
}

impl std::error::Error for ApplicationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationSnapshotKind {
    Run,
    Inventory,
    EmployeeRoster,
    SelectedEvent,
    ActionCapabilities,
    PlayerProfile,
    EncounterCompendium,
}

impl ApplicationSnapshotKind {
    pub const ALL: &'static [Self] = &[
        Self::Run,
        Self::Inventory,
        Self::EmployeeRoster,
        Self::SelectedEvent,
        Self::ActionCapabilities,
        Self::PlayerProfile,
        Self::EncounterCompendium,
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationSnapshot {
    kind: ApplicationSnapshotKind,
    read_model: ApplicationDocument,
}

impl ApplicationSnapshot {
    pub fn kind(&self) -> ApplicationSnapshotKind {
        self.kind
    }

    pub fn read_model(&self) -> &ApplicationDocument {
        &self.read_model
    }
}

/// Exact Core checkpoint bytes and host-monotonic metadata. The payload codec
/// remains owned by P06 and is not an external storage envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationCheckpoint {
    core_payload_json: Arc<[u8]>,
    loads_used: u8,
    run_peak_certified_power: u64,
}

impl ApplicationCheckpoint {
    /// The current Core writer format. Storage hosts may preserve these opaque
    /// bytes; Core still validates the full payload and retained generation.
    pub const FORMAT_VERSION: u32 = GameCore::active_run_checkpoint_format_version();


    pub fn new(core_payload_json: Vec<u8>, loads_used: u8, run_peak_certified_power: u64) -> Self {
        Self {
            core_payload_json: core_payload_json.into(),
            loads_used,
            run_peak_certified_power,
        }
    }

    pub fn core_payload_json(&self) -> &[u8] {
        &self.core_payload_json
    }

    pub fn loads_used(&self) -> u8 {
        self.loads_used
    }

    pub fn run_peak_certified_power(&self) -> u64 {
        self.run_peak_certified_power
    }
}

/// Validated admin application command with no public domain command alias.
#[derive(Debug, Clone)]
pub struct ApplicationAdminCommand {
    document: ApplicationDocument,
    command: AdminCommand,
}

impl ApplicationAdminCommand {
    pub fn from_document(document: ApplicationDocument) -> Result<Self, ApplicationError> {
        let command = serde_json::from_value(document.0.clone()).map_err(|error| {
            ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAdminCommand,
                "invalid_admin_application_command",
                &error,
            )
        })?;
        Ok(Self { document, command })
    }

    pub fn document(&self) -> &ApplicationDocument {
        &self.document
    }
}

/// Sole public owner of the authoritative Core application root.
pub struct CoreApplication {
    core: GameCore,
    battle_fault: Option<ApplicationBattleFault>,
    transaction_owner: Arc<()>,
    transaction_revision: u64,
    #[cfg(feature = "runtime-observation")]
    observer: Option<crate::runtime_observation::SessionObserver>,
}

/// Single-use host persistence rollback for one subsequent application attempt.
/// The private snapshot can only originate outside an active battle and belongs
/// to this exact application. It is neither serializable nor cloneable.
pub struct ApplicationPersistenceRollback {
    owner: Arc<()>,
    revision: u64,
    rollback: crate::game::world::ActiveRunTransactionRollback,
}

/// A staged command that exclusively borrows its originating application.
/// Dropping it discards the draft; commit consumes it without running gameplay
/// again. The result and durable checkpoint are prepared before host publication.
pub struct PreparedApplication<'a, T> {
    application: &'a mut CoreApplication,
    prepared: crate::game::world::PreparedApplicationCommit<T>,
    checkpoint: Option<ApplicationCheckpoint>,
    profile: PlayerProfileState,
    enters_battle: bool,
    terminal: bool,
    #[cfg(feature = "runtime-observation")]
    observation: Option<crate::runtime_observation::PendingObservation>,
}

pub type PreparedNonBattleApplication<'a> = PreparedApplication<'a, ApplicationResult>;
pub type PreparedHostTick<'a> = PreparedApplication<'a, Option<ApplicationResult>>;

impl<T> PreparedApplication<'_, T> {
    pub fn result(&self) -> &T {
        self.prepared.output()
    }
    pub fn checkpoint(&self) -> Option<&ApplicationCheckpoint> {
        self.checkpoint.as_ref()
    }
    pub fn player_profile(&self) -> &PlayerProfileState {
        &self.profile
    }
    pub fn enters_battle(&self) -> bool {
        self.enters_battle
    }
    pub fn terminal(&self) -> bool {
        self.terminal
    }

    pub fn commit(self) -> T {
        let result = self
            .application
            .core
            .commit_prepared_application(self.prepared);
        #[cfg(feature = "runtime-observation")]
        if let Some(observation) = self.observation {
            observation.commit();
        }
        result
    }
}

impl CoreApplication {
    /// Install a host-owned observation lease. It never changes safety settings
    /// or gameplay identity. Replacing/dropping it retires the previous session.
    #[cfg(feature = "runtime-observation")]
    pub fn set_runtime_observer(
        &mut self,
        observer: Option<crate::runtime_observation::SessionObserver>,
    ) {
        self.observer = observer;
    }

    pub(crate) fn new_for_authoring(game_data: ValidatedGameData, run_seed: u64) -> Self {
        Self {
            core: GameCore::new_for_authoring(game_data, run_seed),
            transaction_owner: Arc::new(()),
            transaction_revision: 0,
            battle_fault: None,
            #[cfg(feature = "runtime-observation")]
            observer: None,
        }
    }

    pub(crate) fn set_authoring_trace_options(
        &mut self,
        options: crate::authoring::trace::TraceOptions,
    ) {
        self.core.set_authoring_trace_options(options);
    }

    pub(crate) fn set_amount_observation_enabled(&mut self, enabled: bool) {
        self.core.set_amount_observation_enabled(enabled);
    }

    pub(crate) fn prepare_lab(&mut self,recipe:&crate::authoring::LabPreparation)->Result<crate::authoring::LabPreparationReceipt,ApplicationError>{
        self.core.prepare_lab(recipe).map_err(ApplicationError::from_domain)
    }
    pub(crate) fn set_lab_coverage(&mut self,plan:Option<Arc<crate::mod_validation::CoveragePlan>>)->Result<(),ApplicationError>{
        self.core.set_lab_coverage(plan).map_err(ApplicationError::from_domain)
    }
    pub(crate) fn lab_coverage(&self)->Result<Option<crate::mod_validation::CoverageReport>,ApplicationError>{
        self.core.lab_coverage().map_err(ApplicationError::from_domain)
    }

    pub(crate) fn authoring_trace(
        &self,
    ) -> Result<crate::game::world::authoring_observation::TrialTraceView<'_>, ApplicationError>
    {
        self.core
            .authoring_trace()
            .map_err(ApplicationError::from_domain)
    }

    pub(crate) fn authoring_observation(
        &self,
    ) -> Result<crate::game::world::authoring_observation::TrialObservationView<'_>, ApplicationError>
    {
        self.core
            .authoring_observation()
            .map_err(ApplicationError::from_domain)
    }

    pub(crate) fn amount_observation_enabled(&self) -> Result<bool, ApplicationError> {
        self.core
            .amount_observation_enabled()
            .map_err(ApplicationError::from_domain)
    }

    pub(crate) fn stored_amount(
        &self,
        window: crate::game::world::authoring_observation::StoredAmountWindow,
    ) -> Result<
        Option<crate::game::world::authoring_observation::StoredAmountView<'_>>,
        ApplicationError,
    > {
        self.core
            .stored_amount(window)
            .map_err(ApplicationError::from_domain)
    }

    pub(crate) fn authoring_lifecycle(
        &self,
    ) -> Option<crate::game::world::authoring_lifecycle::TrialLifecycle> {
        self.core.authoring_lifecycle()
    }

    /// Complete opaque fork for the private trial owner. The caller installs
    /// that session's native-storage provenance before admission and commits the
    /// application, provenance and mandatory receipt together after execution.
    pub(crate) fn fork_for_authoring(&self) -> Result<Self, ApplicationError> {
        Ok(Self {
            core: crate::authoring::clone_shape::controlled_clone(&self.core)
                .map_err(ApplicationError::from_domain)?,
            transaction_owner: Arc::new(()),
            transaction_revision: 0,
            battle_fault: self.battle_fault,
            #[cfg(feature = "runtime-observation")]
            observer: None,
        })
    }

    /// Fixed-size identity of the current committed owner. It is observation
    /// metadata and does not grant settings or checkpoint authority.
    #[cfg(feature = "runtime-observation")]
    pub fn measurement_identity(&self) -> crate::runtime_observation::MeasurementIdentity {
        self.core.observation_identity()
    }

    pub fn new(game_data: impl Into<ValidatedGameData>, run_seed: u64) -> Self {
        Self {
            core: GameCore::new(game_data, run_seed),
            transaction_owner: Arc::new(()),
            transaction_revision: 0,
            battle_fault: None,
            #[cfg(feature = "runtime-observation")]
            observer: None,
        }
    }

    #[cfg(feature = "test-fixtures")]
    pub fn replace_active_battle_event_log_fixture(
        &mut self,
        retained_entries: usize,
    ) -> Result<u64, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        use crate::game::battle::{
            event_log::{BattleEventCause, BattleEventLog, BattleEventLogEntry, BattleLogEvent},
            types::BattleWinner,
        };

        self.advance_transaction_revision();
        let event_log = BattleEventLog::from_entries(
            (1..=retained_entries)
                .map(|seq| BattleEventLogEntry {
                    skill_effect_source: None,
                    time_ms: 0,
                    seq: seq as u64,
                    cause: BattleEventCause::default(),
                    source_command_id: None,
                    event: BattleLogEvent::BattleEnd {
                        winner: BattleWinner::Draw,
                    },
                })
                .collect(),
        );
        self.core
            .replace_active_battle_event_log_fixture(event_log)
            .map_err(ApplicationError::from_domain)
    }

    #[cfg(test)]
    pub(crate) fn from_core(core: GameCore) -> Self {
        Self {
            core,
            transaction_owner: Arc::new(()),
            transaction_revision: 0,
            battle_fault: None,
            #[cfg(feature = "runtime-observation")]
            observer: None,
        }
    }

    pub fn execute(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.execute_internal(player_id, command, context, None)
    }

    /// Compatibility entry for callers that explicitly require no active battle.
    pub fn prepare_non_battle_command(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
    ) -> Result<PreparedNonBattleApplication<'_>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        if self.has_active_battle() {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAction,
                "active_battle_preparation",
                "non-battle preparation is unavailable during an active battle",
            ));
        }
        self.prepare_command(player_id, command, context)
    }

    /// Stage either an active-battle or non-battle command. All fallible gameplay,
    /// projection and durable metadata preparation precedes the exclusive commit.
    pub fn prepare_command(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
    ) -> Result<PreparedNonBattleApplication<'_>, ApplicationError> {
        self.prepare_command_with_generation(player_id, command, context, None)
    }

    pub fn prepare_command_with_generation(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
        generation: Option<ValidatedGameData>,
    ) -> Result<PreparedNonBattleApplication<'_>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        let operation = BattleFaultOperation::Command(command.kind());
        let runtime_mutation =
            command.behavior.kind().scope() != crate::game::behavior::ActionScope::Query;
        #[cfg(feature = "runtime-observation")]
        let observation = self.observer.as_mut().and_then(|observer| {
            observer.begin_with_identity(
                crate::runtime_observation::Operation::Command(command.kind()),
                std::ptr::from_ref(&self.core) as usize,
                self.core.observation_identity(),
            )
        });
        self.advance_transaction_revision();
        let candidate = self
            .core
            .prepare_with_transport_context(
                player_id,
                command.into_domain(),
                context.source_command_id.as_deref(),
                context.battle_event_after_seq,
                context.expected_battle_authority.map(Into::into),
                generation,
            )
            .map_err(|error| self.runtime_error(error, operation, runtime_mutation));
        let prepared = candidate.and_then(|prepared| {
            let result = ApplicationResult::from_behavior(prepared.output())?;
            self.prepare_publication(prepared.with_output(result))
        });
        #[cfg(feature = "runtime-observation")]
        return match prepared {
            Ok(mut prepared) => {
                prepared.observation = observation.map(|guard| guard.prepared());
                Ok(prepared)
            }
            Err(error) => {
                if let Some(guard) = observation {
                    guard.finish_error(&error);
                }
                Err(error)
            }
        };
        #[cfg(not(feature = "runtime-observation"))]
        prepared
    }

    pub fn prepare_active_battle_for_host_tick(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<PreparedHostTick<'_>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        #[cfg(feature = "runtime-observation")]
        let observation = self.observer.as_mut().and_then(|observer| {
            observer.begin_with_identity(
                crate::runtime_observation::Operation::HostTick,
                std::ptr::from_ref(&self.core) as usize,
                self.core.observation_identity(),
            )
        });
        self.advance_transaction_revision();
        let candidate = self
            .core
            .prepare_active_battle_for_host_tick(raw_delta_ms, after_seq)
            .map_err(|error| self.runtime_error(error, BattleFaultOperation::HostTick, true));
        let prepared = candidate.and_then(|prepared| {
            let result = prepared
                .output()
                .as_ref()
                .map(ApplicationResult::from_behavior)
                .transpose()?;
            self.prepare_publication(prepared.with_output(result))
        });
        #[cfg(feature = "runtime-observation")]
        return match prepared {
            Ok(mut prepared) => {
                prepared.observation = observation.map(|guard| guard.prepared());
                Ok(prepared)
            }
            Err(error) => {
                if let Some(guard) = observation {
                    guard.finish_error(&error);
                }
                Err(error)
            }
        };
        #[cfg(not(feature = "runtime-observation"))]
        prepared
    }

    fn prepare_publication<T>(
        &mut self,
        mut prepared: crate::game::world::PreparedApplicationCommit<T>,
    ) -> Result<PreparedApplication<'_, T>, ApplicationError> {
        let checkpoint = prepared
            .draft()
            .export_active_run_checkpoint()
            .map_err(ApplicationError::from_domain)?
            .map(|checkpoint| ApplicationCheckpoint {
                core_payload_json: checkpoint.core_payload_json,
                loads_used: checkpoint.loads_used,
                run_peak_certified_power: checkpoint.run_peak_certified_power,
            });
        let profile = prepared
            .take_publication_profile()
            .expect("publication preparation captures the profile inside its operation budget");
        let enters_battle = matches!(
            prepared.draft().get_state(),
            crate::game::resources::GameState::InBattle { .. }
        );
        let terminal = prepared.draft().durable_run_is_terminal();
        Ok(PreparedApplication {
            application: self,
            prepared,
            checkpoint,
            profile,
            enters_battle,
            terminal,
            #[cfg(feature = "runtime-observation")]
            observation: None,
        })
    }

    /// Install a validated generation only through an eligible StartNewGame
    /// draft. State, seed and token commit together after preparation succeeds.
    pub fn execute_with_generation(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
        generation: ValidatedGameData,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        if command.kind() != ApplicationCommandKind::StartNewGame {
            return observed_operation!(
                self,
                crate::runtime_observation::Operation::Command(command.kind()),
                { Err(ApplicationError::from_domain(GameError::InvalidAction)) }
            );
        }
        self.execute_internal(player_id, command, context, Some(generation))
    }

    // Saturation permanently closes rollback admission; no wrapped revision
    // can make a stale token current. Gameplay execution does not depend on it.
    fn advance_transaction_revision(&mut self) {
        self.transaction_revision = self.transaction_revision.saturating_add(1);
    }

    pub fn capture_persistence_rollback(
        &self,
    ) -> Result<Option<ApplicationPersistenceRollback>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        if self.transaction_revision >= u64::MAX - 1 {
            return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAction,
                "persistence_rollback_revision_exhausted",
                "persistence rollback revision is exhausted",
            ));
        }
        self.core
            .capture_active_run_transaction_rollback()
            .map(|rollback| {
                rollback.map(|rollback| ApplicationPersistenceRollback {
                    owner: self.transaction_owner.clone(),
                    revision: self.transaction_revision,
                    rollback,
                })
            })
            .map_err(ApplicationError::from_domain)
    }

    pub fn restore_persistence_rollback(
        &mut self,
        token: ApplicationPersistenceRollback,
    ) -> Result<(), ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(
            self,
            crate::runtime_observation::Operation::RestoreRollback,
            {
                if !Arc::ptr_eq(&self.transaction_owner, &token.owner)
                    || token.revision.checked_add(1) != Some(self.transaction_revision)
                    || self.transaction_revision == u64::MAX
                {
                    return Err(ApplicationError::invalid_input(
                ApplicationErrorKind::InvalidAction,
                "stale_persistence_rollback",
                "rollback must belong to this application and exactly one subsequent attempt",
            ));
                }
                if self.has_active_battle() {
                    return Err(ApplicationError::invalid_input(
                        ApplicationErrorKind::InvalidAction,
                        "active_battle_persistence_rollback",
                        "persistence rollback is unavailable during an active battle",
                    ));
                }
                self.advance_transaction_revision();
                self.core
                    .try_restore_active_run_transaction_rollback(token.rollback)
                    .map_err(ApplicationError::from_domain)
            }
        )
    }

    pub fn has_active_battle(&self) -> bool {
        matches!(
            self.core.get_state(),
            crate::game::resources::GameState::InBattle { .. }
        )
    }

    pub fn is_not_started(&self) -> bool {
        matches!(
            self.core.get_state(),
            crate::game::resources::GameState::NotStarted
        )
    }

    pub fn runtime_safety(&self) -> &crate::runtime_safety::RuntimeSafetySettings {
        self.core.runtime_safety()
    }

    pub fn content_identity(&self) -> &crate::game::data::content_pack::ContentIdentity {
        self.core.content_identity()
    }

    fn execute_internal(
        &mut self,
        player_id: Uuid,
        command: ApplicationCommand,
        context: ApplicationExecutionContext,
        generation: Option<ValidatedGameData>,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        let operation = BattleFaultOperation::Command(command.kind());
        let runtime_mutation =
            command.behavior.kind().scope() != crate::game::behavior::ActionScope::Query;
        observed_operation!(
            self,
            crate::runtime_observation::Operation::Command(command.kind()),
            {
                self.advance_transaction_revision();
                let expected_battle_authority = context.expected_battle_authority.map(Into::into);
                let result = self
                    .core
                    .execute_with_transport_context_and_generation(
                        player_id,
                        command.into_domain(),
                        context.source_command_id.as_deref(),
                        context.battle_event_after_seq,
                        expected_battle_authority,
                        generation,
                    )
                    .map_err(|error| self.runtime_error(error, operation, runtime_mutation))?;
                ApplicationResult::from_behavior(&result)
            }
        )
    }

    /// Durable Run projection; an active battle's uncommitted overlay is queried separately.
    pub fn run_resources(&self) -> Option<&crate::game::resource::RunResourceState> {
        self.core.run_resources()
    }

    pub fn battle_typed_resources(&self) -> Option<crate::game::resource::ResourceView<'_>> {
        self.core.battle_typed_resources()
    }

    /// Always-on facts for the active battle, or its current terminal result.
    /// This read does not reserve a result or depend on event-log retention.
    pub fn battle_runtime_facts(
        &self,
    ) -> Result<Option<crate::game::battle::runtime_facts::RuntimeFactView<'_>>, ApplicationError>
    {
        crate::authoring::control::poll("battle_fact_view")
            .map_err(ApplicationError::from_domain)?;
        Ok(self.core.battle_runtime_facts())
    }

    pub fn certified_input_fingerprint(
        &self,
    ) -> Result<crate::game::combat_power::CertifiedInputFingerprint, ApplicationError> {
        self.core
            .certified_input_fingerprint()
            .map_err(ApplicationError::from_domain)
    }

    pub fn snapshot(
        &self,
        kind: ApplicationSnapshotKind,
    ) -> Result<ApplicationSnapshot, ApplicationError> {
        if kind == ApplicationSnapshotKind::ActionCapabilities && self.battle_fault.is_some() {
            return Ok(ApplicationSnapshot {
                kind,
                read_model: ApplicationDocument(json!([])),
            });
        }
        let mut value = match kind {
            ApplicationSnapshotKind::Run => self.core.get_run_snapshot_json(),
            ApplicationSnapshotKind::Inventory => self.core.get_inventory_snapshot_json(),
            ApplicationSnapshotKind::EmployeeRoster => {
                self.core.get_employee_roster_snapshot_json()
            }
            ApplicationSnapshotKind::SelectedEvent => self
                .core
                .get_selected_event_snapshot_json()
                .map(|selected| selected.unwrap_or(Value::Null)),
            ApplicationSnapshotKind::ActionCapabilities => {
                crate::authoring::projection::to_value(self.core.get_action_capabilities())
                    .map_err(|error| GameError::InvalidStaticData(error.to_string()))
            }
            ApplicationSnapshotKind::PlayerProfile => {
                crate::authoring::projection::to_value(self.core.player_profile())
                    .map_err(|error| GameError::InvalidStaticData(error.to_string()))
            }
            ApplicationSnapshotKind::EncounterCompendium => {
                crate::authoring::projection::to_value(self.core.encounter_compendium_profile())
                    .map_err(|error| GameError::InvalidStaticData(error.to_string()))
            }
        }
        .map_err(ApplicationError::from_domain)?;
        if kind == ApplicationSnapshotKind::Run {
            if let Some(fault) = self.battle_fault {
                value["battle_fault"] = json!(fault);
                value["action_capabilities"] = json!([]);
                value["game_state_context"]["can_retreat"] = json!(false);
                value["game_state_context"]["deployment"] = Value::Null;
            }
        }
        Ok(ApplicationSnapshot {
            kind,
            read_model: ApplicationDocument(value),
        })
    }

    pub fn active_battle_reconnect(
        &self,
        after_seq: u64,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        let result = self
            .core
            .active_battle_reconnect_projection(after_seq)
            .map_err(ApplicationError::from_domain)?;
        ApplicationResult::from_read_only_behavior(&result)
    }

    pub fn advance_active_battle_by(
        &mut self,
        delta_ms: u64,
        after_seq: u64,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::Elapsed, {
            self.advance_transaction_revision();
            let result = self
                .core
                .advance_active_battle_by_after_seq(delta_ms, after_seq)
                .map_err(|error| self.runtime_error(error, BattleFaultOperation::Elapsed, true))?;
            ApplicationResult::from_behavior(&result)
        })
    }

    pub fn advance_active_battle_for_host_tick(
        &mut self,
        raw_delta_ms: u64,
        after_seq: u64,
    ) -> Result<Option<ApplicationResult>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::HostTick, {
            self.advance_transaction_revision();
            self.core
                .advance_active_battle_for_server_tick_after_seq(raw_delta_ms, after_seq)
                .map_err(|error| self.runtime_error(error, BattleFaultOperation::HostTick, true))?
                .as_ref()
                .map(ApplicationResult::from_behavior)
                .transpose()
        })
    }

    pub fn advance_active_battle_to_next_event_bucket(
        &mut self,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::NextEvent, {
            self.advance_transaction_revision();
            let result = self
                .core
                .advance_active_battle_to_next_event_bucket()
                .map_err(|error| {
                    self.runtime_error(error, BattleFaultOperation::NextEvent, true)
                })?;
            ApplicationResult::from_behavior(&result)
        })
    }

    pub fn pause_active_battle_for_disconnect(&mut self) -> Result<bool, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(
            self,
            crate::runtime_observation::Operation::DisconnectPause,
            {
                self.advance_transaction_revision();
                self.core
                    .auto_pause_active_battle_for_disconnect()
                    .map_err(|error| {
                        self.runtime_error(error, BattleFaultOperation::DisconnectPause, true)
                    })
            }
        )
    }

    /// Prepare trusted administrative edits through the same durable boundary.
    pub fn prepare_admin_command(
        &mut self,
        command: ApplicationAdminCommand,
    ) -> Result<PreparedApplication<'_, ApplicationResult>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        #[cfg(feature = "runtime-observation")]
        let observation = self.observer.as_mut().and_then(|observer| {
            observer.begin_with_identity(
                crate::runtime_observation::Operation::Admin,
                std::ptr::from_ref(&self.core) as usize,
                self.core.observation_identity(),
            )
        });
        self.advance_transaction_revision();
        let candidate = self
            .core
            .prepare_admin_command(command.command)
            .map_err(ApplicationError::from_domain);
        let prepared =
            candidate.and_then(|prepared| {
                self.prepare_publication(prepared.map_output(|output| {
                    ApplicationResult::admin(output.result_type, output.payload)
                }))
            });
        #[cfg(feature = "runtime-observation")]
        return match prepared {
            Ok(mut prepared) => {
                prepared.observation = observation.map(|guard| guard.prepared());
                Ok(prepared)
            }
            Err(error) => {
                if let Some(guard) = observation {
                    guard.finish_error(&error);
                }
                Err(error)
            }
        };
        #[cfg(not(feature = "runtime-observation"))]
        prepared
    }

    pub fn execute_admin(
        &mut self,
        command: ApplicationAdminCommand,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::Admin, {
            self.advance_transaction_revision();
            let output = self
                .core
                .execute_admin_command(command.command)
                .map_err(ApplicationError::from_domain)?;
            Ok(ApplicationResult::admin(output.result_type, output.payload))
        })
    }

    /// Retired V7 conversion is explicitly unsupported and leaves the original untouched.
    pub fn convert_active_run_v7(
        &self,
        original: &[u8],
        expected_original_identity: &crate::game::data::content_pack::ContentIdentity,
    ) -> Result<Vec<u8>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        self.core
            .convert_active_run_v7(original, expected_original_identity)
            .map_err(ApplicationError::from_domain)
    }

    /// Retired V8 conversion is explicitly unsupported and leaves the original untouched.
    pub fn convert_active_run_v8(
        &self,
        original: &[u8],
        expected_original_identity: &crate::game::data::content_pack::ContentIdentity,
    ) -> Result<Vec<u8>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        self.core
            .convert_active_run_v8(original, expected_original_identity)
            .map_err(ApplicationError::from_domain)
    }

    pub fn inspect_checkpoint_requirements(bytes: &[u8], max_bytes: usize) -> Result<crate::game::data::content_pack::ContentIdentity, ApplicationError> {
        GameCore::inspect_checkpoint_requirements(bytes, max_bytes).map_err(ApplicationError::from_domain)
    }

    pub fn export_active_run_checkpoint(
        &self,
    ) -> Result<Option<ApplicationCheckpoint>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        self.core
            .export_active_run_checkpoint()
            .map(|checkpoint| {
                checkpoint.map(|checkpoint| ApplicationCheckpoint {
                    core_payload_json: checkpoint.core_payload_json,
                    loads_used: checkpoint.loads_used,
                    run_peak_certified_power: checkpoint.run_peak_certified_power,
                })
            })
            .map_err(ApplicationError::from_domain)
    }

    pub fn continue_from_active_run_checkpoint(
        &mut self,
        player_id: Uuid,
        checkpoint: &ApplicationCheckpoint,
    ) -> Result<ApplicationResult, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(
            self,
            crate::runtime_observation::Operation::ContinueCheckpoint,
            {
                self.advance_transaction_revision();
                let result = self
                    .core
                    .continue_from_active_run_checkpoint(
                        player_id,
                        &checkpoint.core_payload_json,
                        checkpoint.loads_used,
                        checkpoint.run_peak_certified_power,
                    )
                    .map_err(ApplicationError::from_domain)?;
                ApplicationResult::from_behavior(&result)
            }
        )
    }

    pub fn merge_player_profile(
        &mut self,
        profile: ApplicationDocument,
    ) -> Result<(), ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::MergeProfile, {
            self.advance_transaction_revision();
            let profile: PlayerProfileState =
                serde_json::from_value(profile.0).map_err(|error| {
                    ApplicationError::invalid_input(
                        ApplicationErrorKind::InvalidProfile,
                        "invalid_player_profile",
                        &error,
                    )
                })?;
            self.core
                .merge_player_profile(profile)
                .map_err(ApplicationError::from_domain)
        })
    }

    /// Install the profile accepted by the host store, including exact vault ownership.
    pub fn install_authoritative_player_profile(&mut self, profile: ApplicationDocument) -> Result<(), ApplicationError> {
        self.ensure_battle_not_faulted()?;
        self.advance_transaction_revision();
        let profile: PlayerProfileState = serde_json::from_value(profile.0).map_err(|error| {
            ApplicationError::invalid_input(ApplicationErrorKind::InvalidProfile, "invalid_player_profile", &error)
        })?;
        self.core.install_authoritative_player_profile(profile).map_err(ApplicationError::from_domain)
    }

    /// Prepare exact host profile installation before a paired administrative acceptance.
    pub fn prepare_authoritative_profile_install(&mut self, profile: ApplicationDocument) -> Result<PreparedApplication<'_, ()>, ApplicationError> {
        self.ensure_battle_not_faulted()?;
        self.advance_transaction_revision();
        let profile: PlayerProfileState = serde_json::from_value(profile.0).map_err(|error| ApplicationError::invalid_input(ApplicationErrorKind::InvalidProfile, "invalid_player_profile", &error))?;
        let prepared = self.core.prepare_authoritative_profile_install(profile).map_err(ApplicationError::from_domain)?;
        self.prepare_publication(prepared)
    }

    pub fn merge_encounter_compendium_profile(
        &mut self,
        profile: ApplicationDocument,
    ) -> Result<(), ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(
            self,
            crate::runtime_observation::Operation::MergeCompendium,
            {
                self.advance_transaction_revision();
                let profile: EncounterCompendiumProfileState = serde_json::from_value(profile.0)
                    .map_err(|error| {
                        ApplicationError::invalid_input(
                            ApplicationErrorKind::InvalidProfile,
                            "invalid_encounter_compendium_profile",
                            &error,
                        )
                    })?;
                self.core
                    .merge_encounter_compendium_profile(profile)
                    .map_err(ApplicationError::from_domain)
            }
        )
    }

    pub fn reset_player_profile(&mut self) -> Result<(), ApplicationError> {
        self.ensure_battle_not_faulted()?;
        observed_operation!(self, crate::runtime_observation::Operation::ResetProfile, {
            self.advance_transaction_revision();
            self.core
                .reset_player_profile()
                .map_err(ApplicationError::from_domain)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::GameMode;
    use crate::game::{
        behavior::ActionDisabledReasonCode,
        data::skill_fragment_data::{SkillFragmentCompatibilityFailureCode, SkillFragmentId},
    };

    #[test]
    fn command_kind_is_exactly_bijective_with_all_domain_actions() {
        assert_eq!(ApplicationCommandKind::ALL.len(), 88);
        assert_eq!(ActionKind::ALL.len(), 88);
        for domain in ActionKind::ALL {
            let application = ApplicationCommandKind::from_domain(*domain);
            assert_eq!(application.into_domain(), *domain);
        }
    }

    #[test]
    fn command_document_rejects_unknown_and_mismatched_input_before_core_access() {
        let unknown = ApplicationCommand::from_document(ApplicationDocument(json!({
            "type": "not_a_core_command"
        })))
        .unwrap_err();
        assert_eq!(unknown.kind(), ApplicationErrorKind::InvalidCommand);

        let missing = ApplicationCommand::try_new(
            ApplicationCommandKind::StartNewGame,
            ApplicationDocument(json!({ "replace_active_run": false })),
        )
        .unwrap_err();
        assert_eq!(missing.kind(), ApplicationErrorKind::InvalidCommand);

        let valid = ApplicationCommand::try_new(
            ApplicationCommandKind::StartNewGame,
            ApplicationDocument(json!({
                "game_mode": GameMode::Tutorial,
                "replace_active_run": false,
            })),
        )
        .unwrap();
        assert_eq!(valid.kind(), ApplicationCommandKind::StartNewGame);
    }

    #[test]
    fn relic_corruption_command_port_preserves_selection_and_decline() {
        let node_id = Uuid::from_u128(0xCA08_0001);
        for copy_uuid in [Some(Uuid::from_u128(0xCA08_0002)), None] {
            let arguments = json!({
                "node_id": node_id,
                "event_id": "relic_event",
                "current_scene_id": "original_selection",
                "copy_uuid": copy_uuid,
            });
            let command = ApplicationCommand::try_new(
                ApplicationCommandKind::ChooseRelicCorruption,
                ApplicationDocument(arguments.clone()),
            )
            .unwrap();
            assert_eq!(
                command.kind(),
                ApplicationCommandKind::ChooseRelicCorruption
            );
            assert_eq!(command.behavior.kind(), ActionKind::ChooseRelicCorruption);
            let mut wire = arguments;
            wire["type"] = json!("choose_relic_corruption");
            assert_eq!(command.document.as_value(), &wire);
            assert_eq!(serde_json::to_value(command.behavior).unwrap(), wire);
            let decoded =
                ApplicationCommand::from_json_slice(&serde_json::to_vec(&wire).unwrap()).unwrap();
            assert_eq!(
                decoded.kind(),
                ApplicationCommandKind::ChooseRelicCorruption
            );
        }
    }

    #[test]
    fn empty_selected_event_is_an_explicit_null_read_model() {
        let data = crate::game::data::GameDataBuilder::empty().build_arc();
        let application = CoreApplication::new(data, 0xCA07_0001);
        let snapshot = application
            .snapshot(ApplicationSnapshotKind::SelectedEvent)
            .unwrap();
        assert_eq!(snapshot.read_model().as_value(), &Value::Null);
    }

    #[test]
    fn prepared_command_preserves_first_authoring_fault_and_original_root() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        let mut app = CoreApplication::new(
            crate::game::data::GameDataBuilder::empty().build_arc(),
            0xCA07_0008,
        );
        let before = app.snapshot(ApplicationSnapshotKind::Run).unwrap();
        for cancelled in [false, true] {
            let command = ApplicationCommand::try_new(
                ApplicationCommandKind::StartNewGame,
                ApplicationDocument(
                    json!({"game_mode": GameMode::Tutorial, "replace_active_run": false}),
                ),
            )
            .unwrap();
            let cancel = CancellationToken::default();
            let limits = ExecutionLimits {
                max_work: 1,
                ..ExecutionLimits::default()
            };
            let outcome = control::run(&limits, &cancel, ExecutionPhase::TrialExecution, || {
                if cancelled {
                    cancel.cancel();
                } else {
                    assert!(control::charge(2, "prepared_prior_failure").is_err());
                }
                assert!(app
                    .prepare_non_battle_command(
                        Uuid::from_u128(1),
                        command,
                        ApplicationExecutionContext::default(),
                    )
                    .is_err());
            })
            .unwrap_err();
            assert_eq!(
                outcome.code,
                if cancelled {
                    "authoring_cancelled"
                } else {
                    "authoring_work_limit"
                }
            );
            if !cancelled {
                assert_eq!(outcome.site, "prepared_prior_failure");
            }
            assert_eq!(app.snapshot(ApplicationSnapshotKind::Run).unwrap(), before);
        }
    }

    #[test]
    fn every_snapshot_port_is_typed_and_retry_idempotent() {
        let data = crate::game::data::GameDataBuilder::empty().build_arc();
        let application = CoreApplication::new(data, 0xCA07_0004);
        assert_eq!(ApplicationSnapshotKind::ALL.len(), 7);

        for kind in ApplicationSnapshotKind::ALL.iter().copied() {
            let first = application.snapshot(kind).unwrap();
            let retry = application.snapshot(kind).unwrap();
            assert_eq!(first.kind(), kind);
            assert_eq!(first, retry, "{kind:?}");
        }
    }

    #[test]
    fn public_projection_actual_ports_preserve_values_and_reject_control_failure() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        use std::cell::Cell;

        let data = crate::game::data::GameDataBuilder::empty().build_arc();
        let application = CoreApplication::new(data, 0xCA07_0005);
        let cancel = CancellationToken::default();
        for kind in ApplicationSnapshotKind::ALL.iter().copied() {
            let expected = application.snapshot(kind).unwrap();
            let (bounded, _) = control::run(
                &ExecutionLimits::default(),
                &cancel,
                ExecutionPhase::TrialExecution,
                || application.snapshot(kind),
            )
            .unwrap();
            assert_eq!(bounded.unwrap(), expected, "{kind:?}");
            // An absent selected event is a fixed Null and allocates no tree.
            if kind != ApplicationSnapshotKind::SelectedEvent {
                let limits = ExecutionLimits {
                    max_result_bytes: 1,
                    ..ExecutionLimits::default()
                };
                let failure =
                    control::run(&limits, &cancel, ExecutionPhase::TrialExecution, || {
                        application.snapshot(kind)
                    })
                    .unwrap_err();
                assert_eq!(failure.code, "authoring_result_limit", "{kind:?}");
                assert_eq!(failure.site, "public_projection_size", "{kind:?}");
                assert_eq!(failure.phase, ExecutionPhase::TrialExecution);
                assert_eq!(application.snapshot(kind).unwrap(), expected, "{kind:?}");
            }
        }

        let behavior = BehaviorResult::RunStartFlowStarted {
            game_mode: GameMode::Tutorial,
            visual_novel: None,
        };
        let expected = ApplicationResult::from_behavior(&behavior).unwrap();
        assert_eq!(
            expected.disposition(),
            ApplicationResultDisposition::Accepted
        );
        let (bounded, _) = control::run(
            &ExecutionLimits::default(),
            &cancel,
            ExecutionPhase::TrialExecution,
            || ApplicationResult::from_behavior(&behavior),
        )
        .unwrap();
        assert_eq!(bounded.unwrap(), expected);
        let observed = Cell::new(None);
        let limits = ExecutionLimits {
            max_result_bytes: 1,
            ..ExecutionLimits::default()
        };
        let failure = control::run(&limits, &cancel, ExecutionPhase::TrialExecution, || {
            let projected = ApplicationResult::from_behavior(&behavior);
            observed.set(Some(projected.as_ref().unwrap_err().kind()));
            projected
        })
        .unwrap_err();
        assert_eq!(failure.code, "authoring_result_limit");
        assert_eq!(failure.site, "public_projection_size");
        assert_eq!(failure.phase, ExecutionPhase::TrialExecution);
        assert_eq!(
            observed.get(),
            Some(ApplicationErrorKind::ProjectionFailure)
        );
        assert_eq!(
            ApplicationResult::from_behavior(&behavior).unwrap(),
            expected
        );
    }

    #[test]
    fn public_projection_diagnostics_admit_messages_and_skip_formatting_after_interruption() {
        use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
        use std::cell::Cell;

        struct Message<'a>(&'a Cell<usize>, &'a str);
        impl fmt::Display for Message<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                f.write_str(self.1)
            }
        }
        let calls = Cell::new(0);
        let text = "a diagnostic with a newline\nand Unicode 한글";
        let message = Message(&calls, text);
        let expected =
            ApplicationResult::projection_recovery_pending(ApplicationResultKind::Ok, &message)
                .unwrap();
        assert_eq!(
            expected.disposition(),
            ApplicationResultDisposition::AcceptedRecoveryPending
        );
        assert_eq!(
            expected.payload().as_value(),
            &json!({"projection_error": text})
        );
        let wire_bytes = serde_json::to_vec(expected.payload().as_value())
            .unwrap()
            .len();
        for (cap, site) in [
            (text.len() - 1, Some("diagnostic_message_size")),
            (wire_bytes - 1, Some("public_projection_size")),
            (wire_bytes, None),
        ] {
            calls.set(0);
            let limits = ExecutionLimits {
                max_result_bytes: cap,
                ..ExecutionLimits::default()
            };
            let result = control::run(
                &limits,
                &CancellationToken::default(),
                ExecutionPhase::TrialExecution,
                || {
                    ApplicationResult::projection_recovery_pending(
                        ApplicationResultKind::Ok,
                        &message,
                    )
                },
            );
            if let Some(site) = site {
                let error = result.unwrap_err();
                assert_eq!(error.code, "authoring_result_limit");
                assert_eq!(error.site, site);
            } else {
                assert_eq!(result.unwrap().0.unwrap(), expected);
            }
            assert_eq!(calls.get(), 1);
        }

        let expected = ApplicationError::from_domain(GameError::InvalidStaticData(text.into()));
        let (bounded, _) = control::run(
            &ExecutionLimits::default(),
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || ApplicationError::from_domain(GameError::InvalidStaticData(text.into())),
        )
        .unwrap();
        assert_eq!(bounded, expected);

        for cancelled in [false, true] {
            calls.set(0);
            let cancel = CancellationToken::default();
            let limits = ExecutionLimits {
                max_work: 1,
                ..ExecutionLimits::default()
            };
            let failure = control::run(&limits, &cancel, ExecutionPhase::TrialExecution, || {
                if cancelled {
                    cancel.cancel();
                } else {
                    assert!(control::charge(2, "diagnostic_test_prior_failure").is_err());
                }
                let recovery = ApplicationResult::projection_recovery_pending(
                    ApplicationResultKind::Ok,
                    &message,
                )
                .unwrap_err();
                let invalid = ApplicationError::invalid_input(
                    ApplicationErrorKind::InvalidCommand,
                    "invalid_application_command",
                    &message,
                );
                for error in [&recovery, &invalid] {
                    assert_eq!(error.kind(), ApplicationErrorKind::ProjectionFailure);
                    assert!(error.code().is_empty());
                    assert_eq!(error.details().as_value(), &Value::Null);
                }
            })
            .unwrap_err();
            assert_eq!(
                failure.code,
                if cancelled {
                    "authoring_cancelled"
                } else {
                    "authoring_work_limit"
                }
            );
            assert_eq!(failure.phase, ExecutionPhase::TrialExecution);
            assert_eq!(
                calls.get(),
                0,
                "the discarded error envelope never formats Display"
            );
        }
    }

    #[test]
    fn every_domain_error_has_one_typed_application_mapping() {
        let authority = BattleStateAuthorityDto {
            battle_uuid: Uuid::from_u128(0xCA07_0002),
            battle_state_revision: 7,
        };
        let cases = vec![
            (
                GameError::BattleRuntimeFault(
                    crate::game::battle::runtime_control::BattleRuntimeFault {
                        code: crate::game::battle::runtime_control::RuntimeFaultCode::WorkLimit,
                        site: "test_work",
                        limit: Some(1),
                        requested: Some(2),
                    },
                ),
                ApplicationErrorKind::BattleRuntimeFault,
            ),
            (
                GameError::EventNotFound,
                ApplicationErrorKind::EventNotFound,
            ),
            (
                GameError::EventTypeMismatch,
                ApplicationErrorKind::EventTypeMismatch,
            ),
            (
                GameError::InvalidAction,
                ApplicationErrorKind::InvalidAction,
            ),
            (
                GameError::TutorialCompletionRequired {
                    game_mode: GameMode::Expedition,
                },
                ApplicationErrorKind::TutorialCompletionRequired,
            ),
            (
                GameError::ActionUnavailable {
                    action: ActionKind::ConfirmEnterNode,
                    reason: ActionDisabledReasonCode::ContextIrrelevant,
                },
                ApplicationErrorKind::ActionUnavailable,
            ),
            (
                GameError::WeaponRequired,
                ApplicationErrorKind::WeaponRequired,
            ),
            (
                GameError::InvalidBattleResyncSeq {
                    requested: 8,
                    latest: 7,
                },
                ApplicationErrorKind::InvalidBattleResyncSeq,
            ),
            (
                GameError::BattleSnapshotResyncRequired {
                    requested: 1,
                    oldest_available_after_seq: 2,
                    latest: 7,
                },
                ApplicationErrorKind::BattleSnapshotResyncRequired,
            ),
            (
                GameError::BattleStateRevisionOverflow,
                ApplicationErrorKind::BattleStateRevisionOverflow,
            ),
            (
                GameError::MissingBattleStateAuthority { current: authority },
                ApplicationErrorKind::MissingBattleStateAuthority,
            ),
            (
                GameError::StaleBattleStateAuthority {
                    expected: authority,
                    current: authority,
                },
                ApplicationErrorKind::StaleBattleStateAuthority,
            ),
            (
                GameError::NotInShopState,
                ApplicationErrorKind::NotInShopState,
            ),
            (
                GameError::NotInRewardState,
                ApplicationErrorKind::NotInRewardState,
            ),
            (
                GameError::ShopItemNotFound,
                ApplicationErrorKind::ShopItemNotFound,
            ),
            (
                GameError::InventoryFull,
                ApplicationErrorKind::InventoryFull,
            ),
            (
                GameError::InventoryItemNotFound,
                ApplicationErrorKind::InventoryItemNotFound,
            ),
            (
                GameError::InventoryItemNotRemovable,
                ApplicationErrorKind::InventoryItemNotRemovable,
            ),
            (
                GameError::InsufficientResources,
                ApplicationErrorKind::InsufficientResources,
            ),
            (
                GameError::MissingResource("RunState"),
                ApplicationErrorKind::MissingResource,
            ),
            (
                GameError::InvalidUnitStats("fixture"),
                ApplicationErrorKind::InvalidUnitStats,
            ),
            (
                GameError::InvalidStaticData("fixture".to_string()),
                ApplicationErrorKind::InvalidStaticData,
            ),
            (
                GameError::NotImplemented("fixture"),
                ApplicationErrorKind::NotImplemented,
            ),
            (
                GameError::SkillFragmentIncompatible {
                    fragment_id: SkillFragmentId::from("fixture"),
                    failure_codes: vec![SkillFragmentCompatibilityFailureCode::WeaponRequired],
                },
                ApplicationErrorKind::SkillFragmentIncompatible,
            ),
            (GameError::OutOfBounds, ApplicationErrorKind::OutOfBounds),
            (
                GameError::PositionOccupied,
                ApplicationErrorKind::PositionOccupied,
            ),
            (
                GameError::StaticObstacleBlocked,
                ApplicationErrorKind::StaticObstacleBlocked,
            ),
            (
                GameError::UnitAlreadyPlaced,
                ApplicationErrorKind::UnitAlreadyPlaced,
            ),
            (GameError::UnitNotFound, ApplicationErrorKind::UnitNotFound),
        ];
        assert_eq!(cases.len(), ApplicationErrorKind::DOMAIN_ERRORS.len());
        for ((error, expected), listed) in cases
            .into_iter()
            .zip(ApplicationErrorKind::DOMAIN_ERRORS.iter().copied())
        {
            assert_eq!(expected, listed);
            let scoped_error = error.clone();
            let projected = ApplicationError::from_domain(error);
            assert_eq!(projected.kind(), expected);
            assert!(!projected.code().is_empty());
            let (bounded, _) = crate::authoring::control::run(
                &crate::authoring::ExecutionLimits::default(),
                &crate::authoring::CancellationToken::default(),
                crate::authoring::ExecutionPhase::TrialExecution,
                || ApplicationError::from_domain(scoped_error),
            )
            .unwrap();
            assert_eq!(bounded, projected, "{expected:?}");
        }
    }

    #[test]
    fn rejected_domain_command_and_retry_preserve_the_application_snapshot() {
        let data = crate::game::data::GameDataBuilder::empty().build_arc();
        let mut application = CoreApplication::new(data, 0xCA07_0003);
        let before = application.snapshot(ApplicationSnapshotKind::Run).unwrap();
        let command = || {
            ApplicationCommand::from_document(ApplicationDocument(json!({
                "type": "confirm_enter_node"
            })))
            .unwrap()
        };
        let first = application
            .execute(
                Uuid::from_u128(3),
                command(),
                ApplicationExecutionContext::default(),
            )
            .unwrap_err();
        let retry = application
            .execute(
                Uuid::from_u128(3),
                command(),
                ApplicationExecutionContext::default(),
            )
            .unwrap_err();
        assert_eq!(first, retry);
        assert_eq!(
            before,
            application.snapshot(ApplicationSnapshotKind::Run).unwrap()
        );
    }
}

#[cfg(test)]
mod persistence_rollback_tests {
    use super::*;

    fn app() -> CoreApplication {
        CoreApplication::new(crate::game::data::GameDataBuilder::empty().build_arc(), 27)
    }
    fn mutate(app: &mut CoreApplication, amount: u32) {
        let command = ApplicationAdminCommand::from_document(
            json!({
                "type": "admin_set_enkephalin", "amount": amount
            })
            .into(),
        )
        .unwrap();
        app.execute_admin(command).unwrap();
    }
    fn run(app: &CoreApplication) -> ApplicationSnapshot {
        app.snapshot(ApplicationSnapshotKind::Run).unwrap()
    }

    #[test]
    fn persistence_rollback_restores_one_attempt_and_rejects_foreign_stale_and_double_tokens() {
        let mut first = app();
        let mut second = app();
        let original = run(&first);
        let premature = first.capture_persistence_rollback().unwrap().unwrap();
        assert_eq!(
            first
                .restore_persistence_rollback(premature)
                .unwrap_err()
                .code(),
            "stale_persistence_rollback"
        );
        let foreign = first.capture_persistence_rollback().unwrap().unwrap();
        mutate(&mut second, 4);
        let second_before = run(&second);
        assert_eq!(
            second
                .restore_persistence_rollback(foreign)
                .unwrap_err()
                .code(),
            "stale_persistence_rollback"
        );
        assert_eq!(run(&second), second_before);
        let token = first.capture_persistence_rollback().unwrap().unwrap();
        let duplicate_capture = first.capture_persistence_rollback().unwrap().unwrap();
        mutate(&mut first, 42);
        assert_ne!(run(&first), original);
        first.restore_persistence_rollback(token).unwrap();
        assert_eq!(run(&first), original);
        assert_eq!(
            first
                .restore_persistence_rollback(duplicate_capture)
                .unwrap_err()
                .code(),
            "stale_persistence_rollback"
        );
        let stale = first.capture_persistence_rollback().unwrap().unwrap();
        mutate(&mut first, 20);
        mutate(&mut first, 30);
        let latest = run(&first);
        assert_eq!(
            first
                .restore_persistence_rollback(stale)
                .unwrap_err()
                .code(),
            "stale_persistence_rollback"
        );
        assert_eq!(run(&first), latest);
    }

    #[test]
    fn persistence_rollback_preserves_monotonic_profile_and_exhaustion_never_wraps() {
        let mut application = app();
        let token = application.capture_persistence_rollback().unwrap().unwrap();
        let profile = PlayerProfileState {
            highest_floor_reached: 99,
            ..Default::default()
        };
        application
            .merge_player_profile(serde_json::to_value(&profile).unwrap().into())
            .unwrap();
        application.restore_persistence_rollback(token).unwrap();
        assert_eq!(
            application
                .snapshot(ApplicationSnapshotKind::PlayerProfile)
                .unwrap()
                .read_model()
                .as_value(),
            &serde_json::to_value(profile).unwrap()
        );
        application.transaction_revision = u64::MAX - 2;
        let token = application.capture_persistence_rollback().unwrap().unwrap();
        mutate(&mut application, 5);
        application.restore_persistence_rollback(token).unwrap();
        assert_eq!(application.transaction_revision, u64::MAX);
        assert!(application.capture_persistence_rollback().is_err());
        mutate(&mut application, 6);
        assert_eq!(application.transaction_revision, u64::MAX);
        assert!(application.capture_persistence_rollback().is_err());
    }
}

#[cfg(all(test, feature = "runtime-observation"))]
mod runtime_observation_tests;

#[cfg(test)]
mod fault_tests;
