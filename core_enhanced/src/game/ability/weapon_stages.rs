//! Authored basic-shot variants selected by the persistent weapon copy's stage.
//! Evolution remains owned by the existing fragment/weapon transaction.
use super::{weapon_marks::WeaponStage, DeliveryDef};
use crate::game::data::equipment_data::WeaponCombatProfile;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct CompiledWeaponStages {
    pub definition: EquipmentWeaponStages,
    pub base_source: Arc<super::authoring::EffectSource>,
    pub upgraded_source: Arc<super::authoring::EffectSource>,
}
impl CompiledWeaponStages {
    pub(crate) fn source(&self, stage: WeaponStage) -> &Arc<super::authoring::EffectSource> {
        match stage {
            WeaponStage::Base => &self.base_source,
            WeaponStage::Upgraded => &self.upgraded_source,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentWeaponStages {
    pub base: EquipmentWeaponShot,
    pub upgraded: EquipmentWeaponShot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquipmentWeaponShot {
    pub interval_ms: u64,
    pub windup_ms: u32,
    pub delivery: DeliveryDef,
    /// One tile is 1_000_000 units. Zero explicitly disables displacement.
    pub push_distance_units: u32,
}

impl EquipmentWeaponStages {
    pub(crate) fn shot(&self, stage: WeaponStage) -> &EquipmentWeaponShot {
        match stage {
            WeaponStage::Base => &self.base,
            WeaponStage::Upgraded => &self.upgraded,
        }
    }
    pub(crate) fn validate(&self) -> Result<(), (String, String)> {
        for (name, shot) in [("base", &self.base), ("upgraded", &self.upgraded)] {
            let fail = |field: &str, cause: &str| {
                (
                    format!("program.weapon_stages.{name}.{field}"),
                    cause.to_owned(),
                )
            };
            if shot.interval_ms == 0 {
                return Err(fail(
                    "interval_ms",
                    "weapon stage requires positive attack interval",
                ));
            }
            if shot.push_distance_units > 32_000_000 {
                return Err(fail(
                    "push_distance_units",
                    "weapon stage push requires 0..32000000 units",
                ));
            }
            shot.delivery
                .try_validate_directional_basic_attack_contract()
                .map_err(|e| fail(&format!("delivery.{}", e.field_suffix), e.cause))?;
        }
        Ok(())
    }
}
impl EquipmentWeaponShot {
    pub(crate) fn apply(&self, profile: &mut WeaponCombatProfile) {
        profile.interval_ms = self.interval_ms;
        profile.windup_ms = self.windup_ms;
        profile.delivery.clone_from(&self.delivery);
    }
}
