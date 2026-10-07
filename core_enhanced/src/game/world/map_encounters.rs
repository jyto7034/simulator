use std::collections::HashSet;

use crate::game::{
    combat_setup::mission_policy::CombatMissionPolicy,
    data::{
        pve_data::{PveEncounter, PveEncounterClass, PveEncounterDatabase},
        run_policy_data::{OrdinaryCombatDifficultyBand, RunPolicyData},
    },
    determinism,
    encounter_progression::EncounterProgressionState,
    map::{
        GameMode, MapNode, MapNodeCategory, MapNodeCombatDifficultyClass, MapNodeKindId,
        MapNodePayload, RunMap, RunProgression,
    },
};

pub(super) fn assign_map_encounters(
    pve_data: &PveEncounterDatabase,
    map: &mut RunMap,
    run_progression: &RunProgression,
    encounter_progression: &mut EncounterProgressionState,
    run_policy: &RunPolicyData,
) {
    if pve_data.encounters.is_empty() {
        return;
    }

    let mut assigned_elite_source_ids = HashSet::new();
    let mut assigned_natural_elite_abnormality_ids = HashSet::new();
    let mut assigned_normal_boss_source_ids = HashSet::new();

    for node in &mut map.nodes {
        if !matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) || !matches!(
            &node.payload,
            MapNodePayload::Encounter { encounter_id: None }
        ) {
            continue;
        }

        let selected = if run_progression.game_mode == GameMode::Endless
            && node.category == MapNodeCategory::Combat
            && node.kind_id.as_str().contains("elite")
        {
            select_endless_elite(
                pve_data,
                node,
                run_progression,
                encounter_progression,
                &mut assigned_elite_source_ids,
                &mut assigned_natural_elite_abnormality_ids,
            )
        } else if run_progression.game_mode == GameMode::Endless
            && node.category == MapNodeCategory::Boss
        {
            select_endless_normal_boss(
                pve_data,
                run_progression,
                encounter_progression,
                &mut assigned_normal_boss_source_ids,
            )
        } else {
            select_static_class_encounter(pve_data, node, run_progression)
        };

        let selected = match selected {
            Some(selected) => selected,
            None if node.category == MapNodeCategory::Combat
                && node.kind_id.as_str().contains("elite") =>
            {
                node.kind_id = MapNodeKindId::new("combat_normal");
                convert_elite_difficulty_to_authored_ordinary(node, run_policy);
                select_static_class_encounter(pve_data, node, run_progression).unwrap_or_else(|| {
                    panic!(
                        "authored Elite slot has no distinct unmastered/mastered candidate and no ordinary fallback encounter"
                    )
                })
            }
            None => panic!(
                "no authored encounter source is available for map node {:?} ({:?}, kind '{}')",
                node.id,
                node.category,
                node.kind_id.as_str()
            ),
        };

        if let MapNodePayload::Encounter { encounter_id } = &mut node.payload {
            *encounter_id = Some(selected.id.clone());
        }
    }
}

fn select_endless_elite<'a>(
    pve_data: &'a PveEncounterDatabase,
    node: &MapNode,
    run_progression: &RunProgression,
    progression: &mut EncounterProgressionState,
    assigned_source_ids: &mut HashSet<String>,
    assigned_natural_abnormality_ids: &mut HashSet<String>,
) -> Option<&'a PveEncounter> {
    let mut unmastered = pve_data
        .reward_sources
        .iter()
        .filter_map(|source| {
            let encounter = pve_data.get_by_id(&source.encounter_id)?;
            (encounter.encounter_class == PveEncounterClass::Elite).then_some((source, encounter))
        })
        .filter(|(_, encounter)| {
            let abnormality_id = encounter
                .primary_abnormality_id()
                .expect("validated Elite reward source must have a primary abnormality");
            !assigned_natural_abnormality_ids.contains(abnormality_id)
                && !progression
                    .monolith
                    .progress(abnormality_id)
                    .is_some_and(|progress| {
                        progress.natural_outcome
                            == crate::game::monolith::NaturalEliteOutcome::Victory
                    })
        })
        .collect::<Vec<_>>();
    unmastered.sort_by(|left, right| left.0.reward_source_id.cmp(&right.0.reward_source_id));
    if !unmastered.is_empty() {
        let index = encounter_selection_seed(run_progression, node) as usize % unmastered.len();
        let (source, encounter) = unmastered[index];
        assigned_source_ids.insert(source.reward_source_id.clone());
        assigned_natural_abnormality_ids.insert(
            encounter
                .primary_abnormality_id()
                .expect("validated Elite primary abnormality")
                .to_string(),
        );
        return Some(encounter);
    }

    let source_id = progression.reserve_mastered_elite(
        pve_data,
        run_progression.run_seed,
        assigned_source_ids,
    )?;
    assigned_source_ids.insert(source_id.clone());
    let source = pve_data
        .reward_sources
        .iter()
        .find(|source| source.reward_source_id == source_id)
        .expect("validated mastered Elite cycle source must remain authored");
    pve_data.get_by_id(&source.encounter_id)
}

fn select_endless_normal_boss<'a>(
    pve_data: &'a PveEncounterDatabase,
    run_progression: &RunProgression,
    progression: &mut EncounterProgressionState,
    assigned_source_ids: &mut HashSet<String>,
) -> Option<&'a PveEncounter> {
    let source_id =
        progression.reserve_normal_boss(pve_data, run_progression.run_seed, assigned_source_ids)?;
    assigned_source_ids.insert(source_id.clone());
    let source = pve_data
        .reward_sources
        .iter()
        .find(|source| source.reward_source_id == source_id)
        .expect("validated NormalBoss cycle source must remain authored");
    pve_data.get_by_id(&source.encounter_id)
}

fn select_static_class_encounter<'a>(
    pve_data: &'a PveEncounterDatabase,
    node: &MapNode,
    run_progression: &RunProgression,
) -> Option<&'a PveEncounter> {
    let class =
        encounter_class_for_map_node(node.category, node.kind_id.as_str(), run_progression)?;
    let preferred_node_types = CombatMissionPolicy::preferred_node_types_for_map_node(
        node.category,
        node.kind_id.as_str(),
    );
    let mut candidates = pve_data
        .encounters
        .iter()
        .filter(|encounter| encounter.encounter_class == class)
        .filter(|encounter| {
            preferred_node_types.is_empty()
                || encounter
                    .node_type
                    .is_some_and(|node_type| preferred_node_types.contains(&node_type))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.id.cmp(&right.id));
    (!candidates.is_empty()).then(|| {
        let index = encounter_selection_seed(run_progression, node) as usize % candidates.len();
        candidates[index]
    })
}

fn convert_elite_difficulty_to_authored_ordinary(node: &mut MapNode, run_policy: &RunPolicyData) {
    let Some(difficulty) = node.combat_difficulty.as_mut() else {
        return;
    };
    let Some(profile) = run_policy.combat_difficulty_profile(&difficulty.difficulty_profile_id)
    else {
        return;
    };
    let Some(tuning) = profile
        .ordinary_bands
        .iter()
        .find(|entry| entry.band == OrdinaryCombatDifficultyBand::Standard)
    else {
        return;
    };
    difficulty.class =
        MapNodeCombatDifficultyClass::Ordinary(OrdinaryCombatDifficultyBand::Standard);
    difficulty.target_power = percent_of_anchor_ceil(
        difficulty.floor_power_anchor,
        tuning.tuning.target_power_percent,
    );
}

fn percent_of_anchor_ceil(anchor: u64, percent: u32) -> u64 {
    let value = u128::from(anchor)
        .saturating_mul(u128::from(percent))
        .div_ceil(100);
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn encounter_selection_seed(run_progression: &RunProgression, node: &MapNode) -> u64 {
    determinism::seed_with_uuid(
        run_progression.current_floor_seed(),
        0x4D41_505F_454E_4354,
        node.id.0,
    )
}

fn encounter_class_for_map_node(
    category: MapNodeCategory,
    kind_id: &str,
    _run_progression: &RunProgression,
) -> Option<PveEncounterClass> {
    match category {
        MapNodeCategory::Boss => Some(PveEncounterClass::NormalBoss),
        MapNodeCategory::Combat if kind_id.contains("elite") => Some(PveEncounterClass::Elite),
        MapNodeCategory::Combat => Some(PveEncounterClass::Normal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::map::test_facility_map;
    use crate::game::monolith::{MonolithProgress, NaturalEliteOutcome};
    use uuid::Uuid;

    #[test]
    fn endless_elite_slots_use_unmastered_then_distinct_mastered_then_ordinary_fallback() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("embedded game data should load");
        let progression = RunProgression::new(77, GameMode::Endless, 3);
        let elite_sources = data
            .pve_data
            .reward_sources
            .iter()
            .filter_map(|source| {
                let encounter = data.pve_data.get_by_id(&source.encounter_id)?;
                (encounter.encounter_class == PveEncounterClass::Elite).then_some((
                    source,
                    encounter.primary_abnormality_id().unwrap().to_string(),
                ))
            })
            .collect::<Vec<_>>();
        assert!(elite_sources.len() >= 3);

        let mut ledger = EncounterProgressionState::default();
        for (index, (_, abnormality_id)) in elite_sources.iter().enumerate().skip(1) {
            ledger.monolith.progress_by_abnormality.insert(
                abnormality_id.clone(),
                MonolithProgress {
                    natural_outcome: NaturalEliteOutcome::Victory,
                    victories: if index <= 2 { 3 } else { 1 },
                    ..MonolithProgress::default()
                },
            );
        }

        let mut map = test_facility_map(77);
        let template = map.nodes[0].clone();
        map.nodes = (0..4_u128)
            .map(|index| MapNode {
                id: crate::game::map::MapNodeId::new(Uuid::from_u128(0xE110 + index)),
                category: MapNodeCategory::Combat,
                kind_id: MapNodeKindId::new("combat_elite"),
                payload: MapNodePayload::Encounter { encounter_id: None },
                combat_difficulty: None,
                ..template.clone()
            })
            .collect();

        assign_map_encounters(
            &data.pve_data,
            &mut map,
            &progression,
            &mut ledger,
            &data.run_policy,
        );

        let assigned_encounters = map
            .nodes
            .iter()
            .map(|node| {
                let MapNodePayload::Encounter {
                    encounter_id: Some(encounter_id),
                } = &node.payload
                else {
                    panic!("every generated combat slot must resolve an encounter");
                };
                (node.kind_id.as_str(), encounter_id.as_str())
            })
            .collect::<Vec<_>>();
        let assigned_elites = assigned_encounters
            .iter()
            .filter(|(kind_id, _)| *kind_id == "combat_elite")
            .map(|(_, encounter_id)| *encounter_id)
            .collect::<HashSet<_>>();
        assert_eq!(assigned_elites.len(), 3);
        assert!(assigned_elites.contains(elite_sources[0].0.encounter_id.as_str()));
        assert!(assigned_elites.contains(elite_sources[1].0.encounter_id.as_str()));
        assert!(assigned_elites.contains(elite_sources[2].0.encounter_id.as_str()));
        assert_eq!(
            assigned_encounters
                .iter()
                .filter(|(kind_id, _)| *kind_id == "combat_normal")
                .count(),
            1
        );
        assert_eq!(ledger.mastered_elite_cycle.assigned_source_ids.len(), 2);
    }

    #[test]
    fn endless_normal_boss_cycle_excludes_unpublished_moonlit_wail() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded()
            .expect("embedded game data should load");
        let progression = RunProgression::new(42, GameMode::Endless, 3);
        let mut ledger = EncounterProgressionState::default();
        let mut map = test_facility_map(42);
        let mut boss = map.nodes[0].clone();
        boss.category = MapNodeCategory::Boss;
        boss.kind_id = MapNodeKindId::new("boss_abnormality");
        boss.payload = MapNodePayload::Encounter { encounter_id: None };
        boss.combat_difficulty = None;
        map.nodes = vec![boss];

        assign_map_encounters(
            &data.pve_data,
            &mut map,
            &progression,
            &mut ledger,
            &data.run_policy,
        );
        let MapNodePayload::Encounter {
            encounter_id: Some(encounter_id),
        } = &map.nodes[0].payload
        else {
            panic!("boss encounter must be assigned");
        };
        assert_ne!(encounter_id, "suppress_moonlit_wail");
        assert_eq!(ledger.normal_boss_cycle.assigned_source_ids.len(), 1);
    }
}
