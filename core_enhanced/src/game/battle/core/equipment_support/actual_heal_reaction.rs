//! One observed positive allied heal and every equipped reaction it caused.
use super::*;
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct HealShieldOutcome {
    pub actual_heal_percent: u32,
    pub max_hp_cap_percent: u8,
    pub before: u32,
    pub after: u32,
    pub cap: u32,
    pub remainder_before: u8,
    pub remainder_after: u8,
    pub expires_at_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(HealShieldOutcome);
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct HealHasteOutcome {
    pub attack_speed_points: u32,
    pub expires_at_ms: u64,
    pub refreshed: bool,
}
crate::authoring::clone_shape::copy_shape!(HealHasteOutcome);
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActualHealContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub self_shield: Option<HealShieldOutcome>,
    pub target_haste: Option<HealHasteOutcome>,
}
crate::authoring::clone_shape::struct_shape!(ActualHealContribution {
    equipment,
    definition_id,
    source_file,
    field_path,
    self_shield,
    target_haste
});
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActualHealReactionWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub actual_healed: u32,
    pub healing_source: RuntimeFactSource,
    pub contributors: Vec<ActualHealContribution>,
}
crate::authoring::clone_shape::struct_shape!(ActualHealReactionWitness {
    generation,
    time_ms,
    actor,
    target,
    actual_healed,
    healing_source,
    contributors
});
impl ActualHealReactionWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.healing_source.retained_extent()?)?
            .checked_add(
                self.contributors
                    .capacity()
                    .checked_mul(size_of::<ActualHealContribution>())?,
            )?;
        for c in &self.contributors {
            bytes = bytes
                .checked_add(c.definition_id.capacity())?
                .checked_add(c.source_file.capacity())?
                .checked_add(c.field_path.capacity())?;
        }
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
