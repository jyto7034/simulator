use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::combat_setup::mission_policy::CombatMissionPolicy;
use crate::game::data::{
    pve_data::{PveEncounter, PveEncounterClass, PveEncounterDatabase},
    run_policy_data::{
        EmptyEncounterCandidateFallbackPolicy, EncounterRepeatWeightingPolicy, RunPolicyData,
    },
};
use crate::game::determinism;
use crate::game::map::{
    GameMode, MapNode, MapNodeCategory, MapNodePayload, RunMap, RunProgression,
};

pub(super) fn assign_map_encounters(
    pve_data: &PveEncounterDatabase,
    map: &mut RunMap,
    run_progression: &RunProgression,
    abnormality_research: &RunAbnormalityResearchState,
    abnormality_encounter_history: &RunAbnormalityEncounterHistory,
    run_policy: &RunPolicyData,
) {
    let mut encounters = pve_data.encounters.iter().collect::<Vec<_>>();
    encounters.sort_by(|a, b| a.id.cmp(&b.id));
    if encounters.is_empty() {
        return;
    }

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

        let selected = select_encounter_for_map_node(
            &encounters,
            node,
            run_progression,
            abnormality_research,
            abnormality_encounter_history,
            run_policy,
        );

        if let MapNodePayload::Encounter { encounter_id } = &mut node.payload {
            *encounter_id = Some(selected.id.clone());
        }
    }
}

fn select_encounter_for_map_node<'a>(
    encounters: &'a [&'a PveEncounter],
    node: &MapNode,
    run_progression: &RunProgression,
    abnormality_research: &RunAbnormalityResearchState,
    abnormality_encounter_history: &RunAbnormalityEncounterHistory,
    run_policy: &RunPolicyData,
) -> &'a PveEncounter {
    if encounters.is_empty() {
        panic!("cannot assign map encounter: pve encounter database is empty");
    }

    let hard_candidates = encounter_candidates_for_map_node(
        encounters,
        node.category,
        node.kind_id.as_str(),
        run_progression,
    );
    assert!(
        !hard_candidates.is_empty(),
        "no {:?} pve encounters are compatible with map node {:?} ({:?}, kind '{}')",
        encounter_class_for_map_node(node.category, node.kind_id.as_str(), run_progression),
        node.id,
        node.category,
        node.kind_id.as_str()
    );

    let index_seed = encounter_selection_seed(run_progression, node);
    if should_apply_repeat_weighting(run_progression, &hard_candidates) {
        let weighted_candidates = encounter_repeat_weighted_candidates(
            &hard_candidates,
            run_progression.floor_index(),
            abnormality_research,
            abnormality_encounter_history,
            run_policy.encounter_repeat_weighting,
        );
        return select_weighted_candidate(&weighted_candidates, index_seed);
    }

    let index = (index_seed as usize) % hard_candidates.len();
    hard_candidates[index]
}

fn encounter_candidates_for_map_node<'a>(
    encounters: &'a [&'a PveEncounter],
    category: MapNodeCategory,
    kind_id: &str,
    run_progression: &RunProgression,
) -> Vec<&'a PveEncounter> {
    let Some(encounter_class) = encounter_class_for_map_node(category, kind_id, run_progression)
    else {
        return Vec::new();
    };
    let preferred_node_types =
        CombatMissionPolicy::preferred_node_types_for_map_node(category, kind_id);

    encounters
        .iter()
        .copied()
        .filter(|encounter| encounter.encounter_class == encounter_class)
        .filter(|encounter| {
            preferred_node_types.is_empty()
                || encounter
                    .node_type
                    .is_some_and(|node_type| preferred_node_types.contains(&node_type))
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct WeightedEncounterCandidate<'a> {
    encounter: &'a PveEncounter,
    weight: u64,
}

fn encounter_selection_seed(run_progression: &RunProgression, node: &MapNode) -> u64 {
    run_progression.current_floor_seed()
        ^ (u64::from(node.depth) << 32)
        ^ u64::from(node.lane)
        ^ determinism::seed_with_uuid(
            run_progression.current_floor_seed(),
            0x4D41_505F_454E_4354,
            node.id.0,
        )
}

fn should_apply_repeat_weighting(
    run_progression: &RunProgression,
    hard_candidates: &[&PveEncounter],
) -> bool {
    run_progression.game_mode == GameMode::Endless
        && hard_candidates
            .iter()
            .any(|encounter| encounter.primary_abnormality_id().is_some())
}

fn encounter_repeat_weighted_candidates<'a>(
    hard_candidates: &[&'a PveEncounter],
    floor_index: u32,
    abnormality_research: &RunAbnormalityResearchState,
    abnormality_encounter_history: &RunAbnormalityEncounterHistory,
    policy: EncounterRepeatWeightingPolicy,
) -> Vec<WeightedEncounterCandidate<'a>> {
    let soft_filtered = hard_candidates
        .iter()
        .copied()
        .filter(|encounter| {
            !is_recently_encountered(
                encounter,
                floor_index,
                abnormality_encounter_history,
                policy,
            )
        })
        .map(|encounter| WeightedEncounterCandidate {
            encounter,
            weight: repeat_weight_for_encounter(encounter, abnormality_research, policy),
        })
        .collect::<Vec<_>>();

    if !soft_filtered.is_empty() {
        return soft_filtered;
    }

    match policy.empty_pool_fallback {
        EmptyEncounterCandidateFallbackPolicy::ResetSoftRepeatConstraints => hard_candidates
            .iter()
            .copied()
            .map(|encounter| WeightedEncounterCandidate {
                encounter,
                weight: 100,
            })
            .collect(),
    }
}

fn is_recently_encountered(
    encounter: &PveEncounter,
    floor_index: u32,
    abnormality_encounter_history: &RunAbnormalityEncounterHistory,
    policy: EncounterRepeatWeightingPolicy,
) -> bool {
    let Some(abnormality_id) = encounter.primary_abnormality_id() else {
        return false;
    };
    let Some(last_floor) = abnormality_encounter_history.last_appeared_floor(abnormality_id) else {
        return false;
    };

    floor_index.saturating_sub(last_floor) <= policy.recent_floor_exclusion_window
}

fn repeat_weight_for_encounter(
    encounter: &PveEncounter,
    abnormality_research: &RunAbnormalityResearchState,
    policy: EncounterRepeatWeightingPolicy,
) -> u64 {
    let Some(abnormality_id) = encounter.primary_abnormality_id() else {
        return u64::from(policy.incomplete_weight_multiplier_percent);
    };
    let Some(entry) = abnormality_research.entries.get(abnormality_id) else {
        return u64::from(policy.incomplete_weight_multiplier_percent);
    };

    if entry.response_complete {
        u64::from(policy.response_complete_weight_multiplier_percent)
    } else {
        u64::from(policy.incomplete_weight_multiplier_percent)
    }
}

fn select_weighted_candidate<'a>(
    candidates: &[WeightedEncounterCandidate<'a>],
    index_seed: u64,
) -> &'a PveEncounter {
    assert!(
        !candidates.is_empty(),
        "cannot select weighted encounter from empty candidate set"
    );
    let total_weight = candidates
        .iter()
        .map(|candidate| candidate.weight)
        .sum::<u64>();
    assert!(
        total_weight > 0,
        "weighted encounter candidate total weight must be greater than zero"
    );

    let mut pick = index_seed % total_weight;
    for candidate in candidates {
        if pick < candidate.weight {
            return candidate.encounter;
        }
        pick -= candidate.weight;
    }

    candidates
        .last()
        .expect("validated weighted candidate set must be non-empty")
        .encounter
}

fn encounter_class_for_map_node(
    category: MapNodeCategory,
    kind_id: &str,
    run_progression: &RunProgression,
) -> Option<PveEncounterClass> {
    match category {
        MapNodeCategory::Boss if run_progression.is_final_standard_floor() => {
            Some(PveEncounterClass::FinalBoss)
        }
        MapNodeCategory::Boss => Some(PveEncounterClass::NormalBoss),
        MapNodeCategory::Combat if kind_id.contains("elite") => Some(PveEncounterClass::Elite),
        MapNodeCategory::Combat => Some(PveEncounterClass::Normal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{
        abnormality_research::RunAbnormalityResearchEntry,
        combat_preview::CombatNodeType,
        data::run_policy_data::{
            EmptyEncounterCandidateFallbackPolicy, EncounterRepeatWeightingPolicy,
        },
        enums::{RewardMode, RiskLevel},
        map::{MapNodeId, MapNodeKindId, MapNodeState, MapNodeVisibility, MapSlotId},
    };
    use std::collections::BTreeMap;
    use uuid::Uuid;

    fn elite_encounter(id: &str, abnormality_id: &str) -> PveEncounter {
        PveEncounter {
            id: id.to_string(),
            encounter_class: PveEncounterClass::Elite,
            primary_abnormality_id: Some(abnormality_id.to_string()),
            risk_level: RiskLevel::TETH,
            node_type: Some(CombatNodeType::Defense),
            mission_variant: None,
            survive_timer_ms: None,
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: Vec::new(),
            suppression_research: None,
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: Vec::new(),
            static_obstacles: Vec::new(),
        }
    }

    fn normal_encounter(id: &str) -> PveEncounter {
        PveEncounter {
            id: id.to_string(),
            encounter_class: PveEncounterClass::Normal,
            primary_abnormality_id: None,
            risk_level: RiskLevel::TETH,
            node_type: Some(CombatNodeType::Defense),
            mission_variant: None,
            survive_timer_ms: None,
            reward_mode: RewardMode::ClaimAll,
            reward_uuids: Vec::new(),
            suppression_research: None,
            battlefield: None,
            tactical_plan: None,
            win_condition: None,
            waves: Vec::new(),
            static_obstacles: Vec::new(),
        }
    }

    fn elite_node() -> MapNode {
        MapNode {
            id: MapNodeId::new(Uuid::from_u128(0xE117E)),
            depth: 2,
            lane: 1,
            slot_id: MapSlotId::for_grid_position(2, 1),
            kind_id: MapNodeKindId::new("combat_elite"),
            category: MapNodeCategory::Combat,
            state: MapNodeState::Available,
            visibility: MapNodeVisibility::Revealed,
            payload: MapNodePayload::Encounter { encounter_id: None },
            omen: None,
        }
    }

    fn endless_progression_at_floor(floor_index: u32) -> RunProgression {
        let mut progression = RunProgression::new(99, GameMode::Endless, 3);
        progression.mode_state = crate::game::map::RunProgressionModeState::Endless { floor_index };
        progression
    }

    fn standard_progression_at_floor(floor_index: u8) -> RunProgression {
        let mut progression = RunProgression::new(99, GameMode::Standard, 3);
        progression.mode_state = crate::game::map::RunProgressionModeState::Standard {
            floor_index,
            max_floors: 3,
        };
        progression
    }

    fn research_entry(
        last_encountered_floor: Option<u32>,
        response_complete: bool,
    ) -> RunAbnormalityResearchEntry {
        RunAbnormalityResearchEntry {
            research_points: if response_complete { 100 } else { 0 },
            research_required: 100,
            response_complete,
            suppression_wins: 0,
            last_encountered_floor,
            completed_at_floor: response_complete.then_some(0),
            unique_fragment_granted: response_complete,
        }
    }

    fn research(entries: &[(&str, RunAbnormalityResearchEntry)]) -> RunAbnormalityResearchState {
        RunAbnormalityResearchState {
            entries: entries
                .iter()
                .map(|(id, entry)| ((*id).to_string(), entry.clone()))
                .collect::<BTreeMap<_, _>>(),
        }
    }

    fn encounter_history(entries: &[(&str, u32)]) -> RunAbnormalityEncounterHistory {
        let mut history = RunAbnormalityEncounterHistory::default();
        for (abnormality_id, floor_index) in entries {
            history.record_appearance(abnormality_id, *floor_index);
        }
        history
    }

    fn repeat_policy() -> EncounterRepeatWeightingPolicy {
        EncounterRepeatWeightingPolicy {
            recent_floor_exclusion_window: 1,
            response_complete_weight_multiplier_percent: 25,
            incomplete_weight_multiplier_percent: 100,
            empty_pool_fallback: EmptyEncounterCandidateFallbackPolicy::ResetSoftRepeatConstraints,
        }
    }

    fn run_policy_with_repeat_policy(policy: EncounterRepeatWeightingPolicy) -> RunPolicyData {
        let mut run_policy = RunPolicyData::builtin();
        run_policy.encounter_repeat_weighting = policy;
        run_policy
    }

    #[test]
    fn endless_repeat_weighting_excludes_recent_abnormality_candidates() {
        let recent = elite_encounter("recent", "abno_recent");
        let open = elite_encounter("open", "abno_open");
        let encounters = [&recent, &open];
        let node = elite_node();
        let progression = endless_progression_at_floor(1);
        let research = research(&[
            ("abno_recent", research_entry(Some(0), false)),
            ("abno_open", research_entry(None, false)),
        ]);
        let encounter_history = encounter_history(&[("abno_recent", 0)]);
        let run_policy = run_policy_with_repeat_policy(repeat_policy());

        let selected = select_encounter_for_map_node(
            &encounters,
            &node,
            &progression,
            &research,
            &encounter_history,
            &run_policy,
        );

        assert_eq!(selected.id, "open");
    }

    #[test]
    fn recent_appearance_history_excludes_candidate_without_research_victory() {
        let recent = elite_encounter("appeared_without_win", "abno_recent");
        let open = elite_encounter("open", "abno_open");
        let encounters = [&recent, &open];
        let node = elite_node();
        let progression = endless_progression_at_floor(1);
        let research = research(&[]);
        let encounter_history = encounter_history(&[("abno_recent", 0)]);
        let run_policy = run_policy_with_repeat_policy(repeat_policy());

        let selected = select_encounter_for_map_node(
            &encounters,
            &node,
            &progression,
            &research,
            &encounter_history,
            &run_policy,
        );

        assert_eq!(selected.id, "open");
    }

    #[test]
    fn empty_repeat_candidate_pool_resets_soft_constraints_but_keeps_hard_constraints() {
        let recent = elite_encounter("recent_elite", "abno_recent");
        let wrong_class = normal_encounter("normal_decoy");
        let encounters = [&recent, &wrong_class];
        let node = elite_node();
        let progression = endless_progression_at_floor(1);
        let research = research(&[("abno_recent", research_entry(Some(0), true))]);
        let encounter_history = encounter_history(&[("abno_recent", 0)]);
        let run_policy = run_policy_with_repeat_policy(repeat_policy());

        let selected = select_encounter_for_map_node(
            &encounters,
            &node,
            &progression,
            &research,
            &encounter_history,
            &run_policy,
        );

        assert_eq!(selected.id, "recent_elite");
    }

    #[test]
    fn standard_selection_does_not_apply_endless_repeat_soft_constraints() {
        let recent = elite_encounter("standard_recent_elite", "abno_recent");
        let encounters = [&recent];
        let node = elite_node();
        let progression = standard_progression_at_floor(0);
        let research = research(&[("abno_recent", research_entry(Some(0), true))]);
        let encounter_history = encounter_history(&[("abno_recent", 0)]);
        let run_policy = run_policy_with_repeat_policy(repeat_policy());

        let selected = select_encounter_for_map_node(
            &encounters,
            &node,
            &progression,
            &research,
            &encounter_history,
            &run_policy,
        );

        assert_eq!(selected.id, "standard_recent_elite");
    }

    #[test]
    fn response_complete_candidates_receive_lower_repeat_weight() {
        let complete = elite_encounter("complete", "abno_complete");
        let incomplete = elite_encounter("incomplete", "abno_incomplete");
        let candidates = [&complete, &incomplete];
        let research = research(&[
            ("abno_complete", research_entry(None, true)),
            ("abno_incomplete", research_entry(None, false)),
        ]);

        let weighted = encounter_repeat_weighted_candidates(
            &candidates,
            4,
            &research,
            &RunAbnormalityEncounterHistory::default(),
            repeat_policy(),
        );

        let complete_weight = weighted
            .iter()
            .find(|candidate| candidate.encounter.id == "complete")
            .map(|candidate| candidate.weight)
            .expect("complete candidate must exist");
        let incomplete_weight = weighted
            .iter()
            .find(|candidate| candidate.encounter.id == "incomplete")
            .map(|candidate| candidate.weight)
            .expect("incomplete candidate must exist");

        assert_eq!(complete_weight, 25);
        assert_eq!(incomplete_weight, 100);
    }
}
