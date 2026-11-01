use super::GameCore;
use crate::game::{
    behavior::{BehaviorResult, GameError},
    data::{pve_data::PveEncounterClass, run_policy_data::CombatNodeDifficultyRole},
    map::{
        GameMode, MapNodeCategory, MapNodeCombatDifficultyClass, MapNodeExecutor, MapNodeId,
        MapNodePayload, MapNodeState,
    },
    material_economy::MaterialPayment,
    resources::GameState,
    revival::{
        HatchOption, HatchedEncounterView, RevivalNodeTuning, RevivalRunSnapshot, RevivalStage,
    },
};

impl GameCore {
    pub(super) fn hatchable_node(&self) -> Option<MapNodeId> {
        let GameState::NodeConfirm { node_id, .. } = self.state.lifecycle.phase() else {
            return None;
        };
        let run = self.state.run.as_ref()?;
        if run.run_progression.game_mode != GameMode::Expedition
            || run.run_progression.pending_continuation.is_some()
            || run.revival.nodes.contains_key(node_id)
        {
            return None;
        }
        let node = run.map.node(*node_id)?;
        if node.state != MapNodeState::Available
            || node.category != MapNodeCategory::Combat
            || node.omen.is_some()
            || node.kind_id.as_str().contains("elite")
        {
            return None;
        }
        if let Some(rooms) = &run.facility_run_state.rooms {
            let layout = self.facility_layout()?;
            let room = layout.room(&rooms.current_room_id)?;
            let metadata = rooms.activity_metadata.get(node_id)?;
            if room.reservations.iter().any(|r| r.slot_id == node.slot_id)
                || metadata.omen_compatible
                || metadata.activity_id != layout.allocation.normal_activity_id
            {
                return None;
            }
        }
        let MapNodePayload::Encounter {
            encounter_id: Some(id),
        } = &node.payload
        else {
            return None;
        };
        if self.game_data.pve_data.get_by_id(id)?.encounter_class != PveEncounterClass::Normal {
            return None;
        }
        // The existing selectable/session authority enforces navigation. Whether
        // there is an alternate route to the Gate is deliberately irrelevant.
        self.validate_select_map_node(*node_id).ok()?;
        Some(*node_id)
    }

    pub(super) fn revival_snapshot(&self) -> Option<RevivalRunSnapshot> {
        let run = self.state.run.as_ref()?;
        if run.run_progression.game_mode != GameMode::Expedition {
            return None;
        }
        let selected_node_id = self.hatchable_node();
        let hatch_options = run
            .revival
            .eggs
            .iter()
            .filter_map(|(id, stage)| {
                let source = self
                    .game_data
                    .pve_data
                    .reward_sources
                    .iter()
                    .find(|s| &s.reward_source_id == id)?;
                let policy = source.revival.as_ref()?;
                let definition = policy.stage(*stage);
                let abnormality_id = self
                    .game_data
                    .pve_data
                    .get_by_id(&definition.encounter_id)?
                    .primary_abnormality_id()?;
                let metadata = self.game_data.abnormality_data.get_by_id(abnormality_id)?;
                let carried = self
                    .state
                    .inventory
                    .equipment_materials
                    .amount(&policy.catalyst_material_id);
                let warehouse = self
                    .state
                    .player_profile
                    .lifecycle
                    .vault
                    .materials
                    .get(&policy.catalyst_material_id)
                    .copied()
                    .unwrap_or(0);
                Some(HatchOption {
                    reward_source_id: id.clone(),
                    abnormality_id: abnormality_id.to_owned(),
                    display_name: metadata.name.clone(),
                    stage: *stage,
                    encounter_id: definition.encounter_id.clone(),
                    stat_multiplier_percent: definition.stat_multiplier_percent,
                    catalyst_material_id: policy.catalyst_material_id.clone(),
                    catalyst_required: policy.catalyst_amount,
                    catalyst_carried: carried,
                    catalyst_warehouse: warehouse,
                    rewards: policy.victory_rewards(*stage),
                    equipment_choices: definition.equipment_choices.clone(),
                    next_egg: stage.next(),
                    can_hatch: selected_node_id.is_some()
                        && u64::from(carried) + u64::from(warehouse)
                            >= u64::from(policy.catalyst_amount),
                })
            })
            .collect();
        let hatched_nodes = run
            .revival
            .nodes
            .iter()
            .filter_map(|(node_id, encounter)| {
                let source = self
                    .game_data
                    .pve_data
                    .reward_sources
                    .iter()
                    .find(|s| s.reward_source_id == encounter.reward_source_id)?;
                let policy = source.revival.as_ref()?;
                let stage = policy.stage(encounter.stage);
                Some((
                    *node_id,
                    HatchedEncounterView {
                        encounter: encounter.clone(),
                        encounter_id: stage.encounter_id.clone(),
                        stat_multiplier_percent: stage.stat_multiplier_percent,
                        rewards: policy.victory_rewards(encounter.stage),
                        equipment_choices: stage.equipment_choices.clone(),
                        next_egg_on_victory: encounter.stage.next(),
                    },
                ))
            })
            .collect();
        Some(RevivalRunSnapshot {
            eggs: run.revival.eggs.clone(),
            hatched_nodes,
            selected_node_id,
            hatch_options,
        })
    }

    pub(super) fn handle_hatch_encounter(
        &mut self,
        node_id: MapNodeId,
        source_id: &str,
        stage: RevivalStage,
        payment: &[MaterialPayment],
    ) -> Result<BehaviorResult, GameError> {
        if self.hatchable_node() != Some(node_id) {
            return Err(GameError::InvalidAction);
        }
        let source = self
            .game_data
            .pve_data
            .reward_sources
            .iter()
            .find(|s| s.reward_source_id == source_id)
            .ok_or(GameError::InvalidAction)?;
        let policy = crate::authoring::clone_shape::controlled_clone(
            source.revival.as_ref().ok_or(GameError::InvalidAction)?,
        )?;
        let definition = policy.stage(stage);
        let mut progression = crate::authoring::clone_shape::controlled_clone(
            &self.run_state()?.encounter_progression,
        )?;
        let mut economy = super::grant_state::EconomyGrantState::capture(&self.state)?;
        let mut warehouse = crate::authoring::clone_shape::controlled_clone(
            &self.state.player_profile.lifecycle.vault.materials,
        )?;
        let mut debits = crate::authoring::clone_shape::controlled_clone(
            &self.state.player_profile.lifecycle.warehouse_debits,
        )?;
        let payment = crate::game::material_economy::pay_materials(
            &mut economy.inventory,
            &mut economy.crafting_resources,
            &mut warehouse,
            &mut debits,
            &[(policy.catalyst_material_id.clone(), policy.catalyst_amount)].into(),
            Some(payment),
            true,
            &self.game_data.equipment_data,
        )?;
        progression
            .revival
            .hatch(node_id, source_id, stage, payment)?;
        let mut map = crate::authoring::clone_shape::controlled_clone(&self.run_state()?.map)?;
        let node = map.node_mut(node_id).ok_or(GameError::InvalidAction)?;
        node.payload = MapNodePayload::Encounter {
            encounter_id: Some(definition.encounter_id.clone()),
        };
        let difficulty = node
            .combat_difficulty
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        difficulty.class = MapNodeCombatDifficultyClass::Role(CombatNodeDifficultyRole::Revival);
        difficulty.revival = Some(RevivalNodeTuning {
            stage,
            stat_multiplier_percent: definition.stat_multiplier_percent,
        });
        let tuning = self
            .run_policy()
            .resolved_tuning_for_manifested_node(difficulty)
            .ok_or(GameError::InvalidAction)?;
        difficulty.target_power = if self
            .run_policy()
            .combat_difficulty_profile(&difficulty.difficulty_profile_id)
            .is_some_and(|p| p.authored_fixed)
        {
            0
        } else {
            u64::try_from(
                (u128::from(difficulty.floor_power_anchor)
                    * u128::from(tuning.target_power_percent))
                .div_ceil(100),
            )
            .map_err(|_| GameError::InvalidAction)?
        };
        let entered = MapNodeExecutor::enter(node);
        let run_debits = crate::authoring::clone_shape::controlled_clone(&debits)?;
        economy.commit_into(&mut self.state);
        self.state.player_profile.lifecycle.vault.materials = warehouse;
        self.state.player_profile.lifecycle.warehouse_debits = debits;
        let run = self.run_state_mut()?;
        run.encounter_progression = progression;
        run.run_progression.warehouse_debits = run_debits;
        run.map = map;
        run.combat_previews.remove(&node_id);
        // Attempts and first-entry payments belong to the node, not its occupant.
        self.state.node_session = Some(entered.session.clone());
        let combat_preview = self.combat_preview_for_node(node_id)?;
        Ok(BehaviorResult::NodePreview {
            node_id,
            kind_id: entered.kind_id,
            category: entered.category,
            payload: entered.session.payload.clone(),
            session: entered.session,
            map: self.current_map_view()?,
            combat_preview,
            trauma_recovery_preview: None,
        })
    }
}
