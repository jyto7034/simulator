use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::{
    data::{build_string_index, once_lock_with},
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
    pub background_id: String,
    pub script_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventSceneNext {
    Scene { scene_id: EventSceneId },
    Choices { choices: Vec<EventChoiceDefinition> },
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoiceDefinition {
    pub id: EventChoiceId,
    pub label_id: String,
    #[serde(default)]
    pub preview: EventChoicePreview,
    #[serde(default)]
    pub effects: Vec<EventChoiceEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<EventChoiceNext>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventChoicePreview {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default)]
    pub starts_combat: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_hint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub penalty_hint_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventChoiceNext {
    Scene { scene_id: EventSceneId },
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EventChoiceEffect {
    Grant {
        effects: Vec<RewardEffect>,
    },
    StartCombat {
        encounter_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        primary_abnormality_id: Option<String>,
    },
    ApplyBossOmenStepResult,
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

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        for event in &self.events {
            validate_event_definition(event);
        }
    }

    pub fn get_by_id(&self, id: &str) -> Option<&EventDefinition> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.events.get(index))
    }
}

fn validate_event_definition(event: &EventDefinition) {
    assert!(
        !event.id.as_str().trim().is_empty(),
        "event id must not be empty"
    );
    assert!(
        !event.entry_scene_id.as_str().trim().is_empty(),
        "event '{}' entry_scene_id must not be empty",
        event.id.as_str()
    );
    assert!(
        !event.scenes.is_empty(),
        "event '{}' must define at least one scene",
        event.id.as_str()
    );

    let mut seen_scene_ids = HashSet::new();
    let mut scene_by_id = HashMap::new();
    for (scene_index, scene) in event.scenes.iter().enumerate() {
        assert!(
            !scene.id.as_str().trim().is_empty(),
            "event '{}' has empty scene id",
            event.id.as_str()
        );
        assert!(
            seen_scene_ids.insert(scene.id.as_str()),
            "event '{}' has duplicate scene id '{}'",
            event.id.as_str(),
            scene.id.as_str()
        );
        validate_presentation(event, scene);
        validate_scene_next(event, scene);
        scene_by_id.insert(scene.id.as_str(), scene_index);
    }

    assert!(
        scene_by_id.contains_key(event.entry_scene_id.as_str()),
        "event '{}' entry_scene_id '{}' does not exist",
        event.id.as_str(),
        event.entry_scene_id.as_str()
    );

    for scene in &event.scenes {
        for target in scene_targets(scene) {
            assert!(
                scene_by_id.contains_key(target.as_str()),
                "event '{}' scene '{}' references missing scene '{}'",
                event.id.as_str(),
                scene.id.as_str(),
                target.as_str()
            );
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
    );

    assert!(
        has_terminal_path,
        "event '{}' must have at least one terminal path",
        event.id.as_str()
    );
    for scene in &event.scenes {
        assert!(
            visited.contains(scene.id.as_str()),
            "event '{}' has unreachable scene '{}'",
            event.id.as_str(),
            scene.id.as_str()
        );
    }
}

fn validate_presentation(event: &EventDefinition, scene: &EventSceneDefinition) {
    assert!(
        !scene.presentation.background_id.trim().is_empty(),
        "event '{}' scene '{}' background_id must not be empty",
        event.id.as_str(),
        scene.id.as_str()
    );
    assert!(
        !scene.presentation.script_id.trim().is_empty(),
        "event '{}' scene '{}' script_id must not be empty",
        event.id.as_str(),
        scene.id.as_str()
    );
    if let Some(speaker_id) = &scene.presentation.speaker_id {
        assert!(
            !speaker_id.trim().is_empty(),
            "event '{}' scene '{}' speaker_id must not be empty",
            event.id.as_str(),
            scene.id.as_str()
        );
    }
    if let Some(portrait_id) = &scene.presentation.portrait_id {
        assert!(
            !portrait_id.trim().is_empty(),
            "event '{}' scene '{}' portrait_id must not be empty",
            event.id.as_str(),
            scene.id.as_str()
        );
    }
}

fn validate_scene_next(event: &EventDefinition, scene: &EventSceneDefinition) {
    match &scene.next {
        EventSceneNext::Scene { scene_id } => assert!(
            !scene_id.as_str().trim().is_empty(),
            "event '{}' scene '{}' next scene_id must not be empty",
            event.id.as_str(),
            scene.id.as_str()
        ),
        EventSceneNext::Choices { choices } => {
            assert!(
                !choices.is_empty(),
                "event '{}' scene '{}' Choices must not be empty",
                event.id.as_str(),
                scene.id.as_str()
            );
            let mut seen_choice_ids = HashSet::new();
            for choice in choices {
                assert!(
                    !choice.id.as_str().trim().is_empty(),
                    "event '{}' scene '{}' choice id must not be empty",
                    event.id.as_str(),
                    scene.id.as_str()
                );
                assert!(
                    seen_choice_ids.insert(choice.id.as_str()),
                    "event '{}' scene '{}' has duplicate choice id '{}'",
                    event.id.as_str(),
                    scene.id.as_str(),
                    choice.id.as_str()
                );
                assert!(
                    !choice.label_id.trim().is_empty(),
                    "event '{}' scene '{}' choice '{}' label_id must not be empty",
                    event.id.as_str(),
                    scene.id.as_str(),
                    choice.id.as_str()
                );
                if let Some(next) = &choice.next {
                    let EventChoiceNext::Scene { scene_id } = next else {
                        continue;
                    };
                    assert!(
                        !scene_id.as_str().trim().is_empty(),
                        "event '{}' scene '{}' choice '{}' next scene_id must not be empty",
                        event.id.as_str(),
                        scene.id.as_str(),
                        choice.id.as_str()
                    );
                }
            }
        }
        EventSceneNext::End => {}
    }
}

fn scene_targets(scene: &EventSceneDefinition) -> Vec<&EventSceneId> {
    match &scene.next {
        EventSceneNext::Scene { scene_id } => vec![scene_id],
        EventSceneNext::Choices { choices } => choices
            .iter()
            .filter_map(|choice| match &choice.next {
                Some(EventChoiceNext::Scene { scene_id }) => Some(scene_id),
                _ => None,
            })
            .collect(),
        EventSceneNext::End => Vec::new(),
    }
}

fn scene_is_terminal(scene: &EventSceneDefinition) -> bool {
    match &scene.next {
        EventSceneNext::End => true,
        EventSceneNext::Choices { choices } => choices.iter().any(|choice| {
            choice.next.is_none()
                || matches!(choice.next, Some(EventChoiceNext::End))
                || choice
                    .effects
                    .iter()
                    .any(|effect| matches!(effect, EventChoiceEffect::StartCombat { .. }))
        }),
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
) {
    if visited.contains(scene_id) {
        return;
    }
    assert!(
        visiting.insert(scene_id.to_string()),
        "event '{}' scene graph contains a cycle at scene '{}'",
        event.id.as_str(),
        scene_id
    );
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
        );
    }
    visiting.remove(scene_id);
    visited.insert(scene_id.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_database_rejects_unknown_choice_effect_fields() {
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
                                    script_id: "script",
                                ),
                                next: Choices(choices: [
                                    (
                                        id: "choice_a",
                                        label_id: "label",
                                        effects: [
                                            StartCombat(
                                                encounter_id: "encounter_a",
                                                fallback_encounter_id: "encounter_b",
                                            ),
                                        ],
                                        next: Some(End),
                                    ),
                                ]),
                            ),
                        ],
                    ),
                ],
            )
            "#,
        )
        .expect_err("unknown event effect fields should fail");

        assert!(
            err.to_string().contains("fallback_encounter_id"),
            "unexpected error: {err}"
        );
    }
}
