//! A normal primary hit grants one finite, type-filtered personal pool.
use super::*;
use crate::game::battle::{
    damage::{DamageSource, DamageSourceSnapshot, DamageType},
    runtime_facts::ActionHandle,
};

#[derive(Debug, Clone)]
pub(super) struct AdaptiveUse {
    pub(super) owner: UnitInstanceId,
    pub(super) action: ActionHandle,
}
crate::authoring::clone_shape::struct_shape!(AdaptiveUse { owner, action });

/// Immutable decision evidence. Pool balance and cooldown have separate owners.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdaptiveShieldWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub primary: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub damage_type: DamageType,
    pub max_hp: u32,
    pub capacity: u32,
    pub activation: u64,
    pub time_ms: u64,
    pub expires_at_ms: u64,
    pub ready_at_ms: u64,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub attack_source: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(AdaptiveShieldWitness {
    generation,
    actor,
    primary,
    equipment,
    damage_type,
    max_hp,
    capacity,
    activation,
    time_ms,
    expires_at_ms,
    ready_at_ms,
    definition_id,
    source_file,
    field_path,
    attack_source
});
impl AdaptiveShieldWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.attack_source.retained_extent()?)?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}
impl BattleCore {
    pub(in crate::game::battle::core) fn adaptive_shield_copy_valid(
        &self,
        owner: UnitInstanceId,
        copy: uuid::Uuid,
    ) -> bool {
        self.units.get(&owner).is_some_and(|u| {
            u.is_active()
                && !u.corroded
                && u.equipment_slots.as_ref().is_none_or(|s| {
                    s.slots
                        .iter()
                        .flatten()
                        .any(|s| s.runtime_item_instance_id == copy)
                })
        }) && self.items.get(&copy).is_some_and(|i| {
            i.owner_unit_instance == owner
                && self
                    .game_data
                    .equipment_defense_catalog()
                    .by_equipment
                    .get(&i.base_uuid)
                    .is_some_and(|p| {
                        matches!(
                            p.program,
                            EquipmentDefenseProgram::AdaptiveBasicShield { .. }
                        )
                    })
        })
    }

    /// Read the opponent before its lethal hit is finalized. Never infer the
    /// primary from whichever secondary contact happened to arrive first.
    pub(in crate::game::battle::core) fn adaptive_shield_target_type(
        &self,
        snapshot: &DamageSourceSnapshot,
        target: UnitInstanceId,
    ) -> Option<DamageType> {
        if !self.game_data.equipment_defense_catalog().has_adaptive {
            return None;
        }
        if snapshot.source != DamageSource::BasicAttack
            || snapshot.basic_primary != Some(target)
            || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
            || snapshot.fact_source.action.is_none()
            || snapshot.source_id == target
        {
            return None;
        }
        let target = self.units.get(&target).filter(|u| {
            u.is_active()
                && u.is_combatant()
                && u.owner != snapshot.source_side
                && !u.basic_attack.action.is_heal()
        })?;
        match target.basic_attack.damage_type {
            DamageType::Physical | DamageType::Magic => Some(target.basic_attack.damage_type),
            DamageType::True => None,
        }
    }

    pub(in crate::game::battle::core) fn adaptive_shield_after_hit(
        &mut self,
        snapshot: &DamageSourceSnapshot,
        primary: UnitInstanceId,
        damage_type: DamageType,
        now: u64,
    ) -> Result<(), GameError> {
        let actor = snapshot.source_id;
        let Some(unit) = self
            .units
            .get(&actor)
            .filter(|u| u.is_active() && !u.corroded)
        else {
            return Ok(());
        };
        let max_hp = unit.stats.max_health;
        let Some(action) = snapshot.fact_source.action.as_ref() else {
            return Ok(());
        };
        if self.runtime_facts.validate_live_action(action).is_err() {
            return Ok(());
        }
        self.reconcile_sustained(now)?;
        let mut cursor = None;
        loop {
            let mut selected = None;
            self.visit_equipment_defenses(actor, |equipment, program| {
                let copy = equipment.runtime_item_instance_id;
                if cursor.is_some_and(|old| copy <= old)
                    || !matches!(
                        program.program,
                        EquipmentDefenseProgram::AdaptiveBasicShield { .. }
                    )
                    || self
                        .equipment_defense
                        .cooldowns
                        .get(&copy)
                        .is_some_and(|c| now < c.ready_at_ms)
                    || self
                        .equipment_defense
                        .adaptive_uses
                        .contains_key(&(copy, action.ordinal().0))
                {
                    return;
                }
                if selected
                    .as_ref()
                    .is_none_or(|(old, _): &(EquipmentSourceSnapshot, _)| {
                        copy < old.runtime_item_instance_id
                    })
                {
                    selected = Some((equipment, Arc::clone(program)));
                }
            })?;
            let Some((equipment, program)) = selected else {
                break;
            };
            let copy = equipment.runtime_item_instance_id;
            cursor = Some(copy);
            if self.has_adaptive_shield(actor, copy)? {
                continue;
            }
            let EquipmentDefenseProgram::AdaptiveBasicShield {
                max_hp_percent,
                duration_ms,
                cooldown_ms,
                priority,
            } = program.program
            else {
                return Err(runtime_control::invariant("adaptive_shield_program"));
            };
            let capacity = u32::try_from(u64::from(max_hp) * u64::from(max_hp_percent) / 100)
                .map_err(|_| runtime_control::invariant("adaptive_shield_capacity"))?;
            if capacity == 0 {
                continue;
            }
            let ready_at_ms = now
                .checked_add(u64::from(cooldown_ms))
                .ok_or_else(|| runtime_control::invariant("adaptive_shield_deadline"))?;
            let expires_at_ms = now
                .checked_add(u64::from(duration_ms))
                .ok_or_else(|| runtime_control::invariant("adaptive_shield_deadline"))?;
            let retained = self.equipment_defense.retained_extent();
            runtime_control::admit_retained_growth(
                retained.unwrap_or(usize::MAX),
                retained.and_then(|n| {
                    n.checked_add(
                        size_of::<BarrierCooldown>()
                            + size_of::<uuid::Uuid>()
                            + 96
                            + size_of::<AdaptiveUse>()
                            + size_of::<(uuid::Uuid, u64)>()
                            + 192,
                    )
                }),
                "adaptive_shield_gate_storage",
            )?;
            let source = RuntimeFactSource {
                actor: Some(actor),
                equipment: Some(equipment),
                authored: program.adaptive_source.clone(),
                ..Default::default()
            };
            let activation = self.begin_adaptive_shield(
                actor,
                capacity,
                damage_type,
                duration_ms,
                priority,
                source,
                now,
            )?;
            self.equipment_defense.cooldowns.insert(
                copy,
                BarrierCooldown {
                    owner: actor,
                    ready_at_ms,
                },
            );
            self.equipment_defense.adaptive_uses.insert(
                (copy, action.ordinal().0),
                AdaptiveUse {
                    owner: actor,
                    action: action.clone(),
                },
            );
            if self.authoring_observation.is_some() {
                let bytes = snapshot.fact_source.retained_extent().and_then(|n| {
                    n.checked_add(
                        size_of::<AdaptiveShieldWitness>()
                            + program.definition_id.len()
                            + program.source_file.len()
                            + program.field_path.len(),
                    )
                });
                runtime_control::admit_retained_growth(0, bytes, "adaptive_shield_witness")?;
                control::charge(
                    bytes.unwrap_or(usize::MAX) as u64,
                    "adaptive_shield_witness_copy",
                )?;
                let witness = AdaptiveShieldWitness {
                    generation: self.content_identity.clone(),
                    actor,
                    primary,
                    equipment,
                    damage_type,
                    max_hp,
                    capacity,
                    activation,
                    time_ms: now,
                    expires_at_ms,
                    ready_at_ms,
                    definition_id: program.definition_id.clone(),
                    source_file: program.source_file.clone(),
                    field_path: program.field_path.clone(),
                    attack_source: snapshot.fact_source.observation(),
                };
                runtime_control::admit_retained_growth(
                    0,
                    witness.retained_extent(),
                    "adaptive_shield_witness",
                )?;
                self.authoring_observation
                    .as_mut()
                    .unwrap()
                    .latest_adaptive_shield = Some(witness);
            }
            control::poll("adaptive_shield_applied")?;
        }
        Ok(())
    }
}
