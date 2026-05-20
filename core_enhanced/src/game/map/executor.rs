use serde::{Deserialize, Serialize};

use crate::game::map::session::NodeSession;
use crate::game::map::types::{MapNode, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload};

/// Thin boundary between map progression and node content.
///
/// Map progression only knows graph rules. This executor owns the first
/// interpretation step for a node definition and is the future home for
/// routing nodes into combat, shop, support, event, or reward sessions.
pub struct MapNodeExecutor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNodeEnterResult {
    pub node_id: MapNodeId,
    pub kind_id: MapNodeKindId,
    pub category: MapNodeCategory,
    pub payload: MapNodePayload,
    pub session: NodeSession,
}

impl MapNodeExecutor {
    pub fn enter(node: &MapNode) -> MapNodeEnterResult {
        let session = NodeSession::from(node);
        MapNodeEnterResult {
            node_id: node.id,
            kind_id: node.kind_id.clone(),
            category: node.category,
            payload: node.payload.clone(),
            session,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::session::NodeSessionKind;
    use crate::game::map::types::{MapNodeState, SupportNodeMode, SupportNodeType};
    use uuid::Uuid;

    #[test]
    fn enter_creates_a_session_without_interpreting_map_progression() {
        let node = MapNode {
            id: MapNodeId::new(Uuid::from_u128(77)),
            depth: 2,
            lane: 1,
            kind_id: MapNodeKindId::new("support_choice"),
            category: MapNodeCategory::Support,
            state: MapNodeState::Available,
            outgoing: vec![],
            payload: MapNodePayload::Support {
                support_type: SupportNodeType::Rest,
                support_mode: SupportNodeMode::LimitedChoice,
                choices: vec![SupportNodeType::Rest, SupportNodeType::Medical],
            },
        };

        let result = MapNodeExecutor::enter(&node);

        assert_eq!(result.node_id, node.id);
        assert_eq!(result.kind_id, node.kind_id);
        assert_eq!(result.category, MapNodeCategory::Support);
        assert_eq!(result.payload, node.payload);
        assert_eq!(result.session.node_id, node.id);
        assert_eq!(result.session.session_kind, NodeSessionKind::Support);
    }
}
