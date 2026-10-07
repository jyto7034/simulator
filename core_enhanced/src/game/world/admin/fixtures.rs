use serde_json::json;

use crate::game::{
    behavior::GameError,
    enums::RewardMode,
    map::{
        FacilityMapGenerator, FacilityRunState, GameMode, MapNode, MapNodeCategory, MapNodeId,
        MapNodeKindId, MapNodePayload, MapNodeState, MapNodeVisibility, MapProgression, MapSlotId,
        NodeSession,
    },
    resources::{ActiveNodeContent, GameState, HeadquartersContactSessionState},
};

use super::{AdminCommandOutput, GameCore, ADMIN_NODE_NS};

impl GameCore {
    pub(super) fn admin_enter_event(
        &mut self,
        event_id: String,
    ) -> Result<AdminCommandOutput, GameError> {
        let event_id = crate::game::data::event_data::EventId::new(event_id);
        let entry_scene_id = self
            .game_data
            .event_data
            .get_by_id(event_id.as_str())
            .ok_or(GameError::MissingResource("Event"))?
            .entry_scene_id
            .clone();
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::Event,
            "admin_event",
            MapNodePayload::Event {
                event_id: Some(event_id.clone()),
            },
        )?;
        let event_session = crate::game::resources::EventSessionState::new(
            session.node_id,
            event_id.clone(),
            entry_scene_id,
        );
        self.run_state_mut()?
            .event_sessions
            .insert(session.node_id, event_session.clone());
        self.state.active_node_content = Some(ActiveNodeContent::Event(event_session));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::Event,
        })?;

        Ok(self.admin_output(
            "AdminEnteredEvent",
            json!({
                "node_id": session.node_id,
                "event_id": event_id,
            }),
        ))
    }

    fn admin_prepare_fixture_node(
        &mut self,
        category: MapNodeCategory,
        kind_id: impl Into<String>,
        payload: MapNodePayload,
    ) -> Result<NodeSession, GameError> {
        let node_id = self.admin_next_fixture_node_id();
        self.admin_prepare_fixture_node_with_id(node_id, category, kind_id, payload)
    }

    pub(super) fn admin_next_fixture_node_id(&self) -> MapNodeId {
        MapNodeId::new(self.state.uuid_manager.peek(ADMIN_NODE_NS))
    }

    fn admin_prepare_fixture_node_with_id(
        &mut self,
        node_id: MapNodeId,
        category: MapNodeCategory,
        kind_id: impl Into<String>,
        payload: MapNodePayload,
    ) -> Result<NodeSession, GameError> {
        self.state
            .run
            .as_ref()
            .ok_or(GameError::MissingResource("RunState"))?;
        let node = MapNode {
            id: node_id,
            slot_id: MapSlotId::new("admin_fixture"),
            kind_id: MapNodeKindId::new(kind_id),
            category,
            state: MapNodeState::Unavailable,
            visibility: MapNodeVisibility::Revealed,
            payload: payload.clone(),
            omen: None,
            combat_difficulty: None,
        };

        let session = NodeSession::from(&node);
        let consumed_node_id = MapNodeId::new(self.state.uuid_manager.next(ADMIN_NODE_NS));
        debug_assert_eq!(consumed_node_id, node_id);
        let run = self.run_state_mut()?;
        run.map_progression
            .activate_detached_fixture_node(&mut run.map, node);
        self.state.node_session = Some(session.clone());
        Ok(session)
    }

    pub(super) fn admin_enter_save_point(&mut self) -> Result<AdminCommandOutput, GameError> {
        let session = self.admin_prepare_save_point_fixture_node()?;

        let save_point = crate::game::resources::SavePointSessionState::new(session.node_id);
        self.state.active_node_content = Some(ActiveNodeContent::SavePoint(save_point));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::SavePoint,
        })?;

        Ok(self.admin_output(
            "AdminEnteredSavePoint",
            json!({ "node_id": session.node_id }),
        ))
    }

    fn admin_prepare_save_point_fixture_node(&mut self) -> Result<NodeSession, GameError> {
        let (run_seed, game_mode, current_template_id) = {
            let run = self
                .state
                .run
                .as_ref()
                .ok_or(GameError::MissingResource("RunState"))?;
            (
                run.run_progression.run_seed,
                run.run_progression.game_mode,
                run.map.map_template_id.clone(),
            )
        };
        let current_template_supports_save_point = self
            .game_data
            .map_catalogs
            .facility_templates
            .template(&current_template_id)
            .is_some_and(|template| {
                template.slots.iter().any(|slot| {
                    slot.allowed_categories
                        .contains(&MapNodeCategory::SavePoint)
                })
            });

        if !current_template_supports_save_point {
            let fixture_floor_index = match game_mode {
                GameMode::Standard => 1,
                GameMode::Endless => 0,
            };
            let generated = FacilityMapGenerator::generate(
                run_seed,
                game_mode,
                fixture_floor_index,
                &FacilityRunState::default(),
                &self.game_data.map_catalogs,
                self.run_policy(),
                self.state
                    .run_power_authority
                    .current_floor_power_anchor
                    .max(1),
            )
            .map_err(GameError::InvalidStaticData)?;
            let progression = MapProgression::from_map(&generated.map);
            let run = self.run_state_mut()?;
            run.map = generated.map;
            run.map_progression = progression;
            run.facility_run_state = generated.facility_run_state;
            run.event_sessions.clear();
            run.shop_sessions.clear();
            run.combat_previews.clear();
            run.abnormality_attempts.clear();
            run.monolith.clear_floor_local_remanifestations();
            run.boss_omen = Default::default();
            run.combat_recovery_restriction = None;
        }

        let target_slot_id = {
            let run = self.run_state()?;
            self.game_data
                .map_catalogs
                .facility_templates
                .template(&run.map.map_template_id)
                .and_then(|template| {
                    template
                        .slots
                        .iter()
                        .find(|slot| {
                            slot.allowed_categories
                                .contains(&MapNodeCategory::SavePoint)
                        })
                        .map(|slot| slot.id.clone())
                })
                .ok_or_else(|| {
                    GameError::InvalidStaticData(
                        "admin SavePoint fixture could not resolve an authored SavePoint slot"
                            .to_string(),
                    )
                })?
        };
        let node_id = self
            .run_state()?
            .map
            .nodes
            .iter()
            .find(|node| node.slot_id == target_slot_id)
            .map(|node| node.id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(
                    "admin SavePoint fixture map is missing its authored SavePoint slot"
                        .to_string(),
                )
            })?;

        let session = {
            let run = self.run_state_mut()?;
            let node = run
                .map
                .node_mut(node_id)
                .ok_or(GameError::MissingResource("MapNode"))?;
            node.kind_id = MapNodeKindId::new("save_point");
            node.category = MapNodeCategory::SavePoint;
            node.payload = MapNodePayload::SavePoint;
            node.omen = None;
            node.combat_difficulty = None;
            run.map_progression
                .activate_authored_fixture_node(&mut run.map, node_id)
                .map_err(|_| GameError::MissingResource("MapNode"))?;
            let session = NodeSession::from(
                run.map
                    .node(node_id)
                    .ok_or(GameError::MissingResource("MapNode"))?,
            );
            run.event_sessions.remove(&node_id);
            run.shop_sessions.remove(&node_id);
            run.combat_previews.remove(&node_id);
            run.abnormality_attempts.remove(&node_id);
            session
        };
        self.state.node_session = Some(session.clone());
        Ok(session)
    }

    pub(super) fn admin_enter_maintenance(&mut self) -> Result<AdminCommandOutput, GameError> {
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::Maintenance,
            "admin_maintenance",
            MapNodePayload::Maintenance,
        )?;
        let maintenance = crate::game::resources::MaintenanceSessionState::new(session.node_id);
        self.state.active_node_content = Some(ActiveNodeContent::Maintenance(maintenance));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::Maintenance,
        })?;

        Ok(self.admin_output(
            "AdminEnteredMaintenance",
            json!({ "node_id": session.node_id }),
        ))
    }

    pub(super) fn admin_enter_shop(
        &mut self,
        shop_id: Option<String>,
        shop_pool_id: Option<String>,
    ) -> Result<AdminCommandOutput, GameError> {
        let payload = MapNodePayload::Shop {
            shop_id,
            shop_pool_id,
        };
        let node_id = self.admin_next_fixture_node_id();
        let Some(shop) = self.resolve_map_shop(node_id, &payload)? else {
            return Err(GameError::InvalidStaticData(
                "admin shop command could not resolve a shop".to_string(),
            ));
        };
        let session = self.admin_prepare_fixture_node_with_id(
            node_id,
            MapNodeCategory::Shop,
            "admin_shop",
            payload,
        )?;
        let shop_uuid = shop.uuid;
        self.state.active_node_content = Some(ActiveNodeContent::Shop(shop));
        self.transition_to(GameState::InShop { shop_uuid })?;

        Ok(self.admin_output(
            "AdminEnteredShop",
            json!({
                "node_id": session.node_id,
                "shop_uuid": shop_uuid,
            }),
        ))
    }

    pub(super) fn admin_enter_reward(
        &mut self,
        reward_pool_id: Option<String>,
        mode: RewardMode,
        can_skip: bool,
    ) -> Result<AdminCommandOutput, GameError> {
        let payload = MapNodePayload::Reward { reward_pool_id };
        let node_id = self.admin_next_fixture_node_id();
        let Some(mut reward) = self.resolve_map_reward(node_id, &payload)? else {
            return Err(GameError::InvalidStaticData(
                "admin reward command could not resolve a reward".to_string(),
            ));
        };
        let session = self.admin_prepare_fixture_node_with_id(
            node_id,
            MapNodeCategory::Reward,
            "admin_reward",
            payload,
        )?;
        reward.mode = mode;
        reward.can_skip = can_skip;
        let reward_uuid = reward.stage_uuid;
        self.state.active_node_content = Some(ActiveNodeContent::Reward(reward));
        self.transition_to(GameState::InReward { reward_uuid })?;

        Ok(self.admin_output(
            "AdminEnteredReward",
            json!({
                "node_id": session.node_id,
                "reward_uuid": reward_uuid,
                "mode": mode,
                "can_skip": can_skip,
            }),
        ))
    }

    pub(super) fn admin_enter_headquarters_contact(
        &mut self,
        shop_pool_id: Option<String>,
        candidate_count: usize,
    ) -> Result<AdminCommandOutput, GameError> {
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::HeadquartersContact,
            "admin_headquarters_contact",
            MapNodePayload::HeadquartersContact {
                shop_pool_id: shop_pool_id.clone(),
                candidate_count,
            },
        )?;
        self.begin_recruitment_session(session.node_id, candidate_count)?;
        let headquarters = HeadquartersContactSessionState::new(session.node_id, shop_pool_id);
        self.state.active_node_content = Some(ActiveNodeContent::HeadquartersContact(headquarters));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::HeadquartersContact,
        })?;

        Ok(self.admin_output(
            "AdminEnteredHeadquartersContact",
            json!({ "node_id": session.node_id }),
        ))
    }
}
