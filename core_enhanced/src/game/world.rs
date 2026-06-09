use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::game::behavior::{ActionKind, BehaviorResult, GameError, PlayerBehavior, RosterSlotDto};
use crate::game::data::GameDataBase;
use crate::game::enums::{RewardAction, ShopAction};
use crate::game::map::RunProgression;
use crate::game::resources::{ActiveNodeContent, GameState, Qliphoth};

mod admin;
mod combat;
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

use state::{GameCoreState, LiveBattleDeploymentPolicy, RunState};

pub use admin::{AdminCommand, AdminCommandOutput};

pub struct GameCore {
    state: GameCoreState,
    game_data: Arc<GameDataBase>,
    run_seed: u64,
}

const ROSTER_ORDER_SLOTS: usize = 8;

#[derive(Debug, Clone, Copy)]
struct RunSystemPolicy {
    setup: RunSetupPolicy,
    live_deployment: LiveBattleDeploymentPolicy,
    post_battle: PostBattleResolutionPolicy,
    support: SupportPolicy,
    headquarters: HeadquartersPolicy,
}

#[derive(Debug, Clone, Copy)]
struct RunSetupPolicy {
    default_max_acts: u8,
    starter_employee_count: usize,
    starter_enkephalin: u32,
}

#[derive(Debug, Clone, Copy)]
struct PostBattleResolutionPolicy {
    survival_xp: u32,
    incapacitation_trauma: u32,
    incapacitation_run_hp_loss_percent: u32,
}

#[derive(Debug, Clone, Copy)]
struct SupportPolicy {
    medical_hp_heal_percent: u32,
    medical_trauma_heal: u32,
    medical_balanced_hp_heal_percent: u32,
    rest_trauma_heal: u32,
}

#[derive(Debug, Clone, Copy)]
struct HeadquartersPolicy {
    emergency_enkephalin: u32,
}

const RUN_SYSTEM_POLICY: RunSystemPolicy = RunSystemPolicy {
    setup: RunSetupPolicy {
        default_max_acts: 3,
        starter_employee_count: 3,
        starter_enkephalin: 500,
    },
    live_deployment: LiveBattleDeploymentPolicy {
        initial_cost: 20,
        max_cost: 99,
        cost_per_second: 1,
        base_deploy_cost: 10,
        redeploy_cooldown_ms: 30_000,
        redeploy_cost_multiplier_pct: 150,
        first_instance_salt: 10_000,
    },
    post_battle: PostBattleResolutionPolicy {
        survival_xp: 10,
        incapacitation_trauma: 40,
        incapacitation_run_hp_loss_percent: 25,
    },
    support: SupportPolicy {
        medical_hp_heal_percent: 50,
        medical_trauma_heal: 40,
        medical_balanced_hp_heal_percent: 25,
        rest_trauma_heal: 10,
    },
    headquarters: HeadquartersPolicy {
        emergency_enkephalin: 120,
    },
};

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

    pub fn execute(
        &mut self,
        player_id: Uuid,
        behavior: PlayerBehavior,
    ) -> Result<BehaviorResult, GameError> {
        debug!("Executing behavior {:?} for player {}", behavior, player_id);
        let action_kind = behavior.kind();

        // 1. 상태 기반 액션 게이트
        if !self.state.action_validator.is_kind_allowed(action_kind) {
            warn!(
                "Rejected behavior {:?} for player {} (action kind {:?} not allowed in current state)",
                behavior, player_id, action_kind
            );
            return Err(GameError::InvalidAction);
        }

        // 2. payload 검증
        self.validate_behavior_payload(&behavior)?;

        // 3. 행동 처리
        let result = match behavior {
            // 복잡한 행동
            PlayerBehavior::StartNewGame => self.handle_start_new_game(player_id),
            PlayerBehavior::SelectStarterEmployees { candidate_ids } => {
                self.handle_select_starter_employees(candidate_ids)
            }
            PlayerBehavior::RequestMapData => self.handle_request_map_data(),
            PlayerBehavior::SelectMapNode { node_id } => self.handle_select_map_node(node_id),
            PlayerBehavior::ConfirmEnterNode => self.handle_confirm_enter_node(),
            PlayerBehavior::CancelSelectedNode => self.handle_cancel_selected_node(),
            PlayerBehavior::CompleteNode => self.handle_complete_node(),
            PlayerBehavior::ChooseSupport { support_type } => {
                self.handle_choose_support(support_type)
            }
            PlayerBehavior::SelectSupportTarget { employee_uuid } => {
                self.handle_select_support_target(employee_uuid)
            }
            PlayerBehavior::SelectMedicalTreatment { treatment } => {
                self.handle_select_medical_treatment(treatment)
            }
            PlayerBehavior::RecruitEmployee { candidate_id } => {
                self.handle_recruit_employee(&candidate_id)
            }
            PlayerBehavior::RequestEmergencySupplies => self.handle_request_emergency_supplies(),
            PlayerBehavior::OpenHeadquartersShop => self.handle_open_headquarters_shop(),
            PlayerBehavior::SelectReward { reward_id } => self.handle_select_reward(reward_id),
            PlayerBehavior::EquipItem {
                item_uuid,
                target_unit,
            } => self.handle_equip_item(item_uuid, target_unit),
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
            PlayerBehavior::UnEquipItem {
                item_uuid,
                target_unit,
            } => self.handle_unequip_item(item_uuid, target_unit),
            PlayerBehavior::MoveRosterUnit {
                target_unit_uuid,
                dest_slot,
                swap_with_unit_uuid,
            } => self.handle_move_roster_unit(target_unit_uuid, dest_slot, swap_with_unit_uuid),

            // 상점 관련 행동
            PlayerBehavior::PurchaseItem { item_uuid } => {
                self.execute_shop_action(ShopAction::Purchase { item_uuid })
            }
            PlayerBehavior::SellItem { item_uuid } => {
                self.execute_shop_action(ShopAction::Sell { item_uuid })
            }
            PlayerBehavior::RerollShop => self.execute_shop_action(ShopAction::Reroll),
            PlayerBehavior::ExitShop => self.execute_shop_action(ShopAction::Exit),

            // 보너스 관련 행동
            PlayerBehavior::ClaimReward => self.execute_reward_action(RewardAction::Claim),
            PlayerBehavior::ExitReward => self.execute_reward_action(RewardAction::Exit),

            PlayerBehavior::CompleteCombatResult => self.handle_complete_combat_result(),
            PlayerBehavior::RequestBattleState { since_seq } => {
                self.handle_request_battle_state(since_seq)
            }
            PlayerBehavior::DeployUnit {
                employee_uuid,
                position,
                facing,
            } => self.handle_deploy_unit(employee_uuid, position, facing),
            PlayerBehavior::WithdrawUnit { employee_uuid } => {
                self.handle_withdraw_unit(employee_uuid)
            }
            PlayerBehavior::ActivateSkill {
                employee_uuid,
                skill_id,
                target,
            } => self.handle_activate_skill(employee_uuid, skill_id, target),
            PlayerBehavior::RetreatBattle => self.handle_retreat_battle(),
            PlayerBehavior::PauseBattle => self.handle_pause_battle(),
            PlayerBehavior::ResumeBattle => self.handle_resume_battle(),
            PlayerBehavior::SetBattleSpeed { speed } => self.handle_set_battle_speed(speed),
        }?;

        self.sync_roster_order_with_owned_units()?;
        Ok(result)
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
    fn handle_start_new_game(&mut self, player_id: Uuid) -> Result<BehaviorResult, GameError> {
        // 플레이어 생성
        self.initial_player(player_id);
        self.state.starter_candidates = self.game_data.starter_employee_data.candidates.clone();
        if self.state.starter_candidates.len() < RUN_SYSTEM_POLICY.setup.starter_employee_count {
            return Err(GameError::InvalidStaticData(format!(
                "starter employee candidate data must contain at least {} candidates, got {}",
                RUN_SYSTEM_POLICY.setup.starter_employee_count,
                self.state.starter_candidates.len()
            )));
        }
        self.state.roster.clear();
        self.state.run = None;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::SelectingStarterEmployees)?;

        info!("Starter employee selection opened for player {}", player_id);

        Ok(BehaviorResult::StartNewGame {
            candidates: self.state.starter_candidates.clone(),
            required_count: RUN_SYSTEM_POLICY.setup.starter_employee_count,
        })
    }

    fn handle_select_starter_employees(
        &mut self,
        candidate_ids: Vec<String>,
    ) -> Result<BehaviorResult, GameError> {
        let employee_uuids = self.initialize_selected_starter_employees(&candidate_ids)?;
        self.sync_roster_order_with_owned_units()?;

        // 기초 자원 지급: 첫 안전 노드에서 상점에 들어가도 하나는 살 수 있도록 여유 있게.
        self.state.enkephalin.amount = self
            .state
            .enkephalin
            .amount
            .saturating_add(RUN_SYSTEM_POLICY.setup.starter_enkephalin);
        info!(
            "Granted starter enkephalin: amount={}, total={}",
            RUN_SYSTEM_POLICY.setup.starter_enkephalin, self.state.enkephalin.amount
        );

        let run_progression =
            RunProgression::new(self.run_seed, RUN_SYSTEM_POLICY.setup.default_max_acts);
        let (run_map, progression) = self.generate_current_act_map(&run_progression);
        self.state.run = Some(RunState::new(run_map, progression, run_progression));
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

    pub fn get_qliphoth(&self) -> Result<Qliphoth, GameError> {
        Ok(self.state.qliphoth.clone())
    }

    /// 현재 허용된 액션 capability 목록 조회
    ///
    /// # Returns
    /// 현재 허용된 ActionKind 목록
    pub fn get_allowed_actions(&self) -> Vec<ActionKind> {
        self.state.action_validator.allowed_actions()
    }

    /// 특정 행동 종류가 허용되는지 확인
    ///
    /// # Arguments
    /// * `action` - 확인할 행동
    ///
    /// # Returns
    /// payload와 무관한 capability가 허용되면 true, 아니면 false
    pub fn is_action_allowed(&self, action: &PlayerBehavior) -> bool {
        self.state.action_validator.is_action_allowed(action)
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

    pub fn qliphoth_level_name(
        &self,
        level: crate::game::resources::QliphothLevel,
    ) -> &'static str {
        match level {
            crate::game::resources::QliphothLevel::Stable => "stable",
            crate::game::resources::QliphothLevel::Caution => "caution",
            crate::game::resources::QliphothLevel::Critical => "critical",
            crate::game::resources::QliphothLevel::Meltdown => "meltdown",
        }
    }

    pub fn get_combat_result_event_log(
        &self,
    ) -> Option<(
        crate::game::battle::types::BattleWinner,
        crate::game::battle::timeline::Timeline,
    )> {
        self.state
            .active_node_content
            .as_ref()
            .and_then(|selected| match selected {
                ActiveNodeContent::CombatBattle(battle) => {
                    Some((battle.winner, battle.timeline.clone()))
                }
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests;
