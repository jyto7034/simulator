use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::game::data::boss_omen_data::{BossOmenRevealLevel, BossOmenSourceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MapNodeId(pub Uuid);

impl MapNodeId {
    pub const fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(transparent)]
pub struct MapNodeKindId(pub String);

impl MapNodeKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeCategory {
    Start,
    Combat,
    Support,
    Maintenance,
    HeadquartersContact,
    Event,
    Shop,
    Boss,
    Gate,
    Reward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeadquartersContactOption {
    RecruitEmployee,
    RequestEmergencySupplies,
    OpenHeadquartersShop,
}

fn default_headquarters_candidate_count() -> usize {
    3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportNodeType {
    SavePoint,
    Rest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportNodeMode {
    Known,
    LimitedChoice,
    FullChoice,
}

fn default_support_node_mode() -> SupportNodeMode {
    SupportNodeMode::Known
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapNodePayload {
    None,
    Support {
        support_type: SupportNodeType,
        #[serde(default = "default_support_node_mode")]
        support_mode: SupportNodeMode,
        #[serde(default)]
        choices: Vec<SupportNodeType>,
    },
    Maintenance,
    HeadquartersContact {
        shop_pool_id: Option<String>,
        #[serde(default = "default_headquarters_candidate_count")]
        candidate_count: usize,
    },
    Event {
        event_id: Option<crate::game::data::event_data::EventId>,
    },
    Encounter {
        encounter_id: Option<String>,
    },
    Shop {
        shop_id: Option<String>,
        shop_pool_id: Option<String>,
    },
    Reward {
        reward_pool_id: Option<String>,
    },
}

impl Default for MapNodePayload {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeState {
    Available,
    Completed,
    Locked,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapNodeVisibility {
    Concealed,
    Obscured,
    Revealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapEdgeDirection {
    Bidirectional,
    ForwardOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapTemplateId(pub String);

impl MapTemplateId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapSlotId(pub String);

impl MapSlotId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn for_grid_position(depth: u8, lane: u8) -> Self {
        Self(format!("depth_{depth:02}_lane_{lane:02}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub const DEFAULT_MAP_TEMPLATE_ID: &str = "act_01_floor_a";

fn default_map_template_id() -> MapTemplateId {
    MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNode {
    pub id: MapNodeId,
    pub depth: u8,
    pub lane: u8,
    pub slot_id: MapSlotId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub visibility: MapNodeVisibility,
    #[serde(default)]
    pub payload: MapNodePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omen: Option<MapNodeOmenOverlayDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapNodeDefinition {
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub weight: u32,
    pub min_depth: u8,
    pub max_depth: Option<u8>,
    #[serde(default)]
    pub payload: MapNodePayload,
}

impl MapNodeDefinition {
    pub fn is_available_at_depth(&self, depth: u8) -> bool {
        depth >= self.min_depth && self.max_depth.is_none_or(|max_depth| depth <= max_depth)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeDefinitionDatabase {
    pub nodes: Vec<MapNodeDefinition>,
}

impl MapNodeDefinitionDatabase {
    pub fn from_ron_str(input: &str) -> Result<Self, String> {
        ron::de::from_str(input).map_err(|err| err.to_string())
    }

    /// Load the embedded map node definition catalog.
    ///
    /// This is a map-generation domain builtin, not an alternate
    /// `GameDataBase` live bundle loader. The map generator owns it because
    /// node definition authoring is used before a specific encounter/game-data
    /// domain is resolved.
    pub fn builtin() -> Self {
        let database = Self::from_ron_str(include_str!(
            "../../../../game_resources/data/map/node_definitions.ron"
        ))
        .expect("built-in map node definitions must be valid RON");
        database
            .validate_contract()
            .expect("built-in map node definitions must satisfy map generation contract");
        database
    }

    pub fn validate_contract(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("map node definitions must not be empty".to_string());
        }

        let mut seen = HashSet::new();
        for definition in &self.nodes {
            if definition.kind_id.as_str().is_empty() {
                return Err("map node definition kind_id must not be empty".to_string());
            }
            if !seen.insert(definition.kind_id.as_str()) {
                return Err(format!(
                    "duplicate map node definition kind_id '{}'",
                    definition.kind_id.as_str()
                ));
            }
        }

        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Boss)
        {
            return Err("map node definitions must include at least one Boss entry".to_string());
        }
        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Event)
        {
            return Err("map node definitions must include at least one Event entry".to_string());
        }
        if !self
            .nodes
            .iter()
            .any(|definition| definition.category == MapNodeCategory::Gate)
        {
            return Err("map node definitions must include at least one Gate entry".to_string());
        }
        if !self.nodes.iter().any(|definition| {
            !matches!(
                definition.category,
                MapNodeCategory::Boss | MapNodeCategory::Gate
            ) && definition.weight > 0
        }) {
            return Err(
                "map node definitions must include at least one weighted non-terminal entry"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn weighted_candidates(
        &self,
        depth: u8,
        category: Option<MapNodeCategory>,
        include_boss: bool,
    ) -> Vec<&MapNodeDefinition> {
        self.nodes
            .iter()
            .filter(|definition| definition.is_available_at_depth(depth))
            .filter(|definition| {
                include_boss
                    || !matches!(
                        definition.category,
                        MapNodeCategory::Boss | MapNodeCategory::Gate
                    )
            })
            .filter(|definition| category.is_none_or(|category| definition.category == category))
            .filter(|definition| definition.weight > 0)
            .collect()
    }

    pub fn boss_candidates(&self) -> Vec<&MapNodeDefinition> {
        self.nodes
            .iter()
            .filter(|definition| definition.category == MapNodeCategory::Boss)
            .collect()
    }

    pub fn category_candidates(&self, category: MapNodeCategory) -> Vec<&MapNodeDefinition> {
        self.nodes
            .iter()
            .filter(|definition| definition.category == category)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMap {
    #[serde(default = "default_map_template_id")]
    pub map_template_id: MapTemplateId,
    pub nodes: Vec<MapNode>,
    #[serde(default)]
    pub edges: Vec<MapEdgeDto>,
    pub start_node_ids: Vec<MapNodeId>,
    pub terminal_node_id: MapNodeId,
}

impl RunMap {
    pub fn node(&self, node_id: MapNodeId) -> Option<&MapNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn node_mut(&mut self, node_id: MapNodeId) -> Option<&mut MapNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn bidirectional_neighbors(&self, node_id: MapNodeId) -> Vec<MapNodeId> {
        let mut neighbors = Vec::new();
        for edge in &self.edges {
            if edge.from_node_id == node_id {
                neighbors.push(edge.to_node_id);
            }
            if edge.direction == MapEdgeDirection::Bidirectional && edge.to_node_id == node_id {
                neighbors.push(edge.from_node_id);
            }
        }
        neighbors.sort_by_key(|id| id.0.as_u128());
        neighbors.dedup();
        neighbors
    }

    pub fn edge_dtos(&self) -> Vec<MapEdgeDto> {
        let mut edges = self.edges.clone();
        edges.sort_by_key(|edge| (edge.from_node_id.0.as_u128(), edge.to_node_id.0.as_u128()));
        edges
    }

    pub fn validate_facility_template_contract(&self) -> Result<(), String> {
        if self.map_template_id.as_str().is_empty() {
            return Err("map_template_id must not be empty".to_string());
        }

        let mut node_ids = HashSet::new();
        let mut slot_ids = HashSet::new();
        for node in &self.nodes {
            if !node_ids.insert(node.id) {
                return Err(format!("duplicate map node id {:?}", node.id));
            }
            if node.slot_id.as_str().is_empty() {
                return Err(format!("map node {:?} has empty slot_id", node.id));
            }
            if !slot_ids.insert(node.slot_id.as_str()) {
                return Err(format!("duplicate map slot_id '{}'", node.slot_id.as_str()));
            }
            let expected_slot_id = MapSlotId::for_grid_position(node.depth, node.lane);
            if node.slot_id != expected_slot_id {
                return Err(format!(
                    "map node {:?} uses slot_id '{}' but depth/lane require '{}'",
                    node.id,
                    node.slot_id.as_str(),
                    expected_slot_id.as_str()
                ));
            }
        }

        for node in &self.nodes {
            if node.state == MapNodeState::Locked
                && self.edges.iter().any(|edge| edge.from_node_id == node.id)
            {
                return Err(format!("locked map node {:?} must be a leaf room", node.id));
            }
        }

        let mut edge_keys = HashSet::new();
        for edge in &self.edges {
            if !edge_keys.insert((edge.from_node_id.0.as_u128(), edge.to_node_id.0.as_u128())) {
                return Err(format!(
                    "duplicate map edge {:?} -> {:?}",
                    edge.from_node_id, edge.to_node_id
                ));
            }
            if !node_ids.contains(&edge.from_node_id) || !node_ids.contains(&edge.to_node_id) {
                return Err(format!(
                    "map edge {:?} -> {:?} references missing endpoint",
                    edge.from_node_id, edge.to_node_id
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeDto {
    pub id: MapNodeId,
    pub depth: u8,
    pub lane: u8,
    pub slot_id: MapSlotId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub visibility: MapNodeVisibility,
    pub payload: MapNodePayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omen: Option<MapNodeOmenOverlayDto>,
}

impl From<&MapNode> for MapNodeDto {
    fn from(value: &MapNode) -> Self {
        Self {
            id: value.id,
            depth: value.depth,
            lane: value.lane,
            slot_id: value.slot_id.clone(),
            kind_id: value.kind_id.clone(),
            category: value.category,
            state: value.state,
            visibility: value.visibility,
            payload: value.payload.clone(),
            omen: value.omen.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeOmenOverlayDto {
    pub present: bool,
    pub source_kind: BossOmenSourceKind,
    pub hint_id: String,
    pub reveal_level: BossOmenRevealLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapEdgeDto {
    pub from_node_id: MapNodeId,
    pub to_node_id: MapNodeId,
    pub direction: MapEdgeDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapViewDto {
    pub map_template_id: MapTemplateId,
    pub nodes: Vec<MapNodeDto>,
}
