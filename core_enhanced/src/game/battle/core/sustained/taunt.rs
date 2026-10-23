use super::*;
use crate::game::{
    ability::{
        sustained::ApproachTauntProfile, DeliveryDef, SkillDef, SkillEffectDef, SkillTarget,
    },
    battle::{
        event_log::{BattleLogEvent, SkillCastTarget},
        runtime_facts::ActionCancelReason,
    },
};

impl BattleCore {
    pub(in crate::game::battle::core) fn approach_taunt(
        &self,
        target: UnitInstanceId,
    ) -> Option<(UnitInstanceId, ApproachTauntProfile)> {
        control::charge(self.sustained.records.len() as u64, "approach_taunt_query").ok()?;
        self.sustained.records.values().find_map(|r| {
            let Some(ControlKind::ApproachTaunt { profile }) = r.control else {
                return None;
            };
            (r.target == target && self.units.get(&r.caster).is_some_and(|u| u.is_active()))
                .then_some((r.caster, profile))
        })
    }
    pub(in crate::game::battle::core) fn approach_taunt_attack_allowed(
        &self,
        attacker: UnitInstanceId,
        target: UnitInstanceId,
    ) -> bool {
        let Some((source, profile)) = self.approach_taunt(attacker) else {
            return true;
        };
        let radius = profile.attack_range_units as f64
            / f64::from(crate::game::battle::core::movement::types::DATA_UNITS_PER_WORLD);
        source == target
            && self
                .basic_attack_target_distance_sq_world(attacker, source)
                .is_some_and(|d| f64::from(d) <= radius * radius + 0.000001)
    }
    pub(in crate::game::battle::core) fn hostile_attack_skill(skill: &SkillDef) -> bool {
        if control::charge(skill.steps.len() as u64, "taunt_skill_steps").is_err() {
            return false;
        }
        skill.steps.iter().any(|step| {
            if matches!(step.target, SkillTarget::SelfUnit)
                && matches!(step.delivery, DeliveryDef::Instant)
            {
                return false;
            }
            if control::charge(step.effects.len() as u64, "taunt_skill_effects").is_err() {
                return false;
            }
            step.effects.iter().any(|effect| {
                matches!(
                    effect.op,
                    SkillEffectDef::Damage { .. } | SkillEffectDef::DamageExpr { .. }
                )
            })
        })
    }
    pub(in crate::game::battle::core) fn approach_taunt_skill_allowed(
        &self,
        actor: UnitInstanceId,
        skill: &SkillDef,
        target: Option<SkillCastTarget>,
    ) -> bool {
        let Some((source, _)) = self.approach_taunt(actor) else {
            return true;
        };
        if !Self::hostile_attack_skill(skill) {
            return true;
        }
        if !self.approach_taunt_attack_allowed(actor, source) {
            return false;
        }
        match target {
            Some(SkillCastTarget::TilePair { .. } | SkillCastTarget::Direction { .. }) => false,
            Some(SkillCastTarget::Unit { unit_instance_id }) => {
                unit_instance_id == source
                    || (unit_instance_id == actor
                        && skill.cast_selection()
                            == crate::game::ability::SkillCastSelection::SelfUnit)
            }
            Some(SkillCastTarget::Tile { position }) => {
                self.live_unit_projected_tile(source) == Some(position)
            }
            None => false,
        }
    }
    pub(in crate::game::battle::core) fn approach_taunt_damage(
        &self,
        source: UnitInstanceId,
        target: UnitInstanceId,
        basic: bool,
    ) -> (i32, i32) {
        let reduction = self
            .approach_taunt(source)
            .filter(|(taunter, _)| *taunter == target)
            .map_or(0, |(_, p)| p.damage_reduction_percent as i32);
        let bonus = if basic {
            self.approach_taunt(target)
                .filter(|(taunter, _)| *taunter == source)
                .map_or(0, |(_, p)| p.basic_damage_bonus_percent as i32)
        } else {
            0
        };
        (reduction, bonus)
    }
    pub(in crate::game::battle::core) fn replace_approach_taunt(
        &mut self,
        target: UnitInstanceId,
        incoming: ControlKind,
        now: u64,
    ) -> Result<(), GameError> {
        // Legacy priority taunts keep their old stacking policy unless one side
        // explicitly opts into the replaceable approach contract.
        let replace = matches!(incoming, ControlKind::ApproachTaunt { .. })
            || self.approach_taunt(target).is_some();
        if !replace {
            return Ok(());
        }
        loop {
            control::charge(
                self.sustained.records.len() as u64,
                "approach_taunt_replace",
            )?;
            let old = self
                .sustained
                .records
                .iter()
                .find(|(_, r)| r.target == target && r.control.is_some_and(ControlKind::is_taunt))
                .map(|(id, _)| *id);
            let Some(old) = old else {
                break;
            };
            self.end_sustained(old, now, SustainedEndReason::Replaced)?;
        }
        Ok(())
    }
    pub(in crate::game::battle::core) fn cancel_approach_taunt_attacks(
        &mut self,
        caster: UnitInstanceId,
        target: UnitInstanceId,
        now: u64,
    ) -> Result<(), GameError> {
        if let Some(unit) = self
            .units
            .get_mut(&target)
            .filter(|u| !u.basic_attack.action.is_heal())
        {
            if let Some(end) = unit.basic_attack_windup_until_ms.take() {
                if unit.action_locks.basic_attack_until_ms == end {
                    unit.action_locks.basic_attack_until_ms = now;
                }
                if unit.action_locks.movement_until_ms == end {
                    unit.action_locks.movement_until_ms = now;
                }
            }
            self.runtime_facts.cancel_unreleased_basic(
                target,
                ActionCancelReason::Interrupted,
                now,
            )?;
        }
        if self.current_action_interrupt_protected(target) {
            return Ok(());
        }
        self.cancel_hostile_structured_flows(target, now, ActionCancelReason::Interrupted);
        let cancel = self
            .units
            .get(&target)
            .and_then(|u| u.pending_skill_cast.as_ref())
            .and_then(|p| self.pending_skill_view(p, target))
            .is_some_and(Self::hostile_attack_skill);
        if cancel {
            self.cancel_resonance_use_control(target);
            let unit = self.units.get_mut(&target).unwrap();
            let pending = unit.pending_skill_cast.take().unwrap();
            let lock = unit.next_action_time;
            unit.next_action_time = 0;
            unit.pending_cast =
                unit.skill_activation_mode == crate::game::ability::SkillActivationMode::Auto;
            unit.pending_cast_cause = None;
            for until in [
                &mut unit.action_locks.resonance_gain_until_ms,
                &mut unit.action_locks.basic_attack_until_ms,
                &mut unit.action_locks.movement_until_ms,
            ] {
                if *until == lock {
                    *until = now;
                }
            }
            self.cancel_pending_action(
                pending.action.as_ref(),
                now,
                ActionCancelReason::Interrupted,
            );
            self.record_event_log(
                now,
                BattleLogEvent::SkillCastInterrupted {
                    interrupter_instance_id: caster,
                    caster_instance_id: target,
                    interrupted_skill_id: pending.skill_id,
                    interrupted_cast_seq: pending.start_seq,
                },
            );
        }
        control::poll("approach_taunt_cancelled_attacks")
    }
}
