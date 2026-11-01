//! The admitted regular attack owns the clock. Hits only read it.
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::equipment_attack::CompiledEquipmentAttack,
        battle::{
            damage::{DamageSource, DamageSourceSnapshot},
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::EquipmentSourceSnapshot,
        },
        behavior::GameError,
    },
};
use std::{mem::size_of, sync::Arc};

#[derive(Debug, Clone)]
pub(super) struct AttackClock {
    pub owner: UnitInstanceId,
    pub employee: uuid::Uuid,
    pub equipment: EquipmentSourceSnapshot,
    pub program: Arc<CompiledEquipmentAttack>,
    pub started: u64,
    pub through: u64,
}
crate::authoring::clone_shape::struct_shape!(AttackClock {
    owner,
    employee,
    equipment,
    program,
    started,
    through
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct AttackContinuityWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub actor: UnitInstanceId,
    pub target: UnitInstanceId,
    pub time_ms: u64,
    pub started_ms: u64,
    pub through_ms: u64,
    pub elapsed_ms: u64,
    pub percent: u32,
    pub equipment: EquipmentSourceSnapshot,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
}
crate::authoring::clone_shape::struct_shape!(AttackContinuityWitness {
    generation,
    actor,
    target,
    time_ms,
    started_ms,
    through_ms,
    elapsed_ms,
    percent,
    equipment,
    definition_id,
    source_file,
    field_path
});
impl AttackContinuityWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.generation.as_ref().map_or(0, |g| {
                std::mem::size_of_val(g.as_ref()) + g.semantic_sha256.capacity()
            }))
    }
}

impl BattleCore {
    pub(super) fn break_attack_continuity(
        &mut self,
        owner: UnitInstanceId,
    ) -> Result<(), GameError> {
        control::charge(
            self.equipment_attack_state.continuity.len() as u64,
            "attack_continuity_break",
        )?;
        self.equipment_attack_state
            .continuity
            .retain(|_, clock| clock.owner != owner);
        Ok(())
    }

    pub(super) fn reconcile_attack_continuity(&mut self, now: u64) -> Result<(), GameError> {
        if self.equipment_attack_state.continuity.is_empty() {
            return Ok(());
        }
        if self.sustained.closed {
            self.equipment_attack_state.continuity.clear();
            return Ok(());
        }
        control::charge(
            self.equipment_attack_state.continuity.len() as u64,
            "attack_continuity_reconcile",
        )?;
        // No allocation or independent timer: expire at the existing event boundaries.
        loop {
            let mut stale = None;
            for (key, clock) in &self.equipment_attack_state.continuity {
                control::charge(1, "attack_continuity_live_copy")?;
                let live = now <= clock.through
                    && self.units.get(&clock.owner).is_some_and(|u| {
                        u.is_active()
                            && !u.corroded
                            && u.source_owned_uuid == clock.employee
                            && u.equipment_slots.as_ref().is_none_or(|slots| {
                                slots
                                    .slots
                                    .iter()
                                    .flatten()
                                    .any(|slot| slot.runtime_item_instance_id == *key)
                            })
                    })
                    && self.items.get(key).is_some_and(|item| {
                        item.owner_unit_instance == clock.owner
                            && item.source_snapshot() == clock.equipment
                    })
                    && self
                        .active_hard_cc_release_time_ms(clock.owner, now)
                        .is_none();
                if !live {
                    stale = Some(*key);
                    break;
                }
            }
            let Some(key) = stale else { break };
            self.equipment_attack_state.continuity.remove(&key);
        }
        Ok(())
    }

    pub(super) fn continue_attack_activity(
        &mut self,
        owner: UnitInstanceId,
        now: u64,
        interval: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_attack_catalog().has_continuity {
            return Ok(());
        }
        self.reconcile_attack_continuity(now)?;
        let through = now
            .checked_add(interval)
            .ok_or_else(|| runtime_control::invariant("attack_continuity_deadline"))?;
        for (equipment, program) in self
            .equipped_attack_providers(owner, |p| p.continuity.is_some())
            .map_err(|error| match error {
                GameError::InvalidStaticData(_) => {
                    runtime_control::invariant("attack_continuity_source")
                }
                other => runtime_control::trusted_failure(other, "attack_continuity_source"),
            })?
        {
            let key = equipment.runtime_item_instance_id;
            if let Some(clock) = self.equipment_attack_state.continuity.get_mut(&key) {
                clock.through = through;
            } else {
                runtime_control::admit_retained_growth(
                    0,
                    Some(size_of::<AttackClock>() + 128),
                    "attack_continuity_state",
                )?;
                self.equipment_attack_state.continuity.insert(
                    key,
                    AttackClock {
                        owner,
                        employee: self.units[&owner].source_owned_uuid,
                        equipment,
                        program,
                        started: now,
                        through,
                    },
                );
            }
        }
        control::poll("attack_continuity_started")
    }

    pub(super) fn attack_continuity_damage_percent(
        &mut self,
        incoming: &DamageSourceSnapshot,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<i32, GameError> {
        if !self.game_data.equipment_attack_catalog().has_continuity {
            return Ok(0);
        }
        // Types belong to individual formula components, not only the base request.
        if !matches!(
            incoming.source,
            DamageSource::BasicAttack | DamageSource::Ability | DamageSource::BuffTick
        ) || incoming.fact_source.reaction
            || incoming.fact_source.guard_release().is_some()
            || incoming
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
        {
            return Ok(0);
        }
        self.reconcile_attack_continuity(now)?;
        control::charge(
            self.equipment_attack_state.continuity.len() as u64,
            "attack_continuity_damage",
        )?;
        let mut selected = None;
        for clock in self
            .equipment_attack_state
            .continuity
            .values()
            .filter(|c| c.owner == incoming.source_id)
        {
            let policy = clock
                .program
                .continuity
                .as_ref()
                .ok_or_else(|| runtime_control::invariant("attack_continuity_policy"))?;
            let elapsed = now.saturating_sub(clock.started);
            let percent = (u128::from(elapsed / 1000) * u128::from(policy.per_second_percent))
                .min(u128::from(policy.max_percent)) as u32;
            if selected
                .as_ref()
                .is_none_or(|(p, c): &(u32, &AttackClock)| {
                    percent > *p
                        || (percent == *p
                            && clock.equipment.persistent_copy_uuid
                                < c.equipment.persistent_copy_uuid)
                })
            {
                selected = Some((percent, clock));
            }
        }
        let Some((percent, clock)) = selected else {
            return Ok(0);
        };
        if self.authoring_observation.is_some() {
            let source = &clock.program.continuity_source;
            let bytes = size_of::<AttackContinuityWitness>()
                + source.source_file.len()
                + source.field_path.len()
                + source.site.action.definition_id.as_str().len();
            runtime_control::admit_retained_growth(
                0,
                Some(bytes),
                "attack_continuity_witness_copy",
            )?;
            control::charge(bytes as u64, "attack_continuity_witness_copy")?;
            let witness = AttackContinuityWitness {
                generation: self.content_identity.clone(),
                actor: incoming.source_id,
                target,
                time_ms: now,
                started_ms: clock.started,
                through_ms: clock.through,
                elapsed_ms: now.saturating_sub(clock.started),
                percent,
                equipment: clock.equipment,
                definition_id: source.site.action.definition_id.to_string(),
                source_file: source.source_file.clone(),
                field_path: source.field_path.clone(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "attack_continuity_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_attack_continuity = Some(witness);
        }
        control::poll("attack_continuity_evaluated")?;
        Ok(percent as i32)
    }
}
