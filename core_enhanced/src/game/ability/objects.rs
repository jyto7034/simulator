//! Authored entry to the independent object transaction.
use crate::game::{data::independent_object_data::IndependentObjectId, resources::Position};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectPlacementAnchor {
    CastTile,
    SourceTile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateObjectDef {
    pub definition_id: IndependentObjectId,
    pub anchor: ObjectPlacementAnchor,
    pub offsets: Vec<Position>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub allow_same_source_evolution: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_cost: Option<ObjectWallCostDef>,
}
crate::authoring::clone_shape::copy_shape!(ObjectPlacementAnchor);
crate::authoring::clone_shape::struct_shape!(CreateObjectDef {
    definition_id,
    anchor,
    offsets,
    allow_same_source_evolution,
    wall_cost
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectWallCostDef {
    pub pool_id: String,
    #[serde(default)]
    pub pool_source: ObjectPoolSource,
    pub health_cost_percent: u32,
    pub wall_health_percent: u32,
}
crate::authoring::clone_shape::struct_shape!(ObjectWallCostDef {
    pool_id,
    pool_source,
    health_cost_percent,
    wall_health_percent
});

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectPoolSource {
    #[default]
    CurrentEffect,
    EquippedWeapon,
}
crate::authoring::clone_shape::copy_shape!(ObjectPoolSource);

impl CreateObjectDef {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.definition_id.as_str().trim().is_empty() || self.offsets.is_empty() {
            return Err("CreateObject requires a definition and nonempty offset batch".into());
        }
        if self.allow_same_source_evolution && self.offsets.len() != 1 {
            return Err("same-source evolution requires exactly one placement".into());
        }
        if let Some(cost) = &self.wall_cost {
            if self.offsets.len() != 1
                || self.allow_same_source_evolution
                || cost.pool_id.trim().is_empty()
                || cost.health_cost_percent == 0
                || cost.health_cost_percent > 100
                || cost.wall_health_percent == 0
            {
                return Err("wall_cost requires one non-evolving wall placement, a pool ID and positive cost/conversion percentages".into());
            }
        }
        let mut offsets = std::collections::BTreeSet::new();
        for offset in &self.offsets {
            if !offsets.insert((offset.x, offset.y)) {
                return Err("CreateObject offsets must be unique".into());
            }
        }
        Ok(())
    }
}
