use crate::game::{
    ability::{action::ActionPolicy, ground::GroundStatusProgram, SkillHitTargetFilter},
    battle::tile_range::TileRangePattern,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelGroundProgram {
    pub range: TileRangePattern,
    pub duration_ms: u32,
    pub pause_basic: bool,
    pub hit_targets: SkillHitTargetFilter,
    pub include_caster: bool,
    pub status: GroundStatusProgram,
    pub policy: ActionPolicy,
}
impl ChannelGroundProgram {
    pub fn validate(&self) -> Result<(), String> {
        self.range.validate()?;
        if self.duration_ms == 0 {
            return Err("Channel requires positive finite lifetime".into());
        }
        self.status.validate()
    }
}
