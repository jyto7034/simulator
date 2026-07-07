use tracing::info;

use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::reward_data::RewardGrantKind;
use crate::game::determinism;
use crate::game::enums::{RewardMode, ShopEventOption};
use crate::game::map::{MapNodeCategory, MapNodeId, MapNodePayload, SupportNodeMode};
use crate::game::resources::{
    ActiveNodeContent, GameState, HeadquartersContactSessionState, MaintenanceSessionState,
    RewardSessionState, ShopSessionState, SupportSessionState,
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
            .map(|run| run.run_progression.current_floor_seed())
            .unwrap_or(self.run_seed);
        determinism::seed_with_uuid(act_seed, namespace, node_id.0)
    }

    pub(super) fn resolve_map_shop(
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

    pub(super) fn recruitment_candidates_for_node(
        &self,
        node_id: MapNodeId,
        candidate_count: usize,
    ) -> Vec<crate::game::employee::StarterEmployeeCandidate> {
        let mut candidates = self.game_data.recruitment_employee_data.candidates.clone();
        if candidates.is_empty() {
            return candidates;
        }

        use rand::{seq::SliceRandom, SeedableRng};
        let seed = self.node_seed(node_id, 0x4851_5F52_4543_5255);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        candidates.shuffle(&mut rng);
        candidates.truncate(candidate_count.min(candidates.len()));
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
        candidates
    }

    pub(super) fn resolve_map_reward(
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
                let grant_kinds = reward.grant_kinds();
                !grant_kinds.contains(&RewardGrantKind::Experience)
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
                self.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
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
                self.state.active_node_content = Some(ActiveNodeContent::Reward(reward));
                self.transition_to(GameState::InReward { reward_uuid })?;
                Ok(Some(result))
            }
            MapNodeCategory::Event => {
                let MapNodePayload::Event { event_id } = payload else {
                    return Ok(None);
                };
                let event_id = event_id.clone().ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "event map node {:?} must define event_id",
                        node_id
                    ))
                })?;
                Ok(Some(self.enter_event_node(
                    node_id,
                    event_id,
                    research_deliveries,
                )?))
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
                let result = self
                    .support_state_result_with_deliveries(&support_session, research_deliveries);
                self.state.active_node_content = Some(ActiveNodeContent::Support(support_session));
                self.refresh_allowed_actions();

                Ok(Some(result))
            }
            MapNodeCategory::Maintenance => {
                let MapNodePayload::Maintenance = payload else {
                    return Ok(None);
                };
                let session = MaintenanceSessionState::new(node_id);
                let result =
                    self.maintenance_state_result_with_deliveries(&session, research_deliveries);
                self.state.active_node_content = Some(ActiveNodeContent::Maintenance(session));
                self.refresh_allowed_actions();
                Ok(Some(result))
            }
            MapNodeCategory::HeadquartersContact => {
                let MapNodePayload::HeadquartersContact {
                    shop_pool_id,
                    candidate_count,
                } = payload
                else {
                    return Ok(None);
                };
                let session = HeadquartersContactSessionState::new(
                    node_id,
                    self.recruitment_candidates_for_node(node_id, *candidate_count),
                    shop_pool_id.clone(),
                );
                let result = BehaviorResult::HeadquartersContactState {
                    node_id,
                    options: session.options.clone(),
                    recruitment_candidates: session.recruitment_candidates.clone(),
                    shop_pool_id: session.shop_pool_id.clone(),
                    research_deliveries,
                };
                self.state.active_node_content =
                    Some(ActiveNodeContent::HeadquartersContact(session));
                self.refresh_allowed_actions();
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }
}
