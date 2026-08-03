use super::GameCore;
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::data::reward_data::RewardGrantKind;
use crate::game::data::visual_novel_data::{
    VisualNovelId, VisualNovelPresentationMode, VisualNovelSceneId,
};
use crate::game::determinism;
use crate::game::enums::RewardMode;
use crate::game::map::{MapNodeCategory, MapNodeId, MapNodePayload};
use crate::game::resources::{
    ActiveNodeContent, GameState, HeadquartersContactSessionState, MaintenanceSessionState,
    RewardSessionState, SavePointSessionState, ShopSessionState,
};
use crate::game::reward::RewardOption;
use crate::game::visual_novel::{
    VisualNovelSession, VisualNovelSessionPresentationMode, VisualNovelSource,
};

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

        // The wandering merchant is an Endless-only system actor. Keep the
        // ordinary default shop pool usable by Standard while replacing a
        // pooled Shop node with the policy-owned merchant on authored
        // Endless Floor 4+.
        let monolith_policy = &self.run_policy().monolith;
        let use_wandering_merchant = shop_id.is_none()
            && self.state.run.as_ref().is_some_and(|run| {
                run.run_progression.game_mode == crate::game::map::GameMode::Endless
                    && run.run_progression.floor_index().saturating_add(1)
                        >= monolith_policy.wandering_merchant_min_floor
            });
        if use_wandering_merchant {
            let shop = self
                .game_data
                .shop_data
                .get_by_id(&monolith_policy.wandering_merchant_shop_id)
                .ok_or_else(|| {
                    GameError::InvalidStaticData(format!(
                        "monolith policy references missing wandering merchant shop '{}'",
                        monolith_policy.wandering_merchant_shop_id
                    ))
                })?;
            return self.build_shop_session(node_id, shop).map(Some);
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
        let mut recruitment = self.run_state()?.recruitment.clone();
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
        let session = self
            .run_state()?
            .recruitment
            .active_session()
            .ok_or(GameError::InvalidAction)?;
        session
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
            .collect()
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
    ) -> Result<Option<BehaviorResult>, GameError> {
        match category {
            MapNodeCategory::Shop => {
                let Some(shop) = self.resolve_map_shop(node_id, payload)? else {
                    return Ok(None);
                };
                let shop_uuid = shop.uuid;
                let shop_result = Self::shop_event_option(&shop);
                self.run_state_mut()?
                    .shop_sessions
                    .insert(node_id, shop.clone());
                self.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
                self.transition_to(GameState::InShop { shop_uuid })?;
                self.start_monolith_exchange_if_available()?;
                Ok(Some(BehaviorResult::ShopState { shop: shop_result }))
            }
            MapNodeCategory::Reward => {
                let Some(reward) = self.resolve_map_reward(node_id, payload)? else {
                    return Ok(None);
                };
                let reward_uuid = reward.stage_uuid;
                let result = self.reward_state_result(&reward);
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
                let result = self.maintenance_state_result(&session);
                self.state.active_node_content = Some(ActiveNodeContent::Maintenance(session));
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
                let recruitment_candidates =
                    self.begin_recruitment_session(node_id, *candidate_count)?;
                let session = HeadquartersContactSessionState::new(node_id, shop_pool_id.clone());
                let result = BehaviorResult::HeadquartersContactState {
                    node_id,
                    options: session.options.clone(),
                    recruitment_candidates,
                    shop_pool_id: session.shop_pool_id.clone(),
                };
                self.state.active_node_content =
                    Some(ActiveNodeContent::HeadquartersContact(session));
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    fn start_monolith_exchange_if_available(&mut self) -> Result<(), GameError> {
        let threshold = self.run_policy().monolith.residual_exchange_threshold;
        let should_start = self.state.run.as_ref().is_some_and(|run| {
            run.monolith.exchange_state == crate::game::monolith::MonolithExchangeState::Available
                && run.monolith.residuals.len() >= threshold
        });
        if !should_start {
            return Ok(());
        }
        let visual_novel_id = self.run_policy().monolith.exchange_visual_novel_id.clone();
        let definition = self
            .game_data
            .visual_novel_data
            .get_by_id(&visual_novel_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "monolith exchange references missing visual novel '{visual_novel_id}'"
                ))
            })?;
        let presentation_mode = match definition.default_presentation_mode {
            VisualNovelPresentationMode::Cinematic => VisualNovelSessionPresentationMode::Cinematic,
            VisualNovelPresentationMode::Radio => VisualNovelSessionPresentationMode::Radio,
            VisualNovelPresentationMode::Compact => VisualNovelSessionPresentationMode::Compact,
            VisualNovelPresentationMode::NonBlocking => {
                VisualNovelSessionPresentationMode::NonBlocking
            }
        };
        let current_scene_id = VisualNovelSceneId::new(definition.entry_scene_id.as_str());
        self.run_state_mut()?
            .monolith
            .start_exchange(threshold)
            .map_err(|_| GameError::InvalidAction)?;
        let session = VisualNovelSession {
            session_id: self
                .state
                .uuid_manager
                .next_visual_novel_session()
                .to_string(),
            source: VisualNovelSource::Map,
            vn_id: VisualNovelId::new(visual_novel_id),
            current_scene_id,
            committed_choice_id: None,
            blocking: true,
            presentation_mode,
        };
        if self.state.visual_novel.active.is_none() {
            self.state.visual_novel.active = Some(session);
        } else {
            self.state.visual_novel.queue.insert(0, session);
        }
        Ok(())
    }
}
