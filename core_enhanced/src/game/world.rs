use tracing::{debug, info};
use uuid::Uuid;

use crate::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior};
use crate::game::data::visual_novel_data::VisualNovelPresentationMode;
use crate::game::data::{run_policy_data::RunPolicyData, ValidatedGameData};
use crate::game::encounter_compendium::{EncounterCompendiumProfileState, PlayerProfileState};
use crate::game::encounter_progression::EncounterProgressionState;
use crate::game::enums::{RewardAction, ShopAction};
use crate::game::map::{GameMode, RunProgression};
use crate::game::recruitment::RecruitmentState;
#[cfg(test)]
use crate::game::resources::ActiveNodeContent;
use crate::game::resources::GameState;
use crate::game::visual_novel::{
    VisualNovelRuntimeState, VisualNovelSessionPresentationMode, VisualNovelTriggerRuntimeState,
};

mod action_capability;
mod active_run;
mod admin;
mod application;
pub(crate) mod authoring_lifecycle;
pub(crate) mod authoring_observation;
mod battle_capability;
mod battle_interaction;
mod combat;
#[cfg(any(test, debug_assertions, feature = "test-fixtures"))]
mod combat_trace;
mod event_node;
mod expedition_lifecycle;
mod expedition_recovery;
mod facility;
mod facility_rooms;
mod grant_state;
mod headquarters;
mod helpers;
mod maintenance;
mod map_content;
mod map_encounters;
mod mastered_elite_sweep;
mod revival;
mod node_flow;
mod relic_runtime;
mod relic_reward;
mod dealer;
mod resource_runtime;
mod reward;
mod shop;
mod snapshot;
mod state;
mod tactical_preview;
mod visual_novel_flow;
mod visual_novel_triggers;

use state::{GameCoreState, RunState};

#[cfg(test)]
pub(crate) use active_run::ActiveRunCheckpointExport;
pub(crate) use active_run::ActiveRunTransactionRollback;
pub(crate) use admin::AdminCommand;
pub(crate) use application::PreparedApplicationCommit;

#[derive(Clone)]
pub(crate) struct GameCore {
    state: GameCoreState,
    game_data: ValidatedGameData,
    run_seed: u64,
    authoring_lifecycle: Option<authoring_lifecycle::TrialLifecycle>,
    authoring_observation: Option<authoring_observation::TrialObservation>,
}

crate::authoring::clone_shape::struct_shape!(GameCore {
    state,
    game_data,
    run_seed,
    authoring_lifecycle,
    authoring_observation
});

fn runtime_presentation_mode(
    mode: VisualNovelPresentationMode,
) -> VisualNovelSessionPresentationMode {
    match mode {
        VisualNovelPresentationMode::Cinematic => VisualNovelSessionPresentationMode::Cinematic,
        VisualNovelPresentationMode::Radio => VisualNovelSessionPresentationMode::Radio,
        VisualNovelPresentationMode::Compact => VisualNovelSessionPresentationMode::Compact,
        VisualNovelPresentationMode::NonBlocking => VisualNovelSessionPresentationMode::NonBlocking,
    }
}

impl GameCore {
    pub(crate) fn prepare_lab(&mut self,recipe:&crate::authoring::LabPreparation)->Result<crate::authoring::LabPreparationReceipt,GameError>{
        recipe.validate()?;
        crate::authoring::control::metadata_size(recipe)?;
        crate::authoring::control::charge_serialized(recipe,"lab_bulk_request")?;
        self.execute_application(application::ApplicationRoute::AdminCommand,|draft|draft.prepare_lab_draft(recipe))
    }
    pub(crate) fn runtime_safety(&self) -> &crate::runtime_safety::RuntimeSafetySettings {
        self.game_data.runtime_safety()
    }

    pub(crate) fn active_battle_authority(
        &self,
    ) -> Option<crate::game::behavior::BattleStateAuthorityDto> {
        self.state.active_battle.as_ref().map(|active| {
            crate::game::behavior::BattleStateAuthorityDto {
                battle_uuid: active.battle_uuid,
                battle_state_revision: active.battle_state_revision,
            }
        })
    }

    pub(crate) fn content_identity(&self) -> &crate::game::data::content_pack::ContentIdentity {
        self.game_data.identity()
    }

    #[cfg(feature = "test-fixtures")]
    pub(crate) fn replace_active_battle_event_log_fixture(
        &mut self,
        event_log: crate::game::battle::event_log::BattleEventLog,
    ) -> Result<u64, GameError> {
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        let latest = event_log.latest_seq();
        active.battle.event_log = event_log;
        active.battle.event_log_seq = latest.saturating_add(1);
        Ok(latest)
    }

    /// GameCore 생성
    ///
    /// # Arguments
    /// * `game_data` - validated Core game-data publication
    pub fn new(game_data: impl Into<ValidatedGameData>, run_seed: u64) -> Self {
        info!("Initializing GameCore with run_seed={}", run_seed);

        debug!("GameCore state initialized with default resources");
        let game_data = game_data.into();

        Self {
            state: GameCoreState::new(run_seed),
            game_data,
            run_seed,
            authoring_lifecycle: None,
            authoring_observation: None,
        }
    }

    pub(crate) fn new_for_authoring(game_data: ValidatedGameData, run_seed: u64) -> Self {
        let mut core = Self::new(game_data, run_seed);
        core.authoring_lifecycle = Some(Default::default());
        core.authoring_observation = Some(Default::default());
        core
    }

    #[cfg(test)]
    pub fn new_with_encounter_compendium_profile(
        game_data: impl Into<ValidatedGameData>,
        run_seed: u64,
        profile: EncounterCompendiumProfileState,
    ) -> Result<Self, GameError> {
        let mut core = Self::new(game_data, run_seed);
        core.merge_encounter_compendium_profile(profile)?;
        Ok(core)
    }

    pub(crate) fn capture_player_profile(&self) -> Result<PlayerProfileState, GameError> {
        crate::game::resource::store::preflight_resource_clone(&self.state.player_profile)
            .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        crate::authoring::clone_shape::controlled_clone(&self.state.player_profile)
    }

    pub fn player_profile(&self) -> PlayerProfileState {
        self.state.player_profile.clone()
    }

    pub fn encounter_compendium_profile(&self) -> EncounterCompendiumProfileState {
        self.state.player_profile.encounter_compendium.clone()
    }

    pub fn merge_encounter_compendium_profile(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> Result<(), GameError> {
        self.execute_application(
            application::ApplicationRoute::MergeEncounterCompendiumProfile,
            move |draft| draft.merge_encounter_compendium_profile_draft(profile),
        )
    }

    fn merge_encounter_compendium_profile_draft(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> Result<(), GameError> {
        profile
            .validate_persisted()
            .map_err(GameError::InvalidStaticData)?;
        self.state
            .player_profile
            .encounter_compendium
            .merge(profile);
        Ok(())
    }

    pub fn merge_player_profile(&mut self, profile: PlayerProfileState) -> Result<(), GameError> {
        self.execute_application(
            application::ApplicationRoute::MergePlayerProfile,
            move |draft| draft.merge_player_profile_draft(profile),
        )
    }

    fn merge_player_profile_draft(&mut self, profile: PlayerProfileState) -> Result<(), GameError> {
        profile
            .validate_persisted()
            .map_err(GameError::InvalidStaticData)?;
        self.state.player_profile.merge(profile).map_err(GameError::InvalidStaticData)
    }

    /// Host authority installation replaces exact ownership; it never unions vault assets.
    pub fn install_authoritative_player_profile(&mut self, profile: PlayerProfileState) -> Result<(), GameError> {
        self.execute_application(application::ApplicationRoute::InstallPlayerProfile, move |draft| draft.install_authoritative_profile_draft(profile))
    }

    pub(crate) fn prepare_authoritative_profile_install(&mut self, profile: PlayerProfileState) -> Result<PreparedApplicationCommit<()>, GameError> {
        self.prepare_application_for_publication(application::ApplicationRoute::InstallPlayerProfile, move |draft| draft.install_authoritative_profile_draft(profile))
    }

    fn install_authoritative_profile_draft(&mut self, profile: PlayerProfileState) -> Result<(), GameError> {
    profile.validate_persisted().map_err(GameError::InvalidStaticData)?;
    if let Some(run) = self.state.run.as_ref() {
        if run.run_progression.game_mode == GameMode::Expedition
            && !matches!(self.state.lifecycle.phase(), GameState::RunComplete | GameState::RunFailed { .. }) {
            if profile.lifecycle.active_expedition != run.run_progression.expedition_instance {
                return Err(GameError::InvalidAction);
            }
            profile.lifecycle.vault.validate_active_ownership(&self.state.inventory, &self.state.skill_fragments)
                .map_err(GameError::InvalidStaticData)?;
        }
    }
    self.state.player_profile = profile;
    Ok(())
    }

    pub fn reset_player_profile(&mut self) -> Result<(), GameError> {
        self.execute_application(application::ApplicationRoute::ResetPlayerProfile, |draft| {
            if draft.state.player_profile.lifecycle.active_expedition.is_some() { return Err(GameError::InvalidAction); }
            draft.state.player_profile = PlayerProfileState::default();
            Ok(())
        })
    }

    pub(super) fn run_policy(&self) -> &RunPolicyData {
        &self.game_data.run_policy
    }

    #[cfg(test)]
    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_with_source_command_id(player_id, behavior, None)
    }

    #[cfg(test)]
    pub fn execute_with_source_command_id(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_with_source_command_id_and_battle_cursor(
            player_id,
            behavior,
            source_command_id,
            0,
        )
    }

    #[cfg(test)]
    pub fn execute_with_source_command_id_and_battle_cursor(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_with_transport_context(
            player_id,
            behavior,
            source_command_id,
            battle_event_after_seq,
            None,
        )
    }

    #[cfg(test)]
    pub fn execute_with_transport_context(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
        expected_battle_authority: Option<crate::game::behavior::BattleStateAuthorityDto>,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_with_transport_context_and_generation(
            player_id,
            behavior,
            source_command_id,
            battle_event_after_seq,
            expected_battle_authority,
            None,
        )
    }

    pub(crate) fn execute_with_transport_context_and_generation(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
        expected_battle_authority: Option<crate::game::behavior::BattleStateAuthorityDto>,
        generation: Option<ValidatedGameData>,
    ) -> Result<BehaviorResult, GameError> {
        if generation.is_some() && !matches!(behavior, PlayerBehavior::StartNewGame { .. }) {
            return Err(GameError::InvalidAction);
        }
        let route = application::ApplicationRoute::Behavior(behavior.kind());
        self.execute_application(route, move |draft| {
            draft.execute_behavior_draft(
                player_id,
                behavior,
                source_command_id,
                battle_event_after_seq,
                expected_battle_authority,
                generation,
            )
        })
    }

    pub(crate) fn prepare_with_transport_context(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
        expected_battle_authority: Option<crate::game::behavior::BattleStateAuthorityDto>,
        generation: Option<ValidatedGameData>,
    ) -> Result<PreparedApplicationCommit<BehaviorResult>, GameError> {
        if generation.is_some() && !matches!(behavior, PlayerBehavior::StartNewGame { .. }) {
            return Err(GameError::InvalidAction);
        }
        let route = application::ApplicationRoute::Behavior(behavior.kind());
        self.prepare_application_for_publication(route, move |draft| {
            draft.execute_behavior_draft(
                player_id,
                behavior,
                source_command_id,
                battle_event_after_seq,
                expected_battle_authority,
                generation,
            )
        })
    }

    fn execute_behavior_draft(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
        expected_battle_authority: Option<crate::game::behavior::BattleStateAuthorityDto>,
        generation: Option<ValidatedGameData>,
    ) -> Result<BehaviorResult, GameError> {
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        let action_kind = behavior.kind();
        if matches!(action_kind, ActionKind::ConvertMapNodeWithMonolith | ActionKind::SweepMasteredElite | ActionKind::CompleteMasteredEliteSweep) {
            return Err(Self::retired_encounter_action(action_kind));
        }
        let tactical_world_mutation = matches!(
            &behavior,
            PlayerBehavior::DeployUnit { .. }
                | PlayerBehavior::WithdrawUnit { .. }
                | PlayerBehavior::SetEngagementStance { .. }
                | PlayerBehavior::TacticalRelocateUnit { .. }
                | PlayerBehavior::TacticalSwapUnits { .. }
        );

        self.validate_battle_command_authority(&behavior, expected_battle_authority)?;

        if application::behavior_owner(action_kind) == application::ApplicationOwner::Battle
            && !matches!(behavior, PlayerBehavior::RecoverBattleSetupLoss)
        {
            if let Some(active) = self.state.active_battle.as_ref() {
                active.ensure_battle_update_cursor_valid(battle_event_after_seq)?;
            }
        }

        self.gate_behavior_action(player_id, &behavior, action_kind)?;
        if let Some(generation) = generation {
            // The existing eligibility gate runs against the original root.
            // Only an eligible new-run draft installs the candidate, before VN,
            // starter and subsequent projections read any of its definitions.
            self.game_data = generation;
        }
        let mut result = self.execute_owned_behavior_draft(
            player_id,
            behavior,
            source_command_id,
            battle_event_after_seq,
        )?;
        if matches!(result, BehaviorResult::VisualNovelState { .. })
            && matches!(self.state.lifecycle.phase(), GameState::InBattle { .. })
        {
            result = self
                .current_battle_update_result_for_visual_novel_transport(battle_event_after_seq)?;
        }
        // A finishing advance projected its final VN/capabilities before
        // removing the active battle. Re-projecting here would query a removed
        // root, rejecting an otherwise successful immediate-terminal entry.
        let terminal_advance = matches!(
            result,
            BehaviorResult::BattleAdvanced { finished: true, .. }
        );
        if !tactical_world_mutation && !terminal_advance {
            self.attach_visual_novel_to_battle_result(&mut result)?;
        }
        Ok(result)
    }

    fn validate_battle_command_authority(
        &self,
        behavior: &PlayerBehavior,
        expected: Option<crate::game::behavior::BattleStateAuthorityDto>,
    ) -> Result<(), GameError> {
        let Some(active) = self.state.active_battle.as_ref() else {
            return Ok(());
        };
        if !behavior.kind().requires_battle_authority() {
            return Ok(());
        }
        let current = crate::game::behavior::BattleStateAuthorityDto {
            battle_uuid: active.battle_uuid,
            battle_state_revision: active.battle_state_revision,
        };
        let Some(expected) = expected else {
            return Err(GameError::MissingBattleStateAuthority { current });
        };
        let replay=matches!(behavior,PlayerBehavior::CommitInputPlan{battle_uuid,employee_uuid,input,..} if *battle_uuid==current.battle_uuid && active.battle.input_plan_employee_receipt(*employee_uuid,input.preparation,input.request).is_some());
        if expected == current || (expected.battle_uuid==current.battle_uuid && replay) {
            Ok(())
        } else {
            Err(GameError::StaleBattleStateAuthority { expected, current })
        }
    }

    pub fn active_battle_reconnect_projection(
        &self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let mut result = self.handle_recover_battle_setup_loss(after_seq)?;
        self.attach_visual_novel_to_battle_result(&mut result)?;
        Ok(result)
    }

    fn gate_behavior_action(
        &self,
        player_id: Uuid,
        behavior: &PlayerBehavior,
        action_kind: ActionKind,
    ) -> Result<(), GameError> {
        if self.preflight_common_capability_behavior(behavior)? {
            return Ok(());
        }
        if self.preflight_battle_capability_behavior(behavior)? {
            return Ok(());
        }
        debug!(
            "Behavior {:?} for player {} is owned by its domain preflight (action kind {:?})",
            behavior, player_id, action_kind
        );
        Ok(())
    }

    fn execute_owned_behavior_draft(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        match application::behavior_owner(behavior.kind()) {
            application::ApplicationOwner::MapRun => {
                self.execute_map_run_behavior(player_id, behavior)
            }
            application::ApplicationOwner::Headquarters => {
                self.execute_headquarters_behavior(behavior)
            }
            application::ApplicationOwner::Maintenance => {
                self.execute_equipment_maintenance_behavior(behavior)
            }
            application::ApplicationOwner::Shop => self.execute_shop_behavior(behavior),
            application::ApplicationOwner::Reward => self.execute_reward_behavior(behavior),
            application::ApplicationOwner::Event => self.execute_event_behavior(behavior),
            application::ApplicationOwner::VisualNovel => {
                if matches!(self.state.lifecycle.phase(), GameState::InBattle { .. }) {
                    self.state
                        .active_battle
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?
                        .ensure_battle_projection_commit_available()?;
                }
                self.execute_visual_novel_behavior(behavior)
            }
            application::ApplicationOwner::Battle => {
                self.execute_battle_behavior(behavior, source_command_id, battle_event_after_seq)
            }
            application::ApplicationOwner::Admin
            | application::ApplicationOwner::ActiveRunRecovery
            | application::ApplicationOwner::PlayerProfile => {
                unreachable!("non-behavior owner returned for PlayerBehavior")
            }
            #[cfg(test)]
            application::ApplicationOwner::TestFixture => {
                unreachable!("fixture owner returned for PlayerBehavior")
            }
        }
    }

    fn execute_map_run_behavior(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::StartNewGame { game_mode, replace_active_run } => {
                if replace_active_run { return Err(GameError::InvalidAction); }
                self.handle_start_new_game(player_id, game_mode)
            }
            PlayerBehavior::ReturnFromExpedition { materials } => self.handle_return_from_expedition(&materials),
            PlayerBehavior::ChooseExpeditionContinuation { choice } => self.handle_choose_expedition_continuation(choice),
            PlayerBehavior::SkipTutorial => self.handle_skip_tutorial(),
            PlayerBehavior::AcknowledgeFirstReturnStory => self.handle_acknowledge_first_return_story(),
            PlayerBehavior::ContinueRun => Err(GameError::InvalidAction),
            PlayerBehavior::ResetActiveRun => Err(GameError::InvalidAction),
            PlayerBehavior::RetryPlayerProfile => Err(GameError::InvalidAction),
            PlayerBehavior::ResetPlayerProfile => Err(GameError::InvalidAction),
            PlayerBehavior::SelectStarterEmployees { employee_definition_ids, loan_copy_ids } => {
                if self.state.pending_game_mode != Some(GameMode::Expedition) { return Err(GameError::InvalidAction); }
                self.validate_starter_employee_selection(&employee_definition_ids)?;
                self.handle_select_starter_employees(employee_definition_ids, loan_copy_ids)
            }
            PlayerBehavior::RequestMapData => self.handle_request_map_data(),
            PlayerBehavior::MoveToFacilityRoom { room_id } => self.handle_move_to_facility_room(&room_id),
            PlayerBehavior::SelectMapNode { node_id } => {
                self.validate_select_map_node(node_id)?;
                self.handle_select_map_node(node_id)
            }
            PlayerBehavior::HatchEncounter { node_id, reward_source_id, stage, payment } => self.handle_hatch_encounter(node_id, &reward_source_id, stage, &payment),
            PlayerBehavior::ConvertMapNodeWithMonolith {
                node_id,
                abnormality_id,
            } => self.handle_convert_map_node_with_monolith(node_id, &abnormality_id),
            PlayerBehavior::ConfirmEnterNode => self.handle_confirm_enter_node(),
            PlayerBehavior::CancelSelectedNode => self.handle_cancel_selected_node(),
            PlayerBehavior::SweepMasteredElite { .. } => self.handle_sweep_mastered_elite(),
            PlayerBehavior::CompleteMasteredEliteSweep { .. } => {
                self.handle_complete_mastered_elite_sweep()
            }
            PlayerBehavior::CompleteNode => self.handle_complete_node(),
            PlayerBehavior::LoadRunCheckpoint => self.handle_load_run_checkpoint(),
            PlayerBehavior::ConfirmRunFailure => self.handle_confirm_run_failure(),
            PlayerBehavior::AbandonRun => self.handle_abandon_run(),
            _ => unreachable!("behavior routed to map/run domain incorrectly"),
        }
    }

    fn execute_headquarters_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::RecruitEmployee {
                employee_definition_id,
            } => self.handle_recruit_employee(&employee_definition_id),
            PlayerBehavior::RequestEmergencySupplies => self.handle_request_emergency_supplies(),
            PlayerBehavior::OpenHeadquartersShop => self.handle_open_headquarters_shop(),
            _ => unreachable!("behavior routed to headquarters domain incorrectly"),
        }
    }

    fn execute_equipment_maintenance_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::EquipRelic {
                copy_uuid,
                employee_uuid,
                slot,
            } => self.handle_equip_relic(copy_uuid, employee_uuid, slot),
            PlayerBehavior::UnequipRelic { copy_uuid } => self.handle_unequip_relic(copy_uuid),
            PlayerBehavior::RequestRelicState => self.handle_request_relic_state(),
            PlayerBehavior::RequestRelicNumericPlan { employee_uuid } => {
                self.handle_request_relic_numeric_plan(employee_uuid)
            }
            PlayerBehavior::RequestRelicEffectivePlan { employee_uuid } => {
                self.handle_request_relic_effective_plan(employee_uuid)
            }
            PlayerBehavior::PreviewRelicCorruption { copy_uuid } => self
                .handle_preview_relic(copy_uuid, crate::game::relic::RelicOperationKind::Corrupt),
            PlayerBehavior::CorruptRelic { copy_uuid } => self.handle_corrupt_relic(copy_uuid),
            PlayerBehavior::PreviewSkillFragmentBloom { copy_uuid, form_id, materials, weapon_copy_uuid } =>
                self.handle_preview_skill_fragment_bloom(copy_uuid, &form_id, &materials, weapon_copy_uuid),
            PlayerBehavior::PreviewRelicDismantle { copy_uuid } => self
                .handle_preview_relic(copy_uuid, crate::game::relic::RelicOperationKind::Dismantle),
            PlayerBehavior::DismantleRelic {
                copy_uuid,
                confirmed,
            } => self.handle_dismantle_relic(copy_uuid, confirmed),
            PlayerBehavior::EquipItem {
                item_uuid,
                target_unit,
            } => {
                self.validate_equip_item_payload(item_uuid, target_unit)?;
                self.handle_equip_item(item_uuid, target_unit)
            }
            PlayerBehavior::UnEquipItem {
                item_uuid,
                target_unit,
            } => self.handle_unequip_item(item_uuid, target_unit),
            PlayerBehavior::UseConsumableItem {
                item_uuid,
                target_employee_uuid,
            } => self.handle_use_consumable_item(item_uuid, target_employee_uuid),
            PlayerBehavior::EquipSkillFragment {
                employee_uuid,
                copy_uuid,
            } => self.handle_equip_skill_fragment(employee_uuid, copy_uuid),
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid,
                copy_uuid,
            } => self.handle_unequip_skill_fragment(employee_uuid, copy_uuid),
            PlayerBehavior::TransferSkillFragment {
                copy_uuid,
                target_employee_uuid,
            } => self.handle_transfer_skill_fragment(copy_uuid, target_employee_uuid),
            PlayerBehavior::AwakenSkillFragmentWithWeapon { copy_uuid, form_id, weapon_copy_uuid, materials } => self.handle_awaken_skill_fragment_selected(copy_uuid, &form_id, materials.as_deref(), Some(weapon_copy_uuid)),
            PlayerBehavior::AwakenSkillFragmentWithMaterials { copy_uuid, form_id, materials } => self.handle_awaken_skill_fragment_with_materials(copy_uuid, &form_id, Some(&materials)),
            PlayerBehavior::AwakenSkillFragment { copy_uuid, form_id } => {
                self.handle_awaken_skill_fragment(copy_uuid, &form_id)
            }
            PlayerBehavior::SelectSkillFragmentForm { copy_uuid, form_id } => {
                self.handle_select_skill_fragment_form(copy_uuid, &form_id)
            }
            PlayerBehavior::DismantleSkillFragment {
                copy_uuid,
                confirmed,
            } => self.handle_dismantle_skill_fragment(copy_uuid, confirmed),
            PlayerBehavior::DismantleEquipmentConfirmed { item_uuid, confirmed } => self.handle_dismantle_equipment_confirmed(item_uuid, confirmed),
            PlayerBehavior::DismantleEquipment { item_uuid } => {
                self.handle_dismantle_equipment(item_uuid)
            }
            PlayerBehavior::EnhanceEquipment { item_uuid } => {
                self.handle_enhance_equipment(item_uuid)
            }
            _ => unreachable!("behavior routed to equipment/maintenance domain incorrectly"),
        }
    }

    fn execute_shop_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::PurchaseItem { offer_id } => {
                self.execute_shop_action(ShopAction::Purchase { offer_id })
            }
            PlayerBehavior::SellItem {
                item_uuid,
                quantity,
            } => self.execute_shop_action(ShopAction::Sell {
                item_uuid,
                quantity,
            }),
            PlayerBehavior::RefreshShop => self.execute_shop_action(ShopAction::Refresh),
            PlayerBehavior::ExitShop => self.execute_shop_action(ShopAction::Exit),
            _ => unreachable!("behavior routed to shop domain incorrectly"),
        }
    }

    fn execute_reward_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::SelectReward { reward_id } => {
                self.validate_select_reward_payload(reward_id)?;
                self.handle_select_reward(reward_id)
            }
            PlayerBehavior::InvestRelicReward => self.handle_invest_relic_reward(),
            PlayerBehavior::SelectRelicFusionInputs { copies } => self.handle_select_relic_fusion_inputs(copies),
            PlayerBehavior::ConfirmRelicFusion => self.handle_confirm_relic_fusion(),
            PlayerBehavior::DealerAct { revision, action } => self.handle_dealer_action(revision, action),
            PlayerBehavior::ClaimReward => self.execute_reward_action(RewardAction::Claim),
            PlayerBehavior::ExitReward => self.execute_reward_action(RewardAction::Exit),
            _ => unreachable!("behavior routed to reward domain incorrectly"),
        }
    }

    fn execute_event_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::AdvanceEventScene {
                node_id,
                event_id,
                current_scene_id,
            } => self.handle_advance_event_scene(node_id, event_id, current_scene_id),
            PlayerBehavior::ResolveEventChoiceCheck {
                node_id,
                event_id,
                current_scene_id,
                choice_id,
                employee_uuid,
            } => self.handle_resolve_event_choice_check(
                node_id,
                event_id,
                current_scene_id,
                choice_id,
                employee_uuid,
            ),
            PlayerBehavior::ChooseRelicCorruption {
                node_id,
                event_id,
                current_scene_id,
                copy_uuid,
            } => {
                self.handle_choose_relic_corruption(node_id, event_id, current_scene_id, copy_uuid)
            }
            _ => unreachable!("behavior routed to event domain incorrectly"),
        }
    }

    fn execute_visual_novel_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::AdvanceVisualNovel {
                session_id,
                current_scene_id,
            } => self.handle_advance_visual_novel(session_id, current_scene_id),
            PlayerBehavior::SelectVisualNovelChoice {
                session_id,
                current_scene_id,
                choice_id,
            } => self.handle_select_visual_novel_choice(session_id, current_scene_id, choice_id),
            _ => unreachable!("behavior routed to visual novel domain incorrectly"),
        }
    }

    fn execute_battle_behavior(
        &mut self,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::CompleteCombatResult => self.handle_complete_combat_result(),
            PlayerBehavior::RequestBattleState { since_seq } => {
                self.handle_request_battle_state(since_seq)
            }
            PlayerBehavior::RecoverBattleSetupLoss => {
                self.handle_recover_battle_setup_loss(battle_event_after_seq)
            }
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position,
            } => self.handle_request_deployment_range_preview(employee_uuid, position),
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing,
            } => self.handle_deploy_unit(
                employee_uuid,
                position,
                facing,
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::WithdrawUnit { employee_uuid } => {
                self.handle_withdraw_unit(employee_uuid, source_command_id, battle_event_after_seq)
            }
            PlayerBehavior::SetEngagementStance {
                employee_uuid,
                stance,
            } => self.handle_set_engagement_stance(
                employee_uuid,
                stance,
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::TacticalRelocateUnit {
                employee_uuid,
                destination,
                facing,
            } => self.handle_tactical_relocate_unit(
                employee_uuid,
                destination,
                facing,
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::TacticalSwapUnits {
                first_employee_uuid,
                second_employee_uuid,
                first_facing,
                second_facing,
            } => self.handle_tactical_swap_units(
                first_employee_uuid,
                second_employee_uuid,
                first_facing,
                second_facing,
                source_command_id,
                battle_event_after_seq,
            ),
            behavior @ (PlayerBehavior::FocusBattleUnit { .. }
            | PlayerBehavior::ClearBattleFocus
            | PlayerBehavior::BeginDeploymentSelection { .. }
            | PlayerBehavior::CancelDeploymentSelection
            | PlayerBehavior::OpenBattleSettings
            | PlayerBehavior::CloseBattleSettings) => {
                self.handle_battle_interaction(behavior, battle_event_after_seq)
            }
            PlayerBehavior::BeginSkillTargeting {
                employee_uuid,
                skill_id,
            } => self.handle_begin_skill_targeting(employee_uuid, skill_id, battle_event_after_seq),
            PlayerBehavior::CancelSkillTargeting => {
                self.handle_cancel_skill_targeting(battle_event_after_seq)
            }
            PlayerBehavior::PreviewTacticalCommand { command } => {
                self.handle_preview_tactical_command(command)
            }
            PlayerBehavior::PreviewSkillTarget {
                employee_uuid,
                skill_id,
                target,
            } => self.handle_preview_skill_target(employee_uuid, skill_id, target),
            PlayerBehavior::ReinputSustainedSkill {
                employee_uuid,
                skill_id,
                activation,
                target,
            } => self.handle_activate_skill(
                employee_uuid,
                skill_id,
                target,
                Some(activation),
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::SelectSkillRelation { employee_uuid, entry, target } =>
                self.handle_select_skill_relation(employee_uuid, entry, target, source_command_id, battle_event_after_seq),
            PlayerBehavior::CommitInputPlan{battle_uuid,revision,employee_uuid,input}=>self.handle_input_plan_command(battle_uuid,revision,employee_uuid,Some(input),None,source_command_id,battle_event_after_seq),
            PlayerBehavior::CancelInputPlan{battle_uuid,revision,employee_uuid,preparation}=>self.handle_input_plan_command(battle_uuid,revision,employee_uuid,None,Some(preparation),source_command_id,battle_event_after_seq),
            PlayerBehavior::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            } => self.handle_activate_skill(
                employee_uuid,
                skill_id,
                target,
                None,
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::RetreatBattle => self.handle_retreat_battle(),
            PlayerBehavior::StressCombatAct { battle_uuid, completed_rounds, extract } => self.handle_stress_combat_action(battle_uuid, completed_rounds, extract),
            PlayerBehavior::PauseBattle => self.handle_pause_battle(battle_event_after_seq),
            PlayerBehavior::ResumeBattle => self.handle_resume_battle(battle_event_after_seq),
            PlayerBehavior::SetBattleSpeed { speed } => {
                self.handle_set_battle_speed(speed, battle_event_after_seq)
            }
            _ => unreachable!("behavior routed to battle domain incorrectly"),
        }
    }
}

impl GameCore {
    fn run_state(&self) -> Result<&RunState, GameError> {
        self.state
            .run
            .as_ref()
            .ok_or(GameError::MissingResource("RunState"))
    }

    fn run_state_mut(&mut self) -> Result<&mut RunState, GameError> {
        self.state
            .run
            .as_mut()
            .ok_or(GameError::MissingResource("RunState"))
    }

    // 플레이어가 게임에 첫 진입을 하였을 때.
    // 바로 런을 시작하지 않고, 본부가 제시한 시작 직원 후보 선택 단계로 진입한다.
    fn handle_start_new_game(
        &mut self,
        player_id: Uuid,
        game_mode: GameMode,
    ) -> Result<BehaviorResult, GameError> {
        if !matches!(self.state.lifecycle.phase(), GameState::NotStarted | GameState::RunComplete | GameState::RunFailed { .. })
            || self.state.player_profile.lifecycle.active_expedition.is_some() {
            return Err(GameError::InvalidAction);
        }
        let player_profile = self.capture_player_profile()?;
        self.state = GameCoreState::new(self.run_seed);
        self.state.player_profile = player_profile;
        self.initial_player(player_id);
        self.state.pending_game_mode = Some(game_mode);
        let activated = self.evaluate_visual_novel_triggers(
            visual_novel_triggers::VisualNovelTriggerEvent::RunStart { game_mode },
        )?;
        if activated == 0 || self.state.visual_novel.active.is_none() {
            return Err(GameError::InvalidStaticData(format!(
                "missing RunStart visual novel trigger for {:?}",
                game_mode
            )));
        }
        self.transition_to(GameState::PreRunBriefing { game_mode })?;

        info!(
            "Run start flow opened pre-run HQ briefing for player {} in {:?} mode",
            player_id, game_mode
        );
        self.record_lab_definition("RunPolicy","run_policy","run_start_policy")?;
        Ok(BehaviorResult::RunStartFlowStarted {
            game_mode,
            visual_novel: self.visual_novel_snapshot_dto()?,
        })
    }

    fn handle_abandon_run(&mut self) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_none() {
            return Err(GameError::InvalidAction);
        }
        self.settle_expedition_terminal(false)?;
        let player = self.state.player.clone();
        let player_profile = self.capture_player_profile()?;
        let mut next = GameCoreState::new(self.run_seed);
        next.player = player;
        next.player_profile = player_profile;
        self.state = next;
        Ok(BehaviorResult::RunAbandoned)
    }

    fn prepare_starter_employee_candidates(&mut self) -> Result<usize, GameError> {
        let required_count = self.run_policy().setup.starter_employee_count;
        self.state.starter_candidates = self
            .game_data
            .employee_definition_data
            .starter_definitions();
        if self.state.starter_candidates.len() < required_count {
            return Err(GameError::InvalidStaticData(format!(
                "starter employee candidate data must contain at least {} candidates, got {}",
                required_count,
                self.state.starter_candidates.len()
            )));
        }
        Ok(required_count)
    }

    fn handle_select_starter_employees(
        &mut self,
        employee_definition_ids: Vec<String>,
        loan_copy_ids: Vec<Uuid>,
    ) -> Result<BehaviorResult, GameError> {
        let game_mode = self.state.pending_game_mode.ok_or(GameError::InvalidAction)?;
        let run_progression = self.prepare_departure_identity(game_mode)?;
        let setup_policy = self.run_policy().setup;
        let employee_uuids =
            self.initialize_selected_starter_employees(&employee_definition_ids)?;
        // 기초 자원 지급: 첫 안전 노드에서 상점에 들어가도 하나는 살 수 있도록 여유 있게.
        self.state.enkephalin.amount = 0;
        self.state.enkephalin.checked_add(setup_policy.starter_enkephalin)?;
        if game_mode == GameMode::Expedition { self.take_departure_loans(&loan_copy_ids)?; }
        else if !loan_copy_ids.is_empty() { return Err(GameError::InvalidAction); }
        info!(
            "Granted starter enkephalin: amount={}, total={}",
            setup_policy.starter_enkephalin, self.state.enkephalin.amount
        );

        self.state.pending_game_mode = None;
        let mut encounter_progression = EncounterProgressionState::default();
        let recruitment = RecruitmentState::initialize(
            self.run_seed,
            &self.game_data.employee_definition_data.definitions,
            &employee_definition_ids,
        )?;
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &self.state.roster,
        )?;
        let (run_map, progression, facility_run_state) = self.generate_current_floor_map(
            &run_progression,
            &crate::game::map::FacilityRunState::default(),
            &mut encounter_progression,
        );
        if run_progression.game_mode == GameMode::Expedition {
            self.state
                .player_profile
                .record_floor_reached(run_progression.floor_index().saturating_add(1));
        }
        self.state.run = Some(
            RunState::new(run_map, progression, run_progression)
                .with_recruitment(recruitment)
                .with_encounter_progression(encounter_progression)
                .with_facility_run_state(facility_run_state),
        );
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.starter_candidates.clear();
        self.transition_to(GameState::ViewingMap)?;

        let map = self.current_map_view()?;
        Ok(BehaviorResult::StarterEmployeesSelected {
            selected_employee_definition_ids: employee_definition_ids,
            employee_uuids,
            map,
        })
    }
}

// ============================================================
// 테스트 헬퍼 메서드들
// ============================================================

impl GameCore {
    /// 현재 게임 상태 조회
    ///
    /// # Returns
    /// 현재 GameState의 복사본
    pub fn get_state(&self) -> GameState {
        self.state.lifecycle.snapshot()
    }

    /// 현재 Enkephalin 양 조회
    ///
    /// # Returns
    /// 현재 Enkephalin 양. Resource가 없으면 0 반환
    #[cfg(test)]
    pub fn get_enkephalin(&self) -> u32 {
        self.state.enkephalin.amount
    }

    /// Enkephalin 양 설정 (테스트 헬퍼)
    #[cfg(test)]
    pub fn set_enkephalin(&mut self, amount: u32) {
        self.execute_application(
            application::ApplicationRoute::SetEnkephalinFixture,
            move |draft| {
                draft.state.enkephalin.amount = amount;
                Ok(())
            },
        )
        .expect("the private lifecycle context must remain coherent");
    }

    pub fn game_state_name(&self) -> &'static str {
        match self.get_state() {
            GameState::NotStarted => "not_started",
            GameState::PreRunBriefing { .. } => "pre_run_briefing",
            GameState::SelectingStarterEmployees => "selecting_starter_employees",
            GameState::ViewingMap => "viewing_map",
            GameState::NodeConfirm { .. } => "node_confirm",
            GameState::InNode { .. } => "in_node",
            GameState::InShop { .. } => "in_shop",
            GameState::InReward { .. } => "in_reward",
            GameState::InRewardClaimed { .. } => "in_reward_claimed",
            GameState::CombatResult { .. } => "combat_result",
            GameState::MasteredEliteSweepResult { .. } => "mastered_elite_sweep_result",
            GameState::InBattle { .. } => "in_battle",
            GameState::AwaitingRunFailureDecision { .. } => "awaiting_run_failure_decision",
            GameState::GameOver => "game_over",
            GameState::RunComplete => "run_complete",
            GameState::RunFailed { .. } => "run_failed",
        }
    }
}

#[cfg(test)]
mod tests;
