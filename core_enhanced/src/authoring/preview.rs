//! Closed live queries. No arbitrary command document or implicit focus mutation.

use serde::Serialize;
use uuid::Uuid;

use super::control::{self, Resource};
use crate::{
    application::ApplicationExecutionContext,
    game::{
        ability::SkillId, battle::event_log::SkillCastTarget, behavior::GameError,
        resources::board::Position,
    },
};

/// Owned input for the two existing application Preview commands.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrialPreview {
    RequestDeploymentRangePreview {
        employee_uuid: Uuid,
        position: Position,
    },
    PreviewSkillTarget {
        employee_uuid: Uuid,
        skill_id: SkillId,
        target: SkillCastTarget,
    },
}

impl TrialPreview {
    /// Read only lengths before serde can scan/escape strings. All other input
    /// fields are fixed Copy leaves, guarded by exhaustive patterns and types.
    pub(crate) fn admit(&self, context: &ApplicationExecutionContext) -> Result<(), GameError> {
        fn copy_leaf<T: Copy>(_: &T) {}
        let skill_bytes = match self {
            Self::RequestDeploymentRangePreview {
                employee_uuid,
                position,
            } => {
                copy_leaf(employee_uuid);
                copy_leaf(position);
                0
            }
            Self::PreviewSkillTarget {
                employee_uuid,
                skill_id,
                target,
            } => {
                copy_leaf(employee_uuid);
                copy_leaf(target);
                skill_id.as_str().len()
            }
        };
        let ApplicationExecutionContext {
            source_command_id,
            battle_event_after_seq,
            expected_battle_authority,
        } = context;
        copy_leaf(battle_event_after_seq);
        copy_leaf(expected_battle_authority);
        // A conservative fixed allowance for tags, field names and Copy leaves;
        // variable bytes are additionally admitted before the first projection.
        let bytes = 256usize
            .checked_add(skill_bytes)
            .and_then(|n| n.checked_add(source_command_id.as_ref().map_or(0, String::len)))
            .unwrap_or(usize::MAX);
        control::admit(Resource::MetadataBytes, 0, bytes, "preview_input_bytes")?;
        control::charge(bytes as u64, "preview_input_scan")
    }
}
