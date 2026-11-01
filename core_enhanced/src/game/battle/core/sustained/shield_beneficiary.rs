//! Derive support directly from live personal HP pools and their current providers.
//! No mirror buff, membership cache, or independent expiry exists here.
use super::super::equipment_support::ShieldBeneficiaryDamageWitness;
use super::*;
use crate::game::{
    ability::equipment_support::CompiledEquipmentSupport,
    battle::{
        damage::DamageSourceSnapshot, runtime_control, runtime_facts::EquipmentSourceSnapshot,
        types::BattleUnitSourceIdentity,
    },
    enums::Side,
};
use std::mem::size_of;

impl BattleCore {
    pub(in crate::game::battle::core) fn shield_beneficiary_damage_percent(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        if !self
            .game_data
            .equipment_support_catalog()
            .has_shield_beneficiary_damage
        {
            return Ok(0);
        }
        let Some(beneficiary) = self.owned_equipment_damage_source(snapshot, target) else {
            return Ok(0);
        };
        control::charge(
            self.sustained.records.len() as u64,
            "shield_beneficiary_pools",
        )?;
        let mut selected: Option<(
            u32,
            EquipmentSourceSnapshot,
            Arc<CompiledEquipmentSupport>,
            u64,
        )> = None;
        for (activation, record) in &self.sustained.records {
            let Some(pool) = record.shield else {
                continue;
            };
            if pool.remaining == 0
                || pool.shared_radius_units.is_some()
                || record.segmented.is_some()
                || record.expires_at <= now
                || record.members.binary_search(&beneficiary).is_err()
                || !record.adaptive_source_valid(self)
                || (record
                    .structural
                    .as_ref()
                    .is_some_and(|s| s.personal_group.is_some())
                    && !self.personal_source_live(record))
                || !self.units.get(&record.caster).is_some_and(|u| {
                    u.is_active()
                        && u.is_combatant()
                        && !u.corroded
                        && u.stats.current_health > 0
                        && u.owner == Side::Player
                        && matches!(u.source_identity, BattleUnitSourceIdentity::Employee { .. })
                })
            {
                continue;
            }
            self.visit_recipient_supports(record.caster, |equipment, program| {
                let Some(percent) = program
                    .program
                    .shield_beneficiary_damage_percent
                    .filter(|p| *p > 0)
                else {
                    return Ok(());
                };
                let replace = selected.as_ref().is_none_or(|(p, e, _, id)| {
                    percent > *p
                        || (percent == *p
                            && (equipment.runtime_item_instance_id, *activation)
                                < (e.runtime_item_instance_id, *id))
                });
                if replace {
                    runtime_control::admit_retained_growth(
                        0,
                        Some(size_of::<(
                            u32,
                            EquipmentSourceSnapshot,
                            Arc<CompiledEquipmentSupport>,
                            u64,
                        )>()),
                        "shield_beneficiary_choice",
                    )?;
                    selected = Some((percent, equipment, Arc::clone(program), *activation));
                }
                Ok(())
            })?;
        }
        let Some((percent, equipment, program, activation)) = selected else {
            return Ok(0);
        };
        control::poll("shield_beneficiary_evaluated")?;
        if self.authoring_observation.is_some() {
            let record = &self.sustained.records[&activation];
            let suffix = "shield_beneficiary_damage_percent";
            let bytes = record.source.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<ShieldBeneficiaryDamageWitness>()
                        + program.definition_id.len()
                        + program.source_file.len()
                        + program.field_path.len()
                        + suffix.len()
                        + 1,
                )
            });
            runtime_control::admit_retained_growth(0, bytes, "shield_beneficiary_witness_copy")?;
            control::charge(
                bytes.ok_or(GameError::InvalidAction)? as u64,
                "shield_beneficiary_witness_copy",
            )?;
            let witness = ShieldBeneficiaryDamageWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor: snapshot.source_id,
                beneficiary,
                target,
                provider: record.caster,
                equipment,
                shield_activation: activation,
                shield_remaining: record.shield.unwrap().remaining,
                percent,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: format!("{}.{}", program.field_path, suffix),
                shield_source: record.source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "shield_beneficiary_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_shield_beneficiary_damage = Some(witness);
        }
        Ok(percent as i32)
    }
}
