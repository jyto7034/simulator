//! Direct allied healing reactions; pools remain ordinary sustained authority.
use super::super::equipment_support::{HealBarrierWitness, SupportChannel};
use super::*;
use crate::game::{
    ability::equipment_support::CompiledEquipmentSupport,
    battle::{runtime_control, runtime_facts::EquipmentSourceSnapshot},
};
use std::mem::size_of;
use uuid::Uuid;

type GateKey = (Uuid, UnitInstanceId);
// A missing action uses a unique local occurrence, never a fabricated action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PulseKey {
    copy: Uuid,
    target: UnitInstanceId,
    action: Option<u64>,
    at: u64,
    nonce: u64,
}
crate::authoring::clone_shape::copy_shape!(PulseKey);
#[derive(Debug, Clone, Copy)]
struct Cooldown {
    owner: UnitInstanceId,
    ready_at: u64,
}
crate::authoring::clone_shape::copy_shape!(Cooldown);
#[derive(Debug, Clone, Copy)]
struct Pulse {
    owner: UnitInstanceId,
    hp_before: u32,
    max_hp: u32,
    actual: u64,
    total_capacity: u32,
    activation: Option<u64>,
    granted: bool,
    ready_at: Option<u64>,
}
crate::authoring::clone_shape::copy_shape!(Pulse);
#[derive(Debug, Clone, Default)]
pub(super) struct HealBarrierState {
    next: u64,
    cooldowns: BTreeMap<GateKey, Cooldown>,
    pulses: BTreeMap<PulseKey, Pulse>,
}
crate::authoring::clone_shape::struct_shape!(HealBarrierState {
    next,
    cooldowns,
    pulses
});
impl HealBarrierState {
    pub(super) fn entity_count(&self) -> usize {
        self.cooldowns.len().saturating_add(self.pulses.len())
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        self.cooldowns
            .len()
            .checked_mul(size_of::<GateKey>() + size_of::<Cooldown>() + 96)?
            .checked_add(
                self.pulses
                    .len()
                    .checked_mul(size_of::<PulseKey>() + size_of::<Pulse>() + 96)?,
            )
    }
}
impl BattleCore {
    pub(super) fn reconcile_heal_barriers(&mut self, now: u64) -> Result<(), GameError> {
        let state = &mut self.sustained.heal_barriers;
        control::charge(
            (state.cooldowns.len() + state.pulses.len()) as u64,
            "heal_barrier_reconcile",
        )?;
        let valid = |copy: &Uuid, owner: UnitInstanceId, target: UnitInstanceId| {
            self.units.get(&owner).is_some_and(|u| {
                u.is_active()
                    && !u.corroded
                    && u.equipment_slots.as_ref().is_none_or(|s| {
                        s.slots
                            .iter()
                            .flatten()
                            .any(|s| s.runtime_item_instance_id == *copy)
                    })
            }) && self
                .units
                .get(&target)
                .is_some_and(|u| u.is_active() && !u.corroded)
                && self.items.get(copy).is_some_and(|i| {
                    i.owner_unit_instance == owner
                        && self
                            .game_data
                            .equipment_support_catalog()
                            .by_equipment
                            .get(&i.base_uuid)
                            .is_some_and(|p| p.program.direct_heal_barrier.is_some())
                })
        };
        state
            .cooldowns
            .retain(|(copy, target), v| v.ready_at > now && valid(copy, v.owner, *target));
        state
            .pulses
            .retain(|key, v| key.at == now && valid(&key.copy, v.owner, key.target));
        Ok(())
    }
    pub(in crate::game::battle::core) fn heal_barrier_after_direct_heal(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        hp_before: u32,
        max_hp: u32,
        actual: u32,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.equipment_support_pair_eligible(actor, target)
            || actual == 0
            || self.sustained.closed
        {
            return Ok(());
        }
        let data = Arc::clone(&self.game_data);
        let catalog = data.equipment_support_catalog();
        control::charge(catalog.by_equipment.len() as u64, "heal_barrier_catalog")?;
        if !catalog
            .by_equipment
            .values()
            .any(|p| p.program.direct_heal_barrier.is_some())
        {
            return Ok(());
        }
        self.reconcile_heal_barriers(now)?;
        let action = self
            .runtime_fact_source
            .action
            .as_ref()
            .map(|a| a.ordinal().0);
        let nonce = if action.is_none() {
            self.sustained.heal_barriers.next = self
                .sustained
                .heal_barriers
                .next
                .checked_add(1)
                .ok_or_else(|| overflow("heal_barrier_occurrence"))?;
            self.sustained.heal_barriers.next
        } else {
            0
        };
        let mut cursor = None;
        loop {
            control::charge(self.items.len() as u64, "heal_barrier_providers")?;
            let selected = self
                .items
                .iter()
                .filter(|(id, i)| {
                    i.owner_unit_instance == actor
                        && cursor.is_none_or(|c| **id > c)
                        && self.units[&actor].equipment_slots.as_ref().is_none_or(|s| {
                            s.slots
                                .iter()
                                .flatten()
                                .any(|s| s.runtime_item_instance_id == **id)
                        })
                })
                .filter_map(|(id, item)| {
                    catalog
                        .by_equipment
                        .get(&item.base_uuid)
                        .filter(|p| p.program.direct_heal_barrier.is_some())
                        .map(|p| (*id, item.source_snapshot(), Arc::clone(p)))
                })
                .min_by_key(|(id, _, _)| *id);
            let Some((id, equipment, plan)) = selected else {
                break;
            };
            cursor = Some(id);
            self.apply_heal_barrier(
                actor,
                target,
                hp_before,
                max_hp,
                actual,
                now,
                equipment,
                &plan,
                PulseKey {
                    copy: id,
                    target,
                    action,
                    at: now,
                    nonce,
                },
            )?;
        }
        Ok(())
    }
    fn apply_heal_barrier(
        &mut self,
        actor: UnitInstanceId,
        target: UnitInstanceId,
        hp_before: u32,
        max_hp: u32,
        actual: u32,
        now: u64,
        equipment: EquipmentSourceSnapshot,
        plan: &CompiledEquipmentSupport,
        key: PulseKey,
    ) -> Result<(), GameError> {
        let program = plan
            .program
            .direct_heal_barrier
            .expect("selected barrier program");
        let gate = (equipment.runtime_item_instance_id, target);
        let mut pulse = if let Some(p) = self.sustained.heal_barriers.pulses.get(&key) {
            *p
        } else {
            if u64::from(hp_before) * 100
                > u64::from(max_hp.max(1)) * u64::from(program.hp_threshold_percent)
                || self
                    .sustained
                    .heal_barriers
                    .cooldowns
                    .get(&gate)
                    .is_some_and(|c| now < c.ready_at)
            {
                return Ok(());
            }
            runtime_control::admit_retained_growth(
                0,
                Some(size_of::<PulseKey>() + size_of::<Pulse>() + 96),
                "heal_barrier_pulse",
            )?;
            self.admit_authoring_runtime_growth(1, "heal_barrier_pulse")?;
            Pulse {
                owner: actor,
                hp_before,
                max_hp,
                actual: 0,
                total_capacity: 0,
                activation: None,
                granted: false,
                ready_at: None,
            }
        };
        self.sustained.heal_barriers.pulses.insert(key, pulse);
        pulse.actual = pulse
            .actual
            .checked_add(u64::from(actual))
            .ok_or_else(|| overflow("heal_barrier_actual_sum"))?;
        let base = (u128::from(pulse.actual) * u128::from(program.actual_heal_percent) / 100)
            .min(u128::from(u32::MAX)) as u32;
        let total =
            self.equipment_support_amount(actor, target, base, SupportChannel::Shield, now)?;
        if total == 0 {
            self.sustained.heal_barriers.pulses.insert(key, pulse);
            return Ok(());
        }
        if pulse.ready_at.is_none() {
            let ready_at = now
                .checked_add(u64::from(program.cooldown_ms))
                .ok_or_else(|| overflow("heal_barrier_cooldown"))?;
            runtime_control::admit_retained_growth(
                0,
                Some(size_of::<GateKey>() + size_of::<Cooldown>() + 96),
                "heal_barrier_cooldown",
            )?;
            self.admit_authoring_runtime_growth(1, "heal_barrier_cooldown")?;
            self.sustained.heal_barriers.cooldowns.insert(
                gate,
                Cooldown {
                    owner: actor,
                    ready_at,
                },
            );
            pulse.ready_at = Some(ready_at);
        }
        // A previous contribution can have absorbed damage or been replaced.
        // Only the newly earned amount can replenish a contribution we granted.
        let own = pulse
            .activation
            .and_then(|id| self.sustained.records.get(&id))
            .filter(|r| r.expires_at > now);
        let spent = if pulse.granted {
            pulse
                .total_capacity
                .saturating_sub(own.and_then(|r| r.shield).map_or(0, |p| p.remaining))
        } else {
            0
        };
        let candidate = total.saturating_sub(spent);
        control::charge(self.sustained.records.len() as u64, "heal_barrier_existing")?;
        let existing = self
            .sustained
            .records
            .iter()
            .find(|(_, r)| r.heal_barrier && r.target == target && r.expires_at > now)
            .map(|(id, r)| (*id, r.shield.expect("barrier pool"), r.expires_at));
        let mut kept_existing = existing.is_some();
        let (remaining, expires_at) = if let Some((id, pool, expires)) =
            existing.filter(|(id, _, _)| Some(*id) == pulse.activation)
        {
            let addition = total.saturating_sub(pulse.total_capacity);
            let value = self
                .sustained
                .records
                .get_mut(&id)
                .unwrap()
                .shield
                .as_mut()
                .unwrap();
            value.capacity = value.capacity.saturating_add(addition);
            value.remaining = value.remaining.saturating_add(addition);
            kept_existing = false;
            (pool.remaining.saturating_add(addition), expires)
        } else if candidate > existing.map_or(0, |(_, p, _)| p.remaining) {
            let expires = now
                .checked_add(u64::from(program.duration_ms))
                .ok_or_else(|| overflow("heal_barrier_expiry"))?;
            if let Some((id, _, _)) = existing {
                self.end_sustained(id, now, SustainedEndReason::Replaced)?;
            }
            let source = RuntimeFactSource {
                actor: Some(actor),
                equipment: Some(equipment),
                authored: plan.barrier_source.clone(),
                ..Default::default()
            };
            let prior = std::mem::replace(&mut self.runtime_fact_source, source);
            let before = self.sustained.next;
            let result = self.begin_sustained_owned_with_shield(
                actor,
                target,
                SustainedEffectDef::IndependentShield {
                    duration_ms: program.duration_ms,
                    amount: candidate,
                    priority: program.priority,
                },
                now,
                None,
                None,
                None,
                Some(candidate),
            );
            self.runtime_fact_source = prior;
            result?;
            if self.sustained.next == before {
                return Err(runtime_control::invariant("heal_barrier_start"));
            }
            let id = self.sustained.next;
            self.sustained.records.get_mut(&id).unwrap().heal_barrier = true;
            pulse.activation = Some(id);
            pulse.granted = true;
            kept_existing = false;
            (candidate, expires)
        } else {
            existing.map_or((0, now), |(_, p, e)| (p.remaining, e))
        };
        pulse.total_capacity = total.max(pulse.total_capacity);
        self.sustained.heal_barriers.pulses.insert(key, pulse);
        if self.authoring_observation.is_some() {
            let path = plan.barrier_source.as_ref().expect("barrier source");
            let bytes = size_of::<HealBarrierWitness>()
                .checked_add(plan.definition_id.len())
                .and_then(|n| n.checked_add(path.source_file.len()))
                .and_then(|n| n.checked_add(path.field_path.len()))
                .and_then(|n| {
                    self.runtime_fact_source
                        .retained_extent()
                        .and_then(|b| n.checked_add(b))
                });
            runtime_control::admit_retained_growth(0, bytes, "heal_barrier_observation")?;
            control::charge(
                bytes.unwrap_or(usize::MAX) as u64,
                "heal_barrier_observation_copy",
            )?;
            let witness = HealBarrierWitness {
                generation: self.content_identity.clone(),
                time_ms: now,
                actor,
                target,
                equipment,
                definition_id: plan.definition_id.clone(),
                source_file: path.source_file.clone(),
                field_path: path.field_path.clone(),
                hp_before: pulse.hp_before,
                max_hp: pulse.max_hp,
                actual_healed: pulse.actual,
                base_capacity: base,
                increased_capacity: total,
                remaining,
                expires_at_ms: expires_at,
                ready_at_ms: pulse.ready_at.unwrap(),
                kept_existing,
                healing_source: self.runtime_fact_source.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "heal_barrier_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_heal_barrier = Some(witness);
        }
        control::poll("heal_barrier_applied")
    }
}
