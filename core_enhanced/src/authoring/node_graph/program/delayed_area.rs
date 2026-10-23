use crate::game::{
    ability::SkillTileAreaDeliveryDef,
    battle::{damage::DamageType, tile_range::TileRangePattern},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelayedAreaProgram {
    pub prepare_ms: u32,
    pub delivery_delay_ms: u32,
    pub acquisition: TileRangePattern,
    pub footprint: TileRangePattern,
    pub include_air: bool,
    pub damage_type: DamageType,
    pub coefficient_numerator: u64,
    pub coefficient_denominator: u64,
    pub on_hit_heal_percent: Option<u32>,
}
impl DelayedAreaProgram {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.acquisition.validate()?;
        self.footprint.validate()?;
        let limit = (1u64 << 53) - 1;
        if self.coefficient_numerator == 0
            || self.coefficient_denominator == 0
            || self.coefficient_numerator > limit
            || self.coefficient_denominator > limit
        {
            return Err("Area power requires an exact positive bounded ratio".into());
        }
        SkillTileAreaDeliveryDef {
            first_tick_delay_ms: self.delivery_delay_ms,
            on_hit_caster_heal_percent: self.on_hit_heal_percent,
            ..Default::default()
        }
        .try_validate_runtime_contract()
    }
}
