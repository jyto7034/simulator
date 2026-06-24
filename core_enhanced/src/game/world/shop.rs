use tracing::{debug, warn};
use uuid::Uuid;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::Item;
use crate::game::enums::ShopAction;
use crate::game::resources::{ActiveNodeContent, InventoryDiffDto};
use crate::game::reward::RewardEffect;

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
}

impl GameCore {
    fn purchase_item_selected(&mut self, item_uuid: Uuid) -> Result<BehaviorResult, GameError> {
        let shop = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInShopState)?
            .as_shop()?;

        if !shop.visible_items.contains(&item_uuid) {
            warn!(
                "Item uuid {} not found in visible_items of shop '{}'",
                item_uuid, shop.id
            );
            return Err(GameError::ShopItemNotFound);
        }

        let item = self
            .game_data
            .item(&item_uuid)
            .map(crate::game::data::ItemRef::to_owned_item)
            .ok_or(GameError::ShopItemNotFound)?;
        let price = item.price();

        if self.state.enkephalin.amount < price {
            warn!(
                "Insufficient Enkephalin: have={}, price={} (item_uuid={})",
                self.state.enkephalin.amount, price, item_uuid
            );
            return Err(GameError::InsufficientResources);
        }

        if let Item::Artifact(meta) = &item {
            if self.state.inventory.has_artifact(meta.uuid) {
                warn!("Artifact already owned: item_uuid={}", item_uuid);
                return Err(GameError::AlreadyOwnedArtifact);
            }
        }

        if !self.state.inventory.can_add_item(&item) {
            warn!("Inventory full: cannot add item (item_uuid={})", item_uuid);
            return Err(GameError::InventoryFull);
        }

        let effect = match &item {
            Item::Equipment(meta) => RewardEffect::GrantEquipment {
                equipment_id: meta.id.clone(),
            },
            Item::Consumable(meta) => RewardEffect::GrantConsumable {
                consumable_id: meta.id.clone(),
            },
            Item::Artifact(meta) => RewardEffect::GrantArtifact {
                artifact_id: meta.id.clone(),
            },
        };
        let granted = self.apply_grant_effects(&[effect])?;
        self.state.enkephalin.amount -= price;
        self.state
            .active_node_content
            .as_mut()
            .ok_or(GameError::NotInShopState)?
            .as_shop_mut()?
            .remove_visible_item(item_uuid)?;

        Ok(BehaviorResult::PurchaseItem {
            enkephalin: self.state.enkephalin.amount,
            inventory_diff: granted.inventory_diff,
        })
    }

    fn sell_item_selected(&mut self, item_uuid: Uuid) -> Result<BehaviorResult, GameError> {
        let selected = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInShopState)?;
        selected.as_shop()?;

        if let Some(owned_equipment) = self.state.inventory.equipments.get_item(&item_uuid) {
            if owned_equipment.meta.bound {
                warn!(
                    "Rejected sell request: bound item cannot be sold (item_uuid={})",
                    item_uuid
                );
                return Err(GameError::InvalidAction);
            }
            if owned_equipment.equipped_to.is_some() {
                warn!(
                    "Rejected sell request: equipped item cannot be sold (item_uuid={})",
                    item_uuid
                );
                return Err(GameError::InvalidAction);
            }
        }

        if self.state.inventory.has_artifact(item_uuid) {
            return Err(GameError::InventoryItemNotRemovable);
        }

        let item = self
            .state
            .inventory
            .find_item(item_uuid)
            .ok_or(GameError::InventoryItemNotFound)?;
        let sell_price = item.price() / 2;
        self.state
            .enkephalin
            .amount
            .checked_add(sell_price)
            .ok_or(GameError::InvalidAction)?;
        self.state
            .inventory
            .remove_item(item_uuid)
            .map_err(|err| err.into_game_error())?;
        let granted =
            self.apply_grant_effects(&[RewardEffect::GrantEnkephalin { amount: sell_price }])?;

        Ok(BehaviorResult::SellItem {
            enkephalin: self.state.enkephalin.amount,
            inventory_diff: InventoryDiffDto {
                added: granted.inventory_diff.added,
                updated: granted.inventory_diff.updated,
                removed: vec![item_uuid],
                material_stacks: granted.inventory_diff.material_stacks,
            },
        })
    }

    pub(super) fn execute_shop_action(
        &mut self,
        action: ShopAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { item_uuid } => self.purchase_item_selected(item_uuid),
            ShopAction::Sell { item_uuid } => self.sell_item_selected(item_uuid),
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
