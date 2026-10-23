use super::facility::StagedRecoveryEffect;
use super::state::{RunCheckpointPayload, RUN_CHECKPOINT_MAX_LOADS};
use super::{map_encounters, GameCore, RunState};
use crate::game::behavior::{
    BehaviorResult, GameError, MonolithConversionSnapshotDto,
    RevivalSnapshotDto,
};
use crate::game::boss_omen::{
    apply_boss_omen_to_map, complete_forced_boss_with_exit_gate, force_boss_node_if_ready,
    BossOmenRunState,
};
use crate::game::combat_setup::player_spawns::skill_fragment_compatibility_profile_for_employee_with_item_slot;
use crate::game::encounter_progression::EncounterProgressionState;
use crate::game::map::{
    FacilityGenerationRequirements, FacilityMapGenerator, FacilityRunState,
    FacilityTerminalResolution, MapNodeCategory, MapNodeCombatDifficultyClass, MapNodeExecutor,
    MapNodeId, MapNodePayload, MapNodeState, MapProgression, MapViewDto,
    RevivalStage, RunMap, RunProgression,
};
use crate::game::resources::{ActiveNodeContent, GameState};

fn percent_of_anchor_ceil(anchor: u64, percent: u32) -> u64 {
    let value = u128::from(anchor)
        .saturating_mul(u128::from(percent))
        .div_ceil(100);
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn validate_revival_difficulty_authority(
    node_id: MapNodeId,
    difficulty: Option<&crate::game::map::MapNodeCombatDifficulty>,
    revival: &crate::game::revival::RevivalRunState,
    data: &crate::game::data::pve_data::PveEncounterDatabase,
) -> Result<(), GameError> {
    let ledger = revival.nodes.get(&node_id);
    let role = difficulty.is_some_and(|d| matches!(d.class, MapNodeCombatDifficultyClass::Role(crate::game::data::run_policy_data::CombatNodeDifficultyRole::Revival)));
    match (role, difficulty.and_then(|d| d.revival), ledger) {
        (false, None, None) => Ok(()),
        (true, Some(tuning), Some(ledger)) => {
            let authored = data.reward_sources.iter().find(|s| s.reward_source_id == ledger.reward_source_id)
                .and_then(|s| s.revival.as_ref()).map(|p| p.stage(ledger.stage));
            if tuning.stage == ledger.stage && authored.is_some_and(|p| p.stat_multiplier_percent == tuning.stat_multiplier_percent) { Ok(()) }
            else { Err(GameError::InvalidStaticData("revival node tuning differs from its source".into())) }
        }
        _ => Err(GameError::InvalidStaticData(format!("revival difficulty for node '{}' does not match its ledger", node_id.0))),
    }
}

pub(super) enum StagedNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        boss_omen: Option<BossOmenRunState>,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        encounter_progression: EncounterProgressionState,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
    RunComplete {
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        recovery_effect: StagedRecoveryEffect,
        view: MapViewDto,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeCompletionContext {
    CurrentNode,
    ResolvedHeadquartersContact,
}

#[derive(Clone)]
pub(super) enum CombatResultNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        boss_omen: Option<BossOmenRunState>,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        encounter_progression: EncounterProgressionState,
        view: MapViewDto,
    },
    RunComplete {
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
    },
}

impl TryFrom<StagedNodeCompletion> for CombatResultNodeCompletion {
    type Error = GameError;

    fn try_from(completion: StagedNodeCompletion) -> Result<Self, Self::Error> {
        match completion {
            StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::NodeCompleted {
                    map,
                    progression,
                    completing_node_id,
                    boss_omen,
                    view,
                })
            }
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::FloorAdvanced {
                    next_map,
                    next_progression,
                    run_progression,
                    facility_run_state,
                    encounter_progression,
                    view,
                })
            }
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            } => {
                if recovery_effect != StagedRecoveryEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::RunComplete {
                    map,
                    progression,
                    run_progression,
                    view,
                })
            }
        }
    }
}

impl GameCore {
    pub(super) fn facility_template_for_map(
        &self,
        map: &RunMap,
    ) -> Result<crate::game::map::FacilityTemplate, GameError> {
        self.facility_template_for_map_ref(map).cloned()
    }

    /// Borrow the publication's template after the same lookup/version checks
    /// used by callers that need an owned copy across state transitions.
    pub(super) fn facility_template_for_map_ref(
        &self,
        map: &RunMap,
    ) -> Result<&crate::game::map::FacilityTemplate, GameError> {
        let template = self
            .game_data
            .map_catalogs
            .facility_templates
            .template(&map.map_template_id)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "run map references missing facility template '{}'",
                    map.map_template_id.as_str()
                ))
            })?;
        if template.version != map.facility_template_version {
            return Err(GameError::InvalidStaticData(format!(
                "run map template '{}@{}' does not match live template version {}",
                map.map_template_id.as_str(),
                map.facility_template_version,
                template.version
            )));
        }
        Ok(template)
    }

    pub(super) fn current_map_view(&self) -> Result<MapViewDto, GameError> {
        let run = self.run_state()?;
        let template = self.facility_template_for_map_ref(&run.map)?;
        let mut view = run.map_progression.view(&run.map, template);
        self.conceal_current_facility_encounters(&mut view);
        let certified = crate::game::combat_power::certified_combat_power_for_roster(
            &self.state.roster,
            &self.state.inventory,
            &self.state.skill_fragments,
            self.game_data.as_ref(),
        )?;
        let threat_policy = self.run_policy().combat_difficulty.threat_bands;
        let mut readiness_warnings = Vec::new();
        if !self
            .state
            .roster
            .iter()
            .any(|employee| employee.is_battle_deployable())
        {
            readiness_warnings
                .push(crate::game::map::CombatReadinessWarningDto::NoDeployableEmployees);
        }
        if self
            .state
            .roster
            .iter()
            .any(|employee| employee.condition.is_breakdown())
        {
            readiness_warnings.push(crate::game::map::CombatReadinessWarningDto::TraumaBreakdown);
        }
        if self.state.roster.iter().any(|employee| {
            employee.condition.physical_state()
                == crate::game::employee::PhysicalDamageState::Critical
        }) {
            readiness_warnings
                .push(crate::game::map::CombatReadinessWarningDto::CriticalPhysicalDamage);
        }
        for node in &mut view.nodes {
            if let Some(source) = run.map.node(node.id) {
                node.entry_preview = Some(self.node_entry_preview(source)?);
            }
            let Some(target_power) = run
                .map
                .node(node.id)
                .and_then(|source| source.combat_difficulty.as_ref())
                .map(|difficulty| difficulty.target_power)
            else {
                continue;
            };
            let ratio = if certified.total == 0 {
                u64::MAX
            } else {
                u64::try_from(
                    u128::from(target_power)
                        .saturating_mul(100)
                        .div_ceil(u128::from(certified.total)),
                )
                .unwrap_or(u64::MAX)
            };
            node.relative_threat =
                Some(if ratio <= u64::from(threat_policy.low_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::Low
                } else if ratio <= u64::from(threat_policy.equal_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::Equal
                } else if ratio <= u64::from(threat_policy.high_max_ratio_percent) {
                    crate::game::map::RelativeThreatBandDto::High
                } else {
                    crate::game::map::RelativeThreatBandDto::VeryHigh
                });
            node.readiness_warnings = readiness_warnings.clone();
        }
        if let Some(restriction) = run.combat_recovery_restriction.as_ref() {
            for node in &mut view.nodes {
                if node.state == crate::game::map::MapNodeState::Available
                    && !restriction.allowed_node_ids.contains(&node.id)
                {
                    node.state = crate::game::map::MapNodeState::Unavailable;
                }
            }
        }
        Ok(view)
    }

    pub(super) fn generate_current_floor_map(
        &mut self,
        run_progression: &RunProgression,
        facility_run_state: &FacilityRunState,
        encounter_progression: &mut EncounterProgressionState,
    ) -> (RunMap, MapProgression, FacilityRunState) {
        let power_model_version = self.run_policy().combat_difficulty.power_model_version;
        if self.state.run_power_authority.power_model_version == 0 {
            let certified = crate::game::combat_power::certified_combat_power_for_roster(
                &self.state.roster,
                &self.state.inventory,
                &self.state.skill_fragments,
                self.game_data.as_ref(),
            )
            .expect("validated persistent loadout state must certify");
            self.state
                .run_power_authority
                .initialize_run(power_model_version, certified.total);
        }
        let anchor_policy = self
            .run_policy()
            .floor_power_anchor_policy(run_progression.game_mode, run_progression.floor_index())
            .expect("validated run policy must resolve current floor power anchor");
        self.state.run_power_authority.manifest_floor(anchor_policy);
        let generated = FacilityMapGenerator::generate_with_requirements(
            run_progression.run_seed,
            run_progression.game_mode,
            run_progression.floor_index(),
            facility_run_state,
            &self.game_data.map_catalogs,
            self.run_policy(),
            self.state.run_power_authority.current_floor_power_anchor,
            FacilityGenerationRequirements {
                guarantee_reachable_shop: false,
            },
        )
        .expect("validated live facility data must generate the current Floor");
        let mut run_map = generated.map;
        let excluded = generated.facility_run_state.rooms.as_ref().map(|state| {
            let layout = self.game_data.map_catalogs.facility_templates.layouts.iter().find(|l|l.id == state.layout_id).expect("validated layout");
            state.activity_metadata.iter().filter(|(_,m)|layout.activity(&m.activity_id).is_some_and(|a| !a.support.is_supported())).map(|(id,_)|*id).collect()
        }).unwrap_or_default();
        map_encounters::assign_map_encounters_excluding(
            &self.game_data.pve_data,
            &mut run_map,
            run_progression,
            encounter_progression,
            self.run_policy(),
            &excluded,
        );
        let mut progression = MapProgression::from_map(&run_map);
        if generated.facility_run_state.rooms.is_some() {
            return (run_map,progression,generated.facility_run_state);
        }
        force_boss_node_if_ready(
            &mut run_map,
            &mut progression,
            run_progression,
            self.game_data.as_ref(),
            &mut encounter_progression.boss_omen,
        );
        if encounter_progression
            .boss_omen
            .forced_boss_node_id()
            .is_none()
        {
            apply_boss_omen_to_map(
                &mut run_map,
                run_progression,
                &self.state.skill_fragments,
                self.game_data.as_ref(),
                &mut encounter_progression.boss_omen,
            );
            progression = MapProgression::from_map(&run_map);
        }
        (run_map, progression, generated.facility_run_state)
    }

    pub(super) fn handle_request_map_data(&self) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::MapState {
            map: self.current_map_view()?,
        })
    }

    pub(super) fn handle_select_map_node(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<BehaviorResult, GameError> {
        self.validate_select_map_node(node_id)?;
        self.validate_facility_node_entry(node_id)?;
        let run = self.run_state()?;
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let mut enter_result = MapNodeExecutor::enter(node);
        let combat_preview = self.combat_preview_for_node(node_id)?;
        let trauma_recovery_preview = if enter_result.category == MapNodeCategory::Gate && self.facility_layout().is_some() { None } else {
            self.trauma_recovery_preview_for_category(enter_result.category)?
        };
        self.state.node_session = Some(enter_result.session.clone());
        self.transition_to(GameState::NodeConfirm {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: enter_result.category,
        })?;

        enter_result.session = self.facility_session_view(enter_result.session);
        enter_result.payload = enter_result.session.payload.clone();
        Ok(BehaviorResult::NodePreview {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
            map: self.current_map_view()?,
            combat_preview,
            trauma_recovery_preview,
        })
    }

    pub(super) fn handle_cancel_selected_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.transition_to(GameState::ViewingMap)?;
        self.handle_request_map_data()
    }

    pub(super) fn monolith_conversion_snapshot(&self) -> Option<MonolithConversionSnapshotDto> { None }
    pub(super) fn monolith_remanifestation_snapshot(&self) -> Option<RevivalSnapshotDto> { None }
    pub(super) fn handle_convert_map_node_with_monolith(&mut self, _node_id: MapNodeId, _abnormality_id: &str) -> Result<BehaviorResult, GameError> {
        Err(Self::retired_encounter_action(crate::game::behavior::ActionKind::ConvertMapNodeWithMonolith))
    }

    pub(super) fn handle_confirm_enter_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_confirm_enter_node_inner)
    }

    fn handle_confirm_enter_node_inner(&mut self) -> Result<BehaviorResult, GameError> {
        self.handle_confirm_enter_node_resuming(false)
    }

    pub(super) fn handle_confirm_enter_node_resuming(&mut self, resumed: bool) -> Result<BehaviorResult, GameError> {
        let node_id = match self.get_state() {
            GameState::NodeConfirm { node_id, .. } => node_id,
            _ => return Err(GameError::InvalidAction),
        };
        self.validate_facility_node_entry(node_id)?;
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        if matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        ) {
            if let Some(result) = self.block_or_fail_undeployable_combat_selection()? {
                return Ok(result);
            }
        }
        // Preserve the pre-entry continuation before cost/death or exhaustion choice.
        self.capture_expedition_entry_for_continue(node_id)?;
        let living_before_entry = self.state.roster.iter().filter(|employee| employee.is_alive()).count();
        let pending_choice = !resumed && self.prepare_expedition_node_entry(node_id)?;
        if self.state.roster.iter().filter(|employee| employee.is_alive()).count() < living_before_entry {
            if let Some(reason) = self.reassess_combat_recovery()? {
                self.run_state_mut()?.run_progression.pending_continuation = None;
                return self.fail_run(reason);
            }
        }
        if pending_choice {
            return self.handle_request_map_data();
        }
        let battle_entry_snapshot = matches!(
            node.category,
            MapNodeCategory::Combat | MapNodeCategory::Boss
        )
        .then(|| super::state::BattleEntrySnapshot::capture(&self.state))
        .transpose()?;

        let template = self.facility_template_for_map(&map)?;
        progression
            .enter_node(&mut map, &template, node_id)
            .map_err(|_| GameError::InvalidAction)?;
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        let enter_result = MapNodeExecutor::enter(node);

        if let Some(run) = self.state.run.as_mut() {
            run.map = map;
            run.map_progression = progression;
            run.boss_omen.mark_confirmed(node_id);
        }
        self.state.node_session = Some(enter_result.session.clone());

        let maybe_encounter_id = match &enter_result.payload {
            MapNodePayload::Encounter { encounter_id } => encounter_id.clone(),
            _ => None,
        };
        let entered_category = enter_result.category;
        let entered_payload = enter_result.payload.clone();
        if entered_category == MapNodeCategory::Gate {
            return self.handle_confirmed_gate_transition(node_id, resumed);
        }
        self.transition_to(GameState::InNode {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: entered_category,
        })?;
        self.evaluate_visual_novel_triggers(
            super::visual_novel_triggers::VisualNovelTriggerEvent::MapNodeEntered {
                node_id,
                category: entered_category,
                kind_id: enter_result.kind_id.clone(),
            },
        )?;

        let entered = BehaviorResult::NodeEntered {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
        };

        if matches!(
            entered,
            BehaviorResult::NodeEntered {
                category: MapNodeCategory::Combat | MapNodeCategory::Boss,
                ..
            }
        ) {
            if let Some(encounter_id) = maybe_encounter_id {
                let primary_abnormality_id = self
                    .game_data
                    .pve_data
                    .get_by_id(&encounter_id)
                    .ok_or(GameError::MissingResource("PveEncounter"))?
                    .primary_abnormality_id
                    .clone();
                return self.handle_map_combat_node(
                    node_id,
                    primary_abnormality_id,
                    encounter_id,
                    battle_entry_snapshot.ok_or(GameError::InvalidAction)?,
                );
            }
        }

        if let Some(result) =
            self.try_enter_map_content_node(node_id, entered_category, &entered_payload)?
        {
            return Ok(result);
        }

        Ok(entered)
    }

    fn handle_confirmed_gate_transition(
        &mut self,
        node_id: MapNodeId,
        resumed: bool,
    ) -> Result<BehaviorResult, GameError> {
        let recovery_restricted = self.has_combat_recovery_restriction();
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        if progression.current_node_id != Some(node_id) {
            return Err(GameError::InvalidAction);
        }
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        if node.category != MapNodeCategory::Gate {
            return Err(GameError::InvalidAction);
        }
        let template = self.facility_template_for_map(&map)?;
        progression
            .complete_current_node(&mut map, &template)
            .map_err(|_| GameError::InvalidAction)?;

        if self.run_state()?.facility_run_state.rooms.is_some() {
            let run = self.run_state_mut()?;
            run.map = map;
            run.map_progression = progression;
            self.state.node_session = None;
            self.state.active_node_content = None;
            self.transition_to(GameState::ViewingMap)?;
            return self.handle_request_map_data();
        }
        let mut run_progression = self.run_state()?.run_progression.clone();
        if !run_progression.advance_floor() {
            return Err(GameError::InvalidAction);
        }
        let mut encounter_progression = self.run_state()?.encounter_progression.clone();
        encounter_progression.handle_gate_advance();
        let previous_facility_run_state = self.run_state()?.facility_run_state.clone();
        let (next_map, next_progression, facility_run_state) = self.generate_current_floor_map(
            &run_progression,
            &previous_facility_run_state,
            &mut encounter_progression,
        );
        let next_template = self.facility_template_for_map(&next_map)?;
        let view = next_progression.view(&next_map, &next_template);
        self.apply_staged_recovery_effect(StagedRecoveryEffect::GateTransition)?;
        if run_progression.game_mode == crate::game::map::GameMode::Expedition {
            let completed_streak = self.run_state_mut()?.record_progress.complete_floor();
            self.state
                .player_profile
                .record_deathless_streak(completed_streak);
            self.state
                .player_profile
                .record_floor_reached(run_progression.floor_index().saturating_add(1));
        }
        let next_run = {
            let previous_run = self.run_state()?;
            RunState::for_next_floor_from(
                previous_run,
                next_map,
                next_progression,
                run_progression.clone(),
                encounter_progression,
            )?
            .with_facility_run_state(facility_run_state)
        };
        self.state.run = Some(next_run);
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;

        let result = BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
        };
        if recovery_restricted {
            if let Some(reason) = self.reassess_combat_recovery()? {
                if resumed { return Err(GameError::InvalidAction); }
                return self.fail_run(reason);
            }
        }
        Ok(result)
    }

    pub(super) fn handle_complete_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_complete_node_inner)
    }

    fn handle_complete_node_inner(&mut self) -> Result<BehaviorResult, GameError> {
        let completion = self.plan_complete_current_node(true)?;
        self.commit_interactive_node_completion(completion)
    }

    pub(super) fn with_state_transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let before_observation =
            crate::authoring::clone_shape::controlled_clone(&self.authoring_observation)?;
        let before = self
            .state
            .clone_non_battle_transaction_state()?
            .ok_or(GameError::InvalidAction)?;
        let before_lifecycle = self.authoring_lifecycle;
        match operation(self) {
            Ok(result) => Ok(result),
            Err(error) => {
                let player_profile = std::mem::take(&mut self.state.player_profile);
                self.state = before;
                self.authoring_lifecycle = before_lifecycle;
                self.authoring_observation = before_observation;
                // The capture deliberately contains an empty profile. Move the
                // entire live profile back, including knowledge committed during
                // the failed run-local operation, without allocating on rollback.
                self.state.player_profile = player_profile;
                Err(error)
            }
        }
    }

    pub(super) fn plan_complete_current_node(
        &mut self,
        apply_recovery_effect: bool,
    ) -> Result<StagedNodeCompletion, GameError> {
        self.plan_complete_current_node_with_context_and_encounter_progression(
            apply_recovery_effect,
            NodeCompletionContext::CurrentNode,
            None,
        )
    }

    pub(super) fn plan_complete_current_node_with_context(
        &mut self,
        apply_recovery_effect: bool,
        context: NodeCompletionContext,
    ) -> Result<StagedNodeCompletion, GameError> {
        self.plan_complete_current_node_with_context_and_encounter_progression(
            apply_recovery_effect,
            context,
            None,
        )
    }

    pub(super) fn plan_complete_current_node_with_encounter_progression(
        &mut self,
        apply_recovery_effect: bool,
        encounter_progression: Option<&EncounterProgressionState>,
    ) -> Result<StagedNodeCompletion, GameError> {
        self.plan_complete_current_node_with_context_and_encounter_progression(
            apply_recovery_effect,
            NodeCompletionContext::CurrentNode,
            encounter_progression,
        )
    }

    fn plan_complete_current_node_with_context_and_encounter_progression(
        &mut self,
        apply_recovery_effect: bool,
        context: NodeCompletionContext,
        encounter_progression_override: Option<&EncounterProgressionState>,
    ) -> Result<StagedNodeCompletion, GameError> {
        match (self.state.active_node_content.as_ref(), context) {
            (
                Some(ActiveNodeContent::HeadquartersContact(_)),
                NodeCompletionContext::ResolvedHeadquartersContact,
            ) => {}
            (Some(ActiveNodeContent::HeadquartersContact(_)), _)
            | (_, NodeCompletionContext::ResolvedHeadquartersContact) => {
                return Err(GameError::InvalidAction);
            }
            _ => {}
        }

        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let run_progression = self.run_state()?.run_progression.clone();
        let completing_node_id = progression.current_node_id;

        let recovery_effect = if apply_recovery_effect {
            self.plan_current_recovery_effect(&map, &progression)?
        } else {
            StagedRecoveryEffect::None
        };

        if !apply_recovery_effect {
            let mut encounter_progression = match encounter_progression_override {
                Some(progression) => progression.clone(),
                None => self.run_state()?.encounter_progression.clone(),
            };
            let template = self.facility_template_for_map(&map)?;
            if complete_forced_boss_with_exit_gate(
                &mut map,
                &mut progression,
                &template,
                &mut encounter_progression.boss_omen,
            )
            .map_err(|_| GameError::InvalidAction)?
            {
                let view = progression.view(&map, &template);
                return Ok(StagedNodeCompletion::NodeCompleted {
                    map,
                    progression,
                    completing_node_id,
                    boss_omen: Some(encounter_progression.boss_omen),
                    recovery_effect,
                    view,
                });
            }
        }

        let template = self.facility_template_for_map(&map)?;
        let terminal_resolution = progression
            .complete_current_node(&mut map, &template)
            .map_err(|_| GameError::InvalidAction)?;

        if terminal_resolution.is_none() {
            let manifested_rule_ids = map
                .manifest_satisfied_rules(&template)
                .map_err(GameError::InvalidStaticData)?;
            if !manifested_rule_ids.is_empty() {
                progression.refresh_available_nodes(&mut map, &template);
            }
            let view = progression.view(&map, &template);
            return Ok(StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen: None,
                recovery_effect,
                view,
            });
        }

        if !matches!(
            terminal_resolution,
            Some(
                FacilityTerminalResolution::FloorGate
                    | FacilityTerminalResolution::RunFinalBoss
                    | FacilityTerminalResolution::StandardCompletion
                    | FacilityTerminalResolution::SpecialRunCompletion
            )
        ) {
            return Err(GameError::InvalidAction);
        }

        let mut run_progression = run_progression;

        if run_progression.advance_floor() {
            let mut encounter_progression = match encounter_progression_override {
                Some(progression) => progression.clone(),
                None => self.run_state()?.encounter_progression.clone(),
            };
            if let Some(node_id) = completing_node_id {
                encounter_progression
                    .boss_omen
                    .consume_completed_source(node_id);
                encounter_progression
                    .boss_omen
                    .clear_after_forced_boss_victory(node_id);
            }
            encounter_progression.handle_gate_advance();
            let previous_facility_run_state = self.run_state()?.facility_run_state.clone();
            let (next_map, next_progression, facility_run_state) = self.generate_current_floor_map(
                &run_progression,
                &previous_facility_run_state,
                &mut encounter_progression,
            );
            let next_template = self.facility_template_for_map(&next_map)?;
            let view = next_progression.view(&next_map, &next_template);
            Ok(StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                recovery_effect,
                view,
            })
        } else {
            let view = progression.view(&map, &template);
            Ok(StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            })
        }
    }

    pub(super) fn commit_interactive_node_completion(
        &mut self,
        completion: StagedNodeCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match completion {
            StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                recovery_effect,
                view,
            } => self.commit_node_completed_with_recovery(
                map,
                progression,
                completing_node_id,
                boss_omen,
                view,
                recovery_effect,
            ),
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                recovery_effect,
                view,
            } => self.commit_floor_advanced_with_recovery(
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                view,
                recovery_effect,
            ),
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                recovery_effect,
                view,
            } => self.commit_run_complete_with_recovery(
                map,
                progression,
                run_progression,
                view,
                recovery_effect,
                false,
            ),
        }
    }

    pub(super) fn commit_live_combat_result_node_completion(
        &mut self,
        completion: CombatResultNodeCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match completion {
            CombatResultNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                view,
            } => self.commit_run_complete_with_recovery(
                map,
                progression,
                run_progression,
                view,
                StagedRecoveryEffect::None,
                true,
            ),
            completion => self.commit_combat_result_node_completion(completion),
        }
    }

    pub(super) fn commit_combat_result_node_completion(
        &mut self,
        completion: CombatResultNodeCompletion,
    ) -> Result<BehaviorResult, GameError> {
        match completion {
            CombatResultNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                boss_omen,
                view,
            } => self.commit_node_completed_with_recovery(
                map,
                progression,
                completing_node_id,
                boss_omen,
                view,
                StagedRecoveryEffect::None,
            ),
            CombatResultNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                view,
            } => self.commit_floor_advanced_with_recovery(
                next_map,
                next_progression,
                run_progression,
                facility_run_state,
                encounter_progression,
                view,
                StagedRecoveryEffect::None,
            ),
            CombatResultNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                view,
            } => self.commit_run_complete_without_recovery(map, progression, run_progression, view),
        }
    }

    fn commit_node_completed_with_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        planned_boss_omen: Option<BossOmenRunState>,
        mut view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
    ) -> Result<BehaviorResult, GameError> {
        let game_data = self.game_data.clone();
        let mut completed_all_live_final_bosses = None;
        if let Some(run) = self.state.run.as_mut() {
            let has_planned_boss_omen = planned_boss_omen.is_some();
            let mut boss_omen = planned_boss_omen.unwrap_or_else(|| run.boss_omen.clone());
            run.map = map;
            run.map_progression = progression;
            if let Some(node_id) = completing_node_id {
                if !has_planned_boss_omen {
                    boss_omen.consume_completed_source(node_id);
                    force_boss_node_if_ready(
                        &mut run.map,
                        &mut run.map_progression,
                        &run.run_progression,
                        game_data.as_ref(),
                        &mut boss_omen,
                    );
                }
                let template = game_data
                    .map_catalogs
                    .facility_templates
                    .template(&run.map.map_template_id)
                    .expect("committed run map template must remain live");
                view = run.map_progression.view(&run.map, template);
                run.retire_completed_node_setup(node_id);
            }
            run.boss_omen = boss_omen;
            if run.run_progression.game_mode == crate::game::map::GameMode::Expedition {
                completed_all_live_final_bosses = Some(run.boss_omen.clone());
            }
        }
        if let Some(boss_omen) = completed_all_live_final_bosses.as_ref() {
            self.state
                .player_profile
                .refresh_all_live_final_bosses_defeated(game_data.as_ref(), boss_omen);
        }
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        view = self.current_map_view()?;
        let completion = BehaviorResult::NodeCompleted {
            map: view,
            outcome: None,
        };
        let completion = self.finalize_combat_recovery_node_resolution(completion)?;
        let BehaviorResult::NodeCompleted { map, outcome, .. } = completion else {
            return Ok(completion);
        };
        self.save_completion_checkpoint_if_needed(recovery_effect)?;
        Ok(BehaviorResult::NodeCompleted { map, outcome })
    }

    fn commit_floor_advanced_with_recovery(
        &mut self,
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        facility_run_state: FacilityRunState,
        encounter_progression: EncounterProgressionState,
        view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
    ) -> Result<BehaviorResult, GameError> {
        if run_progression.game_mode == crate::game::map::GameMode::Expedition {
            let completed_streak = self.run_state_mut()?.record_progress.complete_floor();
            self.state
                .player_profile
                .record_deathless_streak(completed_streak);
            self.state
                .player_profile
                .record_floor_reached(run_progression.floor_index().saturating_add(1));
        }
        let next_run = {
            let previous_run = self.run_state()?;
            RunState::for_next_floor_from(
                previous_run,
                next_map,
                next_progression,
                run_progression.clone(),
                encounter_progression,
            )?
            .with_facility_run_state(facility_run_state)
        };
        self.state.run = Some(next_run);
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        self.save_completion_checkpoint_if_needed(recovery_effect)?;
        Ok(BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
        })
    }

    fn commit_run_complete_without_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
    ) -> Result<BehaviorResult, GameError> {
        self.commit_run_complete_with_recovery(
            map,
            progression,
            run_progression,
            view,
            StagedRecoveryEffect::None,
            false,
        )
    }

    fn commit_run_complete_with_recovery(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
        recovery_effect: StagedRecoveryEffect,
        defer_terminal_settlement: bool,
    ) -> Result<BehaviorResult, GameError> {
        let completed_run = {
            let previous_run = self.run_state()?;
            RunState::for_run_complete_from(previous_run, map, progression, run_progression)?
        };
        self.state.run = Some(completed_run);
        self.apply_staged_recovery_effect(recovery_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        if !defer_terminal_settlement {
            self.settle_expedition_terminal(false)?;
        }
        self.transition_to(GameState::RunComplete)?;
        Ok(BehaviorResult::RunComplete { map: view })
    }

    fn save_completion_checkpoint_if_needed(
        &mut self,
        recovery_effect: StagedRecoveryEffect,
    ) -> Result<(), GameError> {
        if recovery_effect == StagedRecoveryEffect::SavePoint {
            self.save_run_checkpoint()?;
        }
        Ok(())
    }

    pub(super) fn reconcile_employee_breakdown(&mut self) {
        let limit = self.run_policy().corrosion.breakdown_collapse_cost_limit;
        let node = self
            .state
            .node_session
            .as_ref()
            .map(|session| session.node_id);
        for employee in self.state.roster.iter_mut() {
            employee.reconcile_breakdown(limit, node);
        }
    }
    pub(super) fn consume_employee_breakdown_entry_cost(&mut self, cost: u32) -> Result<(), GameError> {
        let limit = self.run_policy().corrosion.breakdown_collapse_cost_limit;
        let node = self
            .state
            .node_session
            .as_ref()
            .map(|session| session.node_id);
        let mut roster = crate::authoring::clone_shape::controlled_clone(&self.state.roster)?;
        let mut inventory = crate::authoring::clone_shape::controlled_clone(&self.state.inventory)?;
        let mut fragments =
            crate::authoring::clone_shape::controlled_clone(&self.state.skill_fragments)?;
        for employee in roster.iter_mut() {
            employee.consume_breakdown_entry_cost(limit, cost, node);
        }
        self.recover_assets_for_new_deaths(&mut roster, &mut inventory, &mut fragments)?;
        self.state.roster = roster;
        self.state.inventory = inventory;
        self.state.skill_fragments = fragments;
        self.reconcile_relic_loadouts()?;
        Ok(())
    }

    pub(super) fn save_run_checkpoint(&mut self) -> Result<(), GameError> {
        let payload = RunCheckpointPayload::capture(
            &self.state,
            self.run_state()?,
            self.game_data.identity(),
        )?;
        self.state.run_checkpoint.payload = Some(payload);
        Ok(())
    }

    pub(super) fn handle_load_run_checkpoint(&mut self) -> Result<BehaviorResult, GameError> {
        self.with_state_transaction(Self::handle_load_run_checkpoint_inner)
    }

    fn handle_load_run_checkpoint_inner(&mut self) -> Result<BehaviorResult, GameError> {
        if !matches!(
            self.state.lifecycle.phase(),
            GameState::ViewingMap
                | GameState::NodeConfirm { .. }
                | GameState::AwaitingRunFailureDecision { .. }
        ) {
            return Err(GameError::InvalidAction);
        }
        let payload = self
            .state
            .run_checkpoint
            .payload
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        let payload = payload.copy_owned()?;
        self.validate_run_checkpoint_payload(&payload)?;
        if self.state.run_checkpoint.loads_used >= RUN_CHECKPOINT_MAX_LOADS {
            return Err(GameError::InvalidAction);
        }
        let next_loads_used = self.state.run_checkpoint.loads_used.saturating_add(1);
        self.restore_run_checkpoint_payload(payload)?;
        self.state.run_checkpoint.loads_used = next_loads_used;
        self.handle_request_map_data()
    }

    pub(super) fn restore_run_checkpoint_payload(
        &mut self,
        payload: RunCheckpointPayload,
    ) -> Result<(), GameError> {
        payload.restore_into_state(&mut self.state)?;
        self.transition_to(GameState::ViewingMap)?;
        Ok(())
    }

    pub(super) fn checkpoint_payload_is_loadable(&self) -> bool {
        self.state.run_checkpoint.loads_used < RUN_CHECKPOINT_MAX_LOADS
            && self
                .state
                .run_checkpoint
                .payload
                .as_ref()
                .is_some_and(|payload| self.validate_run_checkpoint_payload(payload).is_ok())
    }

    pub(super) fn validate_run_checkpoint_payload(
        &self,
        payload: &RunCheckpointPayload,
    ) -> Result<(), GameError> {
        if payload.run_progression.pending_continuation.is_some() { return Err(GameError::InvalidAction); }
        self.validate_run_checkpoint_payload_with_seed(payload, Some(self.run_seed))
    }

    pub(super) fn validate_run_checkpoint_payload_for_continue(
        &self,
        payload: &RunCheckpointPayload,
    ) -> Result<(), GameError> {
        self.validate_run_checkpoint_payload_with_seed(payload, None)
    }

    fn validate_run_checkpoint_payload_with_seed(
        &self,
        payload: &RunCheckpointPayload,
        expected_run_seed: Option<u64>,
    ) -> Result<(), GameError> {
        if &payload.content_identity != self.game_data.identity() {
            return Err(GameError::InvalidStaticData(
                "run checkpoint content identity does not match the retained generation"
                    .to_string(),
            ));
        }
        if payload.run_progression.instance_namespace != payload.uuid_manager.instance_namespace()
            || payload.run_progression.run_seed != payload.uuid_manager.run_seed()
            || expected_run_seed
                .is_some_and(|run_seed| run_seed != payload.run_progression.run_seed)
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint seed authority is inconsistent".to_string(),
            ));
        }
        if expected_run_seed.is_some() && self.state.run.as_ref().is_some_and(|run|
            run.run_progression.instance_namespace != payload.run_progression.instance_namespace) {
            return Err(GameError::InvalidAction);
        }
        if payload.run_progression.game_mode == crate::game::map::GameMode::Expedition {
            if self.state.player_profile.lifecycle.active_expedition != payload.run_progression.expedition_instance {
                return Err(GameError::InvalidAction);
            }
            self.state.player_profile.lifecycle.vault.validate_active_ownership(&payload.inventory, &payload.skill_fragments)
                .map_err(GameError::InvalidStaticData)?;
        }
        // Victory refills can legitimately exceed the initial supply. Apply the
        // same bound to current and SavePoint snapshots using their own ledger.
        let collected_refill = payload.facility_run_state.rooms.as_ref()
            .and_then(|rooms| rooms.golden_boughs.as_ref())
            .map_or(0, |gold| (gold.collected.len() as u64)
                .saturating_mul(u64::from(gold.deterrence_refill)));
        let supplied_deterrence = u64::from(self.run_policy().expedition_deterrence_initial)
            .saturating_add(collected_refill);
        if u64::from(payload.run_progression.deterrence) > supplied_deterrence
            || payload.run_progression.entry_receipts.iter().any(|id| payload.facility_run_state.node(&payload.map, *id).is_none_or(|node|
                matches!(node.category, MapNodeCategory::Start | MapNodeCategory::Gate)
                    || node.state == MapNodeState::Locked)) {
            return Err(GameError::InvalidAction);
        }
        if !payload
            .run_progression
            .validate_checkpoint_state(self.run_policy().setup.standard_floor_count)
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint progression does not match its map graph".to_string(),
            ));
        }
        if payload.power_model_version != self.run_policy().combat_difficulty.power_model_version
            || payload.current_floor_power_anchor == 0
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint power authority does not match live policy".to_string(),
            ));
        }
        if payload.map.nodes.iter().any(|node| {
            let Some(difficulty) = node.combat_difficulty.as_ref() else {
                return validate_revival_difficulty_authority(
                    node.id,
                    None,
                    &payload.encounter_progression.revival,
                    &self.game_data.pve_data,
                )
                .is_err();
            };
            let Some(tuning) = self
                .run_policy()
                .resolved_tuning_for_manifested_node(difficulty)
            else {
                return true;
            };
            let authored_fixed = self
                .run_policy()
                .combat_difficulty_profile(&difficulty.difficulty_profile_id)
                .is_some_and(|profile| profile.authored_fixed);
            difficulty.power_model_version != payload.power_model_version
                || if authored_fixed {
                    difficulty.floor_power_anchor != payload.current_floor_power_anchor
                        || difficulty.target_power != 0
                } else {
                    difficulty.floor_power_anchor != payload.current_floor_power_anchor
                        || difficulty.target_power
                            != percent_of_anchor_ceil(
                                difficulty.floor_power_anchor,
                                tuning.target_power_percent,
                            )
                }
                || validate_revival_difficulty_authority(
                    node.id,
                    Some(difficulty),
                    &payload.encounter_progression.revival,
                    &self.game_data.pve_data,
                )
                .is_err()
        }) {
            return Err(GameError::InvalidStaticData(
                "run checkpoint map difficulty does not match its power authority".to_string(),
            ));
        }
        self.validate_facility_checkpoint(payload)?;
        let template = self.facility_template_for_map(&payload.map)?;
        payload
            .map
            .validate_facility_template_contract(&template)
            .map_err(GameError::InvalidStaticData)?;
        if !payload
            .map_progression
            .validate_checkpoint_state(&payload.map, &template)
        {
            return Err(GameError::InvalidStaticData(
                "run checkpoint progression does not match its map graph".to_string(),
            ));
        }
        payload.recruitment.validate_exact_ownership(
            &self.game_data.employee_definition_data.definitions,
            &payload.roster,
        )?;
        self.validate_run_checkpoint_economy_graph(payload)?;
        for (node_id, session) in &payload.event_sessions {
            if session.has_relic_history() {
                let node = payload.facility_run_state.node(&payload.map, *node_id).ok_or_else(|| {
                    GameError::InvalidStaticData("relic Event session node is missing".into())
                })?;
                if session.node_id != *node_id
                    || node.category != MapNodeCategory::Event
                    || !matches!(&node.payload, MapNodePayload::Event { event_id: Some(id) } if *id == session.event_id)
                {
                    return Err(GameError::InvalidStaticData(
                        "relic Event session node authority is inconsistent".into(),
                    ));
                }
                self.validate_relic_event_session_history(session, &payload.relics)?;
            }
        }
        if payload.abnormality_attempts.iter().any(|(node, attempt)| {
            payload.facility_run_state.node(&payload.map, *node).is_none()
                || attempt.retreats_used > 2
                || attempt.max_attempts != 3
                || attempt.attempts_started > 3
                || attempt.attempts_started < attempt.retreats_used
        }) {
            return Err(GameError::InvalidStaticData(
                "run checkpoint retreat ledger is inconsistent".into(),
            ));
        }
        for key in &payload.visual_novel_trigger_checkpoint_consumed {
            use crate::game::{
                data::visual_novel_trigger_data::{
                    VisualNovelTriggerCheckpointPolicy as Policy,
                    VisualNovelTriggerRepeat as Repeat,
                },
                visual_novel::VisualNovelTriggerConsumedScope as Scope,
            };
            let valid = self
                .game_data
                .visual_novel_trigger_data
                .get_by_id(key.trigger_id.as_str())
                .is_some_and(|trigger| {
                    trigger.checkpoint_policy == Policy::RewindWithRunCheckpoint
                        && matches!(
                            (trigger.repeat, key.scope),
                            (Repeat::OncePerRun, Scope::Run)
                                | (Repeat::OncePerBattle, Scope::Battle { .. })
                                | (Repeat::OncePerNode, Scope::Node { .. })
                        )
                });
            if !valid {
                return Err(GameError::InvalidStaticData(
                    "run checkpoint visual novel trigger is inconsistent".into(),
                ));
            }
        }

        payload
            .encounter_progression
            .validate_checkpoint_state(&self.game_data.pve_data)
            .map_err(GameError::InvalidStaticData)?;
        payload
            .encounter_progression
            .boss_omen
            .validate_checkpoint_state(self.game_data.as_ref(), |node_id| {
                payload.facility_run_state.node(&payload.map, node_id).is_some()
            })
            .map_err(GameError::InvalidStaticData)?;
        for node_id in payload.encounter_progression.revival.nodes.keys() {
            if payload.facility_run_state.node(&payload.map, *node_id).is_none() {
                return Err(GameError::InvalidStaticData("hatched node is missing from its floor".into()));
            }
        }
        Ok(())
    }

    fn validate_run_checkpoint_economy_graph(
        &self,
        payload: &RunCheckpointPayload,
    ) -> Result<(), GameError> {
        let invalid = |message: &'static str| GameError::InvalidStaticData(message.to_string());
        payload.credit_ledgers.validate(&payload.skill_fragments)?;
        let resource_owners = super::resource_runtime::resource_owners(
            &self.game_data.resource_data,
            &payload.roster,
            &payload.inventory,
            &payload.skill_fragments,
        )?;
        // Check the current generation's host policy before accepting a saved
        // resource graph. Save bytes do not carry authority to relax this cap.
        let retained = payload
            .typed_resources
            .retained_extent()
            .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        crate::game::battle::runtime_control::limit(
            crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,
            "checkpoint_run_resource_extent",
            self.game_data
                .runtime_safety()
                .native_runtime()
                .max_retained_bytes,
            Some(retained),
        )?;
        if payload.typed_resources.closed {
            return Err(invalid("run checkpoint resource timeline is closed"));
        }
        payload
            .typed_resources
            .validate(&self.game_data.resource_data, &resource_owners)
            .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        if !payload.roster.validate_checkpoint_identities() {
            return Err(invalid(
                "run checkpoint roster identity graph is inconsistent",
            ));
        }
        if let Err(error) = payload.inventory.validate_checkpoint_state(&payload.roster) {
            return Err(match error {
                crate::game::resources::inventory::InventoryCheckpointValidationError::Ownership => {
                    invalid("run checkpoint equipment ownership graph is inconsistent")
                }
                crate::game::resources::inventory::InventoryCheckpointValidationError::Stack => {
                    invalid("run checkpoint inventory stack graph is inconsistent")
                }
            });
        }
        payload
            .relics
            .validate(&self.game_data.relic_data, |id| {
                payload.roster.get(&id).is_some()
            })
            .map_err(|_| invalid("run checkpoint relic graph is inconsistent"))?;
        if !payload.skill_fragments.validate_checkpoint_state(
            &self.game_data.skill_fragment_data,
            &self.state.skill_fragment_policy,
        ) {
            return Err(invalid(
                "run checkpoint skill fragment graph is inconsistent",
            ));
        }

        payload
            .skill_fragments
            .validate_bindings(&payload.roster, &self.game_data.skill_fragment_data)
            .map_err(|_| invalid("run checkpoint skill fragment graph is inconsistent"))?;
        for employee in payload.roster.iter() {
            let coherent = match employee.breakdown {
                crate::game::employee::BreakdownCounter::Inactive => {
                    !employee.is_alive() || !employee.condition.is_breakdown()
                }
                crate::game::employee::BreakdownCounter::Active {
                    remaining_cost, ..
                } => {
                    employee.is_alive()
                        && employee.condition.is_breakdown()
                        && remaining_cost > 0
                        && remaining_cost
                            <= self.run_policy().corrosion.breakdown_collapse_cost_limit
                }
            };
            if !coherent
                || employee.condition.trauma > 100
                || employee.condition.physical_damage > crate::game::employee::MAX_PHYSICAL_DAMAGE
            {
                return Err(invalid("run checkpoint breakdown counter is inconsistent"));
            }

            let profile = skill_fragment_compatibility_profile_for_employee_with_item_slot(
                employee,
                &payload.inventory,
                &payload.skill_fragments,
                &self.game_data,
                &employee.loadout.item_slot,
            )
            .map_err(|_| invalid("run checkpoint equipment or fragment graph is inconsistent"))?;
            if let Some(id) = employee.skill_fragments.active_copy_uuid() {
                use crate::game::data::skill_fragment_data::SkillFragmentCompatibilityFailureCode;

                let (copy, metadata, _) = payload
                    .skill_fragments
                    .resolve(&id, &self.game_data.skill_fragment_data)
                    .map_err(|_| invalid("run checkpoint fragment copy invalid"))?;
                let report = metadata
                    .form_compatibility_report(&copy.selected_form, &profile)
                    .ok_or_else(|| invalid("run checkpoint fragment form is incompatible"))?;
                // Public weapon removal preserves the active fragment. Its
                // missing weapon (including base block capacity) is a durable
                // loadout intermediate, while every other failure stays invalid.
                let weaponless_intermediate = !employee.has_equipped_weapon()
                    && profile.weapon_profile.is_none()
                    && report
                        .failure_codes
                        .contains(&SkillFragmentCompatibilityFailureCode::WeaponRequired)
                    && report.failure_codes.iter().all(|failure| {
                        matches!(
                            failure,
                            SkillFragmentCompatibilityFailureCode::WeaponRequired
                                | SkillFragmentCompatibilityFailureCode::BlockCapacityTooLow
                        )
                    });
                if !report.is_compatible && !weaponless_intermediate {
                    return Err(invalid("run checkpoint fragment form is incompatible"));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_live_revival_difficulty_authority(
        &self,
        node_id: MapNodeId,
        difficulty: Option<&crate::game::map::MapNodeCombatDifficulty>,
    ) -> Result<(), GameError> {
        validate_revival_difficulty_authority(node_id, difficulty, &self.run_state()?.revival, &self.game_data.pve_data)
    }
}

#[cfg(test)]
mod revival_difficulty_authority_tests {
    use super::*;
    #[test]
    fn u17_each_authored_stage_round_trips_with_its_node_identity() {
        let data = crate::game::data::ValidatedGameData::load_live_embedded().unwrap();
        let source = data.pve_data.reward_sources.iter().find(|s| s.revival.is_some()).unwrap();
        let policy = source.revival.as_ref().unwrap();
        let node_id = MapNodeId::new(uuid::Uuid::from_u128(7));
        for stage in [RevivalStage::Stage2, RevivalStage::Stage3] {
            let mut revival = crate::game::revival::RevivalRunState::default();
            revival.eggs.insert(source.reward_source_id.clone(), stage);
            revival.hatch(node_id, &source.reward_source_id, stage, Default::default()).unwrap();
            let difficulty = crate::game::map::MapNodeCombatDifficulty {
                power_model_version: data.run_policy.combat_difficulty.power_model_version,
                difficulty_profile_id: "default_facility".into(),
                class: MapNodeCombatDifficultyClass::Role(crate::game::data::run_policy_data::CombatNodeDifficultyRole::Revival),
                revival: Some(crate::game::revival::RevivalNodeTuning { stage, stat_multiplier_percent: policy.stage(stage).stat_multiplier_percent }),
                floor_power_anchor: 100, target_power: 0,
            };
            let encoded = serde_json::to_vec(&revival).unwrap();
            let restored = serde_json::from_slice(&encoded).unwrap();
            assert!(validate_revival_difficulty_authority(node_id, Some(&difficulty), &restored, &data.pve_data).is_ok());
        }
    }
}

crate::authoring::clone_shape::enum_shape!(CombatResultNodeCompletion {
    units: [],
    tuples: [],
    records: [
        NodeCompleted {
            map,
            progression,
            completing_node_id,
            boss_omen,
            view
        },
        FloorAdvanced {
            next_map,
            next_progression,
            run_progression,
            facility_run_state,
            encounter_progression,
            view
        },
        RunComplete {
            map,
            progression,
            run_progression,
            view
        }
    ]
});
