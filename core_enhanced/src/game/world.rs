use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::behavior::{
    ActionKind, BehaviorResult, CombatResultEventLogAttachmentDto, GameError, PlayerBehavior,
    RosterSlotDto,
};
use crate::game::data::{run_policy_data::RunPolicyData, GameDataBase};
use crate::game::enums::{RewardAction, ShopAction};
use crate::game::map::{GameMode, RunProgression};
use crate::game::resources::{ActiveNodeContent, GameState};

mod admin;
mod combat;
mod event_node;
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
mod support;

use state::{GameCoreState, RunState};

pub use admin::{AdminCommand, AdminCommandOutput};

pub struct GameCore {
    state: GameCoreState,
    game_data: Arc<GameDataBase>,
    run_seed: u64,
}

const ROSTER_ORDER_SLOTS: usize = 8;

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
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        let action_kind = behavior.kind();

        self.gate_behavior_action(player_id, &behavior, action_kind)?;
        let result =
            self.execute_validated_behavior_domain(player_id, behavior, source_command_id)?;
        self.postprocess_behavior_execution()?;
        Ok(result)
    }

    fn gate_behavior_action(
        &self,
        player_id: Uuid,
        behavior: &PlayerBehavior,
        action_kind: ActionKind,
    ) -> Result<(), GameError> {
        if !self.get_allowed_actions().contains(&action_kind) {
            warn!(
                "Rejected behavior {:?} for player {} (action kind {:?} not allowed in current state)",
                behavior, player_id, action_kind
            );
            return Err(GameError::InvalidAction);
        }
        Ok(())
    }

    fn execute_validated_behavior_domain(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::StartNewGame { .. }
            | PlayerBehavior::SelectStarterEmployees { .. }
            | PlayerBehavior::RequestMapData
            | PlayerBehavior::SelectMapNode { .. }
            | PlayerBehavior::ConfirmEnterNode
            | PlayerBehavior::CancelSelectedNode
            | PlayerBehavior::CompleteNode
            | PlayerBehavior::LoadRunCheckpoint => {
                self.execute_map_run_behavior(player_id, behavior)
            }

            PlayerBehavior::ChooseSupport { .. }
            | PlayerBehavior::RecruitEmployee { .. }
            | PlayerBehavior::RequestEmergencySupplies
            | PlayerBehavior::OpenHeadquartersShop => {
                self.execute_support_headquarters_behavior(behavior)
            }

            PlayerBehavior::EquipItem { .. }
            | PlayerBehavior::UnEquipItem { .. }
            | PlayerBehavior::UseConsumableItem { .. }
            | PlayerBehavior::EquipSkillFragment { .. }
            | PlayerBehavior::UnequipSkillFragment { .. }
            | PlayerBehavior::UpgradeSkillFragment { .. }
            | PlayerBehavior::AwakenSkillFragment { .. }
            | PlayerBehavior::DismantleSkillFragment { .. }
            | PlayerBehavior::DismantleEquipment { .. }
            | PlayerBehavior::EnhanceEquipment { .. }
            | PlayerBehavior::MoveRosterUnit { .. } => {
                self.execute_equipment_maintenance_behavior(behavior)
            }

            PlayerBehavior::PurchaseItem { .. }
            | PlayerBehavior::SellItem { .. }
            | PlayerBehavior::RerollShop
            | PlayerBehavior::ExitShop => self.execute_shop_behavior(behavior),

            PlayerBehavior::SelectReward { .. }
            | PlayerBehavior::ClaimReward
            | PlayerBehavior::ExitReward => self.execute_reward_behavior(behavior),

            PlayerBehavior::AdvanceEventScene { .. } | PlayerBehavior::SelectEventChoice { .. } => {
                self.execute_event_behavior(behavior)
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
                self.execute_battle_behavior(behavior, source_command_id)
            }
        }
    }

    fn postprocess_behavior_execution(&mut self) -> Result<(), GameError> {
        self.sync_roster_order_with_owned_units()?;
        Ok(())
    }

    fn execute_map_run_behavior(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::StartNewGame { game_mode } => {
                self.handle_start_new_game(player_id, game_mode)
            }
            PlayerBehavior::SelectStarterEmployees { candidate_ids } => {
                self.validate_starter_employee_selection(&candidate_ids)?;
                self.handle_select_starter_employees(candidate_ids)
            }
            PlayerBehavior::RequestMapData => self.handle_request_map_data(),
            PlayerBehavior::SelectMapNode { node_id } => {
                self.validate_select_map_node(node_id)?;
                self.handle_select_map_node(node_id)
            }
            PlayerBehavior::ConfirmEnterNode => self.handle_confirm_enter_node(),
            PlayerBehavior::CancelSelectedNode => self.handle_cancel_selected_node(),
            PlayerBehavior::CompleteNode => self.handle_complete_node(),
            PlayerBehavior::LoadRunCheckpoint => self.handle_load_run_checkpoint(),
            _ => unreachable!("behavior routed to map/run domain incorrectly"),
        }
    }

    fn execute_support_headquarters_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::ChooseSupport { support_type } => {
                self.handle_choose_support(support_type)
            }
            PlayerBehavior::RecruitEmployee { candidate_id } => {
                self.handle_recruit_employee(&candidate_id)
            }
            PlayerBehavior::RequestEmergencySupplies => self.handle_request_emergency_supplies(),
            PlayerBehavior::OpenHeadquartersShop => self.handle_open_headquarters_shop(),
            _ => unreachable!("behavior routed to support/headquarters domain incorrectly"),
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
            PlayerBehavior::MoveRosterUnit {
                target_unit_uuid,
                dest_slot,
                swap_with_unit_uuid,
            } => self.handle_move_roster_unit(target_unit_uuid, dest_slot, swap_with_unit_uuid),
            _ => unreachable!("behavior routed to equipment/maintenance domain incorrectly"),
        }
    }

    fn execute_shop_behavior(
        &mut self,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::PurchaseItem { item_uuid } => {
                self.execute_shop_action(ShopAction::Purchase { item_uuid })
            }
            PlayerBehavior::SellItem { item_uuid } => {
                self.execute_shop_action(ShopAction::Sell { item_uuid })
            }
            PlayerBehavior::RerollShop => self.execute_shop_action(ShopAction::Reroll),
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
            PlayerBehavior::SelectEventChoice {
                node_id,
                event_id,
                choice_id,
            } => self.handle_select_event_choice(node_id, event_id, choice_id),
            _ => unreachable!("behavior routed to event domain incorrectly"),
        }
    }

    fn execute_battle_behavior(
        &mut self,
        behavior: PlayerBehavior,
        source_command_id: Option<&str>,
    ) -> Result<BehaviorResult, GameError> {
        match behavior {
            PlayerBehavior::CompleteCombatResult => self.handle_complete_combat_result(),
            PlayerBehavior::RequestBattleState { since_seq } => {
                self.handle_request_battle_state(since_seq)
            }
            PlayerBehavior::RecoverBattleSetupLoss => self.handle_recover_battle_setup_loss(),
            PlayerBehavior::RequestDeploymentRangePreview {
                employee_uuid,
                position,
            } => self.handle_request_deployment_range_preview(employee_uuid, position),
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing,
            } => self.handle_deploy_unit(employee_uuid, position, facing, source_command_id),
            PlayerBehavior::WithdrawUnit { employee_uuid } => {
                self.handle_withdraw_unit(employee_uuid, source_command_id)
            }
            PlayerBehavior::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            } => self.handle_activate_skill(employee_uuid, skill_id, target, source_command_id),
            PlayerBehavior::RetreatBattle => self.handle_retreat_battle(),
            PlayerBehavior::PauseBattle => self.handle_pause_battle(),
            PlayerBehavior::ResumeBattle => self.handle_resume_battle(),
            PlayerBehavior::SetBattleSpeed { speed } => self.handle_set_battle_speed(speed),
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
        let setup_policy = self.run_policy().setup;
        // 플레이어 생성
        self.initial_player(player_id);
        self.state.starter_candidates = self.game_data.starter_employee_data.candidates.clone();
        if self.state.starter_candidates.len() < setup_policy.starter_employee_count {
            return Err(GameError::InvalidStaticData(format!(
                "starter employee candidate data must contain at least {} candidates, got {}",
                setup_policy.starter_employee_count,
                self.state.starter_candidates.len()
            )));
        }
        self.state.roster.clear();
        self.state.run = None;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.pending_game_mode = Some(game_mode);
        self.transition_to(GameState::SelectingStarterEmployees)?;

        info!(
            "Starter employee selection opened for player {} in {:?} mode",
            player_id, game_mode
        );

        Ok(BehaviorResult::StartNewGame {
            game_mode,
            candidates: self.state.starter_candidates.clone(),
            required_count: setup_policy.starter_employee_count,
        })
    }

    fn handle_select_starter_employees(
        &mut self,
        candidate_ids: Vec<String>,
    ) -> Result<BehaviorResult, GameError> {
        let setup_policy = self.run_policy().setup;
        let employee_uuids = self.initialize_selected_starter_employees(&candidate_ids)?;
        self.sync_roster_order_with_owned_units()?;

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
        let mut boss_omen = crate::game::boss_omen::BossOmenRunState::default();
        let (run_map, progression) = self.generate_current_floor_map(
            &run_progression,
            &abnormality_research,
            &abnormality_encounter_history,
            &mut boss_omen,
        );
        self.state.run = Some(
            RunState::new(run_map, progression, run_progression)
                .with_abnormality_research(abnormality_research)
                .with_abnormality_encounter_history(abnormality_encounter_history)
                .with_boss_omen(boss_omen),
        );
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;

        let map = self.current_map_view()?;
        Ok(BehaviorResult::StarterEmployeesSelected {
            selected_candidate_ids: candidate_ids,
            employee_uuids,
            map,
        })
    }

    fn handle_move_roster_unit(
        &mut self,
        target_unit_uuid: Uuid,
        dest_slot: usize,
        swap_with_unit_uuid: Option<Uuid>,
    ) -> Result<BehaviorResult, GameError> {
        self.sync_roster_order_with_owned_units()?;
        self.validate_owned_unit_exists(target_unit_uuid)?;
        if let Some(swap_unit_uuid) = swap_with_unit_uuid {
            self.validate_owned_unit_exists(swap_unit_uuid)?;
        }

        let roster_order = self.roster_order_mut()?;
        roster_order.move_unit(target_unit_uuid, dest_slot, swap_with_unit_uuid)?;

        let roster_slots = roster_order
            .slots
            .iter()
            .enumerate()
            .map(|(slot, unit_uuid)| RosterSlotDto {
                slot,
                unit_uuid: *unit_uuid,
            })
            .collect();

        Ok(BehaviorResult::MoveRosterUnit { roster_slots })
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

    /// 현재 허용된 액션 capability 목록 조회
    ///
    /// # Returns
    /// 현재 허용된 ActionKind 목록
    pub fn get_allowed_actions(&self) -> Vec<ActionKind> {
        self.allowed_actions_for_state_context(&self.state.game_state)
    }

    /// 특정 행동 종류가 허용되는지 확인
    ///
    /// # Arguments
    /// * `action` - 확인할 행동
    ///
    /// # Returns
    /// payload와 무관한 capability가 허용되면 true, 아니면 false
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.get_allowed_actions().contains(&action.kind())
    }

    pub fn game_state_name(&self) -> &'static str {
        match self.get_state() {
            GameState::NotStarted => "not_started",
            GameState::SelectingStarterEmployees => "selecting_starter_employees",
            GameState::ViewingMap => "viewing_map",
            GameState::NodeConfirm { .. } => "node_confirm",
            GameState::InNode { .. } => "in_node",
            GameState::InShop { .. } => "in_shop",
            GameState::InReward { .. } => "in_reward",
            GameState::InRewardClaimed { .. } => "in_reward_claimed",
            GameState::CombatResult { .. } => "combat_result",
            GameState::InBattle { .. } => "in_battle",
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

    pub fn battle_records(&self) -> &[crate::game::resources::CombatBattleState] {
        self.state
            .run
            .as_ref()
            .map(|run| run.battle_records.as_slice())
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests;
