use crate::owned_collections::OwnedHashMap as HashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::game::battle::{
    core::{
        sim::{BattleDeployCurrentHpPolicy, BattleExecutionState, BattleLiveSignal},
        BattleCore, FieldWorkObjectStatus, TacticalTileEffect, UnitTacticalSnapshot,
    },
    event_log::BattleEventLogEntry,
    ids::UnitInstanceId,
    tile_range::FacingDirection,
    types::BattleUnitRole,
};
use crate::game::behavior::{
    AbnormalityAttemptDto, ActionDisabledReasonCode, BattlePayloadEligibilityDto,
    BattlePlaybackState, GameError, LiveBattleDeployedUnitDto, LiveBattleDeploymentDto,
    LiveBattleEventDeltaDto, LiveBattleFieldWorkCellStateDto,
    LiveBattleFieldWorkCompletionEffectDto, LiveBattleFieldWorkObjectStateDto,
    LiveBattleFieldWorkObjectStatusDto, LiveBattleFieldWorkPresentationKindDto,
    LiveBattleHudBarMode, LiveBattlePlaybackEligibilityDto, LiveBattlePresentationEventDto,
    LiveBattleRedeployUnitDto, LiveBattleSetupBattlefieldDto, LiveBattleSetupCatalogRefsDto,
    LiveBattleSetupFieldWorkCellDto, LiveBattleSetupFieldWorkObjectDto, LiveBattleSetupSnapshotDto,
    LiveBattleSetupSnapshotMessageType, LiveBattleSetupTacticalPointDto,
    LiveBattleSetupTacticalPointType, LiveBattleStanceEligibilityDto, LiveBattleStateCheckpointDto,
    LiveBattleTacticalResourceDto, LiveBattleTileEffectDto, LiveBattleUnitCheckpointDto,
    LiveBattleUnitControlStateDto, LiveBattleUnitDeployCostDto, LiveBattleUnitHudDto,
    LiveBattleUnitTacticalStateDto, LiveBattleUpdateDto, LiveBattleUpdateMessageType,
    RunCheckpointSnapshotDto,
};
use crate::game::combat_preview::{CombatMissionVariant, CombatNodeType, CombatPreview};
use crate::game::combat_setup::rewards::ResolvedCombatOutcomePolicy;
use crate::game::data::run_policy_data::{LiveBattleDeploymentPolicy, TacticalResourcePolicy};
use crate::game::employee::{EmployeeDefinition, EmployeeRoster};
use crate::game::encounter_compendium::PlayerProfileState;
use crate::game::encounter_progression::EncounterProgressionState;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::map::{
    FacilityRunState, GameMode, MapNodeId, MapProgression, NodeSession, RunMap, RunProgression,
};
use crate::game::recruitment::RecruitmentState;
use crate::game::resources::{
    ActiveNodeContent, Enkephalin, EventSessionState, GameState, Inventory, ShopSessionState,
};
use crate::game::skill_fragment::{SkillFragmentInventory, SkillFragmentPolicy};
use crate::game::visual_novel::{
    VisualNovelRuntimeState, VisualNovelTriggerConsumedKey, VisualNovelTriggerRuntimeState,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
}

impl PlayerInfo {
    pub fn new(id: Uuid, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}

crate::authoring::clone_shape::struct_shape!(PlayerInfo { id, name });

pub const ABNORMALITY_MAX_ATTEMPTS: u8 = 3;
pub const BATTLE_MAX_RETREATS: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbnormalityAttemptState {
    pub max_attempts: u8,
    pub attempts_started: u8,
    pub retreats_used: u8,
}

impl AbnormalityAttemptState {
    pub const fn new(max_attempts: u8) -> Self {
        Self {
            max_attempts,
            attempts_started: 0,
            retreats_used: 0,
        }
    }

    pub fn remaining_attempts(self) -> u8 {
        self.max_attempts.saturating_sub(self.attempts_started)
    }

    pub fn start_next_attempt(&mut self) -> bool {
        if self.retreats_used >= self.max_attempts {
            return false;
        }
        // Process-exit recovery may repeat the current challenge; only a
        // successful manual retreat advances the normal challenge ordinal.
        self.attempts_started = self.retreats_used + 1;
        true
    }

    pub fn can_retreat(self) -> bool {
        self.retreats_used < BATTLE_MAX_RETREATS
    }

    pub fn to_dto(self, node_id: crate::game::map::MapNodeId) -> AbnormalityAttemptDto {
        AbnormalityAttemptDto {
            node_id,
            max_attempts: self.max_attempts,
            attempts_started: self.attempts_started,
            remaining_attempts: self.remaining_attempts(),
            retreats_used: self.retreats_used,
            max_retreats: BATTLE_MAX_RETREATS,
            remaining_retreats: BATTLE_MAX_RETREATS.saturating_sub(self.retreats_used),
        }
    }
}

/// Private owner of the closed application lifecycle phase. Domain resources may be
/// assembled while a command is staged, but only a coherent context can cross the
/// application commit port.
#[derive(Debug, Clone)]
pub(super) struct LifecycleContext {
    phase: GameState,
}

crate::authoring::clone_shape::struct_shape!(LifecycleContext { phase });

impl LifecycleContext {
    fn not_started() -> Self {
        Self {
            phase: GameState::NotStarted,
        }
    }

    pub(super) fn phase(&self) -> &GameState {
        &self.phase
    }

    pub(super) fn snapshot(&self) -> GameState {
        self.phase.clone()
    }

    pub(super) fn transition_to(&mut self, phase: GameState) {
        self.phase = phase;
    }
}

/// Presentation of one already-settled battle. No live runtime or event history
/// is retained in the durable receipt, and acknowledging it cannot grant again.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfirmedCombatResult {
    pub battle_uuid: Uuid,
    pub summary: crate::game::behavior::SelectedEventSnapshotDto,
    pub acknowledgement: Arc<crate::game::behavior::BehaviorResult>,
    pub continuation_phase: GameState,
    /// Bounded live-process observation; it is never a Continue input.
    #[serde(skip)]
    pub runtime_facts: Option<Arc<crate::game::battle::runtime_facts::RuntimeFactSnapshot>>,
}

/// The ordinary reward session remains the candidate/selection owner. This
/// continuation retains only the original battle grant context and next phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CombatRewardContinuation {
    pub battle_uuid: Uuid,
    pub context: crate::game::reward::GrantExecutionContext,
    pub return_phase: GameState,
}

crate::authoring::clone_shape::struct_shape!(CombatRewardContinuation {
    battle_uuid,
    context,
    return_phase
});

#[derive(Clone)]
pub struct GameCoreState {
    pub player: Option<PlayerInfo>,
    pub lifecycle: LifecycleContext,
    pub active_node_content: Option<ActiveNodeContent>,
    pub pending_mastered_elite_sweep:
        Option<super::mastered_elite_sweep::MasteredEliteSweepCommitPlan>,
    pub node_session: Option<NodeSession>,
    pub run: Option<RunState>,
    pub visual_novel: VisualNovelRuntimeState,
    pub visual_novel_triggers: VisualNovelTriggerRuntimeState,
    pub uuid_manager: UuidManager,
    pub enkephalin: Enkephalin,
    pub inventory: Inventory,
    pub roster: EmployeeRoster,
    pub starter_candidates: Vec<EmployeeDefinition>,
    pub pending_game_mode: Option<GameMode>,
    pub skill_fragments: SkillFragmentInventory,
    pub relics: crate::game::relic::RelicInventory,
    pub crafting_resources: crate::game::relic::RunCraftingResources,
    pub skill_fragment_policy: SkillFragmentPolicy,
    pub player_profile: PlayerProfileState,
    pub active_battle: Option<ActiveBattleSession>,
    pub(crate) confirmed_combat_result: Option<Arc<ConfirmedCombatResult>>,
    pub(crate) combat_reward_continuation: Option<CombatRewardContinuation>,
    pub run_checkpoint: RunCheckpointState,
    pub(super) continue_state: super::active_run::ContinueState,
    pub run_power_authority: RunPowerAuthorityState,
}

// These are independent owned resource graphs in a World clone. The active
// battle store is separately admitted by BattleCore's existing preflight.
fn preflight_resource_fields(
    run: &Option<RunState>,
    checkpoint: &RunCheckpointState,
    content: &Option<ActiveNodeContent>,
) -> Result<(), GameError> {
    use crate::game::resource::store::{preflight_resource_clone, ResourceError};
    if let Some(run) = run {
        preflight_resource_clone(&run.typed_resources).map_err(ResourceError::into_game_error)?;
        crate::game::resources::selection::preflight_relic_event_histories(&run.event_sessions)?;
    }
    if let Some(checkpoint) = &checkpoint.payload {
        preflight_resource_clone(&checkpoint.typed_resources)
            .map_err(ResourceError::into_game_error)?;
        checkpoint.preflight_relic_history_clone()?;
    }
    if let Some(ActiveNodeContent::Event(session)) = content {
        if session.has_relic_history() {
            session.preflight_relic_clone()?;
        }
    }
    if let Some(ActiveNodeContent::CombatBattle(battle)) = content {
        if let Some(result) = &battle.typed_resource_result {
            result.preflight_native_clone()?;
        }
    }
    Ok(())
}

fn relic_extent_add(a: usize, b: usize) -> Result<usize, GameError> {
    a.checked_add(b)
        .ok_or_else(|| GameError::InvalidStaticData("relic retained extent overflow".into()))
}

impl GameCoreState {
    pub(super) fn has_relic_owner(&self) -> bool {
        !self.relics.is_empty()
            || !self.crafting_resources.is_empty()
            || self.uuid_manager.has_relic_stream()
            || self.run_checkpoint.payload.as_ref().is_some_and(|p| {
                !p.relics.is_empty()
                    || !p.crafting_resources.is_empty()
                    || p.uuid_manager.has_relic_stream()
                    || crate::game::resources::selection::has_relic_event_histories(
                        &p.event_sessions,
                    )
            })
            || self
                .active_battle
                .as_ref()
                .and_then(|a| a.entry_snapshot.as_ref())
                .is_some_and(|e| e.has_relic_owner())
            || self.continue_state.has_relic_owner()
            || self.run.as_ref().is_some_and(|run| {
                crate::game::resources::selection::has_relic_event_histories(&run.event_sessions)
            })
            || crate::game::resources::selection::has_active_relic_history(
                &self.active_node_content,
            )
    }
    pub(super) fn relic_owned_extent(&self) -> Result<usize, GameError> {
        let mut bytes = relic_extent_add(
            self.relics.retained_extent()?,
            self.uuid_manager.relic_stream_extent()?,
        )?;
        if let Some(run) = &self.run {
            bytes = relic_extent_add(
                bytes,
                crate::game::resources::selection::relic_event_histories_extent(
                    &run.event_sessions,
                )?,
            )?;
        }
        bytes = relic_extent_add(
            bytes,
            crate::game::resources::selection::active_relic_history_extent(
                &self.active_node_content,
            )?,
        )?;
        if let Some(p) = &self.run_checkpoint.payload {
            bytes = relic_extent_add(bytes, p.relic_owned_extent()?)?;
        }
        if let Some(e) = self
            .active_battle
            .as_ref()
            .and_then(|a| a.entry_snapshot.as_ref())
        {
            bytes = relic_extent_add(bytes, e.relic_owned_extent()?)?;
        }
        let bytes = relic_extent_add(bytes, self.continue_state.relic_owned_extent()?)?;
        #[cfg(feature = "runtime-observation")]
        crate::runtime_observation::relic_extent(bytes);
        Ok(bytes)
    }
    pub(super) fn preflight_resource_clone(&self, include_entry: bool) -> Result<(), GameError> {
        preflight_resource_fields(&self.run, &self.run_checkpoint, &self.active_node_content)?;
        if include_entry {
            if let Some(entry) = self
                .active_battle
                .as_ref()
                .and_then(|active| active.entry_snapshot.as_ref())
            {
                preflight_resource_fields(
                    &entry.run,
                    &entry.run_checkpoint,
                    &entry.active_node_content,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn clone_non_battle_transaction_state(&self) -> Result<Option<Self>, GameError> {
        if self.active_battle.is_some() {
            return Ok(None);
        }
        self.preflight_resource_clone(false)?;
        let candidate = crate::authoring::clone_shape::capture_shape!(Self {
            player: &self.player,
            lifecycle: &self.lifecycle,
            active_node_content: &self.active_node_content,
            pending_mastered_elite_sweep: &self.pending_mastered_elite_sweep,
            node_session: &self.node_session,
            run: &self.run,
            visual_novel: &self.visual_novel,
            visual_novel_triggers: &self.visual_novel_triggers,
            uuid_manager: &self.uuid_manager,
            enkephalin: &self.enkephalin,
            inventory: &self.inventory,
            roster: &self.roster,
            starter_candidates: &self.starter_candidates,
            pending_game_mode: &self.pending_game_mode,
            skill_fragments: &self.skill_fragments,
            relics: &self.relics,
            crafting_resources: &self.crafting_resources,
            skill_fragment_policy: &self.skill_fragment_policy,
            run_checkpoint: &self.run_checkpoint,
            continue_state: &self.continue_state,
            run_power_authority: &self.run_power_authority,
            confirmed_combat_result: &self.confirmed_combat_result,
            combat_reward_continuation: &self.combat_reward_continuation,
        } sets {} fresh {
            // Local profile knowledge is preserved by the rollback owner.
            player_profile: PlayerProfileState::default(), active_battle: None
        } fresh_sets {})?;
        Ok(Some(candidate))
    }

    /// Rejects a staged impossible lifecycle before it can become authoritative.
    /// The checks bind the closed phase to the identity-bearing optional context;
    /// domain-specific payload validation remains owned by the domain handlers.
    pub(super) fn validate_application_context(&self) -> Result<(), GameError> {
        let phase = self.lifecycle.phase();

        match phase {
            GameState::NotStarted => {
                if self.run.is_some()
                    || self.node_session.is_some()
                    || self.active_node_content.is_some()
                    || self.active_battle.is_some()
                    || self.pending_game_mode.is_some()
                    || self.pending_mastered_elite_sweep.is_some()
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::PreRunBriefing { game_mode } => {
                if self.run.is_some()
                    || self.node_session.is_some()
                    || self.active_node_content.is_some()
                    || self.active_battle.is_some()
                    || self.pending_game_mode != Some(*game_mode)
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::SelectingStarterEmployees => {
                if self.run.is_some()
                    || self.node_session.is_some()
                    || self.active_node_content.is_some()
                    || self.active_battle.is_some()
                    || self.pending_game_mode.is_none()
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::ViewingMap => {
                if self.run.is_none()
                    || self.node_session.is_some()
                    || self.active_node_content.is_some()
                    || self.active_battle.is_some()
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::NodeConfirm {
                node_id,
                kind_id,
                category,
            }
            | GameState::InNode {
                node_id,
                kind_id,
                category,
            } => {
                let Some(session) = self.node_session.as_ref() else {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                };
                if self.run.is_none()
                    || self.active_battle.is_some()
                    || session.node_id != *node_id
                    || session.kind_id != *kind_id
                    || session.category != *category
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::InBattle { battle_uuid } => {
                let Some(active) = self.active_battle.as_ref() else {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                };
                if active.battle_uuid != *battle_uuid || self.run.is_none() {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
                if let Some(deployment) = &active.live_deployment {
                    crate::authoring::control::charge(
                        deployment.deployed_units.len() as u64,
                        "application_deployed_context",
                    )?;
                    for deployed in deployment.deployed_units.values() {
                        if !active.battle.units.contains_key(&deployed.unit_instance_id) {
                            return Err(crate::game::battle::runtime_control::invariant(
                                "application_deployed_unit",
                            ));
                        }
                    }
                }
            }
            GameState::CombatResult { battle_uuid } => {
                if self.active_battle.is_some()
                    || self.confirmed_combat_result.as_ref()
                        .is_none_or(|receipt| receipt.battle_uuid != *battle_uuid)
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::MasteredEliteSweepResult { sweep_uuid } => {
                if self.active_battle.is_some()
                    || self
                        .pending_mastered_elite_sweep
                        .as_ref()
                        .is_none_or(|plan| plan.result.sweep_uuid != *sweep_uuid)
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::InShop { shop_uuid } => {
                if self.active_battle.is_some()
                    || self.active_node_content.as_ref().is_none_or(|content| {
                        !matches!(content, ActiveNodeContent::Shop(shop) if shop.uuid == *shop_uuid)
                    })
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::InReward { reward_uuid }
            | GameState::InRewardClaimed { reward_uuid } => {
                if self.active_battle.is_some()
                    || self.active_node_content.as_ref().is_none_or(|content| {
                        !matches!(content, ActiveNodeContent::Reward(reward) if reward.stage_uuid == *reward_uuid)
                    })
                {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
            GameState::AwaitingRunFailureDecision { .. }
            | GameState::GameOver
            | GameState::RunComplete
            | GameState::RunFailed { .. } => {
                if self.active_battle.is_some() {
                    return Err(crate::game::battle::runtime_control::invariant("application_context"));
                }
            }
        }
        Ok(())
    }
}

pub const RUN_CHECKPOINT_MAX_LOADS: u8 = 3;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPowerAuthorityState {
    pub power_model_version: u32,
    pub run_peak_certified_power: u64,
    pub current_floor_power_anchor: u64,
}

crate::authoring::clone_shape::copy_shape!(RunPowerAuthorityState, AbnormalityAttemptState);

impl RunPowerAuthorityState {
    pub fn initialize_run(&mut self, power_model_version: u32, certified_power: u64) {
        self.power_model_version = power_model_version;
        self.run_peak_certified_power = certified_power;
        self.current_floor_power_anchor = 0;
    }

    pub fn record_combat_entry(&mut self, power_model_version: u32, certified_power: u64) {
        if self.power_model_version == 0 {
            self.power_model_version = power_model_version;
        }
        debug_assert_eq!(self.power_model_version, power_model_version);
        self.run_peak_certified_power = self.run_peak_certified_power.max(certified_power);
    }

    pub fn manifest_floor(
        &mut self,
        policy: crate::game::data::run_policy_data::ResolvedFloorPowerAnchorPolicy,
    ) -> u64 {
        let previous_rise = self
            .current_floor_power_anchor
            .checked_add(policy.minimum_rise)
            .filter(|_| self.current_floor_power_anchor > 0)
            .unwrap_or(0);
        self.current_floor_power_anchor = policy
            .authored_minimum_power
            .max(previous_rise)
            .max(self.run_peak_certified_power);
        self.current_floor_power_anchor
    }

    pub fn merge_run_peak_monotonic(&mut self, other: Self) {
        if self.power_model_version == other.power_model_version {
            self.run_peak_certified_power = self
                .run_peak_certified_power
                .max(other.run_peak_certified_power);
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RunCheckpointState {
    pub payload: Option<RunCheckpointPayload>,
    pub loads_used: u8,
}

macro_rules! define_run_checkpoint_payload {
    ($( $run_field:ident: $run_field_type:ty ),+ $(,)?) => {
        #[derive(Clone, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct RunCheckpointPayload {
            pub content_identity: crate::game::data::content_pack::ContentIdentity,
            pub power_model_version: u32,
            pub current_floor_power_anchor: u64,
            $(pub $run_field: $run_field_type,)+
            pub visual_novel_trigger_checkpoint_consumed:
                std::collections::HashSet<VisualNovelTriggerConsumedKey>,
            pub enkephalin: Enkephalin,
            pub inventory: Inventory,
            pub roster: EmployeeRoster,
            pub skill_fragments: SkillFragmentInventory,
            #[serde(default, skip_serializing_if = "crate::game::relic::RelicInventory::is_empty")]
            pub relics: crate::game::relic::RelicInventory,
            #[serde(default, skip_serializing_if = "crate::game::relic::RunCraftingResources::is_empty")]
            pub crafting_resources: crate::game::relic::RunCraftingResources,
            pub uuid_manager: UuidManager,
        }

        // The checkpoint owns independent copies of every captured root; its
        // consumed set has the same closed fresh/insert-only provenance as VN.
        crate::authoring::clone_shape::struct_shape_with_sets!(RunCheckpointPayload {
            content_identity, power_model_version, current_floor_power_anchor,
            $($run_field,)+
            enkephalin, inventory, roster, skill_fragments, relics, crafting_resources, uuid_manager
        } sets { visual_novel_trigger_checkpoint_consumed });

        impl RunCheckpointPayload {
            pub(super) fn has_relic_owner(&self) -> bool {
                !self.relics.is_empty() || !self.crafting_resources.is_empty() || self.uuid_manager.has_relic_stream()
                    || crate::game::resources::selection::has_relic_event_histories(&self.event_sessions)
            }
            pub(super) fn relic_owned_extent(&self) -> Result<usize, GameError> {
                relic_extent_add(relic_extent_add(self.relics.retained_extent()?, self.uuid_manager.relic_stream_extent()?)?,
                    crate::game::resources::selection::relic_event_histories_extent(&self.event_sessions)?)
            }
            pub(super) fn preflight_relic_history_clone(&self) -> Result<(), GameError> {
                crate::game::resources::selection::preflight_relic_event_histories(&self.event_sessions)
            }
            pub(super) fn capture(
                state: &GameCoreState,
                run: &RunState,
                content_identity: &crate::game::data::content_pack::ContentIdentity,
            ) -> Result<Self, GameError> {
                crate::game::resource::store::preflight_resource_clone(&run.typed_resources)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                state.relics.preflight_clone()?;
                state.uuid_manager.preflight_relic_stream_clone()?;
                crate::game::resources::selection::preflight_relic_event_histories(&run.event_sessions)?;
                crate::authoring::clone_shape::capture_shape!(Self {
                    content_identity: content_identity,
                    power_model_version: &state.run_power_authority.power_model_version,
                    current_floor_power_anchor: &state.run_power_authority.current_floor_power_anchor,
                    $($run_field: &run.$run_field,)+
                    enkephalin: &state.enkephalin,
                    inventory: &state.inventory,
                    roster: &state.roster,
                    skill_fragments: &state.skill_fragments,
            relics: &state.relics,
            crafting_resources: &state.crafting_resources,
                    uuid_manager: &state.uuid_manager,
                } sets {
                    visual_novel_trigger_checkpoint_consumed:
                        &state.visual_novel_triggers.checkpoint_rewind_consumed
                } fresh {} fresh_sets {})
            }

            pub(super) fn restore_into_state(self, state: &mut GameCoreState) {
                let Self {
                    content_identity: _,
                    power_model_version,
                    current_floor_power_anchor,
                    $($run_field,)+
                    visual_novel_trigger_checkpoint_consumed,
                    enkephalin,
                    inventory,
                    roster,
                    skill_fragments,
    relics,
    crafting_resources,
                    uuid_manager,
                } = self;
                state.run_power_authority.power_model_version = power_model_version;
                state.run_power_authority.current_floor_power_anchor =
                    current_floor_power_anchor;
                state.run = Some(RunState {
                    $($run_field,)+
                    combat_recovery_restriction: None,
                });
                state.enkephalin = enkephalin;
                state.inventory = inventory;
                state.roster = roster;
                state.skill_fragments = skill_fragments;
                state.relics = relics;
                state.crafting_resources = crafting_resources;
                state.uuid_manager.restore_checkpoint_preserving_counters(
                    uuid_manager,
                    &[UuidManager::NS_VISUAL_NOVEL_SESSION],
                );
                state.visual_novel_triggers.checkpoint_rewind_consumed =
                    visual_novel_trigger_checkpoint_consumed;
                state.node_session = None;
                state.active_node_content = None;
                state.pending_mastered_elite_sweep = None;
                state.active_battle = None;
                state.confirmed_combat_result = None;
                state.combat_reward_continuation = None;
            }
        }
    };
}

define_run_checkpoint_payload! {
    typed_resources: crate::game::resource::durable::RunResourceState,
    map: RunMap,
    map_progression: MapProgression,
    run_progression: RunProgression,
    facility_run_state: FacilityRunState,
    encounter_progression: EncounterProgressionState,
    event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    shop_sessions: HashMap<crate::game::map::MapNodeId, ShopSessionState>,
    combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    recruitment: RecruitmentState,
    record_progress: RunRecordProgress,
}

impl RunCheckpointState {
    #[cfg(test)]
    pub fn can_load(&self) -> bool {
        self.payload.is_some() && self.loads_used < RUN_CHECKPOINT_MAX_LOADS
    }

    pub fn to_dto(&self, loadable: bool) -> RunCheckpointSnapshotDto {
        let remaining_loads = RUN_CHECKPOINT_MAX_LOADS.saturating_sub(self.loads_used);
        let location = self.payload.as_ref().and_then(|payload| {
            let node_id = payload.map_progression.current_node_id?;
            let node = payload.map.node(node_id)?;
            Some(crate::game::behavior::RunCheckpointLocationDto {
                floor_index: payload.run_progression.floor_index(),
                node_id,
                slot_id: node.slot_id.clone(),
                kind_id: node.kind_id.clone(),
                room_display_name: node.kind_id.as_str().to_string(),
            })
        });
        RunCheckpointSnapshotDto {
            exists: self.payload.is_some(),
            loadable,
            loads_used: self.loads_used,
            max_loads: RUN_CHECKPOINT_MAX_LOADS,
            remaining_loads,
            location,
        }
    }
}

#[derive(Clone)]
pub struct ActiveBattleSession {
    pub node_id: MapNodeId,
    pub battle_uuid: Uuid,
    pub battle_state_revision: u64,
    pub primary_abnormality_id: Option<String>,
    pub encounter_id: String,
    pub node_type: CombatNodeType,
    pub mission_variant: CombatMissionVariant,
    pub outcome_policies: Vec<ResolvedCombatOutcomePolicy>,
    pub combat_preview: CombatPreview,
    pub battle: BattleCore,
    pub execution: BattleExecutionState,
    pub entry_snapshot: Option<BattleEntrySnapshot>,
    pub employee_attempt: BattleEmployeeAttemptState,
    pub tactical_resource: TacticalResourceState,
    pub live_deployment: Option<LiveBattleDeploymentState>,
    pub playback: BattlePlaybackState,
    pub playback_delta_remainder: u64,
    pub tactical_focus: Option<SkillTargetingFocus>,
    pub interaction: BattleInteractionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillTargetingFocus {
    pub purpose: crate::game::behavior::LiveBattleTacticalFocusPurposeDto,
    pub caster_unit_instance_id: UnitInstanceId,
    pub skill_id: crate::game::ability::SkillId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BattleInteractionState {
    pub focused_unit_instance_id: Option<UnitInstanceId>,
    pub deployment_employee_uuid: Option<Uuid>,
    pub restore_playback: Option<BattlePlaybackState>,
    pub settings_restore_playback: Option<BattlePlaybackState>,
}

crate::authoring::clone_shape::copy_shape!(
    crate::game::behavior::LiveBattleTacticalFocusPurposeDto
);
crate::authoring::clone_shape::struct_shape!(SkillTargetingFocus {
    purpose,
    caster_unit_instance_id,
    skill_id
});
crate::authoring::clone_shape::struct_shape!(BattleInteractionState {
    focused_unit_instance_id,
    deployment_employee_uuid,
    restore_playback,
    settings_restore_playback
});

#[derive(Clone)]
pub struct BattleEntrySnapshot {
    game_state: GameState,
    active_node_content: Option<ActiveNodeContent>,
    node_session: Option<NodeSession>,
    run: Option<RunState>,
    visual_novel: VisualNovelRuntimeState,
    visual_novel_triggers: VisualNovelTriggerRuntimeState,
    uuid_manager: UuidManager,
    enkephalin: Enkephalin,
    inventory: Inventory,
    roster: EmployeeRoster,
    starter_candidates: Vec<EmployeeDefinition>,
    pending_game_mode: Option<GameMode>,
    skill_fragments: SkillFragmentInventory,
    relics: crate::game::relic::RelicInventory,
    crafting_resources: crate::game::relic::RunCraftingResources,
    run_checkpoint: RunCheckpointState,
}

impl ActiveBattleSession {
    pub(super) fn preflight_entry_resource_clone(&self) -> Result<(), GameError> {
        if let Some(entry) = &self.entry_snapshot {
            preflight_resource_fields(
                &entry.run,
                &entry.run_checkpoint,
                &entry.active_node_content,
            )?;
        }
        Ok(())
    }
}

impl BattleEntrySnapshot {
    fn has_relic_owner(&self) -> bool {
        !self.relics.is_empty()
            || !self.crafting_resources.is_empty()
            || self.uuid_manager.has_relic_stream()
            || self.run_checkpoint.payload.as_ref().is_some_and(|p| {
                !p.relics.is_empty()
                    || !p.crafting_resources.is_empty()
                    || p.uuid_manager.has_relic_stream()
                    || crate::game::resources::selection::has_relic_event_histories(
                        &p.event_sessions,
                    )
            })
            || self.run.as_ref().is_some_and(|run| {
                crate::game::resources::selection::has_relic_event_histories(&run.event_sessions)
            })
            || crate::game::resources::selection::has_active_relic_history(
                &self.active_node_content,
            )
    }
    fn relic_owned_extent(&self) -> Result<usize, GameError> {
        let mut bytes = relic_extent_add(
            self.relics.retained_extent()?,
            self.uuid_manager.relic_stream_extent()?,
        )?;
        if let Some(run) = &self.run {
            bytes = relic_extent_add(
                bytes,
                crate::game::resources::selection::relic_event_histories_extent(
                    &run.event_sessions,
                )?,
            )?;
        }
        bytes = relic_extent_add(
            bytes,
            crate::game::resources::selection::active_relic_history_extent(
                &self.active_node_content,
            )?,
        )?;
        if let Some(p) = &self.run_checkpoint.payload {
            bytes = relic_extent_add(bytes, p.relic_owned_extent()?)?;
        }
        Ok(bytes)
    }
    pub fn capture(state: &GameCoreState) -> Result<Self, GameError> {
        state.preflight_resource_clone(false)?;
        state.relics.preflight_clone()?;
        state.uuid_manager.preflight_relic_stream_clone()?;
        if let Some(p) = &state.run_checkpoint.payload {
            p.relics.preflight_clone()?;
            p.uuid_manager.preflight_relic_stream_clone()?;
        }
        crate::authoring::clone_shape::capture_shape!(Self {
            game_state: state.lifecycle.phase(),
            active_node_content: &state.active_node_content,
            node_session: &state.node_session,
            run: &state.run,
            visual_novel: &state.visual_novel,
            visual_novel_triggers: &state.visual_novel_triggers,
            uuid_manager: &state.uuid_manager,
            enkephalin: &state.enkephalin,
            inventory: &state.inventory,
            roster: &state.roster,
            starter_candidates: &state.starter_candidates,
            pending_game_mode: &state.pending_game_mode,
            skill_fragments: &state.skill_fragments,
            relics: &state.relics,
            crafting_resources: &state.crafting_resources,
            run_checkpoint: &state.run_checkpoint,
        } sets {} fresh {} fresh_sets {})
    }

    pub fn restore(self, state: &mut GameCoreState) {
        state.lifecycle.transition_to(self.game_state);
        state.active_node_content = self.active_node_content;
        state.node_session = self.node_session;
        state.run = self.run;
        state.visual_novel = self.visual_novel;
        state.visual_novel_triggers = self.visual_novel_triggers;
        state.uuid_manager = self.uuid_manager;
        state.enkephalin = self.enkephalin;
        state.inventory = self.inventory;
        state.roster = self.roster;
        state.starter_candidates = self.starter_candidates;
        state.pending_game_mode = self.pending_game_mode;
        state.skill_fragments = self.skill_fragments;
        state.relics = self.relics;
        state.crafting_resources = self.crafting_resources;
        state.run_checkpoint = self.run_checkpoint;
        state.active_battle = None;
    }

    pub fn as_event_combat_reentry(
        mut self,
        node_id: crate::game::map::MapNodeId,
        kind_id: crate::game::map::MapNodeKindId,
        category: crate::game::map::MapNodeCategory,
        session: NodeSession,
    ) -> Self {
        self.game_state = GameState::NodeConfirm {
            node_id,
            kind_id,
            category,
        };
        self.active_node_content = None;
        self.node_session = Some(session);
        self
    }
}

#[derive(Debug, Clone)]
pub struct BattleEmployeeAttemptState {
    pub roster: EmployeeRoster,
    pub processed_incapacitations: HashSet<UnitInstanceId>,
}

impl BattleEmployeeAttemptState {
    pub fn new(entry_roster: &EmployeeRoster) -> Result<Self, GameError> {
        crate::authoring::clone_shape::capture_shape!(Self {
            roster: entry_roster
        } sets {} fresh {} fresh_sets { processed_incapacitations: HashSet::new() })
    }
}

#[derive(Debug, Clone)]
pub struct LiveBattleDeploymentState {
    pub withdraw_redeploy_cooldown_ms: u64,
    pub defeat_redeploy_cooldown_ms: u64,
    pub next_instance_salt: u32,
    pub deployed_units: HashMap<Uuid, LiveBattleDeployedUnitState>,
    pub redeploy_locks: HashMap<Uuid, LiveBattleRedeployState>,
}

#[derive(Debug, Clone)]
pub struct LiveBattleRedeployState {
    pub ready_at_ms: u64,
    pub deploy_cost: u32,
    pub current_hp_policy: Option<BattleDeployCurrentHpPolicy>,
}

#[derive(Debug, Clone)]
pub struct LiveBattleDeployedUnitState {
    pub unit_instance_id: UnitInstanceId,
    pub facing: FacingDirection,
}

#[derive(Debug, Clone)]
pub struct TacticalResourceState {
    current: u32,
    maximum: u32,
    recovery_per_second: u32,
    last_recovery_at_ms: u64,
    base_deploy_cost: u32,
    redeploy_cost: u32,
}

impl TacticalResourceState {
    pub fn new(policy: TacticalResourcePolicy) -> Self {
        Self {
            current: policy.initial.min(policy.maximum),
            maximum: policy.maximum,
            recovery_per_second: policy.recovery_per_second,
            last_recovery_at_ms: 0,
            base_deploy_cost: policy.base_deploy_cost,
            redeploy_cost: policy.redeploy_cost(),
        }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn base_deploy_cost(&self) -> u32 {
        self.base_deploy_cost
    }

    pub fn redeploy_cost(&self) -> u32 {
        self.redeploy_cost
    }

    pub fn can_pay(&self, amount: u32) -> bool {
        self.current >= amount
    }

    pub fn pay(&mut self, amount: u32) -> bool {
        if !self.can_pay(amount) {
            return false;
        }
        self.current -= amount;
        true
    }

    pub fn recover_until(&mut self, now_ms: u64) {
        if now_ms <= self.last_recovery_at_ms {
            return;
        }
        if self.recovery_per_second == 0 {
            self.last_recovery_at_ms = now_ms;
            return;
        }
        let elapsed_seconds = (now_ms - self.last_recovery_at_ms) / 1_000;
        if elapsed_seconds == 0 {
            return;
        }
        self.last_recovery_at_ms = self
            .last_recovery_at_ms
            .saturating_add(elapsed_seconds.saturating_mul(1_000));
        let recovered = elapsed_seconds.saturating_mul(u64::from(self.recovery_per_second));
        let recovered = u32::try_from(recovered).unwrap_or(u32::MAX);
        self.current = self.maximum.min(self.current.saturating_add(recovered));
    }

    pub fn apply_delta(&mut self, amount: i32) {
        if amount > 0 {
            self.current = self.maximum.min(self.current.saturating_add(amount as u32));
        } else if amount < 0 {
            self.current = self.current.saturating_sub(amount.unsigned_abs());
        }
    }

    pub fn to_dto(&self) -> LiveBattleTacticalResourceDto {
        LiveBattleTacticalResourceDto {
            current: self.current,
            maximum: self.maximum,
            recovery_per_second: self.recovery_per_second,
        }
    }

    #[cfg(test)]
    pub fn set_current_for_test(&mut self, current: u32) {
        self.current = current.min(self.maximum);
    }
}

impl LiveBattleDeploymentState {
    pub fn new(policy: LiveBattleDeploymentPolicy) -> Self {
        Self {
            withdraw_redeploy_cooldown_ms: policy.withdraw_redeploy_cooldown_ms,
            defeat_redeploy_cooldown_ms: policy.defeat_redeploy_cooldown_ms,
            next_instance_salt: policy.first_instance_salt,
            deployed_units: HashMap::new(),
            redeploy_locks: HashMap::new(),
        }
    }
}

impl ActiveBattleSession {
    pub fn employee_roster(&self) -> &EmployeeRoster {
        &self.employee_attempt.roster
    }

    pub fn unprocessed_deployed_employee_incapacitations(&self) -> Vec<(Uuid, UnitInstanceId)> {
        let Some(deployment) = self.live_deployment.as_ref() else {
            return Vec::new();
        };
        let mut incapacitations = deployment
            .deployed_units
            .iter()
            .filter_map(|(employee_uuid, deployed)| {
                let unit = self.battle.units.get(&deployed.unit_instance_id)?;
                (unit.lifecycle.is_dead()
                    && unit.source_owned_uuid == *employee_uuid
                    && matches!(
                        unit.source_identity,
                        crate::game::battle::types::BattleUnitSourceIdentity::Employee { .. }
                    )
                    && !self
                        .employee_attempt
                        .processed_incapacitations
                        .contains(&deployed.unit_instance_id))
                .then_some((*employee_uuid, deployed.unit_instance_id))
            })
            .collect::<Vec<_>>();
        incapacitations.sort_by(|left, right| left.1.cmp(&right.1));
        incapacitations
    }

    pub fn playback_state(&self) -> BattlePlaybackState {
        self.playback
    }

    pub fn deployment_candidate_eligibility(
        &self,
        employee_uuid: Uuid,
        effective_deploy_cost: u32,
        employee_disabled_reason: Option<ActionDisabledReasonCode>,
    ) -> BattlePayloadEligibilityDto {
        if let Some(reason) = employee_disabled_reason {
            return BattlePayloadEligibilityDto::disabled(reason);
        }
        let Some(deployment) = self.live_deployment.as_ref() else {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::UnitUnavailable,
            );
        };
        if deployment.deployed_units.contains_key(&employee_uuid) {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::UnitUnavailable,
            );
        }
        if deployment
            .redeploy_locks
            .get(&employee_uuid)
            .is_some_and(|lock| self.execution.last_event_time_ms() < lock.ready_at_ms)
        {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::RedeployCooldown,
            );
        }
        if deployment.deployed_units.len() >= 8 {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::DeploymentLimit,
            );
        }
        if !self.tactical_resource.can_pay(effective_deploy_cost) {
            return BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::InsufficientResources,
            );
        }
        BattlePayloadEligibilityDto::enabled()
    }

    fn playback_eligibility_dto(&self) -> LiveBattlePlaybackEligibilityDto {
        let mut result = LiveBattlePlaybackEligibilityDto::for_state(self.playback);
        result.forced_pause_remaining_ms = self.battle.forced_pause_remaining_ms();
        if result.forced_pause_remaining_ms.is_some() {
            result.pause = BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::CorrosionForcedPause,
            );
            result.resume = BattlePayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::CorrosionForcedPause,
            );
            for speed in &mut result.speeds {
                speed.eligibility = BattlePayloadEligibilityDto::disabled(
                    ActionDisabledReasonCode::CorrosionForcedPause,
                );
            }
        }
        if result.forced_pause_remaining_ms.is_none() {
            let speed_reason = if self.settings_open() {
                Some(ActionDisabledReasonCode::SettingsOpen)
            } else if self.has_selection() {
                Some(ActionDisabledReasonCode::SelectionActive)
            } else {
                None
            };
            if let Some(reason) = speed_reason {
                for speed in &mut result.speeds {
                    speed.eligibility = BattlePayloadEligibilityDto::disabled(reason);
                }
            }
            if self.settings_open() {
                result.pause =
                    BattlePayloadEligibilityDto::disabled(ActionDisabledReasonCode::SettingsOpen);
                result.resume =
                    BattlePayloadEligibilityDto::disabled(ActionDisabledReasonCode::SettingsOpen);
            }
        }
        result
    }

    pub fn simulation_delta_for_tick(&mut self, raw_delta_ms: u64) -> Option<u64> {
        if self.playback.paused {
            return None;
        }
        let (numerator, denominator) = self.playback.speed.ratio();
        let scaled = raw_delta_ms
            .saturating_mul(numerator)
            .saturating_add(self.playback_delta_remainder);
        let delta_ms = scaled / denominator;
        self.playback_delta_remainder = scaled % denominator;
        if delta_ms == 0 {
            return None;
        }
        Some(delta_ms)
    }

    pub fn refresh_tactical_resource(&mut self) {
        let now_ms = self.execution.last_event_time_ms();
        self.tactical_resource.recover_until(now_ms);
    }

    pub fn apply_live_signals_to_tactical_resource(&mut self) {
        let signals = self.battle.drain_live_signals();
        for signal in signals {
            match signal {
                BattleLiveSignal::TacticalResourceDelta {
                    amount,
                    fact_source,
                    time_ms,
                    ..
                } => {
                    let before = self.tactical_resource.current();
                    self.tactical_resource.apply_delta(amount);
                    if self
                        .battle
                        .record_tactical_resource_signal(
                            fact_source,
                            amount,
                            before,
                            self.tactical_resource.current(),
                            time_ms,
                        )
                        .is_err()
                    {
                        return;
                    }
                }
            }
        }
    }

    pub fn reconcile_live_deployment_with_battle_state(&mut self) {
        let Some(deployment) = self.live_deployment.as_mut() else {
            return;
        };
        let now_ms = self.execution.last_event_time_ms();
        let redeploy_cost = self.tactical_resource.redeploy_cost();
        let stale_deployments = deployment
            .deployed_units
            .iter()
            .filter_map(|(employee_uuid, deployed)| {
                let current_hp_policy = match self.battle.units.get(&deployed.unit_instance_id) {
                    Some(unit) if unit.is_active() => return None,
                    Some(unit)
                        if matches!(
                            unit.lifecycle,
                            crate::game::battle::core::types::RuntimeUnitLifecycle::Withdrawn
                        ) =>
                    {
                        Some(BattleDeployCurrentHpPolicy::withdraw_redeploy(
                            unit.stats.current_health,
                            unit.stats.max_health,
                        ))
                    }
                    Some(_) | None => None,
                };
                Some((*employee_uuid, current_hp_policy))
            })
            .collect::<Vec<_>>();

        for (employee_uuid, current_hp_policy) in stale_deployments {
            deployment.deployed_units.remove(&employee_uuid);
            deployment
                .redeploy_locks
                .entry(employee_uuid)
                .or_insert_with(|| LiveBattleRedeployState {
                    ready_at_ms: now_ms.saturating_add(deployment.defeat_redeploy_cooldown_ms),
                    deploy_cost: redeploy_cost,
                    current_hp_policy,
                });
        }
    }

    pub fn battle_setup_snapshot_dto(&self) -> LiveBattleSetupSnapshotDto {
        let scenario = self.battle.scenario();
        let mut tactical_points = scenario
            .tactical_plan
            .points
            .iter()
            .map(|point| LiveBattleSetupTacticalPointDto {
                id: point.id.0.clone(),
                point_type: LiveBattleSetupTacticalPointType::TacticalPoint,
                position: point.position,
            })
            .collect::<Vec<_>>();

        for route in &self.combat_preview.routes {
            let endpoint_id = format!("{}_endpoint", route.id);
            if tactical_points.iter().any(|point| point.id == endpoint_id) {
                continue;
            }
            tactical_points.push(LiveBattleSetupTacticalPointDto {
                id: endpoint_id,
                point_type: LiveBattleSetupTacticalPointType::RouteEndpoint,
                position: route.end,
            });
        }
        tactical_points.sort_by(|left, right| left.id.cmp(&right.id));

        let is_actual_abnormality_catalog_id = |id: &str| {
            !id.trim().is_empty()
                && self
                    .battle
                    .game_data
                    .abnormality_data
                    .get_by_id(id)
                    .is_some()
        };
        let mut abnormality_ids = self
            .primary_abnormality_id
            .iter()
            .filter(|id| is_actual_abnormality_catalog_id(id))
            .cloned()
            .collect::<Vec<_>>();
        abnormality_ids.extend(
            self.combat_preview
                .enemy_briefing
                .iter()
                .filter(|briefing| {
                    briefing.kind == crate::game::combat_preview::EnemyKind::Abnormality
                        && is_actual_abnormality_catalog_id(&briefing.abnormality_id)
                })
                .map(|briefing| briefing.abnormality_id.clone()),
        );
        abnormality_ids.extend(
            self.combat_preview
                .spawn_waves
                .iter()
                .flat_map(|wave| wave.enemy_entries.iter())
                .filter(|entry| {
                    entry.kind == crate::game::combat_preview::EnemyKind::Abnormality
                        && is_actual_abnormality_catalog_id(&entry.abnormality_id)
                })
                .map(|entry| entry.abnormality_id.clone()),
        );
        abnormality_ids.sort();
        abnormality_ids.dedup();

        LiveBattleSetupSnapshotDto {
            message_type: LiveBattleSetupSnapshotMessageType::BattleSetupSnapshot,
            setup_version: LiveBattleSetupSnapshotDto::SETUP_VERSION,
            battle_uuid: self.battle_uuid,
            encounter_id: self.encounter_id.clone(),
            node_type: self.node_type,
            mission_variant: self.mission_variant,
            survive_timer_ms: self.combat_preview.survive_timer_ms,
            battlefield: LiveBattleSetupBattlefieldDto {
                width: i32::from(scenario.battlefield.width),
                height: i32::from(scenario.battlefield.height),
                tiles: self.combat_preview.tiles.clone(),
                valid_tiles: scenario.battlefield.valid_tiles.clone(),
                blocked_tiles: scenario.battlefield.obstacles.clone(),
                static_obstacles: scenario.battlefield.obstacles.clone(),
            },
            routes: self.combat_preview.routes.clone(),
            deployment_zones: self.combat_preview.deployment_zones.clone(),
            spawn_zones: self.combat_preview.spawn_zones.clone(),
            tactical_points,
            static_objects: Vec::new(),
            field_work_objects: {
                let mut objects = scenario
                    .tactical_plan
                    .field_work_objects
                    .iter()
                    .map(|object| {
                        let mut work_cells = object
                            .work_cells
                            .iter()
                            .map(|cell| LiveBattleSetupFieldWorkCellDto {
                                id: cell.id.0.clone(),
                                position: cell.position,
                                tile_kind: cell.tile_kind,
                            })
                            .collect::<Vec<_>>();
                        work_cells.sort_by(|left, right| left.id.cmp(&right.id));
                        LiveBattleSetupFieldWorkObjectDto {
                            id: object.id.0.clone(),
                            presentation_kind: match object.presentation_kind {
                                crate::game::battle::scenario::FieldWorkPresentationKind::EmergencyRepairTerminal => {
                                    LiveBattleFieldWorkPresentationKindDto::EmergencyRepairTerminal
                                }
                            },
                            position: object.position,
                            work_cells,
                            required_work_ms: object.required_work_ms,
                            completion_effect: match &object.completion_effect {
                                crate::game::battle::scenario::FieldWorkCompletionEffect::RepairProtectedUnit {
                                    unit_ref,
                                    flat_hp,
                                } => LiveBattleFieldWorkCompletionEffectDto::RepairProtectedUnit {
                                    unit_ref: unit_ref.0.clone(),
                                    flat_hp: *flat_hp,
                                },
                            },
                        }
                    })
                    .collect::<Vec<_>>();
                objects.sort_by(|left, right| left.id.cmp(&right.id));
                objects
            },
            initial_units: Vec::new(),
            catalog_refs: LiveBattleSetupCatalogRefsDto {
                battlefield_template_id: self.combat_preview.battlefield_template_id.clone(),
                abnormality_ids,
            },
        }
    }

    pub fn live_deployment_dto(&self, roster: &EmployeeRoster) -> Option<LiveBattleDeploymentDto> {
        let deployment = self.live_deployment.as_ref()?;
        let mut deployed_units = deployment
            .deployed_units
            .iter()
            .map(|(employee_uuid, deployed)| {
                let position = self
                    .battle
                    .unit_projected_tile(deployed.unit_instance_id)
                    .unwrap_or_else(|| {
                        panic!(
                            "live deployment references unit without runtime position: {:?}",
                            deployed.unit_instance_id
                        )
                    });
                LiveBattleDeployedUnitDto {
                    employee_uuid: *employee_uuid,
                    unit_instance_id: deployed.unit_instance_id,
                    position,
                    facing: deployed.facing,
                    withdraw_eligibility: BattlePayloadEligibilityDto::enabled(),
                    tactical_state: self
                        .battle
                        .unit_tactical_snapshot(
                            deployed.unit_instance_id,
                            self.execution.last_event_time_ms(),
                        )
                        .map(|snapshot| {
                            live_unit_tactical_state_dto(
                                snapshot,
                                self.tactical_resource.can_pay(
                                    self.battle.game_data.run_policy.tactical_relocation.cost,
                                ),
                            )
                        })
                        .unwrap_or_else(|| {
                            panic!(
                                "live deployment references unit without tactical state: {:?}",
                                deployed.unit_instance_id
                            )
                        }),
                    skill_readiness: self
                        .battle
                        .live_skill_readiness(
                            self.execution.last_event_time_ms(),
                            deployed.unit_instance_id,
                        )
                        .unwrap_or_else(|| {
                            panic!(
                                "live deployment references unit without skill readiness: {:?}",
                                deployed.unit_instance_id
                            )
                        }),
                }
            })
            .collect::<Vec<_>>();
        deployed_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut redeploying_units = deployment
            .redeploy_locks
            .iter()
            .filter(|(employee_uuid, _)| roster.contains(employee_uuid))
            .map(|(employee_uuid, state)| LiveBattleRedeployUnitDto {
                employee_uuid: *employee_uuid,
                ready_at_ms: state.ready_at_ms,
                deploy_cost: state.deploy_cost,
            })
            .collect::<Vec<_>>();
        redeploying_units.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        let mut unit_deploy_costs = roster
            .iter()
            .filter(|employee| !deployment.deployed_units.contains_key(&employee.uuid))
            .map(|employee| {
                let base_deploy_cost = deployment
                    .redeploy_locks
                    .get(&employee.uuid)
                    .map(|lock| lock.deploy_cost)
                    .unwrap_or_else(|| self.tactical_resource.base_deploy_cost());
                let effective_deploy_cost =
                    employee.deployment_cost_after_consumable(base_deploy_cost);
                LiveBattleUnitDeployCostDto {
                    employee_uuid: employee.uuid,
                    base_deploy_cost,
                    effective_deploy_cost,
                    eligibility: self.deployment_candidate_eligibility(
                        employee.uuid,
                        effective_deploy_cost,
                        employee.battle_deployment_disabled_reason(),
                    ),
                }
            })
            .collect::<Vec<_>>();
        unit_deploy_costs.sort_by(|left, right| left.employee_uuid.cmp(&right.employee_uuid));

        Some(LiveBattleDeploymentDto {
            battle_time_ms: self.execution.last_event_time_ms(),
            unit_deploy_costs,
            deployed_units,
            redeploying_units,
        })
    }

    pub fn commit_battle_projection(&mut self) -> Result<u64, GameError> {
        self.battle_state_revision = self.next_battle_projection_revision()?;
        Ok(self.battle_state_revision)
    }

    pub fn ensure_battle_projection_commit_available(&self) -> Result<(), GameError> {
        self.next_battle_projection_revision().map(|_| ())
    }

    fn next_battle_projection_revision(&self) -> Result<u64, GameError> {
        self.battle_state_revision
            .checked_add(1)
            .ok_or(GameError::BattleStateRevisionOverflow)
    }

    #[cfg(test)]
    pub fn latest_event_log_seq(&self) -> u64 {
        self.battle.event_log.latest_seq()
    }

    pub fn battle_update_dto_after(
        &self,
        after_seq: u64,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleUpdateDto, GameError> {
        self.ensure_battle_update_cursor_valid(after_seq)?;
        let events = self.battle.event_log_entries_after_seq(after_seq);
        let to_seq = events.last().map(|entry| entry.seq).unwrap_or(after_seq);
        self.battle_update_dto_from_events(
            LiveBattleUpdateMessageType::BattleUpdate,
            after_seq,
            to_seq,
            events,
            roster,
        )
    }

    pub fn battle_update_dto_after_or_resync(
        &self,
        after_seq: u64,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleUpdateDto, GameError> {
        let (message_type, effective_after_seq) = match self
            .battle
            .event_log
            .validate_cursor(after_seq)
        {
            Ok(()) => (LiveBattleUpdateMessageType::BattleUpdate, after_seq),
            Err(crate::game::battle::event_log::BattleHistoryCursorError::InvalidFuture {
                requested,
                latest,
            }) => {
                return Err(GameError::InvalidBattleResyncSeq { requested, latest });
            }
            Err(
                crate::game::battle::event_log::BattleHistoryCursorError::SnapshotResyncRequired {
                    oldest_available_after_seq,
                    ..
                },
            ) => (
                LiveBattleUpdateMessageType::SnapshotResyncRequired,
                oldest_available_after_seq,
            ),
        };
        let events = self.battle.event_log_entries_after_seq(effective_after_seq);
        let to_seq = events
            .last()
            .map(|entry| entry.seq)
            .unwrap_or(effective_after_seq);
        self.battle_update_dto_from_events(
            message_type,
            effective_after_seq,
            to_seq,
            events,
            roster,
        )
    }

    pub fn ensure_battle_update_cursor_valid(&self, after_seq: u64) -> Result<(), GameError> {
        match self.battle.event_log.validate_cursor(after_seq) {
            Ok(()) => Ok(()),
            Err(crate::game::battle::event_log::BattleHistoryCursorError::InvalidFuture {
                requested: after_seq,
                latest,
            }) => Err(GameError::InvalidBattleResyncSeq {
                requested: after_seq,
                latest,
            }),
            Err(
                crate::game::battle::event_log::BattleHistoryCursorError::SnapshotResyncRequired {
                    requested,
                    oldest_available_after_seq,
                    latest,
                },
            ) => Err(GameError::BattleSnapshotResyncRequired {
                requested,
                oldest_available_after_seq,
                latest,
            }),
        }
    }

    fn battle_update_dto_from_events(
        &self,
        message_type: LiveBattleUpdateMessageType,
        after_seq: u64,
        to_seq: u64,
        events: Vec<BattleEventLogEntry>,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleUpdateDto, GameError> {
        Ok(LiveBattleUpdateDto {
            message_type,
            battle_uuid: self.battle_uuid,
            battle_state_revision: self.battle_state_revision,
            action_capabilities: Vec::new(),
            server_battle_time_ms: self.execution.last_event_time_ms(),
            events_delta: LiveBattleEventDeltaDto {
                after_seq,
                to_seq,
                events: events
                    .into_iter()
                    .map(LiveBattlePresentationEventDto::from)
                    .collect(),
            },
            checkpoint: self.live_checkpoint_dto(to_seq, roster)?,
            visual_novel: None,
        })
    }

    fn live_checkpoint_dto(
        &self,
        at_seq: u64,
        roster: &EmployeeRoster,
    ) -> Result<LiveBattleStateCheckpointDto, GameError> {
        let battle_time_ms = self.execution.last_event_time_ms();
        let mut units = self
            .battle
            .units
            .values()
            .filter(|unit| unit.is_active())
            .map(|unit| -> Result<LiveBattleUnitCheckpointDto, GameError> {
                let position = unit.body.projected_tile();
                let control_state = self
                    .battle
                    .unit_control_state(unit.instance_id, battle_time_ms)
                    .unwrap_or_default();
                let skill_start_disabled_reason = self
                    .battle
                    .skill_start_disabled_reason(unit.instance_id, battle_time_ms);
                Ok(LiveBattleUnitCheckpointDto {
                    basic_action: self.battle.basic_action_snapshot(
                        unit.instance_id,
                        self.interaction.focused_unit_instance_id == Some(unit.instance_id)
                            || self.tactical_focus.as_ref().is_some_and(|focus| {
                                focus.caster_unit_instance_id == unit.instance_id
                            }),
                    )?,
                    unit_instance_id: unit.instance_id,
                    owner: unit.owner,
                    role: unit.role,
                    hud: live_unit_hud_dto(unit),
                    control: LiveBattleUnitControlStateDto {
                        hard_cc_active: control_state.hard_cc_active(),
                        hard_cc_release_time_ms: control_state.hard_cc_release_time_ms,
                        silenced: control_state.silenced,
                        skill_start_disabled_reason,
                    },
                    tactical_state: self
                        .battle
                        .unit_tactical_snapshot(unit.instance_id, battle_time_ms)
                        .map(|snapshot| {
                            live_unit_tactical_state_dto(
                                snapshot,
                                self.tactical_resource.can_pay(
                                    self.battle.game_data.run_policy.tactical_relocation.cost,
                                ),
                            )
                        }),
                    range_previews: if unit.owner == crate::game::enums::Side::Player
                        || self.interaction.focused_unit_instance_id == Some(unit.instance_id)
                    {
                        Some(self.battle.runtime_range_previews(
                            unit.instance_id,
                            position,
                            unit.facing_direction,
                        )?)
                    } else {
                        None
                    },
                    mobility_kind: unit.mobility_kind,
                    unit_source: unit.source_identity.clone(),
                    position,
                    world_position: unit.world_position(),
                    stats: unit.stats,
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        units.sort_by(|left, right| left.unit_instance_id.cmp(&right.unit_instance_id));

        Ok(LiveBattleStateCheckpointDto {
            sustained_effects: self.battle.sustained_effect_snapshot()?,
            independent_objects: self.battle.independent_objects_snapshot(battle_time_ms)?,
            at_seq,
            battle_time_ms,
            playback: self.playback_state(),
            playback_eligibility: self.playback_eligibility_dto(),
            tactical_focus: self.tactical_focus_dto(),
            interaction: self.interaction_dto(),
            field_work_objects: self
                .battle
                .field_work_object_snapshots()
                .into_iter()
                .map(|object| LiveBattleFieldWorkObjectStateDto {
                    id: object.id.0,
                    status: match object.status {
                        FieldWorkObjectStatus::Active => LiveBattleFieldWorkObjectStatusDto::Active,
                        FieldWorkObjectStatus::Completed => {
                            LiveBattleFieldWorkObjectStatusDto::Completed
                        }
                    },
                    accumulated_work_ms: object.accumulated_work_ms,
                    required_work_ms: object.required_work_ms,
                    work_cells: object
                        .work_cells
                        .into_iter()
                        .map(|cell| LiveBattleFieldWorkCellStateDto {
                            id: cell.id.0,
                            position: cell.position,
                            tile_kind: cell.tile_kind,
                            worker_unit_instance_id: cell.worker_unit_id,
                        })
                        .collect(),
                })
                .collect(),
            units,
            tactical_resource: self.tactical_resource.to_dto(),
            deployment: self.live_deployment_dto(roster),
        })
    }
}

fn live_unit_tactical_state_dto(
    snapshot: UnitTacticalSnapshot,
    relocation_resource_available: bool,
) -> LiveBattleUnitTacticalStateDto {
    let eligibility = |disabled_reason| match disabled_reason {
        Some(reason) => BattlePayloadEligibilityDto::disabled(reason),
        None => BattlePayloadEligibilityDto::enabled(),
    };
    LiveBattleUnitTacticalStateDto {
        stance: snapshot.stance,
        tile_kind: snapshot.tile_kind,
        tile_effect: match snapshot.tile_effect {
            TacticalTileEffect::None => LiveBattleTileEffectDto::None,
            TacticalTileEffect::DefenseFocus => LiveBattleTileEffectDto::DefenseFocus,
            TacticalTileEffect::Rest => LiveBattleTileEffectDto::Rest,
        },
        stance_cooldown_until_ms: snapshot.stance_cooldown_until_ms,
        relocation_cooldown_until_ms: snapshot.relocation_cooldown_until_ms,
        forced_defense_until_ms: snapshot.forced_defense_until_ms,
        forced_defense_remaining_ms: snapshot.forced_defense_remaining_ms,
        relocation_eligibility: eligibility(snapshot.relocation_disabled_reason.or_else(|| {
            (!relocation_resource_available)
                .then_some(ActionDisabledReasonCode::InsufficientResources)
        })),
        stance_eligibility: LiveBattleStanceEligibilityDto {
            attack: eligibility(snapshot.attack_disabled_reason),
            defense: eligibility(snapshot.defense_disabled_reason),
        },
        base_block_capacity: snapshot.base_block_capacity,
        effective_block_capacity: snapshot.effective_block_capacity,
        defense_focus_damage_reduction_percent: snapshot.defense_focus_damage_reduction_percent,
        next_rest_tick_ms: snapshot.next_rest_tick_ms,
        activity: snapshot.activity,
    }
}

fn live_unit_hud_dto(unit: &crate::game::battle::core::types::RuntimeUnit) -> LiveBattleUnitHudDto {
    let uses_resonance = unit.role != BattleUnitRole::DefenseObject
        && (unit.owner == crate::game::enums::Side::Player
            || unit.threat_class.uses_resonance_bar());
    let bar_mode = if uses_resonance {
        LiveBattleHudBarMode::HpAndResonance
    } else {
        LiveBattleHudBarMode::HpOnly
    };

    LiveBattleUnitHudDto {
        threat_class: unit.threat_class,
        bar_mode,
        resonance_current: uses_resonance.then_some(unit.resonance_current),
        resonance_max: uses_resonance.then_some(unit.resonance_max),
    }
}

#[derive(Clone)]
pub struct RunState {
    pub typed_resources: crate::game::resource::durable::RunResourceState,
    pub map: RunMap,
    pub map_progression: MapProgression,
    pub run_progression: RunProgression,
    pub facility_run_state: FacilityRunState,
    pub encounter_progression: EncounterProgressionState,
    pub event_sessions: HashMap<crate::game::map::MapNodeId, EventSessionState>,
    pub shop_sessions: HashMap<crate::game::map::MapNodeId, ShopSessionState>,
    pub combat_previews: HashMap<crate::game::map::MapNodeId, CombatPreview>,
    pub abnormality_attempts: HashMap<crate::game::map::MapNodeId, AbnormalityAttemptState>,
    pub recruitment: RecruitmentState,
    pub record_progress: RunRecordProgress,
    pub combat_recovery_restriction: Option<CombatRecoveryRestriction>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordProgress {
    pub current_deathless_floor_streak: u32,
    pub employee_death_on_current_floor: bool,
}

impl RunRecordProgress {
    pub fn record_employee_death(&mut self) {
        self.employee_death_on_current_floor = true;
    }

    pub fn complete_floor(&mut self) -> u32 {
        if self.employee_death_on_current_floor {
            self.current_deathless_floor_streak = 0;
        } else {
            self.current_deathless_floor_streak =
                self.current_deathless_floor_streak.saturating_add(1);
        }
        self.employee_death_on_current_floor = false;
        self.current_deathless_floor_streak
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatRecoveryRestriction {
    pub allowed_node_ids: Vec<crate::game::map::MapNodeId>,
}

impl RunState {
    pub fn new(
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
    ) -> Self {
        Self {
            typed_resources: Default::default(),
            map,
            map_progression,
            run_progression,
            facility_run_state: FacilityRunState::default(),
            encounter_progression: EncounterProgressionState::default(),
            event_sessions: HashMap::new(),
            shop_sessions: HashMap::new(),
            combat_previews: HashMap::new(),
            abnormality_attempts: HashMap::new(),
            recruitment: RecruitmentState::default(),
            record_progress: RunRecordProgress::default(),
            combat_recovery_restriction: None,
        }
    }

    /// Build the next floor's run state from an existing run.
    ///
    /// Floor-local caches start empty, while run-scoped encounter progression is preserved.
    pub fn for_next_floor_from(
        previous: &Self,
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
        encounter_progression: EncounterProgressionState,
    ) -> Result<Self, GameError> {
        let mut next = Self::new(map, map_progression, run_progression)
            .with_persistent_run_state_from(previous)?
            .with_encounter_progression(encounter_progression);
        next.monolith.clear_floor_local_remanifestations();
        Ok(next)
    }

    /// Build the terminal run state from an existing run while preserving run-scoped progression.
    pub fn for_run_complete_from(
        previous: &Self,
        map: RunMap,
        map_progression: MapProgression,
        run_progression: RunProgression,
    ) -> Result<Self, GameError> {
        Self::new(map, map_progression, run_progression).with_persistent_run_state_from(previous)
    }

    fn with_persistent_run_state_from(mut self, previous: &Self) -> Result<Self, GameError> {
        self.typed_resources =
            crate::game::resource::store::clone_resource_value(&previous.typed_resources)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        self.facility_run_state = previous.facility_run_state.clone();
        self.encounter_progression = previous.encounter_progression.clone();
        self.recruitment = previous.recruitment.clone();
        self.record_progress = previous.record_progress.clone();
        Ok(self)
    }

    pub fn with_recruitment(mut self, recruitment: RecruitmentState) -> Self {
        self.recruitment = recruitment;
        self
    }

    pub fn with_encounter_progression(
        mut self,
        encounter_progression: EncounterProgressionState,
    ) -> Self {
        self.encounter_progression = encounter_progression;
        self
    }

    pub fn with_facility_run_state(mut self, facility_run_state: FacilityRunState) -> Self {
        self.facility_run_state = facility_run_state;
        self
    }

    pub fn abnormality_attempt_state(
        &self,
        node_id: crate::game::map::MapNodeId,
    ) -> AbnormalityAttemptState {
        let max_attempts = ABNORMALITY_MAX_ATTEMPTS;
        self.abnormality_attempts
            .get(&node_id)
            .copied()
            .unwrap_or_else(|| AbnormalityAttemptState::new(max_attempts))
    }

    pub fn abnormality_attempt_dto(
        &self,
        node_id: crate::game::map::MapNodeId,
    ) -> AbnormalityAttemptDto {
        self.abnormality_attempt_state(node_id).to_dto(node_id)
    }

    pub fn start_abnormality_attempt(
        &mut self,
        node_id: crate::game::map::MapNodeId,
    ) -> Option<AbnormalityAttemptState> {
        let max_attempts = ABNORMALITY_MAX_ATTEMPTS;
        let state = self
            .abnormality_attempts
            .entry(node_id)
            .or_insert_with(|| AbnormalityAttemptState::new(max_attempts));
        state.start_next_attempt().then_some(*state)
    }

    pub fn clear_abnormality_attempt(&mut self, node_id: crate::game::map::MapNodeId) {
        self.abnormality_attempts.remove(&node_id);
    }
}

impl std::ops::Deref for RunState {
    type Target = EncounterProgressionState;

    fn deref(&self) -> &Self::Target {
        &self.encounter_progression
    }
}

impl std::ops::DerefMut for RunState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encounter_progression
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        data::event_data::{EventId, EventSceneId},
        employee::{EmployeeDefinition, EmployeeInitialLoadout, EmployeeProfile},
        map::{test_facility_map, MapNodeId},
        recruitment::RecruitmentState,
    };

    fn test_recruitment_state() -> RecruitmentState {
        let definitions = vec![EmployeeDefinition {
            employee_definition_id: "floor_candidate".to_string(),
            name: "Floor Candidate".to_string(),
            starter_eligible: false,
            initial_loadout: EmployeeInitialLoadout::default(),
            profile: EmployeeProfile::default(),
        }];
        RecruitmentState::initialize(41, &definitions, &[]).unwrap()
    }

    fn generated_run_state(seed: u64, run_progression: RunProgression) -> RunState {
        let map = test_facility_map(seed);
        let progression = MapProgression::from_map(&map);
        RunState::new(map, progression, run_progression)
    }

    #[test]
    fn tactical_resource_redeploy_cost_uses_ceiling_for_odd_base_and_saturates() {
        let state = TacticalResourceState::new(TacticalResourcePolicy {
            initial: 0,
            maximum: 100,
            recovery_per_second: 0,
            base_deploy_cost: 3,
            redeploy_cost_multiplier_pct: 150,
        });
        assert_eq!(state.redeploy_cost(), 5);

        let overflow = TacticalResourceState::new(TacticalResourcePolicy {
            initial: 0,
            maximum: u32::MAX,
            recovery_per_second: 0,
            base_deploy_cost: u32::MAX,
            redeploy_cost_multiplier_pct: u32::MAX,
        });
        assert_eq!(overflow.redeploy_cost(), u32::MAX);
    }

    #[test]
    fn tactical_resource_recovery_payment_and_signed_delta_are_clamped_and_atomic() {
        let mut state = TacticalResourceState::new(TacticalResourcePolicy {
            initial: 20,
            maximum: 99,
            recovery_per_second: 1,
            base_deploy_cost: 10,
            redeploy_cost_multiplier_pct: 150,
        });

        state.recover_until(999);
        assert_eq!(state.current(), 20);
        state.recover_until(1_000);
        assert_eq!(state.current(), 21);
        state.recover_until(1_999);
        assert_eq!(state.current(), 21);
        state.recover_until(2_000);
        assert_eq!(state.current(), 22);

        assert!(!state.pay(23));
        assert_eq!(state.current(), 22, "failed full payment must not mutate");
        assert!(state.pay(10));
        assert_eq!(state.current(), 12);

        state.apply_delta(i32::MAX);
        assert_eq!(state.current(), 99);
        state.recover_until(12_000);
        state.apply_delta(-1);
        state.recover_until(12_000);
        assert_eq!(state.current(), 98, "capped recovery must not be banked");
        state.recover_until(13_000);
        assert_eq!(state.current(), 99);
        state.apply_delta(i32::MIN);
        assert_eq!(state.current(), 0);
    }

    #[test]
    fn tactical_resource_recovery_saturates_large_elapsed_products_without_wrapping() {
        let mut state = TacticalResourceState::new(TacticalResourcePolicy {
            initial: 0,
            maximum: u32::MAX,
            recovery_per_second: u32::MAX,
            base_deploy_cost: 0,
            redeploy_cost_multiplier_pct: 100,
        });

        state.recover_until(u64::MAX);
        assert_eq!(state.current(), u32::MAX);
    }

    #[test]
    fn run_power_anchor_uses_authored_minimum_previous_rise_and_peak_monotonically() {
        use crate::game::data::run_policy_data::ResolvedFloorPowerAnchorPolicy;

        let mut authority = RunPowerAuthorityState::default();
        authority.initialize_run(1, 80);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 100,
                minimum_rise: 0,
            }),
            100
        );
        authority.record_combat_entry(1, 130);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 110,
                minimum_rise: 25,
            }),
            130
        );
        authority.record_combat_entry(1, 120);
        assert_eq!(authority.run_peak_certified_power, 130);
        assert_eq!(
            authority.manifest_floor(ResolvedFloorPowerAnchorPolicy {
                authored_minimum_power: 140,
                minimum_rise: 25,
            }),
            155,
            "previous anchor plus minimum rise must win when it is strongest"
        );
    }

    #[test]
    fn next_floor_run_state_preserves_encounter_progression_and_resets_node_scoped_state() {
        let mut previous =
            generated_run_state(0xA11CE, RunProgression::new(0xBEEF, GameMode::Standard, 3));
        previous
            .completed_reward_source_ids
            .insert("test_source".to_string());
        previous
            .normal_boss_cycle
            .remaining_source_ids
            .push("test_boss".to_string());
        let node_id = MapNodeId::new(Uuid::from_u128(0xF100));
        previous.event_sessions.insert(
            node_id,
            EventSessionState::new(
                node_id,
                EventId::new("test_event"),
                EventSceneId::new("start"),
            ),
        );
        previous.abnormality_attempts.insert(
            node_id,
            AbnormalityAttemptState {
                max_attempts: 3,
                attempts_started: 2,
                retreats_used: 1,
            },
        );
        previous.recruitment = test_recruitment_state();
        let mut run_progression = previous.run_progression.clone();
        assert!(run_progression.advance_floor());
        let next_map = test_facility_map(0xF100);
        let next_progression = MapProgression::from_map(&next_map);
        let encounter_progression = previous.encounter_progression.clone();
        let next = RunState::for_next_floor_from(
            &previous,
            next_map,
            next_progression,
            run_progression,
            encounter_progression,
        )
        .unwrap();

        assert_eq!(next.encounter_progression, previous.encounter_progression);
        assert!(next.event_sessions.is_empty());
        assert!(next.combat_previews.is_empty());
        assert!(next.abnormality_attempts.is_empty());
        assert_eq!(next.recruitment, previous.recruitment);
    }

    #[test]
    fn run_complete_state_preserves_run_scoped_encounter_progression() {
        let mut previous =
            generated_run_state(0xC0DE, RunProgression::new(0xCAFE, GameMode::Standard, 1));
        previous
            .completed_reward_source_ids
            .insert("test_source".to_string());
        previous.recruitment = test_recruitment_state();

        let completed = RunState::for_run_complete_from(
            &previous,
            previous.map.clone(),
            previous.map_progression.clone(),
            previous.run_progression.clone(),
        )
        .unwrap();

        assert_eq!(
            completed.encounter_progression,
            previous.encounter_progression
        );
        assert_eq!(completed.recruitment, previous.recruitment);
    }

    #[test]
    fn deathless_floor_streak_counts_completed_floors_and_resets_after_any_employee_death() {
        let mut progress = RunRecordProgress::default();
        assert_eq!(progress.complete_floor(), 1);
        assert_eq!(progress.complete_floor(), 2);
        progress.record_employee_death();
        progress.record_employee_death();
        assert_eq!(progress.complete_floor(), 0);
        assert!(!progress.employee_death_on_current_floor);
        assert_eq!(progress.complete_floor(), 1);
    }
}

impl GameCoreState {
    pub fn new(run_seed: u64) -> Self {
        Self {
            player: None,
            lifecycle: LifecycleContext::not_started(),
            active_node_content: None,
            pending_mastered_elite_sweep: None,
            node_session: None,
            run: None,
            visual_novel: VisualNovelRuntimeState::default(),
            visual_novel_triggers: VisualNovelTriggerRuntimeState::default(),
            uuid_manager: UuidManager::new(run_seed),
            enkephalin: Enkephalin::new(0),
            inventory: Inventory::new(),
            roster: EmployeeRoster::new(),
            starter_candidates: Vec::new(),
            pending_game_mode: None,
            skill_fragments: SkillFragmentInventory::new(),
            relics: Default::default(),
            crafting_resources: Default::default(),
            skill_fragment_policy: SkillFragmentPolicy::default_run_policy(),
            player_profile: PlayerProfileState::default(),
            active_battle: None,
            confirmed_combat_result: None,
            combat_reward_continuation: None,
            run_checkpoint: RunCheckpointState::default(),
            continue_state: Default::default(),
            run_power_authority: RunPowerAuthorityState::default(),
        }
    }

    pub fn transition_to(&mut self, new_state: GameState) {
        self.lifecycle.transition_to(new_state);
    }

    pub fn initialize_player(&mut self, player_id: Uuid) -> bool {
        if self
            .player
            .as_ref()
            .is_some_and(|player| player.id == player_id)
        {
            return false;
        }

        self.player = Some(PlayerInfo::new(player_id, "Hero"));
        true
    }
}

crate::authoring::clone_shape::struct_shape!(GameCoreState {
    player,
    lifecycle,
    active_node_content,
    pending_mastered_elite_sweep,
    node_session,
    run,
    visual_novel,
    visual_novel_triggers,
    uuid_manager,
    enkephalin,
    inventory,
    roster,
    starter_candidates,
    pending_game_mode,
    skill_fragments,
    relics,
    crafting_resources,
    skill_fragment_policy,
    player_profile,
    active_battle,
    confirmed_combat_result,
    combat_reward_continuation,
    run_checkpoint,
    continue_state,
    run_power_authority
});

crate::authoring::clone_shape::struct_shape!(RunCheckpointState {
    payload,
    loads_used
});

crate::authoring::clone_shape::struct_shape!(ActiveBattleSession {
    node_id,
    battle_uuid,
    battle_state_revision,
    primary_abnormality_id,
    encounter_id,
    node_type,
    mission_variant,
    outcome_policies,
    combat_preview,
    battle,
    execution,
    entry_snapshot,
    employee_attempt,
    tactical_resource,
    live_deployment,
    playback,
    playback_delta_remainder,
    tactical_focus,
    interaction
});

crate::authoring::clone_shape::struct_shape!(BattleEntrySnapshot {
    game_state,
    active_node_content,
    node_session,
    run,
    visual_novel,
    visual_novel_triggers,
    uuid_manager,
    enkephalin,
    inventory,
    roster,
    starter_candidates,
    pending_game_mode,
    skill_fragments,
    relics,
    crafting_resources,
    run_checkpoint
});

crate::authoring::clone_shape::struct_shape!(LiveBattleDeploymentState {
    withdraw_redeploy_cooldown_ms,
    defeat_redeploy_cooldown_ms,
    next_instance_salt,
    deployed_units,
    redeploy_locks
});

crate::authoring::clone_shape::struct_shape!(LiveBattleRedeployState {
    ready_at_ms,
    deploy_cost,
    current_hp_policy
});

crate::authoring::clone_shape::struct_shape!(LiveBattleDeployedUnitState {
    unit_instance_id,
    facing
});

crate::authoring::clone_shape::struct_shape!(TacticalResourceState {
    current,
    maximum,
    recovery_per_second,
    last_recovery_at_ms,
    base_deploy_cost,
    redeploy_cost
});

crate::authoring::clone_shape::struct_shape!(RunState {
    typed_resources,
    map,
    map_progression,
    run_progression,
    facility_run_state,
    encounter_progression,
    event_sessions,
    shop_sessions,
    combat_previews,
    abnormality_attempts,
    recruitment,
    record_progress,
    combat_recovery_restriction
});

crate::authoring::clone_shape::struct_shape!(RunRecordProgress {
    current_deathless_floor_streak,
    employee_death_on_current_floor
});

crate::authoring::clone_shape::struct_shape!(CombatRecoveryRestriction { allowed_node_ids });

// Fresh attempt + insertion-only incapacitation keys; restores replace whole sets.
crate::authoring::clone_shape::struct_shape_with_sets!(BattleEmployeeAttemptState {
    roster
} sets { processed_incapacitations });
