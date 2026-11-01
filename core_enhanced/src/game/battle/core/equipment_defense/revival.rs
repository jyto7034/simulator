//! Employee counters survive deployment; the finite immunity has its own owner.
use super::*;
use crate::game::{
    ability::sustained::{SustainedEffectDef, TargetStateProfile},
    battle::{event_log::HpChangeReason, runtime_facts::ProtectionReceipt},
    resource::{ResourceMutation, ResourceOwnerSelector},
};

pub(in crate::game::battle::core) struct RevivalPlan {
    actor: UnitInstanceId,
    restored_hp: u32,
    invulnerable_ms: u32,
    source: RuntimeFactSource,
    transform: Option<Arc<CompiledEquipmentDefense>>,
}

impl BattleCore {
    pub(in crate::game::battle::core) fn prepare_employee_revival(
        &mut self,
        actor: UnitInstanceId,
        pending: u32,
        now: u64,
        incoming: &RuntimeFactSource,
        receipt: &mut ProtectionReceipt,
    ) -> Result<Option<RevivalPlan>, GameError> {
        if self.sustained.closed || !self.game_data.equipment_defense_catalog().has_revival {
            return Ok(None);
        }
        let Some(unit) = self
            .units
            .get(&actor)
            .filter(|u| u.is_active() && !u.corroded && Self::is_employee_resonance_unit(u))
        else {
            return Ok(None);
        };
        let hp = unit.stats.current_health;
        if hp == 0 || pending < hp {
            return Ok(None);
        }
        let max_hp = unit.stats.max_health;
        let mut selected = None;
        self.visit_equipment_defenses(actor, |equipment, program| {
            if program.revival_source.is_some()
                && equipment.persistent_copy_uuid.is_some()
                && selected.is_none()
            {
                selected = Some((equipment, Arc::clone(program)));
            }
        })?;
        let Some((equipment, program)) = selected else {
            return Ok(None);
        };
        let EquipmentDefenseProgram::EmployeeRevival {
            cumulative,
            battle_used,
            max_revivals,
            restore_percent,
            invulnerable_ms,
            ..
        } = &program.program
        else {
            unreachable!()
        };
        let mut source = RuntimeFactSource {
            actor: Some(actor),
            equipment: Some(equipment),
            authored: program.revival_source.clone(),
            reaction: true,
            ..Default::default()
        }
        .inherit_causal_context(incoming)?;
        source.admit_owner("revival_source")?;
        self.prepare_runtime_resources()?;
        let total = self
            .resolve_typed_resource_lease(
                cumulative,
                ResourceOwnerSelector::SourceEmployee,
                &source,
                None,
            )
            .map_err(|e| e.into_game_error())?;
        let used = self
            .resolve_typed_resource_lease(
                battle_used,
                ResourceOwnerSelector::SourceEmployee,
                &source,
                None,
            )
            .map_err(|e| e.into_game_error())?;
        let store = &self.runtime_facts.resources.as_ref().unwrap().store;
        let count = store.value(&total).map_err(|e| e.into_game_error())?.balance;
        let transform = count == *max_revivals && program.transformation_uuid.is_some();
        if store.value(&used).map_err(|e| e.into_game_error())?.balance > 0
            || (count >= *max_revivals && !transform)
        { return Ok(None); }
        if transform {
            source.authored = program.transformation_source.clone();
            source.admit_owner("metamorphosis_source")?;
        }
        let restored_hp = (u64::from(max_hp) * u64::from(*restore_percent) / 100).max(1) as u32;
        if !transform && restored_hp > i32::MAX as u32 {
            return Err(runtime_control::invariant("revival_hp_overflow"));
        }
        if !transform { now.checked_add(u64::from(*invulnerable_ms))
            .ok_or_else(|| runtime_control::invariant("revival_deadline_overflow"))?; }
        self.apply_typed_resource_batch(
            &[
                (total, ResourceMutation::Grant(1)),
                (used, ResourceMutation::Grant(1)),
            ],
            &source,
            now,
        )?;
        receipt.prevented = receipt.prevented.saturating_add(pending - (hp - 1));
        control::poll("revival_reserved")?;
        Ok(Some(RevivalPlan {
            actor,
            restored_hp,
            invulnerable_ms: *invulnerable_ms,
            source,
            transform: transform.then(|| Arc::clone(&program)),
        }))
    }

    pub(in crate::game::battle::core) fn resolve_employee_revival(
        &mut self,
        plan: RevivalPlan,
        now: u64,
    ) -> Result<bool, GameError> {
        if let Some(program) = plan.transform {
            self.resolve_employee_transformation(plan.actor, program, plan.source, now)?;
            return Ok(true);
        }
        let old = std::mem::replace(&mut self.runtime_fact_source, plan.source);
        let result = (|| {
            let current = self
                .units
                .get(&plan.actor)
                .filter(|u| u.is_active())
                .ok_or_else(|| runtime_control::invariant("revival_actor_lost"))?
                .stats
                .current_health;
            let delta = i32::try_from(i64::from(plan.restored_hp) - i64::from(current))
                .map_err(|_| runtime_control::invariant("revival_delta_overflow"))?;
            self.apply_hp_delta_with_reserve(
                Some(plan.actor),
                plan.actor,
                delta,
                now,
                HpChangeReason::Command,
                false,
            );
            control::poll("revival_restored")?;
            self.begin_sustained(
                plan.actor,
                plan.actor,
                SustainedEffectDef::TargetState {
                    duration_ms: plan.invulnerable_ms,
                    profile: TargetStateProfile {
                        definition: "employee-revival-immunity".into(),
                        attack_percent: 0,
                        defense_percent: 0,
                        attack_speed_points: 0,
                        hp_floor_one: false,
                        damage_invulnerable: true,
                        basic_attack_locked: false,
                        active_skill_locked: false,
                        received_heal_percent: 0,
                        end_on_source_exit: true,
                    },
                },
                now,
            )?;
            control::poll("revival_protected")
        })();
        self.runtime_fact_source = old;
        result.map(|()| false)
    }
}
