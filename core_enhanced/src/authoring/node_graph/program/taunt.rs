use crate::game::{
    ability::{action::ActionPolicy, sustained::ApproachTauntProfile},
    battle::tile_range::TileRangePattern,
    resource::WeaponKillGrowthDef,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TauntProgram {
    pub range: TileRangePattern,
    pub duration_ms: u32,
    pub profile: ApproachTauntProfile,
    pub policy: ActionPolicy,
    pub weapon_growth: Option<WeaponGrowthBinding>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponGrowthBinding {
    pub weapon: String,
    pub growth: WeaponKillGrowthDef,
}
impl TauntProgram {
    pub fn validate(&self) -> Result<(), String> {
        self.range.validate()?;
        if self.duration_ms == 0 {
            return Err("Taunt requires positive duration".into());
        }
        self.profile.validate()?;
        if let Some(binding) = &self.weapon_growth {
            if binding.weapon.is_empty()
                || binding.weapon.len() > 128
                || binding.weapon.trim() != binding.weapon
            {
                return Err("Invalid growth weapon identity".into());
            }
            binding.growth.validate()?;
        }
        Ok(())
    }
}
