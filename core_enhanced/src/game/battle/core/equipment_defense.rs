mod idle_hp_guard;
pub use idle_hp_guard::IdleHpGuardWitness;
mod conditional;
mod wearer_reduction;
pub use wearer_reduction::WearerReductionWitness;
pub use conditional::IncomingConditionalWitness;
mod incoming_flat;
pub use incoming_flat::IncomingFlatWitness;
pub(super) use incoming_flat::FlatAllowance;
mod ready_wait;
pub use ready_wait::ReadyWaitWitness;
mod nearby;
pub use nearby::NearbyDefenseWitness;
mod revival;
mod metamorphosis;
pub use metamorphosis::EmployeeTransformationWitness;
mod channel_opening;
pub use channel_opening::ChannelOpeningWitness;
mod adaptive;
mod last_will;
pub use last_will::LastWillWitness;
mod deferred_damage;
mod overheal_reserve;
use super::BattleCore;
use crate::{
    authoring::control,
    game::{
        ability::equipment_defense::{CompiledEquipmentDefense, EquipmentDefenseProgram},
        battle::{
            ids::UnitInstanceId,
            runtime_control,
            runtime_facts::{EquipmentSourceSnapshot, RuntimeFactSource},
        },
        behavior::GameError,
        enums::Side,
    },
};
pub use adaptive::AdaptiveShieldWitness;
pub use deferred_damage::{DeferredDamageWitness, DeferredOperation};
pub use overheal_reserve::{OverhealReserveWitness, ReserveOperation};
use std::{collections::BTreeMap, mem::size_of, sync::Arc};

#[derive(Debug, Clone, Copy)]
pub(super) struct BarrierCooldown {
    owner: UnitInstanceId,
    ready_at_ms: u64,
}
crate::authoring::clone_shape::copy_shape!(BarrierCooldown);

#[derive(Debug, Clone, Default)]
pub(super) struct EquipmentDefenseState {
    idle_guards: BTreeMap<uuid::Uuid, idle_hp_guard::IdleHpGuardState>,
    ready_wait: BTreeMap<uuid::Uuid, ready_wait::ReadyWaitState>,
    required_transformations: std::collections::BTreeSet<UnitInstanceId>,
    pub(super) last_will_used: std::collections::BTreeSet<uuid::Uuid>,
    pub(super) last_will_resolving: std::collections::BTreeSet<UnitInstanceId>,
    pub(super) deferred: BTreeMap<u64, deferred_damage::Debt>,
    next_debt: u64,
    cooldowns: BTreeMap<uuid::Uuid, BarrierCooldown>,
    pub(super) reserves: BTreeMap<uuid::Uuid, overheal_reserve::ReserveBalance>,
    adaptive_uses: BTreeMap<(uuid::Uuid, u64), adaptive::AdaptiveUse>,
}
crate::authoring::clone_shape::struct_shape!(EquipmentDefenseState {
    idle_guards,
    ready_wait,
    required_transformations,
    last_will_used,
    last_will_resolving,
    cooldowns,
    deferred,
    next_debt,
    adaptive_uses,
    reserves
});
impl EquipmentDefenseState {
    pub(super) fn retained_extent(&self) -> Option<usize> {
        control::charge(self.deferred.len() as u64, "deferred_retained_scan").ok()?;
        let debt_bytes = self.deferred.values().try_fold(0usize, |n, debt| {
            n.checked_add(debt.retained_extent()?.checked_add(104)?)
        })?;
        control::charge(self.ready_wait.len() as u64, "ready_wait_retained_scan").ok()?;
        let ready_bytes = self.ready_wait.values().try_fold(0usize, |n, r| n.checked_add(
            size_of::<ready_wait::ReadyWaitState>() + size_of::<uuid::Uuid>() + 96 + r.ready_skill.as_ref().map_or(0, |s| s.as_str().len())
        ))?;
        size_of::<Self>()
            .checked_add(ready_bytes)?
            .checked_add(self.idle_guards.len().checked_mul(size_of::<idle_hp_guard::IdleHpGuardState>() + size_of::<uuid::Uuid>() + 96)?)?
            .checked_add(self.required_transformations.len().checked_mul(128 + size_of::<UnitInstanceId>())?)?
            .checked_add(self.last_will_used.len().checked_mul(128 + size_of::<uuid::Uuid>())?)?
            .checked_add(self.last_will_resolving.len().checked_mul(128 + size_of::<UnitInstanceId>())?)?
            .checked_add(debt_bytes)?
            .checked_add(
                self.cooldowns
                    .len()
                    .checked_mul(size_of::<BarrierCooldown>() + size_of::<uuid::Uuid>() + 96)?,
            )?
            .checked_add(self.reserves.len().checked_mul(
                size_of::<overheal_reserve::ReserveBalance>() + size_of::<uuid::Uuid>() + 96,
            )?)?
            .checked_add(self.adaptive_uses.len().checked_mul(
                size_of::<adaptive::AdaptiveUse>() + size_of::<(uuid::Uuid, u64)>() + 192,
            )?)
    }
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.idle_guards.is_empty() && self.ready_wait.is_empty()
            && self.required_transformations.is_empty()
            && self.last_will_used.is_empty()
            && self.last_will_resolving.is_empty()
            && self.cooldowns.is_empty()
            && self.adaptive_uses.is_empty()
            && self.reserves.is_empty()
            && self.deferred.is_empty()
    }
    pub(super) fn remove_owner(&mut self, owner: UnitInstanceId) {
        // Both controls retain faults until the enclosing transaction polls.
        if control::charge(
            (self.idle_guards.len() as u64).saturating_add(self.ready_wait.len() as u64).saturating_add(self.cooldowns.len() as u64)
                .saturating_add(self.adaptive_uses.len() as u64)
                .saturating_add(self.reserves.len() as u64)
                .saturating_add(self.deferred.len() as u64),
            "equipment_defense_remove_owner",
        )
        .is_err()
        {
            return;
        }
        self.idle_guards.retain(|_, value| value.owner != owner);
        self.ready_wait.retain(|_, value| value.owner != owner);
        self.deferred.retain(|_, value| value.owner != owner);
        self.reserves.retain(|_, value| value.owner != owner);
        self.cooldowns.retain(|_, value| value.owner != owner);
        self.adaptive_uses.retain(|_, value| value.owner != owner);
    }
}

/// Last successful interception. This is evidence, never cooldown authority.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillBarrierWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub ready_at_ms: u64,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(SkillBarrierWitness {
    generation,
    target,
    equipment,
    time_ms,
    ready_at_ms,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl SkillBarrierWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}

pub(super) fn harmful_stat_modifier(modifier: &crate::game::stats::StatModifier) -> bool {
    if modifier.stat == crate::game::stats::StatId::AttackIntervalMs {
        modifier.value > 0
    } else {
        modifier.value < 0
    }
}

impl BattleCore {
    fn visit_equipment_defenses(
        &self,
        target: UnitInstanceId,
        mut visit: impl FnMut(EquipmentSourceSnapshot, &Arc<CompiledEquipmentDefense>),
    ) -> Result<(), GameError> {
        let Some(unit) = self.units.get(&target) else {
            return Ok(());
        };
        let catalog = self.game_data.equipment_defense_catalog();
        let slots = unit.equipment_slots.as_ref();
        control::charge(
            if slots.is_some() { 3 } else { self.items.len() } as u64,
            "equipment_defense_providers",
        )?;
        let slot_items = slots
            .into_iter()
            .flat_map(|s| s.slots.iter().flatten())
            .map(|s| {
                self.items
                    .get(&s.runtime_item_instance_id)
                    .ok_or_else(|| runtime_control::invariant("equipment_defense_slot"))
            });
        let legacy_items = self
            .items
            .values()
            .take(if slots.is_some() { 0 } else { self.items.len() })
            .filter(|i| i.owner_unit_instance == target)
            .map(Ok);
        for item in slot_items.chain(legacy_items) {
            let item = item?;
            if item.owner_unit_instance != target {
                return Err(runtime_control::invariant("equipment_defense_owner"));
            }
            if let Some(program) = catalog.by_equipment.get(&item.base_uuid) {
                visit(item.source_snapshot(), program);
            }
        }
        Ok(())
    }

    pub(super) fn reconcile_equipment_defenses(&mut self) -> Result<(), GameError> {
        self.reconcile_overheal_reserves()?;
        self.reconcile_deferred_damage()?;
        control::charge(
            self.equipment_defense.cooldowns.len() as u64,
            "equipment_defense_reconcile",
        )?;
        self.equipment_defense.cooldowns.retain(|id, cooldown| {
            self.units.get(&cooldown.owner).is_some_and(|u| {
                u.is_active()
                    && !u.corroded
                    && u.equipment_slots.as_ref().is_none_or(|slots| {
                        slots
                            .slots
                            .iter()
                            .flatten()
                            .any(|s| s.runtime_item_instance_id == *id)
                    })
            }) && self.items.get(id).is_some_and(|i| {
                i.owner_unit_instance == cooldown.owner
                    && self
                        .game_data
                        .equipment_defense_catalog()
                        .by_equipment
                        .contains_key(&i.base_uuid)
            })
        });
        control::charge(
            self.equipment_defense.adaptive_uses.len() as u64,
            "adaptive_shield_use_reconcile",
        )?;
        self.equipment_defense
            .adaptive_uses
            .retain(|(copy, _), used| {
                self.equipment_defense
                    .cooldowns
                    .get(copy)
                    .is_some_and(|c| c.owner == used.owner)
                    && self
                        .runtime_facts
                        .validate_live_action(&used.action)
                        .is_ok()
            });
        Ok(())
    }

    /// Called only after the operation's own target/immunity checks. Maintained
    /// aura membership and basic/tick damage never call this interception gate.
    pub(super) fn block_equipment_skill(
        &mut self,
        target: UnitInstanceId,
        caster: UnitInstanceId,
        committed_side: Option<Side>,
        now: u64,
    ) -> Result<bool, GameError> {
        let source = &self.runtime_fact_source;
        if source.reaction
            || source.guard_release().is_some()
            || source
                .cast
                .as_ref()
                .is_none_or(|cast| cast.proc_origin.is_some())
            || caster == target
        {
            return Ok(false);
        }
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && !u.corroded)
        else {
            return Ok(false);
        };
        if committed_side
            .or_else(|| self.units.get(&caster).map(|u| u.owner))
            .is_none_or(|side| side == unit.owner)
        {
            return Ok(false);
        }
        if self.sustained_invulnerable(target, now)? {
            return Ok(false);
        }
        if self.strike_barrier_blocked(target) {
            return Ok(true);
        }
        let catalog = self.game_data.equipment_defense_catalog();
        if catalog.by_equipment.is_empty() {
            return Ok(false);
        }
        // Several equipped barriers consume in stable runtime-copy order.
        let mut selected = None;
        self.visit_equipment_defenses(target, |equipment, program| {
            if !matches!(
                program.program,
                EquipmentDefenseProgram::RechargingSkillBarrier { .. }
            ) {
                return;
            }
            if self
                .equipment_defense
                .cooldowns
                .get(&equipment.runtime_item_instance_id)
                .is_some_and(|c| c.owner == target && now < c.ready_at_ms)
            {
                return;
            }
            if selected
                .as_ref()
                .is_none_or(|(old, _): &(EquipmentSourceSnapshot, _)| {
                    equipment.runtime_item_instance_id < old.runtime_item_instance_id
                })
            {
                selected = Some((equipment, Arc::clone(program)));
            }
        })?;
        let Some((equipment, program)) = selected else {
            return Ok(false);
        };
        let EquipmentDefenseProgram::RechargingSkillBarrier { recharge_ms } = program.program
        else {
            return Err(runtime_control::invariant(
                "equipment_defense_barrier_program",
            ));
        };
        let ready_at_ms = now
            .checked_add(u64::from(recharge_ms))
            .ok_or_else(|| runtime_control::invariant("equipment_defense_deadline"))?;
        let witness = if self.authoring_observation.is_some() {
            let source_bytes = source.retained_extent();
            runtime_control::admit_retained_growth(
                0,
                source_bytes.and_then(|n| {
                    n.checked_add(
                        size_of::<SkillBarrierWitness>()
                            + program.definition_id.len()
                            + program.source_file.len()
                            + program.field_path.len(),
                    )
                }),
                "skill_barrier_witness",
            )?;
            control::charge(
                source_bytes.unwrap_or(usize::MAX) as u64,
                "skill_barrier_witness_copy",
            )?;
            Some(SkillBarrierWitness {
                generation: self.content_identity.clone(),
                target,
                equipment,
                time_ms: now,
                ready_at_ms,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: program.field_path.clone(),
                incoming: source.observation(),
            })
        } else {
            None
        };
        if let Some(witness) = &witness {
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "skill_barrier_observation",
            )?;
        }
        if !self
            .equipment_defense
            .cooldowns
            .contains_key(&equipment.runtime_item_instance_id)
        {
            let retained = self.equipment_defense.retained_extent();
            runtime_control::admit_retained_growth(
                retained.unwrap_or(usize::MAX),
                retained.and_then(|n| {
                    n.checked_add(size_of::<BarrierCooldown>() + size_of::<uuid::Uuid>() + 96)
                }),
                "equipment_defense_cooldown",
            )?;
        }
        self.mark_strike_barrier_blocked(target)?;
        self.equipment_defense.cooldowns.insert(
            equipment.runtime_item_instance_id,
            BarrierCooldown {
                owner: target,
                ready_at_ms,
            },
        );
        if let Some(observer) = &mut self.authoring_observation {
            observer.latest_skill_barrier = witness;
        }
        control::poll("equipment_defense_consumed")?;
        Ok(true)
    }
}

/// Most recent eligible roll, including a miss. Observation is not roll authority.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BasicEvasionWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub strike: u64,
    pub chance_percent: u8,
    pub roll_percent: u8,
    pub evaded: bool,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(BasicEvasionWitness {
    generation,
    target,
    equipment,
    time_ms,
    strike,
    chance_percent,
    roll_percent,
    evaded,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl BasicEvasionWitness {
    pub(crate) fn retained_extent(&self) -> Option<usize> {
        let mut bytes = size_of::<Self>()
            .checked_add(self.definition_id.capacity())?
            .checked_add(self.source_file.capacity())?
            .checked_add(self.field_path.capacity())?
            .checked_add(self.incoming.retained_extent()?)?;
        if let Some(generation) = &self.generation {
            bytes = bytes
                .checked_add(std::mem::size_of_val(generation.as_ref()))?
                .checked_add(generation.semantic_sha256.capacity())?;
        }
        Some(bytes)
    }
}

impl BattleCore {
    pub(super) fn evade_equipment_basic(
        &mut self,
        target: UnitInstanceId,
        snapshot: &crate::game::battle::damage::DamageSourceSnapshot,
        strike: &mut super::runtime_facts::AdmittedStrike,
        now: u64,
    ) -> Result<bool, GameError> {
        use crate::game::battle::damage::DamageSource;
        if snapshot.source != DamageSource::BasicAttack
            || snapshot.fact_source.reaction
            || snapshot.fact_source.guard_release().is_some()
            || snapshot
                .fact_source
                .cast
                .as_ref()
                .is_some_and(|c| c.proc_origin.is_some())
            || snapshot.source_id == target
        {
            return Ok(false);
        }
        let Some(unit) = self
            .units
            .get(&target)
            .filter(|u| u.is_active() && !u.corroded)
        else {
            return Ok(false);
        };
        if snapshot.source_side == unit.owner || self.sustained_invulnerable(target, now)? {
            return Ok(false);
        }
        if let Some(evaded) = Self::equipment_evasion_result(strike) {
            return Ok(evaded);
        }
        if self
            .game_data
            .equipment_defense_catalog()
            .by_equipment
            .is_empty()
        {
            return Ok(false);
        }
        let mut selected: Option<(EquipmentSourceSnapshot, Arc<CompiledEquipmentDefense>, u8)> =
            None;
        self.visit_equipment_defenses(target, |equipment, program| {
            let EquipmentDefenseProgram::BasicAttackEvasion { chance_percent } = program.program
            else {
                return;
            };
            if selected.as_ref().is_none_or(|(old, _, chance)| {
                chance_percent > *chance
                    || (chance_percent == *chance
                        && equipment.runtime_item_instance_id < old.runtime_item_instance_id)
            }) {
                selected = Some((equipment, Arc::clone(program), chance_percent));
            }
        })?;
        let Some((equipment, program, chance_percent)) = selected else {
            return Ok(false);
        };
        control::charge(1, "equipment_evasion_roll")?;
        let seed = crate::game::determinism::seed_with_uuid(
            self.seed,
            0x65766173696f6e31,
            target.as_uuid(),
        );
        let seed = crate::game::determinism::seed_with_uuid(
            seed,
            0x65766173696f6e32,
            equipment.runtime_item_instance_id,
        );
        let roll_percent = (crate::game::determinism::uuid_v4_from_seed(
            seed,
            0x65766173696f6e31,
            strike.ordinal().0,
        )
        .as_u128()
            % 100) as u8;
        let evaded = roll_percent < chance_percent;
        let witness = if self.authoring_observation.is_some() {
            let source_bytes = snapshot.fact_source.retained_extent();
            runtime_control::admit_retained_growth(
                0,
                source_bytes.and_then(|n| {
                    n.checked_add(
                        size_of::<BasicEvasionWitness>()
                            + program.definition_id.len()
                            + program.source_file.len()
                            + program.field_path.len(),
                    )
                }),
                "basic_evasion_witness",
            )?;
            control::charge(
                source_bytes.unwrap_or(usize::MAX) as u64,
                "basic_evasion_witness_copy",
            )?;
            Some(BasicEvasionWitness {
                generation: self.content_identity.clone(),
                target,
                equipment,
                time_ms: now,
                strike: strike.ordinal().0,
                chance_percent,
                roll_percent,
                evaded,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: program.field_path.clone(),
                incoming: snapshot.fact_source.observation(),
            })
        } else {
            None
        };
        if let Some(witness) = &witness {
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "basic_evasion_observation",
            )?;
        }
        self.mark_equipment_evasion(strike, evaded);
        if let Some(observer) = &mut self.authoring_observation {
            observer.latest_basic_evasion = witness;
        }
        control::poll("equipment_evasion_resolved")?;
        Ok(evaded)
    }
}
