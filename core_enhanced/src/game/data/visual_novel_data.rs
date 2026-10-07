use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::data::{build_string_index, once_lock_with};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VisualNovelId(pub String);

impl VisualNovelId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VisualNovelSceneId(pub String);

impl VisualNovelSceneId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VisualNovelChoiceId(pub String);

impl VisualNovelChoiceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelDefinition {
    pub id: VisualNovelId,
    pub entry_scene_id: VisualNovelSceneId,
    pub default_presentation_mode: VisualNovelPresentationMode,
    pub scenes: Vec<VisualNovelSceneDefinition>,
}

impl VisualNovelDefinition {
    pub fn scene_by_id(&self, scene_id: &str) -> Option<&VisualNovelSceneDefinition> {
        self.scenes
            .iter()
            .find(|scene| scene.id.as_str() == scene_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelSceneDefinition {
    pub id: VisualNovelSceneId,
    pub text_kind: VisualNovelTextKind,
    pub background: VisualNovelBackground,
    pub script_id: String,
    pub speakers: Vec<VisualNovelSpeakerDefinition>,
    pub next: VisualNovelSceneNext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelTextKind {
    Dialogue,
    Narration,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelBackground {
    CurrentScreenDimmed,
    Resource { background_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelSpeakerDefinition {
    pub speaker_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait_id: Option<String>,
    pub position: VisualNovelSpeakerPosition,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelSpeakerPosition {
    FarLeft,
    Left,
    Center,
    Right,
    FarRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelPresentationMode {
    Cinematic,
    Radio,
    Compact,
    NonBlocking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelSceneNext {
    Scene {
        scene_id: VisualNovelSceneId,
    },
    Choices {
        choices: Vec<VisualNovelChoiceDefinition>,
    },
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelChoiceDefinition {
    pub id: VisualNovelChoiceId,
    pub label_id: String,
    #[serde(default)]
    pub preview: VisualNovelChoicePreview,
    pub next: VisualNovelChoiceNext,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelChoicePreview {
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
pub enum VisualNovelChoiceNext {
    Scene { scene_id: VisualNovelSceneId },
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelDatabase {
    pub visual_novels: Vec<VisualNovelDefinition>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl VisualNovelDatabase {
    // Only assemble here: final cross-catalog validation must precede indexes.
    pub(crate) fn from_raw_parts(visual_novels: Vec<VisualNovelDefinition>) -> Self {
        Self {
            visual_novels,
            by_id: OnceLock::new(),
        }
    }

    pub fn new(visual_novels: Vec<VisualNovelDefinition>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &visual_novels,
            "visual novel id",
            |visual_novel| visual_novel.id.as_str(),
        ));
        Self {
            visual_novels,
            by_id,
        }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.visual_novels, "visual novel id", |visual_novel| {
                visual_novel.id.as_str()
            })
        })
    }

    pub(crate) fn validate_indexes(&self) {
        self.validate_contract()
            .expect("invalid visual novel authoring contract");
        let _ = self.by_id();
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (visual_novel_index, visual_novel) in self.visual_novels.iter().enumerate() {
            validate_visual_novel_definition(visual_novel, visual_novel_index)?;
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&VisualNovelDefinition> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.visual_novels.get(index))
    }
}

fn visual_novel_error(
    visual_novel: &VisualNovelDefinition,
    field_path: impl Into<String>,
    cause: impl Into<String>,
) -> crate::game::data::DataValidationError {
    crate::game::data::DataValidationError::at(visual_novel.id.as_str(), field_path, cause)
}

fn validate_visual_novel_definition(
    visual_novel: &VisualNovelDefinition,
    visual_novel_index: usize,
) -> Result<(), crate::game::data::DataValidationError> {
    let visual_novel_path = format!("visual_novels[{visual_novel_index}]");
    if visual_novel.id.as_str().trim().is_empty() {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.id"),
            "visual novel id must not be empty",
        ));
    }
    if visual_novel.entry_scene_id.as_str().trim().is_empty() {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.entry_scene_id"),
            "entry_scene_id must not be empty",
        ));
    }
    if visual_novel.scenes.is_empty() {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.scenes"),
            "visual novel must define at least one scene",
        ));
    }

    let mut seen_scene_ids = HashSet::new();
    let mut scene_by_id = HashMap::new();
    for (scene_index, scene) in visual_novel.scenes.iter().enumerate() {
        let scene_path = format!("{visual_novel_path}.scenes[{scene_index}]");
        if scene.id.as_str().trim().is_empty() {
            return Err(visual_novel_error(
                visual_novel,
                format!("{scene_path}.id"),
                "scene id must not be empty",
            ));
        }
        if !seen_scene_ids.insert(scene.id.as_str()) {
            return Err(visual_novel_error(
                visual_novel,
                format!("{scene_path}.id"),
                "duplicate scene id",
            ));
        }
        validate_scene(visual_novel, scene, &scene_path)?;
        scene_by_id.insert(scene.id.as_str(), scene_index);
    }

    if !scene_by_id.contains_key(visual_novel.entry_scene_id.as_str()) {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.entry_scene_id"),
            format!(
                "references missing scene '{}'",
                visual_novel.entry_scene_id.as_str()
            ),
        ));
    }

    for (scene_index, scene) in visual_novel.scenes.iter().enumerate() {
        for target in scene_targets(scene) {
            if !scene_by_id.contains_key(target.as_str()) {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{visual_novel_path}.scenes[{scene_index}].next"),
                    format!("references missing scene '{}'", target.as_str()),
                ));
            }
        }
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut has_terminal_path = false;
    visit_scene(
        visual_novel,
        visual_novel.entry_scene_id.as_str(),
        &scene_by_id,
        &mut visiting,
        &mut visited,
        &mut has_terminal_path,
        &visual_novel_path,
    )?;

    if !has_terminal_path {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.scenes"),
            "visual novel must have at least one terminal path",
        ));
    }
    for (scene_index, scene) in visual_novel.scenes.iter().enumerate() {
        if !visited.contains(scene.id.as_str()) {
            return Err(visual_novel_error(
                visual_novel,
                format!("{visual_novel_path}.scenes[{scene_index}]"),
                "unreachable scene",
            ));
        }
    }
    Ok(())
}

fn validate_scene(
    visual_novel: &VisualNovelDefinition,
    scene: &VisualNovelSceneDefinition,
    scene_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if scene.script_id.trim().is_empty() {
        return Err(visual_novel_error(
            visual_novel,
            format!("{scene_path}.script_id"),
            "script_id must not be empty",
        ));
    }
    if let VisualNovelBackground::Resource { background_id } = &scene.background {
        if background_id.trim().is_empty() {
            return Err(visual_novel_error(
                visual_novel,
                format!("{scene_path}.background.Resource.background_id"),
                "background_id must not be empty",
            ));
        }
    }
    let active_speaker_count = scene
        .speakers
        .iter()
        .filter(|speaker| speaker.active)
        .count();
    match scene.text_kind {
        VisualNovelTextKind::Dialogue => {
            if scene.speakers.is_empty() {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.speakers"),
                    "Dialogue requires at least one speaker",
                ));
            }
            if active_speaker_count != 1 {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.speakers"),
                    "Dialogue requires exactly one active speaker",
                ));
            }
        }
        VisualNovelTextKind::Narration => {
            if active_speaker_count > 1 {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.speakers"),
                    "Narration allows at most one active speaker",
                ));
            }
        }
        VisualNovelTextKind::System => {
            if !scene.speakers.is_empty() {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.speakers"),
                    "System requires empty speakers",
                ));
            }
        }
    }
    for (speaker_index, speaker) in scene.speakers.iter().enumerate() {
        let speaker_path = format!("{scene_path}.speakers[{speaker_index}]");
        if speaker.speaker_id.trim().is_empty() {
            return Err(visual_novel_error(
                visual_novel,
                format!("{speaker_path}.speaker_id"),
                "speaker_id must not be empty",
            ));
        }
        if let Some(portrait_id) = &speaker.portrait_id {
            if portrait_id.trim().is_empty() {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{speaker_path}.portrait_id"),
                    "portrait_id must not be empty",
                ));
            }
        }
    }
    match &scene.next {
        VisualNovelSceneNext::Scene { scene_id } => {
            if scene_id.as_str().trim().is_empty() {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.next.Scene.scene_id"),
                    "next scene_id must not be empty",
                ));
            }
        }
        VisualNovelSceneNext::Choices { choices } => {
            if choices.is_empty() {
                return Err(visual_novel_error(
                    visual_novel,
                    format!("{scene_path}.next.Choices.choices"),
                    "Choices must not be empty",
                ));
            }
            let mut seen_choice_ids = HashSet::new();
            for (choice_index, choice) in choices.iter().enumerate() {
                let choice_path = format!("{scene_path}.next.Choices.choices[{choice_index}]");
                if choice.id.as_str().trim().is_empty() {
                    return Err(visual_novel_error(
                        visual_novel,
                        format!("{choice_path}.id"),
                        "choice id must not be empty",
                    ));
                }
                if !seen_choice_ids.insert(choice.id.as_str()) {
                    return Err(visual_novel_error(
                        visual_novel,
                        format!("{choice_path}.id"),
                        "duplicate choice id",
                    ));
                }
                if choice.label_id.trim().is_empty() {
                    return Err(visual_novel_error(
                        visual_novel,
                        format!("{choice_path}.label_id"),
                        "label_id must not be empty",
                    ));
                }
                if let VisualNovelChoiceNext::Scene { scene_id } = &choice.next {
                    if scene_id.as_str().trim().is_empty() {
                        return Err(visual_novel_error(
                            visual_novel,
                            format!("{choice_path}.next.Scene.scene_id"),
                            "choice next scene_id must not be empty",
                        ));
                    }
                }
            }
        }
        VisualNovelSceneNext::End => {}
    }
    Ok(())
}

fn scene_targets(scene: &VisualNovelSceneDefinition) -> Vec<&VisualNovelSceneId> {
    match &scene.next {
        VisualNovelSceneNext::Scene { scene_id } => vec![scene_id],
        VisualNovelSceneNext::Choices { choices } => choices
            .iter()
            .filter_map(|choice| match &choice.next {
                VisualNovelChoiceNext::Scene { scene_id } => Some(scene_id),
                VisualNovelChoiceNext::End => None,
            })
            .collect(),
        VisualNovelSceneNext::End => Vec::new(),
    }
}

fn scene_is_terminal(scene: &VisualNovelSceneDefinition) -> bool {
    match &scene.next {
        VisualNovelSceneNext::End => true,
        VisualNovelSceneNext::Choices { choices } => choices
            .iter()
            .any(|choice| matches!(choice.next, VisualNovelChoiceNext::End)),
        VisualNovelSceneNext::Scene { .. } => false,
    }
}

fn visit_scene(
    visual_novel: &VisualNovelDefinition,
    scene_id: &str,
    scene_by_id: &HashMap<&str, usize>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    has_terminal_path: &mut bool,
    visual_novel_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if visited.contains(scene_id) {
        return Ok(());
    }
    if !visiting.insert(scene_id.to_string()) {
        return Err(visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.scenes"),
            format!("scene graph contains a cycle at scene '{scene_id}'"),
        ));
    }
    let scene_index = *scene_by_id.get(scene_id).ok_or_else(|| {
        visual_novel_error(
            visual_novel,
            format!("{visual_novel_path}.scenes"),
            format!("scene graph references missing scene '{scene_id}'"),
        )
    })?;
    let scene = &visual_novel.scenes[scene_index];
    if scene_is_terminal(scene) {
        *has_terminal_path = true;
    }
    for target in scene_targets(scene) {
        visit_scene(
            visual_novel,
            target.as_str(),
            scene_by_id,
            visiting,
            visited,
            has_terminal_path,
            visual_novel_path,
        )?;
    }
    visiting.remove(scene_id);
    visited.insert(scene_id.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_dialogue_scene(id: &str, next: VisualNovelSceneNext) -> VisualNovelSceneDefinition {
        VisualNovelSceneDefinition {
            id: VisualNovelSceneId::new(id),
            text_kind: VisualNovelTextKind::Dialogue,
            background: VisualNovelBackground::CurrentScreenDimmed,
            script_id: format!("{id}_script"),
            speakers: vec![VisualNovelSpeakerDefinition {
                speaker_id: "speaker".to_string(),
                portrait_id: None,
                position: VisualNovelSpeakerPosition::Center,
                active: true,
            }],
            next,
        }
    }

    #[test]
    fn visual_novel_database_rejects_dialogue_without_active_speaker() {
        let db = VisualNovelDatabase::new(vec![VisualNovelDefinition {
            id: VisualNovelId::new("vn"),
            entry_scene_id: VisualNovelSceneId::new("start"),
            default_presentation_mode: VisualNovelPresentationMode::Cinematic,
            scenes: vec![VisualNovelSceneDefinition {
                id: VisualNovelSceneId::new("start"),
                text_kind: VisualNovelTextKind::Dialogue,
                background: VisualNovelBackground::CurrentScreenDimmed,
                script_id: "script".to_string(),
                speakers: vec![],
                next: VisualNovelSceneNext::End,
            }],
        }]);

        let error = db
            .validate_contract()
            .expect_err("Dialogue without an active speaker must reject");
        assert_eq!(error.object_id.as_deref(), Some("vn"));
        assert_eq!(
            error.field_path.as_deref(),
            Some("visual_novels[0].scenes[0].speakers")
        );
        assert!(error.cause.contains("at least one speaker"));
    }

    #[test]
    fn visual_novel_database_accepts_branching_graph() {
        let db = VisualNovelDatabase::new(vec![VisualNovelDefinition {
            id: VisualNovelId::new("vn"),
            entry_scene_id: VisualNovelSceneId::new("start"),
            default_presentation_mode: VisualNovelPresentationMode::Cinematic,
            scenes: vec![
                VisualNovelSceneDefinition {
                    id: VisualNovelSceneId::new("start"),
                    text_kind: VisualNovelTextKind::Narration,
                    background: VisualNovelBackground::CurrentScreenDimmed,
                    script_id: "intro".to_string(),
                    speakers: vec![],
                    next: VisualNovelSceneNext::Choices {
                        choices: vec![VisualNovelChoiceDefinition {
                            id: VisualNovelChoiceId::new("continue"),
                            label_id: "continue_label".to_string(),
                            preview: Default::default(),
                            next: VisualNovelChoiceNext::Scene {
                                scene_id: VisualNovelSceneId::new("end"),
                            },
                        }],
                    },
                },
                valid_dialogue_scene("end", VisualNovelSceneNext::End),
            ],
        }]);

        db.validate_indexes();
    }

    #[test]
    fn visual_novel_definition_rejects_removed_default_blocking_authority() {
        let source = r#"
            VisualNovelDefinition(
                id: "vn",
                entry_scene_id: "start",
                default_blocking: true,
                default_presentation_mode: Cinematic,
                scenes: [
                    VisualNovelSceneDefinition(
                        id: "start",
                        text_kind: Narration,
                        background: CurrentScreenDimmed,
                        script_id: "script",
                        speakers: [],
                        next: End,
                    ),
                ],
            )
        "#;

        let error = ron::de::from_str::<VisualNovelDefinition>(source)
            .expect_err("removed default_blocking must fail closed");
        assert!(error.to_string().contains("default_blocking"));
    }
}
