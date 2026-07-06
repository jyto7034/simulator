use super::state::RunCheckpointPayload;
use super::support::StagedSupportEffect;
use super::{map_encounters, GameCore, RunState};
use crate::game::abnormality_research::{
    RunAbnormalityEncounterHistory, RunAbnormalityResearchState,
};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::boss_omen::{apply_boss_omen_to_map, force_boss_node_if_ready, BossOmenRunState};
use crate::game::map::{
    MapGenerationConfig, MapGenerator, MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload,
    MapProgression, MapViewDto, RunMap, RunProgression,
};
use crate::game::resources::{ActiveNodeContent, GameState};

pub(super) enum StagedNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        support_effect: StagedSupportEffect,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        boss_omen: BossOmenRunState,
        support_effect: StagedSupportEffect,
        view: MapViewDto,
    },
    RunComplete {
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        support_effect: StagedSupportEffect,
        view: MapViewDto,
    },
}

pub(super) enum CombatResultNodeCompletion {
    NodeCompleted {
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        view: MapViewDto,
    },
    FloorAdvanced {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        boss_omen: BossOmenRunState,
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
                support_effect,
                view,
            } => {
                if support_effect != StagedSupportEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::NodeCompleted {
                    map,
                    progression,
                    completing_node_id,
                    view,
                })
            }
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                support_effect,
                view,
            } => {
                if support_effect != StagedSupportEffect::None {
                    return Err(GameError::InvalidAction);
                }
                Ok(CombatResultNodeCompletion::FloorAdvanced {
                    next_map,
                    next_progression,
                    run_progression,
                    boss_omen,
                    view,
                })
            }
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                support_effect,
                view,
            } => {
                if support_effect != StagedSupportEffect::None {
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
    pub(super) fn current_map_view(&self) -> Result<MapViewDto, GameError> {
        let run = self.run_state()?;
        Ok(run.map_progression.view(&run.map))
    }

    pub(super) fn generate_current_floor_map(
        &self,
        run_progression: &RunProgression,
        abnormality_research: &RunAbnormalityResearchState,
        abnormality_encounter_history: &RunAbnormalityEncounterHistory,
        boss_omen: &mut crate::game::boss_omen::BossOmenRunState,
    ) -> (RunMap, MapProgression) {
        let config = MapGenerationConfig {
            terminal_category: run_progression.terminal_node_category(),
            pre_terminal_kind_id: run_progression.standard_gate_pre_terminal_kind_id(),
            ..MapGenerationConfig::default()
        };
        let mut run_map = MapGenerator::generate(run_progression.current_floor_seed(), config);
        map_encounters::assign_map_encounters(
            &self.game_data.pve_data,
            &mut run_map,
            run_progression,
            abnormality_research,
            abnormality_encounter_history,
            self.run_policy(),
        );
        let mut progression = MapProgression::from_map(&run_map);
        force_boss_node_if_ready(
            &mut run_map,
            &mut progression,
            run_progression,
            self.game_data.as_ref(),
            boss_omen,
        );
        if boss_omen.forced_boss_node_id().is_none() {
            apply_boss_omen_to_map(
                &mut run_map,
                run_progression,
                &self.state.skill_fragments,
                self.game_data.as_ref(),
                boss_omen,
            );
            progression = MapProgression::from_map(&run_map);
        }
        (run_map, progression)
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
        let run = self.run_state()?;
        if !run.map_progression.is_node_selectable(&run.map, node_id) {
            return Err(GameError::InvalidAction);
        }
        let node = run.map.node(node_id).ok_or(GameError::InvalidAction)?;
        let enter_result = MapNodeExecutor::enter(node);
        let combat_preview = self.combat_preview_for_node(node_id)?;
        self.state.node_session = Some(enter_result.session.clone());
        self.transition_to(GameState::NodeConfirm {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: enter_result.category,
        })?;

        Ok(BehaviorResult::NodePreview {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
            map: self.current_map_view()?,
            combat_preview,
        })
    }

    pub(super) fn handle_cancel_selected_node(&mut self) -> Result<BehaviorResult, GameError> {
        self.state.node_session = None;
        self.transition_to(GameState::ViewingMap)?;
        self.handle_request_map_data()
    }

    pub(super) fn handle_confirm_enter_node(&mut self) -> Result<BehaviorResult, GameError> {
        let node_id = match self.get_state() {
            GameState::NodeConfirm { node_id, .. } => node_id,
            _ => return Err(GameError::InvalidAction),
        };
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

        progression
            .enter_node(&mut map, node_id)
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
            return self.handle_confirmed_gate_transition(node_id);
        }
        let research_deliveries = self.deliver_research_if_safe_node(entered_category)?;
        self.transition_to(GameState::InNode {
            node_id,
            kind_id: enter_result.kind_id.clone(),
            category: entered_category,
        })?;

        let entered = BehaviorResult::NodeEntered {
            node_id: enter_result.node_id,
            kind_id: enter_result.kind_id,
            category: enter_result.category,
            payload: enter_result.payload,
            session: enter_result.session,
            research_deliveries: research_deliveries.clone(),
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
                if let Some(primary_abnormality_id) = primary_abnormality_id.as_deref() {
                    let floor_index = self.run_state()?.run_progression.floor_index();
                    self.run_state_mut()?
                        .abnormality_encounter_history
                        .record_appearance(primary_abnormality_id, floor_index);
                }
                return self.handle_map_combat_node(node_id, primary_abnormality_id, encounter_id);
            }
        }

        if let Some(result) = self.try_enter_map_content_node(
            node_id,
            entered_category,
            &entered_payload,
            research_deliveries,
        )? {
            return Ok(result);
        }

        Ok(entered)
    }

    fn handle_confirmed_gate_transition(
        &mut self,
        node_id: MapNodeId,
    ) -> Result<BehaviorResult, GameError> {
        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        if progression.current_node_id != Some(node_id) {
            return Err(GameError::InvalidAction);
        }
        let node = map.node(node_id).ok_or(GameError::InvalidAction)?;
        if node.category != MapNodeCategory::Gate {
            return Err(GameError::InvalidAction);
        }
        progression
            .complete_current_node(&mut map)
            .map_err(|_| GameError::InvalidAction)?;

        let mut run_progression = self.run_state()?.run_progression.clone();
        if !run_progression.advance_floor() {
            return Err(GameError::InvalidAction);
        }
        let abnormality_research = self.run_state()?.abnormality_research.clone();
        let abnormality_encounter_history = self.run_state()?.abnormality_encounter_history.clone();
        let mut boss_omen = self.run_state()?.boss_omen.clone();
        boss_omen.handle_gate_advance();
        let (next_map, next_progression) = self.generate_current_floor_map(
            &run_progression,
            &abnormality_research,
            &abnormality_encounter_history,
            &mut boss_omen,
        );
        let view = next_progression.view(&next_map);
        self.apply_staged_support_effect(StagedSupportEffect::GateTransition)?;
        self.state.run = Some(
            RunState::new(next_map, next_progression, run_progression.clone())
                .with_abnormality_research(abnormality_research)
                .with_abnormality_encounter_history(abnormality_encounter_history)
                .with_boss_omen(boss_omen),
        );
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;

        Ok(BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
        })
    }

    pub(super) fn handle_complete_node(&mut self) -> Result<BehaviorResult, GameError> {
        let completion = self.plan_complete_current_node(true)?;
        self.commit_interactive_node_completion(completion)
    }

    pub(super) fn plan_complete_current_node(
        &mut self,
        apply_support_effect: bool,
    ) -> Result<StagedNodeCompletion, GameError> {
        if self
            .state
            .active_node_content
            .as_ref()
            .is_some_and(|content| matches!(content, ActiveNodeContent::HeadquartersContact(_)))
        {
            return Err(GameError::InvalidAction);
        }

        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let run_progression = self.run_state()?.run_progression.clone();
        let completing_node_id = progression.current_node_id;

        let support_effect = if apply_support_effect {
            self.plan_current_support_node_effect(&map, &progression)?
        } else {
            StagedSupportEffect::None
        };

        let boss_completed = progression
            .complete_current_node(&mut map)
            .map_err(|_| GameError::InvalidAction)?;

        if !boss_completed {
            let view = progression.view(&map);
            return Ok(StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                support_effect,
                view,
            });
        }

        let mut run_progression = run_progression;

        if run_progression.advance_floor() {
            let abnormality_research = self.run_state()?.abnormality_research.clone();
            let abnormality_encounter_history =
                self.run_state()?.abnormality_encounter_history.clone();
            let mut boss_omen = self.run_state()?.boss_omen.clone();
            if let Some(node_id) = completing_node_id {
                boss_omen.consume_completed_source(node_id);
                boss_omen.clear_after_forced_boss_victory(node_id);
            }
            let (next_map, next_progression) = self.generate_current_floor_map(
                &run_progression,
                &abnormality_research,
                &abnormality_encounter_history,
                &mut boss_omen,
            );
            let view = next_progression.view(&next_map);
            Ok(StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                support_effect,
                view,
            })
        } else {
            let view = progression.view(&map);
            Ok(StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                support_effect,
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
                support_effect,
                view,
            } => self.commit_node_completed_with_support(
                map,
                progression,
                completing_node_id,
                view,
                support_effect,
            ),
            StagedNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                support_effect,
                view,
            } => self.commit_floor_advanced_with_support(
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                view,
                support_effect,
            ),
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                support_effect,
                view,
            } => self.commit_run_complete_with_support(
                map,
                progression,
                run_progression,
                view,
                support_effect,
            ),
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
                view,
            } => self.commit_node_completed_without_support(
                map,
                progression,
                completing_node_id,
                view,
            ),
            CombatResultNodeCompletion::FloorAdvanced {
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                view,
            } => self.commit_floor_advanced_without_support(
                next_map,
                next_progression,
                run_progression,
                boss_omen,
                view,
            ),
            CombatResultNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                view,
            } => self.commit_run_complete_without_support(map, progression, run_progression, view),
        }
    }

    fn commit_node_completed_without_support(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        view: MapViewDto,
    ) -> Result<BehaviorResult, GameError> {
        self.commit_node_completed_with_support(
            map,
            progression,
            completing_node_id,
            view,
            StagedSupportEffect::None,
        )
    }

    fn commit_node_completed_with_support(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        completing_node_id: Option<MapNodeId>,
        mut view: MapViewDto,
        support_effect: StagedSupportEffect,
    ) -> Result<BehaviorResult, GameError> {
        let game_data = self.game_data.clone();
        if let Some(run) = self.state.run.as_mut() {
            let mut boss_omen = run.boss_omen.clone();
            run.map = map;
            run.map_progression = progression;
            if let Some(node_id) = completing_node_id {
                boss_omen.consume_completed_source(node_id);
                force_boss_node_if_ready(
                    &mut run.map,
                    &mut run.map_progression,
                    &run.run_progression,
                    game_data.as_ref(),
                    &mut boss_omen,
                );
                view = run.map_progression.view(&run.map);
                run.clear_abnormality_attempt(node_id);
                run.event_sessions.remove(&node_id);
            }
            run.boss_omen = boss_omen;
        }
        self.apply_staged_support_effect(support_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        self.save_completion_checkpoint_if_needed(support_effect)?;
        Ok(BehaviorResult::NodeCompleted {
            map: view,
            outcome: None,
        })
    }

    fn commit_floor_advanced_without_support(
        &mut self,
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        boss_omen: BossOmenRunState,
        view: MapViewDto,
    ) -> Result<BehaviorResult, GameError> {
        self.commit_floor_advanced_with_support(
            next_map,
            next_progression,
            run_progression,
            boss_omen,
            view,
            StagedSupportEffect::None,
        )
    }

    fn commit_floor_advanced_with_support(
        &mut self,
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
        boss_omen: BossOmenRunState,
        view: MapViewDto,
        support_effect: StagedSupportEffect,
    ) -> Result<BehaviorResult, GameError> {
        let abnormality_research = self
            .state
            .run
            .as_ref()
            .map(|run| run.abnormality_research.clone())
            .unwrap_or_default();
        let abnormality_encounter_history = self
            .state
            .run
            .as_ref()
            .map(|run| run.abnormality_encounter_history.clone())
            .unwrap_or_default();
        self.state.run = Some(
            RunState::new(next_map, next_progression, run_progression.clone())
                .with_abnormality_research(abnormality_research)
                .with_abnormality_encounter_history(abnormality_encounter_history)
                .with_boss_omen(boss_omen),
        );
        self.apply_staged_support_effect(support_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::ViewingMap)?;
        self.save_completion_checkpoint_if_needed(support_effect)?;
        Ok(BehaviorResult::FloorAdvanced {
            game_mode: run_progression.game_mode,
            floor_index: run_progression.floor_index(),
            map: view,
        })
    }

    fn commit_run_complete_without_support(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
    ) -> Result<BehaviorResult, GameError> {
        self.commit_run_complete_with_support(
            map,
            progression,
            run_progression,
            view,
            StagedSupportEffect::None,
        )
    }

    fn commit_run_complete_with_support(
        &mut self,
        map: RunMap,
        progression: MapProgression,
        run_progression: RunProgression,
        view: MapViewDto,
        support_effect: StagedSupportEffect,
    ) -> Result<BehaviorResult, GameError> {
        let abnormality_research = self
            .state
            .run
            .as_ref()
            .map(|run| run.abnormality_research.clone())
            .unwrap_or_default();
        let abnormality_encounter_history = self
            .state
            .run
            .as_ref()
            .map(|run| run.abnormality_encounter_history.clone())
            .unwrap_or_default();
        self.state.run = Some(
            RunState::new(map, progression, run_progression)
                .with_abnormality_research(abnormality_research)
                .with_abnormality_encounter_history(abnormality_encounter_history),
        );
        self.apply_staged_support_effect(support_effect)?;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.transition_to(GameState::RunComplete)?;
        self.save_completion_checkpoint_if_needed(support_effect)?;
        Ok(BehaviorResult::RunComplete { map: view })
    }

    fn save_completion_checkpoint_if_needed(
        &mut self,
        support_effect: StagedSupportEffect,
    ) -> Result<(), GameError> {
        if support_effect == StagedSupportEffect::SavePoint {
            self.save_run_checkpoint()?;
        }
        Ok(())
    }

    pub(super) fn save_run_checkpoint(&mut self) -> Result<(), GameError> {
        let run = self.run_state()?;
        self.state.run_checkpoint.payload = Some(RunCheckpointPayload {
            map: run.map.clone(),
            map_progression: run.map_progression.clone(),
            run_progression: run.run_progression.clone(),
            abnormality_research: run.abnormality_research.clone(),
            abnormality_encounter_history: run.abnormality_encounter_history.clone(),
            boss_omen: run.boss_omen.clone(),
            event_sessions: run.event_sessions.clone(),
            combat_previews: run.combat_previews.clone(),
            abnormality_attempts: run.abnormality_attempts.clone(),
            enkephalin: self.state.enkephalin.clone(),
            inventory: self.state.inventory.clone(),
            roster_order: self.state.roster_order.clone(),
            roster: self.state.roster.clone(),
            skill_fragments: self.state.skill_fragments.clone(),
            uuid_manager: self.state.uuid_manager.clone(),
        });
        self.refresh_allowed_actions();
        Ok(())
    }

    pub(super) fn handle_load_run_checkpoint(&mut self) -> Result<BehaviorResult, GameError> {
        if !matches!(self.state.game_state, GameState::ViewingMap) {
            return Err(GameError::InvalidAction);
        }
        if !self.state.run_checkpoint.can_load() {
            return Err(GameError::InvalidAction);
        }
        let payload = self
            .state
            .run_checkpoint
            .payload
            .clone()
            .ok_or(GameError::InvalidAction)?;
        let preserved_battle_records = self
            .state
            .run
            .as_ref()
            .map(|run| run.battle_records.clone())
            .unwrap_or_default();

        self.state.run = Some(RunState {
            map: payload.map,
            map_progression: payload.map_progression,
            run_progression: payload.run_progression,
            abnormality_research: payload.abnormality_research,
            abnormality_encounter_history: payload.abnormality_encounter_history,
            boss_omen: payload.boss_omen,
            event_sessions: payload.event_sessions,
            combat_previews: payload.combat_previews,
            abnormality_attempts: payload.abnormality_attempts,
            battle_records: preserved_battle_records,
        });
        self.state.enkephalin = payload.enkephalin;
        self.state.inventory = payload.inventory;
        self.state.roster_order = payload.roster_order;
        self.state.roster = payload.roster;
        self.state.skill_fragments = payload.skill_fragments;
        self.state.uuid_manager = payload.uuid_manager;
        self.state.node_session = None;
        self.state.active_node_content = None;
        self.state.active_battle = None;
        self.state.run_checkpoint.loads_used =
            self.state.run_checkpoint.loads_used.saturating_add(1);
        self.transition_to(GameState::ViewingMap)?;
        self.handle_request_map_data()
    }
}
