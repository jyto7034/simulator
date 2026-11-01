use crate::game::ability::{
    tile_burst::{TileBurstDef, TileBurstSource},
    weapon_marks::MarkWeaponDef,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponMarkBinding {
    pub weapon: String,
    pub definition: MarkWeaponDef,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileBurstProgram {
    pub effect: TileBurstDef,
    pub weapon: WeaponMarkBinding,
}
impl TileBurstProgram {
    pub fn validate(&self) -> Result<(), String> {
        self.effect.validate()?;
        self.weapon.definition.validate()?;
        if !crate::game::ability::weapon_marks::valid_id(&self.weapon.weapon) {
            return Err("Invalid mark weapon identity".into());
        }
        if let TileBurstSource::WeaponStacks { weapon, key, .. } = &self.effect.source {
            if weapon != &self.weapon.weapon || key != &self.weapon.definition.key {
                return Err("Stack burst and weapon must share one explicit store".into());
            }
        }
        Ok(())
    }
}
