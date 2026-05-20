use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{GameCore, RewardClaimDestination};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::{GameDataBase, Item};
use crate::game::enums::{RewardAction, RewardMode, ShopAction};
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::resources::{
    Enkephalin, GameState, Inventory, InventoryDiffDto, InventoryItemDto, SelectedEvent,
};
use crate::game::reward::{RewardExecutor, RewardOption};

struct ShopExecutor;

impl ShopExecutor {
    fn reroll_selected(
        selected_event: &mut Option<SelectedEvent>,
    ) -> Result<BehaviorResult, GameError> {
        let selected = selected_event.as_mut().ok_or(GameError::NotInShopState)?;
        let shop = selected.as_shop_mut()?;

        if !shop.can_reroll {
            warn!("Reroll requested but current shop does not allow reroll");
            return Err(GameError::ShopRerollNotAllowed);
        }

        if shop.hidden_items.is_empty() {
            warn!(
                "Reroll requested but shop has no hidden_items to reroll from (shop_uuid={})",
                shop.uuid
            );
            return Err(GameError::ShopRerollNotAllowed);
        }

        shop.reroll_items();
        shop.can_reroll = false;
        debug!("Shop items rerolled (shop_uuid={})", shop.uuid);

        Ok(BehaviorResult::RerollShop {
            new_items: shop.visible_items.clone(),
        })
    }

    fn purchase_item_selected(
        inventory: &mut Inventory,
        selected_event: Option<&mut SelectedEvent>,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected = selected_event.ok_or(GameError::NotInShopState)?;
        let shop = selected.as_shop_mut()?;

        if !shop.visible_items.contains(&item_uuid) {
            warn!(
                "Item uuid {} not found in visible_items of shop '{}'",
                item_uuid, shop.id
            );
            return Err(GameError::ShopItemNotFound);
        }

        let item = game_data
            .item(&item_uuid)
            .map(crate::game::data::ItemRef::to_owned_item)
            .ok_or(GameError::ShopItemNotFound)?;
        let price = item.price();

        if enkephalin.amount < price {
            warn!(
                "Insufficient Enkephalin: have={}, price={} (item_uuid={})",
                enkephalin.amount, price, item_uuid
            );
            return Err(GameError::InsufficientResources);
        }

        if let Item::Artifact(meta) = &item {
            if inventory.has_artifact(meta.uuid) {
                warn!("Artifact already owned: item_uuid={}", item_uuid);
                return Err(GameError::AlreadyOwnedArtifact);
            }
        }

        if !inventory.can_add_item(&item) {
            warn!("Inventory full: cannot add item (item_uuid={})", item_uuid);
            return Err(GameError::InventoryFull);
        }

        shop.remove_visible_item(item_uuid)?;
        enkephalin.amount -= price;

        let owned_uuid = match &item {
            Item::Equipment(_) => uuid_manager.next_owned_equipment(),
            Item::Abnormality(_) => return Err(GameError::InvalidAction),
            Item::Artifact(_) => item.uuid(),
        };
        inventory.add_item_owned(owned_uuid, item.clone())?;

        let item_dto = InventoryItemDto::from_item_with_uuid(&item, owned_uuid)?;
        Ok(BehaviorResult::PurchaseItem {
            enkephalin: enkephalin.amount,
            inventory_diff: InventoryDiffDto {
                added: vec![item_dto],
                updated: Vec::new(),
                removed: Vec::new(),
                material_stacks: Vec::new(),
            },
        })
    }

    fn sell_item_selected(
        inventory: &mut Inventory,
        selected_event: Option<&SelectedEvent>,
        enkephalin: &mut Enkephalin,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected = selected_event.ok_or(GameError::NotInShopState)?;
        selected.as_shop()?;

        if let Some(owned_equipment) = inventory.equipments.get_item(&item_uuid) {
            if owned_equipment.equipped_to.is_some() {
                warn!(
                    "Rejected sell request: equipped item cannot be sold (item_uuid={})",
                    item_uuid
                );
                return Err(GameError::InvalidAction);
            }
        }

        let item = inventory
            .find_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let sell_price = item.price() / 2;
        inventory
            .remove_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        enkephalin.amount = enkephalin
            .amount
            .checked_add(sell_price)
            .ok_or(GameError::InvalidAction)?;

        Ok(BehaviorResult::SellItem {
            enkephalin: enkephalin.amount,
            inventory_diff: InventoryDiffDto {
                added: Vec::new(),
                updated: Vec::new(),
                removed: vec![item_uuid],
                material_stacks: Vec::new(),
            },
        })
    }
}

impl GameCore {
    pub(super) fn execute_shop_action(
        &mut self,
        action: ShopAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { item_uuid } => {
                let state = &mut self.state;
                ShopExecutor::purchase_item_selected(
                    &mut state.inventory,
                    state.selected_event.as_mut(),
                    &mut state.uuid_manager,
                    &mut state.enkephalin,
                    &self.game_data,
                    item_uuid,
                )
            }
            ShopAction::Sell { item_uuid } => {
                let state = &mut self.state;
                ShopExecutor::sell_item_selected(
                    &mut state.inventory,
                    state.selected_event.as_ref(),
                    &mut state.enkephalin,
                    item_uuid,
                )
            }
            ShopAction::Reroll => ShopExecutor::reroll_selected(&mut self.state.selected_event),
            ShopAction::Exit => {
                if self.state.node_session.is_some() {
                    return self.handle_complete_node();
                }
                Err(GameError::InvalidAction)
            }
        }
    }

    pub(super) fn execute_reward_action(
        &mut self,
        action: RewardAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            RewardAction::Claim => {
                self.claim_current_reward_session(RewardClaimDestination::Reward)
            }
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

    pub(super) fn claim_current_reward_session(
        &mut self,
        destination: RewardClaimDestination,
    ) -> Result<BehaviorResult, GameError> {
        let reward = {
            let selected = self
                .state
                .selected_event
                .as_ref()
                .ok_or(GameError::NotInRewardState)?;
            selected.as_reward()?.clone()
        };

        let (enkephalin, inventory_diff) = self.apply_reward_session(&reward)?;

        let next_state = match destination {
            RewardClaimDestination::Reward => GameState::InRewardClaimed {
                reward_uuid: reward.stage_uuid,
            },
        };
        self.transition_to(next_state)?;

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
            .selected_event
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
