use super::state::RunCheckpointPayload;
use super::support::StagedSupportEffect;
use super::{map_encounters, GameCore, RunState};
use crate::game::behavior::{BehaviorResult, GameError};
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
    ActComplete {
        next_map: RunMap,
        next_progression: MapProgression,
        run_progression: RunProgression,
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

impl GameCore {
    pub(super) fn current_map_view(&self) -> Result<MapViewDto, GameError> {
        let run = self.run_state()?;
        Ok(run.map_progression.view(&run.map, &run.run_progression))
    }

    pub(super) fn generate_current_act_map(
        &self,
        run_progression: &RunProgression,
    ) -> (RunMap, MapProgression) {
        let mut run_map = MapGenerator::generate(
            run_progression.current_act_seed(),
            MapGenerationConfig::default(),
        );
        map_encounters::assign_map_encounters(
            &self.game_data.pve_data,
            &mut run_map,
            run_progression,
        );
        let progression = MapProgression::from_map(&run_map);
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
        }
        self.state.node_session = Some(enter_result.session.clone());

        let maybe_encounter_id = match &enter_result.payload {
            MapNodePayload::Encounter { encounter_id } => encounter_id.clone(),
            _ => None,
        };
        let entered_category = enter_result.category;
        let entered_payload = enter_result.payload.clone();
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
                let abnormality_id = self
                    .game_data
                    .pve_data
                    .get_by_id(&encounter_id)
                    .ok_or(GameError::MissingResource("PveEncounter"))?
                    .abnormality_id
                    .clone();
                return self.handle_map_combat_node(node_id, abnormality_id, encounter_id);
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

    pub(super) fn handle_complete_node(&mut self) -> Result<BehaviorResult, GameError> {
        let completion = self.plan_complete_current_node(true)?;
        self.commit_staged_node_completion(completion)
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
            let view = progression.view(&map, &run_progression);
            return Ok(StagedNodeCompletion::NodeCompleted {
                map,
                progression,
                completing_node_id,
                support_effect,
                view,
            });
        }

        let mut run_progression = run_progression;

        if run_progression.advance_act() {
            let (next_map, next_progression) = self.generate_current_act_map(&run_progression);
            let view = next_progression.view(&next_map, &run_progression);
            Ok(StagedNodeCompletion::ActComplete {
                next_map,
                next_progression,
                run_progression,
                support_effect,
                view,
            })
        } else {
            let view = progression.view(&map, &run_progression);
            Ok(StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                support_effect,
                view,
            })
        }
    }

    pub(super) fn commit_staged_node_completion(
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
            } => {
                if let Some(run) = self.state.run.as_mut() {
                    run.map = map;
                    run.map_progression = progression;
                    if let Some(node_id) = completing_node_id {
                        run.clear_abnormality_attempt(node_id);
                    }
                }
                self.apply_staged_support_effect(support_effect)?;
                self.state.node_session = None;
                self.state.active_node_content = None;
                self.transition_to(GameState::ViewingMap)?;
                if support_effect == StagedSupportEffect::SavePoint {
                    self.save_run_checkpoint()?;
                }
                return Ok(BehaviorResult::NodeCompleted {
                    map: view,
                    outcome: None,
                });
            }
            StagedNodeCompletion::ActComplete {
                next_map,
                next_progression,
                run_progression,
                support_effect,
                view,
            } => {
                self.state.run = Some(RunState::new(
                    next_map,
                    next_progression,
                    run_progression.clone(),
                ));
                self.apply_staged_support_effect(support_effect)?;
                self.state.node_session = None;
                self.state.active_node_content = None;
                self.transition_to(GameState::ViewingMap)?;
                if support_effect == StagedSupportEffect::SavePoint {
                    self.save_run_checkpoint()?;
                }
                Ok(BehaviorResult::ActComplete {
                    act_index: run_progression.act_index,
                    map: view,
                })
            }
            StagedNodeCompletion::RunComplete {
                map,
                progression,
                run_progression,
                support_effect,
                view,
            } => {
                self.state.run = Some(RunState::new(map, progression, run_progression));
                self.apply_staged_support_effect(support_effect)?;
                self.state.node_session = None;
                self.state.active_node_content = None;
                self.transition_to(GameState::RunComplete)?;
                if support_effect == StagedSupportEffect::SavePoint {
                    self.save_run_checkpoint()?;
                }
                Ok(BehaviorResult::RunComplete { map: view })
            }
        }
    }

    pub(super) fn save_run_checkpoint(&mut self) -> Result<(), GameError> {
        let run = self.run_state()?;
        self.state.run_checkpoint.payload = Some(RunCheckpointPayload {
            map: run.map.clone(),
            map_progression: run.map_progression.clone(),
            run_progression: run.run_progression.clone(),
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
