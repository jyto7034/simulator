//! Received overheal belongs to the equipped copy for this battle only.
use super::*;

#[derive(Debug, Clone, Copy)]
pub(in crate::game::battle::core) struct ReserveBalance {
    pub(in crate::game::battle::core) owner: UnitInstanceId,
    pub(in crate::game::battle::core) amount: u32,
}
crate::authoring::clone_shape::copy_shape!(ReserveBalance);

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum ReserveOperation {
    Store,
    Release,
}
crate::authoring::clone_shape::copy_shape!(ReserveOperation);

#[derive(Debug, Clone, serde::Serialize)]
pub struct OverhealReserveWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub operation: ReserveOperation,
    pub capacity: u32,
    pub stored_before: u32,
    pub stored_after: u32,
    pub incoming_overheal: u32,
    pub actual_healed: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(OverhealReserveWitness {
    generation,
    target,
    equipment,
    time_ms,
    operation,
    capacity,
    stored_before,
    stored_after,
    incoming_overheal,
    actual_healed,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl OverhealReserveWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(std::mem::size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}
fn capacity(max_hp: u32, percent: u8) -> u32 {
    (u64::from(max_hp) * u64::from(percent) / 100) as u32
}
impl BattleCore {
    pub(in crate::game::battle::core) fn reconcile_overheal_reserves(
        &mut self,
    ) -> Result<(), GameError> {
        control::charge(
            self.equipment_defense.reserves.len() as u64,
            "overheal_reserve_reconcile",
        )?;
        let catalog = self.game_data.equipment_defense_catalog();
        self.equipment_defense.reserves.retain(|copy, reserve| {
            let Some(unit) = self
                .units
                .get(&reserve.owner)
                .filter(|u| u.is_active() && !u.corroded && u.is_combatant())
            else {
                return false;
            };
            if unit.equipment_slots.as_ref().is_some_and(|s| {
                !s.slots
                    .iter()
                    .flatten()
                    .any(|s| s.runtime_item_instance_id == *copy)
            }) {
                return false;
            }
            let Some(item) = self
                .items
                .get(copy)
                .filter(|i| i.owner_unit_instance == reserve.owner)
            else {
                return false;
            };
            let Some(program) = catalog.by_equipment.get(&item.base_uuid) else {
                return false;
            };
            let EquipmentDefenseProgram::OverhealReserve {
                max_hp_cap_percent, ..
            } = program.program
            else {
                return false;
            };
            reserve.amount = reserve
                .amount
                .min(capacity(unit.stats.max_health, max_hp_cap_percent));
            reserve.amount > 0
        });
        Ok(())
    }

    fn next_reserve_provider(
        &self,
        target: UnitInstanceId,
        cursor: Option<uuid::Uuid>,
    ) -> Result<Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentDefense>)>, GameError> {
        let mut selected = None;
        self.visit_equipment_defenses(target, |equipment, plan| {
            let copy = equipment.runtime_item_instance_id;
            if cursor.is_some_and(|c| copy <= c)
                || !matches!(
                    plan.program,
                    EquipmentDefenseProgram::OverhealReserve { .. }
                )
            {
                return;
            }
            if selected
                .as_ref()
                .is_none_or(|(old, _): &(EquipmentSourceSnapshot, _)| {
                    copy < old.runtime_item_instance_id
                })
            {
                selected = Some((equipment, Arc::clone(plan)));
            }
        })?;
        Ok(selected)
    }

    pub(in crate::game::battle::core) fn store_overheal_reserve(
        &mut self,
        target: UnitInstanceId,
        overheal: u32,
        now: u64,
    ) -> Result<(), GameError> {
        if overheal == 0 || !self.game_data.equipment_defense_catalog().has_reserve {
            return Ok(());
        }
        self.reconcile_overheal_reserves()?;
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && !u.corroded && u.is_combatant())
        else {
            return Ok(());
        };
        let max_hp = unit.stats.max_health;
        let mut cursor = None;
        while let Some((equipment, plan)) = self.next_reserve_provider(target, cursor)? {
            let copy = equipment.runtime_item_instance_id;
            cursor = Some(copy);
            let EquipmentDefenseProgram::OverhealReserve {
                max_hp_cap_percent, ..
            } = plan.program
            else {
                unreachable!()
            };
            let cap = capacity(max_hp, max_hp_cap_percent);
            let before = self
                .equipment_defense
                .reserves
                .get(&copy)
                .map_or(0, |r| r.amount);
            let after = before.saturating_add(overheal).min(cap);
            if before == after {
                continue;
            }
            if !self.equipment_defense.reserves.contains_key(&copy) {
                let retained = self.equipment_defense.retained_extent();
                runtime_control::admit_retained_growth(
                    retained.unwrap_or(usize::MAX),
                    retained.and_then(|n| {
                        n.checked_add(size_of::<ReserveBalance>() + size_of::<uuid::Uuid>() + 96)
                    }),
                    "overheal_reserve_storage",
                )?;
            }
            let incoming = &self.runtime_fact_source;
            let witness = self.reserve_witness(
                target,
                equipment,
                &plan,
                now,
                ReserveOperation::Store,
                cap,
                before,
                after,
                overheal,
                0,
                incoming,
            )?;
            self.equipment_defense.reserves.insert(
                copy,
                ReserveBalance {
                    owner: target,
                    amount: after,
                },
            );
            if let Some(o) = &mut self.authoring_observation {
                o.latest_overheal_reserve = witness;
            }
            control::poll("overheal_reserve_stored")?;
        }
        Ok(())
    }

    pub(in crate::game::battle::core) fn release_overheal_reserve(
        &mut self,
        target: UnitInstanceId,
        incoming: &RuntimeFactSource,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_reserve {
            return Ok(());
        }
        self.reconcile_overheal_reserves()?;
        let mut cursor = None;
        while let Some((equipment, plan)) = self.next_reserve_provider(target, cursor)? {
            let copy = equipment.runtime_item_instance_id;
            cursor = Some(copy);
            let Some(unit) = self
                .units
                .get(&target)
                .filter(|u| u.is_active() && !u.corroded && u.is_combatant())
            else {
                break;
            };
            let EquipmentDefenseProgram::OverhealReserve {
                max_hp_cap_percent,
                trigger_hp_percent,
            } = plan.program
            else {
                unreachable!()
            };
            let max_hp = unit.stats.max_health;
            let hp = unit.stats.current_health;
            if hp == 0 || u64::from(hp) * 100 > u64::from(max_hp) * u64::from(trigger_hp_percent) {
                continue;
            }
            let before = self
                .equipment_defense
                .reserves
                .get(&copy)
                .map_or(0, |r| r.amount);
            let amount = before.min(max_hp.saturating_sub(hp)).min(i32::MAX as u32);
            if amount == 0 {
                continue;
            }
            // The incoming action is evidence, not this armor's execution authority.
            let source = RuntimeFactSource {
                actor: Some(target),
                equipment: Some(equipment),
                authored: plan.reserve_source.clone(),
                reaction: true,
                use_control: false,
                ..Default::default()
            };
            source.admit_owner("overheal_reserve_source")?;
            let old = std::mem::replace(&mut self.runtime_fact_source, source);
            self.apply_hp_delta_with_reserve(
                Some(target),
                target,
                amount as i32,
                now,
                crate::game::battle::event_log::HpChangeReason::Command,
                false,
            );
            self.runtime_fact_source = old;
            control::poll("overheal_reserve_healed")?;
            let actual = self
                .units
                .get(&target)
                .map_or(0, |u| u.stats.current_health.saturating_sub(hp));
            let after = before
                .checked_sub(actual)
                .ok_or_else(|| runtime_control::invariant("overheal_reserve_consumption"))?;
            let witness = self.reserve_witness(
                target,
                equipment,
                &plan,
                now,
                ReserveOperation::Release,
                capacity(max_hp, max_hp_cap_percent),
                before,
                after,
                0,
                actual,
                incoming,
            )?;
            if after == 0 {
                self.equipment_defense.reserves.remove(&copy);
            } else {
                self.equipment_defense.reserves.insert(
                    copy,
                    ReserveBalance {
                        owner: target,
                        amount: after,
                    },
                );
            }
            if let Some(o) = &mut self.authoring_observation {
                o.latest_overheal_reserve = witness;
            }
            control::poll("overheal_reserve_released")?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn reserve_witness(
        &self,
        target: UnitInstanceId,
        equipment: EquipmentSourceSnapshot,
        plan: &CompiledEquipmentDefense,
        now: u64,
        operation: ReserveOperation,
        capacity: u32,
        stored_before: u32,
        stored_after: u32,
        incoming_overheal: u32,
        actual_healed: u32,
        incoming: &RuntimeFactSource,
    ) -> Result<Option<OverhealReserveWitness>, GameError> {
        if self.authoring_observation.is_none() {
            return Ok(None);
        }
        runtime_control::admit_retained_growth(
            0,
            incoming.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<OverhealReserveWitness>()
                        + plan.definition_id.len()
                        + plan.source_file.len()
                        + plan.field_path.len(),
                )
            }),
            "overheal_reserve_witness",
        )?;
        control::charge(
            incoming.retained_extent().unwrap_or(usize::MAX) as u64,
            "overheal_reserve_witness_copy",
        )?;
        let witness = OverhealReserveWitness {
            generation: self.content_identity.clone(),
            target,
            equipment,
            time_ms: now,
            operation,
            capacity,
            stored_before,
            stored_after,
            incoming_overheal,
            actual_healed,
            definition_id: plan.definition_id.clone(),
            source_file: plan.source_file.clone(),
            field_path: plan.field_path.clone(),
            incoming: incoming.observation(),
        };
        runtime_control::admit_retained_growth(
            0,
            witness.retained_extent(),
            "overheal_reserve_observation",
        )?;
        Ok(Some(witness))
    }
}
