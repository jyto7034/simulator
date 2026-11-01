//! Copy-owned delayed explosions triggered by an actual hostile death.
use crate::game::battle::{damage::DamageType, tile_range::TileRangePattern};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentDeathBurst {
    pub delay_ms: u32,
    pub range: TileRangePattern,
    pub attack_percent: u32,
    pub damage_type: DamageType,
}
impl EquipmentDeathBurst {
    pub(crate) fn validate(&self) -> Result<(), (String, String)> {
        let fail =
            |field: &str, message: &str| (format!("program.death_burst.{field}"), message.into());
        if self.delay_ms == 0 {
            return Err(fail("delay_ms", "death burst requires positive delay"));
        }
        if !(1..=10000).contains(&self.attack_percent) {
            return Err(fail(
                "attack_percent",
                "death burst percent requires 1..10000",
            ));
        }
        if self.range.rows.len() > 33 || self.range.rows.iter().any(|row| row.len() > 33) {
            return Err(fail("range", "death burst range must fit 33 by 33 tiles"));
        }
        self.range
            .validate()
            .map_err(|message| fail("range", &message))
    }
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        crate::game::relic::structural::support::pattern_extent(&self.range)
    }
}
crate::authoring::clone_shape::struct_shape!(EquipmentDeathBurst {
    delay_ms,
    range,
    attack_percent,
    damage_type
});
