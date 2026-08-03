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
                    assign_combat_difficulties(
                        &mut map,
                        &assignments,
                        difficulty_profile,
                        run_policy.combat_difficulty.power_model_version,
                        floor_power_anchor,
                        assignment_seed,
                    )?;
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
                    },
                    FacilitySlotRole::Entry
                    | FacilitySlotRole::Ordinary
                    | FacilitySlotRole::ManifestedBonus
                    | FacilitySlotRole::Terminal(_) => SlotAssignment {
                        category: slot.allowed_categories[0],
                        fixed_kind_id: None,
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
        GameMode::Standard,
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
    let mut assignments = HashMap::new();
    for slot in &template.slots {
        let fixed = match slot.role {
            FacilitySlotRole::Start => Some(SlotAssignment {
                category: MapNodeCategory::Start,
                fixed_kind_id: Some(MapNodeKindId::new("start")),
            }),
            FacilitySlotRole::Entry | FacilitySlotRole::Terminal(_) => Some(SlotAssignment {
                category: slot.allowed_categories[0],
                fixed_kind_id: None,
            }),
            FacilitySlotRole::Ordinary | FacilitySlotRole::ManifestedBonus
                if requirements.guarantee_reachable_shop
                    && slot.tags.iter().any(|tag| tag == "shop") =>
            {
                Some(SlotAssignment {
                    category: MapNodeCategory::Shop,
                    fixed_kind_id: None,
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
        let (kind_id, payload) = definition.map_or_else(
            || (MapNodeKindId::new("start"), MapNodePayload::None),
            |definition| (definition.kind_id.clone(), definition.payload.clone()),
        );
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
                crate::game::map::FacilityTerminalResolution::SpecialRunCompletion => {
                    Some(CombatNodeDifficultyRole::MonolithRemanifestation)
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
                monolith_stage: None,
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
            monolith_stage: None,
            floor_power_anchor,
            target_power: percent_of_anchor_ceil(
                floor_power_anchor,
                tuning.tuning.target_power_percent,
            ),
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
                GameMode::Standard,
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
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let first = FacilityMapGenerator::generate(
            77,
            GameMode::Endless,
            0,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            100,
        )
        .unwrap();
        let second = FacilityMapGenerator::generate(
            77,
            GameMode::Endless,
            1,
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
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let generated = FacilityMapGenerator::generate_with_requirements(
            77,
            GameMode::Endless,
            3,
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
            GameMode::Standard,
            2,
            &FacilityRunState::default(),
            &catalogs,
            &RunPolicyData::builtin(),
            150,
        )
        .unwrap();
        let right = FacilityMapGenerator::generate(
            91,
            GameMode::Standard,
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
    fn ordinary_quota_and_final_boss_role_are_manifested_once_with_fixed_targets() {
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let policy = RunPolicyData::builtin();
        let generated = FacilityMapGenerator::generate(
            91,
            GameMode::Standard,
            2,
            &FacilityRunState::default(),
            &catalogs,
            &policy,
            150,
        )
        .unwrap();
        let ordinary = generated
            .map
            .nodes
            .iter()
            .filter_map(|node| match node.combat_difficulty.as_ref()?.class {
                MapNodeCombatDifficultyClass::Ordinary(band) => Some(band),
                MapNodeCombatDifficultyClass::Role(_) => None,
            })
            .collect::<Vec<_>>();
        for band in [
            crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Easy,
            crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Standard,
            crate::game::data::run_policy_data::OrdinaryCombatDifficultyBand::Hard,
        ] {
            let quota = policy.combat_difficulty.profiles[0]
                .ordinary_bands
                .iter()
                .find(|entry| entry.band == band)
                .unwrap()
                .quota_percent as usize;
            let count = ordinary.iter().filter(|actual| **actual == band).count();
            let product = ordinary.len() * quota;
            assert!(count == product / 100 || count == product.div_ceil(100));
        }
        assert!(ordinary.len() >= 2);
        let final_boss = generated
            .map
            .nodes
            .iter()
            .find_map(|node| {
                let difficulty = node.combat_difficulty.as_ref()?;
                (difficulty.class
                    == MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::FinalBoss))
                .then_some(difficulty)
            })
            .expect("Standard final floor must manifest the final boss role");
        assert_eq!(final_boss.floor_power_anchor, 150);
        assert_eq!(final_boss.target_power, 300);
        assert_eq!(final_boss.power_model_version, 2);
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
        let catalogs = crate::game::data::live_ron::embedded_map_catalogs_for_fixtures();
        let mut state = FacilityRunState::default();
        let mut puzzle = None;
        for floor_index in 3..8 {
            let generated = FacilityMapGenerator::generate(
                314,
                GameMode::Endless,
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
