use std::collections::{HashMap, HashSet};

use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

use crate::game::{
    data::live_ron::GameplayMapCatalogs,
    data::run_policy_data::{CombatDifficultyProfile, CombatNodeDifficultyRole, RunPolicyData},
    determinism,
    map::{
        FacilityPoolRunState, FacilityRoomSlot, FacilityRunState, FacilitySlotRole,
        FacilityTemplate, FacilityTemplateKind, FacilityTemplatePool, GameMode, MapNode,
        MapNodeCategory, MapNodeCombatDifficulty, MapNodeCombatDifficultyClass, MapNodeDefinition,
        MapNodeId, MapNodeKindId, MapNodePayload, MapNodeState, MapNodeVisibility, MapSlotId,
        MapTemplateId, MapTerminalNode, NodeAssignmentProfile, RunMap,
    },
};

const FACILITY_SELECTION_NS: u64 = 0x4641_4350_4F4F_4C;
const FACILITY_ASSIGNMENT_NS: u64 = 0x4641_4341_5353_4E;
const FACILITY_NODE_NS: u64 = 0x4641_434E_4F44_45;
const FACILITY_DIFFICULTY_NS: u64 = 0x4641_4344_4946_46;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedFacilityMap {
    pub map: RunMap,
    pub facility_run_state: FacilityRunState,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FacilityGenerationRequirements {
    pub guarantee_reachable_shop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FacilityCapstoneEntryInput {
    pub slot_id: MapSlotId,
    pub payload: MapNodePayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FacilityCapstoneRequest {
    pub template_id: MapTemplateId,
    pub entries: Vec<FacilityCapstoneEntryInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GeneratedFacilityCapstone {
    pub map: RunMap,
    pub entry_node_ids: Vec<MapNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlotAssignment {
    category: MapNodeCategory,
    fixed_kind_id: Option<MapNodeKindId>,
    payload_override: Option<MapNodePayload>,
}

pub(crate) struct FacilityMapGenerator;

impl FacilityMapGenerator {
    #[allow(dead_code)]
    pub fn generate(
        run_seed: u64,
        mode: GameMode,
        floor_index: u32,
        current_state: &FacilityRunState,
        catalogs: &GameplayMapCatalogs,
        run_policy: &RunPolicyData,
        floor_power_anchor: u64,
    ) -> Result<GeneratedFacilityMap, String> {
        Self::generate_with_requirements(
            run_seed,
            mode,
            floor_index,
            current_state,
            catalogs,
            run_policy,
            floor_power_anchor,
            FacilityGenerationRequirements::default(),
        )
    }

    pub(crate) fn generate_with_requirements(
        run_seed: u64,
        mode: GameMode,
        floor_index: u32,
        current_state: &FacilityRunState,
        catalogs: &GameplayMapCatalogs,
        run_policy: &RunPolicyData,
        floor_power_anchor: u64,
        requirements: FacilityGenerationRequirements,
    ) -> Result<GeneratedFacilityMap, String> {
        if mode == GameMode::Expedition {
            if let Some(layout) = catalogs.facility_templates.layouts.iter().find(|l| {
                current_state.rooms.as_ref().map_or(l.expedition_default, |r| r.layout_id == l.id)
            }) {
                let room_id = current_state.rooms.as_ref().map_or(layout.initial_room_id.as_str(), |r| r.current_room_id.as_str());
                return Self::generate_room(run_seed, room_id, current_state, layout, catalogs, run_policy, floor_power_anchor);
            }
        }
        let schedule = catalogs.node_assignment_profiles.resolve(
            &catalogs.facility_templates,
            mode,
            floor_index,
        )?;
        let mut next_state = current_state.clone();
        let ordered_candidates = ordered_pool_candidates(
            run_seed,
            schedule.pool,
            next_state
                .pools
                .entry(schedule.pool.id.clone())
                .or_default(),
        );

        let mut last_error = None;
        for template_id in ordered_candidates {
            let template = catalogs
                .facility_templates
                .template(&template_id)
                .ok_or_else(|| {
                    format!(
                        "facility pool '{}' references missing template '{}'",
                        schedule.pool.id,
                        template_id.as_str()
                    )
                })?;
            if requirements.guarantee_reachable_shop
                && !template
                    .slots
                    .iter()
                    .any(|slot| slot.tags.iter().any(|tag| tag == "shop"))
            {
                last_error = Some(format!(
                    "template '{}' has no reachable Shop-tag slot required by the monolith exchange guarantee",
                    template.id.as_str()
                ));
                continue;
            }
            let assignment_seed = determinism::seed_with_namespace(
                run_seed
                    ^ u64::from(floor_index).wrapping_mul(0x9E37_79B9)
                    ^ stable_text_seed(template.id.as_str()),
                FACILITY_ASSIGNMENT_NS,
            );
            match assign_template(
                template,
                schedule.profile,
                catalogs,
                assignment_seed,
                requirements,
            ) {
                Ok(assignments) => {
                    consume_selected_template(
                        next_state
                            .pools
                            .get_mut(&schedule.pool.id)
                            .expect("pool state was initialized"),
                        &template_id,
                    );
                    let mut map = build_run_map(
                        template,
                        &assignments,
                        catalogs,
                        run_seed,
                        floor_index,
                        assignment_seed,
                    )?;
                    let difficulty_profile = run_policy
                        .combat_difficulty_profile(&schedule.profile.difficulty_profile_id)
                        .ok_or_else(|| {
                            format!(
                                "node assignment profile '{}' references missing combat difficulty profile '{}'",
                                schedule.profile.id, schedule.profile.difficulty_profile_id
                            )
                        })?;
                    if !difficulty_profile.authored_fixed {
                        assign_combat_difficulties(
                            &mut map,
                            &assignments,
                            difficulty_profile,
                            run_policy.combat_difficulty.power_model_version,
                            floor_power_anchor,
                            assignment_seed,
                        )?;
                    } else {
                        assign_fixed_standard_combat_difficulties(
                            &mut map,
                            &assignments,
                            difficulty_profile,
                            run_policy.combat_difficulty.power_model_version,
                            floor_power_anchor,
                        )?;
                    }
                    return Ok(GeneratedFacilityMap {
                        map,
                        facility_run_state: next_state,
                    });
                }
                Err(error) => last_error = Some(error),
            }
        }

        Err(format!(
            "facility pool '{}' has no template that satisfies profile '{}': {}",
            schedule.pool.id,
            schedule.profile.id,
            last_error.unwrap_or_else(|| "pool produced no candidates".to_string())
        ))
    }

    fn generate_room(
        run_seed: u64,
        room_id: &str,
        current_state: &FacilityRunState,
        layout: &super::FacilityLayout,
        catalogs: &GameplayMapCatalogs,
        run_policy: &RunPolicyData,
        anchor: u64,
    ) -> Result<GeneratedFacilityMap, String> {
        let room = layout.room(room_id).ok_or("missing current facility room")?;
        if !room.support.is_supported() { return Err(format!("unsupported facility room '{room_id}'")); }
        let template = room.template_id.as_ref().and_then(|id| catalogs.facility_templates.template(id)).ok_or("missing room template")?;
        let seed = determinism::seed_with_namespace(run_seed ^ stable_text_seed(&layout.id) ^ stable_text_seed(room_id), FACILITY_ASSIGNMENT_NS);
        let mut rng = StdRng::seed_from_u64(seed);
        let metadata = layout.sample_activities(room, template, &mut rng)?;
        let mut assignments = HashMap::new();
        for slot in &template.slots {
            let assignment = if slot.role == FacilitySlotRole::Start {
                SlotAssignment { category: MapNodeCategory::Start, fixed_kind_id: None, payload_override: None }
            } else {
                let m = metadata.get(&slot.id).ok_or("missing room activity assignment")?;
                let activity = layout.activity(&m.activity_id).ok_or("missing assigned activity")?;
                let definition = catalogs.map_node_definitions.nodes.iter().find(|d| d.kind_id == activity.kind_id).ok_or("missing activity node kind")?;
                if !slot.allowed_categories.contains(&definition.category) { return Err(format!("room '{room_id}' slot '{}' rejects activity category",slot.id.as_str())); }
                SlotAssignment { category: definition.category, fixed_kind_id: Some(activity.kind_id.clone()), payload_override: Some(activity.payload.clone()) }
            };
            assignments.insert(slot.id.clone(),assignment);
        }
        let mut map = build_run_map(template,&assignments,catalogs,seed,room.depth.saturating_sub(1),seed)?;
        let difficulty = run_policy.combat_difficulty_profile(&layout.difficulty_profile_id).ok_or("unknown room difficulty profile")?;
        assign_combat_difficulties(&mut map,&assignments,difficulty,run_policy.combat_difficulty.power_model_version,anchor,seed)?;
        let mut next = current_state.clone();
        if next.rooms.is_none() {
            next.rooms = Some(super::FacilityExplorationState {
                layout_id: layout.id.clone(), current_room_id: room.id.clone(),
                visited_rooms: Default::default(), activity_metadata: Default::default(),
                golden_boughs: layout.choose_golden_bough_plan(run_seed)?,
            });
        }
        let state = next.rooms.as_mut().expect("room state initialized");
        for node in &map.nodes {
            if let Some(m) = metadata.get(&node.slot_id) {state.activity_metadata.insert(node.id,m.clone());}
        }
        Ok(GeneratedFacilityMap {map,facility_run_state:next})
    }

    pub fn generate_capstone(
        run_seed: u64,
        floor_index: u32,
        request: &FacilityCapstoneRequest,
        catalogs: &GameplayMapCatalogs,
    ) -> Result<GeneratedFacilityCapstone, String> {
        let template = catalogs
            .facility_templates
            .template(&request.template_id)
            .ok_or_else(|| {
                format!(
                    "capstone request references missing template '{}'",
                    request.template_id.as_str()
                )
            })?;
        if template.kind != FacilityTemplateKind::Capstone {
            return Err(format!(
                "facility template '{}' is not a capstone",
                template.id.as_str()
            ));
        }

        let entry_slots = template
            .slots
            .iter()
            .filter(|slot| slot.role == FacilitySlotRole::Entry)
            .collect::<Vec<_>>();
        if entry_slots.len() != request.entries.len() {
            return Err(format!(
                "capstone '{}' requires {} entry inputs, received {}",
                template.id.as_str(),
                entry_slots.len(),
                request.entries.len()
            ));
        }
        let mut input_slot_ids = HashSet::new();
        for input in &request.entries {
            if !input_slot_ids.insert(input.slot_id.as_str()) {
                return Err(format!(
                    "capstone '{}' has duplicate entry input '{}'",
                    template.id.as_str(),
                    input.slot_id.as_str()
                ));
            }
            if !entry_slots.iter().any(|slot| slot.id == input.slot_id) {
                return Err(format!(
                    "capstone '{}' input '{}' is not an Entry slot",
                    template.id.as_str(),
                    input.slot_id.as_str()
                ));
            }
        }

        let assignments = template
            .slots
            .iter()
            .map(|slot| {
                let assignment = match slot.role {
                    FacilitySlotRole::Start => SlotAssignment {
                        category: MapNodeCategory::Start,
                        fixed_kind_id: Some(MapNodeKindId::new("start")),
                        payload_override: None,
                    },
                    FacilitySlotRole::Entry
                    | FacilitySlotRole::Ordinary
                    | FacilitySlotRole::ManifestedBonus
                    | FacilitySlotRole::Terminal(_) => SlotAssignment {
                        category: slot.allowed_categories[0],
                        fixed_kind_id: None,
                        payload_override: None,
                    },
                };
                (slot.id.clone(), assignment)
            })
            .collect::<HashMap<_, _>>();
        let assignment_seed = determinism::seed_with_namespace(
            run_seed
                ^ u64::from(floor_index).wrapping_mul(0x9E37_79B9)
                ^ stable_text_seed(template.id.as_str()),
            FACILITY_ASSIGNMENT_NS,
        );
        let mut map = build_run_map(
            template,
            &assignments,
            catalogs,
            run_seed,
            floor_index,
            assignment_seed,
        )?;
        let mut entry_node_ids = Vec::with_capacity(request.entries.len());
        for input in &request.entries {
            let node = map
                .nodes
                .iter_mut()
                .find(|node| node.slot_id == input.slot_id)
                .ok_or_else(|| {
                    format!(
                        "generated capstone '{}' is missing Entry slot '{}'",
                        template.id.as_str(),
                        input.slot_id.as_str()
                    )
                })?;
            node.payload = input.payload.clone();
            entry_node_ids.push(node.id);
        }
        entry_node_ids.sort_by_key(|id| id.0.as_u128());
        Ok(GeneratedFacilityCapstone {
            map,
            entry_node_ids,
        })
    }
}

#[cfg(test)]
pub(crate) fn test_facility_map(run_seed: u64) -> RunMap {
    let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
    FacilityMapGenerator::generate(
        run_seed,
        GameMode::Tutorial,
        0,
        &FacilityRunState::default(),
        &catalogs,
        &crate::game::data::run_policy_data::RunPolicyData::builtin(),
        100,
    )
    .expect("embedded Standard Floor 1 facility must generate")
    .map
}

fn ordered_pool_candidates(
    run_seed: u64,
    pool: &FacilityTemplatePool,
    state: &mut FacilityPoolRunState,
) -> Vec<MapTemplateId> {
    if state.remaining_template_ids.is_empty() {
        if state.used_template_ids.len() >= pool.template_ids.len() {
            state.cycle = state.cycle.saturating_add(1);
            state.used_template_ids.clear();
        }
        let mut candidates = pool
            .template_ids
            .iter()
            .filter(|template_id| !state.used_template_ids.contains(*template_id))
            .cloned()
            .collect::<Vec<_>>();
        let seed = determinism::seed_with_namespace(
            run_seed
                ^ stable_text_seed(&pool.id)
                ^ u64::from(state.cycle).wrapping_mul(0x9E37_79B9),
            FACILITY_SELECTION_NS,
        );
        candidates.shuffle(&mut StdRng::seed_from_u64(seed));
        state.remaining_template_ids = candidates;
    }
    state.remaining_template_ids.clone()
}

fn consume_selected_template(state: &mut FacilityPoolRunState, selected: &MapTemplateId) {
    state
        .remaining_template_ids
        .retain(|template_id| template_id != selected);
    state.used_template_ids.insert(selected.clone());
}

fn assign_template(
    template: &FacilityTemplate,
    profile: &NodeAssignmentProfile,
    catalogs: &GameplayMapCatalogs,
    seed: u64,
    requirements: FacilityGenerationRequirements,
) -> Result<HashMap<MapSlotId, SlotAssignment>, String> {
    if !profile.fixed_slot_assignments.is_empty() {
        return assign_fixed_template(template, profile, catalogs);
    }
    let mut assignments = HashMap::new();
    for slot in &template.slots {
        let fixed = match slot.role {
            FacilitySlotRole::Start => Some(SlotAssignment {
                category: MapNodeCategory::Start,
                fixed_kind_id: Some(MapNodeKindId::new("start")),
                payload_override: None,
            }),
            FacilitySlotRole::Entry | FacilitySlotRole::Terminal(_) => Some(SlotAssignment {
                category: slot.allowed_categories[0],
                fixed_kind_id: None,
                payload_override: None,
            }),
            FacilitySlotRole::Ordinary | FacilitySlotRole::ManifestedBonus
                if requirements.guarantee_reachable_shop
                    && slot.tags.iter().any(|tag| tag == "shop") =>
            {
                Some(SlotAssignment {
                    category: MapNodeCategory::Shop,
                    fixed_kind_id: None,
                    payload_override: None,
                })
            }
            FacilitySlotRole::Ordinary | FacilitySlotRole::ManifestedBonus => {
                tagged_assignment(slot, profile)?
            }
        };
        if let Some(fixed) = fixed {
            assignments.insert(slot.id.clone(), fixed);
        }
    }

    let mut constraints = profile
        .category_constraints
        .iter()
        .map(|constraint| Ok((constraint.category, constraint.bounds()?)))
        .collect::<Result<HashMap<_, _>, String>>()?;
    if requirements.guarantee_reachable_shop {
        constraints.insert(MapNodeCategory::Shop, (1, 1));
    }
    validate_partial_counts(template, &assignments, &constraints)?;

    let mut unassigned = template
        .slots
        .iter()
        .filter(|slot| !assignments.contains_key(&slot.id))
        .collect::<Vec<_>>();
    unassigned.sort_by_key(|slot| (slot.allowed_categories.len(), slot.id.as_str()));
    let mut rng = StdRng::seed_from_u64(seed);
    if !backtrack_assign(
        template,
        profile,
        &constraints,
        &unassigned,
        0,
        &mut assignments,
        &mut rng,
    ) {
        return Err(format!(
            "template '{}' cannot satisfy assignment profile '{}'",
            template.id.as_str(),
            profile.id
        ));
    }

    for (slot_id, assignment) in &assignments {
        if assignment.category == MapNodeCategory::Start {
            continue;
        }
        if let Some(kind_id) = &assignment.fixed_kind_id {
            let definition = catalogs
                .map_node_definitions
                .nodes
                .iter()
                .find(|definition| definition.kind_id == *kind_id)
                .ok_or_else(|| {
                    format!(
                        "slot '{}' requires missing node definition '{}'",
                        slot_id.as_str(),
                        kind_id.as_str()
                    )
                })?;
            if definition.category != assignment.category {
                return Err(format!(
                    "slot '{}' fixed node '{}' has category {:?}, expected {:?}",
                    slot_id.as_str(),
                    kind_id.as_str(),
                    definition.category,
                    assignment.category
                ));
            }
        } else if catalogs
            .map_node_definitions
            .category_candidates(assignment.category)
            .is_empty()
        {
            return Err(format!(
                "slot '{}' category {:?} has no node definition",
                slot_id.as_str(),
                assignment.category
            ));
        }
    }
    Ok(assignments)
}

fn assign_fixed_template(
    template: &FacilityTemplate,
    profile: &NodeAssignmentProfile,
    catalogs: &GameplayMapCatalogs,
) -> Result<HashMap<MapSlotId, SlotAssignment>, String> {
    let expected = template
        .slots
        .iter()
        .filter(|slot| slot.role != FacilitySlotRole::Start)
        .map(|slot| slot.id.as_str())
        .collect::<HashSet<_>>();
    let authored = profile
        .fixed_slot_assignments
        .iter()
        .map(|assignment| assignment.slot_id.as_str())
        .collect::<HashSet<_>>();
    if authored.len() != profile.fixed_slot_assignments.len() || authored != expected {
        return Err(format!(
            "fixed assignment profile '{}' must author every non-Start slot exactly once",
            profile.id
        ));
    }

    let mut assignments = HashMap::new();
    for slot in &template.slots {
        if slot.role == FacilitySlotRole::Start {
            assignments.insert(
                slot.id.clone(),
                SlotAssignment {
                    category: MapNodeCategory::Start,
                    fixed_kind_id: Some(MapNodeKindId::new("start")),
                    payload_override: None,
                },
            );
            continue;
        }
        let fixed = profile
            .fixed_slot_assignments
            .iter()
            .find(|assignment| assignment.slot_id == slot.id)
            .expect("fixed assignment set equality was checked");
        let definition = catalogs
            .map_node_definitions
            .nodes
            .iter()
            .find(|definition| definition.kind_id == fixed.kind_id)
            .ok_or_else(|| {
                format!(
                    "fixed slot '{}' references missing node definition '{}'",
                    slot.id.as_str(),
                    fixed.kind_id.as_str()
                )
            })?;
        if !slot.allowed_categories.contains(&definition.category) {
            return Err(format!(
                "fixed slot '{}' cannot accept {:?} node '{}'",
                slot.id.as_str(),
                definition.category,
                fixed.kind_id.as_str()
            ));
        }
        assignments.insert(
            slot.id.clone(),
            SlotAssignment {
                category: definition.category,
                fixed_kind_id: Some(fixed.kind_id.clone()),
                payload_override: Some(fixed.payload.clone()),
            },
        );
    }
    Ok(assignments)
}

fn tagged_assignment(
    slot: &FacilityRoomSlot,
    profile: &NodeAssignmentProfile,
) -> Result<Option<SlotAssignment>, String> {
    let matches = profile
        .tagged_slot_assignments
        .iter()
        .filter(|assignment| slot.tags.contains(&assignment.slot_tag))
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(format!(
            "slot '{}' matches multiple tagged assignments in profile '{}'",
            slot.id.as_str(),
            profile.id
        ));
    }
    let Some(assignment) = matches.first() else {
        return Ok(None);
    };
    if !slot.allowed_categories.contains(&assignment.category) {
        return Err(format!(
            "slot '{}' tag '{}' cannot accept category {:?}",
            slot.id.as_str(),
            assignment.slot_tag,
            assignment.category
        ));
    }
    Ok(Some(SlotAssignment {
        category: assignment.category,
        fixed_kind_id: assignment.kind_id.clone(),
        payload_override: None,
    }))
}

fn backtrack_assign(
    template: &FacilityTemplate,
    profile: &NodeAssignmentProfile,
    constraints: &HashMap<MapNodeCategory, (usize, usize)>,
    slots: &[&FacilityRoomSlot],
    index: usize,
    assignments: &mut HashMap<MapSlotId, SlotAssignment>,
    rng: &mut StdRng,
) -> bool {
    if index == slots.len() {
        return final_counts_satisfy(assignments, constraints);
    }
    let slot = slots[index];
    let mut candidates = slot.allowed_categories.clone();
    candidates.retain(|category| {
        profile
            .remaining_category_weights
            .iter()
            .any(|weight| weight.category == *category)
            || constraints.contains_key(category)
    });
    weighted_candidate_order(&mut candidates, profile, rng);

    for category in candidates {
        assignments.insert(
            slot.id.clone(),
            SlotAssignment {
                category,
                fixed_kind_id: None,
                payload_override: None,
            },
        );
        if partial_counts_can_finish(template, assignments, constraints, slots, index + 1)
            && backtrack_assign(
                template,
                profile,
                constraints,
                slots,
                index + 1,
                assignments,
                rng,
            )
        {
            return true;
        }
        assignments.remove(&slot.id);
    }
    false
}

fn weighted_candidate_order(
    candidates: &mut Vec<MapNodeCategory>,
    profile: &NodeAssignmentProfile,
    rng: &mut StdRng,
) {
    candidates.sort_by_key(|category| {
        let weight = profile
            .remaining_category_weights
            .iter()
            .find(|entry| entry.category == *category)
            .map_or(1, |entry| entry.weight);
        let random = rng.gen_range(0..=u64::MAX);
        (
            u64::MAX - random / u64::from(weight.max(1)),
            format!("{category:?}"),
        )
    });
}

fn validate_partial_counts(
    template: &FacilityTemplate,
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    constraints: &HashMap<MapNodeCategory, (usize, usize)>,
) -> Result<(), String> {
    let counts = category_counts(assignments);
    for (category, (_, max)) in constraints {
        if counts.get(category).copied().unwrap_or(0) > *max {
            return Err(format!(
                "template '{}' fixed assignments exceed {:?} max {}",
                template.id.as_str(),
                category,
                max
            ));
        }
    }
    Ok(())
}

fn partial_counts_can_finish(
    _template: &FacilityTemplate,
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    constraints: &HashMap<MapNodeCategory, (usize, usize)>,
    remaining_slots: &[&FacilityRoomSlot],
    next_index: usize,
) -> bool {
    let counts = category_counts(assignments);
    constraints.iter().all(|(category, (min, max))| {
        let current = counts.get(category).copied().unwrap_or(0);
        let possible = remaining_slots[next_index..]
            .iter()
            .filter(|slot| slot.allowed_categories.contains(category))
            .count();
        current <= *max && current.saturating_add(possible) >= *min
    })
}

fn final_counts_satisfy(
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    constraints: &HashMap<MapNodeCategory, (usize, usize)>,
) -> bool {
    let counts = category_counts(assignments);
    constraints.iter().all(|(category, (min, max))| {
        let count = counts.get(category).copied().unwrap_or(0);
        count >= *min && count <= *max
    })
}

fn category_counts(
    assignments: &HashMap<MapSlotId, SlotAssignment>,
) -> HashMap<MapNodeCategory, usize> {
    let mut counts = HashMap::new();
    for assignment in assignments.values() {
        *counts.entry(assignment.category).or_default() += 1;
    }
    counts
}

fn build_run_map(
    template: &FacilityTemplate,
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    catalogs: &GameplayMapCatalogs,
    run_seed: u64,
    floor_index: u32,
    assignment_seed: u64,
) -> Result<RunMap, String> {
    let node_seed = determinism::seed_with_namespace(
        run_seed ^ u64::from(floor_index) ^ stable_text_seed(template.id.as_str()),
        FACILITY_NODE_NS,
    );
    let mut nodes = Vec::with_capacity(template.slots.len());
    let mut node_by_slot = HashMap::new();
    let mut rng = StdRng::seed_from_u64(assignment_seed ^ FACILITY_NODE_NS);

    for (index, slot) in template.slots.iter().enumerate() {
        let assignment = assignments
            .get(&slot.id)
            .ok_or_else(|| format!("facility slot '{}' has no assignment", slot.id.as_str()))?;
        let node_id = MapNodeId::new(determinism::uuid_v4_from_seed(
            node_seed,
            FACILITY_NODE_NS,
            u64::try_from(index).map_err(|_| "facility node index overflowed u64")?,
        ));
        let definition = resolve_definition(assignment, catalogs, &mut rng)?;
        let (kind_id, default_payload) = definition.map_or_else(
            || (MapNodeKindId::new("start"), MapNodePayload::None),
            |definition| (definition.kind_id.clone(), definition.payload.clone()),
        );
        let payload = assignment
            .payload_override
            .clone()
            .unwrap_or(default_payload);
        let state = match slot.role {
            FacilitySlotRole::Start => MapNodeState::Completed,
            FacilitySlotRole::Entry => MapNodeState::Available,
            FacilitySlotRole::ManifestedBonus => MapNodeState::Locked,
            FacilitySlotRole::Ordinary | FacilitySlotRole::Terminal(_) => MapNodeState::Unavailable,
        };
        let visibility = match slot.role {
            FacilitySlotRole::ManifestedBonus => MapNodeVisibility::Concealed,
            FacilitySlotRole::Terminal(
                crate::game::map::FacilityTerminalResolution::BossOmenExitGate,
            ) => MapNodeVisibility::Concealed,
            FacilitySlotRole::Start
            | FacilitySlotRole::Entry
            | FacilitySlotRole::Ordinary
            | FacilitySlotRole::Terminal(_) => MapNodeVisibility::Revealed,
        };
        nodes.push(MapNode {
            id: node_id,
            slot_id: slot.id.clone(),
            kind_id,
            category: assignment.category,
            state,
            visibility,
            payload,
            omen: None,
            combat_difficulty: None,
        });
        node_by_slot.insert(slot.id.as_str(), node_id);
    }

    let start_slot_ids = template.navigation_start_slot_ids();
    let start_node_ids = nodes
        .iter()
        .filter(|node| start_slot_ids.contains(&node.slot_id))
        .map(|node| node.id)
        .collect::<Vec<_>>();
    for node in &mut nodes {
        if start_node_ids.contains(&node.id) && node.visibility != MapNodeVisibility::Concealed {
            node.state = MapNodeState::Available;
        }
    }
    let terminals = template
        .slots
        .iter()
        .filter_map(|slot| {
            let FacilitySlotRole::Terminal(resolution) = slot.role else {
                return None;
            };
            Some(MapTerminalNode {
                node_id: *node_by_slot.get(slot.id.as_str())?,
                resolution,
            })
        })
        .collect::<Vec<_>>();
    if terminals.is_empty() {
        return Err(format!(
            "facility template '{}' has no terminal",
            template.id.as_str()
        ));
    }

    let mut map = RunMap {
        map_template_id: template.id.clone(),
        facility_template_version: template.version,
        nodes,
        start_node_ids,
        terminals,
    };
    map.start_node_ids.sort_by_key(|id| id.0.as_u128());
    Ok(map)
}

fn assign_combat_difficulties(
    map: &mut RunMap,
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    profile: &CombatDifficultyProfile,
    power_model_version: u32,
    floor_power_anchor: u64,
    seed: u64,
) -> Result<(), String> {
    let terminal_roles = map
        .terminals
        .iter()
        .map(|terminal| {
            let role = match terminal.resolution {
                crate::game::map::FacilityTerminalResolution::RunFinalBoss => {
                    Some(CombatNodeDifficultyRole::FinalBoss)
                }
                crate::game::map::FacilityTerminalResolution::StandardCompletion => {
                    Some(CombatNodeDifficultyRole::NormalBoss)
                }
                crate::game::map::FacilityTerminalResolution::SpecialRunCompletion => {
                    Some(CombatNodeDifficultyRole::Revival)
                }
                crate::game::map::FacilityTerminalResolution::FloorGate
                | crate::game::map::FacilityTerminalResolution::BossOmenExitGate => None,
            };
            (terminal.node_id, role)
        })
        .collect::<HashMap<_, _>>();
    let mut ordinary_node_ids = Vec::new();
    for node in &mut map.nodes {
        if !matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) {
            continue;
        }
        let role = if node.omen.is_some() {
            Some(CombatNodeDifficultyRole::BossOmen)
        } else if assignments
            .get(&node.slot_id)
            .and_then(|assignment| assignment.fixed_kind_id.as_ref())
            .is_some_and(|kind_id| kind_id.as_str() == "combat_elite")
        {
            Some(CombatNodeDifficultyRole::Elite)
        } else if node.category == MapNodeCategory::Boss {
            terminal_roles
                .get(&node.id)
                .copied()
                .flatten()
                .or(Some(CombatNodeDifficultyRole::NormalBoss))
        } else {
            None
        };
        if let Some(role) = role {
            let tuning = profile
                .role_tuning
                .iter()
                .find(|entry| entry.role == role)
                .ok_or_else(|| {
                    format!(
                        "combat difficulty profile '{}' has no tuning for {:?}",
                        profile.id, role
                    )
                })?;
            node.combat_difficulty = Some(MapNodeCombatDifficulty {
                power_model_version,
                difficulty_profile_id: profile.id.clone(),
                class: MapNodeCombatDifficultyClass::Role(role),
                revival: None,
                floor_power_anchor,
                target_power: percent_of_anchor_ceil(
                    floor_power_anchor,
                    tuning.tuning.target_power_percent,
                ),
            });
        } else {
            ordinary_node_ids.push(node.id);
        }
    }

    ordinary_node_ids.sort_by_key(|node_id| node_id.0.as_u128());
    let ordinary_count = ordinary_node_ids.len();
    let mut rng = StdRng::seed_from_u64(determinism::seed_with_namespace(
        seed,
        FACILITY_DIFFICULTY_NS,
    ));
    ordinary_node_ids.shuffle(&mut rng);
    let mut allocations = profile
        .ordinary_bands
        .iter()
        .map(|entry| {
            let product = ordinary_count.saturating_mul(entry.quota_percent as usize);
            (entry.band, product / 100, product % 100, rng.gen::<u64>())
        })
        .collect::<Vec<_>>();
    let assigned = allocations
        .iter()
        .map(|(_, count, _, _)| *count)
        .sum::<usize>();
    let mut remaining = ordinary_count.saturating_sub(assigned);
    allocations.sort_by_key(|(_, _, remainder, tie)| (std::cmp::Reverse(*remainder), *tie));
    for (_, count, _, _) in &mut allocations {
        if remaining == 0 {
            break;
        }
        *count = count.saturating_add(1);
        remaining -= 1;
    }

    let mut bands = allocations
        .iter()
        .flat_map(|(band, count, _, _)| std::iter::repeat_n(*band, *count))
        .collect::<Vec<_>>();
    bands.shuffle(&mut rng);
    for (node_id, band) in ordinary_node_ids.into_iter().zip(bands) {
        let tuning = profile
            .ordinary_bands
            .iter()
            .find(|entry| entry.band == band)
            .expect("validated combat difficulty profile must define every ordinary band");
        let node = map
            .nodes
            .iter_mut()
            .find(|node| node.id == node_id)
            .expect("ordinary difficulty node id came from map");
        node.combat_difficulty = Some(MapNodeCombatDifficulty {
            power_model_version,
            difficulty_profile_id: profile.id.clone(),
            class: MapNodeCombatDifficultyClass::Ordinary(band),
            revival: None,
            floor_power_anchor,
            target_power: percent_of_anchor_ceil(
                floor_power_anchor,
                tuning.tuning.target_power_percent,
            ),
        });
    }
    Ok(())
}

fn assign_fixed_standard_combat_difficulties(
    map: &mut RunMap,
    assignments: &HashMap<MapSlotId, SlotAssignment>,
    profile: &CombatDifficultyProfile,
    power_model_version: u32,
    integrity_floor_anchor: u64,
) -> Result<(), String> {
    for node in &mut map.nodes {
        if !matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) {
            continue;
        }
        let class = if node.category == MapNodeCategory::Boss {
            MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::NormalBoss)
        } else if assignments
            .get(&node.slot_id)
            .and_then(|assignment| assignment.fixed_kind_id.as_ref())
            .is_some_and(|kind_id| kind_id.as_str() == "combat_elite")
        {
            MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::Elite)
        } else {
            MapNodeCombatDifficultyClass::Ordinary(
                crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard,
            )
        };
        let tuning_exists = match class {
            MapNodeCombatDifficultyClass::Ordinary(band) => profile
                .ordinary_bands
                .iter()
                .any(|entry| entry.band == band),
            MapNodeCombatDifficultyClass::Role(role) => {
                profile.role_tuning.iter().any(|entry| entry.role == role)
            }
        };
        if !tuning_exists {
            return Err(format!(
                "fixed Standard difficulty profile '{}' has no tuning for {:?}",
                profile.id, class
            ));
        }
        node.combat_difficulty = Some(MapNodeCombatDifficulty {
            power_model_version,
            difficulty_profile_id: profile.id.clone(),
            class,
            revival: None,
            // Fixed Standard never derives tuning from this value. It is retained only
            // as the existing versioned Active Run checkpoint-integrity witness.
            floor_power_anchor: integrity_floor_anchor,
            target_power: 0,
        });
    }
    Ok(())
}

fn percent_of_anchor_ceil(anchor: u64, percent: u32) -> u64 {
    let value = u128::from(anchor)
        .saturating_mul(u128::from(percent))
        .div_ceil(100);
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn resolve_definition<'a>(
    assignment: &SlotAssignment,
    catalogs: &'a GameplayMapCatalogs,
    rng: &mut StdRng,
) -> Result<Option<&'a MapNodeDefinition>, String> {
    if assignment.category == MapNodeCategory::Start {
        return Ok(None);
    }
    if let Some(kind_id) = &assignment.fixed_kind_id {
        return catalogs
            .map_node_definitions
            .nodes
            .iter()
            .find(|definition| definition.kind_id == *kind_id)
            .map(Some)
            .ok_or_else(|| format!("missing fixed node definition '{}'", kind_id.as_str()));
    }
    let candidates = catalogs
        .map_node_definitions
        .category_candidates(assignment.category);
    let total = candidates
        .iter()
        .map(|definition| definition.weight.max(1))
        .sum::<u32>();
    if total == 0 {
        return Err(format!(
            "category {:?} has no weighted node definition",
            assignment.category
        ));
    }
    let mut roll = rng.gen_range(0..total);
    for candidate in candidates {
        let weight = candidate.weight.max(1);
        if roll < weight {
            return Ok(Some(candidate));
        }
        roll -= weight;
    }
    unreachable!("weighted facility node definition roll must select a candidate")
}

fn stable_text_seed(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x1000_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_standard_floors_use_authored_increasing_templates() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let mut state = FacilityRunState::default();
        let mut ids = Vec::new();
        let mut sizes = Vec::new();
        for floor_index in 0..3 {
            let generated = FacilityMapGenerator::generate(
                42,
                GameMode::Tutorial,
                floor_index,
                &state,
                &catalogs,
                &RunPolicyData::builtin(),
                100,
            )
            .unwrap();
            ids.push(generated.map.map_template_id.as_str().to_string());
            sizes.push(generated.map.nodes.len());
            state = generated.facility_run_state;
        }
        assert_eq!(
            ids,
            [
                "standard_intro_a",
                "standard_branching_a",
                "standard_final_a"
            ]
        );
        assert!(sizes[0] < sizes[1] && sizes[1] < sizes[2]);
    }

    #[test]
    fn endless_pool_does_not_repeat_before_exhaustion() {
        let mut catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        std::sync::Arc::make_mut(&mut catalogs.facility_templates).layouts.clear(); // Explicit legacy pool input.
        // Depths 1–4 use the fixed development template; the pool starts at depth 5.
        let first = FacilityMapGenerator::generate(
            77,
            GameMode::Expedition,
            4,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            100,
        )
        .unwrap();
        let second = FacilityMapGenerator::generate(
            77,
            GameMode::Expedition,
            5,
            &first.facility_run_state,
            &catalogs,
            &RunPolicyData::builtin(),
            125,
        )
        .unwrap();
        assert_ne!(first.map.map_template_id, second.map.map_template_id);
    }

    #[test]
    fn monolith_guarantee_authors_exactly_one_shop_on_next_endless_floor() {
        let mut catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        std::sync::Arc::make_mut(&mut catalogs.facility_templates).layouts.clear(); // Explicit legacy pool input.
        let generated = FacilityMapGenerator::generate_with_requirements(
            77,
            GameMode::Expedition,
            4,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            175,
            FacilityGenerationRequirements {
                guarantee_reachable_shop: true,
            },
        )
        .unwrap();
        let shops = generated
            .map
            .nodes
            .iter()
            .filter(|node| node.category == MapNodeCategory::Shop)
            .collect::<Vec<_>>();

        assert_eq!(shops.len(), 1);
        let template = catalogs
            .facility_templates
            .template(&generated.map.map_template_id)
            .unwrap();
        assert!(template
            .slots
            .iter()
            .find(|slot| slot.id == shops[0].slot_id)
            .is_some_and(|slot| slot.tags.iter().any(|tag| tag == "shop")));
    }

    #[test]
    fn same_seed_and_state_reproduce_template_assignment_and_ids() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let left = FacilityMapGenerator::generate(
            91,
            GameMode::Tutorial,
            2,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            150,
        )
        .unwrap();
        let right = FacilityMapGenerator::generate(
            91,
            GameMode::Tutorial,
            2,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            150,
        )
        .unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn standard_is_exact_authored_fixed_and_seed_independent() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let policy = RunPolicyData::builtin();
        let expected = [
            (
                "standard_intro_a",
                crate::game::map::FacilityTerminalResolution::FloorGate,
                vec![
                    ("entry", "start", MapNodePayload::None),
                    (
                        "intake_corridor",
                        "combat_monster",
                        MapNodePayload::Encounter {
                            encounter_id: Some("normal_corroded_patrol".to_string()),
                        },
                    ),
                    (
                        "records_annex",
                        "event_sealed_archive",
                        MapNodePayload::Event {
                            event_id: Some(crate::game::data::event_data::EventId::new(
                                "sealed_archive_recovery_01",
                            )),
                        },
                    ),
                    (
                        "supply_annex",
                        "reward_treasure",
                        MapNodePayload::Reward {
                            reward_pool_id: Some("tutorial_f1_training_budget".to_string()),
                        },
                    ),
                    (
                        "containment_guard",
                        "combat_monster",
                        MapNodePayload::Encounter {
                            encounter_id: Some("normal_breach_response".to_string()),
                        },
                    ),
                    ("floor_gate", "gate_stairs", MapNodePayload::None),
                ],
            ),
            (
                "standard_branching_a",
                crate::game::map::FacilityTerminalResolution::FloorGate,
                vec![
                    ("entry", "start", MapNodePayload::None),
                    (
                        "dispatch_hall",
                        "combat_monster",
                        MapNodePayload::Encounter {
                            encounter_id: Some("normal_black_box_pressure".to_string()),
                        },
                    ),
                    ("save_archive", "save_point", MapNodePayload::SavePoint),
                    (
                        "maintenance_bay",
                        "maintenance",
                        MapNodePayload::Maintenance,
                    ),
                    (
                        "hq_terminal",
                        "headquarters_contact",
                        MapNodePayload::HeadquartersContact {
                            shop_pool_id: Some("headquarters_basic_supplies".to_string()),
                            candidate_count: 3,
                        },
                    ),
                    (
                        "supply_depot",
                        "shop_general",
                        MapNodePayload::Shop {
                            shop_id: Some("unknown_exploration_shop".to_string()),
                            shop_pool_id: None,
                        },
                    ),
                    (
                        "crossroads_archive",
                        "event_sealed_archive",
                        MapNodePayload::Event {
                            event_id: Some(crate::game::data::event_data::EventId::new(
                                "sealed_archive_recovery_01",
                            )),
                        },
                    ),
                    (
                        "containment_guard",
                        "combat_elite",
                        MapNodePayload::Encounter {
                            encounter_id: Some("suppress_porcubbus".to_string()),
                        },
                    ),
                    ("floor_gate", "gate_stairs", MapNodePayload::None),
                ],
            ),
            (
                "standard_final_a",
                crate::game::map::FacilityTerminalResolution::StandardCompletion,
                vec![
                    ("entry", "start", MapNodePayload::None),
                    (
                        "west_containment",
                        "combat_monster",
                        MapNodePayload::Encounter {
                            encounter_id: Some("normal_breach_response".to_string()),
                        },
                    ),
                    (
                        "east_containment",
                        "reward_treasure",
                        MapNodePayload::Reward {
                            reward_pool_id: Some("tutorial_f3_salvage".to_string()),
                        },
                    ),
                    (
                        "records_core",
                        "event_sealed_archive",
                        MapNodePayload::Event {
                            event_id: Some(crate::game::data::event_data::EventId::new(
                                "sealed_archive_recovery_01",
                            )),
                        },
                    ),
                    (
                        "maintenance_core",
                        "maintenance",
                        MapNodePayload::Maintenance,
                    ),
                    (
                        "supply_core",
                        "shop_general",
                        MapNodePayload::Shop {
                            shop_id: Some("unknown_exploration_shop".to_string()),
                            shop_pool_id: None,
                        },
                    ),
                    (
                        "lower_containment",
                        "combat_monster",
                        MapNodePayload::Encounter {
                            encounter_id: Some("normal_black_box_pressure".to_string()),
                        },
                    ),
                    (
                        "upper_containment",
                        "headquarters_contact",
                        MapNodePayload::HeadquartersContact {
                            shop_pool_id: Some("headquarters_basic_supplies".to_string()),
                            candidate_count: 3,
                        },
                    ),
                    (
                        "final_approach",
                        "reward_treasure",
                        MapNodePayload::Reward {
                            reward_pool_id: Some("tutorial_f3_budget".to_string()),
                        },
                    ),
                    (
                        "final_chamber",
                        "boss_abnormality",
                        MapNodePayload::Encounter {
                            encounter_id: Some("suppress_nothing_there".to_string()),
                        },
                    ),
                ],
            ),
        ];

        for (floor_index, (template_id, terminal_resolution, expected_nodes)) in
            expected.iter().enumerate()
        {
            let left = FacilityMapGenerator::generate(
                91,
                GameMode::Tutorial,
                floor_index as u32,
                &FacilityRunState::default(),
                &catalogs,
                &policy,
                150,
            )
            .unwrap();
            let right = FacilityMapGenerator::generate(
                9_901,
                GameMode::Tutorial,
                floor_index as u32,
                &FacilityRunState::default(),
                &catalogs,
                &policy,
                150,
            )
            .unwrap();
            assert_eq!(left.map.map_template_id.as_str(), *template_id);
            assert_eq!(right.map.map_template_id.as_str(), *template_id);
            assert_eq!(left.map.nodes.len(), expected_nodes.len());
            for (slot_id, kind_id, payload) in expected_nodes {
                let left_node = left
                    .map
                    .nodes
                    .iter()
                    .find(|node| node.slot_id.as_str() == *slot_id)
                    .unwrap();
                let right_node = right
                    .map
                    .nodes
                    .iter()
                    .find(|node| node.slot_id.as_str() == *slot_id)
                    .unwrap();
                assert_eq!(left_node.kind_id.as_str(), *kind_id);
                assert_eq!(&left_node.payload, payload);
                assert_eq!(left_node.kind_id, right_node.kind_id);
                assert_eq!(left_node.category, right_node.category);
                assert_eq!(left_node.payload, right_node.payload);
                assert_eq!(left_node.combat_difficulty, right_node.combat_difficulty);
                if let Some(difficulty) = left_node.combat_difficulty.as_ref() {
                    assert_eq!(difficulty.difficulty_profile_id, "tutorial_fixed_v1");
                    assert_eq!(difficulty.floor_power_anchor, 150);
                    assert_eq!(difficulty.target_power, 0);
                    assert_eq!(difficulty.revival, None);
                    assert_ne!(
                        difficulty.class,
                        MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::FinalBoss)
                    );
                }
            }
            assert_eq!(left.map.terminals.len(), 1);
            assert_eq!(left.map.terminals[0].resolution, *terminal_resolution);
        }

        let floor_two = FacilityMapGenerator::generate(
            91,
            GameMode::Tutorial,
            1,
            &FacilityRunState::default(),
            &catalogs,
            &policy,
            150,
        )
        .unwrap();
        assert_eq!(
            floor_two
                .map
                .nodes
                .iter()
                .find(|node| node.slot_id.as_str() == "containment_guard")
                .unwrap()
                .combat_difficulty
                .as_ref()
                .unwrap()
                .class,
            MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::Elite)
        );
        let floor_three = FacilityMapGenerator::generate(
            91,
            GameMode::Tutorial,
            2,
            &FacilityRunState::default(),
            &catalogs,
            &policy,
            150,
        )
        .unwrap();
        assert_eq!(
            floor_three
                .map
                .nodes
                .iter()
                .find(|node| node.slot_id.as_str() == "final_chamber")
                .unwrap()
                .combat_difficulty
                .as_ref()
                .unwrap()
                .class,
            MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::NormalBoss)
        );
    }

    #[test]
    fn authored_capstone_precommits_concealed_terminal_and_applies_entry_payload() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let request = FacilityCapstoneRequest {
            template_id: MapTemplateId::new("boss_omen_capstone"),
            entries: vec![FacilityCapstoneEntryInput {
                slot_id: MapSlotId::new("boss_chamber"),
                payload: MapNodePayload::Encounter {
                    encounter_id: Some("suppress_white_night".to_string()),
                },
            }],
        };

        let generated = FacilityMapGenerator::generate_capstone(91, 7, &request, &catalogs)
            .expect("live Boss Omen capstone should generate");

        assert_eq!(generated.map.map_template_id, request.template_id);
        assert_eq!(generated.entry_node_ids.len(), 1);
        let boss = generated.map.node(generated.entry_node_ids[0]).unwrap();
        assert_eq!(boss.slot_id.as_str(), "boss_chamber");
        assert!(matches!(
            &boss.payload,
            MapNodePayload::Encounter {
                encounter_id: Some(encounter_id)
            } if encounter_id == "suppress_white_night"
        ));
        let gate = generated
            .map
            .terminals
            .iter()
            .find(|terminal| {
                terminal.resolution
                    == crate::game::map::FacilityTerminalResolution::BossOmenExitGate
            })
            .expect("capstone exit Gate terminal");
        assert!(generated.map.node(gate.node_id).is_some_and(|node| {
            node.slot_id.as_str() == "exit_gate"
                && node.state == MapNodeState::Unavailable
                && node.visibility == MapNodeVisibility::Concealed
        }));
    }

    #[test]
    fn capstone_generation_rejects_missing_entry_input() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let error = FacilityMapGenerator::generate_capstone(
            91,
            7,
            &FacilityCapstoneRequest {
                template_id: MapTemplateId::new("boss_omen_capstone"),
                entries: vec![],
            },
            &catalogs,
        )
        .unwrap_err();

        assert!(error.contains("requires 1 entry inputs"));
    }

    #[test]
    fn manifestation_reveals_preassigned_bonus_only_after_all_prerequisites_complete() {
        let mut catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        std::sync::Arc::make_mut(&mut catalogs.facility_templates).layouts.clear(); // Explicit legacy pool input.
        let mut state = FacilityRunState::default();
        let mut puzzle = None;
        for floor_index in 3..8 {
            let generated = FacilityMapGenerator::generate(
                314,
                GameMode::Expedition,
                floor_index,
                &state,
                &catalogs,
                &RunPolicyData::builtin(),
                100 + u64::from(floor_index) * 25,
            )
            .unwrap();
            state = generated.facility_run_state;
            if generated.map.map_template_id.as_str() == "endless_puzzle_controls" {
                puzzle = Some(generated.map);
                break;
            }
        }
        let mut map = puzzle.expect("advanced Endless pool must select its puzzle before reuse");
        let template = catalogs
            .facility_templates
            .template(&map.map_template_id)
            .unwrap();
        let original_bonus = map
            .nodes
            .iter()
            .find(|node| node.slot_id.as_str() == "sealed_bonus")
            .unwrap()
            .clone();
        map.nodes
            .iter_mut()
            .find(|node| node.slot_id.as_str() == "control_west")
            .unwrap()
            .state = MapNodeState::Completed;

        assert!(map.manifest_satisfied_rules(template).unwrap().is_empty());
        assert_eq!(
            map.nodes
                .iter()
                .find(|node| node.slot_id.as_str() == "sealed_bonus")
                .unwrap()
                .visibility,
            MapNodeVisibility::Concealed
        );

        map.nodes
            .iter_mut()
            .find(|node| node.slot_id.as_str() == "control_east")
            .unwrap()
            .state = MapNodeState::Completed;
        assert_eq!(
            map.manifest_satisfied_rules(template).unwrap(),
            vec!["dual_control_bonus"]
        );
        let manifested_bonus = map
            .nodes
            .iter()
            .find(|node| node.slot_id.as_str() == "sealed_bonus")
            .unwrap();
        assert_eq!(manifested_bonus.visibility, MapNodeVisibility::Revealed);
        assert_eq!(manifested_bonus.kind_id, original_bonus.kind_id);
        assert_eq!(manifested_bonus.payload, original_bonus.payload);
        assert!(map.manifest_satisfied_rules(template).unwrap().is_empty());
    }
}
