use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::game::determinism;
use crate::game::map::types::{
    MapNodeCategory, MapNodeDto, MapNodeId, MapNodeState, MapNodeVisibility, MapViewDto, RunMap,
};

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
    pub game_mode: GameMode,
    pub mode_state: RunProgressionModeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunProgressionModeState {
    Standard { floor_index: u8, max_floors: u8 },
    Endless { floor_index: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Standard,
    Endless,
}

impl RunProgression {
    pub fn new(run_seed: u64, game_mode: GameMode, standard_floor_count: u8) -> Self {
        assert!(
            standard_floor_count > 0,
            "standard run must have at least one floor"
        );
        let mode_state = match game_mode {
            GameMode::Standard => RunProgressionModeState::Standard {
                floor_index: 0,
                max_floors: standard_floor_count,
            },
            GameMode::Endless => RunProgressionModeState::Endless { floor_index: 0 },
        };
        Self {
            run_seed,
            game_mode,
            mode_state,
        }
    }

    pub fn floor_index(&self) -> u32 {
        match self.mode_state {
            RunProgressionModeState::Standard { floor_index, .. } => u32::from(floor_index),
            RunProgressionModeState::Endless { floor_index } => floor_index,
        }
    }

    pub fn max_floors(&self) -> Option<u8> {
        match self.mode_state {
            RunProgressionModeState::Standard { max_floors, .. } => Some(max_floors),
            RunProgressionModeState::Endless { .. } => None,
        }
    }

    pub fn current_floor_seed(&self) -> u64 {
        const FLOOR_NS: u64 = 0x4143_544D_4150; // preserves the former ACTMAP seed namespace
        determinism::seed_with_namespace(
            self.run_seed,
            FLOOR_NS ^ u64::from(self.floor_index()).wrapping_mul(0x9E37_79B9),
        )
    }

    pub fn is_final_standard_floor(&self) -> bool {
        matches!(
            self.mode_state,
            RunProgressionModeState::Standard {
                floor_index,
                max_floors
            } if floor_index + 1 >= max_floors
        )
    }

    pub fn terminal_node_category(&self) -> MapNodeCategory {
        match self.game_mode {
            GameMode::Standard if self.is_final_standard_floor() => MapNodeCategory::Boss,
            GameMode::Standard | GameMode::Endless => MapNodeCategory::Gate,
        }
    }

    pub fn standard_gate_pre_terminal_kind_id(&self) -> Option<&'static str> {
        match self.game_mode {
            GameMode::Standard if !self.is_final_standard_floor() => Some("combat_elite"),
            GameMode::Standard | GameMode::Endless => None,
        }
    }

    pub fn advance_floor(&mut self) -> bool {
        match &mut self.mode_state {
            RunProgressionModeState::Standard {
                floor_index,
                max_floors,
            } => {
                if *floor_index + 1 >= *max_floors {
                    return false;
                }
                *floor_index += 1;
                true
            }
            RunProgressionModeState::Endless { floor_index } => {
                *floor_index = floor_index
                    .checked_add(1)
                    .expect("endless floor_index overflowed u32");
                true
            }
        }
    }
}

impl MapProgression {
    pub fn from_map(map: &RunMap) -> Self {
        let current_node_id = map
            .nodes
            .iter()
            .find(|node| {
                node.category == MapNodeCategory::Start && node.state == MapNodeState::Completed
            })
            .map(|node| node.id);
        Self {
            current_node_id,
            available_node_ids: map.start_node_ids.clone(),
            completed_node_ids: current_node_id.into_iter().collect(),
        }
    }

    pub fn is_node_selectable(&self, map: &RunMap, node_id: MapNodeId) -> bool {
        self.selectable_node_ids(map).contains(&node_id)
    }

    pub fn selectable_node_ids(&self, map: &RunMap) -> Vec<MapNodeId> {
        let Some(start) = self.current_node_id else {
            let mut selectable = map
                .start_node_ids
                .iter()
                .copied()
                .filter(|node_id| {
                    map.node(*node_id).is_some_and(|node| {
                        node.state == MapNodeState::Available
                            && node.visibility != MapNodeVisibility::Concealed
                    })
                })
                .collect::<Vec<_>>();
            selectable.sort_by_key(|id| id.0.as_u128());
            selectable.dedup();
            return selectable;
        };

        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([start]);
        let mut selectable = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            if !visited.insert(node_id) {
                continue;
            }
            let Some(node) = map.node(node_id) else {
                continue;
            };
            if node_id != start && node.state != MapNodeState::Completed {
                continue;
            }

            for neighbor_id in map.bidirectional_neighbors(node_id) {
                let Some(neighbor) = map.node(neighbor_id) else {
                    continue;
                };
                match neighbor.state {
                    MapNodeState::Completed => queue.push_back(neighbor_id),
                    MapNodeState::Available
                        if neighbor.visibility != MapNodeVisibility::Concealed =>
                    {
                        selectable.push(neighbor_id);
                    }
                    MapNodeState::Available | MapNodeState::Locked | MapNodeState::Unavailable => {}
                }
            }
        }

        selectable.sort_by_key(|id| id.0.as_u128());
        selectable.dedup();
        selectable
    }

    pub fn enter_node(
        &mut self,
        map: &mut RunMap,
        node_id: MapNodeId,
    ) -> Result<(), MapProgressionError> {
        if self.current_node_id == Some(node_id)
            && map.node(node_id).is_some_and(|node| {
                node.state == MapNodeState::Unavailable
                    && node.visibility == MapNodeVisibility::Revealed
            })
        {
            return Ok(());
        }

        if !self.is_node_selectable(map, node_id) {
            return Err(MapProgressionError::NodeUnavailable);
        }

        let node = map
            .node_mut(node_id)
            .ok_or(MapProgressionError::MissingNode)?;
        node.state = MapNodeState::Unavailable;
        node.visibility = MapNodeVisibility::Revealed;
        self.current_node_id = Some(node_id);
        self.available_node_ids.clear();
        Ok(())
    }

    pub fn complete_current_node(&mut self, map: &mut RunMap) -> Result<bool, MapProgressionError> {
        let node_id = self
            .current_node_id
            .ok_or(MapProgressionError::NoCurrentNode)?;
        if map.node(node_id).is_none() {
            return Err(MapProgressionError::MissingNode);
        }
        let adjacent_node_ids = map.bidirectional_neighbors(node_id);
        {
            let node = map
                .node_mut(node_id)
                .ok_or(MapProgressionError::MissingNode)?;
            node.state = MapNodeState::Completed;
            node.visibility = MapNodeVisibility::Revealed;
        }

        if !self.completed_node_ids.contains(&node_id) {
            self.completed_node_ids.push(node_id);
        }
        self.completed_node_ids.sort_by_key(|id| id.0.as_u128());
        self.available_node_ids.clear();

        if node_id == map.terminal_node_id {
            return Ok(true);
        }

        for next_id in adjacent_node_ids {
            if self.completed_node_ids.contains(&next_id) {
                continue;
            }
            if let Some(node) = map.node_mut(next_id) {
                node.state = MapNodeState::Unavailable;
            }
        }

        self.available_node_ids = Self::reachable_available_node_ids(map, node_id);
        for next_id in self.available_node_ids.iter().copied() {
            if let Some(node) = map.node_mut(next_id) {
                node.state = MapNodeState::Available;
                if node.visibility == MapNodeVisibility::Concealed {
                    node.visibility = MapNodeVisibility::Revealed;
                }
            }
        }
        self.available_node_ids.sort_by_key(|id| id.0.as_u128());
        self.available_node_ids.dedup();
        Ok(false)
    }

    fn reachable_available_node_ids(map: &RunMap, start: MapNodeId) -> Vec<MapNodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([start]);
        let mut available = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            if !visited.insert(node_id) {
                continue;
            }
            let Some(node) = map.node(node_id) else {
                continue;
            };
            if node.state != MapNodeState::Completed {
                continue;
            }

            for neighbor_id in map.bidirectional_neighbors(node_id) {
                let Some(neighbor) = map.node(neighbor_id) else {
                    continue;
                };
                match neighbor.state {
                    MapNodeState::Completed => queue.push_back(neighbor_id),
                    MapNodeState::Unavailable
                        if neighbor.visibility != MapNodeVisibility::Concealed =>
                    {
                        available.push(neighbor_id);
                    }
                    MapNodeState::Unavailable | MapNodeState::Available => {
                        available.push(neighbor_id);
                    }
                    MapNodeState::Locked => {}
                }
            }
        }

        available.sort_by_key(|id| id.0.as_u128());
        available.dedup();
        available
    }

    pub fn view(&self, map: &RunMap) -> MapViewDto {
        let mut nodes = map.nodes.iter().map(MapNodeDto::from).collect::<Vec<_>>();
        nodes.sort_by_key(|node| (node.depth, node.lane, node.id.0.as_u128()));

        MapViewDto {
            map_template_id: map.map_template_id.clone(),
            nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::types::{
        MapEdgeDirection, MapEdgeDto, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodePayload, MapSlotId, MapTemplateId, RunMap, DEFAULT_MAP_TEMPLATE_ID,
    };
    use uuid::Uuid;

    fn node(id: u128, depth: u8) -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(id)),
            depth,
            lane: 0,
            slot_id: MapSlotId::for_grid_position(depth, 0),
            kind_id: MapNodeKindId::new(format!("node_{id}")),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Unavailable,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::None,
            omen: None,
        }
    }

    fn edge(from_node_id: MapNodeId, to_node_id: MapNodeId) -> MapEdgeDto {
        MapEdgeDto {
            from_node_id,
            to_node_id,
            direction: MapEdgeDirection::Bidirectional,
        }
    }

    fn forked_map() -> RunMap {
        let start = MapNodeId::new(Uuid::from_u128(1));
        let left = MapNodeId::new(Uuid::from_u128(2));
        let right = MapNodeId::new(Uuid::from_u128(3));
        let boss = MapNodeId::new(Uuid::from_u128(4));

        let mut map = RunMap {
            map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
            edges: vec![
                edge(start, left),
                edge(start, right),
                edge(left, boss),
                edge(right, boss),
            ],
            nodes: vec![node(1, 0), node(2, 1), node(3, 1), node(4, 2)],
            start_node_ids: vec![start],
            terminal_node_id: boss,
        };
        let start_node = map.node_mut(start).unwrap();
        start_node.state = MapNodeState::Available;
        start_node.visibility = MapNodeVisibility::Revealed;
        let right_node = map.node_mut(right).unwrap();
        right_node.lane = 1;
        right_node.slot_id = MapSlotId::for_grid_position(1, 1);
        map
    }

    #[test]
    fn starts_from_map_start_nodes_without_mutating_node_states() {
        let map = forked_map();
        let start = map.start_node_ids[0];

        let progression = MapProgression::from_map(&map);

        assert_eq!(progression.current_node_id, None);
        assert_eq!(progression.available_node_ids, vec![start]);
        assert!(progression.completed_node_ids.is_empty());
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Available);
        assert_eq!(
            map.node(start).unwrap().visibility,
            MapNodeVisibility::Revealed
        );
    }

    #[test]
    fn entering_and_completing_node_reveals_only_next_choices() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, start).unwrap();

        assert_eq!(progression.current_node_id, Some(start));
        assert!(progression.available_node_ids.is_empty());
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Unavailable);
        assert_eq!(
            map.node(start).unwrap().visibility,
            MapNodeVisibility::Revealed
        );

        let completed_boss = progression.complete_current_node(&mut map).unwrap();

        assert!(!completed_boss);
        assert_eq!(progression.current_node_id, Some(start));
        assert_eq!(progression.completed_node_ids, vec![start]);
        assert_eq!(progression.available_node_ids.len(), 2);
        assert!(progression
            .available_node_ids
            .iter()
            .all(|id| map.node(*id).unwrap().state == MapNodeState::Available));
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Completed);
    }

    #[test]
    fn completed_nodes_are_transit_only_not_selectable_destinations() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, start).unwrap();
        progression.complete_current_node(&mut map).unwrap();

        assert!(!progression.is_node_selectable(&map, start));
        assert_eq!(
            progression.enter_node(&mut map, start),
            Err(MapProgressionError::NodeUnavailable)
        );
        assert!(progression
            .selectable_node_ids(&map)
            .iter()
            .all(|node_id| *node_id != start));
        assert!(!progression.selectable_node_ids(&map).is_empty());
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
    fn completing_terminal_boss_reports_boundary_and_keeps_no_available_nodes() {
        let mut map = forked_map();
        let boss = map.terminal_node_id;
        map.start_node_ids = vec![boss];
        let boss_node = map.node_mut(boss).unwrap();
        boss_node.state = MapNodeState::Available;
        boss_node.visibility = MapNodeVisibility::Revealed;
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, boss).unwrap();
        let completed_boss = progression.complete_current_node(&mut map).unwrap();

        assert!(completed_boss);
        assert_eq!(progression.completed_node_ids, vec![boss]);
        assert!(progression.available_node_ids.is_empty());
        assert_eq!(map.node(boss).unwrap().state, MapNodeState::Completed);
    }

    #[test]
    fn selectability_requires_available_and_not_concealed() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let progression = MapProgression::from_map(&map);

        assert!(progression.is_node_selectable(&map, start));

        map.node_mut(start).unwrap().visibility = MapNodeVisibility::Concealed;
        assert!(!progression.is_node_selectable(&map, start));

        map.node_mut(start).unwrap().visibility = MapNodeVisibility::Obscured;
        map.node_mut(start).unwrap().state = MapNodeState::Locked;
        assert!(!progression.is_node_selectable(&map, start));
    }

    #[test]
    fn current_unresolved_node_can_be_reentered_from_confirm_state() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let mut progression = MapProgression::from_map(&map);

        progression.enter_node(&mut map, start).unwrap();
        progression.enter_node(&mut map, start).unwrap();

        assert_eq!(progression.current_node_id, Some(start));
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Unavailable);
        assert_eq!(
            map.node(start).unwrap().visibility,
            MapNodeVisibility::Revealed
        );
    }

    #[test]
    fn forward_only_edges_do_not_allow_reverse_completed_transit() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let left = MapNodeId::new(Uuid::from_u128(2));
        let right = MapNodeId::new(Uuid::from_u128(3));

        let edge = map
            .edges
            .iter_mut()
            .find(|edge| edge.from_node_id == start && edge.to_node_id == left)
            .expect("start to left edge");
        edge.direction = MapEdgeDirection::ForwardOnly;

        let mut progression = MapProgression::from_map(&map);
        progression.enter_node(&mut map, start).unwrap();
        progression.complete_current_node(&mut map).unwrap();
        progression.enter_node(&mut map, left).unwrap();
        progression.complete_current_node(&mut map).unwrap();

        assert!(!progression.available_node_ids.contains(&right));
    }

    #[test]
    fn facility_contract_rejects_duplicate_or_mismatched_slots() {
        let mut duplicate = forked_map();
        let first_slot = duplicate.nodes[0].slot_id.clone();
        duplicate.nodes[1].slot_id = first_slot;
        assert!(duplicate
            .validate_facility_template_contract()
            .unwrap_err()
            .contains("duplicate map slot_id"));

        let mut mismatched = forked_map();
        mismatched.nodes[0].slot_id = MapSlotId::new("archive_room");
        assert!(mismatched
            .validate_facility_template_contract()
            .unwrap_err()
            .contains("depth/lane require"));
    }

    #[test]
    fn facility_contract_rejects_missing_edge_targets_and_locked_children() {
        let mut missing_edge = forked_map();
        missing_edge.edges.push(MapEdgeDto {
            from_node_id: missing_edge.nodes[0].id,
            to_node_id: MapNodeId::new(Uuid::from_u128(999)),
            direction: MapEdgeDirection::Bidirectional,
        });
        assert!(missing_edge
            .validate_facility_template_contract()
            .unwrap_err()
            .contains("references missing endpoint"));

        let mut locked_parent = forked_map();
        locked_parent.nodes[0].state = MapNodeState::Locked;
        assert!(locked_parent
            .validate_facility_template_contract()
            .unwrap_err()
            .contains("must be a leaf room"));
    }
}
