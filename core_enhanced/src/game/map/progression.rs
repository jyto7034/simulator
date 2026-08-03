use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

use crate::game::determinism;
use crate::game::map::types::{
    MapNodeCategory, MapNodeDto, MapNodeId, MapNodeState, MapNodeVisibility, MapViewDto, RunMap,
};
use crate::game::map::{FacilityTemplate, FacilityTerminalResolution};

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

    pub fn is_node_selectable(
        &self,
        map: &RunMap,
        template: &FacilityTemplate,
        node_id: MapNodeId,
    ) -> bool {
        self.selectable_node_ids(map, template).contains(&node_id)
    }

    pub fn selectable_node_ids(&self, map: &RunMap, template: &FacilityTemplate) -> Vec<MapNodeId> {
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

            for neighbor_id in map.neighbor_node_ids(template, node_id) {
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
        template: &FacilityTemplate,
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

        if !self.is_node_selectable(map, template, node_id) {
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

    pub fn complete_current_node(
        &mut self,
        map: &mut RunMap,
        template: &FacilityTemplate,
    ) -> Result<Option<FacilityTerminalResolution>, MapProgressionError> {
        let node_id = self
            .current_node_id
            .ok_or(MapProgressionError::NoCurrentNode)?;
        if map.node(node_id).is_none() {
            return Err(MapProgressionError::MissingNode);
        }
        let adjacent_node_ids = map.neighbor_node_ids(template, node_id);
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

        if let Some(resolution) = map.terminal_resolution(node_id) {
            let other_terminal_ids = map
                .terminal_node_ids()
                .into_iter()
                .filter(|terminal_id| *terminal_id != node_id)
                .collect::<Vec<_>>();
            for terminal_id in other_terminal_ids {
                if let Some(terminal) = map.node_mut(terminal_id) {
                    terminal.state = MapNodeState::Locked;
                }
            }
            return Ok(Some(resolution));
        }

        for next_id in adjacent_node_ids {
            if self.completed_node_ids.contains(&next_id) {
                continue;
            }
            if let Some(node) = map.node_mut(next_id) {
                node.state = MapNodeState::Unavailable;
            }
        }

        self.available_node_ids = Self::reachable_available_node_ids(map, template, node_id);
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
        Ok(None)
    }

    fn reachable_available_node_ids(
        map: &RunMap,
        template: &FacilityTemplate,
        start: MapNodeId,
    ) -> Vec<MapNodeId> {
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

            for neighbor_id in map.neighbor_node_ids(template, node_id) {
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

    pub(crate) fn refresh_available_nodes(
        &mut self,
        map: &mut RunMap,
        template: &FacilityTemplate,
    ) {
        for node in &mut map.nodes {
            if node.state == MapNodeState::Available {
                node.state = MapNodeState::Unavailable;
            }
        }
        self.available_node_ids.clear();
        let Some(current_node_id) = self.current_node_id else {
            return;
        };
        self.available_node_ids =
            Self::reachable_available_node_ids(map, template, current_node_id);
        for node_id in self.available_node_ids.iter().copied() {
            if let Some(node) = map.node_mut(node_id) {
                node.state = MapNodeState::Available;
            }
        }
    }

    pub fn view(&self, map: &RunMap, template: &FacilityTemplate) -> MapViewDto {
        let mut nodes = map.nodes.iter().map(MapNodeDto::from).collect::<Vec<_>>();
        let selectable = self
            .selectable_node_ids(map, template)
            .into_iter()
            .collect::<HashSet<_>>();
        for node in &mut nodes {
            if node.state == MapNodeState::Available && !selectable.contains(&node.id) {
                node.state = MapNodeState::Unavailable;
            }
        }
        nodes.sort_by(|left, right| {
            left.slot_id
                .as_str()
                .cmp(right.slot_id.as_str())
                .then_with(|| left.id.0.as_u128().cmp(&right.id.0.as_u128()))
        });

        MapViewDto {
            map_template_id: map.map_template_id.clone(),
            facility_template_version: map.facility_template_version,
            nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::{
        FacilityConnection, FacilityRoomSlot, FacilitySlotRole, FacilityTemplate,
        FacilityTemplateKind, MapEdgeDirection, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodePayload, MapSlotId, MapTemplateId, RunMap,
    };
    use uuid::Uuid;

    fn node(id: u128, slot_id: &str) -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(id)),
            slot_id: MapSlotId::new(slot_id),
            kind_id: MapNodeKindId::new(format!("node_{id}")),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Unavailable,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::None,
            omen: None,
            combat_difficulty: None,
        }
    }

    fn forked_template() -> FacilityTemplate {
        FacilityTemplate {
            id: MapTemplateId::new("test_fork"),
            version: crate::game::map::FACILITY_TEMPLATE_VERSION,
            kind: FacilityTemplateKind::Exploration,
            tags: vec![],
            slots: vec![
                FacilityRoomSlot {
                    id: MapSlotId::new("start"),
                    role: FacilitySlotRole::Entry,
                    allowed_categories: vec![MapNodeCategory::Combat],
                    tags: vec![],
                },
                FacilityRoomSlot {
                    id: MapSlotId::new("left"),
                    role: FacilitySlotRole::Ordinary,
                    allowed_categories: vec![MapNodeCategory::Combat],
                    tags: vec![],
                },
                FacilityRoomSlot {
                    id: MapSlotId::new("right"),
                    role: FacilitySlotRole::Ordinary,
                    allowed_categories: vec![MapNodeCategory::Combat],
                    tags: vec![],
                },
                FacilityRoomSlot {
                    id: MapSlotId::new("boss"),
                    role: FacilitySlotRole::Terminal(FacilityTerminalResolution::RunFinalBoss),
                    allowed_categories: vec![MapNodeCategory::Boss],
                    tags: vec![],
                },
            ],
            connections: [
                ("start", "left"),
                ("start", "right"),
                ("left", "boss"),
                ("right", "boss"),
            ]
            .into_iter()
            .map(|(from, to)| FacilityConnection {
                from_slot_id: MapSlotId::new(from),
                to_slot_id: MapSlotId::new(to),
                direction: MapEdgeDirection::Bidirectional,
                reveal_with_slot_id: None,
            })
            .collect(),
            manifestation_rules: vec![],
        }
    }

    fn forked_map() -> RunMap {
        let start = MapNodeId::new(Uuid::from_u128(1));
        let boss = MapNodeId::new(Uuid::from_u128(4));

        let mut map = RunMap {
            map_template_id: MapTemplateId::new("test_fork"),
            facility_template_version: crate::game::map::FACILITY_TEMPLATE_VERSION,
            nodes: vec![
                node(1, "start"),
                node(2, "left"),
                node(3, "right"),
                node(4, "boss"),
            ],
            start_node_ids: vec![start],
            terminals: vec![crate::game::map::MapTerminalNode {
                node_id: boss,
                resolution: FacilityTerminalResolution::RunFinalBoss,
            }],
        };
        let start_node = map.node_mut(start).unwrap();
        start_node.state = MapNodeState::Available;
        start_node.visibility = MapNodeVisibility::Revealed;
        map.node_mut(boss).unwrap().category = MapNodeCategory::Boss;
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

        progression
            .enter_node(&mut map, &forked_template(), start)
            .unwrap();

        assert_eq!(progression.current_node_id, Some(start));
        assert!(progression.available_node_ids.is_empty());
        assert_eq!(map.node(start).unwrap().state, MapNodeState::Unavailable);
        assert_eq!(
            map.node(start).unwrap().visibility,
            MapNodeVisibility::Revealed
        );

        let terminal_resolution = progression
            .complete_current_node(&mut map, &forked_template())
            .unwrap();

        assert_eq!(terminal_resolution, None);
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

        progression
            .enter_node(&mut map, &forked_template(), start)
            .unwrap();
        progression
            .complete_current_node(&mut map, &forked_template())
            .unwrap();

        assert!(!progression.is_node_selectable(&map, &forked_template(), start));
        assert_eq!(
            progression.enter_node(&mut map, &forked_template(), start),
            Err(MapProgressionError::NodeUnavailable)
        );
        assert!(progression
            .selectable_node_ids(&map, &forked_template())
            .iter()
            .all(|node_id| *node_id != start));
        assert!(!progression
            .selectable_node_ids(&map, &forked_template())
            .is_empty());
    }

    #[test]
    fn unavailable_or_missing_nodes_do_not_change_progression() {
        let mut map = forked_map();
        let mut progression = MapProgression::from_map(&map);
        let unavailable = MapNodeId::new(Uuid::from_u128(2));
        let missing = MapNodeId::new(Uuid::from_u128(999));

        assert_eq!(
            progression.enter_node(&mut map, &forked_template(), unavailable),
            Err(MapProgressionError::NodeUnavailable)
        );
        assert_eq!(
            progression.enter_node(&mut map, &forked_template(), missing),
            Err(MapProgressionError::NodeUnavailable)
        );

        assert_eq!(progression.current_node_id, None);
        assert_eq!(progression.available_node_ids, map.start_node_ids);
        assert!(progression.completed_node_ids.is_empty());
    }

    #[test]
    fn completing_terminal_boss_reports_boundary_and_keeps_no_available_nodes() {
        let mut map = forked_map();
        let boss = map.terminals[0].node_id;
        map.start_node_ids = vec![boss];
        let boss_node = map.node_mut(boss).unwrap();
        boss_node.state = MapNodeState::Available;
        boss_node.visibility = MapNodeVisibility::Revealed;
        let mut progression = MapProgression::from_map(&map);

        progression
            .enter_node(&mut map, &forked_template(), boss)
            .unwrap();
        let terminal_resolution = progression
            .complete_current_node(&mut map, &forked_template())
            .unwrap();

        assert_eq!(
            terminal_resolution,
            Some(FacilityTerminalResolution::RunFinalBoss)
        );
        assert_eq!(progression.completed_node_ids, vec![boss]);
        assert!(progression.available_node_ids.is_empty());
        assert_eq!(map.node(boss).unwrap().state, MapNodeState::Completed);
    }

    #[test]
    fn selectability_requires_available_and_not_concealed() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let progression = MapProgression::from_map(&map);

        assert!(progression.is_node_selectable(&map, &forked_template(), start));

        map.node_mut(start).unwrap().visibility = MapNodeVisibility::Concealed;
        assert!(!progression.is_node_selectable(&map, &forked_template(), start));

        map.node_mut(start).unwrap().visibility = MapNodeVisibility::Obscured;
        map.node_mut(start).unwrap().state = MapNodeState::Locked;
        assert!(!progression.is_node_selectable(&map, &forked_template(), start));
    }

    #[test]
    fn current_unresolved_node_can_be_reentered_from_confirm_state() {
        let mut map = forked_map();
        let start = map.start_node_ids[0];
        let mut progression = MapProgression::from_map(&map);

        progression
            .enter_node(&mut map, &forked_template(), start)
            .unwrap();
        progression
            .enter_node(&mut map, &forked_template(), start)
            .unwrap();

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
        let mut template = forked_template();
        let start = map.start_node_ids[0];
        let left = MapNodeId::new(Uuid::from_u128(2));
        let right = MapNodeId::new(Uuid::from_u128(3));

        let connection = template
            .connections
            .iter_mut()
            .find(|connection| {
                connection.from_slot_id.as_str() == "start"
                    && connection.to_slot_id.as_str() == "left"
            })
            .expect("start to left connection");
        connection.direction = MapEdgeDirection::ForwardOnly;

        let mut progression = MapProgression::from_map(&map);
        progression.enter_node(&mut map, &template, start).unwrap();
        progression
            .complete_current_node(&mut map, &template)
            .unwrap();
        progression.enter_node(&mut map, &template, left).unwrap();
        progression
            .complete_current_node(&mut map, &template)
            .unwrap();

        assert!(!progression.available_node_ids.contains(&right));
    }

    #[test]
    fn facility_contract_rejects_duplicate_and_accepts_semantic_slots() {
        let mut duplicate = forked_map();
        let first_slot = duplicate.nodes[0].slot_id.clone();
        duplicate.nodes[1].slot_id = first_slot;
        assert!(duplicate
            .validate_facility_template_contract(&forked_template())
            .unwrap_err()
            .contains("duplicate map slot_id"));

        let mut mismatched = forked_map();
        mismatched.nodes[0].slot_id = MapSlotId::new("archive_room");
        assert!(mismatched
            .validate_facility_template_contract(&forked_template())
            .unwrap_err()
            .contains("outside facility template"));
    }

    #[test]
    fn facility_contract_rejects_missing_terminal_and_allows_locked_nodes() {
        let mut missing_terminal = forked_map();
        missing_terminal.terminals[0].node_id = MapNodeId::new(Uuid::from_u128(999));
        assert!(missing_terminal
            .validate_facility_template_contract(&forked_template())
            .unwrap_err()
            .contains("references missing node"));

        let mut locked_parent = forked_map();
        locked_parent.nodes[0].state = MapNodeState::Locked;
        assert_eq!(
            locked_parent.validate_facility_template_contract(&forked_template()),
            Ok(())
        );
    }
}
