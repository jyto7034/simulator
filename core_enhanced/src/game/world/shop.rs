use tracing::{debug, warn};
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::{GameDataBase, Item};
use crate::game::enums::ShopAction;
use crate::game::managers::uuid_manager::UuidManager;
use crate::game::resources::{
    ActiveNodeContent, Enkephalin, Inventory, InventoryDiffDto, InventoryItemDto,
};

struct ShopExecutor;

impl ShopExecutor {
    fn reroll_selected(
        active_node_content: &mut Option<ActiveNodeContent>,
    ) -> Result<BehaviorResult, GameError> {
        let selected = active_node_content
            .as_mut()
            .ok_or(GameError::NotInShopState)?;
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
        active_node_content: Option<&mut ActiveNodeContent>,
        uuid_manager: &mut UuidManager,
        enkephalin: &mut Enkephalin,
        game_data: &GameDataBase,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected = active_node_content.ok_or(GameError::NotInShopState)?;
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
            Item::Consumable(_) => uuid_manager.next_owned_consumable(),
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
        active_node_content: Option<&ActiveNodeContent>,
        enkephalin: &mut Enkephalin,
        item_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let selected = active_node_content.ok_or(GameError::NotInShopState)?;
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
                    state.active_node_content.as_mut(),
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
                    state.active_node_content.as_ref(),
                    &mut state.enkephalin,
                    item_uuid,
                )
            }
            ShopAction::Reroll => {
                ShopExecutor::reroll_selected(&mut self.state.active_node_content)
            }
            ShopAction::Exit => {
                if self.state.node_session.is_some() {
                    return self.handle_complete_node();
                }
                Err(GameError::InvalidAction)
            }
        }
    }
}
