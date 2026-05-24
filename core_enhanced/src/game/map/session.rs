use serde::{Deserialize, Serialize};

use crate::game::map::types::{MapNode, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeSessionKind {
    Start,
    Combat,
    Boss,
    Support,
    HeadquartersContact,
    Shop,
    Reward,
}

impl From<MapNodeCategory> for NodeSessionKind {
    fn from(value: MapNodeCategory) -> Self {
        match value {
            MapNodeCategory::Start => Self::Start,
            MapNodeCategory::Combat => Self::Combat,
            MapNodeCategory::Boss => Self::Boss,
            MapNodeCategory::Support => Self::Support,
            MapNodeCategory::HeadquartersContact => Self::HeadquartersContact,
            MapNodeCategory::Shop => Self::Shop,
            MapNodeCategory::Reward => Self::Reward,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSession {
    pub node_id: MapNodeId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub session_kind: NodeSessionKind,
    pub payload: MapNodePayload,
}

impl From<&MapNode> for NodeSession {
    fn from(node: &MapNode) -> Self {
        Self {
            node_id: node.id,
            kind_id: node.kind_id.clone(),
            category: node.category,
            session_kind: NodeSessionKind::from(node.category),
            payload: node.payload.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::types::{MapNodeState, SupportNodeMode, SupportNodeType};
    use uuid::Uuid;

    fn support_node() -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(42)),
            depth: 1,
            lane: 2,
            kind_id: MapNodeKindId::new("support_rest"),
            category: MapNodeCategory::Support,
            state: MapNodeState::Available,
            outgoing: vec![],
            payload: MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
        }
    }

    #[test]
    fn session_kind_is_the_domain_mapping_for_every_map_category() {
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Start),
            NodeSessionKind::Start
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Combat),
            NodeSessionKind::Combat
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Boss),
            NodeSessionKind::Boss
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Support),
            NodeSessionKind::Support
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::HeadquartersContact),
            NodeSessionKind::HeadquartersContact
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Shop),
            NodeSessionKind::Shop
        );
        assert_eq!(
            NodeSessionKind::from(MapNodeCategory::Reward),
            NodeSessionKind::Reward
        );
    }

    #[test]
    fn node_session_preserves_identity_routing_category_and_payload() {
        let node = support_node();

        let session = NodeSession::from(&node);

        assert_eq!(session.node_id, node.id);
        assert_eq!(session.kind_id, node.kind_id);
        assert_eq!(session.category, node.category);
        assert_eq!(session.session_kind, NodeSessionKind::Support);
        assert_eq!(session.payload, node.payload);
    }
}
