use serde_json::json;

use crate::game::{
    behavior::GameError,
    enums::RewardMode,
    map::{
        HeadquartersContactOption, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodePayload, MapNodeState, NodeSession, SupportNodeMode, SupportNodeType,
    },
    resources::{ActiveNodeContent, GameState, HeadquartersContactSessionState},
};

use super::{AdminCommandOutput, GameCore, ADMIN_NODE_NS};

impl GameCore {
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
        let outgoing = self
            .state
            .run
            .as_ref()
            .ok_or(GameError::MissingResource("RunState"))?
            .map_progression
            .available_node_ids
            .clone();
        let node = MapNode {
            id: node_id,
            depth: 0,
            lane: 0,
            kind_id: MapNodeKindId::new(kind_id),
            category,
            state: MapNodeState::Revealed,
            outgoing,
            payload: payload.clone(),
        };

        let session = NodeSession::from(&node);
        let consumed_node_id = MapNodeId::new(self.state.uuid_manager.next(ADMIN_NODE_NS));
        debug_assert_eq!(consumed_node_id, node_id);
        let run = self.run_state_mut()?;
        run.map.nodes.push(node);
        run.map_progression.current_node_id = Some(node_id);
        run.map_progression.available_node_ids = vec![node_id];
        self.state.node_session = Some(session.clone());
        Ok(session)
    }

    pub(super) fn admin_enter_support(
        &mut self,
        support_type: SupportNodeType,
        support_mode: SupportNodeMode,
    ) -> Result<AdminCommandOutput, GameError> {
        let choices = match support_mode {
            SupportNodeMode::Known => Vec::new(),
            SupportNodeMode::LimitedChoice | SupportNodeMode::FullChoice => {
                Self::default_support_choices()
            }
        };
        let session = self.admin_prepare_fixture_node(
            MapNodeCategory::Support,
            format!("admin_support_{support_type:?}").to_lowercase(),
            MapNodePayload::Support {
                support_type,
                support_mode,
                choices: choices.clone(),
            },
        )?;

        let support = if support_mode == SupportNodeMode::Known {
            crate::game::resources::SupportSessionState::known(session.node_id, support_type)
        } else {
            crate::game::resources::SupportSessionState::choice(
                session.node_id,
                support_mode,
                choices,
            )
        };
        self.state.active_node_content = Some(ActiveNodeContent::Support(support));
        self.transition_to(GameState::InNode {
            node_id: session.node_id,
            kind_id: session.kind_id.clone(),
            category: MapNodeCategory::Support,
        })?;

        Ok(self.admin_output(
            "AdminEnteredSupport",
            json!({
                "node_id": session.node_id,
                "support_type": support_type,
                "support_mode": support_mode,
            }),
        ))
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
        let headquarters = HeadquartersContactSessionState {
            node_id: session.node_id,
            options: vec![
                HeadquartersContactOption::RecruitEmployee,
                HeadquartersContactOption::RequestEmergencySupplies,
                HeadquartersContactOption::OpenHeadquartersShop,
            ],
            recruitment_candidates: self
                .recruitment_candidates_for_node(session.node_id, candidate_count),
            shop_pool_id,
        };
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
