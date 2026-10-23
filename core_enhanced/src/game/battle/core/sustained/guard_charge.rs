use super::*;
use crate::game::{
    ability::{guard_charge::*, marks::MarkRatio, SkillEffectDef, SkillHitTargetFilter},
    battle::{
        damage::{DamageModifiers, DamageSource},
        runtime_facts::DeflectionFact,
    },
};
use uuid::Uuid;
#[derive(Debug, Clone, Copy)]
pub(super) struct GuardChargeRecord {
    fragment: Uuid,
    relic: Uuid,
    amount: u32,
}
crate::authoring::clone_shape::copy_shape!(GuardChargeRecord);
#[derive(Debug, Clone, Copy)]
pub(super) struct GuardCounterPermit {
    pub activation: u64,
    pub actor: UnitInstanceId,
    pub attacker: UnitInstanceId,
    pub charge: Option<GuardChargeReceipt>,
}
crate::authoring::clone_shape::copy_shape!(GuardCounterPermit);
fn scaled(value: u32, r: MarkRatio) -> Result<u32, GameError> {
    u32::try_from((u128::from(value) * u128::from(r.numerator)).div_ceil(u128::from(r.denominator)))
        .map_err(|_| overflow("guard_counter_power"))
}
impl BattleCore {
    fn guard_charge_declaration(
        &self,
        owner: UnitInstanceId,
    ) -> Result<Option<Arc<GuardChargeDef>>, GameError> {
        let Some(unit) = self
            .units
            .get(&owner)
            .filter(|u| u.is_active() && !u.corroded)
        else {
            return Ok(None);
        };
        let Some(fragment) = unit.fragment_source.as_ref() else {
            return Ok(None);
        };
        if self.equipped_fragment_guard(fragment)?.is_none() {
            return Ok(None);
        }
        let Some(skill) = self.game_data.skill_data.get_by_id(&fragment.skill_id) else {
            return Ok(None);
        };
        for step in &skill.steps {
            control::charge(step.effects.len() as u64, "guard_charge_declaration")?;
            for effect in &step.effects {
                if let SkillEffectDef::Sustained {
                    definition:
                        SustainedEffectDef::Parry {
                            guard_charge: Some(p),
                            ..
                        },
                } = &effect.op
                {
                    return Ok(Some(Arc::clone(p)));
                }
            }
        }
        Ok(None)
    }
    fn guard_charge_binding(
        &self,
        owner: UnitInstanceId,
    ) -> Result<Option<(Arc<GuardChargeDef>, Uuid, Uuid)>, GameError> {
        let Some(p) = self.guard_charge_declaration(owner)? else {
            return Ok(None);
        };
        let Some(copy) = self.equipped_normal_relic_copy(owner, &p.relic_definition) else {
            return Ok(None);
        };
        Ok(Some((
            p,
            self.units[&owner]
                .fragment_source
                .as_ref()
                .unwrap()
                .copy_uuid,
            copy,
        )))
    }
    pub(in crate::game::battle::core) fn accumulate_guard_charge(
        &mut self,
        owner: UnitInstanceId,
        blocked: u32,
    ) -> Result<Option<GuardChargeReceipt>, GameError> {
        let Some((p, fragment, relic)) = self.guard_charge_binding(owner)? else {
            return Ok(None);
        };
        let before = self
            .sustained
            .guard_charges
            .get(&owner)
            .filter(|r| r.fragment == fragment && r.relic == relic)
            .map_or(0, |r| r.amount);
        let after = before.saturating_add(blocked).min(p.capacity);
        if after > 0 {
            if !self.sustained.guard_charges.contains_key(&owner) {
                self.admit_authoring_runtime_growth(1, "guard_charge_entity")?;
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(std::mem::size_of::<GuardChargeRecord>() + 80),
                    "guard_charge_storage",
                )?;
            }
            self.sustained.guard_charges.insert(
                owner,
                GuardChargeRecord {
                    fragment,
                    relic,
                    amount: after,
                },
            );
        }
        control::poll("guard_charge_after_accumulate")?;
        Ok(Some(GuardChargeReceipt {
            relic_copy: relic,
            blocked,
            before,
            after,
        }))
    }
    pub(super) fn consume_guard_charge(
        &mut self,
        owner: UnitInstanceId,
    ) -> Result<Option<GuardChargeReceipt>, GameError> {
        let Some((_, fragment, relic)) = self.guard_charge_binding(owner)? else {
            return Ok(None);
        };
        let before = self
            .sustained
            .guard_charges
            .remove(&owner)
            .filter(|r| r.fragment == fragment && r.relic == relic)
            .map_or(0, |r| r.amount);
        control::poll("guard_charge_after_consume")?;
        Ok(Some(GuardChargeReceipt {
            relic_copy: relic,
            blocked: 0,
            before,
            after: 0,
        }))
    }
    pub(super) fn guard_charge_view(
        &self,
        owner: UnitInstanceId,
    ) -> Result<Option<GuardChargeView>, GameError> {
        let Some((p, fragment, relic)) = self.guard_charge_binding(owner)? else {
            return Ok(None);
        };
        let stored = self
            .sustained
            .guard_charges
            .get(&owner)
            .filter(|r| r.fragment == fragment && r.relic == relic)
            .map_or(0, |r| r.amount);
        Ok(Some(GuardChargeView {
            relic_copy: relic,
            stored,
            capacity: p.capacity,
        }))
    }
    pub(super) fn reconcile_guard_charges(&mut self) -> Result<(), GameError> {
        control::charge(
            self.sustained.guard_charges.len() as u64,
            "guard_charge_reconcile",
        )?;
        // Remove one at a time without allocating a second owner list.
        loop {
            let mut obsolete = None;
            for (owner, record) in &self.sustained.guard_charges {
                control::charge(1, "guard_charge_owner_scan")?;
                if !self
                    .guard_charge_binding(*owner)?
                    .is_some_and(|(_, f, r)| f == record.fragment && r == record.relic)
                {
                    obsolete = Some(*owner);
                    break;
                }
            }
            let Some(owner) = obsolete else { break };
            self.sustained.guard_charges.remove(&owner);
        }
        Ok(())
    }
    pub(super) fn arm_guard_counter(
        &mut self,
        fact: &DeflectionFact,
        actor: UnitInstanceId,
        attacker: UnitInstanceId,
    ) -> Result<(), GameError> {
        if self.sustained.guard_counter_permit.is_some() {
            return Err(super::reaction_invariant("guard_counter_nested_permit"));
        }
        self.sustained.guard_counter_permit = Some(GuardCounterPermit {
            activation: fact.activation,
            actor,
            attacker,
            charge: fact.guard_charge,
        });
        Ok(())
    }
    pub(in crate::game::battle::core) fn execute_guard_counter(
        &mut self,
        owner: UnitInstanceId,
        targets: &[UnitInstanceId],
        program: &GuardCounterDef,
        now: u64,
    ) -> Result<super::super::types::SkillStepResult, GameError> {
        let permit = self
            .sustained
            .guard_counter_permit
            .take()
            .ok_or(GameError::InvalidAction)?;
        if permit.actor != owner || targets != [permit.attacker] {
            return Err(GameError::InvalidAction);
        }
        let record = self
            .sustained
            .records
            .get(&permit.activation)
            .ok_or(GameError::InvalidAction)?;
        let cast = self
            .runtime_fact_source
            .cast
            .as_ref()
            .ok_or(GameError::InvalidAction)?;
        if !record
            .parry
            .as_ref()
            .is_some_and(|p| p.counter_skill == cast.skill_id && p.invulnerable_until_ms.is_some())
            || self.guard_charge_declaration(owner)?.as_deref() != Some(program.charge.as_ref())
        {
            return Err(GameError::InvalidAction);
        }
        let primary = permit.attacker;
        let Some(unit) = self.units.get(&owner) else {
            return Err(GameError::InvalidAction);
        };
        let side = unit.owner;
        let base = scaled(unit.stats.attack, program.weapon)?;
        let position = self
            .sample_unit_body_at(primary, now)
            .ok_or(GameError::InvalidAction)?
            .position
            .project_to_tile();
        let consumed = permit.charge.map_or(0, |r| r.before);
        let primary_extra = scaled(consumed, program.charge.primary)?;
        let splash = scaled(consumed, program.charge.splash)?;
        let power = base
            .checked_add(primary_extra)
            .ok_or_else(|| overflow("guard_counter_sum"))?;
        let mut recipients = if splash > 0 {
            let tiles = self
                .valid_tile_area_tiles(
                    &program.charge.range,
                    position,
                    crate::game::battle::tile_range::FacingDirection::Up,
                )
                .ok_or(GameError::InvalidAction)?;
            self.collect_tile_area_targets_at(
                now,
                side,
                owner,
                &tiles,
                SkillHitTargetFilter::Enemies,
                false,
            )
        } else {
            Vec::new()
        };
        recipients.retain(|id| *id != primary);
        recipients.sort();
        for (target, power, extra, is_primary) in
            std::iter::once((primary, power, primary_extra, true))
                .chain(recipients.into_iter().map(|t| (t, splash, splash, false)))
        {
            if !self.units.get(&target).is_some_and(|u| u.is_active()) {
                continue;
            }
            let mut snapshot = self
                .damage_source_snapshot_for_unit(
                    owner,
                    target,
                    DamageSource::Ability,
                    program.damage_type,
                    power,
                    DamageModifiers::default(),
                    0,
                    now,
                    false,
                )
                .ok_or(GameError::InvalidAction)?;
            if let Some(receipt) = permit.charge {
                snapshot.fact_source =
                    snapshot
                        .fact_source
                        .with_guard_release(GuardReleaseOrigin {
                            relic_copy: receipt.relic_copy,
                            relic_definition: program.charge.relic_definition.clone(),
                            definition: program.charge.definition.clone(),
                            consumed,
                            added_power: extra,
                            primary: is_primary,
                        })?;
            }
            self.execute_committed_damage(super::super::damage_runtime::CommittedDamageInput {
                target_id: target,
                source_snapshot: snapshot,
                committed_at_ms: now,
                policy: super::super::damage_runtime::CommittedDamagePolicy::GeneralCommand,
            })
            .ok_or(GameError::InvalidAction)?;
            control::poll("guard_counter_damage")?;
        }
        Ok(Default::default())
    }
}

#[cfg(test)]
mod tests;
