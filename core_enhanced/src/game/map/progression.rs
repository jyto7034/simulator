use serde::{Deserialize, Serialize};

use crate::game::determinism;
use crate::game::map::types::{MapNodeDto, MapNodeId, MapNodeState, MapViewDto, RunMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapProgressionError {
    MissingNode,
    NodeUnavailable,
    NoCurrentNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MapProgression {
    pub current_node_id: Option<MapNodeId>,
    pub available_node_ids: Vec<MapNodeId>,
    pub completed_node_ids: Vec<MapNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProgression {
    pub run_seed: u64,
    pub act_index: u8,
    pub max_acts: u8,
}

impl RunProgression {
    pub fn new(run_seed: u64, max_acts: u8) -> Self {
        assert!(max_acts > 0, "run must have at least one act");
        Self {
            run_seed,
            act_index: 0,
            max_acts,
        }
    }

    pub fn current_act_seed(&self) -> u64 {
        const ACT_NS: u64 = 0x4143_544D_4150; // "ACTMAP"
        determinism::seed_with_namespace(
            self.run_seed,
            ACT_NS ^ u64::from(self.act_index).wrapping_mul(0x9E37_79B9),
        )
    }

    pub fn advance_act(&mut self) -> bool {
        if self.act_index + 1 >= self.max_acts {
            return false;
        }
        self.act_index += 1;
        true
    }
}

impl MapProgression {
    pub fn from_map(map: &RunMap) -> Self {
        Self {
            current_node_id: None,
            available_node_ids: map.start_node_ids.clone(),
            completed_node_ids: Vec::new(),
        }
    }

    pub fn enter_node(
        &mut self,
        map: &mut RunMap,
        node_id: MapNodeId,
    ) -> Result<(), MapProgressionError> {
        if !self.available_node_ids.contains(&node_id) {
            return Err(MapProgressionError::NodeUnavailable);
        }

        let node = map
            .node_mut(node_id)
            .ok_or(MapProgressionError::MissingNode)?;
        node.state = MapNodeState::Revealed;
        self.current_node_id = Some(node_id);
        self.available_node_ids.retain(|id| *id == node_id);
        Ok(())
    }

    pub fn complete_current_node(&mut self, map: &mut RunMap) -> Result<bool, MapProgressionError> {
        let node_id = self
            .current_node_id
            .ok_or(MapProgressionError::NoCurrentNode)?;
        let outgoing = {
            let node = map
                .node_mut(node_id)
                .ok_or(MapProgressionError::MissingNode)?;
            node.state = MapNodeState::Completed;
            node.outgoing.clone()
        };

        if !self.completed_node_ids.contains(&node_id) {
            self.completed_node_ids.push(node_id);
        }
        self.completed_node_ids.sort_by_key(|id| id.0.as_u128());
        self.current_node_id = None;
        self.available_node_ids.clear();

        if node_id == map.boss_node_id {
            return Ok(true);
        }

        for next_id in outgoing {
            if self.completed_node_ids.contains(&next_id) {
                continue;
            }
            if let Some(node) = map.node_mut(next_id) {
                node.state = MapNodeState::Available;
                self.available_node_ids.push(next_id);
            }
        }
        self.available_node_ids.sort_by_key(|id| id.0.as_u128());
        self.available_node_ids.dedup();
        Ok(false)
    }

    pub fn view(&self, map: &RunMap, run: &RunProgression) -> MapViewDto {
        let mut nodes = map.nodes.iter().map(MapNodeDto::from).collect::<Vec<_>>();
        nodes.sort_by_key(|node| (node.depth, node.lane, node.id.0.as_u128()));

        MapViewDto {
            act_index: run.act_index,
            max_acts: run.max_acts,
            nodes,
            edges: map.edge_dtos(),
            current_node_id: self.current_node_id,
            available_node_ids: self.available_node_ids.clone(),
            completed_node_ids: self.completed_node_ids.clone(),
            boss_node_id: map.boss_node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::types::{
        MapNode, MapNodeCategory, MapNodeId, MapNodeKindId, MapNodePayload, RunMap,
    };
    use uuid::Uuid;

    fn node(id: u128, depth: u8, outgoing: Vec<MapNodeId>) -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(id)),
            depth,
            lane: 0,
            kind_id: MapNodeKindId::new(format!("node_{id}")),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Hidden,
            outgoing,
            payload: MapNodePayload::None,
        }
    }

    fn forked_map() -> RunMap {
        let start = MapNodeId::new(Uuid::from_u128(1));
        let left = MapNodeId::new(Uuid::from_u128(2));
        let right = MapNodeId::new(Uuid::from_u128(3));
        let boss = MapNodeId::new(Uuid::from_u128(4));

        RunMap {
            nodes: vec![
                node(1, 0, vec![left, right]),
                node(2, 1, vec![boss]),
                node(3, 1, vec![boss]),
                node(4, 2, vec![]),
            ],
            start_node_ids: vec![start],
            boss_node_id: boss,
        }
    }

    #[test]
    fn starts_from_map_start_nodes_without_mutating_node_states() {
        let map = forked_map();
        let start = map.start_node_ids[0];

        let progression = MapProgression::from_map(&map);

        assert_eq!(progression.current_node_id, None);
        assert_eq!(progression.available_node_ids, vec![start]);
        assert!(progression.completed_node_ids.is_empty());
        assert!(map
            .nodes
            .iter()
            .all(|node| node.state == MapNodeState::Hidden));
    }

    #[test]
    fn entering_and_completing_node_reveals_only_next_choices() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, start).unwrap();

        assert_eq!(progression.current_node_id, Some(start));
        assert_eq!(progression.available_node_ids, vec![start]);
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Revealed);

        let completed_boss = progression.complete_current_node(&mut map).unwrap();

        assert!(!completed_boss);
        assert_eq!(progression.current_node_id, None);
        assert_eq!(progression.completed_node_ids, vec![start]);
        assert_eq!(progression.available_node_ids.len(), 2);
        assert!(progression
            .available_node_ids
            .iter()
            .all(|id| map.node(*id).unwrap().state == MapNodeState::Available));
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Completed);
    }

    #[test]
    fn unavailable_or_missing_nodes_do_not_change_progression() {
        let mut map = forked_map();
        let mut progression = MapProgression::from_map(&map);
        let unavailable = MapNodeId::new(Uuid::from_u128(2));
        let missing = MapNodeId::new(Uuid::from_u128(999));

        assert_eq!(
            progression.enter_node(&mut map, unavailable),
            Err(MapProgressionError::NodeUnavailable)
        );
        assert_eq!(
            progression.enter_node(&mut map, missing),
            Err(MapProgressionError::NodeUnavailable)
        );

        assert_eq!(progression.current_node_id, None);
        assert_eq!(progression.available_node_ids, map.start_node_ids);
        assert!(progression.completed_node_ids.is_empty());
    }

    #[test]
    fn completing_boss_reports_act_boundary_and_keeps_no_available_nodes() {
        let mut map = forked_map();
        let boss = map.boss_node_id;
        map.start_node_ids = vec![boss];
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, boss).unwrap();
        let completed_boss = progression.complete_current_node(&mut map).unwrap();

        assert!(completed_boss);
        assert_eq!(progression.completed_node_ids, vec![boss]);
        assert!(progression.available_node_ids.is_empty());
        assert_eq!(map.node(boss).unwrap().state, MapNodeState::Completed);
    }
}
