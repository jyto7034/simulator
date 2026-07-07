use serde::{Deserialize, Serialize};

use crate::game::map::types::{MapNode, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload};

/// Current node session snapshot.
///
/// This is intentionally part of the run snapshot wire contract. It carries the
/// selected node identity/routing category plus the authored payload needed to
/// resume or confirm the current node; it is not a hidden runtime-only state bag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSession {
    pub node_id: MapNodeId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub payload: MapNodePayload,
}

impl From<&MapNode> for NodeSession {
    fn from(node: &MapNode) -> Self {
        Self {
            node_id: node.id,
            kind_id: node.kind_id.clone(),
            category: node.category,
            payload: node.payload.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::types::{
        MapNodeState, MapNodeVisibility, MapSlotId, SupportNodeMode, SupportNodeType,
    };
    use uuid::Uuid;

    fn support_node() -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(42)),
            depth: 1,
            lane: 2,
            slot_id: MapSlotId::for_grid_position(1, 2),
            kind_id: MapNodeKindId::new("support_rest"),
            category: MapNodeCategory::Support,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::Known,
                choices: vec![],
            },
            omen: None,
        }
    }

    #[test]
    fn node_session_preserves_identity_routing_category_and_payload() {
        let node = support_node();

        let session = NodeSession::from(&node);

        assert_eq!(session.node_id, node.id);
        assert_eq!(session.kind_id, node.kind_id);
        assert_eq!(session.category, node.category);
        assert_eq!(session.payload, node.payload);
    }
}
