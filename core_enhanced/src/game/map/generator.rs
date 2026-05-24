use std::collections::{BTreeMap, BTreeSet};

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::game::{
    determinism,
    map::types::{
        MapNode, MapNodeCategory, MapNodeDefinition, MapNodeDefinitionDatabase, MapNodeId,
        MapNodeKindId, MapNodePayload, MapNodeState, RunMap,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct MapGenerationConfig {
    pub depth_count: u8,
    pub min_width: u8,
    pub max_width: u8,
}

impl Default for MapGenerationConfig {
    fn default() -> Self {
        Self {
            depth_count: 8,
            min_width: 3,
            max_width: 5,
        }
    }
}

pub struct MapGenerator;

impl MapGenerator {
    pub fn generate(run_seed: u64, config: MapGenerationConfig) -> RunMap {
        Self::generate_with_definitions(run_seed, config, &MapNodeDefinitionDatabase::builtin())
    }

    pub fn generate_with_definitions(
        run_seed: u64,
        config: MapGenerationConfig,
        definitions: &MapNodeDefinitionDatabase,
    ) -> RunMap {
        assert!(config.depth_count >= 3, "map depth must be at least 3");
        assert!(config.min_width > 0, "map min width must be non-zero");
        assert!(
            config.min_width <= config.max_width,
            "map min width must not exceed max width"
        );

        const MAP_NS: u64 = 0x524D_4150; // "RMAP"
        let seed = determinism::seed_with_namespace(run_seed, MAP_NS);
        let mut rng = StdRng::seed_from_u64(seed);
        let boss_depth = config.depth_count;
        let mut rows: BTreeMap<u8, BTreeMap<u8, MapNodeId>> = BTreeMap::new();
        let mut nodes = Vec::new();
        let mut node_index = 0_u64;

        let start_id = MapNodeId::new(determinism::uuid_v4_from_seed(seed, MAP_NS, node_index));
        node_index += 1;
        nodes.push(MapNode {
            id: start_id,
            depth: 0,
            lane: config.max_width / 2,
            kind_id: MapNodeKindId::new("start"),
            category: MapNodeCategory::Start,
            state: MapNodeState::Completed,
            outgoing: Vec::new(),
            payload: MapNodePayload::None,
        });

        let path_lanes = Self::generate_path_lanes(config, boss_depth, &mut rng);
        for depth in 1..boss_depth {
            let mut lanes = path_lanes
                .iter()
                .filter_map(|path| path.get(depth as usize - 1).copied())
                .collect::<BTreeSet<_>>();
            Self::ensure_minimum_row_width(&mut lanes, config, &mut rng);

            for lane in lanes {
                let id = MapNodeId::new(determinism::uuid_v4_from_seed(seed, MAP_NS, node_index));
                node_index += 1;
                let definition =
                    Self::roll_node_definition(depth, boss_depth, definitions, &mut rng);
                rows.entry(depth).or_default().insert(lane, id);
                nodes.push(MapNode {
                    id,
                    depth,
                    lane,
                    kind_id: definition.kind_id.clone(),
                    category: definition.category,
                    state: if depth == 1 {
                        MapNodeState::Available
                    } else {
                        MapNodeState::Hidden
                    },
                    outgoing: Vec::new(),
                    payload: definition.payload.clone(),
                });
            }

            Self::repair_row_distribution(depth, boss_depth, &mut nodes, definitions, &mut rng);
        }

        let boss_id = MapNodeId::new(determinism::uuid_v4_from_seed(seed, MAP_NS, node_index));
        let boss_definition = Self::roll_boss_definition(definitions, &mut rng);
        rows.entry(boss_depth)
            .or_default()
            .insert(config.max_width / 2, boss_id);
        nodes.push(MapNode {
            id: boss_id,
            depth: boss_depth,
            lane: config.max_width / 2,
            kind_id: boss_definition.kind_id.clone(),
            category: boss_definition.category,
            state: MapNodeState::Hidden,
            outgoing: Vec::new(),
            payload: boss_definition.payload.clone(),
        });

        let first_row_ids = rows
            .get(&1)
            .map(|row| row.values().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        Self::set_outgoing(&mut nodes, start_id, first_row_ids.clone());

        for depth in 1..boss_depth {
            let Some(from_row) = rows.get(&depth) else {
                continue;
            };
            let Some(to_row) = rows.get(&(depth + 1)) else {
                continue;
            };
            Self::connect_rows(&mut nodes, from_row, to_row, &mut rng);
        }

        RunMap {
            nodes,
            start_node_ids: first_row_ids,
            boss_node_id: boss_id,
        }
    }

    fn generate_path_lanes(
        config: MapGenerationConfig,
        boss_depth: u8,
        rng: &mut StdRng,
    ) -> Vec<Vec<u8>> {
        let path_count = config.max_width.max(config.min_width);
        let mut paths = Vec::with_capacity(path_count as usize);
        for _ in 0..path_count {
            let mut lane = rng.gen_range(0..config.max_width);
            let mut path = Vec::with_capacity(boss_depth.saturating_sub(1) as usize);
            for _ in 1..boss_depth {
                path.push(lane);
                let delta = rng.gen_range(0..=2_i8) - 1;
                lane = match delta {
                    -1 => lane.saturating_sub(1),
                    1 => (lane + 1).min(config.max_width.saturating_sub(1)),
                    _ => lane,
                };
            }
            paths.push(path);
        }
        paths
    }

    fn ensure_minimum_row_width(
        lanes: &mut BTreeSet<u8>,
        config: MapGenerationConfig,
        rng: &mut StdRng,
    ) {
        while lanes.len() < config.min_width as usize {
            lanes.insert(rng.gen_range(0..config.max_width));
        }
    }

    fn connect_rows(
        nodes: &mut [MapNode],
        from_row: &BTreeMap<u8, MapNodeId>,
        to_row: &BTreeMap<u8, MapNodeId>,
        rng: &mut StdRng,
    ) {
        let mut incoming = BTreeSet::new();
        for (from_lane, from_id) in from_row {
            let mut candidates = to_row
                .iter()
                .map(|(to_lane, to_id)| {
                    let jitter = rng.gen_range(0..=u8::MAX);
                    (from_lane.abs_diff(*to_lane), jitter, *to_lane, *to_id)
                })
                .collect::<Vec<_>>();
            candidates.sort_by_key(|candidate| (candidate.0, candidate.1, candidate.2));

            let target_count = if candidates.len() == 1 || rng.gen_bool(0.75) {
                1
            } else {
                2
            };
            let outgoing = candidates
                .into_iter()
                .take(target_count)
                .map(|candidate| candidate.3)
                .collect::<Vec<_>>();
            incoming.extend(outgoing.iter().map(|id| id.0.as_u128()));
            Self::set_outgoing(nodes, *from_id, outgoing);
        }

        for (to_lane, to_id) in to_row {
            if incoming.contains(&to_id.0.as_u128()) {
                continue;
            }
            if let Some((_, from_id)) = from_row
                .iter()
                .min_by_key(|(from_lane, _)| from_lane.abs_diff(*to_lane))
            {
                Self::push_outgoing(nodes, *from_id, *to_id);
            }
        }
    }

    fn set_outgoing(nodes: &mut [MapNode], node_id: MapNodeId, mut outgoing: Vec<MapNodeId>) {
        outgoing.sort_by_key(|id| id.0.as_u128());
        outgoing.dedup();
        if let Some(node) = nodes.iter_mut().find(|node| node.id == node_id) {
            node.outgoing = outgoing;
        }
    }

    fn push_outgoing(nodes: &mut [MapNode], node_id: MapNodeId, target_id: MapNodeId) {
        if let Some(node) = nodes.iter_mut().find(|node| node.id == node_id) {
            node.outgoing.push(target_id);
            node.outgoing.sort_by_key(|id| id.0.as_u128());
            node.outgoing.dedup();
        }
    }

    fn roll_node_definition<'a>(
        depth: u8,
        boss_depth: u8,
        definitions: &'a MapNodeDefinitionDatabase,
        rng: &mut StdRng,
    ) -> &'a MapNodeDefinition {
        if depth + 1 == boss_depth {
            let category = if rng.gen_bool(0.65) {
                MapNodeCategory::Support
            } else {
                MapNodeCategory::Combat
            };
            return Self::roll_weighted_definition(
                definitions.weighted_candidates(depth, Some(category), false),
                rng,
            )
            .unwrap_or_else(|| {
                Self::roll_weighted_definition(
                    definitions.weighted_candidates(depth, None, false),
                    rng,
                )
                .expect("map node definitions must include non-boss weighted entries")
            });
        }

        if depth <= 1 {
            let category = match rng.gen_range(0..100) {
                0..=49 => MapNodeCategory::Combat,
                50..=74 => MapNodeCategory::Support,
                _ => MapNodeCategory::HeadquartersContact,
            };
            Self::roll_weighted_definition(
                definitions.weighted_candidates(depth, Some(category), false),
                rng,
            )
            .unwrap_or_else(|| {
                Self::roll_weighted_definition(
                    definitions.weighted_candidates(depth, None, false),
                    rng,
                )
                .expect("map node definitions must include weighted entries for early depths")
            })
        } else {
            Self::roll_weighted_definition(definitions.weighted_candidates(depth, None, false), rng)
                .expect("map node definitions must include weighted entries")
        }
    }

    fn repair_row_distribution(
        depth: u8,
        boss_depth: u8,
        nodes: &mut [MapNode],
        definitions: &MapNodeDefinitionDatabase,
        rng: &mut StdRng,
    ) {
        let row_indices = nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (node.depth == depth).then_some(index))
            .collect::<Vec<_>>();
        if row_indices.is_empty() {
            return;
        }

        let max_combat = (row_indices.len() / 2).max(1);
        let mut combat_indices = row_indices
            .iter()
            .copied()
            .filter(|index| nodes[*index].category == MapNodeCategory::Combat)
            .collect::<Vec<_>>();
        combat_indices.sort_by_key(|index| nodes[*index].lane);
        while combat_indices.len() > max_combat {
            let Some(index) = combat_indices.pop() else {
                break;
            };
            let replacement = Self::roll_safe_definition(depth, boss_depth, definitions, rng)
                .unwrap_or_else(|| Self::roll_node_definition(depth, boss_depth, definitions, rng));
            nodes[index].kind_id = replacement.kind_id.clone();
            nodes[index].category = replacement.category;
            nodes[index].payload = replacement.payload.clone();
        }

        if !row_indices
            .iter()
            .any(|index| nodes[*index].category == MapNodeCategory::Combat)
        {
            if let Some(index) = row_indices
                .iter()
                .copied()
                .min_by_key(|index| nodes[*index].lane)
            {
                if let Some(replacement) = Self::roll_weighted_definition(
                    definitions.weighted_candidates(depth, Some(MapNodeCategory::Combat), false),
                    rng,
                ) {
                    nodes[index].kind_id = replacement.kind_id.clone();
                    nodes[index].category = replacement.category;
                    nodes[index].payload = replacement.payload.clone();
                }
            }
        }

        if depth + 1 == boss_depth
            && !row_indices
                .iter()
                .any(|index| nodes[*index].category == MapNodeCategory::Support)
        {
            let Some(index) = row_indices
                .iter()
                .copied()
                .max_by_key(|index| nodes[*index].lane)
            else {
                return;
            };
            if let Some(replacement) = Self::roll_weighted_definition(
                definitions.weighted_candidates(depth, Some(MapNodeCategory::Support), false),
                rng,
            ) {
                nodes[index].kind_id = replacement.kind_id.clone();
                nodes[index].category = replacement.category;
                nodes[index].payload = replacement.payload.clone();
            }
        }
    }

    fn roll_safe_definition<'a>(
        depth: u8,
        boss_depth: u8,
        definitions: &'a MapNodeDefinitionDatabase,
        rng: &mut StdRng,
    ) -> Option<&'a MapNodeDefinition> {
        let categories = if depth + 1 == boss_depth {
            [
                MapNodeCategory::Support,
                MapNodeCategory::Reward,
                MapNodeCategory::HeadquartersContact,
                MapNodeCategory::Shop,
            ]
        } else {
            [
                MapNodeCategory::Support,
                MapNodeCategory::HeadquartersContact,
                MapNodeCategory::Reward,
                MapNodeCategory::Shop,
            ]
        };
        let mut candidates = Vec::new();
        for category in categories {
            candidates.extend(definitions.weighted_candidates(depth, Some(category), false));
        }
        Self::roll_weighted_definition(candidates, rng)
    }

    fn roll_boss_definition<'a>(
        definitions: &'a MapNodeDefinitionDatabase,
        rng: &mut StdRng,
    ) -> &'a MapNodeDefinition {
        let boss_candidates = definitions.boss_candidates();
        Self::roll_weighted_definition(boss_candidates, rng)
            .expect("map node definitions must include at least one boss entry")
    }

    fn roll_weighted_definition<'a>(
        candidates: Vec<&'a MapNodeDefinition>,
        rng: &mut StdRng,
    ) -> Option<&'a MapNodeDefinition> {
        let total_weight = candidates.iter().fold(0_u32, |total, candidate| {
            total.saturating_add(candidate.weight)
        });
        if total_weight == 0 {
            return None;
        }

        let mut roll = rng.gen_range(0..total_weight);
        for candidate in candidates {
            if roll < candidate.weight {
                return Some(candidate);
            }
            roll -= candidate.weight;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_generates_same_map() {
        let a = MapGenerator::generate(42, MapGenerationConfig::default());
        let b = MapGenerator::generate(42, MapGenerationConfig::default());
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_generates_different_map() {
        let a = MapGenerator::generate(42, MapGenerationConfig::default());
        let b = MapGenerator::generate(43, MapGenerationConfig::default());
        assert_ne!(a, b);
    }

    #[test]
    fn generated_map_has_single_boss_at_last_depth() {
        let map = MapGenerator::generate(42, MapGenerationConfig::default());
        let boss = map.node(map.boss_node_id).expect("boss node exists");
        assert_eq!(boss.category, MapNodeCategory::Boss);
        assert_eq!(boss.kind_id.as_str(), "boss_abnormality");
        assert_eq!(boss.depth, MapGenerationConfig::default().depth_count);
        assert!(boss.outgoing.is_empty());
    }

    #[test]
    fn generated_map_has_visual_start_connected_to_first_playable_row() {
        let map = MapGenerator::generate(42, MapGenerationConfig::default());
        let start = map
            .nodes
            .iter()
            .find(|node| node.category == MapNodeCategory::Start)
            .expect("generated map has a visual start node");

        assert_eq!(start.depth, 0);
        assert_eq!(start.state, MapNodeState::Completed);
        assert_eq!(start.outgoing.len(), map.start_node_ids.len());
        assert!(map.start_node_ids.iter().all(|node_id| {
            start.outgoing.contains(node_id)
                && map
                    .node(*node_id)
                    .is_some_and(|node| node.depth == 1 && node.state == MapNodeState::Available)
        }));
    }

    #[test]
    fn generated_rows_are_not_combat_only_corridors() {
        let map = MapGenerator::generate(42, MapGenerationConfig::default());
        let boss_depth = MapGenerationConfig::default().depth_count;

        for depth in 1..boss_depth {
            let row = map
                .nodes
                .iter()
                .filter(|node| node.depth == depth)
                .collect::<Vec<_>>();
            let combat_count = row
                .iter()
                .filter(|node| node.category == MapNodeCategory::Combat)
                .count();

            assert!(!row.is_empty(), "depth {depth} must have nodes");
            assert!(
                combat_count <= (row.len() / 2).max(1),
                "depth {depth} has too many combat nodes"
            );
        }
    }

    #[test]
    fn generated_nodes_use_ron_definitions() {
        let map = MapGenerator::generate(42, MapGenerationConfig::default());
        assert!(map
            .nodes
            .iter()
            .all(|node| !node.kind_id.as_str().is_empty()));
        assert!(map
            .nodes
            .iter()
            .any(|node| node.kind_id.as_str() == "combat_monster"));
    }

    #[test]
    fn builtin_node_definitions_validate_contract() {
        let definitions = MapNodeDefinitionDatabase::builtin();
        assert!(definitions.validate_contract().is_ok());
        assert!(definitions.nodes.iter().any(|definition| {
            definition.kind_id.as_str() == "shop_general"
                && matches!(
                    definition.payload,
                    crate::game::map::MapNodePayload::Shop { .. }
                )
        }));
        assert!(definitions.nodes.iter().any(|definition| {
            definition.kind_id.as_str() == "reward_treasure"
                && matches!(
                    definition.payload,
                    crate::game::map::MapNodePayload::Reward { .. }
                )
        }));
        assert!(definitions.nodes.iter().any(|definition| {
            definition.kind_id.as_str() == "headquarters_contact"
                && matches!(
                    definition.payload,
                    crate::game::map::MapNodePayload::HeadquartersContact { .. }
                )
        }));
    }
}
