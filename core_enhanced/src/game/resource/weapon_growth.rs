//! Weapon-copy growth consumes the ordinary Run resource timeline.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponKillGrowthDef {
    pub resource: ResourceId,
    pub normal: u32,
    pub elite: u32,
    pub abnormality: u32,
    pub damage_per_stack: u32,
    pub secondary_percent: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplier: Option<WeaponGrowthMultiplier>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponGrowthMultiplier {
    pub relic_definition: String,
    pub factor: u32,
}
impl WeaponKillGrowthDef {
    pub fn validate(&self) -> Result<(), String> {
        if self.secondary_percent > 100 || self.damage_per_stack == 0 {
            return Err(
                "Weapon growth requires positive damage and secondary percent <= 100".into(),
            );
        }
        if self.multiplier.as_ref().is_some_and(|m| {
            m.factor == 0
                || m.relic_definition.is_empty()
                || m.relic_definition.trim() != m.relic_definition
                || m.relic_definition.len() > 256
        }) {
            return Err("Invalid weapon growth multiplier".into());
        }
        Ok(())
    }
    pub fn resource_definition(&self) -> ResourceDefinition {
        ResourceDefinition {
            id: self.resource.clone(),
            owner: ResourceOwnerKind::EquipmentCopy,
            lifetime: ResourceLifetime::Run,
            initial: 0,
            cap: u32::MAX,
        }
    }
    pub fn validate_catalog(&self, catalog: &ResourceCatalog) -> Result<(), String> {
        self.validate()?;
        let expected = self.resource_definition();
        if !catalog.resources.iter().any(|d| *d == expected) {
            return Err("Weapon growth requires its declared EquipmentCopy Run resource, initial 0, safety cap u32::MAX".into());
        }
        Ok(())
    }
}
crate::authoring::clone_shape::struct_shape!(WeaponKillGrowthDef {
    resource,
    normal,
    elite,
    abnormality,
    damage_per_stack,
    secondary_percent,
    multiplier
});
crate::authoring::clone_shape::struct_shape!(WeaponGrowthMultiplier {
    relic_definition,
    factor
});
