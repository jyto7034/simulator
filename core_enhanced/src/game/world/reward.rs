use tracing::info;
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::enums::{RewardAction, RewardMode};
use crate::game::resources::{GameState, InventoryDiffDto};
use crate::game::reward::{RewardExecutor, RewardOption};

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

        let (enkephalin, inventory_diff) = self.apply_reward_session(&reward)?;

        self.transition_to(GameState::InRewardClaimed {
            reward_uuid: reward.stage_uuid,
        })?;

        Ok(BehaviorResult::RewardGranted {
            enkephalin,
            inventory_diff,
        })
    }

    pub(super) fn apply_reward_session(
        &mut self,
        reward: &crate::game::resources::RewardSessionState,
    ) -> Result<(u32, InventoryDiffDto), GameError> {
        let rewards_to_apply: Vec<RewardOption> = match reward.mode {
            RewardMode::ClaimAll => reward.rewards.clone(),
            RewardMode::ChooseOne => vec![reward
                .get_selected_reward()
                .cloned()
                .ok_or(GameError::InvalidAction)?],
        };

        let mut inventory_diff = crate::game::resources::InventoryDiffDto::default();

        for (index, reward_option) in rewards_to_apply.iter().enumerate() {
            info!(
                "Applying reward '{}' (uuid={}) with {} effect(s)",
                reward_option.id,
                reward_option.uuid,
                reward_option.effects.len()
            );
            let seed = self.reward_seed(reward.stage_uuid, reward_option.uuid, index as u64);

            let state = &mut self.state;
            let uuid_manager = &mut state.uuid_manager;
            let enkephalin = &mut state.enkephalin;
            let inventory = &mut state.inventory;
            let skill_fragments = &mut state.skill_fragments;
            let skill_fragment_policy = &state.skill_fragment_policy;
            let game_data = &self.game_data;
            let granted = RewardExecutor::grant_reward_with_state(
                inventory,
                skill_fragments,
                uuid_manager,
                enkephalin,
                game_data,
                skill_fragment_policy,
                reward_option,
                seed,
            )?;
            Self::merge_inventory_diff(&mut inventory_diff, granted);
        }

        let enkephalin = self.state.enkephalin.amount;
        Ok((enkephalin, inventory_diff))
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
