use crate::game::combat_mission_policy::CombatMissionPolicy;
use crate::game::combat_preview::CombatNodeType;
use crate::game::data::pve_data::{PveEncounter, PveEncounterDatabase};
use crate::game::determinism;
use crate::game::map::{MapNode, MapNodeCategory, MapNodePayload, RunMap, RunProgression};

pub(super) fn assign_map_encounters(
    pve_data: &PveEncounterDatabase,
    map: &mut RunMap,
    run_progression: &RunProgression,
) {
    let mut encounters = pve_data.encounters.iter().collect::<Vec<_>>();
    encounters.sort_by(|a, b| {
        a.difficulty
            .cmp(&b.difficulty)
            .then_with(|| a.id.cmp(&b.id))
    });
    if encounters.is_empty() {
        return;
    }

    let boss_depth = map.node(map.boss_node_id).map_or(0, |node| node.depth);
    for node in &mut map.nodes {
        if !matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) {
            continue;
        }

        if !matches!(
            &node.payload,
            MapNodePayload::Encounter { encounter_id: None }
        ) {
            continue;
        }

        let selected =
            select_encounter_for_map_node(&encounters, node, run_progression, boss_depth);

        if let (Some(encounter), MapNodePayload::Encounter { encounter_id }) =
            (selected, &mut node.payload)
        {
            *encounter_id = Some(encounter.id.clone());
        }
    }
}

fn select_encounter_for_map_node<'a>(
    encounters: &'a [&'a PveEncounter],
    node: &MapNode,
    run_progression: &RunProgression,
    boss_depth: u8,
) -> Option<&'a PveEncounter> {
    if encounters.is_empty() {
        return None;
    }

    let candidates = encounter_candidates_for_map_node(
        encounters,
        node.category,
        node.kind_id.as_str(),
        node.depth,
        boss_depth,
        run_progression.act_index,
    );
    let candidates = if candidates.is_empty() {
        encounters
            .iter()
            .copied()
            .filter(|encounter| encounter.node_type != Some(CombatNodeType::Boss))
            .collect::<Vec<_>>()
    } else {
        candidates
    };
    let candidates = if candidates.is_empty() {
        encounters.to_vec()
    } else {
        candidates
    };

    let index_seed = run_progression.current_act_seed()
        ^ (u64::from(node.depth) << 32)
        ^ u64::from(node.lane)
        ^ determinism::seed_with_uuid(
            run_progression.current_act_seed(),
            0x4D41_505F_454E_4354,
            node.id.0,
        );
    let index = (index_seed as usize) % candidates.len();
    candidates.get(index).copied()
}

fn encounter_candidates_for_map_node<'a>(
    encounters: &'a [&'a PveEncounter],
    category: MapNodeCategory,
    kind_id: &str,
    depth: u8,
    boss_depth: u8,
    act_index: u8,
) -> Vec<&'a PveEncounter> {
    if category == MapNodeCategory::Boss {
        let boss_candidates = encounters
            .iter()
            .copied()
            .filter(|encounter| encounter.node_type == Some(CombatNodeType::Boss))
            .collect::<Vec<_>>();
        if !boss_candidates.is_empty() {
            return boss_candidates;
        }
        let max_difficulty = encounters
            .iter()
            .map(|encounter| encounter.difficulty)
            .max()
            .unwrap_or_default();
        return encounters
            .iter()
            .copied()
            .filter(|encounter| encounter.difficulty == max_difficulty)
            .collect();
    }

    let act_offset = act_index.saturating_mul(2);
    let is_elite = kind_id.contains("elite");
    let is_late_depth = depth + 2 >= boss_depth;
    let preferred_node_types =
        CombatMissionPolicy::preferred_node_types_for_map_node(category, kind_id);
    let min_difficulty = if is_elite {
        3_u8.saturating_add(act_offset)
    } else {
        1_u8.saturating_add(act_offset)
    };
    let max_difficulty = if is_elite {
        6_u8.saturating_add(act_offset)
    } else if is_late_depth {
        4_u8.saturating_add(act_offset)
    } else {
        2_u8.saturating_add(act_offset).saturating_add(depth / 3)
    };

    let difficulty_candidates = encounters
        .iter()
        .copied()
        .filter(|encounter| encounter.node_type != Some(CombatNodeType::Boss))
        .filter(|encounter| {
            encounter.difficulty >= min_difficulty && encounter.difficulty <= max_difficulty
        })
        .collect::<Vec<_>>();

    for preferred_node_type in preferred_node_types {
        let candidates = difficulty_candidates
            .iter()
            .copied()
            .filter(|encounter| encounter.node_type == Some(*preferred_node_type))
            .collect::<Vec<_>>();
        if !candidates.is_empty() {
            return candidates;
        }
    }

    difficulty_candidates
}
