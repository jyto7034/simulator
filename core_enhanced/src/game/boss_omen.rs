use std::collections::BTreeSet;

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
        MapNodePayload, MapProgression, MapSlotId, MapTemplateId, RunMap, RunProgression,
    },
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BossOmenRunState {
    pub provisional: Option<ProvisionalBossOmenState>,
    pub active: Option<ActiveBossOmenState>,
    pub placed_source: Option<PlacedBossOmenSourceState>,
    pub forced_boss: Option<ForcedBossOmenNodeState>,
    pub cycle_index: u64,
    pub remaining_chain_ids: Vec<BossOmenChainId>,
    pub completed_chain_ids: BTreeSet<BossOmenChainId>,
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
                self.defer_chain(&provisional.chain_id);
            }
            self.placed_source = None;
        }
    }

    pub fn handle_source_failure(&mut self, node_id: MapNodeId) -> bool {
        let Some(placed) = self.placed_source.take() else {
            return false;
        };
        if placed.node_id != node_id {
            self.placed_source = Some(placed);
            return false;
        }
        if self.active.is_none() {
            if let Some(provisional) = self.provisional.take() {
                self.defer_chain(&provisional.chain_id);
            }
        }
        true
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
        if let Some(chain_id) = self
            .forced_boss
            .as_ref()
            .filter(|forced| forced.node_id == node_id)
            .map(|forced| forced.chain_id.clone())
        {
            self.remaining_chain_ids.retain(|id| id != &chain_id);
            self.completed_chain_ids.insert(chain_id);
            self.provisional = None;
            self.active = None;
            self.placed_source = None;
            self.forced_boss = None;
        }
    }

    fn ensure_cycle(&mut self, game_data: &GameDataBase, run_seed: u64) {
        if !self.remaining_chain_ids.is_empty()
            || self.provisional.is_some()
            || self.active.is_some()
            || self.forced_boss.is_some()
        {
            return;
        }
        let cycle_seed =
            determinism::seed_with_namespace(run_seed ^ self.cycle_index, 0x424F_4D45_4E5F_4359);
        let mut members = game_data
            .boss_omen_data
            .chains
            .iter()
            .map(|chain| chain.id.clone())
            .collect::<Vec<_>>();
        members.sort_by_key(|chain_id| {
            determinism::seed_with_namespace(
                cycle_seed ^ stable_text_hash(chain_id.as_str()),
                0x424F_4D45_4E5F_4F52,
            )
        });
        self.remaining_chain_ids = members;
        self.cycle_index = self.cycle_index.saturating_add(1);
    }

    fn defer_chain(&mut self, chain_id: &BossOmenChainId) {
        let Some(index) = self
            .remaining_chain_ids
            .iter()
            .position(|candidate| candidate == chain_id)
        else {
            return;
        };
        let chain_id = self.remaining_chain_ids.remove(index);
        self.remaining_chain_ids.push(chain_id);
    }

    pub fn validate_checkpoint_state(
        &self,
        game_data: &GameDataBase,
        node_exists: impl Fn(MapNodeId) -> bool,
    ) -> Result<(), String> {
        let authored_ids = game_data
            .boss_omen_data
            .chains
            .iter()
            .map(|chain| chain.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let mut remaining = std::collections::HashSet::new();
        for chain_id in &self.remaining_chain_ids {
            if !authored_ids.contains(chain_id.as_str()) || !remaining.insert(chain_id.as_str()) {
                return Err(format!(
                    "BossOmen cycle contains unknown or duplicate chain '{}'",
                    chain_id.as_str()
                ));
            }
        }
        if self
            .completed_chain_ids
            .iter()
            .any(|chain_id| !authored_ids.contains(chain_id.as_str()))
        {
            return Err("BossOmen completed ledger references unknown chain".to_string());
        }
        if self.provisional.is_some() && self.active.is_some() {
            return Err("BossOmen cannot be provisional and active simultaneously".to_string());
        }
        let current_chain_id = self
            .active
            .as_ref()
            .map(|active| &active.chain_id)
            .or_else(|| {
                self.provisional
                    .as_ref()
                    .map(|provisional| &provisional.chain_id)
            });
        if current_chain_id.is_some_and(|chain_id| {
            !self.remaining_chain_ids.contains(chain_id)
                || !authored_ids.contains(chain_id.as_str())
        }) {
            return Err("BossOmen current chain is outside the current cycle".to_string());
        }
        if let Some(active) = &self.active {
            let chain = game_data
                .boss_omen_data
                .get_by_id(active.chain_id.as_str())
                .ok_or_else(|| "BossOmen active chain is missing".to_string())?;
            if active.boss_abnormality_id != chain.boss_abnormality_id
                || active.next_step_index != active.completed_steps.len()
                || active.next_step_index > chain.steps.len()
                || active
                    .completed_steps
                    .iter()
                    .zip(chain.steps.iter())
                    .any(|(actual, expected)| actual != &expected.id)
            {
                return Err("BossOmen active step ledger is inconsistent".to_string());
            }
        }
        if let Some(placed) = &self.placed_source {
            if !node_exists(placed.node_id)
                || current_chain_id != Some(&placed.chain_id)
                || !game_data
                    .boss_omen_data
                    .get_by_id(placed.chain_id.as_str())
                    .and_then(|chain| chain.steps.get(placed.step_index))
                    .is_some_and(|step| {
                        step.id == placed.step_id && step.source_kind == placed.source_kind
                    })
            {
                return Err("BossOmen placed source is inconsistent".to_string());
            }
        }
        if let Some(forced) = &self.forced_boss {
            let Some(active) = &self.active else {
                return Err("BossOmen forced boss has no active chain".to_string());
            };
            let chain = game_data
                .boss_omen_data
                .get_by_id(forced.chain_id.as_str())
                .ok_or_else(|| "BossOmen forced chain is missing".to_string())?;
            if !node_exists(forced.node_id)
                || forced.chain_id != active.chain_id
                || forced.boss_abnormality_id != chain.boss_abnormality_id
                || forced.encounter_id != chain.boss_encounter_id
                || active.completed_steps.len() < chain.required_steps as usize
            {
                return Err("BossOmen forced boss state is inconsistent".to_string());
            }
        }
        if self.cycle_index == 0
            && (!self.remaining_chain_ids.is_empty()
                || self.provisional.is_some()
                || self.active.is_some()
                || self.forced_boss.is_some())
        {
            return Err("BossOmen initialized state must have a positive cycle index".to_string());
        }
        Ok(())
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
    if run_progression.game_mode != GameMode::Expedition {
        return;
    }
    state.ensure_cycle(game_data, run_progression.run_seed);
    if state.active.is_some() {
        apply_active_chain_step(map, run_progression, game_data, state);
        return;
    }
    if state.provisional.is_none() {
        let Some(chain) =
            select_provisional_chain(run_progression, skill_fragments, game_data, state)
        else {
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

    if !progression.reveal_terminal_nodes(map, FacilityTerminalResolution::BossOmenExitGate)? {
        return Ok(false);
    }
    state.clear_after_forced_boss_victory(forced_boss_id);
    Ok(true)
}

fn select_provisional_chain<'a>(
    run_progression: &RunProgression,
    skill_fragments: &SkillFragmentInventory,
    game_data: &'a GameDataBase,
    state: &BossOmenRunState,
) -> Option<&'a BossOmenChain> {
    state
        .remaining_chain_ids
        .iter()
        .filter_map(|chain_id| game_data.boss_omen_data.get_by_id(chain_id.as_str()))
        .filter(|chain| run_progression.floor_index() >= chain.min_floor)
        .filter(|chain| {
            chain
                .dependencies
                .iter()
                .all(|dependency| state.completed_chain_ids.contains(dependency))
        })
        .filter(|chain| {
            chain.required_fragment.as_ref().is_none_or(|required| {
                skill_fragments.contains(&required.fragment_id)
                    && (!required.awakening_required
                        || skill_fragments.any_form_unlocked(&required.fragment_id, "awakened"))
            })
        })
        .next()
}

fn stable_text_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
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
        revival: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn node(value: u128) -> MapNodeId {
        MapNodeId::new(Uuid::from_u128(value))
    }

    fn placed(chain_id: &str, node_id: MapNodeId) -> PlacedBossOmenSourceState {
        PlacedBossOmenSourceState {
            node_id,
            chain_id: BossOmenChainId::new(chain_id),
            boss_abnormality_id: format!("abnormality_{chain_id}"),
            step_id: BossOmenStepId::new("step_1"),
            step_index: 0,
            source_kind: BossOmenSourceKind::Combat,
            hint_id: "hint".to_string(),
            title_id: "title".to_string(),
            description_id: "description".to_string(),
            event_id: None,
            encounter_id: Some("source_encounter".to_string()),
            confirmed: true,
        }
    }

    #[test]
    fn provisional_failure_requeues_but_active_failure_retains_same_step() {
        let mut provisional = BossOmenRunState {
            remaining_chain_ids: vec![BossOmenChainId::new("a"), BossOmenChainId::new("b")],
            provisional: Some(ProvisionalBossOmenState {
                chain_id: BossOmenChainId::new("a"),
                boss_abnormality_id: "abnormality_a".to_string(),
                step_index: 0,
            }),
            placed_source: Some(placed("a", node(1))),
            ..Default::default()
        };
        assert!(provisional.handle_source_failure(node(1)));
        assert!(provisional.provisional.is_none());
        assert_eq!(
            provisional
                .remaining_chain_ids
                .iter()
                .map(BossOmenChainId::as_str)
                .collect::<Vec<_>>(),
            vec!["b", "a"]
        );

        let active_state = ActiveBossOmenState {
            chain_id: BossOmenChainId::new("a"),
            boss_abnormality_id: "abnormality_a".to_string(),
            next_step_index: 1,
            completed_steps: vec![BossOmenStepId::new("step_1")],
        };
        let mut active = BossOmenRunState {
            remaining_chain_ids: vec![BossOmenChainId::new("a"), BossOmenChainId::new("b")],
            active: Some(active_state.clone()),
            placed_source: Some(PlacedBossOmenSourceState {
                step_index: 1,
                ..placed("a", node(2))
            }),
            ..Default::default()
        };
        assert!(active.handle_source_failure(node(2)));
        assert_eq!(active.active, Some(active_state));
        assert!(active.placed_source.is_none());
        assert_eq!(
            active
                .remaining_chain_ids
                .iter()
                .map(BossOmenChainId::as_str)
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[test]
    fn forced_boss_victory_consumes_cycle_member_and_records_run_completion() {
        let mut state = BossOmenRunState {
            remaining_chain_ids: vec![BossOmenChainId::new("a")],
            active: Some(ActiveBossOmenState {
                chain_id: BossOmenChainId::new("a"),
                boss_abnormality_id: "abnormality_a".to_string(),
                next_step_index: 1,
                completed_steps: vec![BossOmenStepId::new("step_1")],
            }),
            forced_boss: Some(ForcedBossOmenNodeState {
                node_id: node(1),
                chain_id: BossOmenChainId::new("a"),
                boss_abnormality_id: "abnormality_a".to_string(),
                encounter_id: "boss_a".to_string(),
            }),
            ..Default::default()
        };

        state.clear_after_forced_boss_victory(node(1));

        assert!(state.remaining_chain_ids.is_empty());
        assert!(state
            .completed_chain_ids
            .contains(&BossOmenChainId::new("a")));
        assert!(state.active.is_none());
        assert!(state.forced_boss.is_none());
    }
}

crate::authoring::clone_shape::struct_shape!(BossOmenRunState {
    provisional,
    active,
    placed_source,
    forced_boss,
    cycle_index,
    remaining_chain_ids,
    completed_chain_ids
});
