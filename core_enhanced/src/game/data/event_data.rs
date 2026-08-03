use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::{
    data::{build_string_index, once_lock_with},
    employee::EventAptitude,
    reward::RewardEffect,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(pub String);

impl EventId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventSceneId(pub String);

impl EventSceneId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventChoiceId(pub String);

impl EventChoiceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventChoiceKeyId(pub String);

impl EventChoiceKeyId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_valid(&self) -> bool {
        let value = self.as_str();
        !value.is_empty()
            && !value.starts_with('_')
            && !value.ends_with('_')
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
            && !value.as_bytes().windows(2).any(|window| window == b"__")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDefinition {
    pub id: EventId,
    pub entry_scene_id: EventSceneId,
    pub scenes: Vec<EventSceneDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventSceneDefinition {
    pub id: EventSceneId,
    pub presentation: EventScenePresentation,
    pub next: EventSceneNext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventScenePresentation {
    pub panels: Vec<EventPresentationPanel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventPresentationPanel {
    pub image_id: String,
    pub text_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventSceneNext {
    Scene { scene_id: EventSceneId },
    EmployeeCheckChoices { choices: Vec<EventChoiceDefinition> },
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoiceDefinition {
    pub id: EventChoiceId,
    pub label_id: String,
    pub check: EventChoiceCheckDefinition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_action: Option<EventSuccessAction>,
    pub success_scene_id: EventSceneId,
    pub failure_scene_id: EventSceneId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoiceCheckDefinition {
    pub aptitude: EventAptitude,
    pub dc: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_choice_key: Option<EventChoiceKeyId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventSuccessAction {
    Grant {
        effects: Vec<RewardEffect>,
    },
    StartCombat {
        encounter_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        primary_abnormality_id: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCheckOutcome {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDatabase {
    pub events: Vec<EventDefinition>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl EventDatabase {
    pub fn new(events: Vec<EventDefinition>) -> Self {
        let by_id = once_lock_with(build_string_index(&events, "event id", |event| {
            event.id.as_str()
        }));
        Self { events, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id
            .get_or_init(|| build_string_index(&self.events, "event id", |event| event.id.as_str()))
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (index, event) in self.events.iter().enumerate() {
            validate_event_definition(event, index)?;
        }
        Ok(())
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .unwrap_or_else(|error| panic!("invalid Event data: {error:?}"));
        let _ = self.by_id();
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EventDefinition> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.events.get(index))
    }
}

fn event_error(
    event: &EventDefinition,
    path: impl Into<String>,
    cause: impl Into<String>,
) -> crate::game::data::DataValidationError {
    crate::game::data::DataValidationError::at(event.id.as_str(), path, cause)
}

fn validate_event_definition(
    event: &EventDefinition,
    event_index: usize,
) -> Result<(), crate::game::data::DataValidationError> {
    let event_path = format!("events[{event_index}]");
    if event.id.as_str().trim().is_empty() {
        return Err(event_error(
            event,
            format!("{event_path}.id"),
            "Event id must not be empty",
        ));
    }
    if event.entry_scene_id.as_str().trim().is_empty() {
        return Err(event_error(
            event,
            format!("{event_path}.entry_scene_id"),
            "entry_scene_id must not be empty",
        ));
    }
    if event.scenes.is_empty() {
        return Err(event_error(
            event,
            format!("{event_path}.scenes"),
            "Event must define at least one scene",
        ));
    }

    let mut seen_scene_ids = HashSet::new();
    let mut scene_by_id = HashMap::new();
    for (scene_index, scene) in event.scenes.iter().enumerate() {
        let scene_path = format!("{event_path}.scenes[{scene_index}]");
        if scene.id.as_str().trim().is_empty() {
            return Err(event_error(
                event,
                format!("{scene_path}.id"),
                "scene id must not be empty",
            ));
        }
        if !seen_scene_ids.insert(scene.id.as_str()) {
            return Err(event_error(
                event,
                format!("{scene_path}.id"),
                "duplicate scene id",
            ));
        }
        validate_presentation(event, scene, &scene_path)?;
        validate_scene_next(event, scene, &scene_path)?;
        scene_by_id.insert(scene.id.as_str(), scene_index);
    }

    if !scene_by_id.contains_key(event.entry_scene_id.as_str()) {
        return Err(event_error(
            event,
            format!("{event_path}.entry_scene_id"),
            format!(
                "references missing scene '{}'",
                event.entry_scene_id.as_str()
            ),
        ));
    }

    for (scene_index, scene) in event.scenes.iter().enumerate() {
        for target in scene_targets(scene) {
            if !scene_by_id.contains_key(target.as_str()) {
                return Err(event_error(
                    event,
                    format!("{event_path}.scenes[{scene_index}].next"),
                    format!("references missing scene '{}'", target.as_str()),
                ));
            }
        }
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut has_terminal_path = false;
    visit_scene(
        event,
        event.entry_scene_id.as_str(),
        &scene_by_id,
        &mut visiting,
        &mut visited,
        &mut has_terminal_path,
        &event_path,
    )?;

    if !has_terminal_path {
        return Err(event_error(
            event,
            format!("{event_path}.scenes"),
            "Event must have at least one terminal path",
        ));
    }
    for (scene_index, scene) in event.scenes.iter().enumerate() {
        if !visited.contains(scene.id.as_str()) {
            return Err(event_error(
                event,
                format!("{event_path}.scenes[{scene_index}]"),
                "unreachable scene",
            ));
        }
    }
    let reachable_check_scene_count = event
        .scenes
        .iter()
        .filter(|scene| {
            visited.contains(scene.id.as_str())
                && matches!(scene.next, EventSceneNext::EmployeeCheckChoices { .. })
        })
        .count();
    if reachable_check_scene_count > 1 {
        return Err(event_error(
            event,
            format!("{event_path}.scenes"),
            "Event must contain at most one reachable EmployeeCheckChoices scene",
        ));
    }
    Ok(())
}

fn validate_presentation(
    event: &EventDefinition,
    scene: &EventSceneDefinition,
    scene_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if scene.presentation.panels.is_empty() {
        return Err(event_error(
            event,
            format!("{scene_path}.presentation.panels"),
            "panels must not be empty",
        ));
    }
    for (panel_index, panel) in scene.presentation.panels.iter().enumerate() {
        let panel_path = format!("{scene_path}.presentation.panels[{panel_index}]");
        if panel.image_id.trim().is_empty() {
            return Err(event_error(
                event,
                format!("{panel_path}.image_id"),
                "image_id must not be empty",
            ));
        }
        if panel.text_ids.is_empty() {
            return Err(event_error(
                event,
                format!("{panel_path}.text_ids"),
                "text_ids must not be empty",
            ));
        }
        for (text_index, text_id) in panel.text_ids.iter().enumerate() {
            if text_id.trim().is_empty() {
                return Err(event_error(
                    event,
                    format!("{panel_path}.text_ids[{text_index}]"),
                    "text id must not be empty",
                ));
            }
        }
    }
    Ok(())
}

fn validate_scene_next(
    event: &EventDefinition,
    scene: &EventSceneDefinition,
    scene_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    match &scene.next {
        EventSceneNext::Scene { scene_id } => {
            if scene_id.as_str().trim().is_empty() {
                return Err(event_error(
                    event,
                    format!("{scene_path}.next.Scene.scene_id"),
                    "next scene_id must not be empty",
                ));
            }
        }
        EventSceneNext::EmployeeCheckChoices { choices } => {
            if choices.is_empty() {
                return Err(event_error(
                    event,
                    format!("{scene_path}.next.EmployeeCheckChoices.choices"),
                    "EmployeeCheckChoices must not be empty",
                ));
            }
            let mut seen_choice_ids = HashSet::new();
            for (choice_index, choice) in choices.iter().enumerate() {
                let choice_path =
                    format!("{scene_path}.next.EmployeeCheckChoices.choices[{choice_index}]");
                if choice.id.as_str().trim().is_empty() {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.id"),
                        "choice id must not be empty",
                    ));
                }
                if !seen_choice_ids.insert(choice.id.as_str()) {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.id"),
                        "duplicate choice id",
                    ));
                }
                if choice.label_id.trim().is_empty() {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.label_id"),
                        "label_id must not be empty",
                    ));
                }
                if !matches!(choice.check.dc, 8 | 12 | 16 | 20) {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.check.dc"),
                        "check dc must be one of 8, 12, 16, or 20",
                    ));
                }
                if let Some(required_choice_key) = &choice.check.required_choice_key {
                    if !required_choice_key.is_valid() {
                        return Err(event_error(
                            event,
                            format!("{choice_path}.check.required_choice_key"),
                            "required_choice_key must be a non-empty lower_snake_case identifier",
                        ));
                    }
                }
                if choice.success_scene_id.as_str().trim().is_empty() {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.success_scene_id"),
                        "success_scene_id must not be empty",
                    ));
                }
                if choice.failure_scene_id.as_str().trim().is_empty() {
                    return Err(event_error(
                        event,
                        format!("{choice_path}.failure_scene_id"),
                        "failure_scene_id must not be empty",
                    ));
                }
                if let Some(EventSuccessAction::Grant { effects }) = &choice.success_action {
                    if effects.is_empty() {
                        return Err(event_error(
                            event,
                            format!("{choice_path}.success_action.Grant.effects"),
                            "Grant effects must not be empty",
                        ));
                    }
                }
            }
        }
        EventSceneNext::End => {}
    }
    Ok(())
}

fn scene_targets(scene: &EventSceneDefinition) -> Vec<&EventSceneId> {
    match &scene.next {
        EventSceneNext::Scene { scene_id } => vec![scene_id],
        EventSceneNext::EmployeeCheckChoices { choices, .. } => choices
            .iter()
            .flat_map(|choice| [&choice.success_scene_id, &choice.failure_scene_id])
            .collect(),
        EventSceneNext::End => Vec::new(),
    }
}

fn scene_is_terminal(scene: &EventSceneDefinition) -> bool {
    match &scene.next {
        EventSceneNext::End => true,
        EventSceneNext::EmployeeCheckChoices { .. } => false,
        EventSceneNext::Scene { .. } => false,
    }
}

fn visit_scene(
    event: &EventDefinition,
    scene_id: &str,
    scene_by_id: &HashMap<&str, usize>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    has_terminal_path: &mut bool,
    event_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if visited.contains(scene_id) {
        return Ok(());
    }
    if !visiting.insert(scene_id.to_string()) {
        return Err(event_error(
            event,
            format!("{event_path}.scenes"),
            format!("scene graph contains a cycle at scene '{scene_id}'"),
        ));
    }
    let scene = &event.scenes[*scene_by_id
        .get(scene_id)
        .expect("scene references are validated before DFS")];
    if scene_is_terminal(scene) {
        *has_terminal_path = true;
    }
    for target in scene_targets(scene) {
        visit_scene(
            event,
            target.as_str(),
            scene_by_id,
            visiting,
            visited,
            has_terminal_path,
            event_path,
        )?;
    }
    visiting.remove(scene_id);
    visited.insert(scene_id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strict_employee_check_database() -> EventDatabase {
        ron::de::from_str::<EventDatabase>(
            r#"
            EventDatabase(events: [(
                id: "event_a",
                entry_scene_id: "check",
                scenes: [
                    (
                        id: "check",
                        presentation: (panels: [(image_id: "image", text_ids: ["text"])]),
                        next: EmployeeCheckChoices(choices: [(
                            id: "choice_a",
                            label_id: "label",
                            check: (aptitude: Analysis, dc: 12),
                            success_scene_id: "success",
                            failure_scene_id: "failure",
                        )]),
                    ),
                    (
                        id: "success",
                        presentation: (panels: [(image_id: "success", text_ids: ["ok"])]),
                        next: End,
                    ),
                    (
                        id: "failure",
                        presentation: (panels: [(image_id: "failure", text_ids: ["no"])]),
                        next: End,
                    ),
                ],
            )])
            "#,
        )
        .expect("strict event check fixture should deserialize")
    }

    #[test]
    fn event_database_rejects_removed_flat_presentation_fields() {
        let err = ron::de::from_str::<EventDatabase>(
            r#"
            EventDatabase(
                events: [
                    (
                        id: "event_a",
                        entry_scene_id: "scene_a",
                        scenes: [
                            (
                                id: "scene_a",
                                presentation: (
                                    background_id: "bg",
                                    script_ids: ["script"],
                                ),
                                next: End,
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .expect_err("removed flat presentation fields should fail");

        assert!(
            err.to_string().contains("background_id"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn event_database_accepts_strict_employee_check_shape() {
        let database = strict_employee_check_database();

        database
            .validate_contract()
            .expect("new event shape should validate");
    }

    #[test]
    fn event_database_rejects_legacy_scene_required_roll() {
        let error = ron::de::from_str::<EventDatabase>(
            r#"EventDatabase(events: [(
                id: "event_a", entry_scene_id: "check", scenes: [(
                    id: "check",
                    presentation: (panels: [(image_id: "image", text_ids: ["text"])]),
                    next: EmployeeCheckChoices(required_roll: 12, choices: []),
                )],
            )])"#,
        )
        .expect_err("legacy scene-level required_roll must fail closed");

        assert!(error.to_string().contains("required_roll"));
    }

    #[test]
    fn event_database_rejects_non_policy_dc() {
        let mut database = strict_employee_check_database();
        let EventSceneNext::EmployeeCheckChoices { choices } =
            &mut database.events[0].scenes[0].next
        else {
            unreachable!()
        };
        choices[0].check.dc = 10;

        let error = database
            .validate_contract()
            .expect_err("non-policy DC must fail closed");
        assert!(error.cause.contains("check dc"));
    }

    #[test]
    fn event_database_rejects_invalid_choice_key() {
        let mut database = strict_employee_check_database();
        let EventSceneNext::EmployeeCheckChoices { choices } =
            &mut database.events[0].scenes[0].next
        else {
            unreachable!()
        };
        choices[0].check.required_choice_key = Some(EventChoiceKeyId::new("Bad Key"));

        let error = database
            .validate_contract()
            .expect_err("invalid choice key must fail closed");
        assert!(error.cause.contains("lower_snake_case"));
    }

    #[test]
    fn event_database_rejects_second_reachable_check_scene() {
        let mut database = strict_employee_check_database();
        let EventSceneNext::EmployeeCheckChoices { choices } =
            database.events[0].scenes[0].next.clone()
        else {
            unreachable!()
        };
        let mut second_choice = choices[0].clone();
        second_choice.id = EventChoiceId::new("choice_b");
        second_choice.success_scene_id = EventSceneId::new("failure");
        second_choice.failure_scene_id = EventSceneId::new("failure");
        database.events[0].scenes[1].next = EventSceneNext::EmployeeCheckChoices {
            choices: vec![second_choice],
        };

        let error = database
            .validate_contract()
            .expect_err("a second reachable check scene must fail closed");
        assert!(error.cause.contains("at most one reachable"));
    }

    #[test]
    fn event_database_rejects_broken_check_scene_reference() {
        let mut database = strict_employee_check_database();
        let EventSceneNext::EmployeeCheckChoices { choices } =
            &mut database.events[0].scenes[0].next
        else {
            unreachable!()
        };
        choices[0].success_scene_id = EventSceneId::new("missing_scene");

        let error = database
            .validate_contract()
            .expect_err("missing check branch target must fail closed");
        assert!(error.cause.contains("missing_scene"));
    }

    #[test]
    fn event_database_rejects_checked_scene_return_cycle() {
        let mut database = strict_employee_check_database();
        database.events[0].scenes[1].next = EventSceneNext::Scene {
            scene_id: EventSceneId::new("check"),
        };

        let error = database
            .validate_contract()
            .expect_err("a return to the checked scene must fail closed");
        assert!(error.cause.contains("cycle"));
    }
}
