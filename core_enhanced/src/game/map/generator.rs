use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::game::{
    determinism,
    map::types::{
        MapNode, MapNodeCategory, MapNodeDefinition, MapNodeDefinitionDatabase, MapNodeId,
        MapNodeState, RunMap,
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
        let boss_depth = config.depth_count - 1;
        let mut rows: Vec<Vec<MapNodeId>> = Vec::with_capacity(config.depth_count as usize);
        let mut nodes = Vec::new();
        let mut node_index = 0_u64;

        for depth in 0..config.depth_count {
            let width = if depth == boss_depth {
                1
            } else {
                rng.gen_range(config.min_width..=config.max_width)
            };

            let mut row = Vec::with_capacity(width as usize);
            for lane in 0..width {
                let id = MapNodeId::new(determinism::uuid_v4_from_seed(seed, MAP_NS, node_index));
                node_index += 1;
                let definition = if depth == boss_depth {
                    Self::roll_boss_definition(definitions, &mut rng)
                } else {
                    Self::roll_node_definition(depth, boss_depth, definitions, &mut rng)
                };
                row.push(id);
                nodes.push(MapNode {
                    id,
                    depth,
                    lane,
                    kind_id: definition.kind_id.clone(),
                    category: definition.category,
                    state: if depth == 0 {
                        MapNodeState::Available
                    } else {
                        MapNodeState::Hidden
                    },
                    outgoing: Vec::new(),
                    payload: definition.payload.clone(),
                });
            }
            rows.push(row);
        }

        for depth in 0..boss_depth {
            let from_row = rows[depth as usize].clone();
            let to_row = rows[(depth + 1) as usize].clone();
            for (lane, from_id) in from_row.iter().copied().enumerate() {
                let target_count = if to_row.len() == 1 || rng.gen_bool(0.65) {
                    1
                } else {
                    2
                };
                let mut candidates = Self::adjacent_lanes(lane, to_row.len());
                candidates.sort_by_key(|candidate| {
                    let jitter = rng.gen_range(0..=u8::MAX) as usize;
                    (candidate.abs_diff(lane), jitter)
                });
                let mut outgoing = candidates
                    .into_iter()
                    .take(target_count)
                    .map(|target_lane| to_row[target_lane])
                    .collect::<Vec<_>>();
                outgoing.sort_by_key(|id| id.0.as_u128());
                outgoing.dedup();

                if let Some(node) = nodes.iter_mut().find(|node| node.id == from_id) {
                    node.outgoing = outgoing;
                }
            }
        }

        let start_node_ids = rows[0].clone();
        let boss_node_id = rows[boss_depth as usize][0];
        RunMap {
            nodes,
            start_node_ids,
            boss_node_id,
        }
    }

    fn adjacent_lanes(lane: usize, width: usize) -> Vec<usize> {
        let mut lanes = Vec::new();
        let start = lane.saturating_sub(1);
        let end = (lane + 1).min(width.saturating_sub(1));
        for candidate in start..=end {
            lanes.push(candidate);
        }
        if lanes.is_empty() {
            lanes.push(0);
        }
        lanes
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
            let category = if rng.gen_range(0..100) <= 69 {
                MapNodeCategory::Combat
            } else {
                MapNodeCategory::Event
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
        assert_eq!(boss.depth, MapGenerationConfig::default().depth_count - 1);
        assert!(boss.outgoing.is_empty());
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
            definition.kind_id.as_str() == "event_random"
                && matches!(
                    definition.payload,
                    crate::game::map::MapNodePayload::Event { .. }
                )
        }));
        assert!(definitions.nodes.iter().any(|definition| {
            definition.kind_id.as_str() == "reward_treasure"
                && matches!(
                    definition.payload,
                    crate::game::map::MapNodePayload::Reward { .. }
                )
        }));
    }
}
