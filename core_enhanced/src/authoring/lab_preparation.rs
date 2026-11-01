//! Finite authoring preparation over the production World loadout and combat
//! planner. This is not a player command, a battle injector or an approval.
use super::{
    AuthoringContent, CancellationToken, TrialFailure, TrialLimits, TrialScenario, TrialSession,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabEmployeeGroup {
    pub definition_id: String,
    pub count: u32,
    /// None retains the published employee definition's initial loadout.
    pub equipment_ids: Option<Vec<String>>,
    pub active_fragment_id: Option<String>,
    #[serde(default)]
    pub relics: Vec<LabRelicLoadout>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabRelicLoadout {
    pub definition_id: String,
    pub corrupt: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabPlacement {
    /// Index in the newly prepared employees, not a supplied runtime identity.
    pub employee_index: usize,
    pub position: crate::game::resources::Position,
    pub facing: crate::game::battle::tile_range::FacingDirection,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabPreparation {
    pub schema_version: u32,
    pub employees: Vec<LabEmployeeGroup>,
    pub encounter_id: String,
    pub category: crate::game::map::MapNodeCategory,
    pub placements: Vec<LabPlacement>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabPreparationReceipt {
    pub employee_ids: Vec<Uuid>,
    pub battle_id: Uuid,
    pub deployed_count: usize,
}
crate::authoring::clone_shape::struct_shape!(LabPreparationReceipt {
    employee_ids,
    battle_id,
    deployed_count
});
impl LabPreparation {
    pub(crate) fn validate(&self) -> Result<usize, crate::game::behavior::GameError> {
        use crate::game::{behavior::GameError, map::MapNodeCategory};
        if self.schema_version != 1
            || self.employees.len() > 128
            || self.encounter_id.is_empty()
            || self.encounter_id.len() > 128
            || !matches!(
                self.category,
                MapNodeCategory::Combat | MapNodeCategory::Boss
            )
        {
            return Err(GameError::InvalidAction);
        }
        let mut count = 0usize;
        for group in &self.employees {
            if group.count == 0
                || group.definition_id.is_empty()
                || group.definition_id.len() > 128
                || group.relics.len() > 3
                || group
                    .equipment_ids
                    .as_ref()
                    .is_some_and(|e| e.len() > 8 || e.iter().any(|s| s.is_empty() || s.len() > 128))
                || group
                    .active_fragment_id
                    .as_ref()
                    .is_some_and(|s| s.is_empty() || s.len() > 128)
                || group
                    .relics
                    .iter()
                    .any(|r| r.definition_id.is_empty() || r.definition_id.len() > 128)
            {
                return Err(GameError::InvalidAction);
            }
            count = count
                .checked_add(group.count as usize)
                .ok_or(GameError::InvalidAction)?;
        }
        // A host authoring allocation ceiling; ordinary recruitment/gameplay
        // roster and authored battlefield constraints remain unchanged.
        if count > 4096 || self.placements.len() > count {
            return Err(GameError::InvalidAction);
        }
        let mut employees = std::collections::BTreeSet::new();
        let mut tiles = std::collections::BTreeSet::new();
        for p in &self.placements {
            if p.employee_index >= count
                || !employees.insert(p.employee_index)
                || !tiles.insert((p.position.x, p.position.y))
            {
                return Err(GameError::InvalidAction);
            }
        }
        Ok(count)
    }
}
pub struct WorldLabPreparation;
impl WorldLabPreparation {
    /// The normal preparation prefix establishes a run/loadout context. The
    /// finite bulk recipe then consumes the same World grant/plan/commit path.
    pub fn prepare(
        content: AuthoringContent,
        scenario: TrialScenario,
        recipe: LabPreparation,
        limits: TrialLimits,
        cancel: &CancellationToken,
    ) -> Result<TrialSession, TrialFailure> {
        TrialSession::prepare_with_lab(content, scenario, recipe, limits, cancel)
    }
}
