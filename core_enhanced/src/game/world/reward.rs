use tracing::info;
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::employee::EmployeeRoster;
use crate::game::enums::{RewardAction, RewardMode};
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::resources::{Enkephalin, GameState, Inventory, InventoryDiffDto};
use crate::game::reward::{
    GrantExecutionContext, GrantExecutionResult, GrantExecutor, RewardEffect, RewardOption,
};
use crate::game::skill_fragment::SkillFragmentInventory;

impl GameCore {
    pub(super) fn execute_reward_action(
        &mut self,
        action: RewardAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            RewardAction::Claim => self.claim_current_reward_session(),
            RewardAction::Exit => {
                if matches!(self.get_state(), GameState::InReward { .. })
                    && !self.current_reward_can_skip()
                {
                    return Err(GameError::InvalidAction);
                }

                if self.state.node_session.is_some() {
                    return self.handle_complete_node();
                }
                Err(GameError::InvalidAction)
            }
        }
    }

    pub(super) fn claim_current_reward_session(&mut self) -> Result<BehaviorResult, GameError> {
        let reward = {
            let selected = self
                .state
                .active_node_content
                .as_ref()
                .ok_or(GameError::NotInRewardState)?;
            selected.as_reward()?.clone()
        };

        let (
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        ) = self.apply_reward_session(&reward)?;

        self.transition_to(GameState::InRewardClaimed {
            reward_uuid: reward.stage_uuid,
        })?;

        Ok(BehaviorResult::RewardGranted {
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        })
    }

    pub(super) fn apply_reward_session(
        &mut self,
        reward: &crate::game::resources::RewardSessionState,
    ) -> Result<
        (
            u32,
            InventoryDiffDto,
            Vec<crate::game::reward::SkillFragmentGrantDiffDto>,
            Vec<crate::game::reward::SkillFragmentResearchDiffDto>,
            Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        ),
        GameError,
    > {
        self.apply_reward_session_with_context(reward, &GrantExecutionContext::default())
    }

    pub(super) fn apply_reward_session_with_context(
        &mut self,
        reward: &crate::game::resources::RewardSessionState,
        context: &GrantExecutionContext,
    ) -> Result<
        (
            u32,
            InventoryDiffDto,
            Vec<crate::game::reward::SkillFragmentGrantDiffDto>,
            Vec<crate::game::reward::SkillFragmentResearchDiffDto>,
            Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        ),
        GameError,
    > {
        let rewards_to_apply: Vec<RewardOption> = match reward.mode {
            RewardMode::ClaimAll => reward.rewards.clone(),
            RewardMode::ChooseOne => vec![reward
                .get_selected_reward()
                .cloned()
                .ok_or(GameError::InvalidAction)?],
        };

        let mut next_inventory = self.state.inventory.clone();
        let mut next_skill_fragments = self.state.skill_fragments.clone();
        let mut next_roster = self.state.roster.clone();
        let mut next_uuid_manager = self.state.uuid_manager.clone();
        let mut next_enkephalin = self.state.enkephalin.clone();

        let (
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        ) = self.apply_reward_options_with_context_to_state(
            reward.stage_uuid,
            &rewards_to_apply,
            context,
            &mut next_inventory,
            &mut next_skill_fragments,
            &mut next_roster,
            &mut next_uuid_manager,
            &mut next_enkephalin,
        )?;

        self.state.inventory = next_inventory;
        self.state.skill_fragments = next_skill_fragments;
        self.state.roster = next_roster;
        self.state.uuid_manager = next_uuid_manager;
        self.state.enkephalin = next_enkephalin;

        let enkephalin = self.state.enkephalin.amount;
        Ok((
            enkephalin,
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        ))
    }

    pub(super) fn apply_reward_options_with_context_to_state(
        &self,
        stage_uuid: Uuid,
        rewards_to_apply: &[RewardOption],
        context: &GrantExecutionContext,
        inventory: &mut Inventory,
        skill_fragments: &mut SkillFragmentInventory,
        roster: &mut EmployeeRoster,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
    ) -> Result<
        (
            InventoryDiffDto,
            Vec<crate::game::reward::SkillFragmentGrantDiffDto>,
            Vec<crate::game::reward::SkillFragmentResearchDiffDto>,
            Vec<crate::game::reward::EmployeeExperienceDiffDto>,
        ),
        GameError,
    > {
        let mut inventory_diff = crate::game::resources::InventoryDiffDto::default();
        let mut skill_fragment_diffs = Vec::new();
        let mut skill_fragment_research_diffs = Vec::new();
        let mut employee_experience_diffs = Vec::new();

        for (index, reward_option) in rewards_to_apply.iter().enumerate() {
            info!(
                "Applying reward '{}' (uuid={}) with {} effect(s)",
                reward_option.id,
                reward_option.uuid,
                reward_option.effects.len()
            );
            let seed = self.reward_seed(stage_uuid, reward_option.uuid, index as u64);

            let granted = GrantExecutor::grant_reward_with_state(
                inventory,
                skill_fragments,
                roster,
                uuid_manager,
                enkephalin,
                &self.game_data,
                &self.state.skill_fragment_policy,
                context,
                reward_option,
                seed,
            )?;
            Self::merge_inventory_diff(&mut inventory_diff, granted.inventory_diff);
            skill_fragment_diffs.extend(granted.skill_fragment_diffs);
            skill_fragment_research_diffs.extend(granted.skill_fragment_research_diffs);
            employee_experience_diffs.extend(granted.employee_experience_diffs);
        }

        Ok((
            inventory_diff,
            skill_fragment_diffs,
            skill_fragment_research_diffs,
            employee_experience_diffs,
        ))
    }

    pub(super) fn apply_grant_effects(
        &mut self,
        effects: &[RewardEffect],
    ) -> Result<GrantExecutionResult, GameError> {
        self.apply_grant_effects_with_context(effects, &GrantExecutionContext::default())
    }

    pub(super) fn apply_grant_effects_with_context(
        &mut self,
        effects: &[RewardEffect],
        context: &GrantExecutionContext,
    ) -> Result<GrantExecutionResult, GameError> {
        let mut next_inventory = self.state.inventory.clone();
        let mut next_skill_fragments = self.state.skill_fragments.clone();
        let mut next_roster = self.state.roster.clone();
        let mut next_uuid_manager = self.state.uuid_manager.clone();
        let mut next_enkephalin = self.state.enkephalin.clone();

        let granted = GrantExecutor::grant_effects_with_state(
            &mut next_inventory,
            &mut next_skill_fragments,
            &mut next_roster,
            &mut next_uuid_manager,
            &mut next_enkephalin,
            &self.game_data,
            &self.state.skill_fragment_policy,
            context,
            effects,
            self.run_seed ^ 0x4752_414e_5446_5853,
        )?;

        self.state.inventory = next_inventory;
        self.state.skill_fragments = next_skill_fragments;
        self.state.roster = next_roster;
        self.state.uuid_manager = next_uuid_manager;
        self.state.enkephalin = next_enkephalin;

        Ok(granted)
    }

    pub(super) fn preview_grant_effects(
        &self,
        effects: &[RewardEffect],
    ) -> Result<GrantExecutionResult, GameError> {
        let mut next_inventory = self.state.inventory.clone();
        let mut next_skill_fragments = self.state.skill_fragments.clone();
        let mut next_roster = self.state.roster.clone();
        let mut next_uuid_manager = self.state.uuid_manager.clone();
        let mut next_enkephalin = self.state.enkephalin.clone();

        GrantExecutor::grant_effects_with_state(
            &mut next_inventory,
            &mut next_skill_fragments,
            &mut next_roster,
            &mut next_uuid_manager,
            &mut next_enkephalin,
            &self.game_data,
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            effects,
            self.run_seed ^ 0x4752_414e_5446_5853,
        )
    }

    pub(super) fn handle_select_reward(
        &mut self,
        selected_reward_id: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected = self
            .state
            .active_node_content
            .as_mut()
            .ok_or(GameError::NotInRewardState)?;
        let reward = selected.as_reward_mut()?;

        if reward.mode != RewardMode::ChooseOne {
            return Err(GameError::InvalidAction);
        }

        if !reward
            .rewards
            .iter()
            .any(|reward| reward.uuid == selected_reward_id)
        {
            return Err(GameError::EventNotFound);
        }

        reward.selected_reward_uuid = Some(selected_reward_id);
        Ok(BehaviorResult::RewardState {
            mode: reward.mode,
            rewards: reward.rewards.clone(),
            selected_reward_uuid: reward.selected_reward_uuid,
            research_deliveries: vec![],
        })
    }

    fn reward_seed(&self, stage_uuid: Uuid, reward_uuid: Uuid, index: u64) -> u64 {
        self.run_seed
            ^ u64::from_be_bytes(stage_uuid.as_bytes()[..8].try_into().unwrap())
            ^ u64::from_be_bytes(reward_uuid.as_bytes()[..8].try_into().unwrap())
            ^ index
    }
}
