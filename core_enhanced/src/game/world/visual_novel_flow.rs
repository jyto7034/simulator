use super::{runtime_presentation_mode, GameCore};
use crate::game::{
    behavior::{BehaviorResult, GameError, LiveBattleUpdateDto},
    data::visual_novel_data::{
        VisualNovelChoiceId, VisualNovelChoiceNext, VisualNovelId, VisualNovelSceneId,
        VisualNovelSceneNext,
    },
    resources::GameState,
    visual_novel::{VisualNovelSession, VisualNovelSessionPresentationMode, VisualNovelSource},
};

pub(super) struct ResolvedVisualNovelSessionDefinition {
    vn_id: VisualNovelId,
    current_scene_id: VisualNovelSceneId,
    presentation_mode: VisualNovelSessionPresentationMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VisualNovelQueuePlacement {
    Tail,
    FrontOfQueue,
}

#[derive(Debug, Clone)]
pub(super) enum VisualNovelProgressionOutcome {
    SceneChanged,
    Completed {
        completed_session: VisualNovelSession,
    },
}

impl GameCore {
    pub(super) fn resolve_visual_novel_session_definition(
        &self,
        vn_id: VisualNovelId,
        missing_definition: impl FnOnce() -> GameError,
    ) -> Result<ResolvedVisualNovelSessionDefinition, GameError> {
        let definition = self
            .game_data
            .visual_novel_data
            .get_by_id(vn_id.as_str())
            .ok_or_else(missing_definition)?;
        Ok(ResolvedVisualNovelSessionDefinition {
            vn_id,
            current_scene_id: VisualNovelSceneId::new(definition.entry_scene_id.as_str()),
            presentation_mode: runtime_presentation_mode(definition.default_presentation_mode),
        })
    }

    pub(super) fn assemble_visual_novel_session(
        &mut self,
        definition: ResolvedVisualNovelSessionDefinition,
        source: VisualNovelSource,
        blocking: bool,
    ) -> VisualNovelSession {
        VisualNovelSession {
            session_id: self
                .state
                .uuid_manager
                .next_visual_novel_session()
                .to_string(),
            source,
            vn_id: definition.vn_id,
            current_scene_id: definition.current_scene_id,
            committed_choice_id: None,
            blocking,
            presentation_mode: definition.presentation_mode,
        }
    }

    pub(super) fn activate_or_queue_visual_novel_session(
        &mut self,
        session: VisualNovelSession,
        placement: VisualNovelQueuePlacement,
    ) {
        if self.state.visual_novel.active.is_none() {
            self.state.visual_novel.active = Some(session);
            return;
        }
        match placement {
            VisualNovelQueuePlacement::Tail => self.state.visual_novel.queue.push(session),
            VisualNovelQueuePlacement::FrontOfQueue => {
                self.state.visual_novel.queue.insert(0, session);
            }
        }
    }

    pub(super) fn handle_advance_visual_novel(
        &mut self,
        session_id: String,
        current_scene_id: VisualNovelSceneId,
    ) -> Result<BehaviorResult, GameError> {
        let next = self.active_visual_novel_scene_next(&session_id, &current_scene_id)?;
        if matches!(next, VisualNovelSceneNext::Choices { .. }) {
            return Err(GameError::InvalidAction);
        }

        let outcome = match next {
            VisualNovelSceneNext::Scene { scene_id } => {
                self.state
                    .visual_novel
                    .active
                    .as_mut()
                    .ok_or(GameError::InvalidAction)?
                    .current_scene_id = scene_id;
                VisualNovelProgressionOutcome::SceneChanged
            }
            VisualNovelSceneNext::End => self.finish_active_visual_novel_session()?,
            VisualNovelSceneNext::Choices { .. } => unreachable!("choices rejected above"),
        };
        self.apply_visual_novel_progression_outcome(&outcome)?;

        Ok(BehaviorResult::VisualNovelState {
            visual_novel: self.visual_novel_snapshot_dto()?,
        })
    }

    pub(super) fn handle_select_visual_novel_choice(
        &mut self,
        session_id: String,
        current_scene_id: VisualNovelSceneId,
        choice_id: VisualNovelChoiceId,
    ) -> Result<BehaviorResult, GameError> {
        let next =
            self.active_visual_novel_choice_next(&session_id, &current_scene_id, &choice_id)?;

        let dealer_action = self.dealer_visual_novel_choice(&session_id, &choice_id)?;
        let outcome = match next {
            VisualNovelChoiceNext::Scene { scene_id } => {
                let active = self
                    .state
                    .visual_novel
                    .active
                    .as_mut()
                    .ok_or(GameError::InvalidAction)?;
                active.current_scene_id = scene_id;
                active.committed_choice_id = None;
                VisualNovelProgressionOutcome::SceneChanged
            }
            VisualNovelChoiceNext::End => self.finish_active_visual_novel_session()?,
        };

        self.apply_visual_novel_progression_outcome(&outcome)?;
        if let Some((revision, action)) = dealer_action { self.handle_dealer_action(revision, action)?; }

        Ok(BehaviorResult::VisualNovelState {
            visual_novel: self.visual_novel_snapshot_dto()?,
        })
    }

    pub(super) fn active_visual_novel_scene_next(
        &self,
        session_id: &str,
        current_scene_id: &VisualNovelSceneId,
    ) -> Result<VisualNovelSceneNext, GameError> {
        let session = self
            .state
            .visual_novel
            .active
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        if session.session_id != session_id || session.current_scene_id != *current_scene_id {
            return Err(GameError::InvalidAction);
        }
        let visual_novel = self
            .game_data
            .visual_novel_data
            .get_by_id(session.vn_id.as_str())
            .ok_or(GameError::MissingResource("VisualNovel"))?;
        let scene = visual_novel
            .scene_by_id(current_scene_id.as_str())
            .ok_or(GameError::MissingResource("VisualNovelScene"))?;
        Ok(scene.next.clone())
    }

    fn active_visual_novel_choice_next(
        &self,
        session_id: &str,
        current_scene_id: &VisualNovelSceneId,
        choice_id: &VisualNovelChoiceId,
    ) -> Result<VisualNovelChoiceNext, GameError> {
        let next = self.active_visual_novel_scene_next(session_id, current_scene_id)?;
        let VisualNovelSceneNext::Choices { choices } = next else {
            return Err(GameError::InvalidAction);
        };
        choices
            .into_iter()
            .find(|choice| choice.id == *choice_id)
            .map(|choice| choice.next)
            .ok_or(GameError::InvalidAction)
    }

    fn finish_active_visual_novel_session(
        &mut self,
    ) -> Result<VisualNovelProgressionOutcome, GameError> {
        let visual_novel = &mut self.state.visual_novel;
        let completed_session = visual_novel.active.take().ok_or(GameError::InvalidAction)?;
        visual_novel.active = if visual_novel.queue.is_empty() {
            None
        } else {
            Some(visual_novel.queue.remove(0))
        };
        Ok(VisualNovelProgressionOutcome::Completed { completed_session })
    }

    fn apply_visual_novel_progression_outcome(
        &mut self,
        outcome: &VisualNovelProgressionOutcome,
    ) -> Result<(), GameError> {
        match outcome {
            VisualNovelProgressionOutcome::SceneChanged => Ok(()),
            VisualNovelProgressionOutcome::Completed { completed_session } => {
                if matches!(completed_session.source, VisualNovelSource::HqBriefing) {
                    let game_mode = match self.state.lifecycle.phase() {
                        GameState::PreRunBriefing { game_mode } => *game_mode,
                        _ => return Ok(()),
                    };
                    self.prepare_starter_employee_candidates()?;
                    if game_mode == crate::game::map::GameMode::Tutorial {
                        let preset = self.run_policy().tutorial_employee_definition_ids.clone();
                        self.handle_select_starter_employees(preset, Vec::new())?;
                    } else {
                        self.transition_to(GameState::SelectingStarterEmployees)?;
                    }
                    tracing::info!(
                        "HQ briefing VN completed; starter employee selection opened for {:?} mode",
                        game_mode
                    );
                }
                Ok(())
            }
        }
    }

    pub(super) fn current_battle_update_result_for_visual_novel_transport(
        &mut self,
        after_seq: u64,
    ) -> Result<BehaviorResult, GameError> {
        let roster = self.roster()?.clone();
        let active = self
            .state
            .active_battle
            .as_mut()
            .ok_or(GameError::InvalidAction)?;
        active.commit_battle_projection()?;
        let battle_update = active.battle_update_dto_after_or_resync(after_seq, &roster)?;
        Ok(BehaviorResult::BattleState {
            battle_uuid: active.battle_uuid,
            node_type: active.node_type,
            mission_variant: active.mission_variant,
            encounter_id: active.encounter_id.clone(),
            combat_preview: active.combat_preview.clone(),
            battle_update,
            finished: active.execution.is_finished(),
        })
    }

    pub(super) fn attach_visual_novel_to_battle_result(
        &self,
        result: &mut BehaviorResult,
    ) -> Result<(), GameError> {
        let visual_novel = self.visual_novel_snapshot_dto()?;
        match result {
            BehaviorResult::BattleAdvanced { battle_update, .. }
            | BehaviorResult::BattleState { battle_update, .. }
            | BehaviorResult::BattlePlaybackChanged { battle_update, .. }
            | BehaviorResult::BattleUnitDeployed { battle_update, .. }
            | BehaviorResult::BattleUnitWithdrawn { battle_update, .. }
            | BehaviorResult::BattleEngagementStanceChanged { battle_update, .. }
            | BehaviorResult::BattleUnitTacticallyRelocated { battle_update, .. }
            | BehaviorResult::BattleUnitsTacticallySwapped { battle_update, .. }
            | BehaviorResult::BattleSkillTargetingFocusChanged { battle_update, .. }
            | BehaviorResult::BattleSkillActivated { battle_update, .. }
            | BehaviorResult::BattleRelationSelected { battle_update, .. } => {
                let action_capabilities =
                    self.project_battle_action_capabilities(visual_novel.as_ref())?;
                attach_visual_novel_to_update(battle_update, visual_novel);
                battle_update.action_capabilities = action_capabilities;
            }
            _ => {}
        }
        Ok(())
    }
}

fn attach_visual_novel_to_update(
    battle_update: &mut LiveBattleUpdateDto,
    visual_novel: Option<crate::game::behavior::VisualNovelSnapshotDto>,
) {
    battle_update.visual_novel = visual_novel;
}
