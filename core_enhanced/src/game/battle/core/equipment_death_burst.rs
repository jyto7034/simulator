//! An actual death schedules one finite, copy-owned area delivery.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::equipment_attack::CompiledEquipmentAttack,
        battle::{
            damage::{DamageModifiers, DamageSource, DamageSourceSnapshot},
            enums::BattleEvent,
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
            tile_range::FacingDirection,
        },
        behavior::GameError,
        resources::Position,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    mem::{size_of, size_of_val},
    sync::Arc,
};

#[derive(Debug, Clone)]
struct Pending {
    victim: UnitInstanceId,
    equipment: EquipmentSourceSnapshot,
    center: Position,
    due: u64,
    program: Arc<CompiledEquipmentAttack>,
    snapshot: DamageSourceSnapshot,
}
crate::authoring::clone_shape::struct_shape!(Pending {
    victim,
    equipment,
    center,
    due,
    program,
    snapshot
});
impl Pending {
    fn retained_extent(&self) -> Option<usize> {
        (size_of::<Self>() + 128)
            .checked_add(self.program.retained_bytes)?
            .checked_add(self.snapshot.fact_retained_extent()?)
    }
}
#[derive(Debug, Clone, Default)]
pub(super) struct DeathBurstState {
    next: u64,
    pending: BTreeMap<u64, Pending>,
    seen: BTreeSet<(uuid::Uuid, UnitInstanceId)>,
}
crate::authoring::clone_shape::struct_shape!(DeathBurstState {
    next,
    pending,
    seen
});
impl DeathBurstState {
    pub(super) fn entity_count(&self) -> usize {
        self.pending.len().saturating_add(self.seen.len())
    }
    pub(super) fn retained_extent(&self) -> Option<usize> {
        let mut n = self
            .seen
            .len()
            .checked_mul(128 + size_of::<(uuid::Uuid, UnitInstanceId)>())?;
        for pending in self.pending.values() {
            n = n.checked_add(pending.retained_extent()?)?;
        }
        Some(n)
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum DeathBurstOperation {
    Reserved,
    Detonated,
    Cancelled,
}
crate::authoring::clone_shape::copy_shape!(DeathBurstOperation);
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeathBurstWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub time_ms: u64,
    pub id: u64,
    pub actor: UnitInstanceId,
    pub equipment_copy: uuid::Uuid,
    pub victim: UnitInstanceId,
    pub center: Position,
    pub due_ms: u64,
    pub attack: u32,
    pub base_damage: u32,
    pub operation: DeathBurstOperation,
    pub targets: u32,
    pub pending: usize,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(DeathBurstWitness {
    generation,
    time_ms,
    id,
    actor,
    equipment_copy,
    victim,
    center,
    due_ms,
    attack,
    base_damage,
    operation,
    targets,
    pending,
    definition_id,
    source_file,
    field_path
});
impl DeathBurstWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut n = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?;
        if let Some(g) = &self.generation {
            n = n
                .checked_add(size_of_val(g.as_ref()))?
                .checked_add(g.semantic_sha256.capacity())?;
        }
        Some(n)
    }
}

impl BattleCore {
    pub(super) fn equipment_death_burst_kill(
        &mut self,
        source: &DamageSourceSnapshot,
        victim: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if self.sustained.closed {
            return Ok(());
        }
        let chain = source.fact_source.death_burst_copy();
        if chain.is_none()
            && (source.fact_source.reaction
                || source.fact_source.guard_release().is_some()
                || source
                    .fact_source
                    .cast
                    .as_ref()
                    .is_some_and(|c| c.proc_origin.is_some())
                || !matches!(
                    source.source,
                    DamageSource::BasicAttack | DamageSource::Ability
                ))
        {
            return Ok(());
        }
        if !self.eligible_encounter_death(victim, source.source_side)? {
            return Ok(());
        }
        let center = self
            .sample_unit_body_at(victim, now)
            .ok_or(GameError::InvalidAction)?
            .projected_tile();
        for (equipment, program) in
            self.equipped_attack_providers(source.source_id, |p| p.death_burst.is_some())?
        {
            let copy = equipment.persistent_copy_uuid.unwrap();
            if chain.is_some_and(|origin| origin != copy)
                || self.sustained.death_bursts.seen.contains(&(copy, victim))
            {
                continue;
            }
            if chain.is_some()
                && (!source.fact_source.reaction
                    || source.source != DamageSource::Ability
                    || source.fact_source.equipment != Some(equipment)
                    || !source
                        .fact_source
                        .authored
                        .as_ref()
                        .is_some_and(|origin| Arc::ptr_eq(origin, &program.death_burst_source)))
            {
                continue;
            }
            let definition = program.death_burst.as_ref().unwrap();
            let attack = self.units[&source.source_id].stats.attack;
            let damage =
                u32::try_from(u64::from(attack) * u64::from(definition.attack_percent) / 100)
                    .map_err(|_| runtime_control::invariant("death_burst_damage_overflow"))?;
            let due = now
                .checked_add(u64::from(definition.delay_ms))
                .ok_or_else(|| runtime_control::invariant("death_burst_time_overflow"))?;
            let fact = RuntimeFactSource {
                actor: Some(source.source_id),
                equipment: Some(equipment),
                reaction: true,
                death_burst: true,
                authored: Some(Arc::clone(&program.death_burst_source)),
                ..Default::default()
            }
            .inherit_causal_context(&source.fact_source)?;
            let prior = std::mem::replace(&mut self.runtime_fact_source, fact);
            let snapshot = self.damage_source_snapshot_template_for_unit(
                source.source_id,
                DamageSource::Ability,
                definition.damage_type,
                damage,
                DamageModifiers::default(),
                0,
                now,
                false,
            );
            self.runtime_fact_source = prior;
            let mut snapshot = snapshot.ok_or(GameError::InvalidAction)?;
            // Reaction damage has no critical roll or basic attack captures.
            snapshot.crit_roll_percent = Some(100);
            let pending = Pending {
                victim,
                equipment,
                center,
                due,
                program,
                snapshot,
            };
            runtime_control::admit_retained_growth(
                0,
                pending
                    .retained_extent()
                    .and_then(|n| n.checked_add(128 + size_of::<(uuid::Uuid, UnitInstanceId)>())),
                "death_burst_storage",
            )?;
            self.admit_authoring_runtime_growth(2, "death_burst_entities")?;
            let id = self
                .sustained
                .death_bursts
                .next
                .checked_add(1)
                .ok_or_else(|| runtime_control::invariant("death_burst_id_overflow"))?;
            self.observe_death_burst(
                id,
                &pending,
                now,
                DeathBurstOperation::Reserved,
                0,
                self.sustained.death_bursts.pending.len() + 1,
            )?;
            self.event_queue.push_with_context(
                BattleEvent::EquipmentDeathBurst { time_ms: due, id },
                None,
                pending.snapshot.fact_source.clone(),
            );
            self.sustained.death_bursts.next = id;
            self.sustained.death_bursts.seen.insert((copy, victim));
            self.sustained.death_bursts.pending.insert(id, pending);
            control::poll("death_burst_reserved")?;
        }
        Ok(())
    }

    pub(super) fn reconcile_equipment_death_bursts(&mut self) -> Result<(), GameError> {
        let count = self.sustained.death_bursts.pending.len();
        if count == 0 {
            return Ok(());
        }
        control::charge(count as u64, "death_burst_reconcile")?;
        runtime_control::admit_retained_growth(
            0,
            count.checked_mul(128),
            "death_burst_cancel_ids",
        )?;
        let mut cancelled = BTreeSet::new();
        for (&id, pending) in &self.sustained.death_bursts.pending {
            let equipped = self
                .equipped_attack_providers(pending.snapshot.source_id, |p| p.death_burst.is_some())?
                .iter()
                .any(|(e, p)| {
                    *e == pending.equipment && p.fingerprint == pending.program.fingerprint
                });
            if self.sustained.closed || !equipped {
                cancelled.insert(id);
            }
        }
        for id in &cancelled {
            let pending = self.sustained.death_bursts.pending.remove(id).unwrap();
            self.observe_death_burst(
                *id,
                &pending,
                self.sustained_observed_time(),
                DeathBurstOperation::Cancelled,
                0,
                self.sustained.death_bursts.pending.len(),
            )?;
        }
        if !cancelled.is_empty() {
            self.event_queue
                .discard_equipment_death_bursts(Some(&cancelled))?;
        }
        Ok(())
    }

    pub(super) fn clear_equipment_death_bursts(&mut self) -> Result<(), GameError> {
        self.reconcile_equipment_death_bursts()?;
        self.sustained.death_bursts.pending.clear();
        self.sustained.death_bursts.seen.clear();
        self.event_queue.discard_equipment_death_bursts(None)
    }

    pub(super) fn equipment_death_burst_tick(
        &mut self,
        id: u64,
        now: u64,
    ) -> Result<(), GameError> {
        self.reconcile_sustained(now)?;
        if !self
            .sustained
            .death_bursts
            .pending
            .get(&id)
            .is_some_and(|p| p.due == now)
        {
            return Ok(());
        }
        let pending = self.sustained.death_bursts.pending.remove(&id).unwrap();
        let tiles = pending
            .program
            .death_burst
            .as_ref()
            .unwrap()
            .range
            .affected_tiles(pending.center, FacingDirection::Up)
            .map_err(GameError::InvalidStaticData)?;
        let targets = self.collect_tile_area_targets_at(
            now,
            pending.snapshot.source_side,
            pending.snapshot.source_id,
            &tiles,
            crate::game::ability::SkillHitTargetFilter::Enemies,
            false,
        );
        let area_percent = self.area_damage_percent(pending.snapshot.source_id, pending.snapshot.source_side,
            &pending.snapshot.fact_source, &targets, now)?;
        control::poll("death_burst_targets")?;
        let old = std::mem::replace(
            &mut self.runtime_fact_source,
            pending.snapshot.fact_source.clone(),
        );
        let scope = self.begin_strike_scope();
        let result = (|| {
            for &target in &targets {
                if !self.units.get(&target).is_some_and(|u| u.is_active()) {
                    continue;
                }
                runtime_control::admit_retained_growth(
                    0,
                    pending.snapshot.fact_retained_extent(),
                    "death_burst_hit_copy",
                )?;
                let mut snapshot = self.materialize_damage_source_snapshot_for_target_hit(
                    pending.snapshot.clone(),
                    target,
                    0,
                );
                snapshot.modifiers.damage_amp_percent = snapshot.modifiers.damage_amp_percent.saturating_add(area_percent);
                snapshot.committed_at_ms = now;
                self.execute_committed_damage(super::damage_runtime::CommittedDamageInput {
                    target_id: target,
                    source_snapshot: snapshot,
                    committed_at_ms: now,
                    policy: super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
                })
                .ok_or(GameError::InvalidAction)?;
                control::poll("death_burst_hit")?;
            }
            self.observe_death_burst(
                id,
                &pending,
                now,
                DeathBurstOperation::Detonated,
                targets.len() as u32,
                self.sustained.death_bursts.pending.len(),
            )
        })();
        self.end_strike_scope(scope);
        self.runtime_fact_source = old;
        result
    }

    fn observe_death_burst(
        &mut self,
        id: u64,
        pending: &Pending,
        now: u64,
        operation: DeathBurstOperation,
        targets: u32,
        count: usize,
    ) -> Result<(), GameError> {
        if self.authoring_observation.is_none() {
            return Ok(());
        }
        let source = &pending.program.death_burst_source;
        let bytes = size_of::<DeathBurstWitness>()
            + source.site.action.definition_id.as_str().len()
            + source.source_file.len()
            + source.field_path.len();
        runtime_control::admit_retained_growth(0, Some(bytes), "death_burst_witness_copy")?;
        control::charge(bytes as u64, "death_burst_witness_copy")?;
        let witness = DeathBurstWitness {
            generation: self.content_identity.clone(),
            time_ms: now,
            id,
            actor: pending.snapshot.source_id,
            equipment_copy: pending.equipment.persistent_copy_uuid.unwrap(),
            victim: pending.victim,
            center: pending.center,
            due_ms: pending.due,
            attack: pending.snapshot.source_attack,
            base_damage: pending.snapshot.base_damage,
            operation,
            targets,
            pending: count,
            definition_id: source.site.action.definition_id.to_string(),
            source_file: source.source_file.clone(),
            field_path: source.field_path.clone(),
        };
        runtime_control::admit_retained_growth(
            0,
            witness.retained_extent(),
            "death_burst_observation",
        )?;
        self.authoring_observation
            .as_mut()
            .unwrap()
            .latest_equipment_death_burst = Some(witness);
        Ok(())
    }
}
