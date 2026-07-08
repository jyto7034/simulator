use super::GameCore;
use crate::game::{
    behavior::{BehaviorResult, EventChoiceSnapshotDto, EventSceneSnapshotDto, GameError},
    data::event_data::{
        EventChoiceEffect, EventChoiceId, EventChoiceNext, EventDefinition, EventId, EventSceneId,
        EventSceneNext,
    },
    employee::EmployeeRoster,
    managers::uuid_manager::UuidManager,
    map::MapNodeId,
    resources::{ActiveNodeContent, EventSessionState, EventStartedCombatState},
    resources::{Enkephalin, Inventory},
    reward::{GrantExecutionContext, GrantExecutor},
    skill_fragment::SkillFragmentInventory,
};

#[derive(Debug, Clone)]
struct EventCombatStart {
    encounter_id: String,
    primary_abnormality_id: Option<String>,
}

struct PlannedEventChoiceEffects {
    combat_start: Option<EventCombatStart>,
    inventory: Inventory,
    skill_fragments: SkillFragmentInventory,
    roster: EmployeeRoster,
    uuid_manager: UuidManager,
    enkephalin: Enkephalin,
}

impl GameCore {
    pub(super) fn enter_event_node(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> Result<BehaviorResult, GameError> {
        let event = self
            .game_data
            .event_data
            .get_by_id(event_id.as_str())
            .ok_or(GameError::MissingResource("Event"))?;
        let session = self
            .run_state()?
            .event_sessions
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| {
                EventSessionState::new(node_id, event_id.clone(), event.entry_scene_id.clone())
            });
        if session.event_id != event_id {
            return Err(GameError::InvalidAction);
        }
        if let Some(started) = session.started_combat.clone() {
            self.run_state_mut()?
                .event_sessions
                .insert(node_id, session);
            return self.handle_event_combat_node(
                node_id,
                started.primary_abnormality_id,
                started.encounter_id,
            );
        }
        let result = self.event_state_result(&session, research_deliveries)?;
        self.run_state_mut()?
            .event_sessions
            .insert(node_id, session.clone());
        self.state.active_node_content = Some(ActiveNodeContent::Event(session));
        self.refresh_allowed_actions();
        Ok(result)
    }

    pub(super) fn handle_advance_event_scene(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        current_scene_id: EventSceneId,
    ) -> Result<BehaviorResult, GameError> {
        let session = self.current_event_session()?.clone();
        if session.node_id != node_id
            || session.event_id != event_id
            || session.current_scene_id != current_scene_id
        {
            return Err(GameError::InvalidAction);
        }
        let event = self.event_definition(&event_id)?;
        let scene = event_scene(event, current_scene_id.as_str())?;
        match &scene.next {
            EventSceneNext::Scene { scene_id } => {
                let next_scene_id = scene_id.clone();
                let updated = {
                    let updated = self.current_event_session_mut()?;
                    updated.current_scene_id = next_scene_id;
                    updated.clone()
                };
                self.run_state_mut()?
                    .event_sessions
                    .insert(node_id, updated.clone());
                let snapshot = self.event_scene_snapshot(&updated)?;
                Ok(BehaviorResult::EventState {
                    node_id,
                    event: snapshot,
                    research_deliveries: vec![],
                })
            }
            EventSceneNext::End => {
                let completion = self.plan_complete_current_node(true)?;
                self.commit_interactive_node_completion(completion)
            }
            EventSceneNext::Choices { .. } => Err(GameError::InvalidAction),
        }
    }

    pub(super) fn handle_select_event_choice(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        choice_id: EventChoiceId,
    ) -> Result<BehaviorResult, GameError> {
        let session = self.current_event_session()?.clone();
        if session.node_id != node_id || session.event_id != event_id {
            return Err(GameError::InvalidAction);
        }
        if let Some(committed) = &session.committed_choice_id {
            if committed != &choice_id {
                return Err(GameError::InvalidAction);
            }
            return Ok(BehaviorResult::EventState {
                node_id,
                event: self.event_scene_snapshot(&session)?,
                research_deliveries: vec![],
            });
        }

        let event = self.event_definition(&event_id)?;
        let scene = event_scene(event, session.current_scene_id.as_str())?;
        let EventSceneNext::Choices { choices } = &scene.next else {
            return Err(GameError::InvalidAction);
        };
        let choice = choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .ok_or(GameError::InvalidAction)?
            .clone();

        let planned_effects = self.plan_event_choice_effects(node_id, &choice.effects)?;
        let next = choice.next.unwrap_or(EventChoiceNext::End);
        if let Some(combat_start) = planned_effects.combat_start.clone() {
            if let Some(result) = self
                .block_or_fail_undeployable_combat_selection_for_roster(&planned_effects.roster)?
            {
                return Ok(result);
            }
            let planned_battle = self.plan_event_combat_node_with_state(
                node_id,
                combat_start.primary_abnormality_id.clone(),
                combat_start.encounter_id.clone(),
                &planned_effects.inventory,
                &planned_effects.skill_fragments,
                &planned_effects.roster,
            )?;
            let mut updated = session.clone();
            updated.committed_choice_id = Some(choice_id);
            updated.started_combat = Some(EventStartedCombatState {
                encounter_id: combat_start.encounter_id.clone(),
                primary_abnormality_id: combat_start.primary_abnormality_id.clone(),
            });
            self.commit_event_choice_effects(planned_effects);
            self.commit_event_session(node_id, updated)?;
            return self.commit_planned_live_battle_start(node_id, planned_battle);
        }
        match next {
            EventChoiceNext::Scene { scene_id } => {
                let mut updated = session.clone();
                updated.committed_choice_id = Some(choice_id);
                updated.current_scene_id = scene_id;
                let snapshot = self.event_scene_snapshot(&updated)?;
                self.commit_event_choice_effects(planned_effects);
                self.commit_event_session(node_id, updated)?;
                Ok(BehaviorResult::EventState {
                    node_id,
                    event: snapshot,
                    research_deliveries: vec![],
                })
            }
            EventChoiceNext::End => {
                let mut updated = session.clone();
                updated.committed_choice_id = Some(choice_id);
                let completion = self.plan_complete_current_node(true)?;
                self.commit_event_choice_effects(planned_effects);
                self.commit_event_session(node_id, updated)?;
                return self.commit_interactive_node_completion(completion);
            }
        }
    }

    fn current_event_session(&self) -> Result<&EventSessionState, GameError> {
        self.state
            .active_node_content
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .as_event()
    }

    fn current_event_session_mut(&mut self) -> Result<&mut EventSessionState, GameError> {
        self.state
            .active_node_content
            .as_mut()
            .ok_or(GameError::InvalidAction)?
            .as_event_mut()
    }

    fn event_definition(&self, event_id: &EventId) -> Result<&EventDefinition, GameError> {
        self.game_data
            .event_data
            .get_by_id(event_id.as_str())
            .ok_or(GameError::MissingResource("Event"))
    }

    fn event_state_result(
        &self,
        session: &EventSessionState,
        research_deliveries: Vec<crate::game::skill_fragment::SkillFragmentResearchDelivery>,
    ) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::EventState {
            node_id: session.node_id,
            event: self.event_scene_snapshot(session)?,
            research_deliveries,
        })
    }

    pub(super) fn event_scene_snapshot(
        &self,
        session: &EventSessionState,
    ) -> Result<EventSceneSnapshotDto, GameError> {
        let event = self.event_definition(&session.event_id)?;
        let scene = event_scene(event, session.current_scene_id.as_str())?;
        let choices = match &scene.next {
            EventSceneNext::Choices { choices } => choices
                .iter()
                .map(|choice| EventChoiceSnapshotDto {
                    choice_id: choice.id.clone(),
                    label_id: choice.label_id.clone(),
                    preview: choice.preview.clone(),
                    selectable: session.committed_choice_id.is_none()
                        || session.committed_choice_id.as_ref() == Some(&choice.id),
                })
                .collect(),
            _ => vec![],
        };
        Ok(EventSceneSnapshotDto {
            event_id: session.event_id.clone(),
            current_scene_id: session.current_scene_id.clone(),
            background_id: scene.presentation.background_id.clone(),
            script_id: scene.presentation.script_id.clone(),
            speaker_id: scene.presentation.speaker_id.clone(),
            portrait_id: scene.presentation.portrait_id.clone(),
            committed_choice_id: session.committed_choice_id.clone(),
            choices,
        })
    }

    fn plan_event_choice_effects(
        &mut self,
        node_id: MapNodeId,
        effects: &[EventChoiceEffect],
    ) -> Result<PlannedEventChoiceEffects, GameError> {
        let mut combat_start = None;
        let mut staged_inventory = self.state.inventory.clone();
        let mut staged_skill_fragments = self.state.skill_fragments.clone();
        let mut staged_roster = self.state.roster.clone();
        let mut staged_uuid_manager = self.state.uuid_manager.clone();
        let mut staged_enkephalin = self.state.enkephalin.clone();

        for effect in effects {
            match effect {
                EventChoiceEffect::Grant { effects } => {
                    let seed = self.node_seed(node_id, 0x4556_5447_5241_4E54);
                    let _ = GrantExecutor::grant_effects_with_state(
                        &mut staged_inventory,
                        &mut staged_skill_fragments,
                        &mut staged_roster,
                        &mut staged_uuid_manager,
                        &mut staged_enkephalin,
                        self.game_data.as_ref(),
                        &self.state.skill_fragment_policy,
                        &GrantExecutionContext::default(),
                        effects,
                        seed,
                    )?;
                }
                EventChoiceEffect::ApplyBossOmenStepResult => {
                    panic!(
                        "EventChoiceEffect::ApplyBossOmenStepResult is not implemented; \
                         boss omen step results must not be ignored silently"
                    );
                }
                EventChoiceEffect::StartCombat {
                    encounter_id,
                    primary_abnormality_id,
                } => {
                    if combat_start.is_some() {
                        panic!("event choice defines multiple StartCombat effects");
                    }
                    let encounter = self
                        .game_data
                        .pve_data
                        .get_by_id(encounter_id)
                        .unwrap_or_else(|| {
                            panic!(
                                "event choice references missing combat encounter '{}'",
                                encounter_id
                            )
                        });
                    let resolved_primary = primary_abnormality_id
                        .clone()
                        .or_else(|| encounter.primary_abnormality_id.clone());
                    combat_start = Some(EventCombatStart {
                        encounter_id: encounter_id.clone(),
                        primary_abnormality_id: resolved_primary,
                    });
                }
            }
        }

        Ok(PlannedEventChoiceEffects {
            combat_start,
            inventory: staged_inventory,
            skill_fragments: staged_skill_fragments,
            roster: staged_roster,
            uuid_manager: staged_uuid_manager,
            enkephalin: staged_enkephalin,
        })
    }

    fn commit_event_choice_effects(&mut self, planned: PlannedEventChoiceEffects) {
        self.state.inventory = planned.inventory;
        self.state.skill_fragments = planned.skill_fragments;
        self.state.roster = planned.roster;
        self.state.uuid_manager = planned.uuid_manager;
        self.state.enkephalin = planned.enkephalin;
    }

    fn commit_event_session(
        &mut self,
        node_id: MapNodeId,
        session: EventSessionState,
    ) -> Result<(), GameError> {
        self.run_state_mut()?
            .event_sessions
            .insert(node_id, session.clone());
        self.state.active_node_content = Some(ActiveNodeContent::Event(session));
        Ok(())
    }
}

fn event_scene<'a>(
    event: &'a EventDefinition,
    scene_id: &str,
) -> Result<&'a crate::game::data::event_data::EventSceneDefinition, GameError> {
    event
        .scenes
        .iter()
        .find(|scene| scene.id.as_str() == scene_id)
        .ok_or(GameError::InvalidStaticData(format!(
            "event '{}' missing scene '{}'",
            event.id.as_str(),
            scene_id
        )))
}
