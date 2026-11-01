use super::{shop::ShopSessionCommitMode, GameCore};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::reward_data::RewardGrantKind;
use crate::game::determinism;
use crate::game::enums::RewardMode;
use crate::game::map::{MapNodeCategory, MapNodeId, MapNodePayload};
use crate::game::resources::{
    ActiveNodeContent, GameState, HeadquartersContactSessionState, MaintenanceSessionState,
    RewardSessionState, SavePointSessionState, ShopSessionState,
};
use crate::game::reward::RewardOption;

impl GameCore {
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

        if let Some(session) = self
            .state
            .run
            .as_ref()
            .and_then(|run| run.shop_sessions.get(&node_id))
        {
            return Ok(Some(session.clone()));
        }

        if let Some(shop_id) = shop_id {
            let shop = self.game_data.shop_data.get_by_id(shop_id).ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "map shop node references missing shop id '{}'",
                    shop_id
                ))
            })?;
            return self.build_shop_session(node_id, shop).map(Some);
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

        self.build_shop_session(node_id, shop).map(Some)
    }

    pub(super) fn begin_recruitment_session(
        &mut self,
        node_id: MapNodeId,
        candidate_count: usize,
    ) -> Result<Vec<crate::game::employee::EmployeeDefinition>, GameError> {
        let run = self.run_state()?;
        let candidate_count = if run.run_progression.game_mode
            == crate::game::map::GameMode::Expedition
            && run.run_progression.floor_index().saturating_add(1)
                > self.run_policy().headquarters.recruit_max_depth
        {
            0
        } else {
            candidate_count
        };
        let mut recruitment = run.recruitment.clone();
        let definition_ids = recruitment
            .begin_session(node_id, candidate_count)?
            .employee_definition_ids
            .clone();
        let candidates = definition_ids
            .iter()
            .map(|definition_id| {
                self.game_data
                    .employee_definition_data
                    .get_by_id(definition_id)
                    .cloned()
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "recruitment queue references missing employee definition '{}'",
                            definition_id
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            self.roster()?,
        )?;
        self.run_state_mut()?.recruitment = recruitment;
        Ok(candidates)
    }

    pub(super) fn active_recruitment_candidates(
        &self,
    ) -> Result<Vec<crate::game::employee::EmployeeDefinition>, GameError> {
        if !self.paid_recruitment_is_available() {
            return Ok(Vec::new());
        }
        let session = self
            .run_state()?
            .recruitment
            .active_session()
            .ok_or(GameError::InvalidAction)?;
        let candidates = session
            .employee_definition_ids
            .iter()
            .map(|definition_id| {
                self.game_data
                    .employee_definition_data
                    .get_by_id(definition_id)
                    .cloned()
                    .ok_or_else(|| {
                        GameError::InvalidStaticData(format!(
                            "recruitment session references missing employee definition '{}'",
                            definition_id
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(candidates.into_iter()
            .filter(|definition| self.recruitment_definition_is_deployable(&definition.employee_definition_id))
            .collect())
    }

    pub(super) fn resolve_map_reward(
        &self,
        node_id: MapNodeId,
        payload: &MapNodePayload,
    ) -> Result<Option<RewardSessionState>, GameError> {
        if self.basic_relic_activity(node_id) == Some(crate::game::map::RewardActivityKind::Gambling) {
            return self.resolve_dealer_reward(node_id).map(Some);
        }
        if let Some(kind) = self.basic_relic_activity(node_id) {
            return self.resolve_basic_relic_reward(node_id, kind).map(Some);
        }
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
    ) -> Result<Option<BehaviorResult>, GameError> {
        match category {
            MapNodeCategory::Shop => {
                let Some(shop) = self.resolve_map_shop(node_id, payload)? else {
                    return Ok(None);
                };
                let shop_uuid = shop.uuid;
                let shop_result = Self::shop_event_option(&shop);
                self.commit_shop_session(shop, ShopSessionCommitMode::RequireRun)?;
                self.transition_to(GameState::InShop { shop_uuid })?;
                Ok(Some(BehaviorResult::ShopState { shop: shop_result }))
            }
            MapNodeCategory::Reward => {
                if self.basic_relic_activity(node_id) == Some(crate::game::map::RewardActivityKind::StressCombat) {
                    return self.start_stress_combat(node_id).map(Some);
                }
                let Some(reward) = self.resolve_map_reward(node_id, payload)? else {
                    return Ok(None);
                };
                let reward_uuid = reward.stage_uuid;
                let result = self.reward_state_result(&reward);
                self.state.active_node_content = Some(ActiveNodeContent::Reward(reward));
                self.transition_to(GameState::InReward { reward_uuid })?;
                self.start_dealer_dialogue()?;
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
                Ok(Some(self.enter_event_node(node_id, event_id)?))
            }
            MapNodeCategory::SavePoint => {
                let MapNodePayload::SavePoint = payload else {
                    return Ok(None);
                };
                let save_point = SavePointSessionState::new(node_id);
                let result = self.save_point_state_result(&save_point)?;
                self.state.active_node_content = Some(ActiveNodeContent::SavePoint(save_point));

                Ok(Some(result))
            }
            MapNodeCategory::Maintenance => {
                let MapNodePayload::Maintenance = payload else {
                    return Ok(None);
                };
                let session = MaintenanceSessionState::new(node_id);
                self.state.active_node_content =
                    Some(ActiveNodeContent::Maintenance(session.clone()));
                let result = self.maintenance_state_result(&session);
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
                self.begin_recruitment_session(node_id, *candidate_count)?;
                let session = HeadquartersContactSessionState::new(node_id, shop_pool_id.clone());
                self.state.active_node_content = Some(ActiveNodeContent::HeadquartersContact(session));
                Ok(Some(self.headquarters_contact_state_result()?))
            }
            _ => Ok(None),
        }
    }


}
