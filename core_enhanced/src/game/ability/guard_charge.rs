//! A finite, source-owned passive reduction meter released by a successful parry.
use super::marks::MarkRatio;
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardChargeDef {
    pub definition: String,
    pub relic_definition: String,
    pub capacity: u32,
    pub primary: MarkRatio,
    pub splash: MarkRatio,
    pub range: TileRangePattern,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardCounterDef {
    pub charge: Arc<GuardChargeDef>,
    pub weapon: MarkRatio,
    pub damage_type: DamageType,
}
impl GuardChargeDef {
    pub fn validate(&self) -> Result<(), String> {
        for id in [&self.definition, &self.relic_definition] {
            if id.is_empty() || id.len() > 256 || id.trim() != id {
                return Err("Guard charge requires bounded identities".into());
            }
        }
        if self.capacity == 0 {
            return Err("Guard charge requires a positive explicit capacity".into());
        }
        self.primary.validate(false)?;
        self.splash.validate(true)?;
        self.range.validate()
    }
}
impl GuardCounterDef {
    pub fn validate(&self) -> Result<(), String> {
        self.charge.validate()?;
        self.weapon.validate(false)?;
        if !matches!(self.damage_type, DamageType::Physical | DamageType::Magic) {
            return Err("Guard counter requires physical or magic damage".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardChargeView {
    pub relic_copy: uuid::Uuid,
    pub stored: u32,
    pub capacity: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardChargeReceipt {
    pub relic_copy: uuid::Uuid,
    pub blocked: u32,
    pub before: u32,
    pub after: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardReleaseOrigin {
    pub relic_copy: uuid::Uuid,
    pub relic_definition: String,
    pub definition: String,
    pub consumed: u32,
    pub added_power: u32,
    pub primary: bool,
}
crate::authoring::clone_shape::struct_shape!(GuardChargeDef {
    definition,
    relic_definition,
    capacity,
    primary,
    splash,
    range
});
crate::authoring::clone_shape::struct_shape!(GuardCounterDef {
    charge,
    weapon,
    damage_type
});
crate::authoring::clone_shape::struct_shape!(GuardReleaseOrigin {
    relic_copy,
    relic_definition,
    definition,
    consumed,
    added_power,
    primary
});
crate::authoring::clone_shape::copy_shape!(GuardChargeReceipt, GuardChargeView);
