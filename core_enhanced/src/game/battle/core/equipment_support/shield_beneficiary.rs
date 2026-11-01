//! A historical damage calculation witness; shield records remain authoritative.
use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ShieldBeneficiaryDamageWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub beneficiary: UnitInstanceId,
    pub target: UnitInstanceId,
    pub provider: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub shield_activation: u64,
    pub shield_remaining: u32,
    pub percent: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub shield_source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(ShieldBeneficiaryDamageWitness {
    generation,
    time_ms,
    actor,
    beneficiary,
    target,
    provider,
    equipment,
    shield_activation,
    shield_remaining,
    percent,
    definition_id,
    source_file,
    field_path,
    shield_source
});
impl ShieldBeneficiaryDamageWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.shield_source.retained_extent()?)?;
        if let Some(g) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
