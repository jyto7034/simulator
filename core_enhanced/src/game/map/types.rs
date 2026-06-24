use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

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
    Shop,
    Boss,
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
    Hidden,
    Revealed,
    Available,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNode {
    pub id: MapNodeId,
    pub depth: u8,
    pub lane: u8,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub outgoing: Vec<MapNodeId>,
    #[serde(default)]
    pub payload: MapNodePayload,
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
            .any(|definition| definition.category != MapNodeCategory::Boss && definition.weight > 0)
        {
            return Err(
                "map node definitions must include at least one weighted non-boss entry"
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
            .filter(|definition| include_boss || definition.category != MapNodeCategory::Boss)
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMap {
    pub nodes: Vec<MapNode>,
    pub start_node_ids: Vec<MapNodeId>,
    pub boss_node_id: MapNodeId,
}

impl RunMap {
    pub fn node(&self, node_id: MapNodeId) -> Option<&MapNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn node_mut(&mut self, node_id: MapNodeId) -> Option<&mut MapNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn edge_dtos(&self) -> Vec<MapEdgeDto> {
        let mut edges = self
            .nodes
            .iter()
            .flat_map(|node| {
                node.outgoing
                    .iter()
                    .copied()
                    .map(|to| MapEdgeDto {
                        from_node_id: node.id,
                        to_node_id: to,
                    })
            })
            .collect::<Vec<_>>();
        edges.sort_by_key(|edge| {
            (
                edge.from_node_id.0.as_u128(),
                edge.to_node_id.0.as_u128(),
            )
        });
        edges
    }
}

pub fn map_template_id_for_act(act_index: u8) -> String {
    format!("act_{:02}_floor_a", u16::from(act_index) + 1)
}

pub fn map_slot_id_for_node(node: &MapNode) -> String {
    format!("depth_{:02}_lane_{:02}", node.depth, node.lane)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeDto {
    pub id: MapNodeId,
    pub slot_id: String,
    pub depth: u8,
    pub lane: u8,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub state: MapNodeState,
    pub payload: MapNodePayload,
}

impl From<&MapNode> for MapNodeDto {
    fn from(value: &MapNode) -> Self {
        Self {
            id: value.id,
            slot_id: map_slot_id_for_node(value),
            depth: value.depth,
            lane: value.lane,
            kind_id: value.kind_id.clone(),
            category: value.category,
            state: value.state,
            payload: value.payload.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapEdgeDto {
    pub from_node_id: MapNodeId,
    pub to_node_id: MapNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapViewDto {
    pub map_template_id: String,
    pub act_index: u8,
    pub max_acts: u8,
    pub nodes: Vec<MapNodeDto>,
    pub edges: Vec<MapEdgeDto>,
    pub current_node_id: Option<MapNodeId>,
    pub boss_node_id: MapNodeId,
}
