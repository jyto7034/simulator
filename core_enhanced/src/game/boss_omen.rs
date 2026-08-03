use serde::{Deserialize, Serialize};
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
        FacilityCapstoneEntryInput, FacilityCapstoneRequest, FacilityMapGenerator,
        FacilityTerminalResolution, GameMode, MapNodeCategory, MapNodeId, MapNodeOmenOverlayDto,
        MapNodePayload, MapNodeState, MapNodeVisibility, MapProgression, MapSlotId, MapTemplateId,
        RunMap, RunProgression,
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
    let floor_difficulty_authority = map
        .nodes
        .iter()
        .find_map(|node| node.combat_difficulty.clone())
        .expect("manifested floor must carry combat difficulty authority");
    let mut generated = FacilityMapGenerator::generate_capstone(
        run_progression.run_seed,
        run_progression.floor_index(),
        &FacilityCapstoneRequest {
            template_id: MapTemplateId::new("boss_omen_capstone"),
            entries: vec![FacilityCapstoneEntryInput {
                slot_id: MapSlotId::new("boss_chamber"),
                payload: MapNodePayload::Encounter {
                    encounter_id: Some(chain.boss_encounter_id.clone()),
                },
            }],
        },
        &game_data.map_catalogs,
    )
    .unwrap_or_else(|error| panic!("validated Boss Omen capstone must generate: {error}"));
    let [boss_node_id] = generated.entry_node_ids.as_slice() else {
        panic!("Boss Omen capstone must expose exactly one Entry node");
    };
    let boss_node_id = *boss_node_id;
    assign_role_difficulty(
        generated
            .map
            .node_mut(boss_node_id)
            .expect("Boss Omen capstone entry must exist"),
        crate::game::data::run_policy_data::CombatNodeDifficultyRole::BossOmen,
        &floor_difficulty_authority,
        game_data,
    );
    *map = generated.map;
    *progression = MapProgression::from_map(map);
    state.forced_boss = Some(ForcedBossOmenNodeState {
        node_id: boss_node_id,
        chain_id: chain.id.clone(),
        boss_abnormality_id: chain.boss_abnormality_id.clone(),
        encounter_id: chain.boss_encounter_id.clone(),
    });
}

pub fn complete_forced_boss_with_exit_gate(
    map: &mut RunMap,
    progression: &mut MapProgression,
    template: &crate::game::map::FacilityTemplate,
    state: &mut BossOmenRunState,
) -> Result<bool, crate::game::map::MapProgressionError> {
    let Some(forced_boss_id) = state.forced_boss_node_id() else {
        return Ok(false);
    };
    if progression.current_node_id != Some(forced_boss_id) {
        return Ok(false);
    }
    let forced_boss = map
        .node(forced_boss_id)
        .ok_or(crate::game::map::MapProgressionError::MissingNode)?
        .category;
    if forced_boss != MapNodeCategory::Boss {
        return Ok(false);
    }

    let reached_terminal = progression.complete_current_node(map, template)?;
    if reached_terminal.is_some() {
        return Ok(false);
    }

    let gate_node_ids = map
        .terminals
        .iter()
        .filter(|terminal| terminal.resolution == FacilityTerminalResolution::BossOmenExitGate)
        .map(|terminal| terminal.node_id)
        .collect::<Vec<_>>();
    if gate_node_ids.is_empty() {
        return Ok(false);
    }
    for gate_node_id in &gate_node_ids {
        let gate = map
            .node_mut(*gate_node_id)
            .ok_or(crate::game::map::MapProgressionError::MissingNode)?;
        gate.state = MapNodeState::Available;
        gate.visibility = MapNodeVisibility::Revealed;
    }
    progression.available_node_ids = gate_node_ids;
    progression
        .available_node_ids
        .sort_by_key(|node_id| node_id.0.as_u128());
    state.clear_after_forced_boss_victory(forced_boss_id);
    Ok(true)
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
    place_step(
        map,
        run_progression,
        chain,
        active.next_step_index,
        game_data,
        state,
    );
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
    place_step(
        map,
        run_progression,
        chain,
        provisional.step_index,
        game_data,
        state,
    );
}

fn place_step(
    map: &mut RunMap,
    run_progression: &RunProgression,
    chain: &BossOmenChain,
    step_index: usize,
    game_data: &GameDataBase,
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
            let authority = node
                .combat_difficulty
                .clone()
                .expect("Boss Omen combat source must start from a manifested combat node");
            assign_role_difficulty(
                node,
                crate::game::data::run_policy_data::CombatNodeDifficultyRole::BossOmen,
                &authority,
                game_data,
            );
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

fn assign_role_difficulty(
    node: &mut crate::game::map::MapNode,
    role: crate::game::data::run_policy_data::CombatNodeDifficultyRole,
    authority: &crate::game::map::MapNodeCombatDifficulty,
    game_data: &GameDataBase,
) {
    let profile = game_data
        .run_policy
        .combat_difficulty_profile(&authority.difficulty_profile_id)
        .expect("manifested combat difficulty profile must remain live");
    let tuning = profile
        .role_tuning
        .iter()
        .find(|entry| entry.role == role)
        .expect("validated difficulty profile must define Boss Omen tuning");
    let target_power = u64::try_from(
        u128::from(authority.floor_power_anchor)
            .saturating_mul(u128::from(tuning.tuning.target_power_percent))
            .div_ceil(100),
    )
    .unwrap_or(u64::MAX);
    node.combat_difficulty = Some(crate::game::map::MapNodeCombatDifficulty {
        power_model_version: authority.power_model_version,
        difficulty_profile_id: authority.difficulty_profile_id.clone(),
        class: crate::game::map::MapNodeCombatDifficultyClass::Role(role),
        monolith_stage: None,
        floor_power_anchor: authority.floor_power_anchor,
        target_power,
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
        determinism::seed_with_uuid(
            run_progression.current_floor_seed(),
            0x424F_4D45_4E32,
            node.id.0,
        )
    });
    candidates.first().map(|node| node.id)
}
