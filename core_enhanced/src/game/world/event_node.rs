use rand::{Rng, SeedableRng};
use uuid::Uuid;

use super::GameCore;
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
    employee::{Employee, EmployeeActivity, EmployeeRoster},
    managers::uuid_manager::UuidManager,
    map::MapNodeId,
    resources::{
        ActiveNodeContent, Enkephalin, EventCheckModifierComponent, EventChoiceKeyEvidence,
        EventCommittedCheckState, EventEquipmentModifierEvidence, EventSessionState,
        EventStartedCombatState, Inventory,
    },
    reward::{GrantExecutionContext, GrantExecutor, RewardEffect},
    skill_fragment::SkillFragmentInventory,
};

struct PlannedEventGrant {
    inventory: Inventory,
    skill_fragments: SkillFragmentInventory,
    roster: EmployeeRoster,
    uuid_manager: UuidManager,
    enkephalin: Enkephalin,
}

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
        let result = self.event_state_result(&session)?;
        self.run_state_mut()?
            .event_sessions
            .insert(node_id, session.clone());
        self.state.active_node_content = Some(ActiveNodeContent::Event(session));
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
                let mut updated = session;
                updated.current_scene_id = scene_id.clone();
                self.commit_event_session(node_id, updated.clone())?;
                self.event_state_result(&updated)
            }
            EventSceneNext::End => {
                let completion = self.plan_complete_current_node(true)?;
                self.commit_event_session(node_id, session)?;
                self.commit_interactive_node_completion(completion)
            }
            EventSceneNext::EmployeeCheckChoices { .. } => Err(GameError::InvalidAction),
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
        let session = self.current_event_session()?.clone();
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
            self.commit_event_session(node_id, updated)?;
            return self.commit_planned_live_battle_start(node_id, planned);
        }

        self.commit_event_session(node_id, updated.clone())?;
        self.event_state_result(&updated)
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
        let interaction = match &scene.next {
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
    ) -> Result<PlannedEventGrant, GameError> {
        let mut planned = PlannedEventGrant {
            inventory: self.state.inventory.clone(),
            skill_fragments: self.state.skill_fragments.clone(),
            roster: self.state.roster.clone(),
            uuid_manager: self.state.uuid_manager.clone(),
            enkephalin: self.state.enkephalin.clone(),
        };
        GrantExecutor::grant_effects_with_state(
            &mut planned.inventory,
            &mut planned.skill_fragments,
            &mut planned.roster,
            &mut planned.uuid_manager,
            &mut planned.enkephalin,
            self.game_data.as_ref(),
            &self.state.skill_fragment_policy,
            &GrantExecutionContext::default(),
            effects,
            self.node_seed(node_id, 0x4556_5447_5241_4E54),
        )?;
        Ok(planned)
    }

    fn commit_event_grant(&mut self, planned: PlannedEventGrant) {
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
