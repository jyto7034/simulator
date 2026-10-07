use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use tracing::warn;
use uuid::Uuid;

use super::{grant_state::EconomyGrantState, GameCore};
use crate::game::{
    behavior::{ActionDisabledReasonCode, ActionKind, BehaviorResult, GameError},
    data::shop_data::{ShopMerchandise, ShopMerchandiseEntry, ShopMetadata},
    determinism,
    enums::{ShopAction, ShopEventOption, ShopOfferOption, ShopRefreshOption},
    map::{GameMode, MapNodeId},
    resources::{
        ConsumableItemDto, InventoryDiffDto, InventoryItemDto, ShopOfferState, ShopSessionState,
    },
    reward::{GrantExecutionContext, RewardEffect},
};

const SHOP_PAGE_SEED_NAMESPACE: u64 = 0x5348_4f50_5041_4745;
const SHOP_OFFER_ID_HIGH_NAMESPACE: u64 = 0x5348_4f50_4f46_4849;
const SHOP_OFFER_ID_LOW_NAMESPACE: u64 = 0x5348_4f50_4f46_4c4f;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShopSessionCommitMode {
    RequireRun,
    AllowActiveOnlyWithoutRun,
}

impl GameCore {
    pub(super) fn commit_shop_session(
        &mut self,
        session: ShopSessionState,
        mode: ShopSessionCommitMode,
    ) -> Result<(), GameError> {
        match mode {
            ShopSessionCommitMode::RequireRun => {
                self.run_state_mut()?
                    .shop_sessions
                    .insert(session.node_id, session.clone());
            }
            ShopSessionCommitMode::AllowActiveOnlyWithoutRun => {
                if let Some(run) = self.state.run.as_mut() {
                    run.shop_sessions.insert(session.node_id, session.clone());
                }
            }
        }
        self.state.active_node_content =
            Some(crate::game::resources::ActiveNodeContent::Shop(session));
        Ok(())
    }

    pub(super) fn build_shop_session(
        &self,
        node_id: MapNodeId,
        shop: &ShopMetadata,
    ) -> Result<ShopSessionState, GameError> {
        let mut session = ShopSessionState {
            id: shop.id.clone(),
            name: shop.name.clone(),
            uuid: shop.uuid,
            context: shop.context,
            node_id,
            offer_count: shop.offer_count,
            refresh_policy: shop.refresh_policy,
            sell_policy: shop.sell_policy.clone(),
            refresh_uses: 0,
            page_index: 0,
            offers: Vec::new(),
            purchased_merchandise: HashSet::new(),
        };
        if self.is_wandering_monolith_shop(shop) {
            let policy = &self.run_policy().monolith;
            session.offer_count = policy.first_page_offer_count;
            session.refresh_policy = shop.refresh_policy.map(|mut refresh| {
                refresh.max_uses = u32::from(policy.paid_refresh_uses);
                refresh
            });
            session.offers = self.generate_wandering_first_page(&session, shop)?;
        } else {
            session.offers = self.generate_shop_page(&session, shop, 0)?;
        }
        Ok(session)
    }

    fn is_wandering_monolith_shop(&self, shop: &ShopMetadata) -> bool {
        self.state.run.as_ref().is_some_and(|run| {
            run.run_progression.game_mode == GameMode::Endless
                && run.run_progression.floor_index().saturating_add(1)
                    >= self.run_policy().monolith.wandering_merchant_min_floor
                && shop.id == self.run_policy().monolith.wandering_merchant_shop_id
        })
    }

    fn generate_wandering_first_page(
        &self,
        session: &ShopSessionState,
        shop: &ShopMetadata,
    ) -> Result<Vec<ShopOfferState>, GameError> {
        let run = self.state.run.as_ref().ok_or(GameError::InvalidAction)?;
        let mut eligible = run
            .monolith
            .progress_by_abnormality
            .iter()
            .filter(|(_, progress)| progress.is_offer_eligible())
            .filter_map(|(abnormality_id, _)| {
                self.game_data
                    .abnormality_data
                    .get_by_id(abnormality_id)
                    .map(|metadata| (abnormality_id.clone(), metadata))
            })
            .collect::<Vec<_>>();
        eligible.sort_by(|left, right| left.0.cmp(&right.0));
        let monolith_count = eligible.len().min(2);
        let item_count = session.offer_count.saturating_sub(monolith_count);
        let mut offers = self.generate_shop_page_with_count(session, shop, 0, item_count)?;

        let page_seed = determinism::seed_with_uuid(
            self.node_seed(session.node_id, SHOP_PAGE_SEED_NAMESPACE),
            0,
            shop.uuid,
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(determinism::seed_with_namespace(
            page_seed,
            0x4d4f_4e4f_4c49_5448,
        ));
        for slot in item_count..session.offer_count {
            let total_weight = eligible
                .len()
                .checked_mul(self.run_policy().monolith.monolith_offer_weight as usize)
                .ok_or(GameError::InvalidAction)?;
            if total_weight == 0 {
                return Err(GameError::InvalidAction);
            }
            let selected_index = rng.gen_range(0..total_weight)
                / self.run_policy().monolith.monolith_offer_weight as usize;
            let (abnormality_id, metadata) = eligible.remove(selected_index);
            offers.push(ShopOfferState {
                offer_id: shop_offer_id(page_seed, slot)?,
                merchandise: ShopMerchandise::Monolith {
                    abnormality_id: abnormality_id.clone(),
                },
                item_uuid: metadata.uuid,
                quantity: u32::from(self.run_policy().monolith.initial_uses),
                total_price: self.run_policy().monolith.monolith_price,
                repeatable: false,
                tags: vec!["monolith".to_string()],
            });
        }
        Ok(offers)
    }

    fn generate_shop_page(
        &self,
        session: &ShopSessionState,
        shop: &ShopMetadata,
        page_index: u32,
    ) -> Result<Vec<ShopOfferState>, GameError> {
        self.generate_shop_page_with_count(session, shop, page_index, session.offer_count)
    }

    fn generate_shop_page_with_count(
        &self,
        session: &ShopSessionState,
        shop: &ShopMetadata,
        page_index: u32,
        offer_count: usize,
    ) -> Result<Vec<ShopOfferState>, GameError> {
        let mut candidates = self
            .game_data
            .shop_data
            .flattened_entries(shop)
            .map_err(GameError::InvalidStaticData)?
            .into_iter()
            .filter(|entry| {
                entry.repeatable || !session.purchased_merchandise.contains(&entry.merchandise)
            })
            .collect::<Vec<_>>();
        if candidates.len() < offer_count {
            return Err(shop_unavailable(
                ActionKind::RefreshShop,
                ActionDisabledReasonCode::PoolExhausted,
            ));
        }

        candidates.sort_by_key(|entry| entry.merchandise.identity_key());
        let seed = determinism::seed_with_uuid(
            self.node_seed(session.node_id, SHOP_PAGE_SEED_NAMESPACE),
            u64::from(page_index),
            shop.uuid,
        );
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut offers = Vec::with_capacity(offer_count);
        for slot in 0..offer_count {
            let total_weight = candidates.iter().try_fold(0_u64, |total, entry| {
                total.checked_add(u64::from(entry.weight))
            });
            let total_weight = total_weight.ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "shop '{}' composed merchandise weight overflows",
                    shop.id
                ))
            })?;
            if total_weight == 0 {
                return Err(GameError::InvalidStaticData(format!(
                    "shop '{}' has zero total merchandise weight",
                    shop.id
                )));
            }
            let mut roll = rng.gen_range(0..total_weight);
            let selected_index = candidates
                .iter()
                .position(|entry| {
                    if roll < u64::from(entry.weight) {
                        true
                    } else {
                        roll -= u64::from(entry.weight);
                        false
                    }
                })
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "shop '{}' failed deterministic weighted selection",
                        shop.id
                    ))
                })?;
            let entry = candidates.remove(selected_index);
            offers.push(self.offer_from_entry(shop, &entry, seed, slot)?);
        }
        Ok(offers)
    }

    fn offer_from_entry(
        &self,
        shop: &ShopMetadata,
        entry: &ShopMerchandiseEntry,
        page_seed: u64,
        slot: usize,
    ) -> Result<ShopOfferState, GameError> {
        let item = self
            .game_data
            .shop_merchandise_item(&entry.merchandise)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "shop '{}' references missing merchandise '{}'",
                    shop.id,
                    entry.merchandise.identity_key()
                ))
            })?;
        let buy_percent = entry.buy_percent.unwrap_or(shop.default_buy_percent);
        let unit_price = checked_ceil_percent(item.price(), buy_percent).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "shop '{}' has invalid buy quote for '{}'",
                shop.id,
                entry.merchandise.identity_key()
            ))
        })?;
        let total_price = unit_price.checked_mul(entry.quantity).ok_or_else(|| {
            GameError::InvalidStaticData(format!(
                "shop '{}' total buy quote overflows for '{}'",
                shop.id,
                entry.merchandise.identity_key()
            ))
        })?;
        if total_price == 0 {
            return Err(GameError::InvalidStaticData(format!(
                "shop '{}' has zero buy quote for '{}'",
                shop.id,
                entry.merchandise.identity_key()
            )));
        }
        Ok(ShopOfferState {
            offer_id: shop_offer_id(page_seed, slot)?,
            merchandise: entry.merchandise.clone(),
            item_uuid: item.uuid(),
            quantity: entry.quantity,
            total_price,
            repeatable: entry.repeatable,
            tags: entry.tags.clone(),
        })
    }

    pub(super) fn shop_event_option(shop: &ShopSessionState) -> ShopEventOption {
        ShopEventOption {
            id: shop.id.clone(),
            name: shop.name.clone(),
            uuid: shop.uuid,
            context: shop.context,
            offers: shop
                .offers
                .iter()
                .map(|offer| ShopOfferOption {
                    offer_id: offer.offer_id,
                    merchandise: offer.merchandise.clone(),
                    item_uuid: offer.item_uuid,
                    quantity: offer.quantity,
                    total_price: offer.total_price,
                })
                .collect(),
            refresh: shop.refresh_policy.map(|policy| ShopRefreshOption {
                current_cost: policy.base_cost,
                max_uses: policy.max_uses,
                uses_consumed: shop.refresh_uses,
                remaining_uses: policy.max_uses.saturating_sub(shop.refresh_uses),
            }),
            selling_enabled: shop.sell_policy.is_some(),
        }
    }

    pub(super) fn shop_purchase_eligibility(
        &self,
        offer: &ShopOfferState,
    ) -> crate::game::behavior::ShopPayloadEligibilityDto {
        if let ShopMerchandise::Monolith { abnormality_id } = &offer.merchandise {
            let eligible = self.state.run.as_ref().is_some_and(|run| {
                run.run_progression.game_mode == GameMode::Endless
                    && run
                        .monolith
                        .progress(abnormality_id)
                        .is_some_and(|progress| progress.is_offer_eligible())
            });
            if !eligible {
                return crate::game::behavior::ShopPayloadEligibilityDto::disabled(
                    ActionDisabledReasonCode::OfferUnavailable,
                );
            }
        }
        if self.state.enkephalin.amount < offer.total_price {
            crate::game::behavior::ShopPayloadEligibilityDto::disabled(
                ActionDisabledReasonCode::InsufficientResources,
            )
        } else {
            crate::game::behavior::ShopPayloadEligibilityDto::enabled()
        }
    }

    pub(super) fn shop_refresh_eligibility(
        &self,
        session: &ShopSessionState,
    ) -> crate::game::behavior::ShopPayloadEligibilityDto {
        match self.refresh_disabled_reason(session) {
            Some(reason) => crate::game::behavior::ShopPayloadEligibilityDto::disabled(reason),
            None => crate::game::behavior::ShopPayloadEligibilityDto::enabled(),
        }
    }

    pub(super) fn shop_sale_candidates(
        &self,
        session: &ShopSessionState,
    ) -> Vec<crate::game::behavior::ShopSaleCandidateSnapshotDto> {
        let mut candidates = self
            .state
            .inventory
            .equipments
            .iter()
            .map(|owned| {
                let (unit_quote, eligibility) =
                    match self.sale_evaluation(session, owned.instance_uuid, 1) {
                        Ok((_, quote)) => (
                            quote,
                            crate::game::behavior::ShopPayloadEligibilityDto::enabled(),
                        ),
                        Err(reason) => (
                            0,
                            crate::game::behavior::ShopPayloadEligibilityDto::disabled(reason),
                        ),
                    };
                crate::game::behavior::ShopSaleCandidateSnapshotDto {
                    item_uuid: owned.instance_uuid,
                    merchandise: ShopMerchandise::Equipment {
                        equipment_id: owned.meta.id.clone(),
                    },
                    available_quantity: 1,
                    unit_quote,
                    eligibility,
                }
            })
            .collect::<Vec<_>>();
        candidates.extend(self.state.inventory.consumables.iter().map(|stack| {
            let item_uuid = stack.meta.uuid;
            let (unit_quote, eligibility) = match self.sale_evaluation(session, item_uuid, 1) {
                Ok((_, quote)) => (
                    quote,
                    crate::game::behavior::ShopPayloadEligibilityDto::enabled(),
                ),
                Err(reason) => (
                    0,
                    crate::game::behavior::ShopPayloadEligibilityDto::disabled(reason),
                ),
            };
            crate::game::behavior::ShopSaleCandidateSnapshotDto {
                item_uuid,
                merchandise: ShopMerchandise::Consumable {
                    consumable_id: stack.meta.id.clone(),
                },
                available_quantity: stack.quantity,
                unit_quote,
                eligibility,
            }
        }));
        candidates.sort_by_key(|candidate| candidate.item_uuid);
        candidates
    }

    fn refresh_disabled_reason(
        &self,
        session: &ShopSessionState,
    ) -> Option<ActionDisabledReasonCode> {
        let Some(policy) = session.refresh_policy else {
            return Some(ActionDisabledReasonCode::RefreshUnavailable);
        };
        if session.refresh_uses >= policy.max_uses {
            return Some(ActionDisabledReasonCode::RefreshUsesExhausted);
        }
        if self.state.enkephalin.amount < policy.base_cost {
            return Some(ActionDisabledReasonCode::InsufficientResources);
        }
        let Some(shop) = self.game_data.shop_data.get_by_id(&session.id) else {
            return Some(ActionDisabledReasonCode::ContentUnavailable);
        };
        let Ok(entries) = self.game_data.shop_data.flattened_entries(shop) else {
            return Some(ActionDisabledReasonCode::ContentUnavailable);
        };
        if entries
            .into_iter()
            .filter(|entry| {
                entry.repeatable || !session.purchased_merchandise.contains(&entry.merchandise)
            })
            .count()
            < session.offer_count
        {
            return Some(ActionDisabledReasonCode::PoolExhausted);
        }
        None
    }

    fn sale_evaluation(
        &self,
        session: &ShopSessionState,
        item_uuid: Uuid,
        quantity: u32,
    ) -> Result<(ShopMerchandise, u32), ActionDisabledReasonCode> {
        if quantity == 0 {
            return Err(ActionDisabledReasonCode::InvalidQuantity);
        }
        let sell_policy = session
            .sell_policy
            .as_ref()
            .ok_or(ActionDisabledReasonCode::SellingUnavailable)?;
        if self.state.relics.get(item_uuid).is_some() {
            return Err(if quantity == 1 {
                ActionDisabledReasonCode::RelicNotSellable
            } else {
                ActionDisabledReasonCode::InvalidQuantity
            });
        }
        let (merchandise, base_price) =
            if let Some(owned) = self.state.inventory.equipments.get_item(&item_uuid) {
                if quantity != 1 {
                    return Err(ActionDisabledReasonCode::InvalidQuantity);
                }
                if owned.meta.bound {
                    return Err(ActionDisabledReasonCode::ItemBound);
                }
                if owned.equipped_to.is_some() {
                    return Err(ActionDisabledReasonCode::ItemEquipped);
                }
                (
                    ShopMerchandise::Equipment {
                        equipment_id: owned.meta.id.clone(),
                    },
                    owned.meta.price,
                )
            } else if let Some(stack) = self.state.inventory.consumables.get_item(&item_uuid) {
                if stack.quantity < quantity {
                    return Err(ActionDisabledReasonCode::InvalidQuantity);
                }
                (
                    ShopMerchandise::Consumable {
                        consumable_id: stack.meta.id.clone(),
                    },
                    stack.meta.price,
                )
            } else {
                return Err(ActionDisabledReasonCode::OfferUnavailable);
            };
        if let Some(accepted_tags) = &sell_policy.accepted_tags {
            let tags = self.game_data.shop_data.merchandise_tags(&merchandise);
            if !accepted_tags.iter().any(|tag| tags.contains(tag.as_str())) {
                return Err(ActionDisabledReasonCode::TagNotAccepted);
            }
        }
        let unit_quote = checked_floor_percent(base_price, sell_policy.price_percent)
            .filter(|quote| *quote > 0)
            .ok_or(ActionDisabledReasonCode::InvalidQuote)?;
        let minimum_live_buy = self
            .game_data
            .shop_data
            .shops
            .iter()
            .filter_map(|shop| {
                self.game_data
                    .shop_data
                    .flattened_entries(shop)
                    .ok()
                    .and_then(|entries| {
                        entries
                            .into_iter()
                            .filter(|entry| entry.merchandise == merchandise)
                            .filter_map(|entry| {
                                checked_ceil_percent(
                                    base_price,
                                    entry.buy_percent.unwrap_or(shop.default_buy_percent),
                                )
                            })
                            .min()
                    })
            })
            .min();
        if minimum_live_buy.is_some_and(|minimum| unit_quote >= minimum) {
            return Err(ActionDisabledReasonCode::InvalidQuote);
        }
        let total_quote = unit_quote
            .checked_mul(quantity)
            .ok_or(ActionDisabledReasonCode::InvalidQuote)?;
        self.state
            .enkephalin
            .amount
            .checked_add(total_quote)
            .ok_or(ActionDisabledReasonCode::InvalidQuote)?;
        Ok((merchandise, total_quote))
    }

    fn purchase_offer(&mut self, offer_id: Uuid) -> Result<BehaviorResult, GameError> {
        let session = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInShopState)?
            .as_shop()?;
        let offer = session
            .offers
            .iter()
            .find(|offer| offer.offer_id == offer_id)
            .cloned()
            .ok_or(GameError::ShopItemNotFound)?;
        if let Some(reason) = self.shop_purchase_eligibility(&offer).disabled_reason_code {
            return Err(shop_unavailable(ActionKind::PurchaseItem, reason));
        }

        if let ShopMerchandise::Monolith { abnormality_id } = &offer.merchandise {
            let mut next_monolith = self.run_state()?.monolith.clone();
            next_monolith
                .purchase(abnormality_id, self.run_policy().monolith.initial_uses)
                .map_err(|_| GameError::InvalidAction)?;
            let next_currency = self
                .state
                .enkephalin
                .amount
                .checked_sub(offer.total_price)
                .ok_or(GameError::InsufficientResources)?;
            let mut next_session = session.clone();
            let removed = next_session.remove_offer(offer_id)?;
            next_session
                .purchased_merchandise
                .insert(removed.merchandise);
            self.run_state_mut()?.monolith = next_monolith;
            self.state.enkephalin.amount = next_currency;
            self.commit_shop_session(
                next_session,
                ShopSessionCommitMode::AllowActiveOnlyWithoutRun,
            )?;
            return Ok(BehaviorResult::PurchaseItem {
                enkephalin: next_currency,
                inventory_diff: InventoryDiffDto::default(),
            });
        }

        let effect = match &offer.merchandise {
            ShopMerchandise::Equipment { equipment_id } => RewardEffect::GrantEquipment {
                equipment_id: equipment_id.clone(),
            },
            ShopMerchandise::Consumable { consumable_id } => RewardEffect::GrantConsumable {
                consumable_id: consumable_id.clone(),
                quantity: offer.quantity,
            },
            ShopMerchandise::Monolith { .. } => return Err(GameError::InvalidAction),
        };
        let mut grant_state = EconomyGrantState::capture(&self.state)?;
        grant_state.enkephalin.amount = grant_state
            .enkephalin
            .amount
            .checked_sub(offer.total_price)
            .ok_or(GameError::InsufficientResources)?;
        let granted = grant_state.grant_effects(
            &self.game_data,
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            &[effect],
            self.run_seed ^ 0x5348_4f50_4255_5931,
        )?;
        let mut next_session = session.clone();
        let removed = next_session.remove_offer(offer_id)?;
        next_session
            .purchased_merchandise
            .insert(removed.merchandise);

        grant_state.commit_into(&mut self.state);
        self.commit_shop_session(
            next_session,
            ShopSessionCommitMode::AllowActiveOnlyWithoutRun,
        )?;

        Ok(BehaviorResult::PurchaseItem {
            enkephalin: self.state.enkephalin.amount,
            inventory_diff: granted.inventory_diff,
        })
    }

    fn sell_item_quantity(
        &mut self,
        item_uuid: Uuid,
        quantity: u32,
    ) -> Result<BehaviorResult, GameError> {
        let session = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInShopState)?
            .as_shop()?;
        let (merchandise, total_quote) = self
            .sale_evaluation(session, item_uuid, quantity)
            .map_err(|reason| shop_unavailable(ActionKind::SellItem, reason))?;

        let mut next_inventory = self.state.inventory.clone();
        let mut diff = InventoryDiffDto::default();
        match &merchandise {
            ShopMerchandise::Equipment { .. } => {
                next_inventory
                    .remove_item(item_uuid)
                    .map_err(|error| error.into_game_error())?;
                diff.removed.push(item_uuid);
            }
            ShopMerchandise::Consumable { consumable_id } => {
                let removal = next_inventory
                    .consumables
                    .remove_quantity(consumable_id, quantity)?;
                if removal.remaining_quantity == 0 {
                    diff.removed.push(item_uuid);
                } else {
                    let stack = next_inventory
                        .consumables
                        .get_by_definition_id(consumable_id)
                        .ok_or(GameError::InventoryItemNotFound)?;
                    diff.updated.push(InventoryItemDto::Consumable(
                        ConsumableItemDto::from_consumable_stack(stack),
                    ));
                }
            }
            ShopMerchandise::Monolith { .. } => return Err(GameError::InvalidAction),
        }
        let next_currency = self.state.enkephalin.amount + total_quote;
        self.state.inventory = next_inventory;
        self.state.enkephalin.amount = next_currency;
        Ok(BehaviorResult::SellItem {
            enkephalin: next_currency,
            inventory_diff: diff,
        })
    }

    fn refresh_shop(&mut self) -> Result<BehaviorResult, GameError> {
        let session = self
            .state
            .active_node_content
            .as_ref()
            .ok_or(GameError::NotInShopState)?
            .as_shop()?;
        if let Some(reason) = self.refresh_disabled_reason(session) {
            return Err(shop_unavailable(ActionKind::RefreshShop, reason));
        }
        let policy = session
            .refresh_policy
            .expect("enabled refresh eligibility requires policy");
        let shop = self
            .game_data
            .shop_data
            .get_by_id(&session.id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "active Shop session references missing Shop '{}'",
                    session.id
                ))
            })?;
        let next_page_index = session
            .page_index
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        let next_offers = self.generate_shop_page(session, shop, next_page_index)?;
        let next_currency = self
            .state
            .enkephalin
            .amount
            .checked_sub(policy.base_cost)
            .ok_or(GameError::InsufficientResources)?;
        let mut next_session = session.clone();
        next_session.refresh_uses = next_session
            .refresh_uses
            .checked_add(1)
            .ok_or(GameError::InvalidAction)?;
        next_session.page_index = next_page_index;
        next_session.offers = next_offers;
        let result = Self::shop_event_option(&next_session);
        self.state.enkephalin.amount = next_currency;
        self.commit_shop_session(
            next_session,
            ShopSessionCommitMode::AllowActiveOnlyWithoutRun,
        )?;
        Ok(BehaviorResult::RefreshShop {
            enkephalin: next_currency,
            shop: result,
        })
    }

    pub(super) fn execute_shop_action(
        &mut self,
        action: ShopAction,
    ) -> Result<BehaviorResult, GameError> {
        match action {
            ShopAction::Purchase { offer_id } => self.purchase_offer(offer_id),
            ShopAction::Sell {
                item_uuid,
                quantity,
            } => self.sell_item_quantity(item_uuid, quantity),
            ShopAction::Refresh => self.refresh_shop(),
            ShopAction::Exit => {
                if self.state.node_session.is_some() {
                    return self.handle_complete_node();
                }
                Err(GameError::InvalidAction)
            }
        }
    }
}

fn checked_ceil_percent(base_price: u32, percent: u32) -> Option<u32> {
    let numerator = u64::from(base_price).checked_mul(u64::from(percent))?;
    let rounded = numerator.checked_add(99)?.checked_div(100)?;
    u32::try_from(rounded).ok()
}

fn shop_offer_id(page_seed: u64, slot: usize) -> Result<Uuid, GameError> {
    let slot = u64::try_from(slot).map_err(|_| GameError::InvalidAction)?;
    let high = determinism::seed_with_namespace(page_seed, SHOP_OFFER_ID_HIGH_NAMESPACE ^ slot);
    let low = determinism::seed_with_namespace(page_seed, SHOP_OFFER_ID_LOW_NAMESPACE ^ slot);
    Ok(Uuid::from_u128((u128::from(high) << 64) | u128::from(low)))
}

fn checked_floor_percent(base_price: u32, percent: u32) -> Option<u32> {
    let value = u64::from(base_price)
        .checked_mul(u64::from(percent))?
        .checked_div(100)?;
    u32::try_from(value).ok()
}

fn shop_unavailable(action: ActionKind, reason: ActionDisabledReasonCode) -> GameError {
    warn!(
        ?action,
        ?reason,
        "Shop action rejected by authoritative eligibility"
    );
    GameError::ActionUnavailable { action, reason }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::game::{
        data::{
            consumable_data::{
                ConsumableDatabase, ConsumableDurationPolicy, ConsumableEffect, ConsumableMetadata,
                ConsumableTargetPolicy, ConsumableTier,
            },
            equipment_data::{EquipmentMetadata, EquipmentType},
            shop_data::{ShopContext, ShopDatabase, ShopRefreshPolicy, ShopSellPolicy},
            GameDataBuilder,
        },
        enums::RiskLevel,
        resources::{ActiveNodeContent, GameState},
    };

    #[test]
    fn quote_rounding_is_checked_and_directional() {
        assert_eq!(checked_ceil_percent(1, 1), Some(1));
        assert_eq!(checked_ceil_percent(35, 110), Some(39));
        assert_eq!(checked_floor_percent(35, 40), Some(14));
        assert_eq!(checked_floor_percent(1, 40), Some(0));
    }

    fn equipment(id: &str, uuid: u128, price: u32) -> EquipmentMetadata {
        EquipmentMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            equipment_type: EquipmentType::Armor,
            rarity: RiskLevel::TETH,
            price,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: Default::default(),
            ability_activations: vec![],
            weapon_profile: None,
            event_check_modifiers: vec![],
            event_choice_keys: vec![],
        }
    }

    fn consumable(id: &str, uuid: u128, price: u32) -> ConsumableMetadata {
        ConsumableMetadata {
            id: id.to_string(),
            uuid: Uuid::from_u128(uuid),
            name: id.to_string(),
            description: String::new(),
            tier: ConsumableTier::Common,
            rarity: RiskLevel::ZAYIN,
            price,
            target_policy: ConsumableTargetPolicy::SingleEmployee,
            duration_policy: ConsumableDurationPolicy::NextCombatNode,
            effect: ConsumableEffect::TraumaMitigation { percent: 10 },
            live_pool: true,
        }
    }

    fn entry(
        merchandise: ShopMerchandise,
        quantity: u32,
        repeatable: bool,
    ) -> ShopMerchandiseEntry {
        ShopMerchandiseEntry {
            merchandise,
            quantity,
            weight: 1,
            repeatable,
            buy_percent: None,
            tags: vec!["test".to_string()],
        }
    }

    fn test_core(offer_count: usize, entry_count: usize) -> (GameCore, MapNodeId) {
        let equipments = (0..entry_count)
            .map(|index| equipment(&format!("armor_{index}"), 0x100 + index as u128, 100))
            .collect::<Vec<_>>();
        let tonic = consumable("tonic", 0x200, 25);
        let mut entries = equipments
            .iter()
            .map(|item| {
                entry(
                    ShopMerchandise::Equipment {
                        equipment_id: item.id.clone(),
                    },
                    1,
                    false,
                )
            })
            .collect::<Vec<_>>();
        entries.push(entry(
            ShopMerchandise::Consumable {
                consumable_id: tonic.id.clone(),
            },
            3,
            true,
        ));
        let shop = ShopMetadata {
            id: "test_shop".to_string(),
            name: "Test Shop".to_string(),
            uuid: Uuid::from_u128(0x300),
            context: ShopContext::UnknownExploration,
            offer_count,
            default_buy_percent: 100,
            preset_refs: vec![],
            local_entries: entries,
            refresh_policy: Some(ShopRefreshPolicy {
                base_cost: 7,
                max_uses: 3,
            }),
            sell_policy: Some(ShopSellPolicy {
                price_percent: 40,
                accepted_tags: None,
            }),
        };
        let data = GameDataBuilder::empty()
            .with_equipment(equipments)
            .with_consumable_data(Arc::new(ConsumableDatabase::new(vec![tonic])))
            .with_shops(ShopDatabase::new(vec![shop]))
            .build_arc();
        let node_id = MapNodeId::new(Uuid::from_u128(0x400));
        let mut core = GameCore::new(data, 123);
        core.state.enkephalin.amount = 1_000;
        let session = core
            .build_shop_session(node_id, &core.game_data.shop_data.shops[0])
            .unwrap();
        core.state.active_node_content = Some(ActiveNodeContent::Shop(session));
        core.state.transition_to(GameState::InShop {
            shop_uuid: Uuid::from_u128(0x300),
        });
        (core, node_id)
    }

    #[test]
    fn deterministic_entry_repeats_exact_offer_identity_order_and_quotes() {
        let (core, node_id) = test_core(3, 4);
        let metadata = &core.game_data.shop_data.shops[0];
        let first = core.build_shop_session(node_id, metadata).unwrap();
        let second = core.build_shop_session(node_id, metadata).unwrap();
        assert_eq!(
            first
                .offers
                .iter()
                .map(|offer| (
                    offer.offer_id,
                    offer.merchandise.clone(),
                    offer.quantity,
                    offer.total_price,
                ))
                .collect::<Vec<_>>(),
            second
                .offers
                .iter()
                .map(|offer| (
                    offer.offer_id,
                    offer.merchandise.clone(),
                    offer.quantity,
                    offer.total_price,
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn purchase_consumable_offer_grants_exact_quantity_and_removes_offer_atomically() {
        let (mut core, _) = test_core(2, 1);
        let offer = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers
            .iter()
            .find(|offer| matches!(offer.merchandise, ShopMerchandise::Consumable { .. }))
            .unwrap()
            .clone();
        let before_currency = core.state.enkephalin.amount;
        let skill_fragments_before = serde_json::to_value(&core.state.skill_fragments).unwrap();
        let roster_before = serde_json::to_value(&core.state.roster).unwrap();
        let uuid_manager_before = serde_json::to_value(&core.state.uuid_manager).unwrap();
        let result = core.purchase_offer(offer.offer_id).unwrap();
        assert!(core.state.run.is_none());
        assert_eq!(
            core.state
                .inventory
                .consumables
                .get_by_definition_id("tonic")
                .unwrap()
                .quantity,
            3
        );
        assert_eq!(
            core.state.enkephalin.amount,
            before_currency - offer.total_price
        );
        assert!(core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers
            .iter()
            .all(|candidate| candidate.offer_id != offer.offer_id));
        let BehaviorResult::PurchaseItem { inventory_diff, .. } = result else {
            panic!("purchase result");
        };
        assert_eq!(inventory_diff.added.len(), 1);
        assert_eq!(
            serde_json::to_value(&core.state.skill_fragments).unwrap(),
            skill_fragments_before
        );
        assert_eq!(
            serde_json::to_value(&core.state.roster).unwrap(),
            roster_before
        );
        assert_eq!(
            serde_json::to_value(&core.state.uuid_manager).unwrap(),
            uuid_manager_before
        );
    }

    #[test]
    fn purchase_grant_failure_preserves_all_five_roots_and_the_offer_session() {
        let (mut core, _) = test_core(1, 1);
        let offer_id = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers[0]
            .offer_id;
        core.state
            .active_node_content
            .as_mut()
            .unwrap()
            .as_shop_mut()
            .unwrap()
            .offers[0]
            .merchandise = ShopMerchandise::Equipment {
            equipment_id: "missing_purchase_equipment".to_string(),
        };
        let roots_before = serde_json::json!({
            "inventory": &core.state.inventory,
            "skill_fragments": &core.state.skill_fragments,
            "roster": &core.state.roster,
            "uuid_manager": &core.state.uuid_manager,
            "enkephalin": &core.state.enkephalin,
        });
        let session_before = serde_json::to_value(
            core.state
                .active_node_content
                .as_ref()
                .unwrap()
                .as_shop()
                .unwrap(),
        )
        .unwrap();

        let error = core.purchase_offer(offer_id).unwrap_err();

        assert!(matches!(error, GameError::InvalidStaticData(_)));
        assert_eq!(
            serde_json::json!({
                "inventory": &core.state.inventory,
                "skill_fragments": &core.state.skill_fragments,
                "roster": &core.state.roster,
                "uuid_manager": &core.state.uuid_manager,
                "enkephalin": &core.state.enkephalin,
            }),
            roots_before
        );
        assert_eq!(
            serde_json::to_value(
                core.state
                    .active_node_content
                    .as_ref()
                    .unwrap()
                    .as_shop()
                    .unwrap(),
            )
            .unwrap(),
            session_before
        );
    }

    #[test]
    fn refresh_is_full_sized_charged_and_excludes_purchased_nonrepeatable_merchandise() {
        let (mut core, _) = test_core(2, 4);
        let purchased_offer = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers[0]
            .clone();
        core.purchase_offer(purchased_offer.offer_id).unwrap();
        let before_refresh_currency = core.state.enkephalin.amount;
        core.refresh_shop().unwrap();
        assert!(core.state.run.is_none());
        let session = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap();
        assert_eq!(session.offers.len(), 2);
        assert_eq!(session.refresh_uses, 1);
        assert_eq!(session.page_index, 1);
        assert_eq!(core.state.enkephalin.amount, before_refresh_currency - 7);
        if !purchased_offer.repeatable {
            assert!(session
                .offers
                .iter()
                .all(|offer| offer.merchandise != purchased_offer.merchandise));
        }
    }

    #[test]
    fn exhausted_refresh_rejects_without_currency_use_or_page_mutation() {
        let (mut core, _) = test_core(2, 1);
        let equipment_offer = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers
            .iter()
            .find(|offer| matches!(offer.merchandise, ShopMerchandise::Equipment { .. }))
            .unwrap()
            .clone();
        core.purchase_offer(equipment_offer.offer_id).unwrap();
        let before_currency = core.state.enkephalin.amount;
        let before_offers = core
            .state
            .active_node_content
            .as_ref()
            .unwrap()
            .as_shop()
            .unwrap()
            .offers
            .iter()
            .map(|offer| offer.offer_id)
            .collect::<Vec<_>>();
        let error = core.refresh_shop().unwrap_err();
        assert!(matches!(
            error,
            GameError::ActionUnavailable {
                action: ActionKind::RefreshShop,
                reason: ActionDisabledReasonCode::PoolExhausted
            }
        ));
        assert_eq!(core.state.enkephalin.amount, before_currency);
        assert_eq!(
            core.state
                .active_node_content
                .as_ref()
                .unwrap()
                .as_shop()
                .unwrap()
                .offers
                .iter()
                .map(|offer| offer.offer_id)
                .collect::<Vec<_>>(),
            before_offers
        );
    }

    #[test]
    fn partial_consumable_sale_updates_stack_and_uses_exact_quantity_quote() {
        let (mut core, _) = test_core(1, 1);
        let meta = Arc::new(
            core.game_data
                .consumable_data
                .get_by_id("tonic")
                .unwrap()
                .clone(),
        );
        core.state.inventory.add_consumable(meta, 5).unwrap();
        let before_currency = core.state.enkephalin.amount;
        let result = core.sell_item_quantity(Uuid::from_u128(0x200), 2).unwrap();
        assert_eq!(core.state.enkephalin.amount, before_currency + 20);
        assert_eq!(
            core.state
                .inventory
                .consumables
                .get_by_definition_id("tonic")
                .unwrap()
                .quantity,
            3
        );
        let BehaviorResult::SellItem { inventory_diff, .. } = result else {
            panic!("sale result");
        };
        let InventoryItemDto::Consumable(stack) = &inventory_diff.updated[0] else {
            panic!("partial sale must update stack");
        };
        assert_eq!(stack.quantity, 3);
    }
}
