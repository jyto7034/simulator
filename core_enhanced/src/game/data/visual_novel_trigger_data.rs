use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::game::{
    combat_preview::{CombatMissionVariant, CombatNodeType},
    data::{build_string_index, once_lock_with, visual_novel_data::VisualNovelId},
    map::{GameMode, MapNodeCategory, MapNodeKindId},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VisualNovelTriggerId(pub String);

impl VisualNovelTriggerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelTrigger {
    pub id: VisualNovelTriggerId,
    pub source: VisualNovelTriggerSource,
    pub vn_id: VisualNovelId,
    pub repeat: VisualNovelTriggerRepeat,
    pub checkpoint_policy: VisualNovelTriggerCheckpointPolicy,
    pub priority: i32,
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelTriggerSource {
    RunStart {
        game_mode: GameMode,
    },
    MapNodeEntered {
        category: MapNodeCategory,
        kind_id: MapNodeKindId,
    },
    BattleStarted {
        encounter_id: String,
        node_type: CombatNodeType,
        mission_variant: CombatMissionVariant,
    },
    BattleTimeReached {
        battle_time_ms: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelTriggerRepeat {
    Always,
    OncePerRun,
    OncePerBattle,
    OncePerNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum VisualNovelTriggerCheckpointPolicy {
    PersistAcrossCheckpointLoad,
    RewindWithRunCheckpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualNovelTriggerDatabase {
    pub triggers: Vec<VisualNovelTrigger>,
    #[serde(skip)]
    by_id: OnceLock<HashMap<String, usize>>,
}

impl VisualNovelTriggerDatabase {
    pub fn new(triggers: Vec<VisualNovelTrigger>) -> Self {
        let by_id = once_lock_with(build_string_index(
            &triggers,
            "visual novel trigger id",
            |trigger| trigger.id.as_str(),
        ));
        Self { triggers, by_id }
    }

    fn by_id(&self) -> &HashMap<String, usize> {
        self.by_id.get_or_init(|| {
            build_string_index(&self.triggers, "visual novel trigger id", |trigger| {
                trigger.id.as_str()
            })
        })
    }

    pub(crate) fn validate_indexes(&self) {
        let _ = self.by_id();
        self.validate_contract()
            .expect("invalid visual novel trigger authoring contract");
    }

    pub(crate) fn validate_contract(&self) -> Result<(), crate::game::data::DataValidationError> {
        for (trigger_index, trigger) in self.triggers.iter().enumerate() {
            validate_trigger(trigger, &format!("triggers[{trigger_index}]"))?;
        }
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Option<&VisualNovelTrigger> {
        self.by_id()
            .get(id)
            .and_then(|&index| self.triggers.get(index))
    }
}

fn trigger_error(
    trigger: &VisualNovelTrigger,
    field_path: impl Into<String>,
    cause: impl Into<String>,
) -> crate::game::data::DataValidationError {
    crate::game::data::DataValidationError::at(trigger.id.as_str(), field_path, cause)
}

fn validate_trigger(
    trigger: &VisualNovelTrigger,
    trigger_path: &str,
) -> Result<(), crate::game::data::DataValidationError> {
    if trigger.id.as_str().trim().is_empty() {
        return Err(trigger_error(
            trigger,
            format!("{trigger_path}.id"),
            "visual novel trigger id must not be empty",
        ));
    }
    if trigger.vn_id.as_str().trim().is_empty() {
        return Err(trigger_error(
            trigger,
            format!("{trigger_path}.vn_id"),
            "vn_id must not be empty",
        ));
    }
    match &trigger.source {
        VisualNovelTriggerSource::RunStart { .. } => {}
        VisualNovelTriggerSource::MapNodeEntered { kind_id, .. } => {
            if kind_id.as_str().trim().is_empty() {
                return Err(trigger_error(
                    trigger,
                    format!("{trigger_path}.source.MapNodeEntered.kind_id"),
                    "kind_id must not be empty",
                ));
            }
        }
        VisualNovelTriggerSource::BattleStarted { encounter_id, .. } => {
            if encounter_id.trim().is_empty() {
                return Err(trigger_error(
                    trigger,
                    format!("{trigger_path}.source.BattleStarted.encounter_id"),
                    "encounter_id must not be empty",
                ));
            }
        }
        VisualNovelTriggerSource::BattleTimeReached { battle_time_ms } => {
            if *battle_time_ms == 0 {
                return Err(trigger_error(
                    trigger,
                    format!("{trigger_path}.source.BattleTimeReached.battle_time_ms"),
                    "battle_time_ms must be greater than zero",
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_visual_novel_trigger_references(
    trigger_data: &VisualNovelTriggerDatabase,
    visual_novel_data: &crate::game::data::visual_novel_data::VisualNovelDatabase,
) -> Result<(), crate::game::data::DataValidationError> {
    for (trigger_index, trigger) in trigger_data.triggers.iter().enumerate() {
        if visual_novel_data
            .get_by_id(trigger.vn_id.as_str())
            .is_none()
        {
            return Err(trigger_error(
                trigger,
                format!("triggers[{trigger_index}].vn_id"),
                format!(
                    "references missing visual novel '{}'",
                    trigger.vn_id.as_str()
                ),
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_no_duplicate_run_start_triggers(trigger_data: &VisualNovelTriggerDatabase) {
    let mut seen = HashSet::new();
    for trigger in &trigger_data.triggers {
        if let VisualNovelTriggerSource::RunStart { game_mode } = trigger.source {
            let key = format!("{game_mode:?}");
            assert!(
                seen.insert(key),
                "duplicate RunStart visual novel trigger for game mode {:?}",
                game_mode
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger(id: &str, vn_id: &str) -> VisualNovelTrigger {
        VisualNovelTrigger {
            id: VisualNovelTriggerId::new(id),
            source: VisualNovelTriggerSource::RunStart {
                game_mode: GameMode::Standard,
            },
            vn_id: VisualNovelId::new(vn_id),
            repeat: VisualNovelTriggerRepeat::OncePerRun,
            checkpoint_policy: VisualNovelTriggerCheckpointPolicy::PersistAcrossCheckpointLoad,
            priority: 100,
            blocking: true,
        }
    }

    #[test]
    fn trigger_database_rejects_empty_trigger_id() {
        let db = VisualNovelTriggerDatabase::new(vec![trigger("", "vn")]);
        let error = db
            .validate_contract()
            .expect_err("empty trigger id must reject");
        assert_eq!(error.object_id.as_deref(), Some(""));
        assert_eq!(error.field_path.as_deref(), Some("triggers[0].id"));
        assert!(error.cause.contains("must not be empty"));
    }

    #[test]
    fn trigger_reference_validation_rejects_missing_visual_novel() {
        let triggers = VisualNovelTriggerDatabase::new(vec![trigger("t", "missing_vn")]);
        let visual_novels = crate::game::data::visual_novel_data::VisualNovelDatabase::new(vec![]);
        let error = validate_visual_novel_trigger_references(&triggers, &visual_novels)
            .expect_err("missing visual novel reference must reject");
        assert_eq!(error.object_id.as_deref(), Some("t"));
        assert_eq!(error.field_path.as_deref(), Some("triggers[0].vn_id"));
        assert!(error.cause.contains("missing_vn"));
    }
}
