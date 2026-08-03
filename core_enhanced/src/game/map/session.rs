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
    use crate::game::map::types::{MapNodeState, MapNodeVisibility, MapSlotId};
    use uuid::Uuid;

    fn save_point_node() -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(42)),
            slot_id: MapSlotId::new("save_archive"),
            kind_id: MapNodeKindId::new("save_point"),
            category: MapNodeCategory::SavePoint,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::SavePoint,
            omen: None,
            combat_difficulty: None,
        }
    }

    #[test]
    fn node_session_preserves_identity_routing_category_and_payload() {
        let node = save_point_node();

        let session = NodeSession::from(&node);

        assert_eq!(session.node_id, node.id);
        assert_eq!(session.kind_id, node.kind_id);
        assert_eq!(session.category, node.category);
        assert_eq!(session.payload, node.payload);
    }
}
