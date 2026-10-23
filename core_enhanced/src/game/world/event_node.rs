use rand::{Rng, SeedableRng};
use uuid::Uuid;

use super::{grant_state::EconomyGrantState, GameCore};
use crate::game::{
    behavior::{
        ActionDisabledReasonCode, ActionKind, BehaviorResult, EventChoiceEvaluationDto,
        EventCommittedCheckDto, EventEmployeeEvaluationDto, EventInteractionDto,
        EventSceneSnapshotDto, GameError, PayloadEligibilityDto,
    },
    data::event_data::{
        EventCheckOutcome, EventChoiceDefinition, EventChoiceId, EventDefinition, EventId,
        EventSceneId, EventSceneNext, EventSuccessAction,
    },
    employee::{Employee, EmployeeActivity},
    map::MapNodeId,
    resources::{
        ActiveNodeContent, CommittedRelicCorruptionSelection, EventCheckModifierComponent,
        EventChoiceKeyEvidence, EventCommittedCheckState, EventEquipmentModifierEvidence,
        EventSessionState, EventStartedCombatState, Inventory,
    },
    reward::{GrantExecutionContext, RewardEffect},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventEmployeeCheckEvaluation {
    pub employee_modifier: u8,
    pub equipment_modifier: u8,
    pub modifier_total: u8,
    pub equipment_modifier_evidence: Option<EventEquipmentModifierEvidence>,
    pub choice_key_evidence: Option<EventChoiceKeyEvidence>,
    pub success_chance_percent: u8,
    pub disabled_reason: Option<ActionDisabledReasonCode>,
}

impl EventEmployeeCheckEvaluation {
    fn eligible(&self) -> bool {
        self.disabled_reason.is_none()
    }
}

impl GameCore {
    pub(super) fn is_exact_committed_event_resolve(
        &self,
        node_id: MapNodeId,
        event_id: &EventId,
        source_scene_id: &EventSceneId,
        choice_id: &EventChoiceId,
        employee_uuid: Uuid,
    ) -> bool {
        let Some(ActiveNodeContent::Event(session)) = self.state.active_node_content.as_ref()
        else {
            return false;
        };
        let Some(committed) = session.committed_check.as_ref() else {
            return false;
        };
        session.node_id == node_id
            && session.event_id == *event_id
            && committed.source_scene_id == *source_scene_id
            && committed.choice_id == *choice_id
            && committed.employee_uuid == employee_uuid
    }

    pub(super) fn evaluate_event_action_kind(
        &self,
        action: ActionKind,
    ) -> Option<Result<(), ActionDisabledReasonCode>> {
        let ActiveNodeContent::Event(session) = self.state.active_node_content.as_ref()? else {
            return None;
        };
        let event = match self
            .game_data
            .event_data
            .get_by_id(session.event_id.as_str())
        {
            Some(event) => event,
            None => return Some(Err(ActionDisabledReasonCode::ContentUnavailable)),
        };
        let scene = match event
            .scenes
            .iter()
            .find(|scene| scene.id == session.current_scene_id)
        {
            Some(scene) => scene,
            None => return Some(Err(ActionDisabledReasonCode::ContentUnavailable)),
        };
        match (action, &scene.next) {
            (ActionKind::ChooseRelicCorruption, EventSceneNext::RelicCorruption { .. }) => {
                Some(Ok(()))
            }
            (
                ActionKind::ResolveEventChoiceCheck,
                EventSceneNext::EmployeeCheckChoices { choices, .. },
            ) => {
                let mut has_eligible_payload = false;
                for choice in choices {
                    for employee in self.state.roster.iter() {
                        match self.evaluate_event_choice_for_employee(choice, employee.uuid) {
                            Ok(evaluation) if evaluation.eligible() => {
                                has_eligible_payload = true;
                                break;
                            }
                            Ok(_) => {}
                            Err(_) => {
                                return Some(Err(ActionDisabledReasonCode::ContentUnavailable));
                            }
                        }
                    }
                    if has_eligible_payload {
                        break;
                    }
                }
                if !has_eligible_payload {
                    Some(Err(ActionDisabledReasonCode::NoEligiblePayload))
                } else {
                    Some(Ok(()))
                }
            }
            (ActionKind::AdvanceEventScene, EventSceneNext::Scene { .. } | EventSceneNext::End) => {
                Some(Ok(()))
            }
            _ => None,
        }
    }

    pub(super) fn validate_event_advance_payload(
        &self,
        node_id: MapNodeId,
        event_id: &EventId,
        current_scene_id: &EventSceneId,
    ) -> Result<(), ActionDisabledReasonCode> {
        let Some(ActiveNodeContent::Event(session)) = self.state.active_node_content.as_ref()
        else {
            return Err(ActionDisabledReasonCode::ContextIrrelevant);
        };
        if session.node_id == node_id
            && session.event_id == *event_id
            && session.current_scene_id == *current_scene_id
        {
            Ok(())
        } else {
            Err(ActionDisabledReasonCode::NoEligiblePayload)
        }
    }

    pub(super) fn validate_event_resolve_payload(
        &self,
        node_id: MapNodeId,
        event_id: &EventId,
        current_scene_id: &EventSceneId,
        choice_id: &EventChoiceId,
        employee_uuid: Uuid,
    ) -> Result<(), ActionDisabledReasonCode> {
        let Some(ActiveNodeContent::Event(session)) = self.state.active_node_content.as_ref()
        else {
            return Err(ActionDisabledReasonCode::ContextIrrelevant);
        };
        if session.node_id != node_id
            || session.event_id != *event_id
            || session.current_scene_id != *current_scene_id
        {
            return Err(ActionDisabledReasonCode::NoEligiblePayload);
        }
        let choice = self
            .game_data
            .event_data
            .get_by_id(event_id.as_str())
            .and_then(|event| {
                event
                    .scenes
                    .iter()
                    .find(|scene| scene.id == *current_scene_id)
            })
            .and_then(|scene| match &scene.next {
                EventSceneNext::EmployeeCheckChoices { choices, .. } => {
                    choices.iter().find(|choice| choice.id == *choice_id)
                }
                _ => None,
            });
        let Some(choice) = choice else {
            return Err(ActionDisabledReasonCode::NoEligiblePayload);
        };
        if !self.state.roster.contains(&employee_uuid) {
            return Err(ActionDisabledReasonCode::NoEligiblePayload);
        }
        let evaluation = self
            .evaluate_event_choice_for_employee(choice, employee_uuid)
            .map_err(|_| ActionDisabledReasonCode::ContentUnavailable)?;
        evaluation.disabled_reason.map_or(Ok(()), Err)
    }

    pub(super) fn enter_event_node(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
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
            .map(EventSessionState::clone_for_relic_transition)
            .transpose()?
            .unwrap_or_else(|| {
                EventSessionState::new(node_id, event_id.clone(), event.entry_scene_id.clone())
            });
        if session.event_id != event_id {
            return Err(GameError::InvalidAction);
        }
        if session.has_relic_history() && session.started_combat.is_some() {
            session.preflight_relic_clone()?;
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
        let result = self.event_state_result(&session)?;
        self.commit_event_session_required_run(node_id, session)?;
        Ok(result)
    }

    pub(super) fn handle_advance_event_scene(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        current_scene_id: EventSceneId,
    ) -> Result<BehaviorResult, GameError> {
        let session = self.current_event_session()?.clone_for_relic_transition()?;
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
                let mut updated = session;
                updated.current_scene_id = scene_id.clone();
                self.commit_event_session_required_run(
                    node_id,
                    updated.clone_for_relic_transition()?,
                )?;
                self.event_state_result(&updated)
            }
            EventSceneNext::End => {
                let completion = self.plan_complete_current_node(true)?;
                self.commit_event_session_required_run(node_id, session)?;
                self.commit_interactive_node_completion(completion)
            }
            EventSceneNext::EmployeeCheckChoices { .. }
            | EventSceneNext::RelicCorruption { .. } => Err(GameError::InvalidAction),
        }
    }

    pub(super) fn handle_resolve_event_choice_check(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        current_scene_id: EventSceneId,
        choice_id: EventChoiceId,
        employee_uuid: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        let session = self.current_event_session()?.clone_for_relic_transition()?;
        if session.node_id != node_id || session.event_id != event_id {
            return Err(GameError::InvalidAction);
        }
        if let Some(committed) = &session.committed_check {
            if committed.source_scene_id != current_scene_id
                || committed.choice_id != choice_id
                || committed.employee_uuid != employee_uuid
            {
                return Err(GameError::InvalidAction);
            }
            return self.event_state_result(&session);
        }
        if session.current_scene_id != current_scene_id {
            return Err(GameError::InvalidAction);
        }
        self.validate_event_resolve_payload(
            node_id,
            &event_id,
            &current_scene_id,
            &choice_id,
            employee_uuid,
        )
        .map_err(|_| GameError::InvalidAction)?;

        let event = self.event_definition(&event_id)?;
        let scene = event_scene(event, current_scene_id.as_str())?;
        let EventSceneNext::EmployeeCheckChoices { choices } = &scene.next else {
            return Err(GameError::InvalidAction);
        };
        let choice = choices
            .iter()
            .find(|choice| choice.id == choice_id)
            .ok_or(GameError::InvalidAction)?
            .clone();

        let evaluation = self
            .evaluate_event_choice_for_employee(&choice, employee_uuid)
            .map_err(|_| GameError::InvalidAction)?;
        if !evaluation.eligible() {
            return Err(GameError::InvalidAction);
        }

        let raw_roll = self.roll_event_check(node_id);
        let total_roll = raw_roll + evaluation.modifier_total;
        let outcome = classify_event_check(raw_roll, total_roll, choice.check.dc);
        let modifier_components = event_modifier_components(choice.check.aptitude, &evaluation);
        let succeeded = matches!(
            outcome,
            EventCheckOutcome::Success | EventCheckOutcome::CriticalSuccess
        );
        let planned_grant = if succeeded {
            match &choice.success_action {
                Some(EventSuccessAction::Grant { effects }) => {
                    Some(self.plan_event_grant(node_id, effects)?)
                }
                _ => None,
            }
        } else {
            None
        };
        let planned_battle = if succeeded {
            match &choice.success_action {
                Some(EventSuccessAction::StartCombat {
                    encounter_id,
                    primary_abnormality_id,
                }) => Some(self.plan_event_combat_node_with_state(
                    node_id,
                    primary_abnormality_id.clone(),
                    encounter_id.clone(),
                    &self.state.inventory,
                    &self.state.skill_fragments,
                    &self.state.roster,
                )?),
                _ => None,
            }
        } else {
            None
        };

        let mut updated = session;
        updated.committed_check = Some(EventCommittedCheckState {
            source_scene_id: current_scene_id,
            choice_id,
            employee_uuid,
            raw_roll,
            aptitude: choice.check.aptitude,
            dc: choice.check.dc,
            modifier_components,
            modifier_total: evaluation.modifier_total,
            total_roll,
            choice_key_evidence: evaluation.choice_key_evidence,
            outcome,
        });
        updated.current_scene_id = if succeeded {
            choice.success_scene_id
        } else {
            choice.failure_scene_id
        };

        if let Some(planned) = planned_grant {
            self.commit_event_grant(planned);
        }
        if let Some(planned) = planned_battle {
            let EventSuccessAction::StartCombat {
                encounter_id,
                primary_abnormality_id,
            } = choice
                .success_action
                .expect("planned battle requires a StartCombat action")
            else {
                unreachable!("planned battle requires a StartCombat action")
            };
            updated.started_combat = Some(EventStartedCombatState {
                encounter_id,
                primary_abnormality_id,
            });
            self.commit_event_session_required_run(node_id, updated)?;
            return self.commit_planned_live_battle_start(node_id, planned);
        }

        self.commit_event_session_required_run(node_id, updated.clone_for_relic_transition()?)?;
        self.event_state_result(&updated)
    }

    pub(super) fn is_exact_committed_relic_corruption(
        &self,
        node_id: MapNodeId,
        event_id: &EventId,
        source_scene_id: &EventSceneId,
        copy_uuid: Option<Uuid>,
    ) -> Result<bool, GameError> {
        let Some(ActiveNodeContent::Event(session)) = &self.state.active_node_content else {
            return Ok(false);
        };
        if session.node_id != node_id || session.event_id != *event_id {
            return Ok(false);
        }
        let Some(history) = &session.relic_corruption_selections else {
            return Ok(false);
        };
        crate::authoring::control::charge(history.len() as u64, "relic_event_history_retry")?;
        Ok(history.iter().any(|receipt| {
            receipt.source_scene_id == *source_scene_id && receipt.copy_uuid == copy_uuid
        }))
    }

    pub(super) fn validate_relic_corruption_payload(
        &self,
        node_id: MapNodeId,
        event_id: &EventId,
        source_scene_id: &EventSceneId,
        copy_uuid: Option<Uuid>,
    ) -> Result<(), ActionDisabledReasonCode> {
        let Some(ActiveNodeContent::Event(session)) = &self.state.active_node_content else {
            return Err(ActionDisabledReasonCode::ContextIrrelevant);
        };
        if session.node_id != node_id
            || session.event_id != *event_id
            || session.current_scene_id != *source_scene_id
        {
            return Err(ActionDisabledReasonCode::NoEligiblePayload);
        }
        let event = self
            .game_data
            .event_data
            .get_by_id(event_id.as_str())
            .ok_or(ActionDisabledReasonCode::ContentUnavailable)?;
        crate::authoring::control::charge(event.scenes.len() as u64, "relic_event_payload_scene")
            .map_err(|_| ActionDisabledReasonCode::ContentUnavailable)?;
        let scene = event
            .scenes
            .iter()
            .find(|scene| scene.id == *source_scene_id)
            .ok_or(ActionDisabledReasonCode::ContentUnavailable)?;
        if !matches!(scene.next, EventSceneNext::RelicCorruption { .. }) {
            return Err(ActionDisabledReasonCode::NoEligiblePayload);
        }
        if let Some(copy) = copy_uuid {
            self.state
                .relics
                .validate_corruption_target(copy, &self.game_data.relic_data)
                .map_err(|_| ActionDisabledReasonCode::NoEligiblePayload)?;
            if let Some(history) = &session.relic_corruption_selections {
                crate::authoring::control::charge(history.len() as u64, "relic_event_payload_copy")
                    .map_err(|_| ActionDisabledReasonCode::ContentUnavailable)?;
                if history
                    .iter()
                    .any(|receipt| receipt.copy_uuid == Some(copy))
                {
                    return Err(ActionDisabledReasonCode::NoEligiblePayload);
                }
            }
        }
        Ok(())
    }

    pub(super) fn handle_choose_relic_corruption(
        &mut self,
        node_id: MapNodeId,
        event_id: EventId,
        source_scene_id: EventSceneId,
        copy_uuid: Option<Uuid>,
    ) -> Result<BehaviorResult, GameError> {
        // Exact retry uses the original scene receipt in this active session.
        // It never re-targets a copy that may since have been dismantled.
        if self.is_exact_committed_relic_corruption(
            node_id,
            &event_id,
            &source_scene_id,
            copy_uuid,
        )? {
            return self.event_state_result(self.current_event_session()?);
        }
        let eligibility =
            self.validate_relic_corruption_payload(node_id, &event_id, &source_scene_id, copy_uuid);
        crate::authoring::control::poll("relic_event_payload")?;
        eligibility.map_err(|_| GameError::InvalidAction)?;
        self.ensure_relic_operation()?;
        let session = self.current_event_session()?;
        session.preflight_relic_clone()?;
        let mut updated = crate::authoring::clone_shape::controlled_clone(session)?;
        let event = self.event_definition(&event_id)?;
        let EventSceneNext::RelicCorruption { next_scene_id } =
            &event_scene(event, source_scene_id.as_str())?.next
        else {
            return Err(GameError::InvalidAction);
        };
        let history_len = updated
            .relic_corruption_selections
            .as_ref()
            .map_or(0, Vec::len);
        crate::authoring::control::charge(event.scenes.len() as u64, "relic_event_scene_bound")?;
        let max_selections = event
            .scenes
            .iter()
            .filter(|scene| matches!(scene.next, EventSceneNext::RelicCorruption { .. }))
            .count();
        if history_len >= max_selections {
            return Err(GameError::InvalidAction);
        }
        // Rebuild the exact-capacity destination, not unbounded Vec geometric
        // growth. Old backing remains admitted and is never refunded.
        let bytes = history_len
            .checked_add(1)
            .and_then(|n| n.checked_mul(std::mem::size_of::<CommittedRelicCorruptionSelection>()))
            .and_then(|n| n.checked_add(source_scene_id.0.len()))
            .and_then(|n| n.checked_add(next_scene_id.0.len()));
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            bytes,
            "relic_event_selection_append",
        )?;
        crate::authoring::control::charge(history_len as u64 + 1, "relic_event_selection_append")?;
        let mut history = Vec::with_capacity(history_len + 1);
        if let Some(prior) = updated.relic_corruption_selections.take() {
            history.extend(prior);
        }
        history.push(CommittedRelicCorruptionSelection {
            source_scene_id,
            copy_uuid,
        });
        updated.relic_corruption_selections = Some(history);
        updated.current_scene_id = next_scene_id.clone();
        if let Some(copy) = copy_uuid {
            self.state
                .relics
                .corrupt(copy, &self.game_data.relic_data)?;
        }
        let result = self.event_state_result(&updated)?;
        self.commit_event_session_required_run(node_id, updated)?;
        Ok(result)
    }

    fn relic_corruption_candidates(
        &self,
    ) -> Result<Vec<crate::game::behavior::RelicCorruptionCandidateDto>, GameError> {
        use crate::game::behavior::RelicCorruptionCandidateDto;
        let len = self.state.relics.copy_count();
        crate::game::battle::runtime_control::admit_retained_growth(
            0,
            len.checked_mul(std::mem::size_of::<RelicCorruptionCandidateDto>()),
            "relic_event_candidates",
        )?;
        crate::authoring::control::charge(len as u64, "relic_event_candidates")?;
        let mut candidates = Vec::with_capacity(len);
        for copy in self.state.relics.copies() {
            match self
                .state
                .relics
                .validate_corruption_target(copy.copy_uuid, &self.game_data.relic_data)
            {
                Ok(_) => (),
                Err(GameError::InvalidAction) => continue,
                Err(error) => return Err(error),
            }
            let corruption = self
                .game_data
                .relic_data
                .get_by_id(&copy.definition_id)
                .and_then(|definition| definition.corruption.as_ref())
                .ok_or(GameError::InvalidAction)?;
            crate::game::relic::allocation::admit_owned_projection(
                &(copy, &corruption.effects, &corruption.penalties),
                "relic_event_candidate_projection",
            )?;
            if !crate::authoring::control::is_active() {
                crate::game::resource::store::preflight_resource_clone(copy)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                crate::game::resource::store::preflight_resource_clone(&corruption.effects)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                crate::game::resource::store::preflight_resource_clone(&corruption.penalties)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            }
            candidates.push(RelicCorruptionCandidateDto {
                copy: crate::authoring::clone_shape::controlled_clone(copy)?,
                eligibility: PayloadEligibilityDto::enabled(),
                effects: crate::authoring::clone_shape::controlled_clone(&corruption.effects)?,
                penalties: crate::authoring::clone_shape::controlled_clone(&corruption.penalties)?,
            });
        }
        // RelicInventory's BTreeMap key equals the validated copy UUID.
        // Filtering preserves this canonical order without sort workspace.
        Ok(candidates)
    }

    pub(super) fn validate_relic_event_session_history(
        &self,
        session: &EventSessionState,
        relics: &crate::game::relic::RelicInventory,
    ) -> Result<(), GameError> {
        let Some(history) = &session.relic_corruption_selections else {
            return Ok(());
        };
        let invalid =
            || GameError::InvalidStaticData("relic Event selection history is inconsistent".into());
        let event = self
            .game_data
            .event_data
            .get_by_id(session.event_id.as_str())
            .ok_or_else(invalid)?;
        let n = history.len();
        let m = event.scenes.len();
        let work = n
            .checked_mul(n)
            .and_then(|a| m.checked_mul(m).and_then(|b| a.checked_add(b)))
            .and_then(|a| n.checked_mul(m).and_then(|b| a.checked_add(b)))
            .and_then(|a| a.checked_add(n))
            .and_then(|a| u64::try_from(a).ok())
            .ok_or_else(invalid)?;
        crate::authoring::control::charge(work, "relic_event_history_validation")?;
        if history.is_empty() || n > m {
            return Err(invalid());
        }
        for (index, receipt) in history.iter().enumerate() {
            if history[..index].iter().any(|prior| {
                prior.source_scene_id == receipt.source_scene_id
                    || receipt.copy_uuid.is_some() && prior.copy_uuid == receipt.copy_uuid
            }) {
                return Err(invalid());
            }
            if !event.scenes.iter().any(|scene| {
                scene.id == receipt.source_scene_id
                    && matches!(scene.next, EventSceneNext::RelicCorruption { .. })
            }) {
                return Err(invalid());
            }
            if receipt.copy_uuid.is_some_and(|id| id.is_nil()) {
                return Err(invalid());
            }
            // A confirmed historical copy can legitimately have been dismantled.
            // If still owned, it must retain the committed irreversible variant.
            if let Some(copy) = receipt.copy_uuid.and_then(|id| relics.get(id)) {
                if copy.variant != crate::game::relic::RelicVariant::Corrupt {
                    return Err(invalid());
                }
            }
        }
        // Published Event graphs are acyclic. Follow their actual single path
        // using the existing employee-check receipt, consuming selections in
        // order. No temporary shadow map or invented branch is needed.
        let mut current = &event.entry_scene_id;
        let mut consumed = 0usize;
        let mut check_seen = false;
        for _ in 0..=m {
            let scene = event
                .scenes
                .iter()
                .find(|scene| scene.id == *current)
                .ok_or_else(invalid)?;
            if *current == session.current_scene_id {
                if consumed != history.len() || session.committed_check.is_some() != check_seen {
                    return Err(invalid());
                }
                return Ok(());
            }
            current = match &scene.next {
                EventSceneNext::Scene { scene_id } => scene_id,
                EventSceneNext::RelicCorruption { next_scene_id } => {
                    let receipt = history.get(consumed).ok_or_else(invalid)?;
                    if receipt.source_scene_id != *current {
                        return Err(invalid());
                    }
                    consumed += 1;
                    next_scene_id
                }
                EventSceneNext::EmployeeCheckChoices { choices } => {
                    let check = session.committed_check.as_ref().ok_or_else(invalid)?;
                    if check_seen || check.source_scene_id != *current {
                        return Err(invalid());
                    }
                    let choice = choices
                        .iter()
                        .find(|choice| choice.id == check.choice_id)
                        .ok_or_else(invalid)?;
                    // New history branches may rely on the old roll receipt.
                    // Check its internal arithmetic against the authored check;
                    // do not re-evaluate employees/equipment after later changes.
                    crate::authoring::control::charge(
                        check.modifier_components.len() as u64,
                        "relic_event_check_receipt",
                    )?;
                    if !(1..=20).contains(&check.raw_roll)
                        || check.employee_uuid.is_nil()
                        || check.aptitude != choice.check.aptitude
                        || check.dc != choice.check.dc
                        || check.modifier_components.len() > 2
                    {
                        return Err(invalid());
                    }
                    let mut modifier_total = 0u8;
                    let mut aptitude_seen = false;
                    let mut equipment_seen = false;
                    for component in &check.modifier_components {
                        let amount = match component {
                            EventCheckModifierComponent::EmployeeAptitude { aptitude, amount } => {
                                if aptitude_seen
                                    || equipment_seen
                                    || *aptitude != check.aptitude
                                    || *amount != 3
                                {
                                    return Err(invalid());
                                }
                                aptitude_seen = true;
                                *amount
                            }
                            EventCheckModifierComponent::Equipment {
                                equipment_instance_uuid,
                                equipment_definition_id,
                                aptitude,
                                amount,
                            } => {
                                if equipment_seen
                                    || equipment_instance_uuid.is_nil()
                                    || *aptitude != check.aptitude
                                    || !matches!(*amount, 1 | 2)
                                    || equipment_definition_id.is_empty()
                                {
                                    return Err(invalid());
                                }
                                equipment_seen = true;
                                *amount
                            }
                        };
                        modifier_total = modifier_total.checked_add(amount).ok_or_else(invalid)?;
                    }
                    let total_roll = check
                        .raw_roll
                        .checked_add(modifier_total)
                        .ok_or_else(invalid)?;
                    if check.modifier_total != modifier_total
                        || check.total_roll != total_roll
                        || check.outcome
                            != classify_event_check(check.raw_roll, total_roll, choice.check.dc)
                    {
                        return Err(invalid());
                    }
                    check_seen = true;
                    if matches!(
                        check.outcome,
                        EventCheckOutcome::Success | EventCheckOutcome::CriticalSuccess
                    ) {
                        &choice.success_scene_id
                    } else {
                        &choice.failure_scene_id
                    }
                }
                EventSceneNext::End => return Err(invalid()),
            };
        }
        Err(invalid())
    }

    fn current_event_session(&self) -> Result<&EventSessionState, GameError> {
        self.state
            .active_node_content
            .as_ref()
            .ok_or(GameError::InvalidAction)?
            .as_event()
    }

    fn event_definition(&self, event_id: &EventId) -> Result<&EventDefinition, GameError> {
        self.game_data
            .event_data
            .get_by_id(event_id.as_str())
            .ok_or(GameError::MissingResource("Event"))
    }

    fn event_state_result(&self, session: &EventSessionState) -> Result<BehaviorResult, GameError> {
        Ok(BehaviorResult::EventState {
            node_id: session.node_id,
            event: self.event_scene_snapshot(session)?,
        })
    }

    pub(super) fn event_scene_snapshot(
        &self,
        session: &EventSessionState,
    ) -> Result<EventSceneSnapshotDto, GameError> {
        let event = self.event_definition(&session.event_id)?;
        let scene = event_scene(event, session.current_scene_id.as_str())?;
        if session.has_relic_history()
            || matches!(scene.next, EventSceneNext::RelicCorruption { .. })
        {
            self.ensure_relic_operation()?;
        }
        let interaction = match &scene.next {
            EventSceneNext::RelicCorruption { .. } => EventInteractionDto::RelicCorruption {
                candidates: self.relic_corruption_candidates()?,
                can_decline: true,
                catalyst_cost: 0,
            },
            EventSceneNext::EmployeeCheckChoices { choices } => {
                EventInteractionDto::EmployeeCheckChoices {
                    choices: choices
                        .iter()
                        .map(|choice| {
                            let employee_candidates = self.event_employee_candidates(choice)?;
                            let has_eligible_employee = employee_candidates
                                .iter()
                                .any(|candidate| candidate.eligibility.enabled);
                            Ok(EventChoiceEvaluationDto {
                                choice_id: choice.id.clone(),
                                label_id: choice.label_id.clone(),
                                aptitude: choice.check.aptitude,
                                dc: choice.check.dc,
                                required_choice_key: choice.check.required_choice_key.clone(),
                                eligibility: if has_eligible_employee {
                                    PayloadEligibilityDto::enabled()
                                } else {
                                    PayloadEligibilityDto::disabled(
                                        ActionDisabledReasonCode::NoEligiblePayload,
                                    )
                                },
                                employee_candidates,
                            })
                        })
                        .collect::<Result<Vec<_>, GameError>>()?,
                }
            }
            EventSceneNext::Scene { .. } | EventSceneNext::End => EventInteractionDto::Advance,
        };
        Ok(EventSceneSnapshotDto {
            event_id: session.event_id.clone(),
            current_scene_id: session.current_scene_id.clone(),
            presentation: scene.presentation.clone(),
            committed_check: session.committed_check.as_ref().map(|committed| {
                EventCommittedCheckDto {
                    source_scene_id: committed.source_scene_id.clone(),
                    choice_id: committed.choice_id.clone(),
                    employee_uuid: committed.employee_uuid,
                    aptitude: committed.aptitude,
                    dc: committed.dc,
                    raw_roll: committed.raw_roll,
                    modifier_components: committed.modifier_components.clone(),
                    choice_key_evidence: committed.choice_key_evidence.clone(),
                    modifier_total: committed.modifier_total,
                    total_roll: committed.total_roll,
                    outcome: committed.outcome,
                }
            }),
            interaction,
            relic_corruption_selections: session.history_snapshot()?,
        })
    }

    fn event_employee_candidates(
        &self,
        choice: &EventChoiceDefinition,
    ) -> Result<Vec<EventEmployeeEvaluationDto>, GameError> {
        let mut candidates = self
            .state
            .roster
            .iter()
            .map(|employee| {
                let evaluation = self.evaluate_event_choice_for_employee(choice, employee.uuid)?;
                Ok(EventEmployeeEvaluationDto {
                    employee_uuid: employee.uuid,
                    name: employee.name.clone(),
                    eligibility: match evaluation.disabled_reason {
                        None => PayloadEligibilityDto::enabled(),
                        Some(reason) => PayloadEligibilityDto::disabled(reason),
                    },
                    employee_modifier: evaluation.employee_modifier,
                    equipment_modifier: evaluation.equipment_modifier,
                    modifier_total: evaluation.modifier_total,
                    modifier_components: event_modifier_components(
                        choice.check.aptitude,
                        &evaluation,
                    ),
                    choice_key_evidence: evaluation.choice_key_evidence,
                    success_chance_percent: evaluation.success_chance_percent,
                })
            })
            .collect::<Result<Vec<_>, GameError>>()?;
        candidates.sort_by_key(|candidate| candidate.employee_uuid.as_u128());
        Ok(candidates)
    }

    pub(crate) fn evaluate_event_choice_for_employee(
        &self,
        choice: &EventChoiceDefinition,
        employee_uuid: Uuid,
    ) -> Result<EventEmployeeCheckEvaluation, GameError> {
        let employee = self
            .state
            .roster
            .get(&employee_uuid)
            .ok_or(GameError::InvalidAction)?;
        evaluate_event_choice_for_employee(choice, employee, &self.state.inventory)
    }

    fn roll_event_check(&self, node_id: MapNodeId) -> u8 {
        let mut rng =
            rand::rngs::StdRng::seed_from_u64(self.node_seed(node_id, 0x4556_454E_545F_4432));
        rng.gen_range(1..=20)
    }

    fn plan_event_grant(
        &self,
        node_id: MapNodeId,
        effects: &[RewardEffect],
    ) -> Result<EconomyGrantState, GameError> {
        self.ensure_relic_grant(effects)?;
        let mut planned = EconomyGrantState::capture(&self.state)?;
        planned.grant_effects(
            self.game_data.as_ref(),
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            effects,
            self.node_seed(node_id, 0x4556_5447_5241_4E54),
        )?;
        Ok(planned)
    }

    fn commit_event_grant(&mut self, planned: EconomyGrantState) {
        planned.commit_into(&mut self.state);
    }

    fn commit_event_session_required_run(
        &mut self,
        node_id: MapNodeId,
        session: EventSessionState,
    ) -> Result<(), GameError> {
        if session.has_relic_history() && !self.run_state()?.event_sessions.contains_key(&node_id) {
            // A selection only replaces the key inserted by enter_event_node.
            // Keep this new receipt scope independent of existing map backing.
            return Err(crate::game::battle::runtime_control::invariant(
                "relic_event_existing_session",
            ));
        }
        let run_copy = session.clone_for_relic_transition()?;
        self.run_state_mut()?
            .event_sessions
            .insert(node_id, run_copy);
        self.state.active_node_content = Some(ActiveNodeContent::Event(session));
        Ok(())
    }
}

fn evaluate_event_choice_for_employee(
    choice: &EventChoiceDefinition,
    employee: &Employee,
    inventory: &Inventory,
) -> Result<EventEmployeeCheckEvaluation, GameError> {
    let employee_modifier = u8::from(employee.profile.event_aptitude == choice.check.aptitude) * 3;
    let mut applicable_modifiers = Vec::new();
    let mut key_providers = Vec::new();

    for equipped in employee.loadout.item_slot.iter() {
        let owned = inventory
            .equipments
            .get_item(&equipped.instance_uuid)
            .ok_or_else(|| {
                GameError::InvalidStaticData(format!(
                    "employee '{}' Event loadout references missing owned equipment '{}'",
                    employee.uuid, equipped.instance_uuid
                ))
            })?;
        if owned.equipped_to != Some(employee.uuid)
            || owned.meta.uuid != equipped.base_uuid
            || owned.instance_uuid != equipped.instance_uuid
        {
            return Err(GameError::InvalidStaticData(format!(
                "employee '{}' Event loadout ownership disagrees for equipment '{}'",
                employee.uuid, equipped.instance_uuid
            )));
        }

        if let Some(modifier) = owned
            .meta
            .event_check_modifiers
            .iter()
            .find(|modifier| modifier.aptitude == choice.check.aptitude)
        {
            applicable_modifiers.push(EventEquipmentModifierEvidence {
                equipment_instance_uuid: owned.instance_uuid,
                equipment_definition_id: owned.meta.id.clone(),
                aptitude: modifier.aptitude,
                amount: modifier.amount,
            });
        }

        if let Some(required_key) = &choice.check.required_choice_key {
            if owned
                .meta
                .event_choice_keys
                .iter()
                .any(|key| key == required_key)
            {
                key_providers.push(EventChoiceKeyEvidence {
                    key_id: required_key.clone(),
                    equipment_instance_uuid: owned.instance_uuid,
                    equipment_definition_id: owned.meta.id.clone(),
                });
            }
        }
    }

    applicable_modifiers.sort_by(|left, right| {
        right.amount.cmp(&left.amount).then_with(|| {
            left.equipment_instance_uuid
                .cmp(&right.equipment_instance_uuid)
        })
    });
    key_providers.sort_by_key(|evidence| evidence.equipment_instance_uuid);
    let equipment_modifier_evidence = applicable_modifiers.into_iter().next();
    let choice_key_evidence = key_providers.into_iter().next();
    let equipment_modifier = equipment_modifier_evidence
        .as_ref()
        .map_or(0, |evidence| evidence.amount);
    let modifier_total = employee_modifier + equipment_modifier;
    let success_chance_percent =
        event_check_success_chance_percent(choice.check.dc, modifier_total);

    let disabled_reason = employee
        .eligibility(EmployeeActivity::Event)
        .disabled_reason
        .map(|reason| reason.action_disabled_reason())
        .or_else(|| {
            (choice.check.required_choice_key.is_some() && choice_key_evidence.is_none())
                .then_some(ActionDisabledReasonCode::MissingEventChoiceKey)
        });

    Ok(EventEmployeeCheckEvaluation {
        employee_modifier,
        equipment_modifier,
        modifier_total,
        equipment_modifier_evidence,
        choice_key_evidence,
        success_chance_percent,
        disabled_reason,
    })
}

fn event_modifier_components(
    aptitude: crate::game::employee::EventAptitude,
    evaluation: &EventEmployeeCheckEvaluation,
) -> Vec<EventCheckModifierComponent> {
    let mut components = Vec::new();
    if evaluation.employee_modifier > 0 {
        components.push(EventCheckModifierComponent::EmployeeAptitude {
            aptitude,
            amount: evaluation.employee_modifier,
        });
    }
    if let Some(evidence) = &evaluation.equipment_modifier_evidence {
        components.push(EventCheckModifierComponent::Equipment {
            equipment_instance_uuid: evidence.equipment_instance_uuid,
            equipment_definition_id: evidence.equipment_definition_id.clone(),
            aptitude: evidence.aptitude,
            amount: evidence.amount,
        });
    }
    components
}

pub(crate) fn classify_event_check(raw_roll: u8, total_roll: u8, dc: u8) -> EventCheckOutcome {
    match raw_roll {
        1 => EventCheckOutcome::CriticalFailure,
        20 => EventCheckOutcome::CriticalSuccess,
        _ if total_roll >= dc => EventCheckOutcome::Success,
        _ => EventCheckOutcome::Failure,
    }
}

pub(crate) fn event_check_success_chance_percent(dc: u8, modifier_total: u8) -> u8 {
    let successful_faces = (1..=20)
        .filter(|raw_roll| {
            matches!(
                classify_event_check(*raw_roll, *raw_roll + modifier_total, dc),
                EventCheckOutcome::Success | EventCheckOutcome::CriticalSuccess
            )
        })
        .count() as u8;
    successful_faces * 5
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::game::{
        data::{
            equipment_data::{EquipmentMetadata, EquipmentType, EventCheckModifierDefinition},
            event_data::{EventChoiceCheckDefinition, EventChoiceKeyId, EventSceneId},
        },
        employee::EventAptitude,
        enums::RiskLevel,
        resources::{item_slot::EquippedRef, OwnedEquipment},
    };

    fn checked_choice(required_key: Option<&str>) -> EventChoiceDefinition {
        EventChoiceDefinition {
            id: EventChoiceId::new("inspect_archive"),
            label_id: "inspect_archive_label".to_string(),
            check: EventChoiceCheckDefinition {
                aptitude: EventAptitude::Analysis,
                dc: 16,
                required_choice_key: required_key.map(EventChoiceKeyId::new),
            },
            success_action: None,
            success_scene_id: EventSceneId::new("success"),
            failure_scene_id: EventSceneId::new("failure"),
        }
    }

    fn event_tool(
        definition_uuid: u128,
        definition_id: &str,
        amount: u8,
        key: Option<&str>,
    ) -> EquipmentMetadata {
        EquipmentMetadata {
            content_family: None,
            id: definition_id.to_string(),
            uuid: Uuid::from_u128(definition_uuid),
            name: definition_id.to_string(),
            equipment_type: EquipmentType::Accessory,
            rarity: RiskLevel::ZAYIN,
            price: 0,
            allow_duplicate_equip: true,
            bound: false,
            cannot_unequip_reason: "equipment_bound".to_string(),
            triggered_effects: Default::default(),
            ability_activations: vec![],
            weapon_profile: None,
            event_check_modifiers: vec![EventCheckModifierDefinition {
                aptitude: EventAptitude::Analysis,
                amount,
            }],
            event_choice_keys: key
                .map(|key| vec![EventChoiceKeyId::new(key)])
                .unwrap_or_default(),
        }
    }

    fn equip_event_tool(
        employee: &mut Employee,
        inventory: &mut Inventory,
        instance_uuid: u128,
        metadata: EquipmentMetadata,
    ) {
        let instance_uuid = Uuid::from_u128(instance_uuid);
        let equipped = EquippedRef {
            instance_uuid,
            base_uuid: metadata.uuid,
            equipment_type: metadata.equipment_type,
        };
        let mut owned = OwnedEquipment::new(instance_uuid, Arc::new(metadata));
        owned.equipped_to = Some(employee.uuid);
        inventory.equipments.add_item(owned).unwrap();
        employee.loadout.item_slot.equip(equipped, true).unwrap();
    }

    #[test]
    fn event_check_outcomes_pin_natural_extremes_and_modified_threshold() {
        assert_eq!(
            classify_event_check(1, 6, 8),
            EventCheckOutcome::CriticalFailure
        );
        assert_eq!(classify_event_check(8, 11, 12), EventCheckOutcome::Failure);
        assert_eq!(classify_event_check(9, 12, 12), EventCheckOutcome::Success);
        assert_eq!(
            classify_event_check(20, 20, 20),
            EventCheckOutcome::CriticalSuccess
        );
    }

    #[test]
    fn success_chance_enumerates_all_allowed_dc_and_modifier_boundaries() {
        let expected = [
            (8, [65, 70, 75, 80, 85, 90]),
            (12, [45, 50, 55, 60, 65, 70]),
            (16, [25, 30, 35, 40, 45, 50]),
            (20, [5, 10, 15, 20, 25, 30]),
        ];
        for (dc, chances) in expected {
            for (modifier, expected_chance) in chances.into_iter().enumerate() {
                assert_eq!(
                    event_check_success_chance_percent(dc, modifier as u8),
                    expected_chance,
                    "unexpected chance for DC {dc}, modifier {modifier}"
                );
            }
        }
    }

    #[test]
    fn outcome_matrix_exhausts_every_face_dc_and_modifier_boundary() {
        for dc in [8, 12, 16, 20] {
            for modifier_total in 0..=5 {
                for raw_roll in 1..=20 {
                    let expected = match raw_roll {
                        1 => EventCheckOutcome::CriticalFailure,
                        20 => EventCheckOutcome::CriticalSuccess,
                        _ if raw_roll + modifier_total >= dc => EventCheckOutcome::Success,
                        _ => EventCheckOutcome::Failure,
                    };
                    assert_eq!(
                        classify_event_check(raw_roll, raw_roll + modifier_total, dc),
                        expected,
                        "unexpected outcome for raw {raw_roll}, modifier {modifier_total}, DC {dc}"
                    );
                }
            }
        }
    }

    #[test]
    fn evaluator_uses_stored_aptitude_highest_only_uuid_ties_and_owned_equipped_key() {
        let mut employee = Employee::new(Uuid::from_u128(0xE1), "analyst", "Analyst");
        employee.profile.event_aptitude = EventAptitude::Analysis;
        employee
            .loadout
            .item_slot
            .switch_layout(crate::game::resources::item_slot::SlotLayoutKind::Any3)
            .unwrap();
        let mut inventory = Inventory::new();
        equip_event_tool(
            &mut employee,
            &mut inventory,
            30,
            event_tool(300, "weak_tool", 1, None),
        );
        equip_event_tool(
            &mut employee,
            &mut inventory,
            20,
            event_tool(200, "later_tied_tool", 2, Some("sealed_archive_access")),
        );
        equip_event_tool(
            &mut employee,
            &mut inventory,
            10,
            event_tool(100, "first_tied_tool", 2, Some("sealed_archive_access")),
        );
        let inventory_only = OwnedEquipment::new(
            Uuid::from_u128(1),
            Arc::new(event_tool(
                1,
                "inventory_only",
                2,
                Some("sealed_archive_access"),
            )),
        );
        inventory.equipments.add_item(inventory_only).unwrap();

        let evaluation = evaluate_event_choice_for_employee(
            &checked_choice(Some("sealed_archive_access")),
            &employee,
            &inventory,
        )
        .unwrap();

        assert!(evaluation.eligible());
        assert_eq!(evaluation.employee_modifier, 3);
        assert_eq!(evaluation.equipment_modifier, 2);
        assert_eq!(evaluation.modifier_total, 5);
        assert_eq!(evaluation.success_chance_percent, 50);
        assert_eq!(
            evaluation
                .equipment_modifier_evidence
                .as_ref()
                .unwrap()
                .equipment_instance_uuid,
            Uuid::from_u128(10)
        );
        assert_eq!(
            evaluation
                .choice_key_evidence
                .as_ref()
                .unwrap()
                .equipment_instance_uuid,
            Uuid::from_u128(10)
        );
        assert_eq!(
            inventory.equipments.len(),
            4,
            "evaluation must not consume a key"
        );
    }

    #[test]
    fn condition_reason_precedes_missing_key_and_mismatch_is_zero_not_ineligible() {
        let mut employee = Employee::new(Uuid::from_u128(0xE2), "medic", "Medic");
        employee.profile.event_aptitude = EventAptitude::Medical;
        employee.condition.physical_damage = crate::game::employee::MAX_PHYSICAL_DAMAGE - 1;
        let mut inventory = Inventory::new();
        let choice = checked_choice(Some("sealed_archive_access"));

        let condition_blocked =
            evaluate_event_choice_for_employee(&choice, &employee, &inventory).unwrap();
        assert_eq!(condition_blocked.employee_modifier, 0);
        assert_eq!(
            condition_blocked.disabled_reason,
            Some(ActionDisabledReasonCode::CriticalPhysicalDamage)
        );

        employee.condition.physical_damage = 0;
        let key_blocked =
            evaluate_event_choice_for_employee(&choice, &employee, &inventory).unwrap();
        assert_eq!(
            key_blocked.disabled_reason,
            Some(ActionDisabledReasonCode::MissingEventChoiceKey)
        );

        let mut key_only = event_tool(400, "key_only_tool", 1, Some("sealed_archive_access"));
        key_only.event_check_modifiers.clear();
        equip_event_tool(&mut employee, &mut inventory, 40, key_only);
        let key_enabled =
            evaluate_event_choice_for_employee(&choice, &employee, &inventory).unwrap();
        assert!(key_enabled.eligible());
        assert_eq!(key_enabled.employee_modifier, 0);
        assert_eq!(key_enabled.equipment_modifier, 0);
        assert_eq!(key_enabled.modifier_total, 0);
        assert_eq!(key_enabled.success_chance_percent, 25);
    }
}
