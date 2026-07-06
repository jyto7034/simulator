use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::warn;

use crate::game::{
    data::{
        boss_omen_data::{
            BossOmenChain, BossOmenChainId, BossOmenRevealLevel, BossOmenSourceKind, BossOmenStepId,
        },
        GameDataBase,
    },
    determinism,
    map::{
        GameMode, MapEdgeDirection, MapEdgeDto, MapNode, MapNodeCategory, MapNodeId, MapNodeKindId,
        MapNodeOmenOverlayDto, MapNodePayload, MapNodeState, MapNodeVisibility, MapProgression,
        MapSlotId, RunMap, RunProgression,
    },
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossOmenRunState {
    pub provisional: Option<ProvisionalBossOmenState>,
    pub active: Option<ActiveBossOmenState>,
    pub placed_source: Option<PlacedBossOmenSourceState>,
    pub forced_boss: Option<ForcedBossOmenNodeState>,
    pub skipped_chain_ids: Vec<BossOmenChainId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionalBossOmenState {
    pub chain_id: BossOmenChainId,
    pub boss_abnormality_id: String,
    pub step_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveBossOmenState {
    pub chain_id: BossOmenChainId,
    pub boss_abnormality_id: String,
    pub next_step_index: usize,
    pub completed_steps: Vec<BossOmenStepId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedBossOmenSourceState {
    pub node_id: MapNodeId,
    pub chain_id: BossOmenChainId,
    pub boss_abnormality_id: String,
    pub step_id: BossOmenStepId,
    pub step_index: usize,
    pub source_kind: BossOmenSourceKind,
    pub hint_id: String,
    pub title_id: String,
    pub description_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<crate::game::data::event_data::EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter_id: Option<String>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForcedBossOmenNodeState {
    pub node_id: MapNodeId,
    pub chain_id: BossOmenChainId,
    pub boss_abnormality_id: String,
    pub encounter_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConfirmOmenDto {
    pub present: bool,
    pub omen_chain_id: BossOmenChainId,
    pub abnormality_id: String,
    pub step_id: BossOmenStepId,
    pub source_kind: BossOmenSourceKind,
    pub title_id: String,
    pub description_id: String,
}

impl BossOmenRunState {
    pub fn node_confirm_omen(&self, node_id: MapNodeId) -> Option<NodeConfirmOmenDto> {
        let placed = self.placed_source.as_ref()?;
        (placed.node_id == node_id).then(|| NodeConfirmOmenDto {
            present: true,
            omen_chain_id: placed.chain_id.clone(),
            abnormality_id: placed.boss_abnormality_id.clone(),
            step_id: placed.step_id.clone(),
            source_kind: placed.source_kind,
            title_id: placed.title_id.clone(),
            description_id: placed.description_id.clone(),
        })
    }

    pub fn mark_confirmed(&mut self, node_id: MapNodeId) {
        if let Some(placed) = self.placed_source.as_mut() {
            if placed.node_id == node_id {
                placed.confirmed = true;
            }
        }
    }

    pub fn handle_gate_advance(&mut self) {
        if self.forced_boss.is_some() {
            return;
        }
        let ignored_first_source = self
            .placed_source
            .as_ref()
            .is_some_and(|placed| !placed.confirmed && self.active.is_none());
        if ignored_first_source {
            if let Some(provisional) = self.provisional.take() {
                self.skipped_chain_ids.push(provisional.chain_id);
            }
            self.placed_source = None;
        }
    }

    pub fn consume_completed_source(&mut self, node_id: MapNodeId) {
        if self.forced_boss.is_some() {
            return;
        }
        let Some(placed) = self.placed_source.take() else {
            return;
        };
        if placed.node_id != node_id {
            self.placed_source = Some(placed);
            return;
        }

        if let Some(active) = self.active.as_mut() {
            active.completed_steps.push(placed.step_id);
            active.next_step_index = placed.step_index.saturating_add(1);
            return;
        }

        self.provisional = None;
        self.active = Some(ActiveBossOmenState {
            chain_id: placed.chain_id,
            boss_abnormality_id: placed.boss_abnormality_id,
            next_step_index: placed.step_index.saturating_add(1),
            completed_steps: vec![placed.step_id],
        });
    }

    pub fn forced_boss_node_id(&self) -> Option<MapNodeId> {
        self.forced_boss.as_ref().map(|boss| boss.node_id)
    }

    pub fn clear_after_forced_boss_victory(&mut self, node_id: MapNodeId) {
        if self
            .forced_boss
            .as_ref()
            .is_some_and(|forced| forced.node_id == node_id)
        {
            self.provisional = None;
            self.active = None;
            self.placed_source = None;
            self.forced_boss = None;
        }
    }
}

pub fn apply_boss_omen_to_map(
    map: &mut RunMap,
    run_progression: &RunProgression,
    skill_fragments: &SkillFragmentInventory,
    game_data: &GameDataBase,
    state: &mut BossOmenRunState,
) {
    state.placed_source = None;
    if state.forced_boss.is_some() {
        return;
    }
    if run_progression.game_mode != GameMode::Endless {
        return;
    }
    if state.active.is_some() {
        apply_active_chain_step(map, run_progression, game_data, state);
        return;
    }
    if state.provisional.is_none() {
        if !has_awakened_skill_fragment(skill_fragments) {
            return;
        }
        let Some(chain) = select_provisional_chain(run_progression, game_data, state) else {
            return;
        };
        state.provisional = Some(ProvisionalBossOmenState {
            chain_id: chain.id.clone(),
            boss_abnormality_id: chain.boss_abnormality_id.clone(),
            step_index: 0,
        });
    }
    apply_provisional_chain_step(map, run_progression, game_data, state);
}

pub fn force_boss_node_if_ready(
    map: &mut RunMap,
    progression: &mut MapProgression,
    run_progression: &RunProgression,
    game_data: &GameDataBase,
    state: &mut BossOmenRunState,
) {
    if state.forced_boss.is_some() {
        return;
    }
    let Some(active) = state.active.as_ref() else {
        return;
    };
    let Some(chain) = game_data.boss_omen_data.get_by_id(active.chain_id.as_str()) else {
        panic!(
            "active boss omen chain '{}' is missing from data",
            active.chain_id.as_str()
        );
    };
    if active.completed_steps.len() < chain.required_steps as usize {
        return;
    }
    let current_node_id = progression
        .current_node_id
        .unwrap_or_else(|| panic!("boss omen chain completed without current map node"));
    let current = map
        .node(current_node_id)
        .unwrap_or_else(|| panic!("boss omen completed source node is missing from map"))
        .clone();

    let boss_node_id = unique_forced_boss_node_id(map, run_progression, current_node_id);
    let (depth, lane) = unique_forced_boss_slot(map, current.depth.saturating_add(1), current.lane);
    let boss_node = MapNode {
        id: boss_node_id,
        depth,
        lane,
        slot_id: MapSlotId::for_grid_position(depth, lane),
        kind_id: MapNodeKindId::new("boss_omen_final_boss"),
        category: MapNodeCategory::Boss,
        state: MapNodeState::Available,
        visibility: MapNodeVisibility::Revealed,
        payload: MapNodePayload::Encounter {
            encounter_id: Some(chain.boss_encounter_id.clone()),
        },
        omen: None,
    };
    for node in &mut map.nodes {
        if node.state == MapNodeState::Available {
            node.state = MapNodeState::Unavailable;
        }
    }
    map.nodes.push(boss_node);
    map.edges.push(MapEdgeDto {
        from_node_id: current_node_id,
        to_node_id: boss_node_id,
        direction: MapEdgeDirection::ForwardOnly,
    });
    map.terminal_node_id = boss_node_id;
    progression.available_node_ids = vec![boss_node_id];
    state.forced_boss = Some(ForcedBossOmenNodeState {
        node_id: boss_node_id,
        chain_id: chain.id.clone(),
        boss_abnormality_id: chain.boss_abnormality_id.clone(),
        encounter_id: chain.boss_encounter_id.clone(),
    });
}

fn unique_forced_boss_node_id(
    map: &RunMap,
    run_progression: &RunProgression,
    source_node_id: MapNodeId,
) -> MapNodeId {
    for index in 0..u64::MAX {
        let uuid = determinism::uuid_v4_from_seed(
            determinism::seed_with_uuid(
                run_progression.current_floor_seed(),
                0x424F_4D45_4E46,
                source_node_id.0,
            ),
            0x424F_4D45_4E42,
            index,
        );
        let node_id = MapNodeId::new(uuid);
        if map.node(node_id).is_none() {
            return node_id;
        }
    }
    unreachable!("deterministic boss omen node id space exhausted")
}

fn unique_forced_boss_slot(map: &RunMap, start_depth: u8, preferred_lane: u8) -> (u8, u8) {
    for depth_offset in 0..u8::MAX {
        let depth = start_depth.saturating_add(depth_offset);
        for lane_offset in 0..u8::MAX {
            let lanes = [
                preferred_lane.saturating_add(lane_offset),
                preferred_lane.saturating_sub(lane_offset),
            ];
            for lane in lanes {
                let slot_id = MapSlotId::for_grid_position(depth, lane);
                if map.nodes.iter().all(|node| node.slot_id != slot_id) {
                    return (depth, lane);
                }
            }
        }
    }
    panic!("unable to allocate unique boss omen map slot")
}

fn has_awakened_skill_fragment(skill_fragments: &SkillFragmentInventory) -> bool {
    skill_fragments
        .progress_entries()
        .any(|(_, progress)| progress.awakened)
}

fn select_provisional_chain<'a>(
    run_progression: &RunProgression,
    game_data: &'a GameDataBase,
    state: &BossOmenRunState,
) -> Option<&'a BossOmenChain> {
    let mut candidates = game_data
        .boss_omen_data
        .chains
        .iter()
        .filter(|chain| run_progression.floor_index() >= chain.min_floor)
        .filter(|chain| !state.skipped_chain_ids.contains(&chain.id))
        .collect::<Vec<_>>();
    if candidates.is_empty() && !state.skipped_chain_ids.is_empty() {
        candidates = game_data
            .boss_omen_data
            .chains
            .iter()
            .filter(|chain| run_progression.floor_index() >= chain.min_floor)
            .collect::<Vec<_>>();
    }
    candidates.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    if candidates.is_empty() {
        return None;
    }
    let seed =
        determinism::seed_with_namespace(run_progression.current_floor_seed(), 0x424F_4D45_4E31);
    let index = (seed as usize) % candidates.len();
    Some(candidates[index])
}

fn apply_active_chain_step(
    map: &mut RunMap,
    run_progression: &RunProgression,
    game_data: &GameDataBase,
    state: &mut BossOmenRunState,
) {
    let Some(active) = &state.active else {
        return;
    };
    let Some(chain) = game_data.boss_omen_data.get_by_id(active.chain_id.as_str()) else {
        panic!(
            "active boss omen chain '{}' is missing from data",
            active.chain_id.as_str()
        );
    };
    place_step(map, run_progression, chain, active.next_step_index, state);
}

fn apply_provisional_chain_step(
    map: &mut RunMap,
    run_progression: &RunProgression,
    game_data: &GameDataBase,
    state: &mut BossOmenRunState,
) {
    let Some(provisional) = &state.provisional else {
        return;
    };
    let Some(chain) = game_data
        .boss_omen_data
        .get_by_id(provisional.chain_id.as_str())
    else {
        panic!(
            "provisional boss omen chain '{}' is missing from data",
            provisional.chain_id.as_str()
        );
    };
    place_step(map, run_progression, chain, provisional.step_index, state);
}

fn place_step(
    map: &mut RunMap,
    run_progression: &RunProgression,
    chain: &BossOmenChain,
    step_index: usize,
    state: &mut BossOmenRunState,
) {
    let Some(step) = chain.steps.get(step_index) else {
        panic!(
            "boss omen chain '{}' missing step index {}",
            chain.id.as_str(),
            step_index
        );
    };
    let Some(node_id) = select_overlay_node(map, run_progression, step.source_kind) else {
        warn!(
            chain_id = chain.id.as_str(),
            step_id = step.id.as_str(),
            source_kind = ?step.source_kind,
            floor_index = run_progression.floor_index(),
            "Boss omen step deferred because no eligible overlay node exists"
        );
        return;
    };
    let node = map
        .node_mut(node_id)
        .expect("selected boss omen overlay node must exist");
    match step.source_kind {
        BossOmenSourceKind::Event => {
            node.category = MapNodeCategory::Event;
            node.payload = MapNodePayload::Event {
                event_id: step.event_id.clone(),
            };
        }
        BossOmenSourceKind::Combat => {
            node.category = MapNodeCategory::Combat;
            node.payload = MapNodePayload::Encounter {
                encounter_id: step.encounter_id.clone(),
            };
        }
    }
    node.omen = Some(MapNodeOmenOverlayDto {
        present: true,
        source_kind: step.source_kind,
        hint_id: step.hint_id.clone(),
        reveal_level: BossOmenRevealLevel::HintOnly,
    });
    state.placed_source = Some(PlacedBossOmenSourceState {
        node_id,
        chain_id: chain.id.clone(),
        boss_abnormality_id: chain.boss_abnormality_id.clone(),
        step_id: step.id.clone(),
        step_index,
        source_kind: step.source_kind,
        hint_id: step.hint_id.clone(),
        title_id: step.title_id.clone(),
        description_id: step.description_id.clone(),
        event_id: step.event_id.clone(),
        encounter_id: step.encounter_id.clone(),
        confirmed: false,
    });
}

fn select_overlay_node(
    map: &RunMap,
    run_progression: &RunProgression,
    source_kind: BossOmenSourceKind,
) -> Option<MapNodeId> {
    let category = match source_kind {
        BossOmenSourceKind::Event => MapNodeCategory::Event,
        BossOmenSourceKind::Combat => MapNodeCategory::Combat,
    };
    let mut candidates = map
        .nodes
        .iter()
        .filter(|node| node.category == category)
        .filter(|node| node.omen.is_none())
        .filter(|node| match source_kind {
            BossOmenSourceKind::Event => true,
            BossOmenSourceKind::Combat => !node.kind_id.as_str().contains("elite"),
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|node| {
        (
            map_distance_to_terminal(map, node.id),
            node.depth,
            node.lane,
            determinism::seed_with_uuid(
                run_progression.current_floor_seed(),
                0x424F_4D45_4E32,
                node.id.0,
            ),
        )
    });
    candidates.first().map(|node| node.id)
}

fn map_distance_to_terminal(map: &RunMap, node_id: MapNodeId) -> u32 {
    let mut frontier = VecDeque::from([(node_id, 0_u32)]);
    let mut visited = Vec::new();
    while let Some((current, distance)) = frontier.pop_front() {
        if current == map.terminal_node_id {
            return distance;
        }
        if visited.contains(&current) {
            continue;
        }
        visited.push(current);
        for neighbor in map.bidirectional_neighbors(current) {
            frontier.push_back((neighbor, distance.saturating_add(1)));
        }
    }
    u32::MAX
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::{MapEdgeDirection, MapTemplateId, DEFAULT_MAP_TEMPLATE_ID};
    use uuid::Uuid;

    fn test_node(raw_id: u128, depth: u8, lane: u8) -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(raw_id)),
            depth,
            lane,
            slot_id: MapSlotId::for_grid_position(depth, lane),
            kind_id: MapNodeKindId::new(format!("node_{raw_id}")),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::None,
            omen: None,
        }
    }

    fn test_edge(from_node_id: MapNodeId, to_node_id: MapNodeId) -> MapEdgeDto {
        MapEdgeDto {
            from_node_id,
            to_node_id,
            direction: MapEdgeDirection::Bidirectional,
        }
    }

    #[test]
    fn map_distance_to_terminal_uses_shortest_bfs_path() {
        let start = MapNodeId::new(Uuid::from_u128(1));
        let long_a = MapNodeId::new(Uuid::from_u128(2));
        let long_b = MapNodeId::new(Uuid::from_u128(3));
        let terminal = MapNodeId::new(Uuid::from_u128(4));
        let short = MapNodeId::new(Uuid::from_u128(99));
        let unreachable = MapNodeId::new(Uuid::from_u128(100));

        let map = RunMap {
            map_template_id: MapTemplateId::new(DEFAULT_MAP_TEMPLATE_ID),
            nodes: vec![
                test_node(1, 0, 0),
                test_node(2, 1, 0),
                test_node(3, 2, 0),
                test_node(4, 3, 0),
                test_node(99, 1, 1),
                test_node(100, 9, 9),
            ],
            edges: vec![
                test_edge(start, long_a),
                test_edge(long_a, long_b),
                test_edge(long_b, terminal),
                test_edge(start, short),
                test_edge(short, terminal),
            ],
            start_node_ids: vec![start],
            terminal_node_id: terminal,
        };

        assert_eq!(map_distance_to_terminal(&map, start), 2);
        assert_eq!(map_distance_to_terminal(&map, short), 1);
        assert_eq!(map_distance_to_terminal(&map, unreachable), u32::MAX);
    }
}
