use crate::game::{ability::sustained::BasicLineProfile, battle::tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttackOverlayProgram {
    pub duration_ms: u32,
    pub line: BasicLineProfile,
    pub damage_type: crate::game::battle::damage::DamageType,
    pub finish_range: TileRangePattern,
    pub base: u32,
    pub per_second: u32,
    pub cap: Option<u32>,
}
impl AttackOverlayProgram {
    pub fn validate(&self) -> Result<(), String> {
        self.line.validate()?;
        self.finish_range.validate()?;
        if self.cap.is_some_and(|cap| cap < self.base) {
            return Err("Finish cap must not be below its base power.".into());
        }
        if self.duration_ms == 0 {
            return Err("Overlay requires positive duration.".into());
        }
        let maximum = u64::from(self.base)
            + (u64::from(self.per_second) * u64::from(self.duration_ms)) / 1000;
        if self.cap.map_or(maximum, |c| maximum.min(c.into())) > u32::MAX.into() {
            return Err("Finish power exceeds native magnitude.".into());
        }
        Ok(())
    }
}
