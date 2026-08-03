use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::behavior::{
    ActionKind, BehaviorResult, CombatResultEventLogAttachmentDto, GameError, PlayerBehavior,
};
use crate::game::data::visual_novel_data::VisualNovelPresentationMode;
use crate::game::data::{run_policy_data::RunPolicyData, GameDataBase};
use crate::game::encounter_compendium::EncounterCompendiumProfileState;
use crate::game::enums::{RewardAction, ShopAction};
use crate::game::map::{GameMode, RunProgression};
use crate::game::recruitment::RecruitmentState;
use crate::game::resources::{ActiveNodeContent, GameState};
use crate::game::visual_novel::{
    VisualNovelRuntimeState, VisualNovelSessionPresentationMode, VisualNovelTriggerRuntimeState,
};

mod action_capability;
mod active_run;
mod admin;
mod battle_capability;
mod combat;
mod combat_trace;
mod event_node;
mod facility;
mod headquarters;
mod helpers;
mod maintenance;
mod map_content;
mod map_encounters;
mod node_flow;
mod reward;
mod shop;
mod snapshot;
mod state;
mod visual_novel_flow;
mod visual_novel_triggers;

use state::{GameCoreState, RunState};

pub use active_run::{ActiveRunCheckpointExport, ActiveRunTransactionRollback};
pub use admin::{AdminCommand, AdminCommandOutput};

pub struct GameCore {
    state: GameCoreState,
    game_data: Arc<GameDataBase>,
    run_seed: u64,
}

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
    /// GameCore 생성
    ///
    /// # Arguments
    /// * `game_data` - game_server에서 로드한 게임 데이터 (Arc로 공유)
    pub fn new(game_data: Arc<GameDataBase>, run_seed: u64) -> Self {
        info!("Initializing GameCore with run_seed={}", run_seed);

        debug!("GameCore state initialized with default resources");

        Self {
            state: GameCoreState::new(run_seed),
            game_data,
            run_seed,
        }
    }

    pub fn new_with_encounter_compendium_profile(
        game_data: Arc<GameDataBase>,
        run_seed: u64,
        profile: EncounterCompendiumProfileState,
    ) -> Result<Self, GameError> {
        let mut core = Self::new(game_data, run_seed);
        core.merge_encounter_compendium_profile(profile)?;
        Ok(core)
    }

    pub fn encounter_compendium_profile(&self) -> EncounterCompendiumProfileState {
        self.state.encounter_compendium.clone()
    }

    pub fn reset_encounter_compendium_profile(&mut self) {
        self.state.encounter_compendium = EncounterCompendiumProfileState::default();
    }

    pub fn merge_encounter_compendium_profile(
        &mut self,
        profile: EncounterCompendiumProfileState,
    ) -> Result<(), GameError> {
        profile
            .validate_persisted()
            .map_err(GameError::InvalidStaticData)?;
        self.state.encounter_compendium.merge(profile);
        Ok(())
    }

    pub(super) fn run_policy(&self) -> &RunPolicyData {
        &self.game_data.run_policy
    }

    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        self.execute_with_source_command_id(player_id, behavior, None)
    }

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

    pub fn execute_with_transport_context(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
        expected_battle_authority: Option<crate::game::behavior::BattleStateAuthorityDto>,
    ) -> Result<BehaviorResult, GameError> {
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        let action_kind = behavior.kind();

        self.validate_battle_command_authority(&behavior, expected_battle_authority)?;

        self.gate_behavior_action(player_id, &behavior, action_kind)?;
        let mut result = self.execute_validated_behavior_domain(
            player_id,
            behavior,
            source_command_id,
            battle_event_after_seq,
        )?;
        if matches!(result, BehaviorResult::VisualNovelState { .. })
            && matches!(self.state.game_state, GameState::InBattle { .. })
        {
            result = self
                .current_battle_update_result_for_visual_novel_transport(battle_event_after_seq)?;
        }
        self.attach_visual_novel_to_battle_result(&mut result)?;
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
        if expected == current {
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

    fn execute_validated_behavior_domain(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
        battle_event_after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::StartNewGame { .. }
            | PlayerBehavior::ContinueRun
            | PlayerBehavior::ResetActiveRun
            | PlayerBehavior::RetryEncounterCompendium
            | PlayerBehavior::ResetEncounterCompendium
            | PlayerBehavior::SelectStarterEmployees { .. }
            | PlayerBehavior::RequestMapData
            | PlayerBehavior::SelectMapNode { .. }
            | PlayerBehavior::ConvertMapNodeWithMonolith { .. }
            | PlayerBehavior::ConfirmEnterNode
            | PlayerBehavior::CancelSelectedNode
            | PlayerBehavior::CompleteNode
            | PlayerBehavior::LoadRunCheckpoint
            | PlayerBehavior::ConfirmRunFailure
            | PlayerBehavior::AbandonRun => self.execute_map_run_behavior(player_id, behavior),

            PlayerBehavior::RecruitEmployee { .. }
            | PlayerBehavior::RequestEmergencySupplies
            | PlayerBehavior::OpenHeadquartersShop => self.execute_headquarters_behavior(behavior),

            PlayerBehavior::EquipItem { .. }
            | PlayerBehavior::UnEquipItem { .. }
            | PlayerBehavior::UseConsumableItem { .. }
            | PlayerBehavior::EquipSkillFragment { .. }
            | PlayerBehavior::UnequipSkillFragment { .. }
            | PlayerBehavior::UpgradeSkillFragment { .. }
            | PlayerBehavior::AwakenSkillFragment { .. }
            | PlayerBehavior::DismantleSkillFragment { .. }
            | PlayerBehavior::DismantleEquipment { .. }
            | PlayerBehavior::EnhanceEquipment { .. } => {
                self.execute_equipment_maintenance_behavior(behavior)
            }

            PlayerBehavior::PurchaseItem { .. }
            | PlayerBehavior::SellItem { .. }
            | PlayerBehavior::RefreshShop
            | PlayerBehavior::ExitShop => self.execute_shop_behavior(behavior),

            PlayerBehavior::SelectReward { .. }
            | PlayerBehavior::ClaimReward
            | PlayerBehavior::ExitReward => self.execute_reward_behavior(behavior),

            PlayerBehavior::AdvanceEventScene { .. }
            | PlayerBehavior::ResolveEventChoiceCheck { .. } => {
                self.execute_event_behavior(behavior)
            }

            PlayerBehavior::AdvanceVisualNovel { .. }
            | PlayerBehavior::SelectVisualNovelChoice { .. } => {
                if matches!(self.state.game_state, GameState::InBattle { .. }) {
                    self.state
                        .active_battle
                        .as_ref()
                        .ok_or(GameError::InvalidAction)?
                        .ensure_battle_projection_commit_available()?;
                }
                self.execute_visual_novel_behavior(behavior)
            }

            PlayerBehavior::CompleteCombatResult
            | PlayerBehavior::RequestBattleState { .. }
            | PlayerBehavior::RecoverBattleSetupLoss
            | PlayerBehavior::RequestDeploymentRangePreview { .. }
            | PlayerBehavior::DeployUnit { .. }
            | PlayerBehavior::WithdrawUnit { .. }
            | PlayerBehavior::ActivateSkill { .. }
            | PlayerBehavior::RetreatBattle
            | PlayerBehavior::PauseBattle
            | PlayerBehavior::ResumeBattle
            | PlayerBehavior::SetBattleSpeed { .. } => {
                self.execute_battle_behavior(behavior, source_command_id, battle_event_after_seq)
            }
        }
    }

    fn execute_map_run_behavior(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::StartNewGame { game_mode, .. } => {
                self.handle_start_new_game(player_id, game_mode)
            }
            PlayerBehavior::ContinueRun => Err(GameError::InvalidAction),
            PlayerBehavior::ResetActiveRun => Err(GameError::InvalidAction),
            PlayerBehavior::RetryEncounterCompendium => Err(GameError::InvalidAction),
            PlayerBehavior::ResetEncounterCompendium => Err(GameError::InvalidAction),
            PlayerBehavior::SelectStarterEmployees {
                employee_definition_ids,
            } => {
                self.validate_starter_employee_selection(&employee_definition_ids)?;
                self.handle_select_starter_employees(employee_definition_ids)
            }
            PlayerBehavior::RequestMapData => self.handle_request_map_data(),
            PlayerBehavior::SelectMapNode { node_id } => {
                self.validate_select_map_node(node_id)?;
                self.handle_select_map_node(node_id)
            }
            PlayerBehavior::ConvertMapNodeWithMonolith {
                node_id,
                abnormality_id,
            } => self.handle_convert_map_node_with_monolith(node_id, &abnormality_id),
            PlayerBehavior::ConfirmEnterNode => self.handle_confirm_enter_node(),
            PlayerBehavior::CancelSelectedNode => self.handle_cancel_selected_node(),
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
                fragment_id,
            } => self.handle_equip_skill_fragment(employee_uuid, &fragment_id),
            PlayerBehavior::UnequipSkillFragment {
                employee_uuid,
                fragment_id,
            } => self.handle_unequip_skill_fragment(employee_uuid, &fragment_id),
            PlayerBehavior::UpgradeSkillFragment { target_fragment_id } => {
                self.handle_upgrade_skill_fragment(&target_fragment_id)
            }
            PlayerBehavior::AwakenSkillFragment { target_fragment_id } => {
                self.handle_awaken_skill_fragment(&target_fragment_id)
            }
            PlayerBehavior::DismantleSkillFragment { fragment_id } => {
                self.handle_dismantle_skill_fragment(&fragment_id)
            }
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
            PlayerBehavior::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            } => self.handle_activate_skill(
                employee_uuid,
                skill_id,
                target,
                source_command_id,
                battle_event_after_seq,
            ),
            PlayerBehavior::RetreatBattle => self.handle_retreat_battle(),
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
        self.initial_player(player_id);
        self.state.roster.clear();
        self.state.starter_candidates.clear();
        self.state.run = None;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        self.state.run_checkpoint = Default::default();
        self.state.visual_novel = VisualNovelRuntimeState::default();
        self.state.visual_novel_triggers = VisualNovelTriggerRuntimeState::default();
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

        Ok(BehaviorResult::RunStartFlowStarted {
            game_mode,
            visual_novel: self.visual_novel_snapshot_dto()?,
        })
    }

    fn handle_abandon_run(&mut self) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_none() {
            return Err(GameError::InvalidAction);
        }
        let player = self.state.player.clone();
        let encounter_compendium = self.state.encounter_compendium.clone();
        let mut next = GameCoreState::new(self.run_seed);
        next.player = player;
        next.encounter_compendium = encounter_compendium;
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
    ) -> Result<BehaviorResult, GameError> {
        let setup_policy = self.run_policy().setup;
        let employee_uuids =
            self.initialize_selected_starter_employees(&employee_definition_ids)?;
        // 기초 자원 지급: 첫 안전 노드에서 상점에 들어가도 하나는 살 수 있도록 여유 있게.
        self.state
            .enkephalin
            .checked_add(setup_policy.starter_enkephalin)?;
        info!(
            "Granted starter enkephalin: amount={}, total={}",
            setup_policy.starter_enkephalin, self.state.enkephalin.amount
        );

        let game_mode = self
            .state
            .pending_game_mode
            .take()
            .ok_or(GameError::InvalidAction)?;
        let run_progression =
            RunProgression::new(self.run_seed, game_mode, setup_policy.standard_floor_count);
        let abnormality_research = RunAbnormalityResearchState::initialize(self.game_data.as_ref());
        let abnormality_encounter_history = RunAbnormalityEncounterHistory::default();
        let monolith = crate::game::monolith::MonolithRunState::default();
        let recruitment = RecruitmentState::initialize(
            self.run_seed,
            &self.game_data.employee_definition_data.definitions,
            &employee_definition_ids,
        )?;
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &self.state.roster,
        )?;
        let mut boss_omen = crate::game::boss_omen::BossOmenRunState::default();
        let (run_map, progression, facility_run_state) = self.generate_current_floor_map(
            &run_progression,
            &crate::game::map::FacilityRunState::default(),
            &abnormality_research,
            &abnormality_encounter_history,
            &monolith,
            &mut boss_omen,
        );
        self.state.run = Some(
            RunState::new(run_map, progression, run_progression)
                .with_abnormality_research(abnormality_research)
                .with_abnormality_encounter_history(abnormality_encounter_history)
                .with_recruitment(recruitment)
                .with_boss_omen(boss_omen)
                .with_facility_run_state(facility_run_state),
        );
        self.state.node_session = None;
        self.state.active_node_content = None;
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
        self.state.game_state.clone()
    }

    /// 현재 Enkephalin 양 조회
    ///
    /// # Returns
    /// 현재 Enkephalin 양. Resource가 없으면 0 반환
    pub fn get_enkephalin(&self) -> u32 {
        self.state.enkephalin.amount
    }

    /// Enkephalin 양 설정 (테스트 헬퍼)
    pub fn set_enkephalin(&mut self, amount: u32) {
        self.state.enkephalin.amount = amount;
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
            GameState::InBattle { .. } => "in_battle",
            GameState::AwaitingRunFailureDecision { .. } => "awaiting_run_failure_decision",
            GameState::GameOver => "game_over",
            GameState::RunComplete => "run_complete",
            GameState::RunFailed { .. } => "run_failed",
        }
    }

    pub fn get_combat_result_event_log_attachment(
        &self,
    ) -> Option<CombatResultEventLogAttachmentDto> {
        self.state
            .active_node_content
            .as_ref()
            .and_then(|selected| match selected {
                ActiveNodeContent::CombatBattle(battle) => {
                    Some(CombatResultEventLogAttachmentDto {
                        winner: battle.winner,
                        event_log: battle.event_log.clone(),
                    })
                }
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests;
