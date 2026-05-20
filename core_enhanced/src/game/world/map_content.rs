use tracing::info;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::random_event_data::RandomEventTarget;
use crate::game::data::reward_data::RewardTag;
use crate::game::determinism;
use crate::game::enums::{RandomEventOption, RewardMode, ShopEventOption};
use crate::game::map::{MapNodeCategory, MapNodeId, MapNodePayload, SupportNodeMode};
use crate::game::resources::{
    GameState, RewardSessionState, SelectedEvent, SelectedEventState, ShopSessionState,
    SupportSessionState,
};
use crate::game::reward::RewardOption;

impl GameCore {
    pub(super) fn deliver_research_if_safe_node(
        &mut self,
        category: MapNodeCategory,
    ) -> Result<Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>, GameError> {
        if !self.state.research_delivery_policy.can_deliver_on(category) {
            return Ok(vec![]);
        }

        let deliveries = self
            .state
            .skill_fragments
            .deliver_pending_research_with_policy(
                &self.game_data.skill_fragment_data,
                &self.state.skill_fragment_policy,
            )?;
        if !deliveries.is_empty() {
            info!(
                "Delivered pending skill fragment research on {:?}: {:?}",
                category, deliveries
            );
        }
        Ok(deliveries)
    }

    pub(super) fn node_seed(&self, node_id: MapNodeId, namespace: u64) -> u64 {
        let act_seed = self
            .state
            .run
            .as_ref()
            .map(|run| run.run_progression.current_act_seed())
            .unwrap_or(self.run_seed);
        determinism::seed_with_uuid(act_seed, namespace, node_id.0)
    }

    fn resolve_map_shop(
        &self,
        node_id: MapNodeId,
        payload: &MapNodePayload,
    ) -> Result<Option<ShopSessionState>, GameError> {
        let MapNodePayload::Shop {
            shop_id,
            shop_pool_id,
        } = payload
        else {
            return Ok(None);
        };

        if let Some(shop_id) = shop_id {
            let shop = self.game_data.shop_data.get_by_id(shop_id).ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "map shop node references missing shop id '{}'",
                    shop_id
                ))
            })?;
            return Ok(Some(ShopSessionState::from(shop)));
        }

        let candidates = if let Some(pool_id) = shop_pool_id {
            let pool = self
                .game_data
                .shop_data
                .pool_by_id(pool_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "map shop node references missing shop pool '{}'",
                        pool_id
                    ))
                })?;
            pool.shop_ids
                .iter()
                .map(|shop_id| {
                    self.game_data.shop_data.get_by_id(shop_id).ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "shop pool '{}' references missing shop '{}'",
                            pool_id, shop_id
                        ))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            self.game_data.shop_data.shops.iter().collect::<Vec<_>>()
        };

        if candidates.is_empty() {
            return Ok(None);
        }

        use rand::{Rng, SeedableRng};
        let seed = self.node_seed(node_id, 0x4D41_505F_5348_4F50);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let index = rng.gen_range(0..candidates.len());
        let shop = candidates[index];

        Ok(Some(ShopSessionState::from(shop)))
    }

    fn resolve_map_reward(
        &self,
        node_id: MapNodeId,
        payload: &MapNodePayload,
    ) -> Result<Option<RewardSessionState>, GameError> {
        let MapNodePayload::Reward { reward_pool_id } = payload else {
            return Ok(None);
        };

        let candidates = if let Some(pool_id) = reward_pool_id {
            let pool = self
                .game_data
                .reward_data
                .pool_by_id(pool_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "map reward node references missing reward pool '{}'",
                        pool_id
                    ))
                })?;
            pool.reward_ids
                .iter()
                .map(|reward_id| {
                    self.game_data
                        .reward_data
                        .get_by_id(reward_id)
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(format!(
                                "reward pool '{}' references missing reward '{}'",
                                pool_id, reward_id
                            ))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            self.game_data
                .reward_data
                .rewards
                .iter()
                .collect::<Vec<_>>()
        };
        let candidates = candidates
            .into_iter()
            .filter(|reward| {
                let tags = reward.resolved_tags();
                !tags.contains(&RewardTag::Forbidden) && !tags.contains(&RewardTag::Experience)
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            return Ok(None);
        }

        use rand::{Rng, SeedableRng};
        let seed = self.node_seed(node_id, 0x4D41_505F_5257_5244);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let index = rng.gen_range(0..candidates.len());
        let reward = candidates[index];

        Ok(Some(self.build_reward_session(
            node_id.0,
            RewardMode::ClaimAll,
            vec![RewardOption::from_metadata(reward)],
            false,
        )))
    }

    fn resolve_map_random_event(
        &self,
        node_id: MapNodeId,
        payload: &MapNodePayload,
    ) -> Result<Option<RandomEventOption>, GameError> {
        let MapNodePayload::Event {
            event_id,
            event_pool_id,
        } = payload
        else {
            return Ok(None);
        };

        if let Some(event_id) = event_id {
            let event = self
                .game_data
                .random_event_data
                .get_by_id(event_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "map event node references missing random event id '{}'",
                        event_id
                    ))
                })?;
            return Ok(Some(RandomEventOption::from(event)));
        }

        let candidates = if let Some(pool_id) = event_pool_id {
            let pool = self
                .game_data
                .random_event_data
                .pool_by_id(pool_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "map event node references missing random event pool '{}'",
                        pool_id
                    ))
                })?;
            pool.event_ids
                .iter()
                .map(|event_id| {
                    self.game_data
                        .random_event_data
                        .get_by_id(event_id)
                        .ok_or_else(|| {
                            GameError::InvalidStaticData(format!(
                                "random event pool '{}' references missing event '{}'",
                                pool_id, event_id
                            ))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            self.game_data
                .random_event_data
                .events
                .iter()
                .collect::<Vec<_>>()
        };
        let candidates = candidates
            .into_iter()
            .filter(|event| {
                !matches!(
                    event.inner_metadata,
                    crate::game::data::random_event_data::RandomEventInnerMetadata::Suppress(_)
                )
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            return Ok(None);
        }

        use rand::{Rng, SeedableRng};
        let seed = self.node_seed(node_id, 0x4D41_505F_4556_4E54);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let index = rng.gen_range(0..candidates.len());
        let event = candidates[index];

        Ok(Some(RandomEventOption::from(event)))
    }

    pub(super) fn try_enter_map_content_node(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        payload: &MapNodePayload,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> Result<Option<BehaviorResult>, GameError> {
        match category {
            MapNodeCategory::Shop => {
                let Some(shop) = self.resolve_map_shop(node_id, payload)? else {
                    return Ok(None);
                };
                let shop_uuid = shop.uuid;
                let shop_result = ShopEventOption {
                    id: shop.id.clone(),
                    name: shop.name.clone(),
                    uuid: shop.uuid,
                    shop_type: shop.shop_type,
                    can_reroll: shop.can_reroll,
                    visible_items: shop.visible_items.clone(),
                };
                self.state.selected_event =
                    Some(SelectedEvent::new(SelectedEventState::Shop(shop)));
                self.transition_to(GameState::InShop { shop_uuid })?;
                Ok(Some(BehaviorResult::ShopState {
                    shop: shop_result,
                    research_deliveries,
                }))
            }
            MapNodeCategory::Reward => {
                let Some(reward) = self.resolve_map_reward(node_id, payload)? else {
                    return Ok(None);
                };
                let reward_uuid = reward.stage_uuid;
                let result = self.reward_state_result_with_deliveries(&reward, research_deliveries);
                self.state.selected_event =
                    Some(SelectedEvent::new(SelectedEventState::Reward(reward)));
                self.transition_to(GameState::InReward { reward_uuid })?;
                Ok(Some(result))
            }
            MapNodeCategory::Event => {
                let Some(event) = self.resolve_map_random_event(node_id, payload)? else {
                    return Ok(None);
                };
                match event.inner_metadata.resolve(&self.game_data)? {
                    RandomEventTarget::Shop(shop) => {
                        let shop = ShopSessionState::from(shop);
                        let shop_uuid = shop.uuid;
                        let shop_result = ShopEventOption {
                            id: shop.id.clone(),
                            name: shop.name.clone(),
                            uuid: shop.uuid,
                            shop_type: shop.shop_type,
                            can_reroll: shop.can_reroll,
                            visible_items: shop.visible_items.clone(),
                        };
                        self.state.selected_event =
                            Some(SelectedEvent::new(SelectedEventState::Shop(shop)));
                        self.transition_to(GameState::InShop { shop_uuid })?;
                        Ok(Some(BehaviorResult::ShopState {
                            shop: shop_result,
                            research_deliveries,
                        }))
                    }
                    RandomEventTarget::Reward(reward) => {
                        if reward.tags.contains(&RewardTag::Forbidden)
                            || reward.tags.contains(&RewardTag::Experience)
                        {
                            return Err(GameError::InvalidStaticData(format!(
                                "event node reward '{}' uses a non-combat-only reward tag",
                                reward.id
                            )));
                        }
                        let reward = self.build_reward_session(
                            event.uuid,
                            RewardMode::ClaimAll,
                            vec![reward],
                            false,
                        );
                        let reward_uuid = reward.stage_uuid;
                        let result =
                            self.reward_state_result_with_deliveries(&reward, research_deliveries);
                        self.state.selected_event =
                            Some(SelectedEvent::new(SelectedEventState::Reward(reward)));
                        self.transition_to(GameState::InReward { reward_uuid })?;
                        Ok(Some(result))
                    }
                    RandomEventTarget::Suppress(_) => Err(GameError::InvalidAction),
                }
            }
            MapNodeCategory::Support => {
                let MapNodePayload::Support {
                    support_type,
                    support_mode,
                    choices,
                } = payload
                else {
                    return Ok(None);
                };
                let support_type = *support_type;
                let support_mode = *support_mode;
                let choices = if support_mode == SupportNodeMode::FullChoice {
                    Self::default_support_choices()
                } else {
                    choices.clone()
                };

                let support_session = if matches!(
                    support_mode,
                    SupportNodeMode::LimitedChoice | SupportNodeMode::FullChoice
                ) {
                    SupportSessionState::choice(node_id, support_mode, choices.clone())
                } else {
                    SupportSessionState::known(node_id, support_type)
                };
                let mut support_session = support_session;
                self.refresh_support_target_candidates(&mut support_session)?;
                let result = Self::support_state_result_with_deliveries(
                    &support_session,
                    research_deliveries,
                );
                self.state.selected_event = Some(SelectedEvent::new(SelectedEventState::Support(
                    support_session,
                )));
                if let Some(support) = self
                    .state
                    .selected_event
                    .as_ref()
                    .and_then(|selected| selected.as_support().ok())
                    .cloned()
                {
                    self.refresh_maintenance_action_gate(&support)?;
                }

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }
}
