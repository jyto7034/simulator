//! One preparation per mounted equipment copy. Readiness derives from battle
//! time; optional observation never owns or advances the charge.
use super::*;
use crate::game::battle::{
    runtime_facts::{IdleHpGuardFact, ProtectionReceipt},
    types::BattleUnitSourceIdentity,
};

#[derive(Debug, Clone)]
pub(super) struct IdleHpGuardState {
    pub(super) owner: UnitInstanceId,
    equipment: EquipmentSourceSnapshot,
    program: Arc<CompiledEquipmentDefense>,
    since: u64,
}
crate::authoring::clone_shape::struct_shape!(IdleHpGuardState {
    owner,
    equipment,
    program,
    since
});

#[derive(Debug, Clone, serde::Serialize)]
pub struct IdleHpGuardWitness {
    #[serde(skip)]
    pub(crate) generation: Option<Arc<crate::game::data::content_pack::ContentIdentity>>,
    pub target: UnitInstanceId,
    pub equipment: EquipmentSourceSnapshot,
    pub time_ms: u64,
    pub charged_since_ms: u64,
    pub recharge_ms: u32,
    pub reduction_percent: u8,
    pub incoming_hp_damage: u32,
    pub prevented: u32,
    pub definition_id: String,
    pub source_file: String,
    pub field_path: String,
    pub incoming: RuntimeFactSource,
}
crate::authoring::clone_shape::struct_shape!(IdleHpGuardWitness {
    generation,
    target,
    equipment,
    time_ms,
    charged_since_ms,
    recharge_ms,
    reduction_percent,
    incoming_hp_damage,
    prevented,
    definition_id,
    source_file,
    field_path,
    incoming
});
impl IdleHpGuardWitness {
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

impl BattleCore {
    pub(in crate::game::battle::core) fn reconcile_idle_hp_guards(
        &mut self,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_idle_guard {
            return Ok(());
        }
        if self.sustained.closed {
            self.equipment_defense.idle_guards.clear();
            return Ok(());
        }
        control::charge(self.units.len() as u64, "idle_hp_guard_wearers")?;
        let mut providers = Vec::new();
        for (owner, wearer) in &self.units {
            if !wearer.is_active()
                || wearer.corroded
                || wearer.owner != Side::Player
                || !matches!(
                    wearer.source_identity,
                    BattleUnitSourceIdentity::Employee { .. }
                )
            {
                continue;
            }
            let mut result = Ok(());
            self.visit_equipment_defenses(*owner, |equipment, program| {
                if result.is_err()
                    || !matches!(program.program, EquipmentDefenseProgram::IdleHpGuard { .. })
                {
                    return;
                }
                result = (|| {
                    if equipment.persistent_copy_uuid.is_none_or(|id| id.is_nil()) {
                        return Err(GameError::InvalidStaticData(
                            "idle HP guard requires an actual persistent equipment copy".into(),
                        ));
                    }
                    runtime_control::admit_retained_growth(
                        0,
                        Some(
                            size_of::<(
                                UnitInstanceId,
                                EquipmentSourceSnapshot,
                                Arc<CompiledEquipmentDefense>,
                            )>() + 128,
                        ),
                        "idle_hp_guard_provider",
                    )?;
                    providers.reserve_exact(1);
                    providers.push((*owner, equipment, Arc::clone(program)));
                    Ok(())
                })();
            })?;
            result?;
        }
        let mut live = std::collections::BTreeSet::new();
        for (owner, equipment, program) in providers {
            control::charge(1, "idle_hp_guard_provider")?;
            runtime_control::admit_retained_growth(
                0,
                Some(size_of::<uuid::Uuid>() + 96),
                "idle_hp_guard_live_copy",
            )?;
            let key = equipment.runtime_item_instance_id;
            live.insert(key);
            if self
                .equipment_defense
                .idle_guards
                .get(&key)
                .is_none_or(|s| s.owner != owner || s.equipment != equipment)
            {
                runtime_control::admit_retained_growth(
                    self.equipment_defense
                        .retained_extent()
                        .unwrap_or(usize::MAX),
                    Some(size_of::<IdleHpGuardState>() + size_of::<uuid::Uuid>() + 96),
                    "idle_hp_guard_state",
                )?;
                self.equipment_defense.idle_guards.insert(
                    key,
                    IdleHpGuardState {
                        owner,
                        equipment,
                        program,
                        since: now,
                    },
                );
            }
        }
        control::charge(
            self.equipment_defense.idle_guards.len() as u64,
            "idle_hp_guard_cleanup",
        )?;
        self.equipment_defense
            .idle_guards
            .retain(|key, _| live.contains(key));
        control::poll("idle_hp_guard_reconciled")
    }

    pub(in crate::game::battle::core) fn after_idle_hp_guard(amount: u32, percent: u8) -> u32 {
        (u64::from(amount) * u64::from(100 - percent) / 100) as u32
    }

    /// Invoked only for a positive post-shield packet, before deferral/survival.
    pub(in crate::game::battle::core) fn consume_idle_hp_guard(
        &mut self,
        target: UnitInstanceId,
        amount: u32,
        now: u64,
        incoming: &RuntimeFactSource,
        protection: &mut ProtectionReceipt,
    ) -> Result<u8, GameError> {
        if amount == 0 || !self.game_data.equipment_defense_catalog().has_idle_guard {
            return Ok(0);
        }
        control::charge(
            self.equipment_defense.idle_guards.len() as u64,
            "idle_hp_guard_consume",
        )?;
        let mut chosen: Option<(
            EquipmentSourceSnapshot,
            Arc<CompiledEquipmentDefense>,
            u64,
            u32,
            u8,
        )> = None;
        for state in self
            .equipment_defense
            .idle_guards
            .values_mut()
            .filter(|s| s.owner == target)
        {
            let EquipmentDefenseProgram::IdleHpGuard {
                recharge_ms,
                reduction_percent,
            } = state.program.program
            else {
                return Err(runtime_control::invariant("idle_hp_guard_program"));
            };
            if now.saturating_sub(state.since) < u64::from(recharge_ms) {
                continue;
            }
            if chosen.as_ref().is_none_or(|(prior, _, _, _, percent)| {
                reduction_percent > *percent
                    || (reduction_percent == *percent
                        && state.equipment.runtime_item_instance_id
                            < prior.runtime_item_instance_id)
            }) {
                chosen = Some((
                    state.equipment,
                    Arc::clone(&state.program),
                    state.since,
                    recharge_ms,
                    reduction_percent,
                ));
            }
            // All simultaneously ready copies belong to this packet, not later hits.
            state.since = now;
        }
        let Some((equipment, program, since, recharge, percent)) = chosen else {
            return Ok(0);
        };
        let prevented = amount - Self::after_idle_hp_guard(amount, percent);
        let source = RuntimeFactSource {
            actor: Some(target),
            equipment: Some(equipment),
            authored: Some(Arc::clone(
                program
                    .idle_guard_source
                    .as_ref()
                    .ok_or_else(|| runtime_control::invariant("idle_hp_guard_source"))?,
            )),
            ..Default::default()
        };
        runtime_control::admit_retained_growth(
            0,
            source
                .retained_extent()
                .and_then(|n| n.checked_add(size_of::<IdleHpGuardFact>() + 64)),
            "idle_hp_guard_fact",
        )?;
        protection.details_mut()?.idle_hp_guard = Some(Box::new(IdleHpGuardFact {
            source,
            charged_since_ms: since,
            recharge_ms: recharge,
            reduction_percent: percent,
            prevented,
        }));
        protection.prevented = protection
            .prevented
            .checked_add(prevented)
            .ok_or_else(|| runtime_control::invariant("idle_hp_guard_prevented"))?;
        if self.authoring_observation.is_some() {
            let extent = incoming.retained_extent().and_then(|n| {
                n.checked_add(
                    size_of::<IdleHpGuardWitness>()
                        + program.definition_id.len()
                        + program.source_file.len()
                        + program.field_path.len(),
                )
            });
            runtime_control::admit_retained_growth(0, extent, "idle_hp_guard_witness_copy")?;
            control::charge(
                extent.unwrap_or(usize::MAX) as u64,
                "idle_hp_guard_witness_copy",
            )?;
            let witness = IdleHpGuardWitness {
                generation: self.content_identity.clone(),
                target,
                equipment,
                time_ms: now,
                charged_since_ms: since,
                recharge_ms: recharge,
                reduction_percent: percent,
                incoming_hp_damage: amount,
                prevented,
                definition_id: program.definition_id.clone(),
                source_file: program.source_file.clone(),
                field_path: program.field_path.clone(),
                incoming: incoming.observation(),
            };
            runtime_control::admit_retained_growth(
                0,
                witness.retained_extent(),
                "idle_hp_guard_observation",
            )?;
            self.authoring_observation
                .as_mut()
                .unwrap()
                .latest_idle_hp_guard = Some(witness);
        }
        control::poll("idle_hp_guard_consumed")?;
        Ok(percent)
    }

    /// All committed actual HP damage resets charging, including repayment of
    /// previously deferred HP damage. Costs and direct execution bypass damage.
    pub(in crate::game::battle::core) fn reset_idle_hp_guard_damage(
        &mut self,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if !self.game_data.equipment_defense_catalog().has_idle_guard {
            return Ok(());
        }
        control::charge(
            self.equipment_defense.idle_guards.len() as u64,
            "idle_hp_guard_damage_reset",
        )?;
        for state in self
            .equipment_defense
            .idle_guards
            .values_mut()
            .filter(|s| s.owner == target)
        {
            state.since = now;
        }
        control::poll("idle_hp_guard_damage_reset")
    }
}
