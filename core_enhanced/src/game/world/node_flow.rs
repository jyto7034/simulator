use super::{GameCore, RunState};
use crate::game::behavior::{BehaviorResult, GameError};
use crate::game::map::{
    MapGenerationConfig, MapGenerator, MapNodeCategory, MapNodeExecutor, MapNodeId, MapNodePayload,
    MapProgression, MapViewDto, RunMap, RunProgression,
};
use crate::game::resources::{GameState, SelectedEventState};

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
        self.assign_map_encounters(&mut run_map, run_progression);
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
        let combat_preview = self.combat_preview_for_node(node_id, false)?;
        let combat_deployment = self.combat_deployment_for_node(node_id)?;
        let recon_charge = self.run_state()?.recon_charge;
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
            combat_deployment,
            recon_charge,
        })
    }

    pub(super) fn handle_use_recon_scan(&mut self) -> Result<BehaviorResult, GameError> {
        let node_id = match self.get_state() {
            GameState::NodeConfirm { node_id, .. } => node_id,
            _ => return Err(GameError::InvalidAction),
        };
        let preview = self
            .combat_preview_for_node(node_id, true)?
            .ok_or(GameError::InvalidAction)?;
        let run = self.run_state_mut()?;
        if run.recon_charge == 0 {
            return Err(GameError::InvalidAction);
        }
        run.recon_charge = run.recon_charge.saturating_sub(1);
        let scan_count = run.recon_scanned_nodes.entry(node_id).or_insert(0);
        *scan_count = scan_count.saturating_add(1);
        run.combat_previews.insert(node_id, preview.clone());
        Ok(BehaviorResult::ReconScanUsed {
            node_id,
            remaining_recon_charge: run.recon_charge,
            combat_preview: preview,
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
            self.ensure_combat_deployment_ready(node_id)?;
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
        if self.state.selected_event.as_ref().is_some_and(|selected| {
            matches!(selected.event, SelectedEventState::HeadquartersContact(_))
        }) {
            return Err(GameError::InvalidAction);
        }

        let mut map = self.run_state()?.map.clone();
        let mut progression = self.run_state()?.map_progression.clone();
        let run_progression = self.run_state()?.run_progression.clone();

        self.apply_current_support_node_effect(&map, &progression)?;

        let boss_completed = progression
            .complete_current_node(&mut map)
            .map_err(|_| GameError::InvalidAction)?;

        if !boss_completed {
            let view = progression.view(&map, &run_progression);
            if let Some(run) = self.state.run.as_mut() {
                run.map = map;
                run.map_progression = progression;
            }
            self.state.node_session = None;
            self.state.selected_event = None;
            self.transition_to(GameState::ViewingMap)?;
            return Ok(BehaviorResult::NodeCompleted {
                map: view,
                outcome: None,
            });
        }

        let mut run_progression = run_progression;

        if run_progression.advance_act() {
            let (next_map, next_progression) = self.generate_current_act_map(&run_progression);
            let view = next_progression.view(&next_map, &run_progression);
            let recon_charge = self.run_state()?.recon_charge;
            let mut next_run = RunState::new(next_map, next_progression, run_progression.clone());
            next_run.recon_charge = recon_charge;
            self.state.run = Some(next_run);
            self.state.node_session = None;
            self.state.selected_event = None;
            self.transition_to(GameState::ViewingMap)?;
            Ok(BehaviorResult::ActComplete {
                act_index: run_progression.act_index,
                map: view,
            })
        } else {
            let view = progression.view(&map, &run_progression);
            self.state.run = Some(RunState::new(map, progression, run_progression));
            self.state.node_session = None;
            self.state.selected_event = None;
            self.transition_to(GameState::RunComplete)?;
            Ok(BehaviorResult::RunComplete { map: view })
        }
    }
}
