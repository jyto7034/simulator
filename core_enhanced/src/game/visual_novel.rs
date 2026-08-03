use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::game::{
    data::{
        visual_novel_data::{VisualNovelChoiceId, VisualNovelId, VisualNovelSceneId},
        visual_novel_trigger_data::VisualNovelTriggerId,
    },
    map::MapNodeId,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualNovelRuntimeState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<VisualNovelSession>,
    #[serde(default)]
    pub queue: Vec<VisualNovelSession>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualNovelSession {
    pub session_id: String,
    pub source: VisualNovelSource,
    pub vn_id: VisualNovelId,
    pub current_scene_id: VisualNovelSceneId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed_choice_id: Option<VisualNovelChoiceId>,
    pub blocking: bool,
    pub presentation_mode: VisualNovelSessionPresentationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VisualNovelSource {
    EventNode { node_id: MapNodeId },
    HqBriefing,
    Map,
    Loadout,
    Battle { battle_uuid: Uuid },
    Tutorial,
    Radio,
    Cutscene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualNovelSessionPresentationMode {
    Cinematic,
    Radio,
    Compact,
    NonBlocking,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualNovelTriggerRuntimeState {
    #[serde(default)]
    pub persistent_consumed: HashSet<VisualNovelTriggerConsumedKey>,
    #[serde(default)]
    pub checkpoint_rewind_consumed: HashSet<VisualNovelTriggerConsumedKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VisualNovelTriggerConsumedKey {
    pub trigger_id: VisualNovelTriggerId,
    pub scope: VisualNovelTriggerConsumedScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VisualNovelTriggerConsumedScope {
    Run,
    Battle { battle_uuid: Uuid },
    Node { node_id: MapNodeId },
}
