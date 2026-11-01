//! Current equipment contributions to direct physical/magic damage. No target state.
use super::BattleCore;
use crate::game::{
    battle::{
        damage::{DamageSource, DamageSourceSnapshot, DamageType, DirectPenetration},
        ids::UnitInstanceId,
        runtime_control,
        runtime_facts::EquipmentSourceSnapshot,
    },
    behavior::GameError,
};

use crate::authoring::control;
use std::{
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirectPenetrationContribution {
    pub equipment: EquipmentSourceSnapshot,
    pub damage_type: DamageType,
    pub percent: u32,
    pub flat: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(DirectPenetrationContribution {
    equipment,
    damage_type,
    percent,
    flat,
    definition_id,
    source_file,
    field_path
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirectPenetrationWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub target_defense: i32,
    pub target_magic_resist: i32,
    /// Configured equipment sums before other modifiers and final percentage cap.
    pub percent: u32,
    pub flat: u32,
    pub magic_percent: u32,
    pub magic_flat: u32,
    pub contributors: Vec<DirectPenetrationContribution>,
}
crate::authoring::clone_shape::struct_shape!(DirectPenetrationWitness {
    generation,
    time_ms,
    actor,
    target,
    target_defense,
    target_magic_resist,
    percent,
    flat,
    magic_percent,
    magic_flat,
    contributors
});
impl DirectPenetrationWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>().checked_add(
            self.contributors
                .capacity()
                .checked_mul(size_of::<DirectPenetrationContribution>())?,
        )?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        for contribution in &self.contributors {
            bytes = bytes
                .checked_add(contribution.definition_id.capacity())?
                .checked_add(contribution.source_file.capacity())?
                .checked_add(contribution.field_path.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(super) fn direct_penetration(
        &mut self,
        source: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<DirectPenetration, GameError> {
        if !self
            .game_data
            .equipment_attack_catalog()
            .has_direct_penetration
            || !matches!(
                source.source,
                DamageSource::BasicAttack | DamageSource::Ability
            )
            || source.fact_source.reaction
            || source.fact_source.guard_release().is_some()
            || source.fact_source.independent_object().is_some()
            || source
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|cast| cast.proc_origin.is_some())
        {
            return Ok(DirectPenetration::default());
        }
        let Some(owner) = self
            .owned_equipment_damage_source(source, target)
            .filter(|owner| *owner == source.source_id)
        else {
            return Ok(DirectPenetration::default());
        };
        let providers =
            self.equipped_attack_providers(owner, |p| !p.direct_penetration.is_empty())?;
        let mut total = DirectPenetration::default();
        let mut contributors = Vec::new();
        for (equipment, program) in providers {
            for contribution in &program.direct_penetration {
                let (percent, flat) = match contribution.damage_type {
                    DamageType::Physical => (&mut total.physical_percent, &mut total.physical_flat),
                    DamageType::Magic => (&mut total.magic_percent, &mut total.magic_flat),
                    DamageType::True => {
                        return Err(runtime_control::invariant("direct_penetration_type"))
                    }
                };
                *percent = percent.saturating_add(contribution.percent as i32);
                *flat = flat.saturating_add(contribution.flat as i32);
                if self.authoring_observation.is_some() {
                    let origin = &contribution.source;
                    let bytes = size_of::<DirectPenetrationContribution>()
                        .checked_add(origin.site.action.definition_id.as_str().len())
                        .and_then(|n| n.checked_add(origin.source_file.len()))
                        .and_then(|n| n.checked_add(origin.field_path.len()));
                    runtime_control::admit_retained_growth(
                        0,
                        bytes,
                        "direct_penetration_witness_copy",
                    )?;
                    control::charge(
                        bytes.ok_or(GameError::InvalidAction)? as u64,
                        "direct_penetration_witness_copy",
                    )?;
                    contributors.reserve_exact(1);
                    contributors.push(DirectPenetrationContribution {
                        equipment: equipment.clone(),
                        damage_type: contribution.damage_type,
                        percent: contribution.percent,
                        flat: contribution.flat,
                        definition_id: origin.site.action.definition_id.to_string(),
                        source_file: origin.source_file.clone(),
                        field_path: origin.field_path.clone(),
                    });
                }
            }
        }
        if !contributors.is_empty() {
            let witness = DirectPenetrationWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor: owner,
                target,
                target_defense: self
                    .units
                    .get(&target)
                    .ok_or(GameError::InvalidAction)?
                    .stats
                    .defense,
                target_magic_resist: self
                    .units
                    .get(&target)
                    .ok_or(GameError::InvalidAction)?
                    .stats
                    .magic_resist,
                percent: total.physical_percent as u32,
                flat: total.physical_flat as u32,
                magic_percent: total.magic_percent as u32,
                magic_flat: total.magic_flat as u32,
                contributors,
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "direct_penetration_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_direct_penetration = Some(witness);
        }
        crate::authoring::control::poll("direct_penetration_evaluated")?;
        Ok(total)
    }
}
